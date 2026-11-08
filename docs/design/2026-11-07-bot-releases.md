# Releasing the heuristic bots as downloads (2026-11-07)

The user: "work towards releasing WRDistilled as a standalone downloadable bot. We will want to arrange for the
different bots in the repo to have their own version numbers and release CI processes. Users on windows and linux
should be able to download and use the bot files without a build chain. For reference, you can see how apex-ai does
its releases." This is the design, written before anything is built; §5 lists the decisions it needs.

## 1. What a release is, for the engine

The engine finds a skirmish AI at `<data dir>/AI/Skirmish/<shortName>/<version>/` holding `AIInfo.lua` and the
library, `libSkirmishAI.so` on Linux and `SkirmishAI.dll` on Windows (`AILibraryManager.cpp:185` walks every data
dir; `AIInterfaceLibrary.cpp:390` adds the platform's extension). A bot of `bots/` is such a library: a `cdylib`
over `crates/ai-host`, which exports the three C entry points (`init`, `release`, `handleEvent`) and speaks the
engine's C interface (`interfaceShortName C`, `interfaceVersion 0.1`, shipped with every BAR engine on both
platforms). So a release is two files per platform plus an installer; no game-side scripts, configs or nets as Apex
has.

Checked in the dependency tree of `bot-distilled` (`cargo tree`, grep of the shared crates):

