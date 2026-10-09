//! Chores of running a headless engine from a match directory, shared by the arena and the duel runner.

use std::fs::{self, File};
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use crate::autohost::Autohost;

pub const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
/// The game version matches are played on, as a rapid tag; resolved to its full name for the start script.
pub const GAME_TAG: &str = "byar:test";
/// Wall-clock allowance for engine start-up and map loading.
pub const LOAD_ALLOWANCE: Duration = Duration::from_secs(90);
/// Resident memory one engine needs with the match template's settings, with headroom.
pub const ENGINE_MEMORY_GB: u64 = 4;

/// The full name a rapid tag stands for ("byar:test" -> "Beyond All Reason test-NNNNN-hash"), from the rapid
/// index in our data directory. A start script may name either, but the name ends up in the replay's header, and
/// the BAR lobby offers to download a game called "byar:test" for ever instead of opening the replay.
pub fn resolve_game(repo: &Path, tag: &str) -> io::Result<String> {
    let rapid = repo.join("run/data/rapid");
    let repository = tag.split(':').next().unwrap_or_default();
    for host in fs::read_dir(&rapid)?.flatten() {
        let Ok(file) = File::open(host.path().join(repository).join("versions.gz")) else { continue };
        let mut index = String::new();
        flate2::read::GzDecoder::new(file).read_to_string(&mut index)?;
        // tag,package hash,dependencies,full name
        let name = index.lines().find_map(|line| {
            let mut fields = line.split(',');
            (fields.next() == Some(tag)).then(|| fields.nth(2)).flatten()
        });
        if let Some(name) = name {
            return Ok(name.to_string());
        }
    }
    Err(io::Error::other(format!("rapid tag {tag} not found under {}", rapid.display())))
}

/// Latest game frame in the engine log: the engine prefixes its lines with `[f=NNNNNNN]` and our
/// shim prints `heartbeat f=N` once per game minute. Reads only the tail, so it is cheap to poll.
pub fn last_frame(engine_log: &Path) -> u32 {
    const TAIL: u64 = 64 * 1024;
    let Ok(mut file) = File::open(engine_log) else { return 0 };
    let len = file.metadata().map_or(0, |m| m.len());
    let mut tail = Vec::new();
    if file.seek(SeekFrom::Start(len.saturating_sub(TAIL))).is_err() || file.read_to_end(&mut tail).is_err() {
        return 0;
    }
    let tail = String::from_utf8_lossy(&tail);
    let number_after = |marker: &str| {
        tail.rmatch_indices(marker).find_map(|(at, _)| {
            let digits: String = tail[at + marker.len()..].chars().take_while(char::is_ascii_digit).collect();
            digits.parse::<u32>().ok()
        })
    };
    number_after("[f=").max(number_after("heartbeat f=")).unwrap_or(0)
}

/// Jev's price: dollars a million input tokens (the output is free).
pub const JEV_DOLLARS_PER_MILLION: f64 = 0.042;

/// A match's Jev bill so far, read from every seat's `jev-<ai>.jsonl` as the bot writes them: the `input_tokens` of
/// each request's usage, summed. The logs run to gigabytes, so each poll reads only what was written since the last
/// one, up to the last whole line.
#[derive(Default)]
pub struct JevMeter {
    read: std::collections::HashMap<PathBuf, u64>,
    pub tokens: u64,
}

impl JevMeter {
    pub fn dollars(&self) -> f64 {
        self.tokens as f64 * JEV_DOLLARS_PER_MILLION / 1e6
    }

    /// Reads what the seats' logs in `dir` gained since the last poll.
    pub fn poll(&mut self, dir: &Path) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for path in entries.flatten().map(|e| e.path()) {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if !(name.starts_with("jev-") && name.ends_with(".jsonl")) {
                continue;
            }
            let from = self.read.get(&path).copied().unwrap_or(0);
            let Ok(mut file) = File::open(&path) else { continue };
            let mut fresh = Vec::new();
            if file.seek(SeekFrom::Start(from)).is_err() || file.read_to_end(&mut fresh).is_err() {
                continue;
            }
            let Some(end) = fresh.iter().rposition(|b| *b == b'\n') else { continue };
            let text = String::from_utf8_lossy(&fresh[..=end]);
            const KEY: &str = "\"input_tokens\":";
            for (at, _) in text.match_indices(KEY) {
                let digits: String = text[at + KEY.len()..].chars().skip_while(|c| *c == ' ').take_while(char::is_ascii_digit).collect();
                self.tokens += digits.parse::<u64>().unwrap_or(0);
            }
            self.read.insert(path, from + end as u64 + 1);
        }
    }
}

