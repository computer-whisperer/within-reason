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
use super::GroupTask;
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

/// How long a phase runs at most before the next.
pub(crate) const PHASE_UNTIL: i32 = 60 * FRAMES_PER_SECOND;
/// The top plan is taken when Jev puts more than this on it (and more than on the decline).
pub(crate) const PLAN_BAR: f64 = 0.4;
/// The question's id in the request.
pub(crate) const QUESTION: &str = "engagement.plan";

/// What a phase does.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PhaseKind {
    /// A whole-body attack on its targets, from its point when it has one (a Fight to the point, then Attack by id).
    Attack,
    /// Walk to the point and hold until most of the body is there.
    Gather,
    /// The artillery shells the targets from the point, the rest standing with it.
    Shell,
    /// Fall back to the point without fighting.
    Decline,
}

/// One phase of a plan.
#[derive(Clone, Debug)]
pub(crate) struct Phase {
    pub kind: PhaseKind,
    /// The elements it is fought against, by name.
    pub elements: Vec<String>,
    /// Their units: id and where last seen, nearest first.
    pub targets: Vec<(UnitId, Vec3)>,
    /// Where the body goes first: the stand-off, the gathering point, the shelling point, the hold it falls back to.
    pub point: Option<Vec3>,
    /// Its targets stand under a static's reach: the next phase begins by stepping out of it.
    pub covered: bool,
}

/// A candidate plan: its phases and its words.
#[derive(Clone, Debug)]
pub(crate) struct Candidate {
    pub key: &'static str,
    pub phases: Vec<Phase>,
    pub words: String,
}

/// The examples block (the design's §5): what the studies found in positions like this, one line each.
pub(crate) const EXAMPLES: &str = "Examples from our studies, each a position fought again in the engine: (1) E3, fourteen Blitzes (reach 180) against three Centurions (reach 325) at a rock's corner with two light turrets (reach 430) 400 east of them: the Centurions first from the corner outside the turrets' reach, then out of the reach, gather outside the 430, the turrets last, took the position ahead 24 times of 24; the turrets first 5 of 24; attack-move on everything 21 of 24; the orders the bot gave (one Centurion, then home after 5 s) 0 of 24. (2) F3, four Rocketeers (reach 475) under a light turret out-range Blitzes (180) and Stouts (350): the stand-off must be beyond 475 or the fight is declined; walking in under them cost the body. (3) A ball of Stouts against a few Fatboys (area damage) loses worse the tighter and bigger it is: against area damage, spread and few. (4) The pros' Rovers kill unguarded things only and never fight Pawns.";

