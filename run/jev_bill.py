#!/usr/bin/env python3
"""Where a game's Jev input tokens go: by request kind, and inside the requests by the picture's sections and the
question kinds (characters, scaled to the tokens the log reports).

    run/jev_bill.py run/matches/<batch> [...]
"""
import json, sys, collections, glob, itertools
for D in sys.argv[1:]:
    D = D.rstrip("/") + "/00/"
    kinds = collections.Counter(); calls = collections.Counter(); batches = collections.Counter()
    state_chars = collections.Counter(); q_chars = collections.Counter(); q_n = collections.Counter()
    tok_state = 0.0; tok_q = 0.0; header = None; packet = ""; rules = ""
    # Every seat of ours: a team game's bill was seat 0's alone (human-19 read $0.37 of its $0.74).
    for l in itertools.chain.from_iterable(open(f) for f in sorted(glob.glob(D + "jev-*.jsonl"))):
        try: r = json.loads(l)
        except ValueError: continue
        if not isinstance(r, dict): continue
        if "rules" in r and isinstance(r["rules"], str): rules = r["rules"]
        if "instructions" in r and isinstance(r["instructions"], str): packet = r["instructions"]
        t = r.get("t")
        if t == "decode":
            u = r.get("usage") or {}
            kinds["decode"] += u.get("input_tokens", u.get("prompt_tokens", 0)) or 0; calls["decode"] += 1
            continue
        if t != "call" or "questions" not in r: continue
        u = r.get("usage") or {}
        tok = u.get("input_tokens", u.get("prompt_tokens", 0)) or 0
        q = r["questions"]
        kind = "pick stage 1" if "worlds.pick" in q else "pick stage 2" if any(k.startswith("worlds.") for k in q) else "gate"
        kinds[kind] += tok; calls[kind] += 1; b = r.get("batches", 1) or 1; batches[kind] += b
        st = r.get("state") or {}
        sc = {k: len(json.dumps(v)) for k, v in st.items()}
        sc["instructions (packet)"] = len(packet); sc["rules"] = len(rules)
        qc = collections.Counter()
        for k, v in q.items():
            verb = k.split(".", 1)[1] if "." in k else k
            verb = "forbidden" if verb.endswith(".forbidden") else verb.split("_")[0]
            if kind != "gate": verb = kind
            qc[verb] += len(json.dumps(v)); q_n[verb] += 1
        s_total = sum(sc.values()) * b; q_total = sum(qc.values())
        if s_total + q_total == 0: continue
        for k, v in sc.items(): state_chars[(kind, k)] += tok * (v * b) / (s_total + q_total)
        for k, v in qc.items(): q_chars[k] += tok * v / (s_total + q_total)
        tok_state += tok * s_total / (s_total + q_total); tok_q += tok * q_total / (s_total + q_total)
    total = sum(kinds.values())
    print(f"== {D.split('/')[2][11:]}: {total/1e6:.2f}M input tokens, ${total*0.042/1e6:.2f}")
    for k, v in kinds.most_common():
        print(f"  {k:13s} {v/1e6:6.2f}M {100*v/total:4.0f}%  {calls[k]} calls, {batches[k]} requests, {v/max(1,batches[k]):.0f} tokens a request")
    print(f"  the picture: {tok_state/1e6:.2f}M ({100*tok_state/total:.0f}%); the questions: {tok_q/1e6:.2f}M ({100*tok_q/total:.0f}%)")
    by = collections.Counter()
    for (kind, k), v in state_chars.items(): by[k] += v
    print("  picture by section: " + ", ".join(f"{k} {v/1e6:.2f}M" for k, v in by.most_common()))
    for kind in ("gate", "pick stage 1", "pick stage 2"):
        print(f"    in {kind}: " + ", ".join(f"{k} {v/1e6:.2f}M" for (kk, k), v in sorted(state_chars.items(), key=lambda x: -x[1]) if kk == kind)[:300])
    print("  questions by kind: " + ", ".join(f"{k} {v/1e6:.2f}M ({q_n[k]})" for k, v in q_chars.most_common(14)))
