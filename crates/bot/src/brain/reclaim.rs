//! Wrecks: metal lying on the ground, and units that can be raised again.
//!
//! H-REC-FIELDS: wrecks in sight are remembered and grouped into fields; a field is safe with no enemy soldier in sight
//! near it and a way to walk there. H-REC-CREW: resurrection bots are built in proportion to the
//! metal lying in safe fields and work the richest field for the walk. H-REC-RESURRECT: a wreck of a soldier worth
//! having is raised when energy is plentiful; everything else is taken apart. Constructors short of metal go to the
//! same fields (H-ECO-RECLAIM) and, while we have a resurrection bot, leave it the wrecks worth raising.
//!
//! His units: a builder takes one apart in its health share times its build time over the builder's reclaim speed, and
//! we get its whole cost (K-mech-reclaim-values). What comes apart within `TAKE_APART_SECONDS` near a builder is a
//! constructor's menu move (H-HANDS-TAKE-APART-HIS) and a resurrection bot's first job (H-REC-TAKE-APART).

use std::collections::HashMap;

use bot_protocol::{Command, EnemyUnit, FeatureId, OwnUnit, Tick, UnitId, Vec3, Wreck};

use super::roster::Kit;
use super::{Brain, FRAMES_PER_SECOND};

/// Wrecks this close together are one field.
pub(super) const FIELD_RADIUS: f32 = 400.0;
/// A field poorer than this is not worth a walk.
const FIELD_MIN_METAL: f32 = 100.0;
/// An unseen wreck is believed in for this long.
const MEMORY_FRAMES: i32 = 2 * 60 * FRAMES_PER_SECOND;
/// A wreck we stand this close to and are not told of is gone.
const IN_PLAIN_SIGHT: f32 = 250.0;
const ENEMY_NEAR: f32 = 700.0;
/// A field takes one worker per this much metal (at least one).
const METAL_PER_WORKER: f32 = 300.0;
/// Wrecks queued in one order.
const QUEUE: usize = 5;
/// H-REC-RESURRECT: soldiers worth at least this much metal, with stored energy above this share of storage. Raising
/// costs energy and no metal; a test on banked metal stood here and failed whenever the metal was being spent well
/// (the commander games, 2026-09-20: 869 wrecks taken apart for 89 raised).
const RAISE_MIN_METAL: f32 = 100.0;
const RAISE_ENERGY: f32 = 0.5;
/// His units and frames a builder is offered, or a resurrection bot takes, come apart within this at its reclaim speed
/// (the user, 2026-10-27: "only units and frames with health remaining that should take a short enough time period --
/// 10s should do it"): a construction bot's Pawn at a quarter health (5 s), not a whole one (21 s, while the Pawn
/// kills it in 8).
pub(super) const TAKE_APART_SECONDS: f32 = 10.0;
/// ... and stand this near the builder.
pub(super) const TAKE_APART_REACH: f32 = 600.0;

pub struct WreckField {
    pub at: Vec3,
    pub metal: f32,
    pub safe: bool,
    pub(super) wrecks: Vec<FeatureId>,
}

#[derive(Default)]
pub struct Reclaim {
    wrecks: HashMap<FeatureId, (Wreck, i32)>,
    pub fields: Vec<WreckField>,
    /// Who is working which field (by its place), and since when.
    workers: HashMap<UnitId, (Vec3, i32)>,
    /// Units just raised and the bot that raised them: a raised unit arrives with a twentieth of its health (rec-2:
    /// 48-102 of 755-1890), and the bot that raised it mends it before anything else.
    pub(super) to_mend: Vec<(UnitId, UnitId)>,
    /// Resurrection bots we have.
    crew: usize,
    /// H-REC-TAKE-APART: the unit of his each resurrection bot is taking apart.
    taking: HashMap<UnitId, UnitId>,
}

/// One of his a builder can take apart: the seconds it takes and the metal it pays.
pub(super) struct Takeable<'a> {
    pub enemy: &'a EnemyUnit,
    pub seconds: f32,
    pub metal: f32,
}

