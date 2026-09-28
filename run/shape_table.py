#!/usr/bin/env python3
"""One markdown row per duel batch with the body-shape judge's columns (`docs/design/2026-09-28-body-shape.md`).

    run/shape_table.py <batch dir> [<batch dir> ...] [--position i,j,k]

Side 0 / the first army is `x` (ours). Per batch: duels, W-L-D (a side wiped; D neither), the mean margin (value left
ours minus theirs) with its sd and the worst, and our army's fire and shape instrument pooled over the batch's duels:
in reach while engaged (in-reach seconds over engaged seconds: an enemy within the unit's reach + 20, over an enemy
within 700), muzzled (muzzled seconds over in-reach seconds), friend on the line (units in reach with a friend within
24 of the line to their nearest enemy, over units in reach, at the first damage and ten seconds later), the median
nearest-friend distance at those two moments (mean over duels), and the friendly-fire share of the damage we did.
`--position` as in `run/scenario_table.py` (side 1's units that make up the position): adds "fell, ahead".
"""
import csv
import json
import statistics
import sys
from pathlib import Path


def f(r, k):
    try:
        return float(r.get(k) or "nan")
    except ValueError:
        return float("nan")


def artefact(r):
    return f(r, "seconds") < 15 and f(r, "dealt_x") == 0 and f(r, "dealt_y") == 0


def row(directory, position):
    d = Path(directory)
    rs = [r for r in csv.DictReader(open(d / "duels.csv")) if r["reason"] != "spawn_failed" and not artefact(r)]
    if not rs:
        return f"| {d.name} | 0 |"
    margin = [f(r, "value_left_x") - f(r, "value_left_y") for r in rs]
    w = sum(r["winner"] == "x" for r in rs)
    l = sum(r["winner"] == "y" for r in rs)
    s = lambda k: sum(f(r, k) for r in rs if f(r, k) == f(r, k))
    reach, engaged = s("reach_s_x"), s("engaged_s_x")
    muzzled = s("muzzled_line_x") + s("muzzled_edge_x") + s("muzzled_clear_x")
    fol_in, fol = s("fol_in_x"), s("fol_blocked_x")
    nns = [f(r, "nn_x") for r in rs if f(r, "nn_x") == f(r, "nn_x")]
    ff, dealt = s("ff_x"), s("dealt_x")
    fell = ""
    if position is not None:
        n = 0
        for r in rs:
            if not r.get("log"):
                continue
            last = json.loads((d / "seconds" / r["log"]).read_text().strip().splitlines()[-1])
            gone = r["winner"] == "x" or not ({u[0] for u in last["y"]} & position)
            n += gone and margin[rs.index(r)] > 0
        fell = f" {n}/{len(rs)} |"
    sd = statistics.stdev(margin) if len(margin) > 1 else 0.0
    ratio = lambda a, b: f"{a / b:.2f}" if b else "-"
    return (f"| {d.name.split('-duel-', 1)[-1]} | {len(rs)} | {w}-{l}-{len(rs) - w - l} | {statistics.mean(margin):+.3f} ± {sd:.3f} | "
            f"{min(margin):+.3f} | {ratio(reach, engaged)} | {ratio(muzzled, reach)} | {ratio(fol, fol_in)} | "
            f"{statistics.mean(nns) if nns else float('nan'):.0f} | {ratio(ff, ff + dealt)} |{fell}")


def main():
    args = sys.argv[1:]
    position = None
    if "--position" in args:
        i = args.index("--position")
        position = {int(x) for x in args[i + 1].split(",")}
        del args[i:i + 2]
    head = "| Batch | Duels | W-L-D | Margin, mean ± sd | Worst | In reach while engaged | Muzzled | Friend on the line | Nearest friend | Friendly fire |"
    rule = "|---|---|---|---|---|---|---|---|---|---|"
    if position is not None:
        head += " Fell, ahead |"
        rule += "---|"
    print(head)
    print(rule)
    for d in args:
        print(row(d, position))


if __name__ == "__main__":
    main()
