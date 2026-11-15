#!/usr/bin/env python3
"""What each new order did to a group's block, and whether the block was whole again before the next one came.

usage: run/group_orders.py <match dir> [--least N] [--from M:SS] [--list]

A new order is counted as the micro engine counts one (`crates/micro/src/group.rs`, `Group::update`): another kind
(a walk and an advance on one route are the same march), a route point moved more than 48, another party attacked,
another ward followed. Each is read off the hands' log: the group's task at one request against its task at the
next, with the pick the record holds between the two (`d` rows of kind `group`). An order with a pick is Jev's; one
without is the hands' own (a group that reached its place holds there). Two changes in one second are one order,
and so is the hands' own with Jev's a second after it (a task read as `hold` between two places).
The block is read off the record's samples: `formed` is the share of the group's members within the formed band of
their slot, every company together.

  new orders   to groups of `--least` soldiers or more (20 unless given), and the seconds to the next one
  alone        orders with no other within 20 s before and 60 s after: how far `formed` fell within 10 s and how
               long until it was back within 5 points of the three seconds before the order
  unfinished   Jev's orders that took `formed` 15 points or more below the three seconds before them and were
               followed by the next order before it had been back within 5 points (`--list` prints each)
  turn         the angle at the body between the place it was going to and the new one (the ranks turn at 60)
  closing      how fast the body's middle closed on the place it was ordered to, a step of five seconds with no new
               order in it, against the group's full pace and the members' own walking, by the seconds since the
               last new order; a body under 600 from its place is left out (it spreads onto its slots there)
"""
import argparse, bisect, collections, json, math, os, re, statistics

import jevlog

ROUTE_MOVED = 48.0


def clock(second):
    return f"{second // 60}:{second % 60:02d}"


