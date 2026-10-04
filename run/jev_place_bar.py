#!/usr/bin/env python3
"""The `places` layer under a stricter opening, replayed on a game's `jev-<ai>.jsonl`: every move aimed at a place
that was asked is set beside what its place's question said at its last ask, and so is every such move that was
played. Per bar, and per "the best k places a gate": the place-aimed move questions that would not have been asked,
and the played moves whose place would have been shut.

    run/jev_place_bar.py run/matches/<batch> [...]
"""
import collections, glob, json, re, sys

PLACE = re.compile(r"^(?:go|fight_to|raid|shell|build_[a-z0-9]+)_((?:spot|passage)_\d+|home|.+)$")


def place_of(move):
    if "party_" in move or move.endswith(".forbidden"): return None
    m = PLACE.match(move)
    return m.group(1) if m else None


def kind(actor): return "group" if actor.startswith("group_") else "builder"


for D in sys.argv[1:]:
    said = {}          # place -> (frame, noul)
    rank = {}          # place -> its rank among the places asked at its last gate (0 the best)
    asked = collections.defaultdict(list); played = collections.defaultdict(list); tokens = collections.Counter(); total = 0
    for l in open(glob.glob(D.rstrip("/") + "/00/jev-0.jsonl")[0]):
        try: r = json.loads(l)
        except ValueError: continue
        t = r.get("t")
        if t in ("call", "decode"): total += (r.get("usage") or {}).get("input_tokens", 0) or 0
        if t == "call" and "questions" in r and not any(k.startswith("worlds") for k in r["questions"]):
            q = r["questions"]; ans = r.get("answers") or {}; T = r["usage"]["input_tokens"]
            tot = (sum(len(json.dumps(v)) for v in (r.get("state") or {}).values()) + 9000) * (r.get("batches", 1) or 1) + sum(len(json.dumps(v)) for v in q.values())
            # The moves first: they are asked on what the places said at the gate before.
            for k, v in q.items():
                actor, _, move = k.partition(".")
                p = place_of(move)
                if p and p in said and r["f"] - said[p][0] <= 600 and actor.startswith(("group_", "constructor", "commander")):
                    asked[kind(actor)].append((said[p][1], rank.get(p, 99), T * len(json.dumps(v)) / tot))
            now = sorted(((ans[k]["noul"], k[:-6]) for k in q if k.endswith(".place") and "noul" in (ans.get(k) or {})), reverse=True)
            for i, (n, p) in enumerate(now): said[p] = (r["f"], n); rank[p] = i
        elif t == "plan":
            for m in r.get("played") or []:
                actor, _, move = (m.get("played") or "").partition(".")
                p = place_of(move)
                if p and p in said and m.get("source") == "plan": played[kind(actor)].append((said[p][1], rank.get(p, 99)))
    print(f"== {D.rstrip('/').split('/')[-1][11:]}: {total/1e6:.1f}M tokens")
    for who in ("group", "builder"):
        a, p = asked[who], played[who]
        if not a or not p: continue
        tok = sum(x[2] for x in a)
        print(f"  {who}s: {len(a)} place-aimed move questions ({tok/1e6:.2f}M, {100*tok/total:.1f}% of the game), {len(p)} such moves played")
        for bar in (0.3, 0.4, 0.5, 0.6, 0.7):
            cut = sum(x[2] for x in a if x[0] < bar); lost = sum(1 for x in p if x[0] < bar)
            print(f"    open at {bar}: {100*cut/tok:3.0f}% of those questions not asked ({100*cut/total:.1f}% of the game); {lost} of {len(p)} played moves shut ({100*lost/len(p):.0f}%)")
        for k in (3, 5, 8, 12):
            cut = sum(x[2] for x in a if x[1] >= k); lost = sum(1 for x in p if x[1] >= k)
            print(f"    the best {k:2d} places a gate: {100*cut/tok:3.0f}% not asked ({100*cut/total:.1f}% of the game); {lost} of {len(p)} played moves shut ({100*lost/len(p):.0f}%)")
