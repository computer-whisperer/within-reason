//! Geothermal vents (H-HANDS-VENTS): where the map's are, what stands on each, and which of them a vent building may
//! be ordered onto. A vent is a place (`vent_N`, numbered west to east from 1), and a building that needs one is
//! sited on it exactly, as an extractor is on its spot: the base layout knows no vents and the engine refuses any
//! other ground (five advanced plants were ordered onto ordinary ground and refused in three games before
//! 2026-11-15; no seat of ours had built one).

use bot_protocol::{OwnUnit, UnitDefId, UnitId, Vec3};

use super::Brain;

/// How near a vent a building on it stands: the engine lets its centre lie within 36 elmos of the vent on each axis.
pub(super) const VENT_HELD: f32 = 60.0;

/// What stands on a vent, as far as this seat knows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Holder {
    Free,
    Ours { id: UnitId, def: UnitDefId, being_built: bool },
    Ally { def: UnitDefId, being_built: bool },
    /// His, with the frame it was last seen standing.
    His { def: UnitDefId, seen: i32 },
}

/// How a vent building is begun on a vent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Take {
    /// On the bare vent.
    Free,
    /// Over the tier-1 plant of ours standing there: the game destroys the plant beneath when the new one is
    /// finished and gives back its metal (`unit_geo_upgrade_reclaimer.lua`).
    Over(UnitDefId),
    /// The same building is already framed there: it is helped to finish, not ordered again.
    UnderWay(UnitId),
}

impl Brain {
    pub(super) fn vent_name(index: usize) -> String {
        format!("vent_{}", index + 1)
    }

    /// The vent a place name stands for.
    pub(super) fn vent_named(&self, name: &str) -> Option<(usize, Vec3)> {
        let index = name.strip_prefix("vent_")?.parse::<usize>().ok()?.checked_sub(1)?;
        self.world.vents.get(index).map(|at| (index, *at))
    }

    /// What stands on the vent: ours first (the costlier of two while one is built over the other), then an ally's,
    /// then his as last seen.
    pub(super) fn vent_holder(&self, vent: Vec3, own: &[OwnUnit]) -> Holder {
        let on_it = |def: UnitDefId, pos: Vec3| pos.dist2d(vent) < VENT_HELD && self.world.needs_vent(def);
        let cost = |def: UnitDefId| self.world.def(def).map_or(0.0, |d| d.metal_cost);
        if let Some(u) = own.iter().filter(|u| on_it(u.def, u.pos)).max_by(|a, b| cost(a.def).total_cmp(&cost(b.def))) {
            return Holder::Ours { id: u.id, def: u.def, being_built: u.being_built };
        }
        if let Some(a) = self.allies.iter().filter(|a| on_it(a.def, a.pos)).max_by(|a, b| cost(a.def).total_cmp(&cost(b.def))) {
            return Holder::Ally { def: a.def, being_built: a.being_built };
        }
        match self.enemy_buildings.values().filter(|(def, pos, _)| on_it(*def, *pos)).max_by_key(|(_, _, seen)| *seen) {
            Some((def, _, seen)) => Holder::His { def: *def, seen: *seen },
            None => Holder::Free,
        }
    }

    /// Whether the vent's ground is ground the building stands on: a land plant on a dry vent, an offshore one on a
    /// vent under water (the unit's own water band, as the engine asks it of every square).
    fn vent_ground_fits(&self, def: UnitDefId, vent: Vec3) -> bool {
        let depth = -vent.y;
        self.world.def(def).is_some_and(|d| depth <= d.max_water_depth && (d.min_water_depth <= 0.0 || depth >= d.min_water_depth))
    }

