//! T-COVER and T-RADAR (`docs/design/2026-10-25-territory-defence.md`, step 3's first form): turrets in the regions
//! holding something of ours, to the weight of the largest raid of his seen lately and never more turret metal than
//! what they guard; radar where the race's warning is short (T-RACE). A builder with no order takes the nearest
//! wanted site before the plan's flows. The planner does not see the turrets.

use std::collections::{BTreeMap, VecDeque};

use bot_protocol::{BuildSite, Command, EnemyUnit, UnitId, Vec3};
use buildorder::units::Role;

use crate::hands::{Order, What};
use crate::{Territory, MINUTE, SECOND};

/// His armed mobile units this near a building of ours are a raid on it.
pub(crate) const RAID_NEAR: f32 = 800.0;
/// The raid wanted against is the largest seen over this long.
const RAID_MEMORY: i32 = 5 * MINUTE;
/// His commander and a moving radar dot weigh this much (the distilled bot's D-WEIGHT); a dot moving slower than
/// DOT_MOVING elmos a frame is taken for one of his buildings and weighs nothing.
const COMMANDER_WEIGHT: f32 = 1800.0;
const DOT_WEIGHT: f32 = 110.0;
const DOT_MOVING: f32 = 0.3;
/// At most this much raid is covered by turrets: his push (3,000-4,000 at 10:00 on the bench, terr208-219) is the
/// guard's (T-GUARD).
const COVER_MOST: f32 = 1000.0;
/// A turret weighs its metal times this (D-WEIGHT's, until combatsim measures one).
pub(crate) const TURRET_WEIGHT: f32 = 3.0;
/// A turret goes this far from the building it covers toward his start; a radar this far behind its building.
const TURRET_FORWARD: f32 = 120.0;
const RADAR_BACK: f32 = 150.0;
/// A region whose warning is under this many seconds wants a radar, unless one stands, is framed or is ordered
/// within RADAR_SHARED of its forward building.
const RADAR_WANT: f32 = 8.0;
const RADAR_SHARED: f32 = 1000.0;
/// A radar is a type of this much radar range or more.
const RADAR_LEAST: f32 = 1000.0;
/// No defence order before this (the opening's commander and constructors are the plan's).
pub(crate) const DEFENCE_FROM: i32 = 3 * MINUTE;
/// A site is not taken with an armed unit of his within this of it (a light laser turret's reach), nor within the
/// dead radius of his start. The builders' dead ground (1,000 of any armed unit of his in sight) forbade every site
/// while a raid was seen, which is when one is wanted: Great Divide, terr222 game 00, a raid seen from 3:30 and no
/// turret or radar all game.
const SITE_CLEAR: f32 = 450.0;
/// A constructor takes a wanted site within this; the commander within its leash of home.
const DEFENCE_REACH: f32 = 1200.0;
const SITE_SEARCH: f32 = 200.0;
/// The `defence:` line every this many frames.
pub(crate) const DEFENCE_LOG: i32 = 30 * SECOND;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Kind {
    Turret,
    Radar,
}

#[derive(Clone, Debug)]
pub(crate) struct Want {
    kind: Kind,
    at: Vec3,
    region: usize,
}

#[derive(Default)]
pub(crate) struct Defence {
    /// (frame, metal) of his armed units seen near our buildings, over RAID_MEMORY.
    raids: VecDeque<(i32, f32)>,
    /// The sites wanted, remade every hands tick.
    wants: Vec<Want>,
    /// Per builder type (the simulator's index): its turret and its radar, chosen on first use.
    kit: BTreeMap<usize, (Option<usize>, Option<usize>)>,
    /// The game's unit table with the weapons (combatsim's), loaded on first use.
    table: Option<combatsim::Units>,
}

pub(crate) fn toward(from: Vec3, to: Vec3, by: f32) -> Vec3 {
    let d = from.dist2d(to);
    if d < 1.0 {
        return from;
    }
    Vec3 { x: from.x + (to.x - from.x) / d * by, y: from.y, z: from.z + (to.z - from.z) / d * by }
}

/// One turret of ours or ordered: where, its reach, its metal.
struct Gun {
    at: Vec3,
    reach: f32,
    metal: f32,
}

