#!/usr/bin/env python3
"""Where the soldiers with no group stand at given moments of a flagship game on the slots, by map cell, with how
far each moved in the 30 s before and its distance to group_A's place: the comers and the unassigned
(K-deal-a-comer-alone-is-held-at-the-edge-of-a-threat-its-body-passed). Armada's ground soldiers only.

usage: run/slots_comers.py run/matches/<batch>/<NN> <seconds of game> [...]
"""
import bisect, collections, json, subprocess, sys, math
ARMY = {"armfav", "armflash", "armstump", "armsam", "armjanus", "armart", "armpincer", "armbull", "armlatnk", "armmart", "armcroc", "armmanni", "armyork", "armseer", "armmerl", "armpw", "armrock", "armham", "armwar", "armjeth"}
d = sys.argv[1]; times = [int(x) for x in sys.argv[2:]]
text = subprocess.run(["zstd", "-dc", d + "/jev-0.jsonl.zst"], capture_output=True).stdout.decode()
calls = []
for line in text.splitlines():
    try: r = json.loads(line)
    except Exception: continue
    if r.get("t") == "call" and r.get("groups") is not None: calls.append(r)
cf = [c["f"] for c in calls]
defs = None; ud = {}; samples = {}
for line in open(d + "/record-0.jsonl"):
    r = json.loads(line)
    if "unit_defs" in r: defs = r["unit_defs"]
    if r.get("t") == "ev" and r.get("k") == "finished":
        dd = defs[r["d"]]; ud[r["u"]] = dd if isinstance(dd, str) else dd.get("name")
    if r.get("t") == "s": samples[r["f"]] = {u[0]: (u[2], u[3]) for u in r["own"] if ud.get(u[0]) in ARMY and not (u[5] & 1)}
sf = sorted(samples)
def cell(x, z): return "ABCDEFGH"[min(7, int(x // 1024))] + str(min(8, int(z // 1024) + 1))
for t in times:
    f = sf[bisect.bisect_left(sf, t * 30)]
    i = bisect.bisect_right(cf, f) - 1
    c = calls[i]
    mem = {u: g["name"] for g in c["groups"] for u in g["members"]}
    ga = next((g for g in c["groups"] if g["name"] == "A"), None)
    now = samples[f]; before = samples.get(sf[max(0, bisect.bisect_left(sf, f - 900))], {})
    print(f"\n== {t//60}:{t%60:02d} (call f={c['f']}) group A: {len(ga['members']) if ga else 0} with it at {ga['at'] if ga else None} {cell(*ga['at']) if ga and ga.get('at') else ''} task {ga['task'] if ga else None}"[:330])
    byg = collections.Counter(mem.get(u, "-") for u in now)
    print("   soldiers by group:", dict(byg))
    cells = collections.defaultdict(list)
    for u, (x, z) in now.items():
        if u in mem: continue
        b = before.get(u)
        moved = math.hypot(x - b[0], z - b[1]) if b else None
        dist = math.hypot(x - ga["at"][0], z - ga["at"][1]) if ga and ga.get("at") else 0
        cells[cell(x, z)].append((moved, dist, ud[u]))
    for k in sorted(cells):
        v = cells[k]; mv = [m for m, _, _ in v if m is not None]
        types = collections.Counter(t for _, _, t in v)
        print(f"   not with any group, at {k}: {len(v):3}  moved in the last 30 s: median {sorted(mv)[len(mv)//2] if mv else 0:5.0f}, under 100: {sum(m < 100 for m in mv):2}   to A's place: {min(x[1] for x in v):5.0f}-{max(x[1] for x in v):5.0f}  {dict(types)}")
