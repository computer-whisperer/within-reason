//! Construction turrets (H-ECO-NANO-GUARD). The engine gives a static builder no order of its own (escalate-7: five
//! nano turrets beside the plant, no command to any of them, idle in every sample; the user, watching: "the
//! construction turrets don't actually appear to be configured to assist the lab they were placed near"). The game's
//! UI does: BAR's ImmobileBuilder widget, on by default, sets every immobile builder to the maneuver move state and
//! gives it a fight order with the META bit at a point 50 elmos off its position toward the map's centre, under
//! which every slow update the engine's builder fight loop repairs or helps build whatever of ours stands within
//! its build distance, stationary targets first and the nearest of them, else takes apart wrecks and enemy units
//! there (`CBuilderCAI::ExecuteFight`); finding nothing, the fight ends at once (its point is within the 64 a fight
//! finishes in), which is why the order is told again every second while the turret is idle (`RETELL_FRAMES`): a
//! turret told once searches once (drill H of the rove-2 design, 2026-11-04: told once, a turret healed nothing
//! that came in reach nine seconds later). The maneuver state
//! matters: under hold position the loop skips mobile units being built, a factory's products among them. A second
//! default widget, Priority Construction Turrets, puts every such turret in reach of a newly started turret frame
//! onto that frame, so a row of turrets builds itself.
//!
//! The brain replicated both from 2026-10-30 (the user, watching econ-flagship-1: "nano turrets are currently ordered
//! to focus on a single target rather than being free to assist generally. The default configuration for a nano
//! turret from the player ui is opportunistically helpful, and we should probably figure out how to replicate that
//! rather than binding it to a facility"). From 2026-09-23 to 2026-10-30 every idle turret guarded the nearest
//! finished factory in its reach instead, which fed that factory's product and nothing else in reach.
//!
//! Since 2026-11-08 the fight loop is the fallback and the hands rank the work (the user, after econ-flagship-8 and
//! -9: "turret priority is the next lever"): the loop takes the nearest thing under construction, and a plant with a
//! product under way is always that, so the turrets in reach of a plant fed the plant while the fusion frames beside
//! them crawled (econ-flagship-9, 10:12: 2,800 build power on the old vehicle plant and 2,900 on the advanced plant,
//! a fusion at 95 % gaining under 1 % a second; the player had the plant taken apart to free them). Each turret is
//! ordered onto the highest-ranked thing in its reach by the player's `tend` order (`TEND_CATEGORIES` the default:
//! turret frames first, as the widget has it, then the frames by kind, the plants last), a frame by a repair order
//! and a plant by a guard; the fight-in-place stands when nothing ranked is in reach.

use std::collections::{HashMap, HashSet};

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitId, Vec3};

use super::{Brain, FRAMES_PER_SECOND};

/// How long an order given to a turret is trusted before an idle turret is told again: one second, past the tick
/// the snapshot lags by (an order the engine refused leaves the turret idle, so the gap is the only guard against
/// telling it every tick). It was ten seconds, and a turret that had finished a job (a turret frame it was pulled
/// onto, a product of the plant) sat idle until then: econ-flagship-5's turrets idled 1 to 9 s between jobs of 2
/// to 7, 44 % of their samples in minute 12, while the first advanced fusion crawled at the walking builders' power.
const RETELL_FRAMES: i32 = FRAMES_PER_SECOND;
/// Slack over the turret's build distance for the target's own size (the engine adds the target's radius).
const FACTORY_REACH_SLACK: f32 = 100.0;
/// The widget's offset of the fight point from the turret, toward the map's centre: past the 64 elmos within
/// which a fight order finishes, inside the map.
const BESIDE: f32 = 50.0;
/// The engine's maneuver move state.
const MANEUVER: i32 = 1;

