#!/usr/bin/env python3
"""The commander's early opportunity cost, from the replay records (docs/design/2026-09-23-replay-survey.md): the
user (2026-09-24), on the fourth solar too early: "when you pause building and let the commander simply assist the
bot lab. That and the metal availability are the opportunity costs".

    run/replays/assist.py [--floors 25,40] [--until 240] [--player NAME] [--rows] [--ours run/matches/<batch>...]

Per side, over the first `--until` seconds (4:00): seconds the commander spent assisting the factory (a `guard`
command from the widget's command log, until its next command), its own builds and their clocks (the third and fourth
solar), the factory's units in the window and how fast they came against the factory alone (the build events, the
glossary's build times), and the metal: the bank at 1:00, 2:00, 3:00 and the share of seconds the store sat below 20
(stalled) or above 150 (floating). `--rows` prints every side; otherwise the pools' medians, winners against losers,
and `--player`'s own rows. `--ours` reads our own arena records (the bot's `cmd` lines have the same shape) so our
opening sits in the same table as the pros'.
"""
import argparse, collections, json, os, statistics, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
MANIFEST = os.path.join(REPO, "run/data/replays/manifest.jsonl")
SOLAR = ("armsolar", "corsolar")
FACTORY_SPEED = {"armvp": 150, "corvp": 150, "armlab": 100, "corlab": 100}


def glossary():
    g = json.load(open(os.path.join(REPO, "crates/bot/data/units.json")))["units"]
    rows = g if isinstance(g, list) else [dict(v, name=k) for k, v in g.items()]
    return {r["name"]: r for r in rows}


def clock(s):
    return "-" if s is None else f"{int(s) // 60}:{int(s) % 60:02d}"


def side_of(match_dir, team, until, g):
    rec = os.path.join(REPO, match_dir, f"record-{team}.jsonl")
    if not os.path.exists(rec):
        return None
    f = open(rec)
    header = json.loads(f.readline())
    defs = header["unit_defs"]
    com = None
    created = {}
    finished = []
    com_cmds = []  # (seconds, kind)
    metal = []     # (seconds, stored)
    for line in f:
        r = json.loads(line)
        s = r["f"] / 30.0
        if s > until + 1:
            break
        if r["t"] == "s":
            if com is None:
                for u in r["own"]:
                    if defs[u[1]]["name"] in ("armcom", "corcom"):
                        com = u[0]
            metal.append((s, r["m"][0]))
        elif r["t"] == "ev" and r["k"] == "created":
            created[r["u"]] = (s, defs[r["d"]]["name"] if r["d"] >= 0 else "?", r.get("by"))
        elif r["t"] == "ev" and r["k"] == "finished" and r["u"] in created:
            c = created[r["u"]]
            finished.append((c[0], s, c[1], c[2]))
        elif r["t"] == "cmd" and com is not None:
            for c in r["c"]:
                if c[1] == com:
                    com_cmds.append((s, str(c[0]).split(":")[0]))
    # Assist: from a guard command until the commander's next command of another kind (a build, a move).
    assist = 0.0
    guarding_from = None
    for s, kind in com_cmds:
        if kind == "guard":
            if guarding_from is None:
                guarding_from = s
        elif kind in ("build", "move", "fight", "attack", "stop", "repair", "reclaim"):
            if guarding_from is not None:
                assist += min(s, until) - guarding_from
                guarding_from = None
    if guarding_from is not None:
        assist += until - guarding_from
    first_guard = next((s for s, k in com_cmds if k == "guard"), None)
    com_builds = [(c0, name) for c0, f1, name, by in finished if by == com or (by is not None and created.get(by, (0, "", None))[1] in ("armcom", "corcom"))]
    solars = sorted(c0 for c0, name in com_builds if name in SOLAR)
    all_solars = sorted(c0 for c0, f1, name, by in finished if name in SOLAR)
    factory = next(((c0, f1, name) for c0, f1, name, by in finished if name in FACTORY_SPEED), None)
    fac_units = []
    if factory:
        for c0, f1, name, by in finished:
            b = created.get(by) if by else None
            if b and b[1] in FACTORY_SPEED and c0 >= factory[1] - 1:
                nominal = (g.get(name, {}).get("build_time") or 0) / FACTORY_SPEED[b[1]]
                fac_units.append((c0, f1 - c0, nominal, name))
    speedup = [n / max(t, 0.1) for c0, t, n, name in fac_units if n > 0]
    at = lambda sec: next((m for s, m in metal if s >= sec), None)
    window = [m for s, m in metal if 60 <= s <= until]
    return {
        "assist_s": round(assist), "first_guard": first_guard, "plant_done": factory[1] if factory else None, "plant": factory[2] if factory else None,
        "solar3": all_solars[2] if len(all_solars) > 2 else None, "solar4": all_solars[3] if len(all_solars) > 3 else None,
        "com_solars": len(solars), "com_builds": len(com_builds), "fac_units": len(fac_units),
        "speedup_med": round(statistics.median(speedup), 2) if speedup else None,
        "bank1": at(60), "bank2": at(120), "bank3": at(180),
        "stalled": round(100 * sum(1 for m in window if m < 20) / max(1, len(window))), "floating": round(100 * sum(1 for m in window if m > 150) / max(1, len(window))),
    }


