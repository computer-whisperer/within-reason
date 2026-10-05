#!/usr/bin/env python3
"""Enemy parties a seat would be asked about, counted from a replay's records.

usage: run/replay_party_counts.py <match dir> [seat ...]

A replay's seat is a spectator, so its `en` is the whole enemy and not what a player saw. Each minute: the enemy's
finished mobile units, the parties they make by the hands' rule (mobile units chained within 400, picture.rs
PARTY_RADIUS), how many of those are one unit, the largest, the parties within NEAR of a unit of any seat named (a
stand-in for sight), the units in those, and for each seat the parties within NEAR of one of its own units.
"""
import json, sys, os

RADIUS, NEAR = 400.0, 1200.0

def parties(points):
    left = list(range(len(points)))
    out = []
    seen = [False] * len(points)
    for s in left:
        if seen[s]: continue
        seen[s] = True
        members, i = [s], 0
        while i < len(members):
            ax, az = points[members[i]]
            for o in left:
                if not seen[o] and (points[o][0]-ax)**2 + (points[o][1]-az)**2 < RADIUS**2:
                    seen[o] = True; members.append(o)
            i += 1
        out.append([points[m] for m in members])
    return out

def main():
    d = sys.argv[1]
    seats = [int(s) for s in sys.argv[2:]] or [0]
    head = json.loads(open(os.path.join(d, f"record-{seats[0]}.jsonl")).readline())
    defs = head["unit_defs"]
    rows = {}
    for seat in seats:
        for line in open(os.path.join(d, f"record-{seat}.jsonl")):
            if '"t":"s"' not in line: continue
            r = json.loads(line)
            if r["f"] % 1800: continue
            row = rows.setdefault(r["f"], {"own": {}})
            if "ps" not in row:
                en = [(e[2], e[3]) for e in r.get("en", []) if defs[e[1]]["speed"] > 0 and (len(e) < 6 or not e[5])]
                row["ps"] = parties(en)
            row["own"][seat] = [(u[2], u[3]) for u in r.get("own", [])]
    def near(party, own):
        return any((x-ox)**2 + (z-oz)**2 < NEAR**2 for x, z in party for ox, oz in own)
    print("min mobile parties singles biggest | near our side: parties singles units | near each seat")
    for f in sorted(rows):
        ps, own = rows[f]["ps"], rows[f]["own"]
        side = [p for p in ps if near(p, [u for o in own.values() for u in o])]
        each = " ".join(str(sum(near(p, own.get(s, [])) for p in ps)) for s in seats)
        print(f"{f//1800:3d} {sum(map(len, ps)):6d} {len(ps):7d} {sum(len(p)==1 for p in ps):7d} {max(map(len, ps), default=0):7d} | {len(side):7d} {sum(len(p)==1 for p in side):7d} {sum(map(len, side)):6d} | {each}")

if __name__ == "__main__":
    main()