/// The kinds a `tend` order names, in the hands' default order (H-ECO-NANO-GUARD): the game's own first (a turret
/// frame pulls the turrets in reach, the Priority Construction Turrets widget), then the frames by what they are,
/// the plants' products last. A frame of a kind not named comes after the named ones.
pub const TEND_CATEGORIES: [&str; 10] = ["turret", "advanced_fusion", "fusion", "converter", "extractor", "generator", "storage", "building", "advanced_plant", "plant"];
/// A fusion reactor costing this much metal or more is the advanced one (the advanced fusion 9,700 against the
/// fusion's 3,350 and the cloakable fusion's 4,300).
const ADVANCED_FUSION_METAL: f32 = 6000.0;

/// The turrets' state: when each was last told and what (its standing fight-in-place is `None`), and which have
/// their move state set.
#[derive(Default)]
pub(super) struct Nanos {
    /// When each turret was last told and what (`None`: its standing fight-in-place, nothing ranked in reach).
    pub(super) told: HashMap<UnitId, (i32, Option<UnitId>)>,
    set_up: HashSet<UnitId>,
}

/// The player's `tend` order parsed: the words best first, and where `never` cut it (everything from there gets no
/// order).
struct TendOrder {
    words: Vec<String>,
    never_from: usize,
}

impl TendOrder {
    fn of(words: &[String]) -> TendOrder {
        let words: Vec<String> = if words.is_empty() { TEND_CATEGORIES.iter().map(|w| w.to_string()).collect() } else { words.to_vec() };
        let never_from = words.iter().position(|w| w == "never").unwrap_or(words.len());
        TendOrder { words, never_from }
    }

    /// The rank of a candidate whose names are `names` (its kind, its handle, its actor name, its ids): the first
    /// word that names it, a kind not named ranking after every named one; None when it is on the `never` side.
    fn rank(&self, names: &[String]) -> Option<usize> {
        match self.words.iter().position(|w| names.iter().any(|n| n == w)) {
            Some(i) if i > self.never_from => None,
            Some(i) => Some(i),
            None => Some(self.never_from),
        }
    }
}

impl Brain {
    /// A builder that cannot move and makes no units: a construction turret (nano).
    pub(super) fn is_static_builder(&self, def: UnitDefId) -> bool {
        self.world.def(def).is_some_and(|d| d.speed == 0.0 && d.build_speed > 0.0 && d.build_options.is_empty())
    }

    /// Whether a turret's build distance (with the slack for the target's size) covers `target`.
    pub(super) fn turret_reaches(&self, nano: &OwnUnit, target: &OwnUnit) -> bool {
        nano.pos.dist2d(target.pos) <= self.world.def(nano.def).map_or(0.0, |d| d.build_distance) + FACTORY_REACH_SLACK
    }

