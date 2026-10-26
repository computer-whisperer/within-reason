#!/usr/bin/env python3
"""Defence against an opponent that attacks: losses, his army, and the race (T-RACE;
docs/design/2026-10-25-territory-defence.md).

    run/territory_race.py TAG [--start START]

For the newest batch of each start labelled terrN-TAG-START, game by game and pooled:
- the outcome and the minute it came;
- the win line (the user, 2026-10-25: "double the enemy metal income at some point past the opening", then "Let's
  make the win line start at 15:00"): from the truth file's `income` (both sides' metal income, since 2026-10-25),
  each side's mean over 30 s, our best ratio to his from 15:00 and when, the ratio at 5, 10, 15 and 20 minutes, and
  how many games were twice his at each of those marks (a game over before a mark is not; from 5:00 the bot without
  defence met it in 20 of 24 games, terr226-231);
- the turrets and radars standing at 10 and 15 minutes (the `defence:` lines, T-COVER and T-RADAR);
- the guard at 10 and 15 minutes (the `guard:` lines, T-GUARD): the army he has shown beside his army on the map
  (the truth file), the guard's metal standing, and the soldiers ordered; with stations (T-STATIONS), how many stood
  and the metal they left uncovered, and the soldiers' time by what they did (summed over the `guard:` lines);
- our metal lost to his weapons by minute window and by kind (from the `died:` lines with death=Weapon), and by
  killer: his aircraft, his ground units, or a blast (a weapon no unit owns: a commander's or a fusion's death);
- our commander's death, its minute and its killer;
- with the `air:` lines (T-AIR), at 15, 20 and 25 minutes: the air he has shown beside his air on the map, and our
  anti-air turrets standing and their metal (standing, framed and ordered);
- his armed metal on the map at each mark (his commander apart) and the part of it that flies (T-AIR, the air bench:
  docs/design/2026-10-25-territory-defence.md, step 3d), and the raids that killed: at each of our losses, his armed metal within
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
WINDOWS = ((0, 5), (5, 10), (10, 15), (15, 20), (20, 25))
MARKS = (5, 10, 15, 20)
# His air on the map at these (the air bench runs to 25:00).
AIR_MARKS = (15, 20, 25)
NEAR = 800.0
RACE_WINDOW = 30.0
OURS = re.compile(r"\[territory ai=\d+\] f=(\d+) (.*)")
DIED = re.compile(r"died: (\w+) unit \d+ at \((-?\d+),(-?\d+)\) death=Weapon\(Weapon \{ name: \"(\w+)\"")
CELL = re.compile(r"r(\d+) (\d+) warn (\S+) ours (\d+)/(\d+)/(\d+)/(\d+) turrets (\d+)")
DEFENCE = re.compile(r"defence: raid (\d+) .*?\| turrets (\d+) \((\d+) metal\), radars (\d+)")
GUARD = re.compile(r"guard: shown (\d+) \((\d+) units\), wanted (\d+) \| guard (\d+) standing \((\d+) \S+\), (\d+) making, (\d+) ordered \| (.*)")
STATIONS = re.compile(r"stations: (\d+) of at most (\d+) .*\| uncovered: (.*)")
AIR = re.compile(r"air: shown (\d+) \((\d+) units.*?answer (\d+) \| anti-air (\d+) standing of (\d+) \((\d+) metal\)")
DOING = re.compile(r"(\d+) fighting \d+ raids, (\d+) holding, (\d+) retreating, (\d+) called home, (\d+) at stations")
WIN_FROM = 900.0
INCOME_MEAN = 30.0
WIN = 2.0


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


def flies(d):
    return d.get("speed", 0) > 0 and d.get("move_class") is None


def killer(weapon, defs):
    """What fired a weapon: its unit is the name before the first underscore (`corcrwh_krowlaserh`); a weapon of no
    unit is a blast (`commanderexplosion`, `customfusionexplo`)."""
    d = defs.get(weapon.split("_")[0])
    if d is None:
        return "blast"
    return "aircraft" if flies(d) else "ground"


def read(path):
    dump = glob.glob(f"{path}/territory-*.json")
    if not dump:
        return None
    dump = json.load(open(dump[0]))
    defs = {u["name"]: u for u in dump["hello"]["unit_defs"]}
    spots = [(s["x"], s["z"]) for s in dump["hello"]["metal_spots"]]
    home = tuple(dump["home"])
    g = {"defs": defs, "spots": spots, "home": home, "lost": [], "race": [], "of_spot": None, "defence": [], "guard": [], "stations": [], "doing": [0] * 5, "income": [], "commander": None, "air": []}
    for line in open(f"{path}/engine.log", errors="replace"):
        m = OURS.search(line)
        if not m:
            continue
        t, text = int(m.group(1)) / 30.0, m.group(2)
        if text.startswith("died: "):
            d = DIED.match(text)
            if d:
                name, by = d.group(1), killer(d.group(4), defs)
                g["lost"].append((t, name, float(d.group(2)), float(d.group(3)), defs.get(name, {}).get("metal_cost", 0), by))
                if kind(name, defs.get(name, {})) == "commander" and g["commander"] is None:
                    g["commander"] = (t, d.group(4).split("_")[0], by)
        elif text.startswith("defence: "):
            d = DEFENCE.match(text)
            if d:
                g["defence"].append((t, int(d.group(1)), int(d.group(2)), int(d.group(3)), int(d.group(4))))
        elif text.startswith("guard: "):
            d = GUARD.match(text)
            if d:
                g["guard"].append((t, int(d.group(1)), int(d.group(4)), int(d.group(5)), int(d.group(7)), d.group(8)))
                n = DOING.search(text)
                if n:
                    g["doing"] = [a + int(b) for a, b in zip(g["doing"], n.groups())]
        elif text.startswith("air: shown"):
            d = AIR.match(text)
            if d:
                g["air"].append((t, int(d.group(1)), int(d.group(4)), int(d.group(6))))
        elif text.startswith("stations: "):
            d = STATIONS.match(text)
            if d:
                late = sum(int(c.split()[1]) for c in d.group(3).split(", ") if c != "-")
                g["stations"].append((t, int(d.group(1)), late))
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
            if "income" in row:
                g["income"].append((row["f"] / 30.0, row["income"][0], row["income"][1]))
    g["truth_times"] = [t for t, _ in g["truth"]]
    return g


def incomes(g):
    """Each side's metal income, the mean over the INCOME_MEAN seconds to each truth sample: [(t, ours, his)]."""
    rows, out = g["income"], []
    for i, (t, _, _) in enumerate(rows):
        window = [r for r in rows[max(0, i - int(INCOME_MEAN / 2)):i + 1] if r[0] > t - INCOME_MEAN]
        out.append((t, statistics.mean(r[1] for r in window), statistics.mean(r[2] for r in window)))
    return out


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


