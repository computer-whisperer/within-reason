#!/usr/bin/env python3
"""Export the unit glossary as one sheet for experienced players to annotate.

One row per reachable unit of the three factions, from crates/bot/data/units.json (the
numbers run/unit_stats.py generates from the game source, the hand-written gloss and prose
the player reads), joined with the other per-unit facts the repo holds:

  - the death blast and the self-destruct blast, as the engine reports them in a match
    record's header (`unit_defs`; the newest records under run/matches are read until
    every unit has a definition);
  - whether a builder's usual menu offers it to Jev (crates/bot/src/brain/roster.rs);
  - the combat simulator's equal-metal duel margins (crates/bot/data/matchups.csv),
    as the units it clearly beats and clearly loses to;
  - the experienced players' body sizes from the replay pool
    (docs/studies/2026-09-28-body-sizes.md).

The last columns are empty for the players: a verdict on our text, what the unit is for
and when to build it, what it beats and loses to, tips and mistakes, and who wrote the
row. The first column, the game's internal name, is the key the import matches on and
must not be edited in the sheet.

    run/glossary_sheet.py                       # writes docs/glossary/units.csv
    run/glossary_sheet.py --out some/where.csv
    run/glossary_sheet.py --no-records          # skip the blast columns

Import the sheet back into Google Sheets with File > Import > Upload, "Replace spreadsheet",
separator comma; then freeze the first row and set wrap on the prose columns.
"""

import argparse
import csv
import glob
import json
import os
import re
import sys
from collections import OrderedDict

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GLOSSARY = os.path.join(REPO, "crates", "bot", "data", "units.json")
MATCHUPS = os.path.join(REPO, "crates", "bot", "data", "matchups.csv")
ROSTER_RS = os.path.join(REPO, "crates", "bot", "src", "brain", "roster.rs")
BODY_SIZES = os.path.join(REPO, "docs", "studies", "2026-09-28-body-sizes.md")
MATCHES = os.path.join(REPO, "run", "matches")
DEFAULT_OUT = os.path.join(REPO, "docs", "glossary", "units.csv")

# A duel margin this clear counts as a win or a loss (matchups.csv: +1 wiped them untouched, -1 the reverse).
DUEL_MARGIN = 0.3

FACTIONS = ("armada", "cortex", "legion")

OUR_COLUMNS = [
    "unit", "name", "faction", "tier", "class", "made by",
    "metal", "energy", "build time", "health", "speed", "range", "dps", "dps vs air", "against aircraft",
    "sight", "build power", "energy make", "energy upkeep", "energy storage", "metal storage", "flags",
    "death blast (radius / damage)", "self-destruct blast (radius / damage)",
    "on the builders' usual menu", "sim duels: clearly beats", "sim duels: clearly loses to",
    "pros' body (typical largest / largest seen)",
    "our gloss (one line)", "our prose (what the player reads)",
]
PLAYER_COLUMNS = [
    "verdict on our text (ok / wrong / missing)",
    "what it is for, when to build it, when not",
    "beats / loses to",
    "tips, micro, mistakes",
    "who",
]


def load_glossary():
    with open(GLOSSARY, encoding="utf-8") as f:
        doc = json.load(f)
    return doc.get("source", ""), doc["units"]


def usual_menus():
    """The unit names on the three usual builder menus, read from the Rust constants."""
    with open(ROSTER_RS, encoding="utf-8") as f:
        text = f.read()
    names = set()
    for m in re.finditer(r"const USUAL_[A-Z]+: &\[&str\] = &\[(.*?)\];", text, re.S):
        names |= set(re.findall(r'"([a-z0-9_]+)"', m.group(1)))
    return names


def duels(english):
    """Per unit: the units it clearly beats and clearly loses to at equal metal in the simulator."""
    beats, loses = {}, {}
    if not os.path.exists(MATCHUPS):
        return beats, loses
    with open(MATCHUPS, encoding="utf-8") as f:
        for row in csv.reader(f):
            if not row or row[0].startswith("#") or len(row) < 3:
                continue
            unit, against, margin = row[0], row[1], float(row[2])
            if unit == against:
                continue
            if margin >= DUEL_MARGIN:
                beats.setdefault(unit, []).append((margin, against))
            elif margin <= -DUEL_MARGIN:
                loses.setdefault(unit, []).append((margin, against))

    def words(entries, reverse):
        return ", ".join(f"{english.get(n, n)} ({m:+.2f})" for m, n in sorted(entries, reverse=reverse))

    return ({u: words(v, True) for u, v in beats.items()}, {u: words(v, False) for u, v in loses.items()})


def body_sizes():
    """Per unit: 'typical largest / largest seen' from the body-sizes study's first table."""
    out = {}
    if not os.path.exists(BODY_SIZES):
        return out
    with open(BODY_SIZES, encoding="utf-8") as f:
        for line in f:
            cells = [c.strip().strip("*") for c in line.strip().strip("|").split("|")]
            if len(cells) < 4:
                continue
            m = re.match(r"^([A-Z][A-Za-z]+) ([a-z0-9_]+)$", cells[0])
            if not m or not re.match(r"^\d+$", cells[1]) or not re.match(r"^\d+$", cells[2]):
                continue
            largest, typical = cells[1], cells[2]
            out[m.group(2)] = f"{typical} / {largest}"
    return out


