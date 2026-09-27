//! When to wake the field commander: it names the conditions (`wait` tool), the brain watches for them each tick
//! and holds the game while the commander takes its turn (`DESIGN.md`, "Field commander").

use std::sync::atomic::Ordering;

use std::collections::BTreeMap;

use bot_protocol::Tick;

use super::pianist::GroupTask;
use super::roster::Kit;
use super::{Brain, FRAMES_PER_SECOND};

/// Turns are at least this far apart in game time, however many conditions fire.
const MIN_GAP_FRAMES: i32 = 5 * FRAMES_PER_SECOND;
/// After delayed orders land, the hands get this long to act on them before the next turn: the flight-review wake
/// fired the moment they landed and the report showed the state from before them, which four of six models read
/// as orders that had failed and re-issued or reversed (the models-medium reviews, 2026-09-27).
const LANDING_GRACE: i32 = 3 * FRAMES_PER_SECOND;
const THREAT_RADIUS: f32 = 600.0;
/// The commander is woken when our extractor count has made no new high for this long, and again this long after.
const STAGNATION_FRAMES: i32 = 4 * 60 * FRAMES_PER_SECOND;
/// H-WAKE-HOT-FLOOR: while a fight is on (a group or squad engaged, or a unit of ours lost within `HOT_LOSS_FRAMES`)
/// the quiet time the commander set is capped at this (upgrade-2: the player set 40 s during the collapse, slept 35 s,
/// and its next turn had nine losses on the way and seven in the one after).
const HOT_MAX_SECONDS: u32 = 10;
/// The first turn waits this long at most for every seat of ours to publish.
const SEATS_WAIT_FRAMES: i32 = 15 * FRAMES_PER_SECOND;
const HOT_LOSS_FRAMES: i32 = 30 * FRAMES_PER_SECOND;
/// H-WAKE-WRECKS: metal lying in wreck fields with no enemy in sight wakes the player at this much, and again each time it
/// has grown by this much since (the replay survey: resurrection bots in 24 of 58 sides, a median of ten a side).
const WRECK_WAKE_STEP: f32 = 500.0;

#[derive(Default)]
pub struct WakeState {
    threatened_extractors: usize,
    pool_met: bool,
    /// The most extractors we have held, and when we first held that many.
    pub(super) extractor_peak: usize,
    pub(super) growth_frame: i32,
    last_stagnation_wake: i32,
    /// Once a minute: (frame, extractors, metal income, army metal). The commander is shown the curve, not only the level.
    pub(super) history: Vec<(i32, usize, f32, u32)>,
    /// Frames at which we lost an extractor, for the last few minutes (the brain's own list forgets after one).
    pub(super) losses: Vec<i32>,
    /// Reasons that fired while a turn could not be taken yet.
    pending: Vec<String>,
    /// The turn whose orders are on their way (the think penalty), while they are (H-WAKE-FLIGHT-REVIEW).
    in_flight: Option<i32>,
    /// The frame the last orders in flight landed: no turn until the hands have acted on them (`LANDING_GRACE`).
    landed: Option<i32>,
    /// The safe wreck metal the player was last woken for (H-WAKE-WRECKS).
    wreck_wake_level: f32,
}

impl Brain {
    pub(super) fn track_growth(&mut self, tick: &Tick, kit: &Kit) {
        let extractors = tick.snapshot.own_units.iter().filter(|u| kit.is_extractor(u.def) && !u.being_built).count();
        if extractors > self.wake.extractor_peak {
            self.wake.extractor_peak = extractors;
            self.wake.growth_frame = tick.frame;
        }
        let lost_now = self.extractor_losses.iter().filter(|f| **f == tick.frame).count();
        self.wake.losses.extend(std::iter::repeat_n(tick.frame, lost_now));
        self.wake.losses.retain(|f| tick.frame - f < 3 * 60 * FRAMES_PER_SECOND);
        if self.wake.history.last().is_none_or(|(frame, ..)| tick.frame - frame >= 60 * FRAMES_PER_SECOND) {
            let army: f32 = tick.snapshot.own_units.iter().filter(|u| self.is_army(u, kit)).filter_map(|u| self.world.def(u.def)).map(|d| d.metal_cost).sum();
            self.wake.history.push((tick.frame, extractors, tick.snapshot.metal.income, army as u32));
        }
    }

    /// The history sample nearest to `minutes` ago, if the game is that old: (extractors, metal income, army metal).
    pub(super) fn minutes_ago(&self, frame: i32, minutes: i32) -> Option<(usize, f32, u32)> {
        let then = frame - minutes * 60 * FRAMES_PER_SECOND;
        if then < 0 {
            return None;
        }
        self.wake.history.iter().min_by_key(|(f, ..)| (f - then).abs()).map(|(_, x, income, army)| (*x, *income, *army))
    }

