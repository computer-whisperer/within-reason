#!/usr/bin/env python3
"""The property drills' table (docs/design/2026-11-15-army-control-loop.md, section 9).

usage: run/drill_props.py [--place R] [--band B] <duel batch dir>...

Reads each duel's per-second log (`seconds/*.jsonl`) and the batch's command line, and says what the army did
against the closed-loop properties, from the units' positions and the orders sent alone: nothing here reads a
formation's own books but the two columns marked `lane`. One row a duel.

  first     second the first member stood within 300 of the place
  at80      second four fifths of the living members stood within R of the place (`--place`, default 900)
  rest      second from which no living member moved 5 elmos in a second again (blank: never within the run)
  last_ord  last second an order went to the army; ord_rest: orders sent after `rest`
  ord_s     orders a living member a second while the army walks: up to `at80`, or over the run when it never came
  lane      second the lane's group first said `arrived`, and `!` when it took the word back later
  off       at the end, the median and the largest distance of a member from the point it was last sent to
  from_p    at the end, the median and the largest distance of a member from the place
  walk      metres walked a member over the straight line from its start to where it ended (1.00: no milling)
  events    for each timed event of the command line (kill, build, late, shift, restart, and each nudge),
            members that moved more than the band (`--band`, default 128) relative to the body (their own
            displacement less the median one), four and ten seconds after it: `kill@15: 0/1`
  lag       a rear party's (`--behind`) median distance behind the body's median along the axis, at 1, 10 and 20
            seconds and at the end
"""
import glob
import json
import math
import os
import statistics
import sys


def events_of(argv):
    """(name, second) for each timed event of the command line; nudge as (every, by)."""
    out, nudge, by = [], None, 60.0
    it = iter(range(len(argv)))
    for i in it:
        a = argv[i]
        v = argv[i + 1] if i + 1 < len(argv) else ""
        if a == "--kill":
            out.append(("kill", float(v.split("@")[1])))
        elif a == "--build":
            out.append(("build", float(v.split(",")[-1])))
        elif a == "--late":
            out.append(("late", float(v)))
        elif a == "--shift":
            out.append(("shift", float(v.split("@")[1])))
        elif a == "--restart":
            out.append(("restart", float(v)))
        elif a == "--nudge":
            nudge = float(v)
        elif a == "--nudge-by":
            by = float(v)
    return out, nudge, by


def med(xs):
    return statistics.median(xs) if xs else float("nan")


def read(path):
    meta, rows = None, []
    with open(path) as f:
        for line in f:
            r = json.loads(line)
            if "meta" in r:
                meta = r["meta"]
            if "x" in r and "t" in r:
                rows.append(r)
    return meta, rows


def moved_at(pos, t, w, band):
    """Members that moved more than `band` relative to the body between the second before `t` and `w` seconds after
    it: each one's displacement less the median displacement. Of how many alive at both. (Read on the positions:
    the points the members were last sent to are sent at different moments on a march and differ by more than the
    band with nothing happening.)"""
    a, b = pos.get(int(t) - 1), pos.get(int(t) + w)
    if a is None or b is None:
        return None
    both = [i for i in a if i in b]
    if not both:
        return None
    dx = [b[i][0] - a[i][0] for i in both]
    dz = [b[i][1] - a[i][1] for i in both]
    mx, mz = med(dx), med(dz)
    return sum(1 for x, z in zip(dx, dz) if math.hypot(x - mx, z - mz) > band), len(both)


