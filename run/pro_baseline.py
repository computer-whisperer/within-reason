#!/usr/bin/env python3
"""One of our recorded games beside the pros' cards, minute by minute: where our curve sits against the pool's
quartiles on the same map, from the same start (or its mirror), and the opening milestones the cards give
(docs/design/2026-09-23-replay-survey.md; the reviewer's first table, `.claude/skills/bar-review/SKILL.md`).

    run/pro_baseline.py run/matches/<batch>/<NN> [...] [--floor 40] [--start CELL] [--all-starts] [--faction arm|cor|any]
                        [--until 20] [--manifest run/data/replays/manifest.jsonl] [--json]

Our side is read from the match's record (`record-<ai>.jsonl`, docs/harness/record-format.md) the way
`run/replays/card.py` reads a replay's: extractors, constructors and soldiers are counts of `own` at the minute's
sample (frames included, as the cards count them), army metal the soldiers' cost, income and store the sample's.
The pool is every carded side of the manifest on the record's map with every player at `--floor` OS and above
(default 40, the players' advice of 2026-09-24), from our start cell or its half-turn mirror (the map is
point-symmetric, K-open-start-geometry-mirror-pairs; `--all-starts` takes every start, `--start B5` names one), playing
our faction (the record header's `side`; `--faction any` takes both: Cortex opens later, with a turret,
K-map-comet-catcher-remake-1-8-two-openings).
A side counts at a minute only if its game reached it, so the late minutes are the longer games.

Columns per minute: ours, then the pool's p25 / median / p75, then the pool's winners' median; a flag marks ours
outside the pool's middle half (below p25, or above p75 for the metal store, where high means unspent). The
milestones are clocks from the build order: the first factory started and finished, its first unit, the first
soldier, the first constructor, the second factory, the first extractor lost (and to what), and counts by 8:00
(light turrets, solars, constructors made, radars, nano turrets), each against the pool's median and quartiles.
`--json` prints the same numbers as one object per match.
"""
import glob
import json
import os
import statistics
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "replays"))
from card import Record, secs as card_secs  # noqa: E402


def secs(c):
    """A card clock ("m:ss") as seconds; None for a missing one (a factory never finished)."""
    return None if not c else card_secs(c)

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FPS = 30
MINUTES = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16, 18, 20)
CURVE_KEYS = (("extractors", "extractors"), ("constructors", "constructors"), ("metal_income", "income"),
              ("army_metal", "army metal"), ("soldiers", "soldiers"), ("metal_stored", "metal stored"))
LLT = ("armllt", "corllt")
SOLAR = ("armsolar", "corsolar")
RADAR = ("armrad", "corrad")
NANO = ("armnanotc", "cornanotc")
REZ = ("armrectr", "cornecro")


def clock(s):
    return "-" if s is None else f"{int(s) // 60}:{int(s) % 60:02d}"


def quartiles(xs):
    xs = sorted(x for x in xs if x is not None)
    if not xs:
        return None
    q = statistics.quantiles(xs, n=4, method="inclusive") if len(xs) >= 2 else [xs[0], xs[0], xs[0]]
    return {"p25": q[0], "median": statistics.median(xs), "p75": q[2], "n": len(xs)}


def mirror(cell, columns=8, rows=8):
    return f"{chr(65 + columns - 1 - (ord(cell[0]) - 65))}{rows + 1 - int(cell[1:])}"


# ---------------------------------------------------------------- milestones from a build order

