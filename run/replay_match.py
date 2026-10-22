#!/usr/bin/env python3
"""Play an engine replay (a .sdfz demo, a human's game for instance) through the headless engine and write our match
records for every team of it, so run/army_use.py, run/batch_trade.py, run/contact_use.py and the viewer read it as
they read an arena match.

    run/replay_match.py <demo.sdfz> [--label NAME] [--timeout SECONDS] [--port N]

Makes run/matches/<unix time>-replay-<label>/ from run/replay-template/ (the LuaUI widget replay_dump.lua does the
writing: record-<team>.jsonl and truth-<team>.jsonl per team), copies the demo in, runs `spring-headless` on it with
the arena's data directory (the engine build the demo's header names when run/engines has it, else run/engine), and
prints where the records are. The game version the demo names must be in run/data
(pr-downloader --download-game "<full name>", docs/harness/engine.md); the engine says so in infolog.txt if not.
"""
import os, re, shutil, subprocess, sys, time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    demo = os.path.abspath(args[0])
    label = args[args.index("--label") + 1] if "--label" in args else re.sub(r"[^A-Za-z0-9]+", "-", os.path.basename(demo))[:40]
    timeout = float(args[args.index("--timeout") + 1]) if "--timeout" in args else 3600
    out = os.path.join(REPO, "run", "matches", f"{int(time.time())}-replay-{label}")
    shutil.copytree(os.path.join(REPO, "run", "replay-template"), out)
    shutil.copy(demo, os.path.join(out, os.path.basename(demo)))
    # The engine starts a local server for the demo on `HostPortDefault` (8452); two replays at once need two ports.
    port = int(args[args.index("--port") + 1]) if "--port" in args else 8452 + (os.getpid() % 1000)
    with open(os.path.join(out, "springsettings.cfg"), "a") as settings:
        settings.write(f"HostPortDefault = {port}\n")
    # The dump widget may write only under LuaUI/Config/ (game test-31677 on, barwidgets.lua `userWritableFolders`).
    os.makedirs(os.path.join(out, "LuaUI", "Config", "within-reason"), exist_ok=True)
    log = open(os.path.join(out, "engine.log"), "w")
    started = time.time()
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
    try:
        engine.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        print(f"engine still running after {timeout:.0f} s; leaving it (stop it with run/stop_match.py {out})")
        return
    written = os.path.join(out, "LuaUI", "Config", "within-reason")
    for name in sorted(os.listdir(written)) if os.path.isdir(written) else []:
        if name.startswith(("record-", "truth-")) and name.endswith(".jsonl"):
            os.replace(os.path.join(written, name), os.path.join(out, name))
    records = sorted(f for f in os.listdir(out) if f.startswith("record-"))
    print(f"{out}\n  engine exit {engine.returncode} after {time.time() - started:.0f} s; records: {', '.join(records) or 'none (see engine.log and infolog.txt)'}")


main()
