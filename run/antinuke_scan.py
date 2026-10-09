#!/usr/bin/env python3
"""When the opponent builds an anti-nuke or a nuke silo, from the truth files (K-barb-anti-nuke-by-minute).

    run/antinuke_scan.py [ROOT...]        (default: run/matches and the archive named in docs/experiments.md's head)

Walks every match directory under the roots for a `truth-0.jsonl` over 1 MB, takes the first frame at which each of
armamd, corfmd, armsilo, corsilo appears in the opponent's unit list (`grep -m1`, so a few seconds a file) and the
last frame, and prints one line a match plus a summary over the games that passed 20 minutes. Labels `player-N` are
the Comet hard_aggressive player games; the September batches are mostly BARb medium on Quicksilver.
"""
import os, re, statistics as st, subprocess, sys

ROOTS = sys.argv[1:] or ["run/matches", "/ceph/christian/within-reason/run/matches-archive"]
NAMES = ["armamd", "corfmd", "armsilo", "corsilo"]
FPS = 30


def frame_of(line):
    m = re.search(r'"f": *(\d+)', line)
    return int(m.group(1)) if m else None


def first_frame(path, name):
    r = subprocess.run(["grep", "-m1", "-F", f'"{name}"', path], capture_output=True, text=True)
    return frame_of(r.stdout) if r.returncode == 0 else None


def last_frame(path):
    r = subprocess.run(["tail", "-n", "1", path], capture_output=True, text=True)
    return frame_of(r.stdout) or 0


rows = []
for root in ROOTS:
    for dirpath, _, files in os.walk(root):
        if "truth-0.jsonl" not in files:
            continue
        path = os.path.join(dirpath, "truth-0.jsonl")
        if os.path.getsize(path) < 1 << 20:
            continue
        label = os.path.relpath(dirpath, root)
        end = last_frame(path) / FPS / 60
        hits = {n: first_frame(path, n) for n in NAMES}
        hits = {n: f / FPS / 60 for n, f in hits.items() if f is not None}
        rows.append((label, end, hits))
        print(f"{label} end={end:.0f}" + "".join(f" {n}@{m:.0f}" for n, m in hits.items()))

long = [r for r in rows if r[1] >= 20]
anti = [min(m for n, m in h.items() if n in ("armamd", "corfmd")) for _, _, h in long if any(n in h for n in ("armamd", "corfmd"))]
silo = [(l, m) for l, _, h in long for n, m in h.items() if n in ("armsilo", "corsilo")]
print(f"\n{len(rows)} games with a truth file over 1 MB; {len(long)} passed 20 minutes; an anti-nuke of his in {len(anti)} of them"
      + (f" (minutes: min {min(anti):.0f}, median {st.median(anti):.0f}, max {max(anti):.0f})" if anti else "") + f"; silos in {len(silo)}")
player = [(l, h) for l, e, h in long if re.match(r"\d+-player-\d+", l)]
pa = [min(m for n, m in h.items() if n in ("armamd", "corfmd")) for _, h in player if any(n in h for n in ("armamd", "corfmd"))]
if player:
    print(f"the player-N games (Comet hard_aggressive): {len(pa)} of {len(player)}" + (f", median minute {st.median(pa):.0f}" if pa else ""))
