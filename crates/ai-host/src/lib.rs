//! The engine's side of every bot: what a skirmish AI library for the Recoil engine's C AI interface does whatever
//! plays the seat. A library the engine loads as `libSkirmishAI.so` is this host over one [`Transport`], its three
//! exports (`SSkirmishAILibrary.h`) written by [`export!`]. The host holds no strategy: it sends events and state
//! snapshots through the transport and applies the commands it gets back. The transport is where the bot is: in
//! its own process behind a socket (`crates/ai-shim`, the flagship's), or inside the library ([`InProcess`], a
//! [`Bot`] called on the engine thread: a bot shipped as one file). See `DESIGN.md`.

mod engine;
mod script;

use std::collections::BTreeMap;
use std::ffi::{c_int, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;
use std::time::Instant;

use bot_protocol::{Bot, Command, Commands, Event, Tick, ToBot, UnitDefId, UnitId, Vec3};
pub use recoil_ai_sys as sys;

use engine::Engine;

/// How the host reaches the bot. The exchange is the protocol's (`bot_protocol`): a `Hello`, then one `Commands`
/// per message sent, each answer the credit for the next tick.
pub trait Transport: Send + Sized {
    /// Tried at `init` and then about once a second until it succeeds; an error is the idle state, not a fault.
    fn connect() -> std::io::Result<Self>;
    fn send(&mut self, message: ToBot) -> std::io::Result<()>;
    /// The bot's reply if one has fully arrived. Never blocks.
    fn poll(&mut self) -> std::io::Result<Option<Commands>>;
    /// Blocks until the bot's reply has arrived. Lockstep mode: the game stands still while the bot thinks.
    fn wait(&mut self) -> std::io::Result<Commands>;
}

/// The git commit this library was built from (`build.rs`; a `+` when the tree was dirty), for the line a bot says
/// in the game chat so a replay or an engine log says which build played.
pub fn commit() -> &'static str {
    env!("WITHIN_REASON_COMMIT")
}

/// The bot inside the library, called on the engine thread: its answer to a message is there when `send` returns,
/// so the game waits for every tick of it, lockstep or not. For bots that answer well inside a frame.
pub struct InProcess<B: Bot> {
    bot: Option<B>,
    reply: Option<Commands>,
}

impl<B: Bot> Transport for InProcess<B> {
    fn connect() -> std::io::Result<Self> {
        Ok(InProcess { bot: None, reply: None })
    }

    fn send(&mut self, message: ToBot) -> std::io::Result<()> {
        self.reply = Some(match (message, &mut self.bot) {
            (ToBot::Hello(hello), bot) => {
                *bot = Some(B::start(hello));
                Commands::default()
            }
            (ToBot::Tick(tick), Some(bot)) => bot.tick(tick),
            (ToBot::Tick(_), None) => return Err(std::io::Error::other("a tick before the hello")),
        });
        Ok(())
    }

    fn poll(&mut self) -> std::io::Result<Option<Commands>> {
        Ok(self.reply.take())
    }

    fn wait(&mut self) -> std::io::Result<Commands> {
        self.reply.take().ok_or_else(|| std::io::Error::other("no message is waiting for an answer"))
    }
}

