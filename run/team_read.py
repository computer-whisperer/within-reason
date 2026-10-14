#!/usr/bin/env python3
"""Our side of a many-seat game against the truth file, then the model bill: every N minutes the extractors, builders,
armed mobile units and commanders of ours summed over the seats (and each seat's extractors/soldiers), his from
truth-0 (every team of his; his builders and Lazarus are not army), each seat's end state, and for every
strategist-N.jsonl and commander.jsonl the sum of per-session maxima of total_cost_usd with the turns and tool calls.
Written for the 8v8 games against BARb (glitters-8v8-*, 2026-10-09).

    run/team_read.py run/matches/<batch> [step_minutes]
"""
import glob
import os
import sys
from collections import Counter

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import match_read  # noqa: E402
import t2_frames  # noqa: E402

BUILDERS = {"armck", "armcv", "armack", "armacv", "armca", "armaca", "armrectr", "armcom", "armfark", "armconsul", "armbeaver", "armmlv", "armch", "armcs", "armacsub",
            "corck", "corcv", "corack", "coracv", "corca", "coraca", "cornecro", "corfast", "corcom", "cormuskrat", "cormlv", "corch", "corcs", "coracsub"}
# His buildings, for the truth file (names only; the truth carries no def table).
HIS_BUILDINGS = {"cormex", "cormoho", "corsolar", "coradvsol", "corwin", "corllt", "corhllt", "corhlt", "corrl", "corrad", "cornanotc", "cormakr", "cormmkr", "corestor", "cormstor", "corfus", "corafus",
                 "corvp", "coravp", "corlab", "coralab", "corap", "coraap", "corgant", "corfmd", "corsilo", "corflak", "corpun", "cortoast", "corerad", "corjamt", "corsy", "corasy", "coruwms", "coruwes",
                 "corgate", "cordrag", "cormaw", "corshroud", "corhp", "corplat", "corexp", "corgeo", "coradvestore", "coradvmstore", "cortarg", "corsd", "corarad", "cormexp", "coruwadves", "coruwadvms",
                 "corjuno", "cordoom", "corint", "corbhmth", "corvipe", "cormadsam", "corscreamer",
                 "armmex", "armmoho", "armsolar", "armadvsol", "armwin", "armllt", "armhlt", "armrl", "armrad", "armnanotc", "armmakr", "armmmkr", "armestor", "armmstor", "armfus", "armafus",
                 "armvp", "armavp", "armlab", "armalab", "armap", "armaap", "armgant", "armamd", "armsilo", "armflak", "armguard", "armbeamer", "armjamt", "armsy", "armasy", "armhp", "armgeo", "armtarg", "armsd", "armarad", "armmexp",
                 "legmex", "legmoho", "legsolar", "legadvsol", "legwin", "leglht", "legmg", "legdtr", "legrl", "legrhapsis", "leglupara", "legrad", "legarad", "legnanotc", "legeconv", "legadveconv", "legestor", "legmstor", "legamstor", "legadvestore",
                 "legfus", "legafus", "leggeo", "legageo", "legvp", "legavp", "leglab", "legalab", "legap", "legaap", "leghp", "legsy", "legadvshipyard", "leggant", "legabm", "legsilo", "legflak", "leglraa", "legcluster", "legacluster", "legapopupdef", "legbastion",
                 "legperdition", "leglrpc", "legstarfall", "legjam", "legajam", "legeyes", "legjuno", "legtarg", "legsd", "leghive", "legdrag", "legforti", "legdeflector", "legrampart", "legmohocon", "legtide", "legtl", "legctl", "legfrad"}


def tally(defs, own):
    c = Counter(defs[o[1]]["name"] for o in own)
    mex = sum(v for k, v in c.items() if k.endswith("mex") or k.endswith("moho"))
    builders = sum(v for k, v in c.items() if k in BUILDERS)
    army = sum(1 for o in own if defs[o[1]].get("speed", 0) > 0 and defs[o[1]].get("weapons") and defs[o[1]]["name"] not in BUILDERS)
    return mex, builders, army, c["armcom"] + c["corcom"], len(own)


def his_at(truth, f):
    rows = [r for r in truth if r["f"] <= f]
    if not rows:
        return None
    c = Counter(e[1] for e in max(rows, key=lambda r: r["f"])["enemy"] if len(e) <= 5 or not e[5])
    mex = sum(v for k, v in c.items() if k.endswith("mex") or k.endswith("moho"))
    army = sum(v for k, v in c.items() if k not in HIS_BUILDINGS and k not in BUILDERS)
    return mex, army, c["corcom"] + c["armcom"], sum(c.values())


def seats_table(d, step):
    seats = []
    for p in sorted(glob.glob(d + "/record-*.jsonl")):
        defs, _, _, samples = t2_frames.frames_of(p)
        seats.append((os.path.basename(p), defs, samples))
    truth = list(match_read.rows(match_read.first(d, "truth-0")))
    last_f = max(s[2][-1]["f"] for s in seats if s[2])
    print(f"last sample {match_read.clock(last_f)}; seats {len(seats)}; truth rows {len(truth)} to {match_read.clock(truth[-1]['f']) if truth else '-'}")
    print("clock | ours: extractors builders army commanders units (per seat: mex/army) | his: extractors army commanders units")
    for minute in range(step, int(last_f / 1800) + 1, step):
        f = minute * 1800
        tot = [0, 0, 0, 0, 0]
        per = []
        for _, defs, samples in seats:
            s = max((s for s in samples if s["f"] <= f), key=lambda s: s["f"], default=None)
            t = tally(defs, s.get("own", [])) if s else (0, 0, 0, 0, 0)
            tot = [a + b for a, b in zip(tot, t)]
            per.append(f"{t[0]}/{t[2]}")
        h = his_at(truth, f)
        his = f"{h[0]:3d} {h[1]:4d} {h[2]} {h[3]:4d}" if h else "-"
        print(f"{minute:2d}:00 | {tot[0]:3d} {tot[1]:3d} {tot[2]:4d} {tot[3]} {tot[4]:4d} ({' '.join(per)}) | {his}")
    print("end state per seat (last sample): mex builders army commander units")
    for name, defs, samples in seats:
        if samples:
            t = tally(defs, samples[-1].get("own", []))
            print(f"  {name} {match_read.clock(samples[-1]['f'])}: {t[0]} {t[1]} {t[2]} {'alive' if t[3] else 'DEAD'} {t[4]}")


def bill(d):
    total = 0.0
    for p in sorted(glob.glob(d + "/strategist-*.jsonl*") + glob.glob(d + "/commander.jsonl*")):
        if "stderr" in p:
            continue
        cost = prev = 0.0
        turns = 0
        tools = Counter()
        for r in match_read.rows(p):
            k = r.get("kind")
            if k == "turn":
                turns += 1
            elif k == "tool_call":
                tools[r.get("tool")] += 1
            elif k == "result":
                c = (r.get("message") or {}).get("total_cost_usd") or 0.0
                if c < prev:
                    cost += prev
                prev = c
        cost += prev
        total += cost
        print(f"{os.path.basename(p):22s} ${cost:6.2f} nominal, {turns:4d} turns; tools {dict(tools.most_common(8))}")
    print(f"total ${total:.2f} nominal (the subscription's price for these sessions, not money spent)")


def main():
    d = sys.argv[1].rstrip("/")
    step = int(sys.argv[2]) if len(sys.argv) > 2 else 4
    if os.path.isdir(d + "/00"):
        d += "/00"
    seats_table(d, step)
    print()
    bill(d)


if __name__ == "__main__":
    main()
