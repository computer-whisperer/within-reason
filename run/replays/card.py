#!/usr/bin/env python3
"""The game card: what one replayed game says, in numbers an Opus agent can quote and a script can check
(docs/design/2026-09-23-replay-survey.md, decision 4).

    run/replays/card.py <match dir> [--manifest run/data/replays/manifest.jsonl] [--until-minutes 8]

Reads every record-<team>.jsonl in the directory (run/replay_match.py's output), the start script the engine wrote
beside them (`_script.txt`: who sat in every team, a person with OS and rank or an AI with its short name, the ally
teams, the handicaps; run/replays/startscript.py) and the manifest line whose demo file name matches a demo in the
directory (the API's start positions and winner; the winner is the record's `winners` when the dump widget wrote
one). Writes card.json and card.md beside the records. Per team: the seat (name, OS, AI, handicap, lane), the build
order to --until-minutes by the clock with each extractor's spot number and grid cell, the factories with start and
finish and their kinds, the first air plant and the first advanced plant, the first tier-2 unit, the curves by
minute (extractors, constructors, metal income, energy income, army metal, and in replays from 2026-10-22 the
cumulative metal and energy sent to and received from the side), the army by type every two minutes, the first
extractor lost, the losses and kills by type, what was built over the whole game by type, the units given to and
taken from other seats, the players' chat, the order the spots were taken, and where the fighting was (the grid
cells where most units died, by minute). Spot numbers are the record's metal spots (the engine's centroid spots),
so cards from one map speak of the same spots. Lanes: the seats of a side ranked along the axis their starts spread
most on (west to east, or north to south), 1 first; a team game's seats read as lanes that way.
"""
import glob, json, os, re, sys
from collections import Counter, defaultdict

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import startscript  # noqa: E402

# The game's player list writes a seat's gifts, take-overs and asks into the side's chat as
# `ui.playersList.chat.<kind>:field=value:...` (giveMetal and giveEnergy with the amount and the receiver's name,
# takeTeamAmount when a seat takes a departed player's units and stores, needEnergy, needSupport).
UI_LINE = re.compile(r"ui\.playersList\.chat\.(\w+)((?::\w+=[^:]*)*)")


def _num(v):
    try:
        return int(float(v))
    except (TypeError, ValueError):
        return 0

FPS = 30
FACTORY_KINDS = {"lab": "bots", "vp": "vehicles", "ap": "air", "hp": "hovers", "sy": "ships", "fhp": "hovers", "plat": "seaplanes", "amsub": "amphibious",
                 "alab": "advanced bots", "avp": "advanced vehicles", "aap": "advanced air", "asy": "advanced ships", "shltx": "gantry", "gant": "gantry", "shltxuw": "gantry"}


def factory_kind(name):
    """"vehicles", "advanced air", "gantry": the plant's kind from its name without the faction prefix."""
    base = name[3:] if name[:3] in ("arm", "cor", "leg") else name
    return FACTORY_KINDS.get(base, base)


def clock(frame):
    return f"{frame // (60 * FPS)}:{(frame % (60 * FPS)) // FPS:02d}"


class Record:
    def __init__(self, path):
        self.path = path
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


NEAR_HIS_START = 1500.0


def enemy_start(rec, truth_path=None):
    """Where his commander stood first: a replay's records see everything, so the first sample's enemies carry it;
    one of our own records needs the truth file beside it (`truth-<ai>.jsonl`, the arena's), else nothing."""
    for s in rec.samples[:3]:
        for e in s.get("en", []):
            if rec.name(e[1]).endswith("com"):
                return (e[2], e[3])
    if truth_path and os.path.exists(truth_path):
        with open(truth_path) as fh:
            for line in fh:
                o = json.loads(line)
                for e in o.get("enemy", []):
                    if str(e[1]).endswith("com"):
                        return (e[2], e[3])
                if o.get("f", 0) > 60 * FPS:
                    break
    return None


