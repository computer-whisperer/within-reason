#!/usr/bin/env python3
"""The fire instrument read out of match records (docs/harness/record-format.md, the `s` line's `dealt`, `shots`,
`ff`, `xo`, `xi`; records from 2026-09-24 on).

    run/fire.py <match dir or record>... [--quiet 3] [--pairs 12]

For every soldier-second with an enemy inside the soldier's reach (the header's `reach` plus the micro's 20 of slack):
did it shoot? A soldier in reach for `--quiet` seconds, or twice its reload, whichever is longer, without a shot is
counted muzzled: the engine refuses a shot whose line crosses a friend (`avoidFriendly`), and a unit with its target in
range does not step aside. The share is given by how many of our soldiers stand within 120 of the muzzled one, since a
ball hides its own rear ranks, and each muzzled second is sorted by its line to the nearest enemy in reach: a friend
within a hull's width (`HULL`) of that line, the enemy at the edge of the reach (the last `EDGE` elmos, where the
engine's own range test and ours can disagree), or a clear line (terrain, turning, a target it will not shoot). Then friendly fire by shooter type, and the two exchange tables by type pair (our hits
on them, theirs on us, -1 a type never seen: fire from out of sight).
"""
import argparse, collections, json, math, os, sys

SLACK = 20.0     # the micro's FOCUS_SLACK: a target this far beyond the reach still counts as in it
BALL = 120.0     # friends within this of a soldier: its ball
HULL = 24.0      # a friend this close to the line of fire blocks it (a tank's collision radius is about 20)
EDGE = 40.0      # a target within this of the reach's edge: the range test may say out of reach


def near_segment(px, pz, ax, az, bx, bz):
    """Distance from P to the segment AB."""
    dx, dz = bx - ax, bz - az
    l2 = dx * dx + dz * dz
    if l2 <= 0.0:
        return math.hypot(px - ax, pz - az)
    t = max(0.0, min(1.0, ((px - ax) * dx + (pz - az) * dz) / l2))
    return math.hypot(px - (ax + t * dx), pz - (az + t * dz))


def record_of(path):
    if os.path.isfile(path):
        return path
    for cand in (os.path.join(path, "record-0.jsonl"), os.path.join(path, "00", "record-0.jsonl")):
        if os.path.isfile(cand):
            return cand
    return None


def bucket(n):
    return "alone" if n == 0 else "1-2" if n <= 2 else "3-5" if n <= 5 else "6+"


def read(path, quiet_floor):
    with open(path) as f:
        header = json.loads(f.readline())
        defs = header["unit_defs"]
        if "reach" not in defs[0]:
            return None
        out = {
            "reach_s": collections.Counter(), "shots": collections.Counter(), "muzzled_s": collections.Counter(),
            "ball": collections.Counter(), "ball_reach": collections.Counter(), "why": collections.Counter(),
            "ff": collections.Counter(), "dealt": 0.0, "xo": collections.Counter(), "xi": collections.Counter(),
            "seconds": 0, "instrumented": False,
        }
        quiet = collections.Counter()
        for line in f:
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                continue
            if r.get("t") != "s":
                continue
            if "shots" not in r:
                continue
            out["instrumented"] = True
            out["seconds"] += 1
            shots = {u: n for u, n in r["shots"]}
            for u, d in r["dealt"]:
                out["dealt"] += d
            for u, d in r["ff"]:
                out["ff"][u] += d  # by unit id; typed below when the unit is in `own`
            for a, b, d in r["xo"]:
                out["xo"][(a, b)] += d
            for a, b, d in r["xi"]:
                out["xi"][(a, b)] += d
            own = r["own"]
            soldiers = [(u, d, x, z) for u, d, x, z, hp, fl in own if defs[d]["class"] == "army" and defs[d]["reach"] > 0 and not fl & 1]
            enemies = [(x, z) for e, d, x, z, hp in r["en"]]
            for u, d, x, z in soldiers:
                reach = defs[d]["reach"] + SLACK
                in_reach = [(ex, ez) for ex, ez in enemies if (x - ex) ** 2 + (z - ez) ** 2 < reach * reach]
                if not in_reach:
                    quiet[u] = 0
                    continue
                ex, ez = min(in_reach, key=lambda e: (x - e[0]) ** 2 + (z - e[1]) ** 2)
                name = defs[d]["name"]
                out["reach_s"][name] += 1
                out["shots"][name] += shots.get(u, 0)
                friends = sum(1 for v, _, fx, fz in soldiers if v != u and (x - fx) ** 2 + (z - fz) ** 2 < BALL * BALL)
                out["ball_reach"][bucket(friends)] += 1
                if shots.get(u, 0):
                    quiet[u] = 0
                    continue
                quiet[u] += 1
                if quiet[u] >= max(quiet_floor, 2 * defs[d]["reload"]):
                    out["muzzled_s"][name] += 1
                    out["ball"][bucket(friends)] += 1
                    blocked = any(v != u and near_segment(fx, fz, x, z, ex, ez) < HULL and (fx - x) ** 2 + (fz - z) ** 2 < (x - ex) ** 2 + (z - ez) ** 2 for v, _, fx, fz in soldiers)
                    edge = math.hypot(x - ex, z - ez) > reach - EDGE
                    out["why"]["a friend on the line" if blocked else "target at the reach's edge" if edge else "clear line"] += 1
        # friendly fire by type: the last type seen for the shooter id
        types = {}
        f.seek(0)
        f.readline()
        for line in f:
            if '"t":"s"' not in line[:12]:
                continue
            r = json.loads(line)
            for u, d, *_ in r["own"]:
                types[u] = defs[d]["name"]
        ff = collections.Counter()
        for u, d in out["ff"].items():
            ff[types.get(u, "?")] += d
        out["ff"] = ff
        out["defs"] = defs
        return out


