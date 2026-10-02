#!/usr/bin/env python3
"""Offline: a gate's walks to named places (`go_<place>`, `fight_to_<place>`: one noul each, 40% of player-38's gate
text and 2% of them at 0.5 or over) asked as one Choice an actor among its walks and staying on its course,
against the same gate asked again as recorded. `choice2`: every recorded walk question is an option; `choice1`: one
option a place where both manners were asked. The key is read, never printed.

    run/jev_walk_choice_ab.py run/matches/<batch>/00 [gates] [out.jsonl]
"""
import json, random, re, sys, collections, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask, requests_of
from match_read import Match
D = sys.argv[1] if len(sys.argv) > 1 else "run/matches/1790957410-player-38-split/00"
N = int(sys.argv[2]) if len(sys.argv) > 2 else 40
OUT = sys.argv[3] if len(sys.argv) > 3 else "/dev/null"
ARMS = ("recorded", "choice2", "choice1")
m = Match(D)
key = api_key()
OWN = re.compile(r"^Given `actors\.[^`]+`, `economy`, `ours` and the player's `instructions`: is this what (\S+) should do now, rather than (.*)\? The move: (.*)$", re.S)
FIGHT = re.compile(r"^(\S+) advances to (\S+ \(.*?\)), fighting on the way(.*)$", re.S)
def place(k):
    move = k.split(".", 1)[1]
    return move[3:] if move.startswith("go_") else move[9:] if move.startswith("fight_to_") else None
def walks_of(q):
    """actor -> its walk question keys, for the group actors with four walks or more."""
    by = collections.defaultdict(list)
    for k, v in q.items():
        if k.startswith("group_") and place(k) and not k.endswith(".forbidden") and OWN.match(v["instructions"]): by[k.split(".")[0]].append(k)
    return {a: ks for a, ks in by.items() if len(ks) >= 4}
def nouls(answers):
    return {k: a.get("noul") for k, a in answers.items() if isinstance(a, dict) and a.get("noul") is not None}
gates = []
for c, state, packet in requests_of(m):
    q = c["questions"]
    if any(k.startswith("worlds.") for k in q): continue
    if walks_of(q): gates.append((c, state))
random.seed(11)
random.shuffle(gates)
gates = gates[:N]
def arm(q, which):
    """The questions of the arm and, for a choice arm, actor -> option id -> the places it stands for."""
    if which == "recorded": return q, {}
    out = dict(q); options = {}
    for actor, ks in walks_of(q).items():
        course = OWN.match(q[ks[0]]["instructions"]).group(2)
        criteria = {"stay": f"{actor} keeps to what it does now: {course}"}; mine = {}
        seen = {}
        for k in ks:
            del out[k]
            words = OWN.match(q[k]["instructions"]).group(3)
            p = place(k)
            if which == "choice1":
                other = f"{actor}.{'fight_to_' if '.go_' in k else 'go_'}{p}"
                if other in q:
                    if p in seen: continue
                    ft = FIGHT.match(OWN.match(q[f"{actor}.fight_to_{p}"]["instructions"]).group(3))
                    if ft: words = f"{ft.group(1)} goes to {ft.group(2)}{ft.group(3)}"
                    seen[p] = True
            oid = f"o{len(criteria)}"
            criteria[oid] = words; mine[oid] = p
        out[f"{actor}.walk"] = {"type": "choice", "instructions": f"Given `actors.{actor}`, `economy`, `ours` and the player's `instructions`: what should {actor} do now? `stay` is what it does now; every other option is a walk of its own to a place the player named. Choose `stay` unless a walk is what the instructions and the picture call for now.", "criteria": criteria}
        options[actor] = mine
    return out, options
def run(job):
    c, state, which = job
    q, options = arm(c["questions"], which)
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
    return c["f"], which, answers, options, tokens
jobs = [(c, s, w) for c, s in gates for w in ARMS]
res = collections.defaultdict(dict); toks = collections.Counter()
with concurrent.futures.ThreadPoolExecutor(8) as ex:
    for f, which, ans, options, t in ex.map(run, jobs):
        res[f][which] = (ans, options); toks[which] += t
rec = {c["f"]: c for c, _ in gates}
out = open(OUT, "w")
stats = {w: collections.Counter() for w in ARMS}
for f, arms in res.items():
    c = rec[f]; orig = nouls(c["answers"])
    out.write(json.dumps({"f": f, "orig": orig, **{w: {"answers": a, "options": o} for w, (a, o) in arms.items()}}) + "\n")
    for actor, ks in walks_of(c["questions"]).items():
        best = max(ks, key=lambda k: orig.get(k, 0)); strong = {place(k) for k in ks if orig.get(k, 0) >= 0.5}
        for w, (ans, options) in arms.items():
            s = stats[w]
            if w == "recorded":
                now = nouls(ans); nb = max(ks, key=lambda k: now.get(k, 0))
                walked = now.get(nb, 0) >= 0.5; where = place(nb)
            else:
                a = ans.get(f"{actor}.walk")
                if not isinstance(a, dict) or "choice" not in a: s["missing"] += 1; continue
                walked = a["choice"] != "stay"; where = options[actor].get(a["choice"])
                s["p(stay) sum"] += (a.get("probabilities") or {}).get("stay", 0)
            if strong:
                s["strong"] += 1; s["strong: walks"] += walked
                s["strong: to the best place"] += walked and where == place(best)
                s["strong: to a 0.5 place"] += walked and where in strong
            else:
                s["weak"] += 1; s["weak: walks"] += walked
for w, s in stats.items():
    n = s["strong"] + s["weak"]
    print(f"{w:9s} tokens {toks[w]/1e6:.2f}M ({100*toks[w]/max(1,toks['recorded']):.0f}%)  actors {n} (missing {s['missing']}); with a walk at 0.5 or over as recorded {s['strong']}: walks {s['strong: walks']}, to a place that was at 0.5 or over {s['strong: to a 0.5 place']}, to the best place {s['strong: to the best place']}; with none {s['weak']}: walks {s['weak: walks']}" + (f"; mean p(stay) {s['p(stay) sum']/max(1,n):.2f}" if w != "recorded" else ""))
