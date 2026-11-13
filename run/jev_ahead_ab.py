#!/usr/bin/env python3
"""Offline: H-HANDS-AHEAD's rung asked of a recorded game (the slots design §5.2, 6;
K-jev-a-detachment-ahead-of-a-party-rates-as-a-detachment-after-it). At each gate call that asked a `send_N_party_P`
of a group, the rung the build would offer beside it (`ahead_N_party_P`: the group's N soonest to the first place of
ours on the party's way that they reach before it) is rebuilt from the logs, put as a question in the build's
words with its forbidden mark where the send had one, and asked beside the recorded questions: once under the rules
as recorded, once with a rules sentence on `ahead` added after the `send` sentence of the call's rules (the long
rules or the lean gate's). Nothing is played. The key is read, never printed.

    run/jev_ahead_ab.py run/matches/<batch>/<NN> <from m:ss> <to m:ss> <party_N | any> [--every K] [--max N] [--dry]

The rung is rebuilt, not composed by the bot: a party's heading and pace from the movement of its centre between
two calls (the bot sums its members' velocities), our extractors and our soldiers' places from the record, the
party's words, the odds ("against them alone, ...") and the lender's clause from the recorded send of the same
size, what stands at the place from the recorded advance to it. `--every K` asks every K-th call that has a rung.
Prints each rung beside the send of its size (as recorded rules / with the sentence), then the means by what the
send said of catching the party and by the odds.
"""
import bisect, json, math, os, re, sys, time, urllib.request, urllib.error
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import jev_ab
from jev_ab import api_key, requests_of
from match_read import Match

