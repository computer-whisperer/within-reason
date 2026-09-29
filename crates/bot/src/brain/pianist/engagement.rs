//! Stage 2 of the micro answer (`docs/design/2026-09-28-engagement-plan.md`, H-HANDS-ENGAGEMENT-PLAN): a body that
//! has a position in sight or remembered (two or more armed enemy elements within `LINK` of each other) is given
//! an order of operations; the battlefield words carry the distance. Code finds the position, prices each element with its cover and finds where our reach
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

/// Armed elements this close to each other (their nearest members) are one position.
const LINK: f32 = 600.0;
/// A plan is asked again for the same position after this long (not while a phase runs).
pub(crate) const PLAN_STALE: i32 = 45 * FRAMES_PER_SECOND;
/// A party that appears is a change of the position when its metal is above this share of the body's.
const NEW_PARTY_SHARE: f32 = 0.2;
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
    /// The elements by name, sorted and joined by commas: a plan is for one signature, and an element appearing or
    /// dying changes it.
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
    /// After a covered phase, where this one steps to first (its point, else the first point outside every reach
    /// on the way back from its first target).
    pub step_to: Option<Vec3>,
    /// The phase in a few words, for the player's line: "party_12 from spot_44".
    pub short: String,
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
    /// the rest (else, when a static has a stand-off, those statics one at a time from theirs); the statics first; the nearest first then by distance; a gather before the screen-first plan when
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
            Phase { kind: PhaseKind::Attack, elements: indices.iter().map(|i| pos.elements[*i].name.clone()).collect(), targets, point, covered: indices.iter().any(|i| !bf.facts[*i].cover.is_empty()), step_to: None, short: String::new() }
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
        // With no mobile element to take first from outside cover (a turret line, or a screen standing in every
        // reach): the statics our reach reaches from outside every other reach, one at a time from there, then the
        // rest by distance. In place of the screen-first plan, so the candidates stay six.
        let alone: Vec<usize> = statics.iter().copied().filter(|i| bf.facts[*i].stand_off.is_some()).collect();
        if screen_first.is_none() && !alone.is_empty() {
            let mut phases: Vec<Phase> = alone.iter().map(|i| attack(&[*i], bf.facts[*i].stand_off.map(|(p, _)| p))).collect();
            let rest: Vec<usize> = statics.iter().copied().filter(|i| !alone.contains(i)).collect();
            if !rest.is_empty() {
                phases.push(attack(&rest, None));
            }
            phases.extend(by_distance.iter().copied().filter(|i| !pos.elements[*i].fixed).map(|i| attack(&[i], None)));
            out.push(self.candidate("one_at_a_time", "The statics our reach reaches from outside every other reach, one at a time from there, then the rest", phases, bf, places));
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
            let mut all = vec![Phase { kind: PhaseKind::Gather, elements: Vec::new(), targets: Vec::new(), point: Some(gather), covered: false, step_to: None, short: String::new() }];
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
        let decline = Phase { kind: PhaseKind::Decline, elements: Vec::new(), targets: Vec::new(), point: Some(back.0), covered: false, step_to: None, short: format!("fall back to {}", back.1) };
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
    fn candidate(&self, key: &'static str, title: &str, mut phases: Vec<Phase>, bf: &Battlefield, places: &[Place]) -> Candidate {
        let pos = &bf.position;
        for n in 0..phases.len() {
            let covered_before = n > 0 && phases[n - 1].covered;
            let phase = &mut phases[n];
            if covered_before {
                phase.step_to = phase.point.or_else(|| phase.targets.first().map(|(_, at)| Brain::gathering_point(pos, *at, bf.ours.centre)));
            }
            if phase.short.is_empty() {
                let at = |p: Option<Vec3>| p.map_or(String::new(), |p| format!(" from {}", self.place_words(places, p)));
                phase.short = match phase.kind {
                    PhaseKind::Gather => format!("gather{}", at(phase.point).replacen(" from ", " at ", 1)),
                    PhaseKind::Shell => format!("shell {}{}", phase.elements.join(" and "), at(phase.point)),
                    _ => format!("{}{}", phase.elements.join(" and "), at(phase.point)),
                };
            }
        }
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
        // The hold an advance arrives in is still an attack (it fights everything there): player-14's group_C held
        // at its station spot_20 when E3's Centurions came into sight at 9:15.
        if task_gate && !group.task.busy() && !matches!(group.task, GroupTask::Hold { committed: true, .. }) {
            return Err(format!("{name} holds: a body plans on a course, an attack or the hold an advance arrived in"));
        }
        let metal: f32 = core.iter().filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
        let nearest = |p: &Position| core.iter().map(|u| p.distance_to(u.pos)).fold(f32::INFINITY, f32::min);
        let position = Brain::positions(self.armed_elements(parties, enemies), metal).into_iter().min_by(|a, b| nearest(a).total_cmp(&nearest(b))).ok_or_else(|| format!("{name}: no position in sight or remembered"))?;
        // A position at a place the player's `never` names is not planned against, as a chase there is not made
        // (groups.rs): player-18's plans took group_B north at the E2 nest from 21:33 with spot_23 under `never`
        // since 21:04 (engagement #18, 675 of ours for 280).
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

/// A phase whose targets have all been out of sight this long is over.
const PHASE_LOST: i32 = 10 * FRAMES_PER_SECOND;
/// Orders of a running plan are given again this often.
const PLAN_ORDERS: i32 = 2 * FRAMES_PER_SECOND;
/// A step out of the reach ends when this share of the body stands within `GATHERED` of its point, or after
/// `STEP_OUT_MAX`.
const STEP_OUT_MAX: i32 = 15 * FRAMES_PER_SECOND;
/// A gather ends when this share of the body is within `GATHERED` of its point (the design's 80%).
const GATHER_SHARE: f32 = 0.8;
/// A decline ends when this share of the body is back.
const BACK_SHARE: f32 = 0.6;

/// A plan the body runs as its task (`GroupTask::Plan`).
#[derive(Clone, Debug)]
pub(crate) struct PlanTask {
    pub key: String,
    /// The position's elements when it was chosen.
    pub position: Vec<String>,
    pub phases: Vec<Phase>,
    pub phase: usize,
    /// When the phase (or its step out) began.
    pub since: i32,
    /// Stepping out of a static's reach to this point before the phase begins.
    pub stepping_out: Option<Vec3>,
    /// A target of the phase last in sight.
    pub last_seen: i32,
    pub last_order: i32,
}

impl PlanTask {
    fn with_position(candidate: &Candidate, position: Vec<String>, frame: i32) -> PlanTask {
        PlanTask { key: candidate.key.to_string(), position, phases: candidate.phases.clone(), phase: 0, since: frame, stepping_out: None, last_seen: frame, last_order: i32::MIN / 2 }
    }

    fn current(&self) -> &Phase {
        &self.phases[self.phase.min(self.phases.len() - 1)]
    }

    /// Where the body is going now: the step out, the phase's point, else its first target.
    pub(crate) fn goal(&self) -> Vec3 {
        let phase = self.current();
        self.stepping_out.or(phase.point).or_else(|| phase.targets.first().map(|(_, at)| *at)).unwrap_or_default()
    }

    /// The statics the current phase attacks (the lane is priced against those and no other building).
    pub(crate) fn turrets(&self) -> Vec<UnitId> {
        let phase = self.current();
        if self.stepping_out.is_some() || !matches!(phase.kind, PhaseKind::Attack | PhaseKind::Shell) {
            return Vec::new();
        }
        phase.elements.iter().filter_map(|e| e.strip_prefix("turret_")).filter_map(|id| id.parse::<i32>().ok()).map(UnitId).collect()
    }

    /// Whether the current step fights (an attack or a shelling, not a step out, a gather or the way back).
    pub(crate) fn fighting(&self) -> bool {
        self.stepping_out.is_none() && matches!(self.current().kind, PhaseKind::Attack | PhaseKind::Shell)
    }

    /// The player's line: "plan screen_first: party_12 from spot_44, then turret_3 and turret_4, then party_9; phase
    /// 1 of 3, 20 s in".
    pub(crate) fn line(&self, frame: i32) -> String {
        let phases: Vec<&str> = self.phases.iter().map(|p| p.short.as_str()).collect();
        let step = if self.stepping_out.is_some() { ", stepping out of the statics' reach first" } else { "" };
        format!("on its engagement plan {}: {}; phase {} of {}, {} s in{step}", self.key, phases.join(", then "), self.phase + 1, self.phases.len(), (frame - self.since) / FRAMES_PER_SECOND)
    }
}

impl Brain {
    /// One think of a group's plan: the phase's end (its targets dead, out of sight for `PHASE_LOST`, or
    /// `PHASE_UNTIL`; a gather at 80% of the body within `GATHERED` of its point), the step out of a static's reach
    /// after a covered phase, and the orders: a Fight to the phase's point, then an Attack by id on the nearest target
    /// in sight for each soldier near enough (a building out of sight is fought at where it stands: the engine drops
    /// an attack on an unseen unit); the shelling's long-reach members attack from the point, the rest stand there.
    /// Returns what the player's report should hear (a phase begun, the plan done).
    pub(super) fn tick_plan(&self, group: &mut super::Group, units: &[&OwnUnit], enemies: &[EnemyUnit], frame: i32, commands: &mut Vec<bot_protocol::Command>) -> Option<String> {
        use bot_protocol::Command;
        let name = group.name.clone();
        let GroupTask::Plan(plan) = &mut group.task else { return None };
        if units.is_empty() {
            return None;
        }
        let near = |to: Vec3| units.iter().filter(|u| u.pos.dist2d(to) <= GATHERED).count() as f32 / units.len() as f32;
        let mut news: Option<String> = None;
        for _ in 0..=plan.phases.len() {
            if let Some(to) = plan.stepping_out {
                if near(to) >= BACK_SHARE || frame - plan.since > STEP_OUT_MAX {
                    (plan.stepping_out, plan.since, plan.last_seen, plan.last_order) = (None, frame, frame, i32::MIN / 2);
                    continue;
                }
                if frame - plan.last_order >= PLAN_ORDERS {
                    plan.last_order = frame;
                    commands.extend(units.iter().map(|u| Command::Move { unit: u.id, to, queue: false }));
                }
                return news;
            }
            let phase = plan.current().clone();
            let living: Vec<(UnitId, Vec3, bool)> = phase
                .targets
                .iter()
                .filter_map(|(id, at)| match enemies.iter().find(|e| e.id == *id) {
                    Some(e) => Some((*id, e.pos, e.def.is_some())),
                    // Out of sight: a building where it stands, a soldier where it was when the plan was made,
                    // until it is known dead.
                    None if self.enemy_deaths.iter().any(|(dead, ..)| *dead == id.0 as u32) => None,
                    None => Some((*id, self.enemy_buildings.get(id).map_or(*at, |(_, pos, _)| *pos), false)),
                })
                .collect();
            if living.iter().any(|(_, _, seen)| *seen) {
                plan.last_seen = frame;
            }
            let done = frame - plan.since > PHASE_UNTIL
                || match phase.kind {
                    PhaseKind::Gather => phase.point.is_none_or(|p| near(p) >= GATHER_SHARE),
                    PhaseKind::Decline => phase.point.is_none_or(|p| near(p) >= BACK_SHARE),
                    PhaseKind::Attack | PhaseKind::Shell => living.is_empty() || frame - plan.last_seen > PHASE_LOST,
                };
            if done {
                plan.phase += 1;
                if plan.phase >= plan.phases.len() {
                    let key = plan.key.clone();
                    group.set_task(GroupTask::Hold { since: frame, committed: false }, frame);
                    return Some(format!("group_{name} finished its engagement plan {key}"));
                }
                (plan.since, plan.last_seen, plan.last_order) = (frame, frame, i32::MIN / 2);
                if phase.covered {
                    plan.stepping_out = plan.current().step_to;
                }
                news = Some(format!("group_{name}'s engagement plan {}: phase {} of {}, {}", plan.key, plan.phase + 1, plan.phases.len(), plan.current().short));
                continue;
            }
            if frame - plan.last_order < PLAN_ORDERS {
                return news;
            }
            plan.last_order = frame;
            let seen: Vec<&(UnitId, Vec3, bool)> = living.iter().filter(|(_, _, s)| *s).collect();
            let reach_of = |u: &OwnUnit| self.world.def(u.def).map_or(0.0, |d| d.reach);
            for u in units {
                let nearest_seen = seen.iter().min_by(|a, b| a.1.dist2d(u.pos).total_cmp(&b.1.dist2d(u.pos)));
                let nearest_living = living.iter().min_by(|a, b| a.1.dist2d(u.pos).total_cmp(&b.1.dist2d(u.pos)));
                let order = match phase.kind {
                    PhaseKind::Gather | PhaseKind::Decline => phase.point.map(|to| Command::Move { unit: u.id, to, queue: false }),
                    PhaseKind::Shell if reach_of(u) < ARTILLERY_REACH => phase.point.map(|to| Command::Move { unit: u.id, to, queue: false }),
                    PhaseKind::Attack | PhaseKind::Shell => {
                        let close = |at: Vec3| u.pos.dist2d(at) <= reach_of(u) + 150.0;
                        let at_point = phase.point.is_none_or(|p| u.pos.dist2d(p) <= 250.0);
                        match (nearest_seen, phase.point) {
                            (Some((id, at, _)), _) if at_point || close(*at) => Some(Command::Attack { unit: u.id, target: *id, queue: false }),
                            (_, Some(p)) if !at_point => Some(Command::Fight { unit: u.id, to: p, queue: false }),
                            _ => nearest_living.map(|(_, at, _)| Command::Fight { unit: u.id, to: *at, queue: false }),
                        }
                    }
                };
                commands.extend(order);
            }
            return news;
        }
        news
    }

    /// The engagement pass, once a second after the one pass: for every group whose gate opens (`engagement_of`),
    /// the plan question when its position is new to it (another signature, or `PLAN_STALE` since the last ask) and
    /// no ask is in flight; in lockstep the answer is played at once, in realtime when it comes.
    pub(super) fn engagement_pass(&mut self, tick: &bot_protocol::Tick, picture: &super::picture::Picture) {
        self.collect_plans(tick);
        let frame = tick.frame;
        let Some(pianist) = self.pianist.as_ref() else { return };
        if pianist.client.is_none() {
            return;
        }
        let own = &tick.snapshot.own_units;
        let instructions = picture.state["instructions"].as_str().unwrap_or_default();
        let mut asks: Vec<(String, String, String, PlanMemory, Vec<Candidate>, jev::Request)> = Vec::new();
        for group in &pianist.groups {
            let name = format!("group_{}", group.name);
            let Ok((bf, candidates)) = self.engagement_of(group, own, &picture.parties, &tick.snapshot.enemies, &picture.places, true) else { continue };
            let signature = bf.position.signature();
            if pianist.plan_pending.contains_key(&name) {
                continue;
            }
            let running = match &group.task {
                GroupTask::Plan(plan) => Some(plan.phase),
                _ => None,
            };
            let dead = |id: UnitId, fixed: bool| if fixed { !self.enemy_buildings.contains_key(&id) } else { self.enemy_deaths.iter().any(|(d, ..)| *d == id.0 as u32) };
            let Some(reason) = ask_reason(pianist.plan_memory.get(&name), &bf.position, bf.ours.metal, running, frame, &dead) else { continue };
            let mut memory = PlanMemory::of(&bf.position, frame);
            memory.phase = running;
            let paragraph = super::diet::paragraph(instructions, &name).unwrap_or_default();
            let words = self.battlefield_words(&bf, &picture.places);
            let request = request(&name, words, &candidates, paragraph, true);
            asks.push((name, signature, reason, memory, candidates, request));
        }
        for (name, signature, reason, memory, candidates, request) in asks {
            let pianist = self.pianist.as_mut().expect("checked");
            pianist.plan_memory.insert(name.clone(), memory);
            match &pianist.plan_worker {
                Some(worker) => {
                    if worker.to.send((name.clone(), request.clone())).is_ok() {
                        pianist.plan_pending.insert(name, PlanPending { frame, signature, reason, candidates, request });
                    }
                }
                None => {
                    let result = pianist.client.as_ref().expect("checked").ask(&request);
                    self.plan_answered(frame, &name, &signature, &reason, &candidates, &request, result);
                }
            }
        }
    }

    /// Realtime: the plan answers that have come, played; an answer older than `PLAN_ANSWER_STALE` is dropped.
    pub(super) fn collect_plans(&mut self, tick: &bot_protocol::Tick) {
        let Some(pianist) = self.pianist.as_mut() else { return };
        let Some(worker) = &pianist.plan_worker else { return };
        let mut arrived = Vec::new();
        while let Ok((name, result)) = worker.from.try_recv() {
            if let Some(pending) = pianist.plan_pending.remove(&name) {
                arrived.push((name, pending, result));
            }
        }
        pianist.plan_pending.retain(|_, p| tick.frame - p.frame <= PLAN_ANSWER_STALE);
        for (name, pending, result) in arrived {
            if tick.frame - pending.frame > PLAN_ANSWER_STALE {
                continue;
            }
            self.plan_answered(tick.frame, &name, &pending.signature, &pending.reason, &pending.candidates, &pending.request, result);
        }
    }

    /// An answer: the plan taken (`choose`), logged as an `engagement` line of the Jev log, and made the group's task.
    #[allow(clippy::too_many_arguments)]
    fn plan_answered(&mut self, frame: i32, name: &str, signature: &str, reason: &str, candidates: &[Candidate], request: &jev::Request, result: Result<jev::Response, jev::Error>) {
        let Some(pianist) = self.pianist.as_mut() else { return };
        let options: BTreeMap<&str, &str> = candidates.iter().map(|c| (c.key, c.words.as_str())).collect();
        let response = match result {
            Ok(r) => r,
            Err(e) => {
                pianist.write_log(json!({ "t": "engagement", "f": frame, "group": name, "position": signature, "reason": reason, "error": e.to_string() }));
                return;
            }
        };
        let taken = choose(&response.answers, candidates);
        pianist.write_log(json!({
            "t": "engagement", "f": frame, "group": name, "position": signature, "reason": reason, "ms": response.latency.as_millis() as u64, "model": response.model,
            "battlefield": request.state["battlefield"], "options": options,
            "probabilities": taken.as_ref().map(|(_, p)| json!(p)), "taken": taken.as_ref().map(|(i, _)| candidates[*i].key),
        }));
        let Some((index, probabilities)) = taken else { return };
        let candidate = &candidates[index];
        if let Some(m) = pianist.plan_memory.get_mut(name) {
            (m.taken, m.asked) = (Some(candidate.key.to_string()), frame);
        }
        let Some(group) = pianist.groups.iter_mut().find(|g| format!("group_{}", g.name) == name) else { return };
        // The same plan again goes on where it is (its phase keeps running), with the position as it is now.
        if let GroupTask::Plan(plan) = &mut group.task
            && plan.key == candidate.key
        {
            plan.position = signature.split(',').map(str::to_string).collect();
            let phase = plan.phase;
            if let Some(m) = pianist.plan_memory.get_mut(name) {
                m.phase = Some(phase);
            }
            return;
        }
        group.set_task(GroupTask::Plan(PlanTask::with_position(candidate, signature.split(',').map(str::to_string).collect(), frame)), frame);
        if let Some(m) = pianist.plan_memory.get_mut(name) {
            m.phase = Some(0);
        }
        let p = probabilities.get(candidate.key).copied().unwrap_or(0.0);
        let text = format!("{name} takes the engagement plan {} against {signature} (p {p:.2}): {}", candidate.key, candidate.phases.iter().map(|ph| ph.short.as_str()).collect::<Vec<_>>().join(", then "));
        pianist.note(frame, text.clone());
        pianist.done.push(format!("{} {text}", super::picture::clock(frame)));
        pianist.played.push(json!({ "actor": name, "kind": "group", "played": format!("{name}.plan_{}", candidate.key), "did": text, "source": "engagement" }));
        self.journal.note_from("engagement", frame, "group", json!({ "actor": name, "position": signature }), json!({ "plan": candidate.key, "p": p }));
    }
}

/// What a group was last asked about (H-HANDS-ENGAGEMENT-PLAN's hysteresis): the position's elements by their units,
/// when, what was taken, and the plan's phase at the ask.
#[derive(Clone, Debug)]
pub(crate) struct PlanMemory {
    pub elements: Vec<(String, bool, Vec<UnitId>)>,
    pub asked: i32,
    /// The plan taken; None until answered (or when the call failed).
    pub taken: Option<String>,
    pub phase: Option<usize>,
}

impl PlanMemory {
    pub(crate) fn of(position: &Position, frame: i32) -> PlanMemory {
        PlanMemory { elements: position.elements.iter().map(|e| (e.name.clone(), e.fixed, e.members.iter().map(|m| m.id).collect())).collect(), asked: frame, taken: None, phase: None }
    }
}

/// Why the group's plan is asked again now, or None to keep what stands: a first ask; else, never while the phase
/// that was running at the last ask still runs, an
/// element of the remembered position dead (a party all of whose units are dead, a building gone from memory), a new
/// static, a new party (none of its units in the remembered position) above a fifth of the body's metal, or
/// `PLAN_STALE`. A party renamed, or a member more or less, is no change.
pub(crate) fn ask_reason(memory: Option<&PlanMemory>, position: &Position, body_metal: f32, running_phase: Option<usize>, frame: i32, dead: &dyn Fn(UnitId, bool) -> bool) -> Option<String> {
    let Some(m) = memory else { return Some("first".into()) };
    if running_phase.is_some() && running_phase == m.phase {
        return None;
    }
    if let Some((name, _, _)) = m.elements.iter().find(|(_, fixed, ids)| !ids.is_empty() && ids.iter().all(|id| dead(*id, *fixed))) {
        return Some(format!("{name} died"));
    }
    let known = |id: &UnitId| m.elements.iter().any(|(_, _, ids)| ids.contains(id));
    for e in &position.elements {
        if e.members.iter().any(|x| known(&x.id)) {
            continue;
        }
        if e.fixed {
            return Some(format!("new static {}", e.name));
        }
        if e.metal > NEW_PARTY_SHARE * body_metal {
            return Some(format!("new party {} ({:.0} metal)", e.name, e.metal));
        }
    }
    (frame - m.asked >= PLAN_STALE).then(|| format!("stale: {} s since the last ask", (frame - m.asked) / FRAMES_PER_SECOND))
}

/// A plan answer older than this judges a battlefield too old to play (realtime).
const PLAN_ANSWER_STALE: i32 = 3 * FRAMES_PER_SECOND;

/// A plan question in flight (realtime).
pub(crate) struct PlanPending {
    frame: i32,
    signature: String,
    reason: String,
    candidates: Vec<Candidate>,
    request: jev::Request,
}

/// The thread that asks the plan questions in realtime, so the game never waits on them.
pub(crate) struct PlanWorker {
    to: std::sync::mpsc::Sender<(String, jev::Request)>,
    from: std::sync::mpsc::Receiver<(String, Result<jev::Response, jev::Error>)>,
}

pub(crate) fn spawn_plan_worker(client: jev::Client) -> PlanWorker {
    let (to, requests) = std::sync::mpsc::channel::<(String, jev::Request)>();
    let (answers, from) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        while let Ok((name, request)) = requests.recv() {
            if answers.send((name, client.ask(&request))).is_err() {
                break;
            }
        }
    });
    PlanWorker { to, from }
}

