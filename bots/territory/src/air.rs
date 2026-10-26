//! T-AIR (`docs/design/2026-10-25-territory-defence.md`, step 3d): anti-air turrets to the size of the air he has
//! shown, in the regions whose guarded metal covers the whole answer, the region guarding the most first, each turret
//! beside the region's building with the most of ours within the type's reach against aircraft. His aircraft are
//! remembered with his army (`observe_army`: the same memory, the dead left out); the sites join the defence wants and
//! are taken by `defence_order`. The planner does not see the turrets.

use std::collections::BTreeMap;

use bot_protocol::{UnitId, Vec3};
use buildorder::units::Role;

use crate::defence::{Gun, Kind, Want, DEFENCE_FROM, TURRET_WEIGHT};
use crate::hands::What;
use crate::{Territory, SECOND};

/// The `air:` line every this many frames.
pub(crate) const AIR_LOG: i32 = 30 * SECOND;

#[derive(Default)]
pub(crate) struct Air {
    /// His armed aircraft seen (the air shown): metal and the frame last seen.
    pub shown: BTreeMap<UnitId, (f32, i32)>,
    /// The anti-air turret types (`air_strength`), each with its reach against aircraft and its strength against
    /// them per metal; made at the start.
    pub types: BTreeMap<usize, (f32, f32)>,
    /// Regions holding something of ours passed over at the last plan as guarding less than the whole answer.
    passed: usize,
}

impl Territory {
    /// At the start: the turret types that are anti-air, logged with their strength against aircraft per metal.
    pub(crate) fn init_air(&mut self) {
        let turrets: Vec<usize> = (0..self.game().units.list.len()).filter(|&u| self.game().units.list[u].role == Role::Turret).collect();
        let mut named = Vec::new();
        for u in turrets {
            if let Some((strength, reach)) = self.air_strength(u) {
                self.air.types.insert(u, (reach, strength));
                named.push(format!("{} {strength:.2} (reach {reach:.0}, {:.0} metal)", self.game().units.list[u].name, self.hello.unit_defs[u].metal_cost));
            }
        }
        self.log(format_args!("air: anti-air turrets {}", if named.is_empty() { "none".to_string() } else { named.join(", ") }));
    }

    pub(crate) fn is_anti_air(&self, u: usize) -> bool {
        self.air.types.contains_key(&u)
    }

    /// The air he has shown, in metal.
    fn air_shown(&self) -> f32 {
        self.air.shown.values().fold(0.0, |sum, (m, _)| sum + m)
    }

    /// The anti-air types a builder type builds: (type, strength against aircraft per metal, metal).
    fn air_types_of(&self, builder: usize) -> Vec<(usize, f32, f32)> {
        let units = &self.game().units;
        units.list[builder].builds.iter().filter_map(|u| self.air.types.get(u).map(|(_, s)| (*u, *s, units.list[*u].metal_cost as f32))).collect()
    }

    /// The builder type's anti-air turret for a want of `need` metal: the most strength against aircraft per metal
    /// among its anti-air types that cost no more than that; none when it makes no such type, and the want is left to
    /// a builder that does. Always the best per metal, a turret was many times the answer: two bombers seen (300
    /// metal, an answer of 100) bought a 750-820 metal turret in every region, 5,300-8,600 metal a game (terr324); with
    /// its cheapest type when none fit, the advanced constructors still put their 820-metal flak where 250 was
    /// wanted (terr325).
    pub(crate) fn air_type(&self, builder: usize, need: f32) -> Option<usize> {
        self.air_types_of(builder).into_iter().filter(|(_, _, m)| *m <= need).max_by(|a, b| a.1.total_cmp(&b.1)).map(|(u, _, _)| u)
    }

    /// Our anti-air turrets standing, framed and ordered.
    fn air_guns(&self) -> Vec<Gun> {
        let units = &self.game().units;
        let mut guns = Vec::new();
        for u in self.own.values() {
            if let Some(i) = self.unit_index(u.def)
                && let Some((reach, _)) = self.air.types.get(&i)
            {
                guns.push(Gun { at: u.pos, reach: *reach, metal: units.list[i].metal_cost as f32 });
            }
        }
        for order in self.hands.orders.values() {
            if let What::Build { unit, near, frame: None, .. } = &order.what
                && let Some((reach, _)) = self.air.types.get(unit)
            {
                guns.push(Gun { at: *near, reach: *reach, metal: units.list[*unit].metal_cost as f32 });
            }
        }
        guns
    }

