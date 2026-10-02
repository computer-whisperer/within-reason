#!/usr/bin/env python3
"""A reviewer's blind verdicts (`<path>.verdicts.jsonl`: one `{"id", "verdict": "yes"|"no"|"unclear"}` a line) on a
packet of `run/jev_review_packet.py moves`, against the answers as recorded and with the arm (`<path>.key.json`).

    run/jev_review_score.py <path>...

For packets of `run/jev_review_packet.py disagree` (several arms an item) it prints each arm's count of verdicts
matched, pooled over the paths given.
"""
import json, sys, collections
D = ""
pooled = collections.defaultdict(collections.Counter)
for label in sys.argv[1:]:
    key = json.load(open(D + label + ".key.json")); ver = {}
    for line in open(D + label + ".verdicts.jsonl"):
        if line.strip():
            r = json.loads(line); ver[r["id"]] = r
    if all("arms" in k for k in key.values()):
        # A `disagree` packet: every arm's answer against the verdict, pooled over the packets given.
        for i, k in key.items():
            truth = ver[i]["verdict"]
            if truth not in ("yes", "no"): continue
            for arm, x in k["arms"].items():
                c = pooled[arm]; said = x >= 0.5
                c["n"] += 1; c["right"] += said == (truth == "yes"); c["yeses"] += truth == "yes"
                c["yeses right"] += truth == "yes" and said; c["noes right"] += truth == "no" and not said; c["said yes"] += said
        continue
    c = collections.Counter(); extra = collections.Counter()
    for i, k in key.items():
        v = ver.get(i)
        if not v: c["missing"] += 1; continue
        truth = v["verdict"]; rec = k["recorded"] >= 0.5; arm = k["arm"] >= 0.5
        if not k["flip"]:
            c["controls"] += 1
            if truth == "unclear": c["controls unclear"] += 1
            else: c["controls: recorded agrees"] += (truth == "yes") == rec
            continue
        c["flips"] += 1; up = arm and not rec
        c["flips up" if up else "flips down"] += 1
        if truth == "unclear": c["flips unclear"] += 1; continue
        right = (truth == "yes") == arm
        c["arm right"] += right; c["recorded right"] += not right
        c[("up" if up else "down") + (": arm right" if right else ": recorded right")] += 1
        for name in ("roving",):
            if v.get(name): extra["roving mattered, arm " + ("right" if right else "wrong")] += 1
        if v.get("others"): extra["others leaned on, arm " + ("right" if right else "wrong")] += 1
    print(f"{label}: " + "; ".join(f"{a} {b}" for a, b in c.items()) + (" | " + "; ".join(f"{a} {b}" for a, b in extra.items()) if extra else ""))
for arm, c in pooled.items():
    print(f"{arm:32s} right {c['right']} of {c['n']}; of {c['yeses']} yeses {c['yeses right']}; of {c['n'] - c['yeses']} noes {c['noes right']}; said yes {c['said yes']}")