def blasts(wanted):
    """Per unit: (death blast, self-destruct blast) from the newest match records' headers."""
    out = {}
    records = sorted(glob.glob(os.path.join(MATCHES, "*", "*", "record-*.jsonl")), key=os.path.getmtime, reverse=True)
    for path in records:
        try:
            with open(path, encoding="utf-8") as f:
                header = json.loads(f.readline())
        except (OSError, ValueError):
            continue
        for d in header.get("unit_defs") or []:
            name = d.get("name")
            if name in wanted and name not in out:
                out[name] = (d.get("death_blast"), d.get("selfd_blast"))
        if len(out) >= len(wanted):
            break
    return out


def blast_words(b):
    if not b:
        return ""
    radius, damage = b
    if not radius or not damage:
        return ""
    return f"{radius:.0f} / {damage:.0f}"


def num(v):
    if v is None:
        return ""
    if isinstance(v, float) and v == int(v):
        return str(int(v))
    return str(v)


def home(entry, units):
    """The factory that makes it, else '' for a building a constructor builds; for sorting."""
    for b in entry.get("made_by") or []:
        cls = (units.get(b) or {}).get("class", "")
        if any(w in cls for w in ("factory", "gantry", "shipyard", "platform", "plant", "lab", "hive")):
            return units[b]["name"]
    return ""


def rows(source, units, menus, beats, loses, bodies, blast_by_unit):
    english = {n: e["name"] for n, e in units.items()}
    order = sorted(
        units.items(),
        key=lambda kv: (
            FACTIONS.index(kv[1]["faction"]) if kv[1]["faction"] in FACTIONS else 9,
            kv[1]["tier"] or 0,
            home(kv[1], units),
            kv[1].get("speed") is not None,
            kv[1]["class"],
            kv[1].get("metal") or 0,
            kv[0],
        ),
    )
    for name, e in order:
        death, selfd = blast_by_unit.get(name, (None, None))
        yield OrderedDict([
            ("unit", name),
            ("name", e["name"]),
            ("faction", e["faction"]),
            ("tier", num(e["tier"])),
            ("class", e["class"]),
            ("made by", ", ".join(f"{english.get(b, b)} ({b})" for b in e.get("made_by") or [])),
            ("metal", num(e.get("metal"))),
            ("energy", num(e.get("energy"))),
            ("build time", num(e.get("build_time"))),
            ("health", num(e.get("health"))),
            ("speed", num(e.get("speed"))),
            ("range", num(e.get("range"))),
            ("dps", num(e.get("dps"))),
            ("dps vs air", num(e.get("dps_air"))),
            ("against aircraft", e.get("air", "")),
            ("sight", num(e.get("sight"))),
            ("build power", num(e.get("build_power"))),
            ("energy make", num(e.get("energy_make"))),
            ("energy upkeep", num(e.get("energy_upkeep"))),
            ("energy storage", num(e.get("energy_storage"))),
            ("metal storage", num(e.get("metal_storage"))),
            ("flags", ", ".join(e.get("flags") or [])),
            ("death blast (radius / damage)", blast_words(death)),
            ("self-destruct blast (radius / damage)", blast_words(selfd)),
            ("on the builders' usual menu", "yes" if name in menus else ""),
            ("sim duels: clearly beats", beats.get(name, "")),
            ("sim duels: clearly loses to", loses.get(name, "")),
            ("pros' body (typical largest / largest seen)", bodies.get(name, "")),
            ("our gloss (one line)", e.get("gloss", "")),
            ("our prose (what the player reads)", e.get("prose", "")),
        ] + [(c, "") for c in PLAYER_COLUMNS])


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", default=DEFAULT_OUT, help="the CSV to write (default: docs/glossary/units.csv)")
    ap.add_argument("--no-records", action="store_true", help="do not read match records for the blast columns")
    args = ap.parse_args()

    source, units = load_glossary()
    english = {n: e["name"] for n, e in units.items()}
    menus = usual_menus()
    beats, loses = duels(english)
    bodies = body_sizes()
    blast_by_unit = {} if args.no_records else blasts(set(units))

    os.makedirs(os.path.dirname(args.out), exist_ok=True)
    count = 0
    with open(args.out, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=OUR_COLUMNS + PLAYER_COLUMNS, lineterminator="\n")
        w.writeheader()
        for row in rows(source, units, menus, beats, loses, bodies, blast_by_unit):
            w.writerow(row)
            count += 1
    missing_blasts = sorted(n for n in units if n not in blast_by_unit) if not args.no_records else []
    print(f"wrote {args.out}: {count} units, game source {source[:12]}; usual menu {len(menus & set(units))}, duels {len(beats | loses)}, bodies {len(bodies)}, blasts {len(blast_by_unit)}")
    if missing_blasts:
        print(f"no record carried {len(missing_blasts)} units: {' '.join(missing_blasts[:12])}{' ...' if len(missing_blasts) > 12 else ''}", file=sys.stderr)


if __name__ == "__main__":
    main()
