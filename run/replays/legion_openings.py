#!/usr/bin/env python3
"""Tally the Legion sides' openings over the bar-replays games cached under run/data/bar-replays (fetched by
run/replays/bar_replays.py): the first factory and its clock, what came before it, the lab's products to 5:00 and
10:00, the second factory, the economy's shape at 10:00, the marks against the opponent's, and the result. One line a
game, then the tallies. Written 2026-11-05 for selection 6 (legion-1v1-os40; docs/knowledge/legion.md).

    run/replays/legion_openings.py [ids file]      default run/data/bar-replays/selection6.ids
"""
import collections
import json
import os
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CACHE = os.path.join(REPO, "run", "data", "bar-replays")
FACTORIES = {"leglab": "bots", "legvp": "vehicles", "legap": "air", "leghp": "hover", "legsy": "ships", "legamphlab": "amph",
             "legalab": "adv bots", "legavp": "adv vehicles", "legaap": "adv air", "leggant": "gantry",
             "armlab": "bots", "armvp": "vehicles", "armap": "air", "armhp": "hover", "armsy": "ships", "armalab": "adv bots", "armavp": "adv vehicles", "armaap": "adv air",
             "corlab": "bots", "corvp": "vehicles", "corap": "air", "corhp": "hover", "corsy": "ships", "coralab": "adv bots", "coravp": "adv vehicles", "coraap": "adv air"}
ECON = {"legmex": "mex", "legsolar": "solar", "legwin": "wind", "legadvsol": "advsol", "legnanotc": "nano", "leglht": "lht", "leghive": "hive",
        "legestor": "estor", "legmstor": "mstor", "legeconv": "conv", "legrad": "radar", "legdtr": "jaw", "legmg": "caco", "legrl": "bramble"}


def clock(frame):
    s = frame / 30
    return f"{int(s // 60)}:{int(s % 60):02d}"


def load(gid):
    m = json.load(open(os.path.join(CACHE, "matches", f"{gid}.json")))
    ev = json.load(open(os.path.join(CACHE, "events", f"{gid}.json")))
    return m, ev


def side(m, ev, tid):
    defs = {d["definitionID"]: d["definitionName"] for d in ev["unitDefinitions"]}
    name = lambda i: defs.get(i, str(i))
    fac_ids = {u["unitID"]: u for u in ev["factoryUnitCreated"] if u["teamID"] == tid}
    created = sorted((u for u in ev["unitsCreated"] if u["teamID"] == tid), key=lambda u: u["frame"])
    builds = [u for u in created if u["unitID"] not in fac_ids and name(u["definitionID"]) not in ("armcom", "corcom", "legcom")]
    products = [u for u in created if u["unitID"] in fac_ids]
    factories = [(u["frame"], FACTORIES[name(u["definitionID"])], name(u["definitionID"])) for u in builds if name(u["definitionID"]) in FACTORIES]
    first = factories[0] if factories else None
    before = collections.Counter(ECON.get(name(u["definitionID"]), name(u["definitionID"])) for u in builds if first and u["frame"] < first[0])
    second = next((f for f in factories[1:]), None)
    def prod(upto):
        return collections.Counter(name(u["definitionID"]) for u in products if u["frame"] <= upto * 1800)
    def econ(upto):
        return collections.Counter(ECON.get(name(u["definitionID"]), "") for u in builds if u["frame"] <= upto * 1800)
    marks = {}
    for s in ev.get("extraStats", []):
        if s["teamID"] == tid:
            mnt = s["frame"] // 1800
            if mnt in (5, 8, 10) and mnt not in marks:
                marks[mnt] = s
    tstats = {}
    for s in ev.get("teamStats", []):
        if s["teamID"] == tid:
            mnt = s["frame"] // 1800
            if mnt in (5, 8, 10) and mnt not in tstats:
                tstats[mnt] = s
    return dict(first=first, before=before, second=second, factories=factories, prod5=prod(5), prod10=prod(10), econ10=econ(10),
                marks=marks, tstats=tstats, n_factories=len(factories))


