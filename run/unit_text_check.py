#!/usr/bin/env python3
"""After the unit tables are regenerated from a new game version: which hand-written texts still state a number
the game has changed.

    run/unit_text_check.py                         # the tables in the working tree against HEAD's
    run/unit_text_check.py --since <git rev>       # against another commit's
    run/unit_text_check.py --docs docs/briefs      # also the lines of these documents that name a changed unit

The generators keep the hand-written fields of the glossary (`gloss`, `prose` in crates/bot/data/units.json) as they
are, so a description goes on saying "1600 metal" after the table beside it says 1700. For every unit whose entry
changed in the glossary or in the combat table (crates/combatsim/data/units.json), this takes each old value (cost,
health, speed, reach, sight, and a weapon's damage, reload, blast and range) and looks for it

  - in the unit's own gloss and prose,
  - in every other unit's text within 140 characters after a mention of the unit (its internal name or its
    display name, unless several units share that name),
  - with `--docs`, in the lines of the documents under the given directories that name the unit (generated table
    rows, which begin `| <internal name> |`, are skipped).

It prints candidates, not verdicts: a number is matched as written, so the Helios's old 160 damage also finds its
160 metal. Read each line against the unit's entry (`run/unit_text_check.py --show <unit> ...` prints a unit's text
beside its numbers and weapons now). A comparison whose numbers are not written out ("at Cortex's price", "half
Armada's") is not found by this; read the texts of units that name a changed one.

The procedure it belongs to is in docs/harness/engine.md, "Game data".
"""
import argparse
import json
import os
import re
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GLOSSARY = "crates/bot/data/units.json"
COMBAT = "crates/combatsim/data/units.json"
# Fields nobody writes into a description, and build time, whose values collide with energy prices.
SKIP = ("acceleration", "start_velocity", "velocity", "edge", "mid", "top", "radius", "lead_limit", "accuracy", "spray", "predict_boost", "build_time")
WINDOW = 140


def units(text):
    table = json.loads(text)["units"]
    return {entry["name"]: entry for entry in table} if isinstance(table, list) else table


def at(rev, path):
    return subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"], check=True, capture_output=True, text=True).stdout


def flat(value, prefix=""):
    out = {}
    if isinstance(value, dict):
        for key, inner in value.items():
            out.update(flat(inner, f"{prefix}{key}."))
    elif isinstance(value, list):
        for index, inner in enumerate(value):
            out.update(flat(inner, f"{prefix}{index}."))
    else:
        out[prefix[:-1]] = value
    return out


def written(value):
    """The ways a number is written in the texts: 1220, 1,220, 105.6 as 106; one-digit numbers match everything."""
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        return set()
    forms = {f"{float(value):g}", f"{round(value):d}", f"{round(value):,d}"}
    return {form for form in forms if len(form.replace(",", "")) >= 2}


def changed_values(name, old_glossary, new_glossary, old_combat, new_combat):
    values = {}
    for label, old, new in (("", old_glossary, new_glossary), ("combat.", old_combat, new_combat)):
        if name not in old or old[name] == new.get(name):
            continue
        before = flat({k: v for k, v in old[name].items() if k not in ("gloss", "prose", "air")})
        after = flat({k: v for k, v in new.get(name, {}).items() if k not in ("gloss", "prose", "air")})
        for key, value in before.items():
            if value != after.get(key) and not key.endswith(SKIP):
                values.setdefault(label + key, (value, after.get(key)))
    return values


def finds(text, values):
    for key, (old, new) in values.items():
        for form in written(old):
            for match in re.finditer(r"(?<![\d.,])" + re.escape(form) + r"(?![\d])", text):
                yield key, old, new, text[max(0, match.start() - 60): match.end() + 40].replace("\n", " ")


def show(names, glossary, combat):
    for name in names:
        entry = glossary[name]
        numbers = {k: v for k, v in entry.items() if k not in ("gloss", "prose", "made_by", "flags", "class", "name", "faction")}
        print(f"=== {name} | {entry.get('name')} | {numbers}")
        for weapon in combat.get(name, {}).get("weapons", []):
            print("   weapon:", {k: weapon[k] for k in ("kind", "range", "reload", "damage", "aoe", "burst", "projectiles", "energy_per_shot", "only_targets") if k in weapon})
        print("   gloss:", entry.get("gloss"))
        print("   prose:", entry.get("prose"))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--since", default="HEAD", help="the commit whose tables are the old ones (default HEAD)")
    ap.add_argument("--docs", nargs="*", default=[], help="directories of documents to search too")
    ap.add_argument("--show", nargs="+", metavar="UNIT", help="print these units' texts beside their numbers now, and stop")
    args = ap.parse_args()
    os.chdir(REPO)
    new_glossary, new_combat = units(open(GLOSSARY, encoding="utf-8").read()), units(open(COMBAT, encoding="utf-8").read())
    if args.show:
        show(args.show, new_glossary, new_combat)
        return
    old_glossary, old_combat = units(at(args.since, GLOSSARY)), units(at(args.since, COMBAT))
    shared = {entry.get("name") for entry in new_glossary.values()}
    shared = {name for name in shared if sum(1 for entry in new_glossary.values() if entry.get("name") == name) > 1}
    units_changed = lines_found = 0
    for name in sorted(set(old_glossary) | set(old_combat)):
        values = changed_values(name, old_glossary, new_glossary, old_combat, new_combat)
        if not values:
            continue
        units_changed += 1
        display = new_glossary.get(name, {}).get("name", name)
        lines = []
        own = " ".join(str(new_glossary.get(name, {}).get(field, "")) for field in ("gloss", "prose"))
        for key, old, new, context in finds(own, values):
            lines.append(f"   own text, {key} {old} -> {new}: ...{context}...")
        names = [re.escape(name)] + ([re.escape(display)] if len(display) > 3 and display not in shared else [])
        mention = re.compile(r"\b(" + "|".join(names) + r")\b")
        for other, entry in new_glossary.items():
            if other == name:
                continue
            text = " ".join(str(entry.get(field, "")) for field in ("gloss", "prose"))
            for match in mention.finditer(text):
                for key, old, new, context in finds(text[max(0, match.start() - 40): match.end() + WINDOW], values):
                    lines.append(f"   in {other}'s text, {key} {old} -> {new}: ...{context}...")
        for directory in args.docs:
            for root, _, files in os.walk(directory):
                for file in sorted(files):
                    if not file.endswith(".md"):
                        continue
                    path = os.path.join(root, file)
                    for number, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
                        if line.startswith(f"| {name} |") or not mention.search(line):
                            continue
                        for key, old, new, context in finds(line, values):
                            lines.append(f"   {path}:{number}, {key} {old} -> {new}: ...{context}...")
        lines = list(dict.fromkeys(lines))
        if lines:
            lines_found += len(lines)
            print(f"{name} ({display})")
            print("\n".join(lines))
    print(f"{units_changed} units changed since {args.since}; {lines_found} candidate lines", file=sys.stderr)


if __name__ == "__main__":
    main()