impl Brain {
    /// The candidate plans for a battlefield, at most six: the screen first from its stand-off then the statics then
    /// the rest; the statics first; the nearest first then by distance; a gather before the screen-first plan when
    /// the body is strung out; hold and shell when the body's artillery out-reaches the statics and one is in sight;
    /// the decline (fall back to `back`).
    pub(crate) fn candidates(&self, bf: &Battlefield, places: &[Place], back: (Vec3, String)) -> Vec<Candidate> {
        let pos = &bf.position;
        let mut by_distance: Vec<usize> = (0..pos.elements.len()).collect();
        by_distance.sort_by(|a, b| bf.facts[*a].distance.total_cmp(&bf.facts[*b].distance));
        let statics: Vec<usize> = by_distance.iter().copied().filter(|i| pos.elements[*i].fixed).collect();
        let attack = |indices: &[usize], point: Option<Vec3>| -> Phase {
            let mut targets: Vec<(UnitId, Vec3)> = indices.iter().flat_map(|i| pos.elements[*i].members.iter().map(|m| (m.id, m.at))).collect();
            let from = point.unwrap_or(bf.ours.front);
            targets.sort_by(|a, b| a.1.dist2d(from).total_cmp(&b.1.dist2d(from)));
            Phase { kind: PhaseKind::Attack, elements: indices.iter().map(|i| pos.elements[*i].name.clone()).collect(), targets, point, covered: indices.iter().any(|i| !bf.facts[*i].cover.is_empty()) }
        };
        let mut out: Vec<Candidate> = Vec::new();
        // The screen first: the mobile elements our reach reaches from outside every other reach, screens first.
        let mut first: Vec<usize> = by_distance.iter().copied().filter(|i| !pos.elements[*i].fixed && bf.facts[*i].stand_off.is_some()).collect();
        first.sort_by_key(|i| !bf.facts[*i].screen);
        let screen_first: Option<Vec<Phase>> = (!first.is_empty() && !statics.is_empty()).then(|| {
            let mut phases: Vec<Phase> = first.iter().map(|i| attack(&[*i], bf.facts[*i].stand_off.map(|(p, _)| p))).collect();
            phases.push(attack(&statics, None));
            phases.extend(by_distance.iter().copied().filter(|i| !pos.elements[*i].fixed && !first.contains(i)).map(|i| attack(&[i], None)));
            phases
        });
        if let Some(phases) = &screen_first {
            out.push(self.candidate("screen_first", "The screen first from outside the statics' reach, then the statics, then the rest", phases.clone(), bf, places));
        }
        if !statics.is_empty() {
            let mut phases = vec![attack(&statics, None)];
            phases.extend(by_distance.iter().copied().filter(|i| !pos.elements[*i].fixed).map(|i| attack(&[i], None)));
            out.push(self.candidate("statics_first", "The statics first, walking in on them; then the mobile elements by distance", phases, bf, places));
        }
        out.push(self.candidate("nearest_first", "The nearest element first, then each by distance, walking in on each", by_distance.iter().map(|i| attack(&[*i], None)).collect(), bf, places));
        if !bf.ours.gathered()
            && let Some(phases) = &screen_first
            && let Some(point) = phases[0].point
        {
            let gather = Brain::gathering_point(pos, point, bf.ours.centre);
            let mut all = vec![Phase { kind: PhaseKind::Gather, elements: Vec::new(), targets: Vec::new(), point: Some(gather), covered: false }];
            all.extend(phases.iter().cloned());
            out.push(self.candidate("gather_first", "Gather first outside every reach, then the screen first, the statics, the rest", all, bf, places));
        }
        // Hold and shell: the artillery out-reaches every static and a static is in sight (the spotter).
        let static_reach = statics.iter().map(|i| pos.elements[*i].reach).fold(0.0, f32::max);
        if !bf.ours.artillery.is_empty() && bf.ours.long_reach.0 > static_reach && statics.iter().any(|i| pos.elements[*i].in_sight) {
            let nearest = statics[0];
            if let Some((point, _)) = self.stand_off(pos, nearest, bf.ours.long_reach.0, bf.ours.front, bf.ours.walker).filter(|(p, _)| pos.reaching(*p, None).is_empty()) {
                let mut phase = attack(&statics, Some(point));
                phase.kind = PhaseKind::Shell;
                out.push(self.candidate("shell", "Hold outside every reach and shell the statics with the artillery", vec![phase], bf, places));
            }
        }
        let decline = Phase { kind: PhaseKind::Decline, elements: Vec::new(), targets: Vec::new(), point: Some(back.0), covered: false };
        let mut words = self.candidate("decline", "Decline", vec![decline], bf, places);
        words.words = format!("Decline: fall back to {} ({:.0} from our front) and leave the position; against all of it at once with the cover, {} ({:.1})", back.1, back.0.dist2d(bf.ours.front), verdict(bf.odds_whole), bf.odds_whole);
        out.push(words);
        out
    }

    /// Where a body gathers before its first phase: the first point no element reaches on the way from the first
    /// stand-off `near` back past the body's centre (to 1,500 beyond the stand-off); the centre when every such
    /// point is in reach.
    fn gathering_point(pos: &Position, near: Vec3, centre: Vec3) -> Vec3 {
        let (dx, dz) = (centre.x - near.x, centre.z - near.z);
        let len = dx.hypot(dz);
        if len < 1.0 {
            return centre;
        }
        (1..=30).map(|k| 50.0 * k as f32).map(|d| Vec3 { x: near.x + dx / len * d, y: 0.0, z: near.z + dz / len * d }).find(|p| pos.reaching(*p, None).is_empty()).unwrap_or(centre)
    }

