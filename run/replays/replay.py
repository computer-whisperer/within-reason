#!/usr/bin/env python3
"""Replay the manifest's fetched demos that have no records yet, several engines at once, and write each one's card
(docs/design/2026-09-23-replay-survey.md, decision 3).

    run/replays/replay.py [--manifest run/data/replays/manifest.jsonl] [--parallel 4] [--base-port 9300] [--limit N] [--map "Name"]

Each demo runs through run/replay_match.py on its own port from --base-port up; the match directory is named
<time>-replay-<match id>; the manifest line gets `match` (the directory) and `card` when run/replays/card.py wrote
one. A demo whose game version is not in the pool is skipped and said (run/replays/fetch.py --versions). Do not run
this while an arena batch runs on the same ports, and not at all while a batch runs if the machine is short of cores:
a replay engine runs at about 1.7 times game speed and takes a core.
"""
import json, os, queue, shutil, subprocess, sys, time
from concurrent.futures import ThreadPoolExecutor

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fetch import installed_versions  # noqa: E402


def run_one(row, ports):
    """One demo through the engine on a port taken from the pool for the run (the first batch gave ports by index and
    two engines met on 9300: twelve replays died in a second). A run without records is a failure: its directory goes."""
    demo = os.path.join(REPO, row["demo"])
    label = row["id"]
    started = time.time()
    port = ports.get()
    try:
        proc = subprocess.run([sys.executable, os.path.join(REPO, "run", "replay_match.py"), demo, "--label", label, "--port", str(port), "--timeout", "3600"], capture_output=True, text=True)
    finally:
        ports.put(port)
    match = None
    for line in proc.stdout.splitlines():
        line = line.strip()
        if line.startswith("/") and "replay-" + label in line:
            match = line
    if match is None:
        candidates = sorted(p for p in os.listdir(os.path.join(REPO, "run", "matches")) if p.endswith("replay-" + label))
        match = os.path.join(REPO, "run", "matches", candidates[-1]) if candidates else None
    ok = bool(match) and os.path.exists(os.path.join(match, "record-0.jsonl")) and os.path.getsize(os.path.join(match, "record-0.jsonl")) > 1000
    if match and not ok:
        shutil.rmtree(match, ignore_errors=True)
        match = None
    card = False
    if ok:
        c = subprocess.run([sys.executable, os.path.join(REPO, "run", "replays", "card.py"), match], capture_output=True, text=True)
        card = c.returncode == 0
    return row["id"], match, card, int(time.time() - started), proc.returncode


def main():
    args = sys.argv[1:]
    def opt(name, default):
        return args[args.index(name) + 1] if name in args else default
    manifest = opt("--manifest", os.path.join(REPO, "run", "data", "replays", "manifest.jsonl"))
    parallel = int(opt("--parallel", "4"))
    base_port = int(opt("--base-port", "9300"))
    limit = int(opt("--limit", "0")) or None
    only_map = opt("--map", None)
    rows = [json.loads(l) for l in open(manifest) if l.strip()]
    # A line whose match directory has no records (an earlier failed run) is replayed again.
    for r in rows:
        if r.get("match") and not os.path.exists(os.path.join(REPO, r["match"], "record-0.jsonl")):
            shutil.rmtree(os.path.join(REPO, r["match"]), ignore_errors=True)
            r.pop("match", None)
            r.pop("card", None)
    have = installed_versions()
    todo = [r for r in rows if r.get("keep") and r.get("demo") and not r.get("match") and (only_map is None or r.get("map_script") == only_map)]
    skipped = [r for r in todo if r["game_version"] not in have]
    todo = [r for r in todo if r["game_version"] in have][:limit]
    if skipped:
        print(f"{len(skipped)} demos need game versions not in the pool: {sorted({r['game_version'][-13:] for r in skipped})} (run/replays/fetch.py --versions)")
    print(f"{len(todo)} demos to replay, {parallel} at a time from port {base_port}")
    by_id = {r["id"]: r for r in rows}
    done = 0
    ports: queue.Queue = queue.Queue()
    for i in range(parallel):
        ports.put(base_port + i * 10)
    with ThreadPoolExecutor(max_workers=parallel) as pool:
        futures = {pool.submit(run_one, r, ports): r for r in todo}
        for f in futures:
            match_id, match, card, seconds, rc = f.result()
            row = by_id[match_id]
            if match:
                row["match"] = os.path.relpath(match, REPO)
                row["card"] = card
            done += 1
            print(f"  {match_id[:8]} {row['map_script']} {row['duration_ms'] // 60000} min: {'records' if match else 'FAILED'}{' + card' if card else ''} in {seconds} s (exit {rc})")
            with open(manifest, "w") as out:
                for r in rows:
                    out.write(json.dumps(r) + "\n")
    print(f"{done} replayed; {sum(1 for r in rows if r.get('card'))} of {sum(1 for r in rows if r.get('keep'))} kept replays have cards")


if __name__ == "__main__":
    main()