    /// The `tend` kind of a building: what a turret's order ranks it as.
    pub(super) fn tend_category(&self, def: UnitDefId) -> &'static str {
        let Some(d) = self.world.def(def) else { return "building" };
        let entry = super::pianist::glossary::entry(&d.name);
        if self.world.is_factory_def(def) {
            return if entry.map_or(1, |e| e.tier) >= 2 { "advanced_plant" } else { "plant" };
        }
        let class = entry.map_or("", |e| e.class.as_str());
        if class.contains("fusion") {
            return if d.metal_cost >= ADVANCED_FUSION_METAL { "advanced_fusion" } else { "fusion" };
        }
        if d.converter.is_some() || class.contains("converter") {
            return "converter";
        }
        if d.extracts_metal > 0.0 || class.contains("extractor") {
            return "extractor";
        }
        if self.is_static_builder(def) {
            return "turret";
        }
        if d.metal_storage > 0.0 || d.energy_storage > 0.0 {
            return "storage";
        }
        if d.energy_make > 0.0 || d.wind_cap > 0.0 || d.tidal_make > 0.0 {
            return "generator";
        }
        "building"
    }

    /// The `tend` order in force: the player's, else the default.
    fn tend_order(&self) -> TendOrder {
        let words = self.strategist.as_ref().map(|s| s.tend.lock().unwrap().clone()).unwrap_or_default();
        TendOrder::of(&words)
    }

    /// The words the block line reads the order back with.
    pub(super) fn tend_words(&self) -> String {
        let order = self.tend_order();
        let mut words = order.words.clone();
        if order.never_from == words.len() {
            words.push("then any other kind".to_string());
        }
        words.join(", ")
    }

    /// The widget's fight point: 50 elmos off the turret toward the map's centre on each axis.
    fn beside(&self, at: Vec3) -> Vec3 {
        let map = &self.world.hello.map;
        let toward = |p: f32, size: f32| if p > size / 2.0 { p - BESIDE } else { p + BESIDE };
        Vec3 { x: toward(at.x, map.width), y: at.y, z: toward(at.z, map.height) }
    }

    /// Every finished construction turret is put onto the highest-ranked thing in its reach by the `tend` order
    /// (a frame by a repair order, a plant's products by a guard), the nearest among equals, once a second at most:
    /// when the thing changes, or when the turret is idle. With nothing ranked in reach it holds the game's default
    /// order, the fight-in-place beside itself, set to maneuver once; with a `never` thing in reach and nothing
    /// ranked, it is stopped instead, so the fight loop does not feed what the player forbade.
    pub(super) fn tend_nanos(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let own = &tick.snapshot.own_units;
        let order = self.tend_order();
        let turrets: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && self.is_static_builder(u.def)).collect();
        // What a turret may be put onto: our frames of buildings, and our plants with a product under way.
        let candidates: Vec<(&OwnUnit, bool, Vec<String>)> = own
            .iter()
            .filter(|u| (u.being_built && self.world.def(u.def).is_some_and(|d| d.speed == 0.0)) || (!u.being_built && !u.idle && self.world.is_factory_def(u.def)))
            .map(|u| {
                let plant = !u.being_built;
                let mut names = vec![self.tend_category(u.def).to_string(), self.handle(u), self.actor_name(u.id)];
                if let Some(pianist) = self.pianist.as_ref() {
                    names.extend(pianist.tagged.iter().filter(|(_, t)| t.building == Some(u.id)).map(|(tag, _)| format!("#{tag}")));
                }
                (u, plant, names)
            })
            .collect();
        let mut told: Vec<(UnitId, Option<UnitId>)> = Vec::new();
        for nano in turrets {
            let (told_at, told_to) = self.nanos.told.get(&nano.id).copied().unwrap_or((i32::MIN / 2, None));
            let fresh = tick.frame - told_at >= RETELL_FRAMES;
            let mut best: Option<(usize, f32, UnitId, bool)> = None;
            let mut forbidden_in_reach = false;
            for (u, plant, names) in candidates.iter().filter(|(u, ..)| self.turret_reaches(nano, u)) {
                match order.rank(names) {
                    None => forbidden_in_reach = true,
                    Some(rank) => {
                        let d = nano.pos.dist2d(u.pos);
                        if best.is_none_or(|(br, bd, ..)| rank < br || (rank == br && d < bd)) {
                            best = Some((rank, d, u.id, *plant));
                        }
                    }
                }
            }
            match best {
                Some((_, _, target, plant)) => {
                    if !fresh || (told_to == Some(target) && !nano.idle) {
                        continue;
                    }
                    if plant {
                        commands.push(Command::Guard { unit: nano.id, target });
                    } else {
                        commands.push(Command::Repair { unit: nano.id, target, queue: false });
                    }
                    told.push((nano.id, Some(target)));
                }
                None => {
                    if !nano.idle || !fresh {
                        continue;
                    }
                    if !self.nanos.set_up.contains(&nano.id) {
                        commands.push(Command::MoveState { unit: nano.id, state: MANEUVER });
                    }
                    if forbidden_in_reach {
                        commands.push(Command::Stop { unit: nano.id });
                    } else {
                        commands.push(Command::Tend { unit: nano.id, to: self.beside(nano.pos) });
                    }
                    told.push((nano.id, None));
                }
            }
        }
        for (nano, target) in told {
            self.nanos.told.insert(nano, (tick.frame, target));
            self.nanos.set_up.insert(nano);
        }
        self.nanos.told.retain(|nano, _| own.iter().any(|u| u.id == *nano));
        self.nanos.set_up.retain(|nano| own.iter().any(|u| u.id == *nano));
    }

    /// The seat's build power by kind, and what of it stands in reach of each factory now (a builder is in reach
    /// when its build range covers the factory, with the slack `nanos_on` allows).
    pub(super) fn build_power(&self, own: &[OwnUnit]) -> crate::strategist::shared::BuildPower {
        use crate::strategist::shared::{BuildPower, FactoryPower};
        let power = |u: &OwnUnit| self.world.def(u.def).map_or(0.0, |d| d.build_speed);
        let reaches = |builder: &OwnUnit, factory: &OwnUnit| builder.pos.dist2d(factory.pos) <= self.world.def(builder.def).map_or(0.0, |d| d.build_distance) + FACTORY_REACH_SLACK;
        let standing: Vec<&OwnUnit> = own.iter().filter(|u| !u.being_built && power(u) > 0.0).collect();
        let factories: Vec<&OwnUnit> = standing.iter().copied().filter(|u| self.world.is_factory_def(u.def)).collect();
        let turrets: Vec<&OwnUnit> = standing.iter().copied().filter(|u| self.is_static_builder(u.def)).collect();
        let mobile: Vec<&OwnUnit> = standing.iter().copied().filter(|u| self.world.def(u.def).is_some_and(|d| d.speed > 0.0)).collect();
        let sum = |units: &[&OwnUnit]| (units.len(), units.iter().map(|u| power(u)).fold(0.0, |a, b| a + b));
        BuildPower {
            mobile: sum(&mobile),
            turrets: sum(&turrets),
            mobile_away: sum(&mobile.iter().copied().filter(|b| !factories.iter().any(|f| reaches(b, f))).collect::<Vec<_>>()).1,
            factories: factories
                .iter()
                .map(|f| FactoryPower {
                    name: format!("{}_{}", self.world.def(f.def).map_or("factory", |d| d.name.as_str()), f.id.0),
                    own: power(f),
                    turrets: sum(&turrets.iter().copied().filter(|b| reaches(b, f)).collect::<Vec<_>>()),
                    mobile: sum(&mobile.iter().copied().filter(|b| reaches(b, f)).collect::<Vec<_>>()),
                })
                .collect(),
        }
    }

    /// (turrets within reach of this factory, of them idle) for the picture's factory entry.
    pub(super) fn nanos_on(&self, factory: &OwnUnit, own: &[OwnUnit]) -> (usize, usize) {
        let mut near = 0;
        let mut idle = 0;
        for nano in own.iter().filter(|u| !u.being_built && self.is_static_builder(u.def)) {
            if self.turret_reaches(nano, factory) {
                near += 1;
                idle += usize::from(nano.idle);
            }
        }
        (near, idle)
    }
}