def pressure(rec, his_start, our_start):
    """The first army's pressure on him (the players, 2026-10-03: "the first handful of units need to run to the
    other side of the map to harass the opponent"): the first soldier in his half (nearer his start than ours), the
    first within `NEAR_HIS_START` of his start, the first building and the first extractor of his killed, and by
    minute the army metal and soldiers standing in his half. None of it when his start is unknown."""
    out = {"first_soldier_in_his_half": None, "first_soldier_near_his_start": None, "first_his_building_killed": None, "first_his_extractor_killed": None, "in_his_half": {}}
    if his_start is None or our_start is None:
        return out
    hx, hz = his_start
    ox, oz = our_start
    his_half = lambda x, z: (x - hx) ** 2 + (z - hz) ** 2 < (x - ox) ** 2 + (z - oz) ** 2
    near = lambda x, z: ((x - hx) ** 2 + (z - hz) ** 2) ** 0.5 < NEAR_HIS_START
    last_minute = -1
    for s in rec.samples:
        soldiers = [u for u in s.get("own", []) if rec.is_soldier(u[1])]
        there = [u for u in soldiers if his_half(u[2], u[3])]
        if out["first_soldier_in_his_half"] is None and there:
            u = there[0]
            out["first_soldier_in_his_half"] = {"clock": clock(s["f"]), "unit": rec.name(u[1]), "grid": rec.grid(u[2], u[3]), "soldiers": len(there)}
        if out["first_soldier_near_his_start"] is None and any(near(u[2], u[3]) for u in soldiers):
            u = next(u for u in soldiers if near(u[2], u[3]))
            out["first_soldier_near_his_start"] = {"clock": clock(s["f"]), "unit": rec.name(u[1]), "grid": rec.grid(u[2], u[3])}
        minute = s["f"] // (60 * FPS)
        if minute != last_minute and s["f"] % (60 * FPS) <= 2 * FPS:
            last_minute = minute
            out["in_his_half"][minute] = {"army_metal_in_his_half": round(sum(rec.d(u[1]).get("metal") or 0 for u in there)), "soldiers_in_his_half": len(there)}
    for e in rec.events:
        if e.get("k") != "enemy_destroyed" or e.get("d") is None or "x" not in e:
            continue
        d = rec.d(e["d"])
        if (d.get("speed") or 0) > 0:
            continue
        if out["first_his_building_killed"] is None:
            out["first_his_building_killed"] = {"clock": clock(e["f"]), "unit": rec.name(e["d"]), "grid": rec.grid(e["x"], e["z"])}
        if out["first_his_extractor_killed"] is None and rec.is_extractor(e["d"]):
            spot = rec.spot_of(e["x"], e["z"])
            out["first_his_extractor_killed"] = {"clock": clock(e["f"]), "spot": spot, "grid": rec.grid(e["x"], e["z"])}
        if out["first_his_building_killed"] and out["first_his_extractor_killed"]:
            break
    return out


