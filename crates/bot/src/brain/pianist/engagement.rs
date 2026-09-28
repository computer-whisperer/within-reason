//! Stage 2 of the micro answer (`docs/design/2026-09-28-engagement-plan.md`, H-HANDS-ENGAGEMENT-PLAN): a body that
//! comes within `PLAN_REACH` of a position (two or more armed enemy elements within `LINK` of each other) is given
//! an order of operations. Code finds the position, prices each element with its cover and finds where our reach
//! reaches it from outside every other reach (the battlefield), enumerates the candidate plans, and Jev's choice
//! question picks one; the body executes it phase by phase as its task.
//!
//! The TAS of player-14's E3 fight (K-micro-the-front-stops-on-the-nearest-and-blocks-the-rest): the position fell
//! 24 of 24 to the screen first from outside turret cover and the turrets last, 5 of 24 to the turrets first, 21 of
//! 24 to attack-move and 0 of 24 to the recorded orders; the pass offered the whole-group attack on the nearest party
//! and the body met the screen and the turrets at once.

use std::collections::BTreeMap;

use bot_protocol::{EnemyUnit, OwnUnit, UnitDefId, UnitId, Vec3};
use serde_json::{Value, json};

use super::super::combat::Force;
use super::super::routes::Walker;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::picture::{Party, Place};

/// A body whose front comes this close to an element of a position plans the engagement.
pub(crate) const PLAN_REACH: f32 = 1200.0;
/// Armed elements this close to each other (their nearest members) are one position.
const LINK: f32 = 600.0;
/// A plan is asked again for the same position after this long.
pub(crate) const PLAN_STALE: i32 = 45 * FRAMES_PER_SECOND;
/// The stand-off point stands this far inside our shortest reach from the element.
const STANDOFF_SLACK: f32 = 20.0;
/// Directions tried round each member for a stand-off point.
const STANDOFF_DIRECTIONS: usize = 36;
/// A mobile element within this of a static is its screen.
const SCREEN_REACH: f32 = 400.0;
/// A body whose tail is within this of its front is gathered.
pub(crate) const GATHERED: f32 = 300.0;
/// A contact of unknown type is taken to reach this far (a tier-1 soldier's 300-350).
const UNIDENTIFIED_REACH: f32 = 300.0;
/// A member with this reach or more is artillery (as the pass's `shell` state has it).
pub(crate) const ARTILLERY_REACH: f32 = 600.0;

/// One unit of an element: where it stands, how far it reaches, how hurt it is (a share, when the type is known).
#[derive(Clone, Debug)]
pub(crate) struct Member {
    pub id: UnitId,
    pub def: Option<UnitDefId>,
    pub at: Vec3,
    pub reach: f32,
    pub health: Option<f32>,
}

/// An armed enemy element: a party in sight with an armed member, or one armed building (seen or remembered).
#[derive(Clone, Debug)]
pub(crate) struct Element {
    /// `party_N` as the picture names it, or `turret_<id>` for a building.
    pub name: String,
    pub fixed: bool,
    pub members: Vec<Member>,
    pub at: Vec3,
    pub metal: f32,
    /// "3 Centurion (armwar)".
    pub composition: String,
    /// Its longest member's reach.
    pub reach: f32,
    /// A building in sight now (not only remembered): what a spotter gives artillery.
    pub in_sight: bool,
}

impl Element {
    fn distance_to(&self, at: Vec3) -> f32 {
        self.members.iter().map(|m| m.at.dist2d(at)).fold(f32::INFINITY, f32::min)
    }

    fn distance_to_element(&self, other: &Element) -> f32 {
        self.members.iter().map(|m| other.distance_to(m.at)).fold(f32::INFINITY, f32::min)
    }

    /// Whether any member's reach covers the point.
    pub(crate) fn reaches(&self, at: Vec3) -> bool {
        self.members.iter().any(|m| m.at.dist2d(at) <= m.reach)
    }
}

/// Two or more armed elements within `LINK` of each other, at least one static or their metal above a third of the
/// body's.
#[derive(Clone, Debug)]
pub(crate) struct Position {
    pub elements: Vec<Element>,
}

