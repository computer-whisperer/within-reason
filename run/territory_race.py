#!/usr/bin/env python3
"""Defence against an opponent that attacks: losses, his army, and the race (T-RACE;
docs/design/2026-10-25-territory-defence.md).

    run/territory_race.py TAG [--start START]

For the newest batch of each start labelled terrN-TAG-START, game by game and pooled:
- the outcome and the minute it came;
- our metal lost to his weapons by minute window and by kind (from the `died:` lines with death=Weapon);
- his armed metal on the map at each mark (his commander apart), and the raids that killed: at each of our losses, his armed metal within
  NEAR of it then (the truth files, every 2 s), and its fastest unit;
- with the bot's `race:` lines (T-RACE), the warning per region by minute and the kill test: every region and
  30-second window with his armed metal near the region's spots, split by whether our armed metal in reach 15 s after
  his arrival (turrets counted at their metal) met his metal there, and by home's region or another; how often a
  building was lost in the window, and the metal lost as a share of the metal standing (a region with more standing
  loses a building more often whatever holds it: terr214-219, where held meant home).
His armed units: a weapon, a reach and a speed, and no build menu, or his commander.
"""
import bisect, glob, json, os, re, statistics, sys
from collections import defaultdict

STARTS = ["comet-his-start", "comet-boxes", "great-divide", "glitters", "mithril", "quicksilver"]
WINDOWS = ((0, 5), (5, 10), (10, 15), (15, 20))
MARKS = (5, 10, 15, 20)
NEAR = 800.0
RACE_WINDOW = 30.0
OURS = re.compile(r"\[territory ai=\d+\] f=(\d+) (.*)")
DIED = re.compile(r"died: (\w+) unit \d+ at \((-?\d+),(-?\d+)\) death=Weapon")
CELL = re.compile(r"r(\d+) (\d+) warn (\S+) ours (\d+)/(\d+)/(\d+)/(\d+) turrets (\d+)")


def kind(name, d):
    if d.get("extracts_metal", 0) > 0:
        return "extractor"
    if name.endswith("com"):
        return "commander"
    if d.get("speed", 0) > 0 and d.get("build_options"):
        return "constructor"
    if d.get("speed", 0) > 0:
        return "soldier"
    if d.get("weapon_count", 0) > 0:
        return "turret"
    return "building"


def armed(name, d):
    return d.get("weapon_count", 0) > 0 and d.get("reach", 0) > 0 and d.get("speed", 0) > 0 and (not d.get("build_options") or name.endswith("com"))


def read(path):
    dump = glob.glob(f"{path}/territory-*.json")
    if not dump:
        return None
    dump = json.load(open(dump[0]))
    defs = {u["name"]: u for u in dump["hello"]["unit_defs"]}
    spots = [(s["x"], s["z"]) for s in dump["hello"]["metal_spots"]]
    home = tuple(dump["home"])
    g = {"defs": defs, "spots": spots, "home": home, "lost": [], "race": [], "of_spot": None}
    for line in open(f"{path}/engine.log", errors="replace"):
        m = OURS.search(line)
        if not m:
            continue
        t, text = int(m.group(1)) / 30.0, m.group(2)
        if text.startswith("died: "):
            d = DIED.match(text)
            if d:
                name = d.group(1)
                g["lost"].append((t, name, float(d.group(2)), float(d.group(3)), defs.get(name, {}).get("metal_cost", 0)))
        elif text.startswith("regions of spots: "):
            g["of_spot"] = [int(r) for r in text.split(": ", 1)[1].split()]
        elif text.startswith("race: "):
            cells = {}
            for c in CELL.finditer(text):
                warn = None if c.group(3) == "-" else float(c.group(3))
                cells[int(c.group(1))] = (float(c.group(2)), warn, [float(c.group(i)) for i in range(4, 8)], float(c.group(8)))
            g["race"].append((t, cells))
    truth = glob.glob(f"{path}/truth-*.jsonl")
    g["truth"] = []
    if truth:
        for line in open(truth[0]):
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            units = [(n, x, z) for _, n, x, z, _, built in row["enemy"] if not built and armed(n, defs.get(n, {}))]
            g["truth"].append((row["f"] / 30.0, units))
    g["truth_times"] = [t for t, _ in g["truth"]]
    return g


def his_near(g, t, x, z, radius=NEAR):
    """His armed units within radius of (x, z) at the truth sample nearest t: (metal, fastest speed)."""
    times = g["truth_times"]
    if not times:
        return 0.0, 0.0
    i = min(bisect.bisect_left(times, t), len(times) - 1)
    metal, fastest = 0.0, 0.0
    for n, ux, uz in g["truth"][i][1]:
        if (ux - x) ** 2 + (uz - z) ** 2 <= radius ** 2:
            d = g["defs"].get(n, {})
            metal += d.get("metal_cost", 0)
            fastest = max(fastest, d.get("speed", 0))
    return metal, fastest


def his_army(g, t):
    times = g["truth_times"]
    if not times or t > times[-1] + 2:
        return None
    i = min(bisect.bisect_left(times, t), len(times) - 1)
    return sum(g["defs"].get(n, {}).get("metal_cost", 0) for n, _, _ in g["truth"][i][1] if not n.endswith("com"))


