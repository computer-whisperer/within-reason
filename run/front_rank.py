#!/usr/bin/env python3
"""Read a scenario batch's per-second logs by rank: does our front stop on the nearest enemy, and are the units behind
it kept out of reach? (The user's registered prediction on the E3 fight, K-micro-the-front-stops-on-the-nearest-and-
blocks-the-rest; `docs/studies/2026-09-28-tas-e3.md`.)

    run/front_rank.py --scenario FILE <batch dir> [<batch dir> ...] [--reach 180] [--seconds 60]

Needs logs whose `dealt` rows name the victim (`[attacker, victim, damage]`, from 2026-09-28 evening).

Ranks: at the first line (the first orders) our units are sorted by distance to the nearest enemy unit and cut in
thirds: front, middle, back. Per rank, over each unit's living seconds up to `--seconds`:
  in reach   seconds with an enemy within reach + 20, over living seconds;
  stood      of the in-reach seconds, those in which the unit moved under 15 elmos (stopped to shoot);
  blocked    seconds out of reach with an enemy within 700 and a friend within 40 of the line to the nearest enemy
             and nearer to it, over the out-of-reach engaged seconds (the fire instrument's `blocked` without its
             condition that the friend fired);
  first hit  the type of the first enemy the unit damaged, and whether it was the enemy nearest to it at that second;
  damage by type  the share of the rank's damage that went to each enemy type.
"""
import argparse
import glob
import json
import math
from collections import Counter, defaultdict
from pathlib import Path

ENGAGED = 700.0
IN_THE_WAY = 40.0
STOOD = 15.0


def near_segment(p, a, b):
    dx, dz = b[0] - a[0], b[1] - a[1]
    l2 = dx * dx + dz * dz
    if l2 <= 0:
        return math.dist(p, a)
    t = max(0.0, min(1.0, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dz) / l2))
    return math.dist(p, (a[0] + t * dx, a[1] + t * dz))


def read(path, types, reach, seconds):
    lines = [json.loads(l) for l in open(path)]
    first = lines[0]
    enemies0 = [(u[1], u[2]) for u in first["y"]]
    order = sorted(first["x"], key=lambda u: min(math.dist((u[1], u[2]), e) for e in enemies0))
    third = math.ceil(len(order) / 3)
    rank = {u[0]: ("front" if k < third else "middle" if k < 2 * third else "back") for k, u in enumerate(order)}
    stats = defaultdict(Counter)
    first_hit = {}
    damage = defaultdict(Counter)
    last = {}
    for r in lines:
        if r["t"] > seconds:
            break
        ours = {u[0]: (u[1], u[2]) for u in r["x"]}
        theirs = {u[0]: (u[1], u[2]) for u in r["y"]}
        for i, at in ours.items():
            s = stats[rank[i]]
            s["alive"] += 1
            if not theirs:
                continue
            nearest_j, nearest = min(theirs.items(), key=lambda kv: math.dist(at, kv[1]))
            d = math.dist(at, nearest)
            if d < reach + 20:
                s["reach"] += 1
                if i in last and math.dist(at, last[i]) < STOOD:
                    s["stood"] += 1
            elif d < ENGAGED:
                s["engaged_out"] += 1
                if any(k != i and near_segment(p, at, nearest) < IN_THE_WAY and math.dist(p, at) < d for k, p in ours.items()):
                    s["blocked"] += 1
        for row in r["dealt"]["x"]:
            if len(row) < 3:
                raise SystemExit(f"{path}: `dealt` rows without the victim (a log from before 2026-09-28 evening)")
            i, j, dmg = row
            if i not in rank:
                continue
            damage[rank[i]][types[j]] += dmg
            if i not in first_hit and i in ours and theirs:
                nearest_j = min(theirs, key=lambda k: math.dist(ours[i], theirs[k]))
                first_hit[i] = (types[j], j == nearest_j)
        last = ours
    return rank, stats, first_hit, damage


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("batches", nargs="+")
    ap.add_argument("--scenario", required=True)
    ap.add_argument("--reach", type=float, default=180.0)
    ap.add_argument("--seconds", type=float, default=60.0)
    a = ap.parse_args()
    types = [u["type"].replace("arm", "") for u in json.load(open(a.scenario))["sides"][1]["units"]]
    print("| Batch | Rank | Units | In reach | Stood, of in reach | Blocked, of engaged out of reach | First hit: type (nearest?) | Damage by type |")
    print("|---|---|---|---|---|---|---|---|")
    for batch in a.batches:
        total = defaultdict(Counter)
        firsts = defaultdict(Counter)
        damage = defaultdict(Counter)
        units = Counter()
        for path in sorted(glob.glob(str(Path(batch) / "seconds" / "*.jsonl"))):
            rank, stats, first_hit, dmg = read(path, types, a.reach, a.seconds)
            for k, v in rank.items():
                units[v] += 1
            for k, s in stats.items():
                total[k].update(s)
            for i, (t, nearest) in first_hit.items():
                firsts[rank[i]][f"{t} ({'nearest' if nearest else 'not nearest'})"] += 1
            for k, c in dmg.items():
                damage[k].update(c)
        name = Path(batch).name.split("-duel-", 1)[-1]
        for k in ("front", "middle", "back"):
            s = total[k]
            share = lambda n, d: f"{n / d:.2f}" if d else "-"
            dsum = sum(damage[k].values()) or 1
            dtext = ", ".join(f"{t} {v / dsum:.0%}" for t, v in damage[k].most_common())
            ftext = ", ".join(f"{t} {v}" for t, v in firsts[k].most_common(4))
            print(f"| {name} | {k} | {units[k]} | {share(s['reach'], s['alive'])} | {share(s['stood'], s['reach'])} | "
                  f"{share(s['blocked'], s['engaged_out'])} | {ftext} | {dtext} |")


if __name__ == "__main__":
    main()
