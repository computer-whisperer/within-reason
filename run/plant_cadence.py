#!/usr/bin/env python3
"""The plants' cadence in a territory batch: per game and per factory, the products begun each minute and the median
interval between them, from the bot's `begun:` lines (a factory's product is begun "by unit <factory>"), with the
turrets at the plants from the `turrets:` lines. His advanced vehicle plant at 9.7 times its power made an advanced
constructor every 6-8 s from 9:25 (docs/design/2026-10-25-territory-plant-ramp.md); the roll-off constant of the
proposal's third change is to be read from these.

    run/plant_cadence.py <batch dir> [--to 900] [--game NN]
"""
import collections, glob, re, statistics, sys

args = sys.argv[1:]
batch = args[0]
to = float(args[args.index("--to") + 1]) if "--to" in args else 900.0
only = args[args.index("--game") + 1] if "--game" in args else None
BEGUN = re.compile(r"f=(\d+) begun: (\w+) by unit (\d+) at (\d+), ([\d.]+) s after its order")
TURRETS = re.compile(r"f=(\d+) turrets: (\d+) standing, (\d+) busy, (\d+) at the plants, the program's assist (-?[\d.]+) turrets.*?(?:rings: (.*))?$")
MOBILE = ("cv", "ck", "ca", "acv", "ack", "aca", "beaver", "muskrat", "fark", "consul")
for game in sorted(glob.glob(f"{batch}/0*")):
    if only and not game.endswith(only):
        continue
    products = collections.defaultdict(list)  # factory unit -> [(t, product)]
    turrets = []
    for line in open(f"{game}/engine.log", errors="replace"):
        m = BEGUN.search(line)
        if m:
            t = int(m.group(1)) / 30
            if t > to:
                continue
            what = m.group(2)
            if any(what.endswith(k) for k in MOBILE):
                products[int(m.group(3))].append((t, what))
            continue
        m = TURRETS.search(line)
        if m and int(m.group(1)) / 30 <= to:
            turrets.append((int(m.group(1)) / 30, int(m.group(4)), float(m.group(5)), m.group(6) or ""))
    print(f"== {game}")
    for factory, items in sorted(products.items(), key=lambda kv: kv[1][0][0]):
        kinds = collections.Counter(w for _, w in items)
        by_minute = collections.Counter(int(t // 60) for t, _ in items)
        gaps = [b - a for (a, _), (b, _) in zip(items, items[1:])]
        first = items[0][0]
        print(f"  factory unit {factory} ({', '.join(f'{k} x{n}' for k, n in kinds.most_common())}): first product {int(first // 60)}:{int(first % 60):02d}, "
              f"{len(items)} by {int(to // 60)}:00, median interval {statistics.median(gaps):.0f} s" if gaps else f"  factory unit {factory}: one product at {int(first // 60)}:{int(first % 60):02d}")
        print("    per minute: " + " ".join(f"m{m}:{by_minute[m]}" for m in sorted(by_minute)))
        if gaps:
            per_min_gap = collections.defaultdict(list)
            for (a, _), (b, _) in zip(items, items[1:]):
                per_min_gap[int(b // 60)].append(b - a)
            print("    median interval by minute: " + " ".join(f"m{m}:{statistics.median(g):.0f}s" for m, g in sorted(per_min_gap.items())))
    for t, at_plants, wanted, rings in turrets:
        if int(t) % 120 == 0 or t == turrets[-1][0]:
            print(f"  {int(t // 60):2d}:00 turrets at the plants {at_plants}, the program's assist {wanted:.1f}; rings: {rings}")
