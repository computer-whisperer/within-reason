#!/usr/bin/env python3
"""Offline: what the `same` layer would let stand in a recorded game, under variants of what "the same question"
means and of the counts it is tied to. A question stands when it was last asked fresh under RE_ASK ago with the
same key and the same counts; its recorded answer is then the fresh one the layer would not have had, so a standing
answer on the other side of 0.5 from it is a fault. Recorded gates only: a gate that would not have gone changes
what the next one asks, which this does not follow.

    run/jev_same_replay.py run/matches/<batch> [...]

Keys: `words` (the question's words), `numbers` (its words with every number replaced), `clauses` (numbers replaced
and the clauses listed in VOLATILE dropped). Counts: `as built` (`layers::counts`: `ours` without wrecks, each line up
to its colon; the economy's metal and energy lines without digits), `status` (the energy line cut before "Our
production at full speed would draw", which names who draws), `builders` (the `status` counts for a question of a
builder or a plant, whose answers the packet's tables key on, and none for a group's, a party's or a place's), `none`.
"""
import jevlog
import collections, glob, json, re, sys
RE_ASK = 600
NUM = re.compile(r"\d+(?:\.\d+)?")
VOLATILE = [re.compile(p) for p in sys.argv[sys.argv.index("--drop") + 1].split("||")] if "--drop" in sys.argv else []

def counts(state, status):
    parts = []
    for k, v in sorted((state.get("ours") or {}).items()):
        if k in ("wrecks", "resurrection_bots"): continue
        parts.append(k + "=" + (v.split(":")[0] if isinstance(v, str) else json.dumps(v)))
    for line in ("metal", "energy"):
        text = (state.get("economy") or {}).get(line) or ""
        if status: text = text.split(" Our production at full speed would draw")[0]
        parts.append(re.sub(r"\d", "", text))
    return hash("|".join(parts))

def key_of(words, kind):
    if kind == "words": return words
    words = NUM.sub("#", words)
    if kind == "clauses":
        for p in VOLATILE: words = p.sub("", words)
    return words

ARMS = [("words", "as built"), ("words", "status"), ("words", "builders"), ("words", "none"), ("numbers", "status"), ("numbers", "builders"), ("numbers", "none")] + ([("clauses", "builders")] if VOLATILE else [])
for D in [a for a in sys.argv[1:] if a.startswith("run/")]:
    D = D.rstrip("/") + "/00/"
    total = 0; q_tok = 0
    stood = collections.Counter(); faults = collections.Counter(); n_stood = collections.Counter(); whole = collections.Counter(); gates = 0
    for f in jevlog.logs(D):
        fresh = {arm: {} for arm in ARMS}
        for l in jevlog.lines(f):
            if '"t":"call"' not in l[-200:]: continue
            r = json.loads(l); q = r["questions"]; tok = (r.get("usage") or {}).get("input_tokens", 0); total += tok
            if any(k.startswith("worlds.") for k in q): continue
            gates += 1
            a = r.get("answers") or {}; fr = r["f"]; st = r["state"]
            chars = {k: len(json.dumps(v)) for k, v in q.items()}
            pic = (sum(len(json.dumps(v)) for v in st.values()) + 6000) * (r.get("batches") or 1)
            all_chars = pic + sum(chars.values())
            h = {"as built": counts(st, False), "status": counts(st, True), "none": 0}
            q_tok += tok * sum(chars.values()) / all_chars
            for arm in ARMS:
                kind, cnt = arm; every = True
                for k, v in q.items():
                    n = (a.get(k) or {}).get("noul")
                    if n is None: every = False; continue
                    tie = (h["status"] if k.startswith(("constructor_", "commander", "plant_", "lab_")) else 0) if cnt == "builders" else h[cnt]
                    key = (key_of(v["instructions"], kind), tie); p = fresh[arm].get(k)
                    if p and fr - p[0] < RE_ASK and p[1] == key:
                        stood[arm] += tok * chars[k] / all_chars; n_stood[arm] += 1
                        faults[arm] += (p[2] >= 0.5) != (n >= 0.5)
                    else:
                        fresh[arm][k] = (fr, key, n); every = False
                if every: whole[arm] += tok * pic / all_chars
    print(f"== {D.split('/')[-3][11:]}: {total/1e6:.1f}M tokens, {gates} gates, {q_tok/1e6:.1f}M of gate questions")
    for arm in ARMS:
        s = stood[arm] + whole[arm]
        print(f"  {arm[0]:8s} counts {arm[1]:9s}: stands {n_stood[arm]:7d} questions, {stood[arm]/1e6:6.2f}M + {whole[arm]/1e6:5.2f}M of gates not sent = {100*s/total:5.1f}% of the bill; on the other side of 0.5: {faults[arm]} ({100*faults[arm]/max(1,n_stood[arm]):.2f}%)")
