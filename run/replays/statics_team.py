#!/usr/bin/env python3
"""Shield generators and the other standing units in the public team games we replayed (the 15- and 16-seat replay
directories under run/matches and the archive, with their run/data/replays/manifest.jsonl entries): who builds a
shield and when, what stands under it, how far forward, how it dies, what kills the towers under one against those
outside, and the census of every non-economy structure by the seat's own faction against the hands' usual menu.
Written for docs/studies/2026-11-08-shields-and-standing-units.md.

    run/replays/statics_team.py            the team pool (builds run/data/statics-team.jsonl on the first run, ~6 min)
    run/replays/statics_team.py --ours     what killed our own seats' structures in our team games, by the killer's guns
"""
import collections, glob, json, math, multiprocessing as mp, os, re, statistics as st, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ARCHIVE = os.environ.get("WR_ARCHIVE", "/ceph/christian/within-reason/run/matches-archive")
CACHE = os.path.join(REPO, "run", "data", "statics-team.jsonl")
U = json.load(open(os.path.join(REPO, "crates/bot/data/units.json")))["units"]
SHIELDS = {"armgate", "corgate", "legdeflector", "leggatet3", "armfgate", "corfgate"}
RADIUS = 550
ECONOMY = ("metal extractor", "solar collector", "wind generator", "tidal generator", "energy storage", "metal storage",
           "energy converter", "fusion reactor", "construction turret", "geothermal plant", "shipyard", "experimental gantry")


def weapon_types():
    """Every unit's weapon types from the game's unit files: a shield of intercept type 1 stops `Cannon` alone."""
    out = {}
    for f in glob.glob(os.path.join(REPO, "upstream/Beyond-All-Reason/units/**/*.lua"), recursive=True):
        txt = open(f, errors="replace").read()
        m = re.match(r"\s*(?:return\s*)?\{\s*(\w+)\s*=\s*\{", txt)
        if m:
            out[m.group(1)] = set(re.findall(r'weapontype\s*=\s*"(\w+)"', txt))
    return out


def guns(name, types):
    if name is None:
        return "no killer named"
    ts = types.get(name) or set()
    if not ts:
        return "a blast or unarmed"
    if U.get(name, {}).get("class") == "long-range cannon":
        return "long-range cannon"
    if ts <= {"Cannon"}:
        return "cannon only"
    return "cannon and other" if "Cannon" in ts else "no cannon"


def structures_of(d):
    """One replay directory: per seat every finished structure as [name, frame, x, z, frame it died, its killer]."""
    game = {"dir": os.path.basename(d), "teams": {}}
    for rec in sorted(glob.glob(d + "/record-*.jsonl")):
        f = open(rec, errors="replace")
        try:
            hdr = json.loads(f.readline())
        except ValueError:
            continue
        defs = hdr.get("unit_defs") or []
        name = [x.get("name") for x in defs]
        static = [not x.get("speed") for x in defs]
        built = {}
        for line in f:
            head = line[:160]   # the event rows are short and their keys sorted; a state row is never parsed
            if '"k":"finished"' in head:
                r = json.loads(line)
                if r.get("d") is not None and r["d"] < len(name) and static[r["d"]]:
                    built[r["u"]] = [name[r["d"]], r["f"], r.get("x"), r.get("z"), None, None]
            elif '"k":"destroyed"' in head:
                r = json.loads(line)
                b = built.get(r.get("u"))
                if b is not None:
                    by = r.get("by_d")
                    b[4], b[5] = r["f"], name[by] if isinstance(by, int) and 0 <= by < len(name) else None
        game["teams"][hdr.get("team")] = list(built.values())
    return game


def build_cache():
    dirs = sorted(set(os.path.dirname(p) for p in glob.glob(ARCHIVE + "/*-replay-*/record-*.jsonl")
                      + glob.glob(os.path.join(REPO, "run/matches/*-replay-*/record-*.jsonl"))))
    dirs = [d for d in dirs if len(glob.glob(d + "/record-*.jsonl")) >= 15]
    with mp.Pool(10) as pool, open(CACHE, "w") as out:
        for game in pool.imap_unordered(structures_of, dirs):
            out.write(json.dumps(game) + "\n")


def kind(n):
    c = U.get(n, {}).get("class", "?")
    if c in ("fusion reactor", "geothermal plant", "solar collector", "wind generator", "tidal generator", "energy storage"):
        return "energy"
    if c == "energy converter":
        return "converters"
    if c in ("metal extractor", "metal storage", "armed metal extractor"):
        return "metal"
    if "factory" in c or c in ("shipyard", "experimental gantry"):
        return "factories"
    if c == "construction turret":
        return "construction turrets"
    if c in ("defence turret", "artillery turret", "armed geothermal plant", "drone hive", "torpedo launcher"):
        return f"towers tier {U[n].get('tier')}"
    if c == "shield generator":
        return "shields"
    return c if c in ("long-range cannon", "nuclear silo", "anti-nuke") else ("anti-air" if c == "anti-air turret" else "other")


