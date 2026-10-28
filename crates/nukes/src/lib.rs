//! The nuclear silos' strikes, for the heuristic bots of `bots/`: WRDistilled's D-NUKE-FIRE to D-NUKE-LOOK
//! (`bots/distilled/README.md`, `docs/design/2026-10-27-distilled-nukes.md`) and WRTerritory's T-NUKE
//! (`bots/territory/README.md`, `docs/design/2026-10-27-territory-nukes.md`). The flagship does not use it.
//!
//! Each silo with a missile fires at a place of his chosen at random by worth, outside his known and suspected
//! anti-nuke cover. What became of a strike is read off the ground: a missile that lands digs a crater players see, one
//! intercepted bursts in the air (the user, 2026-10-27). A landing's ruins are handed back for the bot to forget; an
//! interception marks his cover, and a scout, for a bot that offers one, goes to look for the anti-nuke behind it. How
//! a bot builds its silos and what it knows of his buildings are its own: it hands the engine a `View` once a second.

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{Blast, Command, EnemyUnit, OwnUnit, Sensors, UnitDefId, UnitDefInfo, UnitId, Vec3};

const SECOND: i32 = 30;
const MINUTE: i32 = 60 * SECOND;

/// A unit's worth: its metal and its energy at the converter's rate (70 energy a metal, K-mech-converter-rates).
pub const ENERGY_A_METAL: f32 = 70.0;
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
/// With no place worth striking this long, the silos stop stockpiling (D-NUKE-HOLD).
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

/// A silo's missile: the radius of its blast and the damage at the centre, falling to nothing at the edge (BAR's
/// warheads have an edge effectiveness of 0).
#[derive(Clone, Copy, Debug)]
pub struct Warhead {
    pub radius: f32,
    pub damage: f32,
}

/// A type's warhead in the combat tables: its stockpiled weapon fired on command (an attack at the ground) that
/// paralyzes nothing. A nuclear silo has one; so do the tactical launchers, with far smaller blasts, and the EMP
/// launcher's is a paralyzer's.
pub fn warhead(table: &combatsim::units::Units, name: &str) -> Option<Warhead> {
    let unit = &table.list[table.index(name)?];
    unit.weapons.iter().find(|w| w.stockpile && w.command_fire && !w.paralyzer).map(|w| Warhead { radius: w.aoe / 2.0, damage: w.damage.get("default").copied().unwrap_or(0.0) })
}

/// A building of his known to the bot.
#[derive(Clone, Copy, Debug)]
pub struct Building {
    pub id: UnitId,
    pub pos: Vec3,
    pub def: UnitDefId,
}

/// D-NUKE-LOOK's means, for a bot that has them: our scout planes and our ground scouts (each id, place, idle), and
/// the map the planes flank along.
pub struct Scouting {
    pub planes: Vec<(UnitId, Vec3, bool)>,
    pub ground: Vec<(UnitId, Vec3, bool)>,
    pub home: Vec3,
    pub enemy_home: Vec3,
    /// The map's width and height in elmos.
    pub map: (f32, f32),
}

/// What the bot hands the engine each second.
pub struct View<'a> {
    pub frame: i32,
    /// Our silos standing.
    pub silos: Vec<OwnUnit>,
    /// His buildings known, with their types.
    pub buildings: Vec<Building>,
    /// Places no missile may land near: ours and our allies' on the ground, and any place our soldiers are sent.
    pub friends: Vec<Vec3>,
    /// His units in sight (his mobile anti-nukes among them).
    pub enemies: &'a [EnemyUnit],
    pub sensors: Option<&'a Sensors>,
    /// None for a bot that sends no scout: D-NUKE-LOOK does not run.
    pub scouting: Option<Scouting>,
}