def ask(key, state, questions, model):
    body = {"model": model, "state": state, "questions": questions}
    request = urllib.request.Request(jev_ab.URL, data=json.dumps(body).encode(), method="POST", headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json", "User-Agent": "jev-client"})
    for attempt in range(4):
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                return json.load(response)
        except urllib.error.HTTPError as e:
            if e.code in (429, 500, 502, 503) and attempt < 3:
                time.sleep(2 * (attempt + 1)); continue
            raise

d = sys.argv[1]; lo, hi = [int(a.split(":")[0]) * 60 + int(a.split(":")[1]) for a in sys.argv[2:4]]
target = sys.argv[4]; dry = "--dry" in sys.argv
# The sentence on `ahead`, after the `send` sentence of whichever rules the call was asked under: the long rules
# (`rules.md`) or the lean gate's (`rules_gate.md`, `--hands-effort lean`).
RULES = [
    ("a chaser slower than the raider still drives it off, without catching it.", " A raider heading for a place of ours is also met there: a detachment sent ahead of it (`ahead`), which its words say is at the place before the raider, stands there and follows nothing, and is the answer where a chase from behind arrives after the damage."),
    ("A raider is met by a small detachment (`send`), not the whole group.", " A raider heading for a place of ours is met there by a detachment sent ahead of it (`ahead`) when its words say it is there first."),
]
m = Match(d)
defs = None; mex = {}; dead = {}; udef = {}; samples = {}
for line in open(d + "/record-0.jsonl"):
    r = json.loads(line)
    if "unit_defs" in r: defs = r["unit_defs"]
    if r.get("t") == "ev" and r.get("k") == "finished":
        udef[r["u"]] = defs[r["d"]]
        if defs[r["d"]]["name"].endswith("mex"): mex[r["u"]] = r["f"]
    if r.get("t") == "ev" and r.get("k") == "destroyed": dead[r["u"]] = r["f"]
    if r.get("t") == "s": samples[r["f"]] = {u[0]: (u[2], u[3]) for u in r["own"]}
sf = sorted(samples)
def pos_at(f): return samples[sf[max(0, bisect.bisect_right(sf, f) - 1)]]
def secs_words(s): return "a few seconds" if s < 8 else f"about {max(10, round(s / 10) * 10):.0f} s"
key = None if dry else api_key()
prev = {}; rows = []; asked = 0
limit = int(sys.argv[sys.argv.index("--max") + 1]) if "--max" in sys.argv else 10**9
every = int(sys.argv[sys.argv.index("--every") + 1]) if "--every" in sys.argv else 1
hits = 0
for c, state, _ in requests_of(m):
    s = c["f"] // 30
    parties = {p["name"]: p for p in c.get("parties") or []}
    vel = {n: ((p["x"] - prev[n][0]) / max(1, c["f"] - prev[n][2]), (p["z"] - prev[n][1]) / max(1, c["f"] - prev[n][2])) for n, p in parties.items() if n in prev and c["f"] > prev[n][2]}
    for n, p in parties.items(): prev[n] = (p["x"], p["z"], c["f"])
    if not (lo <= s <= hi) or asked >= limit: continue
    qs = c["questions"]
    places = {p["name"]: (p["x"], p["z"]) for p in c.get("places") or []}
    now = pos_at(c["f"])
    new = {}; meta = {}
    for sk in [k for k in qs if re.fullmatch(r"group_\w+?\.send_\d+_party_\d+", k)]:
        gname, n, tgt = re.fullmatch(r"group_(\w+?)\.send_(\d+)_(party_\d+)", sk).groups(); n = int(n)
        if target != "any" and tgt != target: continue
        if tgt not in vel or tgt not in parties or gname.startswith("answer_"): continue
        g = next((g for g in c.get("groups") or [] if g["name"] == gname), None)
        if not g or any(a["name"] == f"answer_{tgt}" for a in c["groups"]): continue
        units = [(u, now[u], udef[u]) for u in g["members"] if u in now and u in udef and udef[u].get("speed", 0) > 0 and udef[u].get("weapon_count", 1) > 0]
        lent = sum(len(a["members"]) for a in c["groups"] if a["name"].startswith("answer_"))
        if n * 2 + lent > len(units): continue
        p = parties[tgt]; vx, vz = vel[tgt]; pace = math.hypot(vx, vz)
        if pace * 30 < 9: continue
        ux, uz = vx / pace, vz / pace
        way = {}
        for u, born in mex.items():
            if born > c["f"] or dead.get(u, 10**9) <= c["f"] or u not in now: continue
            dx, dz = now[u][0] - p["x"], now[u][1] - p["z"]
            along, across = dx * ux + dz * uz, abs(dx * uz - dz * ux)
            if along > 200 and across < along * 0.6 + 200 and along / (pace * 30) <= 60:
                name, dist = min(((pn, math.hypot(now[u][0] - px, now[u][1] - pz)) for pn, (px, pz) in places.items()), key=lambda t: t[1])
                if dist < 400: way[name] = min(way.get(name, 1e9), along / (pace * 30))
        for place, his in sorted(way.items(), key=lambda t: t[1]):
            px, pz = places[place]
            goers = sorted(((math.hypot(x - px, z - pz) / dfn["speed"], math.hypot(x - px, z - pz), dfn) for u, (x, z), dfn in units), key=lambda t: t[0])[:n]
            if len(goers) < n: continue
            last = max(t[1] for t in goers) / min(t[2]["speed"] for t in goers)
            if last + 3 >= his: continue
            kinds = {}
            for t in goers: kinds[t[2]["name"]] = kinds.get(t[2]["name"], 0) + 1
            who = ", ".join(f"{k} {name}" for name, k in sorted(kinds.items()))
            text = qs[sk]["instructions"]
            head, move = text.split("The move: ", 1)
            pw = re.search(rf"answer ({tgt} \(.*\)), the first there in", move)
            odds = re.search(r"against them alone, ([^;]*);", move)
            keeps = f"the rest of group_{gname} keeps its course"
            tail = move.split(keeps, 1)[1] if keeps in move else "."
            ft = qs.get(f"group_{gname}.fight_to_{place}") or qs.get(f"group_{gname}.go_{place}")
            end = ""
            if ft:
                e = re.search(r"(; at its end: .*?)(?:; leaves |; abandons |; stepping back|; closing on|; staying as near|\.$)", ft["instructions"])
                end = e.group(1) if e else ""
                end = re.sub(rf"; {tgt} \(.*?\) stands at it: [^;]*", "", end)
            if not pw or not odds: continue
            words = f"{who} of group_{gname} (its soonest to {place}) go ahead of {pw.group(1)} to {place}, which is on its way: the last of them is there in {last:.0f} s and the party in {his:.0f} s at its pace, so they are there first; they advance to it, stand there, shoot what comes in reach and follow nothing: against them alone, {odds.group(1)}{end}; they stay of no group of their own and go back when the party is dead or out of sight; {keeps}{tail}"
            k = f"group_{gname}.ahead_{n}_{tgt}"
            new[k] = {**qs[sk], "instructions": head + "The move: " + words}
            fq = qs.get(sk + ".forbidden")
            if fq: new[k + ".forbidden"] = {**fq, "instructions": fq["instructions"].split("The move: ", 1)[0] + "The move: " + words}
            meta[k] = (place, last, his, who, sk, odds.group(1), move)
            break
    if not new: continue
    hits += 1
    if hits % every: continue
    rec = lambda k: (c["answers"].get(k) or {}).get("noul")
    if dry:
        print(f"{s//60}:{s%60:02d}")
        for k, q in new.items():
            if not k.endswith(".forbidden"): print("   ", k, meta[k][:3], "\n      ", q["instructions"], "\n       SEND:", meta[k][6])
        continue
    asked += 1
    with_rule = dict(state)
    after = next(((a, r) for a, r in RULES if a in state["rules"]), None)
    if after is None: print("the rules have no send sentence to follow: the second ask is the first again", file=sys.stderr)
    else: with_rule["rules"] = state["rules"].replace(after[0], after[0] + after[1])
    a = ask(key, state, {**qs, **new}, c.get("model", "jev-1.14.0"))["answers"]
    b = ask(key, with_rule, {**qs, **new}, c.get("model", "jev-1.14.0"))["answers"]
    for k in sorted(k for k in new if not k.endswith(".forbidden")):
        place, last, his, who, sk, odds, move = meta[k]
        catch = "outruns" if "it outruns them" in move else "catch" if "they can catch it" in move else "unknown"
        fa = a.get(k + ".forbidden", {}).get("noul"); fb = b.get(k + ".forbidden", {}).get("noul")
        print(f"{s//60}:{s%60:02d} {k} ->{place} ({last:.0f}s vs {his:.0f}s; {odds}; the send: {catch}): ahead {a[k]['noul']:.2f}/{b[k]['noul']:.2f}  forbidden {fa if fa is None else round(fa, 2)}/{fb if fb is None else round(fb, 2)} | send rec {rec(sk)} now {a[sk]['noul']:.2f}/{b[sk]['noul']:.2f}", flush=True)
        rows.append((s, k, odds, catch, a[k]["noul"], b[k]["noul"], fa, fb, a[sk]["noul"], b[sk]["noul"]))
if rows:
    mean = lambda v: sum(v) / max(1, len(v))
    def show(label, rs):
        if not rs: return
        print(f"{label}: n {len(rs)}  ahead {mean([r[4] for r in rs]):.3f} / with the rule {mean([r[5] for r in rs]):.3f}; the send of that size {mean([r[8] for r in rs]):.3f} / {mean([r[9] for r in rs]):.3f}; ahead at 0.5 or over {sum(r[4] >= 0.5 for r in rs)} / {sum(r[5] >= 0.5 for r in rs)}, send {sum(r[8] >= 0.5 for r in rs)} / {sum(r[9] >= 0.5 for r in rs)}; ahead over the send by 0.05 or more {sum(r[4] - r[8] >= 0.05 for r in rs)} / {sum(r[5] - r[9] >= 0.05 for r in rs)}, under it by as much {sum(r[8] - r[4] >= 0.05 for r in rs)} / {sum(r[9] - r[5] >= 0.05 for r in rs)}")
    print()
    show("all", rows)
    for catch in ("outruns", "catch", "unknown"): show(f"  the send says: {catch}", [r for r in rows if r[3] == catch])
    for word in ("we outweigh", "even", "it outweighs"): show(f"  odds: {word}", [r for r in rows if word in r[2]])
    fz = [r for r in rows if r[6] is not None]
    if fz: print(f"  forbidden mark asked on {len(fz)}: {mean([r[6] for r in fz]):.3f} / {mean([r[7] for r in fz]):.3f}")
