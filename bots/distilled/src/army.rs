//! The army: a guard that holds our frontier and answers raids, a strike body that pushes feasible targets as one
//! and retreats as one, and scouts (`README.md`, D-BODY to D-RETREAT and D-SCOUTS; the design
//! `docs/design/2026-10-20-distilled-army.md`). Rebuilt 2026-10-20 from the death and body traces of the day's
//! batteries: the earlier rules raided his half five raiders at a time, toggled between advance and step-back every
//! second, gathered on a mean pulled home by reinforcements, and pushed into turrets.

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{Command, Tick, UnitDefId, UnitId, Vec3};

use crate::{Distilled, MINUTE, SECOND};

/// D-HOLD: any armed mobile ground enemy this near a building of ours is a raid, whatever it weighs. Until
/// 2026-11-03 night a raid under 80 metal was "a scout" and let be, and a lone Pawn (54) or Grunt (43) took 104 of
/// the 161 things of ours that died before 10:00 across sixteen games against Apex (65 extractors, 20 builders;
/// dist95), 92 of 167 against BARb (dist96): one Pawn ate three extractors, a constructor and the radar in a minute
/// from 3:13 with the guard of three standing at the hold point (dist95 game 02). The floor at 40 halved the early
/// deaths against Apex (dist97, 7-0-9); the scout cars (21-31) that it still let be took 33 things before 10:00 in a
/// BARb row (dist98). The user: a slow responder still ends the damage, even one that cannot catch the raider.
const DEFEND: f32 = 900.0;
/// The hold point: the frontier extractor pulled back this far toward home, never nearer home than `HOLD_LEAST`;
/// the base point, where a guard lighter than a raid meets it, this far out from home.
const HOLD_BACK: f32 = 300.0;
const HOLD_LEAST: f32 = 500.0;
const BASE_OUT: f32 = 300.0;
/// D-PARTY (2026-11-06): each raid (his raiders within `RAID_CLUSTER` of each other) is answered by the guard's
/// members nearest to it up to `PARTY_ODDS` times its weight, and the rest of the guard stays at the hold point.
/// Until then the whole guard walked to the raider nearest home, wherever on the map our building stood, and came
/// back: dist137-column-smoke2 held 40 to 81 soldiers of 8,100 to 15,750 metal in the guard from 15:30 to 18:30
/// beside a body of 25 to 38, of which a launch found 13 to 41 gathered; and while the guard answered his Thugs in
/// the middle one Grunt took two constructors, three extractors and two turret frames in the north in eighty
/// seconds with nothing of ours within 1,700 (dist135-audit-apex 13:09-14:29).
const RAID_CLUSTER: f32 = 600.0;
const PARTY_ODDS: f32 = 2.0;
/// The hold point moves to a new frontier only this far from it, and no sooner than this after its last move: an
/// extractor begun or lost at the frontier moved the whole guard between places 500 to 1,000 apart.
const HOLD_MOVED: f32 = 400.0;
const HOLD_FOR: i32 = 20 * SECOND;
/// D-LAUNCH: the guard goes out at this much metal of soldiers (times the income bonus), plus `LAUNCH_A_MINUTE` for
/// every minute past `LAUNCH_FROM` up to `LAUNCH_AT_MOST` (his army grows whether or not we have seen it: twelve
/// Blitzes launched at 5:56 toward his start with 686 metal of his seen met his commander and twenty soldiers,
/// dist7-army-barb-hardagg-comet-1 game 00), and `PUSH_ODDS` of what he has shown; a body out keeps its target down
/// to `KEEP_ODDS` (without the hysteresis the body launched, saw more of his army, found no target and merged back
/// every thirty seconds for four minutes, game 04). With the body out, whatever of the guard is beyond the reserve
/// follows it as its column (D-COLUMN) from `REINFORCE` on, from wherever it stands: the plants are 85 seconds'
/// walk behind the hold point, and with the followers gathered there first and sent at a third of the body's
/// weight, 44 % of the soldiers' weight was the guard's while a body was out (dist140-cohesion-apex).
const LAUNCH: f32 = 1200.0;
const LAUNCH_FROM: i32 = 5 * MINUTE;
const LAUNCH_A_MINUTE: f32 = 250.0;
const LAUNCH_AT_MOST: f32 = 4000.0;
const PUSH_ODDS: f32 = 1.5;
const KEEP_ODDS: f32 = 1.0;
const REINFORCE: f32 = 600.0;
/// D-COLUMN (2026-11-06): a reinforcement is the body's once it stands within this of the body's core; until then
/// it is the column, walking there. Until then every joiner was a member the second it left the hold point: the
/// body's core fell on the empty ground between the head and the joiners, so the odds round the core read nobody of
/// ours (dist135-audit-apex 15:01: 55 soldiers of 10,710 "repelled" by 220 metal of his with none of ours within
/// 900 of the core, and the whole body walked back for twenty seconds), the target nearest that core changed every
/// ten seconds between the two sides of his base, and the head fought his turrets with a tenth of the body
/// (run/audit.py's cohesion table: 6 of 42 out with the head at 14:00). In the two rows before it the body was
/// under 60 % together on 53 % and 79 % of its trace lines, and 28 of 33 retreats were called with under 30 % of
/// the body within 900 of the core (dist132-econ2-apex, dist134-street-ground-barb).
const JOIN: f32 = 900.0;
/// Only the guard's members within this of the hold point, none of them out to a raid (D-PARTY), weigh for the
/// launch of a body and go out with it. A body sent back to the guard was the guard that second and the launch counted
/// it where it stood: dist136-column-smoke 14:15 "no target" for 18 soldiers at his base, 14:17 a launch of 37
/// with two of them together, the old body on its way back and the guard at the hold point in one formation.
const GATHERED: f32 = 900.0;
/// A body whose core has stood within `STOOD_AT` of its target for `STOOD_FOR` with nothing of his in sight there
/// has taken it: what was remembered within `STOOD_GONE` of the place is forgotten and a spot there is looked at.
/// A remembered building is otherwise forgotten only when a soldier stands within 150 of it and sees none, and a
/// kept target (D-TARGET) none of them reached held the body for good: dist140-cohesion-apex game 06, 325 soldiers
/// of 67,140 standing 250 from a place sent at 24:00 to the cap at 30:00, his commander and one extractor left
/// 2,000 away; game 01 the same with 333.
const STOOD_AT: f32 = 450.0;
const STOOD_FOR: i32 = 12 * SECOND;
const STOOD_GONE: f32 = 250.0;
/// D-KING (2026-11-06): the game ends when his commander dies (the arena's games carry the game's default end,
/// and every win of the map bench was that: dist157 game 01 won at 11:18 with his ten extractors, fourteen
/// constructors and eight turrets standing), and no rule aimed at him: D-TARGET took buildings, then spots nobody
/// had looked at, then his start. Seven of 26 timeouts against Apex on the map bench ended with his army at 0 to 9
/// and ours at 78 to 423, his commander alone on the map from 19:00 to the cap in one of them (dist149 game 05).
/// His commander where last seen within `KING_SEEN_FOR` is the body's target before any building, on D-TARGET's own
/// odds (his turrets within `TARGET_REACH` of him and what he has shown, of which he is 1,800).
const KING_SEEN_FOR: i32 = 2 * MINUTE;
/// His commander defends where he stands: his 1,800 counts in the defence of a target within this of where he was
/// last seen (his start while he has not been seen for `SEEN_FOR`), and in the odds of a fight he is in. Until
/// 2026-11-06 it counted in what he had shown of his army, so one sighting of him held every launch and every
/// target on the map, his farthest extractor included, to 2,700 more of ours for five minutes: what he had shown
/// bound the launch weight on 78 to 92 % of the minutes before the first body on the map bench, and without his
/// 1,800 in it the guard met the weight a median 1 to 9 minutes sooner on 19 of 22 rows (dist144-dist171, the
/// minute lines' `launch=` read back).
const KING_REACH: f32 = 1200.0;
/// D-STUCK (2026-11-06): a target the body's core has not come `STUCK_GAIN` nearer to over `STUCK_FOR`, with nothing
/// of his in the fight's reach, is given up for `UNREACHED_FOR`, as is everything within `STOOD_GONE` of it: the
/// body of dist149 game 05 stood 750 from its order point for eleven minutes, the formation reporting no legs.
const STUCK_FOR: i32 = 40 * SECOND;
const STUCK_GAIN: f32 = 100.0;
const UNREACHED_FOR: i32 = 5 * MINUTE;
/// A launch leaves this much metal of soldiers at home, the nearest to it (times the income bonus): with the whole
/// guard out, packs of one to eight Grunts killed 23 extractors and 16 builders of one game with nothing of ours
/// near (dist9-army-apex-v032-comet-cold-1 game 00). The body comes home only for a raid worth this share of
/// itself (the same game: a 4,400 body walked home for 258 metal of Grunts and back out).
const GUARD_RESERVE: f32 = 450.0;
const DEFEND_SHARE: f32 = 0.3;
/// D-BODY: the core is the mean of this share of the members nearest to the plain mean; a member farther than `BODY`
/// from the core is a straggler.
const CORE_SHARE: f32 = 0.6;
const BODY: f32 = 600.0;
/// Fight odds are read within this of the core (D-RETREAT): his weight over ours by `OUTWEIGHED` sends the body back
/// for `RETREAT` before it reads the odds again; the memory of what drove it back halves every `FORGET`.
const FIGHT: f32 = 900.0;
const OUTWEIGHED: f32 = 1.3;
const RETREAT: i32 = 20 * SECOND;
const FORGET: i32 = 3 * MINUTE;
/// D-WEIGHT: a turret counts this many times its metal, once only with artillery at `ARTILLERY_SHARE` of the body;
/// his commander counts this much (at 600 a body of twelve Blitzes pushed into it and lost six to the D-gun in two
/// seconds, dist7-army-barb-hardagg-comet-1 game 00; it is 3,700 health and a shot that kills a tier-1 unit a second).
const TURRET_WEIGHT: f32 = 3.0;
const ARTILLERY_SHARE: f32 = 0.2;
const COMMANDER_WEIGHT: f32 = 1800.0;
/// D-TARGET: a target's defence counts his turrets within this of it.
const TARGET_REACH: f32 = 700.0;
/// A place where the body was outweighed is avoided this long, this far round.
const AVOID: f32 = 900.0;
const AVOID_FOR: i32 = 2 * MINUTE;
/// His armed mobile units count as shown for this long after their last sighting (two minutes forgot the army that
/// had just driven the body back before the next launch), unless seen destroyed.
pub(crate) const SEEN_FOR: i32 = 5 * MINUTE;
/// D-FIELD: the field is ours when the body has pushed this far toward his start, outweighs what he has shown, and
/// has held that for this long (a field "won" the second the body launched started the advanced plant at 6:10-8:51
/// in six of eight games of dist7-army-apex-v032-comet-cold-1, and the plants paused for it while Apex had thirty
/// soldiers).
const FIELD_PROGRESS: f32 = 0.6;
const FIELD_HELD: i32 = 2 * MINUTE;
/// An order is repeated to a body no more often than this unless its place moved.
const REORDER: i32 = 5 * SECOND;
/// A radar contact's weight, as a raider's.
const BLIP_WEIGHT: f32 = 110.0;
/// D-RAIDERS (2026-10-20): a group of up to `raid=N` raiders (four by default) drawn from the guard once it holds `RAIDERS_FROM`
/// soldiers, sent at his known extractors with no known turret within `RAID_CLEAR` of them; it runs to the hold
/// point for `RAID_FLEE` and avoids the place when an armed ground enemy within `RAID_SIGHT` of its core weighs
/// `RAID_FLEE_SHARE` of it or a turret stands within `RAID_SIGHT`. The Opus player's raids held Apex to 13
/// extractors (apex-v029-comet-sym0-opus-1); without them he reaches 29-40 at 15:00 (dist10-army-apex-v032-comet-cold-1).
const RAIDERS_FROM: usize = 6;
const RAID_CLEAR: f32 = 600.0;
const RAID_SIGHT: f32 = 700.0;
const RAID_FLEE_SHARE: f32 = 0.5;
const RAID_FLEE: i32 = 15 * SECOND;
const RAID_REACHED: f32 = 250.0;

