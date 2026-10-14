//! The unit table: the numbers the engine reports for this game's unit types, as the bot's `Hello` or a match
//! record's header carries them. Nothing is compiled in.

use std::collections::HashMap;

use bot_protocol::{MoveClass, UnitDefInfo};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Commander,
    /// Economy building: extractor, generator, converter, storage.
    Eco,
    Factory,
    /// Mobile constructor: anything that moves and has a build menu.
    Builder,
    Nano,
    Turret,
    Army,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub name: String,
    pub role: Role,
    /// Indices into `Units::list` of what it builds.
    pub builds: Vec<usize>,
    pub metal_cost: f64,
    pub energy_cost: f64,
    pub build_time: f64,
    pub worker_time: f64,
    pub build_distance: f64,
    pub speed: f64,
    pub move_class: Option<MoveClass>,
    pub metal_make: f64,
    /// Net constant energy production: `energy_make - energy_upkeep`.
    pub energy_make: f64,
    pub extracts_metal: f64,
    pub wind_cap: f64,
    pub metal_storage: f64,
    pub energy_storage: f64,
    pub radar_range: f64,
    pub conv_capacity: f64,
    pub conv_efficiency: f64,
}

pub struct Units {
    pub list: Vec<Unit>,
    by_name: HashMap<String, usize>,
}

/// A round a unit stockpiles (a nuclear missile, an anti-nuke's interceptor): its cost and the seconds one round takes.
/// The engine advances a stockpile `1 / seconds` a frame when the frame's share of the cost can be paid, all or
/// nothing that frame, and no builder adds to it (`CWeapon::UpdateStockpile`, nothing in `Builder.cpp`; BAR's stockpile
/// gadgets only limit and announce), so a round is a flow of resources, never of build power
/// (K-rules-a-stockpile-takes-no-build-power).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stockpile {
    pub metal: f64,
    pub energy: f64,
    pub seconds: f64,
}

/// The stockpiling units of the standard roster, by name, from the game's unit files (`weapondefs` of each:
/// `metalpershot`, `energypershot`, `stockpiletime`); the engine reports none of these numbers to an AI.
const STOCKPILES: [(&str, Stockpile); 6] = [
    ("armsilo", Stockpile { metal: 1000.0, energy: 125_000.0, seconds: 120.0 }),
    ("corsilo", Stockpile { metal: 1500.0, energy: 187_500.0, seconds: 180.0 }),
    // Legion's Supernova is the Cortex silo's numbers, its Aegis the Armada anti-nuke's (`Legion/Defenses/legsilo.lua`, `legabm.lua`).
    ("legsilo", Stockpile { metal: 1500.0, energy: 187_500.0, seconds: 180.0 }),
    ("armamd", Stockpile { metal: 150.0, energy: 7500.0, seconds: 90.0 }),
    ("corfmd", Stockpile { metal: 150.0, energy: 7500.0, seconds: 90.0 }),
    ("legabm", Stockpile { metal: 150.0, energy: 7500.0, seconds: 90.0 }),
];

fn role(n: &UnitDefInfo) -> Role {
    let (moves, menu) = (n.speed > 0.0, !n.build_options.is_empty());
    match () {
        _ if moves && menu && n.name.ends_with("com") => Role::Commander,
        _ if moves && menu => Role::Builder,
        _ if menu => Role::Factory,
        _ if !moves && n.build_speed > 0.0 => Role::Nano,
        _ if !moves && n.weapon_count > 0 => Role::Turret,
        _ if moves && n.weapon_count > 0 => Role::Army,
        _ => Role::Eco,
    }
}

impl Units {
    pub fn new(defs: &[UnitDefInfo]) -> Units {
        let index_of: HashMap<_, usize> = defs.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
        let list: Vec<Unit> = defs
            .iter()
            .map(|n| Unit {
                name: n.name.clone(),
                role: role(n),
                builds: n.build_options.iter().filter_map(|id| index_of.get(id).copied()).collect(),
                metal_cost: n.metal_cost as f64,
                energy_cost: n.energy_cost as f64,
                build_time: n.build_time as f64,
                worker_time: n.build_speed as f64,
                build_distance: n.build_distance as f64,
                speed: n.speed as f64,
                move_class: n.move_class,
                metal_make: n.metal_make as f64,
                energy_make: (n.energy_make - n.energy_upkeep) as f64,
                extracts_metal: n.extracts_metal as f64,
                wind_cap: n.wind_cap as f64,
                metal_storage: n.metal_storage as f64,
                energy_storage: n.energy_storage as f64,
                radar_range: n.radar_range as f64,
                conv_capacity: n.converter.map_or(0.0, |c| c.capacity as f64),
                conv_efficiency: n.converter.map_or(0.0, |c| c.efficiency as f64),
            })
            .collect();
        let by_name = list.iter().enumerate().map(|(i, u)| (u.name.clone(), i)).collect();
        Units { list, by_name }
    }

