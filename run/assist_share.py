#!/usr/bin/env python3
"""The assist pattern of a game's builders, from a replay record (run/replay_match.py's record-<team>.jsonl: the dump
widget's `bp` rows carry the unit each builder is working on; the bot's own records do not): per builder kind the
share of its seconds on its own frames, helping another's frame (a plant's current unit counts here: the engine
names the unit, not the factory), repairing, or idle and walking; and the commander's by minute. Written 2026-10-20
for the distilled bot's commander rule (the user: "the assist patterns are also important to track"); the bot's own
pattern is its `jobs:` line a minute in engine.log.

    run/assist_share.py run/matches/<replay match> [team] [to-minute]
"""
import glob, json, sys, collections
match = sys.argv[1]
team = sys.argv[2] if len(sys.argv) > 2 else "0"
rows = [json.loads(l) for l in open(f"{match}/record-{team}.jsonl", errors="replace") if l.strip().endswith("}")]
header = next(r for r in rows if r.get("t") == "header")
defs = header["unit_defs"]
cls = [d.get("class", "") for d in defs]
created_by = {}
for r in rows:
    if r.get("t") == "ev" and r.get("k") == "created" and r.get("by") is not None:
        created_by[r["u"]] = r["by"]
to = float(sys.argv[3]) if len(sys.argv) > 3 else 15.0
tally = collections.defaultdict(collections.Counter)
com_by_min = collections.defaultdict(collections.Counter)
for s in rows:
    if s.get("t") != "s" or s["f"] > to * 1800: continue
    own = {u[0]: u for u in s["own"]}
    for bp in s.get("bp", []):
        uid, prio, target = bp[0], bp[1], bp[2]
        u = own.get(uid)
        if not u or u[5] & 1: continue
        k = cls[u[1]]
        if k not in ("commander", "builder"): continue
        if target and target in own:
            t = own[target]
            if t[5] & 1:
                what = "own frame" if created_by.get(target) == uid else "helping a frame"
            elif cls[t[1]] == "factory":
                what = "assisting a plant"
            else:
                what = "repairing"
        elif target:
            what = "target unseen"
        else:
            what = "idle or walking"
        tally[k][what] += 1
        if k == "commander":
            com_by_min[s["f"] // 1800][what] += 1
for k in ("commander", "builder"):
    total = sum(tally[k].values()) or 1
    print(f"{k}: " + ", ".join(f"{w} {100*n/total:.0f}%" for w, n in tally[k].most_common()))
print("commander by minute (share of samples):")
for m in sorted(com_by_min):
    total = sum(com_by_min[m].values()) or 1
    print(f"  {m:>2}: " + ", ".join(f"{w} {100*n/total:.0f}%" for w, n in com_by_min[m].most_common(3)))
