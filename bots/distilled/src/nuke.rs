//! The nuclear silos (`README.md` D-NUKE to D-NUKE-LOOK; the design `docs/design/2026-10-27-distilled-nukes.md`): a
//! later-game way to spend a bank nothing else spends. A silo is started when the bank holds its metal, the energy
//! covers its draw and there is something to strike; each fires at a place of his chosen at random by worth, outside
//! his known and suspected anti-nuke cover. What became of a strike is read off the ground: a missile that lands digs
//! a crater players see, one intercepted bursts in the air (the user, 2026-10-27). A landing's ruins are forgotten; an
//! interception marks his cover, and an air scout goes to look for the anti-nuke behind it.

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};

use crate::air::ENERGY_A_METAL;
use crate::economy::Job;
use crate::tier2::BANK_FULL;
use crate::{Distilled, MINUTE, SECOND};

/// A silo is told to fire at most once in this long: about 9 s from the order to the launch, then its 30 s reload.
const FIRE_GAP: i32 = 40 * SECOND;
/// A place struck with no verdict yet, or none to be had, is not struck again within the blast for this long.
const STRIKE_REST: i32 = 5 * MINUTE;
/// The least worth (metal plus energy over `ENERGY_A_METAL`) a strike must be expected to kill.
const STRIKE_LEAST: f32 = 500.0;
/// Kept beyond his known anti-nuke's coverage, and beyond the blast from any unit of ours or our allies' (a missile
/// lands about 21 s after the order, and a tank walks a thousand in that time).
const COVER_MARGIN: f32 = 200.0;
const FRIENDLY_MARGIN: f32 = 400.0;
/// With no launch seen this long after the order, the strike was not launched.
const LAUNCH_WAIT: i32 = 30 * SECOND;
/// The ground is read again this long after the launch: 12 s of flight, and the crater the engine digs over the
/// frames after the blast.
const SETTLE: i32 = 25 * SECOND;
/// A probe unanswered this long has no answer.
const PROBE_WAIT: i32 = 5 * SECOND;
/// The probe's points: the aim, and eight on each ring round it.
const PROBE_RINGS: [f32; 2] = [150.0, 300.0];
/// A strike landed when the ground at the probe's points fell by this much on the mean, in elmos. On Comet Catcher an
/// Armada missile that landed took it down 21 on the mean (52 at the aim, 37 at 150, 7 at 300) and one intercepted
/// not at all (nuke-crater-witness, nuke-crater-antinuke, 2026-10-27); this leaves room for harder ground.
const CRATER_DEPTH: f32 = 2.0;
/// Interceptions and landings are remembered this long: he builds anti-nukes, and loses them.
const COVER_MEMORY: i32 = 10 * MINUTE;
/// A place within this of a remembered landing, and not suspected, is this much likelier to be drawn: no loaded
/// anti-nuke covered the landing, and he rebuilds there.
const LANDED_NEAR: f32 = 1000.0;
const LANDED_FAVOUR: f32 = 2.0;
/// With no place worth striking this long, the silos stop stockpiling and no silo is started (D-NUKE-HOLD).
const HOLD_AFTER: i32 = MINUTE;
/// A silo's kept count is moved by this to pause it or bring it back: the game clamps it within 0 and the file's ten.
const KEPT_STEP: i32 = 10;
/// D-NUKE-LOOK: the grid over which his anti-nuke's place is narrowed, scouts lost before the place is in sight
/// before it is given up, and how often an idle scout is told again.
const LOOK_GRID: f32 = 250.0;
const LOOK_TRIES: u32 = 3;
const LOOK_RETELL: i32 = 10 * SECOND;
/// A scout flanking along the map's side flies this far in from the edge.
const FLANK_INSET: f32 = 250.0;

/// The silo's missile: the radius of its blast and the damage at the centre, falling to nothing at the edge (BAR's
/// warheads have an edge effectiveness of 0).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Warhead {
    pub radius: f32,
    pub damage: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Verdict {
    Pending,
    Landed,
    Intercepted,
    /// The ground told nothing: the lobby turned deformation off, or a probe went unanswered.
    Unknown,
    NotLaunched,
}

struct Strike {
    silo: UnitId,
    at: Vec3,
    ordered: i32,
    launched: Option<i32>,
    /// What the order expected: the worth and the number of his known buildings killed.
    worth: f32,
    buildings: usize,
    /// The ground at the probe's points before the launch, and when the second probe was asked.
    before: Option<Vec<f32>>,
    asked_after: Option<i32>,
    verdict: Verdict,
    /// Kills and damage the events named the silo for since the launch (only where we see), and the kills' worth.
    seen_killed: usize,
    seen_worth: f32,
    seen_damaged: usize,
}

struct Interception {
    at: Vec3,
    frame: i32,
    /// A scout has been to look, or given up.
    looked: bool,
}

