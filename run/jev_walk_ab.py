#!/usr/bin/env python3
"""Offline: a gate whose groups are asked each walk twice (`go_<place>`, without stopping to fight, and
`fight_to_<place>`, fighting on the way) asked again with one question a place, against the same gate asked again
as recorded (`merged` says both manners in the one question, `bare` says neither); the key is read, never printed.

    run/jev_walk_ab.py run/matches/<batch>/00 [gates] [out.jsonl]
"""
import json, random, re, sys, collections, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask, requests_of
from match_read import Match
D = sys.argv[1] if len(sys.argv) > 1 else "run/matches/1790957410-player-38-split/00"
N = int(sys.argv[2]) if len(sys.argv) > 2 else 40
OUT = sys.argv[3] if len(sys.argv) > 3 else "/dev/null"
m = Match(D)
key = api_key()
ARMS = ("recorded", "merged", "bare")
FIGHT = re.compile(r"^(.*\? The move: \S+) advances to (\S+ \(.*?\)), fighting on the way(.*)$", re.S)
def pairs_of(q):
    out = []
    for k in q:
        if ".fight_to_" not in k: continue
        actor, move = k.split(".", 1)
        go = f"{actor}.go_{move[len('fight_to_'):]}"
        if go in q and FIGHT.match(q[k]["instructions"]): out.append((go, k))
    return out
def nouls(answers):
    return {k: a.get("noul") for k, a in answers.items() if isinstance(a, dict) and a.get("noul") is not None}
gates = []
for c, state, packet in requests_of(m):
    q = c["questions"]
    if any(k.startswith("worlds.") for k in q): continue
    ps = pairs_of(q); a = nouls(c["answers"])
    if len(ps) >= 4 and any(max(a.get(g, 0), a.get(f, 0)) >= 0.5 for g, f in ps): gates.append((c, state))
random.seed(7)
random.shuffle(gates)
gates = gates[:N]
def arm(q, which):
    if which == "recorded": return q
    out = dict(q)
    for go, ft in pairs_of(q):
        mm = FIGHT.match(q[ft]["instructions"])
        del out[go], out[ft]
        manner = ", either fighting on the way or without stopping to fight (which of the two is asked once this is taken)" if which == "merged" else ""
        text = f"{mm.group(1)} goes to {mm.group(2)}{manner}{mm.group(3)}"
        out[ft.replace(".fight_to_", ".walk_")] = {**q[ft], "instructions": text}
    return out
def run(job):
    c, state, which = job
    q = arm(c["questions"], which)
    answers = {}; tokens = 0
    base = len(json.dumps(state)); chunk = {}; size = base; chunks = []
    for k, v in q.items():
        L = len(json.dumps(v)) + len(k) + 6
        if chunk and size + L > 140_000:
            chunks.append(chunk); chunk = {}; size = base
        chunk[k] = v; size += L
    if chunk: chunks.append(chunk)
    for part in chunks:
        r = ask(key, state, part, c.get("model", "jev-latest"))
        answers.update(r.get("answers") or {}); tokens += (r.get("usage") or {}).get("input_tokens", 0)
    return c["f"], which, nouls(answers), tokens
jobs = [(c, s, w) for c, s in gates for w in ARMS]
res = collections.defaultdict(dict); toks = collections.Counter()
with concurrent.futures.ThreadPoolExecutor(8) as ex:
    for f, which, ans, t in ex.map(run, jobs):
        res[f][which] = ans; toks[which] += t
rec = {c["f"]: c for c, _ in gates}
out = open(OUT, "w")
stats = {w: collections.Counter() for w in ARMS}
for f, arms in res.items():
    c = rec[f]; orig = nouls(c["answers"]); ps = pairs_of(c["questions"])
    out.write(json.dumps({"f": f, "orig": orig, **arms}) + "\n")
    actors = collections.defaultdict(list)
    for go, ft in ps: actors[go.split(".")[0]].append((go, ft))
    for w, ans in arms.items():
        s = stats[w]
        def now(go, ft):
            return ans.get(ft.replace(".fight_to_", ".walk_")) if w != "recorded" else max(ans.get(go, 0), ans.get(ft, 0))
        for go, ft in ps:
            was = max(orig.get(go, 0), orig.get(ft, 0)); x = now(go, ft)
            if x is None: s["missing"] += 1; continue
            s["pairs"] += 1; s["absdiff"] += abs(x - was)
            if was >= 0.5:
                s["was 0.5+"] += 1; s["still 0.5+"] += x >= 0.5
            else:
                s["was under"] += 1; s["now 0.5+"] += x >= 0.5
        # the actor's best walk: the same place?
        for actor, mine in actors.items():
            best = max(mine, key=lambda p: max(orig.get(p[0], 0), orig.get(p[1], 0)))
            if max(orig.get(best[0], 0), orig.get(best[1], 0)) < 0.5: continue
            s["actors with a walk at 0.5+"] += 1
            nb = max(mine, key=lambda p: now(*p) or 0)
            s["same best place"] += nb == best
for w, s in stats.items():
    print(f"{w:9s} tokens {toks[w]/1e6:.2f}M ({100*toks[w]/max(1,toks['recorded']):.0f}%)  walks {s['pairs']} (missing {s['missing']}): mean |diff| {s['absdiff']/max(1,s['pairs']):.3f}; at 0.5 or over as recorded {s['was 0.5+']}, still {s['still 0.5+']}; under as recorded {s['was under']}, now 0.5 or over {s['now 0.5+']}; actors with a walk at 0.5 or over {s['actors with a walk at 0.5+']}, the same best place {s['same best place']}")
