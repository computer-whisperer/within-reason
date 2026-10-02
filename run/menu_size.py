#!/usr/bin/env python3
"""Offline: a group's menu under the rebuild's laws (docs/design/2026-10-01-hands-rebuild.md), counted on recorded gate
states: every party for every group, whole and the ladder of 1, 2, 4, 8; a join to every other group; hold and gather;
and the places both ways. Variant A: places from the group's own paragraph (exact heading, else the whole packet:
the code's reading today). Variant B: every place the packet names, for every group. Variant C, with
--reading <file from run/jev_read_ab.py>: the places Jev read as the group's at 0.5 or over, in the packet in force.

usage: run/menu_size.py run/matches/<batch>/<NN> [...] [--reading <file.jsonl>]"""
import sys, json, re, statistics
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from match_read import Match
from jev_ab import requests_of
def paragraph(text, actor):
    for p in text.split("\n\n"):
        if p.split(":")[0].split("(")[0].strip() == actor: return p
    return None
def soldiers(e):
    u = e.get("units") if isinstance(e, dict) else e
    mm = re.search(r"\((\d+) soldiers?", u or "")
    return int(mm.group(1)) if mm else 1
PLACE = r"spot_\d+|passage_\d+|\bhome\b"
reading = {}
args = sys.argv[1:]
if "--reading" in args:
    i = args.index("--reading")
    for line in open(args[i + 1]):
        r = json.loads(line)
        if r.get("noul") is not None and "actor" in r:
            reading.setdefault(r["f"], {}).setdefault(r["actor"], set())
            if r["noul"] >= 0.5:
                reading[r["f"]][r["actor"]].add(r["place"])
    del args[i:i + 2]
read_frames = sorted(reading)
for match in args:
    m = Match(match); rows = []; seen = set(); tokens = []
    for c, state, packet in requests_of(m):
        if "worlds.pick" in c["questions"] or c["f"] in seen: continue
        seen.add(c["f"])
        actors = state.get("actors", {})
        groups = {k: v for k, v in actors.items() if k.startswith("group_")}
        parties = set(re.findall(r"party_\d+", json.dumps(state.get("enemy", {})) + json.dumps(actors)))
        text = state.get("instructions", "")
        everywhere = set(re.findall(PLACE, text))
        a = b = fight = c_read = 0
        at = max((f for f in read_frames if f <= c["f"]), default=None)
        own_exact = 0
        for g, e in groups.items():
            n = soldiers(e)
            par = paragraph(text, g)
            own_exact += par is not None
            mine = set(re.findall(PLACE, par if par is not None else text))
            ladder = sum(1 for s in (1, 2, 4, 8) if s < n)
            common = len(parties) * (1 + ladder) + (len(groups) - 1) + 2
            fight += len(parties) * (1 + ladder)
            a += common + len(mine) * 2
            b += common + len(everywhere) * 2
            c_read += common + len(reading.get(at, {}).get(g, ())) * 2
        old = sum(1 for k in c["questions"] if k.startswith(("group_", "party_")) and not k.endswith((".change", ".answer", ".forbidden")))
        walks = sum(1 for k in c["questions"] if ".walk_" in k or ".advance_" in k)
        rows.append((len(groups), len(parties), len(everywhere), own_exact, fight, a, b, old, walks, len(json.dumps(c["questions"])) / max(len(c["questions"]), 1), c_read))
    def q(xs, p): xs = sorted(xs); return xs[min(len(xs) - 1, int(len(xs) * p))]
    print(match.split("/")[2], "gates", len(rows))
    for name, i in (("groups", 0), ("of them with a paragraph headed by their name alone", 3), ("parties", 1), ("places the packet names anywhere", 2), ("attack/detach questions, all groups x all parties", 4), ("A: own-paragraph places both ways + the rest", 5), ("B: every named place both ways + the rest", 6), ("recorded group+party option questions", 7), ("  of them walks and advances", 8), ("characters a recorded question", 9)) + ((("C: the places Jev read as the group's, both ways + the rest", 10),) if reading else ()):
        xs = [r[i] for r in rows]
        print(f"  {name:58s} median {statistics.median(xs):6.0f}  p90 {q(xs, .9):6.0f}  max {max(xs):6.0f}  sum {sum(xs):9.0f}")
