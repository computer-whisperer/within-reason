//! One writer thread for the bot's logs. The brain and the lane hand it a line and go on; it opens the files,
//! serialises and writes. Until 2026-11-07 the Jev log (rows of up to 3 MB, 51 MB in one minute on one seat of
//! 1794076407-human, 2.4 GB a game over eight seats) was serialised and written on the brain's thread and the record
//! on the lane's. Neither thread was seen to block on this NVMe in the two lobby games of that day (the lane's
//! slowest tick a second 27-87 ms at the 99th percentile), and a slower disk, or a full page cache, would have
//! stalled a decision; no thread that decides touches a file now. Bounded: past `BUDGET_BYTES` of text or
//! `BUDGET_ROWS` of rows waiting, a line is dropped and counted rather than waited for, and the count is said at
//! the end (`flush`).

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel, sync_channel};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

/// Bytes of text waiting to be written before lines are dropped.
const BUDGET_BYTES: usize = 256 << 20;
/// JSON rows waiting to be serialised and written before rows are dropped (a row is up to a few MB).
const BUDGET_ROWS: usize = 2_000;
/// How long `flush` waits for the writer at the end.
const FLUSH_WAIT: Duration = Duration::from_secs(30);

enum Job {
    Text(Arc<PathBuf>, String),
    Json(Arc<PathBuf>, serde_json::Value),
    Flush(std::sync::mpsc::SyncSender<()>),
}

struct Writer {
    to: Mutex<Sender<Job>>,
    bytes: AtomicUsize,
    rows: AtomicUsize,
    dropped: AtomicUsize,
}

static WRITER: OnceLock<Writer> = OnceLock::new();

fn writer() -> &'static Writer {
    WRITER.get_or_init(|| {
        let (to, from) = channel();
        std::thread::Builder::new().name("logs".into()).spawn(move || run(from)).expect("a logs thread");
        Writer { to: Mutex::new(to), bytes: AtomicUsize::new(0), rows: AtomicUsize::new(0), dropped: AtomicUsize::new(0) }
    })
}

fn run(from: Receiver<Job>) {
    let mut files: HashMap<Arc<PathBuf>, File> = HashMap::new();
    let write = |files: &mut HashMap<Arc<PathBuf>, File>, path: &Arc<PathBuf>, text: &str| {
        if !files.contains_key(path) {
            match OpenOptions::new().create(true).append(true).open(path.as_path()) {
                Ok(f) => {
                    files.insert(path.clone(), f);
                }
                Err(e) => {
                    eprintln!("logs: {} cannot be opened: {e}", path.display());
                    return;
                }
            }
        }
        if let Some(f) = files.get_mut(path) {
            let _ = f.write_all(text.as_bytes());
        }
    };
    while let Ok(job) = from.recv() {
        match job {
            Job::Text(path, text) => {
                write(&mut files, &path, &text);
                writer().bytes.fetch_sub(text.len(), Ordering::Relaxed);
            }
            Job::Json(path, value) => {
                // One write a line: a `Value` written straight to a file is a system call a fragment.
                let mut text = value.to_string();
                text.push('\n');
                write(&mut files, &path, &text);
                writer().rows.fetch_sub(1, Ordering::Relaxed);
            }
            Job::Flush(done) => {
                let _ = done.send(());
            }
        }
    }
}

/// A log file written by the writer thread. Cloning shares the file.
#[derive(Clone)]
pub struct Log {
    path: Arc<PathBuf>,
}

impl Log {
    /// The file made anew (empty) now, written through the writer from here on.
    pub fn create(path: PathBuf) -> std::io::Result<Log> {
        File::create(&path)?;
        Ok(Log { path: Arc::new(path) })
    }

    /// The file appended to (made if missing), written through the writer from here on.
    pub fn append(path: PathBuf) -> std::io::Result<Log> {
        OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Log { path: Arc::new(path) })
    }

    /// Text as given, written as is.
    pub fn write(&self, text: String) {
        let w = writer();
        if w.bytes.load(Ordering::Relaxed) > BUDGET_BYTES {
            w.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        w.bytes.fetch_add(text.len(), Ordering::Relaxed);
        let _ = w.to.lock().unwrap().send(Job::Text(self.path.clone(), text));
    }

    /// A JSON row, serialised on the writer's thread and written as one line.
    pub fn write_json(&self, value: serde_json::Value) {
        let w = writer();
        if w.rows.load(Ordering::Relaxed) > BUDGET_ROWS {
            w.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        w.rows.fetch_add(1, Ordering::Relaxed);
        let _ = w.to.lock().unwrap().send(Job::Json(self.path.clone(), value));
    }
}

/// Waits for everything handed to the writer so far to be written (up to `FLUSH_WAIT`), and says what was dropped.
/// For the process's end: `std::process::exit` does not wait for the writer.
pub fn flush() {
    let Some(w) = WRITER.get() else { return };
    let (done, wait) = sync_channel(1);
    if w.to.lock().unwrap().send(Job::Flush(done)).is_ok() && wait.recv_timeout(FLUSH_WAIT).is_err() {
        eprintln!("logs: the writer did not finish in {} s; {} MB of lines left unwritten", FLUSH_WAIT.as_secs(), w.bytes.load(Ordering::Relaxed) >> 20);
    }
    let dropped = w.dropped.load(Ordering::Relaxed);
    if dropped > 0 {
        eprintln!("logs: {dropped} lines dropped past the writer's budget");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Text and rows reach the file in the order given, and `flush` sees them written.
    #[test]
    fn lines_reach_the_file_in_order() {
        let dir = std::env::temp_dir().join(format!("wr-logs-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.jsonl");
        let log = Log::create(path.clone()).unwrap();
        log.write("first\n".to_string());
        log.write_json(serde_json::json!({ "t": "row", "n": 2 }));
        log.clone().write("third\n".to_string());
        flush();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text, "first\n{\"n\":2,\"t\":\"row\"}\nthird\n");
        // Appending continues the file; creating starts it over.
        Log::append(path.clone()).unwrap().write("fourth\n".to_string());
        flush();
        assert!(std::fs::read_to_string(&path).unwrap().ends_with("third\nfourth\n"));
        Log::create(path.clone()).unwrap();
        flush();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