def clock(minutes):
    return f"{int(minutes)}:{int(minutes % 1 * 60):02d}"


def quartiles(v, fmt=lambda x: round(x, 1)):
    return [fmt(x) for x in st.quantiles(v, n=4)] if len(v) >= 4 else [fmt(x) for x in v]


def team_pool():
    if not os.path.exists(CACHE):
        build_cache()
    types = weapon_types()
    manifest = {}
    for line in open(os.path.join(REPO, "run/data/replays/manifest.jsonl")):
        r = json.loads(line)
        manifest[r["id"]] = r
    games = []
    for line in open(CACHE):
        g = json.loads(line)
        m = re.search(r"replay-([0-9a-f]{32})$", g["dir"])
        if m and m.group(1) in manifest:
            games.append((g, manifest[m.group(1)]))
    minutes = lambda mf: mf["duration_ms"] / 60000
    metal = lambda n: U.get(n, {}).get("metal", 0) or 0
    shields = lambda built: [b for b in built if b[0] in SHIELDS]
    # A person's seat: (game, manifest entry, the player, its structures).
    seats = [(g, mf, p, g["teams"][str(p["team"])]) for g, mf in games for p in mf["players"]
             if p.get("team") is not None and str(p["team"]) in g["teams"]]
    ai = [b for g, mf in games for t, b in g["teams"].items() if int(t) not in {p.get("team") for p in mf["players"]}]
    print(f"public games of 15 or 16 seats: {len(games)} ({sum(1 for g, mf in games if mf.get('humans_vs_ais'))} people against AI); "
          f"length quartiles {quartiles([minutes(mf) for g, mf in games])} min; people's seats {len(seats)}, AI seats {len(ai)}; "
          f"games with a shield {sum(1 for g, mf in games if any(shields(b) for b in g['teams'].values()))}")
    for least in (0, 20, 30, 40):
        k = [s for s in seats if minutes(s[1]) >= least]
        print(f"  people's seats in games of {least}+ min: {len(k)}, with a shield {100 * sum(1 for s in k if shields(s[3])) / len(k):.0f} %")
    for lo, hi in ((0, 10), (10, 20), (20, 30), (30, 100)):
        k = [s for s in seats if lo <= (s[2].get("os") or 0) < hi and minutes(s[1]) >= 30]
        w = [s for s in k if shields(s[3])]
        print(f"  OS {lo}-{hi}, games of 30+ min: {len(k)} seats, {100 * len(w) / max(len(k), 1):.0f} % with a shield, "
              f"first at {clock(st.median([min(b[1] for b in shields(s[3])) / 1800 for s in w]))}")
    w = [s for s in seats if shields(s[3])]
    firsts = [min(b[1] for b in shields(s[3])) / 1800 for s in w]
    print(f"seats with a shield {len(w)}: the first at quartiles {quartiles(firsts, clock)}, earliest {clock(min(firsts))}; "
          f"shields a seat quartiles {quartiles([len(shields(s[3])) for s in w])}, most {max(len(shields(s[3])) for s in w)}")
    before = collections.defaultdict(list)
    for g, mf, p, built in w:
        f0 = min(b[1] for b in shields(built))
        standing = [b[0] for b in built if b[1] < f0 and (b[4] is None or b[4] > f0)]
        before["fusion reactors"].append(sum(1 for n in standing if U.get(n, {}).get("class") == "fusion reactor"))
        before["tier-2 towers"].append(sum(1 for n in standing if kind(n) == "towers tier 2"))
        before["anti-nukes"].append(sum(1 for n in standing if kind(n) == "anti-nuke"))
    print("  the seat's own standing when its first shield finished, medians:", {k: st.median(v) for k, v in before.items()},
          f"; an anti-nuke already in {100 * sum(1 for x in before['anti-nukes'] if x) / len(w):.0f} %")
    for label, against_ai in (("people against people", False), ("people against AI", True)):
        won, lost = [0, 0], [0, 0]
        for g, mf, p, built in seats:
            if bool(mf.get("humans_vs_ais")) != against_ai or minutes(mf) < 30 or mf.get("winner_ally") is None:
                continue
            allies = sorted(set(q["ally"] for q in mf["players"]))
            tally = won if allies.index(p["ally"]) == mf["winner_ally"] else lost
            tally[0] += 1
            tally[1] += bool(shields(built))
        print(f"  {label}, 30+ min: winners {won[0]} seats, {100 * won[1] / max(won[0], 1):.0f} % with a shield; losers {lost[0]}, {100 * lost[1] / max(lost[0], 1):.0f} %")
    print(f"AI seats with a shield: {100 * sum(1 for b in ai if shields(b)) / max(len(ai), 1):.0f} % of {len(ai)}")
    # What stands under each of the people's shields five minutes after it stood, and where.
    first_kind, under, forward, out, stacked, lived, killers, covered = collections.Counter(), [], [], [], [], [], collections.Counter(), collections.Counter()
    for g, mf, p, built in seats:
        ally = {q["team"]: q["ally"] for q in mf["players"] if q.get("team") is not None}
        start = {q["team"]: (q["start"]["x"], q["start"]["z"]) for q in mf["players"] if q.get("team") is not None and q.get("start")}
        side = [b for t, b in g["teams"].items() if ally.get(int(t)) == p["ally"]]
        enemy = []
        for t, b in g["teams"].items():
            if ally.get(int(t)) == p["ally"]:
                continue
            first = [x for x in b if x[2] is not None][:5]   # an AI seat's start: its first structures
            enemy.append(start.get(int(t)) or (first and (sum(x[2] for x in first) / len(first), sum(x[3] for x in first) / len(first))))
        for sh in shields(built):
            if sh[4] is not None:
                lived.append((sh[4] - sh[1]) / 1800)
                killers[sh[5]] += 1
            if sh[2] is None:
                continue
            at, by_kind, others = sh[1] + 9000, collections.Counter(), 0
            for b in (x for bl in side for x in bl):
                if b is sh or b[2] is None or b[1] > at or (b[4] is not None and b[4] < at) or math.hypot(b[2] - sh[2], b[3] - sh[3]) > RADIUS:
                    continue
                by_kind[kind(b[0])] += metal(b[0])
                others += b[0] in SHIELDS
                if kind(b[0]) in ("towers tier 2", "long-range cannon"):
                    covered[b[0]] += 1
            by_kind.pop("shields", None)
            if by_kind:
                first_kind[max(by_kind, key=by_kind.get)] += 1
                under.append(sum(by_kind.values()))
            stacked.append(others)
            mine = start.get(p["team"])
            if mine and all(enemy):
                d0 = math.hypot(sh[2] - mine[0], sh[3] - mine[1])
                out.append(d0)
                forward.append(d0 / (d0 + min(math.hypot(sh[2] - e[0], sh[3] - e[1]) for e in enemy)))
    n = sum(len(shields(s[3])) for s in seats)
    print(f"the people's shields {n}: {100 * len(lived) / n:.0f} % died, those after a median {st.median(lived):.1f} min; killers {killers.most_common(10)}")
    print("  the kind with the most metal under one:", [(k, f"{100 * v / sum(first_kind.values()):.0f} %") for k, v in first_kind.most_common(8)])
    print(f"  structure metal under one, quartiles {quartiles(under, round)}; another shield within {RADIUS} in {100 * sum(1 for x in stacked if x) / len(stacked):.0f} %, quartiles {quartiles(stacked)}")
    print(f"  from the seat's start, quartiles {quartiles(out, round)}; share of the way to the nearest enemy start {quartiles(forward, lambda x: round(x, 2))}")
    print("  tier-2 towers and long-range cannon under them:", covered.most_common(10))
    # What kills the big structures of sides that had shields, under a living shield or not.
    for label, pick in (("tier-2 towers", lambda n: kind(n) == "towers tier 2"), ("fusion reactors", lambda n: U.get(n, {}).get("class") == "fusion reactor")):
        for after_first in (False, True):
            died, stood = {True: collections.Counter(), False: collections.Counter()}, {True: 0, False: 0}
            for g, mf in games:
                ally = {q["team"]: q["ally"] for q in mf["players"] if q.get("team") is not None}
                sides = collections.defaultdict(list)
                for t, b in g["teams"].items():
                    if int(t) in ally:
                        sides[ally[int(t)]] += b
                for built in sides.values():
                    sh = [b for b in built if b[0] in SHIELDS and b[2] is not None]
                    if not sh:
                        continue
                    f0 = min(s[1] for s in sh)
                    for b in built:
                        if not pick(b[0]) or b[2] is None or (after_first and b[1] < f0):
                            continue
                        at = b[4]
                        cov = any((s[4] is None if at is None else s[1] <= at and (s[4] is None or s[4] >= at))
                                  and math.hypot(s[2] - b[2], s[3] - b[3]) <= RADIUS for s in sh)
                        stood[cov] += 1
                        if at is not None:
                            died[cov][guns(b[5], types)] += 1
            for cov in (True, False):
                tot = sum(died[cov].values())
                print(f"{label}{' finished after the side’s first shield' if after_first else ''}, {'under a living shield' if cov else 'not under one'}: "
                      f"{stood[cov]} built, {100 * tot / max(stood[cov], 1):.0f} % died; killers " + ", ".join(f"{k} {100 * v / max(tot, 1):.0f} %" for k, v in died[cov].most_common()))
    # The census by the seat's own faction, games of 30+ min.
    prefix = {"Armada": "arm", "Cortex": "cor", "Legion": "leg"}
    long_games = [s for s in seats if minutes(s[1]) >= 30 and s[2].get("faction") in prefix]
    of_faction = collections.Counter(s[2]["faction"] for s in long_games)
    usual = set(re.findall(r'"([a-z0-9]+)"', open(os.path.join(REPO, "crates/bot/src/brain/roster.rs")).read()))
    count, first = collections.defaultdict(list), collections.defaultdict(list)
    for g, mf, p, built in long_games:
        c = collections.Counter(b[0] for b in built if b[0].startswith(prefix[p["faction"]]))
        for name, n in c.items():
            count[name].append(n)
            first[name].append(min(b[1] for b in built if b[0] == name) / 1800)
    print(f"\ncensus, people's seats in games of 30+ min by their own faction {dict(of_faction)}:")
    print("  unit | class | share of the faction's seats | median count | median first | on the hands' usual menu | metal")
    rows = [(len(v) / of_faction[{v: k for k, v in prefix.items()}[n[:3]]], n) for n, v in count.items()
            if not any(e == U.get(n, {}).get("class") or "factory" in U.get(n, {}).get("class", "factory") for e in ECONOMY)]
    for share, n in sorted(rows, reverse=True)[:60]:
        print(f"  {U[n]['name']:20s} {n:13s} {U[n]['class']:24s} {100 * share:4.0f} %  x{st.median(count[n]):<5} {clock(st.median(first[n])):>6}  {'usual' if n in usual else '-':5s} {U[n]['metal']}")


