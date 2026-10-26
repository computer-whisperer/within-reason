#!/usr/bin/env python3
"""One team's builders in the opening of a match record (docs/harness/record-format.md): every building begun before
--to with who began it, where, how far from the start and from that builder's last site, and the metal income then;
with --targets, the per-second table of what each builder (the commander, the factory, the constructors in birth
order) is building, from the record's `bp` rows (replay records from 2026-10-13 on), with both stocks. Read against
the territory bot's order log (bots/territory/README.md, T-OPENING) to place the early divergences: thebluegecko's
commander guarding one constructor, walking to two spots 1,007 and 1,215 out at 1:40 and back guarding at 2:57 came
out of this (terr90-93, 2026-10-25).

    run/record_builders.py <record-N.jsonl> [--to 300] [--targets]
"""
import collections, json, math, sys

args = sys.argv[1:]
rec = args[0]
to = float(args[args.index("--to") + 1]) if "--to" in args else 300.0
header = None
created, finished, samples = [], {}, []
for line in open(rec, errors="replace"):
    try:
        r = json.loads(line)
    except ValueError:
        continue
    if r.get("format") == "within-reason-record":
        header = r
        continue
    if r.get("t") == "ev":
        if r["k"] == "created":
            created.append(r)
        elif r["k"] == "finished":
            finished[r["u"]] = r
    elif r.get("t") == "s" and r["f"] <= 30 * to:
        samples.append(r)
defs = header["unit_defs"]
name = [d["name"] for d in defs]
spots = [(s[0], s[1]) for s in header["metal_spots"]]
start = next((e["x"], e["z"]) for e in created if defs[e["d"]].get("class") == "commander")
# The builders labelled by type and birth order: com, cv1, cv2 ... and the factories by name.
label, counts, by_unit = {}, collections.Counter(), {e["u"]: e for e in created}
for e in sorted(created, key=lambda e: e["f"]):
    d = defs[e["d"]]
    if d.get("class") == "commander":
        label[e["u"]] = "com"
    elif d.get("class") == "factory":
        label[e["u"]] = name[e["d"]][3:]
    elif d.get("build_speed", 0) > 0 and d.get("speed", 0) > 0:
        short = name[e["d"]][3:]
        counts[short] += 1
        label[e["u"]] = f"{short}{counts[short]}"

if "--targets" in args:
    def target(t):
        if not t:
            return "-"
        e = by_unit.get(t)
        return (label.get(t) or name[e["d"]]) if e else f"u{t}"
    print("    s  metal income energy | builder:target ...")
    for s in samples:
        bp = {row[0]: row for row in s.get("bp", [])}
        parts = [f"{l}:{target(bp[u][2])}" for u, l in label.items() if u in bp and by_unit[u]["f"] <= s["f"]]
        print(f"{s['f'] / 30:5.0f} {s['m'][0]:6.0f} {s['m'][1]:6.1f} {s['e'][0]:6.0f} | " + " ".join(parts))
    sys.exit(0)

income = {s["f"]: s["m"][1] for s in samples}
print(f"start {start}; spots by distance:", ", ".join(f"{math.dist(start, s):.0f}" for s in sorted(spots, key=lambda s: math.dist(start, s))[:12]))
print(f"{'begun':>6} {'done':>6} {'what':<10} {'by':<8} {'site':<13} {'from start':>10} {'from prev':>9}  income")
last_site = {}
for e in sorted(created, key=lambda e: e["f"]):
    if e["f"] / 30 > to:
        break
    d = defs[e["d"]]
    if d.get("class") == "commander" or (d.get("speed", 0) > 0 and d.get("class") != "builder"):
        continue
    by = e.get("by")
    site = (e["x"], e["z"])
    prev = last_site.get(by)
    fin = finished.get(e["u"], {}).get("f")
    inc = income.get((e["f"] // 30) * 30)
    mobile = d.get("speed", 0) > 0
    print(f"{e['f'] / 30:6.1f} {(fin / 30 if fin else float('nan')):6.1f} {name[e['d']]:<10} {label.get(by, f'u{by}'):<8} "
          f"{'' if mobile else str(site):<13} {'' if mobile else f'{math.dist(start, site):10.0f}'} "
          f"{'' if mobile or prev is None else f'{math.dist(prev, site):9.0f}'}  {'' if inc is None else inc}")
    if not mobile:
        last_site[by] = site
