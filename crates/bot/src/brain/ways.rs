//! Penned units (H-HANDS-STUCK-WORDS, amended 2026-11-08 night; H-ECO-LAYOUT's veto instrumented). The layer's
//! way-out veto (`layout::choose`) passes over a site that would wall in a mobile builder of ours, and
//! econ-flagship-17 penned ten constructors even so, 2,525 constructor-seconds, two by rings of their own solars and
//! three advanced constructors inside the dense block, with no trace of what the veto saw: a refused try is silent,
//! and only a site refused in every try is logged. Two instruments and one answer:
//! - a record row each time the veto refuses a site (`d` row `layout` / `walled`: what was placed for whom, and the
//!   verdict's words);
//! - a watch once a second over every mobile builder's way to the open, with a row the second it closes naming the
//!   footprints of ours that came up beside it since the last look (`layout` / `penned`);
//! - for any unit penned or stuck, the cheapest footprints of ours whose removal frees it
//!   (`layout::way_out_state`), in the unit's words for the player's `remove`, and a row when found
//!   (`layout` / `freeing`).
use std::collections::{HashMap, HashSet};

use bot_protocol::{OwnUnit, Tick, UnitId};
use layout::{Builder, WayOut};
use serde_json::{Value, json};

use super::{Brain, FRAMES_PER_SECOND};

/// How often the ways are flooded: once a second (a flood a builder, 22,500 cells).
const WAYS_FRAMES: i32 = FRAMES_PER_SECOND;
/// How often a penned or stuck unit's freeing is searched again (a search is up to seventy floods).
const FREEING_FRAMES: i32 = 10 * FRAMES_PER_SECOND;
/// A footprint that closed a builder's way is one of ours that came up within this of the builder since the last
/// look.
const CLOSER_NEAR: f32 = 300.0;

/// A builder the watch saw lose its way to the open: since when, and what came up beside it that second.
pub(super) struct Pen {
    pub since: i32,
    pub closer: Vec<String>,
}

/// The cheapest way to free a unit, as last searched.
pub(super) struct Freeing {
    pub frame: i32,
    pub way: FreeingWay,
}

pub(super) enum FreeingWay {
    /// Reclaim these of ours (by handle) and it walks: the fewest that open a way, the least metal among as many.
    Remove { handles: Vec<String>, metal: f32 },
    /// The ground, or what is not ours, shuts it: no one or two of ours opens a way.
    Shut,
}

impl Brain {
    /// A mobile unit as the layer floods it: its path width (the engine's move class), its ground.
    fn body_of(&self, u: &OwnUnit) -> Option<Builder> {
        let d = self.world.def(u.def)?;
        (d.speed > 0.0).then(|| Builder { at: u.pos, half: d.path_width() / 2.0, passable: self.passable_for_class(d.move_class) })
    }

