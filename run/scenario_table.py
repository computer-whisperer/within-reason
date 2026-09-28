#!/usr/bin/env python3
"""One markdown row per `duel --scenario` batch: the margins a scripted or directed engagement reached, by seed.

    run/scenario_table.py <batch dir> [<batch dir> ...] [--by-seed]

Per batch (`duels.csv`, side 0 = x = ours): duels, W-L-draw, the mean margin (value left ours minus theirs, at the
end) with its standard deviation, the worst and best, each side's mean value left at 30 s and 60 s after the first
orders and at the end, and the mean seconds. `--by-seed` adds a row per engine seed (`--seeds`). A duel the director
could not spawn is left out, as `duel --report` does.
"""
import csv
import statistics
import sys
from pathlib import Path


def rows(directory):
    with open(Path(directory) / "duels.csv") as handle:
        return [r for r in csv.DictReader(handle) if r["reason"] != "spawn_failed"]


def number(r, key):
    try:
        return float(r.get(key) or "nan")
    except ValueError:
        return float("nan")


def line(name, rs):
    margins = [number(r, "value_left_x") - number(r, "value_left_y") for r in rs]
    wins = sum(r["winner"] == "x" for r in rs)
    losses = sum(r["winner"] == "y" for r in rs)
    above = sum(m > 0 for m in margins)
    mean = lambda key: statistics.fmean(number(r, key) for r in rs)
    sd = statistics.stdev(margins) if len(margins) > 1 else 0.0
    return (f"| {name} | {len(rs)} | {wins}-{losses}-{len(rs) - wins - losses} | {above}/{len(rs)} | "
            f"{statistics.fmean(margins):+.3f} ± {sd:.3f} | {min(margins):+.3f} / {max(margins):+.3f} | "
            f"{mean('left30_x'):.3f} / {mean('left30_y'):.3f} | {mean('left60_x'):.3f} / {mean('left60_y'):.3f} | "
            f"{mean('value_left_x'):.3f} / {mean('value_left_y'):.3f} | {mean('seconds'):.0f} |")


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    by_seed = "--by-seed" in sys.argv
    if not args:
        sys.exit(__doc__)
    print("| Batch | Duels | W-L-D (wiped) | Margin > 0 | Margin, mean ± sd | Worst / best | Left at 30 s, ours / theirs "
          "| Left at 60 s | Left at the end | Seconds |")
    print("|---|---|---|---|---|---|---|---|---|---|")
    for directory in args:
        rs = rows(directory)
        name = Path(directory).name.split("-duel-", 1)[-1]
        if not rs:
            print(f"| {name} | 0 | | | | | | | | |")
            continue
        print(line(name, rs))
        if by_seed:
            for seed in sorted({r.get("seed", "") for r in rs}):
                print(line(f"&nbsp;&nbsp;seed {seed or '-'}", [r for r in rs if r.get("seed", "") == seed]))


if __name__ == "__main__":
    main()
