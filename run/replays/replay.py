#!/usr/bin/env python3
"""Replay the manifest's fetched demos that have no records yet, several engines at once, and write each one's card
(docs/design/2026-09-23-replay-survey.md, decision 3).

    run/replays/replay.py [--manifest run/data/replays/manifest.jsonl] [--parallel 4] [--base-port 9300] [--limit N] [--map "Name"]
                          [--ais any|none|NAME] [--until-minutes 8]

Each demo runs through run/replay_match.py on its own port from --base-port up; the match directory is named
<time>-replay-<match id>; the manifest line gets `match` (the directory) and `card` when run/replays/card.py wrote
one. A demo whose game version is not in the pool is skipped and said (run/replays/fetch.py --versions). Do not run
this while an arena batch runs on the same ports, and not at all while a batch runs if the machine is short of cores:
a replay engine runs at about 1.7 times game speed and takes a core (an 8v8 alone at about game speed, eight 8v8s at
once at 0.6). Each demo's engine gets four times the game's length plus ten minutes, at least an hour, and is killed
at that (the first batch gave every demo an hour: three long games timed out with their engines left on their ports,
and the three ports then went back into the pool and failed the remaining 23 demos in two seconds each). A port a
stray engine still holds after a failure is retired from the pool and said. `--ais` keeps the manifest lines by their
AIs: `none` the games without one, `any` the games with one, a name (SMRTBARb, BARb) the games with that AI, so a
batch can take the competition first. `--until-minutes` is passed to the card.
"""
import json, os, queue, shutil, socket, subprocess, sys, time
from concurrent.futures import ThreadPoolExecutor

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fetch import installed_versions  # noqa: E402


def port_free(port):
    """Whether nothing holds the engine's UDP port."""
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
            s.bind(("", port))
        return True
    except OSError:
        return False


def run_one(row, ports, until=None):
    """One demo through the engine on a port taken from the pool for the run (the first batch gave ports by index and
    two engines met on 9300: twelve replays died in a second). A run without records is a failure: its directory goes,
    and the port goes back to the pool only when nothing holds it any more."""
    demo = os.path.join(REPO, row["demo"])
    label = row["id"]
    started = time.time()
    port = ports.get()
    timeout = max(3600, 4 * row.get("duration_ms", 0) // 1000 + 600)
    try:
        proc = subprocess.run([sys.executable, os.path.join(REPO, "run", "replay_match.py"), demo, "--label", label, "--port", str(port), "--timeout", str(timeout), "--kill-on-timeout"], capture_output=True, text=True)
    finally:
        if port_free(port):
            ports.put(port)
        else:
            print(f"  port {port} is still held after {label[:8]}; retired from the pool", flush=True)
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
    if not ok and time.time() - started < 60:
        tail = [l for l in (proc.stdout + proc.stderr).splitlines() if l.strip()][-2:]
        print(f"  {label[:8]} failed in {int(time.time() - started)} s: {' | '.join(tail)[:300]}", flush=True)
    card = False
    if ok:
        c = subprocess.run([sys.executable, os.path.join(REPO, "run", "replays", "card.py"), match] + (["--until-minutes", until] if until else []), capture_output=True, text=True)
        card = c.returncode == 0
    return row["id"], match, card, int(time.time() - started), proc.returncode


def main():
    sys.stdout.reconfigure(line_buffering=True)
    args = sys.argv[1:]
    def opt(name, default):
        return args[args.index(name) + 1] if name in args else default
    manifest = opt("--manifest", os.path.join(REPO, "run", "data", "replays", "manifest.jsonl"))
    parallel = int(opt("--parallel", "4"))
    ais = opt("--ais", None)
    until = opt("--until-minutes", None)
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
    def ai_ok(r):
        kinds = {p.get("ai") for p in r.get("players", []) if p.get("ai")}
        return ais is None or (ais == "any" and kinds) or (ais == "none" and not kinds) or ais in kinds
    todo = [r for r in rows if r.get("keep") and r.get("demo") and not r.get("match") and (only_map is None or r.get("map_script") == only_map) and ai_ok(r)]
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
        futures = {pool.submit(run_one, r, ports, until): r for r in todo}
        for f in futures:
            match_id, match, card, seconds, rc = f.result()
            row = by_id[match_id]
            if match:
                row["match"] = os.path.relpath(match, REPO)
                row["card"] = card
            done += 1
            print(f"  {match_id[:8]} {row['map_script']} {row['duration_ms'] // 60000} min: {'records' if match else 'FAILED'}{' + card' if card else ''} in {seconds} s (exit {rc})")
            # Merged into the file as it is now, not the copy read at the start: a picker or fetcher run meanwhile
            # appends or amends lines, and a rewrite from the copy lost four of them (2026-09-24).
            current = [json.loads(l) for l in open(manifest) if l.strip()]
            for r in current:
                if r["id"] == match_id:
                    r["match"] = row.get("match")
                    r["card"] = row.get("card")
                    if r["match"] is None:
                        r.pop("match", None)
                        r.pop("card", None)
            with open(manifest, "w") as out:
                for r in current:
                    out.write(json.dumps(r) + "\n")
    print(f"{done} replayed, {sum(1 for r in todo if not by_id[r['id']].get('match'))} failed; {sum(1 for r in rows if r.get('card'))} of {sum(1 for r in rows if r.get('keep'))} kept replays have cards")


if __name__ == "__main__":
    main()