/// D-NUKE-LOOK: the interceptions a look answers (by their frames), the points of the place not yet in our sight,
/// the scout, the point it was sent to and when, and how many scouts have been lost.
struct Look {
    cluster: Vec<i32>,
    points: Vec<Vec3>,
    scout: Option<UnitId>,
    to: Option<Vec3>,
    told: i32,
    tries: u32,
}

#[derive(Default)]
pub(crate) struct Nuke {
    pub warhead: Option<Warhead>,
    /// Each type's health, from the combat tables: what the warhead kills.
    health: BTreeMap<UnitDefId, f32>,
    /// An anti-nuke's coverage (2,000 in BAR): the widest any type has.
    coverage: f32,
    /// The lobby turned map deformation off: no crater tells a landing.
    pub flat: bool,
    /// Per silo, the frame of our last order to fire.
    ordered: BTreeMap<UnitId, i32>,
    strikes: Vec<Strike>,
    interceptions: Vec<Interception>,
    landings: Vec<(Vec3, i32)>,
    /// His mobile anti-nukes as last seen: where, their coverage, when (they leave sight and keep covering).
    mobile: BTreeMap<UnitId, (Vec3, f32, i32)>,
    /// A place worth striking was found at the last look; since when none has been; the silos paused for it.
    open: bool,
    closed_since: Option<i32>,
    paused: BTreeSet<UnitId>,
    look: Option<Look>,
    /// The frame until which no silo is started, after its site was refused (D-ORDERS).
    pub site_refused: i32,
    rng: u64,
}

/// A building of his as the warhead sees it.
struct Target {
    id: UnitId,
    pos: Vec3,
    health: f32,
    worth: f32,
    blast: Option<bot_protocol::Blast>,
}

/// A place worth striking: the centre, the worth killed, the buildings killed, and how likely it is to be drawn.
struct Place {
    at: Vec3,
    worth: f32,
    buildings: usize,
    weight: f32,
}

impl Distilled {
    /// The warhead of our faction's silo, every type's health (the combat tables), the anti-nukes' coverage, and
    /// whether the lobby keeps the ground flat.
    pub(crate) fn init_nuke(&mut self, table: &combatsim::units::Units) {
        self.nuke.health = self.defs.values().filter_map(|d| table.index(&d.name).map(|i| (d.id, table.list[i].health))).collect();
        let silo = self.kit.silo.and_then(|d| self.def(d)).and_then(|d| table.index(&d.name));
        self.nuke.warhead = silo.and_then(|i| table.list[i].weapons.iter().find(|w| w.stockpile && w.command_fire)).map(|w| Warhead { radius: w.aoe / 2.0, damage: w.damage.get("default").copied().unwrap_or(0.0) });
        self.nuke.coverage = self.defs.values().map(|d| d.coverage).fold(0.0, f32::max);
        self.nuke.flat = deformation_off(&self.hello.script);
        self.nuke.rng = 0x9e37_79b9_7f4a_7c15 ^ (self.hello.ai_id as u64).wrapping_mul(0x2545_f491_4f6c_dd1d);
    }

    /// D-NUKE: a silo is wanted when the bank holds its metal (or `BANK_FULL` of a store smaller than that), its site
    /// was not refused lately, there is a place worth striking, and the silos standing are not sitting on missiles.
    pub(crate) fn silo_wanted(&self, tick: &Tick) -> bool {
        let Some(bill) = self.kit.silo.and_then(|d| self.def(d)).map(|d| d.metal_cost) else { return false };
        let metal = tick.snapshot.metal;
        let piling = self.finished(self.kit.silo).next().is_some() && self.finished(self.kit.silo).all(|s| s.stockpile >= 2);
        self.options.nuke && self.frame >= self.nuke.site_refused && metal.current >= bill.min(BANK_FULL * metal.storage) && self.nuke.open && !piling
    }

    /// D-ENERGY's share for the silos: each silo's stockpile draw, frames counted and the paused left out, and one
    /// more while a silo is wanted (the energy comes before the silo it is for).
    pub(crate) fn silo_draw(&self, tick: &Tick) -> f32 {
        let Some(def) = self.kit.silo.and_then(|d| self.def(d)) else { return 0.0 };
        let drawing = self.own.values().filter(|u| Some(u.def) == self.kit.silo && !self.nuke.paused.contains(&u.id)).count();
        let silos = drawing + usize::from(self.silo_wanted(tick));
        def.stockpile_energy / def.stockpile_seconds.max(1.0) * silos as f32
    }

    /// D-NUKE: a builder that can build the silo starts one when one is wanted, no project is under way, and the
    /// energy, with this silo's draw counted, is not short.
    pub(crate) fn silo_job(&self, builder: &UnitDefInfo, tick: &Tick) -> Option<Job> {
        let silo = self.kit.silo?;
        let wanted = builder.build_options.contains(&silo) && self.silo_wanted(tick) && !self.project_under_way() && !self.energy_short(tick);
        wanted.then_some(Job::Silo)
    }