#[cfg(test)]
mod tests {
    use super::super::pianist::fixtures::{at, brain_of, own};
    use bot_protocol::{Command, OwnUnit, Resource, Snapshot, Tick, UnitId};

    fn unit(brain: &super::Brain, id: i32, name: &str, x: f32, being_built: bool, idle: bool) -> OwnUnit {
        let def = brain.world.hello.unit_defs.iter().find(|d| d.name == name).unwrap().id;
        OwnUnit { being_built, idle, ..own(id, def.0, at(x, 1000.0)) }
    }

    fn tick(own: Vec<OwnUnit>, frame: i32) -> Tick {
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        Tick { frame, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: own, allies: Vec::new(), enemies: Vec::new(), wrecks: None, sensors: None, conversion: 0.0 } }
    }

    /// A turret in reach of a running plant and of fusion frames takes the frames first by the default order (the
    /// advanced fusion before the fusion, the plant last), by a repair order; with the frames gone it guards the
    /// plant; after `never plant_N` it stands stopped with the plant alone in reach.
    #[test]
    fn a_turret_takes_the_highest_ranked_thing_in_its_reach_and_the_plant_last() {
        let mut brain = brain_of(&["armcom", "armnanotc", "armavp", "armfus", "armafus"]);
        for d in brain.world.hello.unit_defs.iter_mut() {
            match d.name.as_str() {
                "armnanotc" => {
                    d.build_speed = 200.0;
                    d.build_distance = 400.0;
                }
                "armavp" => d.build_options = vec![bot_protocol::UnitDefId(1)],
                _ => {}
            }
        }
        brain.strategist = Some(std::sync::Arc::new(crate::strategist::shared::Shared::default()));
        let nano = unit(&brain, 1, "armnanotc", 1000.0, false, true);
        let plant = unit(&brain, 2, "armavp", 1100.0, false, false);
        let fus = unit(&brain, 3, "armfus", 1050.0, true, false);
        let afus = unit(&brain, 4, "armafus", 1300.0, true, false);
        assert_eq!(brain.tend_category(afus.def), "advanced_fusion");
        assert_eq!(brain.tend_category(fus.def), "fusion");
        assert_eq!(brain.tend_category(plant.def), "advanced_plant");
        let mut commands = Vec::new();
        brain.tend_nanos(&tick(vec![nano.clone(), plant.clone(), fus.clone(), afus.clone()], 100), &mut commands);
        assert!(matches!(commands.as_slice(), [Command::Repair { unit: UnitId(1), target: UnitId(4), .. }]), "{commands:?}");
        // Told a second ago and not idle: nothing new while the target stands.
        let busy = OwnUnit { idle: false, ..nano.clone() };
        commands.clear();
        brain.tend_nanos(&tick(vec![busy.clone(), plant.clone(), fus.clone(), afus.clone()], 140), &mut commands);
        assert!(commands.is_empty(), "{commands:?}");
        // The advanced fusion stands: the fusion is next, once a second has passed.
        commands.clear();
        brain.tend_nanos(&tick(vec![busy.clone(), plant.clone(), fus.clone()], 200), &mut commands);
        assert!(matches!(commands.as_slice(), [Command::Repair { unit: UnitId(1), target: UnitId(3), .. }]), "{commands:?}");
        // No frame left: the plant's products, by a guard.
        commands.clear();
        brain.tend_nanos(&tick(vec![nano.clone(), plant.clone()], 300), &mut commands);
        assert!(matches!(commands.as_slice(), [Command::Guard { unit: UnitId(1), target: UnitId(2) }]), "{commands:?}");
        // The player's order puts the plant on the never side: the turret is stopped, not left to the fight loop.
        *brain.strategist.as_ref().unwrap().tend.lock().unwrap() = vec!["fusion".into(), "never".into(), "advanced_plant".into()];
        commands.clear();
        brain.tend_nanos(&tick(vec![nano.clone(), plant.clone()], 400), &mut commands);
        assert!(commands.iter().any(|c| matches!(c, Command::Stop { unit: UnitId(1) })) && !commands.iter().any(|c| matches!(c, Command::Guard { .. })), "{commands:?}");
        // With nothing in reach at all: the game's fight-in-place.
        commands.clear();
        brain.tend_nanos(&tick(vec![nano.clone()], 500), &mut commands);
        assert!(commands.iter().any(|c| matches!(c, Command::Tend { unit: UnitId(1), .. })), "{commands:?}");
        assert!(brain.tend_words().starts_with("fusion, never, advanced_plant"));
    }
}