pub(crate) struct Seen {
    pub pos: Vec3,
    pub def: Option<UnitDefId>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Stance {
    Push,
    /// Walking to the hold point until this frame.
    Retreat(i32),
}

#[derive(Default)]
pub(crate) struct Army {
    pub home_guard: BTreeSet<UnitId>,
    pub strike: BTreeSet<UnitId>,
    /// Reinforcements walking to the strike body (D-COLUMN), and where they were last sent.
    pub column: BTreeSet<UnitId>,
    column_order: Option<(Vec3, i32)>,
    /// Each scout and the frame of its last order.
    scouts: BTreeMap<UnitId, i32>,
    /// His buildings as last seen.
    pub his_buildings: BTreeMap<UnitId, Seen>,
    /// Metal spots a scout or the body has been sent to look at.
    looked: BTreeSet<usize>,
    /// Where each body's core was last sent, and when; under `lane=on` the lane's formations walk there (D-LANE).
    pub(crate) strike_order: Option<(Vec3, i32)>,
    pub(crate) guard_order: Option<(Vec3, i32)>,
    /// The raider group (D-RAIDERS), its order, and until when it is running away.
    pub raiders: BTreeSet<UnitId>,
    raid_order: Option<(Vec3, i32)>,
    raid_fleeing: Option<i32>,
    /// The guard's members out answering a raid (D-PARTY): the raid's place and the frame of the order.
    responding: BTreeMap<UnitId, (Vec3, i32)>,
    /// The hold point in force, and since when (`HOLD_FOR`).
    hold: Option<(Vec3, i32)>,
    /// Per straggler, when it was last told to close on the core.
    told: BTreeMap<UnitId, i32>,
    stance: Option<Stance>,
    /// The weight that last drove the body back, and when.
    repelled: (f32, i32),
    /// The frame of the last raid answered (D-HOLD).
    last_raid: i32,
    /// How far the body's core has pushed toward his start, as a share of the way, and since when the field has
    /// been ours by every test but the time (D-FIELD).
    progress: f32,
    field_since: Option<i32>,
    /// The first thing of his of tier 2 seen, and when (D-T2-GATE, `gate=tech`).
    pub his_tier2: Option<(UnitDefId, i32)>,
    /// His armed mobile ground units as last seen: weight and frame (his aircraft are `air.rs`'s, D-AIR-SEEN).
    pub his_army: BTreeMap<UnitId, (f32, i32)>,
    /// Where the body was outweighed, and when.
    threats: Vec<(Vec3, i32)>,
    /// The target the body's core stands at, and since when (`STOOD_FOR`).
    stood: Option<(Vec3, i32)>,
    /// His commander where last seen, and when (D-KING).
    pub his_commander: Option<(Vec3, i32)>,
    /// The body's target, how far its core stood from it when the window began, and when that was (D-STUCK).
    closing: Option<(Vec3, f32, i32)>,
    /// Targets the body gave up as not reached, and when (D-STUCK).
    unreached: Vec<(Vec3, i32)>,
    /// Where the raiders ran, and when: theirs alone. Until 2026-11-06 a raider's scare went into `threats`, and four
    /// raiders running from 390 metal of his made every known building of his a place the strike body avoided:
    /// dist135-audit-apex 18:20, "no target" for 137 soldiers of 26,125 against his shown 3,558, and the body walked
    /// home (sixteen such walks home by a body over three times his shown army in dist132-econ2-apex's games).
    raid_threats: Vec<(Vec3, i32)>,
}

impl Army {
    pub(crate) fn is_scout(&self, unit: UnitId) -> bool {
        self.scouts.contains_key(&unit)
    }