def our_losses(d, types):
    """Our own seats' records of one game: structure metal lost by the killer's guns and reach."""
    lost, far, seats = collections.Counter(), collections.Counter(), 0
    for rec in sorted(glob.glob(d + "/00/record-*.jsonl")):
        f = open(rec, errors="replace")
        try:
            hdr = json.loads(f.readline())
        except ValueError:
            continue
        seats += 1
        defs = hdr["unit_defs"]
        name = [x["name"] for x in defs]
        for line in f:
            if '"k":"destroyed"' not in line[:200]:
                continue
            r = json.loads(line)
            d_ = defs[r["d"]] if r.get("d") is not None else None
            if not d_ or d_.get("speed") or (d_.get("metal") or 0) < 60 or r.get("c") in ("reclaimed", "decayed", "selfd"):
                continue
            by = r.get("by_d")
            weapon = str(r.get("w") or "")
            named = isinstance(by, int) and 0 <= by < len(name)
            if not named and weapon.split("_")[0] in name:   # an unnamed killer's weapon names its type
                by, named = name.index(weapon.split("_")[0]), True
            if named:
                k = guns(name[by], types)
                if k not in ("long-range cannon", "a blast or unarmed"):
                    k += ", reach over 550" if (defs[by].get("reach") or 0) > RADIUS else ", reach 550 or less"
                if "over 550" in k or k == "long-range cannon":
                    far[f"{name[by]} ({k.split(',')[0]})"] += d_["metal"]
            else:
                k = "a death explosion" if "expl" in weapon or "maker" in weapon or "boom" in weapon else "no killer named"
            lost[k] += d_["metal"]
    return os.path.basename(d), seats, lost, far


