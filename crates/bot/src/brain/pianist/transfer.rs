//! Transfers between seats of ours (`docs/design/2026-09-27-posing-changes.md` §12b): the `transfer` tool's
//! orders, taken by the seat that owns what is sent, and the units given to this seat taken into a group.

use bot_protocol::{Command, Event, OwnUnit, Tick, UnitId};

use super::super::Brain;
use crate::strategist::shared::{Shared, Transfer};

impl Brain {
    /// The `transfer` tool's orders since the last ask: this seat sends the resources it was told to and the units
    /// of its own that were named; the rest stay for their seats.
    pub(in crate::brain) fn take_transfers(&mut self, tick: &Tick, commands: &mut Vec<Command>, shared: &Shared) {
        let transfers = std::mem::take(&mut *shared.transfers.lock().unwrap());
        if transfers.is_empty() {
            return;
        }
        let own = &tick.snapshot.own_units;
        let team = self.world.hello.team;
        let mut others: Vec<Transfer> = Vec::new();
        let mut notes: Vec<String> = Vec::new();
        for transfer in transfers {
            match transfer {
                Transfer::Resources { from_team, to_team, metal, energy } => {
                    if from_team != team {
                        others.push(Transfer::Resources { from_team, to_team, metal, energy });
                        continue;
                    }
                    let (have_m, have_e) = (tick.snapshot.metal.current, tick.snapshot.energy.current);
                    let (m, e) = (metal.min(have_m).max(0.0), energy.min(have_e).max(0.0));
                    commands.push(Command::SendResources { metal: m, energy: e, to_team });
                    notes.push(format!("sent {m:.0} metal and {e:.0} energy to t{to_team} on order{}", if m < metal || e < energy { format!(" (the store held {have_m:.0} metal and {have_e:.0} energy: less than asked)") } else { String::new() }));
                }
                Transfer::Units { handles, to_team } => {
                    let mut mine: Vec<UnitId> = Vec::new();
                    let mut theirs: Vec<String> = Vec::new();
                    for h in handles {
                        if let Some(g) = h.strip_prefix("group_") {
                            match self.pianist.as_ref().and_then(|p| p.groups.iter().find(|x| x.name == g)) {
                                Some(group) => mine.extend(group.members.iter().copied()),
                                None => theirs.push(h.clone()),
                            }
                        } else {
                            match self.unit_by_handle(&h, own) {
                                Some(u) => mine.push(u.id),
                                None => theirs.push(h.clone()),
                            }
                        }
                    }
                    if !theirs.is_empty() {
                        others.push(Transfer::Units { handles: theirs, to_team });
                    }
                    if !mine.is_empty() {
                        if to_team == team {
                            notes.push("a transfer to this seat's own team: nothing to do".to_string());
                            continue;
                        }
                        let n = mine.len();
                        if let Some(pianist) = self.pianist.as_mut() {
                            for group in pianist.groups.iter_mut() {
                                group.members.retain(|id| !mine.contains(id));
                            }
                            for id in &mine {
                                pianist.tasks.remove(id);
                                pianist.queued.remove(id);
                            }
                        }
                        commands.push(Command::SendUnits { units: mine, to_team });
                        notes.push(format!("gave {n} units to t{to_team} on order"));
                    }
                }
            }
        }
        if !others.is_empty() {
            shared.transfers.lock().unwrap().extend(others);
        }
        let frame = tick.frame;
        if let Some(pianist) = self.pianist.as_mut() {
            for text in notes {
                pianist.done.push(format!("{} {text}", super::clock(frame)));
                pianist.note(frame, text);
            }
        }
    }

    /// The units given to this seat this tick, for `keep_groups` to take into one group of their own.
    pub(in crate::brain) fn given_this_tick(tick: &Tick) -> Vec<UnitId> {
        tick.events.iter().filter_map(|e| if let Event::UnitGiven { unit, .. } = e { Some(*unit) } else { None }).collect()
    }

    /// Whether a soldier was given to us this tick (a newcomer of no factory, to be grouped with the others given).
    pub(in crate::brain) fn given_soldiers<'a>(tick: &'a Tick, soldiers: &[&'a OwnUnit]) -> Vec<&'a OwnUnit> {
        let given = Self::given_this_tick(tick);
        soldiers.iter().copied().filter(|u| given.contains(&u.id)).collect()
    }
}