    /// Once a second: the strikes' verdicts, the memory of his cover, the hold (D-NUKE-HOLD), the strikes
    /// (D-NUKE-FIRE) and the scout (D-NUKE-LOOK).
    pub(crate) fn run_nukes(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let Some(warhead) = self.nuke.warhead else { return };
        self.remember_mobile_anti_nukes(tick);
        self.follow_strikes(commands);
        let frame = self.frame;
        self.nuke.interceptions.retain(|i| frame - i.frame < COVER_MEMORY);
        self.nuke.landings.retain(|(_, at)| frame - at < COVER_MEMORY);
        self.nuke.strikes.retain(|s| s.verdict == Verdict::Pending || frame - s.ordered < STRIKE_REST);
        let open = !self.places(tick, warhead).is_empty();
        self.hold(open, commands);
        let ready: Vec<UnitId> = self.finished(self.kit.silo).filter(|s| s.stockpile > 0 && self.nuke.ordered.get(&s.id).is_none_or(|at| frame - at >= FIRE_GAP)).map(|s| s.id).collect();
        for silo in ready {
            let places = self.places(tick, warhead);
            let Some(place) = self.draw(&places) else { break };
            let Place { at, worth, buildings, .. } = places[place];
            let cover = self.his_anti_nukes().iter().map(|(p, _)| p.dist2d(at)).min_by(f32::total_cmp);
            eprintln!(
                "[distilled ai={}] f={frame} nuke: silo {} fires at ({:.0},{:.0}) blast {:.0}: {buildings} of his buildings worth {worth:.0} expected, nearest known anti-nuke {}, the place in our {}; {} places worth striking",
                self.hello.ai_id,
                silo.0,
                at.x,
                at.z,
                warhead.radius,
                cover.map_or("none".to_string(), |d| format!("{d:.0} away")),
                sensed(tick, at),
                places.len()
            );
            commands.push(Command::FireAt { unit: silo, at });
            if !self.nuke.flat {
                commands.push(Command::Probe { points: probe_points(at) });
            }
            self.nuke.ordered.insert(silo, frame);
            self.nuke.strikes.push(Strike { silo, at, ordered: frame, launched: None, worth, buildings, before: None, asked_after: None, verdict: Verdict::Pending, seen_killed: 0, seen_worth: 0.0, seen_damaged: 0 });
        }
        self.run_look(tick, commands);
    }

    /// Each pending strike: not launched in time; the ground asked again once the warhead has landed; no answer, or
    /// no ground to ask (the lobby's flat map), and the verdict is unknown.
    fn follow_strikes(&mut self, commands: &mut Vec<Command>) {
        let frame = self.frame;
        let mut decided = Vec::new();
        for (i, s) in self.nuke.strikes.iter_mut().enumerate().filter(|(_, s)| s.verdict == Verdict::Pending) {
            match (s.launched, s.asked_after) {
                (None, _) if frame - s.ordered >= LAUNCH_WAIT => {
                    s.verdict = Verdict::NotLaunched;
                    decided.push(i);
                }
                (Some(launched), None) if frame - launched >= SETTLE => {
                    if self.nuke.flat || s.before.is_none() {
                        s.verdict = Verdict::Unknown;
                        decided.push(i);
                    } else {
                        s.asked_after = Some(frame);
                        commands.push(Command::Probe { points: probe_points(s.at) });
                    }
                }
                (Some(_), Some(asked)) if frame - asked >= PROBE_WAIT => {
                    s.verdict = Verdict::Unknown;
                    decided.push(i);
                }
                _ => {}
            }
        }
        for i in decided {
            self.report_strike(i, None);
        }
    }

    /// The shim's answer to a probe: the ground before a strike, or after it and the verdict.
    pub(crate) fn nuke_ground(&mut self, points: &[Vec3]) {
        let Some(aim) = points.first() else { return };
        let Some(i) = self.nuke.strikes.iter().rposition(|s| s.verdict == Verdict::Pending && s.at.x == aim.x && s.at.z == aim.z) else { return };
        let heights: Vec<f32> = points.iter().map(|p| p.y).collect();
        let strike = &mut self.nuke.strikes[i];
        let Some(before) = strike.before.as_ref() else {
            strike.before = Some(heights);
            return;
        };
        if strike.asked_after.is_none() || before.len() != heights.len() {
            return;
        }
        let drops: Vec<f32> = before.iter().zip(&heights).map(|(b, a)| b - a).collect();
        let mean = drops.iter().sum::<f32>() / drops.len().max(1) as f32;
        let landed = mean >= CRATER_DEPTH;
        strike.verdict = if landed { Verdict::Landed } else { Verdict::Intercepted };
        let (at, frame) = (strike.at, self.frame);
        if landed {
            self.nuke.landings.push((at, frame));
        } else {
            self.nuke.interceptions.push(Interception { at, frame, looked: false });
        }
        let max = drops.iter().copied().fold(f32::MIN, f32::max);
        self.report_strike(i, Some((mean, max)));
    }