    pub fn index(&self, name: &str) -> Option<usize> {
        self.by_name.get(name).copied()
    }

    pub fn get(&self, name: &str) -> &Unit {
        &self.list[self.index(name).unwrap_or_else(|| panic!("no unit type {name} in this game"))]
    }

    /// What a unit type stockpiles once it stands, if anything.
    pub fn stockpile(&self, unit: usize) -> Option<Stockpile> {
        STOCKPILES.iter().find(|(name, _)| *name == self.list[unit].name).map(|(_, s)| *s)
    }

    /// The cheapest thing `builder` builds that passes `test`.
    pub fn cheapest(&self, builder: usize, test: impl Fn(&Unit) -> bool) -> Option<usize> {
        self.list[builder].builds.iter().copied().filter(|u| test(&self.list[*u])).min_by(|a, b| self.list[*a].metal_cost.total_cmp(&self.list[*b].metal_cost))
    }

    pub fn extractor(&self, builder: usize) -> Option<usize> {
        self.cheapest(builder, |u| u.extracts_metal > 0.0)
    }

    /// Every unit type reachable from `root` by build menus, `root` first, in breadth-first order.
    pub fn reachable(&self, root: usize) -> Vec<usize> {
        let mut seen = vec![root];
        let mut i = 0;
        while i < seen.len() {
            for next in &self.list[seen[i]].builds {
                if !seen.contains(next) {
                    seen.push(*next);
                }
            }
            i += 1;
        }
        seen
    }

    /// The shortest chain of makers from `root` to `target` by build menus (`root` first, `target` last), if any.
    pub fn chain(&self, root: usize, target: usize) -> Option<Vec<usize>> {
        let mut parent: Vec<Option<usize>> = vec![None; self.list.len()];
        let mut queue = std::collections::VecDeque::from([root]);
        let mut seen = vec![false; self.list.len()];
        seen[root] = true;
        while let Some(at) = queue.pop_front() {
            if at == target {
                let mut path = vec![at];
                while let Some(p) = parent[*path.last().unwrap()] {
                    path.push(p);
                }
                path.reverse();
                return Some(path);
            }
            for next in &self.list[at].builds {
                if !seen[*next] {
                    seen[*next] = true;
                    parent[*next] = Some(at);
                    queue.push_back(*next);
                }
            }
        }
        None
    }

    /// Whether a unit type is sea-bound: a ship, a submarine, or a building whose name marks it as on or under
    /// water (the numbers carry no such flag; the names do: `armsy`, `armuwes`, `coruwmex`).
    pub fn sea_bound(&self, unit: usize) -> bool {
        let u = &self.list[unit];
        let ship = u.move_class.is_some_and(|m| matches!(m.kind, bot_protocol::MoveKind::Ship));
        let name = u.name.get(3..).unwrap_or("");
        ship || name.starts_with("uw") || name == "sy" || name.starts_with("asy") || name.starts_with("fhp") || name.contains("float") || name.starts_with("tl") || name.starts_with("dl")
    }

    /// Whether a unit type stands only on a geothermal vent (the numbers carry no such flag; the names do: `armgeo`,
    /// `corageo`, `armgmm`, `corbhmth`, `armuwgeo`). The map's vents never reach the simulator, so the roster leaves
    /// these out rather than raise a plant where no vent is.
    pub fn needs_vent(&self, unit: usize) -> bool {
        let name = self.list[unit].name.get(3..).unwrap_or("");
        name.contains("geo") || name == "gmm" || name == "bhmth"
    }

    /// The least an extractor type extracts: the tier-1 rate that a spot's metal is quoted for.
    pub fn basic_extraction(&self) -> f64 {
        self.list.iter().filter(|u| u.extracts_metal > 0.0).map(|u| u.extracts_metal).fold(f64::INFINITY, f64::min)
    }
}