def main():
    args = sys.argv[1:]
    if not args or args[0].startswith("-"):
        print(__doc__.split("\n\n")[1].strip())
        sys.exit(2)
    tag = args[0]
    only = args[args.index("--start") + 1] if "--start" in args else None
    for start in STARTS:
        if only and start != only:
            continue
        found = sorted(glob.glob(f"run/matches/*-terr*-{tag}-{start}"), key=lambda d: os.path.basename(d).split("-")[0])
        if not found:
            continue
        batch = found[-1]
        results = {}
        if os.path.exists(f"{batch}/results.jsonl"):
            for line in open(f"{batch}/results.jsonl"):
                r = json.loads(line)
                results[r["index"]] = r
        print(f"== {start} ({os.path.basename(batch)})")
        lost_by = defaultdict(float)  # (window, kind) -> metal
        raids = []  # (t, his metal near, fastest)
        army = defaultdict(list)
        warn_by = defaultdict(list)  # minute window -> warns
        test = defaultdict(lambda: [0, 0, 0.0, 0.0, []])  # (held/open, home/away) -> [windows, with a loss, lost, standing, his metal]
        games = sorted(glob.glob(f"{batch}/[0-9][0-9]"))
        for path in games:
            g = read(path)
            if g is None:
                continue
            idx = int(os.path.basename(path))
            r = results.get(idx, {})
            end = r.get("game_minutes", 0)
            lost = sum(m for *_, m in g["lost"])
            print(f"   game {idx:02}: {r.get('outcome', '?'):8} at {end:5.1f} min, {r.get('our_side', '?')}; lost to his weapons {len(g['lost'])} units, {lost:.0f} metal")
            for t, name, x, z, metal in g["lost"]:
                w = next((w for w in WINDOWS if w[0] <= t / 60 < w[1]), None)
                if w:
                    lost_by[(w, kind(name, g["defs"].get(name, {})))] += metal
                raids.append((t,) + his_near(g, t, x, z))
            for mark in MARKS:
                a = his_army(g, mark * 60)
                if a is not None:
                    army[mark].append(a)
            # The race and the kill test.
            if g["race"] and g["of_spot"]:
                members = defaultdict(list)
                for s, rgn in enumerate(g["of_spot"]):
                    members[rgn].append(g["spots"][s])
                home = g["of_spot"][min(range(len(g["spots"])), key=lambda s: (g["spots"][s][0] - g["home"][0]) ** 2 + (g["spots"][s][1] - g["home"][1]) ** 2)]
                for i, (t, cells) in enumerate(g["race"]):
                    for rgn, (metal, warn, ours, turrets) in cells.items():
                        if warn is not None:
                            warn_by[int(t // 300)].append(warn)
                        # His armed metal near the region's spots over the window: the most at any truth sample.
                        his = 0.0
                        for dt in range(0, int(RACE_WINDOW) + 1, 2):
                            his = max(his, max((his_near(g, t + dt, x, z)[0] for x, z in members[rgn]), default=0.0))
                        if his <= 0:
                            continue
                        state = "held" if ours[1] + turrets >= his else "open"
                        loss = 0.0
                        for lt, name, x, z, m in g["lost"]:
                            if t < lt <= t + RACE_WINDOW and g["defs"].get(name, {}).get("speed", 0) == 0:
                                near = min(g["spots"], key=lambda s: (s[0] - x) ** 2 + (s[1] - z) ** 2)
                                if g["of_spot"][g["spots"].index(near)] == rgn:
                                    loss += m
                        cell = test[(state, "home" if rgn == home else "away")]
                        cell[0] += 1
                        cell[1] += loss > 0
                        cell[2] += loss
                        cell[3] += metal
                        cell[4].append(his)
        print("   lost to his weapons, metal summed over the games, by window and kind:")
        kinds = sorted({k for _, k in lost_by})
        for w in WINDOWS:
            text = ", ".join(f"{k} {lost_by[(w, k)]:.0f}" for k in kinds if lost_by[(w, k)] > 0)
            print(f"     {w[0]:2}-{w[1]:2} min: {text or '-'}")
        text = " | ".join(f"{m}:00 {statistics.median(army[m]):.0f} ({len(army[m])} games)" for m in MARKS if army[m])
        print(f"   his armed metal on the map, his commander apart (median): {text or '-'}")
        if raids:
            for w in WINDOWS:
                here = [r for r in raids if w[0] <= r[0] / 60 < w[1]]
                if here:
                    print(f"     raids that killed, {w[0]:2}-{w[1]:2} min: {len(here)} losses, his metal within {NEAR:.0f} median {statistics.median(r[1] for r in here):.0f} (max {max(r[1] for r in here):.0f}), fastest median {statistics.median(r[2] for r in here):.0f}")
        if warn_by:
            text = " | ".join(f"{5 * k}-{5 * k + 5} min {statistics.median(v):.0f} s ({len(v)})" for k, v in sorted(warn_by.items()))
            print(f"   warning per region-sample (median): {text}")
        if test:
            print("   kill test, region-windows with his metal near; held where ours 15 s after his arrival met it:")
            for state in ("held", "open"):
                for where in ("home", "away"):
                    n, hit, metal, standing, his = test[(state, where)]
                    if n:
                        print(f"     {state} {where}: {n:4} windows, his metal median {statistics.median(his):5.0f}, a building lost in {hit / n:4.0%}, {metal / max(standing, 1):5.1%} of the metal standing lost a window")
        print()


main()