    /// A strike decided: the log line, and a landing's ruins forgotten (what the warhead kills by the model leaves
    /// `his_buildings`, so neither the silos nor the army go back for it until it is seen again).
    fn report_strike(&mut self, i: usize, drop: Option<(f32, f32)>) {
        let Some(warhead) = self.nuke.warhead else { return };
        let s = &self.nuke.strikes[i];
        let (at, verdict) = (s.at, s.verdict);
        let mut ruins = 0;
        if verdict == Verdict::Landed {
            let targets = self.targets();
            let killed: Vec<UnitId> = blast_kills(at, warhead, &targets).into_iter().map(|k| targets[k].id).collect();
            ruins = killed.len();
            for id in killed {
                self.army.his_buildings.remove(&id);
            }
        }
        let s = &self.nuke.strikes[i];
        let ground = drop.map_or(String::new(), |(mean, max)| format!(" (the ground down {mean:.1} on the mean, {max:.1} at most)"));
        let after = match verdict {
            Verdict::Landed => format!("; {ruins} of his buildings presumed dead"),
            Verdict::Intercepted => "; an anti-nuke of his within its coverage, remembered".to_string(),
            _ => String::new(),
        };
        eprintln!(
            "[distilled ai={}] f={} nuke: strike at ({:.0},{:.0}) by silo {}: {}{ground}; expected {} worth {:.0}; the events saw killed {} worth {:.0}, damaged {}{after}",
            self.hello.ai_id,
            self.frame,
            at.x,
            at.z,
            s.silo.0,
            format!("{verdict:?}").to_ascii_lowercase(),
            s.buildings,
            s.worth,
            s.seen_killed,
            s.seen_worth,
            s.seen_damaged
        );
    }

    /// D-NUKE-HOLD: with no place worth striking for `HOLD_AFTER`, every silo's kept count goes to nothing, so it
    /// draws no energy (D-NUKE wants no silo meanwhile); a place found again brings them back to the game's ten.
    fn hold(&mut self, open: bool, commands: &mut Vec<Command>) {
        let frame = self.frame;
        self.nuke.open = open;
        let silos: Vec<UnitId> = self.finished(self.kit.silo).map(|s| s.id).collect();
        self.nuke.paused.retain(|id| silos.contains(id));
        if open {
            self.nuke.closed_since = None;
            if !self.nuke.paused.is_empty() {
                eprintln!("[distilled ai={}] f={frame} nuke: a place worth striking again; {} silos stockpile again", self.hello.ai_id, self.nuke.paused.len());
                commands.extend(self.nuke.paused.iter().map(|&unit| Command::Stockpile { unit, change: KEPT_STEP }));
                self.nuke.paused.clear();
            }
            return;
        }
        let since = *self.nuke.closed_since.get_or_insert(frame);
        if frame - since < HOLD_AFTER {
            return;
        }
        let pause: Vec<UnitId> = silos.into_iter().filter(|id| !self.nuke.paused.contains(id)).collect();
        if !pause.is_empty() {
            eprintln!("[distilled ai={}] f={frame} nuke: no place worth striking for {} s; {} silos stop stockpiling", self.hello.ai_id, (frame - since) / SECOND, pause.len());
            commands.extend(pause.iter().map(|&unit| Command::Stockpile { unit, change: -KEPT_STEP }));
            self.nuke.paused.extend(pause);
        }
    }

    /// His anti-nukes known: his buildings with an interceptor's coverage, and his mobile ones where last seen.
    fn his_anti_nukes(&self) -> Vec<(Vec3, f32)> {
        let coverage = |def: Option<UnitDefId>| def.and_then(|d| self.def(d)).map_or(0.0, |d| d.coverage);
        let buildings = self.army.his_buildings.values().map(|b| (b.pos, coverage(b.def)));
        let mobile = self.nuke.mobile.values().map(|(p, c, _)| (*p, *c));
        buildings.chain(mobile).filter(|(_, c)| *c > 0.0).collect()
    }

    /// His mobile anti-nukes in sight are remembered where they were last seen, `COVER_MEMORY` long (a BARb seat drives
    /// one round its base: dist54 game 01, the cormabm took strikes 2,000 apart while the bot knew of no anti-nuke).
    fn remember_mobile_anti_nukes(&mut self, tick: &Tick) {
        let frame = self.frame;
        for e in &tick.snapshot.enemies {
            if let Some(d) = e.def.and_then(|d| self.def(d)).filter(|d| d.speed > 0.0 && d.coverage > 0.0) {
                self.nuke.mobile.insert(e.id, (e.pos, d.coverage, frame));
            }
        }
        self.nuke.mobile.retain(|_, (_, _, seen)| frame - *seen < COVER_MEMORY);
    }

    /// His known buildings as the warhead sees them.
    fn targets(&self) -> Vec<Target> {
        self.army
            .his_buildings
            .iter()
            .filter_map(|(id, b)| {
                let def = self.def(b.def?)?;
                let health = self.nuke.health.get(&def.id).copied()?;
                Some(Target { id: *id, pos: b.pos, health, worth: def.metal_cost + def.energy_cost / ENERGY_A_METAL, blast: def.death_blast })
            })
            .collect()
    }

