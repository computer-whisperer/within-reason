//! The player fires a nuclear silo (`fire` tool; docs/design/2026-10-09-late-economy.md §10, the user's ruling of
//! 2026-10-09: a player tool, since the player chooses the exact place). The pianist carries the orders out as it does
//! the `remove` tool's: a ready missile goes as `Command::FireAt` at the place, and what happened is said in the hands'
//! report. The silo's rounds are the game's to make (K-rules-a-stockpile-fills-itself-to-its-limit); nothing here
//! decides when or where.

use bot_protocol::{Command, Tick, Vec3};

use super::super::Brain;
use crate::strategist::shared::{Firing, Shared};

impl Brain {
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