pub fn available_memory_gb() -> io::Result<u64> {
    let meminfo = fs::read_to_string("/proc/meminfo")?;
    let kb = meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemAvailable:")?.trim().strip_suffix("kB")?.trim().parse::<u64>().ok())
        .ok_or_else(|| io::Error::other("no MemAvailable in /proc/meminfo"))?;
    Ok(kb / (1024 * 1024))
}

pub fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// The engine's archive cache copied into every match directory (`run/cache-template`, 4 MB a match, the same bytes in
/// each) is of no use once the match has been played: removed from every match of a batch, after the batch has
/// refreshed the template from it.
pub fn drop_match_caches(batch_dir: &Path) {
    let Ok(entries) = fs::read_dir(batch_dir) else { return };
    for cache in entries.flatten().map(|e| e.path().join("cache")).filter(|c| c.is_dir()) {
        if let Err(e) = fs::remove_dir_all(&cache) {
            eprintln!("could not remove {}: {e}", cache.display());
        }
    }
}

/// The zstd level the hands' logs are compressed at: 25 to 1 on a line of menus, at two gigabytes a second.
const ZSTD_LEVEL: i32 = 3;
const SKIPPABLE_MAGIC: u32 = 0x184D_2A5E;
const SEEKABLE_MAGIC: u32 = 0x8F92_EAB1;

/// The pianist's logs of a finished match (`jev-<n>.jsonl`, a gigabyte a seat: 30 to 50 MB a game minute), compressed
/// into `jev-<n>.jsonl.zst`: one zstd frame a line, and last a seek table in the Zstandard Seekable Format's skippable
/// frame (an entry a frame, its compressed and its uncompressed size as little-endian u32s; then the frame count, a
/// descriptor byte and the magic 0x8F92EAB1), so that `run/jevlog.py` reaches any line without reading the rest while
/// `zstd -d` reads the whole. The raw log is removed only after every frame has read back as its line. Returns the raw
/// and the compressed bytes summed over the seats' logs; an empty log is left as it is.
pub fn compress_jev_logs(dir: &Path) -> io::Result<(u64, u64)> {
    let mut totals = (0, 0);
    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        let seat = name.strip_prefix("jev-").and_then(|rest| rest.strip_suffix(".jsonl"));
        if !seat.is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())) || entry.metadata()?.len() == 0 {
            continue;
        }
        let (raw, packed) = compress_log(&path)?;
        totals.0 += raw;
        totals.1 += packed;
    }
    Ok(totals)
}

