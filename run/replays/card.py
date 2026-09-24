#!/usr/bin/env python3
"""The game card: what one replayed game says, in numbers an Opus agent can quote and a script can check
(docs/design/2026-09-23-replay-survey.md, decision 4).

    run/replays/card.py <match dir> [--manifest run/data/replays/manifest.jsonl] [--until-minutes 8]

Reads every record-<team>.jsonl in the directory (run/replay_match.py's output) and the manifest line whose demo file
name matches a demo in the directory (the players, their OS, factions and start positions: the replay records carry
no seats). Writes card.json and card.md beside the records. Per team: the build order to --until-minutes by the
clock with each extractor's spot number and grid cell, the factories with start and finish, the first tier-2 unit,
the curves by minute (extractors, constructors, metal income, energy income, army metal), the army by type every
two minutes, the first enemy seen, the first extractor lost, the losses and kills by type, the order the spots were
taken, and where the fighting was (the grid cells where most units died, by minute). Spot numbers are the record's
metal spots (the engine's centroid spots), so cards from one map speak of the same spots.
"""
import glob, json, os, sys
from collections import Counter, defaultdict

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
FPS = 30


def clock(frame):
    return f"{frame // (60 * FPS)}:{(frame % (60 * FPS)) // FPS:02d}"


class Record:
    def __init__(self, path):
        lines = open(path).readlines()
        self.header = json.loads(lines[0])
        self.defs = self.header["unit_defs"]
        self.events, self.samples, self.result = [], [], None
        for line in lines[1:]:
            o = json.loads(line)
            t = o.get("t")
            if t == "ev":
                self.events.append(o)
            elif t == "s":
                self.samples.append(o)
            elif t == "result":
                self.result = o
        self.spots = [(s[0], s[1]) for s in self.header["metal_spots"]]
        m = self.header["map"]
        g = self.header.get("grid") or {"columns": 8, "rows": 8}
        self.cell_w, self.cell_h = m["width"] / g["columns"], m["height"] / g["rows"]

    def name(self, d):
        return self.defs[d]["name"] if d is not None and 0 <= d < len(self.defs) else "?"

    def d(self, d):
        return self.defs[d] if d is not None and 0 <= d < len(self.defs) else {}

    def grid(self, x, z):
        return f"{chr(65 + min(int(x // self.cell_w), 25))}{int(z // self.cell_h) + 1}"

    def spot_of(self, x, z):
        best = min(range(len(self.spots)), key=lambda i: (self.spots[i][0] - x) ** 2 + (self.spots[i][1] - z) ** 2, default=None)
        if best is None:
            return None
        sx, sz = self.spots[best]
        return best if ((sx - x) ** 2 + (sz - z) ** 2) ** 0.5 < 130 else None

    # Roles by the record's class words (the replay widget's: commander, extractor, factory, turret, building, builder,
    # army, other; the bot's records carry the fields instead), with the fields as the fallback.
    def is_soldier(self, d):
        u = self.d(d)
        if "class" in u:
            return u["class"] == "army"
        return bool(u.get("weapons")) and (u.get("speed") or 0) > 0 and (u.get("build_speed") or 0) == 0 and "com" not in u.get("name", "")[-3:]

    def is_builder(self, d):
        u = self.d(d)
        if "class" in u:
            return u["class"] in ("builder", "commander", "factory")
        return (u.get("build_speed") or 0) > 0

    def is_constructor(self, d):
        u = self.d(d)
        return u.get("class") == "builder" if "class" in u else (u.get("build_speed") or 0) > 0 and (u.get("speed") or 0) > 0 and not u.get("name", "").endswith("com")

    def is_factory(self, d):
        u = self.d(d)
        if "class" in u:
            return u["class"] == "factory"
        return (u.get("speed") or 0) == 0 and (u.get("build_speed") or 0) > 0 and bool(u.get("builds"))

    def is_extractor(self, d):
        u = self.d(d)
        if "class" in u:
            return u["class"] == "extractor"
        return (u.get("extracts_metal") or 0) > 0

    def tier2(self, d):
        return self.d(d).get("tech", 1) >= 2 if "tech" in self.d(d) else self.name(d) in T2


