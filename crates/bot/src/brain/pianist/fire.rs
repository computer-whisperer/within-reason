//! The player fires a nuclear silo (`fire` tool; docs/design/2026-10-09-late-economy.md §10, the user's ruling of
//! 2026-10-09: a player tool, since the player chooses the exact place). The pianist carries the orders out as it does
//! the `remove` tool's: a ready missile goes as `Command::FireAt` at the place, and what happened is said in the hands'
//! report. The silo's rounds are the game's to make (K-rules-a-stockpile-fills-itself-to-its-limit); nothing here
//! decides when or where.

use std::collections::{HashMap, HashSet};

use bot_protocol::{Command, OwnUnit, Tick, UnitId, Vec3};

use super::super::Brain;
use crate::strategist::shared::{Firing, Shared};

impl Brain {
    /// The rounds that came ready since the last tick, against `last` (the count each stockpiler held): (the line,
    /// whether it wakes the player). A silo's every round is news and wakes it (nuke-2-hard, 2026-10-09: two missiles
    /// sat ready five minutes before his anti-nuke while nothing the player read named them); an anti-nuke's first
    /// interceptor is news and does not wake. A count that fell (a firing) is not news: the fire line said it.
    pub(in crate::brain) fn round_news(&self, own: &[OwnUnit], last: &mut HashMap<UnitId, i32>) -> Vec<(String, bool)> {
        let mut news = Vec::new();
        let mut standing: HashSet<UnitId> = HashSet::new();
        for u in own.iter().filter(|u| !u.being_built) {
            let Some(d) = self.world.def(u.def).filter(|d| d.stockpile_seconds > 0.0) else { continue };
            standing.insert(u.id);
            let before = last.insert(u.id, u.stockpile).unwrap_or(0);
            if u.stockpile <= before {
                continue;
            }
            if d.coverage > 0.0 {
                if before == 0 {
                    news.push((format!("{} holds an interceptor: a missile of his aimed within {:.0} of it is taken", self.handle(u), d.coverage), false));
                }
            } else {
                let piling = if u.stockpile_queued > 0 { format!(", {} stockpiling", u.stockpile_queued) } else { ", none stockpiling: it is full".to_string() };
                news.push((format!("{} holds {} missile{} ready{piling}; `fire` sends one at a place", self.handle(u), u.stockpile, if u.stockpile == 1 { "" } else { "s" }), true));
            }
        }
        last.retain(|id, _| standing.contains(id));
        news
    }

    /// Every tick: a round coming ready goes to the hands' done lines and the picture's notes, and a silo's round to
    /// the wake (`WakeState::rounds_ready`, read when the player's `missile_ready` is on).
    pub(in crate::brain) fn note_rounds(&mut self, tick: &Tick) {
        let mut last = std::mem::take(&mut self.pianist.as_mut().expect("pianist mode").rounds);
        let news = self.round_news(&tick.snapshot.own_units, &mut last);
        let with_player = self.strategist.is_some();
        let pianist = self.pianist.as_mut().expect("pianist mode");
        pianist.rounds = last;
        let mut wakes = Vec::new();
        for (text, wakes_player) in news {
            pianist.done.push(format!("{} {text}", super::clock(tick.frame)));
            if wakes_player && with_player {
                wakes.push(text.clone());
            }
            pianist.note(tick.frame, text);
        }
        self.wake.rounds_ready.extend(wakes);
    }

    /// The `fire` tool's orders since the last ask.
    pub(in crate::brain) fn take_firings(&mut self, tick: &Tick, commands: &mut Vec<Command>, shared: &Shared) {
        let firings = std::mem::take(&mut *shared.firings.lock().unwrap());
        if firings.is_empty() {
            return;
        }
        let own = &tick.snapshot.own_units;
        // Another seat's silo is left for that seat (the handles are unit ids, unique across the game).
        let mut others: Vec<Firing> = Vec::new();
        let mut notes: Vec<String> = Vec::new();
        for firing in firings {
            if self.handle_owner(&firing.silo, own) == super::super::HandleOwner::AnotherSeat {
                others.push(firing);
                continue;
            }
            let Some(silo) = self.unit_by_handle(&firing.silo, own) else {
                notes.push(format!("{} no longer stands; nothing to fire", firing.silo));
                continue;
            };
            if silo.stockpile < 1 {
                notes.push(format!("{} has no missile ready ({} stockpiling); nothing fired", firing.silo, silo.stockpile_queued));
                continue;
            }
            let at = Vec3 { x: firing.at.0, y: 0.0, z: firing.at.1 };
            commands.push(Command::FireAt { unit: silo.id, at });
            notes.push(format!("{} fires a missile at ({:.0}, {:.0}) {} on the player's order; it launches in about 9 s and lands 13 s later", firing.silo, at.x, at.z, self.world.grid(at)));
        }
        if !others.is_empty() {
            shared.firings.lock().unwrap().extend(others);
        }
        let frame = tick.frame;
        let pianist = self.pianist.as_mut().expect("pianist mode");
        for text in notes {
            pianist.done.push(format!("{} {text}", super::clock(frame)));
            pianist.note(frame, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::fixtures::{at, brain_of, own};

    /// A silo's round is news and wakes; the same count again is not; an anti-nuke's first interceptor is news and does
    /// not wake, its second is nothing; a fired round (the count falling) is nothing.
    #[test]
    fn a_round_coming_ready_is_news_once_and_a_silos_wakes() {
        let brain = brain_of(&["armsilo", "armamd", "armsolar"]);
        let unit = |id: i32, def: i32, ready: i32, queued: i32| {
            let mut u = own(id, def, at(500.0, 500.0));
            u.stockpile = ready;
            u.stockpile_queued = queued;
            u
        };
        let mut last = HashMap::new();
        assert!(brain.round_news(&[unit(1, 1, 0, 10), unit(2, 2, 0, 5), unit(3, 3, 0, 0)], &mut last).is_empty(), "nothing ready yet");
        let news = brain.round_news(&[unit(1, 1, 1, 9), unit(2, 2, 1, 4)], &mut last);
        assert_eq!(news.len(), 2, "{news:?}");
        assert_eq!(news[0], ("armsilo_1 holds 1 missile ready, 9 stockpiling; `fire` sends one at a place".to_string(), true));
        assert_eq!(news[1], ("armamd_2 holds an interceptor: a missile of his aimed within 2000 of it is taken".to_string(), false));
        assert!(brain.round_news(&[unit(1, 1, 1, 9), unit(2, 2, 2, 3)], &mut last).is_empty(), "the same count and a second interceptor are not news");
        let news = brain.round_news(&[unit(1, 1, 2, 8)], &mut last);
        assert_eq!(news, vec![("armsilo_1 holds 2 missiles ready, 8 stockpiling; `fire` sends one at a place".to_string(), true)]);
        assert!(brain.round_news(&[unit(1, 1, 1, 9)], &mut last).is_empty(), "a firing is not news");
        assert!(!last.contains_key(&bot_protocol::UnitId(2)), "a stockpiler gone is forgotten");
    }
}