    /// D-NUKE-COVER: a place is suspected covered when it lies within an anti-nuke's coverage of a remembered
    /// interception (one of his stood within that of the aim, K-rules-interception-needs-the-aim-point-in-coverage),
    /// unless a later landing lies nearer to it than that aim.
    fn suspected(&self, at: Vec3) -> bool {
        self.nuke.interceptions.iter().any(|i| {
            let d = i.at.dist2d(at);
            d < self.nuke.coverage && !self.nuke.landings.iter().any(|(l, when)| *when > i.frame && l.dist2d(at) < d)
        })
    }

    /// The places worth striking: each of his known buildings as the blast's centre, with the worth the warhead would
    /// kill there; none under his known or suspected anti-nuke cover, near ours or our allies', at our strike body's
    /// order, or within the blast of a strike with no verdict lately. Near a remembered landing a place is likelier.
    fn places(&self, tick: &Tick, warhead: Warhead) -> Vec<Place> {
        let targets = self.targets();
        let covers = self.his_anti_nukes();
        // Ours and our allies' on the ground: a scout of ours is not worth a strike, and aircraft pass.
        let grounded = |def: UnitDefId| self.def(def).is_some_and(|d| !crate::air::flies(d));
        let ours = self.own.values().filter(|u| grounded(u.def) && !self.army.is_scout(u.id)).map(|u| u.pos);
        let allies = tick.snapshot.allies.iter().filter(|u| grounded(u.def)).map(|u| u.pos);
        let friends: Vec<Vec3> = ours.chain(allies).chain(self.army.strike_target().map(|(p, _)| p)).collect();
        let resting = |at: Vec3| self.nuke.strikes.iter().any(|s| !matches!(s.verdict, Verdict::Landed | Verdict::Intercepted) && s.at.dist2d(at) < warhead.radius);
        targets
            .iter()
            .map(|t| t.pos)
            .filter(|at| !covers.iter().any(|(p, c)| p.dist2d(*at) < c + COVER_MARGIN))
            .filter(|at| !self.suspected(*at))
            .filter(|at| !friends.iter().any(|p| p.dist2d(*at) < warhead.radius + FRIENDLY_MARGIN))
            .filter(|at| !resting(*at))
            .map(|at| {
                let killed = blast_kills(at, warhead, &targets);
                let worth: f32 = killed.iter().map(|k| targets[*k].worth).sum();
                let favour = if self.nuke.landings.iter().any(|(l, _)| l.dist2d(at) < LANDED_NEAR) { LANDED_FAVOUR } else { 1.0 };
                Place { at, worth, buildings: killed.len(), weight: worth * favour }
            })
            .filter(|p| p.worth >= STRIKE_LEAST)
            .collect()
    }

    /// A place drawn at random, each as likely as its weight.
    fn draw(&mut self, places: &[Place]) -> Option<usize> {
        let total: f32 = places.iter().map(|p| p.weight).sum();
        if places.is_empty() || total <= 0.0 {
            return None;
        }
        // xorshift64*: a fixed sequence a seat, so a seed's games repeat.
        let rng = &mut self.nuke.rng;
        *rng ^= *rng >> 12;
        *rng ^= *rng << 25;
        *rng ^= *rng >> 27;
        let roll = (rng.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 11) as f32 / (1u64 << 53) as f32 * total;
        let mut sum = 0.0;
        Some(
            places
                .iter()
                .position(|p| {
                    sum += p.weight;
                    roll < sum
                })
                .unwrap_or(places.len() - 1),
        )
    }

    /// One of our silos fired: its strike is launched.
    pub(crate) fn nuke_launched(&mut self, unit: UnitId) {
        if !self.own.get(&unit).is_some_and(|u| Some(u.def) == self.kit.silo) {
            return;
        }
        let Some(strike) = self.nuke.strikes.iter_mut().rev().find(|s| s.silo == unit && s.launched.is_none() && s.verdict == Verdict::Pending) else { return };
        strike.launched = Some(self.frame);
        eprintln!("[distilled ai={}] f={} nuke: silo {} launched at ({:.0},{:.0}), {:.1} s after the order", self.hello.ai_id, self.frame, unit.0, strike.at.x, strike.at.z, (self.frame - strike.ordered) as f32 / SECOND as f32);
    }