/// Frames between ticks sent to the bot (the sim runs 30 per second): 10 Hz, the control lane's rate
/// (`docs/design/2026-09-20-micro-lane.md`). `WITHIN_REASON_TICK_FRAMES` overrides it with another divisor of the
/// brain's own interval (15 frames).
const TICK_FRAMES: i32 = 3;
const BRAIN_FRAMES: i32 = 15;
const RECONNECT_INTERVAL: i32 = 30;
/// Frames between heartbeat lines in the engine log; the arena reads game time from them.
const HEARTBEAT_INTERVAL: i32 = 30 * 30;
/// Frames between census lines (`WITHIN_REASON_OBSERVE`): what the opponent owns, for studying how it plays.
const CENSUS_INTERVAL: i32 = 60 * 30;
/// Frames between ground-truth samples of the opponent's units and of both sides' metal income (`WITHIN_REASON_OBSERVE`
/// with `WITHIN_REASON_TRUTH_DIR`).
const TRUTH_INTERVAL: i32 = 2 * 30;
/// Frames between looks at the map's features (wrecks); a multiple of the brain's interval.
const WRECKS_INTERVAL: i32 = 3 * 30;
/// Frames between reads of our side's sensor coverage grids (sight, radar, air sight): once a second, a multiple of the
/// brain's interval, so every brain think within the second reads the same grids.
const SENSORS_INTERVAL: i32 = 30;
/// What our allies' overflow put into our stock is read this often: the engine's own second, the team's slow update.
const RECEIVED_INTERVAL: i32 = 30;

struct Instance<L: Transport> {
    ai_id: c_int,
    engine: Engine,
    link: Option<L>,
    /// The bot has answered the previous message, so it may be sent another.
    has_credit: bool,
    tick_frames: i32,
    /// The frame a tick fell due at and has not been sent, for want of credit: sent as soon as the answer comes,
    /// marked late by the frames it waited.
    tick_due: Option<i32>,
    lockstep: bool,
    /// Where the opponent's true state is written for post-game analysis; the bot never sees it.
    truth: Option<std::io::BufWriter<std::fs::File>>,
    events: Vec<Event>,
    /// Units to create once the export has let go of the instance table (see `engine::Spawner`).
    spawns: Vec<(UnitDefId, Vec3)>,
    /// The frame `/cheat 1` last went out, which `/give` needs (`engine::Spawner::give`). It goes out with the first
    /// give and none before: no bot of a game with people gives units, and there the server refuses the command and
    /// tells every player so, or on a client hosting the game turns cheating on (the hosts' first update sent it from
    /// 2026-10-24 to 2026-10-27, `docs/design/2026-10-27-engine-thread.md`). Spawns are held until `CHEAT_LAG` frames
    /// after a sending, when the flag has come back from the server; the command goes out again `CHEAT_RESEND` frames
    /// after it, since the game has swallowed one (2026-10-24 night, two engines starting together: no "Cheating is
    /// enabled!" in the infolog, every give refused, `spawn_failed`), and again before a give `CHEAT_REFRESH` frames
    /// after the last sending (the director's retried spawn).
    cheats_sent_at: Option<i32>,
    frame: i32,
    /// The engine thread's time on this instance since the last `timing` line.
    spent: Spent,
}

/// Wall seconds the engine thread spent on one instance (`docs/design/2026-10-27-engine-thread.md`): an in-library
/// bot's tick runs on it and the game waits, and a client hosting AIs for people falls behind the server by that much.
#[derive(Default)]
struct Spent {
    /// Reading the game for a tick: the snapshot, and the wrecks and sensors when due.
    reads: f64,
    /// The bot's answer: the in-library bot's ticks, the shim's socket writes (the start is said once, at the hello).
    bot: f64,
    /// Carrying out the bot's commands, the build-site searches among them.
    apply: f64,
}

/// A `timing` line names any single read, tick or apply that held the engine thread this long, in seconds.
const SLOW_CALL: f64 = 0.05;
/// Frames between `timing` lines: once a game minute.
const TIMING_INTERVAL: i32 = 60 * 30;

/// Frames from `/cheat 1` to the first `/give`: the command comes back from the server within a frame in a local game
/// when it comes back at all; half a second covers the second sending too.
const CHEAT_LAG: i32 = 15;
/// `/cheat 1` again this many frames after the first, in case the game dropped the first (setting the flag twice is
/// nothing).
const CHEAT_RESEND: i32 = 8;
/// A give arriving this long after the last `/cheat 1` sends the command again first (a retried spawn: the game had
/// swallowed both earlier sendings, 2026-10-24 night, four of nine matches).
const CHEAT_REFRESH: i32 = 2 * 30;

