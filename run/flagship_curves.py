#!/usr/bin/env python3
"""The flagship's economy curves from its records, the yardstick for a bot of bots/ (the user, 2026-10-20: "focus on
replicating how the main within reason expands"): per minute over the matches given, the range and median of
finished extractors (and frames), constructors, plants, nanos, energy buildings, turrets, soldiers alive, metal
store / income / usage and energy store / income / usage; then per match the milestones (plants, nanos, constructors,
the first advanced solar), production and soldier losses by minute, and the extractors finished by clock with their
distance from home. A bot batch's own curves at the same minutes: run/bot_curves.py.

    run/flagship_curves.py run/matches/<batch>/00 [...]

The twelve Comet wins read on 2026-10-20 (docs/design/2026-10-20-distilled-army.md, the economy section):
barb-ladder-0-1, barb-hard-1, barb-hard-10-1, barb-hard-20-1, barb-ladder-10-2, barb-ladder-10-3, barb-ladder-35-1,
sight-regress-1, apex-v029-comet-sym0-opus-1, spending-player-1, predict-player-1, chase-player-2.
"""
import glob, json, math, statistics, sys

MIN = [2, 3, 4, 5, 6, 8, 10, 12, 15]

def read(match):
    rec = sorted(glob.glob(f"{match}/record-*.jsonl"))[0]
    header = None; samples = []; finished = []
    for line in open(rec, errors="replace"):
        try: r = json.loads(line)
        except ValueError: continue
        t = r.get("t")
        if t == "header" and header is None: header = r
        elif t == "s": samples.append(r)
        elif t == "ev" and r.get("k") == "finished": finished.append(r)
    defs = header["unit_defs"]
    # The record's keys are sorted, so an event row starts with "by", not "t".
    destroyed = [r for r in (json.loads(l) for l in open(rec, errors="replace") if '"k":"destroyed"' in l) if r.get("k") == "destroyed"]
    header["_destroyed"] = destroyed
    # Home: the commander's position in the first sample.
    cls0 = [d.get("class", "") for d in defs]
    home = None
    for u in samples[0]["own"]:
        if cls0[u[1]] == "commander": home = (u[2], u[3]); break
    return header, defs, samples, finished, home

def classify(d):
    n = d["name"]; c = d.get("class", "")
    if d.get("extracts_metal", 0) > 0 or n.endswith("mex") or n.endswith("moho"): return "mex"
    if n.endswith("nanotc"): return "nano"
    if n.endswith("solar") or n.endswith("advsol") or n.endswith("win"): return "energy"
    if c == "factory": return "plant"
    if c == "commander": return "com"
    if c == "builder": return "cons"
    if c == "turret": return "turret"
    if c == "army": return "army"
    return "other"