    pub(crate) fn is_raider(&self, unit: UnitId) -> bool {
        self.raiders.contains(&unit)
    }

    /// An enemy seen destroyed is no longer shown, nor known as a building.
    pub(crate) fn forget(&mut self, enemy: UnitId) {
        self.his_buildings.remove(&enemy);
        self.his_army.remove(&enemy);
    }

    pub(crate) fn stance(&self) -> Option<Stance> {
        self.stance
    }

    /// Where the strike body's core was last sent, and when.
    pub(crate) fn strike_target(&self) -> Option<(Vec3, i32)> {
        self.strike_order
    }
}

impl Distilled {
    /// D-WEIGHT: a soldier its metal, a turret `turret_factor` times its metal (D-REACH: its weight from the
    /// combat tables instead), his commander `COMMANDER_WEIGHT`, an unknown contact a raider's.
    fn weight(&self, def: Option<UnitDefId>, turret_factor: f32) -> f32 {
        let Some(def) = def.and_then(|d| self.def(d)) else { return BLIP_WEIGHT };
        if def.weapon_count == 0 || (def.speed > 0.0 && def.move_class.is_none()) {
            // Unarmed, or an aircraft: nothing the body can fight (his air is `air.rs`'s question).
            0.0
        } else if def.speed == 0.0 {
            if self.options.reach
                && let Some(weight) = self.table_turret_weight(def, turret_factor)
            {
                return weight;
            }
            turret_factor * def.metal_cost
        } else if !def.build_options.is_empty() {
            COMMANDER_WEIGHT
        } else {
            def.metal_cost
        }
    }

    /// D-REACH (2026-11-03, the brief's reach article: a turret's weight by the square law is one to two times its
    /// metal, a Sentry 1.2 Blitzes or 0.8 Stout, a Gauntlet a fifth of its metal and a toll; "the old three times its
    /// metal is roughly a Sentry or Beamer against raiders arriving in a stream, and nothing else"): a turret's weight
    /// is the metal of our line tank matching its fighting strength, √(damage a second × health), plus the toll its
    /// reach over the tank's takes on the approach (the reach difference over the tank's speed, times the turret's
    /// damage a second, as the share of the tank's health it costs, in the tank's metal). No toll once the body
    /// carries artillery (`factor` 1.0: it outreaches the tier-1 turrets). `None` when the tables lack a row.
    fn table_turret_weight(&self, def: &bot_protocol::UnitDefInfo, factor: f32) -> Option<f32> {
        let row = |d: UnitDefId| self.control.rows.get(&d).map(|r| &self.control.rules.units.list[*r]);
        let turret = row(def.id)?;
        let reference = row(self.kit.tank.or(self.kit.raider)?)?;
        let strength = |u: &combatsim::units::Unit| (u.dps() * u.health).sqrt();
        let (ref_strength, ref_metal) = (strength(reference), reference.metal.max(1.0));
        if ref_strength <= 0.0 || reference.speed <= 0.0 {
            return None;
        }
        let weight = ref_metal * strength(turret) / ref_strength;
        let toll = if factor > 1.0 { (turret.reach() - reference.reach()).max(0.0) / reference.speed * turret.dps() / reference.health.max(1.0) * ref_metal } else { 0.0 };
        Some(weight + toll)
    }

    fn weight_of(&self, units: &BTreeSet<UnitId>) -> f32 {
        units.iter().filter_map(|id| self.own.get(id)).map(|u| self.weight(Some(u.def), TURRET_WEIGHT)).sum()
    }

    /// The metal of soldiers in the guard (D-T2-PAUSE reads it: the plants pause only behind a guard).
    pub(crate) fn home_guard_weight(&self) -> f32 {
        self.weight_of(&self.army.home_guard)
    }

    pub(crate) fn strike_weight(&self) -> f32 {
        self.weight_of(&self.army.strike)
    }

    /// The turret factor for a body: once with artillery at `ARTILLERY_SHARE` of its weight, else `TURRET_WEIGHT`.
    fn turret_factor(&self, units: &BTreeSet<UnitId>) -> f32 {
        let whole = self.weight_of(units);
        let artillery: f32 = units.iter().filter_map(|id| self.own.get(id)).filter(|u| Some(u.def) == self.kit.artillery || Some(u.def) == self.kit.t2_artillery).map(|u| self.weight(Some(u.def), 1.0)).sum();
        if whole > 0.0 && artillery >= ARTILLERY_SHARE * whole { 1.0 } else { TURRET_WEIGHT }
    }

    pub(crate) fn his_half(&self, at: Vec3) -> bool {
        at.dist2d(self.enemy_home) < at.dist2d(self.home)
    }

    /// What he has shown of his army in the last `SEEN_FOR`, in metal (his commander apart: `KING_REACH`).
    pub(crate) fn his_army_weight(&self) -> f32 {
        self.army.his_army.values().map(|(w, _)| w).sum()
    }

    /// D-HOLD's hold point: the frontier extractor pulled back toward home, as last settled (`settle_hold`).
    pub(crate) fn hold_point(&self) -> Vec3 {
        self.army.hold.map_or_else(|| self.frontier_hold(), |(at, _)| at)
    }

    fn frontier_hold(&self) -> Vec3 {
        let frontier = self.finished(self.kit.extractor).map(|u| u.pos).min_by(|a, b| a.dist2d(self.enemy_home).total_cmp(&b.dist2d(self.enemy_home)));
        let least = self.toward_enemy(self.home, HOLD_LEAST);
        let Some(frontier) = frontier else { return least };
        let back = self.toward_point(frontier, self.home, HOLD_BACK);
        if back.dist2d(self.home) < HOLD_LEAST { least } else { back }
    }

    /// The hold point follows the frontier in steps: only a frontier `HOLD_MOVED` from it moves it, and not within
    /// `HOLD_FOR` of its last move.
    fn settle_hold(&mut self) {
        let now = self.frontier_hold();
        if self.army.hold.is_none_or(|(at, since)| at.dist2d(now) >= HOLD_MOVED && self.frame - since >= HOLD_FOR) {
            self.army.hold = Some((now, self.frame));
        }
    }

    fn base_point(&self) -> Vec3 {
        self.toward_enemy(self.home, BASE_OUT)
    }

