//! The build-order simulator's view of the game for the player's `plan` and `search` tools
//! (`docs/design/2026-09-22-plan-search.md`): the game as the simulator sees it, a snapshot of its state every
//! `PLAN_CONTEXT_FRAMES`, and the scenario a plan is priced in. The bot's own rolling search (H-OPEN-SEARCH,
//! H-OPEN-PLAN) was deleted 2026-09-25 with the heuristic bot (`docs/design/2026-09-25-one-decider.md`); the
//! player's lists (`queue`) are the plan now.

use std::sync::Arc;

use bot_protocol::{OwnUnit, Tick, UnitDefId, UnitId, Vec3};
use buildorder::game::{distance, Game, Ground, Spot};
use buildorder::sim::{Job, Scenario, Standing, State, StateBuilder};
use buildorder::units::{Role, Units};

use super::roster::Kit;
use super::{Brain, FRAMES_PER_SECOND};

/// How often the plan context for the `plan` and `search` tools is published.
pub(super) const PLAN_CONTEXT_FRAMES: i32 = 10 * FRAMES_PER_SECOND;
/// Metal spots within this of an armed enemy in sight are left out of the scenario.
const RAID_RADIUS: f32 = 1000.0;
/// A nanoframe this close to the builder whose task is that type is that task.
const JOB_REACH: f32 = 350.0;

impl Brain {
    /// The game as the build-order simulator sees it, from what the engine told us at the start.
    pub(super) fn buildorder_game(&self, kit: &Kit) -> Option<Game> {
        let hello = &self.world.hello;
        let units = Units::new(&hello.unit_defs);
        let commander = hello.unit_defs.iter().position(|d| d.id == kit.commander)?;
        Some(Game {
            units,
            commander,
            home: (self.home.x as f64, self.home.z as f64),
            size: (hello.map.width as f64, hello.map.height as f64),
            spots: hello.metal_spots.iter().map(|s| ((s.x as f64, s.z as f64), s.y as f64)).collect(),
            wind: (hello.map.wind_min as f64, hello.map.wind_max as f64),
            // H-OPEN-WIND: plans priced at the engine's process mean; off, at the middle of the range as before.
            wind_override: (!self.enabled("H-OPEN-WIND")).then(|| (hello.map.wind_min as f64 + hello.map.wind_max as f64) / 2.0),
            terrain: hello.terrain.clone(),
        })
    }

    /// The game as it stands now, for the simulator: standing economy buildings, the builders with their jobs, the
    /// stock. Returns the state and the builders in its order, each with its rank.
    fn snapshot(&self, tick: &Tick, kit: &Kit, game: &Game) -> (State, Vec<(UnitId, usize)>) {
        let hello = &self.world.hello;
        let own = &tick.snapshot.own_units;
        let index = |def: UnitDefId| hello.unit_defs.iter().position(|d| d.id == def);
        let at = |p: Vec3| (p.x as f64, p.z as f64);
        let pays = |unit: usize, pos: Vec3| {
            if game.units.list[unit].extracts_metal == 0.0 {
                return 0.0;
            }
            hello.metal_spots.iter().filter(|s| s.dist2d(pos) < hello.map.spot_radius()).map(|s| game.spot_metal(s.y as f64)).next().unwrap_or(0.0)
        };
        let standing: Vec<Standing> = own
            .iter()
            .filter(|u| !u.being_built)
            .filter_map(|u| {
                let unit = index(u.def)?;
                matches!(game.units.list[unit].role, Role::Eco | Role::Turret | Role::Nano).then(|| Standing { unit, site: at(u.pos), pays: pays(unit, u.pos) })
            })
            .collect();
        // By role, not by the Kit (docs/design/2026-09-22-plan-search.md, decision 3): every factory and every mobile
        // builder that stands gets a queue, whatever its type.
        let rank = |def: UnitDefId| match def {
            d if d == kit.commander => Some(0),
            d if self.world.is_factory_def(d) => Some(1),
            d if self.world.is_mobile_builder(d) => Some(2),
            _ => None,
        };
        let mut builders: Vec<(&OwnUnit, usize, usize)> = own.iter().filter(|u| !u.being_built).filter_map(|u| Some((u, rank(u.def)?, index(u.def)?))).collect();
        builders.sort_by_key(|(u, rank, _)| (*rank, u.id.0));
        let job_of = |builder: &OwnUnit| -> Option<Job> {
            // The builder's task is the pianist's: the nanoframe of that type nearest the site, else the site it walks to.
            let pianist = self.pianist.as_ref()?;
            match pianist.tasks.get(&builder.id) {
                Some(super::pianist::Task::Build { def, near, .. }) => {
                    let unit = index(*def)?;
                    let frame = own.iter().filter(|u| u.being_built && u.def == *def && u.pos.dist2d(*near) < JOB_REACH).min_by(|a, b| a.pos.dist2d(*near).total_cmp(&b.pos.dist2d(*near)));
                    Some(match frame {
                        Some(f) => Job { unit, site: at(f.pos), progress: (f.health / f.max_health.max(1.0)).clamp(0.0, 1.0) as f64, pays: pays(unit, f.pos) },
                        None => Job { unit, site: at(*near), progress: 0.0, pays: pays(unit, *near) },
                    })
                }
                _ => None,
            }
        };
        let army_metal: f64 = own.iter().filter(|u| !u.being_built).filter_map(|u| index(u.def)).filter(|i| game.units.list[*i].role == Role::Army).map(|i| game.units.list[i].metal_cost).sum();
        let state = State {
            t0: (tick.frame / FRAMES_PER_SECOND) as f64,
            army_metal,
            metal: tick.snapshot.metal.current as f64,
            energy: tick.snapshot.energy.current as f64,
            metal_storage: tick.snapshot.metal.storage as f64,
            energy_storage: tick.snapshot.energy.storage as f64,
            standing,
            builders: builders.iter().map(|(u, _, unit)| StateBuilder { unit: *unit, place: at(u.pos), job: job_of(u) }).collect(),
        };
        (state, builders.iter().map(|(u, rank, _)| (u.id, *rank)).collect())
    }