def milestones(order, factories, first_lost, classify):
    """Milestone clocks (seconds) from a card-shaped build order: [{clock, unit, by}], factories [{started, finished}]."""
    out = {}
    ff = factories[0] if factories else {}
    out["first_factory_started"] = secs(ff.get("started"))
    out["first_factory_finished"] = secs(ff.get("finished"))
    out["second_factory_started"] = secs(factories[1].get("started")) if len(factories) > 1 else None
    out["first_factory_kind"] = ff.get("unit")
    from_factory = [b for b in order if str(b.get("by", "")).startswith("factory")]
    out["first_factory_unit"] = from_factory[0]["unit"] if from_factory else None
    out["first_factory_unit_at"] = secs(from_factory[0]["clock"]) if from_factory else None
    out["first_soldier"] = next((secs(b["clock"]) for b in from_factory if classify(b["unit"]) == "army"), None)
    out["first_constructor"] = next((secs(b["clock"]) for b in from_factory if classify(b["unit"]) == "builder"), None)
    by_8 = lambda names: sum(1 for b in order if b["unit"] in names and secs(b["clock"]) is not None and secs(b["clock"]) < 480)
    out["turrets_by_8"] = by_8(LLT)
    out["solars_by_8"] = by_8(SOLAR)
    out["radars_by_8"] = by_8(RADAR)
    out["nanos_by_8"] = by_8(NANO)
    out["rez_bots_by_8"] = by_8(REZ)
    out["constructors_by_8"] = sum(1 for b in from_factory if classify(b["unit"]) == "builder" and secs(b["clock"]) is not None and secs(b["clock"]) < 480)
    out["units_by_4"] = sum(1 for b in from_factory if secs(b["clock"]) is not None and secs(b["clock"]) < 240)
    out["solars_before_plant"] = sum(1 for b in order if b["unit"] in SOLAR and (out["first_factory_started"] is None or secs(b["clock"]) < out["first_factory_started"]))
    out["first_extractor_lost"] = secs((first_lost or {}).get("clock"))
    out["first_extractor_lost_to"] = (first_lost or {}).get("by")
    return out


MILESTONE_ROWS = (
    ("first_factory_started", "first factory started", "clock"), ("first_factory_finished", "first factory standing", "clock"),
    ("first_factory_unit_at", "its first unit", "clock"), ("first_soldier", "first soldier", "clock"),
    ("first_constructor", "first constructor", "clock"), ("second_factory_started", "second factory started", "clock"),
    ("first_extractor_lost", "first extractor lost", "clock"), ("solars_before_plant", "solars before the plant", "count"),
    ("units_by_4", "factory units by 4:00", "count"), ("constructors_by_8", "constructors made by 8:00", "count"),
    ("turrets_by_8", "light turrets by 8:00", "count"), ("solars_by_8", "solars by 8:00", "count"),
    ("radars_by_8", "radars by 8:00", "count"), ("nanos_by_8", "nano turrets by 8:00", "count"), ("rez_bots_by_8", "rez bots by 8:00", "count"),
)


# ---------------------------------------------------------------- our side, card-shaped

def our_side(rec):
    """Build order, factories, first extractor lost and curves from our record, in the card's shapes."""
    builders = {}
    for s in rec.samples:
        for u in s.get("own", []):
            builders.setdefault(u[0], u[1])
    order, factories = [], {}
    for e in rec.events:
        if e.get("k") == "created":
            by = e.get("by")
            role = "?"
            if by is not None and by in builders:
                d = rec.d(builders[by])
                role = "commander" if d.get("class") == "commander" else f"factory {d.get('name')}" if rec.is_factory(builders[by]) else "constructor"
            entry = {"clock": f"{e['f'] // FPS // 60}:{e['f'] // FPS % 60:02d}", "unit": rec.name(e.get("d")), "by": role}
            order.append(entry)
            if rec.is_factory(e.get("d")):
                factories[e["u"]] = {"unit": entry["unit"], "started": entry["clock"], "finished": None}
        elif e.get("k") == "finished" and e["u"] in factories:
            factories[e["u"]]["finished"] = f"{e['f'] // FPS // 60}:{e['f'] // FPS % 60:02d}"
    first_lost = None
    for e in rec.events:
        if e.get("k") == "destroyed" and rec.is_extractor(e.get("d")):
            first_lost = {"clock": f"{e['f'] // FPS // 60}:{e['f'] // FPS % 60:02d}", "by": rec.name(e.get("by_d")) if e.get("by_d") is not None else "?"}
            break
    curves = {}
    for s in rec.samples:
        minute = s["f"] // (60 * FPS)
        if minute in curves or s["f"] % (60 * FPS) > 2 * FPS:
            continue
        own = s.get("own", [])
        soldiers = [u for u in own if rec.is_soldier(u[1])]
        curves[minute] = {"extractors": sum(1 for u in own if rec.is_extractor(u[1])), "constructors": sum(1 for u in own if rec.is_constructor(u[1])),
                          "metal_income": round(s["m"][1], 1), "metal_stored": round(s["m"][0]), "army_metal": round(sum(rec.d(u[1]).get("metal") or 0 for u in soldiers)),
                          "soldiers": len(soldiers)}
    start = None
    for u in (rec.samples[0].get("own", []) if rec.samples else []):
        if rec.d(u[1]).get("class") == "commander":
            start = rec.grid(u[2], u[3])
    return {"build_order": order, "factories": sorted(factories.values(), key=lambda f: secs(f["started"])), "first_extractor_lost": first_lost, "curves": curves, "start": start}