    /// A candidate's words: each phase with its geometry (where it is fought from, what reaches us there), the odds
    /// of the phase with the cover still standing priced in, the walk, and the metal at risk.
    fn candidate(&self, key: &'static str, title: &str, phases: Vec<Phase>, bf: &Battlefield, places: &[Place]) -> Candidate {
        let pos = &bf.position;
        let index = |name: &str| pos.elements.iter().position(|e| e.name == name);
        let mut dead: Vec<usize> = Vec::new();
        let mut from = bf.ours.front;
        let mut parts: Vec<String> = Vec::new();
        for (n, phase) in phases.iter().enumerate() {
            let targets: Vec<usize> = phase.elements.iter().filter_map(|e| index(e)).collect();
            let names = targets.iter().map(|i| format!("{} ({})", pos.elements[*i].name, pos.elements[*i].composition)).collect::<Vec<_>>().join(" and ");
            let alive = |i: &usize| !dead.contains(i) && !targets.contains(i);
            let text = match phase.kind {
                PhaseKind::Gather => {
                    let p = phase.point.unwrap_or(from);
                    let outside = if pos.reaching(p, None).is_empty() { "outside every reach" } else { "inside the reach of some of his elements (no point on the way back is outside them all)" };
                    let t = format!("gather at {} ({:.0} from our front, about {:.0} s), {outside}, until 80% of the body is within 300 of it", self.place_words(places, p), p.dist2d(from), bf.ours.walk_seconds(p.dist2d(from)));
                    from = p;
                    t
                }
                PhaseKind::Decline => String::new(),
                PhaseKind::Attack | PhaseKind::Shell => {
                    // What reaches us where the phase is fought: at the point, the elements still standing whose reach
                    // covers it; walking in, those covering any target.
                    let joined: Vec<usize> = match phase.point {
                        Some(p) => pos.reaching(p, None).into_iter().filter(|i| alive(i)).collect(),
                        // Walking in, whatever reaches the ground our reach fights the targets from.
                        None => (0..pos.elements.len()).filter(|i| alive(i) && targets.iter().any(|t| pos.elements[*t].members.iter().any(|m| pos.elements[*i].members.iter().any(|o| o.at.dist2d(m.at) <= o.reach + bf.ours.short_reach.0)))).collect(),
                    };
                    let odds = self.odds(&bf.ours.force, &self.force_of_elements(targets.iter().chain(&joined).map(|i| &pos.elements[*i])));
                    let also = if joined.is_empty() { "nothing else reaches us there".to_string() } else { format!("also in reach of us there: {}", joined.iter().map(|i| format!("{} (reach {:.0})", pos.elements[*i].name, pos.elements[*i].reach)).collect::<Vec<_>>().join(", ")) };
                    let target_metal: f32 = targets.iter().map(|i| pos.elements[*i].metal).sum();
                    let t = match (phase.kind.clone(), phase.point) {
                        (PhaseKind::Shell, Some(p)) => format!("hold at {} ({:.0} from our front, about {:.0} s), outside every reach, and shell {names} with the {} artillery (reach {:.0}) while a spotter keeps them in sight; the rest of the body stands with it; {} ({:.1}) against their {target_metal:.0} metal", self.place_words(places, p), p.dist2d(from), bf.ours.walk_seconds(p.dist2d(from)), bf.ours.artillery.len(), bf.ours.long_reach.0, verdict(odds), odds),
                        (_, Some(p)) => format!("{names} from its stand-off at {} ({:.0} from where the body is, about {:.0} s): our {:.0} reaches it there and {also}; {} ({:.1}) against their {target_metal:.0} metal", self.place_words(places, p), p.dist2d(from), bf.ours.walk_seconds(p.dist2d(from)), bf.ours.short_reach.0, verdict(odds), odds),
                        (_, None) => {
                            let d = targets.iter().map(|i| pos.elements[*i].distance_to(from)).fold(f32::INFINITY, f32::min);
                            format!("{names}, walking in on {} ({:.0} from where the body is, about {:.0} s); {also}; {} ({:.1}) against their {target_metal:.0} metal", if targets.len() > 1 { "them" } else { "it" }, d, bf.ours.walk_seconds(d), verdict(odds), odds)
                        }
                    };
                    dead.extend(targets.iter().copied());
                    if let Some(p) = phase.point {
                        from = p;
                    } else if let Some(t) = targets.first() {
                        from = pos.elements[*t].at;
                    }
                    let out_of_reach = if phase.covered && n + 1 < phases.len() { ", then out of the statics' reach before the next" } else { "" };
                    format!("{t}{out_of_reach}")
                }
            };
            parts.push(format!("({}) {text}", n + 1));
        }
        let metal: f32 = pos.elements.iter().map(|e| e.metal).sum();
        let words = format!("{title}: {}. Our {:.0} metal at risk against their {metal:.0}.", parts.join("; "), bf.ours.metal);
        Candidate { key, phases, words }
    }
}

