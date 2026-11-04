//! One session, two threads (`docs/design/2026-10-03-lane-off-the-brain-thread.md`). The fast thread owns the socket,
//! the recorder and the control lane (`crates/micro`): it answers every tick, and in a realtime game never waits for
//! the brain. The brain's thread owns the `Brain` and runs `decide` on every tick it is handed, folding the ticks that
//! piled up while it worked into one. In lockstep the fast thread waits for the brain's answer at every tick, so the
//! game is the one the single thread played: think, standing orders, declared groups, the lane, on the tick's own
//! snapshot, the world held while the brain and Jev work.

use std::collections::{HashMap, HashSet};
use std::io;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, mpsc};
use std::time::Instant;

use bot_protocol::{Command, Commands, Event, Terrain, Tick, ToBot, UnitDefId, UnitDefInfo, UnitId, Vec3, write_frame};
use micro::{GroupOrder, GroupState, HuntEvent, Lane, RoveEvent, RoveGoal, RoveReport, Stats, View};

use crate::brain::journal::{Intent, Journal};
use crate::brain::{BRAIN_FRAMES, Brain};
use crate::recorder::Recorder;

/// The lane's view of the brain, the part that never changes: built by the brain once the faction is adopted and
/// sent once.
pub(crate) struct LaneFixed {
    pub defs: HashMap<UnitDefId, UnitDefInfo>,
    pub stats: HashMap<UnitDefId, Stats>,
    pub terrain: Option<Terrain>,
    pub grid_spec: (f32, usize, usize),
    pub home: Vec3,
    /// Heuristic IDs switched off (`WITHIN_REASON_DISABLE`).
    pub disabled: Vec<String>,
    pub label: String,
    /// The types the lane commands and the host's orders to which are standing orders: armed, mobile, not a builder,
    /// not the commander, not aircraft.
    pub soldier_defs: HashSet<UnitDefId>,
    /// The combat simulator's rules and each type's row in its table, for the push's odds (H-MICRO-PUSH).
    pub sim: Option<(Arc<combatsim::sim::Rules>, HashMap<UnitDefId, usize>)>,
}

/// The lane's view of the brain, the part that changes: built by the brain at every think, from its trackers.
#[derive(Default)]
pub(crate) struct LaneScene {
    /// Remembered enemy buildings that stand finished (an unfinished one shoots nothing).
    pub remembered: Vec<(UnitId, UnitDefId, Vec3)>,
    /// Where every remembered enemy building is, finished or not.
    pub buildings: HashMap<UnitId, Vec3>,
    /// The enemy soldiers the brain has seen.
    pub known_soldiers: HashSet<UnitId>,
    /// What a radar contact is taken for: the soldier of theirs seen most.
    pub blip: Option<UnitDefId>,
    /// Seconds each enemy stays stunned.
    pub stunned: HashMap<UnitId, f32>,
    /// The rove goals for every type of ours that can rove.
    pub rove_goals: HashMap<UnitDefId, Vec<RoveGoal>>,
    /// Builders of ours that heal no rover now: the commander under fire or on an attack (H-HANDS-HEAL).
    pub no_heal: HashSet<UnitId>,
}

/// The two parts together: what `Lane::tick` reads. Nothing borrows the brain.
struct SceneView {
    fixed: Arc<LaneFixed>,
    scene: Arc<LaneScene>,
}

impl View for SceneView {
    fn def(&self, def: UnitDefId) -> Option<&UnitDefInfo> {
        self.fixed.defs.get(&def)
    }

    fn stats(&self, def: UnitDefId) -> Option<Stats> {
        self.fixed.stats.get(&def).copied()
    }

    fn grid_spec(&self) -> (f32, usize, usize) {
        self.fixed.grid_spec
    }

    fn terrain(&self) -> Option<&Terrain> {
        self.fixed.terrain.as_ref()
    }

    fn home(&self) -> Vec3 {
        self.fixed.home
    }

    fn remembered_buildings(&self) -> Vec<(UnitId, UnitDefId, Vec3)> {
        self.scene.remembered.clone()
    }

    /// Construction turrets heal under their standing order (`nanos.rs`), constructors and the commander under the
    /// hands' `Repair` (H-HANDS-HEAL, `brain/heal.rs`), except the commander while it is under fire or on an attack.
    fn heals(&self, builder: UnitId, _def: UnitDefId) -> bool {
        !self.scene.no_heal.contains(&builder)
    }

