//! Events that ask the pass again out of turn (H-HANDS-SCHEDULE's events, feeding the one pass's ask-on-change:
//! `docs/design/2026-09-26-one-pass.md` §5): a member of a group hit or killed, an enemy come into sight within a
//! group's alarm reach, a builder under fire, a party entering a builder's alarm reach. The events of the ticks
//! between calls are kept until the next ask (fixes-1: the player's retreat reached the group six seconds after it
//! was written; the user: "RTS actions on the front line and on scouting units need to be frequently second-scale").

use std::collections::HashSet;

use bot_protocol::{Event, Tick, UnitId};

use super::super::Brain;
use super::plan::ALARM;

impl Brain {
    /// Every tick: remembers this tick's hits (the pass reads them at the next call, whatever tick that is) and
    /// names the actors whose situation the tick's events changed.
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
        // A party entering a builder's alarm reach (the parties are the last picture's; their place is where their
        // units stand now) is an event once, and again when it comes back after leaving.
        let mut alarmed: HashSet<(UnitId, String)> = HashSet::new();
        for unit in own.iter().filter(|u| !u.being_built && self.world.is_mobile_builder(u.def)) {
            for party in &pianist.parties {
                let units: Vec<&bot_protocol::EnemyUnit> = tick.snapshot.enemies.iter().filter(|e| party.ids.contains(&e.id)).collect();
                let Some(at) = super::groups::centre_of_enemies(&units) else { continue };
                if at.dist2d(unit.pos) < ALARM {
                    let key = (unit.id, party.name.clone());
                    if !pianist.alarmed.contains(&key) {
                        pianist.events.insert(format!("{} alarmed by {}", self.actor_name(unit.id), party.name));
                    }
                    alarmed.insert(key);
                }
            }
        }
        pianist.alarmed = alarmed;
        if hit.is_empty() && gone.is_empty() && seen.is_empty() {
            self.pianist = Some(pianist);
            return;
        }
        let seen_at: Vec<bot_protocol::Vec3> = seen.iter().filter_map(|id| tick.snapshot.enemies.iter().find(|e| e.id == *id)).map(|e| e.pos).collect();
        for group in &pianist.groups {
            let touched = group.members.iter().any(|m| hit.contains(m) || gone.contains(m));
            let alarmed = !seen_at.is_empty() && {
                let units = group.units(own);
                super::groups::centre_of(&units).is_some_and(|c| seen_at.iter().any(|p| p.dist2d(c) < ALARM))
            };
            if touched {
                pianist.events.insert(format!("group_{} hit", group.name));
            }
            if alarmed {
                pianist.events.insert(format!("group_{} sighted a party", group.name));
            }
        }
        for unit in own.iter().filter(|u| hit.contains(&u.id) && !u.being_built && self.world.is_mobile_builder(u.def)) {
            pianist.events.insert(format!("{} hit", self.actor_name(unit.id)));
        }
        self.pianist = Some(pianist);
    }
}
