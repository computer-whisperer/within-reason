#!/usr/bin/env python3
"""Rewrite every generated document of the experience brief (docs/briefs/reference/) from the recorded games and the
pool: the reference sets (run/reference_set.py, both factions), the whole games (run/timelines.py), the opponent set
(run/opponent_set.py), the frontier's brief section (run/frontier_set.py --brief; a frontier run itself is hours and is
not started here) and the map sheets (run/map_sheets.py). Each generator writes its own file; the brief is assembled
from them by the bot at session start (docs/briefs/experience.manifest), so there is nothing else to rebuild.

    run/regenerate_reference.py
"""
import os
import subprocess

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(REPO)
REF = "docs/briefs/reference"
COMET = "Comet Catcher Remake 1.8"


def main():
    import sys
    sys.path.insert(0, "run")
    from map_sheets import latest_record  # noqa: E402
    src = latest_record(COMET)
    for faction in ("arm", "cor"):
        out = subprocess.run(["python3", "run/reference_set.py", "--map", COMET, "--faction", faction, "--sheet-from", src, "--json-out", f"{REF}/comet-catcher-remake-1.8-{faction}.json"], capture_output=True, text=True, check=True).stdout
        open(f"{REF}/comet-catcher-remake-1.8-{faction}.md", "w").write(out)
    open(f"{REF}/comet-catcher-remake-1.8-arm-games.md", "w").write(subprocess.run(["python3", "run/timelines.py", "--map", COMET, "--faction", "arm"], capture_output=True, text=True, check=True).stdout)
    open(f"{REF}/comet-catcher-remake-1.8-barb-hard_aggressive.md", "w").write(subprocess.run(["python3", "run/opponent_set.py", "--map", COMET, "--profile", "hard_aggressive", "--archive"], capture_output=True, text=True, check=True).stdout)
    subprocess.run(["python3", "run/frontier_set.py", "--brief"], check=True)
    subprocess.run(["python3", "run/map_sheets.py"], check=True)
    print(subprocess.run(["git", "status", "--short", REF], capture_output=True, text=True).stdout or "nothing changed")


if __name__ == "__main__":
    main()
