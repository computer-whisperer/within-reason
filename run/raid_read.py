#!/usr/bin/env python3
"""The raid scenario's score over a batch (`duel --scenario raid`, docs/harness/duels.md): raids, how they ended, the
first raider killed (median seconds), extractors and hunters lost a raid, and how the hunts ended.
usage: run/raid_read.py run/matches/<batch> [...]"""
import collections, csv, statistics, sys

for batch in sys.argv[1:]:
    with open(f"{batch}/duels.csv") as f:
        rows = list(csv.DictReader(f))
    if not rows or "extractors_lost" not in rows[0]:
        print(f"{batch}: no raid columns")
        continue
    reasons = collections.Counter(r["reason"] for r in rows)
    kills = [float(r["first_kill_s"]) for r in rows if r["first_kill_s"]]
    ends = collections.Counter()
    for r in rows:
        for hunt in r["hunts"].split():
            quarry, rest = hunt.split(":", 1)
            ends[(quarry, rest.split("@")[0])] += 1
    lane = rows[0].get("lane_x", "?")
    print(f"== {batch.rstrip('/').split('/')[-1]}: {len(rows)} raids, lane {lane}; ended {dict(reasons)}; seconds {statistics.median(float(r['seconds']) for r in rows):.0f} median")
    print(f"   first kill: median {statistics.median(kills) if kills else float('nan'):.1f} s in {len(kills)} of {len(rows)}; extractors lost {statistics.mean(int(r['extractors_lost']) for r in rows):.2f} a raid; hunters lost {statistics.mean(int(r['hunters_lost']) for r in rows):.2f}")
    print(f"   hunt ends: {', '.join(f'{q} {w} {n}' for (q, w), n in sorted(ends.items()))}")
