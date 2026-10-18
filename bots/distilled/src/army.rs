//! The army: a home guard that answers raids, a strike body that gathers, advances on what it knows of his and
//! steps back from what outweighs it, and scouts (`README.md`, D-GUARD to D-SCOUTS). These rules are a first guess
//! over the verbs the flagship's hands picked most, not a fit to its picks.

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{Command, Tick, UnitDefId, UnitId, Vec3};

use crate::{Distilled, MINUTE, SECOND};

/// A mobile enemy this near a building of ours is a raid (D-GUARD).
const DEFEND: f32 = 900.0;
/// The home guard waits this far out from home toward the enemy.
const RALLY: f32 = 500.0;
/// The home guard becomes the strike body at this much metal of soldiers: five raiders early, a dozen tanks' worth
/// of a quarter later (D-LAUNCH).
const LAUNCH_EARLY: f32 = 550.0;
const LAUNCH_LATE: f32 = 1200.0;
const LATE_FROM: i32 = 8 * MINUTE;
/// With the strike body out, the home guard follows it at this much metal.
const REINFORCE: f32 = 400.0;
/// The body is together when this share of it stands within `BODY` of its middle (D-GATHER).
const BODY: f32 = 600.0;
const BODY_SHARE: f32 = 0.6;
/// What counts in a fight around the body, and by how much he must outweigh it to make it step back (D-STEP-BACK).
const FIGHT: f32 = 900.0;
const OUTWEIGHED: f32 = 1.3;
/// A turret counts three times its metal, as in the flagship's rules text.
const TURRET_WEIGHT: f32 = 3.0;
/// After being driven back the body pushes again only with this much more than what drove it back, the memory
/// halving every three minutes; until then it raids (D-RAID).
const AGAIN: f32 = 1.5;
const FORGET: i32 = 3 * MINUTE;
const REORDER: i32 = 5 * SECOND;
/// D-RAID: the body pushes only when it outweighs what it has seen of his army in the last two minutes by this
/// much; lighter, it takes his extractors and steps back from even odds, and a place where it met more than
/// itself is left alone for two minutes. A raid under this much metal (a scout at an outer extractor) neither holds
/// the launch nor counts against the field (D-FIELD).
const PUSH_ODDS: f32 = 1.5;
/// A raiding body fights only what it outweighs two to one within `FIGHT` (a lone sentry at an extractor, not his
/// army): the Opus player's raiders lost 7 soldiers by 10:00 against Apex, the bot's body 10-24 in every battery.
const RAID_ODDS: f32 = 0.5;
const SEEN_FOR: i32 = 2 * MINUTE;
const AVOID: f32 = 900.0;
const AVOID_FOR: i32 = 2 * MINUTE;
const RAID_WEIGHT: f32 = 100.0;

pub(crate) struct Seen {
    pub pos: Vec3,
    pub def: Option<UnitDefId>,
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
    /// Where each body was last sent, and when.
    strike_order: Option<(Vec3, i32)>,
    guard_order: Option<(Vec3, i32)>,
    /// The weight that last drove the body back, and when.
    repelled: (f32, i32),
    /// The frame of the last raid answered (D-GUARD).
    last_raid: i32,
    /// His armed mobile units as last seen: weight and frame (D-RAID).
    pub his_army: BTreeMap<UnitId, (f32, i32)>,
    /// Where the body met more than itself, and when (D-RAID).
    threats: Vec<(Vec3, i32)>,
}

impl Distilled {
    fn weight(&self, def: Option<UnitDefId>) -> f32 {
        // An enemy whose type is not known yet (a radar dot) counts as a raider.
        let Some(def) = def.and_then(|d| self.def(d)) else { return 110.0 };
        match (def.weapon_count > 0, def.speed > 0.0) {
            (false, _) => 0.0,
            (true, true) => def.metal_cost,
            (true, false) => TURRET_WEIGHT * def.metal_cost,
        }
    }

    fn weight_of(&self, units: &BTreeSet<UnitId>) -> f32 {
        units.iter().filter_map(|id| self.own.get(id)).map(|u| self.weight(Some(u.def))).sum()
    }

    /// The metal of soldiers standing at home (D-T2-PAUSE reads it: the plants pause only behind a guard).
    pub(crate) fn home_guard_weight(&self) -> f32 {
        self.weight_of(&self.army.home_guard)
    }

    pub(crate) fn strike_weight(&self) -> f32 {
        self.weight_of(&self.army.strike)
    }

    fn his_half(&self, at: Vec3) -> bool {
        at.dist2d(self.enemy_home) < at.dist2d(self.home)
    }

    /// What he has shown of his army in the last two minutes, in metal (D-RAID).
    pub(crate) fn his_army_weight(&self) -> f32 {
        self.army.his_army.values().map(|(w, _)| w).sum()
    }