    /// A point `distance` from `from` toward `to` (no farther than `to`).
    pub(crate) fn toward_point(&self, from: Vec3, to: Vec3, distance: f32) -> Vec3 {
        let (dx, dz) = (to.x - from.x, to.z - from.z);
        let length = dx.hypot(dz).max(1.0);
        let step = distance.min(length);
        Vec3 { x: from.x + dx / length * step, y: from.y, z: from.z + dz / length * step }
    }

    pub(crate) fn run_army(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        // `army=off` (the econ bench): nothing is enlisted and nothing moves.
        if !self.options.army {
            return;
        }
        self.remember(tick);
        self.enlist();
        self.settle_hold();
        self.run_scouts(commands);
        let raids = self.raids(tick);
        // The raid nearest home with the weight of them all: what the strike body weighs coming home for (D-HOLD).
        let raid = raids.first().map(|(at, _)| (*at, raids.iter().map(|(_, w)| w).sum::<f32>()));
        if raid.is_some() {
            self.army.last_raid = self.frame;
        }
        self.run_guard(&raids, commands);
        self.launch();
        self.run_column(commands);
        self.run_strike(tick, raid, commands);
        self.run_raiders(tick, commands);
        self.army.field_since = if self.field_now() { self.army.field_since.or(Some(self.frame)) } else { None };
    }

    /// D-FIELD: the field is ours while the strike body is out and pushing beyond `FIELD_PROGRESS` of the way to his
    /// start, outweighs what he has shown by `PUSH_ODDS`, nothing has driven it back for `FORGET`, and no raid has
    /// been answered for two minutes. The tier-2 gate waits for it (D-T2-GATE); a body merely out counted as the
    /// field won at 10:00 in six of eight games of dist7-army-barb-hardagg-comet-1 while his army was twice ours.
    /// The strike body is out and pushing past the middle: the lead is being spent in his half (D-T2-GATE waits).
    pub(crate) fn pushing_in_his_half(&self) -> bool {
        !self.army.strike.is_empty() && matches!(self.army.stance, Some(Stance::Push)) && self.army.progress >= 0.5
    }

    pub(crate) fn field_won(&self) -> bool {
        self.army.field_since.is_some_and(|since| self.frame - since >= FIELD_HELD)
    }

    fn field_now(&self) -> bool {
        !self.army.strike.is_empty()
            && matches!(self.army.stance, Some(Stance::Push))
            && self.army.progress >= FIELD_PROGRESS
            && self.strike_weight() >= PUSH_ODDS * self.his_army_weight()
            && self.frame - self.army.repelled.1 >= FORGET
            && self.frame - self.army.last_raid >= 2 * MINUTE
    }

    /// D-LAUNCH's weight: what the guard must weigh to go out now, or to follow a body that is out.
    pub(crate) fn launch_weight(&self) -> f32 {
        let scale = self.income_multiplier;
        if self.army.strike.is_empty() {
            let by_clock = LAUNCH + LAUNCH_A_MINUTE * ((self.frame - LAUNCH_FROM).max(0) as f32 / MINUTE as f32);
            (scale * by_clock.min(LAUNCH_AT_MOST)).max(PUSH_ODDS * self.his_army_weight())
        } else {
            scale * REINFORCE
        }
    }

    /// The strike body's stance as a word, for the traces.
    pub(crate) fn stance_word(&self) -> &'static str {
        match self.army.stance {
            None => "home",
            Some(Stance::Push) => "push",
            Some(Stance::Retreat(_)) => "retreat",
        }
    }

    /// His buildings stay known after they leave sight, and are forgotten when one of ours stands there and sees
    /// none; his armed mobile ground units are remembered for `SEEN_FOR` from their last sighting.
    fn remember(&mut self, tick: &Tick) {
        for enemy in &tick.snapshot.enemies {
            if let Some(id) = enemy.def.filter(|d| self.army.his_tier2.is_none() && self.tier(*d) >= 2) {
                eprintln!("[distilled ai={}] f={} his tier 2 seen: {}", self.hello.ai_id, self.frame, self.def(id).map_or("?", |d| d.name.as_str()));
                self.army.his_tier2 = Some((id, self.frame));
            }
            let Some(def) = enemy.def.and_then(|d| self.def(d)) else { continue };
            if def.speed == 0.0 {
                self.army.his_buildings.insert(enemy.id, Seen { pos: enemy.pos, def: enemy.def });
            } else if def.weapon_count > 0 && def.move_class.is_some() {
                // An armed builder that walks is his commander (as D-WEIGHT reads it): he counts where he stands
                // (`KING_REACH`), not in what his army has shown.
                if !def.build_options.is_empty() {
                    self.army.his_commander = Some((enemy.pos, self.frame));
                    if self.options.king_local {
                        continue;
                    }
                }
                let weight = self.weight(enemy.def, TURRET_WEIGHT);
                self.army.his_army.insert(enemy.id, (weight, self.frame));
            }
        }
        let frame = self.frame;
        self.army.unreached.retain(|(_, at)| frame - *at < UNREACHED_FOR);
        self.army.his_army.retain(|_, (_, seen)| frame - *seen < SEEN_FOR);
        self.army.threats.retain(|(_, at)| frame - *at < AVOID_FOR);
        self.army.raid_threats.retain(|(_, at)| frame - *at < AVOID_FOR);
        let in_sight: BTreeSet<UnitId> = tick.snapshot.enemies.iter().map(|e| e.id).collect();
        let soldiers: Vec<Vec3> = self.army.strike.iter().chain(self.army.home_guard.iter()).chain(self.army.scouts.keys()).filter_map(|id| self.own.get(id)).map(|u| u.pos).collect();
        self.army.his_buildings.retain(|id, seen| in_sight.contains(id) || !soldiers.iter().any(|p| p.dist2d(seen.pos) < 150.0));
    }

    /// A finished soldier joins the scouts or the guard (an aircraft is `air.rs`'s); the dead leave every body.
    fn enlist(&mut self) {
        let alive = |id: &UnitId| self.own.contains_key(id);
        self.army.home_guard.retain(alive);
        self.army.strike.retain(alive);
        self.army.column.retain(alive);
        self.army.raiders.retain(alive);
        self.army.scouts.retain(|id, _| self.own.contains_key(id));
        self.army.told.retain(|id, _| self.own.contains_key(id));
        self.army.responding.retain(|id, _| self.own.contains_key(id) && self.army.home_guard.contains(id));
        let new: Vec<(UnitId, UnitDefId)> = self
            .own
            .values()
            .filter(|u| !u.being_built && self.def(u.def).is_some_and(|d| d.speed > 0.0 && d.move_class.is_some() && d.weapon_count > 0 && d.build_options.is_empty()))
            .filter(|u| !self.army.home_guard.contains(&u.id) && !self.army.strike.contains(&u.id) && !self.army.column.contains(&u.id) && !self.army.scouts.contains_key(&u.id) && !self.army.raiders.contains(&u.id))
            .map(|u| (u.id, u.def))
            .collect();
        for (unit, def) in new {
            if Some(def) == self.kit.scout {
                self.army.scouts.insert(unit, 0);
            } else {
                self.army.home_guard.insert(unit);
                // The guard's next order reaches it: a joiner is a straggler of the guard until then.
                self.army.guard_order = None;
            }
        }
    }