/// The plan question: a Choice over the candidates, the battlefield, the player's words for the group and its
/// standing orders in the state; the examples block in the instructions unless `examples` is false.
pub(crate) fn request(group: &str, battlefield: Value, candidates: &[Candidate], instructions: &str, examples: bool) -> jev::Request {
    let text = format!(
        "Given `battlefield` (our body {group}; his armed elements at this position; what covers each; where our reach reaches each from outside every other element's reach; the odds, with the cover priced in) and the player's `instructions` for {group}: which order of operations does {group} take against this position? Each option is a plan: its phases in order, where each is fought from, what else reaches us there, and the odds of each phase as the body stands now. decline falls back and leaves the position.{}",
        if examples { format!(" {EXAMPLES}") } else { String::new() }
    );
    let state = json!({
        "battlefield": battlefield,
        "instructions": if instructions.is_empty() { "(the player wrote nothing for this group)" } else { instructions },
    });
    let question = jev::Question::choice(text, candidates.iter().map(|c| (c.key, json!(c.words))));
    jev::Request { state, questions: BTreeMap::from([(QUESTION.to_string(), question)]) }
}

/// The plan taken from Jev's answer: the option it put the most on (the decline is one of them). The index and the
/// probabilities.
pub(crate) fn choose(answers: &BTreeMap<String, jev::Answer>, candidates: &[Candidate]) -> Option<(usize, BTreeMap<String, f64>)> {
    let jev::Answer::Choice { probabilities, .. } = answers.get(QUESTION)? else { return None };
    let (top, _) = candidates.iter().enumerate().map(|(i, c)| (i, probabilities.get(c.key).copied().unwrap_or(0.0))).max_by(|a, b| a.1.total_cmp(&b.1))?;
    Some((top, probabilities.clone()))
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
                    blast_radius: 0.0,
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

    fn e3_group(ours: &[OwnUnit], task: GroupTask) -> super::super::Group {
        super::super::Group::new("C".into(), crate::world::Domain::Ground, ours.iter().map(|u| u.id).collect(), task, 0)
    }

    /// The gate and the candidates on the E3 scene: an advancing body (gathered, so no gather) is offered the screen
    /// first, the statics first, the nearest first and the decline; a holding body and the player's `plan: no` are
    /// not planned, the hold an advance arrived in is.
    #[test]
    fn the_e3_body_is_offered_the_screen_first_and_the_standing_plan_no_declines_it() {
        let (brain, ours, enemies, parties) = e3();
        let advancing = GroupTask::Move { to: at(4500.0, 1400.0), place: "spot_20".into(), fight: true, since: 0 };
        let group = e3_group(&ours, advancing.clone());
        let (bf, candidates) = brain.engagement_of(&group, &ours, &parties, &enemies, &[], true).expect("planned");
        let keys: Vec<&str> = candidates.iter().map(|c| c.key).collect();
        assert_eq!(keys, ["screen_first", "statics_first", "nearest_first", "decline"], "{:?}", bf.ours);
        let screen = &candidates[0];
        assert_eq!(screen.phases[0].elements, ["party_12"]);
        assert!(screen.phases[0].point.is_some() && screen.phases[0].covered);
        assert!(screen.phases[1].step_to.is_some(), "the statics' phase steps out of the reach first");
        assert!(screen.words.contains("nothing else reaches us there"), "{}", screen.words);
        let holding = e3_group(&ours, GroupTask::Hold { since: 0, committed: false });
        assert!(brain.engagement_of(&holding, &ours, &parties, &enemies, &[], true).is_err());
        let arrived = e3_group(&ours, GroupTask::Hold { since: 0, committed: true });
        assert!(brain.engagement_of(&arrived, &ours, &parties, &enemies, &[], true).is_ok(), "an advance's arrival hold is planned");
    }

    /// The taken plan: the option Jev put the most on, the decline one of them.
    #[test]
    fn the_top_plan_is_what_jev_put_the_most_on() {
        let (brain, ours, enemies, parties) = e3();
        let group = e3_group(&ours, GroupTask::Move { to: at(4500.0, 1400.0), place: "spot_20".into(), fight: true, since: 0 });
        let (_, candidates) = brain.engagement_of(&group, &ours, &parties, &enemies, &[], true).expect("planned");
        let answer = |pairs: &[(&str, f64)]| BTreeMap::from([(QUESTION.to_string(), jev::Answer::Choice { choice: pairs[0].0.to_string(), probabilities: pairs.iter().map(|(k, p)| (k.to_string(), *p)).collect(), confidence: 0.5 })]);
        let key = |a: &BTreeMap<String, jev::Answer>| candidates[choose(a, &candidates).expect("an answer").0].key;
        assert_eq!(key(&answer(&[("screen_first", 0.74), ("decline", 0.04)])), "screen_first");
        assert_eq!(key(&answer(&[("screen_first", 0.38), ("nearest_first", 0.33), ("decline", 0.29)])), "screen_first", "no bar: the top option");
        assert_eq!(key(&answer(&[("decline", 0.55), ("screen_first", 0.45)])), "decline");
    }

    /// The plan runs phase by phase: a Fight to the stand-off and Attacks on the screen; the screen dead, a step out
    /// of the turrets' reach first (a Move); there, the turrets; the turrets dead, the body holds.
    #[test]
    fn a_plan_runs_its_phases_and_steps_out_of_the_reach_between_them() {
        use bot_protocol::Command;
        let (mut brain, mut ours, mut enemies, parties) = e3();
        let mut group = e3_group(&ours, GroupTask::Move { to: at(4500.0, 1400.0), place: "spot_20".into(), fight: true, since: 0 });
        let (_, candidates) = brain.engagement_of(&group, &ours, &parties, &enemies, &[], true).expect("planned");
        group.set_task(GroupTask::Plan(PlanTask::with_position(&candidates[0], vec!["party_12".into(), "turret_10".into(), "turret_11".into()], 0)), 0);
        let mut frame = 30;
        let tick = |brain: &Brain, group: &mut super::super::Group, ours: &[OwnUnit], enemies: &[EnemyUnit], frame: i32| {
            let units: Vec<&OwnUnit> = ours.iter().collect();
            let mut commands = Vec::new();
            let news = brain.tick_plan(group, &units, enemies, frame, &mut commands);
            (commands, news)
        };
        let (commands, _) = tick(&brain, &mut group, &ours, &enemies, frame);
        assert!(!commands.is_empty() && commands.iter().all(|c| matches!(c, Command::Fight { .. } | Command::Attack { .. })), "{commands:?}");
        // The screen dies.
        enemies.retain(|e| e.def != Some(UnitDefId(2)));
        brain.enemy_deaths.extend([(1, frame, 270), (2, frame, 270), (3, frame, 270)]);
        frame += 60;
        let (commands, news) = tick(&brain, &mut group, &ours, &enemies, frame);
        assert!(news.is_some_and(|n| n.contains("phase 2 of 2")));
        let GroupTask::Plan(plan) = &group.task else { panic!("still planning") };
        let out = plan.stepping_out.expect("stepping out of the turrets' reach");
        assert!(commands.iter().all(|c| matches!(c, Command::Move { to, .. } if *to == out)), "{commands:?}");
        // At the step's point: the turrets.
        for u in ours.iter_mut() {
            u.pos = out;
        }
        frame += 60;
        let (commands, _) = tick(&brain, &mut group, &ours, &enemies, frame);
        assert!(commands.iter().any(|c| matches!(c, Command::Attack { target, .. } if target.0 == 10 || target.0 == 11) || matches!(c, Command::Fight { .. })), "{commands:?}");
        // The turrets die: the plan is done.
        enemies.clear();
        brain.enemy_buildings.clear();
        brain.enemy_deaths.extend([(10, frame, 85), (11, frame, 85)]);
        frame += 60;
        let (_, news) = tick(&brain, &mut group, &ours, &enemies, frame);
        assert!(news.is_some_and(|n| n.contains("finished")));
        assert!(matches!(group.task, GroupTask::Hold { .. }));
    }

    /// The re-ask: a first ask; nothing while the phase asked
    /// under still runs; a renamed party or a member more is no change; a death, a new static or a party above a fifth
    /// of the body's metal is; 45 s is stale.
    #[test]
    fn a_plan_is_asked_again_only_on_a_material_change() {
        let (brain, ours, enemies, parties) = e3();
        let group = e3_group(&ours, GroupTask::Move { to: at(4500.0, 1400.0), place: "spot_20".into(), fight: true, since: 0 });
        let (bf, _) = brain.engagement_of(&group, &ours, &parties, &enemies, &[], true).expect("planned");
        let alive = |_: UnitId, _: bool| false;
        let metal = bf.ours.metal;
        assert_eq!(ask_reason(None, &bf.position, metal, None, 0, &alive).as_deref(), Some("first"));
        let mut memory = PlanMemory::of(&bf.position, 0);
        memory.taken = Some("screen_first".into());
        memory.phase = Some(0);
        let s = FRAMES_PER_SECOND;
        assert!(ask_reason(Some(&memory), &bf.position, metal, Some(0), 60 * s, &alive).is_none(), "the phase asked under still runs");
        assert!(ask_reason(Some(&memory), &bf.position, metal, Some(1), 25 * s, &alive).is_none(), "nothing changed");
        // The party renamed and one Centurion more: no change.
        let mut renamed = bf.position.clone();
        let screen = renamed.elements.iter_mut().find(|e| !e.fixed).unwrap();
        screen.name = "party_99".into();
        screen.members.push(Member { id: UnitId(4), def: None, at: at(4500.0, 1400.0), reach: 325.0, health: None });
        assert!(ask_reason(Some(&memory), &renamed, metal, Some(1), 25 * s, &alive).is_none(), "a rename is no change");
        // A turret destroyed.
        let turret_dead = |id: UnitId, fixed: bool| fixed && id.0 == 10;
        assert_eq!(ask_reason(Some(&memory), &bf.position, metal, Some(1), 25 * s, &turret_dead).as_deref(), Some("turret_10 died"));
        // A new party: a Pawn (a twentieth of the body) is no change, three Centurions are.
        let mut bigger = bf.position.clone();
        let mut pawn = bigger.elements[0].clone();
        (pawn.name, pawn.fixed, pawn.metal) = ("party_50".into(), false, 54.0);
        pawn.members = vec![Member { id: UnitId(50), def: None, at: at(4700.0, 1500.0), reach: 180.0, health: None }];
        bigger.elements.push(pawn.clone());
        assert!(ask_reason(Some(&memory), &bigger, metal, None, 25 * s, &alive).is_none(), "a small party is no change");
        bigger.elements.last_mut().unwrap().metal = 810.0;
        assert!(ask_reason(Some(&memory), &bigger, metal, None, 25 * s, &alive).is_some_and(|r| r.starts_with("new party party_50")));
        // Stale at 45 s.
        assert!(ask_reason(Some(&memory), &bf.position, metal, None, 46 * s, &alive).is_some_and(|r| r.starts_with("stale")));
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
