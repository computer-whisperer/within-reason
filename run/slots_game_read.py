#!/usr/bin/env python3
"""A flagship game's numbers for groups as slots (docs/design/2026-11-11-groups-as-slots.md §11), side by side for
several matches: the groups, the lends and the top-ups among them, the withdrawals, world 1's share of the pick by
the number of worlds, the gate's answer about a party already answered, the orders to soldiers, the hunt's attack
orders, the soldier-seconds and the losses by standing (with a group, lent, coming or with no slot), and the player's turns and `groups` calls. Soldiers are the report's (`match_read.is_army_def`), of any faction.

usage: run/slots_game_read.py run/matches/<batch>/<NN> [...]

Extractors lost to raids are `run/raid_ledger.py`'s to count (this tool's own count missed kills and was taken out).
"""
import bisect, collections, json, os, re, statistics, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from match_read import first, is_army_def, rows as read_rows  # noqa: E402


def clock(f):
    s = int(f) // 30
    return f"{s // 60}:{s % 60:02d}"


def read(d):
    out = collections.OrderedDict()
    batch = os.path.dirname(d.rstrip("/"))
    # A game with people has no referee and no results file.
    if os.path.exists(batch + "/results.jsonl"):
        result = json.loads(open(batch + "/results.jsonl").readline())
        out["result"] = f"{result['outcome']} {result['game_minutes']:.1f} min, {result['wall_seconds']:.0f} s wall"
    else:
        out["result"] = "no results file (a game with people)"
    rows = list(read_rows(first(d, "jev-")))
    calls = [r for r in rows if r.get("t") == "call"]
    gates = [c for c in calls if isinstance(c.get("questions"), dict) and not any(k.startswith("worlds.") for k in c["questions"])]
    picks = [c for c in calls if isinstance(c.get("questions"), dict) and any(k.startswith("worlds.c") for k in c["questions"])]
    plans = [r for r in rows if r.get("t") == "plan"]
    # groups
    names = collections.Counter()
    for c in calls:
        for g in c.get("groups") or []:
            names[g["name"]] += 1
    player = [n for n in names if not n.startswith("answer_")]
    out["groups (not answers)"] = len(player)
    # lends
    sends, wd, joins = collections.defaultdict(list), collections.defaultdict(list), 0
    for p in plans:
        for played in p.get("played") or []:
            pl = played["played"]
            if ".send_" in pl:
                sends[pl.split("_party_")[1]].append(p["f"])
            if pl.endswith(".withdraw"):
                wd[played["actor"].split("party_")[1]].append(p["f"])
            if ".join_" in pl:
                joins += 1
    n = sum(len(v) for v in sends.values())
    top = sum(1 for s in sends.values() for a, b in zip(s, s[1:]) if b - a <= 300)
    again = sum(1 for p, ws in wd.items() for w in ws if any(0 < s - w <= 300 for s in sends.get(p, [])))
    out["lends, parties lent at"] = f"{n} at {len(sends)}"
    out["lends within 10 s of one at the same party"] = f"{top} ({top / max(1, n):.0%})"
    out["withdrawals, lent at again within 10 s"] = f"{sum(len(w) for w in wd.values())}, {again}"
    # world 1's share by worlds
    share = collections.defaultdict(list)
    for c in picks:
        for qid, q in c["questions"].items():
            if qid.startswith("worlds.c"):
                p = (c["answers"].get(qid) or {}).get("probabilities") or {}
                if p:
                    share[min(len(q["criteria"]), 4)].append(p.get("w1", 0) / (sum(p.values()) or 1))
    for k in sorted(share):
        out[f"world 1's share, {k}{'+' if k == 4 else ''} worlds"] = f"{statistics.mean(share[k]):.2f} (n {len(share[k])})"
    drawn = [s for p in plans for s in p.get("split") or []]
    out["components drawn, a change taken"] = f"{len(drawn)}, {sum(bool(s.get('taken')) for s in drawn) / max(1, len(drawn)):.0%}"
    # the gate about an answered party
    walking, reach = [], []
    for c in gates:
        answers = {g["name"][len("answer_"):] for g in c.get("groups") or [] if g["name"].startswith("answer_")}
        for qid, q in c["questions"].items():
            m = re.fullmatch(r"(party_\d+)\.answer", qid)
            noul = (c["answers"].get(qid) or {}).get("noul")
            if m and noul is not None and m.group(1) in answers:
                text = q.get("instructions") or json.dumps(q)
                (reach if "on it:" in text else walking).append(noul)
    out["gate: party needs answering, its answer on its way"] = f"{sum(v >= 0.5 for v in walking) / max(1, len(walking)):.0%} of {len(walking)}"
    out["gate: the same, its answer in reach"] = f"{sum(v >= 0.5 for v in reach) / max(1, len(reach)):.0%} of {len(reach)}"
    # the record
    defs = None
    soldiers, born, died = set(), {}, {}
    orders, hunt_attacks, hunt_secs = collections.Counter(), 0, 0
    by_sec = collections.Counter()
    lost_standing = collections.Counter()
    cf = [c["f"] for c in calls]
    members = [{u: (g["name"], g["task"].get("kind")) for g in c.get("groups") or [] for u in g["members"]} for c in calls]
    secs = collections.Counter()
    end = 0
    for line in open(d + "/record-0.jsonl"):
        r = json.loads(line)
        if "unit_defs" in r:
            defs = r["unit_defs"]
        f = r.get("f", 0)
        end = max(end, f)
        i = bisect.bisect_right(cf, f) - 1
        mem = members[i] if i >= 0 else {}
        if r.get("t") == "ev":
            if r.get("k") == "finished":
                if is_army_def(defs[r["d"]]):
                    soldiers.add(r["u"])
                born[r["u"]] = f
            if r.get("k") == "destroyed":
                died[r["u"]] = f
                if r["u"] in soldiers:
                    j = bisect.bisect_right(cf, f - 30) - 1
                    before = members[j] if j >= 0 else {}
                    g = before.get(r["u"])
                    lost_standing["answer" if g and g[0].startswith("answer_") else "group" if g else "coming or no slot"] += 1
        if r.get("t") == "s":
            for u in r["own"]:
                if u[0] in soldiers and not (u[5] & 1):
                    g = mem.get(u[0])
                    secs["lent" if g and g[0].startswith("answer_") else "with a group" if g else "coming or no slot"] += 1
                    if g and g[1] == "hunt":
                        hunt_secs += 1
        if r.get("t") == "cmd":
            for c in r["c"]:
                if c[0] in ("move", "fight", "attack") and c[1] in soldiers:
                    orders[c[0]] += 1
                    by_sec[f // 30] += 1
                    g = mem.get(c[1])
                    if c[0] == "attack" and g and g[1] == "hunt":
                        hunt_attacks += 1
    um = sum((died.get(u, end) - born[u]) / 1800 for u in born if u in soldiers)
    total = sum(orders.values())
    worst = max((sum(v for s, v in by_sec.items() if s // 60 == m) for m in range(end // 1800 + 1)), default=0)
    out["orders to soldiers a unit-minute"] = f"{total / max(1, um):.1f} ({dict(orders)})"
    out["worst minute, orders a second"] = f"{worst / 60:.0f}"
    out["the record ends at"] = f"{end // 1800}:{end // 30 % 60:02d}"
    out["hunt attack orders a hunter-second"] = f"{hunt_attacks / max(1, hunt_secs):.2f} ({hunt_attacks} over {hunt_secs})"
    all_secs = sum(secs.values()) or 1
    out["soldier-seconds: with a group / lent / coming or no slot"] = " / ".join(f"{secs[k] / all_secs:.0%}" for k in ("with a group", "lent", "coming or no slot"))
    out["soldiers lost: in a group / in an answer / coming or no slot"] = f"{lost_standing['group']} / {lost_standing['answer']} / {lost_standing['coming or no slot']}"
    # the player
    turns, groups_calls, refused = 0, 0, []
    # A game of the pianist alone has no player.
    for line in open(d + "/strategist-0.jsonl") if os.path.exists(d + "/strategist-0.jsonl") else []:
        try:
            r = json.loads(line)
        except Exception:
            continue
        if r.get("kind") == "turn":
            turns += 1
        if r.get("kind") == "tool_call":
            a = r.get("arguments")
            s = a if isinstance(a, str) else json.dumps(a)
            res = r.get("result") or ""
            groups_calls += s.count('"tool": "groups"') + (1 if r.get("tool") == "groups" else 0)
            refused += re.findall(r"(\w+): REFUSED[^\n]{0,120}", res)
    out["player turns, groups calls"] = f"{turns}, {groups_calls}"
    out["refused calls"] = f"{len(refused)}" + (": " + "; ".join(sorted(set(refused)))[:300] if refused else "")
    return out


tables = [(os.path.basename(os.path.dirname(d.rstrip("/"))).split("-", 1)[1], read(d)) for d in sys.argv[1:]]
keys = []
for _, t in tables:
    for k in t:
        if k not in keys:
            keys.append(k)
for k in keys:
    print(f"{k}")
    for name, t in tables:
        print(f"    {name:38} {t.get(k, '-')}")