    /// D-SCOUTS: an idle scout drives to the spot on his half nearest to it that nobody has been sent to look at.
    fn run_scouts(&mut self, commands: &mut Vec<Command>) {
        let idle: Vec<(UnitId, Vec3)> = self.army.scouts.iter().filter(|(_, at)| self.frame - **at > 2 * SECOND).filter_map(|(id, _)| self.own.get(id)).filter(|u| u.idle).map(|u| (u.id, u.pos)).collect();
        for (unit, pos) in idle {
            let Some(spot) = self.unlooked(pos) else {
                // Everything has been looked at once: look again.
                self.army.looked.clear();
                continue;
            };
            self.army.looked.insert(spot);
            self.army.scouts.insert(unit, self.frame);
            commands.push(Command::Move { unit, to: self.hello.metal_spots[spot], queue: false });
        }
    }

    fn unlooked(&self, from: Vec3) -> Option<usize> {
        let spots = &self.hello.metal_spots;
        (0..spots.len()).filter(|i| self.his_half(spots[*i]) && !self.army.looked.contains(i) && self.army_reaches(spots[*i])).min_by(|a, b| from.dist2d(spots[*a]).total_cmp(&from.dist2d(spots[*b])))
    }

    /// D-HOLD: the raids: his armed mobile ground units within `DEFEND` of a building of ours, those within
    /// `RAID_CLUSTER` of each other one raid, each its middle and weight, the nearest to home first (an aircraft is
    /// not a raid the body can answer: 105 Brutes stood at home under Apex's bombers chasing them,
    /// dist8-army-apex-v032-comet-cold-1 game 02).
    fn raids(&self, tick: &Tick) -> Vec<(Vec3, f32)> {
        let buildings: Vec<Vec3> = self.own.values().filter(|u| self.def(u.def).is_some_and(|d| d.speed == 0.0)).map(|u| u.pos).collect();
        let mut raids: Vec<(Vec3, f32, f32)> = Vec::new();
        for enemy in &tick.snapshot.enemies {
            if !enemy.def.and_then(|d| self.def(d)).is_none_or(|d| d.speed > 0.0 && d.weapon_count > 0 && d.move_class.is_some()) || !buildings.iter().any(|b| b.dist2d(enemy.pos) < DEFEND) {
                continue;
            }
            let weight = self.weight(enemy.def, TURRET_WEIGHT);
            match raids.iter_mut().find(|(at, _, _)| at.dist2d(enemy.pos) < RAID_CLUSTER) {
                Some((at, total, n)) => {
                    *n += 1.0;
                    *at = Vec3 { x: at.x + (enemy.pos.x - at.x) / *n, y: at.y, z: at.z + (enemy.pos.z - at.z) / *n };
                    *total += weight;
                }
                None => raids.push((enemy.pos, weight, 1.0)),
            }
        }
        raids.sort_by(|a, b| a.0.dist2d(self.home).total_cmp(&b.0.dist2d(self.home)));
        raids.into_iter().map(|(at, weight, _)| (at, weight)).collect()
    }

    /// Orders a body to a place, again only when the place has moved or `REORDER` has passed.
    fn send(&mut self, strike: bool, to: Vec3, fight: bool, commands: &mut Vec<Command>) {
        let last = if strike { &mut self.army.strike_order } else { &mut self.army.guard_order };
        if last.is_some_and(|(at, frame)| at.dist2d(to) < 100.0 && self.frame - frame < REORDER) {
            return;
        }
        *last = Some((to, self.frame));
        // Under `lane=on` the strike body is a formation of the lane's and the order is its route (D-LANE, `lane.rs`).
        if strike && self.options.lane {
            return;
        }
        let units = if strike { &self.army.strike } else { &self.army.home_guard };
        commands.extend(units.iter().map(|unit| if fight { Command::Fight { unit: *unit, to, queue: false } } else { Command::Move { unit: *unit, to, queue: false } }));
    }

    /// D-HOLD, D-PARTY: each raid is answered at its place by the guard's members nearest to it up to `PARTY_ODDS`
    /// times its weight (a member already out to it before a nearer one); raids heavier together than the whole
    /// guard are met at the base by all of it; everyone else stands at the hold point.
    fn run_guard(&mut self, raids: &[(Vec3, f32)], commands: &mut Vec<Command>) {
        if self.army.home_guard.is_empty() {
            return;
        }
        let whole: f32 = raids.iter().map(|(_, w)| w).sum();
        if !raids.is_empty() && self.home_guard_weight() < whole {
            self.army.responding.clear();
            return self.send(false, self.base_point(), true, commands);
        }
        let mut free: Vec<(UnitId, Vec3, f32)> = self.army.home_guard.iter().filter_map(|id| self.own.get(id)).map(|u| (u.id, u.pos, self.weight(Some(u.def), TURRET_WEIGHT))).collect();
        let mut parties: BTreeMap<UnitId, Vec3> = BTreeMap::new();
        for (at, weight) in raids {
            // Those already out to this raid keep it: 400 nearer than they stand.
            let out = |id: &UnitId| self.army.responding.get(id).is_some_and(|(to, _)| to.dist2d(*at) < RAID_CLUSTER);
            free.sort_by(|a, b| (a.1.dist2d(*at) - if out(&a.0) { 400.0 } else { 0.0 }).total_cmp(&(b.1.dist2d(*at) - if out(&b.0) { 400.0 } else { 0.0 })));
            let mut sent = 0.0;
            while sent < PARTY_ODDS * weight && !free.is_empty() {
                let (id, _, w) = free.remove(0);
                sent += w;
                parties.insert(id, *at);
            }
        }
        self.army.responding.retain(|id, _| parties.contains_key(id));
        for (unit, at) in &parties {
            if self.army.responding.get(unit).is_none_or(|(to, frame)| to.dist2d(*at) >= 100.0 || self.frame - frame >= REORDER) {
                self.army.responding.insert(*unit, (*at, self.frame));
                commands.push(Command::Fight { unit: *unit, to: *at, queue: false });
            }
        }
        let hold = self.hold_point();
        if self.army.guard_order.is_some_and(|(at, frame)| at.dist2d(hold) < 100.0 && self.frame - frame < REORDER) {
            return;
        }
        self.army.guard_order = Some((hold, self.frame));
        commands.extend(free.iter().map(|(unit, _, _)| Command::Fight { unit: *unit, to: hold, queue: false }));
    }