impl Brain {
    pub(super) fn track_wrecks(&mut self, tick: &Tick) {
        // What a resurrection gives us, for the log: the unit and the health it arrives with.
        for event in &tick.events {
            let bot_protocol::Event::UnitCreated { unit, builder: Some(builder) } = event else { continue };
            let raised_by_crew = self.known_units.get(builder).is_some_and(|(def, _)| self.kit.is_some_and(|kit| kit.is_resurrector(*def)));
            if let Some(raised) = tick.snapshot.own_units.iter().find(|u| u.id == *unit).filter(|_| raised_by_crew) {
                eprintln!("[ai {}] f={} raised {} at {:.0} of {:.0} health", self.ai(), tick.frame, self.name(raised.def), raised.health, raised.max_health);
                self.reclaim.to_mend.push((*builder, *unit));
            }
        }
        self.reclaim.crew = tick.snapshot.own_units.iter().filter(|u| !u.being_built && self.kit.is_some_and(|kit| kit.is_resurrector(u.def))).count();
        let Some(seen) = &tick.snapshot.wrecks else { return };
        let listed: std::collections::HashSet<FeatureId> = seen.iter().map(|w| w.id).collect();
        for wreck in seen {
            self.reclaim.wrecks.insert(wreck.id, (wreck.clone(), tick.frame));
        }
        let own = &tick.snapshot.own_units;
        self.reclaim.wrecks.retain(|id, (wreck, last_seen)| {
            let looked_at = own.iter().any(|u| u.pos.dist2d(wreck.pos) < IN_PLAIN_SIGHT);
            listed.contains(id) || (!looked_at && tick.frame - *last_seen < MEMORY_FRAMES)
        });
        // Fields: the richest wreck not yet in one, with everything near it.
        let mut left: Vec<&Wreck> = self.reclaim.wrecks.values().map(|(w, _)| w).collect();
        left.sort_by(|a, b| b.metal.total_cmp(&a.metal).then(a.id.cmp(&b.id)));
        let mut fields = Vec::new();
        while let Some(seed) = left.first().copied() {
            let (inside, outside): (Vec<&Wreck>, Vec<&Wreck>) = left.into_iter().partition(|w| w.pos.dist2d(seed.pos) < FIELD_RADIUS);
            left = outside;
            let metal: f32 = inside.iter().map(|w| w.metal).sum();
            if metal < FIELD_MIN_METAL {
                continue;
            }
            let at = inside.iter().fold(Vec3::default(), |sum, w| Vec3 { x: sum.x + w.pos.x * w.metal / metal, y: 0.0, z: sum.z + w.pos.z * w.metal / metal });
            let enemy_near = tick.snapshot.enemies.iter().any(|e| e.pos.dist2d(at) < ENEMY_NEAR);
            let safe = !enemy_near && self.reachable_on_foot(at);
            fields.push(WreckField { at, metal, safe, wrecks: inside.iter().map(|w| w.id).collect() });
        }
        self.reclaim.fields = fields;
        if tick.due() % (60 * FRAMES_PER_SECOND) < 3 * FRAMES_PER_SECOND {
            let safe: f32 = self.reclaim.fields.iter().filter(|f| f.safe).map(|f| f.metal).sum();
            let all: f32 = self.reclaim.fields.iter().map(|f| f.metal).sum();
            eprintln!(
                "[ai {}] f={} wrecks: {} known in {} fields, {all:.0} metal ({safe:.0} safe to work), {} at work",
                self.ai(), tick.frame, self.reclaim.wrecks.len(), self.reclaim.fields.len(), self.reclaim.workers.len()
            );
        }
        self.reclaim.workers.retain(|id, _| own.iter().any(|u| u.id == *id && !u.idle));
    }

    /// The safe field most worth this unit's walk that still has room for a worker.
    fn field_for(&self, unit: &OwnUnit, within: f32) -> Option<usize> {
        let room = |field: &WreckField| {
            let working = self.reclaim.workers.values().filter(|(at, _)| at.dist2d(field.at) < FIELD_RADIUS).count();
            working < ((field.metal / METAL_PER_WORKER).ceil() as usize).max(1)
        };
        (0..self.reclaim.fields.len())
            .filter(|i| self.reclaim.fields[*i].safe && room(&self.reclaim.fields[*i]) && self.reclaim.fields[*i].at.dist2d(unit.pos) < within)
            .max_by(|a, b| {
                let worth = |i: &usize| self.reclaim.fields[*i].metal / (self.reclaim.fields[*i].at.dist2d(unit.pos) + 500.0);
                worth(a).total_cmp(&worth(b))
            })
    }

    /// A soldier's wreck worth having back.
    fn worth_raising(&self, wreck: &Wreck) -> bool {
        wreck.resurrects_into.and_then(|def| self.world.def(def)).is_some_and(|d| d.weapon_count > 0 && d.speed > 0.0 && d.build_speed == 0.0 && d.metal_cost >= RAISE_MIN_METAL)
    }