    pub(crate) fn run_army(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        self.remember(tick);
        self.enlist(commands);
        self.run_scouts(commands);
        let raid = self.raid(tick);
        if let Some((at, weight)) = raid {
            if weight >= RAID_WEIGHT {
                self.army.last_raid = self.frame;
            }
            self.send(false, at, true, commands);
        }
        self.launch(raid.is_some_and(|(_, weight)| weight >= RAID_WEIGHT));
        self.run_strike(tick, commands);
    }

    /// D-FIELD: the field is ours while the strike body is out, nothing has driven it back for `FORGET`, and no raid
    /// has been answered for two minutes. The tier-2 gate waits for it (D-T2-GATE): the Opus player against Apex
    /// v0.2.9 went to tier 2 at 15:49, after its 10:25 push had won the field, where the bot went at 6:00-9:30 on
    /// income alone and paused its only plant while his army tripled (docs/studies/2026-10-18-distilled-apex-losses.md).
    pub(crate) fn field_won(&self) -> bool {
        // A raiding body is out because it is lighter than him, not because it has won (apex-v029-distilled-comet-7:
        // "won" from 5:00 in every game with the body raiding, and tier 2 at 6:08-7:34).
        !self.army.strike.is_empty() && !self.raiding(self.strike_weight()) && self.frame - self.army.repelled.1 >= FORGET && self.frame - self.army.last_raid >= 2 * MINUTE
    }

    /// D-LAUNCH's weight: what the home guard must weigh to go out now (the log line reads it beside the guard's
    /// weight). The base weight alone: what the body does out there is D-RAID's question, not whether it goes
    /// (apex-v029-distilled-comet-4: with the memory of what drove it back in the weight, 13-16 soldiers sat at home
    /// from 8:00 to 10:00 against a needed 2,500-4,000 while he took the map).
    pub(crate) fn launch_weight(&self) -> f32 {
        let scale = self.income_multiplier;
        let base = scale * if self.frame < LATE_FROM { LAUNCH_EARLY } else { LAUNCH_LATE };
        if self.army.strike.is_empty() { base } else { scale * REINFORCE }
    }

    /// D-RAID: a body of this weight raids rather than pushes: it is under `PUSH_ODDS` of what he has shown of his
    /// army, or under what drove it back lately.
    pub(crate) fn raiding(&self, ours: f32) -> bool {
        let (repelled, at) = self.army.repelled;
        let remembered = AGAIN * repelled * 0.5f32.powf((self.frame - at) as f32 / FORGET as f32);
        ours < PUSH_ODDS * self.his_army_weight() || ours < remembered
    }

    /// His buildings stay known after they leave sight, and are forgotten when one of ours stands there and sees
    /// none; his armed mobile units are remembered for two minutes from their last sighting (D-RAID).
    fn remember(&mut self, tick: &Tick) {
        for enemy in &tick.snapshot.enemies {
            let Some(def) = enemy.def.and_then(|d| self.def(d)) else { continue };
            if def.speed == 0.0 {
                self.army.his_buildings.insert(enemy.id, Seen { pos: enemy.pos, def: enemy.def });
            } else if def.weapon_count > 0 {
                self.army.his_army.insert(enemy.id, (def.metal_cost, self.frame));
            }
        }
        let frame = self.frame;
        self.army.his_army.retain(|_, (_, seen)| frame - *seen < SEEN_FOR);
        self.army.threats.retain(|(_, at)| frame - *at < AVOID_FOR);
        let in_sight: BTreeSet<UnitId> = tick.snapshot.enemies.iter().map(|e| e.id).collect();
        let soldiers: Vec<Vec3> = self.army.strike.iter().chain(self.army.scouts.keys()).filter_map(|id| self.own.get(id)).map(|u| u.pos).collect();
        self.army.his_buildings.retain(|id, seen| in_sight.contains(id) || !soldiers.iter().any(|p| p.dist2d(seen.pos) < 150.0));
    }

