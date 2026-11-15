#!/usr/bin/env python3
"""The orders a soldier is sent a second, by what it and its group were doing: the lane's order rate read off a
flagship record, the same way for the group engine of before 2026-11-15 and the one after (both records name a
group's members in `formed` and `exceptions`).

usage: run/soldier_orders.py run/matches/<batch>/<NN> [...]

Orders are the record's `cmd` rows of kind move, fight and attack to a soldier (`match_read.is_army_def`), counted
in the second before each sample.

  walking / not  a soldier-second is walking when the soldier moved more than 30 elmos in it
  at rest        member-seconds of a formation group that says `arrived`, the member in its slot
  on the way     member-seconds of a formation group that has not arrived, by whether the member is in its slot and
                 whether it is of the group's slowest type that second (the pace type, which the frame walks at) or
                 a faster one

The faster type in its slot is the row to read: its point is its slot itself (`crates/micro/src/army.rs`,
`Company::tick`: only the pace type's point is carried ahead), and a member within 100 of the point it was last
sent to is sent again each time the point has moved 16 (`crates/micro/src/lib.rs`, `resend_due`).
"""
import collections, math, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from match_read import first, is_army_def, rows as read_rows  # noqa: E402


def read(d):
    defs, soldiers, prev = None, {}, {}
    sent = collections.Counter()          # (unit, second) -> orders
    secs, orders = collections.Counter(), collections.Counter()
    for r in read_rows(first(d, "record-")):
        t = r.get("t")
        if "unit_defs" in r:
            defs = r["unit_defs"]
        if t == "ev" and r.get("k") == "finished" and is_army_def(defs[r["d"]]):
            soldiers[r["u"]] = defs[r["d"]]
        elif t == "cmd":
            for c in r["c"]:
                if c and c[0] in ("move", "fight", "attack") and c[1] in soldiers:
                    sent[(c[1], r["f"] // 30)] += 1
        elif t == "s":
            sec = r["f"] // 30
            now = {u[0]: (u[2], u[3]) for u in r["own"] if u[0] in soldiers and not (u[5] & 1)}
            for u, p in now.items():
                if u in prev:
                    key = "walking" if math.dist(p, prev[u]) > 30 else "not walking"
                    secs[key] += 1
                    orders[key] += sent.get((u, sec - 1), 0)
            prev = now
            for name, g in (r.get("groups") or {}).items():
                if "answer" in name:
                    continue
                members = set(g["formed"]) | set(e[0] for e in g["exceptions"])
                speeds = {u: soldiers[u].get("speed", 0) for u in members if u in soldiers}
                if not speeds:
                    continue
                slowest = min(speeds.values())
                for u in speeds:
                    slot = "in its slot" if u in g["formed"] else "out of its slot"
                    if g["arrived"]:
                        key = f"at rest, {slot}"
                    else:
                        key = f"on the way, {slot}, {'pace type' if speeds[u] <= slowest * 1.05 else 'faster type'}"
                    secs[key] += 1
                    orders[key] += sent.get((u, sec - 1), 0)
                    if not g["arrived"]:
                        secs["on the way, all"] += 1
                        orders["on the way, all"] += sent.get((u, sec - 1), 0)
    return secs, orders


ROWS = [
    "walking", "not walking", "at rest, in its slot", "at rest, out of its slot", "on the way, all",
    "on the way, in its slot, pace type", "on the way, in its slot, faster type",
    "on the way, out of its slot, pace type", "on the way, out of its slot, faster type",
]

if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    games = [(os.path.basename(os.path.dirname(d.rstrip("/"))).split("-", 1)[-1], read(d.rstrip("/"))) for d in sys.argv[1:]]
    print(f"{'orders a second (over soldier-seconds)':<42}" + "".join(f"{name[:22]:>24}" for name, _ in games))
    for row in ROWS:
        cells = "".join(f"{(f'{o[row] / s[row]:.2f} ({s[row]})' if s[row] else '-'):>24}" for _, (s, o) in games)
        print(f"{row:<42}{cells}")
