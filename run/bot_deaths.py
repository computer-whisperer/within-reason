#!/usr/bin/env python3
"""Where and to what the soldiers of a bot of bots/ die, from the bot's own `died:` lines in engine.log (the death
instrument, `bots/distilled/src/lib.rs` `unit_died`): per game, the deaths by minute band, by what killed them
(a turret, a mobile unit, his commander, or an engine cause), by half of the map, and by the body and what it was
doing; then the killer types; then, for the pushing body's deaths, what stood within 600 of the fallen (his turrets'
and mobile units' metal against ours).

    run/bot_deaths.py <batch dir or label> [--to MINUTE] [--full NN] [--killers N]

Written 2026-10-20: the exchange is the distilled bot's wall and the census says nothing about where or to what.
"""
import collections, glob, json, re, statistics, sys

LINE = re.compile(
    r"\[distilled ai=\d\] f=(\d+) died: (\S+) unit (\d+) at \((-?\d+),(-?\d+)\) half=(\w+) body=(\w+) mode=(\w+) killer=(\S+) kind=(\w+) attacker=(\S+) "
    r"near600: turrets (\d+) w(\d+) mobiles (\d+) w(\d+) ours (\d+) w(\d+); body n=(\d+) w(\d+) order=(\S+)"
)
BANDS = [(0, 5), (5, 10), (10, 15), (15, 20), (20, 30), (30, 99)]
SOLDIER_BODIES = {"strike", "guard", "scout", "raiders", "none"}


def deaths_of(game):
    out = []
    for line in open(f"{game}/engine.log", errors="replace"):
        m = LINE.search(line)
        if not m:
            continue
        g = m.groups()
        out.append(dict(frame=int(g[0]), minute=int(g[0]) / 1800, type=g[1], unit=int(g[2]), x=int(g[3]), z=int(g[4]), half=g[5], body=g[6], mode=g[7], killer=g[8], kind=g[9], attacker=g[10], turrets=int(g[11]), turret_w=int(g[12]), mobiles=int(g[13]), mobile_w=int(g[14]), ours=int(g[15]), ours_w=int(g[16]), body_n=int(g[17]), body_w=int(g[18]), order=g[19]))
    return out


def clock(minute):
    return f"{int(minute)}:{int(minute * 60) % 60:02d}"


def main():
    args = sys.argv[1:]
    name = args[0]
    batch = name if "/" in name else sorted(glob.glob(f"run/matches/*-{name}"))[-1]
    to = float(args[args.index("--to") + 1]) if "--to" in args else 999
    full = args[args.index("--full") + 1] if "--full" in args else None
    killers_n = int(args[args.index("--killers") + 1]) if "--killers" in args else 6
    results = {}
    try:
        results = {json.loads(l)["index"]: json.loads(l) for l in open(f"{batch}/results.jsonl")}
    except OSError:
        pass
    print(f"== {batch} (deaths to {to:.0f}:00)")
    for game in sorted(glob.glob(f"{batch}/[0-9][0-9]")):
        nn = game[-2:]
        r = results.get(int(nn))
        every = [d for d in deaths_of(game) if d["minute"] < to]
        # Soldiers by body; the builders and buildings the instrument logs since 2026-10-20 evening are counted apart.
        deaths = [d for d in every if d["body"] in SOLDIER_BODIES]
        others = collections.Counter(d["body"] for d in every if d["body"] not in SOLDIER_BODIES)
        head = f"{r['outcome'][0]}{r['game_minutes']:.0f} seed {r['seed']} {r['our_side']}" if r else "?"
        if others:
            head += " | lost " + " ".join(f"{k} {n}" for k, n in others.most_common())
        if full == nn:
            print(f"-- {nn} {head}: every death")
            for d in every:
                print(f"  {clock(d['minute'])} {d['type']:<10} {d['half']:<4} {d['body']}/{d['mode']:<5} by {d['killer']:<12} ({d['kind']}) near: turrets {d['turrets']} w{d['turret_w']} mobiles {d['mobiles']} w{d['mobile_w']} ours {d['ours']} w{d['ours_w']}; body n={d['body_n']} w{d['body_w']} -> {d['order']}")
            continue
        bands = collections.Counter()
        for d in deaths:
            for lo, hi in BANDS:
                if lo <= d["minute"] < hi:
                    bands[(lo, hi)] += 1
        kinds = collections.Counter(d["kind"] for d in deaths)
        halves = collections.Counter(d["half"] for d in deaths)
        bodies = collections.Counter(f"{d['body']}/{d['mode']}" for d in deaths)
        killers = collections.Counter(d["killer"] for d in deaths).most_common(killers_n)
        print(f"-- {nn} {head}: {len(deaths)} soldiers dead")
        print("   by minute: " + "  ".join(f"{lo}-{hi if hi < 99 else ''}: {bands[(lo, hi)]}" for lo, hi in BANDS if bands[(lo, hi)]))
        print("   by killer: " + "  ".join(f"{k} {n}" for k, n in kinds.most_common()) + " | by half: " + "  ".join(f"{k} {n}" for k, n in halves.most_common()))
        print("   by body:   " + "  ".join(f"{k} {n}" for k, n in bodies.most_common()))
        print("   killers:   " + "  ".join(f"{k} {n}" for k, n in killers))
        push = [d for d in deaths if d["body"] == "strike" and d["mode"] == "push"]
        if push:
            med = lambda key: statistics.median(d[key] for d in push)
            outgunned = sum(1 for d in push if 3 * d["turret_w"] + d["mobile_w"] > 1.3 * d["ours_w"])
            print(f"   push deaths {len(push)}: within 600 at the median his turrets {med('turrets'):.0f} (w{med('turret_w'):.0f}) mobiles {med('mobiles'):.0f} (w{med('mobile_w'):.0f}) ours {med('ours'):.0f} (w{med('ours_w'):.0f}); {outgunned} died where his weight (turrets x3) was over 1.3x ours")


main()
