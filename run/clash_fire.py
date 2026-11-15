#!/usr/bin/env python3
"""One clash of a formation group by who fired, from the match record (`docs/harness/record-format.md`, the `s` line's
`groups`, `own`, `en`, `dealt`, `dmg`, `xo`, `xi`). Every second: the group's soldiers by type, how many of each dealt
damage, the damage dealt and taken, how far the group's middle and his tanks' middle moved and the gap between them,
the nearest and the median soldier's distance to his nearest, and how many had one of his within their reach. Then
the totals: the share of soldier-seconds that dealt damage, that had one in reach and that stood inside his reach,
the soldiers that never dealt any, and the hits by type pair.

    run/clash_fire.py <match dir> <from m:ss> <to m:ss> [--group group_A] [--his armjanus,armstump,armflash] [--quiet]

Read it beside `run/scenario_soldiers.py` on the same clash cut and fought again (`run/engagement.py --track`,
`duel --scenario`): a share of one soldier-second in ten dealing damage is what a fight of two reaches against a small
party looks like under any orders (loop-pianist-smoke-2's clash: 10 % live, 9 % in formation on the bench, 14 %
under the engine's own fight order), so it is not by itself a fault.
"""
import argparse, glob, json, math, statistics


def clock(text):
    minutes, seconds = text.split(":")
    return (int(minutes) * 60 + int(seconds)) * 30


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("match")
    ap.add_argument("start")
    ap.add_argument("end")
    ap.add_argument("--group", default="group_A")
    ap.add_argument("--his", default="armjanus,armstump,armflash")
    ap.add_argument("--quiet", action="store_true", help="the totals alone")
    args = ap.parse_args()
    lo, hi = clock(args.start), clock(args.end)
    his_types = set(args.his.split(","))
    defs = None
    prev_c = prev_e = None
    dealt_by, taken_by, ever, seen, ours_on, his_on = {}, {}, set(), {}, {}, {}
    seconds = soldier_s = fire_s = reach_s = under_s = 0
    for line in open(glob.glob(args.match.rstrip("/") + "/record-*.jsonl")[0]):
        r = json.loads(line)
        if "unit_defs" in r:
            defs = r["unit_defs"]
            continue
        f = r.get("f", -1)
        if r.get("t") != "s" or not (lo <= f <= hi):
            continue
        g = (r.get("groups") or {}).get(args.group)
        if not g:
            continue
        name = lambda i: defs[i]["name"] if i >= 0 else "unseen"
        own = {u[0]: u for u in r["own"]}
        mine = [own[u] for u in list(g["formed"]) + [e[0] for e in g["exceptions"]] if u in own]
        if not mine:
            continue
        c = (statistics.median(m[2] for m in mine), statistics.median(m[3] for m in mine))
        en = [e for e in r.get("en", []) if math.dist((e[2], e[3]), c) < 1100 and name(e[1]) in his_types]
        dealt = dict((u, v) for u, v in r.get("dealt", []))
        taken = dict((u, v) for u, v in r.get("dmg", []))
        by = {}
        for m in mine:
            n = name(m[1])
            seen[m[0]] = n
            b = by.setdefault(n, [0, 0, 0.0, 0.0])
            b[0] += 1
            if dealt.get(m[0], 0) > 0:
                b[1] += 1
                ever.add(m[0])
            b[2] += dealt.get(m[0], 0)
            b[3] += taken.get(m[0], 0)
            dealt_by[n] = dealt_by.get(n, 0) + dealt.get(m[0], 0)
            taken_by[n] = taken_by.get(n, 0) + taken.get(m[0], 0)
        for a, b, v in r.get("xo", []):
            ours_on[(name(a), name(b))] = ours_on.get((name(a), name(b)), 0) + v
        for a, b, v in r.get("xi", []):
            his_on[(name(a), name(b))] = his_on.get((name(a), name(b)), 0) + v
        seconds += 1
        soldier_s += len(mine)
        fire_s += sum(1 for m in mine if dealt.get(m[0], 0) > 0)
        s = f // 30
        if en:
            e = (statistics.median(x[2] for x in en), statistics.median(x[3] for x in en))
            near = sorted(min(math.dist((m[2], m[3]), (x[2], x[3])) for x in en) for m in mine)
            in_reach = sum(1 for m in mine if any(math.dist((m[2], m[3]), (x[2], x[3])) <= defs[m[1]]["reach"] for x in en))
            reach_s += in_reach
            under_s += sum(1 for m in mine if any(math.dist((m[2], m[3]), (x[2], x[3])) <= defs[x[1]]["reach"] for x in en))
            kinds = {}
            for x in en:
                kinds[name(x[1])[3:]] = kinds.get(name(x[1])[3:], 0) + 1
            moved_c = math.dist(c, prev_c) if prev_c else 0
            moved_e = math.dist(e, prev_e) if prev_e else 0
            prev_e = e
            text = f"his {kinds} gap {math.dist(c, e):4.0f} (we moved {moved_c:3.0f}, he moved {moved_e:3.0f}); nearest {near[0]:3.0f} median {near[len(near) // 2]:3.0f}; in reach {in_reach}"
        else:
            prev_e = None
            text = "none of his in sight"
        prev_c = c
        if not args.quiet:
            row = " ".join(f"{n[3:]} {b[0]}:{b[1]} fired, dealt {b[2]:4.0f} took {b[3]:4.0f};" for n, b in sorted(by.items()))
            print(f"{s // 60}:{s % 60:02d} {row} | {text}")
    if not seconds:
        print("no second of the group in the window")
        return
    print(f"\n{seconds} s. dealt by type {dict((k, round(v)) for k, v in dealt_by.items())}; taken by type {dict((k, round(v)) for k, v in taken_by.items())}")
    print(f"soldier-seconds {soldier_s}: dealing damage {fire_s} ({100 * fire_s / soldier_s:.0f} %), with one of his in reach {reach_s} ({100 * reach_s / soldier_s:.0f} %), inside the reach of one of his {under_s} ({100 * under_s / soldier_s:.0f} %)")
    kinds = {}
    for u, n in seen.items():
        k = kinds.setdefault(n, [0, 0])
        k[0] += 1
        k[1] += u in ever
    print("soldiers in the group at some second, and how many ever dealt damage:", kinds)
    print("our hits by pair:", dict((k, round(v)) for k, v in sorted(ours_on.items(), key=lambda kv: -kv[1])[:8]))
    print("his hits by pair:", dict((k, round(v)) for k, v in sorted(his_on.items(), key=lambda kv: -kv[1])[:8]))


if __name__ == "__main__":
    main()
