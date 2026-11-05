#!/usr/bin/env python3
"""Read a game processed by the bar-replays tool (http://b-dtop.b.gecko.network:8770, a friend's processing of the
public replay bucket: the engine replay's events in a DuckDB behind a Gex match page) and print each side's opening
in the card's shape: builds by clock, factory output, counts, the first extractor lost, deaths by minute with their
killers, and the team-stat marks. Fetched JSON is cached under run/data/bar-replays/.

    run/replays/bar_replays.py <game id> [minutes]          one game (default: the first 8 minutes of builds)
    run/replays/bar_replays.py --selection <n>              list the games of a selection page with their factions

The site's selection preview (`/selections/new?where=...`) counts catalogue games read-only; enqueueing (`POST
/selections`) spends the friend's CPU and is done by hand on the user's word (2026-11-05: selection 6,
legion-1v1-os40). Written 2026-11-05 for the Legion duels (docs/knowledge/legion.md, K-leg-open-os49-canis-river).
"""
import collections
import json
import os
import re
import sys
import urllib.request

HOST = os.environ.get("BAR_REPLAYS", "http://b-dtop.b.gecko.network:8770")
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CACHE = os.path.join(REPO, "run", "data", "bar-replays")


def get(url, path):
    if not os.path.exists(path):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with urllib.request.urlopen(url, timeout=120) as r:
            open(path, "wb").write(r.read())
    return json.load(open(path))


def match(gid):
    return get(f"{HOST}/api/match/{gid}?includeTeams=true&includePlayers=true&includeAllyTeams=true",
               os.path.join(CACHE, "matches", f"{gid}.json"))


def events(gid):
    return get(f"{HOST}/api/game-event/{gid}?includeTeamStats=true&includeUnitsKilled=true",
               os.path.join(CACHE, "events", f"{gid}.json"))


def clock(frame):
    s = frame / 30
    return f"{int(s // 60)}:{int(s % 60):02d}"


def opening(gid, minutes=8):
    m, ev = match(gid), events(gid)
    defs = {d["definitionID"]: d["definitionName"] for d in ev["unitDefinitions"]}
    name = lambda i: defs.get(i, str(i))
    teams = {t["teamID"]: t for t in m["teams"]}
    players = {p["teamID"]: p for p in m["players"]}
    won = {a["allyTeamID"]: a.get("won") for a in m.get("allyTeams", [])}
    md = m.get("mapData") or {}
    print(f"== {gid} {m['map']} {m['startTime']} {m['gameVersion']} {m['durationMs'] // 60000} min; "
          f"wind {md.get('minimumWind')}-{md.get('maximumWind')}, tidal {md.get('tidalStrength')}")
    for tid, t in sorted(teams.items()):
        p = players.get(tid, {})
        print(f"team {tid} {t['faction']} {p.get('name')} OS {p.get('skill', 0):.1f} won={won.get(t['allyTeamID'])}")
    cut = minutes * 60 * 30
    commanders = {"armcom", "corcom", "legcom"}
    created = collections.defaultdict(list)
    for u in ev["unitsCreated"]:
        if u["frame"] <= cut:
            created[u["teamID"]].append(u)
    fac = collections.defaultdict(list)
    for u in ev["factoryUnitCreated"]:
        if u["frame"] <= cut:
            fac[u["teamID"]].append(u)
    for tid in sorted(teams):
        fac_ids = {u["unitID"] for u in fac[tid]}
        rows = [u for u in created[tid] if u["unitID"] not in fac_ids and name(u["definitionID"]) not in commanders]
        print(f"-- team {tid} {teams[tid]['faction']}: builds (not from a factory) to {minutes}:00")
        print("   " + ", ".join(f"{clock(u['frame'])} {name(u['definitionID'])}" for u in rows))
        print(f"-- team {tid}: factory output to {minutes}:00")
        print("   " + ", ".join(f"{clock(u['frame'])} {name(u['definitionID'])}" for u in fac[tid]))
        c = collections.Counter(name(u["definitionID"]) for u in created[tid])
        print("   counts: " + ", ".join(f"{k} {v}" for k, v in c.most_common(25)))
    first_mex, per, pairs = {}, collections.defaultdict(collections.Counter), collections.Counter()
    for k in ev["unitsKilled"]:
        mnt, d, a = k["frame"] // 1800, name(k["definitionID"]), name(k.get("attackerDefinitionID"))
        per[(k["teamID"], mnt)][d] += 1
        pairs[(a, d)] += 1
        if d.endswith("mex") and k["teamID"] not in first_mex:
            first_mex[k["teamID"]] = clock(k["frame"])
    print("-- first extractor lost:", first_mex)
    for tid in sorted(teams):
        print(f"-- team {tid} deaths by minute:",
              {mnt: dict(per[(tid, mnt)].most_common(4)) for mnt in range(0, 61) if per[(tid, mnt)]})
    print("-- killer -> victim, top:", pairs.most_common(12))
    for label, key in (("team stats (metalProduced, energyProduced, unitsProduced)", "teamStats"),
                       ("value (army, eco, defense, buildPowerAvailable/Used)", "extraStats")):
        marks = collections.defaultdict(dict)
        for s in ev.get(key, []):
            mnt = s["frame"] // 1800
            if mnt in (3, 5, 8, 10, 12, 15, 20) and mnt not in marks[s["teamID"]]:
                marks[s["teamID"]][mnt] = s
        for tid in sorted(teams):
            print(f"-- team {tid} {label}:")
            for mnt, s in sorted(marks[tid].items()):
                if key == "teamStats":
                    print(f"   {mnt}: {s.get('metalProduced', 0):.0f} {s.get('energyProduced', 0):.0f} {s.get('unitsProduced')}")
                else:
                    print(f"   {mnt}: {s.get('armyValue')} {s.get('ecoValue')} {s.get('defenseValue')} "
                          f"{s.get('buildPowerAvailable')}/{s.get('buildPowerUsed')}")


def selection(n):
    with urllib.request.urlopen(f"{HOST}/selection/{n}", timeout=60) as r:
        page = r.read().decode()
    ids = list(dict.fromkeys(re.findall(r"/match/([0-9a-f]{32})", page)))
    print(f"selection {n}: {len(ids)} games listed (the page shows at most the newest 500)")
    for gid in ids:
        try:
            m = match(gid)
        except Exception as e:  # noqa: BLE001 - a game not yet ingested has no match record
            print(gid, "not readable:", e)
            continue
        print(gid, m["map"], m["startTime"][:10], m["durationMs"] // 60000, "min",
              [(t["faction"], players_of(m).get(t["teamID"], {}).get("name"), round(players_of(m).get(t["teamID"], {}).get("skill", 0), 1)) for t in m["teams"]])


def players_of(m):
    return {p["teamID"]: p for p in m["players"]}


if __name__ == "__main__":
    if len(sys.argv) >= 3 and sys.argv[1] == "--selection":
        selection(int(sys.argv[2]))
    elif len(sys.argv) >= 2:
        opening(sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 8)
    else:
        print(__doc__)