    /// A finished soldier joins the scouts or the home guard, and walks to the rally point.
    fn enlist(&mut self, commands: &mut Vec<Command>) {
        let alive = |id: &UnitId| self.own.contains_key(id);
        self.army.home_guard.retain(alive);
        self.army.strike.retain(alive);
        self.army.scouts.retain(|id, _| self.own.contains_key(id));
        let rally = self.toward_enemy(self.home, RALLY);
        let new: Vec<(UnitId, UnitDefId)> = self
            .own
            .values()
            .filter(|u| !u.being_built && self.def(u.def).is_some_and(|d| d.speed > 0.0 && d.weapon_count > 0 && d.build_options.is_empty()))
            .filter(|u| !self.army.home_guard.contains(&u.id) && !self.army.strike.contains(&u.id) && !self.army.scouts.contains_key(&u.id))
            .map(|u| (u.id, u.def))
            .collect();
        for (unit, def) in new {
            if Some(def) == self.kit.scout {
                self.army.scouts.insert(unit, 0);
            } else {
                self.army.home_guard.insert(unit);
                commands.push(Command::Fight { unit, to: rally, queue: false });
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

    /// D-GUARD: the mobile enemy within `DEFEND` of a building of ours that is nearest to home, and the weight of
    /// all of them.
    fn raid(&self, tick: &Tick) -> Option<(Vec3, f32)> {
        let buildings: Vec<Vec3> = self.own.values().filter(|u| self.def(u.def).is_some_and(|d| d.speed == 0.0)).map(|u| u.pos).collect();
        let raiders: Vec<(Vec3, f32)> = tick
            .snapshot
            .enemies
            .iter()
            .filter(|e| e.def.and_then(|d| self.def(d)).is_none_or(|d| d.speed > 0.0))
            .filter(|e| buildings.iter().any(|b| b.dist2d(e.pos) < DEFEND))
            .map(|e| (e.pos, self.weight(e.def)))
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

    /// D-LAUNCH: the home guard goes out as the strike body, or after it, once it is heavy enough and no raid is on.
    fn launch(&mut self, raid: bool) {
        if raid {
            return;
        }
        // The weights scale with the income bonus: at +50 % every side's army is half again as big, and a body
        // launched at the +0 weight met Apex's 40 soldiers at 10:00 with 24 (apex-v022-distilled-comet-sym50-2).
        if self.weight_of(&self.army.home_guard) >= self.launch_weight() {
            let guard = std::mem::take(&mut self.army.home_guard);
            self.army.strike.extend(guard);
            self.army.strike_order = None;
        }
    }

    /// D-GATHER, D-STEP-BACK, D-ADVANCE, D-RAID. A body lighter than what he has shown raids: it takes his known
    /// extractors nearest to it, steps back to the rally from even odds without going home, and leaves the places
    /// where it met more than itself alone for two minutes (the Opus player against Apex: Rovers and Blitzes on his
    /// extractors from 7:22, the one push at 10:25 with 43 against 13). A body heavier than that pushes as before.
    fn run_strike(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let members: Vec<(Vec3, UnitDefId)> = self.army.strike.iter().filter_map(|id| self.own.get(id)).map(|u| (u.pos, u.def)).collect();
        if members.is_empty() {
            return;
        }
        let n = members.len() as f32;
        let middle = Vec3 { x: members.iter().map(|(p, _)| p.x).sum::<f32>() / n, y: 0.0, z: members.iter().map(|(p, _)| p.z).sum::<f32>() / n };
        let whole: f32 = members.iter().map(|(_, def)| self.weight(Some(*def))).sum();
        let ours: f32 = members.iter().filter(|(p, _)| p.dist2d(middle) < FIGHT).map(|(_, def)| self.weight(Some(*def))).sum();
        let his: f32 = tick.snapshot.enemies.iter().filter(|e| e.pos.dist2d(middle) < FIGHT).map(|e| self.weight(e.def)).sum();
        let raiding = self.raiding(whole);
        let rally = self.toward_enemy(self.home, RALLY);
        if his > if raiding { RAID_ODDS } else { OUTWEIGHED } * ours {
            self.army.threats.push((middle, self.frame));
            self.army.strike_order = None;
            if raiding {
                // Step back toward the rally and go on from there; the body stays out.
                return self.send(true, rally, false, commands);
            }
            self.army.repelled = (his, self.frame);
            self.send(true, rally, false, commands);
            let strike = std::mem::take(&mut self.army.strike);
            self.army.home_guard.extend(strike);
            return;
        }
        let together = members.iter().filter(|(p, _)| p.dist2d(middle) < BODY).count() as f32 >= BODY_SHARE * n;
        if !together {
            return self.send(true, middle, true, commands);
        }
        let avoided = |at: Vec3| self.army.threats.iter().any(|(t, _)| t.dist2d(at) < AVOID);
        let nearest = |a: &Vec3, b: &Vec3| a.dist2d(middle).total_cmp(&b.dist2d(middle));
        // Raiding: the nearest known extractor of his, else the nearest spot on his half not looked at. Pushing: the
        // nearest building of his that is known, else the nearest enemy in sight anywhere, else that spot, else his
        // start; and with his start reached and nothing there, every spot is looked at again.
        let extractor = |def: Option<UnitDefId>| def.and_then(|d| self.def(d)).is_some_and(|d| d.extracts_metal > 0.0);
        let building = self.army.his_buildings.values().filter(|b| !avoided(b.pos) && (!raiding || extractor(b.def))).map(|b| b.pos).min_by(nearest);
        let in_sight = || (!raiding).then(|| tick.snapshot.enemies.iter().map(|e| e.pos).filter(|p| !avoided(*p)).min_by(nearest)).flatten();
        if self.unlooked(middle).is_none() && middle.dist2d(self.enemy_home) < 400.0 {
            self.army.looked.clear();
        }
        let target = building.or_else(in_sight).or_else(|| {
            let spot = self.unlooked(middle)?;
            let at = self.hello.metal_spots[spot];
            if at.dist2d(middle) < 300.0 {
                self.army.looked.insert(spot);
            }
            Some(at)
        });
        self.send(true, target.unwrap_or(if raiding { rally } else { self.enemy_home }), true, commands);
    }
}