    /// The wrecks of the field at `at` a constructor takes apart, the nearest first: not the ones worth raising while
    /// there is a resurrection bot to raise them. An area order took everything, and 71-91 % of those orders were given
    /// over a field where a soldier worth raising had just died (the commander games, 2026-09-20).
    pub(super) fn wrecks_to_take(&self, at: Vec3, builder: &OwnUnit) -> Vec<FeatureId> {
        let Some(field) = self.reclaim.fields.iter().find(|f| f.at.dist2d(at) < 1.0) else { return Vec::new() };
        let mut wrecks: Vec<&Wreck> = field.wrecks.iter().filter_map(|id| self.reclaim.wrecks.get(id)).map(|(w, _)| w).collect();
        wrecks.retain(|w| self.reclaim.crew == 0 || !self.worth_raising(w));
        wrecks.sort_by(|a, b| a.pos.dist2d(builder.pos).total_cmp(&b.pos.dist2d(builder.pos)));
        wrecks.into_iter().take(QUEUE).map(|w| w.id).collect()
    }

    /// His units and frames in sight that `builder` takes apart within `TAKE_APART_SECONDS`, standing within
    /// `TAKE_APART_REACH` of it, the nearest first. Not aircraft (out of a builder's reach in flight), and a finished
    /// unit only of a type that can be taken apart (no commander); a frame always can. A frame pays its cost times its
    /// progress, which its health share stands for.
    pub(super) fn takeable<'a>(&self, builder: &OwnUnit, enemies: &'a [EnemyUnit]) -> Vec<Takeable<'a>> {
        let speed = self.world.def(builder.def).map_or(0.0, |d| d.reclaim_speed);
        if speed <= 0.0 {
            return Vec::new();
        }
        let mut out: Vec<Takeable> = enemies
            .iter()
            .filter(|e| e.pos.dist2d(builder.pos) < TAKE_APART_REACH)
            .filter_map(|e| {
                let def = e.def?;
                let d = self.world.def(def).filter(|d| d.health > 0.0 && (d.reclaimable || e.being_built))?;
                if self.world.domain_of(def) == crate::world::Domain::Air {
                    return None;
                }
                let share = (e.health / d.health).clamp(0.0, 1.0);
                let seconds = share * d.build_time / speed;
                (seconds <= TAKE_APART_SECONDS).then(|| Takeable { enemy: e, seconds, metal: d.metal_cost * if e.being_built { share } else { 1.0 } })
            })
            .collect();
        out.sort_by(|a, b| a.enemy.pos.dist2d(builder.pos).total_cmp(&b.enemy.pos.dist2d(builder.pos)));
        out
    }

    /// H-REC-TAKE-APART: a resurrection bot takes apart the nearest of his it can within `TAKE_APART_SECONDS`
    /// before anything else; one whose quarry has gone out of `TAKE_APART_REACH` stops (a single-unit reclaim follows its
    /// target anywhere) and goes back to the wrecks. Returns the bots busy with it.
    pub(super) fn take_apart_his(&mut self, crew: &[&OwnUnit], enemies: &[EnemyUnit], commands: &mut Vec<Command>) -> Vec<UnitId> {
        self.reclaim.taking.retain(|bot, target| {
            let Some(unit) = crew.iter().find(|u| u.id == *bot) else { return false };
            match enemies.iter().find(|e| e.id == *target) {
                Some(e) if e.pos.dist2d(unit.pos) < TAKE_APART_REACH => !unit.idle,
                Some(_) => {
                    commands.push(Command::Stop { unit: *bot });
                    false
                }
                None => false,
            }
        });
        if !self.enabled("H-REC-TAKE-APART") {
            return Vec::new();
        }
        for unit in crew {
            if self.reclaim.taking.contains_key(&unit.id) {
                continue;
            }
            let Some(target) = self.takeable(unit, enemies).first().map(|t| t.enemy.id) else { continue };
            commands.push(Command::ReclaimUnit { unit: unit.id, target, queue: false });
            self.reclaim.taking.insert(unit.id, target);
            self.reclaim.workers.remove(&unit.id);
            self.fire("H-REC-TAKE-APART");
        }
        self.reclaim.taking.keys().copied().collect()
    }

    /// Orders for an idle resurrection bot.
    pub(super) fn work_wrecks(&mut self, unit: &OwnUnit, tick: &Tick, commands: &mut Vec<Command>) {
        let Some(index) = self.field_for(unit, f32::INFINITY) else {
            // Nothing to do: wait with the army, where the next wrecks will be (under the pianist, at home: the
            // station is the heuristic army's).
            let station = self.home;
            if unit.pos.dist2d(station) > 600.0 {
                commands.push(Command::Move { unit: unit.id, to: station, queue: false });
            }
            return;
        };
        let energy = &tick.snapshot.energy;
        let allowed = self.enabled("H-REC-RESURRECT");
        let can_raise = allowed && energy.current > RAISE_ENERGY * energy.storage;
        let field = &self.reclaim.fields[index];
        let mut wrecks: Vec<&Wreck> = field.wrecks.iter().filter_map(|id| self.reclaim.wrecks.get(id)).map(|(w, _)| w).collect();
        wrecks.sort_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
        let mut raised = 0;
        for (n, wreck) in wrecks.iter().take(QUEUE).enumerate() {
            if can_raise && self.worth_raising(wreck) {
                raised += 1;
                commands.push(Command::Resurrect { unit: unit.id, feature: wreck.id, queue: n > 0 });
            } else {
                commands.push(Command::ReclaimFeature { unit: unit.id, feature: wreck.id, queue: n > 0 });
            }
        }
        let at = field.at;
        self.fire(if raised > 0 { "H-REC-RESURRECT" } else { "H-REC-CREW" });
        self.reclaim.workers.insert(unit.id, (at, tick.frame));
    }
}

