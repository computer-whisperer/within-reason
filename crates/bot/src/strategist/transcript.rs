//! `strategist.jsonl`: everything the strategist saw, said and set, for post-game analysis.

use std::path::Path;

use serde_json::Value;

pub struct Transcript {
    file: crate::logs::Log,
}

impl Transcript {
    pub fn create(path: &Path) -> std::io::Result<Self> {
        Ok(Transcript { file: crate::logs::Log::create(path.to_path_buf())? })
    }

    /// One row, serialised and written on the logs thread (`crate::logs`).
    pub fn record(&self, entry: Value) {
        self.file.write_json(entry);
    }
}
