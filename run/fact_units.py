#!/usr/bin/env python3
"""Write the fact layer's unit tables, docs/briefs/facts/units-<domain>.md, from crates/bot/data/units.json: the
glossary's numbers, read from the game's unit definitions by run/unit_stats.py at the game commit the file names. One
document a domain (economy, land, structures, air, sea), one table a faction, one row a unit, every row a citable fact
`F-unit-<internal name>` (the table's `<!--meta -->` block lists them under `ids:`). Generated documents of the brief
under construction (docs/briefs/rework.manifest); rewritten by this tool alone, never edited by hand
(docs/design/2026-10-21-brief-rework.md §3: the fact layer is generated where it can be).

    run/fact_units.py            writes docs/briefs/facts/units-*.md
    run/fact_units.py --check    says whether the documents on disk are what it would write
"""
import json
import os
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(REPO)
SOURCE = "crates/bot/data/units.json"
OUT = "docs/briefs/facts"
FACTIONS = [("armada", "Armada"), ("cortex", "Cortex"), ("legion", "Legion")]
# A unit's domain, by words of its class, first match wins.
DOMAINS = [
    ("economy", ["constructor", "factory", "shipyard", "gantry", "extractor", "solar", "wind", "tidal", "geothermal", "fusion", "converter", "storage",
                 "construction turret", "commander", "resurrection", "assist bot"]),
    ("air", ["aircraft", "bomber", "gunship", "plane", "drone"]),
    ("sea", ["ship", "submarine", "cruiser", "destroyer", "frigate", "boat", "battleship", "flagship"]),
    ("structures", ["turret", "tower", "launcher", "silo", "wall", "shield", "anti-nuke", "facility", "station", "platform", "decoy building", "hive", "mine"]),
    ("land", []),
]
INTRO = {
    "economy": "What produces, stores and builds: extractors, energy makers, converters, storages, construction turrets, factories and constructors of every faction, with the commanders.",
    "land": "The land fighters: bots, tanks, mechs, hovercraft, and the mobile support beside them (radar, jammers, minelayers, mines and bombs are under structures).",
    "structures": "What stands still and fights, sees or hides: defence and anti-air turrets, artillery and long-range cannon, missile launchers and silos, anti-nuke, shields, walls, radar, jammers, cameras, sonar, mines.",
    "air": "What flies: fighters, bombers, gunships, radar and transport planes, drones.",
    "sea": "What floats or dives: warships of every class and submarines.",
}
UNITS = "Metal and energy are the costs. Build time is in build units: a builder with build power P finishes the unit in build time / P seconds, and draws the costs over those seconds. Speed is elmos a second; range and sight are elmos; dps is damage a second against ground targets. The ratios are per metal of cost."


def load():
    data = json.load(open(SOURCE))
    units = data.get("units", {k: v for k, v in data.items() if k != "source"})
    return data.get("source", "?"), units


def domain_of(cls):
    low = cls.lower()
    # Mobile anti-nuke and mobile support fight on land; the mine family stands still.
    if low.startswith("mobile "):
        return "land"
    for name, words in DOMAINS:
        if any(w in low for w in words):
            return name
    return "land"


def fmt(v):
    if v is None or v == "":
        return ""
    if isinstance(v, float):
        return f"{v:.1f}".rstrip("0").rstrip(".") if v != int(v) else str(int(v))
    return str(v)


def ratio(a, b):
    return f"{a / b:.2f}" if a and b else ""


def table(domain, units):
    if domain == "economy":
        head = ["unit", "name", "tier", "class", "from", "metal", "energy", "build time", "health", "speed", "build power", "energy make", "energy upkeep", "e storage", "m storage", "flags"]
        row = lambda k, u: [k, u["name"], u["tier"], u["class"], (u.get("made_by") or [""])[0], u["metal"], u["energy"], u["build_time"], u["health"], u["speed"] or "",
                            u.get("build_power", ""), u.get("energy_make", ""), u.get("energy_upkeep", ""), u.get("energy_storage", ""), u.get("metal_storage", ""), ", ".join(u.get("flags") or [])]
    elif domain == "structures":
        head = ["unit", "name", "tier", "class", "metal", "energy", "build time", "health", "range", "dps", "dps vs air", "sight", "dps per metal", "flags"]
        row = lambda k, u: [k, u["name"], u["tier"], u["class"], u["metal"], u["energy"], u["build_time"], u["health"], u["range"] or "", u["dps"] or "", u["dps_air"] or "", u["sight"], ratio(u["dps"], u["metal"]), ", ".join(u.get("flags") or [])]
    else:
        head = ["unit", "name", "tier", "class", "from", "metal", "energy", "build time", "health", "speed", "range", "dps", "dps vs air", "sight", "health per metal", "dps per metal", "flags"]
        row = lambda k, u: [k, u["name"], u["tier"], u["class"], (u.get("made_by") or [""])[0], u["metal"], u["energy"], u["build_time"], u["health"], u["speed"], u["range"] or "", u["dps"] or "", u["dps_air"] or "", u["sight"],
                            ratio(u["health"], u["metal"]), ratio(u["dps"], u["metal"]), ", ".join(u.get("flags") or [])]
    lines = ["| " + " | ".join(head) + " |", "|" + "---|" * len(head)]
    for k, u in sorted(units.items(), key=lambda kv: (kv[1]["tier"], kv[1]["class"], kv[1]["metal"], kv[0])):
        lines.append("| " + " | ".join(fmt(c).replace("|", "/") for c in row(k, u)) + " |")
    return "\n".join(lines)


def documents():
    source, units = load()
    out = {}
    for domain, _ in DOMAINS:
        chosen = {k: u for k, u in units.items() if domain_of(u["class"]) == domain}
        parts = [f"## Unit facts: {domain}\n", f"{INTRO[domain]} From the game's unit definitions at commit {source[:9]} (`run/unit_stats.py` into `crates/bot/data/units.json`; this document by `run/fact_units.py`). {UNITS}\n"]
        for key, title in FACTIONS:
            mine = {k: u for k, u in chosen.items() if u["faction"] == key}
            if not mine:
                continue
            ids = ", ".join(f"F-unit-{k}" for k in sorted(mine))
            parts.append(f"### {title}: {domain} ({len(mine)} units)\n")
            parts.append(f"<!--meta\nid: F-units-{domain}-{key}\nlayer: fact\nids: [{ids}]\ngrounding: {SOURCE} at game commit {source} (run/unit_stats.py)\ngenerated: run/fact_units.py\nstatus: live\n-->")
            parts.append(table(domain, mine) + "\n")
        out[f"{OUT}/units-{domain}.md"] = "\n".join(parts)
    return out


def main():
    docs = documents()
    if "--check" in sys.argv:
        stale = [p for p, t in docs.items() if not os.path.exists(p) or open(p).read() != t]
        print("stale: " + ", ".join(stale) if stale else "every unit-fact document is current")
        sys.exit(1 if stale else 0)
    os.makedirs(OUT, exist_ok=True)
    for path, text in docs.items():
        open(path, "w").write(text)
        print(f"{path}: {text.count(chr(10) + '| ') - 2 * text.count(chr(10) + '### ')} rows, {len(text.split())} words")


if __name__ == "__main__":
    main()