fn compress_log(path: &Path) -> io::Result<(u64, u64)> {
    let packed_path = path.with_extension("jsonl.zst");
    let temp = path.with_extension("jsonl.zst.tmp");
    let too_big = |_| io::Error::other("a line past 4 GB");
    let mut entries: Vec<(u32, u32)> = Vec::new();
    let mut raw_bytes = 0u64;
    {
        let mut reader = io::BufReader::new(File::open(path)?);
        let mut writer = io::BufWriter::new(File::create(&temp)?);
        let mut line = Vec::new();
        loop {
            line.clear();
            if reader.read_until(b'\n', &mut line)? == 0 {
                break;
            }
            let frame = zstd::bulk::compress(&line, ZSTD_LEVEL)?;
            writer.write_all(&frame)?;
            entries.push((u32::try_from(frame.len()).map_err(too_big)?, u32::try_from(line.len()).map_err(too_big)?));
            raw_bytes += line.len() as u64;
        }
        let mut table = Vec::with_capacity(entries.len() * 8 + 9);
        for (compressed, uncompressed) in &entries {
            table.extend_from_slice(&compressed.to_le_bytes());
            table.extend_from_slice(&uncompressed.to_le_bytes());
        }
        table.extend_from_slice(&u32::try_from(entries.len()).map_err(too_big)?.to_le_bytes());
        table.push(0);
        table.extend_from_slice(&SEEKABLE_MAGIC.to_le_bytes());
        writer.write_all(&SKIPPABLE_MAGIC.to_le_bytes())?;
        writer.write_all(&u32::try_from(table.len()).map_err(too_big)?.to_le_bytes())?;
        writer.write_all(&table)?;
        writer.flush()?;
    }
    // Read back before the raw log goes: every frame is its line.
    {
        let mut raw = io::BufReader::new(File::open(path)?);
        let mut packed = io::BufReader::new(File::open(&temp)?);
        let (mut line, mut frame) = (Vec::new(), Vec::new());
        for (compressed, uncompressed) in &entries {
            line.clear();
            raw.read_until(b'\n', &mut line)?;
            frame.resize(*compressed as usize, 0);
            packed.read_exact(&mut frame)?;
            if zstd::bulk::decompress(&frame, *uncompressed as usize)? != line {
                let _ = fs::remove_file(&temp);
                return Err(io::Error::other(format!("{} did not read back as written; the raw log stays", temp.display())));
            }
        }
    }
    let packed_bytes = fs::metadata(&temp)?.len();
    fs::rename(&temp, &packed_path)?;
    fs::remove_file(path)?;
    Ok((raw_bytes, packed_bytes))
}

/// The engine checksums every archive unless the write dir already holds a cache that matches
/// their paths and mtimes. Keep a copy of the newest one for the next batch's match dirs.
pub fn refresh_cache_template(repo: &Path, batch_dir: &Path) {
    let source = batch_dir.join("00/cache");
    let template = repo.join("run/cache-template");
    if source.join("ArchiveCache22.lua").is_file() {
        let _ = fs::remove_dir_all(&template);
        if let Err(e) = copy_tree(&source, &template) {
            eprintln!("could not refresh {}: {e}", template.display());
        }
    }
}

