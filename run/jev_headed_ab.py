#!/usr/bin/env python3
"""Offline: H-HANDS-HEADED-FOR asked of a recorded window (the slots design §5.2, 5;
K-jev-the-advance-that-stood-between-read-as-a-walk-back). The gate's recorded questions about group_A's walks and
advances are asked again as recorded and with the line "our extractor there is on the way of party_N (...), about
T s ahead of it at its pace: group_A gets there first / about when it does / after it" on the places the build
would say it of. Nothing is played. The key is read, never printed.

    run/jev_headed_ab.py run/matches/<batch>/<NN> <from m:ss> <to m:ss> [--dry]

The line is rebuilt from the logs, not by the bot: a party's heading and pace from the movement of its centre
between two calls (the bot sums its members' velocities), our extractors from the record, the group's time from
its place over a slowest speed of 54 (the Janus of the game this was written for: change `SLOWEST` for another).
Prints each call's tagged moves (recorded, asked again, with the line) and the means by what the line said.
"""
import json, math, os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from match_read import Match

d = sys.argv[1]; lo, hi = [int(a.split(":")[0]) * 60 + int(a.split(":")[1]) for a in sys.argv[2:4]]
dry = "--dry" in sys.argv
m = Match(d)
# our extractors: id -> (x, z), alive by frame
defs = None; mex = {}; dead = {}; pos = {}
for line in open(d + "/record-0.jsonl"):
    r = json.loads(line)
    if "unit_defs" in r: defs = r["unit_defs"]
    if r.get("t") == "ev" and r.get("k") == "finished" and defs[r["d"]]["name"].endswith("mex"): mex[r["u"]] = r["f"]
    if r.get("t") == "ev" and r.get("k") == "destroyed": dead[r["u"]] = r["f"]
    if r.get("t") == "s":
        for u in r["own"]:
            if u[0] in mex: pos[u[0]] = (u[2], u[3])
SLOWEST = 54.0
key = None if dry else api_key()
model = "jev-1.14.0"
prev = {}
rows = []
for c, state, _ in requests_of(m):
    s = c["f"] // 30
    parties = {p["name"]: p for p in c.get("parties") or []}
    vel = {n: ((p["x"] - prev[n][0]) / max(1, c["f"] - prev[n][2]), (p["z"] - prev[n][1]) / max(1, c["f"] - prev[n][2])) for n, p in parties.items() if n in prev and c["f"] > prev[n][2]}
    for n, p in parties.items(): prev[n] = (p["x"], p["z"], c["f"])
    if not (lo <= s <= hi): continue
    qs = {k: q for k, q in c["questions"].items() if re.fullmatch(r"group_A\.(go|fight_to)_\w+", k)}
    if not qs: continue
    g = next((g for g in c.get("groups") or [] if g["name"] == "A"), None)
    places = {p["name"]: (p["x"], p["z"]) for p in c.get("places") or []}
    if not g or not g.get("at"): continue
    # tags: place -> (party, seconds), soonest party first
    tags = {}
    for n, p in parties.items():
        if n not in vel: continue
        vx, vz = vel[n]; pace = math.hypot(vx, vz)           # elmos a frame, of the centre
        if pace * 30 < 9: continue
        ux, uz = vx / pace, vz / pace
        for u, born in mex.items():
            if born > c["f"] or dead.get(u, 10**9) <= c["f"] or u not in pos: continue
            dx, dz = pos[u][0] - p["x"], pos[u][1] - p["z"]
            along, across = dx * ux + dz * uz, abs(dx * uz - dz * ux)
            if along > 200 and across < along * 0.6 + 200 and along / (pace * 30) <= 60:
                name, dist = min(((pn, math.hypot(pos[u][0] - px, pos[u][1] - pz)) for pn, (px, pz) in places.items()), key=lambda t: t[1])
                if dist < 400:
                    secs = along / (pace * 30)
                    best = tags.get(name)
                    if best is None or (p.get("metal") or 0, -secs) > (best[3], -best[1]): tags[name] = (n, secs, p.get("composition"), p.get("metal") or 0)
    new = {}; said = {}
    for k, q in qs.items():
        place = re.fullmatch(r"group_A\.(?:go|fight_to)_(\w+)", k).group(1)
        text = q["instructions"]
        if place in tags and place in places:
            n, secs, comp, _ = tags[place]
            ours = math.hypot(places[place][0] - g["at"][0], places[place][1] - g["at"][1]) / SLOWEST
            first = "gets there first" if ours + 3 < secs else "gets there after it" if ours > secs + 3 else "gets there about when it does"
            sw = "a few seconds" if secs < 8 else f"about {max(10, round(secs / 10) * 10):.0f} s"
            clause = f"; our extractor there is on the way of {n} ({comp}), {sw} ahead of it at its pace: group_A {first}"
            cut = min([i for i in (text.find(x) for x in ("; stepping back from", "; closing on", "; staying as near to", "; leaves ", "; abandons ")) if i >= 0] or [len(text.rstrip(".")) ])
            text = text[:cut] + clause + text[cut:]
            said[k] = first
        new[k] = {**q, "instructions": text}
    tagged = [k for k in qs if new[k]["instructions"] != qs[k]["instructions"]]
    if not tagged: continue
    rec = {k: (c["answers"].get(k) or {}).get("noul") for k in qs}
    if dry:
        print(f"{s//60}:{s%60:02d}", {k: (rec[k], new[k]["instructions"][new[k]["instructions"].find('; our extractor'):][:150]) for k in tagged}); continue
    a = ask(key, state, {**c["questions"]}, c.get("model", model))["answers"]
    b = ask(key, state, {**c["questions"], **new}, c.get("model", model))["answers"]
    for k in sorted(qs):
        rows.append((s, k, k in tagged, rec[k], a[k]["noul"], b[k]["noul"], said.get(k)))
    print(f"{s//60}:{s%60:02d}  " + "  ".join(f"{k[8:]}{'*' if k in tagged else ''} rec {rec[k]} again {a[k]['noul']:.2f} new {b[k]['noul']:.2f}" for k in sorted(qs) if k in tagged or k.endswith('_home')), flush=True)
if rows:
    t = [r for r in rows if r[2]]; u = [r for r in rows if not r[2]]
    mean = lambda v: sum(v) / max(1, len(v))
    print(f"tagged moves: n {len(t)}  as recorded asked again {mean([r[4] for r in t]):.3f}  with the clause {mean([r[5] for r in t]):.3f}; over 0.5: {sum(r[4] >= 0.5 for r in t)} -> {sum(r[5] >= 0.5 for r in t)}")
    print(f"untagged moves in the same calls: n {len(u)}  {mean([r[4] for r in u]):.3f} -> {mean([r[5] for r in u]):.3f}")
    for first in ("gets there first", "gets there about when it does", "gets there after it"):
        v = [r for r in t if r[6] == first]
        print(f"  '{first}': n {len(v)}  {mean([r[4] for r in v]):.3f} -> {mean([r[5] for r in v]):.3f}; over 0.5: {sum(r[4] >= 0.5 for r in v)} -> {sum(r[5] >= 0.5 for r in v)}; rose by 0.05 or more {sum(r[5] - r[4] >= 0.05 for r in v)}, fell by 0.05 or more {sum(r[4] - r[5] >= 0.05 for r in v)}")