    /// How `def` could be begun on the vent now, if at all. A bare vent takes any vent building its ground fits. A
    /// vent holding a finished plant of ours takes a building of at least twice the plant's metal: every tier-2
    /// vent building of the three factions (the advanced plant, the Prude, the Cerberus, the Rampart) is laid out
    /// to stand over the tier-1 plant (their yardmaps' stackable squares lie on the plant's blocked ones), and
    /// nothing stands over a tier-2 one or over its own kind. An ally's vent and his are not ours to take: a plant
    /// built over an ally's becomes the ally's.
    pub(super) fn vent_take(&self, def: UnitDefId, vent: Vec3, own: &[OwnUnit]) -> Option<Take> {
        if !self.vent_ground_fits(def, vent) {
            return None;
        }
        let cost = |d: UnitDefId| self.world.def(d).map_or(0.0, |x| x.metal_cost);
        match self.vent_holder(vent, own) {
            Holder::Free => Some(Take::Free),
            Holder::Ours { id, def: standing, being_built: true } if standing == def => Some(Take::UnderWay(id)),
            Holder::Ours { def: standing, being_built: false, .. } if cost(def) >= 2.0 * cost(standing) => Some(Take::Over(standing)),
            _ => None,
        }
    }

    /// The vents `def` may be begun on, the one nearest `near` first, each with its index.
    pub(super) fn vents_for(&self, def: UnitDefId, near: Vec3, own: &[OwnUnit]) -> Vec<(usize, Vec3, Take)> {
        let mut takes: Vec<(usize, Vec3, Take)> = self.world.vents.iter().enumerate().filter_map(|(i, v)| self.vent_take(def, *v, own).map(|t| (i, *v, t))).collect();
        takes.sort_by(|a, b| a.1.dist2d(near).total_cmp(&b.1.dist2d(near)));
        takes
    }

    /// The map's vents for the `map` tool and the first report: each with its name, its place and the walk from our
    /// start (null where our bots cannot walk). What stands on each is the report's, turn by turn.
    pub(super) fn vents_description(&self) -> serde_json::Value {
        serde_json::Value::Array(
            self.world
                .vents
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let walk = self.reachable_on_foot(*v).then(|| self.walk_from_home(*v) as i32);
                    let mut entry = serde_json::json!({ "name": Self::vent_name(i), "grid": self.world.grid(*v), "x": v.x as i32, "z": v.z as i32, "walk_from_home": walk, "straight_from_home": v.dist2d(self.home) as i32 });
                    if v.y < 0.0 {
                        entry["under_water"] = serde_json::json!(true);
                    }
                    entry
                })
                .collect(),
        )
    }

    /// The same for the published map: its `vents` and the note that says how one is taken.
    pub(super) fn describe_vents(&self, map: &mut serde_json::Value) {
        map["vents"] = self.vents_description();
        map["vents_note"] = serde_json::json!(if self.world.vents.is_empty() {
            "no geothermal vent has been seen on this map: a geothermal plant stands only on one, so none can be built here"
        } else {
            "geothermal vents: the only ground a geothermal plant stands on (the plants are in your unit tables). vent_N is a place: `geothermal vent_2` in a queue list builds the tier-1 plant there (a constructor's; the commander builds none), `advanced_geothermal vent_2` the advanced one (an advanced constructor's), which may also go over our own tier-1 plant: the game takes the old one away when the new one is finished and gives its metal back. The bare word takes the vent nearest the builder that it may be begun on. The score line of every report says what stands on each vent"
        });
    }

    /// The vents for the report's score line, nearest our start first: each vent's name, cell and what stands on
    /// it, a free one with the walk from our start.
    pub(super) fn vents_report(&self, own: &[OwnUnit], frame: i32) -> Vec<(String, String)> {
        let mut vents: Vec<(f32, String, String)> = self
            .world
            .vents
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let mut words = self.vent_words(*v, own, frame);
                if self.vent_holder(*v, own) == Holder::Free {
                    words += &if self.reachable_on_foot(*v) { format!(", {:.0} on foot", self.walk_from_home(*v)) } else { ", off the ground our bots walk".to_string() };
                }
                (v.dist2d(self.home), format!("{} {}", Self::vent_name(i), self.world.grid(*v)), words)
            })
            .collect();
        vents.sort_by(|a, b| a.0.total_cmp(&b.0));
        vents.into_iter().map(|(_, name, words)| (name, words)).collect()
    }

    /// The vents told to the player when the shim's list arrives after the map was published (a map whose Lua
    /// places its vents once the game runs): the published map is given them, the costly sheet left as it is.
    pub(super) fn publish_vents(&self) {
        let Some(shared) = &self.strategist else { return };
        let patch = |map: &mut serde_json::Value| {
            if map.get("sheet").is_some() {
                self.describe_vents(map);
            }
        };
        if let Some(side) = shared.side.get() {
            patch(&mut side.map.lock().unwrap());
        }
        patch(&mut shared.map.lock().unwrap());
    }

    /// What stands on the vent, in the words a place's description and the report use.
    pub(super) fn vent_words(&self, vent: Vec3, own: &[OwnUnit], frame: i32) -> String {
        let built = |being_built: bool| if being_built { " (being built)" } else { "" };
        match self.vent_holder(vent, own) {
            Holder::Free => "free".to_string(),
            Holder::Ours { def, being_built, .. } => format!("our {}{}", self.short_words(def), built(being_built)),
            Holder::Ally { def, being_built } => format!("an ally's {}{}", self.short_words(def), built(being_built)),
            Holder::His { def, seen } => format!("his {}, seen {} ago", self.short_words(def), super::pianist::picture::clock(frame - seen)),
        }
    }
}