impl Position {
    /// The elements by name, sorted: a plan is for one signature, and an element appearing or dying changes it.
    pub(crate) fn signature(&self) -> String {
        let mut names: Vec<&str> = self.elements.iter().map(|e| e.name.as_str()).collect();
        names.sort_unstable();
        names.join(",")
    }

    pub(crate) fn distance_to(&self, at: Vec3) -> f32 {
        self.elements.iter().map(|e| e.distance_to(at)).fold(f32::INFINITY, f32::min)
    }

    pub(crate) fn centre(&self) -> Vec3 {
        let n = self.elements.iter().map(|e| e.members.len()).sum::<usize>().max(1) as f32;
        self.elements.iter().flat_map(|e| e.members.iter()).fold(Vec3::default(), |s, m| Vec3 { x: s.x + m.at.x / n, y: 0.0, z: s.z + m.at.z / n })
    }

    /// Every element but `index` whose reach covers the point.
    pub(crate) fn reaching(&self, at: Vec3, but: Option<usize>) -> Vec<usize> {
        (0..self.elements.len()).filter(|i| Some(*i) != but && self.elements[*i].reaches(at)).collect()
    }
}

/// Our body as the battlefield says it.
#[derive(Clone, Debug)]
pub(crate) struct Ours {
    pub group: String,
    pub ids: Vec<UnitId>,
    pub force: Force,
    pub front: Vec3,
    pub centre: Vec3,
    /// From the front to the tail.
    pub length: f32,
    /// The shortest and longest ground reach among its armed members, with the type that has it.
    pub short_reach: (f32, String),
    pub long_reach: (f32, String),
    /// Its slowest member's speed.
    pub speed: f32,
    pub metal: f32,
    /// "14 Blitz (armflash)".
    pub composition: String,
    /// "12 of 14 at full, 1 under half".
    pub health: String,
    /// Its members with `ARTILLERY_REACH` or more.
    pub artillery: Vec<UnitId>,
    pub walker: Walker,
}

impl Ours {
    pub(crate) fn gathered(&self) -> bool {
        self.length <= GATHERED
    }

    pub(crate) fn walk_seconds(&self, distance: f32) -> f32 {
        if self.speed > 0.0 { distance / self.speed } else { 0.0 }
    }
}

/// What code knows of one element in the battlefield.
#[derive(Clone, Debug)]
pub(crate) struct Facts {
    /// The statics (indices into the position) whose reach it stands inside.
    pub cover: Vec<usize>,
    /// A point at our shortest reach less `STANDOFF_SLACK` from one of its members that lies outside every other
    /// element's reach, nearest our front: the point and the member it reaches.
    pub stand_off: Option<(Vec3, UnitId)>,
    /// Mobile, within `SCREEN_REACH` of a static.
    pub screen: bool,
    /// Our body's odds against it alone, and against it with its cover.
    pub odds_alone: f32,
    pub odds_covered: f32,
    pub distance: f32,
}

/// The battlefield: our body, the position, and per element what code knows.
#[derive(Clone, Debug)]
pub(crate) struct Battlefield {
    pub ours: Ours,
    pub position: Position,
    pub facts: Vec<Facts>,
    /// Our body's odds against the whole position.
    pub odds_whole: f32,
}

/// The verdict of a power ratio, in the picture's words.
pub(crate) fn verdict(ratio: f32) -> &'static str {
    if ratio >= 2.5 {
        "we outweigh it heavily"
    } else if ratio >= 1.3 {
        "we outweigh it"
    } else if ratio >= 0.8 {
        "an even fight"
    } else {
        "it outweighs us"
    }
}

