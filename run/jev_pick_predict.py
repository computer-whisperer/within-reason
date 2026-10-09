#!/usr/bin/env python3
"""Does the gate's own answer foretell the pick? For every split-pick component of one actor and one change: the
gate's rating of that change (the actor's best-rated move) set beside whether the pick took it.

    run/jev_pick_predict.py run/matches/<batch> [...]
"""
import jevlog
import json, sys, glob, collections

EDGES = [0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.01]

def bucket(r):
    return next(i for i, e in enumerate(EDGES) if r < e)

for D in sys.argv[1:]:
    D = D.rstrip("/") + "/00/"
    n = collections.Counter(); took = collections.Counter(); picks = alone = one = 0; pick_tok = alone_tok = 0
    for f in jevlog.logs(D):
        gate = None; tok = 0
        for l in jevlog.lines(f):
            if '"t":"worlds_gate"' in l:
                gate = json.loads(l); tok = 0
            elif '"t":"call"' in l and '"worlds.c' in l:
                r = json.loads(l); tok = (r.get("usage") or {}).get("input_tokens", 0)
            elif '"t":"plan"' in l and gate is not None:
                plan = json.loads(l); comps = gate.get("components") or []; parts = plan.get("split") or []
                flags = gate.get("flags") or {}
                if not parts: gate = None; continue
                picks += 1; pick_tok += tok
                simple = True; rows = []
                for c in comps:
                    part = next((p for p in parts if p["actors"] == c["actors"]), None)
                    if len(c["actors"]) != 1 or len(c["lines"]) != 2 or part is None or part.get("taken") is None:
                        simple = False; continue
                    a = c["actors"][0]
                    rs = [v for k, v in flags.items() if k.startswith(a + ".") and not k.endswith((".change", ".forbidden"))]
                    if not rs: simple = False; continue
                    kind = "idle or party" if a + ".change" not in flags else "changing course"
                    rows.append((kind, bucket(max(rs)), part["taken"]))
                for kind, b, t in rows:
                    n[(kind, b)] += 1; took[(kind, b)] += t; one += 1
                if simple and len(rows) == len(comps):
                    alone += 1; alone_tok += tok
                gate = None
    print(f"== {D.split('/')[-3][11:]}: {picks} picks, {pick_tok/1e6:.1f}M tokens; {alone} ({alone_tok/1e6:.1f}M) made only of one-actor one-change components; {one} such components")
    for kind in ("idle or party", "changing course"):
        print(f"  {kind}: the gate's rating of the change -> components, taken")
        lo = 0.0
        for i, e in enumerate(EDGES):
            if n[(kind, i)]:
                print(f"    {lo:.1f}-{min(e, 1.0):.1f}  {n[(kind, i)]:6d}  taken {100 * took[(kind, i)] / n[(kind, i)]:5.1f}%")
            lo = e