// SAFETY: the engine only calls the exports from its own thread; the raw callback pointer
// inside `Engine` never leaves that thread. The mutex only guards the instance table.
unsafe impl<L: Transport> Send for Instance<L> {}

impl<L: Transport> Instance<L> {
    fn log(&self, text: impl std::fmt::Display) {
        eprintln!("[wreason ai={}] {text}", self.ai_id);
    }

    /// The time a call held the engine thread, added to its account; said when it held it `SLOW_CALL` or more.
    fn spent(&mut self, began: Instant, what: &str, account: fn(&mut Spent) -> &mut f64) {
        let seconds = began.elapsed().as_secs_f64();
        *account(&mut self.spent) += seconds;
        if seconds >= SLOW_CALL {
            self.log(format_args!("timing f={} slow {what} {:.0} ms", self.frame, seconds * 1000.0));
        }
    }

    fn cheat(&mut self) {
        if let Err(code) = self.engine.console("/cheat 1") {
            self.log(format!("/cheat 1 refused: {code}"));
        }
    }

    fn update(&mut self, frame: i32) {
        self.frame = frame;
        if self.cheats_sent_at.is_some_and(|sent| frame == sent + CHEAT_RESEND) {
            self.cheat();
        }
        if frame % HEARTBEAT_INTERVAL == 0 {
            self.log(format_args!("heartbeat f={frame}"));
            // For the arena's referee, which may call a settled game (`WITHIN_REASON_BALANCE`).
            if std::env::var_os("WITHIN_REASON_BALANCE").is_some() {
                let ((army, extractors), (their_army, their_extractors)) = self.engine.balance();
                self.log(format_args!("balance f={frame} ours={army:.0}/{extractors} theirs={their_army:.0}/{their_extractors}"));
            }
        }
        if frame % TRUTH_INTERVAL == 0 && let Some(truth) = &mut self.truth {
            use std::io::Write;
            let units = self.engine.enemy_truth();
            let (ours, theirs) = self.engine.metal_incomes();
            let _ = writeln!(truth, "{{\"f\":{frame},\"enemy\":{units},\"income\":[{ours:.2},{theirs:.2}]}}").and_then(|()| truth.flush());
        }
        if frame % RECEIVED_INTERVAL == 0 {
            let (metal, energy) = self.engine.received();
            if metal > 0.0 || energy > 0.0 {
                self.events.push(Event::Received { metal, energy });
            }
        }
        if frame % CENSUS_INTERVAL == 0 && std::env::var_os("WITHIN_REASON_OBSERVE").is_some() {
            let census = self.engine.enemy_census();
            self.log(format_args!("census f={frame} enemy {census}"));
            let census = self.engine.own_census();
            self.log(format_args!("census f={frame} own {census}"));
        }
        if self.link.is_none() && frame % RECONNECT_INTERVAL == 0 {
            self.connect(frame);
        }
        if frame % TIMING_INTERVAL == 0 && frame > 0 {
            let Spent { reads, bot, apply } = std::mem::take(&mut self.spent);
            self.log(format_args!("timing f={frame} minute: reads {reads:.3} s, bot {bot:.3} s, apply {apply:.3} s"));
        }
        let Some(link) = &mut self.link else { return };
        match link.poll() {
            Ok(Some(commands)) => {
                self.has_credit = true;
                let began = Instant::now();
                self.apply(commands);
                self.spent(began, "apply", |s| &mut s.apply);
            }
            Ok(None) => {}
            Err(e) => return self.disconnect(e),
        }
        if frame % self.tick_frames == 0 && self.tick_due.is_none() {
            self.tick_due = Some(frame);
        }
        if self.has_credit && let Some(due) = self.tick_due.take() {
            let began = Instant::now();
            let mut snapshot = self.engine.snapshot();
            if due % WRECKS_INTERVAL == 0 {
                snapshot.wrecks = Some(self.engine.wrecks());
            }
            if due % SENSORS_INTERVAL == 0 {
                snapshot.sensors = self.engine.sensors();
            }
            self.spent(began, "reads", |s| &mut s.reads);
            let tick = Tick { frame, late: frame - due, events: std::mem::take(&mut self.events), snapshot };
            let began = Instant::now();
            self.send(ToBot::Tick(tick));
            self.spent(began, "tick", |s| &mut s.bot);
            // Lockstep (`WITHIN_REASON_LOCKSTEP`, headless study runs only): the engine waits here for the answer,
            // so a bot that holds its reply while a language model thinks has in effect paused the game.
            if self.lockstep && let Some(link) = &mut self.link {
                match link.wait() {
                    Ok(commands) => {
                        self.has_credit = true;
                        self.apply(commands);
                    }
                    Err(e) => self.disconnect(e),
                }
            }
        }
    }

