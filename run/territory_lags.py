#!/usr/bin/env python3
"""The plan's lags against the hands' choices (T-LAGS; docs/design/2026-10-25-territory-regions.md, step 3).

    run/territory_lags.py BATCH-OR-LABEL... [--minutes 2,5,8,10,12,15,18] [--column NAME] [--top 4]

From the bot's `lags:` lines (every 10 s: each column the program runs over the lead, seconds behind its plan at the
program's own rate on it, units behind, the rate in units a minute) and its `lag:` lines (at each order a dispatch
gave: the open items with the most seconds behind less the walk, the first being what the matching rule would give).
Per batch, pooled over its games:
- per minute shown, the columns most behind (the median over the minute's lines and games of the seconds and units
  behind, and in how many games the column was run);
- per minute, how often the order the cascade gave is the item the matching rule would have given, and the commonest
  disagreements (the cascade's order, the rule's item);
- with --column, that column's seconds and units behind minute by minute, and the orders given in each minute by
  kind, summed over the games (factories' orders apart).
"""
import collections, glob, os, re, statistics, sys

LAGS = re.compile(r"f=(\d+) lags: (.*)")
ITEM = re.compile(r"(\S+) (-?\d+) s \((-?[\d.]+), (-?[\d.]+)/min\)")
ORDER = re.compile(r"f=(\d+) order: unit (\d+) (\w+): (.*?), \d+ off, \d+ from home \(([^)]*)\)")
LAG = re.compile(r"f=(\d+) lag: unit (\d+) would take (.*)")
LEAD, RUN_LEAST = 45.0, 0.1


def order_key(text):
    if text.startswith("build "):
        return text.split()[1]
    m = re.match(r"help unit \d+ \((\w+)\)", text)
    if m:
        return "help:" + m.group(1)
    if text.startswith("assist") or text.startswith("pad-clear"):
        return "guard"
    if text.startswith("walk"):
        return "walk"
    return text.split()[0]


def pick_key(item):
    name = item.split()[0]
    name = re.sub(r"\(frames\)|\[r\d+\]", "", name)
    return "guard" if name.startswith("guard:") else name


def main():
    args = sys.argv[1:]
    value = lambda flag, default: args[args.index(flag) + 1] if flag in args else default
    minutes = [int(m) for m in value("--minutes", "2,5,8,10,12,15,18").split(",")]
    column, top = value("--column", None), int(value("--top", "4"))
    flags = {"--minutes", "--column", "--top"}
    names = [a for i, a in enumerate(args) if not a.startswith("--") and (i == 0 or args[i - 1] not in flags)]
    for name in names:
        batch = name if "/" in name else sorted(glob.glob(f"run/matches/*-{name}"))[-1]
        behind = collections.defaultdict(lambda: collections.defaultdict(list))  # minute -> column -> [(seconds, units, game)]
        agree = collections.defaultdict(lambda: [0, 0])
        disagree = collections.defaultdict(collections.Counter)
        orders_by_minute = collections.defaultdict(collections.Counter)
        games = sorted(glob.glob(f"{batch}/[0-9][0-9]"))
        for g, game in enumerate(games):
            if not os.path.exists(f"{game}/engine.log"):
                continue
            last_order = {}
            for line in open(f"{game}/engine.log", errors="replace"):
                if "[territory ai=" not in line:
                    continue
                m = LAGS.search(line)
                if m:
                    minute = int(m.group(1)) // 1800
                    for col, sec, units, per_minute in ITEM.findall(m.group(2)):
                        # A trickle (less than a tenth of a unit within the lead, as work.rs's RUN_LEAST drops since
                        # terr163; terr157-162 logged them).
                        if float(per_minute) * LEAD / 60.0 < RUN_LEAST:
                            continue
                        behind[minute][col].append((float(sec), float(units), g))
                    continue
                m = ORDER.search(line)
                if m:
                    minute = int(m.group(1)) // 1800
                    key = order_key(m.group(4))
                    last_order[m.group(2)] = (int(m.group(1)), key, m.group(5))
                    if not m.group(3).endswith(("vp", "lab", "avp", "alab", "ap", "aap")):
                        orders_by_minute[minute][key] += 1
                    continue
                m = LAG.search(line)
                if m and m.group(2) in last_order and last_order[m.group(2)][0] == int(m.group(1)):
                    minute = int(m.group(1)) // 1800
                    _, key, rule = last_order[m.group(2)]
                    pick = m.group(3).split(" | ")[0]
                    if pick == "nothing open":
                        continue
                    agree[minute][1] += 1
                    if key == pick_key(pick):
                        agree[minute][0] += 1
                    else:
                        disagree[minute][(f"{key} ({rule})", pick_key(pick))] += 1
        print(f"== {os.path.basename(batch)}: {len(games)} games")
        for minute in minutes:
            cols = behind.get(minute, {})
            ranked = sorted(cols.items(), key=lambda kv: -statistics.median(s for s, _, _ in kv[1]))[:top]
            text = " | ".join(f"{c} {statistics.median(s for s, _, _ in v):.0f} s ({statistics.median(u for _, u, _ in v):.1f}, {len({gg for _, _, gg in v})} games)" for c, v in ranked)
            a = agree.get(minute, [0, 0])
            share = f"{a[0]}/{a[1]} orders the rule's pick" if a[1] else "no orders"
            worst = ", ".join(f"{k[0]} for {k[1]} x{n}" for k, n in disagree[minute].most_common(3))
            print(f"   {minute:2}:00 most behind: {text or '-'}")
            print(f"         {share}; the commonest other: {worst or '-'}")
        if column:
            print(f"   {column} by minute (seconds and units behind, median over the games; the orders given that minute, summed over the games):")
            for minute in sorted(behind):
                v = behind[minute].get(column)
                if not v:
                    continue
                kinds = ", ".join(f"{k} {n}" for k, n in orders_by_minute[minute].most_common(5))
                print(f"   {minute:2}:00 {statistics.median(s for s, _, _ in v):5.0f} s {statistics.median(u for _, u, _ in v):5.1f} units | {kinds}")


main()