pub fn stop(engine: &mut Child, autohost: &mut Autohost) {
    let _ = autohost.send("/kill");
    let give_up = Instant::now() + Duration::from_secs(30);
    while Instant::now() < give_up {
        if matches!(engine.try_wait(), Ok(Some(_))) {
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let _ = engine.kill();
    let _ = engine.wait();
}

/// Short commit hash, with `+` when the working tree has uncommitted changes to tracked files.
pub fn git_commit(repo: &Path) -> String {
    let git = |args: &[&str]| Command::new("git").arg("-C").arg(repo).args(args).output().ok();
    let hash = git(&["rev-parse", "--short", "HEAD"]).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let dirty = git(&["status", "--porcelain", "--untracked-files=no"]).is_some_and(|o| !o.stdout.is_empty());
    format!("{}{}", hash.unwrap_or_else(|| "unknown".into()), if dirty { "+" } else { "" })
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// The meter adds each seat's usage once, reads only whole lines, and takes up where it left off.
    #[test]
    fn the_jev_meter_reads_what_is_new_in_every_seat_s_log() {
        let dir = std::env::temp_dir().join(format!("jev-meter-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let line = |tokens: u32| format!("{{\"t\":\"call\",\"usage\":{{\"input_tokens\":{tokens},\"output_tokens\":7}}}}\n");
        fs::write(dir.join("jev-0.jsonl"), format!("{}{}", line(1_000_000), &line(500)[..20])).unwrap();
        fs::write(dir.join("jev-1.jsonl"), line(250_000)).unwrap();
        fs::write(dir.join("bot.log"), line(9)).unwrap();
        let mut meter = JevMeter::default();
        meter.poll(&dir);
        assert_eq!(meter.tokens, 1_250_000, "the half-written line waits");
        let mut log = fs::OpenOptions::new().append(true).open(dir.join("jev-0.jsonl")).unwrap();
        log.write_all(line(500)[20..].as_bytes()).unwrap();
        log.write_all(line(3).as_bytes()).unwrap();
        meter.poll(&dir);
        meter.poll(&dir);
        assert_eq!(meter.tokens, 1_250_503);
        assert!((meter.dollars() - 1.250503 * JEV_DOLLARS_PER_MILLION).abs() < 1e-9);
        fs::remove_dir_all(&dir).unwrap();
    }

    /// Each seat's log becomes a frame a line with a seek table; the whole decodes as the raw log did, a frame alone
    /// decodes as its line, a torn last line is kept, and files that are not a seat's log are left alone.
    #[test]
    fn the_hands_logs_are_compressed_a_frame_a_line_and_read_back() {
        let dir = std::env::temp_dir().join(format!("jev-pack-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let menus = "{\"t\":\"pass\",\"menus\":[".to_string() + &"{\"words\":\"group_A walks to spot_12\"},".repeat(2000) + "{}]}\n";
        let raw = format!("{{\"t\":\"header\",\"version\":3}}\n{menus}{{\"t\":\"call\",\"usage\":{{\"input_tokens\":9}}}}\n{{\"t\":\"pla");
        fs::write(dir.join("jev-0.jsonl"), &raw).unwrap();
        fs::write(dir.join("jev-1.jsonl"), "").unwrap();
        fs::write(dir.join("jev-sweep.jsonl"), "{}\n").unwrap();
        let (raw_bytes, packed_bytes) = compress_jev_logs(&dir).unwrap();
        assert_eq!(raw_bytes, raw.len() as u64);
        assert!(packed_bytes < raw_bytes / 10, "{packed_bytes} of {raw_bytes}");
        assert!(!dir.join("jev-0.jsonl").exists() && dir.join("jev-1.jsonl").exists() && dir.join("jev-sweep.jsonl").exists());
        let packed = fs::read(dir.join("jev-0.jsonl.zst")).unwrap();
        assert_eq!(packed.len() as u64, packed_bytes);
        assert_eq!(zstd::stream::decode_all(&packed[..]).unwrap(), raw.as_bytes(), "the stream decodes whole, the table skipped");
        let n = packed.len();
        assert_eq!(u32::from_le_bytes(packed[n - 4..].try_into().unwrap()), SEEKABLE_MAGIC);
        let frames = u32::from_le_bytes(packed[n - 9..n - 5].try_into().unwrap()) as usize;
        assert_eq!(frames, 4);
        let table = n - 8 - frames * 8 - 9;
        assert_eq!(u32::from_le_bytes(packed[table..table + 4].try_into().unwrap()), SKIPPABLE_MAGIC);
        let mut offset = 0;
        for (i, line) in raw.split_inclusive('\n').enumerate() {
            let at = table + 8 + i * 8;
            let compressed = u32::from_le_bytes(packed[at..at + 4].try_into().unwrap()) as usize;
            let uncompressed = u32::from_le_bytes(packed[at + 4..at + 8].try_into().unwrap()) as usize;
            assert_eq!(uncompressed, line.len());
            assert_eq!(zstd::bulk::decompress(&packed[offset..offset + compressed], uncompressed).unwrap(), line.as_bytes());
            offset += compressed;
        }
        assert_eq!(offset, table);
        fs::remove_dir_all(&dir).unwrap();
    }

    /// Every match of a batch loses its copy of the engine's archive cache; nothing else goes.
    #[test]
    fn the_matches_caches_go_after_the_batch() {
        let batch = std::env::temp_dir().join(format!("jev-caches-{}", std::process::id()));
        for n in ["00", "01"] {
            fs::create_dir_all(batch.join(n).join("cache")).unwrap();
            fs::write(batch.join(n).join("cache/ArchiveCache22.lua"), "x").unwrap();
            fs::write(batch.join(n).join("engine.log"), "x").unwrap();
        }
        drop_match_caches(&batch);
        assert!(!batch.join("00/cache").exists() && !batch.join("01/cache").exists());
        assert!(batch.join("00/engine.log").exists() && batch.join("01/engine.log").exists());
        fs::remove_dir_all(&batch).unwrap();
    }
}
