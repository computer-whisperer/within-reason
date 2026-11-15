#!/usr/bin/env python3
"""A scenario batch by our soldiers alone: the share of side 0's mobile units' metal x health left at 30 s and at the
end, beside the other side's whole value. The duel's own `left` counts everything in the cut, so a cut that carries
our buildings (added by hand: `run/engagement.py` takes armed units only) reads our side through them; this leaves
them out. Then, over the seconds while both sides lived: the share of our soldier-seconds that dealt damage and that
had one of his within reach, and the damage dealt and taken a second.

    run/scenario_soldiers.py <scenario file> <batch dir> [<scenario file> <batch dir> ...]

Costs, speeds and reaches are the glossary sheet's (`docs/glossary/units.csv`).
"""
import csv, glob, json, math, os, statistics, sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
SHEET = {row["unit"]: row for row in csv.DictReader(open(os.path.join(ROOT, "docs/glossary/units.csv")))}


def number(unit, column):
    try:
        return float(SHEET[unit][column])
    except (KeyError, ValueError):
        return 0.0


def main():
    args = sys.argv[1:]
    if len(args) < 2 or len(args) % 2:
        sys.exit(__doc__)
    for scen_path, batch in zip(args[0::2], args[1::2]):
        units = json.load(open(scen_path))["sides"][0]["units"]
        kind = [u["type"] for u in units]
        soldier = [number(k, "speed") > 0 for k in kind]
        metal = [number(k, "metal") for k in kind]
        reach = [number(k, "range") for k in kind]
        start = sum(m * u.get("health", 1.0) for m, u, s in zip(metal, units, soldier) if s)
        a30, a60, h30, h60, firing, in_reach, dealt_s, taken_s = [], [], [], [], [], [], [], []
        for path in sorted(glob.glob(batch.rstrip("/") + "/seconds/*.jsonl")):
            at30 = last = None
            seconds = soldier_s = fire_s = reach_s = 0
            dealt = taken = 0.0
            for line in open(path):
                r = json.loads(line)
                if "x" not in r or "t" not in r:
                    continue
                mine = [u for u in r["x"] if soldier[u[0]] and u[3] > 0]
                his = [u for u in r["y"] if u[3] > 0]
                last = (sum(metal[u[0]] * u[3] for u in mine) / start, r["left"][1])
                if r["t"] == 30:
                    at30 = last
                if not mine or not his:
                    continue
                by = {}
                for index, _, damage in (r.get("dealt") or {}).get("x", []):
                    if soldier[index]:
                        by[index] = by.get(index, 0.0) + damage
                seconds += 1
                soldier_s += len(mine)
                fire_s += len(by)
                reach_s += sum(1 for u in mine if any(math.dist((u[1], u[2]), (h[1], h[2])) <= reach[u[0]] for h in his))
                dealt += sum(by.values())
                taken += sum(row[2] for row in (r.get("dealt") or {}).get("y", []))
            if last is None:
                continue
            at30 = at30 or last
            a30.append(at30[0]); h30.append(at30[1]); a60.append(last[0]); h60.append(last[1])
            firing.append(100 * fire_s / max(soldier_s, 1))
            in_reach.append(100 * reach_s / max(soldier_s, 1))
            dealt_s.append(dealt / max(seconds, 1))
            taken_s.append(taken / max(seconds, 1))
        if not a30:
            print(f"{batch}: no per-second logs")
            continue
        m = statistics.mean
        name = os.path.basename(batch.rstrip("/")).split("-duel-")[-1]
        print(f"{name:24} n={len(a30)}  at 30 s our soldiers {m(a30):.2f} ({min(a30):.2f} to {max(a30):.2f}), his {m(h30):.2f} ({min(h30):.2f} to {max(h30):.2f});  "
              f"at the end {m(a60):.2f} ({min(a60):.2f} to {max(a60):.2f}), his {m(h60):.2f} ({min(h60):.2f} to {max(h60):.2f});  "
              f"soldier-seconds dealing damage {m(firing):.0f} %, with one of his in reach {m(in_reach):.0f} %;  dealt {m(dealt_s):.0f} a second, taken {m(taken_s):.0f}")


if __name__ == "__main__":
    main()
