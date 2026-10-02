#!/usr/bin/env python3
"""Group-seconds with an attack task in which every soldier was idle and over 400 from the party (the Jev log's
`engage` group rows against the record's samples): what H-HANDS-ATTACK-FOLLOWS is judged by.

    run/engage_idle.py run/matches/<batch> [...]
"""
import json,math,collections,sys
for D in sys.argv[1:]:
    D=D.rstrip("/")+"/00/"
    samples={}
    for l in open(D+"record-0.jsonl"):
        try: r=json.loads(l)
        except: continue
        if isinstance(r,dict) and r.get("t")=="s": samples[r["f"]//30]={u[0]:u for u in r["own"]}
    tot=collections.Counter(); seen=set()
    for l in open(D+"jev-0.jsonl"):
        try: r=json.loads(l)
        except: continue
        if not isinstance(r,dict) or r.get("t")!="call" or r.get("f") is None: continue
        sec=r["f"]//30
        if sec in seen: continue
        seen.add(sec)
        own=samples.get(sec)
        if not own: continue
        for g in r.get("groups") or []:
            t=g["task"]
            if t.get("kind")!="engage" or "to" not in t: continue
            us=[own[m] for m in g["members"] if m in own]
            if not us: continue
            tot["engage"]+=1
            idle=sum(1 for u in us if u[5]&2)
            d=min(math.hypot(u[2]-t["to"][0],u[3]-t["to"][1]) for u in us)
            if idle==len(us) and d>400: tot["idle_far"]+=1
    e=tot["engage"] or 1
    print(D.split("/")[2][11:], "engage group-seconds", tot["engage"], "all idle and >400 away", tot["idle_far"], f"{100*tot['idle_far']/e:.0f}%")
