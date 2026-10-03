//! `strategist.jsonl`: everything the strategist saw, said and set, for post-game analysis.

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use serde_json::Value;

pub struct Transcript {
    file: Mutex<File>,
}

impl Transcript {
    pub fn create(path: &Path) -> std::io::Result<Self> {
        Ok(Transcript { file: Mutex::new(File::create(path)?) })
    }

    pub fn record(&self, entry: Value) {
        // One write a line: a `Value` written straight to the file is a system call a fragment (player-51's
        // profile: a quarter of the bot's time).
        let mut line = entry.to_string();
        line.push('\n');
        let _ = self.file.lock().unwrap().write_all(line.as_bytes());
    }
}
