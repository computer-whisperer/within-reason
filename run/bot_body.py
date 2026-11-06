#!/usr/bin/env python3
"""What the distilled bot's strike body did, from its body trace in engine.log (`body:` every ten seconds, `launch:`
and `repelled:` as they happen; `bots/distilled/src/army.rs` `run_strike`, `launch`): per game, a line a minute with
the body's size and weight, where it stood as a share of the way from our start to his (0 % at home, 100 % at his
start), how much of it was together, whether it was raiding or pushing, what it was doing (advancing, gathering,
outweighed) and where it was sent; then the launches and repulses with their clocks.

    run/bot_body.py <batch dir or label> [--to MINUTE] [--game NN] [--every SECONDS]
    run/bot_body.py <label>[,<label>...] --summary      one line a batch: the result, the wins' clocks and the
        body's cohesion as rates (2026-11-06: the share of the trace's lines of a body of eight or more with under
        60 % of it together, the retreats called with under 30 % of the body near the core, the walks home for want
        of a target and how many of them by a body over three times his shown army, the guard's share of the
        soldiers while a body is out)

Written 2026-10-20 after the film of apex-v029-distilled-comet-8 game 02 (an army as large as his spread as a cloud
over our half from 16:00) and the census of dist6-barb-hardagg-comet-1 game 00 (107 Stouts and 44 Shellshockers
wandering mid-map at 25:00 against 18 turrets and never reaching them).
"""
import glob, json, math, re, sys

BODY = re.compile(r"\[distilled ai=\d\] f=(\d+) body: n=(\d+) w(-?\d+) middle=\((-?\d+),(-?\d+)\) together=(\d+)/(\d+) near: ours w(-?\d+) his w(-?\d+) mode=(\w+) what=(\w+) order=(\S+) guard n=(\d+) w(\d+) his_seen w(-?\d+) repelled w(\d+)")
LAUNCH = re.compile(r"\[distilled ai=\d\] f=(\d+) launch: guard n=(\d+) w(-?\d+) joins a body of n=(\d+) w(-?\d+)")
NO_TARGET = re.compile(r"\[distilled ai=\d\] f=(\d+) no target: n=(\d+) w(\d+) against his shown w(\d+)")
REPEL = re.compile(r"\[distilled ai=\d\] f=(\d+) repelled: n=(\d+) w(\d+) \(near w(-?\d+)\) by his w(-?\d+) at \((-?\d+),(-?\d+)\)")
START = re.compile(r"\[distilled ai=\d\] home \((-?\d+), (-?\d+)\), enemy start taken as \((-?\d+), (-?\d+)\)")


def clock(frame):
    return f"{frame // 1800}:{(frame // 30) % 60:02d}"


def read(game):
    home = enemy = None
    bodies, launches, repels = [], [], []
    for line in open(f"{game}/engine.log", errors="replace"):
        if "[distilled ai=" not in line:
            continue
        m = START.search(line)
        if m:
            home, enemy = (int(m[1]), int(m[2])), (int(m[3]), int(m[4]))
            continue
        m = BODY.search(line)
        if m:
            g = [int(x) if x.lstrip("-").isdigit() else x for x in m.groups()]
            bodies.append(dict(frame=g[0], n=g[1], w=g[2], x=g[3], z=g[4], together=g[5], of=g[6], ours=g[7], his=g[8], mode=g[9], what=g[10], order=g[11], guard_n=g[12], guard_w=g[13], his_seen=g[14], repelled=g[15]))
            continue
        m = LAUNCH.search(line)
        if m:
            launches.append(tuple(int(x) for x in m.groups()))
            continue
        m = REPEL.search(line)
        if m:
            repels.append(tuple(int(x) for x in m.groups()))
    return home, enemy, bodies, launches, repels