def cell(p):
    return chr(ord("A") + int(p[0] // 1024)) + str(int(p[1] // 1024) + 1)


def middle(points):
    points = list(points)
    return (sum(p[0] for p in points) / len(points), sum(p[1] for p in points) / len(points))


def read_seat(match_dir, log):
    """Every declared group of one seat: its block second by second, and its new orders."""
    seat = re.search(r"jev-(\d+)", log).group(1)
    blocks, picks = collections.defaultdict(dict), collections.defaultdict(list)
    for line in open(os.path.join(match_dir, f"record-{seat}.jsonl")):
        if '"t":"s"' in line and '"groups"' in line:
            r = json.loads(line)
            own = {u[0]: (u[2], u[3]) for u in r["own"]}
            for name, g in r["groups"].items():
                members = set(g["formed"]) | {e[0] for e in g["exceptions"]}
                blocks[name][r["f"] // 30] = {
                    "n": len(members),
                    "formed": len(g["formed"]) / max(1, len(members)),
                    "at": {m: own[m] for m in members if m in own},
                    "full": g["full_pace"],
                    "arrived": g["arrived"],
                    "engaged": g["engaged"] is not None,
                    "passing": g.get("passing") is not None,
                    "walking": g["pace"] > 0,
                }
        elif '"kind":"group"' in line:
            r = json.loads(line)
            move = r["inputs"]["move"].split(".", 1)[1]
            if not move.startswith(("send_", "ahead_")):
                picks[r["inputs"]["actor"]].append((r["f"], move))
    tasks = collections.defaultdict(list)
    for line in jevlog.lines(log):
        if '"t":"call"' not in line:
            continue
        r = json.loads(line)
        for g in r.get("groups") or []:
            t = g.get("task") or {}
            tasks["group_" + g["name"]].append((r["f"], t.get("kind"), t.get("place") or t.get("ward"), t.get("to")))
    return {name: (blocks[name], orders(tasks[name], picks[name]), tasks[name]) for name in blocks}


def orders(tasks, picks):
    walk = ("move_to", "fight_to")
    out, last, party, i = [], None, None, 0
    for frame, kind, what, to in tasks:
        pick, attacked = None, None
        while i < len(picks) and picks[i][0] < frame:
            pick = picks[i]
            if pick[1].startswith(("attack_party_", "shell_party_")):
                attacked = pick[1]
            i += 1
        if last is not None:
            was_kind, was_what, was_to = last
            moved = (was_to is None) != (to is None) or (to is not None and math.dist(was_to, to) > ROUTE_MOVED)
            new = (
                (kind != was_kind and not (kind in walk and was_kind in walk and not moved))
                or (kind in walk and kind == was_kind and moved)
                or (kind == "engage" and kind == was_kind and attacked is not None and attacked != party)
                or (kind == "follow" and kind == was_kind and what != was_what)
            )
            if new:
                second = (pick[0] if pick else frame) // 30
                if out and (second == out[-1]["second"] or (second - out[-1]["second"] == 1 and not out[-1]["pick"])):
                    out[-1].update(to=to, pick=(pick[1] if pick else None) or out[-1]["pick"])
                else:
                    out.append({"second": second, "pick": pick[1] if pick else None, "was": was_to, "to": to, "kind": kind})
        party = attacked or party
        last = (kind, what, to)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("--least", type=int, default=20)
    ap.add_argument("--from", dest="start", default="0:00")
    ap.add_argument("--list", action="store_true")
    args = ap.parse_args()
    start = int(args.start.split(":")[0]) * 60 + int(args.start.split(":")[1])
    least = args.least
    groups = {}
    for log in jevlog.logs(args.dir):
        groups.update(read_seat(args.dir, log))

    def big(block, second):
        s = block.get(second - 1) or block.get(second)
        return s if s and s["n"] >= least and second >= start else None

    # new orders
    events, gaps = [], []
    for name, (block, new, _) in groups.items():
        mine = [e for e in new if big(block, e["second"])]
        events += [(name, e) for e in mine]
        gaps += [b["second"] - a["second"] for a, b in zip(mine, mine[1:])]
    jevs = sum(1 for _, e in events if e["pick"])
    print(f"{args.dir}: from {clock(start)}, groups of {least} or more")
    print(f"  new orders: {len(events)} (Jev's pick {jevs}, the hands' own {len(events) - jevs})")
    if gaps:
        gaps.sort()
        print(f"    seconds to the group's next: quartiles {gaps[len(gaps) // 4]} / {gaps[len(gaps) // 2]} / {gaps[3 * len(gaps) // 4]}; under 5 s {100 * sum(1 for g in gaps if g < 5) // len(gaps)}%, under 20 s {100 * sum(1 for g in gaps if g < 20) // len(gaps)}%")

    # alone
    fell, back = [], []
    alone = 0
    for name, (block, new, _) in groups.items():
        seconds = [e["second"] for e in new]
        for e in new:
            t = e["second"]
            if not big(block, t) or any(o != t and -20 <= o - t <= 60 for o in seconds):
                continue
            before = [block[k]["formed"] for k in range(t - 3, t) if k in block]
            after = [(block[k]["formed"], k) for k in range(t, t + 11) if k in block]
            if not before or not after:
                continue
            alone += 1
            pre, (low, at) = statistics.median(before), min(after)
            if pre - low >= 0.15:
                fell.append(pre - low)
                back.append(next((k - t for k in range(at, t + 61) if k in block and block[k]["formed"] >= pre - 0.05), None))
    if alone:
        came = sorted(b for b in back if b is not None)
        print(f"  alone: {alone} orders with no other near; {len(fell)} took `formed` 15 points or more down (median fall {100 * statistics.median(fell):.0f} points)" if fell else f"  alone: {alone} orders with no other near; none took `formed` 15 points down")
        if came:
            print(f"    back within 5 points after: quartiles {came[len(came) // 4]} / {came[len(came) // 2]} / {came[3 * len(came) // 4]} s ({len(came)} of {len(back)}; the rest not within 60 s)")

    # unfinished
    followed, unfinished = 0, []
    for name, (block, new, _) in groups.items():
        for a, b in zip(new, new[1:]):
            t, u = a["second"], b["second"]
            s = big(block, t)
            if not s or not a["pick"]:
                continue
            before = [block[k]["formed"] for k in range(t - 3, t) if k in block]
            between = [(block[k]["formed"], k) for k in range(t, u) if k in block]
            if not before or not between:
                continue
            followed += 1
            pre, (low, at) = statistics.median(before), min(between)
            if pre - low >= 0.15 and not any(f >= pre - 0.05 for f, k in between if k > at):
                where = cell(middle(s["at"].values())) if s["at"] else "?"
                unfinished.append((t, name, s["n"], where, a["pick"], pre, low, between[-1][0], u - t, b["pick"] or "holds (the hands)"))
    print(f"  unfinished: {len(unfinished)} of Jev's {followed} orders that another followed")
    if unfinished:
        waits = sorted(r[8] for r in unfinished)
        print(f"    the next order came after: quartiles {waits[len(waits) // 4]} / {waits[len(waits) // 2]} / {waits[3 * len(waits) // 4]} s; `formed` then, median {100 * statistics.median(r[7] for r in unfinished):.0f}% against {100 * statistics.median(r[5] for r in unfinished):.0f}% before the order")
        by_group = collections.Counter(r[1] for r in unfinished).most_common(6)
        print("    most: " + ", ".join(f"{g} {n}" for g, n in by_group))
    if args.list:
        for t, name, n, where, pick, pre, low, end, wait, then in sorted(unfinished):
            print(f"    {clock(t):>6} {name:14} {n:3} at {where:3} {pick[:26]:26} formed {100 * pre:3.0f}% -> {100 * low:3.0f}%, {100 * end:3.0f}% when the next came {wait:3} s later: {then[:26]}")

    # turn
    turns = []
    for name, (block, new, _) in groups.items():
        going = None
        for e in new:
            s = big(block, e["second"])
            was = e["was"] or going
            going = e["was"] or going
            if e["to"]:
                if s and was and s["at"]:
                    at = middle(s["at"].values())
                    a, b = (was[0] - at[0], was[1] - at[1]), (e["to"][0] - at[0], e["to"][1] - at[1])
                    if math.hypot(*a) > 200 and math.hypot(*b) > 200:
                        turns.append(math.degrees(math.acos(max(-1.0, min(1.0, (a[0] * b[0] + a[1] * b[1]) / (math.hypot(*a) * math.hypot(*b)))))))
                going = e["to"]
    if turns:
        print(f"  turn: {len(turns)} new places with the last one and the new one both 200 or more off; under 30 degrees {100 * sum(1 for a in turns if a < 30) // len(turns)}%, 60 or more {100 * sum(1 for a in turns if a >= 60) // len(turns)}%, 90 or more {100 * sum(1 for a in turns if a >= 90) // len(turns)}%")

    # closing
    steps = []
    for name, (block, new, tasks) in groups.items():
        seconds = [e["second"] for e in new]
        asked = [f // 30 for f, *_ in tasks]
        for t, s in block.items():
            e = block.get(t + 5)
            i = bisect.bisect_right(asked, t) - 1
            if t < start or s["n"] < least or s["arrived"] or not e or e["n"] < least or i < 0:
                continue
            _, kind, _, to = tasks[i]
            j = bisect.bisect_right(seconds, t)
            if kind not in ("move_to", "fight_to") or not to or (j < len(seconds) and seconds[j] <= t + 5):
                continue
            both = [m for m in s["at"] if m in e["at"]]
            if len(both) < least / 2:
                continue
            a, b = middle(s["at"][m] for m in both), middle(e["at"][m] for m in both)
            if math.dist(a, to) < 600:
                continue
            walked = sum(math.dist(s["at"][m], e["at"][m]) for m in both) / len(both) / 5
            steps.append((t - seconds[j - 1] if j else 999, (math.dist(a, to) - math.dist(b, to)) / 5, walked, s["full"], s["formed"], s["engaged"] or e["engaged"], s["passing"]))

    def table(title, rows):
        if not rows:
            return
        print(f"  closing, {title}: {len(rows)} group-seconds")
        print(f"    {'since the last new order':26} {'seconds':>7} {'closing':>8} {'walking':>8} {'full pace':>9} {'closing/full':>12} {'formed':>7}")
        for label, lo, hi in (("0-4 s", 0, 5), ("5-9 s", 5, 10), ("10-19 s", 10, 20), ("20-39 s", 20, 40), ("40 s or more", 40, 10**9), ("all", 0, 10**9)):
            x = [r for r in rows if lo <= r[0] < hi]
            if x:
                closing, full = statistics.mean(r[1] for r in x), statistics.mean(r[3] for r in x)
                print(f"    {label:26} {len(x):7} {closing:8.0f} {statistics.mean(r[2] for r in x):8.0f} {full:9.0f} {closing / full:12.2f} {100 * statistics.mean(r[4] for r in x):6.0f}%")

    table("no enemy in any member's reach", [r for r in steps if not r[5]])
    table("no enemy in reach, every company in ranks", [r for r in steps if not r[5] and not r[6]])
    table("no enemy in reach, a company forming up beyond tight ground", [r for r in steps if not r[5] and r[6]])
    table("an enemy in some member's reach", [r for r in steps if r[5]])


if __name__ == "__main__":
    main()
