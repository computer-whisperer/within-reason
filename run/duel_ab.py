#!/usr/bin/env python3
"""Compare two duel batches run on the same pairings: what a change to the first army's orders was worth.

    run/duel_ab.py <without dir> <with dir> [--by-them] [--min-duels N]

Prints, per pairing seen from the first unit's side: mean margin in each arm with its standard error, the
metal each side lost summed over the arm's duels (the ledger's "metal killed per metal lost"), and how spread
out each army stood when the first shot landed (`spread_x` in `duels.csv`, the probe added for this), and the
first army's muzzled share of its in-reach seconds (the fire instrument, `docs/harness/duels.md`; batches from
before 2026-09-24 have none: shown as nan), its friendly-fire share of the damage it did, and the shape instrument
(2026-09-25): the first army's median nearest-friend distance at contact and the share of its units in reach with
a friend on their line of fire. A batch fought in several formations (`--formation`) is split by them; the lane
(`--lane`) is what an A/B compares, so it is not part of the key.
"""

import csv
import statistics
import sys
from pathlib import Path


def rows(directory):
    with open(Path(directory) / "duels.csv") as handle:
        # A wipe in under 15 s in which neither army hurt the other is the field, not a fight (a stray bomb from the
        # sweep before: form-stout-on-nofocus rep 3, 27,240 damage to one side in 4.9 s, 0 dealt by either).
        return [r for r in csv.DictReader(handle) if r["reason"] != "spawn_failed" and not artefact(r)]


def artefact(r):
    try:
        return float(r["seconds"]) < 15 and float(r.get("dealt_x") or 1) == 0 and float(r.get("dealt_y") or 1) == 0
    except ValueError:
        return False


def tally(rs):
    margin = [float(r["value_left_x"]) - float(r["value_left_y"]) for r in rs]
    killed = sum((1 - float(r["value_left_y"])) * float(r["metal_y"]) for r in rs)
    lost = sum((1 - float(r["value_left_x"])) * float(r["metal_x"]) for r in rs)
    # Batches recorded before the dispersion probe have no spread columns; report them as unknown, not as zero.
    spread = [statistics.mean(float(r[c]) for r in rs) if all(r.get(c) for r in rs) else float("nan") for c in ("spread_x", "spread_y")]
    error = statistics.stdev(margin) / len(margin) ** 0.5 if len(margin) > 1 else 0.0
    if all(r.get("reach_s_x") for r in rs):
        muzzled = sum(float(r[f"muzzled_{c}_x"]) for r in rs for c in ("line", "edge", "clear"))
        muzzled /= max(1.0, sum(float(r["reach_s_x"]) for r in rs))
        ff = sum(float(r["ff_x"]) for r in rs)
        ff /= max(1.0, ff + sum(float(r["dealt_x"]) for r in rs))
    else:
        muzzled = ff = float("nan")
    if all(r.get("fol_in_x") for r in rs):
        nns = [float(r["nn_x"]) for r in rs if r["nn_x"] not in ("", "NaN", "nan")]
        nn = statistics.mean(nns) if nns else float("nan")
        fol_in = sum(float(r["fol_in_x"]) for r in rs)
        fol = sum(float(r["fol_blocked_x"]) for r in rs) / fol_in if fol_in else float("nan")
    else:
        nn = fol = float("nan")
    return len(margin), statistics.mean(margin), error, killed, lost, spread, muzzled, ff, nn, fol


def key(r):
    """The pairing, and the formations when the batch names them."""
    forms = (r.get("form_x") or "", r.get("form_y") or "")
    return (r["x"], r["y"]) + (forms if any(forms) else ())


def label(k):
    return f"{k[0]} v {k[1]}" + (f" {k[2]}/{k[3]}" if len(k) > 2 else "")


def main(argv):
    by_them = "--by-them" in argv
    floor, skip = 1, set()
    if "--min-duels" in argv:
        i = argv.index("--min-duels")
        floor, skip = int(argv[i + 1]), {i + 1}
    elif any(a.startswith("--min-duels=") for a in argv):
        floor = int(next(a for a in argv if a.startswith("--min-duels=")).split("=")[1])
    args = [a for i, a in enumerate(argv) if not a.startswith("--") and i not in skip]
    if len(args) != 2:
        print(__doc__)
        return 1
    without, with_it = rows(args[0]), rows(args[1])
    keys = sorted({key(r) for r in without} | {key(r) for r in with_it})
    if by_them:
        groups = {k[1]: [j for j in keys if j[1] == k[1]] for k in keys}
    else:
        groups = {label(k): [k] for k in keys}
    width = max([22] + [len(g) + 2 for g in groups])
    print(f"{'pairing':<{width}}{'n':>4}{'margin off':>12}{'margin on':>12}{'gain':>9}{'k/l off':>9}{'k/l on':>8}{'spread off':>12}{'spread on':>11}{'muzzled off':>13}{'on':>6}{'ff off':>8}{'on':>6}{'nn off':>8}{'on':>6}{'fol off':>9}{'on':>6}")
    overall = [0.0, 0.0, 0.0, 0.0]
    for name, members in groups.items():
        a = [r for r in without if key(r) in members]
        b = [r for r in with_it if key(r) in members]
        if len(a) < floor or len(b) < floor:
            continue
        na, ma, ea, ka, la, sa, za, fa, nna, fola = tally(a)
        nb, mb, eb, kb, lb, sb, zb, fb, nnb, folb = tally(b)
        overall = [overall[0] + ka, overall[1] + la, overall[2] + kb, overall[3] + lb]
        print(
            f"{name:<{width}}{na:>4}{ma:>+9.3f}+-{ea:<4.3f}{mb:>+9.3f}+-{eb:<4.3f}{mb - ma:>+9.3f}"
            f"{ka / max(la, 1):>9.2f}{kb / max(lb, 1):>8.2f}{sa[0]:>9.0f}/{sa[1]:<3.0f}{sb[0]:>8.0f}/{sb[1]:<3.0f}"
            f"{100 * za:>12.0f}%{100 * zb:>5.0f}%{100 * fa:>7.1f}%{100 * fb:>5.1f}%{nna:>8.0f}{nnb:>6.0f}{100 * fola:>8.0f}%{100 * folb:>5.0f}%"
        )
    print(
        f"\nover everything: metal killed per metal lost {overall[0] / max(overall[1], 1):.2f} without,"
        f" {overall[2] / max(overall[3], 1):.2f} with"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