def summary(names):
    print("batch | W-L-D | wins' minutes (median) | body lines | apart | retreats (under 30 % near) | walks home (over 3x his shown) | guard's share of the soldiers, body out")
    for name in names:
        batch = name if "/" in name else sorted(glob.glob(f"run/matches/*-{name}"))[-1]
        outcomes, wins = {"Win": 0, "Loss": 0}, []
        try:
            for line in open(f"{batch}/results.jsonl"):
                r = json.loads(line)
                outcomes[r["outcome"]] = outcomes.get(r["outcome"], 0) + 1
                if r["outcome"] == "Win":
                    wins.append(r["game_minutes"])
        except OSError:
            pass
        lines = apart = phantom = repulses = walks = big = 0
        shares = []
        for game in sorted(glob.glob(f"{batch}/[0-9][0-9]")):
            _, _, bodies, _, repels = read(game)
            for b in bodies:
                if b["n"] >= 8:
                    lines += 1
                    apart += b["together"] < 0.6 * b["of"]
                    shares.append(b["guard_w"] / max(1, b["guard_w"] + b["w"]))
            repulses += len(repels)
            phantom += sum(1 for _, _, w, near, _, _, _ in repels if near < 0.3 * w)
            for line in open(f"{game}/engine.log", errors="replace"):
                m = NO_TARGET.search(line)
                if m:
                    walks += 1
                    big += int(m[3]) >= 3 * max(1, int(m[4]))
        wins.sort()
        draws = sum(v for k, v in outcomes.items() if k not in ("Win", "Loss"))
        shares.sort()
        print(f"{name.split('/')[-1]} | {outcomes['Win']}-{outcomes['Loss']}-{draws} | {' '.join(f'{w:.0f}' for w in wins)} ({wins[len(wins) // 2]:.1f}) | {lines} | {100 * apart // max(1, lines)}% | {repulses} ({phantom}) | {walks} ({big}) | {100 * shares[len(shares) // 2]:.0f}%" if wins and shares else f"{name} | {outcomes} | no wins or no body lines")


def main():
    args = sys.argv[1:]
    if "--summary" in args:
        return summary(args[0].split(","))
    name = args[0]
    batch = name if "/" in name else sorted(glob.glob(f"run/matches/*-{name}"))[-1]
    to = float(args[args.index("--to") + 1]) if "--to" in args else 999
    only = args[args.index("--game") + 1] if "--game" in args else None
    every = int(args[args.index("--every") + 1]) if "--every" in args else 60
    results = {}
    try:
        results = {json.loads(l)["index"]: json.loads(l) for l in open(f"{batch}/results.jsonl")}
    except OSError:
        pass
    for game in sorted(glob.glob(f"{batch}/[0-9][0-9]")):
        nn = game[-2:]
        if only and nn != only:
            continue
        r = results.get(int(nn))
        home, enemy, bodies, launches, repels = read(game)
        head = f"{r['outcome'][0]}{r['game_minutes']:.0f} seed {r['seed']} {r['our_side']}" if r else "?"
        print(f"== {nn} {head}; launches {len(launches)}, repulses {len(repels)}")
        if not home:
            print("   no start line")
            continue
        span = math.hypot(enemy[0] - home[0], enemy[1] - home[1]) or 1.0

        def progress(x, z):
            # The projection onto the line from our start to his, as a share of its length.
            return ((x - home[0]) * (enemy[0] - home[0]) + (z - home[1]) * (enemy[1] - home[1])) / span / span * 100

        def order_progress(order):
            m = re.match(r"\((-?\d+),(-?\d+)\)", order)
            return f"{progress(int(m[1]), int(m[2])):4.0f}%" if m else order

        print("   clock | body n   w | at (way to his start) | together | near ours/his | mode  what       -> sent    | guard n   w | his seen | repelled")
        last = -1
        for b in bodies:
            if b["frame"] / 1800 >= to or b["frame"] - last < every * 30:
                continue
            last = b["frame"]
            print(f"   {clock(b['frame']):>5} | {b['n']:4} {b['w']:6} | ({b['x']:4},{b['z']:4}) {progress(b['x'], b['z']):4.0f}% | {100 * b['together'] / max(b['of'], 1):3.0f}% | {b['ours']:5}/{b['his']:<5} | {b['mode']:<5} {b['what']:<10} -> {order_progress(b['order']):>7} | {b['guard_n']:4} {b['guard_w']:5} | {b['his_seen']:8} | {b['repelled']:6}")
        if launches:
            print("   launches: " + "  ".join(f"{clock(f)} guard {n} (w{w}) -> body {bn} (w{bw})" for f, n, w, bn, bw in launches[:14]) + (" ..." if len(launches) > 14 else ""))
        if repels:
            print("   repulses: " + "  ".join(f"{clock(f)} n={n} w{w} (near w{near}) by w{his} at {progress(x, z):.0f}%" for f, n, w, near, his, x, z in repels[:14]) + (" ..." if len(repels) > 14 else ""))


main()