- Nothing OS-specific outside `#[cfg(test)]` (one `UnixStream` pair in `bot-protocol`'s framing tests).
- Nothing read from disk at run time: the combat table and the duel pairs are `include_str!` (`combatsim`), the
  arms are environment variables, all optional. The library is self-contained.
- Build-time needs: a Rust toolchain of edition 2024 (1.85+; the checkout builds on 1.96), `bindgen` over the
  vendored engine headers (`crates/recoil-ai-sys/vendor`, so libclang at build time and no engine checkout), git
  for the banner's commit (`crates/ai-host/build.rs`).
- The release library is 3.1 MB on Linux unstripped.

Where Apex's C++ is compiled against one engine's headers with an ABI that broke between 2026.06.12 and 2026.07.04
(`apex-bot-build` memory, his `release.env`), the C interface is the engine's stable boundary: the vendored headers
are 2026.07.04's and the structs have not changed in years. So the install need not follow the engine folder (§3).

## 2. How Apex releases (`apexearth/bar-ai`, read 2026-11-07)

- One repo, one bot, so a `v*` tag is the trigger; `workflow_dispatch` builds the same package as an artifact
  without releasing. Pins (engine ref, C++ SHA, docker images) in `.github/release.env`, read into the job.
- One ubuntu job builds the DLL and the .so in BAR's two `recoil-build-amd64-*` images, strips them, and
  `tools/package_ai.py` lays out `payload/<Short>/<version>/{AIInfo.lua, AIOptions.lua, config/, script/,
  library}` beside `Install.bat` + `payload/install.ps1` + `INSTALL.txt` (the Windows zip) and `install.sh` (the
  Linux tarball). `gh release create` with a notes template: which file to download, the three steps, "only the
  host needs it", the engine it was built for.
- The installers find BAR (Windows: `%LOCALAPPDATA%\Programs\Beyond-All-Reason`, Program Files, Steam libraries
  from `libraryfolders.vdf`, the uninstall registry keys, else a folder picker; Linux: `~/.local/state/Beyond All
  Reason`, `~/.local/share/Beyond All Reason`, the Flatpak dirs, Steam), copy the payload into
  `<data>/engine/<engine>/AI/Skirmish/<Short>/<version>/` (the engine folder, because his ABI binds him to one
  engine; an engine update wipes it and his notes say to reinstall), unblock the files on Windows, and with BAR
  closed create `devmode.txt` and set `simpleAiList = false` in `LuaMenu/Config/IGL_data.lua`: the lobby hides
  every AI it has no friendly name for while the simple list is on (our `docs/harness/lobby.md` found the same;
  `run/install_to_bar.sh` touches `devmode.txt` and leaves the checkbox to the user).
- No smoke in CI: nothing runs the engine there.

## 3. The design

### 3.1 One version per bot, from its Cargo.toml

The crate's `version` in `bots/<bot>/Cargo.toml` is the only place a bot's version is written. Today every bot says
`0.1` in `data/AIInfo.lua`, `0.1.0` in Cargo.toml, and `0.1` again in `run/install_ai.sh` and six `run/*.sh`
scripts and the arena's default; the Apex author's logs call a clone of ours "WRDistilled 0.2", a label of his.

- `bots/<bot>/data/AIInfo.lua` carries `version = '@VERSION@'` and `url = '@URL@'` (today the url is the
  placeholder `https://github.com/` in all four AIInfo files). A new `run/bot_payload.sh <bot> <library> <out dir>`
  writes `<out dir>/AIInfo.lua` with both filled (the version from `cargo pkgid -p <crate>`, no jq) and copies the
  library beside it under the platform's name. `run/install_ai.sh` installs through it, so the arena's data dir
  holds `WRDistilled/0.2.0/` and the release holds the identical directory.
- The arena takes a bare `--our-ai WRDistilled` (and `--ally-ai`) and resolves the version to the one directory
  installed under `run/data/AI/Skirmish/WRDistilled/` (an error if there are several: a stale version left behind
  is a trap, so `install_ai.sh` removes the other versions of a bot it installs). `NAME:VERSION` still works. The
  six scripts and the README drop their `:0.1`. The flagship stays `WReason:0.1` as it is (not released by this
  design).
- The banner (`bots/distilled/src/lib.rs`, `ai_host::commit`) says the version before the commit: `WRDistilled
  0.2.0 (3fd09294)`; a dirty tree keeps its `+`.
- First numbers: WRDistilled **0.2.0** (0.1 is what every development game to date ran as, and the Apex author
  already names a clone 0.2), WRTerritory and WRTemplate **0.1.0** (unreleased, bumped only when released). Three
  parts; the engine compares version strings textually (Apex renamed v0.2.10 to v0.3.0 over it), which matters only
  when two versions of one bot sit in one data dir, and the installer removes the older ones (§3.3).
- Bumping: the commit that is tagged bumps the crate version; CI refuses a tag whose version is not the crate's.

### 3.2 Tags and the workflow

- Tag `<bot>-v<version>`: `distilled-v0.2.0`, `territory-v0.1.0`. `.github/workflows/release.yml` triggers on
  `push: tags: ['*-v*']` and on `workflow_dispatch` with a `bot` input (a dry run: the packages as artifacts, no
  release). The tag's bot part names the directory under `bots/`, the crate name is `bot-<bot>`, the short name is
  read from the bot's `AIInfo.lua`.
- Jobs: `build` on a matrix of `ubuntu-22.04` (glibc 2.35, so the .so loads on every distribution and Flatpak
  runtime current users have) and `windows-latest`: `cargo test -p bot-<bot>` and the shared crates' tests on
  Linux, `cargo build --release -p bot-<bot>` on both; Windows with `RUSTFLAGS=-C target-feature=+crt-static`, so
  the DLL carries its C runtime and needs no Visual C++ redistributable (the engine is a MinGW build and promises
  none). libclang: `libclang-dev` on Ubuntu, the preinstalled LLVM on the Windows runner (`LIBCLANG_PATH`). Then
  `package` on Ubuntu takes both libraries and writes, through `run/bot_payload.sh`:
  - `<Short>-<version>-INSTALL.zip`: `Install.bat`, `INSTALL.txt`, `LICENSE`, `README.txt`, `payload/install.ps1`,
    `payload/<Short>/<version>/{AIInfo.lua, SkirmishAI.dll}`.
  - `<Short>-<version>-LINUX.tar.gz`: `install.sh`, `INSTALL.txt`, `LICENSE`, `README.txt`,
    `payload/<Short>/<version>/{AIInfo.lua, libSkirmishAI.so}`.
  - `gh release create <tag>` with both, titled `<Short> <version>`, the notes from the top section of
    `bots/<bot>/CHANGELOG.md` (a player-facing changelog, new; the README stays the development record) above the
    fixed download-and-install text. GitHub's "Source code" archives sit beside them, as Apex's notes warn.
- The license: linking the host makes every bot GPL-2.0-or-later (`bots/README.md`); `LICENSE` is in every package
  and the release notes link the source at the tag.

### 3.3 The installers (`release/installer/`, ours, written new)

Same shape as Apex's, differing in where they put the files:

- **Target `<data>/AI/Skirmish/<Short>/<version>/`**, the data root, not the engine folder: it survives engine
  updates (the C interface is stable, §1) and is where `run/install_to_bar.sh` has put the flagship since
  September and the lobby found it (`lobby.md`). On Windows the data root is `<install>\data\`, the folder holding
  `engine\`; the first Windows test confirms the engine lists an AI from there (expected: the write dir is a data
  dir on every platform).
- Finding BAR as Apex does (the same candidate list, the Linux one with `~/.local/share/BeyondAllReason`, the
  current launcher's directory, which Apex's list lacks); an explicit path as the argument; a folder picker on
  Windows when nothing is found.
- Older installed versions of the same bot are removed (the lobby would list each, and the engine's textual order
  would put 0.10.0 under 0.9.0), the new one copied in whole, `Unblock-File` on Windows, `chmod 755` on Linux; a
  running BAR is detected and the user asked to close it first, since a mapped library must never be overwritten
  (`install_ai.sh`'s rule).
- The lobby's full AI list switched on exactly as Apex does it (`devmode.txt` + `simpleAiList = false`, BAR closed,
  the setting read back). This edits a user's lobby settings file; it is what makes the bot visible at all, and
  the only alternative is a friendly name in the lobby's own list, which is a request to the BAR lobby maintainers
  once the bot has users.
- `INSTALL.txt` and `README.txt`: what the bot is (one paragraph from the AIInfo description), the by-hand install
  path, "only the host needs it", where to report (the repo's issues), the engine it was tested on.

### 3.4 Smoke before a tag

CI runs no engine (BAR's engine and game archive are large; Apex has none either; a later step may add one). The
rule instead: `run/release_smoke.sh <bot>` packages the Linux tarball exactly as CI does, installs it with its own
`install.sh` into a scratch data dir, and plays one four-minute game through the arena against BARb easy on Comet
with `WITHIN_REASON_KEEP_INSTALLED=1` and that data dir; the banner line in the engine log with the right version
is the pass. The tag is pushed only after it passes. The Windows package is tested by a Windows player of the
first release (the user's own machine if there is one; otherwise a tester in the community), and the release notes
say so until it has been.

### 3.5 What changes in the repository

| Where | Change |
|---|---|
| `bots/*/data/AIInfo.lua` | `@VERSION@`, `@URL@` |
| `run/bot_payload.sh` | new: fills AIInfo.lua, lays out the payload directory |
| `run/install_ai.sh` | installs through `bot_payload.sh`; removes other versions of a bot |
| `crates/arena/src/{main,script}.rs` | bare `NAME` resolves to the installed version |
| `run/*.sh`, `README.md`, `bots/README.md` | `:0.1` dropped; "Linux only" line replaced by a pointer here |
| `bots/distilled/src/lib.rs` | banner says the version |
| `bots/<bot>/Cargo.toml` | distilled 0.2.0 |
| `bots/distilled/CHANGELOG.md` | new, player-facing |
| `release/installer/{Install.bat, install.ps1, install.sh, INSTALL.txt.in, README.txt.in}` | new |
| `run/release_smoke.sh` | new |
| `.github/workflows/release.yml` | new |
| `docs/harness/release.md` | how to cut a release; the Windows pitfalls as they are found |

Not in scope: releasing the flagship (it needs the bot process and a Claude account); a Windows build of the arena
or the shim; CI that runs games.

## 4. Order of work

1. Versions: AIInfo templates, `bot_payload.sh`, `install_ai.sh`, the arena's bare name, the scripts, the banner;
   one arena game to show the installed `WRDistilled/0.2.0` plays as before.
2. Installers and packaging on Linux, `release_smoke.sh` passing locally.
3. The workflow, run by `workflow_dispatch` first (artifacts only), the Windows DLL checked for exports with
   `dumpbin`/`llvm-objdump` in the job (the three names, undecorated).
4. `distilled-v0.2.0` tagged; the first Windows test; `docs/harness/release.md` from what it taught.

Steps 1 and 2 are a day's work together; step 3 is one CI iteration loop (the Windows runner, bindgen's libclang,
crt-static) and is where the surprises will be.

## 5. Decisions asked of the user

Ruled 2026-11-07 evening: all six as proposed. On 3 the user asked for "whatever is standard recommended for BAR
custom AIs in general, plus whatever our documentation says", with an easy install guide or script. Checked: BAR
publishes no guide for custom AIs (its site and FAQ have none); the Spring engine's wiki installs a skirmish AI
under the data directory's `AI/Skirmish/<Short>/<version>/`, which is where `docs/harness/lobby.md` and
`run/install_to_bar.sh` have put the flagship; Apex's engine folder is forced by his C++ ABI. So the data root,
with the installer scripts and the by-hand steps of §3.3.

1. Version source: the crate's Cargo.toml, templated into AIInfo.lua (§3.1); the arena accepts bare names.
2. Tag scheme `<bot>-v<version>`, one workflow for all the bots of `bots/`, the flagship excluded (§3.2).
3. Install to the data root, not the engine folder (§3.3), on the C interface's stability.
4. The installer switches the lobby's full AI list on, as Apex's does (§3.3).
5. WRDistilled's first release is 0.2.0; the others stay 0.1.0 until released (§3.1).
6. Build platforms: ubuntu-22.04 and windows-latest native, no cross-compile (nothing here can test a MinGW
   cross-build: no `x86_64-w64-mingw32-gcc` on this machine).