impl Kit {
    /// A resurrection bot: it raises wrecks and takes them apart, and builds nothing.
    pub fn is_resurrector(&self, def: bot_protocol::UnitDefId) -> bool {
        def == self.resurrector
    }
}

#[cfg(test)]
mod tests {
    use crate::brain::pianist::fixtures::{at, brain_of, enemy, own};
    use super::*;

    /// A construction bot (1), a Pawn (2), a commander (3), a resurrection bot (4), with the engine's numbers.
    fn brain() -> Brain {
        let mut brain = brain_of(&["armck", "armpw", "armcom", "armrectr"]);
        let defs = &mut brain.world.hello.unit_defs;
        defs[0].reclaim_speed = 80.0;
        defs[1].build_time = 1650.0;
        defs[2].build_time = 75000.0;
        defs[2].reclaimable = false;
        defs[3].reclaim_speed = 200.0;
        brain
    }

    #[test]
    fn a_hurt_pawn_and_a_fresh_frame_come_apart_in_seconds_a_whole_pawn_and_a_commander_do_not() {
        let brain = brain();
        let builder = own(10, 1, at(1000.0, 1000.0));
        let pawn = |id: i32, x: f32, health: f32| bot_protocol::EnemyUnit { health, ..enemy(id, 2, at(x, 1000.0)) };
        let enemies = vec![
            pawn(20, 1200.0, 370.0),
            pawn(21, 1300.0, 92.5),
            bot_protocol::EnemyUnit { being_built: true, ..pawn(22, 1100.0, 74.0) },
            pawn(23, 1700.0, 37.0),
            bot_protocol::EnemyUnit { health: 100.0, ..enemy(24, 3, at(1050.0, 1000.0)) },
        ];
        let takeable = brain.takeable(&builder, &enemies);
        let seen: Vec<(i32, f32, f32)> = takeable.iter().map(|t| (t.enemy.id.0, (t.seconds * 10.0).round() / 10.0, t.metal.round())).collect();
        // A quarter-health Pawn in 5.2 s for its whole 54 metal (the table's cost), a 20 % frame in 4.1 s for a fifth
        // of it; the whole Pawn takes 20.6 s, the one 700 away is out of reach, a commander is never taken apart.
        let cost = brain.world.hello.unit_defs[1].metal_cost;
        assert_eq!(seen, vec![(22, 4.1, (cost * 0.2).round()), (21, 5.2, cost.round())]);
    }

    #[test]
    fn a_resurrection_bot_takes_a_hurt_raider_apart_and_lets_it_go_when_it_runs_out_of_reach() {
        let mut brain = brain();
        let bot = OwnUnit { idle: true, ..own(10, 4, at(1000.0, 1000.0)) };
        let mut enemies = vec![bot_protocol::EnemyUnit { health: 185.0, ..enemy(21, 2, at(1300.0, 1000.0)) }];
        let mut commands = Vec::new();
        let busy = brain.take_apart_his(&[&bot], &enemies, &mut commands);
        assert_eq!(busy, vec![UnitId(10)]);
        assert!(matches!(commands[..], [Command::ReclaimUnit { unit: UnitId(10), target: UnitId(21), queue: false }]), "{commands:?}");
        let bot = OwnUnit { idle: false, ..bot };
        enemies[0].pos = at(1700.0, 1000.0);
        commands.clear();
        let busy = brain.take_apart_his(&[&bot], &enemies, &mut commands);
        assert!(busy.is_empty());
        assert!(matches!(commands[..], [Command::Stop { unit: UnitId(10) }]), "{commands:?}");
    }
}