def ours():
    types = weapon_types()
    dirs = []
    for pattern in ("*glitters-8v8-*", "*glitters-4v5*", "*human-8v8*", "*defence-*8v8*", "*barb-8v8*", "*-human"):
        dirs += glob.glob(ARCHIVE + "/" + pattern) + glob.glob(os.path.join(REPO, "run/matches", pattern))
    dirs = sorted(set(d for d in dirs if "replay" not in d and len(glob.glob(d + "/00/record-*.jsonl")) >= 2))
    total, far = collections.Counter(), collections.Counter()
    with mp.Pool(10) as pool:
        for name, seats, lost, f in pool.starmap(our_losses, [(d, types) for d in dirs]):
            s = sum(lost.values())
            if s:
                print(f"{name} | {seats} seats | structures lost {s:,.0f}: " + ", ".join(f"{k} {100 * v / s:.0f} %" for k, v in lost.most_common(5)))
                total.update(lost)
                far.update(f)
    s = sum(total.values())
    print(f"all {len(dirs)} games, {s:,.0f} of structures:", ", ".join(f"{k} {100 * v / s:.0f} %" for k, v in total.most_common()))
    print("killers that reach over 550, by our structure metal:", [(k, round(v)) for k, v in far.most_common(12)])


if __name__ == "__main__":
    ours() if "--ours" in sys.argv else team_pool()
