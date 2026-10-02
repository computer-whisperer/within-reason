#!/usr/bin/env python3
"""Offline: would the decode fuse a detachment's menu rightly? For the groups of a game that the packet in force
does not name (so the fuse layer leaves them unfused), the decode's questions are asked as the bot would ask them
("<units>; a detachment that left group_X"), and each reading is set beside what the gate rated the move."""
import json, re, sys, collections, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask
D = sys.argv[1]
key = api_key()
rows = []
for l in open(D + "/jev-0.jsonl"):
    try: r = json.loads(l)
    except ValueError: continue
    if isinstance(r, dict): rows.append(r)
packet = ""; asks = {}  # (packet id, actor, kind, target) -> {"what", "gate": [ratings]}
packets = []
for r in rows:
    if isinstance(r.get("instructions"), str):
        packet = r["instructions"]; packets.append(packet)
    if r.get("t") != "call" or "questions" not in r or any(k.startswith("worlds.") for k in r["questions"]): continue
    actors = (r.get("state") or {}).get("actors") or {}
    for k, a in r["answers"].items():
        if not isinstance(a, dict) or "noul" not in a: continue
        m = re.match(r"(group_\w+?)\.(go|fight_to|join|follow)_(.+)$", k)
        if not m: continue
        g, verb, target = m.groups()
        if re.search(r"\b" + re.escape(g) + r"\b", packet): continue
        e = actors.get(g) or {}
        parent = (e.get("split_from") or "").split(",")[0]
        if not parent: continue
        units = (e.get("units") or "")[:120]
        kind = "place" if verb in ("go", "fight_to") else verb
        if kind == "join": target = target  # join_group_X -> group_X
        rec = asks.setdefault((len(packets) - 1, g, kind, target), {"what": f"{units}; a detachment that left {parent}", "parent": parent, "gate": []})
        rec["gate"].append(a["noul"])
def question(actor, what, kind, target):
    if kind == "place": return f"Read the player's `instructions` alone. Do they name {target} in anything they say to or about {actor} ({what}): a place it goes to, stands at, fights at, builds at, falls back to, or must keep away from?"
    if kind == "join": return f"Read the player's `instructions` alone. Do they tell {actor} ({what}) to join {target}, now or under some condition: {actor} to merge into {target} and take {target}'s name and course? No when it is {target} they tell to join {actor}."
    return f"Read the player's `instructions` alone. Do they tell {actor} ({what}) to follow, escort or stay beside {target}, now or under some condition?"
by_packet = collections.defaultdict(list)
for k in asks: by_packet[k[0]].append(k)
def run(pi):
    out = {}
    ks = by_packet[pi]
    for i in range(0, len(ks), 200):
        qs = {f"q{j}": {"type": "noul", "instructions": question(k[1], asks[k]["what"], k[2], k[3])} for j, k in enumerate(ks[i:i + 200])}
        r = ask(key, {"instructions": packets[pi]}, qs, "jev-latest")
        for j, k in enumerate(ks[i:i + 200]):
            a = (r.get("answers") or {}).get(f"q{j}")
            if isinstance(a, dict): out[k] = a["noul"]
    return out
reads = {}
with concurrent.futures.ThreadPoolExecutor(8) as ex:
    for out in ex.map(run, list(by_packet)): reads.update(out)
print(f"{len(asks)} readings over {len(by_packet)} packets, {len({k[1] for k in asks})} unnamed detachments")
tab = collections.defaultdict(lambda: collections.Counter())
for k, p in reads.items():
    kind = k[2] if k[2] != "join" else ("join its parent" if k[3] == asks[k]["parent"] else "join another group")
    b = "under 0.2 (would be fused off)" if p < 0.2 else "0.2 to 0.8" if p < 0.8 else "0.8 or over"
    g = asks[k]["gate"]
    tab[(kind, b)]["readings"] += 1; tab[(kind, b)]["gate asks"] += len(g); tab[(kind, b)]["gate asks at 0.5+"] += sum(1 for x in g if x >= 0.5)
for (kind, b), c in sorted(tab.items()):
    print(f"  {kind:22s} {b:30s} readings {c['readings']:5d}  gate asks {c['gate asks']:6d}  of them rated 0.5 or over {c['gate asks at 0.5+']:4d}")
