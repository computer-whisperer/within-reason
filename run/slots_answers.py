#!/usr/bin/env python3
"""Every answer of a flagship game on the slots (H-HANDS-ANSWER), one row each: when it began, how long it lasted,
how many soldiers, the party, the lender and the lender's task at the send (`DUP`: the lender was itself attacking
that party), what ended it (its withdraw, or the lane's hunt: the quarry dead, out of sight, the leash), where the
party stood then, and the task of the lender its soldiers were dealt back to. Then the tally
(K-hands-an-answer-carved-from-a-body-in-a-fight-returns-it-into-the-shells-course).

usage: run/slots_answers.py run/matches/<batch>/<NN> [...]
"""
import json, math, os, re, sys, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from match_read import first, rows  # noqa: E402

def clock(f):
    s = f // 30
    return f"{s // 60}:{s % 60:02d}"

def kind_of(w, how):
    return "withdrawn" if w else "quarry dead" if "quarry is dead" in how else "out of sight" if "out of sight" in how else "leash" if "ran" in how else how

def read(d):
    calls, plans, ends = [], [], []
    for r in rows(first(d, "jev-")):
        t = r.get("t")
        if t == "call" and r.get("groups") is not None:
            calls.append(r)
        if t == "plan":
            plans.append(r)
        if t == "pass":
            for h in r.get("hunts") or []:
                m = re.match(r"the answer to (party_\d+) ended after (\d+) s: ([^;]*)", h)
                if m: ends.append((r["f"], m.group(1), int(m.group(2)), m.group(3)))
    # answers as spans of presence in the groups lists
    spans = []   # [name, first f, last f, max members, lender-task at start, ...]
    open_ = {}
    prev_groups = {}
    for c in calls:
        now = {g["name"]: g for g in c["groups"]}
        for name, g in now.items():
            if name.startswith("answer_"):
                if name not in open_:
                    open_[name] = {"name": name, "f0": c["f"], "f1": c["f"], "max": len(g["members"]), "units": set(g["members"]), "before": dict(prev_groups)}
                o = open_[name]; o["f1"] = c["f"]; o["max"] = max(o["max"], len(g["members"])); o["units"] |= set(g["members"]); o["last"] = g
                o.setdefault("party0", next((p for p in c.get("parties") or [] if p["name"] == name.replace("answer_", "")), None))
        for name in list(open_):
            if name not in now:
                o = open_.pop(name); o["after"] = now; o["after_f"] = c["f"]; o["parties"] = c.get("parties"); spans.append(o)
        prev_groups = now
    for o in open_.values():
        o["after"] = {}; o["after_f"] = None; o["parties"] = None; spans.append(o)
    sends = collections.defaultdict(list); withdraws = collections.defaultdict(list)
    for p in plans:
        for pl in p.get("played") or []:
            m = re.match(r"group_(\w+?)\.send_(\d+)_(party_\d+)$", pl["played"])
            if m: sends[m.group(3)].append((p["f"], m.group(1), int(m.group(2)), pl["did"]))
            if pl["played"].endswith(".withdraw"): withdraws[pl["actor"].replace("group_answer_", "")].append(p["f"])
    print(f"\n=== {d}: {len(spans)} answers")
    print("  start  secs  n  party      lender (its task at the send)             ended by                                   | at the end: party then; lender's task the soldiers came back to")
    tally = collections.Counter()
    for o in sorted(spans, key=lambda o: o["f0"]):
        party = o["name"].replace("answer_", "")
        s = [x for x in sends.get(party, []) if o["f0"] - 60 <= x[0] <= o["f1"]]
        lender = s[0][1] if s else "?"
        lt = (o["before"].get(lender) or {}).get("task") or {}
        p0 = o.get("party0")
        dup = lt.get("kind") == "engage" and p0 is not None and lt.get("to") and math.hypot(lt["to"][0] - p0["x"], lt["to"][1] - p0["z"]) < 400
        w = [f for f in withdraws.get(party, []) if o["f0"] <= f <= (o["after_f"] or 10**9)]
        e = [x for x in ends if x[1] == party and o["f1"] - 30 <= x[0] <= (o["after_f"] or o["f1"]) + 60]
        how = "withdrawn" if w else (e[0][3] if e else ("still open at the end" if o["after_f"] is None else "?"))
        # the party at the end
        pstr = "-"
        if o["parties"] is not None:
            pp = [p for p in o["parties"] if p.get("name") == party]
            if pp:
                p = pp[0]
                la = o["last"].get("at")
                dist = math.hypot(p["x"] - la[0], p["z"] - la[1]) if la else -1
                pstr = f"{p.get('composition')} ({p.get('metal')} metal) {dist:.0f} from the answer"
                if kind_of(w, how) == "quarry dead" and dist < 900: tally["quarry dead with the party within 900"] += 1
            else:
                pstr = "gone from the picture"
        back = (o["after"].get(lender) or {}).get("task") or {}
        secs = (o["f1"] - o["f0"]) / 30
        kind = kind_of(w, how)
        tally[kind] += 1
        if dup: tally["lent at the party the lender itself was attacking"] += 1
        if secs <= 3: tally["lasted 3 s or less"] += 1
        print(f"  {clock(o['f0']):>5} {secs:5.0f} {o['max']:2}  {party:9}  {lender:6} ({lt.get('kind')}{' ' + str(lt.get('place')) if lt.get('place') else ''}){'  DUP' if dup else ''}  | {how[:60]:60} | {pstr[:60]:60}; {back.get('kind')} {back.get('place') or ''}")
    print("  tally:", dict(tally))

for d in sys.argv[1:]:
    read(d)
