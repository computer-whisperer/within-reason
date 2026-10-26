//! The army: a guard that holds our frontier and answers raids, a strike body that pushes feasible targets as one
//! and retreats as one, and scouts (`README.md`, D-BODY to D-RETREAT and D-SCOUTS; the design
//! `docs/design/2026-10-20-distilled-army.md`). Rebuilt 2026-10-20 from the death and body traces of the day's
//! batteries: the earlier rules raided his half five raiders at a time, toggled between advance and step-back every
//! second, gathered on a mean pulled home by reinforcements, and pushed into turrets.

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{Command, Tick, UnitDefId, UnitId, Vec3};

use crate::{Distilled, MINUTE, SECOND};

/// D-HOLD: a mobile enemy this near a building of ours is a raid; a raid under `RAID_WEIGHT` (a scout) is let be.
const DEFEND: f32 = 900.0;
const RAID_WEIGHT: f32 = 80.0;
/// The hold point: the frontier extractor pulled back this far toward home, never nearer home than `HOLD_LEAST`;
/// the base point, where a guard lighter than a raid meets it, this far out from home.
const HOLD_BACK: f32 = 300.0;
const HOLD_LEAST: f32 = 500.0;
const BASE_OUT: f32 = 300.0;
/// With no raid for this long the guard returns to the hold point.
const RAID_OVER: i32 = 20 * SECOND;
/// D-LAUNCH: the guard goes out at this much metal of soldiers (times the income bonus), plus `LAUNCH_A_MINUTE` for
/// every minute past `LAUNCH_FROM` up to `LAUNCH_AT_MOST` (his army grows whether or not we have seen it: twelve
/// Blitzes launched at 5:56 toward his start with 686 metal of his seen met his commander and twenty soldiers,
/// dist7-army-barb-hardagg-comet-1 game 00), and `PUSH_ODDS` of what he has shown; a body out keeps its target down
/// to `KEEP_ODDS` (without the hysteresis the body launched, saw more of his army, found no target and merged back
/// every thirty seconds for four minutes, game 04). With the body out, the guard joins it at this share of the
/// body's weight, and at least `REINFORCE`.
const LAUNCH: f32 = 1200.0;
const LAUNCH_FROM: i32 = 5 * MINUTE;
const LAUNCH_A_MINUTE: f32 = 250.0;
const LAUNCH_AT_MOST: f32 = 4000.0;
const PUSH_ODDS: f32 = 1.5;
const KEEP_ODDS: f32 = 1.0;
const REINFORCE_SHARE: f32 = 0.34;
const REINFORCE: f32 = 600.0;
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
/// D-RAIDERS (2026-10-20): a group of up to `RAIDERS` raiders drawn from the guard once it holds `RAIDERS_FROM`
/// soldiers, sent at his known extractors with no known turret within `RAID_CLEAR` of them; it runs to the hold
/// point for `RAID_FLEE` and avoids the place when an armed ground enemy within `RAID_SIGHT` of its core weighs
/// `RAID_FLEE_SHARE` of it or a turret stands within `RAID_SIGHT`. The Opus player's raids held Apex to 13
/// extractors (apex-v029-comet-sym0-opus-1); without them he reaches 29-40 at 15:00 (dist10-army-apex-v032-comet-cold-1).
const RAIDERS: usize = 4;
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
enum Stance {
    Push,
    /// Walking to the hold point until this frame.
    Retreat(i32),
}

#[derive(Default)]
pub(crate) struct Army {
    pub home_guard: BTreeSet<UnitId>,
    pub strike: BTreeSet<UnitId>,
    /// Each scout and the frame of its last order.
    scouts: BTreeMap<UnitId, i32>,
    /// His buildings as last seen.
    pub his_buildings: BTreeMap<UnitId, Seen>,
    /// Metal spots a scout or the body has been sent to look at.
    looked: BTreeSet<usize>,
    /// Where each body's core was last sent, and when.
    strike_order: Option<(Vec3, i32)>,
    guard_order: Option<(Vec3, i32)>,
    /// The raider group (D-RAIDERS), its order, and until when it is running away.
    pub raiders: BTreeSet<UnitId>,
    raid_order: Option<(Vec3, i32)>,
    raid_fleeing: Option<i32>,
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
    /// His armed mobile ground units as last seen: weight and frame (his aircraft are `air.rs`'s, D-AIR-SEEN).
    pub his_army: BTreeMap<UnitId, (f32, i32)>,
    /// Where the body was outweighed, and when.
    threats: Vec<(Vec3, i32)>,
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