# ---------------------------------------------------------------- the pool

def pool_sides(manifest, map_name, floor):
    for line in open(manifest):
        r = json.loads(line)
        if not (r.get("keep") and r.get("card")) or r.get("map_script") != map_name:
            continue
        if any(p.get("os") is None or p["os"] < floor for p in r["players"]):
            continue
        path = os.path.join(REPO, r["match"], "card.json") if not os.path.isabs(r["match"]) else os.path.join(r["match"], "card.json")
        if not os.path.exists(path):
            continue
        card = json.load(open(path))
        for t in card["teams"]:
            yield {"id": r["id"], "won": bool(t.get("won")), "start": (t.get("start") or {}).get("grid"), "side": t.get("side"), "curves": {c["minute"]: c for c in t.get("curves") or []},
                   "build_order": t.get("build_order") or [], "factories": t.get("factories") or [], "first_extractor_lost": t.get("first_extractor_lost"),
                   "duration": card.get("duration")}


def compare(match_dir, manifest, floor, start_cell, all_starts, faction, until):
    record = sorted(glob.glob(os.path.join(match_dir, "record-*.jsonl")))
    if not record:
        sys.exit(f"{match_dir}: no record-*.jsonl")
    rec = Record(record[0])
    classify = lambda name: (rec.defs[next((i for i, d in enumerate(rec.defs) if d["name"] == name), -1)].get("class") if any(d["name"] == name for d in rec.defs) else "?")
    ours = our_side(rec)
    map_name = rec.header["map"]["name"]
    start = start_cell or ours["start"]
    starts = None if all_starts else {start, mirror(start)}
    faction = None if faction == "any" else (faction or rec.header.get("side"))
    sides = [s for s in pool_sides(manifest, map_name, floor) if (starts is None or s["start"] in starts) and (faction is None or s["side"] == faction)]
    if not sides:
        sys.exit(f"{match_dir}: no carded side on {map_name} at OS {floor}+ from {sorted(starts) if starts else 'any start'} as {faction or 'any faction'} (run/replays/card.py; --all-starts, --faction any?)")
    our_m = milestones(ours["build_order"], ours["factories"], ours["first_extractor_lost"], classify)
    pool_m = [milestones(s["build_order"], s["factories"], s["first_extractor_lost"], classify) for s in sides]
    last_minute = max(ours["curves"]) if ours["curves"] else 0
    out = {"match": match_dir, "map": map_name, "floor": floor, "our_start": ours["start"], "pool_starts": sorted(starts) if starts else "all", "faction": faction or "any",
           "pool_sides": len(sides), "pool_games": len({s["id"] for s in sides}), "pool_winners": sum(1 for s in sides if s["won"]),
           "first_factory_kind": {"ours": our_m["first_factory_kind"], "pool": {}}, "milestones": {}, "minutes": []}
    kinds = {}
    for m in pool_m:
        kinds[m["first_factory_kind"]] = kinds.get(m["first_factory_kind"], 0) + 1
    out["first_factory_kind"]["pool"] = kinds
    for key, label, kind in MILESTONE_ROWS:
        q = quartiles([m[key] for m in pool_m])
        qw = quartiles([m[key] for m, s in zip(pool_m, sides) if s["won"]])
        out["milestones"][key] = {"label": label, "kind": kind, "ours": our_m[key], "pool": q, "winners": qw}
    out["first_extractor_lost_to"] = {"ours": our_m["first_extractor_lost_to"]}
    for minute in MINUTES:
        if minute > min(until, last_minute):
            break
        row = {"minute": minute, "ours": ours["curves"].get(minute), "pool": {}, "winners": {}}
        for key, _ in CURVE_KEYS:
            row["pool"][key] = quartiles([s["curves"][minute][key] for s in sides if minute in s["curves"] and key in s["curves"][minute]])
            row["winners"][key] = quartiles([s["curves"][minute][key] for s in sides if s["won"] and minute in s["curves"] and key in s["curves"][minute]])
        out["minutes"].append(row)
    return out