/// What the engine reads of a unit type.
#[derive(Clone, Copy)]
struct Type {
    /// From the combat tables: what the warhead kills.
    health: Option<f32>,
    worth: f32,
    coverage: f32,
    blast: Option<Blast>,
    mobile: bool,
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

/// A building of his as the warhead sees it.
struct Target {
    id: UnitId,
    pos: Vec3,
    health: f32,
    worth: f32,
    blast: Option<Blast>,
}

/// A place worth striking: the centre, the worth killed, the buildings killed, and how likely it is to be drawn.
struct Place {
    at: Vec3,
    worth: f32,
    buildings: usize,
    weight: f32,
}

#[derive(Default)]
pub struct Nukes {
    /// The bot's log prefix (`[distilled ai=1]`).
    prefix: String,
    warhead: Option<Warhead>,
    types: BTreeMap<UnitDefId, Type>,
    /// An anti-nuke's coverage (2,000 in BAR): the widest any type has.
    coverage: f32,
    /// The lobby turned map deformation off: no crater tells a landing.
    flat: bool,
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
    rng: u64,
}

impl Nukes {
    /// The engine for a seat: the log prefix, every type's health (the combat tables), worth, coverage and death
    /// blast, the warhead of our silo, and whether the lobby keeps the ground flat.
    pub fn new(prefix: String, seat: i32, defs: &[UnitDefInfo], table: &combatsim::units::Units, silo: Option<&UnitDefInfo>, script: &str) -> Nukes {
        let types = defs
            .iter()
            .map(|d| (d.id, Type { health: table.index(&d.name).map(|i| table.list[i].health), worth: d.metal_cost + d.energy_cost / ENERGY_A_METAL, coverage: d.coverage, blast: d.death_blast, mobile: d.speed > 0.0 }))
            .collect();
        Nukes {
            prefix,
            warhead: silo.and_then(|d| warhead(table, &d.name)),
            types,
            coverage: defs.iter().map(|d| d.coverage).fold(0.0, f32::max),
            flat: deformation_off(script),
            rng: 0x9e37_79b9_7f4a_7c15 ^ (seat as u64).wrapping_mul(0x2545_f491_4f6c_dd1d),
            ..Nukes::default()
        }
    }

    pub fn warhead(&self) -> Option<Warhead> {
        self.warhead
    }

    /// The lobby keeps the ground flat: no verdicts.
    pub fn flat(&self) -> bool {
        self.flat
    }

    /// A place worth striking was found at the last look.
    pub fn open(&self) -> bool {
        self.open
    }

    /// The silo is paused by D-NUKE-HOLD: it draws nothing.
    pub fn is_paused(&self, silo: UnitId) -> bool {
        self.paused.contains(&silo)
    }

    /// A look for his anti-nuke is under way (D-NUKE-LOOK).
    pub fn looking(&self) -> bool {
        self.look.is_some()
    }

    /// Once a second: the strikes' verdicts, the memory of his cover, the hold (D-NUKE-HOLD), the strikes
    /// (D-NUKE-FIRE) and, with scouts, the look (D-NUKE-LOOK).
    pub fn run(&mut self, view: &View, commands: &mut Vec<Command>) {
        let Some(warhead) = self.warhead else { return };
        self.remember_mobile_anti_nukes(view);
        self.follow_strikes(view, commands);
        let frame = view.frame;
        self.interceptions.retain(|i| frame - i.frame < COVER_MEMORY);
        self.landings.retain(|(_, at)| frame - at < COVER_MEMORY);
        self.strikes.retain(|s| s.verdict == Verdict::Pending || frame - s.ordered < STRIKE_REST);
        let mut places = self.places(view, warhead);
        self.hold(!places.is_empty(), view, commands);
        let ready: Vec<UnitId> = view.silos.iter().filter(|s| s.stockpile > 0 && self.ordered.get(&s.id).is_none_or(|at| frame - at >= FIRE_GAP)).map(|s| s.id).collect();
        for silo in ready {
            let Some(place) = self.draw(&places) else { break };
            let Place { at, worth, buildings, .. } = places[place];
            let cover = self.his_anti_nukes(view).iter().map(|(p, _)| p.dist2d(at)).min_by(f32::total_cmp);
            eprintln!(
                "{} f={frame} nuke: silo {} fires at ({:.0},{:.0}) blast {:.0}: {buildings} of his buildings worth {worth:.0} expected, nearest known anti-nuke {}, the place in our {}; {} places worth striking",
                self.prefix,
                silo.0,
                at.x,
                at.z,
                warhead.radius,
                cover.map_or("none".to_string(), |d| format!("{d:.0} away")),
                sensed(view.sensors, at),
                places.len()
            );
            commands.push(Command::FireAt { unit: silo, at });
            if !self.flat {
                commands.push(Command::Probe { points: probe_points(at) });
            }
            self.ordered.insert(silo, frame);
            self.strikes.push(Strike { silo, at, ordered: frame, launched: None, worth, buildings, before: None, asked_after: None, verdict: Verdict::Pending, seen_killed: 0, seen_worth: 0.0, seen_damaged: 0 });
            // The places within the blast of this strike rest until its verdict, as `places` would reckon them.
            places.retain(|p| p.at.dist2d(at) >= warhead.radius);
        }
        if let Some(scouting) = &view.scouting {
            self.run_look(view, scouting, commands);
        }
    }