    /// Where the strike body's core was last sent, and when.
    pub(crate) fn strike_target(&self) -> Option<(Vec3, i32)> {
        self.strike_order
    }
}

impl Distilled {
    /// D-WEIGHT: a soldier its metal, a turret `turret_factor` times its metal, his commander `COMMANDER_WEIGHT`,
    /// an unknown contact a raider's.
    fn weight(&self, def: Option<UnitDefId>, turret_factor: f32) -> f32 {
        let Some(def) = def.and_then(|d| self.def(d)) else { return BLIP_WEIGHT };
        if def.weapon_count == 0 || (def.speed > 0.0 && def.move_class.is_none()) {
            // Unarmed, or an aircraft: nothing the body can fight (his air is `air.rs`'s question).
            0.0
        } else if def.speed == 0.0 {
            turret_factor * def.metal_cost
        } else if !def.build_options.is_empty() {
            COMMANDER_WEIGHT
        } else {
            def.metal_cost
        }
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

    /// What he has shown of his army in the last `SEEN_FOR`, in metal.
    pub(crate) fn his_army_weight(&self) -> f32 {
        self.army.his_army.values().map(|(w, _)| w).sum()
    }

    /// D-HOLD's hold point: the frontier extractor pulled back toward home.
    pub(crate) fn hold_point(&self) -> Vec3 {
        let frontier = self.finished(self.kit.extractor).map(|u| u.pos).min_by(|a, b| a.dist2d(self.enemy_home).total_cmp(&b.dist2d(self.enemy_home)));
        let least = self.toward_enemy(self.home, HOLD_LEAST);
        let Some(frontier) = frontier else { return least };
        let back = self.toward_point(frontier, self.home, HOLD_BACK);
        if back.dist2d(self.home) < HOLD_LEAST { least } else { back }
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
        self.remember(tick);
        self.enlist();
        self.run_scouts(commands);
        let raid = self.raid(tick).filter(|(_, weight)| *weight >= RAID_WEIGHT);
        if raid.is_some() {
            self.army.last_raid = self.frame;
        }
        self.run_guard(raid, commands);
        self.launch();
        self.run_strike(tick, raid, commands);
        self.run_raiders(tick, commands);
        self.army.field_since = if self.field_now() { self.army.field_since.or(Some(self.frame)) } else { None };
    }

    /// D-FIELD: the field is ours while the strike body is out and pushing beyond `FIELD_PROGRESS` of the way to his
    /// start, outweighs what he has shown by `PUSH_ODDS`, nothing has driven it back for `FORGET`, and no raid has
    /// been answered for two minutes. The tier-2 gate waits for it (D-T2-GATE); a body merely out counted as the
    /// field won at 10:00 in six of eight games of dist7-army-barb-hardagg-comet-1 while his army was twice ours.
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
            (REINFORCE_SHARE * self.strike_weight()).max(scale * REINFORCE)
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
            let Some(def) = enemy.def.and_then(|d| self.def(d)) else { continue };
            if def.speed == 0.0 {
                self.army.his_buildings.insert(enemy.id, Seen { pos: enemy.pos, def: enemy.def });
            } else if def.weapon_count > 0 && def.move_class.is_some() {
                let weight = self.weight(enemy.def, TURRET_WEIGHT);
                self.army.his_army.insert(enemy.id, (weight, self.frame));
            }
        }
        let frame = self.frame;
        self.army.his_army.retain(|_, (_, seen)| frame - *seen < SEEN_FOR);
        self.army.threats.retain(|(_, at)| frame - *at < AVOID_FOR);
        let in_sight: BTreeSet<UnitId> = tick.snapshot.enemies.iter().map(|e| e.id).collect();
        let soldiers: Vec<Vec3> = self.army.strike.iter().chain(self.army.home_guard.iter()).chain(self.army.scouts.keys()).filter_map(|id| self.own.get(id)).map(|u| u.pos).collect();
        self.army.his_buildings.retain(|id, seen| in_sight.contains(id) || !soldiers.iter().any(|p| p.dist2d(seen.pos) < 150.0));
    }

    /// A finished soldier joins the scouts or the guard (an aircraft is `air.rs`'s); the dead leave every body.
    fn enlist(&mut self) {
        let alive = |id: &UnitId| self.own.contains_key(id);
        self.army.home_guard.retain(alive);
        self.army.strike.retain(alive);
        self.army.raiders.retain(alive);
        self.army.scouts.retain(|id, _| self.own.contains_key(id));
        self.army.told.retain(|id, _| self.own.contains_key(id));
        let new: Vec<(UnitId, UnitDefId)> = self
            .own
            .values()
            .filter(|u| !u.being_built && self.def(u.def).is_some_and(|d| d.speed > 0.0 && d.move_class.is_some() && d.weapon_count > 0 && d.build_options.is_empty()))
            .filter(|u| !self.army.home_guard.contains(&u.id) && !self.army.strike.contains(&u.id) && !self.army.scouts.contains_key(&u.id) && !self.army.raiders.contains(&u.id))
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
        (0..spots.len()).filter(|i| self.his_half(spots[*i]) && !self.army.looked.contains(i)).min_by(|a, b| from.dist2d(spots[*a]).total_cmp(&from.dist2d(spots[*b])))
    }

    /// D-HOLD: the armed mobile ground enemy within `DEFEND` of a building of ours that is nearest to home, and the
    /// weight of all of them (an aircraft is not a raid the body can answer: 105 Brutes stood at home under Apex's
    /// bombers chasing them, dist8-army-apex-v032-comet-cold-1 game 02).
    fn raid(&self, tick: &Tick) -> Option<(Vec3, f32)> {
        let buildings: Vec<Vec3> = self.own.values().filter(|u| self.def(u.def).is_some_and(|d| d.speed == 0.0)).map(|u| u.pos).collect();
        let raiders: Vec<(Vec3, f32)> = tick
            .snapshot
            .enemies
            .iter()
            .filter(|e| e.def.and_then(|d| self.def(d)).is_none_or(|d| d.speed > 0.0 && d.weapon_count > 0 && d.move_class.is_some()))
            .filter(|e| buildings.iter().any(|b| b.dist2d(e.pos) < DEFEND))
            .map(|e| (e.pos, self.weight(e.def, TURRET_WEIGHT)))
            .collect();
        let weight: f32 = raiders.iter().map(|(_, w)| w).sum();
        raiders.into_iter().map(|(at, _)| at).min_by(|a, b| a.dist2d(self.home).total_cmp(&b.dist2d(self.home))).map(|at| (at, weight))
    }

    /// Orders a body to a place, again only when the place has moved or `REORDER` has passed.
    fn send(&mut self, strike: bool, to: Vec3, fight: bool, commands: &mut Vec<Command>) {
        let last = if strike { &mut self.army.strike_order } else { &mut self.army.guard_order };
        if last.is_some_and(|(at, frame)| at.dist2d(to) < 100.0 && self.frame - frame < REORDER) {
            return;
        }
        *last = Some((to, self.frame));
        let units = if strike { &self.army.strike } else { &self.army.home_guard };
        commands.extend(units.iter().map(|unit| if fight { Command::Fight { unit: *unit, to, queue: false } } else { Command::Move { unit: *unit, to, queue: false } }));
    }

    /// D-HOLD: the guard answers a raid it can match at the raid's place, meets a heavier one at the base, and
    /// otherwise stands at the hold point.
    fn run_guard(&mut self, raid: Option<(Vec3, f32)>, commands: &mut Vec<Command>) {
        if self.army.home_guard.is_empty() {
            return;
        }
        if let Some((at, weight)) = raid {
            let to = if self.home_guard_weight() >= weight { at } else { self.base_point() };
            return self.send(false, to, true, commands);
        }
        if self.frame - self.army.last_raid < RAID_OVER {
            return;
        }
        let hold = self.hold_point();
        self.send(false, hold, true, commands);
    }

    /// D-LAUNCH: the guard goes out as the strike body when it is heavy enough and has a feasible target, and
    /// follows a body that is out once it is a third of it.
    fn launch(&mut self) {
        // Not held by a raid: against BARb hard_aggressive a raid is on every twenty seconds, and the guard of 57
        // Stouts sat at home from 15:00 to 17:30 for it (dist8-army-barb-hardagg-comet-1 game 05).
        let guard = self.home_guard_weight();
        if guard < self.launch_weight() {
            return;
        }
        if self.army.strike.is_empty() {
            let from = self.hold_point();
            if self.target_for(&self.army.home_guard, from, PUSH_ODDS).is_none() {
                return;
            }
            self.army.stance = Some(Stance::Push);
        }
        // The reserve stays: the guard's members nearest to home up to `GUARD_RESERVE`.
        let mut by_home: Vec<(f32, UnitId, f32)> = self.army.home_guard.iter().filter_map(|id| self.own.get(id)).map(|u| (u.pos.dist2d(self.home), u.id, self.weight(Some(u.def), TURRET_WEIGHT))).collect();
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
        let going: BTreeSet<UnitId> = self.army.home_guard.difference(&staying).copied().collect();
        if going.is_empty() {
            return;
        }
        eprintln!("[distilled ai={}] f={} launch: guard n={} w{:.0} joins a body of n={} w{:.0}; {} stay", self.hello.ai_id, self.frame, going.len(), guard - kept, self.army.strike.len(), self.strike_weight(), staying.len());
        self.army.home_guard = staying;
        self.army.guard_order = None;
        self.army.strike.extend(going);
        self.army.strike_order = None;
    }

    /// D-TARGET: the feasible target nearest to `from` for a body of these units: a known building of his whose
    /// defence (his turrets within `TARGET_REACH` of it, weighted, plus what he has shown of his army) the body
    /// outweighs by `odds` (`PUSH_ODDS` to launch, `KEEP_ODDS` once out) and that is not a place lately avoided;
    /// with nothing of his known, the nearest spot on his half nobody has looked at, else his start.
    fn target_for(&self, units: &BTreeSet<UnitId>, from: Vec3, odds: f32) -> Option<Vec3> {
        let whole = self.weight_of(units);
        let factor = self.turret_factor(units);
        let avoided = |at: Vec3| self.army.threats.iter().any(|(t, _)| t.dist2d(at) < AVOID);
        let defence = |at: Vec3| self.army.his_buildings.values().filter(|b| b.pos.dist2d(at) < TARGET_REACH).map(|b| self.weight(b.def, factor)).sum::<f32>() + self.his_army_weight();
        let nearest = |a: &Vec3, b: &Vec3| a.dist2d(from).total_cmp(&b.dist2d(from));
        if !self.army.his_buildings.is_empty() {
            return self.army.his_buildings.values().map(|b| b.pos).filter(|p| !avoided(*p) && whole >= odds * defence(*p)).min_by(nearest);
        }
        if whole < odds * self.his_army_weight() {
            return None;
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
        let Some(target) = defend.or_else(|| self.target_for(&self.army.strike, core, KEEP_ODDS)) else {
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
        // The core's members carry the order; the stragglers close on the core.
        let moved = self.army.strike_order.is_none_or(|(at, frame)| at.dist2d(target) >= 100.0 || self.frame - frame >= REORDER);
        if moved {
            self.army.strike_order = Some((target, self.frame));
            commands.extend(together.iter().map(|unit| Command::Fight { unit: *unit, to: target, queue: false }));
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
        if !self.options.raiders {
            return;
        }
        // Fill from the guard once it is big enough, raiders first.
        if self.army.raiders.len() < RAIDERS && self.army.home_guard.len() >= RAIDERS_FROM {
            let spare: Vec<UnitId> = self.army.home_guard.iter().filter(|id| self.own.get(id).is_some_and(|u| Some(u.def) == self.kit.raider)).take(RAIDERS - self.army.raiders.len()).copied().collect();
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
            self.army.threats.push((core, self.frame));
            self.army.raid_fleeing = Some(self.frame + RAID_FLEE);
            self.army.raid_order = None;
            eprintln!("[distilled ai={}] f={} raiders flee: n={} w{:.0} from his w{:.0} turret={} at ({:.0},{:.0})", self.hello.ai_id, self.frame, members.len(), ours, his, u8::from(turret_near), core.x, core.z);
            return self.send_raiders(hold, false, commands);
        }
        let avoided = |at: Vec3| self.army.threats.iter().any(|(t, _)| t.dist2d(at) < AVOID);
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
