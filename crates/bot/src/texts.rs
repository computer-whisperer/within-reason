//! The texts the models read: the player's, commander's and strategist's prompts, the briefs behind them, and the
//! hands' rules and default instructions. Each is a file in the repository, read at every use, so an edit takes
//! effect without a rebuild (the user, 2026-09-22): the hands' rules on their next call, a session prompt on the
//! next session, which the driver starts as soon as the prompt on disk has changed. The copy compiled in is the
//! fallback for a bot run away from its checkout; `WITHIN_REASON_TEXTS` names the root the texts are read from.
//!
//! A game reads from a snapshot, not the checkout (the user, 2026-10-21: edits to the codebase must not reach a
//! running game; docs/design/2026-10-21-brief-rework.md §5). `bot --snapshot DIR` copies every text, the reference
//! tables, the binary itself and the checkout's git state into DIR, with the checkout's relative paths kept; the
//! arena (per batch) and `run/human_game.sh` (per game) make one and run `DIR/bot` with `WITHIN_REASON_TEXTS=DIR`.
//! The marker file is copied too, so `DIR/bot` finds DIR as its root even without the variable. The record then
//! carries the texts a game was played with.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub struct Text {
    /// Relative to the repository root.
    pub path: &'static str,
    pub compiled: &'static str,
}

pub const PLAYER_PROMPT: Text = Text { path: "crates/bot/src/strategist/player.md", compiled: include_str!("strategist/player.md") };
/// Added to the player's role text in a game with a commander above it (`strategist/command.rs`).
pub const PLAYER_UNDER_COMMAND: Text = Text { path: "crates/bot/src/strategist/player-under-command.md", compiled: include_str!("strategist/player-under-command.md") };
/// Added to the player's role text by `arena --packet-example` (`WITHIN_REASON_PACKET_EXAMPLE`): the shape of a
/// packet with two of Opus's from player-59, after Sonnet's packets were found half as long and without a plan
/// paragraph or a paragraph an actor (human-23).
pub const PACKET_EXAMPLE: Text = Text { path: "crates/bot/src/strategist/packet-example.md", compiled: include_str!("strategist/packet-example.md") };
/// The role text of the side's commander.
pub const COMMANDER_PROMPT: Text = Text { path: "crates/bot/src/strategist/commander.md", compiled: include_str!("strategist/commander.md") };
pub const PLAYER_BRIEF: Text = Text { path: "docs/briefs/player.md", compiled: include_str!("../../../docs/briefs/player.md") };
/// The brief as game experience (`arena --brief experience`, `WITHIN_REASON_BRIEF=experience`): assembled by
/// `run/build_brief.py` from `docs/briefs/experience/` and the generated reference sets
/// (docs/design/2026-10-03-brief-rewrite-and-commander-pack.md). Played against `PLAYER_BRIEF` on the same seeds
/// until one of them goes.
pub const PLAYER_BRIEF_EXPERIENCE: Text = Text { path: "docs/briefs/player-experience.md", compiled: include_str!("../../../docs/briefs/player-experience.md") };
/// Read beside the brief by an Opus player only (`strategist::system_prompt`): what that model does in the seat
/// that the brief's facts do not correct (docs/studies/2026-10-07-raid-packets.md).
pub const PLAYER_OPUS: Text = Text { path: "docs/briefs/player-opus.md", compiled: include_str!("../../../docs/briefs/player-opus.md") };
pub const HANDS_RULES: Text = Text { path: "crates/bot/src/brain/pianist/rules.md", compiled: include_str!("brain/pianist/rules.md") };
/// The rules a gate is sent under the lean diet's `standing_cut` (`pianist/diet.rs`): the gate answers nearly as it
/// does with no rules at all, the pick does not, so the pick keeps `HANDS_RULES`
/// (`docs/studies/2026-10-04-jev-where-the-tokens-go.md` §14).
pub const HANDS_RULES_GATE: Text = Text { path: "crates/bot/src/brain/pianist/rules_gate.md", compiled: include_str!("brain/pianist/rules_gate.md") };
pub const HANDS_DEFAULT: Text = Text { path: "crates/bot/src/brain/pianist/default.md", compiled: include_str!("brain/pianist/default.md") };

/// Every text, for the snapshot and the tests.
pub const ALL: [&Text; 10] = [&PLAYER_PROMPT, &PLAYER_UNDER_COMMAND, &PACKET_EXAMPLE, &COMMANDER_PROMPT, &PLAYER_BRIEF, &PLAYER_BRIEF_EXPERIENCE, &PLAYER_OPUS, &HANDS_RULES, &HANDS_RULES_GATE, &HANDS_DEFAULT];

/// The directories read whole at run time, beside the texts: the reference tables (`strategist/reference.rs` reads
/// `docs/briefs/reference/<map>-<faction>.json` by the game's map).
pub const DIRECTORIES: [&str; 1] = ["docs/briefs/reference"];

/// The file that marks a checkout.
const MARKER: &str = "crates/bot/src/strategist/player.md";

/// The checkout this binary was built in, found from the binary by the marker, whatever `WITHIN_REASON_TEXTS` says:
/// where a snapshot is taken from.
pub fn checkout() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    exe.ancestors().find(|a| a.join(MARKER).is_file()).map(PathBuf::from)
}