    /// D-LAUNCH: the guard goes out as the strike body when it is heavy enough and has a feasible target, and
    /// follows a body that is out as its column (D-COLUMN).
    fn launch(&mut self) {
        // Not held by a raid: against BARb hard_aggressive a raid is on every twenty seconds, and the guard of 57
        // Stouts sat at home from 15:00 to 17:30 for it (dist8-army-barb-hardagg-comet-1 game 05).
        let hold = self.hold_point();
        let out = !self.army.strike.is_empty();
        let ready: BTreeSet<UnitId> = self.army.home_guard.iter().filter(|id| !self.army.responding.contains_key(id) && self.own.get(id).is_some_and(|u| out || u.pos.dist2d(hold) < GATHERED)).copied().collect();
        let guard = self.weight_of(&ready);
        if guard < self.launch_weight() {
            return;
        }
        if self.army.strike.is_empty() {
            if self.target_for(&ready, hold, PUSH_ODDS, None).is_none() {
                return;
            }
            self.army.stance = Some(Stance::Push);
        }
        // The reserve stays: the gathered members nearest to home up to `GUARD_RESERVE`.
        let mut by_home: Vec<(f32, UnitId, f32)> = ready.iter().filter_map(|id| self.own.get(id)).map(|u| (u.pos.dist2d(self.home), u.id, self.weight(Some(u.def), TURRET_WEIGHT))).collect();
        by_home.sort_by(|a, b| a.0.total_cmp(&b.0));
        let reserve = self.income_multiplier * GUARD_RESERVE;
        let (mut kept, mut staying) = (0.0, BTreeSet::new());
        for (_, id, weight) in &by_home {
            if kept >= reserve {
                break;
            }
            kept += weight;
            staying.insert(*id);
        }
        let going: BTreeSet<UnitId> = ready.difference(&staying).copied().collect();
        if going.is_empty() {
            return;
        }
        eprintln!("[distilled ai={}] f={} launch: guard n={} w{:.0} joins a body of n={} w{:.0}; {} stay{}", self.hello.ai_id, self.frame, going.len(), guard - kept, self.army.strike.len(), self.strike_weight(), staying.len(), if self.army.strike.is_empty() { "" } else { " (as its column)" });
        self.army.home_guard.retain(|id| !going.contains(id));
        self.army.guard_order = None;
        if self.army.strike.is_empty() {
            self.army.strike = going;
            self.army.strike_order = None;
        } else {
            self.army.column.extend(going);
            self.army.column_order = None;
        }
    }

    /// D-COLUMN: the reinforcements fight their way to the body's core and are the body's once within `JOIN` of
    /// it; with no body left to join they are the guard again.
    fn run_column(&mut self, commands: &mut Vec<Command>) {
        if self.army.column.is_empty() {
            return;
        }
        let members: Vec<(UnitId, Vec3, UnitDefId)> = self.army.strike.iter().filter_map(|id| self.own.get(id)).map(|u| (u.id, u.pos, u.def)).collect();
        if members.is_empty() {
            let column = std::mem::take(&mut self.army.column);
            self.army.home_guard.extend(column);
            self.army.guard_order = None;
            return;
        }
        let core = self.core(&members);
        let arrived: Vec<UnitId> = self.army.column.iter().filter(|id| self.own.get(id).is_some_and(|u| u.pos.dist2d(core) < JOIN)).copied().collect();
        if !arrived.is_empty() {
            eprintln!("[distilled ai={}] f={} column: n={} joins the body of n={} at ({:.0},{:.0}); {} still walking", self.hello.ai_id, self.frame, arrived.len(), members.len(), core.x, core.z, self.army.column.len() - arrived.len());
            for id in arrived {
                self.army.column.remove(&id);
                self.army.strike.insert(id);
            }
        }
        if self.army.column_order.is_some_and(|(at, frame)| at.dist2d(core) < 200.0 && self.frame - frame < REORDER) {
            return;
        }
        self.army.column_order = Some((core, self.frame));
        // Under `lane=on` the column is a formation of the lane's, as the body is (D-LANE).
        if !self.options.lane {
            commands.extend(self.army.column.iter().map(|unit| Command::Fight { unit: *unit, to: core, queue: false }));
        }
    }

    /// Where the column was last sent: the body's core then.
    pub(crate) fn column_target(&self) -> Option<Vec3> {
        self.army.column_order.map(|(at, _)| at)
    }

    /// D-TARGET: the feasible target for a body of these units. Feasible: a place the army can walk to (D-GROUND)
    /// that the body has not given up (D-STUCK) or lately been driven from, whose defence (his turrets within
    /// `TARGET_REACH` of it, weighted, plus what he has shown of his army) the body outweighs by `odds` (`PUSH_ODDS`
    /// to launch, `KEEP_ODDS` once out). First his commander where last seen (D-KING); then the known building of
    /// his nearest to `from`; with nothing of his known that can be walked to, the nearest spot on his half nobody
    /// has looked at, else his start.
    ///
    /// `kept` is the target the body is on: it stays the target while a building of his is known there and it is
    /// still feasible (2026-11-06: the nearest to a moving core is another building every few seconds, and the body
    /// of dist135-audit-apex was sent to twelve places in two minutes, one side of his base and the other).
    fn target_for(&self, units: &BTreeSet<UnitId>, from: Vec3, odds: f32, kept: Option<Vec3>) -> Option<Vec3> {
        let whole = self.weight_of(units);
        let factor = self.turret_factor(units);
        let avoided = |at: Vec3| self.army.threats.iter().any(|(t, _)| t.dist2d(at) < AVOID);
        let given_up = |at: Vec3| self.army.unreached.iter().any(|(t, _)| t.dist2d(at) < STOOD_GONE);
        let open = |at: Vec3| !given_up(at) && self.army_reaches(at);
        let king_at = self.army.his_commander.filter(|(_, seen)| self.frame - *seen < SEEN_FOR).map_or(self.enemy_home, |(at, _)| at);
        let defence = |at: Vec3| {
            let turrets: f32 = self.army.his_buildings.values().filter(|b| b.pos.dist2d(at) < TARGET_REACH).map(|b| self.weight(b.def, factor)).sum();
            turrets + self.his_army_weight() + if self.options.king_local && king_at.dist2d(at) < KING_REACH { COMMANDER_WEIGHT } else { 0.0 }
        };
        let nearest = |a: &Vec3, b: &Vec3| a.dist2d(from).total_cmp(&b.dist2d(from));
        let feasible = |p: &Vec3| open(*p) && !avoided(*p) && whole >= odds * defence(*p);
        if let Some(king) = self.army.his_commander.filter(|(_, seen)| self.options.king && self.frame - *seen < KING_SEEN_FOR).map(|(at, _)| at).filter(feasible) {
            return Some(king);
        }
        if self.army.his_buildings.values().any(|b| open(b.pos)) {
            if let Some(kept) = kept.filter(|k| self.army.his_buildings.values().any(|b| b.pos.dist2d(*k) < 50.0)).filter(feasible) {
                return Some(kept);
            }
            // D-REACH: "a post is a toll, not a wall; the economy behind it is the target": what his turrets cover
            // (his unarmed buildings) before the turrets themselves, which the body walks through to reach it.
            if self.options.reach {
                let unarmed = |b: &&Seen| b.def.and_then(|d| self.def(d)).is_some_and(|d| d.weapon_count == 0);
                if let Some(p) = self.army.his_buildings.values().filter(unarmed).map(|b| b.pos).filter(feasible).min_by(nearest) {
                    return Some(p);
                }
            }
            return self.army.his_buildings.values().map(|b| b.pos).filter(feasible).min_by(nearest);
        }
        if whole < odds * self.his_army_weight() {
            return None;
        }
        // A spot the body is walking to stays its target until it is looked at.
        if let Some(kept) = kept.filter(|k| !given_up(*k) && (0..self.hello.metal_spots.len()).any(|i| !self.army.looked.contains(&i) && self.his_half(self.hello.metal_spots[i]) && self.hello.metal_spots[i].dist2d(*k) < 50.0)) {
            return Some(kept);
        }
        Some(self.unlooked(from).map_or(self.enemy_home, |spot| self.hello.metal_spots[spot]))
    }

