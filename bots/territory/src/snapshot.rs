//! The game as it stands, for the simulator: the stocks, the standing economy, every builder with its place and
//! its job (docs/design/2026-10-20-territory-bot.md, decision 1); and the scenario a plan is priced in (decision 4).

use bot_protocol::{OwnUnit, Tick, UnitId, Vec3};
use buildorder::game::{distance, Spot};
use buildorder::sim::{Committed, Job, Scenario, Standing, State, StateBuilder};
use buildorder::units::Role;

use crate::hands::What;
use crate::Territory;

/// Ground within this of his start, or of an armed unit of his in sight, is dead: no spot is taken there and no
/// building placed. NullAI's commander shoots whatever comes near and follows it (terr9-lp-nullai-comet-1: it walked
/// to our base behind fleeing constructors, killed three there and died to ours, which ended a game at 16:00).
/// Stage 2's reach field in its first form.
pub(crate) const DEAD_SPOT: f32 = 1000.0;

impl Territory {
    /// Whether `p` is dead ground (DEAD_SPOT): for the spots the scenario keeps (T-SCENARIO) and every site the
    /// hands choose (T-SITES). Spots alone before: on Great Divide the constructors placed wind generators beside
    /// themselves a slot nearer his start at a time until his commander's laser reached them, 19-52 losses a game
    /// from 15:00, all within 350 of him (terr110).
    pub(crate) fn dead_ground(&self, p: Vec3) -> bool {
        self.enemy_starts.iter().any(|s| s.dist2d(p) <= DEAD_SPOT)
            || self.enemies.iter().any(|e| e.pos.dist2d(p) <= DEAD_SPOT && e.def.and_then(|d| self.def(d)).is_some_and(|d| d.weapon_count > 0))
    }

    /// T-ALLIES: an extractor of an ally's stands or is framed on the spot at `p`.
    pub(crate) fn ally_holds(&self, p: Vec3) -> bool {
        self.allies.iter().any(|u| u.pos.dist2d(p) < crate::hands::SPOT_TAKEN && self.def(u.def).is_some_and(|d| d.extracts_metal > 0.0))
    }

    /// T-ALLIES: the spot at `p` is within an allied seat's start's claim (the game's own, `CLAIM_RADIUS`) while that
    /// seat has units: his opening's, not ours to race him for.
    pub(crate) fn ally_claims(&self, p: Vec3) -> bool {
        self.ally_starts.iter().any(|(team, at)| at.dist2d(p) <= crate::his_start::CLAIM_RADIUS && self.allies.iter().any(|u| u.team == *team))
    }

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
        // A factory ordered or framed is a commitment the program takes whole (`Committed`), not a job.
        let committed: Vec<Committed> = self
            .hands
            .orders
            .values()
            .filter_map(|o| match &o.what {
                What::Build { unit, frame, .. } if game.units.list[*unit].role == Role::Factory => {
                    let frame = frame.and_then(|f| self.own.get(&f));
                    if frame.is_some_and(|f| !f.being_built) {
                        return None;
                    }
                    Some(Committed { unit: *unit, left: frame.map_or(1.0, |f| 1.0 - (f.health / f.max_health.max(1.0)).clamp(0.0, 1.0) as f64) })
                }
                _ => None,
            })
            .collect();
        let job_of = |builder: &OwnUnit| -> Option<Job> {
            match &self.hands.orders.get(&builder.id)?.what {
                What::Build { unit, .. } if game.units.list[*unit].role == Role::Factory => None,
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
            committed,
        };
        (state, builders.iter().map(|(u, _, _)| u.id).collect())
    }

    /// The scenario a plan is priced in: every spot (our half with `spots=half`) but the dead and the allies', nearest first;
    /// constructors with an empty queue take the nearest free spot; construction turrets feed the first factory, as
    /// the hands have them do.
    pub(crate) fn scenario(&self) -> Scenario {
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
        spots.retain(|s| !self.dead_ground(Vec3 { x: s.at.0 as f32, y: 0.0, z: s.at.1 as f32 }));
        // T-ALLIES: not a spot an ally holds, nor one his start claims while his seat has units.
        spots.retain(|s| {
            let p = Vec3 { x: s.at.0 as f32, y: 0.0, z: s.at.1 as f32 };
            !self.ally_holds(p) && !self.ally_claims(p)
        });
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