    pub(super) fn wake_commander_if_due(&mut self, tick: &Tick, kit: &Kit) {
        let Some(shared) = self.strategist.clone() else { return };
        if !shared.gated.load(Ordering::Relaxed) {
            return;
        }
        // Of several seats under one commander only the lead asks for turns (held, it holds the engine and so every
        // seat); the others hand it what is theirs alone to know.
        if shared.lead().is_some_and(|lead| lead != self.world.hello.team) {
            if shared.wake.lock().unwrap().extractor_lost && self.extractor_losses.back() == Some(&tick.frame) {
                shared.trigger("an extractor was destroyed".into());
            }
            return;
        }
        let last_turn_frame = shared.last_turn_frame.load(Ordering::Relaxed);
        // A commander still "thinking" (its last orders not yet in force) cannot be asked again; what happens meanwhile
        // is kept for its next turn.
        let busy = shared.apply_delayed(tick.frame);
        let wake = shared.wake.lock().unwrap().clone();
        let field = shared.field();
        let mut reasons: Vec<String> = std::mem::take(&mut self.wake.pending);
        reasons.append(&mut shared.triggers.lock().unwrap());
        // H-WAKE-FLIGHT-REVIEW: orders that land after losses on their way are reviewed the moment they land (upgrade-2:
        // 45 of 84 flights had a loss of ours inside, 9 woke a turn at landing, 11 waited 7 to 37 s for the timer).
        match (busy, self.wake.in_flight) {
            (true, None) => self.wake.in_flight = Some(last_turn_frame),
            (false, Some(turn)) => {
                self.wake.in_flight = None;
                self.wake.landed = Some(tick.frame);
                let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
                let mut metal = 0.0;
                for (_, _, def) in self.unit_losses.iter().filter(|(f, ..)| *f >= turn) {
                    *kinds.entry(self.name(*def)).or_default() += 1;
                    metal += self.world.def(*def).map_or(0.0, |d| d.metal_cost);
                }
                if !kinds.is_empty() {
                    let names: Vec<String> = kinds.iter().map(|(name, n)| format!("{n} {name}")).collect();
                    reasons.push(format!(
                        "while your orders were on their way ({} s) we lost {} units worth {metal:.0} metal: {}",
                        (tick.frame - turn) / FRAMES_PER_SECOND,
                        kinds.values().sum::<usize>(),
                        names.join(", ")
                    ));
                }
            }
            _ => {}
        }

        // Conditions wake on their rising edge: a raid is news when it starts, not every tick it lasts.
        let threatened: Vec<&str> =
            field.extractors.iter().filter(|x| x.enemies_within_600 > 0).map(|x| x.at.grid.as_str()).collect();
        if wake.enemy_near_extractor && threatened.len() > self.wake.threatened_extractors {
            reasons.push(format!("enemies within {THREAT_RADIUS:.0} of our extractors at {}", threatened.join(", ")));
        }
        self.wake.threatened_extractors = threatened.len();
        // The pianist's groups: an engagement is news when the hands begin it.
        let began: Vec<String> = std::mem::take(&mut shared.hands.lock().unwrap().entry(self.world.hello.team).or_default().engaged);
        if wake.squad_engaged && !began.is_empty() {
            reasons.push(format!("your hands sent {} to attack an enemy party", began.join(", ")));
        }
        if wake.extractor_lost && self.extractor_losses.back() == Some(&tick.frame) {
            reasons.push("an extractor was destroyed".into());
        }
        // A person spoke: news once, when the line arrives (the report carries the words).
        if wake.chat && self.heard_chat_at == tick.frame {
            reasons.push("someone in the game said something (see the chat lines)".into());
        }
        let pool_met = !wake.pool_reaches.is_empty()
            && wake.pool_reaches.iter().all(|(name, n)| field.unassigned.iter().any(|(have, count)| have == name && count >= n));
        if pool_met && !self.wake.pool_met {
            reasons.push("the unassigned soldiers you were waiting for are ready".into());
        }
        self.wake.pool_met = pool_met;
        // Not the commander's to switch off: every other condition is a threat, and a commander woken only by threats
        // defends four extractors for half an hour.
        let stagnant = tick.frame - self.wake.growth_frame.max(self.wake.last_stagnation_wake);
        if stagnant >= STAGNATION_FRAMES && field.score.free_spots > 0 {
            self.wake.last_stagnation_wake = tick.frame;
            reasons.push(format!(
                "no growth: we have not held more than {} extractors for {} min, with {} free spots we can walk to",
                self.wake.extractor_peak,
                (tick.frame - self.wake.growth_frame) / (60 * FRAMES_PER_SECOND),
                field.score.free_spots
            ));
        }

        // H-WAKE-WRECKS: metal on the ground is income nobody is collecting.
        let safe_wrecks: f32 = self.reclaim.fields.iter().filter(|f| f.safe).map(|f| f.metal).sum();
        if safe_wrecks < WRECK_WAKE_STEP / 2.0 {
            self.wake.wreck_wake_level = 0.0;
        } else if safe_wrecks >= self.wake.wreck_wake_level + WRECK_WAKE_STEP {
            self.wake.wreck_wake_level = (safe_wrecks / WRECK_WAKE_STEP).floor() * WRECK_WAKE_STEP;
            let fields: Vec<String> = self.reclaim.fields.iter().filter(|f| f.safe && f.metal >= 100.0).map(|f| format!("{:.0} at {}", f.metal, self.world.grid(f.at))).collect();
            reasons.push(format!(
                "wrecks worth {safe_wrecks:.0} metal lie where no enemy is in sight ({}): constructors take them apart (the reclaim option, within 1,800 of a field), resurrection bots (`produce` {}) raise the soldiers among them and take the rest apart on their own",
                fields.join(", "),
                self.name(kit.resurrector)
            ));
        }
        let since = tick.frame - last_turn_frame;
        let hot = self.unit_losses.back().is_some_and(|(f, ..)| tick.frame - f <= HOT_LOSS_FRAMES)
            || self.pianist.as_ref().is_some_and(|p| p.groups.iter().any(|g| matches!(g.task, GroupTask::Engage { .. })));
        let max_seconds = if hot { wake.max_seconds.min(HOT_MAX_SECONDS) } else { wake.max_seconds };
        if reasons.is_empty() && since >= max_seconds as i32 * FRAMES_PER_SECOND {
            reasons.push(if max_seconds < wake.max_seconds {
                format!("{} s have passed (your wait of {} s is capped at {HOT_MAX_SECONDS} s while a fight is on: a group engaged, or a loss in the last 30 s)", since / FRAMES_PER_SECOND, wake.max_seconds)
            } else {
                format!("{} s have passed", since / FRAMES_PER_SECOND)
            });
        }
        // Under the pianist the player opens the game, since the factory is its choice (comet-0: the hands built a
        // bot lab under the default text before the player's first turn, on a vehicles map). The field commander
        // still comes in when the first factory stands.
        let opens = self.pianist.is_some();
        // With several seats of ours the first turn waits until every seat has published, else the report shows
        // one seat and the player takes itself for that seat alone (bluegecko-2v1-great-divide, turn 1 at frame 1:
        // "south-east seat here, Cortex", a Cortex opening for the Armada seat too). 15 s at most.
        // ... and its field (roster, faction): bluegecko-3v1-comet-catcher-7's first turn came at frame 1 with the
        // other seats' faction "chosen at start", the player wrote both factions' names in one list, and the
        // whole list was refused.
        let seats_in = (shared.live_seats().len() >= self.seats_of_ours() && shared.field().factions.len() >= self.seats_of_ours()) || tick.frame >= SEATS_WAIT_FRAMES;
        let first_turn = last_turn_frame == 0 && seats_in && (opens || tick.snapshot.own_units.iter().any(|u| kit.is_factory(u.def)));
        if first_turn {
            reasons.push(if opens { "the game begins: the opening is yours" } else { "our first factory is up" }.into());
        }
        let settling = self.wake.landed.is_some_and(|f| tick.frame - f < LANDING_GRACE);
        let too_soon = if last_turn_frame == 0 { !first_turn } else { since < MIN_GAP_FRAMES || settling };
        if reasons.is_empty() || too_soon || busy {
            // Keep the newest word on each subject: "enemies within ... at C3" then "... at C3, C4" is one piece of news.
            let subject = |r: &String| r.split(|c: char| c == ':' || c.is_ascii_digit()).next().unwrap_or_default().to_string();
            let mut kept: Vec<String> = Vec::new();
            for reason in reasons.into_iter().rev() {
                if !kept.iter().any(|k| subject(k) == subject(&reason)) {
                    kept.push(reason);
                }
            }
            kept.reverse();
            let reasons = kept;
            self.wake.pending = reasons;
            return;
        }
        shared.last_turn_frame.store(tick.frame, Ordering::Relaxed);
        if shared.lockstep.load(Ordering::Relaxed) {
            shared.hold_for_turn(reasons.join("; "), tick.frame);
        } else {
            shared.request_turn(reasons.join("; "));
        }
    }
}
