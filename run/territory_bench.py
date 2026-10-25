#!/usr/bin/env python3
"""The territory bot's bench, read per start (docs/design/2026-10-25-territory-regions.md, "The bench"; run by
run/territory_bench.sh, whose batches are labelled terrN-TAG-START).

    run/territory_bench.py TAG [TAG...]

For each of the six starts and each TAG, the newest batch run/matches/*-terrN-TAG-START, game by game: metal income
a second at 5, 10, 15 and 20 minutes; the 20:00 mark as a share of the plan's forecast; our units lost to his
weapons; when the first factory and the first advanced factory were begun; the factory sites the bot chose, those it
found none for, and those the host refused; the commander's farthest from home at an order; where his start was
taken from. The plan's forecast is the bot's first plan (its `targets:` line at frame 0): the planner's own ceiling
from that start under the bot's own settings, the yardstick off Comet.

With two TAGs or more the last is read against the others: per start and mark, the others' range over all their
games, and the last's median marked - below it or + above it. A change counts when it moves the bench and no start
falls below its spread (the design's "What counts").
"""
import glob, json, os, re, statistics, sys

STARTS = [
    ("comet-his-start", "Comet Catcher, thebluegecko's start"),
    ("comet-boxes", "Comet Catcher, the lobby's boxes"),
    ("great-divide", "Great Divide"),
    ("glitters", "All That Glitters"),
    ("mithril", "Mithril Mountain"),
    ("quicksilver", "Quicksilver"),
]
MARKS = (5, 10, 15, 20)

OURS = re.compile(r"\[territory ai=\d+\] f=(\d+) (.*)")
CURVE = re.compile(r"curve: .*? metal -?\d+/\d+ \+([\d.]+)")
FACTORY = re.compile(r"^(arm|cor|leg)(lab|vp|ap|sy|hp|fhp)$")
ADVANCED = re.compile(r"^(arm|cor|leg)a(lab|vp|ap|sy)$")
BEGUN = re.compile(r"^begun: (\w+) by unit")
COMMANDER_ORDER = re.compile(r"^order: unit \d+ \w+com: .*, \d+ off, (\d+) from home")
REFUSED = re.compile(r"^dropped: unit \d+ (\w+) at .*: no site$")
HIS_START = re.compile(r"enemy start taken as \([-\d, ]+\) \(([^)]*)\)")
TARGET = re.compile(r"(\d+):mex\S* .*? inc(-?\d+) bank")


def clock(frame):
    return f"{frame // 1800}:{frame // 30 % 60:02d}" if frame is not None else "never"


def num(v):
    return "-" if v is None else (f"{v:.1f}" if v < 100 else f"{v:,.0f}")


