#!/usr/bin/env python3
"""Make a game's tape: play its demo through the headless engine with the tape widget and pack what it wrote into
game.tape, the one seekable file the 3D viewer reads (docs/harness/tape-format.md).

    run/tape.py <match dir>              the match's own demo (demos/*.sdfz, or a .sdfz in the directory); game.tape is
                                         written into the match directory, beside its records
    run/tape.py <demo.sdfz> [--label L]  any demo: makes run/matches/<unix time>-tape-<label>/ with game.tape in it
                                         (and with --records the per-team records too, as run/replay_match.py writes)

    --no-pieces      without the units' moving pieces (turrets, legs, arms; docs/design/2026-11-15-game-tape.md sections 9
                     and 10): a 10v8's engine time 883 s against 1,339 with them, which are 44 MB of its 105
    --keep           keep the engine's write directory and tape.raw (a match's: <match dir>/tape-capture/)
    --no-check       skip reading the tape back against the raw stream
    --timeout S      seconds the engine may take (default: an hour); it is killed then and what it wrote is packed
    --port N         the local port the engine's demo server takes (default: by process id)

A duel of twenty minutes takes about a minute; a 10v8 of thirty about twenty-five minutes, nearly all of it the
engine simulating the game. The models, textures and map files the viewer draws with are copied out of the engine's
archives into run/data/tape-assets (named by content, shared by every tape).
"""
import json, os, re, shutil, subprocess, sys, time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "run"))
sys.path.insert(0, os.path.join(REPO, "run", "replays"))
import replay_match
import startscript


def demo_of(match_dir):
    found = []
    for folder in (os.path.join(match_dir, "demos"), match_dir):
        if os.path.isdir(folder):
            found += sorted(os.path.join(folder, f) for f in os.listdir(folder) if f.endswith(".sdfz") and not f.endswith(".fixed.sdfz"))
    return found[-1] if found else None


def main():
    args = sys.argv[1:]
    if not args or args[0].startswith("-"):
        sys.exit(__doc__)
    source = os.path.abspath(args[0])
    value = lambda name, default=None: args[args.index(name) + 1] if name in args else default
    packer = os.path.join(REPO, "target", "release", "tape")
    if not os.path.exists(packer):
        sys.exit(f"{packer} is not built: cargo build --release -p tape")
    flags = ["--tape"] + ([] if "--no-pieces" in args else ["--tape-pieces"])
    stamp = int(time.time())
    if os.path.isdir(source):
        demo = demo_of(source)
        if not demo:
            sys.exit(f"{source} holds no demo (demos/*.sdfz)")
        match_dir, out = source, os.path.join(source, "tape-capture")
        if os.path.exists(out):
            shutil.rmtree(out)  # the leavings of an earlier capture of this match, kept with --keep
        flags.append("--no-records")
    else:
        demo = source
        label = value("--label", re.sub(r"[^A-Za-z0-9]+", "-", os.path.basename(demo))[:40])
        match_dir = out = os.path.join(REPO, "run", "matches", f"{stamp}-tape-{label}")
        if "--records" not in args:
            flags.append("--no-records")
    started = time.time()
    print(f"playing {os.path.basename(demo)} ...", flush=True)
    code = replay_match.replay(demo, out, flags, float(value("--timeout", 3600)), True, int(value("--port", 0)) or None)
    raw = os.path.join(out, "tape.raw")
    if not os.path.exists(raw):
        sys.exit(f"the engine wrote no tape.raw (exit {code}); see {out}/engine.log and infolog.txt")
    engine_s = time.time() - started
    tape = os.path.join(match_dir, "game.tape")
    # Who sat where, from the demo's own start script: in a replay the engine names every seat "UNKNOWN".
    seats = os.path.join(out, "seats.json")
    try:
        with open(seats, "w") as f:
            json.dump(startscript.seats(startscript.parse(startscript.read_demo(demo)[1])), f)
    except Exception as e:
        print(f"the demo's start script did not read ({e}); the tape's seats keep the engine's names")
        seats = None
    pack = subprocess.run([packer, "pack", raw, "-o", tape] + (["--seats", seats] if seats else []) + ([] if "--no-check" in args else ["--check"]), capture_output=True, text=True)
    sys.stdout.write(pack.stdout)
    if pack.returncode != 0:
        sys.exit(f"tape pack failed:\n{pack.stderr}\nthe raw stream is kept: {raw}")
    if "--keep" not in args:
        if out == match_dir:
            os.remove(raw)
        else:
            shutil.rmtree(out)
    ended = f"exit {code}" if code is not None else "killed at the timeout: the tape ends where the engine was"
    print(f"{tape}\n  engine {engine_s:.0f} s ({ended}), in all {time.time() - started:.0f} s; open it with run/view_match.py")


if __name__ == "__main__":
    main()
