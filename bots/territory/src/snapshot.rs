//! The game as it stands, for the simulator: the stocks, the standing economy, every builder with its place and
//! its job (docs/design/2026-10-20-territory-bot.md, decision 1); and the scenario a plan is priced in (decision 4).

use bot_protocol::{OwnUnit, Tick, UnitId, Vec3};
use buildorder::game::{distance, Spot};
use buildorder::sim::{Job, Scenario, Standing, State, StateBuilder};
use buildorder::units::Role;

use crate::hands::What;
use crate::Territory;

/// Spots within this of his start, or of an armed unit of his in sight, are not taken: NullAI's commander shoots
/// whatever comes near and follows it (terr9-lp-nullai-comet-1: it walked to our base behind fleeing constructors,
/// killed three there and died to ours, which ended a game at 16:00). Stage 2's reach field in its first form.
const DEAD_SPOT: f64 = 1000.0;

impl Territory {
    /// The amount of the metal spot under `pos`, 0 off any spot.
    pub(crate) fn spot_amount(&self, pos: Vec3) -> f64 {
        let radius = self.hello.map.spot_radius();
        self.hello.metal_spots.iter().filter(|s| s.dist2d(pos) < radius).map(|s| s.y as f64).next().unwrap_or(0.0)
    }

    /// Metal a second an extractor of this type pays on the spot under `pos`: for an upgrade, what it adds over the
    /// basic rate (the simulator counts an upgrade's increment).
    pub(crate) fn pays(&self, unit: usize, pos: Vec3, standing: bool) -> f64 {
        let units = &self.game().units;
        let rate = units.list[unit].extracts_metal;
        if rate == 0.0 {
            return 0.0;
        }
        let amount = self.spot_amount(pos);
        if standing || !self.is_upgrade(unit) { amount * rate } else { amount * (rate - units.basic_extraction()) }
    }

    pub(crate) fn is_upgrade(&self, unit: usize) -> bool {
        let units = &self.game().units;
        units.list[unit].extracts_metal > units.basic_extraction() * 1.5
    }

    /// The state and the builders in its queue order (the commander, the factories by id, the constructors by id).
    pub(crate) fn snapshot(&self, tick: &Tick) -> (State, Vec<UnitId>) {
        let game = self.game();
        let at = |p: Vec3| (p.x as f64, p.z as f64);
        let standing: Vec<Standing> = self
            .own
            .values()
            .filter(|u| !u.being_built)
            .filter_map(|u| {
                let unit = self.unit_index(u.def)?;
                matches!(game.units.list[unit].role, Role::Eco | Role::Turret | Role::Nano).then(|| Standing { unit, site: at(u.pos), pays: self.pays(unit, u.pos, true) })
            })
            .collect();
        let rank = |role: Role| match role {
            Role::Commander => Some(0),
            Role::Factory => Some(1),
            Role::Builder => Some(2),
            _ => None,
        };
        let mut builders: Vec<(&OwnUnit, usize, usize)> = self
            .own
            .values()
            .filter(|u| !u.being_built)
            .filter_map(|u| {
                let unit = self.unit_index(u.def)?;
                Some((u, rank(game.units.list[unit].role)?, unit))
            })
            .collect();
        builders.sort_by_key(|(u, rank, _)| (*rank, u.id.0));
        let job_of = |builder: &OwnUnit| -> Option<Job> {
            match &self.hands.orders.get(&builder.id)?.what {
                What::Build { unit, near, frame, .. } => {
                    let frame = frame.and_then(|f| self.own.get(&f));
                    // A frame that stands finished is a standing unit already, not a job: counted as both, the
                    // program saw two labs where one stood and planned a second (terr12-lp-nullai-comet-1).
                    if frame.is_some_and(|f| !f.being_built) {
                        return None;
                    }
                    let site = frame.map_or(*near, |f| f.pos);
                    let progress = frame.map_or(0.0, |f| (f.health / f.max_health.max(1.0)).clamp(0.0, 1.0) as f64);
                    Some(Job { unit: *unit, site: at(site), progress, pays: self.pays(*unit, site, false) })
                }
                _ => None,
            }
        };
        let state = State {
            t0: (tick.frame / 30) as f64,
            army_metal: 0.0,
            metal: tick.snapshot.metal.current as f64,
            energy: tick.snapshot.energy.current as f64,
            metal_storage: tick.snapshot.metal.storage as f64,
            energy_storage: tick.snapshot.energy.storage as f64,
            standing,
            builders: builders.iter().map(|(u, _, unit)| StateBuilder { unit: *unit, place: at(u.pos), job: job_of(u) }).collect(),
        };
        (state, builders.iter().map(|(u, _, _)| u.id).collect())
    }

    /// The scenario a plan is priced in: our half of the spots (every spot with `spots=all`), nearest first;
    /// constructors with an empty queue take the nearest free spot; construction turrets feed the first factory, as
    /// the hands have them do.
    pub(crate) fn scenario(&self, tick: &Tick) -> Scenario {
        let game = self.game();
        let mut spots: Vec<Spot> = if self.options.all_spots {
            let mut spots: Vec<Spot> = game.spots.iter().map(|(at, amount)| Spot { at: *at, metal: game.spot_metal(*amount) }).collect();
            spots.sort_by(|a, b| distance(a.at, game.home).total_cmp(&distance(b.at, game.home)));
            spots
        } else {
            game.own_half()
        };
        // The dead spots round his start (the user, 2026-10-20: NullAI's commander "automatically attacks anything
        // near it. There will be a dead spot of 2-3 mexes we will just have to ignore"; by a radius, his ruling).
        let mut his: Vec<(f64, f64)> = vec![(self.enemy_home.x as f64, self.enemy_home.z as f64)];
        his.extend(tick.snapshot.enemies.iter().filter(|e| e.def.and_then(|d| self.def(d)).is_some_and(|d| d.weapon_count > 0)).map(|e| (e.pos.x as f64, e.pos.z as f64)));
        spots.retain(|s| his.iter().all(|h| distance(s.at, *h) > DEAD_SPOT));
        let mut scenario = game.scenario(spots, self.ground());
        scenario.constructors_default_to_extractors = true;
        scenario.nanos_help_frames = false;
        scenario
    }

    /// The scenario's spots as indices into the game's list: the spots the hands may take.
    pub(crate) fn spot_indices(&self, scenario: &Scenario) -> Vec<usize> {
        let game = self.game();
        scenario.spots.iter().filter_map(|s| game.spots.iter().position(|(at, _)| distance(*at, s.at) < 1.0)).collect()
    }
}