def sides(floor, until, g, map_name):
    out = []
    for line in open(MANIFEST):
        r = json.loads(line)
        if not (r.get("keep") and r.get("card") and r.get("match")):
            continue
        if map_name and r.get("map_script") != map_name:
            continue
        oss = [p.get("os") for p in r["players"]]
        if any(o is None or o < floor for o in oss):
            continue
        winners = set(r.get("winners") or [])
        for p in r["players"]:
            row = side_of(r["match"], p["team"], until, g)
            if row is None:
                continue
            row.update(id=r["id"][:8], player=p["name"], os=p.get("os"), won=p["name"] in winners, faction=p.get("faction"), date=r["start_time"][:10])
            out.append(row)
    return out


def med(rows, key):
    vals = [r[key] for r in rows if r.get(key) is not None]
    return statistics.median(vals) if vals else None


def fmt(v, key):
    if v is None:
        return "-"
    if key in ("first_guard", "plant_done", "solar3", "solar4"):
        return clock(v)
    return f"{v:.0f}" if isinstance(v, float) and key not in ("speedup_med",) else str(v)


KEYS = ["assist_s", "first_guard", "plant_done", "solar3", "solar4", "com_solars", "com_builds", "fac_units", "speedup_med", "bank1", "bank2", "bank3", "stalled", "floating"]


def print_rows(rows):
    print(f"{'date':10} {'player':22} {'os':5} {'W':1} " + " ".join(f"{k:>11}" for k in KEYS))
    for r in rows:
        print(f"{r['date']:10} {r['player'][:22]:22} {r['os']:5} {'W' if r['won'] else 'L'} " + " ".join(f"{fmt(r[k], k):>11}" for k in KEYS))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--floors", default="25,40")
    ap.add_argument("--until", type=float, default=240.0)
    ap.add_argument("--player")
    ap.add_argument("--map", default="Comet Catcher Remake 1.8")
    ap.add_argument("--rows", action="store_true")
    ap.add_argument("--ours", nargs="*", default=[])
    a = ap.parse_args()
    g = glossary()
    for floor in a.floors.split(","):
        rows = sides(float(floor), a.until, g, a.map)
        print(f"== OS {floor}+: {len(rows)} sides, the first {clock(a.until)}")
        if a.rows:
            print_rows(rows)
        for label, rs in (("all", rows), ("winners", [r for r in rows if r["won"]]), ("losers", [r for r in rows if not r["won"]])):
            print(f"  {label:8} medians: " + ", ".join(f"{k} {fmt(med(rs, k), k)}" for k in KEYS))
        assisted = [r for r in rows if r["assist_s"] >= 30]
        print(f"  commander assisted the factory 30 s or more: {len(assisted)} of {len(rows)}; won {sum(1 for r in assisted if r['won'])} of {len(assisted)} against {sum(1 for r in rows if r['won'] and r['assist_s'] < 30)} of {len(rows) - len(assisted)}")
        early4 = [r for r in rows if r["solar4"] is not None and r["solar4"] < 120]
        print(f"  fourth solar before 2:00: {len(early4)} of {len(rows)}; their assist median {fmt(med(early4, 'assist_s'), 'assist_s')} s against {fmt(med([r for r in rows if r not in early4], 'assist_s'), 'assist_s')} s; won {sum(1 for r in early4 if r['won'])} of {len(early4)}")
    if a.ours:
        rows = []
        for m in a.ours:
            d = m if os.path.basename(m) == "00" else os.path.join(m, "00")
            row = side_of(os.path.relpath(d, REPO), 0, a.until, g)
            if row is None:
                print(f"{m}: no record", file=sys.stderr)
                continue
            try:
                res = [json.loads(l) for l in open(os.path.join(REPO, os.path.dirname(d) if os.path.basename(d) == "00" else d, "results.jsonl"))]
                won = res[0].get("outcome") == "Win"
            except Exception:
                won = False
            row.update(id=os.path.basename(os.path.dirname(d))[:8], player=os.path.basename(os.path.dirname(d)), os=0, won=won, faction="?", date="ours")
            rows.append(row)
        print(f"== ours: {len(rows)} games")
        print_rows(rows)
        print("  medians: " + ", ".join(f"{k} {fmt(med(rows, k), k)}" for k in KEYS))
    if a.player:
        rows = [r for r in sides(0.0, a.until, g, a.map) if a.player.lower() in r["player"].lower()]
        print(f"== {a.player}: {len(rows)} sides")
        print_rows(rows)
        print("  medians: " + ", ".join(f"{k} {fmt(med(rows, k), k)}" for k in KEYS))


if __name__ == "__main__":
    main()