    /// The simulator's verdict on the units as they stand, eight seeds; `None` for a type the table lacks.
    fn odds(&self, ours: &[(UnitDefId, Vec3)], theirs: &[(UnitDefId, Vec3)]) -> Option<(f32, f32)> {
        let (rules, rows) = self.fixed.sim.as_ref()?;
        let placed = |side: &[(UnitDefId, Vec3)]| side.iter().map(|(def, at)| Some((*rows.get(def)?, combatsim::scenario::Vec2::new(at.x, at.z)))).collect::<Option<Vec<_>>>();
        Some(combatsim::scenario::odds_placed(rules, &placed(ours)?, &placed(theirs)?, 8))
    }

    fn building_at(&self, id: UnitId) -> Option<Vec3> {
        self.scene.buildings.get(&id).copied()
    }

    fn enemy_known(&self, id: UnitId) -> bool {
        self.scene.known_soldiers.contains(&id)
    }

    fn blip_def(&self) -> Option<UnitDefId> {
        self.scene.blip
    }

    fn enabled(&self, rule: &str) -> bool {
        !self.fixed.disabled.iter().any(|id| id == rule)
    }

    fn mine(&self, _unit: UnitId) -> bool {
        true
    }

    fn label(&self) -> String {
        self.fixed.label.clone()
    }

    fn stunned(&self, unit: UnitId) -> f32 {
        self.scene.stunned.get(&unit).copied().unwrap_or(0.0)
    }

    fn rove_goals(&self, def: UnitDefId) -> Vec<RoveGoal> {
        self.scene.rove_goals.get(&def).cloned().unwrap_or_default()
    }
}

/// A group declared to the lane: its lane name, its members, its order.
pub(crate) type Declared = (String, Vec<UnitId>, GroupOrder);

/// What the lane tells the brain of a tick, handed over with the next one.
#[derive(Default)]
pub(crate) struct LaneReport {
    /// Hunts that ended (H-MICRO-HUNT).
    pub hunts: Vec<HuntEvent>,
    /// What the rovers found, attacked, ran from and gave up (H-MICRO-ROVE).
    pub rove: Vec<RoveEvent>,
    /// Rule IDs that fired, once per firing, for the brain's per-minute line (the record counts them on the fast
    /// thread).
    pub fired: Vec<&'static str>,
    /// Every declared group's state, by lane name.
    pub groups: HashMap<String, GroupState>,
    /// Every rover's report.
    pub roving: HashMap<UnitId, RoveReport>,
}

/// What the brain answers a tick with.
pub(crate) struct BrainResult {
    pub frame: i32,
    pub commands: Vec<Command>,
    /// The groups declared to the lane, when the brain re-declared them this tick: those not in the list are cleared.
    pub declared: Option<Vec<Declared>>,
    /// The view's fixed part, with the first answer after the faction is adopted.
    pub fixed: Option<Arc<LaneFixed>>,
    /// The view's changing part, with every think's answer.
    pub scene: Option<Arc<LaneScene>>,
    pub journal: Journal,
    /// The brain's own wall time for this answer.
    pub decide_ms: f32,
}

