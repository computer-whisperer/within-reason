#!/usr/bin/env python3
"""The ground a territory batch played on, per game, from the bot's dump (`territory-0.json`) and its first lines in
engine.log: what the bot's fixed settings assume of a map, measured on this one (the cross-map baseline, 2026-10-25).

    run/territory_geometry.py <batch dir or label> [...]

Per game: the map, side and box; the spots on the map, in the bot's scenario (`spots=all` less the dead radius
round his start) and their metal; how many lie within the opening's radius (600), the commander's leash (700),
1,500 and 3,000 of home; the distance from home of the 10th, 20th and 40th nearest; the spacing to each spot's
nearest neighbour among the scenario's (median and 90th percentile, in elmos and in seconds at the basic
constructor's speed, against the program's fixed extractor walk of 15 s); the share of the map under water and the
spots under it; the farm's origin and the building slots of ground in its core. Distances are straight lines.
"""
import glob, json, math, os, re, statistics, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HOME = re.compile(r"home \((-?\d+), (-?\d+)\), enemy start taken as \((-?\d+), (-?\d+)\); (\d+) spots of (\d+) in the scenario")
FARM = re.compile(r"farm: origin \((-?\d+),(-?\d+)\), (\d+) from home at bearing (\d+)/16 \(his start at ([\d.]+)/16\), (\d+) building slots")
DEAD_SPOT = 1000.0


def batch_dirs(arg):
    if os.path.isdir(arg):
        return [arg]
    return sorted(glob.glob(f"{REPO}/run/matches/*-{arg}"))


def quantile(xs, q):
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(q * len(xs)))] if xs else float("nan")


def game_line(game):
    dumps = sorted(glob.glob(f"{game}/territory-*.json"))
    if not dumps:
        return f"{os.path.basename(game)}: no dump"
    dump = json.load(open(dumps[0]))
    hello = dump["hello"]
    home = tuple(dump["home"])
    defs = {d["id"]: d for d in hello["unit_defs"]}
    commander = defs.get(dump["commander"], {})
    side = commander.get("name", "?")[:3]
    extractor = next((d for d in hello["unit_defs"] if d["name"] == f"{side}mex"), None)
    rate = extractor["extracts_metal"] if extractor else 0.001
    constructor = next((d for d in hello["unit_defs"] if d["name"] == f"{side}cv"), None)
    speed = constructor["speed"] if constructor else 54.0
    enemy, farm = None, None
    for line in open(f"{game}/engine.log", errors="replace"):
        if enemy is None and (m := HOME.search(line)):
            enemy = (float(m.group(3)), float(m.group(4)))
        if farm is None and (m := FARM.search(line)):
            farm = m.groups()
        if enemy and farm:
            break
    spots = [(s["x"], s["z"], s["y"] * rate) for s in hello["metal_spots"]]
    dist = lambda a, b: math.hypot(a[0] - b[0], a[1] - b[1])
    scenario = [s for s in spots if enemy is None or dist(s, enemy) > DEAD_SPOT]
    from_home = sorted(dist(s, home) for s in scenario)
    within = lambda r: sum(1 for d in from_home if d < r)
    nth = lambda n: from_home[n - 1] if len(from_home) >= n else float("nan")
    nearest = [min(dist(s, t) for t in scenario if t is not s) for s in scenario] if len(scenario) > 1 else []
    terrain = hello["terrain"]
    heights, cell, width = terrain["heights"], terrain["cell"], terrain["width"]
    wet = sum(1 for h in heights if h < 0) / max(1, len(heights))
    def under(s):
        i, j = int(s[0] // cell), int(s[1] // cell)
        at = j * width + i
        return 0 <= at < len(heights) and heights[at] < 0
    wet_spots = sum(1 for s in scenario if under(s))
    results = os.path.join(os.path.dirname(game), "results.jsonl")
    corner = "?"
    if os.path.exists(results):
        index = int(os.path.basename(game))
        for line in open(results):
            r = json.loads(line)
            if r.get("index") == index:
                corner = r.get("our_corner", "?")
    metal = [s[2] for s in scenario]
    lines = [
        f"{os.path.basename(game)} {hello['map']['name']} {side} {corner}: home ({home[0]:.0f},{home[1]:.0f}), his start ({enemy[0]:.0f},{enemy[1]:.0f}) {dist(home, enemy):.0f} off" if enemy else f"{os.path.basename(game)}: no home line",
        f"   spots {len(spots)} on the map, {len(scenario)} in the scenario, metal {statistics.median(metal):.2f}/s median ({min(metal):.2f}-{max(metal):.2f}), {sum(metal):.0f}/s in all" if metal else "   no spots",
        f"   from home: {within(600)} within 600, {within(700)} within 700, {within(1500)} within 1,500, {within(3000)} within 3,000; the 10th at {nth(10):.0f}, the 20th at {nth(20):.0f}, the 40th at {nth(40):.0f}",
        f"   nearest neighbour {statistics.median(nearest):.0f} median, {quantile(nearest, 0.9):.0f} p90: {statistics.median(nearest) / speed:.1f} s / {quantile(nearest, 0.9) / speed:.1f} s at {speed:.0f} (the program's walk: 15 s)" if nearest else "   no neighbours",
        f"   water {100 * wet:.0f}% of the map, {wet_spots} scenario spots under it; map {hello['map']['width']:.0f} x {hello['map']['height']:.0f}, wind {hello['map']['wind_min']:.0f}-{hello['map']['wind_max']:.0f}, tidal {hello['map']['tidal']:.0f}",
    ]
    if farm:
        lines.append(f"   farm origin ({farm[0]},{farm[1]}) {farm[2]} from home, {farm[5]} building slots of ground in its core")
    return "\n".join(lines)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        print(__doc__)
        sys.exit(1)
    for arg in args:
        for batch in batch_dirs(arg):
            print(f"== {os.path.basename(batch)}")
            for game in sorted(glob.glob(f"{batch}/[0-9][0-9]")):
                print(game_line(game))


if __name__ == "__main__":
    main()