/// Writes the snapshot a game runs from: every text and reference table under its checkout path, `bot` (this
/// binary), and the git state (`git-head.txt`, `git-status.txt`, `git-diff.patch`, `snapshot.json`). Fails rather
/// than leaving a partial snapshot a game could start from.
pub fn snapshot(dir: &Path) -> std::io::Result<()> {
    use std::io::{Error, Write};
    let checkout = checkout().ok_or_else(|| Error::other("no checkout found from this binary: a snapshot is taken from a checkout"))?;
    std::fs::create_dir_all(dir)?;
    let mut files = Vec::new();
    let copy = |rel: &str, files: &mut Vec<String>| -> std::io::Result<()> {
        let to = dir.join(rel);
        std::fs::create_dir_all(to.parent().unwrap())?;
        std::fs::copy(checkout.join(rel), &to).map_err(|e| Error::other(format!("{rel}: {e}")))?;
        files.push(rel.to_string());
        Ok(())
    };
    for text in ALL {
        copy(text.path, &mut files)?;
    }
    for directory in DIRECTORIES {
        let mut entries: Vec<_> = std::fs::read_dir(checkout.join(directory))?.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            copy(&format!("{directory}/{}", entry.file_name().to_string_lossy()), &mut files)?;
        }
    }
    let exe = std::env::current_exe()?;
    std::fs::copy(&exe, dir.join("bot"))?;
    let git = |args: &[&str]| -> String {
        std::process::Command::new("git").args(args).current_dir(&checkout).output().map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default()
    };
    let head = git(&["rev-parse", "HEAD"]).trim().to_string();
    let status = git(&["status", "--short"]);
    std::fs::write(dir.join("git-head.txt"), format!("{head}\n"))?;
    std::fs::write(dir.join("git-status.txt"), &status)?;
    std::fs::write(dir.join("git-diff.patch"), git(&["diff", "HEAD"]))?;
    let dirty = status.lines().filter(|l| !l.starts_with("??")).count();
    let when = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let mut json = std::fs::File::create(dir.join("snapshot.json"))?;
    writeln!(json, "{}", serde_json::json!({"when": when, "head": head, "dirty_files": dirty, "checkout": checkout.display().to_string(), "binary": exe.display().to_string(), "files": files}))?;
    Ok(())
}

/// The checkout the texts are read from: `WITHIN_REASON_TEXTS`, else the ancestor of the running binary that holds
/// the marker (`target/release/bot` is two below it), else none and every text is its compiled copy.
fn root() -> Option<&'static PathBuf> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        if let Some(dir) = std::env::var_os("WITHIN_REASON_TEXTS") {
            return Some(PathBuf::from(dir));
        }
        let exe = std::env::current_exe().ok()?;
        exe.ancestors().find(|a| a.join(MARKER).is_file()).map(PathBuf::from)
    })
    .as_ref()
}

/// A file of the checkout that has no compiled copy (the reference tables): its contents, or None away from a
/// checkout or when it is not there.
pub fn read_file(path: &str) -> Option<String> {
    std::fs::read_to_string(root()?.join(path)).ok()
}

/// The text as it is on disk now, else as compiled. Says once per file where it reads from, and once per file when a
/// read fails.
pub fn read(text: &Text) -> String {
    static SAID: Mutex<Option<HashSet<String>>> = Mutex::new(None);
    let say_once = |key: String, line: String| {
        let mut said = SAID.lock().unwrap();
        if said.get_or_insert_with(HashSet::new).insert(key) {
            eprintln!("[texts] {line}");
        }
    };
    let Some(root) = root() else {
        say_once("root".into(), "no checkout found (set WITHIN_REASON_TEXTS): the prompts and rules are the compiled copies".into());
        return text.compiled.to_string();
    };
    match std::fs::read_to_string(root.join(text.path)) {
        Ok(contents) => {
            say_once(format!("read {}", text.path), format!("{} is read from {}", text.path, root.display()));
            contents
        }
        Err(e) => {
            say_once(format!("fail {}", text.path), format!("{}: {e}; the compiled copy is used", text.path));
            text.compiled.to_string()
        }
    }
}

/// A hash of a text, for noticing an edit between sessions.
pub fn digest(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_checkout_is_found_from_the_test_binary_and_its_files_match_the_compiled_copies() {
        assert!(super::root().is_some(), "the test binary runs under target/, below the checkout");
        for text in super::ALL {
            assert_eq!(super::read(text), text.compiled, "{} on disk differs from the copy compiled into this test binary", text.path);
        }
    }

    #[test]
    fn a_snapshot_holds_every_text_the_reference_tables_the_binary_and_the_git_state() {
        let dir = std::env::temp_dir().join(format!("within-reason-snapshot-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        super::snapshot(&dir).expect("the snapshot is written from the checkout the test binary was built in");
        for text in super::ALL {
            assert_eq!(std::fs::read_to_string(dir.join(text.path)).unwrap(), super::read(text), "{} in the snapshot is the checkout's", text.path);
        }
        for directory in super::DIRECTORIES {
            assert!(std::fs::read_dir(dir.join(directory)).unwrap().count() > 0, "{directory} is copied whole");
        }
        for file in ["bot", "git-head.txt", "git-status.txt", "git-diff.patch", "snapshot.json"] {
            assert!(dir.join(file).is_file(), "{file} is in the snapshot");
        }
        assert!(dir.join(super::MARKER).is_file(), "the marker is in the snapshot, so DIR/bot finds DIR as its root");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
