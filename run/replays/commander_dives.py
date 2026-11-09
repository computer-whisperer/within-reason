#!/usr/bin/env python3
"""How commanders die, and what dies to them, in the public games we replayed (every *-replay-* directory with
records under run/matches and the archive: duels and 16-seat team games of people). For every commander of every
seat: its death (the killer's type and weapon, what of the other side stood within 700 a second before, what of its
own stood there, how long the kill took from its last moment at nine tenths, what it had killed in the 30 s before,
how far from its start), and every dive it lived through (three or more of the other side's units killed by this
commander with no gap over ten seconds, the commander alive 30 s after the last).
Written for docs/briefs/derived/commander-dive.md.

    run/replays/commander_dives.py            builds run/data/commander-dives.jsonl on the first run, then the read
    run/replays/commander_dives.py --rows     every death and dive, one line each
"""
import collections, glob, json, math, multiprocessing as mp, os, re, statistics as st, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ARCHIVE = os.environ.get("WR_ARCHIVE", "/ceph/christian/within-reason/run/matches-archive")
CACHE = os.path.join(REPO, "run", "data", "commander-dives.jsonl")
COMMANDERS = {"armcom", "corcom", "legcom"}
NEAR = 700
FRAME = re.compile(r'"f":(\d+)')


def clock(f):
    return f"{f // 1800}:{f // 30 % 60:02d}"


def read_game(d):
    seats = {}
    for rec in sorted(glob.glob(d + "/record-*.jsonl")):
        f = open(rec, errors="replace")
        try:
            hdr = json.loads(f.readline())
        except ValueError:
            continue
        defs = hdr.get("unit_defs") or []
        seat = {"rec": rec, "team": hdr.get("team"), "ally": hdr.get("ally_team"), "defs": defs, "coms": {}, "dead": [], "last": 0}
        names = [x.get("name") for x in defs]
        for line in f:
            if '"t":"s"' in line[-40:] or '"t":"s"' in line:
                try:
                    row = json.loads(line)
                except ValueError:
                    continue
                if row.get("t") != "s":
                    if row.get("t") == "ev" and row.get("k") == "destroyed":
                        seat["dead"].append(row)
                    continue
                seat["last"] = row["f"]
                for u in row["own"]:
                    if 0 <= u[1] < len(names) and names[u[1]] in COMMANDERS and not u[5] & 1:
                        seat["coms"].setdefault(u[0], {"def": names[u[1]], "trace": []})["trace"].append((row["f"], u[2], u[3], u[4]))
            elif '"k":"destroyed"' in line:
                try:
                    row = json.loads(line)
                except ValueError:
                    continue
                if row.get("t") == "ev" and row.get("k") == "destroyed":
                    seat["dead"].append(row)
        seats[seat["team"]] = seat
    return seats


def units_at(seats, frames):
    """Every seat's finished units at the sample frames asked for: {frame: {team: [(name, class, metal, x, z, health)]}}."""
    out = collections.defaultdict(dict)
    for team, seat in seats.items():
        defs = seat["defs"]
        for line in open(seat["rec"], errors="replace"):
            if '"t":"s"' not in line:
                continue
            m = FRAME.search(line)
            if not m or int(m.group(1)) not in frames:
                continue
            row = json.loads(line)
            out[row["f"]][team] = [(defs[u[1]]["name"], defs[u[1]].get("class"), defs[u[1]].get("metal", 0), u[2], u[3], u[4]) for u in row["own"] if 0 <= u[1] < len(defs) and not u[5] & 1]
    return out


