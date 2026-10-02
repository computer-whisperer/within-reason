#!/usr/bin/env python3
"""Offline: a gate's own-move questions with the standing course cut from each question (the entry under `actors`
carries it), against the same gate asked again as recorded; the key is read, never printed.

    run/jev_words_ab.py run/matches/<batch>/00 [gates] [out.jsonl]
"""
import json, random, re, sys, collections, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask, requests_of
from match_read import Match
D = sys.argv[1] if len(sys.argv) > 1 else "run/matches/1790954859-player-36-standing/00"
N = int(sys.argv[2]) if len(sys.argv) > 2 else 40
OUT = sys.argv[3] if len(sys.argv) > 3 else "/dev/null"
m = Match(D)
key = api_key()
OWN = re.compile(r"^(Given `actors\.[^`]+`, `economy`, `ours` and the player's `instructions`: is this what (\S+) should do now, rather than )(.*)(\? The move: .*)$", re.S)
gates = []
for c, state, packet in requests_of(m):
    q = c["questions"]
    if any(k.startswith("worlds.") for k in q): continue
    own = [k for k, v in q.items() if OWN.match(v["instructions"])]
    if len(own) >= 10: gates.append((c, state))
random.seed(7)
random.shuffle(gates)
gates = gates[:N]
def arm(q, state, which):
    if which == "recorded": return q, state
    out = {}; courses = {}
    for k, v in q.items():
        mm = OWN.match(v["instructions"])
        if not mm: out[k] = v; continue
        courses[mm.group(2)] = mm.group(3)
        if which == "cut":
            text = mm.group(1) + "what it does now" + mm.group(4)
        else:  # "entry": the course's words once, in the actor's entry
            text = mm.group(1) + "its `course` there" + mm.group(4)
        out[k] = {**v, "instructions": text}
    st = state
    if which == "entry":
        st = json.loads(json.dumps(state))
        for actor, words in courses.items():
            if actor in st.get("actors", {}): st["actors"][actor]["course"] = words
    return out, st
def run(job):
    c, state, which = job
    q, st = arm(c["questions"], state, which)
    answers = {}; tokens = 0
    base = len(json.dumps(st)); chunk = {}; size = base; chunks = []
    for k, v in q.items():
        L = len(json.dumps(v)) + len(k) + 6
        if chunk and size + L > 140_000:
            chunks.append(chunk); chunk = {}; size = base
        chunk[k] = v; size += L
    if chunk: chunks.append(chunk)
    for part in chunks:
        r = ask(key, st, part, c.get("model", "jev-latest"))
        answers.update(r.get("answers") or {}); tokens += (r.get("usage") or {}).get("input_tokens", 0)
    return c["f"], which, {k: a.get("noul") for k, a in answers.items() if isinstance(a, dict) and "noul" in a}, tokens
jobs = [(c, s, w) for c, s in gates for w in ("recorded", "cut", "entry")]
res = collections.defaultdict(dict); toks = collections.Counter()
with concurrent.futures.ThreadPoolExecutor(8) as ex:
    for f, which, ans, t in ex.map(run, jobs):
        res[f][which] = ans; toks[which] += t
rec = {c["f"]: c for c, _ in gates}
out = open(OUT, "w")
stats = {w: collections.Counter() for w in ("recorded", "cut", "entry")}
for f, arms in res.items():
    c = rec[f]; orig = {k: a.get("noul") for k, a in c["answers"].items() if isinstance(a, dict) and "noul" in a}
    own = [k for k, v in c["questions"].items() if OWN.match(v["instructions"])]
    by_actor = collections.defaultdict(list)
    for k in own: by_actor[k.split(".")[0]].append(k)
    out.write(json.dumps({"f": f, "orig": orig, **arms}) + "\n")
    for w, ans in arms.items():
        s = stats[w]
        for k in own:
            if k in ans and k in orig:
                s["n"] += 1; s["absdiff"] += abs(ans[k] - orig[k])
                if (ans[k] >= 0.5) != (orig[k] >= 0.5): s["crossed"] += 1
        for actor, ks in by_actor.items():
            best = max(ks, key=lambda k: orig.get(k, 0))
            if orig.get(best, 0) >= 0.5:
                s["strong"] += 1
                nb = max(ks, key=lambda k: ans.get(k, 0))
                if nb == best: s["strong still best"] += 1
                if ans.get(best, 0) >= 0.5: s["strong still 0.5+"] += 1
            else:
                s["weak"] += 1
                if max(ans.get(k, 0) for k in ks) >= 0.5: s["weak now 0.5+"] += 1
for w, s in stats.items():
    print(f"{w:9s} tokens {toks[w]/1e6:.2f}M ({100*toks[w]/max(1,toks['recorded']):.0f}%)  own-move answers {s['n']}: mean |diff| {s['absdiff']/max(1,s['n']):.3f}, crossed 0.5 {s['crossed']} ({100*s['crossed']/max(1,s['n']):.1f}%); strong actors {s['strong']}: still best {s['strong still best']}, still 0.5+ {s['strong still 0.5+']}; actors with nothing at 0.5: {s['weak']}, now something {s['weak now 0.5+']}")