T2 = {"armavp", "armalab", "armaap", "armasy", "coravp", "coralab", "coraap", "corasy", "armmoho", "cormoho", "armfus", "corfus", "armack", "armacv", "corack", "coracv"}


def secs(c):
    m, s = c.split(":")
    return int(m) * 60 + int(s)


def side_of(order, header_side):
    """The faction from what was built (the replay records' header `side` was wrong on 22 of 58 sides)."""
    prefixes = Counter(o["unit"][:3] for o in order if o["unit"][:3] in ("arm", "cor", "leg"))
    return prefixes.most_common(1)[0][0] if prefixes else header_side


def team_card(rec, until_frames):
    own_ids = set()
    role_of = {}  # unit id -> role words for the builder
    by_role = lambda by: "factory" if rec.is_factory(role_of.get(by, (None,))[0]) else role_of.get(by, ("", "?"))[1] if by in role_of else "?"
    order, factories, spots_taken = [], {}, []
    first_t2 = None
    first_t2_unit = None
    commander = None
    for e in rec.events:
        if e.get("k") == "created":
            own_ids.add(e["u"])
            d = e.get("d")
            nm = rec.name(d)
            if nm[3:4] == "_":
                continue  # a cosmetic part (arm_leftshoulder_nationwars_eec), not a build
            if nm.endswith("com") and commander is None:
                commander = e["u"]
                role_of[e["u"]] = (d, "commander")
            elif rec.is_factory(d):
                factories[e["u"]] = {"unit": nm, "started": clock(e["f"]), "finished": None, "at": rec.grid(e["x"], e["z"])}
                role_of[e["u"]] = (d, f"factory {nm}")
            elif rec.is_constructor(d):
                n = sum(1 for r in role_of.values() if r[1].startswith("constructor")) + 1
                role_of[e["u"]] = (d, f"constructor {n}")
            if first_t2 is None and rec.tier2(d):
                first_t2 = {"unit": nm, "clock": clock(e["f"])}
            spot = rec.spot_of(e["x"], e["z"]) if rec.is_extractor(d) else None
            # A spot's cell is the spot's own, not the extractor's (a snapped extractor on a cell boundary gave 28 of
            # 80 spots two cells in the first survey); every building carries its cell, so turret positions can be read.
            if spot is not None:
                spots_taken.append({"clock": clock(e["f"]), "spot": spot, "grid": rec.grid(*rec.spots[spot])})
            if first_t2_unit is None and rec.tier2(d) and not rec.is_factory(d) and not rec.d(d).get("speed", 0) == 0:
                first_t2_unit = {"unit": nm, "clock": clock(e["f"])}
            if e["f"] <= until_frames and not nm.endswith("com"):
                by = e.get("by")
                who = role_of.get(by, (None, "?"))[1] if by is not None else "?"
                entry = {"clock": clock(e["f"]), "unit": nm, "by": who}
                if spot is not None:
                    entry["spot"] = spot
                    entry["grid"] = rec.grid(*rec.spots[spot])
                elif rec.d(d).get("speed", 0) == 0:
                    entry["grid"] = rec.grid(e["x"], e["z"])
                order.append(entry)
        elif e.get("k") == "finished" and e["u"] in factories:
            factories[e["u"]]["finished"] = clock(e["f"])
    losses, kills, end_kills = Counter(), Counter(), Counter()
    end_frame = max((s["f"] for s in rec.samples), default=0) - 15 * FPS
    first_extractor_lost = None
    fights = defaultdict(lambda: {"ours": 0, "theirs": 0, "first": None, "last": None})
    for e in rec.events:
        k = e.get("k")
        if k == "destroyed":
            nm = rec.name(e.get("d"))
            losses[nm] += 1
            if rec.is_extractor(e.get("d")) and first_extractor_lost is None:
                spot = rec.spot_of(e["x"], e["z"])
                first_extractor_lost = {"clock": clock(e["f"]), "spot": spot, "grid": rec.grid(*rec.spots[spot]) if spot is not None else rec.grid(e["x"], e["z"]), "by": rec.name(e.get("by_d"))}
            cell = (e["f"] // (60 * FPS), rec.grid(e["x"], e["z"]))
            fights[cell]["ours"] += 1
        elif k == "enemy_destroyed":
            nm = rec.name(e.get("d")) if e.get("d") is not None else "?"
            if e["f"] >= end_frame:
                end_kills[nm] += 1  # the last 15 s: a resign's self-destruct looks like kills
                continue
            kills[nm] += 1
            if "x" in e:
                cell = (e["f"] // (60 * FPS), rec.grid(e["x"], e["z"]))
                fights[cell]["theirs"] += 1
    curves = []
    composition = []
    last_minute = -1
    for s in rec.samples:
        # A replay's records see everything (the widget writes them with full vision), so "first enemy seen" is 0:00
        # on every side and is not written; first contact needs the first fight instead.
        minute = s["f"] // (60 * FPS)
        if minute == last_minute or s["f"] % (60 * FPS) > 2 * FPS:
            continue
        last_minute = minute
        own = s.get("own", [])
        soldiers = [u for u in own if rec.is_soldier(u[1])]
        curves.append({
            "minute": minute,
            "extractors": sum(1 for u in own if rec.is_extractor(u[1])),
            "constructors": sum(1 for u in own if rec.is_constructor(u[1])),
            "metal_income": round(s["m"][1], 1), "metal_stored": round(s["m"][0]), "energy_income": round(s["e"][1], 1), "energy_stored": round(s["e"][0]),
            "army_metal": round(sum(rec.d(u[1]).get("metal") or 0 for u in soldiers)), "soldiers": len(soldiers),
        })
        if minute % 2 == 0:
            composition.append({"minute": minute, "army": dict(Counter(rec.name(u[1]) for u in soldiers).most_common(6))})
    hot = sorted(fights.items(), key=lambda kv: -(kv[1]["ours"] + kv[1]["theirs"]))[:8]
    fights_out = [{"minute": m, "grid": g, "ours_lost": v["ours"], "theirs_lost": v["theirs"]} for (m, g), v in sorted(hot)]
    start = None
    if rec.samples:
        for u in rec.samples[0].get("own", []):
            if rec.name(u[1]).endswith("com"):
                start = {"x": u[2], "z": u[3], "grid": rec.grid(u[2], u[3])}
    return {
        "team": rec.header.get("team"), "ally_team": rec.header.get("ally_team"), "side": side_of(order, rec.header.get("side")), "start": start,
        "build_order": order, "factories": sorted(factories.values(), key=lambda f: secs(f["started"])), "first_tier2": first_t2, "first_tier2_unit": first_t2_unit,
        "spots_taken": spots_taken, "curves": curves, "army_by_type": composition,
        "first_extractor_lost": first_extractor_lost,
        "losses": dict(losses.most_common()), "kills": dict(kills.most_common()), "end_kills": dict(end_kills.most_common()), "fights": fights_out,
    }


def markdown(card):
    out = [f"# {card['map']['name']}: {card.get('title', 'a game')}", ""]
    out.append(f"Map {card['map']['width']}x{card['map']['height']}, {card['map']['spots']} metal spots, grid {card['map']['grid']}. Duration {card['duration']}. Result: {card['result']}.")
    for p in card.get("players", []):
        out.append(f"- {p['name']} (OS {p.get('os')}, rank {p.get('rank')}, {p.get('faction')}, team {p.get('team')}){' — won' if p.get('won') else ''}")
    for t in card["teams"]:
        who = t.get("player") or f"team {t['team']}"
        out += ["", f"## {who} ({t['side']}, start {t['start']['grid'] if t['start'] else '?'})", ""]
        out.append("Build order to the cut-off: " + "; ".join(f"{o['clock']} {o['unit']}" + (f" spot_{o['spot']} {o['grid']}" if "spot" in o else "") + (f" ({o['by']})" if o["by"] not in ("?", "commander") else "") for o in t["build_order"]))
        out.append("Factories: " + ", ".join(f"{f['unit']} {f['started']}-{f['finished'] or '?'} at {f['at']}" for f in t["factories"]) + (f". First tier 2: {t['first_tier2']['unit']} at {t['first_tier2']['clock']}" + (f", first tier-2 unit {t['first_tier2_unit']['unit']} at {t['first_tier2_unit']['clock']}" if t.get("first_tier2_unit") else "") + "." if t["first_tier2"] else ". No tier 2."))
        out.append("By minute (extractors / constructors / metal income / army metal): " + " ".join(f"{c['minute']}:{c['extractors']}/{c['constructors']}/{c['metal_income']:.0f}/{c['army_metal']}" for c in t["curves"] if c["minute"] % 2 == 0))
        out.append("Army every two minutes: " + "; ".join(f"{c['minute']}: " + ", ".join(f"{n} {k}" for k, n in c["army"].items()) for c in t["army_by_type"] if c["army"]))
        if t["first_extractor_lost"]:
            out.append(f"First extractor lost {t['first_extractor_lost']['clock']} at spot_{t['first_extractor_lost']['spot']} ({t['first_extractor_lost'].get('grid', '?')}) to a {t['first_extractor_lost']['by']}.")
        out.append("Spots taken: " + " ".join(f"{s['clock']} spot_{s['spot']}({s['grid']})" for s in t["spots_taken"]))
        out.append("Lost: " + ", ".join(f"{n} {k}" for k, n in list(t["losses"].items())[:8]) + ". Killed: " + ", ".join(f"{n} {k}" for k, n in list(t["kills"].items())[:8]) + "." + (f" In the last 15 s (the end, or a resign): {sum(t['end_kills'].values())} of theirs died." if t.get("end_kills") else ""))
        out.append("Fighting (minute, cell, ours lost / theirs lost): " + "; ".join(f"{f['minute']} {f['grid']} {f['ours_lost']}/{f['theirs_lost']}" for f in t["fights"]))
    return "\n".join(out) + "\n"


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    match = os.path.abspath(args[0])
    manifest = args[args.index("--manifest") + 1] if "--manifest" in args else os.path.join(REPO, "run", "data", "replays", "manifest.jsonl")
    until = float(args[args.index("--until-minutes") + 1]) if "--until-minutes" in args else 8.0
    records = sorted(glob.glob(os.path.join(match, "record-*.jsonl")))
    if not records:
        sys.exit(f"no record-*.jsonl in {match}")
    recs = [Record(p) for p in records]
    demos = [os.path.basename(p) for p in glob.glob(os.path.join(match, "*.sdfz"))]
    entry = None
    if os.path.exists(manifest):
        for line in open(manifest):
            if line.strip():
                row = json.loads(line)
                if row.get("file") in demos:
                    entry = row
    teams = [team_card(r, int(until * 60 * FPS)) for r in recs]
    players = []
    if entry:
        for p in entry["players"]:
            won = p["ally"] == entry.get("winner_ally")
            players.append({**p, "won": won})
            for t in teams:
                if t["team"] == p["team"]:
                    t["player"] = f"{p['name']} (OS {p['os']})"
                    t["won"] = won
    h = recs[0].header
    last = max((r.samples[-1]["f"] for r in recs if r.samples), default=0)
    result = "; ".join(f"{t.get('player') or 'team ' + str(t['team'])} {'won' if t.get('won') else 'lost' if 'won' in t else '?'}" for t in teams) if entry else str(recs[0].result)
    card = {
        "match": os.path.relpath(match, REPO), "id": entry["id"] if entry else None, "title": " vs ".join(p["name"] for p in players) if players else os.path.basename(match),
        "map": {"name": h["map"]["name"], "width": h["map"]["width"], "height": h["map"]["height"], "spots": len(h["metal_spots"]), "grid": f"{(h.get('grid') or {}).get('columns', 8)}x{(h.get('grid') or {}).get('rows', 8)}"},
        "game_version": entry["game_version"] if entry else None, "duration": clock(last), "result": result, "players": players, "teams": teams,
    }
    with open(os.path.join(match, "card.json"), "w") as f:
        json.dump(card, f, indent=1)
    md = markdown(card)
    with open(os.path.join(match, "card.md"), "w") as f:
        f.write(md)
    print(f"wrote {os.path.relpath(match, REPO)}/card.json and card.md ({len(md)} chars)")


if __name__ == "__main__":
    main()