def team_card(rec, until_frames):
    own_ids = set()
    role_of = {}  # unit id -> role words for the builder
    by_role = lambda by: "factory" if rec.is_factory(role_of.get(by, (None,))[0]) else role_of.get(by, ("", "?"))[1] if by in role_of else "?"
    order, factories, spots_taken = [], {}, []
    first_t2 = None
    first_t2_unit = None
    commander = None
    built = Counter()
    given, taken = [], []
    chat, gifts, took_over, ui_lines = [], [], [], []
    for e in rec.events:
        k = e.get("k")
        if k == "given":
            given.append({"clock": clock(e["f"]), "unit": rec.name(e.get("d")), "from_team": e.get("from_team")})
        elif k == "taken":
            taken.append({"clock": clock(e["f"]), "unit": rec.name(e.get("d")), "to_team": e.get("to_team")})
        elif k == "chat":
            # The player list's lines (UI_LINE) are the deliberate transfers, told apart from the engine's cumulative
            # figure, which counts a full store's overflow too; the players' own words stay in `chat`.
            text = e.get("text", "")
            m = UI_LINE.search(text)
            if not m:
                chat.append({"clock": clock(e["f"]), "text": text})
                continue
            kind, fields = m.group(1), dict(re.findall(r":(\w+)=([^:]*)", m.group(2)))
            if kind in ("giveMetal", "giveEnergy"):
                gifts.append({"clock": clock(e["f"]), "resource": "metal" if kind == "giveMetal" else "energy", "amount": _num(fields.get("amount")), "to": fields.get("name")})
            elif kind == "takeTeamAmount":
                took_over.append({"clock": clock(e["f"]), "from": fields.get("name"), "units": _num(fields.get("units")), "metal": _num(fields.get("metal")), "energy": _num(fields.get("energy"))})
            else:
                ui_lines.append({"clock": clock(e["f"]), "kind": kind, **fields})
        if k == "created":
            own_ids.add(e["u"])
            d = e.get("d")
            nm = rec.name(d)
            if nm[3:4] == "_":
                continue  # a cosmetic part (arm_leftshoulder_nationwars_eec), not a build
            if not nm.endswith("com"):
                built[nm] += 1
            if nm.endswith("com") and commander is None:
                commander = e["u"]
                role_of[e["u"]] = (d, "commander")
            elif rec.is_factory(d):
                factories[e["u"]] = {"unit": nm, "kind": factory_kind(nm), "started": clock(e["f"]), "finished": None, "at": rec.grid(e["x"], e["z"])}
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
        row = {
            "minute": minute,
            "extractors": sum(1 for u in own if rec.is_extractor(u[1])),
            "constructors": sum(1 for u in own if rec.is_constructor(u[1])),
            "metal_income": round(s["m"][1], 1), "metal_stored": round(s["m"][0]), "energy_income": round(s["e"][1], 1), "energy_stored": round(s["e"][0]),
            "army_metal": round(sum(rec.d(u[1]).get("metal") or 0 for u in soldiers)), "soldiers": len(soldiers),
        }
        if s.get("tr"):
            # The engine's cumulative team statistics (the dump widget from 2026-10-22): sent and received by the side's
            # sharing, by hand or by a full store's overflow, and units handed over.
            tr = s["tr"]
            row.update({"metal_sent": tr[0], "metal_received": tr[1], "energy_sent": tr[2], "energy_received": tr[3], "units_sent": tr[4], "units_received": tr[5]})
        curves.append(row)
        if minute % 2 == 0:
            composition.append({"minute": minute, "army": dict(Counter(rec.name(u[1]) for u in soldiers).most_common(6))})
    hot = sorted(fights.items(), key=lambda kv: -(kv[1]["ours"] + kv[1]["theirs"]))[:8]
    fights_out = [{"minute": m, "grid": g, "ours_lost": v["ours"], "theirs_lost": v["theirs"]} for (m, g), v in sorted(hot)]
    start = None
    if rec.samples:
        for u in rec.samples[0].get("own", []):
            if rec.name(u[1]).endswith("com"):
                start = {"x": u[2], "z": u[3], "grid": rec.grid(u[2], u[3])}
    press = pressure(rec, enemy_start(rec), (start["x"], start["z"]) if start else None)
    for c in curves:
        c.update(press["in_his_half"].get(c["minute"], {}))
    plants = sorted(factories.values(), key=lambda f: secs(f["started"]))
    first_air = next((f for f in plants if f["kind"] in ("air", "advanced air", "seaplanes")), None)
    first_advanced = next((f for f in plants if f["kind"].startswith("advanced") or f["kind"] == "gantry"), None)
    last = curves[-1] if curves else {}
    transfers = None
    if "metal_sent" in last:
        # The engine's sent figure split by the store the second before: at or above nine tenths of storage the increment
        # is the full store's overflow to the side (the share level is 0.99 for metal and 0.95 for energy by default),
        # below it a gift by hand. The player list's gift lines are the exact reading for people; AIs write none.
        split = {"metal": [0, 0], "energy": [0, 0]}  # by hand, overflow
        prev = None
        for smp in rec.samples:
            if prev is not None and smp.get("tr") and prev.get("tr"):
                for i, res, key in ((0, "metal", "m"), (2, "energy", "e")):
                    d = smp["tr"][i] - prev["tr"][i]
                    if d > 0:
                        store = prev.get(key) or [0, 0, 0, 0]
                        split[res][1 if store[3] > 0 and store[0] >= 0.9 * store[3] else 0] += d
            prev = smp
        first_sent = next((c["minute"] for c in curves if c.get("metal_sent", 0) + c.get("energy_sent", 0) >= 500), None)
        first_received = next((c["minute"] for c in curves if c.get("metal_received", 0) + c.get("energy_received", 0) >= 500), None)
        # The totals from the last sample, not the last whole minute: a seat gave 4,000 metal away in a game's last 48 s.
        end = next((smp["tr"] for smp in reversed(rec.samples) if smp.get("tr")), None) or [last["metal_sent"], last["metal_received"], last["energy_sent"], last["energy_received"], last["units_sent"], last["units_received"]]
        transfers = {"metal_sent": end[0], "metal_received": end[1], "energy_sent": end[2], "energy_received": end[3],
                     "units_sent": end[4], "units_received": end[5], "first_minute_sent": first_sent, "first_minute_received": first_received,
                     "metal_sent_by_hand": split["metal"][0], "metal_sent_overflow": split["metal"][1], "energy_sent_by_hand": split["energy"][0], "energy_sent_overflow": split["energy"][1],
                     "metal_given": sum(g["amount"] for g in gifts if g["resource"] == "metal"), "energy_given": sum(g["amount"] for g in gifts if g["resource"] == "energy"),
                     "gifts": len(gifts), "first_gift": gifts[0]["clock"] if gifts else None}
    return {
        "team": rec.header.get("team"), "ally_team": rec.header.get("ally_team"), "side": side_of(order, rec.header.get("side")), "start": start,
        "build_order": order, "factories": plants, "first_air_plant": first_air, "first_advanced_plant": first_advanced, "first_tier2": first_t2, "first_tier2_unit": first_t2_unit,
        "spots_taken": spots_taken, "curves": curves, "army_by_type": composition,
        "first_extractor_lost": first_extractor_lost,
        "pressure": {k: v for k, v in press.items() if k != "in_his_half"},
        "losses": dict(losses.most_common()), "kills": dict(kills.most_common()), "end_kills": dict(end_kills.most_common()), "fights": fights_out,
        "built": dict(built.most_common()), "transfers": transfers, "units_given": given, "units_taken": taken, "chat": chat,
        "gifts": gifts, "took_over": took_over, "ui_lines": ui_lines,
    }