/// The fast thread's loop: from the first tick to the end of the connection.
pub(crate) fn run(mut stream: UnixStream, mut next: impl FnMut() -> io::Result<ToBot>, brain: Brain, mut recorder: Option<Recorder>) -> io::Result<()> {
    let lockstep = crate::strategist::lockstep();
    let (to_brain, brain_in) = mpsc::channel::<(Arc<Tick>, LaneReport)>();
    let (from_brain, brain_out) = mpsc::channel::<BrainResult>();
    std::thread::Builder::new().name("brain".into()).spawn(move || brain_thread(brain, brain_in, from_brain))?;
    let debug = std::env::var_os("WITHIN_REASON_MICRO_DEBUG").is_some();
    let mut lane = Lane::default();
    let (mut fixed, mut scene): (Option<Arc<LaneFixed>>, Option<Arc<LaneScene>>) = (None, None);
    let mut report = LaneReport::default();
    let mut intent = Intent::default();
    let (mut last_frame, mut last_own) = (0, 0);
    loop {
        let tick = match next() {
            Ok(ToBot::Tick(tick)) => Arc::new(tick),
            Ok(_) => return Err(io::Error::other("unexpected second Hello")),
            Err(e) => {
                // The engine closed the connection: the game is over as far as this seat can know.
                if let Some(r) = &mut recorder {
                    r.ended(last_frame, last_own, &e.to_string());
                }
                return Err(e);
            }
        };
        (last_frame, last_own) = (tick.frame, tick.snapshot.own_units.len());
        let started = Instant::now();
        if to_brain.send((tick.clone(), std::mem::take(&mut report))).is_err() {
            return Err(io::Error::other("the brain's thread has ended"));
        }
        // Lockstep: this tick's answer, nothing else. Realtime: whatever answers have come, this tick's or none.
        let mut answers = Vec::new();
        if lockstep {
            match brain_out.recv() {
                Ok(answer) => answers.push(answer),
                Err(_) => return Err(io::Error::other("the brain's thread has ended")),
            }
        }
        while let Ok(answer) = brain_out.try_recv() {
            answers.push(answer);
        }
        let mut commands = Vec::new();
        let mut journal = Journal { intent, ..Journal::default() };
        let mut brain_ms = 0.0f32;
        for answer in answers {
            if let Some(f) = answer.fixed {
                fixed = Some(f);
            }
            if let Some(s) = answer.scene {
                scene = Some(s);
            }
            // The brain's orders to soldiers become their standing orders, before the lane runs over them.
            if let Some(f) = &fixed {
                let own: HashMap<UnitId, UnitDefId> = tick.snapshot.own_units.iter().map(|u| (u.id, u.def)).collect();
                lane.note_standing_orders(&answer.commands, answer.frame, |unit| own.get(&unit).is_some_and(|def| f.soldier_defs.contains(def)));
            }
            if let Some(declared) = answer.declared {
                for name in lane.group_names() {
                    if !declared.iter().any(|(n, ..)| *n == name) {
                        lane.clear_group(&name);
                    }
                }
                for (name, members, order) in declared {
                    lane.set_group(&name, members, order);
                }
            }
            commands.extend(answer.commands);
            for (rule, n) in answer.journal.rules {
                *journal.rules.entry(rule).or_default() += n;
            }
            journal.notes.extend(answer.journal.notes);
            journal.intent = answer.journal.intent;
            intent = answer.journal.intent;
            brain_ms = brain_ms.max(answer.decide_ms);
        }
        // The lane, once the brain has told it what the game is made of (as before: nothing until the faction is
        // adopted).
        if let (Some(f), Some(s)) = (&fixed, &scene) {
            let view = SceneView { fixed: f.clone(), scene: s.clone() };
            let output = lane.tick(&view, &tick, &mut commands, debug);
            for rule in &output.fired {
                journal.rule(rule);
            }
            journal.milling += output.milling;
            let groups: HashMap<String, GroupState> = lane.group_names().into_iter().filter_map(|name| lane.group_state(&name).cloned().map(|state| (name, state))).collect();
            journal.groups = groups.iter().map(|(name, state)| (name.clone(), state.clone())).collect();
            commands.extend(output.commands);
            report = LaneReport { hunts: output.hunts, rove: output.rove, fired: output.fired, groups, roving: lane.rovers_roving() };
        }
        let ms = started.elapsed().as_secs_f32() * 1000.0;
        if let Some(r) = &mut recorder
            && let Err(e) = r.tick(&tick, journal, &commands, ms, brain_ms)
        {
            // A full disk must not lose the match.
            eprintln!("match record abandoned: {e}");
            recorder = None;
        }
        write_frame(&mut stream, &Commands(commands))?;
    }
}

/// The brain's thread: `decide` on every tick handed over, until the fast thread is gone. Ticks that piled up while
/// the brain worked (realtime, the brain slower than the tick) are folded into one: the newest snapshot, every event
/// in order, and `late` set so that `Tick::due` is the think frame among them when there was one, so the brain's
/// schedule sees the think it owes and not a missed one. In lockstep nothing piles up.
fn brain_thread(mut brain: Brain, from_fast: mpsc::Receiver<(Arc<Tick>, LaneReport)>, to_fast: mpsc::Sender<BrainResult>) {
    while let Ok((first, report)) = from_fast.recv() {
        let mut reports = vec![report];
        let mut newest = first;
        let mut events: Vec<Event> = Vec::new();
        let mut think_due = (newest.due() % BRAIN_FRAMES == 0).then(|| newest.due());
        let mut folded = false;
        while let Ok((later, report)) = from_fast.try_recv() {
            folded = true;
            events.extend(newest.events.iter().cloned());
            if later.due() % BRAIN_FRAMES == 0 {
                think_due = Some(later.due());
            }
            newest = later;
            reports.push(report);
        }
        let started = Instant::now();
        let mut answer = if folded {
            events.extend(newest.events.iter().cloned());
            let late = think_due.map_or(newest.late, |due| newest.frame - due);
            let tick = Tick { frame: newest.frame, late, events, snapshot: newest.snapshot.clone() };
            brain.decide(&tick, reports)
        } else {
            brain.decide(&newest, reports)
        };
        answer.decide_ms = started.elapsed().as_secs_f32() * 1000.0;
        if to_fast.send(answer).is_err() {
            return;
        }
    }
}