impl Territory {
    /// Every hands tick: his armed metal in sight within RAID_NEAR of a building of ours, kept when there is any.
    pub(crate) fn observe_raids(&mut self) {
        let buildings: Vec<Vec3> = self.own.values().filter(|u| self.def(u.def).is_some_and(|d| d.speed == 0.0)).map(|u| u.pos).collect();
        let mut metal = 0.0;
        for e in &self.enemies {
            let weight = self.his_weight(e);
            if weight > 0.0 && buildings.iter().any(|b| b.dist2d(e.pos) <= RAID_NEAR) {
                metal += weight;
            }
        }
        let frame = self.frame;
        self.defence.raids.retain(|(at, _)| frame - *at <= RAID_MEMORY);
        if metal > 0.0 {
            self.defence.raids.push_back((frame, metal));
        }
    }

    /// What a unit of his weighs in a raid: an armed mobile unit its metal, his commander COMMANDER_WEIGHT, a radar dot
    /// moving DOT_WEIGHT; a constructor, a building or a dot standing still nothing.
    pub(crate) fn his_weight(&self, e: &EnemyUnit) -> f32 {
        match e.def.and_then(|d| self.def(d).map(|info| (d, info))) {
            Some((d, info)) if info.speed > 0.0 && info.weapon_count > 0 && info.reach > 0.0 => {
                if self.role_of_def(d) == Some(Role::Commander) {
                    COMMANDER_WEIGHT
                } else if info.build_options.is_empty() {
                    info.metal_cost
                } else {
                    0.0
                }
            }
            Some(_) => 0.0,
            None if e.vel.x.hypot(e.vel.z) >= DOT_MOVING => DOT_WEIGHT,
            None => 0.0,
        }
    }

    /// The raid the turrets are wanted against: the largest seen over RAID_MEMORY, at most COVER_MOST.
    fn raid(&self) -> f32 {
        self.defence.raids.iter().map(|(_, m)| *m).fold(0.0, f32::max).min(COVER_MOST)
    }

    /// A type's ground fighting strength per metal, √(damage a second × health) / metal in combatsim's table (a
    /// force's strength goes with its numbers squared, so a type's worth per metal is the root); none for a type the
    /// table lacks or that cannot fight on the ground (a weapon for aircraft alone, one for water, a stockpile).
    pub(crate) fn strength(&mut self, u: usize) -> Option<f32> {
        let table = self.defence.table.get_or_insert_with(combatsim::Units::default);
        let info = &self.hello.unit_defs[u];
        let unit = table.list.get(table.index(&info.name)?)?;
        let dps = unit.dps();
        (!info.water_only && info.stockpile_seconds == 0.0 && unit.reach() > 0.0 && dps > 0.0).then(|| (dps * unit.health).sqrt() / info.metal_cost.max(1.0))
    }

    /// The builder type's turret (the most ground fighting strength per metal) and its radar (the cheapest).
    fn kit(&mut self, builder: usize) -> (Option<usize>, Option<usize>) {
        if let Some(kit) = self.defence.kit.get(&builder) {
            return *kit;
        }
        let game = self.game.clone().expect("started");
        let units = &game.units;
        let builds = &units.list[builder].builds;
        let turret = builds.iter().copied().filter(|u| units.list[*u].role == Role::Turret).filter_map(|u| self.strength(u).map(|s| (u, s))).max_by(|a, b| a.1.total_cmp(&b.1)).map(|(u, _)| u);
        let radar = builds
            .iter()
            .copied()
            .filter(|u| {
                let info = &self.hello.unit_defs[*u];
                units.list[*u].role == Role::Eco && info.radar_range >= RADAR_LEAST && !info.water_only && info.speed == 0.0
            })
            .min_by(|a, b| units.list[*a].metal_cost.total_cmp(&units.list[*b].metal_cost));
        self.defence.kit.insert(builder, (turret, radar));
        (turret, radar)
    }

    fn is_radar(&self, u: usize) -> bool {
        let info = &self.hello.unit_defs[u];
        self.game().units.list[u].role == Role::Eco && info.radar_range >= RADAR_LEAST
    }

    /// Our turrets standing, framed and ordered, and our radars' places likewise.
    fn guns_and_radars(&self) -> (Vec<Gun>, Vec<Vec3>) {
        let units = &self.game().units;
        let (mut guns, mut radars) = (Vec::new(), Vec::new());
        for u in self.own.values() {
            let Some(i) = self.unit_index(u.def) else { continue };
            if units.list[i].role == Role::Turret {
                guns.push(Gun { at: u.pos, reach: self.hello.unit_defs[i].reach, metal: units.list[i].metal_cost as f32 });
            } else if self.is_radar(i) {
                radars.push(u.pos);
            }
        }
        for order in self.hands.orders.values() {
            if let What::Build { unit, near, frame: None, .. } = &order.what {
                if units.list[*unit].role == Role::Turret {
                    guns.push(Gun { at: *near, reach: self.hello.unit_defs[*unit].reach, metal: units.list[*unit].metal_cost as f32 });
                } else if self.is_radar(*unit) {
                    radars.push(*near);
                }
            }
        }
        (guns, radars)
    }