#[cfg(test)]
mod tests {
    use bot_protocol::{OwnUnit, UnitDefId, UnitId};

    use super::super::economy::Plan;
    use super::super::pianist::fixtures::{at, brain_of, own};
    use super::*;

    const CON: i32 = 1;
    const GEO: i32 = 2;
    const AGEO: i32 = 3;
    const GMM: i32 = 4;

    /// A brain whose map has two vents, 1,000 and 5,000 east of the west edge, and whose plants need one.
    fn brain() -> Brain {
        let mut brain = brain_of(&["armck", "armgeo", "armageo", "armgmm", "armsolar"]);
        for d in brain.world.hello.unit_defs.iter_mut() {
            d.needs_vent = d.name.contains("geo") || d.name == "armgmm";
            d.max_water_depth = 0.0;
            d.min_water_depth = -1e7;
        }
        brain.world.vents = vec![at(1000.0, 1000.0), at(5000.0, 1000.0)];
        brain.home = at(800.0, 800.0);
        brain
    }

    /// A bare vent takes any plant; a vent with our finished tier-1 plant takes the advanced plant and the Prude
    /// over it and no second tier-1 plant; a tier-2 building on a vent leaves it to nothing; the same building
    /// framed there is helped; an ally's and his are not ours to take.
    #[test]
    fn a_vent_takes_a_plant_bare_or_a_tier_2_one_over_our_tier_1() {
        let mut brain = brain();
        let vent = brain.world.vents[0];
        let (geo, ageo, gmm) = (UnitDefId(GEO), UnitDefId(AGEO), UnitDefId(GMM));
        assert_eq!(brain.vent_take(geo, vent, &[]), Some(Take::Free));
        assert_eq!(brain.vent_take(ageo, vent, &[]), Some(Take::Free));
        // The plant stands a few squares off the vent, as the engine's grid puts it.
        let plant = own(10, GEO, at(1008.0, 992.0));
        assert_eq!(brain.vent_holder(vent, &[plant.clone()]), Holder::Ours { id: UnitId(10), def: geo, being_built: false });
        assert_eq!(brain.vent_take(geo, vent, &[plant.clone()]), None);
        assert_eq!(brain.vent_take(ageo, vent, &[plant.clone()]), Some(Take::Over(geo)));
        assert_eq!(brain.vent_take(gmm, vent, &[plant.clone()]), Some(Take::Over(geo)));
        // Unfinished, the tier-1 plant is helped by its own kind and built over by nothing.
        let frame = OwnUnit { being_built: true, ..plant.clone() };
        assert_eq!(brain.vent_take(geo, vent, &[frame.clone()]), Some(Take::UnderWay(UnitId(10))));
        assert_eq!(brain.vent_take(ageo, vent, &[frame]), None);
        // While the advanced plant is framed over the tier-1 one, the vent is the advanced plant's.
        let over = OwnUnit { being_built: true, ..own(11, AGEO, at(1008.0, 992.0)) };
        assert_eq!(brain.vent_take(ageo, vent, &[plant.clone(), over.clone()]), Some(Take::UnderWay(UnitId(11))));
        assert_eq!(brain.vent_take(gmm, vent, &[plant.clone(), over]), None);
        // A finished advanced plant, or a Prude: nothing goes over it (the advanced plant is not twice a Prude).
        assert_eq!(brain.vent_take(ageo, vent, &[own(12, AGEO, vent)]), None);
        assert_eq!(brain.vent_take(ageo, vent, &[own(13, GMM, vent)]), None);
        // The nearest vent a plant may be begun on comes first; a taken one is passed over.
        assert_eq!(brain.vents_for(geo, at(900.0, 900.0), &[]).iter().map(|(i, ..)| *i).collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(brain.vents_for(geo, at(900.0, 900.0), &[plant]).iter().map(|(i, ..)| *i).collect::<Vec<_>>(), vec![1]);
        // His plant, as last seen.
        brain.enemy_buildings.insert(UnitId(90), (geo, at(5000.0, 1008.0), 600));
        assert_eq!(brain.vent_holder(brain.world.vents[1], &[]), Holder::His { def: geo, seen: 600 });
        assert_eq!(brain.vent_take(ageo, brain.world.vents[1], &[]), None);
        assert!(brain.vent_words(brain.world.vents[1], &[], 1500).starts_with("his "), "{}", brain.vent_words(brain.world.vents[1], &[], 1500));
        // An under-water vent takes no land plant.
        brain.world.vents[0].y = -40.0;
        assert_eq!(brain.vent_take(geo, brain.world.vents[0], &[]), None);
    }

    /// The names: `vent_N` from 1, west to east as the shim sends them; anything else is no vent.
    #[test]
    fn a_vent_is_a_place_by_its_number() {
        let brain = brain();
        assert_eq!(Brain::vent_name(0), "vent_1");
        assert_eq!(brain.vent_named("vent_2").map(|(i, v)| (i, v.x)), Some((1, 5000.0)));
        assert!(brain.vent_named("vent_0").is_none() && brain.vent_named("vent_3").is_none() && brain.vent_named("spot_1").is_none());
    }

    /// The order's site is the vent itself, exactly: the shim asks the engine for that ground alone.
    #[test]
    fn a_vent_plans_site_is_the_vent() {
        let mut brain = brain();
        if let Some(d) = brain.world.hello.unit_defs.iter_mut().find(|d| d.id == UnitDefId(CON)) {
            d.build_options.push(UnitDefId(GEO));
        }
        let builder = own(1, CON, at(900.0, 900.0));
        let id = UnitDefId(9);
        let kit = super::super::roster::Kit { commander: id, extractor: id, converter: id, lab: id, turret: id, constructor: id, vehicle_constructor: id, advanced_constructor: id, advanced_extractor: id, raider: id, line: id, resurrector: id, roster: &super::super::roster::ROSTERS[0] };
        let (def, site) = brain.build_site_for(&Plan::Vent(UnitDefId(GEO), brain.world.vents[0]), &builder, &[builder.clone()], &kit).expect("a site");
        assert_eq!((def, site.near.x, site.near.z, site.exact), (UnitDefId(GEO), 1000.0, 1000.0, true));
        // A builder that cannot make it has no site.
        assert!(brain.build_site_for(&Plan::Vent(UnitDefId(AGEO), brain.world.vents[0]), &builder, &[builder.clone()], &kit).is_none());
    }
}