    /// Every tick: the veto's verdicts since the last tick go to the record; once a second, every mobile builder's
    /// way to the open is flooded, a way that closed is a pen with its closer named, and every penned or stuck unit
    /// without a fresh answer gets the cheapest way to free it.
    pub(super) fn track_ways(&mut self, tick: &Tick) {
        let verdicts: Vec<(Value, Value)> = self.layout_verdicts.borrow_mut().drain(..).collect();
        for (inputs, outputs) in verdicts {
            self.journal.note_from("layout", tick.frame, "walled", inputs, outputs);
        }
        if tick.frame - self.ways_frame < WAYS_FRAMES {
            return;
        }
        self.ways_frame = tick.frame;
        let own = &tick.snapshot.own_units;
        self.pens.retain(|id, _| own.iter().any(|u| u.id == *id));
        self.freeings.retain(|id, _| own.iter().any(|u| u.id == *id));
        let frame = tick.frame;
        // Everything below reads the world (a borrow of self); what it decides is applied after.
        let (ways, standing_now, pens_new, freed, freeings_new) = {
            let world = self.layout_world(own);
            let blocks = world.blocking_rects();
            let open = &*world.open;
            let standing_now: HashSet<String> = world.standing.iter().filter(|s| s.ours).map(|s| s.name.clone()).collect();
            let mut ways: HashMap<UnitId, bool> = HashMap::new();
            let mut pens_new: Vec<(UnitId, OwnUnit, Vec<String>)> = Vec::new();
            let mut freed: Vec<UnitId> = Vec::new();
            for u in own.iter().filter(|u| !u.being_built && self.world.def(u.def).is_some_and(|d| d.speed > 0.0 && d.build_speed > 0.0)) {
                let Some(body) = self.body_of(u) else { continue };
                let has_way = layout::way_out(&world.ground, &body, &blocks, open);
                ways.insert(u.id, has_way);
                match (self.ways.get(&u.id).copied(), has_way) {
                    (Some(true), false) => {
                        let closer: Vec<String> = world.standing.iter().filter(|s| s.ours && !self.ways_seen.contains(&s.name) && layout::distance_to(&s.rect, u.pos) <= CLOSER_NEAR).map(|s| s.name.clone()).collect();
                        pens_new.push((u.id, u.clone(), closer));
                    }
                    (_, true) => freed.push(u.id),
                    _ => {}
                }
            }
            // The cheapest way to free every penned or stuck unit whose answer is stale.
            let ours: Vec<(usize, f32)> = world.standing.iter().enumerate().filter(|(_, s)| s.ours && s.def.is_some()).map(|(i, s)| (i, s.metal)).collect();
            let wants: Vec<UnitId> = self
                .pens
                .keys()
                .chain(pens_new.iter().map(|(id, ..)| id))
                .chain(self.stuck.keys())
                .copied()
                .filter(|id| !freed.contains(id) && self.freeings.get(id).is_none_or(|f| frame - f.frame >= FREEING_FRAMES))
                .collect::<HashSet<UnitId>>()
                .into_iter()
                .collect();
            let mut freeings_new: Vec<(UnitId, FreeingWay)> = Vec::new();
            for id in wants {
                let Some(u) = own.iter().find(|u| u.id == id) else { continue };
                let Some(body) = self.body_of(u) else { continue };
                let way = match layout::way_out_state(&world.ground, &body, &blocks, &ours, open) {
                    WayOut::Open => continue,
                    WayOut::Freed { remove, metal } => FreeingWay::Remove { handles: remove.iter().map(|i| world.standing[*i].name.clone()).collect(), metal },
                    WayOut::Shut => FreeingWay::Shut,
                };
                freeings_new.push((id, way));
            }
            (ways, standing_now, pens_new, freed, freeings_new)
        };
        for id in freed {
            self.pens.remove(&id);
            self.freeings.remove(&id);
        }
        for (id, u, closer) in pens_new {
            let handle = self.handle(&u);
            self.journal.note_from("layout", frame, "penned", json!({ "unit": handle, "at": [u.pos.x.round(), u.pos.z.round()], "cell": self.world.grid(u.pos) }), json!({ "closer": closer }));
            self.pens.insert(id, Pen { since: frame, closer });
        }
        for (id, way) in freeings_new {
            if let Some(u) = own.iter().find(|u| u.id == id) {
                let outputs = match &way {
                    FreeingWay::Remove { handles, metal } => json!({ "remove": handles, "metal": metal.round() }),
                    FreeingWay::Shut => json!({ "shut": true }),
                };
                let changed = match (&way, self.freeings.get(&id).map(|f| &f.way)) {
                    (FreeingWay::Remove { handles: a, .. }, Some(FreeingWay::Remove { handles: b, .. })) => a != b,
                    (FreeingWay::Shut, Some(FreeingWay::Shut)) => false,
                    _ => true,
                };
                if changed {
                    self.journal.note_from("layout", frame, "freeing", json!({ "unit": self.handle(u), "penned_since": self.pens.get(&id).map(|p| p.since), "stuck_since": self.stuck.get(&id).map(|s| s.since) }), outputs);
                }
            }
            self.freeings.insert(id, Freeing { frame, way });
        }
        self.ways = ways;
        self.ways_seen = standing_now;
    }