    fn connect(&mut self, frame: i32) {
        // A missing bot is the normal idle state, not worth a log line every second.
        let Ok(link) = L::connect() else { return };
        self.link = Some(link);
        self.log(format_args!("connected to bot at frame {frame}"));
        let began = Instant::now();
        let hello = self.engine.hello(frame, self.tick_frames);
        let reads = began.elapsed();
        let began = Instant::now();
        self.send(ToBot::Hello(hello));
        self.log(format_args!("timing f={frame} hello: reads {} ms, bot start {} ms", reads.as_millis(), began.elapsed().as_millis()));
    }

    fn send(&mut self, message: ToBot) {
        self.has_credit = false;
        let Some(link) = &mut self.link else { return };
        if let Err(e) = link.send(message) {
            self.disconnect(e);
        }
    }

    fn disconnect(&mut self, error: std::io::Error) {
        self.log(format_args!("lost bot connection: {error}"));
        self.link = None;
        self.has_credit = false;
        self.tick_due = None;
    }

    fn apply(&mut self, commands: Commands) {
        for mut command in commands.0 {
            if let Command::Build { unit, def, site: Some(site), .. } = &mut command {
                match self.engine.find_build_site(*def, site) {
                    Some((pos, placement)) => {
                        if std::env::var_os("WITHIN_REASON_TRACE_BUILDS").is_some() {
                            self.log(format_args!(
                                "build unit={} def={} wanted=({:.0},{:.0}) placed=({:.0},{:.0}) placement={:?} of {}",
                                unit.0, def.0, site.near.x, site.near.z, pos.x, pos.z, placement.map(|i| i + 1), site.placements.len()
                            ));
                        }
                        site.near = pos;
                        self.events.push(Event::BuildSited { unit: *unit, def: *def, at: pos });
                        // The placement the site was found for is the one to build with.
                        if let Some(i) = placement {
                            site.placements = vec![site.placements[i].clone()];
                        }
                    }
                    None => {
                        if std::env::var_os("WITHIN_REASON_TRACE_BUILDS").is_some() {
                            let what = self.engine.describe_site(*def, site.near, 64.0);
                            self.log(format_args!(
                                "no site unit={} def={} wanted=({:.0},{:.0}) {what}", unit.0, def.0, site.near.x, site.near.z
                            ));
                        }
                        self.events.push(Event::BuildSiteNotFound { unit: *unit, def: *def });
                        continue;
                    }
                }
            }
            if let Command::Probe { points } = &command {
                let points = points.iter().map(|p| Vec3 { y: self.engine.elevation(p.x, p.z), ..*p }).collect();
                self.events.push(Event::Ground { points });
                continue;
            }
            if let Command::GiveUnit { def, at } = command {
                // The first give, or one long after the cheat command: `/cheat 1` first, and the give waits
                // `CHEAT_LAG` for it (the game swallows the command now and then, both sendings; the director gives an
                // army of which nothing appeared again three seconds on, and that give needs a fresh flag).
                if self.cheats_sent_at.is_none_or(|sent| self.frame - sent > CHEAT_REFRESH) {
                    self.cheat();
                    self.cheats_sent_at = Some(self.frame);
                }
                self.spawns.push((def, at));
                continue;
            }
            if let Err(code) = self.engine.issue(&command) {
                let (Command::Build { unit, .. }
                | Command::Move { unit, .. }
                | Command::Fight { unit, .. }
                | Command::Attack { unit, .. }
                | Command::Stop { unit }
                | Command::SetRepeat { unit, .. }
                | Command::MoveState { unit, .. }
                | Command::FireState { unit, .. }
                | Command::Guard { unit, .. }
                | Command::ReclaimFeature { unit, .. }
                | Command::ReclaimUnit { unit, .. }
                | Command::Resurrect { unit, .. }
                | Command::Repair { unit, .. }
                | Command::SelfDestruct { unit }) = command
                else {
                    // No unit to report it against: say so here (a refused resource cheat was silent until 2026-10-24).
                    self.log(format!("{command:?} refused: {code}"));
                    continue;
                };
                self.events.push(Event::CommandRejected { unit, code });
            }
        }
    }