def main(path):
    ids = [l.strip() for l in open(path) if l.strip()]
    rows = []
    for gid in ids:
        try:
            m, ev = load(gid)
        except FileNotFoundError:
            print(f"{gid[:8]} not cached")
            continue
        teams = {t["teamID"]: t for t in m["teams"]}
        players = {p["teamID"]: p for p in m["players"]}
        won = {a["allyTeamID"]: a.get("won") for a in m.get("allyTeams", [])}
        md = m.get("mapData") or {}
        for tid, t in sorted(teams.items()):
            if t["faction"] != "Legion":
                continue
            opp = next((o for o in teams.values() if o["teamID"] != tid), None)
            s = side(m, ev, tid)
            so = side(m, ev, opp["teamID"]) if opp else None
            rows.append(dict(gid=gid, map=m["map"], wind=(md.get("minimumWind"), md.get("maximumWind")), date=m["startTime"][:10], ver=m["gameVersion"].split("test-")[-1][:5],
                             dur=m["durationMs"] // 60000, player=players[tid]["name"], os=players[tid]["skill"], won=won.get(t["allyTeamID"]),
                             opp=(opp["faction"], players[opp["teamID"]]["name"], players[opp["teamID"]]["skill"]) if opp else None, s=s, so=so))
    print("game | map (wind) | date ver | Legion player OS | won | opp | first factory clock, what came before | second | lab products to 5:00 / 10:00 | econ at 10:00 | army value 5/8/10 ours:his | metal produced 10:00 ours:his")
    for r in rows:
        s, so = r["s"], r["so"]
        f = s["first"]
        first = f"{f[1]} {clock(f[0])} after {dict(s['before'])}" if f else "no factory"
        second = f"{s['second'][1]} {clock(s['second'][0])}" if s["second"] else "-"
        p5 = ", ".join(f"{k} {v}" for k, v in s["prod5"].most_common(4))
        p10 = ", ".join(f"{k} {v}" for k, v in s["prod10"].most_common(5))
        e10 = {k: v for k, v in s["econ10"].items() if k}
        av = "/".join(f"{s['marks'][mn]['armyValue']}:{so['marks'][mn]['armyValue'] if so and mn in so['marks'] else '?'}" if mn in s["marks"] else "?" for mn in (5, 8, 10))
        mp = f"{s['tstats'][10]['metalProduced']:.0f}:{so['tstats'][10]['metalProduced']:.0f}" if 10 in s["tstats"] and so and 10 in so["tstats"] else "?"
        of = so["first"] if so else None
        oppf = f"{r['opp'][0]} {r['opp'][1]} {r['opp'][2]:.0f} ({of[1]} {clock(of[0])})" if r["opp"] and of else (r["opp"][0] if r["opp"] else "?")
        print(f"{r['gid'][:8]} | {r['map']} {r['wind']} | {r['date']} {r['ver']} | {r['player']} {r['os']:.1f} | {'W' if r['won'] else 'L'} {r['dur']}m | {oppf} | {first} | {second} | {p5} / {p10} | {e10} | {av} | {mp}")
    print()
    print("Legion sides:", len(rows), "won", sum(1 for r in rows if r["won"]))
    print("first factory:", collections.Counter(r["s"]["first"][1] if r["s"]["first"] else "none" for r in rows))
    print("first factory clock (median):", sorted(clock(r["s"]["first"][0]) for r in rows if r["s"]["first"]))
    print("won by first factory:", collections.Counter((r["s"]["first"][1] if r["s"]["first"] else "none", bool(r["won"])) for r in rows))
    print("second factory:", collections.Counter(r["s"]["second"][1] if r["s"]["second"] else "none" for r in rows))
    print("first lab product:", collections.Counter(next(iter(r["s"]["prod5"].elements()), "-") if r["s"]["prod5"] else "-" for r in rows))
    print("Goblins by 10:00 (bot openers):", sorted(r["s"]["prod10"].get("leggob", 0) for r in rows if r["s"]["first"] and r["s"]["first"][1] == "bots"))
    print("Alaris by 10:00 (vehicle openers):", sorted(r["s"]["prod10"].get("leghades", 0) for r in rows if r["s"]["first"] and r["s"]["first"][1] == "vehicles"))
    print("wind by 10:00 / solar by 10:00:", [(r["s"]["econ10"].get("wind", 0), r["s"]["econ10"].get("solar", 0), r["wind"][1]) for r in rows])
    print("maps:", collections.Counter(r["map"] for r in rows))
    print("players:", collections.Counter(r["player"] for r in rows))
    print("opponent factions:", collections.Counter(r["opp"][0] for r in rows if r["opp"]))
    print("hives by 10:00:", sum(r["s"]["econ10"].get("hive", 0) for r in rows), "in", sum(1 for r in rows if r["s"]["econ10"].get("hive", 0)), "games")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else os.path.join(CACHE, "selection6.ids"))