    /// An enemy damaged or destroyed by one of our silos (the events name the attacker only where the enemy is in our
    /// sight or radar): it counts to that silo's last launched strike. An anti-nuke of his seen destroyed takes the
    /// interceptions it may have made with it. Before `army.forget`.
    pub(crate) fn nuke_hit(&mut self, enemy: UnitId, attacker: Option<UnitId>, destroyed: bool) {
        let seen = self.army.his_buildings.get(&enemy).and_then(|b| b.def.and_then(|d| self.def(d)).map(|d| (b.pos, d.coverage, d.metal_cost + d.energy_cost / ENERGY_A_METAL)));
        let mobile = if destroyed { self.nuke.mobile.remove(&enemy).map(|(p, c, _)| (p, c, 0.0)) } else { None };
        if destroyed && let Some((pos, coverage, _)) = seen.filter(|(_, c, _)| *c > 0.0).or(mobile) {
            let before = self.nuke.interceptions.len();
            self.nuke.interceptions.retain(|i| i.at.dist2d(pos) >= coverage);
            eprintln!("[distilled ai={}] f={} nuke: his anti-nuke at ({:.0},{:.0}) destroyed; {} interceptions forgotten", self.hello.ai_id, self.frame, pos.x, pos.z, before - self.nuke.interceptions.len());
        }
        let Some(silo) = attacker.filter(|a| self.own.get(a).is_some_and(|u| Some(u.def) == self.kit.silo)) else { return };
        let Some(strike) = self.nuke.strikes.iter_mut().rev().find(|s| s.silo == silo && s.launched.is_some()) else { return };
        if destroyed {
            strike.seen_killed += 1;
            strike.seen_worth += seen.map_or(0.0, |(_, _, w)| w);
        } else {
            strike.seen_damaged += 1;
        }
    }

    /// D-NUKE-LOOK: where the anti-nuke behind the latest unexplained interception may stand: the grid points within
    /// its coverage of every unexplained aim near it and within its coverage of no later landing (the aim alone when
    /// none is), and the interceptions that place answers, by their frames. Unexplained: not looked at, and no known
    /// anti-nuke covers the aim.
    fn look_place(&self) -> Option<(Vec<Vec3>, Vec<i32>)> {
        let coverage = self.nuke.coverage;
        let known = self.his_anti_nukes();
        let open: Vec<&Interception> = self.nuke.interceptions.iter().filter(|i| !i.looked && !known.iter().any(|(p, c)| p.dist2d(i.at) < c + COVER_MARGIN)).collect();
        let centre = open.iter().max_by_key(|i| i.frame)?.at;
        let cluster: Vec<&Interception> = open.into_iter().filter(|i| i.at.dist2d(centre) < 2.0 * coverage).collect();
        let since = cluster.iter().map(|i| i.frame).min().unwrap_or(0);
        let steps = (coverage / LOOK_GRID) as i32;
        let mut points = Vec::new();
        for gx in -steps..=steps {
            for gz in -steps..=steps {
                let p = Vec3 { x: centre.x + gx as f32 * LOOK_GRID, y: centre.y, z: centre.z + gz as f32 * LOOK_GRID };
                let inside = cluster.iter().all(|i| i.at.dist2d(p) < coverage);
                let refuted = self.nuke.landings.iter().any(|(l, when)| *when > since && l.dist2d(p) < coverage);
                if inside && !refuted {
                    points.push(p);
                }
            }
        }
        if points.is_empty() {
            points.push(centre);
        }
        Some((points, cluster.iter().map(|i| i.frame).collect()))
    }

