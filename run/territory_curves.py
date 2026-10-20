#!/usr/bin/env python3
"""The territory bot's economy against its own predictions and the oracle, from its lines in engine.log
(`bots/territory`: `curve:` a minute, `plan:` per re-plan with the predicted income and extractors at +60, +120 and
+300 s; docs/design/2026-10-20-territory-bot.md, decision 7).

    run/territory_curves.py <batch dir or label> [--marks 5,10,15,20] [--oracle] [--plans]

Per game: the realized curve at the marks (extractors, constructors, factories, nanos, build power, metal and energy
income, metal made) and the search's predictions against what came (the model error: the median and the worst of
predicted minus realized income at each look-ahead). With --oracle, `buildorder optimize --hello territory-0.json
--objective made --minutes END --palette roster --builders-only` is run once per game (cached as oracle.csv in the
match directory) and its curve printed beside the bot's (the search regret, in the model's own terms). With --plans,
every plan line.
"""
import glob, json, os, re, statistics, subprocess, sys

CURVE = re.compile(r"\[territory ai=\d\] f=(\d+) curve: extractors=(\d+) constructors=(\d+) factories=(\d+) nanos=(\d+) build_power=(\d+) metal (-?\d+)/(\d+) \+([\d.]+) -([\d.]+) energy (-?\d+)/(\d+) \+(\d+) -(\d+) made=(\d+) orders=(\d+) begun=(\d+) dropped=(\d+) skipped=(\d+) plans=(\d+)")
PLAN = re.compile(r"\[territory ai=\d\] f=(\d+) plan: n=(\d+) t0=(\d+) wall=(\d+)ms score=(-?\d+) horizon=(\d+) predicted \+60: income ([\d.]+) mex (\d+) \| \+120: income ([\d.]+) mex (\d+) \| \+300: income ([\d.]+) mex (\d+) \| end: income ([\d.]+) mex (\d+) cons (\d+) fac (\d+) nanos (\d+) made (\d+); stale ([\d.]+) s")
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def read(game):
    curves, plans = [], []
    for line in open(f"{game}/engine.log", errors="replace"):
        m = CURVE.search(line)
        if m:
            g = m.groups()
            curves.append(dict(minute=int(g[0]) // 1800, extractors=int(g[1]), constructors=int(g[2]), factories=int(g[3]), nanos=int(g[4]), build_power=int(g[5]), metal=int(g[6]), metal_income=float(g[8]), metal_usage=float(g[9]), energy=int(g[10]), energy_income=int(g[12]), made=int(g[14]), orders=int(g[15]), begun=int(g[16]), dropped=int(g[17]), skipped=int(g[18]), plans=int(g[19])))
            continue
        m = PLAN.search(line)
        if m:
            g = m.groups()
            plans.append(dict(frame=int(g[0]), n=int(g[1]), t0=int(g[2]), wall=int(g[3]), score=int(g[4]), horizon=int(g[5]), p60=(float(g[6]), int(g[7])), p120=(float(g[8]), int(g[9])), p300=(float(g[10]), int(g[11])), end=(float(g[12]), int(g[13]), int(g[14]), int(g[15]), int(g[16]), int(g[17])), stale=float(g[18])))
    return curves, plans


def oracle(game, minutes):
    """The oracle's curve by minute from the bot's dump, cached beside it."""
    dumps = sorted(glob.glob(f"{game}/territory-*.json"))
    csv = f"{game}/oracle.csv"
    if not dumps:
        return None
    if not os.path.exists(csv):
        cmd = [f"{REPO}/target/release/buildorder", "optimize", "--hello", dumps[0], "--objective", "made", "--minutes", str(minutes), "--palette", "roster", "--builders-only", "--factories", "3", "--constructors", "14", "--csv", csv, "--plan-out", f"{game}/oracle.txt"]
        with open(f"{game}/oracle.md", "w") as out:
            subprocess.run(cmd, stdout=out, stderr=subprocess.STDOUT, check=False)
        if not os.path.exists(csv):
            return None
    rows = {}
    with open(csv) as f:
        header = f.readline().strip().split(",")
        for line in f:
            r = dict(zip(header, line.strip().split(",")))
            t = float(r["t"])
            if t % 60 == 0:
                rows[int(t // 60)] = r
    return rows


def main():
    args = sys.argv[1:]
    name = args[0]
    batch = name if "/" in name else sorted(glob.glob(f"run/matches/*-{name}"))[-1]
    marks = [int(m) for m in (args[args.index("--marks") + 1] if "--marks" in args else "5,10,15,20").split(",")]
    results = {}
    try:
        results = {json.loads(l)["index"]: json.loads(l) for l in open(f"{batch}/results.jsonl")}
    except OSError:
        pass
    print(f"== {batch}")
    for game in sorted(glob.glob(f"{batch}/[0-9][0-9]")):
        nn = game[-2:]
        r = results.get(int(nn))
        if not os.path.exists(f"{game}/engine.log"):
            print(f"-- {nn}: no engine log (aborted)")
            continue
        curves, plans = read(game)
        head = f"{r['outcome'][0]}{r['game_minutes']:.0f} seed {r['seed']} {r['our_side']}" if r else "?"
        print(f"-- {nn} {head}: {len(curves)} curve lines, {len(plans)} plans" + (f", wall per plan median {statistics.median(p['wall'] for p in plans):.0f} ms, stale median {statistics.median(p['stale'] for p in plans):.1f} s" if plans else ""))
        by_minute = {c["minute"]: c for c in curves}
        print("   min | mex cons fac nanos | build power | metal/s  energy/s | bank    | made   | begun dropped skipped")
        for m in marks:
            c = by_minute.get(m)
            if not c:
                continue
            print(f"   {m:3} | {c['extractors']:3} {c['constructors']:4} {c['factories']:3} {c['nanos']:5} | {c['build_power']:11} | {c['metal_income']:7.1f} {c['energy_income']:8} | {c['metal']:7} | {c['made']:6} | {c['begun']:5} {c['dropped']:7} {c['skipped']:7}")
        if "--oracle" in args and curves:
            rows = oracle(game, max(marks))
            if rows:
                print("   oracle (the model's best from this start, no enemy):")
                print("   min | mex cons fac nanos | build power | metal/s  energy/s | made")
                made = 0.0
                for m in marks:
                    o = rows.get(m)
                    if not o:
                        continue
                    print(f"   {m:3} | {int(o['extractors']):3} {int(o['constructors']):4} {int(o['factories']):3} {int(o['nanos']):5} | {float(o['build_power']):11.0f} | {float(o['metal_income']):7.1f} {float(o['energy_income']):8.0f} | ")
            else:
                print("   oracle: no dump or the run failed (see oracle.md)")
        # The model error: each plan's prediction at +60/+120/+300 against the realized income at that clock, read
        # between the minute lines.
        def realized(t):
            lo, hi = by_minute.get(int(t // 60)), by_minute.get(int(t // 60) + 1)
            if not lo or not hi:
                return None
            share = (t % 60) / 60.0
            return lo["metal_income"] + (hi["metal_income"] - lo["metal_income"]) * share

        errors = {60: [], 120: [], 300: []}
        for p in plans:
            for dt, key in ((60, "p60"), (120, "p120"), (300, "p300")):
                real = realized(p["t0"] + dt)
                if real is not None and p[key][0] > 0:
                    errors[dt].append(p[key][0] - real)
        parts = []
        for dt, e in errors.items():
            if e:
                parts.append(f"+{dt}s: median {statistics.median(e):+.1f}, worst {max(e, key=abs):+.1f} over {len(e)}")
        if parts:
            print("   predicted minus realized metal/s: " + "; ".join(parts))
        if "--plans" in args:
            for p in plans:
                print(f"   plan {p['n']:2} at {p['t0'] // 60}:{p['t0'] % 60:02d} wall {p['wall']} ms stale {p['stale']:.1f} s score {p['score']} -> +60 {p['p60'][0]:.1f}/{p['p60'][1]} +120 {p['p120'][0]:.1f}/{p['p120'][1]} +300 {p['p300'][0]:.1f}/{p['p300'][1]} end {p['end'][0]:.1f}/{p['end'][1]} cons {p['end'][2]} fac {p['end'][3]} nanos {p['end'][4]} made {p['end'][5]}")


main()
