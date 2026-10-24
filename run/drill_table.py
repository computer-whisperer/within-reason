#!/usr/bin/env python3
"""The drill bench's table (`duel --drill`; docs/design/2026-10-24-formation-drills.md §3, D6).

    run/drill_table.py <batch dir> [<batch dir> ...] [--by x,form_x,lane_x] [--types]

One row per arm (by default the army, its spawn formation and its lane mode) from `duels.csv`: duels, the mean of the
first, median and last arrival (seconds from the first order), the spread (root mean square from the army's centre)
and the median nearest-friend distance at the median arrival, the speed share (each unit's speed over the second as a
share of its type's, averaged over units and seconds up to the median arrival, a unit counted until its own arrival),
the spin-up (seconds until the median
unit is at 90% of its type's speed: every army spawns facing south and turns first), a reverse's recovery after the
turn, and a gather's units produced and median seconds from appearance to arrival. `--types` adds, from the
per-second logs (`seconds/*.jsonl`, the drill's meta line first), each type's own arrival at the goal and speed share
before the median arrival, which is where a mixed column's stringing shows (type speeds from the unit glossary,
`crates/bot/data/units.json`). A reverse's per-type arrival is at the start after the turn (the way back); its speed
share covers both legs.
"""
import csv
import json
import os
import statistics
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ARRIVED = 300.0
UP_TO_SPEED = 0.9


def mean(values):
    values = [v for v in values if v is not None]
    return statistics.mean(values) if values else None


def number(text):
    try:
        return float(text)
    except (TypeError, ValueError):
        return None


def fmt(value, digits=0):
    if value is None:
        return "-"
    return f"{value:.{digits}f}"


def rows_of(batch):
    path = os.path.join(batch, "duels.csv")
    with open(path) as handle:
        rows = [r for r in csv.DictReader(handle) if r.get("drill")]
    return rows


def type_speeds():
    path = os.path.join(HERE, "..", "crates", "bot", "data", "units.json")
    try:
        with open(path) as handle:
            data = json.load(handle)
    except OSError:
        return {}
    data = data.get("units", data)
    return {name: (entry or {}).get("speed") or 0.0 for name, entry in data.items() if isinstance(entry, dict)}


def per_type(batch, row, speeds):
    """From one duel's per-second log: each type's median arrival at the goal and mean speed share before the army's
    median arrival. None when the log is missing."""
    log = row.get("log")
    if not log:
        return None
    path = os.path.join(batch, "seconds", log)
    if not os.path.exists(path):
        return None
    meta = None
    seconds = []
    with open(path) as handle:
        for line in handle:
            try:
                r = json.loads(line)
            except ValueError:
                continue
            if "meta" in r:
                meta = r["meta"]
            elif "t" in r:
                seconds.append(r)
    if meta is None or not seconds:
        return None
    names = meta.get("x", [])
    goal = meta.get("goal")
    # A reverse arrives back at the start, after its turn.
    after = None
    if meta.get("drill") == "reverse":
        goal = meta.get("start")
        after = meta.get("turn_at_s")
    median_arrival = number(row.get("arrive_median_s"))
    arrival = {}
    shares = {}
    last = {}
    for r in seconds:
        t = r["t"]
        for index, x, z, _health in r.get("x", []):
            name = names[index] if index < len(names) else f"unit{index}"
            if index in last and index not in arrival and (median_arrival is None or t <= median_arrival):
                lx, lz, lt = last[index]
                speed = speeds.get(name) or 0.0
                if speed > 0 and t > lt:
                    moved = ((x - lx) ** 2 + (z - lz) ** 2) ** 0.5 / (t - lt)
                    shares.setdefault(name, []).append(min(moved / speed, 1.5))
            near = goal is not None and ((x - goal[0]) ** 2 + (z - goal[1]) ** 2) ** 0.5 <= ARRIVED
            if near and index not in arrival and (after is None or t > after):
                arrival[index] = t
            last[index] = (x, z, t)
    by_type = {}
    for index, t in arrival.items():
        name = names[index] if index < len(names) else f"unit{index}"
        by_type.setdefault(name, []).append(t)
    out = {}
    for name in set(list(by_type) + list(shares)):
        arrivals = sorted(by_type.get(name, []))
        out[name] = {
            "arrive_median": arrivals[len(arrivals) // 2] if arrivals else None,
            "arrive_last": arrivals[-1] if arrivals else None,
            "share": mean(shares.get(name, [])),
            "n": len(arrivals),
        }
    return out


def main():
    args = sys.argv[1:]
    by = ["x", "form_x", "lane_x"]
    types = False
    batches = []
    i = 0
    while i < len(args):
        if args[i] == "--by":
            by = args[i + 1].split(",")
            i += 2
        elif args[i] == "--types":
            types = True
            i += 1
        else:
            batches.append(args[i])
            i += 1
    if not batches:
        print(__doc__)
        sys.exit(2)
    speeds = type_speeds() if types else {}
    for batch in batches:
        rows = rows_of(batch)
        meta = {}
        try:
            with open(os.path.join(batch, "batch.json")) as handle:
                meta = json.load(handle).get("drill") or {}
        except OSError:
            pass
        label = os.path.basename(os.path.normpath(batch))
        print(f"## {label}: drill {meta.get('kind', '?')}, {meta.get('distance', '?')} elmos, order {meta.get('order', '?')}, reissue {meta.get('reissue_s') or 'once'}, {len(rows)} duels")
        if not rows:
            continue
        groups = {}
        for r in rows:
            groups.setdefault(tuple(r.get(k, "") for k in by), []).append(r)
        head = "| " + " | ".join(by) + " | n | arrived (timeout) | first / median / last s | spread | nearest friend | speed share | spin-up s | recover s | produced | join s |"
        print(head)
        print("|" + "---|" * (len(by) + 10))
        for key, rs in sorted(groups.items()):
            arrived = sum(1 for r in rs if r.get("reason") == "arrived")
            timeouts = sum(1 for r in rs if r.get("reason") == "timeout")
            cells = [
                str(len(rs)),
                f"{arrived} ({timeouts})",
                " / ".join(fmt(mean(number(r.get(c)) for r in rs)) for c in ("arrive_first_s", "arrive_median_s", "arrive_last_s")),
                fmt(mean(number(r.get("spread_arrival")) for r in rs)),
                fmt(mean(number(r.get("nn_arrival")) for r in rs)),
                fmt(mean(number(r.get("speed_share")) for r in rs), 2),
                fmt(mean(number(r.get("spin_up_s")) for r in rs), 1),
                fmt(mean(number(r.get("recover_s")) for r in rs), 1),
                fmt(mean(number(r.get("produced")) for r in rs)),
                fmt(mean(number(r.get("join_median_s")) for r in rs)),
            ]
            print("| " + " | ".join(key) + " | " + " | ".join(cells) + " |")
            if types:
                per = {}
                for r in rs:
                    for name, v in (per_type(batch, r, speeds) or {}).items():
                        p = per.setdefault(name, {"arrive_median": [], "arrive_last": [], "share": []})
                        for k in p:
                            p[k].append(v[k])
                for name, p in sorted(per.items()):
                    print(f"|   {name} (speed {fmt(speeds.get(name))}) |" + " |" * (len(by) - 1) + f" {len(p['share'])} | | {fmt(mean(p['arrive_median']))} / {fmt(mean(p['arrive_last']))} | | | {fmt(mean(p['share']), 2)} | | | | |")
        print()


if __name__ == "__main__":
    main()