def row(path, argv, place_r, band):
    meta, rows = read(path)
    if not rows or not meta or "goal" not in meta:
        return None
    events, nudge, _ = events_of(argv)
    start, goal = meta["start"], meta["goal"]
    for a in range(len(argv) - 1):
        if argv[a] == "--shift":
            along, across = (float(v) for v in argv[a + 1].split("@")[0].split(","))
            ax = (goal[0] - start[0], goal[1] - start[1])
            n = math.hypot(*ax) or 1.0
            ux, uz = ax[0] / n, ax[1] / n
            goal = [goal[0] + ux * along - uz * across, goal[1] + uz * along + ux * across]
    # The army: the spawn indices whose type the force names (a wall's or a yard's structures are side x's too).
    types = meta.get("x", [])
    force = set()
    for a in range(len(argv) - 1):
        if argv[a] in ("--ours", "--behind"):
            force.update(part.split("*")[0].split("@")[0] for part in argv[a + 1].split("+"))
    army = lambda i: not force or (i < len(types) and types[i] in force)
    by_t = {r["t"]: r for r in rows}
    pos = {r["t"]: {u[0]: (u[1], u[2]) for u in r["x"] if u[3] > 0 and army(u[0])} for r in rows}
    ts = sorted(pos)
    last = ts[-1]
    alive_end = pos[last]
    d = lambda p, q: math.hypot(p[0] - q[0], p[1] - q[1])
    first = next((t for t in ts if any(d(p, goal) <= 300 for p in pos[t].values())), None)
    at80 = next((t for t in ts if pos[t] and sum(d(p, goal) <= place_r for p in pos[t].values()) >= 0.8 * len(pos[t])), None)
    # Rest: the last second anybody moved 5 or more, plus one.
    moving = [t for a, t in zip(ts, ts[1:]) if any(i in pos[a] and d(p, pos[a][i]) >= 5 for i, p in pos[t].items())]
    rest = (moving[-1] + 1) if moving else ts[0]
    rest = rest if rest <= last - 2 else None
    sent = {r["t"]: (r.get("sent") or [0, 0])[0] for r in rows}
    last_ord = max((t for t, n in sent.items() if n > 0), default=None)
    ord_rest = sum(n for t, n in sent.items() if rest is not None and t > rest)
    walking = [t for t in ts if t <= (at80 if at80 is not None else last)]
    unit_secs = sum(len(pos[t]) for t in walking)
    ord_s = sum(sent.get(t, 0) for t in walking) / unit_secs if unit_secs else None
    arrived = [r["t"] for r in rows if r.get("group", {}).get("arrived")]
    lane = arrived[0] if arrived else None
    took_back = bool(arrived) and any(not by_t[t].get("group", {}).get("arrived", False) for t in ts if t > arrived[0] and "group" in by_t[t])
    goals_end = {g[0]: (g[1], g[2]) for g in by_t[last].get("goals", [])}
    off = [d(p, goals_end[i]) for i, p in alive_end.items() if i in goals_end]
    from_p = [d(p, goal) for p in alive_end.values()]
    first_pos = {}
    walked = {}
    for a, t in zip(ts, ts[1:]):
        for i, p in pos[t].items():
            if i in pos[a]:
                walked[i] = walked.get(i, 0.0) + d(p, pos[a][i])
    for t in ts:
        for i, p in pos[t].items():
            first_pos.setdefault(i, p)
    ratios = [walked[i] / max(d(first_pos[i], alive_end[i]), 1.0) for i in alive_end if i in walked and d(first_pos[i], alive_end[i]) > 100]
    ev = []
    for name, t in events:
        a, b = moved_at(pos, t, 4, band), moved_at(pos, t, 10, band)
        ev.append(f"{name}@{t:g}: {a[0] if a else '-'}/{b[0] if b else '-'}")
    if nudge:
        counts = []
        t = nudge
        while t < last - 1:
            m = moved_at(pos, t, 1, band)
            if m:
                counts.append(m[0])
            t += nudge
        if counts:
            ev.append(f"nudge x{len(counts)}: median {med(counts):g}, most {max(counts)} moved")
    # The rear party: the spawn indices `--behind` added after the force (the meta's `rear`, else none).
    lag = ""
    rear = set(meta.get("rear") or [])
    if rear:
        ax = (goal[0] - start[0], goal[1] - start[1])
        n = math.hypot(*ax) or 1.0
        ux, uz = ax[0] / n, ax[1] / n
        proj = lambda p: p[0] * ux + p[1] * uz
        body_n = sum(1 for i in first_pos if i not in rear)
        depth = (math.ceil(min(body_n, 24) / 6) - 1) * 96 + 64

        def behind(t):
            body = [proj(p) for i, p in pos[t].items() if i not in rear]
            back = [proj(p) for i, p in pos[t].items() if i in rear]
            return (med(body) - med(back)) if body and back else None

        at = lambda t: "-" if t not in pos or behind(t) is None else f"{behind(t):.0f}"
        lag = f"behind the body's median by {at(1)} at 1 s, {at(10)} at 10 s, {at(20)} at 20 s, {at(last)} at the end (a rear rank stands about {depth - 64} behind)"
    f = lambda v: "-" if v is None else str(v)
    return {
        "duel": os.path.basename(path).replace(".jsonl", ""),
        "n": f"{len(alive_end)}/{len(first_pos)}",
        "first": f(first), "at80": f(at80), "rest": f(rest), "last_ord": f(last_ord), "ord_rest": str(ord_rest) if rest is not None else "-",
        "ord_s": "-" if ord_s is None else f"{ord_s:.2f}",
        "lane": f(lane) + ("!" if took_back else ""),
        "off": f"{med(off):.0f}/{max(off):.0f}" if off else "-",
        "from_p": f"{med(from_p):.0f}/{max(from_p):.0f}" if from_p else "-",
        "walk": f"{med(ratios):.2f}" if ratios else "-",
        "end": str(last),
        "events": "; ".join(ev),
        "lag": lag,
    }


def main():
    args = sys.argv[1:]
    place_r, band = 900.0, 128.0
    dirs = []
    while args:
        a = args.pop(0)
        if a == "--place":
            place_r = float(args.pop(0))
        elif a == "--band":
            band = float(args.pop(0))
        else:
            dirs.append(a)
    if not dirs:
        sys.exit(__doc__)
    cols = ["duel", "n", "first", "at80", "rest", "last_ord", "ord_rest", "ord_s", "lane", "off", "from_p", "walk", "end"]
    for batch in dirs:
        try:
            meta = json.load(open(os.path.join(batch, "batch.json")))
        except OSError:
            print(f"{batch}: no batch.json")
            continue
        argv = meta.get("argv", [])
        print(f"{os.path.basename(batch.rstrip('/'))}  ({meta.get('commit', '?')}): {' '.join(argv[1:])}")
        out = [r for r in (row(p, argv, place_r, band) for p in sorted(glob.glob(os.path.join(batch, "seconds", "*.jsonl")))) if r]
        if not out:
            print("  no drill log")
            continue
        widths = [max(len(c), *(len(r[c]) for r in out)) for c in cols]
        print("  " + "  ".join(c.ljust(w) for c, w in zip(cols, widths)))
        for r in out:
            print("  " + "  ".join(r[c].ljust(w) for c, w in zip(cols, widths)))
            if r["events"]:
                print("    events: " + r["events"])
            if r["lag"]:
                print("    rear party: " + r["lag"])


if __name__ == "__main__":
    main()
