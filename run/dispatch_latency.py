#!/usr/bin/env python3
"""The hands' dispatch rules measured on a territory batch: for each rule and unit type the orders' walk (the builder's
distance from the site when ordered) and the seconds from the order to the frame begun, the help and walk-home orders'
walks the same way, and the minutes 7-14 of each game's bank against its cap with the float (income less spend). From
the bot's `order:`, `begun:` and `curve:` lines in each game's engine.log (bots/territory/README.md, T-DISPATCH).

    run/dispatch_latency.py <batch dir>

Read 2026-10-25 on terr100 and terr97: the count rule's turret orders at a 90th-percentile walk of 4,250-5,450 elmos
and 90-107 s to the frame against the flows' 1,300-1,500 and 33-37 s, the dearest rule's fusion helpers from
5,700-6,800, and the metal at its cap at 10:00 and 11:00 in every game while the turrets stood at 2-5.
"""
import collections, glob, re, statistics as st, sys

if len(sys.argv) < 2 or sys.argv[1].startswith("-"):
    sys.exit(__doc__)
batch = sys.argv[1]
ORDER = re.compile(r"f=(\d+) order: unit (\d+) (\w+): build (\w+) at \(([-\d.]+),([-\d.]+)\), (\d+) off, (\d+) from home \((\w[\w-]*)\)")
HELP = re.compile(r"f=(\d+) order: unit (\d+) (\w+): help unit (\d+) \((\w+)\), (\d+) off, (\d+) from home \((\w[\w-]*)\)")
WALK = re.compile(r"f=(\d+) order: unit (\d+) (\w+): walk home to \(([-\d.]+),([-\d.]+)\), (\d+) off, (\d+) from home \((\w[\w-]*)\)")
BEGUN = re.compile(r"f=(\d+) begun: (\w+) by unit (\d+) at \d+, ([\d.]+) s after its order, site \(([-\d.]+),([-\d.]+)\)")
CURVE = re.compile(r"f=(\d+) curve: .* nanos=(\d+) build_power=(\d+) metal (\d+)/(\d+) \+([\d.]+) -([\d.]+) energy (\d+)/(\d+)")


def p90(xs):
    return xs[min(len(xs) - 1, int(0.9 * len(xs)))]


for game in sorted(glob.glob(f"{batch}/0*")):
    orders = {}  # (builder, type) -> (frame, off, rule)
    lat = collections.defaultdict(list)  # (rule, type) -> [(latency, off)]
    helps = collections.defaultdict(list)  # (rule, type) -> [off]
    walks = []  # (frame, builder type, off)
    curves = []
    for line in open(f"{game}/engine.log", errors="replace"):
        if m := ORDER.search(line):
            f, b, _bt, u, _x, _y, off, _home, rule = m.groups()
            orders[(b, u)] = (int(f), int(off), rule)
        elif m := HELP.search(line):
            _f, _b, _bt, _fr, u, off, _home, rule = m.groups()
            helps[(rule, u)].append(int(off))
        elif m := WALK.search(line):
            f, _b, bt, _x, _y, off, _home, _rule = m.groups()
            walks.append((int(f), bt, int(off)))
        elif m := BEGUN.search(line):
            _f, u, b, after, _x, _y = m.groups()
            if o := orders.pop((b, u), None):
                lat[(o[2], u)].append((float(after), o[1]))
        elif m := CURVE.search(line):
            curves.append(tuple(float(v) for v in m.groups()))
    print(f"== {game}")
    print("  min  nanos  bp     bank/cap      income  spend  float")
    for c in curves:
        minute = int(c[0] // 1800)
        if 7 <= minute <= 14:
            print(f"  {minute:3d}  {int(c[1]):5d}  {int(c[2]):5d}  {int(c[3]):6d}/{int(c[4]):<6d}  {c[5]:6.1f} {c[6]:6.1f}  {c[5] - c[6]:+6.1f}")
    print("  build orders by rule and type: n, latency median/p90 s, walk median/p90 elmos")
    for (rule, u), v in sorted(lat.items(), key=lambda kv: (kv[0][0], -len(kv[1]))):
        if len(v) < 3:
            continue
        latency = sorted(a for a, _ in v)
        walk = sorted(o for _, o in v)
        print(f"  {rule:11s} {u:10s} n={len(v):3d}  latency {st.median(latency):6.1f}/{p90(latency):6.1f}  walk {st.median(walk):6.0f}/{p90(walk):6.0f}")
    print("  help orders by rule and frame type: n, walk median/p90")
    for (rule, u), v in sorted(helps.items(), key=lambda kv: (kv[0][0], -len(kv[1]))):
        if len(v) < 3:
            continue
        walk = sorted(v)
        print(f"  {rule:11s} {u:10s} n={len(v):3d}  walk {st.median(walk):6.0f}/{p90(walk):6.0f}")
    if walks:
        by_minute = collections.Counter(f // 1800 for f, _, _ in walks)
        offs = sorted(o for _, _, o in walks)
        print(f"  walk-home orders: n={len(walks)}, distance to the farm median/p90 {st.median(offs):.0f}/{p90(offs):.0f}; by minute " + " ".join(f"{m}:{n}" for m, n in sorted(by_minute.items())))