impl Brain {
    /// The armed elements a ground body can meet: every party in sight with an armed member (a radar contact counts
    /// as armed) and every finished armed building that hits ground, in sight or remembered.
    pub(crate) fn armed_elements(&self, parties: &[Party], enemies: &[EnemyUnit]) -> Vec<Element> {
        let reach_of = |def: Option<UnitDefId>| def.and_then(|d| self.world.def(d)).map_or(UNIDENTIFIED_REACH, |d| d.reach);
        let health_of = |def: Option<UnitDefId>, health: f32| {
            let full = def.and_then(|d| self.sim.defs.get(&d)).map(|i| self.sim.rules.units.list[*i].health)?;
            (full > 0.0).then(|| (health / full).clamp(0.0, 1.0))
        };
        let mut out = Vec::new();
        for party in parties {
            let seen: Vec<&EnemyUnit> = enemies.iter().filter(|e| party.ids.contains(&e.id)).collect();
            let armed = seen.iter().any(|e| e.def.is_none_or(|d| self.world.def(d).is_some_and(|x| x.weapon_count > 0)));
            if seen.is_empty() || !armed {
                continue;
            }
            let members: Vec<Member> = seen.iter().map(|e| Member { id: e.id, def: e.def, at: e.pos, reach: reach_of(e.def), health: health_of(e.def, e.health) }).collect();
            let reach = members.iter().map(|m| m.reach).fold(0.0, f32::max);
            out.push(Element { name: party.name.clone(), fixed: false, at: party.at, metal: party.metal, composition: self.composition_of(&members), reach, members, in_sight: true });
        }
        let mut buildings: Vec<(&UnitId, &(UnitDefId, Vec3, i32))> = self.enemy_buildings.iter().collect();
        buildings.sort_by_key(|(id, _)| id.0);
        for (id, (def, pos, _)) in buildings {
            let Some(d) = self.world.def(*def) else { continue };
            if d.weapon_count == 0 || d.reach <= 0.0 || self.enemy_unfinished.contains(id) || !self.can_hit(*def, false) {
                continue;
            }
            let seen = enemies.iter().find(|e| e.id == *id);
            let member = Member { id: *id, def: Some(*def), at: *pos, reach: d.reach, health: seen.and_then(|e| health_of(Some(*def), e.health)) };
            out.push(Element { name: format!("turret_{}", id.0), fixed: true, at: *pos, metal: d.metal_cost, composition: format!("1 {}", self.short_words(*def)), reach: d.reach, members: vec![member], in_sight: seen.is_some() });
        }
        out
    }

    fn composition_of(&self, members: &[Member]) -> String {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for m in members {
            *counts.entry(m.def.map_or("unidentified contact".to_string(), |d| self.short_words(d))).or_default() += 1;
        }
        counts.iter().map(|(n, k)| format!("{k} {n}")).collect::<Vec<_>>().join(", ")
    }

    /// The positions among the elements: linked at `LINK` by their nearest members; two or more elements, at least
    /// one static or their metal above a third of the body's.
    pub(crate) fn positions(elements: Vec<Element>, body_metal: f32) -> Vec<Position> {
        let mut cluster: Vec<Option<usize>> = vec![None; elements.len()];
        let mut clusters: Vec<Vec<usize>> = Vec::new();
        for start in 0..elements.len() {
            if cluster[start].is_some() {
                continue;
            }
            cluster[start] = Some(clusters.len());
            let mut members = vec![start];
            let mut next = 0;
            while next < members.len() {
                let from = members[next];
                for other in 0..elements.len() {
                    if cluster[other].is_none() && elements[from].distance_to_element(&elements[other]) <= LINK {
                        cluster[other] = Some(clusters.len());
                        members.push(other);
                    }
                }
                next += 1;
            }
            clusters.push(members);
        }
        let mut slots: Vec<Option<Element>> = elements.into_iter().map(Some).collect();
        clusters
            .into_iter()
            .filter_map(|members| {
                let elements: Vec<Element> = members.into_iter().filter_map(|i| slots[i].take()).collect();
                let metal: f32 = elements.iter().map(|e| e.metal).sum();
                (elements.len() >= 2 && (elements.iter().any(|e| e.fixed) || metal > body_metal / 3.0)).then_some(Position { elements })
            })
            .collect()
    }