impl Brain {
    /// The engagement of a group this second, or why it has none: not a roving or air group, not a scout body
    /// (H-HANDS-SCOUTS-FIGHT-NOTHING-ARMED), not under the player's `plan: no`, on a course or an attack (when
    /// `task_gate`; the replay tool plans a holding body too and says so), and a position within `PLAN_REACH` of its
    /// front. The battlefield and the candidates, with the decline back to its last hold, else home.
    pub(crate) fn engagement_of(&self, group: &super::Group, own: &[OwnUnit], parties: &[Party], enemies: &[EnemyUnit], places: &[Place], task_gate: bool) -> Result<(Battlefield, Vec<Candidate>), String> {
        let name = format!("group_{}", group.name);
        if group.roving {
            return Err(format!("{name} roves: its soldiers are the lane's"));
        }
        if group.domain == crate::world::Domain::Air {
            return Err(format!("{name} flies"));
        }
        let body = group.body(own, None).ok_or_else(|| format!("{name} has nobody standing"))?;
        let core = body.core.clone();
        if self.scouts_only(&core) {
            return Err(format!("{name} is a scout body: it fights nothing armed (H-HANDS-SCOUTS-FIGHT-NOTHING-ARMED)"));
        }
        if self.pianist.as_ref().is_some_and(|p| p.standing.rules_for(&name).get("plan").is_some_and(|v| v == "no")) {
            return Err(format!("{name}: the player's standing `plan: no`"));
        }
        // The hold an advance arrives in is still an attack (it fights everything there): player-14's group_C held
        // at its station spot_20 when E3's Centurions came into sight at 9:15.
        if task_gate && !group.task.busy() && !matches!(group.task, GroupTask::Hold { committed: true, .. }) {
            return Err(format!("{name} holds: a body plans on a course, an attack or the hold an advance arrived in"));
        }
        let metal: f32 = core.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
        let nearest = |p: &Position| core.iter().map(|u| p.distance_to(u.pos)).fold(f32::INFINITY, f32::min);
        let position = Brain::positions(self.armed_elements(parties, enemies), metal).into_iter().min_by(|a, b| nearest(a).total_cmp(&nearest(b))).ok_or_else(|| format!("{name}: no position in sight or remembered"))?;
        let d = nearest(&position);
        if d > PLAN_REACH {
            return Err(format!("{name}: the nearest position ({}) is {d:.0} from its front, beyond {PLAN_REACH:.0}", position.signature()));
        }
        let own_units: Vec<OwnUnit> = own.to_vec();
        let walker = self.group_walker(group, &own_units);
        let ours = self.ours_of(&name, &core, position.centre(), walker).ok_or_else(|| format!("{name} has nobody standing"))?;
        let back = match group.last_hold {
            Some(at) => (at, format!("where it last held, {}", self.place_words(places, at))),
            None => (self.home, "home".to_string()),
        };
        let bf = self.battlefield(ours, position);
        let candidates = self.candidates(&bf, places, back);
        Ok((bf, candidates))
    }
}

/// The plan question: a Choice over the candidates, the battlefield, the player's words for the group and its
/// standing orders in the state; the examples block in the instructions unless `examples` is false.
pub(crate) fn request(group: &str, battlefield: Value, candidates: &[Candidate], instructions: &str, standing: &str, examples: bool) -> jev::Request {
    let text = format!(
        "Given `battlefield` (our body {group}; his armed elements at this position; what covers each; where our reach reaches each from outside every other element's reach; the odds, with the cover priced in) and the player's `instructions` for {group}: which order of operations does {group} take against this position? Each option is a plan: its phases in order, where each is fought from, what else reaches us there, and the odds of each phase as the body stands now. decline falls back and leaves the position.{}",
        if examples { format!(" {EXAMPLES}") } else { String::new() }
    );
    let state = json!({
        "battlefield": battlefield,
        "instructions": if instructions.is_empty() { "(the player wrote nothing for this group)" } else { instructions },
        "standing": if standing.is_empty() { "(none)" } else { standing },
    });
    let question = jev::Question::choice(text, candidates.iter().map(|c| (c.key, json!(c.words))));
    jev::Request { state, questions: BTreeMap::from([(QUESTION.to_string(), question)]) }
}

/// The plan taken from Jev's answer: the top option when its probability is above `PLAN_BAR` and above the
/// decline's, else the decline. The index and the probabilities.
pub(crate) fn choose(answers: &BTreeMap<String, jev::Answer>, candidates: &[Candidate]) -> Option<(usize, BTreeMap<String, f64>)> {
    let jev::Answer::Choice { probabilities, .. } = answers.get(QUESTION)? else { return None };
    let decline = candidates.iter().position(|c| c.key == "decline")?;
    let (top, p) = candidates.iter().enumerate().map(|(i, c)| (i, probabilities.get(c.key).copied().unwrap_or(0.0))).max_by(|a, b| a.1.total_cmp(&b.1))?;
    let p_decline = probabilities.get("decline").copied().unwrap_or(0.0);
    let taken = if top != decline && (p <= PLAN_BAR || p <= p_decline) { decline } else { top };
    Some((taken, probabilities.clone()))
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
