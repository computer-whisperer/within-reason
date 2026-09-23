//! The ask scheduler (H-HANDS-SCHEDULE). Jev is asked about an actor on a review period, ten seconds for a group with a
//! task, five for one holding, and out of turn for what has changed: a new packet from the player, a place marked or
//! renamed, a member of a group hit or killed, an enemy come into sight within a group's alarm reach, a builder under
//! fire. Before 2026-09-23 only the review periods and a party newly within 600 could put an actor on a call, and the
//! events of the ticks between calls were not seen at all (fixes-1: the player's retreat reached the group six seconds
//! after it was written; the user: "RTS actions on the front line and on scouting units need to be frequently
//! second-scale"). Withholding an actor saves about 800 Jev tokens a question on a 4,600-token picture, so the
//! scheduler stays for the cost, and is made to know what changed.

use std::collections::HashSet;

use bot_protocol::{Event, Tick, UnitId};

use super::super::{Brain, FRAMES_PER_SECOND};
use super::menu::ALARM;

/// An actor is put on a call for an event no oftener than this.
pub(super) const EVENT_ASK_GAP: i32 = 2 * FRAMES_PER_SECOND;
/// The least time between two calls when an event asks for one (the interval otherwise).
pub(super) const EVENT_CALL_GAP: i32 = FRAMES_PER_SECOND / 2;

impl Brain {
    /// Every tick, before the cadence is decided: remembers this tick's hits (the menus read them at the next call,
    /// whatever tick that is) and marks the actors whose situation the tick's events changed as due.
    pub(super) fn schedule_asks(&mut self, tick: &Tick) {
        let Some(mut pianist) = self.pianist.take() else { return };
        let frame = tick.frame;
        let own = &tick.snapshot.own_units;
        let mut hit: Vec<UnitId> = Vec::new();
        let mut gone: Vec<UnitId> = Vec::new();
        let mut seen: Vec<UnitId> = Vec::new();
        for event in &tick.events {
            match event {
                Event::UnitDamaged { unit, .. } => hit.push(*unit),
                Event::UnitDestroyed { unit, .. } => gone.push(*unit),
                Event::EnemyEnterLos { enemy } => seen.push(*enemy),
                _ => {}
            }
        }
        for unit in &hit {
            pianist.hits.insert(*unit, frame);
        }
        if hit.is_empty() && gone.is_empty() && seen.is_empty() {
            self.pianist = Some(pianist);
            return;
        }
        let mut due: HashSet<String> = HashSet::new();
        let seen_at: Vec<bot_protocol::Vec3> = seen.iter().filter_map(|id| tick.snapshot.enemies.iter().find(|e| e.id == *id)).map(|e| e.pos).collect();
        for group in &pianist.groups {
            let touched = group.members.iter().any(|m| hit.contains(m) || gone.contains(m));
            let alarmed = !seen_at.is_empty() && {
                let units = group.units(own);
                super::groups::centre_of(&units).is_some_and(|c| seen_at.iter().any(|p| p.dist2d(c) < ALARM))
            };
            if touched || alarmed {
                due.insert(format!("group_{}", group.name));
            }
        }
        for unit in own.iter().filter(|u| hit.contains(&u.id) && !u.being_built && self.world.is_mobile_builder(u.def)) {
            due.insert(self.actor_name(unit.id));
        }
        for name in due {
            let last = pianist.last_asked.get(&name).copied().unwrap_or(i32::MIN / 2);
            if frame - last >= EVENT_ASK_GAP {
                pianist.due_now.insert(name);
            }
        }
        self.pianist = Some(pianist);
    }

    /// A new packet: every group and every builder not on a list is asked at the next call (the player's words are
    /// for now, not for the next review period).
    pub(super) fn schedule_all_for_packet(&mut self, tick: &Tick) {
        let Some(mut pianist) = self.pianist.take() else { return };
        let names: Vec<String> = pianist.groups.iter().map(|g| format!("group_{}", g.name)).collect();
        pianist.due_now.extend(names);
        for unit in tick.snapshot.own_units.iter().filter(|u| !u.being_built && self.world.is_mobile_builder(u.def)) {
            let name = self.actor_name(unit.id);
            if !pianist.scripts.get(&name).is_some_and(|s| !s.is_empty()) {
                pianist.due_now.insert(name);
            }
        }
        self.pianist = Some(pianist);
    }
}