def his_army(g, t, air=False):
    """His armed metal on the map at the truth sample nearest t, his commander apart; with `air`, only what flies."""
    times = g["truth_times"]
    if not times or t > times[-1] + 2:
        return None
    i = min(bisect.bisect_left(times, t), len(times) - 1)
    return sum(g["defs"].get(n, {}).get("metal_cost", 0) for n, _, _ in g["truth"][i][1] if not n.endswith("com") and (not air or flies(g["defs"].get(n, {}))))


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
        lost_to = defaultdict(float)  # (window, killer) -> metal
        air = defaultdict(list)
        anti_air = defaultdict(list)  # mark -> [(shown, his air on the map, turrets standing, metal)]
        raids = []  # (t, his metal near, fastest)
        army = defaultdict(list)
        warn_by = defaultdict(list)  # minute window -> warns
        wins = []
        held_at = defaultdict(list)  # mark -> [ratio >= WIN there]
        defence = defaultdict(list)  # mark -> [(turrets, metal, radars)]
        guard = defaultdict(list)  # mark -> [(shown, his army, guard standing, soldiers, ordered)]
        stations = defaultdict(list)  # mark -> [(stations, metal uncovered)]
        doing = [0] * 5
        test = defaultdict(lambda: [0, 0, 0.0, 0.0, []])  # (held/open, home/away) -> [windows, with a loss, lost, standing, his metal]
        games = sorted(glob.glob(f"{batch}/[0-9][0-9]"))
        for path in games:
            g = read(path)
            if g is None:
                continue
            idx = int(os.path.basename(path))
            r = results.get(idx, {})
            end = r.get("game_minutes", 0)
            lost = sum(row[4] for row in g["lost"])
            by = {k: sum(row[4] for row in g["lost"] if row[5] == k) for k in ("aircraft", "ground", "blast")}
            com = g["commander"]
            com_text = f"; commander killed at {int(com[0] // 60)}:{int(com[0] % 60):02} by {com[1]} ({com[2]})" if com else ""
            print(f"   game {idx:02}: {r.get('outcome', '?'):8} at {end:5.1f} min, {r.get('our_side', '?')}; lost to his weapons {len(g['lost'])} units, {lost:.0f} metal (aircraft {by['aircraft']:.0f}, ground {by['ground']:.0f}, blasts {by['blast']:.0f}){com_text}")
            means = incomes(g)
            if means:
                after = [(t, o / h) for t, o, h in means if t >= WIN_FROM and h > 0]
                best = max(after, key=lambda x: x[1], default=None)
                wins.append(best is not None and best[1] >= WIN)
                at = lambda m: next((f"{o:.0f}/{h:.0f}" for t, o, h in means if t >= m * 60), "-")
                for m in MARKS:
                    # A game over before the mark has not kept its ratio there.
                    o_h = next(((o, h) for t, o, h in means if t >= m * 60), None)
                    held_at[m].append(o_h is not None and o_h[1] > 0 and o_h[0] / o_h[1] >= WIN)
                best_text = f"best ratio from 15:00 {best[1]:.2f} at {int(best[0] // 60)}:{int(best[0] % 60):02}{'  WIN' if best[1] >= WIN else ''}" if best else "over before 15:00"
                print(f"     income ours/his: {' '.join(f'{m}:00 {at(m)}' for m in MARKS)}; {best_text}")
            for mark in (10, 15):
                d = [row for row in g["defence"] if row[0] <= mark * 60]
                if d and d[-1][0] > mark * 60 - 60:
                    defence[mark].append((d[-1][2], d[-1][3], d[-1][4]))
                q = [row for row in g["stations"] if row[0] <= mark * 60]
                if q and q[-1][0] > mark * 60 - 60:
                    stations[mark].append((q[-1][1], q[-1][2]))
                q = [row for row in g["guard"] if row[0] <= mark * 60]
                if q and q[-1][0] > mark * 60 - 60:
                    guard[mark].append((q[-1][1], his_army(g, mark * 60) or 0, q[-1][2], q[-1][3], q[-1][4]))
            doing = [a + b for a, b in zip(doing, g["doing"])]
            for t, name, x, z, metal, by in g["lost"]:
                w = next((w for w in WINDOWS if w[0] <= t / 60 < w[1]), None)
                if w:
                    lost_by[(w, kind(name, g["defs"].get(name, {})))] += metal
                    lost_to[(w, by)] += metal
                raids.append((t,) + his_near(g, t, x, z))
            for mark in MARKS:
                a = his_army(g, mark * 60)
                if a is not None:
                    army[mark].append(a)
            for mark in AIR_MARKS:
                a = his_army(g, mark * 60, air=True)
                if a is not None:
                    air[mark].append(a)
                q = [row for row in g["air"] if row[0] <= mark * 60]
                if q and q[-1][0] > mark * 60 - 60:
                    anti_air[mark].append((q[-1][1], a or 0, q[-1][2], q[-1][3]))
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
                        for lt, name, x, z, m, _ in g["lost"]:
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
        if wins:
            print(f"   the win line (our income twice his at some point from 15:00): {sum(wins)} of {len(wins)} games; twice his at " + ", ".join(f"{m}:00 {sum(held_at[m])}" for m in MARKS))
        for mark in (10, 15):
            if defence[mark]:
                print(f"   at {mark}:00 (games alive): turrets {' / '.join(str(t) for t, _, _ in defence[mark])} ({' / '.join(str(m) for _, m, _ in defence[mark])} metal), radars {' / '.join(str(r) for _, _, r in defence[mark])}")
            if stations[mark]:
                print(f"   stations at {mark}:00 (games alive): {' / '.join(str(n) for n, _ in stations[mark])}, metal uncovered {' / '.join(str(m) for _, m in stations[mark])}")
            if guard[mark]:
                print(f"   guard at {mark}:00 (games alive): his army shown / on the map {' '.join(f'{s}/{a:.0f}' for s, a, _, _, _ in guard[mark])}; guard standing {' / '.join(str(m) for _, _, m, _, _ in guard[mark])} ({' / '.join(str(n) for _, _, _, n, _ in guard[mark])} soldiers), ordered {' / '.join(str(o) for *_, o in guard[mark])}")
        if sum(doing):
            print("   the soldiers' time (soldier-lines): " + ", ".join(f"{w} {n / sum(doing):.0%}" for w, n in zip(("fighting", "holding", "retreating", "called home", "at stations"), doing)))
        print("   lost to his weapons, metal summed over the games, by window and kind:")
        kinds = sorted({k for _, k in lost_by})
        for w in WINDOWS:
            text = ", ".join(f"{k} {lost_by[(w, k)]:.0f}" for k in kinds if lost_by[(w, k)] > 0)
            to = ", ".join(f"{k} {lost_to[(w, k)]:.0f}" for k in ("aircraft", "ground", "blast") if lost_to[(w, k)] > 0)
            print(f"     {w[0]:2}-{w[1]:2} min: {text or '-'}{f'; to {to}' if to else ''}")
        text = " | ".join(f"{m}:00 {statistics.median(army[m]):.0f} ({len(army[m])} games)" for m in MARKS if army[m])
        print(f"   his armed metal on the map, his commander apart (median): {text or '-'}")
        text = " | ".join(f"{m}:00 {' / '.join(f'{a:.0f}' for a in air[m])}" for m in AIR_MARKS if air[m] and any(air[m]))
        if text:
            print(f"   of it flying, by game: {text}")
        for mark in AIR_MARKS:
            if anti_air[mark]:
                print(f"   air at {mark}:00 (games alive): his air shown / on the map {' '.join(f'{s}/{a:.0f}' for s, a, _, _ in anti_air[mark])}; anti-air standing {' / '.join(str(n) for _, _, n, _ in anti_air[mark])} ({' / '.join(str(m) for *_, m in anti_air[mark])} metal with frames and orders)")
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