    /// # Safety
    /// `data` must point to the event struct matching `topic`.
    unsafe fn handle_event(&mut self, topic: sys::EventTopic, data: *const c_void) {
        /// The engine passes -1 for "no such unit".
        fn unit(id: c_int) -> Option<UnitId> {
            (id >= 0).then_some(UnitId(id))
        }
        macro_rules! event {
            ($ty:ident) => {
                unsafe { &*data.cast::<sys::$ty>() }
            };
        }
        let event = match topic {
            sys::EVENT_UPDATE => return self.update(event!(SUpdateEvent).frame),
            sys::EVENT_UNIT_CREATED => {
                let e = event!(SUnitCreatedEvent);
                Event::UnitCreated { unit: UnitId(e.unit), builder: unit(e.builder) }
            }
            sys::EVENT_UNIT_FINISHED => Event::UnitFinished { unit: UnitId(event!(SUnitFinishedEvent).unit) },
            sys::EVENT_UNIT_IDLE => Event::UnitIdle { unit: UnitId(event!(SUnitIdleEvent).unit) },
            sys::EVENT_UNIT_MOVE_FAILED => {
                Event::UnitMoveFailed { unit: UnitId(event!(SUnitMoveFailedEvent).unit) }
            }
            sys::EVENT_UNIT_DAMAGED => {
                let e = event!(SUnitDamagedEvent);
                // The direction is the zero vector when nothing attacked (a collision); the engine fills it in for an
                // attacker out of our sight as well as one in it.
                let from = (!e.dir_posF3.is_null())
                    .then(|| unsafe { std::slice::from_raw_parts(e.dir_posF3, 3) })
                    .map(|d| Vec3 { x: d[0], y: d[1], z: d[2] })
                    .filter(|d| d.x.hypot(d.z) > 0.01 || d.y.abs() > 0.01);
                Event::UnitDamaged { unit: UnitId(e.unit), attacker: unit(e.attacker), damage: e.damage, from, weapon: self.engine.weapon(e.weaponDefId) }
            }
            sys::EVENT_UNIT_DESTROYED => {
                let e = event!(SUnitDestroyedEvent);
                Event::UnitDestroyed { unit: UnitId(e.unit), attacker: unit(e.attacker), death: self.engine.death(e.weaponDefID) }
            }
            sys::EVENT_ENEMY_ENTER_LOS => Event::EnemyEnterLos { enemy: UnitId(event!(SEnemyEnterLOSEvent).enemy) },
            sys::EVENT_ENEMY_LEAVE_LOS => Event::EnemyLeaveLos { enemy: UnitId(event!(SEnemyLeaveLOSEvent).enemy) },
            sys::EVENT_MESSAGE => {
                let e = event!(SMessageEvent);
                let text = if e.message.is_null() { String::new() } else { unsafe { std::ffi::CStr::from_ptr(e.message) }.to_string_lossy().into_owned() };
                if text.trim().is_empty() {
                    return;
                }
                Event::Chat { player: e.player, text }
            }
            sys::EVENT_ENEMY_DESTROYED => {
                // The engine's `attacker` is an allied unit's id, or -1 for none or a killer not allied with us.
                let e = event!(SEnemyDestroyedEvent);
                Event::EnemyDestroyed { enemy: UnitId(e.enemy), attacker: unit(e.attacker) }
            }
            // The engine tells the new owner (and its allies not allied with the old) `given`, the old owner `captured`.
            sys::EVENT_UNIT_GIVEN => {
                let e = event!(SUnitGivenEvent);
                Event::UnitGiven { unit: UnitId(e.unitId), from_team: e.oldTeamId, to_team: e.newTeamId }
            }
            sys::EVENT_UNIT_CAPTURED => {
                let e = event!(SUnitCapturedEvent);
                Event::UnitTaken { unit: UnitId(e.unitId), from_team: e.oldTeamId, to_team: e.newTeamId }
            }
            sys::EVENT_ENEMY_DAMAGED => {
                let e = event!(SEnemyDamagedEvent);
                Event::EnemyDamaged { enemy: UnitId(e.enemy), attacker: unit(e.attacker), damage: e.damage, weapon: self.engine.weapon(e.weaponDefId) }
            }
            sys::EVENT_WEAPON_FIRED => {
                let e = event!(SWeaponFiredEvent);
                Event::WeaponFired { unit: UnitId(e.unitId), weapon: self.engine.weapon(e.weaponDefId) }
            }
            _ => return,
        };
        self.events.push(event);
    }
}

