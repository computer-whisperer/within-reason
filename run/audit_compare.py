#!/usr/bin/env python3
"""Two audit dumps of one game against each other, row by row: the one the audit widget writes in a replay
(audit_dump.lua) and the one `target/release/tape audit` writes from the game's tape (docs/harness/audit.md, the two
dumps against each other). Make both in one replay with both widgets:

    run/replay_match.py <demo> --label <name> --tape --audit --no-records
    target/release/tape pack run/matches/<time>-replay-<name>/tape.raw --check
    target/release/tape audit run/matches/<time>-replay-<name>/game.tape -o .../tape-audit.jsonl
    run/audit_compare.py .../audit.jsonl .../tape-audit.jsonl

It prints, per kind of row, what differs beyond the last figure: the unit rows column by column (x, z, health and build
within 1, experience within 0.001), the teams' numbers and statistics, the damage taken, the events and their fields,
the commands, the wrecks and the end.
"""
import json, sys, collections
COLS = ["id", "def", "team", "x", "z", "health", "build", "queue", "cmd", "tx", "tz", "target", "builds", "power", "lined", "seen", "xp"]
def load(path):
    rows = collections.defaultdict(list)
    for line in open(path):
        r = json.loads(line)
        rows[r["t"]].append(r)
    return rows
a, b = load(sys.argv[1]), load(sys.argv[2])
print("rows by kind:", {k: (len(a[k]), len(b[k])) for k in sorted(set(a) | set(b))})
sa, sb = {r["f"]: r for r in a["s"]}, {r["f"]: r for r in b["s"]}
print("sample frames: widget", min(sa), "-", max(sa), len(sa), "| tape", min(sb), "-", max(sb), len(sb), "| common", len(set(sa) & set(sb)))
miss = collections.Counter(); total = 0; only = collections.Counter(); worst = collections.defaultdict(float); example = {}
side_diff = collections.defaultdict(float); side_n = 0; st_diff = collections.Counter(); hits_diff = collections.Counter()
for f in sorted(set(sa) & set(sb)):
    ua = {u[0]: u for u in sa[f]["u"]}; ub = {u[0]: u for u in sb[f]["u"]}
    only["widget only"] += len(set(ua) - set(ub)); only["tape only"] += len(set(ub) - set(ua))
    for uid in set(ua) & set(ub):
        total += 1
        for i, name in enumerate(COLS):
            x, y = ua[uid][i], ub[uid][i]
            if x != y:
                d = abs(x - y) if isinstance(x, (int, float)) and isinstance(y, (int, float)) else 1
                tol = {"x": 1, "z": 1, "health": 1, "build": 1, "xp": 1e-3}.get(name, 0)
                if d > tol:
                    miss[name] += 1
                    if d > worst[name]: worst[name] = d; example[name] = (f, uid, x, y)
    for s1, s2 in zip(sa[f]["sides"], sb[f]["sides"]):
        side_n += 1
        for kind in ("m", "e"):
            for i, (x, y) in enumerate(zip(s1[kind], s2[kind])):
                side_diff[(kind, i)] = max(side_diff[(kind, i)], abs(x - y))
        for k in set(s1.get("st", {})) | set(s2.get("st", {})):
            if abs((s1.get("st", {}).get(k) or 0) - (s2.get("st", {}).get(k) or 0)) > 1: st_diff[k] += 1
    ha = {h[0]: h[1] for h in sa[f].get("hits", [])}; hb = {h[0]: h[1] for h in sb[f].get("hits", [])}
    for v in set(ha) | set(hb):
        if abs(ha.get(v, 0) - hb.get(v, 0)) > 1: hits_diff["differ"] += 1
        else: hits_diff["same"] += 1
print(f"unit rows compared: {total}; {dict(only)}")
for name in COLS:
    if miss[name]: print(f"  {name}: {miss[name]} differ ({100 * miss[name] / total:.2f} %), worst {worst[name]} e.g. frame {example[name][0]} unit {example[name][1]}: widget {example[name][2]} tape {example[name][3]}")
print("  every other column the same (x, z, health, build within 1, xp within 0.001)")
print("sides, the largest difference by column:", {f"{k}{i}": round(v, 2) for (k, i), v in sorted(side_diff.items()) if v > 0.05})
print("team statistics differing by more than 1:", dict(st_diff) or "none")
print("damage taken per unit per second:", dict(hits_diff))
def keyed(rows, key): return collections.Counter(key(r) for r in rows)
ea = keyed(a["ev"], lambda r: (r["k"], r["u"], r["f"])); eb = keyed(b["ev"], lambda r: (r["k"], r["u"], r["f"]))
print("events: widget", sum(ea.values()), "tape", sum(eb.values()), "| only the widget's", sum((ea - eb).values()), list((ea - eb))[:5], "| only the tape's", sum((eb - ea).values()), list((eb - ea))[:5])
evb = {(r["k"], r["u"], r["f"]): r for r in b["ev"]}
fields = collections.Counter()
for r in a["ev"]:
    o = evb.get((r["k"], r["u"], r["f"]))
    if not o: continue
    for k in r:
        if k in ("t",): continue
        x, y = r.get(k), o.get(k)
        if x != y and not (isinstance(x, (int, float)) and isinstance(y, (int, float)) and abs(x - y) <= 1): fields[(r["k"], k)] += 1
print("event fields that differ:", dict(fields) or "none")
ca = keyed(a["cmd"], lambda r: (r["f"], r["u"], r["c"])); cb = keyed(b["cmd"], lambda r: (r["f"], r["u"], r["c"]))
print("commands: widget", sum(ca.values()), "tape", sum(cb.values()), "| only the widget's", sum((ca - cb).values()), list(ca - cb)[:4], "| only the tape's", sum((cb - ca).values()), list(cb - ca)[:4])
wa = {r["f"]: {w[0]: w[4] for w in r["w"]} for r in a["wrecks"]}; wb = {r["f"]: {w[0]: w[4] for w in r["w"]} for r in b["wrecks"]}
wd = collections.Counter()
for f in set(wa) & set(wb):
    for fid in set(wa[f]) | set(wb[f]):
        wd["same" if abs(wa[f].get(fid, -9) - wb[f].get(fid, -9)) <= 1 else "differ"] += 1
print("wrecks:", len(wa), "looks by the widget,", len(wb), "by the tape;", dict(wd))
print("over:", a["over"], b["over"])
