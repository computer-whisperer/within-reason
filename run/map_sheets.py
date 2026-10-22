#!/usr/bin/env python3
"""Write the brief's map sheets, docs/briefs/reference/map-sheets.md: one sheet per map with a recorded game, from
`target/release/mapsheet` over the latest record on that map (the one with a truth file preferred), with the map's
own metal scale (`maxMetal` of its mapinfo.lua). A generated document of the experience brief
(docs/briefs/experience.manifest); committed like the other reference documents, rewritten by this tool alone.

    run/map_sheets.py            writes docs/briefs/reference/map-sheets.md
    run/map_sheets.py --print    prints it instead

`latest_record(map_name)` is shared with run/frontier_set.py.
"""
import glob
import json
import os
import re
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(REPO)
OUT = "docs/briefs/reference/map-sheets.md"
# Where recorded games live: the working directory and the archive the ledger's head names (batches of 2026-09-19
# through 2026-10-06 were moved there; a map whose only games are archived keeps its sheet).
ROOTS = ["run/matches", "/ceph/christian/within-reason/run/matches-archive"]
# The sheets are computed from starts people use where there is a pool (Comet), our games' starts otherwise.
COMET = "Comet Catcher Remake 1.8"


def records():
    """Every recorded game with a terrain file, oldest first by its directory name (a timestamp), over every root."""
    found = [p for root in ROOTS for p in glob.glob(root + "/*/00/record-0.jsonl")]
    return sorted((p for p in found if os.path.exists(os.path.dirname(p) + "/terrain-0.bin")), key=lambda p: os.path.basename(os.path.dirname(os.path.dirname(p))))


def latest_record(map_name):
    best = None
    for p in records():
        d = os.path.dirname(p)
        try:
            if json.loads(open(p).readline())["map"]["name"] == map_name:
                best = d if best is None or os.path.exists(d + "/truth-0.jsonl") else best
        except (ValueError, KeyError):
            continue
    return best


def max_metal(map_name):
    f = "run/data/maps/" + map_name.lower().replace(" ", "_") + ".sd7"
    if not os.path.exists(f):
        return None
    text = subprocess.run(["bsdtar", "-xOf", f, "mapinfo.lua"], capture_output=True, text=True).stdout
    m = re.search(r"maxmetal\s*=\s*([0-9.]+)", text, re.I)
    return m and m.group(1)


def sheets():
    maps = {}
    for p in records():
        try:
            maps[json.loads(open(p).readline())["map"]["name"]] = 1
        except (ValueError, KeyError):
            pass
    out = ["## The map sheets\n", "Every map we have a recorded game on, in one table each. Each sheet's table is computed from the first start it lists, which was ours in the game the sheet was made from; from the other start, read \"us\" and \"him\" the other way round. "
           "Walks are in elmos at full speed; a unit's seconds are the walk over its speed. \"The ways at their tightest\" is how wide the routes between two starts are "
           "at their narrowest against their median width: a pass is a tightest width a fraction of the median. The game you are in sends its own sheet, from your "
           "start, in the first report.\n"]
    for name in sorted(maps, key=lambda n: (n != COMET, n)):
        d = latest_record(name)
        args = ["target/release/mapsheet", d]
        if name == COMET:
            args += ["--start", "ours (the pool's west start, B5):1420,3550", "--start", "the opponent's (the pool's east start, H5):7290,3600", "--start", "the arena's old corner start of his (G1):6899,681"]
        mm = max_metal(name)
        if mm:
            args += ["--max-metal", mm]
        r = subprocess.run(args, capture_output=True, text=True)
        out.append(r.stdout if r.returncode == 0 else f"### {name}\n\n(no sheet: {r.stderr.strip()})\n")
    return "\n".join(out).strip() + "\n"


def main():
    text = sheets()
    if "--print" in sys.argv:
        sys.stdout.write(text)
        return
    open(OUT, "w").write(text)
    print(f"{OUT}: {len(text.split())} words, {text.count(chr(10) + '### ')} maps")


if __name__ == "__main__":
    main()