    /// Each pending strike: not launched in time; the ground asked again once the warhead has landed; no answer, or
    /// no ground to ask (the lobby's flat map), and the verdict is unknown.
    fn follow_strikes(&mut self, view: &View, commands: &mut Vec<Command>) {
        let frame = view.frame;
        let mut decided = Vec::new();
        for (i, s) in self.strikes.iter_mut().enumerate().filter(|(_, s)| s.verdict == Verdict::Pending) {
            match (s.launched, s.asked_after) {
                (None, _) if frame - s.ordered >= LAUNCH_WAIT => {
                    s.verdict = Verdict::NotLaunched;
                    decided.push(i);
                }
                (Some(launched), None) if frame - launched >= SETTLE => {
                    if self.flat || s.before.is_none() {
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
            self.report_strike(i, None, &view.buildings, frame);
        }
    }

    /// The shim's answer to a probe: the ground before a strike, or after it and the verdict. A landing's ruins (what
    /// the warhead killed by the model among his buildings known) are returned, for the bot to forget (D-NUKE-RUINS).
    pub fn ground(&mut self, points: &[Vec3], buildings: &[Building], frame: i32) -> Vec<UnitId> {
        let Some(aim) = points.first() else { return Vec::new() };
        let Some(i) = self.strikes.iter().rposition(|s| s.verdict == Verdict::Pending && s.at.x == aim.x && s.at.z == aim.z) else { return Vec::new() };
        let heights: Vec<f32> = points.iter().map(|p| p.y).collect();
        let strike = &mut self.strikes[i];
        let Some(before) = strike.before.as_ref() else {
            strike.before = Some(heights);
            return Vec::new();
        };
        if strike.asked_after.is_none() || before.len() != heights.len() {
            return Vec::new();
        }
        let drops: Vec<f32> = before.iter().zip(&heights).map(|(b, a)| b - a).collect();
        let mean = drops.iter().sum::<f32>() / drops.len().max(1) as f32;
        let landed = mean >= CRATER_DEPTH;
        strike.verdict = if landed { Verdict::Landed } else { Verdict::Intercepted };
        let at = strike.at;
        if landed {
            self.landings.push((at, frame));
        } else {
            self.interceptions.push(Interception { at, frame, looked: false });
        }
        let max = drops.iter().copied().fold(f32::MIN, f32::max);
        self.report_strike(i, Some((mean, max)), buildings, frame)
    }

    /// A strike decided: the log line, and a landing's ruins returned (what the warhead kills by the model among his
    /// buildings known, so that neither the silos nor the army go back for it until it is seen again).
    fn report_strike(&mut self, i: usize, drop: Option<(f32, f32)>, buildings: &[Building], frame: i32) -> Vec<UnitId> {
        let Some(warhead) = self.warhead else { return Vec::new() };
        let s = &self.strikes[i];
        let (at, verdict) = (s.at, s.verdict);
        let mut ruins = Vec::new();
        if verdict == Verdict::Landed {
            let targets = self.targets(buildings);
            ruins = blast_kills(at, warhead, &targets).into_iter().map(|k| targets[k].id).collect();
        }
        let ground = drop.map_or(String::new(), |(mean, max)| format!(" (the ground down {mean:.1} on the mean, {max:.1} at most)"));
        let after = match verdict {
            Verdict::Landed => format!("; {} of his buildings presumed dead", ruins.len()),
            Verdict::Intercepted => "; an anti-nuke of his within its coverage, remembered".to_string(),
            _ => String::new(),
        };
        eprintln!(
            "{} f={frame} nuke: strike at ({:.0},{:.0}) by silo {}: {}{ground}; expected {} worth {:.0}; the events saw killed {} worth {:.0}, damaged {}{after}",
            self.prefix,
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
        ruins
    }

    /// D-NUKE-HOLD: with no place worth striking for `HOLD_AFTER`, every silo's kept count goes to nothing, so it
    /// draws no energy (the bots start no silo meanwhile); a place found again brings them back to the game's ten.
    fn hold(&mut self, open: bool, view: &View, commands: &mut Vec<Command>) {
        let frame = view.frame;
        self.open = open;
        let silos: Vec<UnitId> = view.silos.iter().map(|s| s.id).collect();
        self.paused.retain(|id| silos.contains(id));
        if open {
            self.closed_since = None;
            if !self.paused.is_empty() {
                eprintln!("{} f={frame} nuke: a place worth striking again; {} silos stockpile again", self.prefix, self.paused.len());
                commands.extend(self.paused.iter().map(|&unit| Command::Stockpile { unit, change: KEPT_STEP }));
                self.paused.clear();
            }
            return;
        }
        let since = *self.closed_since.get_or_insert(frame);
        if frame - since < HOLD_AFTER {
            return;
        }
        let pause: Vec<UnitId> = silos.into_iter().filter(|id| !self.paused.contains(id)).collect();
        if !pause.is_empty() {
            eprintln!("{} f={frame} nuke: no place worth striking for {} s; {} silos stop stockpiling", self.prefix, (frame - since) / SECOND, pause.len());
            commands.extend(pause.iter().map(|&unit| Command::Stockpile { unit, change: -KEPT_STEP }));
            self.paused.extend(pause);
        }
    }

    /// His anti-nukes known: his buildings with an interceptor's coverage, and his mobile ones where last seen.
    fn his_anti_nukes(&self, view: &View) -> Vec<(Vec3, f32)> {
        let buildings = view.buildings.iter().map(|b| (b.pos, self.types.get(&b.def).map_or(0.0, |t| t.coverage)));
        let mobile = self.mobile.values().map(|(p, c, _)| (*p, *c));
        buildings.chain(mobile).filter(|(_, c)| *c > 0.0).collect()
    }

    /// His mobile anti-nukes in sight are remembered where they were last seen, `COVER_MEMORY` long (a BARb seat drives
    /// one round its base: dist54 game 01, the cormabm took strikes 2,000 apart while the bot knew of no anti-nuke).
    fn remember_mobile_anti_nukes(&mut self, view: &View) {
        let frame = view.frame;
        for e in view.enemies {
            if let Some(t) = e.def.and_then(|d| self.types.get(&d)).filter(|t| t.mobile && t.coverage > 0.0) {
                self.mobile.insert(e.id, (e.pos, t.coverage, frame));
            }
        }
        self.mobile.retain(|_, (_, _, seen)| frame - *seen < COVER_MEMORY);
    }

    /// His known buildings as the warhead sees them.
    fn targets(&self, buildings: &[Building]) -> Vec<Target> {
        buildings
            .iter()
            .filter_map(|b| {
                let t = self.types.get(&b.def)?;
                Some(Target { id: b.id, pos: b.pos, health: t.health?, worth: t.worth, blast: t.blast })
            })
            .collect()
    }

    /// D-NUKE-COVER: a place is suspected covered when it lies within an anti-nuke's coverage of a remembered
    /// interception (one of his stood within that of the aim, K-rules-interception-needs-the-aim-point-in-coverage),
    /// unless a later landing lies nearer to it than that aim.
    fn suspected(&self, at: Vec3) -> bool {
        self.interceptions.iter().any(|i| {
            let d = i.at.dist2d(at);
            d < self.coverage && !self.landings.iter().any(|(l, when)| *when > i.frame && l.dist2d(at) < d)
        })
    }

    /// The places worth striking: each of his known buildings as the blast's centre, with the worth the warhead would
    /// kill there; none under his known or suspected anti-nuke cover, near the bot's friends, or within the blast of a
    /// strike with no verdict lately. Near a remembered landing a place is likelier.
    fn places(&self, view: &View, warhead: Warhead) -> Vec<Place> {
        let targets = self.targets(&view.buildings);
        let covers = self.his_anti_nukes(view);
        let resting = |at: Vec3| self.strikes.iter().any(|s| !matches!(s.verdict, Verdict::Landed | Verdict::Intercepted) && s.at.dist2d(at) < warhead.radius);
        targets
            .iter()
            .map(|t| t.pos)
            .filter(|at| !covers.iter().any(|(p, c)| p.dist2d(*at) < c + COVER_MARGIN))
            .filter(|at| !self.suspected(*at))
            .filter(|at| !view.friends.iter().any(|p| p.dist2d(*at) < warhead.radius + FRIENDLY_MARGIN))
            .filter(|at| !resting(*at))
            .map(|at| {
                let killed = blast_kills(at, warhead, &targets);
                let worth: f32 = killed.iter().map(|k| targets[*k].worth).sum();
                let favour = if self.landings.iter().any(|(l, _)| l.dist2d(at) < LANDED_NEAR) { LANDED_FAVOUR } else { 1.0 };
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
        let rng = &mut self.rng;
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

    /// A unit of ours fired: a strike of that silo ordered and not yet launched is launched.
    pub fn launched(&mut self, unit: UnitId, frame: i32) {
        let Some(strike) = self.strikes.iter_mut().rev().find(|s| s.silo == unit && s.launched.is_none() && s.verdict == Verdict::Pending) else { return };
        strike.launched = Some(frame);
        eprintln!("{} f={frame} nuke: silo {} launched at ({:.0},{:.0}), {:.1} s after the order", self.prefix, unit.0, strike.at.x, strike.at.z, (frame - strike.ordered) as f32 / SECOND as f32);
    }

    /// An enemy damaged or destroyed, with where the bot knew it as a building of his (`seen`) and the attacker the
    /// event named (only where the enemy is in our sight or radar): a hit by a silo of ours counts to its last launched
    /// strike. An anti-nuke of his seen destroyed takes the interceptions it may have made with it. Before the bot
    /// forgets the building.
    pub fn hit(&mut self, enemy: UnitId, seen: Option<(Vec3, UnitDefId)>, attacker: Option<UnitId>, destroyed: bool, frame: i32) {
        let seen = seen.and_then(|(pos, def)| self.types.get(&def).map(|t| (pos, t.coverage, t.worth)));
        let mobile = if destroyed { self.mobile.remove(&enemy).map(|(p, c, _)| (p, c, 0.0)) } else { None };
        if destroyed && let Some((pos, coverage, _)) = seen.filter(|(_, c, _)| *c > 0.0).or(mobile) {
            let before = self.interceptions.len();
            self.interceptions.retain(|i| i.at.dist2d(pos) >= coverage);
            eprintln!("{} f={frame} nuke: his anti-nuke at ({:.0},{:.0}) destroyed; {} interceptions forgotten", self.prefix, pos.x, pos.z, before - self.interceptions.len());
        }
        let Some(silo) = attacker else { return };
        let Some(strike) = self.strikes.iter_mut().rev().find(|s| s.silo == silo && s.launched.is_some()) else { return };
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
    fn look_place(&self, view: &View) -> Option<(Vec<Vec3>, Vec<i32>)> {
        let coverage = self.coverage;
        let known = self.his_anti_nukes(view);
        let open: Vec<&Interception> = self.interceptions.iter().filter(|i| !i.looked && !known.iter().any(|(p, c)| p.dist2d(i.at) < c + COVER_MARGIN)).collect();
        let centre = open.iter().max_by_key(|i| i.frame)?.at;
        let cluster: Vec<&Interception> = open.into_iter().filter(|i| i.at.dist2d(centre) < 2.0 * coverage).collect();
        let since = cluster.iter().map(|i| i.frame).min().unwrap_or(0);
        let steps = (coverage / LOOK_GRID) as i32;
        let mut points = Vec::new();
        for gx in -steps..=steps {
            for gz in -steps..=steps {
                let p = Vec3 { x: centre.x + gx as f32 * LOOK_GRID, y: centre.y, z: centre.z + gz as f32 * LOOK_GRID };
                let inside = cluster.iter().all(|i| i.at.dist2d(p) < coverage);
                let refuted = self.landings.iter().any(|(l, when)| *when > since && l.dist2d(p) < coverage);
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
    /// (looked over), or `LOOK_TRIES` scouts were lost (given up). A scout plane when the bot has one; else a ground
    /// scout, if any lives. One look at a time, kept until it ends.
    fn run_look(&mut self, view: &View, scouting: &Scouting, commands: &mut Vec<Command>) {
        let frame = view.frame;
        let mut look = match self.look.take() {
            Some(look) => look,
            None => {
                let Some((points, cluster)) = self.look_place(view) else { return };
                let n = points.len() as f32;
                let (x, z) = points.iter().fold((0.0, 0.0), |(x, z), p| (x + p.x / n, z + p.z / n));
                eprintln!("{} f={frame} nuke: his anti-nuke may stand round ({x:.0},{z:.0}), {} points to look at, from {} interceptions; a scout goes to look", self.prefix, points.len(), cluster.len());
                Look { cluster, points, scout: None, to: None, told: 0, tries: 0 }
            }
        };
        let aims: Vec<Vec3> = self.interceptions.iter().filter(|i| look.cluster.contains(&i.frame)).map(|i| i.at).collect();
        let known = self.his_anti_nukes(view);
        let found = !aims.is_empty() && aims.iter().all(|a| known.iter().any(|(p, c)| p.dist2d(*a) < c + COVER_MARGIN));
        if let Some(sensors) = view.sensors {
            look.points.retain(|p| !sensors.los.at(p.x, p.z));
        }
        let unit = |id: UnitId| scouting.planes.iter().chain(&scouting.ground).find(|(u, _, _)| *u == id).copied();
        if look.scout.is_some_and(|id| unit(id).is_none()) {
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
            for i in self.interceptions.iter_mut().filter(|i| look.cluster.contains(&i.frame)) {
                i.looked = true;
            }
            eprintln!("{} f={frame} nuke: the look {how}, {} points left, {} scouts lost; his anti-nukes known {}", self.prefix, look.points.len(), look.tries, known.len());
            return;
        }
        let from = look.scout.and_then(unit).map_or(scouting.home, |(_, p, _)| p);
        let nearest = |at: Vec3| look.points.iter().copied().min_by(|a, b| a.dist2d(at).total_cmp(&b.dist2d(at)));
        let closest = |list: &[(UnitId, Vec3, bool)]| list.iter().min_by(|a, b| a.1.dist2d(from).total_cmp(&b.1.dist2d(from))).map(|(id, _, idle)| (*id, *idle));
        let scout = look.scout.and_then(|id| unit(id).map(|(u, _, idle)| (u, idle))).or_else(|| closest(&scouting.planes)).or_else(|| closest(&scouting.ground));
        if let Some((id, idle)) = scout {
            // A new scout, a point reached or seen, or an idle scout due a word: on to the nearest point left.
            let arrived = look.to.is_none_or(|to| !look.points.contains(&to));
            if look.scout != Some(id) || arrived || (idle && frame - look.told >= LOOK_RETELL) {
                let pos = unit(id).map(|(_, p, _)| p);
                if let Some((to, from)) = pos.and_then(nearest).zip(pos) {
                    look.scout = Some(id);
                    look.to = Some(to);
                    look.told = frame;
                    for (i, at) in flank_route(scouting, from, to).into_iter().enumerate() {
                        commands.push(Command::Move { unit: id, to: at, queue: i > 0 });
                    }
                }
            }
        }
        self.look = Some(look);
    }

    /// The minute's `nuke:` line: our silos standing (and paused) and `frames` framed, their missiles, the silos'
    /// stockpile `draw` as the bot counts it, the strikes of the last minutes by verdict, the cover memory, the places
    /// worth striking among his buildings known, and his anti-nukes known.
    pub fn log_line(&self, view: &View, frames: usize, draw: f32) {
        let Some(warhead) = self.warhead else { return };
        let (ready, queued): (i32, i32) = view.silos.iter().fold((0, 0), |(r, q), s| (r + s.stockpile, q + s.stockpile_queued));
        let verdicts = |v: Verdict| self.strikes.iter().filter(|s| s.verdict == v).count();
        eprintln!(
            "{} f={} nuke: silos {} standing ({} paused), {frames} frames; missiles {ready} ready, {queued} stockpiling; draw {draw:.0}/s; strikes in the last {} min: {} landed, {} intercepted, {} unknown, {} pending; remembered {} interceptions, {} landings; places worth striking {} of {} buildings known, his anti-nukes known {}{}",
            self.prefix,
            view.frame,
            view.silos.len(),
            self.paused.len(),
            STRIKE_REST / MINUTE,
            verdicts(Verdict::Landed),
            verdicts(Verdict::Intercepted),
            verdicts(Verdict::Unknown),
            verdicts(Verdict::Pending),
            self.interceptions.len(),
            self.landings.len(),
            self.places(view, warhead).len(),
            view.buildings.len(),
            self.his_anti_nukes(view).len(),
            if self.flat { "; the lobby keeps the ground flat: no verdicts" } else { "" }
        );
    }
}

/// D-NUKE-LOOK: the way a scout flies to a point. From our half along the side edge of the map nearer the point,
/// `FLANK_INSET` in, to the point's level, then in to it: his armies fight down the middle with their mobile
/// anti-air (dist55: nineteen scout planes flown straight lost over the middle, many in our half, to Archangels,
/// Manticores and Sentries, none having seen a point of the place). From his half, straight there.
fn flank_route(scouting: &Scouting, from: Vec3, to: Vec3) -> Vec<Vec3> {
    let (home, enemy_home) = (scouting.home, scouting.enemy_home);
    if from.dist2d(home) >= from.dist2d(enemy_home) {
        return vec![to];
    }
    let (width, height) = scouting.map;
    let (dx, dz) = (enemy_home.x - home.x, enemy_home.z - home.z);
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

/// Whether the place is in our team's sight or radar, as a word.
fn sensed(sensors: Option<&Sensors>, at: Vec3) -> &'static str {
    match sensors {
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
/// those a killed building's own death blast kills in turn (one step of the chain). Only the targets within the
/// warhead's radius and the widest death blast of `at` can die, so the chain searches those alone (over every target
/// it cost each place its kills times every building known, on the engine thread, once a second).
fn blast_kills(at: Vec3, warhead: Warhead, targets: &[Target]) -> Vec<usize> {
    let falloff = |damage: f32, radius: f32, d: f32| if d >= radius { 0.0 } else { damage * (1.0 - d / radius) };
    let reach = warhead.radius + targets.iter().filter_map(|t| t.blast).map(|b| b.radius).fold(0.0, f32::max);
    let near: Vec<usize> = (0..targets.len()).filter(|&i| targets[i].pos.dist2d(at) < reach).collect();
    let mut killed = vec![false; targets.len()];
    for &i in &near {
        killed[i] = falloff(warhead.damage, warhead.radius, targets[i].pos.dist2d(at)) > targets[i].health;
    }
    let first: Vec<usize> = near.iter().copied().filter(|i| killed[*i]).collect();
    for i in first {
        let Some(blast) = targets[i].blast else { continue };
        for &j in &near {
            if !killed[j] && falloff(blast.damage, blast.radius, targets[j].pos.dist2d(targets[i].pos)) > targets[j].health {
                killed[j] = true;
            }
        }
    }
    (0..targets.len()).filter(|i| killed[*i]).collect()
}

#[cfg(test)]
mod tests {
    use super::{blast_kills, deformation_off, warhead, Target, Warhead};
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

    /// The chain searched among the targets near the aim kills what it kills searched over every target: a field of
    /// 900 buildings on a jittered lattice, some with death blasts, struck at points across it.
    #[test]
    fn the_chain_searched_near_the_aim_kills_what_it_kills_searched_everywhere() {
        fn everywhere(at: Vec3, warhead: Warhead, targets: &[Target]) -> Vec<usize> {
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
        let mut rng = 0x2545_f491_4f6c_dd1du64;
        let mut next = || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            (rng >> 11) as f32 / (1u64 << 53) as f32
        };
        let mut targets = Vec::new();
        for k in 0..900 {
            let (x, z) = ((k % 30) as f32 * 120.0 + 80.0 * next(), (k / 30) as f32 * 120.0 + 80.0 * next());
            let blast = (next() < 0.3).then(|| Blast { radius: 100.0 + 500.0 * next(), damage: 500.0 + 6000.0 * next() });
            targets.push(Target { id: UnitId(k), pos: Vec3 { x, y: 0.0, z }, health: 200.0 + 4000.0 * next(), worth: 100.0, blast });
        }
        let warhead = Warhead { radius: 640.0, damage: 12500.0 };
        for k in 0..60 {
            let at = Vec3 { x: 3600.0 * next(), y: 0.0, z: 3600.0 * next() };
            let kills = blast_kills(at, warhead, &targets);
            assert_eq!(kills, everywhere(at, warhead, &targets), "strike {k}");
        }
    }

    #[test]
    fn the_lobby_option_for_a_flat_map_is_read_from_the_mod_options() {
        let on = "[GAME]\n{\n\t[modoptions]\n\t{\n\t\tdisablemapdamage=1;\n\t\tdeathmode=com;\n\t}\n}\n";
        let off = "[GAME]\n{\n\t[modoptions]\n\t{\n\t\tdisablemapdamage = 0;\n\t}\n}\n";
        assert!(deformation_off(on));
        assert!(!deformation_off(off));
        assert!(!deformation_off(""));
    }

    /// The silos' warheads by their files (Armada 640 and 12,500, Cortex 960 and 15,000); an anti-nuke fires its
    /// interceptors by itself, and the EMP launcher's missile paralyzes: neither is a warhead.
    #[test]
    fn a_warhead_is_a_stockpiled_command_fire_weapon_that_does_not_paralyze() {
        let table = combatsim::units::Units::default();
        let arm = warhead(&table, "armsilo").expect("Armada's silo");
        assert_eq!((arm.radius, arm.damage), (640.0, 12500.0));
        let cor = warhead(&table, "corsilo").expect("Cortex's silo");
        assert_eq!((cor.radius, cor.damage), (960.0, 15000.0));
        assert!(warhead(&table, "armamd").is_none());
        assert!(warhead(&table, "armemp").is_none());
    }
}
