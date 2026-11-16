#!/usr/bin/env python3
"""Play an engine replay (a .sdfz demo, a human's game for instance) through the headless engine and write our match
records for every team of it, so run/army_use.py, run/batch_trade.py, run/contact_use.py and the viewer read it as
they read an arena match.

    run/replay_match.py <demo.sdfz> [--label NAME] [--timeout SECONDS] [--kill-on-timeout] [--port N] [--audit]
                        [--tape] [--tape-aim] [--no-records]

Makes run/matches/<unix time>-replay-<label>/ from run/replay-template/ (the LuaUI widget replay_dump.lua does the
writing: record-<team>.jsonl and truth-<team>.jsonl per team), copies the demo in, runs `spring-headless` on it with
the arena's data directory (the engine build the demo's header names when run/engines has it, else run/engine), and
prints where the records are. With `--audit` the widget audit_dump.lua also writes audit.jsonl, every unit's order,
build power and damage once a second for run/audit.py (docs/harness/audit.md; about 2 MB a game minute). On `--timeout` the engine is left running (a slow replay by hand is not lost) unless
`--kill-on-timeout` is given, which a batch passes so the port is free for the next demo. With `--tape` the widget
tape_dump.lua writes tape.raw, the raw stream of everything a viewer draws (docs/harness/tape-format.md; run/tape.py is
the tool that makes a game's tape, and calls this); `--tape-aim` adds every weapon's direction to it; `--no-records`
leaves the per-team records out (a replay played for its tape alone). The game version the demo names must be in run/data
(pr-downloader --download-game "<full name>", docs/harness/engine.md); the engine says so in infolog.txt if not.
"""
import os, re, shutil, subprocess, sys, time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


FLAGS = (("--audit", "audit.on"), ("--tape", "tape.on"), ("--tape-aim", "tape.aim"), ("--no-records", "records.off"))


def replay(demo, out, flags=(), timeout=3600, kill_on_timeout=False, port=None):
    """Plays `demo` in the headless engine with the write directory `out` (made here from run/replay-template) and the
    widgets the `flags` (of FLAGS) switch on. Returns the engine's exit code, or None when it was still running at
    the timeout (killed if `kill_on_timeout`, else left). What the widgets wrote is moved up into `out`; the tape
    widget's asset copies go to the cache run/data/tape-assets."""
    shutil.copytree(os.path.join(REPO, "run", "replay-template"), out)
    shutil.copy(demo, os.path.join(out, os.path.basename(demo)))
    # The engine starts a local server for the demo on `HostPortDefault` (8452); two replays at once need two ports.
    port = port or 8452 + (os.getpid() % 1000)
    with open(os.path.join(out, "springsettings.cfg"), "a") as settings:
        settings.write(f"HostPortDefault = {port}\n")
    # The dump widgets may write only under LuaUI/Config/ (game test-31677 on, barwidgets.lua `userWritableFolders`).
    written = os.path.join(out, "LuaUI", "Config", "within-reason")
    os.makedirs(written, exist_ok=True)
    # The marker files the widgets look for.
    for flag, marker in FLAGS:
        if flag in flags:
            open(os.path.join(written, marker), "w").close()
    # The tape widget copies the models, textures and map files a viewer draws with into `assets/`, named by content,
    # and skips the names `assets.have` lists: the ones the cache already holds. Moved into the cache below.
    cache = os.path.join(REPO, "run", "data", "tape-assets")
    if "--tape" in flags:
        os.makedirs(cache, exist_ok=True)
        os.makedirs(os.path.join(written, "assets"), exist_ok=True)
        with open(os.path.join(written, "assets.have"), "w") as have:
            have.write("".join(name + "\n" for name in sorted(os.listdir(cache))))
    log = open(os.path.join(out, "engine.log"), "w")
    # The engine build the demo names when it is under run/engines (a demo plays only on its own engine's version);
    # else the arena's run/engine.
    binary = os.path.join(REPO, "run", "engine", "spring-headless")
    try:
        sys.path.insert(0, os.path.join(REPO, "run", "replays"))
        import startscript
        wanted = startscript.read_demo(demo)[0].get("engine", "")
        candidate = os.path.join(REPO, "run", "engines", wanted, "spring-headless")
        if wanted and os.path.exists(candidate):
            binary = candidate
    except Exception as e:  # a demo whose header does not read: the default engine, and the engine will say
        print(f"demo header not read ({e}); using {binary}")
    engine = subprocess.Popen(
        [binary, "--isolation", "--write-dir", out, os.path.join(out, os.path.basename(demo))],
        cwd=out, stdout=log, stderr=subprocess.STDOUT, env={**os.environ, "SPRING_DATADIR": os.path.join(REPO, "run", "data")},
    )
    code = None
    try:
        code = engine.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        if not kill_on_timeout:
            return None
        engine.terminate()
        try:
            engine.wait(timeout=15)
        except subprocess.TimeoutExpired:
            engine.kill()
            engine.wait()
    # What was written is moved up whether the engine ended or was killed: a replay killed after its last frame has
    # its files whole (docs/harness/pitfalls.md, the lobby demos of 2026-11-07, moved by hand then).
    copies = os.path.join(written, "assets")
    for name in sorted(os.listdir(copies)) if os.path.isdir(copies) else []:
        if os.path.exists(os.path.join(cache, name)):
            os.remove(os.path.join(copies, name))
        else:
            os.replace(os.path.join(copies, name), os.path.join(cache, name))
    for name in sorted(os.listdir(written)):
        if (name.startswith(("record-", "truth-", "audit")) and name.endswith(".jsonl")) or name == "tape.raw":
            os.replace(os.path.join(written, name), os.path.join(out, name))
    return code


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    demo = os.path.abspath(args[0])
    label = args[args.index("--label") + 1] if "--label" in args else re.sub(r"[^A-Za-z0-9]+", "-", os.path.basename(demo))[:40]
    timeout = float(args[args.index("--timeout") + 1]) if "--timeout" in args else 3600
    kill_on_timeout = "--kill-on-timeout" in args
    out = os.path.join(REPO, "run", "matches", f"{int(time.time())}-replay-{label}")
    started = time.time()
    code = replay(demo, out, [flag for flag, _ in FLAGS if flag in args], timeout, kill_on_timeout, int(args[args.index("--port") + 1]) if "--port" in args else None)
    if code is None and not kill_on_timeout:
        print(f"engine still running after {timeout:.0f} s; leaving it (stop it with run/stop_match.py {out})")
        return
    records = sorted(f for f in os.listdir(out) if f.startswith("record-"))
    ended = f"engine exit {code}" if code is not None else f"engine killed at the timeout ({timeout:.0f} s, --kill-on-timeout)"
    print(f"{out}\n  {ended} after {time.time() - started:.0f} s; records: {', '.join(records) or 'none (see engine.log and infolog.txt)'}")
    if os.path.exists(os.path.join(out, "tape.raw")):
        print("  tape.raw written: `target/release/tape pack` makes game.tape of it (run/tape.py does both)")


if __name__ == "__main__":
    main()