    /// Every hands tick: the turret and radar sites wanted (T-COVER, T-RADAR).
    pub(crate) fn plan_defence(&mut self) {
        self.defence.wants.clear();
        if self.frame < DEFENCE_FROM || !self.options.defence {
            return;
        }
        let Some(regions) = &self.regions else { return };
        let units = &self.game().units;
        let (guns, radars) = self.guns_and_radars();
        // What each region guards: every building of ours but a turret or a radar, frames at their progress.
        let mut guarded: Vec<Vec<(Vec3, f32)>> = vec![Vec::new(); regions.list.len()];
        for u in self.own.values() {
            let Some(i) = self.unit_index(u.def) else { continue };
            if units.list[i].speed > 0.0 || units.list[i].role == Role::Turret || self.is_radar(i) {
                continue;
            }
            let Some(k) = self.region_of(u.pos) else { continue };
            let worth = units.list[i].metal_cost as f32 * if u.being_built { (u.health / u.max_health.max(1.0)).clamp(0.0, 1.0) } else { 1.0 };
            guarded[k].push((u.pos, worth));
        }
        let mut gun_metal = vec![0.0f32; regions.list.len()];
        for g in &guns {
            if let Some(k) = self.region_of(g.at) {
                gun_metal[k] += g.metal;
            }
        }
        let least = self.cheapest_turret();
        let raid = self.raid();
        let mut wants = Vec::new();
        let mut radar_at: Vec<Vec3> = radars.clone();
        for (k, here) in guarded.iter().enumerate() {
            let metal: f32 = here.iter().map(|(_, w)| w).sum();
            if metal < 1.0 {
                continue;
            }
            let nearest_him = |a: &&(Vec3, f32), b: &&(Vec3, f32)| a.0.dist2d(self.enemy_home).total_cmp(&b.0.dist2d(self.enemy_home));
            // T-COVER: under the raid's weight, and the next turret no more than what the region guards.
            if raid > 0.0 && gun_metal[k] * TURRET_WEIGHT < raid && least.is_some_and(|c| gun_metal[k] + c <= metal) {
                let cover = |p: Vec3| guns.iter().filter(|g| g.at.dist2d(p) <= g.reach).map(|g| g.metal * TURRET_WEIGHT).sum::<f32>();
                let weakest = here.iter().min_by(|a, b| cover(a.0).total_cmp(&cover(b.0)).then(nearest_him(a, b)));
                if let Some((p, _)) = weakest {
                    let at = toward(*p, self.enemy_home, TURRET_FORWARD);
                    if self.site_clear(at) {
                        wants.push(Want { kind: Kind::Turret, at, region: k });
                    }
                }
            }
            // T-RADAR: a short warning, the region's forward building out of our radar's coverage, and no radar near it
            // (terr250-255 against NullAI: a radar wanted wherever the warning was short, 4-35 a game).
            if self.race.warn.get(&k).is_some_and(|w| *w < RADAR_WANT)
                && let Some((forward, _)) = here.iter().min_by(nearest_him)
                && !self.race.sensors.as_ref().is_some_and(|s| s.radar.at(forward.x, forward.z))
                && !radar_at.iter().any(|r| r.dist2d(*forward) <= RADAR_SHARED)
            {
                let at = toward(*forward, self.home, RADAR_BACK);
                if self.site_clear(at) {
                    radar_at.push(at);
                    wants.push(Want { kind: Kind::Radar, at, region: k });
                }
            }
        }
        self.defence.wants = wants;
    }

    /// A defence site may be taken: outside his start's dead radius, and no armed unit of his in sight within
    /// SITE_CLEAR of it.
    fn site_clear(&self, p: Vec3) -> bool {
        p.dist2d(self.enemy_home) > crate::snapshot::DEAD_SPOT
            && !self.enemies.iter().any(|e| e.pos.dist2d(p) <= SITE_CLEAR && e.def.and_then(|d| self.def(d)).is_some_and(|d| d.weapon_count > 0))
    }

