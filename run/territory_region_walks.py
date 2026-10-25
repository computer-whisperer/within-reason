#!/usr/bin/env python3
"""The extractor walk per region, priced against realized (T-REGIONS; docs/design/2026-10-25-territory-regions.md,
"What could go wrong": the trip shared over a region's spots is a guess, and the realized walk is the check).

    run/territory_region_walks.py BATCH-OR-LABEL... [--types cv,acv] [--until 10]

Per game, the regions from its dump (`buildorder regions --hello DUMP --csv`, cached as regions.csv beside it); each
basic extractor (or upgrade) begun before the minute by a builder of the given types (the names without the faction
prefix), put in the region of its site, with its order's age at its frame (the `begun:` line's seconds after its
order: the walk and the overhead). Against it the region's price for that type at the start: the `regions:` line's
seconds an extractor plus the 3-s overhead the regions' program charges (branch territory-regions-step2). Printed per label: by region, its
price, the realized median and the count; then over every extractor, the realized total over the priced total.
"""
import glob, os, re, statistics, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OVERHEAD = 3.0
ORDER = re.compile(r"f=\d+ order: unit (\d+) (\w+): build (\w+) at")
BEGUN = re.compile(r"f=(\d+) begun: (\w+) by unit (\d+) at \d+, ([\d.]+) s after its order, site \((\d+),(\d+)\)")
REGION = re.compile(r"r(\d+) (\d+) spots[^|]*?s an extractor ([^|]*)")


def regions_of(game):
    dumps = sorted(glob.glob(f"{game}/territory-*.json"))
    if not dumps:
        return None
    csv = f"{game}/regions.csv"
    if not os.path.exists(csv):
        subprocess.run([f"{REPO}/target/release/buildorder", "regions", "--hello", dumps[0], "--csv", csv], stdout=subprocess.DEVNULL, check=False)
    if not os.path.exists(csv):
        return None
    rows = [line.strip().split(",") for line in open(csv)][1:]
    return [(float(r[1]), float(r[2]), int(r[3])) for r in rows]


def main():
    args = sys.argv[1:]
    types = (args[args.index("--types") + 1] if "--types" in args else "cv,acv").split(",")
    until = float(args[args.index("--until") + 1]) if "--until" in args else 10.0
    flags = {"--types", "--until"}
    names = [a for i, a in enumerate(args) if not a.startswith("--") and (i == 0 or args[i - 1] not in flags)]
    for name in names:
        batch = name if "/" in name else sorted(glob.glob(f"run/matches/*-{name}"))[-1]
        by_region = {}
        priced_total, real_total = 0.0, 0.0
        for game in sorted(glob.glob(f"{batch}/[0-9][0-9]")):
            spots = regions_of(game)
            if not spots or not os.path.exists(f"{game}/engine.log"):
                continue
            prices, kind_of = {}, {}
            for line in open(f"{game}/engine.log", errors="replace"):
                if " regions: " in line and not prices:
                    for r, _, rest in REGION.findall(line):
                        prices[int(r)] = {k[3:]: float(v) for k, v in re.findall(r"(\w+) (\d+)", rest)}
                    continue
                m = ORDER.search(line)
                if m:
                    kind_of[m.group(1)] = m.group(2)[3:]
                    continue
                b = BEGUN.search(line)
                if not b or int(b.group(1)) > until * 60 * 30 or b.group(2)[3:] not in ("mex", "moho"):
                    continue
                kind = kind_of.get(b.group(3), "?")
                if kind not in types:
                    continue
                site = (int(b.group(5)), int(b.group(6)))
                region = min(spots, key=lambda s: (s[0] - site[0]) ** 2 + (s[1] - site[1]) ** 2)[2]
                price = prices.get(region, {}).get(kind)
                if price is None:
                    continue
                age = float(b.group(4))
                by_region.setdefault(region, []).append((price + OVERHEAD, age))
                priced_total += price + OVERHEAD
                real_total += age
        print(f"== {os.path.basename(batch)}: {', '.join(types)} extractors to {until:.0f}:00, by region (the region numbers are each game's own)")
        for r in sorted(by_region):
            v = by_region[r]
            print(f"   r{r:<3} priced {statistics.median(p for p, _ in v):5.1f} s, realized median {statistics.median(a for _, a in v):5.1f} s over {len(v)}")
        if priced_total:
            print(f"   all: realized {real_total:.0f} s against priced {priced_total:.0f} s ({real_total / priced_total:.2f})")


main()
