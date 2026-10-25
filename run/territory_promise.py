#!/usr/bin/env python3
"""Each plan's promise by unit type against what came to stand (T-PROMISE; docs/design/2026-10-25-territory-walks.md).

    run/territory_promise.py TAG [--start START] [--top N]

For the newest batch of each start labelled terrN-TAG-START: every plan's `promise:` line (its count of each unit type
at its snapshot and 1, 2 and 5 minutes on, the work paid on frames counted) against the `standing:` lines (each type's
finished units, frames and the frames' progress, every 30 s), pooled over the games and over the plans made in each
window. Per type, most planned work first (the growth in units times the type's build time): the growth the plans
promised over 2 and 5 minutes, and the share of it that came, counting the frames' progress as the plan does
("work") and counting finished units only ("stood"). A spot is counted as the plan counts it: a basic extractor
whether or not its upgrade has replaced it, the upgrade apart; `spots under upgrade` is how many spots had their
basic extractor gone to an upgrade's frame (the engine takes the basic down when the frame goes up, the plan keeps
its income). `at the plan's start` compares each plan's own count at its snapshot with the game's then.
"""
import glob, json, os, re, statistics, sys
from bisect import bisect_left
from collections import defaultdict

STARTS = ["comet-his-start", "comet-boxes", "great-divide", "glitters", "mithril", "quicksilver"]
WINDOWS = ((1, 5), (5, 11), (11, 18))
HORIZONS = (60, 120, 300)
OURS = re.compile(r"\[territory ai=\d+\] f=(\d+) (promise|standing): (.*)")


def kind(name):
    """The type's name without its faction, so the Armada and Cortex games pool."""
    return re.sub(r"^(arm|cor|leg)", "", name)


def read(path):
    dumps = glob.glob(f"{path}/territory-*.json")
    defs = {u["name"]: u for u in json.load(open(dumps[0]))["hello"]["unit_defs"]} if dumps else {}
    extracts = {n: d["extracts_metal"] for n, d in defs.items() if d["extracts_metal"] > 0}
    basic_rate = min(extracts.values()) if extracts else 0
    upgrade = {kind(n) for n, e in extracts.items() if e > basic_rate * 1.5}
    basic = {kind(n) for n, e in extracts.items() if e <= basic_rate * 1.5}
    build_time = {kind(n): d["build_time"] for n, d in defs.items()}
    plans, samples = [], []
    for line in open(f"{path}/engine.log", errors="replace"):
        m = OURS.search(line)
        if not m:
            continue
        frame, what, text = int(m.group(1)), m.group(2), m.group(3)
        if what == "promise":
            head, *cells = text.split(" | ")
            t0 = float(head.split("=")[1])
            counts = {}
            for c in cells:
                name, values = c.rsplit(" ", 1)
                counts[kind(name)] = [float(v) for v in values.split("/")]
            plans.append((t0, counts))
        else:
            row = {}
            for c in text.split(" | "):
                parts = c.split(" ")
                if len(parts) == 4:
                    row[kind(parts[0])] = (int(parts[1]), int(parts[2]), float(parts[3]))
            samples.append((frame / 30, row))
    return plans, samples, basic, upgrade, build_time


def counts_at(samples, t, basic, upgrade):
    """The game's count per type at second t, as the plan counts it: (with the frames' progress, finished only), and
    the spots whose basic extractor is gone to an upgrade's frame. None outside the samples."""
    times = [s[0] for s in samples]
    i = bisect_left(times, t)
    if i == 0 or i >= len(samples):
        return None

    def one(row):
        work, stood = {}, {}
        for k, (finished, frames, progress) in row.items():
            work[k] = finished + progress
            stood[k] = finished
        under = sum(row.get(u, (0, 0, 0))[1] for u in upgrade)
        upgraded = sum(row.get(u, (0, 0, 0))[0] for u in upgrade)
        for b in basic:
            if b in row or upgraded or under:
                # The plan's basic count holds every spot taken: the upgraded ones and those under upgrade too.
                work[b] = work.get(b, 0) + upgraded + under
                stood[b] = stood.get(b, 0) + upgraded + under
        return work, stood, under

    (ta, ra), (tb, rb) = samples[i - 1], samples[i]
    (wa, sa, ua), (wb, sb, ub) = one(ra), one(rb)
    share = (t - ta) / (tb - ta)
    keys = set(wa) | set(wb)
    mix = lambda a, b: {k: a.get(k, 0) + (b.get(k, 0) - a.get(k, 0)) * share for k in keys}
    return mix(wa, wb), mix(sa, sb), ua + (ub - ua) * share