    /// D-NUKE-LOOK: a scout sweeps the place where his anti-nuke may stand, each time to the nearest of its points not
    /// yet in our sight, until an anti-nuke known covers every aim of the look (found), every point has been in sight
    /// (looked over), or `LOOK_TRIES` scouts were lost (given up). The aircraft plant's scout plane (Peeper, Finch) when
    /// it has one; else a ground scout of ours, if any lives. One look at a time, kept until it ends.
    fn run_look(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let (ai, frame) = (self.hello.ai_id, self.frame);
        let mut look = match self.nuke.look.take() {
            Some(look) => look,
            None => {
                let Some((points, cluster)) = self.look_place() else { return };
                let n = points.len() as f32;
                let (x, z) = points.iter().fold((0.0, 0.0), |(x, z), p| (x + p.x / n, z + p.z / n));
                eprintln!("[distilled ai={ai}] f={frame} nuke: his anti-nuke may stand round ({x:.0},{z:.0}), {} points to look at, from {} interceptions; a scout goes to look", points.len(), cluster.len());
                Look { cluster, points, scout: None, to: None, told: 0, tries: 0 }
            }
        };
        let aims: Vec<Vec3> = self.nuke.interceptions.iter().filter(|i| look.cluster.contains(&i.frame)).map(|i| i.at).collect();
        let known = self.his_anti_nukes();
        let found = !aims.is_empty() && aims.iter().all(|a| known.iter().any(|(p, c)| p.dist2d(*a) < c + COVER_MARGIN));
        if let Some(sensors) = &tick.snapshot.sensors {
            look.points.retain(|p| !sensors.los.at(p.x, p.z));
        }
        if look.scout.is_some_and(|id| !self.own.contains_key(&id)) {
            look.tries += 1;
            look.scout = None;
        }
        let ended = if aims.is_empty() {
            Some("forgotten")
        } else if found {
            Some("found")
        } else if look.points.is_empty() {
            Some("looked over")
        } else if look.tries >= LOOK_TRIES {
            Some("given up")
        } else {
            None
        };
        if let Some(how) = ended {
            for i in self.nuke.interceptions.iter_mut().filter(|i| look.cluster.contains(&i.frame)) {
                i.looked = true;
            }
            eprintln!("[distilled ai={ai}] f={frame} nuke: the look {how}, {} points left, {} scouts lost; his anti-nukes known {}", look.points.len(), look.tries, known.len());
            return;
        }
        let from = look.scout.and_then(|id| self.own.get(&id)).map_or(self.home, |u| u.pos);
        let nearest = |at: Vec3| look.points.iter().copied().min_by(|a, b| a.dist2d(at).total_cmp(&b.dist2d(at)));
        let air = self.finished(self.kit.air_scout).min_by(|a, b| a.pos.dist2d(from).total_cmp(&b.pos.dist2d(from))).map(|u| (u.id, u.idle));
        let ground = || self.own.values().filter(|u| self.army.is_scout(u.id)).min_by(|a, b| a.pos.dist2d(from).total_cmp(&b.pos.dist2d(from))).map(|u| (u.id, u.idle));
        let scout = look.scout.and_then(|id| self.own.get(&id).map(|u| (u.id, u.idle))).or(air).or_else(ground);
        if let Some((id, idle)) = scout {
            // A new scout, a point reached or seen, or an idle scout due a word: on to the nearest point left.
            let arrived = look.to.is_none_or(|to| !look.points.contains(&to));
            if look.scout != Some(id) || arrived || (idle && frame - look.told >= LOOK_RETELL) {
                let to = self.own.get(&id).and_then(|u| nearest(u.pos));
                if let Some((to, from)) = to.zip(self.own.get(&id).map(|u| u.pos)) {
                    look.scout = Some(id);
                    look.to = Some(to);
                    look.told = frame;
                    for (i, at) in self.flank_route(from, to).into_iter().enumerate() {
                        commands.push(Command::Move { unit: id, to: at, queue: i > 0 });
                    }
                }
            }
        }
        self.nuke.look = Some(look);
    }

    /// D-NUKE-LOOK: the way a scout flies to a point. From our half along the side edge of the map nearer the point,
    /// `FLANK_INSET` in, to the point's level, then in to it: his armies fight down the middle with their mobile
    /// anti-air (dist55: nineteen scout planes flown straight lost over the middle, many in our half, to Archangels,
    /// Manticores and Sentries, none having seen a point of the place). From his half, straight there.
    fn flank_route(&self, from: Vec3, to: Vec3) -> Vec<Vec3> {
        if from.dist2d(self.home) >= from.dist2d(self.enemy_home) {
            return vec![to];
        }
        let (width, height) = (self.hello.map.width, self.hello.map.height);
        let (dx, dz) = (self.enemy_home.x - self.home.x, self.enemy_home.z - self.home.z);
        let edge = |v: f32, size: f32| if v < size - v { FLANK_INSET } else { size - FLANK_INSET };
        if dz.abs() >= dx.abs() {
            // The fronts face along z: the side edges are x = 0 and x = width.
            let x = edge(to.x, width);
            vec![Vec3 { x, ..from }, Vec3 { x, ..to }, to]
        } else {
            let z = edge(to.z, height);
            vec![Vec3 { z, ..from }, Vec3 { z, ..to }, to]
        }
    }

    /// D-NUKE-LOOK: the aircraft plant makes a scout plane while one is wanted and none is alive.
    pub(crate) fn air_scout_wanted(&self) -> bool {
        self.nuke.look.is_some() && self.kit.air_scout.is_some() && self.count(self.kit.air_scout) == 0
    }

    /// The minute's `nuke:` line, once a silo has been begun.
    pub(crate) fn log_nukes(&self, tick: &Tick) {
        let Some(warhead) = self.nuke.warhead.filter(|_| self.count(self.kit.silo) > 0) else { return };
        let silos: Vec<&OwnUnit> = self.finished(self.kit.silo).collect();
        let (ready, queued): (i32, i32) = silos.iter().fold((0, 0), |(r, q), s| (r + s.stockpile, q + s.stockpile_queued));
        let verdicts = |v: Verdict| self.nuke.strikes.iter().filter(|s| s.verdict == v).count();
        eprintln!(
            "[distilled ai={}] f={} nuke: silos {} standing ({} paused), {} frames; missiles {ready} ready, {queued} stockpiling; draw {:.0}/s; strikes in the last {} min: {} landed, {} intercepted, {} unknown, {} pending; remembered {} interceptions, {} landings; places worth striking {} of {} buildings known, his anti-nukes known {}{}",
            self.hello.ai_id,
            self.frame,
            silos.len(),
            self.nuke.paused.len(),
            self.count(self.kit.silo) - silos.len(),
            self.silo_draw(tick),
            STRIKE_REST / MINUTE,
            verdicts(Verdict::Landed),
            verdicts(Verdict::Intercepted),
            verdicts(Verdict::Unknown),
            verdicts(Verdict::Pending),
            self.nuke.interceptions.len(),
            self.nuke.landings.len(),
            self.places(tick, warhead).len(),
            self.army.his_buildings.len(),
            self.his_anti_nukes().len(),
            if self.nuke.flat { "; the lobby keeps the ground flat: no verdicts" } else { "" }
        );
    }
}