    /// D-BODY: the core of a body: the mean of the `CORE_SHARE` of its members nearest to the plain mean.
    fn core(&self, members: &[(UnitId, Vec3, UnitDefId)]) -> Vec3 {
        let n = members.len() as f32;
        let mean = Vec3 { x: members.iter().map(|(_, p, _)| p.x).sum::<f32>() / n, y: 0.0, z: members.iter().map(|(_, p, _)| p.z).sum::<f32>() / n };
        let mut by_distance: Vec<&(UnitId, Vec3, UnitDefId)> = members.iter().collect();
        by_distance.sort_by(|a, b| a.1.dist2d(mean).total_cmp(&b.1.dist2d(mean)));
        let keep = ((CORE_SHARE * n).ceil() as usize).clamp(1, members.len());
        let core = &by_distance[..keep];
        Vec3 { x: core.iter().map(|(_, p, _)| p.x).sum::<f32>() / keep as f32, y: 0.0, z: core.iter().map(|(_, p, _)| p.z).sum::<f32>() / keep as f32 }
    }

    /// D-BODY, D-TARGET, D-RETREAT: the strike body reads the odds round its core, retreats as one when outweighed,
    /// else fights its way to its target with its stragglers closing on the core.
    fn run_strike(&mut self, tick: &Tick, raid: Option<(Vec3, f32)>, commands: &mut Vec<Command>) {
        let members: Vec<(UnitId, Vec3, UnitDefId)> = self.army.strike.iter().filter_map(|id| self.own.get(id)).map(|u| (u.id, u.pos, u.def)).collect();
        if members.is_empty() {
            self.army.stance = None;
            return;
        }
        let core = self.core(&members);
        let span = self.home.dist2d(self.enemy_home).max(1.0);
        self.army.progress = ((core.x - self.home.x) * (self.enemy_home.x - self.home.x) + (core.z - self.home.z) * (self.enemy_home.z - self.home.z)) / (span * span);
        let factor = self.turret_factor(&self.army.strike);
        let whole: f32 = members.iter().map(|(_, _, def)| self.weight(Some(*def), factor)).sum();
        let together: Vec<UnitId> = members.iter().filter(|(_, p, _)| p.dist2d(core) < BODY).map(|(id, _, _)| *id).collect();
        let ours: f32 = members.iter().filter(|(_, p, _)| p.dist2d(core) < FIGHT).map(|(_, _, def)| self.weight(Some(*def), factor)).sum();
        let his: f32 = tick.snapshot.enemies.iter().filter(|e| !e.being_built && e.pos.dist2d(core) < FIGHT).map(|e| self.weight(e.def, factor)).sum();
        let hold = self.hold_point();
        let outweighed = his > OUTWEIGHED * ours;
        let retreating = matches!(self.army.stance, Some(Stance::Retreat(until)) if self.frame < until);
        let what = if retreating {
            "retreat"
        } else if outweighed {
            "outweighed"
        } else if (together.len() as f32) < CORE_SHARE * members.len() as f32 {
            "gather"
        } else {
            "advance"
        };
        // The body trace (2026-10-20): every ten seconds, where the body is and what it is doing, for run/bot_body.py.
        if self.frame % (10 * SECOND) == 0 {
            eprintln!(
                "[distilled ai={}] f={} body: n={} w{:.0} middle=({:.0},{:.0}) together={}/{} near: ours w{:.0} his w{:.0} mode={} what={what} order={} guard n={} w{:.0} his_seen w{:.0} repelled w{:.0}",
                self.hello.ai_id, self.frame, members.len(), whole, core.x, core.z, together.len(), members.len(), ours, his, self.stance_word(),
                self.army.strike_order.map_or("none".to_string(), |(p, _)| format!("({:.0},{:.0})", p.x, p.z)), self.army.home_guard.len(), self.home_guard_weight(), self.his_army_weight(), self.army.repelled.0 * 0.5f32.powf((self.frame - self.army.repelled.1) as f32 / FORGET as f32)
            );
            // Under `lane=on`, what the lane's formation of it reports (D-LANE).
            if let Some(state) = self.control.lane.group_state(crate::lane::STRIKE) {
                eprintln!(
                    "[distilled ai={}] f={} formation: strike anchor=({:.0},{:.0}) pace={:.0}/{:.0} formed={}/{} exceptions={} arrived={} engaged={} legs={} companies={}",
                    self.hello.ai_id, self.frame, state.anchor.x, state.anchor.z, state.pace, state.full_pace, state.formed.len(), members.len(), state.exceptions.len(), u8::from(state.arrived), u8::from(state.engaged.is_some()), state.legs_left, state.companies.len()
                );
            }
        }
        if retreating || outweighed {
            // A body driven back begins its walk to the target anew (D-STUCK).
            self.army.closing = None;
        }
        if retreating {
            return self.send(true, hold, false, commands);
        }
        if outweighed {
            self.army.threats.push((core, self.frame));
            self.army.repelled = (his, self.frame);
            self.army.stance = Some(Stance::Retreat(self.frame + RETREAT));
            self.army.strike_order = None;
            eprintln!("[distilled ai={}] f={} repelled: n={} w{:.0} (near w{:.0}) by his w{:.0} at ({:.0},{:.0})", self.hello.ai_id, self.frame, members.len(), whole, ours, his, core.x, core.z);
            return self.send(true, hold, false, commands);
        }
        self.army.stance = Some(Stance::Push);
        if self.unlooked(core).is_none() && core.dist2d(self.enemy_home) < 400.0 {
            self.army.looked.clear();
        }
        // D-HOLD: a raid in our half the guard cannot match, worth `DEFEND_SHARE` of the body, is the body's business
        // before its target.
        let defend = raid.filter(|(at, weight)| self.home_guard_weight() < *weight && *weight >= DEFEND_SHARE * whole && !self.his_half(*at)).map(|(at, _)| at);
        let kept = self.army.strike_order.map(|(at, _)| at);
        // A target is kept on the weight of the body and its column together: what is on its way counts for whether
        // to stay out, as it did while every follower was a member (the odds of the fight at hand, above, are the
        // body's alone). On the body alone the walks home for want of a target rose from 22 to 36 a row against BARb
        // hard_aggressive, each with the body at his shown army's weight and the column behind it uncounted, and
        // the wins came two minutes later (dist143-stream-barb against dist139-base-barb).
        let committed: BTreeSet<UnitId> = self.army.strike.union(&self.army.column).copied().collect();
        let Some(target) = defend.or_else(|| self.target_for(&committed, core, KEEP_ODDS, kept)) else {
            // Nothing the body can take: it goes back to the hold point and is the guard again.
            eprintln!("[distilled ai={}] f={} no target: n={} w{:.0} against his shown w{:.0}; back to the guard", self.hello.ai_id, self.frame, members.len(), whole, self.his_army_weight());
            self.send(true, hold, true, commands);
            let strike = std::mem::take(&mut self.army.strike);
            self.army.home_guard.extend(strike);
            self.army.guard_order = None;
            self.army.stance = None;
            return;
        };
        // A spot the core stands on is looked at.
        if let Some(spot) = self.unlooked(core).filter(|s| self.hello.metal_spots[*s].dist2d(core) < 300.0) {
            self.army.looked.insert(spot);
        }
        // A target the core has stood at with nothing of his in sight is taken (`STOOD_FOR`).
        let nothing_there = !tick.snapshot.enemies.iter().any(|e| e.pos.dist2d(target) < STOOD_AT);
        if core.dist2d(target) < STOOD_AT && nothing_there {
            let since = self.army.stood.filter(|(at, _)| at.dist2d(target) < 50.0).map_or(self.frame, |(_, since)| since);
            self.army.stood = Some((target, since));
            if self.frame - since >= STOOD_FOR {
                self.army.his_buildings.retain(|_, seen| seen.pos.dist2d(target) >= STOOD_GONE);
                if self.army.his_commander.is_some_and(|(at, _)| at.dist2d(target) < STOOD_GONE) {
                    self.army.his_commander = None;
                }
                let spots: Vec<usize> = (0..self.hello.metal_spots.len()).filter(|i| self.hello.metal_spots[*i].dist2d(target) < STOOD_GONE).collect();
                self.army.looked.extend(spots);
                self.army.stood = None;
            }
        } else {
            self.army.stood = None;
        }
        // D-STUCK: a target the core has come no nearer to over a window of `STUCK_FOR`, with nothing of his at hand,
        // is given up. The window begins again while the body fights or stands at the target (D-TARGET's own release).
        let gap = core.dist2d(target);
        match self.army.closing {
            Some((at, _, _)) if at.dist2d(target) < 100.0 && (his > 0.0 || gap < STOOD_AT) => self.army.closing = Some((target, gap, self.frame)),
            Some((at, from, since)) if at.dist2d(target) < 100.0 => {
                if self.frame - since >= STUCK_FOR {
                    if self.options.ground && gap > from - STUCK_GAIN {
                        eprintln!("[distilled ai={}] f={} gave up: n={} w{:.0} from {:.0} to {:.0} of ({:.0},{:.0}) in {} s", self.hello.ai_id, self.frame, members.len(), whole, from, gap, target.x, target.z, (self.frame - since) / SECOND);
                        self.army.unreached.push((target, self.frame));
                        if self.army.his_commander.is_some_and(|(at, _)| at.dist2d(target) < STOOD_GONE) {
                            self.army.his_commander = None;
                        }
                        let spots: Vec<usize> = (0..self.hello.metal_spots.len()).filter(|i| self.hello.metal_spots[*i].dist2d(target) < STOOD_GONE).collect();
                        self.army.looked.extend(spots);
                        self.army.closing = None;
                    } else {
                        self.army.closing = Some((target, gap, self.frame));
                    }
                }
            }
            _ => self.army.closing = Some((target, gap, self.frame)),
        }
        // The core's members carry the order; the stragglers close on the core.
        let moved = self.army.strike_order.is_none_or(|(at, frame)| at.dist2d(target) >= 100.0 || self.frame - frame >= REORDER);
        if moved {
            self.army.strike_order = Some((target, self.frame));
            if !self.options.lane {
                commands.extend(together.iter().map(|unit| Command::Fight { unit: *unit, to: target, queue: false }));
            }
        }
        // Under `lane=on` the formation's slots gather the stragglers (D-LANE).
        if self.options.lane {
            return;
        }
        let together: BTreeSet<UnitId> = together.into_iter().collect();
        for (unit, _, _) in members.iter().filter(|(id, _, _)| !together.contains(id)) {
            if self.army.told.get(unit).is_none_or(|at| self.frame - at >= REORDER) {
                self.army.told.insert(*unit, self.frame);
                commands.push(Command::Move { unit: *unit, to: core, queue: false });
            }
        }
    }