/// `WITHIN_REASON_TICK_FRAMES` when it divides the brain's interval, else the default.
fn tick_frames() -> i32 {
    let wanted = std::env::var("WITHIN_REASON_TICK_FRAMES").ok().and_then(|v| v.parse::<i32>().ok());
    match wanted {
        Some(n) if n >= 1 && BRAIN_FRAMES % n == 0 => n,
        Some(n) => {
            eprintln!("[wreason] WITHIN_REASON_TICK_FRAMES={n} does not divide {BRAIN_FRAMES}; using {TICK_FRAMES}");
            TICK_FRAMES
        }
        None => TICK_FRAMES,
    }
}

/// Runs `body` so that a panic cannot unwind into the engine.
fn guarded(ai_id: c_int, body: impl FnOnce() -> c_int) -> c_int {
    catch_unwind(AssertUnwindSafe(body)).unwrap_or_else(|_| {
        eprintln!("[wreason ai={ai_id}] panic in AI library");
        -1
    })
}

/// A library's AI instances, one entry each; the engine may run several from one library. A library holds one
/// `Host` in a static and its exports call it ([`export!`]).
pub struct Host<L: Transport> {
    instances: Mutex<BTreeMap<c_int, Instance<L>>>,
}

impl<L: Transport> Default for Host<L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: Transport> Host<L> {
    pub const fn new() -> Self {
        Host { instances: Mutex::new(BTreeMap::new()) }
    }

