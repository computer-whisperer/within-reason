//! What the brain decided, in structured form, for the match record (`docs/harness/record-format.md`).
//!
//! The brain only notes; `main` drains the journal after every tick and the recorder writes it out. Notes are
//! cheap (a counter bump, or one small value per wave) so they are taken whether or not a record is being written.

use std::collections::BTreeMap;

use bot_protocol::Vec3;
use serde_json::Value;

use super::Brain;

/// One decision worth a line of its own: a note for the player, a scout sent, a list step taken.
pub struct Note {
    /// Which layer decided: `heuristic` for the bot's own rules, `jev` for the pianist's.
    pub source: &'static str,
    pub frame: i32,
    pub kind: &'static str,
    /// What the decision was made on; `Null` when the text says it all.
    pub inputs: Value,
    pub outputs: Value,
}

/// Where home and the enemy's start are in the brain's mind this tick.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct Intent {
    pub home: Vec3,
    pub enemy_start: Vec3,
}

pub use micro::Milling;

#[derive(Default)]
pub struct Journal {
    /// Heuristic firings (docs/heuristics.md) since the last drain.
    pub rules: BTreeMap<&'static str, u32>,
    /// The lane's milling counters since the last drain.
    pub milling: Milling,
    pub notes: Vec<Note>,
    pub intent: Intent,
}

impl Journal {
    pub fn rule(&mut self, rule: &'static str) {
        *self.rules.entry(rule).or_default() += 1;
    }

    /// A decision of the heuristic brain.
    pub fn note(&mut self, frame: i32, kind: &'static str, inputs: Value, outputs: Value) {
        self.note_from("heuristic", frame, kind, inputs, outputs);
    }

    /// A decision of another layer (`jev` for the pianist's).
    pub fn note_from(&mut self, source: &'static str, frame: i32, kind: &'static str, inputs: Value, outputs: Value) {
        self.notes.push(Note { source, frame, kind, inputs, outputs });
    }
}

impl Brain {
    /// Everything noted since the last call; the intent is carried over.
    pub fn take_journal(&mut self) -> Journal {
        let intent = self.journal.intent;
        std::mem::replace(&mut self.journal, Journal { intent, ..Journal::default() })
    }

    pub(super) fn journal_intent(&mut self) {
        self.journal.intent = Intent { home: self.home, enemy_start: self.enemy_base(self.home) };
    }
}