def name(defs, i):
    return "unseen" if i < 0 else defs[i]["name"]


def report(label, r, pairs):
    print(f"== {label}: {r['seconds']} s sampled")
    total_reach = sum(r["reach_s"].values())
    total_muzzled = sum(r["muzzled_s"].values())
    print(f"soldier-seconds with an enemy in reach {total_reach}, shots {sum(r['shots'].values())}, muzzled {total_muzzled} ({100 * total_muzzled / max(1, total_reach):.0f}%)")
    print("  by type (in-reach s, shots, shots per in-reach s, muzzled s):")
    for t, s in r["reach_s"].most_common(10):
        print(f"    {t:10} {s:6} {r['shots'][t]:6} {r['shots'][t] / s:5.2f} {r['muzzled_s'][t]:6}")
    print("  muzzled share by friends within 120: " + ", ".join(f"{b} {100 * r['ball'][b] / max(1, r['ball_reach'][b]):.0f}% of {r['ball_reach'][b]} s" for b in ("alone", "1-2", "3-5", "6+")))
    print("  muzzled seconds by cause: " + ", ".join(f"{k} {n} ({100 * n / max(1, total_muzzled):.0f}%)" for k, n in r["why"].most_common()))
    ff = sum(r["ff"].values())
    print(f"friendly fire {ff:.0f} of {r['dealt']:.0f} dealt ({100 * ff / max(1.0, r['dealt'] + ff):.1f}%): " + ", ".join(f"{t} {d:.0f}" for t, d in r["ff"].most_common(6)))
    defs = r["defs"]
    print("  our hits on them (type -> type, damage): " + ", ".join(f"{name(defs, a)}->{name(defs, b)} {d:.0f}" for (a, b), d in r["xo"].most_common(pairs)))
    print("  their hits on us: " + ", ".join(f"{name(defs, a)}->{name(defs, b)} {d:.0f}" for (a, b), d in r["xi"].most_common(pairs)))
    unseen = sum(d for (a, b), d in r["xi"].items() if a < 0)
    print(f"  from out of sight: {unseen:.0f} of {sum(r['xi'].values()):.0f} taken ({100 * unseen / max(1.0, sum(r['xi'].values())):.0f}%)")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("paths", nargs="+")
    ap.add_argument("--quiet", type=float, default=3.0, help="seconds in reach without a shot before a soldier counts muzzled (floor; twice the reload when longer)")
    ap.add_argument("--pairs", type=int, default=12)
    a = ap.parse_args()
    for p in a.paths:
        rec = record_of(p)
        if rec is None:
            print(f"== {p}: no record", file=sys.stderr)
            continue
        r = read(rec, a.quiet)
        if r is None or not r["instrumented"]:
            print(f"== {p}: the record predates the fire instrument", file=sys.stderr)
            continue
        report(p, r, a.pairs)


if __name__ == "__main__":
    main()