    /// The cheapest turret any of our builder types makes, the parity check's step.
    fn cheapest_turret(&mut self) -> Option<f32> {
        let builders: Vec<usize> = self.own.values().filter(|u| !u.being_built).filter_map(|u| self.unit_index(u.def)).filter(|i| matches!(self.game().units.list[*i].role, Role::Builder | Role::Commander)).collect();
        let mut least: Option<f32> = None;
        for b in builders {
            if let (Some(t), _) = self.kit(b) {
                let cost = self.game().units.list[t].metal_cost as f32;
                least = Some(least.map_or(cost, |l| l.min(cost)));
            }
        }
        least
    }

    /// A builder with no order takes the nearest wanted site it can build within its reach (T-COVER, T-RADAR).
    pub(crate) fn defence_order(&mut self, id: UnitId, unit: usize, role: Role, commands: &mut Vec<Command>) -> bool {
        if self.frame < DEFENCE_FROM || !matches!(role, Role::Builder | Role::Commander) || self.defence.wants.is_empty() {
            return false;
        }
        let Some(builder) = self.own.get(&id).cloned() else { return false };
        let (turret, radar) = self.kit(unit);
        let leash = self.options.com_leash;
        let reachable = |w: &Want| if role == Role::Commander { w.at.dist2d(self.home) <= leash } else { w.at.dist2d(builder.pos) <= DEFENCE_REACH };
        let pick = self
            .defence
            .wants
            .iter()
            .enumerate()
            .filter(|(_, w)| reachable(w))
            .filter_map(|(i, w)| match w.kind {
                Kind::Turret => turret.map(|u| (i, u)),
                Kind::Radar => radar.map(|u| (i, u)),
            })
            .filter(|(_, u)| !self.hands.struck.contains(u) && self.hands.type_refused.get(u).is_none_or(|until| self.frame >= *until))
            .min_by(|a, b| self.defence.wants[a.0].at.dist2d(builder.pos).total_cmp(&self.defence.wants[b.0].at.dist2d(builder.pos)));
        let Some((i, u)) = pick else { return false };
        if !self.affordable(u) {
            return false;
        }
        if !self.energy_fits(u, unit) {
            self.hands.energy_refused += 1;
            *self.hands.energy_refused_by.entry(u).or_default() += 1;
            return false;
        }
        let want = self.defence.wants.remove(i);
        let def = self.hello.unit_defs[u].id;
        let site = BuildSite { near: want.at, search_radius: SITE_SEARCH, min_dist: 1, placements: Vec::new(), exact: false };
        commands.push(Command::Build { unit: id, def, site: Some(site), queue: false });
        let what = What::Build { unit: u, def, near: want.at, frame: None };
        let why = match want.kind {
            Kind::Turret => "cover",
            Kind::Radar => "radar",
        };
        self.note_order(id, &what, why);
        self.hands.orders.insert(id, Order { what, given: self.frame, told: self.frame, arrived: None, step: None });
        *self.hands.ordered.entry((u, why)).or_default() += 1;
        true
    }

    /// `defence:` every DEFENCE_LOG: the raid wanted against and how many sightings it is the largest of, the turrets
    /// and radars standing and framed, and the sites wanted with their regions.
    pub(crate) fn log_defence(&mut self) {
        let units = &self.game().units;
        let (mut turrets, mut turret_metal, mut radars) = (0, 0.0f32, 0);
        for u in self.own.values() {
            let Some(i) = self.unit_index(u.def) else { continue };
            if units.list[i].role == Role::Turret {
                turrets += 1;
                turret_metal += units.list[i].metal_cost as f32;
            } else if self.is_radar(i) {
                radars += 1;
            }
        }
        let wanted = |k: Kind| self.defence.wants.iter().filter(|w| w.kind == k).count();
        let regions: Vec<String> = self.defence.wants.iter().map(|w| format!("{} r{} ({:.0},{:.0})", if w.kind == Kind::Turret { "turret" } else { "radar" }, w.region, w.at.x, w.at.z)).collect();
        self.log(format_args!(
            "defence: raid {:.0} (seen {}) | turrets {turrets} ({turret_metal:.0} metal), radars {radars} | wanted {} turrets, {} radars: {}",
            self.raid(),
            self.defence.raids.len(),
            wanted(Kind::Turret),
            wanted(Kind::Radar),
            if regions.is_empty() { "-".to_string() } else { regions.join(", ") }
        ));
    }
}
