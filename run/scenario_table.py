#!/usr/bin/env python3
"""One markdown row per `duel --scenario` batch: the margins a scripted or directed engagement reached, by seed.

    run/scenario_table.py <batch dir> [<batch dir> ...] [--by-seed] [--position i,j,k]

Per batch (`duels.csv`, side 0 = x = ours): duels, W-L-draw, the mean margin (value left ours minus theirs, at the
end) with its standard deviation, the worst and best, each side's mean value left at 30 s and 60 s after the first
orders and at the end, and the mean seconds. `--by-seed` adds a row per engine seed (`--seeds`). A duel the director
could not spawn is left out, as `duel --report` does. `--position` names side 1's units (indices in the file) that
make up the position: a duel then also counts as "fell" when none of them stands in its last per-second log line,
and "fell, ahead" when it fell with our value left above theirs.
"""
import csv
import json
import statistics
import sys
from pathlib import Path


def rows(directory, position=None):
    with open(Path(directory) / "duels.csv") as handle:
        rs = [r for r in csv.DictReader(handle) if r["reason"] != "spawn_failed"]
    for r in rs:
        r["fell"] = None
        if position is not None and r.get("log"):

            with open(Path(directory) / "seconds" / r["log"]) as log:
                last = json.loads(log.read().strip().splitlines()[-1])
            standing = {u[0] for u in last["y"]}
            # A side wiped on the deciding tick may not have had its last line written: take the result's word.
            r["fell"] = r["winner"] == "x" or not (standing & position)
    return rs


def number(r, key):
    try:
        return float(r.get(key) or "nan")
    except ValueError:
        return float("nan")


def line(name, rs):
    margins = [number(r, "value_left_x") - number(r, "value_left_y") for r in rs]
    wins = sum(r["winner"] == "x" for r in rs)
    fell = sum(bool(r["fell"]) for r in rs)
    ahead = sum(bool(r["fell"]) and m > 0 for r, m in zip(rs, margins))
    fell_text = f" {fell}/{len(rs)} | {ahead}/{len(rs)} |" if rs and rs[0]["fell"] is not None else ""
    losses = sum(r["winner"] == "y" for r in rs)
    above = sum(m > 0 for m in margins)
    mean = lambda key: statistics.fmean(number(r, key) for r in rs)
    sd = statistics.stdev(margins) if len(margins) > 1 else 0.0
    return (f"| {name} | {len(rs)} | {wins}-{losses}-{len(rs) - wins - losses} | {above}/{len(rs)} | "
            f"{statistics.fmean(margins):+.3f} ± {sd:.3f} | {min(margins):+.3f} / {max(margins):+.3f} | "
            f"{mean('left30_x'):.3f} / {mean('left30_y'):.3f} | {mean('left60_x'):.3f} / {mean('left60_y'):.3f} | "
            f"{mean('value_left_x'):.3f} / {mean('value_left_y'):.3f} | {mean('seconds'):.0f} |" + fell_text)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    by_seed = "--by-seed" in sys.argv
    position = None
    if "--position" in sys.argv:
        value = sys.argv[sys.argv.index("--position") + 1]
        position = {int(i) for i in value.split(",")}
        args = [a for a in args if a != value]
    if not args:
        sys.exit(__doc__)
    extra = (" Fell | Fell, ahead |", "---|---|") if position else ("", "")
    print("| Batch | Duels | W-L-D (wiped) | Margin > 0 | Margin, mean ± sd | Worst / best | Left at 30 s, ours / theirs "
          "| Left at 60 s | Left at the end | Seconds |" + extra[0])
    print("|---|---|---|---|---|---|---|---|---|---|" + extra[1])
    for directory in args:
        rs = rows(directory, position)
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
