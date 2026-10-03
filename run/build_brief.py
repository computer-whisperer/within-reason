#!/usr/bin/env python3
"""Assemble the experience brief (docs/briefs/player-experience.md) from its parts: the hand-written ones under
docs/briefs/experience/, the generated ones under docs/briefs/reference/ (run/reference_set.py, run/timelines.py,
run/opponent_set.py), and the map sheets of every map with a recorded game (target/release/mapsheet).
docs/design/2026-10-03-brief-rewrite-and-commander-pack.md.

    run/build_brief.py [--regenerate]     (--regenerate runs the generators again before assembling)
"""
import glob, json, os, re, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(REPO)
REF, EXP = "docs/briefs/reference", "docs/briefs/experience"
# The maps the sheets are made for, each from starts people use there (the pool's commonest pair where there is a
# pool; our games' starts otherwise), with the map's own metal scale (`maxMetal` of its mapinfo.lua).
COMET = "Comet Catcher Remake 1.8"


def latest_record(map_name):
    best = None
    for p in sorted(glob.glob("run/matches/*/00/record-0.jsonl")):
        d = os.path.dirname(p)
        if not os.path.exists(d + "/terrain-0.bin"):
            continue
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
    for p in glob.glob("run/matches/*/00/record-0.jsonl"):
        if os.path.exists(os.path.dirname(p) + "/terrain-0.bin"):
            try:
                maps[json.loads(open(p).readline())["map"]["name"]] = 1
            except (ValueError, KeyError):
                pass
    out = ["## The map sheets\n", "Every map we have a recorded game on, in one table each. The first start is ours in the game the sheet was made from. "
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
    return "\n".join(out)


def main():
    if "--regenerate" in sys.argv:
        src = latest_record(COMET)
        for faction in ("arm", "cor"):
            open(f"{REF}/comet-catcher-remake-1.8-{faction}.md", "w").write(subprocess.run(["python3", "run/reference_set.py", "--map", COMET, "--faction", faction, "--sheet-from", src, "--json-out", f"{REF}/comet-catcher-remake-1.8-{faction}.json"], capture_output=True, text=True, check=True).stdout)
        open(f"{REF}/comet-catcher-remake-1.8-arm-games.md", "w").write(subprocess.run(["python3", "run/timelines.py", "--map", COMET, "--faction", "arm"], capture_output=True, text=True, check=True).stdout)
        open(f"{REF}/comet-catcher-remake-1.8-barb-hard_aggressive.md", "w").write(subprocess.run(["python3", "run/opponent_set.py", "--map", COMET, "--profile", "hard_aggressive"], capture_output=True, text=True, check=True).stdout)
    read = lambda p: open(p).read().strip() + "\n"
    parts = [read(f"{EXP}/intro.md"), read(f"{EXP}/winning.md"), sheets(), read(f"{REF}/comet-catcher-remake-1.8-arm.md"), read(f"{REF}/comet-catcher-remake-1.8-cor.md"), read(f"{REF}/comet-catcher-remake-1.8-arm-games.md"),
             read(f"{EXP}/opponent-barb.md"), read(f"{REF}/comet-catcher-remake-1.8-barb-hard_aggressive.md"), read(f"{EXP}/units.md"), read(f"{EXP}/hands.md"), read(f"{EXP}/cases.md"), read(f"{EXP}/players-words.md")]
    text = "\n\n" + "\n\n".join(parts)
    open("docs/briefs/player-experience.md", "w").write(text)
    print(f"docs/briefs/player-experience.md: {len(text.split())} words, {len(text)} characters")


if __name__ == "__main__":
    main()
