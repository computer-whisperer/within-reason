#!/usr/bin/env python3
"""Offline: the split pick's request asked again as recorded and without the rules text in its state."""
import json, random, sys, collections, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask, requests_of
from match_read import Match
m = Match(sys.argv[1]); N = int(sys.argv[2])
key = api_key()
picks = [(c, s) for c, s, p in requests_of(m) if any(k.startswith("worlds.c") for k in c["questions"])]
random.seed(11); random.shuffle(picks); picks = picks[:N]
def run(job):
    c, state, which = job
    st = dict(state)
    if which == "no rules": st.pop("rules", None)
    r = ask(key, st, c["questions"], c.get("model", "jev-latest"))
    return c["f"], which, r.get("answers") or {}, (r.get("usage") or {}).get("input_tokens", 0)
res = collections.defaultdict(dict); toks = collections.Counter()
with concurrent.futures.ThreadPoolExecutor(8) as ex:
    for f, which, ans, t in ex.map(run, [(c, s, w) for c, s in picks for w in ("recorded", "no rules")]):
        res[f][which] = ans; toks[which] += t
rec = {c["f"]: c for c, _ in picks}
st = {w: collections.Counter() for w in ("recorded", "no rules")}
for f, arms in res.items():
    orig = rec[f]["answers"]
    for w, ans in arms.items():
        for k, a in orig.items():
            b = ans.get(k)
            if not isinstance(b, dict): continue
            kind = "first stage (which change)" if k.endswith(".pick") else "second stage (change or not)"
            st[w][kind] += 1
            if a.get("choice") == b.get("choice"): st[w][kind + " same"] += 1
            if not k.endswith(".pick"):
                if a.get("choice") != "w1": st[w]["took (recorded)"] += 1
                if b.get("choice") != "w1": st[w]["took (asked again)"] += 1
for w, s in st.items():
    print(f"{w:9s} tokens {toks[w]/1e6:.2f}M ({100*toks[w]/toks['recorded']:.0f}%): " + "; ".join(f"{k} {v}" for k, v in sorted(s.items())))
