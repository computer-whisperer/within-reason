#!/usr/bin/env python3
"""Offline: the split pick's request asked again as recorded, without the rules text in its state, and with the
state's `actors` cut to the actors its questions name.

    run/jev_pick_rules_ab.py run/matches/<batch>/00 <picks>
"""
import json, random, re, sys, collections, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask, requests_of
from match_read import Match
m = Match(sys.argv[1]); N = int(sys.argv[2])
key = api_key()
ARMS = ("recorded", "no rules", "named actors")
picks = [(c, s) for c, s, p in requests_of(m) if any(k.startswith("worlds.c") for k in c["questions"])]
random.seed(11); random.shuffle(picks); picks = picks[:N]
def run(job):
    c, state, which = job
    st = dict(state)
    if which == "no rules": st.pop("rules", None)
    if which == "named actors":
        text = json.dumps(c["questions"])
        st["actors"] = {a: v for a, v in (state.get("actors") or {}).items() if re.search(r"\b" + re.escape(a) + r"\b", text)}
    r = ask(key, st, c["questions"], c.get("model", "jev-latest"))
    return c["f"], which, r.get("answers") or {}, (r.get("usage") or {}).get("input_tokens", 0)
res = collections.defaultdict(dict); toks = collections.Counter()
with concurrent.futures.ThreadPoolExecutor(8) as ex:
    for f, which, ans, t in ex.map(run, [(c, s, w) for c, s in picks for w in ARMS]):
        res[f][which] = ans; toks[which] += t
rec = {c["f"]: c for c, _ in picks}
st = {w: collections.Counter() for w in ARMS}
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
    print(f"{w:12s} tokens {toks[w]/1e6:.2f}M ({100*toks[w]/toks['recorded']:.0f}%): " + "; ".join(f"{k} {v}" for k, v in sorted(s.items())))
