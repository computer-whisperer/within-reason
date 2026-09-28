#!/usr/bin/env python3
"""How many of each unit the experienced players ever have in one body, and how many they own at once (the user,
2026-09-28: units with no special micro lose effectiveness at a scale, and we throw an army that cannot all engage at
once against a few tier-2 units).
    run/replays/composition.py [--map "Comet Catcher Remake 1.8"] [--floor 40] [--link 200] [--ours <match dir>...]
Pros: every carded game on the map with both players at OS `--floor` and above (run/replays/shapes.py's selection and
its reading of records: side 0 from `own`, side 1 from `en`, every sample). A body: soldiers chained at `--link`
elmos or less (shapes.py's core, 200 by default; commanders and aircraft left out). Per unit type: the largest number
that ever stood in one body over the pool (with the game and clock), the median over games of each game's largest
body count, the largest number owned at once, and the same for `--ours` records (us only). Rows sorted by the pros'
largest body.
"""
import argparse
import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import shapes  # noqa: E402


def clock(t):
    return f"{t // 60}:{t % 60:02d}"


def scan(g, sides, link):
    """Per type: (max in one body, where), per-game max body count, max owned at once."""
    body_max, owned_max, per_game = {}, {}, {}
    shapes.GROUP_LINK = link
    for side in sides:
        who = g.players[side] or f"side {side}"
        for t in sorted(g.units[side]):
            units = {uid: u for uid, u in g.units[side][t].items() if shapes.soldier(u[0])}
            owned = collections.Counter(u[0] for u in units.values())
            for name, n in owned.items():
                if n > owned_max.get(name, (0,))[0]:
                    owned_max[name] = (n, f"{g.label} {who} {clock(t)}")
            for grp in shapes.components(units):
                c = collections.Counter(units[i][0] for i in grp)
                for name, n in c.items():
                    key = (g.label, side, name)
                    per_game[key] = max(per_game.get(key, 0), n)
                    if n > body_max.get(name, (0,))[0]:
                        others = ", ".join(f"{k} {v}" for k, v in c.most_common() if k != name)
                        body_max[name] = (n, f"{g.label} {who} {clock(t)}, body of {len(grp)}" + (f" with {others}" if others else ""))
    return body_max, owned_max, per_game


def merge(acc, new):
    for name, (n, where) in new.items():
        if n > acc.get(name, (0,))[0]:
            acc[name] = (n, where)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--map", default="Comet Catcher Remake 1.8")
    ap.add_argument("--floor", type=float, default=40)
    ap.add_argument("--link", type=float, default=200)
    ap.add_argument("--ours", nargs="*", default=[])
    a = ap.parse_args()
    shapes.T = shapes.load_tables()
    games = shapes.pro_games(a.map, a.floor)
    print(f"pro games: {len(games)} ({a.map}, both players OS {a.floor:g}+), bodies chained at {a.link:g}")
    body, owned, per_game = {}, {}, {}
    for g in games:
        b, o, p = scan(g, (0, 1), a.link)
        merge(body, b)
        merge(owned, o)
        per_game.update(p)
    ours_body, ours_owned = {}, {}
    for path in a.ours:
        g = shapes.our_game(path)
        b, o, _ = scan(g, (0,), a.link)
        merge(ours_body, b)
        merge(ours_owned, o)
    names = sorted(set(body) | set(ours_body), key=lambda n: -body.get(n, (0,))[0])
    print(f"{'unit':12} {'pro max body':>12} {'median game max':>15} {'pro max owned':>13} {'ours max body':>13} {'ours max owned':>14}  where the pros' largest body stood")
    for n in names:
        gm = sorted(v for (gl, s, nm), v in per_game.items() if nm == n)
        med = gm[len(gm) // 2] if gm else 0
        print(f"{n:12} {body.get(n, (0,))[0]:12} {med:15} {owned.get(n, (0,))[0]:13} {ours_body.get(n, (0,))[0]:13} {ours_owned.get(n, (0,))[0]:14}  {body.get(n, (0, ''))[1]}")
    print("\nours, where the largest bodies stood:")
    for n, (v, where) in sorted(ours_body.items(), key=lambda kv: -kv[1][0])[:8]:
        print(f"  {n:12} {v:3}  {where}")


if __name__ == "__main__":
    main()