    /// D-RAIDERS: the raider group is filled from the guard's raiders, runs from what it cannot beat, and otherwise
    /// fights to his known extractor nearest to it that has no known turret near, else to a spot on his half nobody
    /// has looked at. Its deaths carry `body=raiders` in the death instrument.
    fn run_raiders(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        if self.options.raiders == 0 {
            return;
        }
        // Fill from the guard once it is big enough, raiders first.
        let wanted = self.options.raiders;
        if self.army.raiders.len() < wanted && self.army.home_guard.len() >= RAIDERS_FROM {
            let spare: Vec<UnitId> = self.army.home_guard.iter().filter(|id| self.own.get(id).is_some_and(|u| Some(u.def) == self.kit.raider)).take(wanted - self.army.raiders.len()).copied().collect();
            for id in spare {
                self.army.home_guard.remove(&id);
                self.army.raiders.insert(id);
                self.army.raid_order = None;
            }
        }
        let members: Vec<(UnitId, Vec3, UnitDefId)> = self.army.raiders.iter().filter_map(|id| self.own.get(id)).map(|u| (u.id, u.pos, u.def)).collect();
        if members.is_empty() {
            return;
        }
        let core = self.core(&members);
        let ours: f32 = members.iter().map(|(_, _, def)| self.weight(Some(*def), 1.0)).sum();
        let hold = self.hold_point();
        let turret_near = tick.snapshot.enemies.iter().any(|e| !e.being_built && e.pos.dist2d(core) < RAID_SIGHT && e.def.and_then(|d| self.def(d)).is_some_and(|d| d.speed == 0.0 && d.weapon_count > 0));
        let his: f32 = tick.snapshot.enemies.iter().filter(|e| !e.being_built && e.pos.dist2d(core) < RAID_SIGHT).map(|e| self.weight(e.def, 1.0)).sum();
        if let Some(until) = self.army.raid_fleeing {
            if self.frame < until {
                return self.send_raiders(hold, false, commands);
            }
            self.army.raid_fleeing = None;
        }
        if turret_near || his > RAID_FLEE_SHARE * ours {
            self.army.raid_threats.push((core, self.frame));
            self.army.raid_fleeing = Some(self.frame + RAID_FLEE);
            self.army.raid_order = None;
            eprintln!("[distilled ai={}] f={} raiders flee: n={} w{:.0} from his w{:.0} turret={} at ({:.0},{:.0})", self.hello.ai_id, self.frame, members.len(), ours, his, u8::from(turret_near), core.x, core.z);
            return self.send_raiders(hold, false, commands);
        }
        let avoided = |at: Vec3| self.army.raid_threats.iter().any(|(t, _)| t.dist2d(at) < AVOID);
        let extractor = |def: Option<UnitDefId>| def.and_then(|d| self.def(d)).is_some_and(|d| d.extracts_metal > 0.0);
        let turret_by = |at: Vec3| self.army.his_buildings.values().any(|b| b.pos.dist2d(at) < RAID_CLEAR && b.def.and_then(|d| self.def(d)).is_some_and(|d| d.weapon_count > 0));
        let nearest = |a: &Vec3, b: &Vec3| a.dist2d(core).total_cmp(&b.dist2d(core));
        let mut target = self.army.his_buildings.values().filter(|b| extractor(b.def) && !turret_by(b.pos) && !avoided(b.pos)).map(|b| b.pos).min_by(nearest);
        if target.is_none()
            && let Some(spot) = self.unlooked(core)
        {
            let at = self.hello.metal_spots[spot];
            if at.dist2d(core) < RAID_REACHED {
                self.army.looked.insert(spot);
            }
            if !avoided(at) {
                target = Some(at);
            }
        }
        let Some(target) = target else { return self.send_raiders(hold, true, commands) };
        self.send_raiders(target, true, commands);
    }

    fn send_raiders(&mut self, to: Vec3, fight: bool, commands: &mut Vec<Command>) {
        if self.army.raid_order.is_some_and(|(at, frame)| at.dist2d(to) < 100.0 && self.frame - frame < REORDER) {
            return;
        }
        self.army.raid_order = Some((to, self.frame));
        commands.extend(self.army.raiders.iter().map(|unit| if fight { Command::Fight { unit: *unit, to, queue: false } } else { Command::Move { unit: *unit, to, queue: false } }));
    }
}
