//! H-HANDS-HEAL (the rove-2 design §3, the hands' half; a code decision, `docs/code-decisions.md` §2): a constructor
//! or the commander of ours that a hurt rover has come to is told `Repair` on the rover (`pianist/heal.rs`). The lane's word is the
//! rover's report (`RoveReport::healer`, H-MICRO-HEAL): the builder it chose, and the order goes only once the rover
//! stands inside that builder's build distance, so the engine has nothing to walk to: **the builder never moves**
//! (the user, 2026-11-04: "I picture the rover running back to the nearest heal-capable unit to receive its
//! healing: meaning the constructors never have to waste time moving"). Told again once a second while the rover
//! stands there (the shape of `nanos.rs`); the builder's list step goes back to the front of its list and the list
//! waits (`Pianist::healing`, read by `list_is_held`) until the rover leaves, whole (the engine's repair ends by
//! itself, and the rover leaves a mobile healer at full health) or for a threat (then a `Stop`, since the engine's
//! repair would follow it). The commander never while it is under fire or on an attack (H-HANDS-COMMANDER-FIGHTS
//! outranks it). Construction turrets heal under their own standing order and need nothing here.

use std::collections::{HashMap, HashSet};

use bot_protocol::{Command, Tick, UnitId};

use super::super::roster::Kit;
use super::super::{Brain, FRAMES_PER_SECOND};
use super::menu::Order;
use super::picture::{Picture, clock};
use super::Task;

/// A healing builder is told its repair again this often while the rover stands there.
const HEAL_RETELL: i32 = FRAMES_PER_SECOND;

impl Brain {
    /// Every mobile builder a rover stands at to be healed: the repair, the hold on its list, and the release.
    pub(crate) fn heal_rovers(&mut self, tick: &Tick, kit: &Kit, picture: &Picture, commands: &mut Vec<Command>) {
        let frame = tick.frame;
        let own = &tick.snapshot.own_units;
        let Some(pianist) = self.pianist.as_ref() else { return };
        let under_fire: HashSet<UnitId> = pianist.under_fire(frame).collect();
        // What the rovers ask: each mobile builder of ours that a rover's report names and that the rover stands
        // within the build distance of; one rover a builder (the first asked).
        let mut asked: HashMap<UnitId, UnitId> = HashMap::new();
        for (rover, report) in &self.lane_roving {
            let Some(healer) = report.healer else { continue };
            let (Some(r), Some(b)) = (own.iter().find(|u| u.id == *rover), own.iter().find(|u| u.id == healer)) else { continue };
            let Some(d) = self.world.def(b.def) else { continue };
            if d.speed == 0.0 || b.being_built || r.pos.dist2d(b.pos) > d.build_distance {
                continue;
            }
            if under_fire.contains(&healer) || matches!(pianist.tasks.get(&healer), Some(Task::Attack { .. })) {
                continue;
            }
            asked.entry(healer).or_insert(*rover);
        }
        // Heals ended: the rover is no longer at this builder (whole and gone, or away from a threat), or another
        // rover has it. The repair task goes, the list resumes next pass, and a rover gone short of whole is not
        // followed.
        let ended: Vec<(UnitId, UnitId, String)> = pianist.healing.iter().filter(|(b, (r, _))| asked.get(b) != Some(r)).map(|(b, (r, _))| (*b, *r, self.actor_name(*b))).collect();
        let heals: Vec<(UnitId, UnitId, bool)> = asked
            .iter()
            .map(|(b, r)| {
                let tell = match pianist.healing.get(b) {
                    Some((rover, told)) => rover != r || frame - told >= HEAL_RETELL,
                    None => true,
                };
                (*b, *r, tell)
            })
            .collect();
        let pianist = self.pianist.as_mut().expect("pianist mode");
        for (b, r, name) in ended {
            pianist.healing.remove(&b);
            if matches!(pianist.tasks.get(&b), Some(Task::Repair { target, .. }) if *target == r) {
                pianist.tasks.remove(&b);
                let whole = own.iter().find(|u| u.id == r).is_none_or(|u| u.health >= u.max_health - 0.5);
                if !whole {
                    commands.push(Command::Stop { unit: b });
                }
                pianist.done.push(format!("{} {name} is done healing our rover {}; its list goes on", clock(frame), r.0));
            }
        }
        for (b, r, tell) in heals {
            if !tell {
                continue;
            }
            let fresh = !self.pianist.as_ref().expect("pianist mode").healing.contains_key(&b);
            self.pianist.as_mut().expect("pianist mode").healing.insert(b, (r, frame));
            // The order through the executor: the repair task, and the builder's list step back to the front of
            // its list (the list waits while `healing` holds the builder).
            let did = self.execute_builder(tick, kit, picture, b, &Order::Repair(r), false, None, commands);
            if fresh && did.is_some() {
                let words = format!("{} {} heals our rover {} standing beside it (H-HANDS-HEAL)", clock(frame), self.actor_name(b), r.0);
                self.pianist.as_mut().expect("pianist mode").done.push(words);
            }
        }
    }
}
