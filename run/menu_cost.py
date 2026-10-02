#!/usr/bin/env python3
"""Offline: what the rebuild's group menu (docs/design/2026-10-01-hands-rebuild.md) would cost in Jev tokens, counted on a
game's recorded gates under the same asking rule the game had: only the groups the recorded gate asked are asked.
Question lengths are the recorded ones by kind (an attack as today's whole-group question, a detachment as today's
hunt, a walk in today's short form or in the long form); tokens are fitted on the game's own gates.

usage: run/menu_cost.py run/matches/<batch>/<NN> [...]"""
import collections, json, os, re, statistics, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import requests_of
from match_read import Match

PLACE = r"spot_\d+|passage_\d+|\bhome\b"
LIMIT = 150_000  # crates/jev REQUEST_CHARS


def soldiers(e):
    u = e.get("units") if isinstance(e, dict) else e
    mm = re.search(r"\((\d+) soldiers?", u or "")
    return int(mm.group(1)) if mm else 1


for match in sys.argv[1:]:
    m = Match(match)
    gates, seen = [], set()
    for c, state, _ in requests_of(m):
        if "worlds.pick" in c["questions"] or c["f"] in seen:
            continue
        seen.add(c["f"])
        gates.append((c, state))
    size = collections.defaultdict(list)
    for c, _ in gates:
        for k, q in c["questions"].items():
            n = len(json.dumps(q)) + len(k) + 8
            kind = "whole" if ".whole_group" in k else "hunt" if ".hunt_group" in k and not k.endswith(".forbidden") else "forbidden" if k.endswith(".forbidden") else "walk" if ".walk_" in k or ".advance_" in k else "join" if ".join_" in k else "long" if k.startswith("group_") and not k.endswith(".change") else None
            if kind:
                size[kind].append(n)
    avg = {k: statistics.mean(v) for k, v in size.items()}
    xs = [(len(json.dumps(c["questions"])), c["usage"]["input_tokens"]) for c, _ in gates if c.get("usage")]
    n = len(xs); sx = sum(x for x, _ in xs); sy = sum(y for _, y in xs)
    per_char = sum((x - sx / n) * (y - sy / n) for x, y in xs) / sum((x - sx / n) ** 2 for x, _ in xs)
    total = sum(c["usage"]["input_tokens"] for c in m.calls if c.get("t") == "call" and c.get("usage"))
    rows = collections.defaultdict(lambda: [0, 0.0, 0, 0])  # questions, characters, gates over one request, gates over two
    for c, state in gates:
        actors = state.get("actors", {})
        asked = {k.split(".")[0] for k in c["questions"] if k.startswith("group_")} | {re.search(r"_(group_\w+?)(\.forbidden)?$", k).group(1) for k in c["questions"] if k.startswith("party_") and re.search(r"_(group_\w+?)(\.forbidden)?$", k)}
        groups = {k: v for k, v in actors.items() if k.startswith("group_")}
        parties = set(re.findall(r"party_\d+", json.dumps(state.get("enemy", {})) + json.dumps(actors)))
        places = set(re.findall(PLACE, state.get("instructions", "")))
        old_chars = sum(len(json.dumps(q)) + len(k) + 8 for k, q in c["questions"].items() if k.startswith(("group_", "party_")) and not k.endswith((".change", ".answer")))
        old_n = sum(1 for k in c["questions"] if k.startswith(("group_", "party_")) and not k.endswith((".change", ".answer")))
        rest = len(json.dumps(state)) + len(json.dumps(c["questions"])) - old_chars
        variants = {"as recorded": (old_n, old_chars)}
        for name, ladder_sizes, walk in (("every named place, ladder 1 2 4 8, walks short", (1, 2, 4, 8), "walk"), ("the same, walks in the long form", (1, 2, 4, 8), "long"), ("every named place, ladder 2 8, walks short", (2, 8), "walk")):
            q = ch = 0
            for g in asked & set(groups):
                size_g = soldiers(groups[g])
                ladder = sum(1 for s in ladder_sizes if s < size_g)
                q += len(parties) * (1 + ladder) + len(places) * 2 + 2 * (len(groups) - 1) + 2
                ch += len(parties) * (avg.get("whole", 600) + ladder * (avg.get("hunt", 600) + avg.get("forbidden", 500))) + len(places) * 2 * avg.get(walk, 300) + 2 * (len(groups) - 1) * avg.get("join", 600) + 2 * avg.get("long", 500)
                q += len(parties) * ladder  # the forbidden question beside each detachment
            variants[name] = (q, ch)
        for name, (q, ch) in variants.items():
            r = rows[name]
            r[0] += q; r[1] += ch
            r[2] += rest + ch > LIMIT; r[3] += rest + ch > 2 * LIMIT
    base = rows["as recorded"][1]
    print(f"{match.split('/')[2]}: {len(gates)} gates; {per_char:.3f} tokens a question character; the game's Jev bill {total / 1e6:.1f}M tokens, ${total * 0.042 / 1e6:.2f}")
    print(f"  recorded question sizes: " + ", ".join(f"{k} {v:.0f}" for k, v in sorted(avg.items())))
    for name, (q, ch, over, over2) in rows.items():
        new_total = total + (ch - base) * per_char
        print(f"  {name:52s} {q:8.0f} questions  {ch * per_char / 1e6:6.1f}M tokens  bill ${new_total * 0.042 / 1e6:5.2f}  gates in two requests or more {over:4d}, three or more {over2:4d}")
