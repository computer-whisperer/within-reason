#!/usr/bin/env python3
"""Two cuts of a group's menu, replayed on a game's Jev logs.

1. A detachment's short menu. A detachment is a group that left another (`split_from` in its entry) and has at most
   eight soldiers (the largest detachment; the army itself often has a `split_from` from long ago). Short: it keeps
   its moves against parties (an attack, a shelling), its joins, `hold` and its `change`; it loses the walks and
   advances to places, the shelling of places, detachments of its own, the rove, the gather and the follows. Also
   counted for a group of one or two soldiers, whatever its origin.
   And for the same small groups: moves against a party only for the k parties nearest the group.
2. Joins and follows only with the k nearest groups (straight line between the groups' centres).

Counted: the question tokens that would not be sent, and the moves the pick played that the cut would have shut.

    run/jev_group_menus.py run/matches/<batch> [...]
"""
import collections, glob, json, math, re, sys


def kind_of(move):
    if move.endswith(".forbidden"): return "send"
    if move == "change" or move.startswith("change"): return "change"
    for k in ("send", "attack", "shell_party", "join", "follow", "go", "fight_to", "raid", "shell", "hold", "gather", "scout"):
        if move == k or move.startswith(k + "_"): return k
    return move.split("_")[0]


KEPT = {"change", "hold", "attack", "shell_party"}


def short_keeps(move, parent):
    k = kind_of(move)
    return k in KEPT or k == "join"