    /// A panic in one export must not take the other instances down with it, so poisoning is ignored.
    fn instances(&self) -> std::sync::MutexGuard<'_, BTreeMap<c_int, Instance<L>>> {
        self.instances.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// # Safety
    /// `callback` must be the engine's callback table for `skirmish_ai_id`, valid until `release`.
    pub unsafe fn init(&self, skirmish_ai_id: c_int, callback: *const sys::SSkirmishAICallback) -> c_int {
        guarded(skirmish_ai_id, || {
            let mut instance = Instance {
                ai_id: skirmish_ai_id,
                engine: unsafe { Engine::new(skirmish_ai_id, callback) },
                link: None,
                has_credit: false,
                tick_frames: tick_frames(),
                tick_due: None,
                lockstep: std::env::var_os("WITHIN_REASON_LOCKSTEP").is_some(),
                truth: std::env::var_os("WITHIN_REASON_OBSERVE")
                    .and(std::env::var_os("WITHIN_REASON_TRUTH_DIR"))
                    .and_then(|dir| std::fs::File::create(std::path::Path::new(&dir).join(format!("truth-{skirmish_ai_id}.jsonl"))).ok())
                    .map(std::io::BufWriter::new),
                events: Vec::new(),
                spawns: Vec::new(),
                cheats_sent_at: None,
                frame: -1,
                spent: Spent::default(),
            };
            instance.log("init");
            // Hello at init, before the people place their starts and long before the first update: the player takes
            // its opening turn while the game has not begun (`docs/design/2026-09-30-opening-turn.md`). With no bot
            // listening yet the first update connects, as before.
            instance.connect(0);
            self.instances().insert(skirmish_ai_id, instance);
            0
        })
    }

    pub fn release(&self, skirmish_ai_id: c_int) -> c_int {
        guarded(skirmish_ai_id, || {
            if let Some(instance) = self.instances().remove(&skirmish_ai_id) {
                instance.log("release");
            }
            0
        })
    }

    /// # Safety
    /// `data` must point to the event struct matching `topic`, as the engine guarantees.
    pub unsafe fn handle_event(&self, skirmish_ai_id: c_int, topic: c_int, data: *const c_void) -> c_int {
        guarded(skirmish_ai_id, || {
            let (spawner, spawns) = {
                let mut instances = self.instances();
                let Some(instance) = instances.get_mut(&skirmish_ai_id) else { return -1 };
                if !data.is_null() {
                    unsafe { instance.handle_event(topic as sys::EventTopic, data) };
                }
                // Spawns wait for the cheat flag (`/cheat 1` from the first update) to have come back from the server.
                let ready = instance.cheats_sent_at.is_some_and(|f| instance.frame >= f + CHEAT_LAG);
                let spawns = if ready { std::mem::take(&mut instance.spawns) } else { Vec::new() };
                (instance.engine.spawner(), spawns)
            };
            // The table is unlocked here, as it was when the cheat interface re-entered this function with the
            // new unit's events; the synced `/give` brings the unit a frame or two on instead.
            for (def, at) in spawns {
                if let Err(code) = spawner.give(def, at) {
                    eprintln!("[wreason ai={skirmish_ai_id}] give unit def={} at ({:.0},{:.0}) refused: {code}", def.0, at.x, at.z);
                }
            }
            0
        })
    }
}

/// The three exports the engine calls (`SSkirmishAILibrary.h`), over a [`Host`] of the given [`Transport`]. Once,
/// at the root of a `cdylib` crate.
#[macro_export]
macro_rules! export {
    ($transport:ty) => {
        static HOST: $crate::Host<$transport> = $crate::Host::new();

        /// # Safety
        /// `callback` must be the engine's callback table for `skirmish_ai_id`, valid until `release`.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn init(skirmish_ai_id: ::std::ffi::c_int, callback: *const $crate::sys::SSkirmishAICallback) -> ::std::ffi::c_int {
            unsafe { HOST.init(skirmish_ai_id, callback) }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn release(skirmish_ai_id: ::std::ffi::c_int) -> ::std::ffi::c_int {
            HOST.release(skirmish_ai_id)
        }

        /// # Safety
        /// `data` must point to the event struct matching `topic`, as the engine guarantees.
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn handleEvent(skirmish_ai_id: ::std::ffi::c_int, topic: ::std::ffi::c_int, data: *const ::std::ffi::c_void) -> ::std::ffi::c_int {
            unsafe { HOST.handle_event(skirmish_ai_id, topic, data) }
        }
    };
}