/// Whether the place is in our team's sight or radar, as a word.
fn sensed(tick: &Tick, at: Vec3) -> &'static str {
    match &tick.snapshot.sensors {
        Some(s) if s.los.at(at.x, at.z) => "sight",
        Some(s) if s.radar.at(at.x, at.z) => "radar",
        Some(_) => "none",
        None => "unknown",
    }
}

/// The probe's points: the aim first (the answer is matched to its strike by it), then eight on each ring.
fn probe_points(at: Vec3) -> Vec<Vec3> {
    let mut points = vec![at];
    for r in PROBE_RINGS {
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::FRAC_PI_4;
            points.push(Vec3 { x: at.x + r * a.cos(), y: 0.0, z: at.z + r * a.sin() });
        }
    }
    points
}

/// Whether the start script's mod options turn map deformation off (BAR's "Disable Map Deformation",
/// `disablemapdamage`).
fn deformation_off(script: &str) -> bool {
    let lower = script.to_ascii_lowercase();
    lower.split(';').filter_map(|field| field.split_once('=')).any(|(key, value)| key.trim().trim_start_matches('{').trim().ends_with("disablemapdamage") && value.trim() == "1")
}

/// The buildings a warhead at `at` kills, by index: each whose health is under the damage at its distance, and then
/// those a killed building's own death blast kills in turn (one step of the chain).
fn blast_kills(at: Vec3, warhead: Warhead, targets: &[Target]) -> Vec<usize> {
    let falloff = |damage: f32, radius: f32, d: f32| if d >= radius { 0.0 } else { damage * (1.0 - d / radius) };
    let mut killed: Vec<bool> = targets.iter().map(|t| falloff(warhead.damage, warhead.radius, t.pos.dist2d(at)) > t.health).collect();
    let first: Vec<usize> = (0..targets.len()).filter(|i| killed[*i]).collect();
    for i in first {
        let Some(blast) = targets[i].blast else { continue };
        for (j, t) in targets.iter().enumerate() {
            if !killed[j] && falloff(blast.damage, blast.radius, t.pos.dist2d(targets[i].pos)) > t.health {
                killed[j] = true;
            }
        }
    }
    (0..targets.len()).filter(|i| killed[*i]).collect()
}

#[cfg(test)]
mod tests {
    use super::{Target, Warhead, blast_kills, deformation_off};
    use bot_protocol::{Blast, UnitId, Vec3};

    fn target(x: f32, health: f32, worth: f32, blast: Option<Blast>) -> Target {
        Target { id: UnitId(x as i32), pos: Vec3 { x, y: 0.0, z: 0.0 }, health, worth, blast }
    }

    /// Armada's warhead (640, 12,500 at the centre): a solar at 300 dies (6,600 against its 340), a fusion at 400 dies
    /// (4,700 against 3,800) and one at 500 stands (2,700), and a fusion killed takes what its own blast reaches.
    #[test]
    fn the_warhead_kills_by_health_at_distance_and_the_chain_one_step() {
        let warhead = Warhead { radius: 640.0, damage: 12500.0 };
        let at = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let fusion_blast = Some(Blast { radius: 400.0, damage: 5000.0 });
        // The solar at 300 and the fusion at 400 die; the fusion's blast reaches the fusion at 500 (100 away: 3,750
        // against 3,800, it stands) and not the solar at 900 (500 away, beyond its 400).
        let targets = vec![target(300.0, 340.0, 155.0, None), target(400.0, 3800.0, 3600.0, fusion_blast), target(500.0, 3800.0, 3600.0, None), target(900.0, 340.0, 155.0, None)];
        assert_eq!(blast_kills(at, warhead, &targets), vec![0, 1]);
        // A bigger blast takes the solar at 900 (500 from the fusion: 1,500 against 340).
        let stronger = vec![target(300.0, 340.0, 155.0, None), target(400.0, 3800.0, 3600.0, Some(Blast { radius: 600.0, damage: 9000.0 })), target(900.0, 340.0, 155.0, None)];
        assert_eq!(blast_kills(at, warhead, &stronger), vec![0, 1, 2]);
    }

    #[test]
    fn the_lobby_option_for_a_flat_map_is_read_from_the_mod_options() {
        let on = "[GAME]\n{\n\t[modoptions]\n\t{\n\t\tdisablemapdamage=1;\n\t\tdeathmode=com;\n\t}\n}\n";
        let off = "[GAME]\n{\n\t[modoptions]\n\t{\n\t\tdisablemapdamage = 0;\n\t}\n}\n";
        assert!(deformation_off(on));
        assert!(!deformation_off(off));
        assert!(!deformation_off(""));
    }
}