    /// Every hands tick, after T-COVER's: the anti-air sites wanted (T-AIR), into the defence wants.
    pub(crate) fn plan_air(&mut self) {
        self.air.passed = 0;
        let shown = self.air_shown();
        if self.frame < DEFENCE_FROM || !self.options.air || shown <= 0.0 || self.regions.is_none() {
            return;
        }
        // The anti-air turrets our builders make: the cheapest's metal for the cap, the shortest reach for the site.
        let builders: Vec<usize> = self.own.values().filter(|u| !u.being_built).filter_map(|u| self.unit_index(u.def)).filter(|i| matches!(self.game().units.list[*i].role, Role::Builder | Role::Commander)).collect();
        let kinds: Vec<(usize, f32, f32)> = builders.into_iter().flat_map(|b| self.air_types_of(b)).collect();
        let Some(least) = kinds.iter().map(|(_, _, m)| *m).reduce(f32::min) else { return };
        let reach = kinds.iter().map(|(u, _, _)| self.air.types[u].0).fold(f32::INFINITY, f32::min);
        let guarded = self.guarded();
        let everything: Vec<(Vec3, f32)> = guarded.iter().flatten().copied().collect();
        let guns = self.air_guns();
        let mut gun_metal = vec![0.0f32; guarded.len()];
        for g in &guns {
            if let Some(k) = self.region_of(g.at) {
                gun_metal[k] += g.metal;
            }
        }
        let answer = shown / TURRET_WEIGHT;
        let metal: Vec<f32> = guarded.iter().map(|here| here.iter().map(|(_, w)| w).sum()).collect();
        let mut regions: Vec<usize> = (0..guarded.len()).filter(|&k| metal[k] >= 1.0).collect();
        regions.sort_by(|a, b| metal[*b].total_cmp(&metal[*a]));
        let mut wants = Vec::new();
        for k in regions {
            // Answered whole or not at all: his air strikes one place with all of it.
            if metal[k] < answer {
                self.air.passed += 1;
                continue;
            }
            if gun_metal[k] * TURRET_WEIGHT >= shown || gun_metal[k] + least > metal[k] {
                continue;
            }
            // Beside the region's building with the most of ours in reach, so that the turrets overlap.
            let within = |p: Vec3| everything.iter().filter(|(q, _)| q.dist2d(p) <= reach).map(|(_, w)| w).sum::<f32>();
            let Some((at, _)) = guarded[k].iter().max_by(|a, b| within(a.0).total_cmp(&within(b.0))) else { continue };
            if self.site_clear(*at, false) {
                // The shortfall, and never less than the cheapest type any of our builders makes.
                wants.push(Want { kind: Kind::AntiAir { need: (answer - gun_metal[k]).max(least) }, at: *at, region: k });
            }
        }
        self.defence.wants.extend(wants);
    }

    /// `air:` every AIR_LOG: the air shown and its types, the whole answer, our anti-air standing, framed and ordered by
    /// region, the regions passed over, and the sites wanted.
    pub(crate) fn log_air(&mut self) {
        let mut types: BTreeMap<String, usize> = BTreeMap::new();
        for e in &self.enemies {
            if self.air.shown.contains_key(&e.id) && let Some(info) = e.def.and_then(|d| self.def(d)) {
                *types.entry(info.name.clone()).or_default() += 1;
            }
        }
        let seen: Vec<String> = types.iter().map(|(n, c)| format!("{c} {n}")).collect();
        let guns = self.air_guns();
        let mut by_region: BTreeMap<usize, (usize, f32)> = BTreeMap::new();
        for g in &guns {
            let entry = by_region.entry(self.region_of(g.at).unwrap_or(usize::MAX)).or_default();
            entry.0 += 1;
            entry.1 += g.metal;
        }
        let standing = self.own.values().filter(|u| !u.being_built && self.unit_index(u.def).is_some_and(|i| self.is_anti_air(i))).count();
        let regions: Vec<String> = by_region.iter().map(|(k, (n, m))| format!("r{k} {n} ({m:.0} metal)")).collect();
        let wanted: Vec<String> = self.defence.wants.iter().filter(|w| matches!(w.kind, Kind::AntiAir { .. })).map(|w| format!("r{} ({:.0},{:.0})", w.region, w.at.x, w.at.z)).collect();
        self.log(format_args!(
            "air: shown {:.0} ({} units{}{}), answer {:.0} | anti-air {} standing of {} ({:.0} metal) {} | passed over {} regions | wanted {}",
            self.air_shown(),
            self.air.shown.len(),
            if seen.is_empty() { "" } else { " in sight: " },
            seen.join(", "),
            self.air_shown() / TURRET_WEIGHT,
            standing,
            guns.len(),
            guns.iter().fold(0.0, |sum, g| sum + g.metal),
            if regions.is_empty() { "-".to_string() } else { regions.join(", ") },
            self.air.passed,
            if wanted.is_empty() { "-".to_string() } else { wanted.join(" ") },
        ));
    }
}
