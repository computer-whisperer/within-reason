#!/usr/bin/env python3
"""A made-up raid against a cut base: an air drill's cut (`run/air_scene.py`) with his recorded bombers replaced by
groups we choose, flown the way we say. For asking how a defence does against raids the game did not happen to show
(another direction, a dogleg, other bombers, a target away from the wing's place, two groups, a decoy, an escort).

    run/air_raid.py <cut prefix> --target X,Z --his SPEC [--his SPEC ...] [--without anti-air | TYPE,TYPE] --out PREFIX

`<cut prefix>` names an existing `PREFIX.json` and `PREFIX.script.json`: our side and its rows (the arm) are kept as
they are. Each SPEC is one group of his:

    TYPE*N@X,Z[>X,Z ...]>END[+S][~W]

- `TYPE*N@X,Z`  N units of the type, given about that place (ranks of four, 90 apart; `~W`: one line abreast, W apart,
                across the way to its first point)
- `>X,Z`        waypoints, flown in turn
- `>END`        `bomb` (an attack at the ground at `--target`), `fight` (a fight order to `--target`: an escort) or
                `pass` (nothing after the last waypoint: a scout flying through)
- `+S`          the group's first order S seconds after the others'

A bomb order after waypoints goes out when the group is about at its last one (the way at the type's speed, and two
seconds for the take-off): the engine's attack at the ground cannot be queued behind a move by a script row.
`--without` takes units of ours out of the cut: the types named, or `anti-air` for everything the glossary classes
so (what the wing does alone). The fighters the cut added stay.

The rows are side 1's, timed from the cut's lead. The scenario's `centre` becomes the target, so `run/air_drill.py`
measures from it.
"""
import argparse, csv, json, math, os, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def place(text):
    x, z = text.split(",")
    return float(x), float(z)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cut")
    ap.add_argument("--target", type=place, required=True)
    ap.add_argument("--his", action="append", required=True)
    ap.add_argument("--without")
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    scenario = json.load(open(args.cut + ".json"))
    script = json.load(open(args.cut + ".script.json"))
    with open(os.path.join(REPO, "docs/glossary/units.csv")) as f:
        table = list(csv.DictReader(f))
    speed = {r["unit"]: float(r["speed"] or 0) for r in table}
    if args.without:
        gone = {r["unit"] for r in table if "anti-air" in r["class"]} if args.without == "anti-air" else set(args.without.split(","))
        fighters = set(scenario["source"].get("fighters", []))
        ours = scenario["sides"][0]["units"]
        keep = [i for i, u in enumerate(ours) if i in fighters or u["type"] not in gone]
        new = {old: k for k, old in enumerate(keep)}
        scenario["sides"][0]["units"] = [ours[i] for i in keep]
        scenario["source"]["fighters"] = [new[i] for i in scenario["source"].get("fighters", [])]
        scenario["source"]["without"] = f"{args.without}: {len(ours) - len(keep)} units"
        for row in script["rows"]:
            if row.get("side", 0) == 0 and isinstance(row["unit"], list):
                row["unit"] = [new[i] for i in row["unit"] if i in new]
    lead = scenario["source"].get("lead", 12.0)
    tx, tz = args.target
    units, rows = [], []
    for spec in args.his:
        spec, _, width = spec.partition("~")
        spec, _, delay = spec.partition("+")
        steps = spec.split(">")
        force, at = steps[0].split("@")
        kind, count = force.split("*")
        if kind not in speed:
            sys.exit(f"{kind}: no such unit in the glossary")
        end = steps[-1]
        if end not in ("bomb", "fight", "pass"):
            sys.exit(f"{spec}: a group ends in bomb, fight or pass")
        way = [place(p) for p in steps[1:-1]]
        sx, sz = place(at)
        first = way[0] if way else (tx, tz)
        d = math.hypot(first[0] - sx, first[1] - sz) or 1.0
        ux, uz = (first[0] - sx) / d, (first[1] - sz) / d
        members = []
        for k in range(int(count)):
            if width:
                off = (k - (int(count) - 1) / 2) * float(width)
                x, z = sx - uz * off, sz + ux * off
            else:
                across, back = (k % 4 - 1.5) * 90, (k // 4) * 90
                x, z = sx - uz * across - ux * back, sz + ux * across - uz * back
            members.append(len(units))
            units.append({"type": kind, "x": round(x), "z": round(z)})
        t = lead + (float(delay) if delay else 0.0)
        flown, here = 0.0, (sx, sz)
        for k, (x, z) in enumerate(way):
            rows.append({"t": t, "side": 1, "unit": members, "cmd": "move", "x": x, "z": z, "queue": k > 0})
            flown += math.hypot(x - here[0], z - here[1])
            here = (x, z)
        if end != "pass":
            when = t + (flown / speed[kind] + 2.0 if way else 0.0)
            rows.append({"t": round(when, 2), "side": 1, "unit": members, "cmd": "attack_ground" if end == "bomb" else "fight", "x": tx, "z": tz})
    scenario["sides"][1] = {"name": "theirs", "units": units}
    scenario["centre"] = [tx, tz]
    scenario["source"] = dict(scenario["source"], raid=args.his, bombers=len(units), tool="run/air_raid.py over " + scenario["source"].get("tool", "?"))
    script["rows"] = [r for r in script["rows"] if r.get("side", 0) == 0] + rows
    script["note"] = script.get("note", "") + " + air_raid.py"
    with open(args.out + ".json", "w") as f:
        json.dump(scenario, f)
    with open(args.out + ".script.json", "w") as f:
        json.dump(script, f)
    print(f"{args.out}: {len(units)} of his in {len(args.his)} groups, {len(rows)} rows")


if __name__ == "__main__":
    main()
