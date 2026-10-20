#!/usr/bin/env python3
"""What the distilled bot's strike body did, from its body trace in engine.log (`body:` every ten seconds, `launch:`
and `repelled:` as they happen; `bots/distilled/src/army.rs` `run_strike`, `launch`): per game, a line a minute with
the body's size and weight, where it stood as a share of the way from our start to his (0 % at home, 100 % at his
start), how much of it was together, whether it was raiding or pushing, what it was doing (advancing, gathering,
outweighed) and where it was sent; then the launches and repulses with their clocks.

    run/bot_body.py <batch dir or label> [--to MINUTE] [--game NN] [--every SECONDS]

Written 2026-10-20 after the film of apex-v029-distilled-comet-8 game 02 (an army as large as his spread as a cloud
over our half from 16:00) and the census of dist6-barb-hardagg-comet-1 game 00 (107 Stouts and 44 Shellshockers
wandering mid-map at 25:00 against 18 turrets and never reaching them).
"""
import glob, json, math, re, sys

BODY = re.compile(r"\[distilled ai=\d\] f=(\d+) body: n=(\d+) w(-?\d+) middle=\((-?\d+),(-?\d+)\) together=(\d+)/(\d+) near: ours w(\d+) his w(\d+) mode=(\w+) what=(\w+) order=(\S+) guard n=(\d+) w(\d+) his_seen w(\d+) repelled w(\d+)")
LAUNCH = re.compile(r"\[distilled ai=\d\] f=(\d+) launch: guard n=(\d+) w(-?\d+) joins a body of n=(\d+) w(-?\d+)")
REPEL = re.compile(r"\[distilled ai=\d\] f=(\d+) repelled: n=(\d+) w(\d+) \(near w(\d+)\) by his w(\d+) at \((-?\d+),(-?\d+)\)")
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


def main():
    args = sys.argv[1:]
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