for D in sys.argv[1:]:
    total = 0; gq = 0.0
    cut = collections.Counter(); plays = collections.Counter(); shut = collections.defaultdict(collections.Counter)
    pty_tok = 0.0; pty_cut = collections.Counter(); pty_plays = 0; pty_shut = collections.Counter(); prank = {}
    jf = collections.Counter(); jf_tok = 0.0; jf_cut = collections.Counter(); jf_plays = 0; jf_shut = collections.Counter(); ranks = []
    for fn in sorted(glob.glob(D.rstrip("/") + "/00/jev-*.jsonl")):
        det = {}; size = {}; at = {}
        for l in open(fn):
            try: r = json.loads(l)
            except ValueError: continue
            t = r.get("t"); T = (r.get("usage") or {}).get("input_tokens", 0) or 0
            if t in ("call", "decode"): total += T
            if t == "call" and "questions" in r and not any(k.startswith("worlds") for k in r["questions"]):
                q = r["questions"]; st = r.get("state") or {}
                tot = (sum(len(json.dumps(v)) for v in st.values()) + 9000) * (r.get("batches", 1) or 1) + sum(len(json.dumps(v)) for v in q.values())
                for name, e in (st.get("actors") or {}).items():
                    if name.startswith("group_") and isinstance(e, dict):
                        det[name] = (e.get("split_from") or "").split(",")[0].split(" ")[0] or None
                names = [n for n in (st.get("actors") or {}) if n.startswith("group_")]; now_at = {}
                for g in r.get("groups") or []:
                    full = next((n for n in names if n == "group_" + g.get("name", "") or n.startswith("group_" + g.get("name", "") + "_t")), None)
                    if full and g.get("at"): now_at[full] = g["at"]
                    if full: size[full] = len(g["members"])
                at = now_at
                def rank(a, target):
                    if a not in at or target not in at: return None
                    mine = math.dist(at[a], at[target])
                    return sum(1 for o, p in at.items() if o not in (a, target) and math.dist(at[a], p) < mine)
                parties = {p["name"]: (p["x"], p["z"]) for p in r.get("parties") or []}
                prank = {}
                for a, here in at.items():
                    order = sorted(parties, key=lambda n: math.dist(here, parties[n]))
                    for i, n in enumerate(order): prank[(a, n)] = i
                for k, v in q.items():
                    a, _, move = k.partition(".")
                    if not a.startswith("group_"): continue
                    c = T * len(json.dumps(v)) / tot; gq += c
                    pm = re.search(r"(party_\d+\w*?)(?:\.forbidden)?$", move)
                    if pm and size.get(a, 99) <= 2:
                        pty_tok += c; rk = prank.get((a, pm.group(1)))
                        if rk is not None:
                            for n in (1, 2):
                                if rk >= n: pty_cut[n] += c
                    for label, small in (("detachment", det.get(a) is not None and size.get(a, 99) <= 8), ("one or two soldiers", size.get(a, 99) <= 2)):
                        if small and not short_keeps(move, det.get(a)): cut[label] += c
                    kk = kind_of(move)
                    if kk in ("join", "follow"):
                        jf_tok += c; target = move.split("_", 1)[1]
                        rk = rank(a, target) if target.startswith("group_") else None
                        jf["to a group" if target.startswith("group_") else "to a builder"] += c
                        if rk is not None:
                            for n in (1, 2, 3):
                                if rk >= n: jf_cut[n] += c
            elif t == "plan":
                for m in r.get("played") or []:
                    a, _, move = (m.get("played") or "").partition(".")
                    if not a.startswith("group_") or m.get("source") != "plan": continue
                    kk = kind_of(move)
                    for label, small in (("detachment", det.get(a) is not None and size.get(a, 99) <= 8), ("one or two soldiers", size.get(a, 99) <= 2)):
                        if small:
                            plays[label] += 1
                            if not short_keeps(move, det.get(a)): shut[label][kk] += 1
                    pm = re.search(r"(party_\d+\w*?)$", move)
                    if pm and size.get(a, 99) <= 2 and (a, pm.group(1)) in prank:
                        pty_plays += 1
                        for n in (1, 2):
                            if prank[(a, pm.group(1))] >= n: pty_shut[n] += 1
                    if kk in ("join", "follow"):
                        target = move.split("_", 1)[1]
                        if target.startswith("group_") and a in at and target in at:
                            mine = math.dist(at[a], at[target]); rk = sum(1 for o, p in at.items() if o not in (a, target) and math.dist(at[a], p) < mine)
                            jf_plays += 1; ranks.append(rk)
                            for n in (1, 2, 3):
                                if rk >= n: jf_shut[n] += 1
    print(f"== {D.rstrip('/').split('/')[-1][11:]}: {total/1e6:.1f}M tokens; group questions {gq/1e6:.1f}M ({100*gq/total:.0f}%)")
    for label in ("detachment", "one or two soldiers"):
        s = shut[label]
        print(f"   short menu for a {label}: {cut[label]/1e6:.2f}M not asked ({100*cut[label]/total:.1f}% of the game); of {plays[label]} moves such groups played, {sum(s.values())} shut: " + ", ".join(f"{k} {v}" for k, v in s.most_common()))
    print(f"   groups of one or two soldiers against parties: {pty_tok/1e6:.2f}M ({100*pty_tok/total:.1f}% of the game); the nearest party only: {pty_cut[1]/1e6:.2f}M not asked ({100*pty_cut[1]/total:.1f}%), {pty_shut[1]} of {pty_plays} played shut; the two nearest: {pty_cut[2]/1e6:.2f}M ({100*pty_cut[2]/total:.1f}%), {pty_shut[2]} of {pty_plays} shut")
    print(f"   joins and follows: {jf_tok/1e6:.2f}M ({100*jf_tok/total:.1f}%), to a group {jf['to a group']/1e6:.2f}M, to a builder {jf['to a builder']/1e6:.2f}M; {jf_plays} played between groups, the target's rank by distance: " + str(sorted(collections.Counter(min(r, 5) for r in ranks).items())))
    for n in (1, 2, 3):
        print(f"     the {n} nearest group{'s' if n > 1 else ''} only: {jf_cut[n]/1e6:.2f}M not asked ({100*jf_cut[n]/total:.1f}% of the game); {jf_shut[n]} of {jf_plays} played shut")