def main():
    args = sys.argv[1:]
    if not args or args[0].startswith("-"):
        print(__doc__.split("\n\n")[1].strip())
        sys.exit(2)
    tag = args[0]
    only = args[args.index("--start") + 1] if "--start" in args else None
    top = int(args[args.index("--top") + 1]) if "--top" in args else 8
    for start in STARTS:
        if only and start != only:
            continue
        found = sorted(glob.glob(f"run/matches/*-terr*-{tag}-{start}"), key=lambda d: os.path.basename(d).split("-")[0])
        if not found:
            continue
        # (window, horizon, type) -> [planned, work, stood]; (window, type) -> [plan's own start, the game's]
        acc = defaultdict(lambda: [0.0, 0.0, 0.0])
        start_gap = defaultdict(lambda: [0.0, 0.0, 0])
        under = defaultdict(list)
        times = {}
        for path in sorted(glob.glob(f"{found[-1]}/[0-9][0-9]")):
            if not os.path.exists(f"{path}/engine.log"):
                continue
            plans, samples, basic, upgrade, build_time = read(path)
            times.update(build_time)
            for t0, counts in plans:
                w = next((w for w in WINDOWS if w[0] <= t0 / 60 < w[1]), None)
                now = counts_at(samples, t0, basic, upgrade)
                if not w or not now:
                    continue
                under[w].append(now[2])
                for k, c in counts.items():
                    g = start_gap[(w, k)]
                    g[0] += c[0]
                    g[1] += now[0].get(k, 0)
                    g[2] += 1
                for h_index, h in enumerate(HORIZONS, start=1):
                    then = counts_at(samples, t0 + h, basic, upgrade)
                    if not then:
                        continue
                    for k, c in counts.items():
                        a = acc[(w, h, k)]
                        a[0] += c[h_index] - c[0]
                        a[1] += then[0].get(k, 0) - now[0].get(k, 0)
                        a[2] += then[1].get(k, 0) - now[1].get(k, 0)
        print(f"== {start} ({os.path.basename(found[-1])})")
        for w in WINDOWS:
            kinds = {k for (ww, h, k) in acc if ww == w}
            if not kinds:
                continue
            weight = lambda k: acc[(w, 300, k)][0] * times.get(k, 0)
            ranked = sorted(kinds, key=weight, reverse=True)[:top]
            total = [sum(acc[(w, h, k)][i] * times.get(k, 0) for k in kinds if acc[(w, h, k)][0] > 0) for h in HORIZONS for i in range(3)]
            print(f"   plans made in minutes {w[0]}-{w[1]} ({len(under[w])}); spots under upgrade at a plan's start {statistics.mean(under[w]):.1f}")
            print("     type            planned +2   work  stood | planned +5   work  stood | at the plan's start: plan / game")
            for k in ranked:
                p2, w2, s2 = acc[(w, 120, k)]
                p5, w5, s5 = acc[(w, 300, k)]
                g = start_gap[(w, k)]
                r = lambda x, p: f"{x / p:5.2f}" if abs(p) > 1e-9 else "    -"
                print(f"     {k:14} {p2:11.1f} {r(w2, p2)} {r(s2, p2)} | {p5:10.1f} {r(w5, p5)} {r(s5, p5)} | {g[0] / g[2]:6.1f} / {g[1] / g[2]:6.1f}")
            # The work over every type the plans grew, in build-power-seconds of build time.
            print(f"     all, by build time: +1 {total[1] / total[0]:.2f} work {total[2] / total[0]:.2f} stood | +2 {total[4] / total[3]:.2f} / {total[5] / total[3]:.2f} | +5 {total[7] / total[6]:.2f} / {total[8] / total[6]:.2f}")
        print()


main()