    /// Publishes what the `plan` and `search` tools simulate from (docs/design/2026-09-22-plan-search.md, decision 8).
    pub(super) fn publish_plan_context(&mut self, tick: &Tick, kit: &Kit) {
        let Some(shared) = self.strategist.clone() else { return };
        if self.plan_game.is_none() {
            let Some(game) = self.buildorder_game(kit) else { return };
            let ground = game.ground();
            self.plan_game = Some((Arc::new(game), ground));
        }
        let (game, ground) = self.plan_game.clone().expect("built above");
        let (state, order) = self.snapshot(tick, kit, &game);
        let scenario = self.scenario(tick, &game, ground);
        let standing_factories = order.iter().filter(|(_, rank)| *rank == 1).count();
        let standing_constructors = order.iter().filter(|(_, rank)| *rank == 2).count();
        let (mut factories, mut constructors) = (0, 0);
        let actors: Vec<(String, usize)> = order
            .iter()
            .map(|(id, rank)| {
                let queue = match rank {
                    0 => 0,
                    1 => {
                        factories += 1;
                        factories
                    }
                    _ => {
                        constructors += 1;
                        standing_factories + constructors
                    }
                };
                (self.actor_name(*id), queue)
            })
            .collect();
        let hello = &self.world.hello;
        let wind = match scenario.wind {
            buildorder::sim::Wind::Constant(w) => w,
            _ => game.mean_wind(),
        };
        let context = crate::strategist::shared::PlanContext {
            game,
            scenario,
            state,
            actors,
            standing_factories,
            standing_constructors,
            spots: hello.metal_spots.iter().map(|s| (s.x as f64, s.z as f64)).collect(),
            spot_radius: hello.map.spot_radius() as f64,
            turret: hello.unit_defs.iter().position(|d| d.id == kit.turret),
            water: hello.terrain.heights.iter().any(|h| *h < 0),
            wind,
            frame: tick.frame,
        };
        *shared.plan_context.lock().unwrap() = Some(Arc::new(context));
    }

    /// The scenario a plan is priced in now: the spots ours on foot, less those an armed enemy in sight stands by.
    fn scenario(&self, tick: &Tick, game: &Game, ground: Arc<dyn Ground>) -> Scenario {
        let raiders: Vec<Vec3> = tick.snapshot.enemies.iter().filter(|e| self.armed(e.def)).map(|e| e.pos).collect();
        let mut spots: Vec<Spot> = self
            .world
            .hello
            .metal_spots
            .iter()
            .filter(|s| self.reachable_on_foot(**s) && self.spot_is_ours(**s) && !raiders.iter().any(|r| r.dist2d(**s) < RAID_RADIUS))
            .map(|s| Spot { at: (s.x as f64, s.z as f64), metal: game.spot_metal(s.y as f64) })
            .collect();
        spots.sort_by(|a, b| distance(a.at, game.home).total_cmp(&distance(b.at, game.home)));
        let mut scenario = game.scenario(spots, ground);
        scenario.constructors_default_to_extractors = true;
        scenario.commander_leash = super::economy::EARLY_COMMANDER_LEASH as f64;
        scenario
    }

    /// Armed and mobile, or unknown: what walks up to the base is not a constructor. Radar contacts count.
    fn armed(&self, def: Option<UnitDefId>) -> bool {
        def.is_none_or(|d| self.world.def(d).is_some_and(|d| d.weapon_count > 0 && d.speed > 0.0))
    }
}