def curves(match):
    header, defs, samples, finished, home = read(match)
    cls = [classify(d) for d in defs]
    out = {}
    for m in MIN:
        f = m * 1800
        s = min(samples, key=lambda s: abs(s["f"] - f))
        if abs(s["f"] - f) > 90: continue
        cnt = {}; frames = 0
        for u in s["own"]:
            k = cls[u[1]]; built = u[5] & 1
            if built:
                if k == "mex": frames += 1
                continue
            cnt[k] = cnt.get(k, 0) + 1
        out[m] = dict(mex=cnt.get("mex", 0), mexf=frames, cons=cnt.get("cons", 0), plant=cnt.get("plant", 0), nano=cnt.get("nano", 0), energy=cnt.get("energy", 0), turret=cnt.get("turret", 0), army=cnt.get("army", 0), ms=s["m"][0], mi=s["m"][1], mu=s["m"][2], es=s["e"][0], ei=s["e"][1], eu=s["e"][2])
    # milestones from finished events
    ms = {}
    plants = sorted(e["f"] for e in finished if cls[e["d"]] == "plant")
    nanos = sorted(e["f"] for e in finished if cls[e["d"]] == "nano")
    cons = sorted(e["f"] for e in finished if cls[e["d"]] == "cons")
    advsol = sorted(e["f"] for e in finished if defs[e["d"]]["name"].endswith("advsol"))
    ms["plant1"] = plants[0] if plants else None; ms["plant2"] = plants[1] if len(plants) > 1 else None; ms["plant3"] = plants[2] if len(plants) > 2 else None
    ms["nano1"] = nanos[0] if nanos else None; ms["nano2"] = nanos[1] if len(nanos) > 1 else None
    ms["cons3"] = cons[2] if len(cons) > 2 else None; ms["cons6"] = cons[5] if len(cons) > 5 else None; ms["cons10"] = cons[9] if len(cons) > 9 else None
    ms["advsol1"] = advsol[0] if advsol else None
    # the extractors' order: distance from home of each finished extractor by clock
    mexes = [(e["f"], math.hypot(e["x"] - home[0], e["z"] - home[1]) if home else None) for e in finished if cls[e["d"]] == "mex"]
    # Production and losses by minute: soldiers (army class, scouts included) and constructors finished, soldiers destroyed.
    prod = {}
    for m in [5, 8, 10, 12, 15]:
        f = m * 1800
        soldiers = sum(1 for e in finished if e["f"] <= f and cls[e["d"]] == "army")
        scouts = sum(1 for e in finished if e["f"] <= f and defs[e["d"]]["name"].endswith("fav"))
        cons = sum(1 for e in finished if e["f"] <= f and cls[e["d"]] == "cons")
        lost = sum(1 for e in header["_destroyed"] if e["f"] <= f and cls[e["d"]] == "army")
        prod[m] = (soldiers, cons, scouts, lost)
    return out, ms, mexes, prod

def clock(f):
    return "-" if f is None else f"{f // 1800}:{(f // 30) % 60:02d}"

def main():
    matches = sys.argv[1:]
    rows = {}; mss = []; mexes_all = []; prods = []
    for m in matches:
        try:
            c, ms, mexes, prod = curves(m)
        except Exception as e:
            print("skip", m, e); continue
        for k, v in c.items(): rows.setdefault(k, []).append(v)
        mss.append((m.split('/')[-2][11:], ms)); mexes_all.append((m.split('/')[-2][11:], mexes)); prods.append((m.split('/')[-2][11:], prod))
    keys = ["mex", "mexf", "cons", "plant", "nano", "energy", "turret", "army", "ms", "mi", "mu", "es", "ei", "eu"]
    print(f"{'min':>3} n " + " ".join(f"{k:>13}" for k in keys))
    for m in MIN:
        rs = rows.get(m, [])
        if not rs: continue
        cells = []
        for k in keys:
            vals = [r[k] for r in rs]
            cells.append(f"{min(vals):.0f}-{max(vals):.0f} ({statistics.median(vals):.0f})")
        print(f"{m:>3} {len(rs)} " + " ".join(f"{c:>13}" for c in cells))
    print("made by 5/8/10/12/15 (soldiers+scouts / constructors) and soldiers lost by then:")
    for name, prod in prods:
        print(f"  {name:<36} " + " ".join(f"{prod[m][0]}+{prod[m][1]}/{prod[m][2]} lost {prod[m][3]}" for m in [5, 8, 10, 12, 15]))
    print("milestones (plant1 plant2 plant3 nano1 nano2 cons3 cons6 cons10 advsol1):")
    for name, ms in mss:
        print(f"  {name:<36} " + " ".join(f"{clock(ms[k]):>6}" for k in ["plant1", "plant2", "plant3", "nano1", "nano2", "cons3", "cons6", "cons10", "advsol1"]))
    print("extractors finished by clock (distance from home): first 24 of each game")
    for name, mexes in mexes_all:
        print(f"  {name:<36} " + " ".join(f"{clock(f)}@{d:.0f}" if d is not None else clock(f) for f, d in mexes[:24]))

main()
