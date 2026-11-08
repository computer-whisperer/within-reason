# Releasing a bot of `bots/` as a download

The design and its rulings: `docs/design/2026-11-07-bot-releases.md`. This is the procedure and what the releases
taught. The flagship is not released this way (it needs the bot process and a Claude account).

## What a release is

Two archives per bot version on the repository's GitHub Releases page, built by `.github/workflows/release.yml`:

- `<Short>-<version>-INSTALL.zip` (Windows): `Install.bat` runs `payload/install.ps1`, which finds BAR, copies
  `payload/<Short>/<version>/{AIInfo.lua, SkirmishAI.dll}` into `<BAR install>\data\AI\Skirmish\<Short>\<version>\`,
  removes other versions of the bot, and with BAR closed makes `devmode.txt` and sets `simpleAiList = false` in the
  lobby's `IGL_data.lua` (the simple list hides every AI without a friendly name; `lobby.md`).
- `<Short>-<version>-LINUX.tar.gz`: `install.sh`, the same into `~/.local/share/BeyondAllReason/AI/Skirmish/...`.

Both hold `INSTALL.txt` (the steps by hand), `README.txt` and `LICENSE`. The data root, not the engine folder: the
engine's C AI interface is stable across engine versions, so an engine update keeps the bot. Apex installs into the
engine folder because his C++ binds him to one engine; we do not.

## Where the version lives

`bots/<bot>/Cargo.toml` is the only place. `run/bot_payload.sh` fills it into `AIInfo.lua` (`@VERSION@`, `@URL@`) when
`run/install_ai.sh` installs the bot into the arena's data dir and when `run/release_package.py` packages it, so a
development game and a release hold the identical directory. The arena resolves a bare `--our-ai WRDistilled` to the
one version installed. The banner in the game chat says `WRDistilled 0.2.0 (<commit>)`.

## Cutting a release

1. In one commit: bump the version in `bots/<bot>/Cargo.toml`; retitle the top section of `bots/<bot>/CHANGELOG.md`
   `## <version> (<date>)` and write what changed for a player (the notes are that section verbatim,
   `run/release_notes.sh <bot> <version>` shows them).
2. `run/release_smoke.sh <bot>`: packages the Linux tarball as CI does, installs it with its own `install.sh` into
   `run/data` (the bot's directory removed first), plays four minutes against BARb easy; the pass is the engine log
   naming the bot at that version. A ledger line in `docs/experiments.md` as for any batch.
3. `git tag <bot>-v<version>` (`distilled-v0.2.0`) on that commit and push the tag with the branch. The workflow
   refuses a tag whose version is not the crate's or whose changelog top section is another version's.
4. `gh run watch` the workflow: `name` (the tag read), `build` on `ubuntu-22.04` and `windows-latest` (tests on Linux,
   the library on both, the three C entry points checked with `nm` and `dumpbin`), `package`, and on a tag the release
   with the notes. A dry run without a tag: `gh workflow run release.yml -f bot=<bot>` builds the same packages as a
   workflow artifact named `<Short>-<version>`.
5. The Windows package is played by a Windows player of the release; until one has, the notes say so
   (`run/release_notes.sh`'s last line). What the first Windows test teaches goes below.

## Pitfalls as they are found

- 2026-11-07, the first dry run (`gh workflow run release.yml -f bot=distilled`, run 42436685696) passed as written:
  3 minutes end to end (Linux build with tests 1:28, Windows 2:48, package 8 s). The Windows DLL is a PE32+ x64 with
  exactly `handleEvent`, `init`, `release` exported (undecorated); 2.7 MB with the static C runtime, the stripped
  Linux library 2.7 MB. libclang was found on both runners without more than `libclang-dev` and `LIBCLANG_PATH`.
- 2026-11-07, the first release: tag `distilled-v0.2.0` on db2cc739 (the commit dates the changelog section), run
  42437043866 passed in 3 minutes and published https://github.com/computer-whisperer/within-reason/releases/tag/distilled-v0.2.0
  with both archives and the notes from `run/release_notes.sh`. The published Linux tarball was downloaded, installed
  by its own `install.sh` into `run/data` and played (release-published-distilled): the banner said 0.2.0 from the
  release's files.
- The runners warn that `actions/checkout@v4`, `upload-artifact@v4` and `download-artifact@v4` target Node 20 and are
  run on Node 24; they work. Bumping the majors clears the warning and costs one dry run to verify.
- Not yet tested: the Windows package played in BAR (the installer's BAR search, `Unblock-File`, the engine listing an
  AI from `<install>\data\AI\Skirmish`). The first Windows player's report goes here.