def game(d):
    seats = read_game(d)
    if len(seats) < 2:
        return None
    name = lambda seat, i: seat["defs"][i]["name"] if isinstance(i, int) and 0 <= i < len(seat["defs"]) else None
    metal = lambda seat, i: seat["defs"][i].get("metal", 0) if isinstance(i, int) and 0 <= i < len(seat["defs"]) else 0
    killed_by = collections.defaultdict(list)  # commander id -> the other side's units it killed
    for team, seat in seats.items():
        for e in seat["dead"]:
            if e.get("by") is not None:
                killed_by[e["by"]].append((e["f"], name(seat, e.get("d")), metal(seat, e.get("d")), e.get("w"), team))
    events, frames = [], set()
    for team, seat in seats.items():
        for uid, com in seat["coms"].items():
            trace = com["trace"]
            start = trace[0][1:3]
            death = next((e for e in seat["dead"] if e["u"] == uid), None)
            kills = sorted(k for k in killed_by.get(uid, []) if seats[k[4]]["ally"] != seat["ally"])
            base = {"dir": os.path.basename(d), "seats": len(seats), "team": team, "ally": seat["ally"], "com": com["def"], "start": start}
            if death:
                full = max((t[0] for t in trace if t[3] >= 90), default=trace[0][0])
                at = trace[-1]
                before = [k for k in kills if death["f"] - 900 <= k[0] <= death["f"]]
                ev = dict(base, kind="death", f=death["f"], x=at[1], z=at[2], cause=death.get("c"), by=name(seat, death.get("by_d")), w=death.get("w"), seconds=(death["f"] - full) / 30,
                          from_start=math.hypot(at[1] - start[0], at[2] - start[1]), took=[(k[1], k[2], k[3]) for k in before], sample=(death["f"] // 30) * 30 - 30, game_end=max(s["last"] for s in seats.values()))
                events.append(ev)
                frames.add(ev["sample"])
            # Dives it lived through.
            run = []
            for k in kills + [None]:
                if k is not None and (not run or k[0] - run[-1][0] <= 300):
                    run.append(k)
                    continue
                if len(run) >= 3 and (death is None or death["f"] - run[-1][0] > 900):
                    mid = (run[0][0] // 30) * 30
                    hp = [t[3] for t in trace if run[0][0] - 60 <= t[0] <= run[-1][0] + 150]
                    where = next((t for t in trace if t[0] >= mid), trace[-1])
                    events.append(dict(base, kind="dive", f=run[0][0], x=where[1], z=where[2], seconds=(run[-1][0] - run[0][0]) / 30, took=[(r[1], r[2], r[3]) for r in run],
                                       hp_before=hp[0] if hp else None, hp_low=min(hp) if hp else None, from_start=math.hypot(where[1] - start[0], where[2] - start[1]), sample=mid))
                    frames.add(mid)
                run = [k] if k is not None else []
    if not events:
        return {"dir": os.path.basename(d), "seats": len(seats), "events": []}
    at = units_at(seats, frames)
    armed = ("army", "hover", "ship", "aircraft", "commander")
    for ev in events:
        his, ours, towers = collections.Counter(), collections.Counter(), collections.Counter()
        his_metal = ours_metal = 0
        for team, units in at.get(ev["sample"], {}).items():
            enemy = seats[team]["ally"] != ev["ally"]
            for n, cls, m, x, z, hp in units:
                if math.hypot(x - ev["x"], z - ev["z"]) > NEAR or (not enemy and team == ev["team"] and n == ev["com"]):
                    continue
                if cls in armed:
                    if enemy:
                        his[n] += 1
                        his_metal += m if cls != "commander" else 0
                    else:
                        ours[n] += 1
                        ours_metal += m if cls != "commander" else 0
                elif cls == "turret":
                    (his if enemy else towers)[n] += 1
        ev.update(his=dict(his), his_metal=his_metal, ours=dict(ours), ours_metal=ours_metal, towers=dict(towers))
    return {"dir": os.path.basename(d), "seats": len(seats), "events": events}


def build():
    dirs = sorted(set(os.path.dirname(p) for p in glob.glob(ARCHIVE + "/*-replay-*/record-*.jsonl") + glob.glob(os.path.join(REPO, "run/matches/*-replay-*/record-*.jsonl"))))
    dirs = [d for d in dirs if len(glob.glob(d + "/record-*.jsonl")) in (2, 16)]
    with mp.Pool(10) as pool, open(CACHE, "w") as out:
        for i, g in enumerate(pool.imap_unordered(game, dirs)):
            if g:
                out.write(json.dumps(g) + "\n")
            if i % 40 == 0:
                print(f"{i}/{len(dirs)}", file=sys.stderr)


def share(n, d):
    return f"{n} of {d} ({100 * n / d:.0f} %)" if d else "0"


def med(v):
    return st.median(v) if v else float("nan")


def kind_of(units, U):
    """What stood round the commander, by metal: raiders (every reach inside the D-gun's 262), stand-off (a reach
    over the laser's 300), aircraft, or mixed. Commanders and towers are left out of the shares."""
    shares = collections.Counter()
    for n, count in units.items():
        u = U.get(n) or {}
        if u.get("class") == "commander" or not u.get("speed"):
            continue
        reach = u.get("range") or 0
        cls = "aircraft" if "flies" in (u.get("flags") or []) else "raiders" if reach <= 262 else "stand-off" if reach > 300 else "between"
        shares[cls] += count * (u.get("metal") or 0)
    total = sum(shares.values())
    if not total:
        return "nothing armed that walks"
    cls, most = shares.most_common(1)[0]
    return cls if most >= 0.7 * total else "mixed"


def read(rows=False):
    games, seen = [], set()
    for line in open(CACHE):
        g = json.loads(line)
        # A public replay we replayed twice under two run numbers is one game: the same events at the same frames.
        sig = (g["seats"], tuple(sorted((e["team"], e["kind"], e["f"], round(e["x"])) for e in g["events"])))
        if g["events"] and sig in seen:
            continue
        seen.add(sig)
        games.append(g)
    U = json.load(open(os.path.join(REPO, "crates/bot/data/units.json")))["units"]
    reach = lambda n: (U.get(n) or {}).get("range") or 0
    metal_of = lambda took: sum(m for _, m, _ in took)
    quart = lambda v: "{:.0f} / {:.0f} / {:.0f}".format(sorted(v)[len(v) // 4], med(v), sorted(v)[3 * len(v) // 4]) if v else "none"
    for seats, label in ((2, "duels"), (16, "8v8 team games")):
        gs = [g for g in games if g["seats"] == seats]
        deaths = [e for g in gs for e in g["events"] if e["kind"] == "death"]
        dives = [e for g in gs for e in g["events"] if e["kind"] == "dive"]
        # A death with no attacker named: the game's end (a side that resigns or loses has its commanders blown up)
        # when it stood at nine tenths or over three seconds before, else a blast or a missile (the engine names none).
        fought = [e for e in deaths if e["by"]]
        ended = [e for e in deaths if not e["by"] and e["seconds"] <= 3]
        unnamed = [e for e in deaths if not e["by"] and e["seconds"] > 3]
        print(f"\n== {label}: {len(gs)} games; commander deaths {len(deaths)}: {len(fought)} to a named unit, {len(unnamed)} worn down with no attacker named (a blast, a missile), {len(ended)} blown up whole (a resignation or the side's end); dives lived through {len(dives)}")
        print("  killing blow by:", collections.Counter(e["by"] for e in fought).most_common(18))
        print(f"  the killing blow from a reach over the laser's 300: {share(sum(1 for e in fought if reach(e['by']) > 300), len(fought))}; inside the D-gun's 262: {share(sum(1 for e in fought if 0 < reach(e['by']) <= 262), len(fought))}; an aircraft: {share(sum(1 for e in fought if 'flies' in ((U.get(e['by']) or {}).get('flags') or [])), len(fought))}; a tower: {share(sum(1 for e in fought if e['by'] in U and not U[e['by']].get('speed')), len(fought))}; another commander: {share(sum(1 for e in fought if e['by'] in COMMANDERS), len(fought))}")
        print(f"  minute of death: quartiles {quart([e['f'] / 1800 for e in fought])}; from its start: quartiles {quart([e['from_start'] for e in fought])}, within 800 of it {share(sum(1 for e in fought if e['from_start'] <= 800), len(fought))}")
        print(f"  seconds from nine tenths to dead: quartiles {quart([e['seconds'] for e in fought])}")
        print(f"  its own armed units within {NEAR}: none at {share(sum(1 for e in fought if not e['ours_metal']), len(fought))}; towers of its own there at {share(sum(1 for e in fought if e['towers']), len(fought))}")
        print(f"  what it killed in its last 30 s: nothing at {share(sum(1 for e in fought if not e['took']), len(fought))}; metal quartiles {quart([metal_of(e['took']) for e in fought])}")
        print("  by what stood round it a second before (deaths) or at the first kill (dives lived through), the other side's metal there plus what the commander took:")
        table = collections.defaultdict(lambda: {"death": [], "dive": []})
        for e in fought + dives:
            table[kind_of(e["his"], U)][e["kind"]].append(e)
        for cls, t in sorted(table.items(), key=lambda kv: -len(kv[1]["death"]) - len(kv[1]["dive"])):
            d, l = t["death"], t["dive"]
            spent_d = [e["his_metal"] + metal_of(e["took"]) for e in d]
            spent_l = [e["his_metal"] for e in l]
            n_d = [sum(c for n, c in e["his"].items() if n not in COMMANDERS and (U.get(n) or {}).get("speed")) + len(e["took"]) for e in d]
            n_l = [sum(c for n, c in e["his"].items() if n not in COMMANDERS and (U.get(n) or {}).get("speed")) for e in l]
            print(f"    {cls:26s} it died {len(d):4d} (metal quartiles {quart(spent_d)}, units {quart(n_d)}, lost to it {med([metal_of(e['took']) for e in d]):.0f}) | it lived {len(l):4d} (metal {quart(spent_l)}, units {quart(n_l)}, lost to it {med([metal_of(e['took']) for e in l]):.0f}; its lowest health {med([e['hp_low'] for e in l if e['hp_low'] is not None]):.0f} %)")
        # A commander with a few raiders beside it that died to a gun farther off is not the raiders' kill: the
        # last hit has to come from inside the D-gun's reach.
        raid = [e for e in fought + dives if kind_of(e["his"], U) == "raiders" and (e["kind"] == "dive" or 0 < reach(e["by"]) <= 262)]
        print("  raiders alone, by how many came (present plus killed) | it died to one of them / it lived:")
        for lo, hi in ((0, 6), (6, 10), (10, 15), (15, 25), (25, 999)):
            def n_of(e):
                n = sum(c for n, c in e["his"].items() if n not in COMMANDERS and (U.get(n) or {}).get("speed"))
                return n + (len(e["took"]) if e["kind"] == "death" else 0)
            d = [e for e in raid if e["kind"] == "death" and lo <= n_of(e) < hi]
            l = [e for e in raid if e["kind"] == "dive" and lo <= n_of(e) < hi]
            guarded = lambda es: sum(1 for e in es if e["towers"] or e["ours_metal"])
            print(f"    {lo:3d} to {hi - 1:3d}: {len(d):4d} / {len(l):4d}   (with towers or soldiers of its own there: {guarded(d)} / {guarded(l)})")
        types = collections.Counter(n for e in dives for n, _, _ in e["took"])
        print("  what died in the dives it lived through:", types.most_common(14))
        print(f"  those dives: units killed quartiles {quart([len(e['took']) for e in dives])}, most {max([len(e['took']) for e in dives], default=0)}; seconds {quart([e['seconds'] for e in dives])}; at home (within 800 of its start) {share(sum(1 for e in dives if e['from_start'] <= 800), len(dives))}; with towers of its own within {NEAR} {share(sum(1 for e in dives if e['towers']), len(dives))}; alone (no tower, no soldier) {share(sum(1 for e in dives if not e['towers'] and not e['ours_metal']), len(dives))}")
        if rows:
            for e in sorted(deaths + dives, key=lambda e: (e["dir"], e["f"])):
                took = collections.Counter(n for n, _, _ in e["took"])
                print(f"    {e['dir'][:40]:40s} {clock(e['f'])} {e['kind']:5s} {e['com']} by {e.get('by')} | his {e['his']} ({e['his_metal']:.0f}) | own {e['ours']} towers {e['towers']} | took {dict(took)} | {e['seconds']:.0f} s, {e['from_start']:.0f} from start")


if __name__ == "__main__":
    if not os.path.exists(CACHE):
        build()
    read("--rows" in sys.argv)