    /// A unit's pen and the way to free it, for its line in the picture: "; penned since 12 s ago by armsolar_31002:
    /// no way to the open from where it stands; the cheapest way out: reclaim armsolar_31002 (150 metal)".
    pub(super) fn pen_words(&self, unit: UnitId, frame: i32) -> Option<String> {
        let pen = self.pens.get(&unit);
        let freeing = self.freeings.get(&unit);
        if pen.is_none() && freeing.is_none() {
            return None;
        }
        let mut words = String::new();
        if let Some(pen) = pen {
            words.push_str(&format!("; penned since {} s ago", (frame - pen.since) / FRAMES_PER_SECOND));
            if !pen.closer.is_empty() {
                words.push_str(&format!(" by {}", pen.closer.join(", ")));
            }
            words.push_str(": no way to the open from where it stands");
        }
        match freeing.map(|f| &f.way) {
            Some(FreeingWay::Remove { handles, metal }) => words.push_str(&format!("; the cheapest way out: reclaim {} ({metal:.0} metal; `remove`)", handles.join(" and "))),
            Some(FreeingWay::Shut) => words.push_str("; nothing of ours frees it: the ground or others' buildings shut it"),
            None => {}
        }
        Some(words)
    }

    /// The veto's verdicts from a site search, kept for the record (`track_ways` writes them).
    pub(super) fn note_walled(&self, trace: &[String], what: &str, actor: &str, centre: bot_protocol::Vec3) {
        for line in trace.iter().filter(|l| l.contains("walls in builder")) {
            self.layout_verdicts.borrow_mut().push((json!({ "what": what, "for": actor, "round": [centre.x.round(), centre.z.round()] }), json!({ "verdict": line.trim() })));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::pianist::fixtures::{at, brain_of, own};
    use bot_protocol::{OwnUnit, Resource, Snapshot, Tick, UnitId};

    fn tick(own: Vec<OwnUnit>, frame: i32) -> Tick {
        let resource = || Resource { current: 0.0, income: 0.0, usage: 0.0, storage: 0.0 };
        Tick { frame, late: 0, events: Vec::new(), snapshot: Snapshot { metal: resource(), energy: resource(), wind: 0.0, own_units: own, allies: Vec::new(), enemies: Vec::new(), wrecks: None, sensors: None, conversion: 0.0 } }
    }

    /// A constructor with solars west, east and north has a way out; the fourth solar, south, pens it: the pen names
    /// that solar, the cheapest way out is one solar, and the pen ends when a solar goes.
    #[test]
    fn a_builder_walled_in_by_the_last_solar_is_penned_with_the_solar_named_and_a_way_out_costed() {
        let mut brain = brain_of(&["armcom", "armcv", "armsolar"]);
        for d in brain.world.hello.unit_defs.iter_mut() {
            match d.name.as_str() {
                "armsolar" => d.footprint = (10, 10),
                "armcv" => {
                    d.footprint = (6, 6);
                    d.build_speed = 90.0;
                }
                _ => {}
            }
        }
        let def = |name: &str| brain.world.hello.unit_defs.iter().find(|d| d.name == name).unwrap().id.0;
        let (cv, solar) = (def("armcv"), def("armsolar"));
        let builder = own(1, cv, at(2000.0, 2000.0));
        let west = own(2, solar, at(1920.0, 2000.0));
        let east = own(3, solar, at(2080.0, 2000.0));
        let north = own(4, solar, at(2000.0, 1920.0));
        let south = own(5, solar, at(2000.0, 2080.0));
        brain.track_ways(&tick(vec![builder.clone(), west.clone(), east.clone(), north.clone()], 30));
        assert_eq!(brain.ways.get(&UnitId(1)), Some(&true));
        assert!(brain.pens.is_empty());
        brain.track_ways(&tick(vec![builder.clone(), west.clone(), east.clone(), north.clone(), south.clone()], 60));
        let pen = brain.pens.get(&UnitId(1)).expect("penned");
        assert_eq!(pen.closer, vec!["armsolar_5".to_string()]);
        let words = brain.pen_words(UnitId(1), 90).unwrap();
        assert!(words.contains("penned since 1 s ago by armsolar_5") && words.contains("the cheapest way out: reclaim armsolar_"), "{words}");
        assert_eq!(brain.journal.notes.iter().map(|n| n.kind).collect::<Vec<_>>(), vec!["penned", "freeing"]);
        // The south solar reclaimed: the way is back and the pen is gone.
        brain.track_ways(&tick(vec![builder.clone(), west.clone(), east.clone(), north.clone()], 90));
        assert!(brain.pens.is_empty() && brain.pen_words(UnitId(1), 90).is_none());
    }
}