def lanes(teams):
    """Rank every side's seats along the axis its starts spread most on: `lane` 1..n and `lane_axis` ("west to
    east" or "north to south") on each team. One seat a side gets lane 1."""
    by_ally = defaultdict(list)
    for t in teams:
        if t.get("start"):
            by_ally[t.get("ally_team")].append(t)
    for ally, side in by_ally.items():
        xs = [t["start"]["x"] for t in side]
        zs = [t["start"]["z"] for t in side]
        axis = "x" if (max(xs) - min(xs)) >= (max(zs) - min(zs)) else "z"
        for i, t in enumerate(sorted(side, key=lambda t: t["start"][axis])):
            t["lane"] = i + 1
            t["lane_axis"] = "west to east" if axis == "x" else "north to south"


def markdown(card):
    out = [f"# {card['map']['name']}: {card.get('title', 'a game')}", ""]
    out.append(f"Map {card['map']['width']}x{card['map']['height']}, {card['map']['spots']} metal spots, grid {card['map']['grid']}. Duration {card['duration']}. Result: {card['result']}.")
    for s in card.get("sides", []):
        out.append(f"- Side {s['ally']}: {s['description']}" + (f"; handicap +{s['handicap']}" if s.get("handicap") else "") + (" — won" if s.get("won") else " — lost" if s.get("won") is False else ""))
    for p in card.get("players", []):
        who = f"AI {p['ai']}" if p.get("ai") else f"OS {p.get('os')}, rank {p.get('rank')}"
        out.append(f"- {p['name']} ({who}, {p.get('faction')}, team {p.get('team')}, side {p.get('ally')}" + (f", lane {p['lane']}" if p.get("lane") else "") + f"){' — won' if p.get('won') else ''}")
    for t in card["teams"]:
        who = t.get("player") or f"team {t['team']}"
        out += ["", f"## {who} ({t['side']}, start {t['start']['grid'] if t['start'] else '?'}" + (f", lane {t['lane']} {t['lane_axis']}" if t.get("lane") else "") + ")", ""]
        seat = []
        if t.get("first_air_plant"):
            seat.append(f"air plant {t['first_air_plant']['unit']} begun {t['first_air_plant']['started']}")
        if t.get("first_advanced_plant"):
            seat.append(f"advanced plant {t['first_advanced_plant']['unit']} begun {t['first_advanced_plant']['started']}")
        tr = t.get("transfers")
        if tr:
            seat.append(f"sent {tr['metal_sent']:,} metal and {tr['energy_sent']:,} energy to the side" + (f" from minute {tr['first_minute_sent']}" if tr.get("first_minute_sent") is not None else "")
                        + f", received {tr['metal_received']:,} metal and {tr['energy_received']:,} energy" + (f" from minute {tr['first_minute_received']}" if tr.get("first_minute_received") is not None else "")
                        + (f", {tr['units_sent']} units given away, {tr['units_received']} received" if tr["units_sent"] or tr["units_received"] else ""))
            if "metal_sent_by_hand" in tr:
                seat.append(f"of the metal sent {tr['metal_sent_by_hand']:,} went with the store below nine tenths and {tr['metal_sent_overflow']:,} as a full store's overflow, of the energy {tr['energy_sent_by_hand']:,} and {tr['energy_sent_overflow']:,}")
        if t.get("gifts"):
            c = Counter(g["to"] for g in t["gifts"])
            seat.append(f"gave by hand {sum(g['amount'] for g in t['gifts'] if g['resource'] == 'metal'):,} metal and {sum(g['amount'] for g in t['gifts'] if g['resource'] == 'energy'):,} energy in {len(t['gifts'])} gifts from {t['gifts'][0]['clock']}, to " + ", ".join(f"{w} ({n})" for w, n in c.most_common(4)))
        if t.get("took_over"):
            seat.append("took over " + ", ".join(f"{x['from']}'s seat at {x['clock']} ({x['units']} units, {x['metal']:,} metal, {x['energy']:,} energy)" for x in t["took_over"]))
        if t.get("ui_lines"):
            seat.append("asked the side: " + ", ".join(f"{u['kind']} at {u['clock']}" for u in t["ui_lines"][:6]))
        if t.get("units_given"):
            c = Counter((g["unit"], g["from_team"]) for g in t["units_given"])
            seat.append("received " + ", ".join(f"{n} {u} from team {f}" for (u, f), n in c.most_common(5)))
        if t.get("units_taken"):
            c = Counter((g["unit"], g["to_team"]) for g in t["units_taken"])
            seat.append("gave " + ", ".join(f"{n} {u} to team {f}" for (u, f), n in c.most_common(5)))
        if seat:
            out.append("Seat: " + "; ".join(seat) + ".")
        if t.get("built"):
            out.append("Built over the game: " + ", ".join(f"{n} {k}" for k, n in list(t["built"].items())[:14]) + ".")
        out.append("Build order to the cut-off: " + "; ".join(f"{o['clock']} {o['unit']}" + (f" spot_{o['spot']} {o['grid']}" if "spot" in o else "") + (f" ({o['by']})" if o["by"] not in ("?", "commander") else "") for o in t["build_order"]))
        out.append("Factories: " + ", ".join(f"{f['unit']} {f['started']}-{f['finished'] or '?'} at {f['at']}" for f in t["factories"]) + (f". First tier 2: {t['first_tier2']['unit']} at {t['first_tier2']['clock']}" + (f", first tier-2 unit {t['first_tier2_unit']['unit']} at {t['first_tier2_unit']['clock']}" if t.get("first_tier2_unit") else "") + "." if t["first_tier2"] else ". No tier 2."))
        out.append("By minute (extractors / constructors / metal income / army metal): " + " ".join(f"{c['minute']}:{c['extractors']}/{c['constructors']}/{c['metal_income']:.0f}/{c['army_metal']}" for c in t["curves"] if c["minute"] % 2 == 0))
        out.append("Army every two minutes: " + "; ".join(f"{c['minute']}: " + ", ".join(f"{n} {k}" for k, n in c["army"].items()) for c in t["army_by_type"] if c["army"]))
        pr = t.get("pressure") or {}
        if pr.get("first_soldier_in_his_half"):
            a, b, c, d = pr["first_soldier_in_his_half"], pr.get("first_soldier_near_his_start"), pr.get("first_his_building_killed"), pr.get("first_his_extractor_killed")
            out.append(f"Pressure: first soldier in his half {a['clock']} ({a['unit']} at {a['grid']})" + (f", within 1,500 of his start {b['clock']}" if b else ", never within 1,500 of his start")
                       + (f"; first building of his killed {c['clock']} ({c['unit']} at {c['grid']})" if c else "; no building of his killed")
                       + (f"; first extractor of his killed {d['clock']} (spot_{d['spot']} {d['grid']})" if d and d.get("spot") is not None else "")
                       + ". Army metal in his half by minute: " + " ".join(f"{c2['minute']}:{c2.get('army_metal_in_his_half', 0)}" for c2 in t["curves"] if c2["minute"] % 2 == 0) + ".")
        if t["first_extractor_lost"]:
            out.append(f"First extractor lost {t['first_extractor_lost']['clock']} at spot_{t['first_extractor_lost']['spot']} ({t['first_extractor_lost'].get('grid', '?')}) to a {t['first_extractor_lost']['by']}.")
        out.append("Spots taken: " + " ".join(f"{s['clock']} spot_{s['spot']}({s['grid']})" for s in t["spots_taken"]))
        out.append("Lost: " + ", ".join(f"{n} {k}" for k, n in list(t["losses"].items())[:8]) + ". Killed: " + ", ".join(f"{n} {k}" for k, n in list(t["kills"].items())[:8]) + "." + (f" In the last 15 s (the end, or a resign): {sum(t['end_kills'].values())} of theirs died." if t.get("end_kills") else ""))
        out.append("Fighting (minute, cell, ours lost / theirs lost): " + "; ".join(f"{f['minute']} {f['grid']} {f['ours_lost']}/{f['theirs_lost']}" for f in t["fights"]))
        if t.get("chat"):
            out.append("Chat: " + " | ".join(f"{c['clock']} {c['text']}" for c in t["chat"][:40]) + (f" | ... {len(t['chat']) - 40} more" if len(t["chat"]) > 40 else ""))
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
    lanes(teams)
    by_team = {t["team"]: t for t in teams}
    # The seats: the start script first (every team, people and AIs, with the ally teams and handicaps as the engine
    # had them), the manifest's rows for what only the API knows (start positions, the winner of an old replay).
    players = []
    script_path = os.path.join(match, "_script.txt")
    if os.path.exists(script_path):
        _, game = startscript.load_script(script_path)
        players = startscript.players_of(startscript.seats(game))
        api_rows = {p["name"]: p for p in (entry or {}).get("players", [])}
        for p in players:
            p["start"] = (api_rows.get(p["name"]) or {}).get("start")
    elif entry:
        players = [dict(p) for p in entry["players"]]
    # The winner: the engine's own list in the result row (the dump widget from 2026-10-22), else the API's.
    winners = next((r.result.get("winners") for r in recs if r.result and r.result.get("winners")), None)
    if winners is None and entry and entry.get("winner_ally") is not None:
        names = set(entry.get("winners") or [])
        # The API's winner is the side's in-game number; its player rows carry database ids as their ally (the
        # picker's name list was empty for every 8v8 row until 2026-10-22), so the number itself is the fallback.
        winners = sorted({p["ally"] for p in players if p["name"] in names and p.get("ally") is not None}) or [entry["winner_ally"]]
    for p in players:
        p["won"] = (p.get("ally") in winners) if winners else None
        t = by_team.get(p.get("team"))
        if t is None:
            continue
        t["player"] = f"{p['name']} (OS {p['os']})" if not p.get("ai") else f"{p['name']} (AI {p['ai']}" + (f", +{p['handicap']}%" if p.get("handicap") else "") + ")"
        t["ai"] = p.get("ai")
        t["handicap"] = p.get("handicap") or 0
        t["os"] = p.get("os")
        if p["won"] is not None:
            t["won"] = p["won"]
        p["lane"] = t.get("lane")
    sides = []
    for ally in sorted({t.get("ally_team") for t in teams if t.get("ally_team") is not None}):
        seats = [p for p in players if p.get("ally") == ally]
        ais = Counter(p["ai"] for p in seats if p.get("ai"))
        people = [p for p in seats if not p.get("ai")]
        oss = sorted(p["os"] for p in people if p.get("os") is not None)
        parts = []
        if people:
            parts.append(f"{len(people)} {'person' if len(people) == 1 else 'people'}" + (f" (OS {oss[0]:.0f} to {oss[-1]:.0f})" if len(oss) > 1 else f" (OS {oss[0]:.0f})" if oss else ""))
        parts += [f"{n} {k}" for k, n in ais.most_common()]
        hc = Counter(p.get("handicap") or 0 for p in seats).most_common(1)[0][0] if seats else 0
        sides.append({"ally": ally, "seats": len(seats), "people": len(people), "ais": dict(ais), "handicap": hc, "description": ", ".join(parts) or "nobody", "won": (ally in winners) if winners else None})
    h = recs[0].header
    last = max((r.samples[-1]["f"] for r in recs if r.samples), default=0)
    if winners:
        result = "; ".join(f"side {s['ally']} ({s['description']}) {'won' if s['won'] else 'lost'}" for s in sides)
    elif players:
        result = "unknown winner"
    else:
        result = str(recs[0].result)
    # Small games are titled by their names, large ones by their sides.
    if sides and len(players) <= 4:
        title = " vs ".join(", ".join(p["name"] for p in players if p.get("ally") == s["ally"]) for s in sides)
    elif sides:
        title = " vs ".join(s["description"] for s in sides)
    else:
        title = os.path.basename(match)
    card = {
        "match": os.path.relpath(match, REPO), "id": entry["id"] if entry else None, "title": title,
        "map": {"name": h["map"]["name"], "width": h["map"]["width"], "height": h["map"]["height"], "spots": len(h["metal_spots"]), "grid": f"{(h.get('grid') or {}).get('columns', 8)}x{(h.get('grid') or {}).get('rows', 8)}"},
        "game_version": entry["game_version"] if entry else (game.get("gametype") if os.path.exists(script_path) else None), "duration": clock(last), "result": result, "winners": winners,
        "sides": sides, "players": players, "teams": teams,
    }
    with open(os.path.join(match, "card.json"), "w") as f:
        json.dump(card, f, indent=1)
    md = markdown(card)
    with open(os.path.join(match, "card.md"), "w") as f:
        f.write(md)
    print(f"wrote {os.path.relpath(match, REPO)}/card.json and card.md ({len(md)} chars)")


if __name__ == "__main__":
    main()