    /// The force of these elements as `combat.rs` prices it: the soldiers by type, a radar contact as a Pawn, a
    /// building's metal as turret metal.
    pub(crate) fn force_of_elements<'a>(&self, elements: impl IntoIterator<Item = &'a Element>) -> Force {
        let mut force = Force::default();
        for e in elements {
            for m in &e.members {
                match (e.fixed, m.def) {
                    (true, Some(def)) => {
                        let metal = self.world.def(def).map_or(0.0, |d| d.metal_cost);
                        if self.can_hit(def, false) {
                            force.turret_metal += metal;
                        }
                        if self.can_hit(def, true) {
                            force.turret_metal_air += metal;
                        }
                    }
                    (_, Some(def)) => force.add(def),
                    (_, None) => force.unidentified += 1,
                }
            }
        }
        force
    }

    /// Our body as the battlefield says it: the core members (not those still joining), its front toward `toward`.
    pub(crate) fn ours_of(&self, group: &str, units: &[&OwnUnit], toward: Vec3, walker: Walker) -> Option<Ours> {
        if units.is_empty() {
            return None;
        }
        let mut sorted: Vec<&OwnUnit> = units.to_vec();
        sorted.sort_by(|a, b| a.pos.dist2d(toward).total_cmp(&b.pos.dist2d(toward)));
        let front = sorted[0].pos;
        let tail = sorted[sorted.len() - 1].pos;
        let centre = super::groups::centre_of(&sorted).unwrap_or(front);
        let armed: Vec<(f32, String)> = sorted.iter().filter_map(|u| self.world.def(u.def).filter(|d| d.weapon_count > 0 && d.reach > 0.0).map(|d| (d.reach, self.short_words(u.def)))).collect();
        let short_reach = armed.iter().cloned().min_by(|a, b| a.0.total_cmp(&b.0)).unwrap_or((0.0, String::new()));
        let long_reach = armed.iter().cloned().max_by(|a, b| a.0.total_cmp(&b.0)).unwrap_or((0.0, String::new()));
        let speed = sorted.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.speed).filter(|s| *s > 0.0).fold(f32::INFINITY, f32::min);
        let metal = sorted.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for u in &sorted {
            *counts.entry(self.short_words(u.def)).or_default() += 1;
        }
        let share = |u: &&OwnUnit| u.health / u.max_health.max(1.0);
        let full = sorted.iter().filter(|u| share(u) >= 0.95).count();
        let half = sorted.iter().filter(|u| share(u) < 0.5).count();
        let health = format!("{full} of {} at full{}", sorted.len(), if half > 0 { format!(", {half} under half") } else { String::new() });
        let artillery = sorted.iter().filter(|u| self.world.def(u.def).is_some_and(|d| d.reach >= ARTILLERY_REACH && d.speed > 0.0)).map(|u| u.id).collect();
        Some(Ours {
            group: group.to_string(),
            ids: sorted.iter().map(|u| u.id).collect(),
            force: Brain::force_of(&sorted),
            front,
            centre,
            length: front.dist2d(tail),
            short_reach,
            long_reach,
            speed: if speed.is_finite() { speed } else { 0.0 },
            metal,
            composition: counts.iter().map(|(n, k)| format!("{k} {n}")).collect::<Vec<_>>().join(", "),
            health,
            artillery,
            walker,
        })
    }

    /// A point at `reach` from one of the element's members, outside every other element's reach and on ground our
    /// body reaches, nearest `from`: the stand-off. None when every such point stands in another element's reach.
    pub(crate) fn stand_off(&self, position: &Position, index: usize, reach: f32, from: Vec3, walker: Walker) -> Option<(Vec3, UnitId)> {
        if reach <= STANDOFF_SLACK {
            return None;
        }
        let r = reach - STANDOFF_SLACK;
        let mut best: Option<(f32, Vec3, UnitId)> = None;
        for m in &position.elements[index].members {
            for k in 0..STANDOFF_DIRECTIONS {
                let angle = k as f32 / STANDOFF_DIRECTIONS as f32 * std::f32::consts::TAU;
                let p = Vec3 { x: m.at.x + r * angle.cos(), y: 0.0, z: m.at.z + r * angle.sin() };
                let d = p.dist2d(from);
                if best.as_ref().is_some_and(|(b, _, _)| *b <= d) {
                    continue;
                }
                if !position.reaching(p, Some(index)).is_empty() || !self.reachable_for(walker, p) {
                    continue;
                }
                best = Some((d, p, m.id));
            }
        }
        best.map(|(_, p, id)| (p, id))
    }

    /// The battlefield of a body before a position: per element its cover, its stand-off, whether it is a screen,
    /// and the odds alone and with the cover priced in.
    pub(crate) fn battlefield(&self, ours: Ours, position: Position) -> Battlefield {
        let facts = (0..position.elements.len())
            .map(|i| {
                let e = &position.elements[i];
                let cover: Vec<usize> = (0..position.elements.len()).filter(|j| *j != i && position.elements[*j].fixed && e.members.iter().any(|m| position.elements[*j].reaches(m.at))).collect();
                let screen = !e.fixed && position.elements.iter().any(|s| s.fixed && e.distance_to_element(s) <= SCREEN_REACH);
                let alone = self.odds(&ours.force, &self.force_of_elements([e]));
                let covered = self.odds(&ours.force, &self.force_of_elements(std::iter::once(e).chain(cover.iter().map(|j| &position.elements[*j]))));
                Facts { stand_off: self.stand_off(&position, i, ours.short_reach.0, ours.front, ours.walker), cover, screen, odds_alone: alone, odds_covered: covered, distance: e.distance_to(ours.front) }
            })
            .collect();
        let odds_whole = self.odds(&ours.force, &self.force_of_elements(&position.elements));
        Battlefield { ours, position, facts, odds_whole }
    }

    /// The battlefield in words, for Jev (the picture's conventions: places by name, qualitative verdicts beside
    /// the numbers that matter).
    pub(crate) fn battlefield_words(&self, bf: &Battlefield, places: &[Place]) -> Value {
        let o = &bf.ours;
        let shape = if o.gathered() { format!("gathered: its tail within {GATHERED:.0} of its front") } else { format!("strung out: its tail {:.0} behind its front", o.length) };
        let reach = if o.short_reach.0 == o.long_reach.0 { format!("{:.0} ({})", o.short_reach.0, o.short_reach.1) } else { format!("shortest {:.0} ({}), longest {:.0} ({})", o.short_reach.0, o.short_reach.1, o.long_reach.0, o.long_reach.1) };
        let ours = json!({
            "group": o.group,
            "units": o.composition,
            "metal": o.metal.round(),
            "reach": reach,
            "speed": format!("{:.0} (its slowest)", o.speed),
            "health": o.health,
            "front": self.place_words(places, o.front),
            "centre": self.place_words(places, o.centre),
            "shape": shape,
        });
        let mut elements = serde_json::Map::new();
        for (e, f) in bf.position.elements.iter().zip(&bf.facts) {
            let to = Vec3 { x: e.at.x - o.front.x, y: 0.0, z: e.at.z - o.front.z };
            let health: Vec<f32> = e.members.iter().filter_map(|m| m.health).collect();
            let health = if health.is_empty() { String::new() } else { format!(", health {:.0}%", 100.0 * health.iter().sum::<f32>() / health.len() as f32) };
            let what = if e.fixed {
                format!("a static: {}{}", e.composition, if e.in_sight { ", in sight" } else { ", remembered where it was seen" })
            } else if f.screen {
                format!("a screen: {}, mobile, within {SCREEN_REACH:.0} of a static", e.composition)
            } else {
                format!("mobile: {}", e.composition)
            };
            let cover = if f.cover.is_empty() {
                "no static's reach covers it".to_string()
            } else {
                format!("it stands inside the reach of {}", f.cover.iter().map(|j| { let s = &bf.position.elements[*j]; format!("{} ({}, reach {:.0})", s.name, s.composition.trim_start_matches("1 "), s.reach) }).collect::<Vec<_>>().join(" and "))
            };
            let stand_off = match &f.stand_off {
                Some((p, _)) => format!("our {:.0} reaches it from outside every other element's reach at {} ({:.0} from our front)", o.short_reach.0, self.place_words(places, *p), p.dist2d(o.front)),
                None => format!("no point at our {:.0} from it lies outside every other element's reach: anything that fights it stands in another's reach", o.short_reach.0),
            };
            let covered = if f.cover.is_empty() { String::new() } else { format!("; with its cover, {} ({:.1})", verdict(f.odds_covered), f.odds_covered) };
            elements.insert(
                e.name.clone(),
                json!(format!(
                    "{what}; reach {:.0}; {:.0} metal{health}; at {}, {} of our front, {:.0} away; {cover}; stand-off: {stand_off}; against it alone, {} ({:.1}){covered}",
                    e.reach,
                    e.metal,
                    self.place_words(places, e.at),
                    super::super::shelling::compass(to),
                    f.distance,
                    verdict(f.odds_alone),
                    f.odds_alone
                )),
            );
        }
        json!({
            "ours": ours,
            "position": format!("{} armed elements within {LINK:.0} of each other around {}; against all of it at once, {} ({:.1})", bf.position.elements.len(), self.place_words(places, bf.position.centre()), verdict(bf.odds_whole), bf.odds_whole),
            "elements": elements,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use bot_protocol::{Hello, MapInfo, MoveClass, MoveKind, Terrain, UnitDefInfo};

    /// A brain over the named types of the simulator's table (no terrain: every point is reachable).
    pub(crate) fn brain_of(names: &[&str]) -> Brain {
        let units = combatsim::units::Units::default();
        let unit_defs: Vec<UnitDefInfo> = names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let u = &units.list[units.index(name).expect(name)];
                UnitDefInfo {
                    id: UnitDefId(i as i32 + 1),
                    name: name.to_string(),
                    metal_cost: u.metal,
                    energy_cost: u.energy,
                    speed: u.speed,
                    build_speed: 0.0,
                    build_time: 1.0,
                    build_distance: 0.0,
                    extracts_metal: 0.0,
                    metal_make: 0.0,
                    energy_make: 0.0,
                    energy_upkeep: 0.0,
                    wind_cap: 0.0,
                    tidal_make: 0.0,
                    metal_storage: 0.0,
                    energy_storage: 0.0,
                    sonar_range: 0.0,
                    hits_submerged: false,
                    water_only: false,
                    radar_range: 0.0,
                    converter: None,
                    weapon_count: u.weapons.len() as i32,
                    build_options: Vec::new(),
                    move_class: u.mobile().then_some(MoveClass { kind: MoveKind::Tank, max_slope: 0.1, depth: 20.0, slope_mod: 0.0 }),
                    footprint: (2, 2),
                    death_blast: None,
                    self_destruct_blast: None,
                    self_destruct_seconds: 5.0,
                    reach: u.reach(),
                    reload: 1.0,
                }
            })
            .collect();
        let hello = Hello {
            ai_id: 0,
            team: 0,
            ally_team: 0,
            game_id: 0,
            teams: Vec::new(),
            start_boxes: Vec::new(),
            start_pos_type: None,
            script: String::new(),
            frame: 0,
            tick_frames: 3,
            map: MapInfo { name: "test".into(), width: 8192.0, height: 6144.0, wind_min: 0.0, wind_max: 0.0, tidal: 0.0, extractor_radius: 120.0 },
            unit_defs,
            metal_spots: Vec::new(),
            metal_spot_squares: Vec::new(),
            terrain: Terrain { cell: 16.0, width: 0, height: 0, heights: Vec::new(), slopes: Vec::new(), metal: Vec::new() },
        };
        let mut brain = Brain::new(crate::world::World::new(hello), None, std::sync::Arc::new(crate::team::TeamBoard::default()), String::new(), None);
        brain.survey_sim_defs();
        brain
    }

    pub(crate) fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    pub(crate) fn own(id: i32, def: i32, pos: Vec3) -> OwnUnit {
        OwnUnit { id: UnitId(id), def: UnitDefId(def), pos, vel: Vec3::default(), health: 1.0, max_health: 1.0, being_built: false, idle: false, reload_frame: 0, facing: 0 }
    }

    pub(crate) fn enemy(id: i32, def: i32, pos: Vec3) -> EnemyUnit {
        EnemyUnit { id: UnitId(id), def: Some(UnitDefId(def)), pos, vel: Vec3::default(), health: 1000.0, team: Some(1), being_built: false }
    }

    /// The E3 scene of player-14 at 9:17, as the TAS study cut it: fourteen Blitzes south-west of three Centurions
    /// at a rock's south edge and two light turrets 400-600 east of them.
    pub(crate) fn e3() -> (Brain, Vec<OwnUnit>, Vec<EnemyUnit>, Vec<Party>) {
        let mut brain = brain_of(&["armflash", "armwar", "armllt"]);
        let ours: Vec<OwnUnit> = (0..14).map(|i| own(100 + i, 1, at(4150.0 + 20.0 * (i % 7) as f32, 1900.0 + 40.0 * (i / 7) as f32))).collect();
        let centurions = vec![enemy(1, 2, at(4404.0, 1228.0)), enemy(2, 2, at(4576.0, 1430.0)), enemy(3, 2, at(4527.0, 1436.0))];
        let turrets = [(10, at(4912.0, 1632.0)), (11, at(5024.0, 1616.0))];
        for (id, pos) in turrets {
            brain.enemy_buildings.insert(UnitId(id), (UnitDefId(3), pos, 0));
        }
        let mut enemies = centurions.clone();
        enemies.extend(turrets.iter().map(|(id, pos)| enemy(*id, 3, *pos)));
        let party = Party { name: "party_12".into(), ids: centurions.iter().map(|e| e.id).collect(), at: at(4502.0, 1365.0), metal: 810.0, composition: "3 armwar".into(), has_commander: false, killing: None, turret_metal: 0.0, turret_metal_air: 0.0, turrets: String::new() };
        (brain, ours, enemies, vec![party])
    }

    /// Step 1's test (the design's order of work): three Centurions and two turrets are one position; the
    /// Centurions' stand-off points exist (our 180 reaches one from outside both turrets' 430), the turrets' do not
    /// (each stands in the other's reach); the Centurions are a screen, and their odds with the cover priced in are
    /// worse than alone.
    #[test]
    fn the_e3_scene_is_one_position_the_screen_has_a_stand_off_and_the_turrets_none() {
        let (brain, ours, enemies, parties) = e3();
        let elements = brain.armed_elements(&parties, &enemies);
        assert_eq!(elements.len(), 3, "{elements:?}");
        let units: Vec<&OwnUnit> = ours.iter().collect();
        let body = brain.ours_of("group_C", &units, at(4502.0, 1365.0), Walker::Air).expect("a body");
        let positions = Brain::positions(elements, body.metal);
        assert_eq!(positions.len(), 1);
        let bf = brain.battlefield(body, positions.into_iter().next().unwrap());
        let index = |name: &str| bf.position.elements.iter().position(|e| e.name == name).expect(name);
        let screen = &bf.facts[index("party_12")];
        let (point, _) = screen.stand_off.expect("the Centurions' stand-off exists");
        for t in ["turret_10", "turret_11"] {
            let turret = &bf.position.elements[index(t)];
            assert!(point.dist2d(turret.at) > turret.reach, "the stand-off {point:?} is outside {t}'s reach");
            assert!(bf.facts[index(t)].stand_off.is_none(), "{t} has no stand-off: it stands in the other turret's reach");
        }
        assert!(screen.screen, "the Centurions are a screen");
        assert!(!screen.cover.is_empty(), "a turret covers the Centurions");
        assert!(screen.odds_covered < screen.odds_alone, "the cover is priced in: {} against {}", screen.odds_covered, screen.odds_alone);
        let words = brain.battlefield_words(&bf, &[]);
        assert!(words["elements"]["party_12"].as_str().unwrap().contains("reaches it from outside"), "{words}");
        assert!(words["elements"]["turret_10"].as_str().unwrap().contains("no point"), "{words}");
    }

    /// A lone raider and a scattered pair are not positions; a static alone is not either.
    #[test]
    fn one_element_or_elements_far_apart_are_no_position() {
        let (brain, _, enemies, parties) = e3();
        let mut elements = brain.armed_elements(&parties, &enemies);
        elements.retain(|e| e.name == "party_12");
        assert!(Brain::positions(elements.clone(), 1500.0).is_empty());
        let mut far = elements[0].clone();
        far.name = "party_13".into();
        for m in far.members.iter_mut() {
            m.at.x += 3000.0;
        }
        elements.push(far);
        assert!(Brain::positions(elements, 1500.0).is_empty(), "two parties 3,000 apart are two lone elements");
    }
}