def flag(ours, q, high_is_bad=False):
    if ours is None or not q:
        return ""
    if high_is_bad:
        return ">p75" if ours > q["p75"] else ""
    return "<p25" if ours < q["p25"] else ">p75" if ours > q["p75"] else ""


def fmt(v, kind):
    if v is None:
        return "-"
    return clock(v) if kind == "clock" else f"{v:.0f}" if isinstance(v, float) else str(v)


def print_report(o):
    print(f"== {o['match']}: {o['map']}, our start {o['our_start']}; the pool: {o['pool_sides']} sides of {o['pool_games']} games with every player at OS {o['floor']}+, "
          f"from {o['pool_starts']}, faction {o['faction']} ({o['pool_winners']} winners)")
    print(f"   first factory: ours {o['first_factory_kind']['ours']}; the pool {o['first_factory_kind']['pool']}; our first extractor lost to {o['first_extractor_lost_to']['ours']}")
    print(f"   {'milestone':<28} {'ours':>7}   {'pool p25':>8} {'median':>7} {'p75':>7}   {'winners':>7}  flag")
    for key, m in o["milestones"].items():
        q, w = m["pool"], m["winners"]
        high_bad = key in ("first_factory_started", "first_factory_finished", "first_factory_unit_at", "first_soldier", "first_constructor", "second_factory_started", "solars_before_plant")
        f = ""
        if m["ours"] is not None and q:
            f = (">p75 (late)" if high_bad and m["ours"] > q["p75"] else "<p25 (early)" if high_bad and m["ours"] < q["p25"] else "") if high_bad else flag(m["ours"], q)
        print(f"   {m['label']:<28} {fmt(m['ours'], m['kind']):>7}   {fmt(q and q['p25'], m['kind']):>8} {fmt(q and q['median'], m['kind']):>7} {fmt(q and q['p75'], m['kind']):>7}   {fmt(w and w['median'], m['kind']):>7}  {f}")
    print()
    print("   by minute: ours | pool p25 / median / p75 | winners' median   (flag: ours below the pool's p25; for the store, above p75)")
    head = "   min " + " | ".join(f"{label:^28}" for _, label in CURVE_KEYS)
    print(head)
    for row in o["minutes"]:
        cells = []
        for key, _ in CURVE_KEYS:
            ours = (row["ours"] or {}).get(key)
            q, w = row["pool"].get(key), row["winners"].get(key)
            f = flag(ours, q, high_is_bad=(key == "metal_stored"))
            if q:
                cells.append(f"{fmt(ours, 'count'):>5} |{q['p25']:>5.0f}/{q['median']:>5.0f}/{q['p75']:>5.0f} |{w['median'] if w else 0:>5.0f} {f:<4}")
            else:
                cells.append(f"{fmt(ours, 'count'):>5} | {'-':^17} | {'-':>5}     ")
        n = next((q["n"] for q in row["pool"].values() if q), 0)
        print(f"   {row['minute']:>3} " + " | ".join(cells) + f"  (pool sides {n})")
    print()


def main():
    args = sys.argv[1:]
    if not args or "-h" in args or "--help" in args:
        print(__doc__)
        sys.exit(2)
    manifest = os.path.join(REPO, "run/data/replays/manifest.jsonl")
    floor, start, all_starts, faction, until, as_json = 40, None, False, None, 20, False
    dirs = []
    i = 0
    while i < len(args):
        a = args[i]
        if a == "--floor":
            floor = float(args[i + 1]); i += 1
        elif a == "--start":
            start = args[i + 1]; i += 1
        elif a == "--manifest":
            manifest = args[i + 1]; i += 1
        elif a == "--until":
            until = int(args[i + 1]); i += 1
        elif a == "--all-starts":
            all_starts = True
        elif a == "--faction":
            faction = args[i + 1]; i += 1
        elif a == "--json":
            as_json = True
        else:
            dirs.append(a)
        i += 1
    for d in dirs:
        d = d.rstrip("/")
        if not glob.glob(os.path.join(d, "record-*.jsonl")) and os.path.isdir(os.path.join(d, "00")):
            d = os.path.join(d, "00")
        o = compare(d, manifest, floor, start, all_starts, faction, until)
        if as_json:
            print(json.dumps(o))
        else:
            print_report(o)


if __name__ == "__main__":
    main()
