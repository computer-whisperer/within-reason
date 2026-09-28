#!/usr/bin/env python3
"""Read one scenario duel's per-second log (`duel --scenario`, `seconds/*.jsonl`) as a timeline for a TAS author.

    run/tas_read.py <log.jsonl> [--every N] [--scenario FILE]

Per second (every N-th, and every second something died): each side's value left, our living units and theirs with
health, who died since the last line, the damage each of their units did that second, and the script rows that went
out. With `--scenario` the units are named by type.
"""
import json
import sys


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    every = int(sys.argv[sys.argv.index("--every") + 1]) if "--every" in sys.argv else 5
    names = None
    if "--scenario" in sys.argv:
        s = json.load(open(sys.argv[sys.argv.index("--scenario") + 1]))
        names = [[u["type"].replace("arm", "") for u in side["units"]] for side in s["sides"]]
        args = [a for a in args if a != sys.argv[sys.argv.index("--scenario") + 1]]
    alive = None
    dealt_y = {}
    for line in open(args[0]):
        r = json.loads(line)
        now = ({u[0] for u in r["x"]}, {u[0] for u in r["y"]})
        for row in r["dealt"]["y"]:
            dealt_y[row[0]] = dealt_y.get(row[0], 0) + row[-1]
        died = (alive[0] - now[0], alive[1] - now[1]) if alive else (set(), set())
        alive = now
        if r["t"] % every and not died[0] and not died[1] and not r["orders"]:
            continue
        label = lambda side, i: f"{names[side][i]}{i}" if names else str(i)
        ours = " ".join(f"{u[0]}:{u[3]:.2f}@{u[1]:.0f},{u[2]:.0f}" for u in r["x"])
        theirs = " ".join(f"{label(1, u[0])}:{u[3]:.2f}@{u[1]:.0f},{u[2]:.0f}" for u in r["y"])
        print(f"t={r['t']:3d} left {r['left'][0]:.3f}/{r['left'][1]:.3f}"
              + (f" died ours {sorted(died[0])} theirs {[label(1, i) for i in sorted(died[1])]}" if died[0] or died[1] else "")
              + (f" orders {[(o['row'], o['sent']) for o in r['orders']]}" if r["orders"] else ""))
        print(f"   theirs {theirs}")
        print(f"   ours({len(r['x'])}) {ours}")
    print("damage done by each of theirs over the fight:", {label(1, i): round(d) for i, d in sorted(dealt_y.items())})


if __name__ == "__main__":
    main()