def game(path):
    g = dict(marks={}, forecast={}, lost=0, plant=None, advanced=None, sites=0, none=0, refused=0, farthest=0, his="?")
    for line in open(f"{path}/engine.log", errors="replace"):
        m = OURS.search(line)
        if not m:
            continue
        frame, text = int(m.group(1)), m.group(2)
        if text.startswith("curve: "):
            c = CURVE.match(text)
            if c:
                g["marks"][frame // 1800] = float(c.group(1))
        elif text.startswith("died: ") and "death=Weapon" in text:
            g["lost"] += 1
        elif text.startswith("begun: "):
            name = BEGUN.match(text).group(1)
            if g["plant"] is None and FACTORY.match(name):
                g["plant"] = frame
            if g["advanced"] is None and ADVANCED.match(name):
                g["advanced"] = frame
        elif text.startswith("order: "):
            o = COMMANDER_ORDER.match(text)
            if o:
                g["farthest"] = max(g["farthest"], int(o.group(1)))
        elif text.startswith("factory: no site"):
            g["none"] += 1
        elif text.startswith("factory: "):
            g["sites"] += 1
        elif text.startswith("dropped: "):
            r = REFUSED.match(text)
            if r and (FACTORY.match(r.group(1)) or ADVANCED.match(r.group(1))):
                g["refused"] += 1
        elif text.startswith("targets: ") and frame == 0 and not g["forecast"]:
            for minute, income in TARGET.findall(text.split(" | factory kinds")[0]):
                g["forecast"][int(minute)] = float(income)
        elif "enemy start taken as" in text:
            h = HIS_START.search(text)
            g["his"] = h.group(1) if h else "?"
    return g


def batch(tag, start):
    found = sorted(glob.glob(f"run/matches/*-terr*-{tag}-{start}"), key=lambda d: os.path.basename(d).split("-")[0])
    return found[-1] if found else None


def main():
    tags = [a for a in sys.argv[1:] if not a.startswith("-")]
    if not tags:
        print(__doc__.split("\n\n")[1].strip())
        sys.exit(2)
    medians = {}
    for start, title in STARTS:
        print(f"== {title} ({start})")
        print("   tag/batch          game seed side         5:00  10:00  15:00  20:00 | of plan | lost | factory  advanced | sites none refused | commander | his start")
        values = {}
        for tag in tags:
            d = batch(tag, start)
            if not d:
                print(f"   {tag}: no batch")
                continue
            results = {}
            if os.path.exists(f"{d}/results.jsonl"):
                results = {r["index"]: r for r in map(json.loads, open(f"{d}/results.jsonl"))}
            label = os.path.basename(d).split("-")[1]
            for path in sorted(glob.glob(f"{d}/[0-9][0-9]")):
                index = int(path[-2:])
                r = results.get(index, {})
                side = f"{r.get('our_side', '?')} {r.get('our_corner') or ''}".strip()
                if not os.path.exists(f"{path}/engine.log"):
                    print(f"   {tag + ' ' + label:18} {index:02d}   aborted")
                    continue
                g = game(path)
                for m in MARKS:
                    if m in g["marks"]:
                        values.setdefault(tag, {}).setdefault(m, []).append(g["marks"][m])
                row = " ".join(f"{num(g['marks'].get(m)):>6}" for m in MARKS)
                share = g["marks"].get(20, 0) / g["forecast"][20] if g["forecast"].get(20) else None
                print(f"   {tag + ' ' + label:18} {index:02d} {r.get('seed', '?'):>4} {side:11} {row} | {('-' if share is None else f'{share:.2f}'):>7} | {g['lost']:4} | {clock(g['plant']):>7}  {clock(g['advanced']):>8} | {g['sites']:5} {g['none']:4} {g['refused']:7} | {g['farthest']:9,} | {g['his']}")
                values.setdefault((tag, "forecast"), []).append(g["forecast"])
        # The plan's forecast from these starts: one start gives one number, the boxes' alternating corners two.
        forecasts = [f for tag in tags for f in values.get((tag, "forecast"), [])]
        if forecasts:
            spans = []
            for m in MARKS:
                v = [f[m] for f in forecasts if m in f]
                spans.append(f"{m}:00 " + ("-" if not v else num(min(v)) if min(v) == max(v) else f"{num(min(v))}-{num(max(v))}"))
            print("   the plan's forecast (first plan, the bot's settings): " + ", ".join(spans))
        for tag in tags:
            medians[(start, tag)] = {m: statistics.median(v) for m, v in values.get(tag, {}).items()}
        if len(tags) > 1 and tags[-1] in values:
            last = values[tags[-1]]
            parts = []
            for m in MARKS:
                ref = [v for tag in tags[:-1] for v in values.get(tag, {}).get(m, [])]
                if not ref or m not in last:
                    continue
                med = statistics.median(last[m])
                flag = "-" if med < min(ref) else "+" if med > max(ref) else " "
                parts.append(f"{m}:00 {num(min(ref))}-{num(max(ref))} vs {num(med)}{flag}")
            print(f"   {tags[-1]} against {', '.join(tags[:-1])}: " + "; ".join(parts))
        print()
    print("== medians")
    print("   start             tag           5:00  10:00  15:00  20:00")
    for start, _ in STARTS:
        for tag in tags:
            m = medians.get((start, tag))
            if m:
                print(f"   {start:17} {tag:10} " + " ".join(f"{num(m.get(k)):>6}" for k in MARKS))


main()
