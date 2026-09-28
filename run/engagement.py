#!/usr/bin/env python3
"""Cut a duel scenario from a live match record: the units of an engagement as they stood at one game clock, so the
engine (`target/release/duel --scenario FILE`) and the simulator can fight it again from one file.

    run/engagement.py <match dir or record> <MM:SS> [--radius 900] [--at X,Z] [--enemies sight|known|truth]
                      [--orders 20] [--after 30,60] [--out FILE]

Reads the record's once-a-second `s` line at the clock (`docs/harness/record-format.md`): our units (`own`, health in
percent) and the enemies (`en`). Only armed units are taken (a type with a `reach`), never a commander (its death ends
the game) and never a unit still being built; buildings are taken like anything else (a turret is armed).

Enemies, by `--enemies`:
  sight  (default) those in sight at the clock (`en` with a health), with their absolute hit points;
  known  also radar contacts and remembered units whose type the record knows, at full health (unknown);
  truth  the opponent's ground truth, `truth-<ai>.jsonl` (written when the match ran with WITHIN_REASON_OBSERVE=1),
         the line at or before the clock, with health in percent (-1 there, rarely: full health): every enemy unit,
         seen or not.

The place: `--at X,Z`, else the contact: halfway between the centroid of our units that have an enemy (as selected
above) within 800 and the centroid of the enemies that have one of ours within 800, so neither side's numbers pull it
off the front. Everything within `--radius` of it is taken.

Heading: a unit that moved 8 elmos or more since the sample a second earlier (two for truth lines) is given that
direction. First orders, ours only (the enemy's are not in the record): the last `move`, `fight` or `attack` the bot
gave the unit in the `--orders` seconds up to the clock (a `stop` clears it); an attack on an enemy outside the cut
becomes a fight at where that enemy was. Units without an order get the duel's default (fight at the other side).

How it went live (`--after`, seconds, default 30 and 60): each side's value left that long after the clock, scored as
the duel harness scores a fight (metal x health at the end over metal x health at the start, over the cut's units): ours
from the record's `s` lines, theirs from the truth file (whatever `--enemies` says; nothing when the match has none).
It goes into the file's `source.live` and to stderr, to set beside the engine's and the simulator's outcomes; it is
not a prediction of them (the live units kept their orders, reinforcements came, the rest of the game went on).

The other side's track (`--track`, 2026-09-28): the positions of the cut's enemies over the `--track-seconds` (60)
after the clock, as `sides[1].track` rows `{"t", "unit", "x", "z"}` (`t` seconds from the clock, `unit` the enemy's
index in the file; rows from the sample just before the clock on, so every unit has a place at 0): from the truth
file (every two seconds) when `--enemies truth`, else from the record's `en` rows (once a second, only while in sight
or on radar). `duel --theirs track` walks them; the sight version is what a script's author may plan from, and so
it also lists, as `sides[1].later`, the armed enemies first seen within `--radius` of the place after the clock (type,
first second, place, hit points), whose track rows carry indices after the cut's units (the duel spawns none of them).

The record's own orders as a script (`--script FILE`, 2026-09-28): every move, fight, attack, stop and guard the bot
gave a unit of the cut in the `--track-seconds` after the clock, as `duel --script` rows (`crates/arena/src/bin/duel/
scenario.rs`, `Script`): an attack on an enemy outside the cut becomes a fight at where it was last seen, an order
repeating the unit's last one is dropped (the lane re-sends its focus every tick), and a unit's second and later
orders of one tick are queued (the record keeps no queue flag; the bot queues within a tick, never across). A move or
fight to a point further than `FIELD` from the place is pulled in to `FIELD` along its line: a retreat home would walk
into the duel's commanders, which stand in the corners farthest from the place (smoke-e3: the record's retreat
killed one and ended the match).

A unit of ours whose health reads -1 at the clock died that second and is left out.

The file format is `crates/arena/src/bin/duel/scenario.rs`'s doc comment.
"""
import argparse, json, math, os, sys

CONTACT = 800.0
MOVED = 8.0
FIELD = 1500.0


def record_of(path):
    if os.path.isfile(path):
        return path
    for cand in (os.path.join(path, "record-0.jsonl"), os.path.join(path, "00", "record-0.jsonl")):
        if os.path.isfile(cand):
            return cand
    sys.exit(f"no record in {path}")


def frame_of(clock):
    minutes, seconds = clock.split(":")
    return (int(minutes) * 60 + int(seconds)) * 30


def samples(path, frame):
    """The `s` lines a second before and at (the first at or after) `frame`, and the `cmd` lines before it."""
    before, at, commands = None, None, []
    with open(path) as f:
        header = json.loads(f.readline())
        for line in f:
            if line.startswith('{"c":') or '"t":"cmd"' in line[-40:]:
                try:
                    r = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if r.get("t") == "cmd" and r["f"] <= frame:
                    commands.append(r)
                continue
            if '"t":"s"' not in line[:40]:
                continue
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                break
            if r["f"] < frame:
                before = r
            else:
                at = r
                break
    if at is None:
        sys.exit(f"the record ends before frame {frame}")
    return header, before, at, commands


def truth_lines(path, frame):
    """The truth line at or before `frame` and the one before it."""
    previous, current = None, None
    with open(path) as f:
        for line in f:
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                break
            if r["f"] > frame:
                break
            previous, current = current, r
    if current is None:
        sys.exit(f"{path} has nothing at or before frame {frame}")
    return previous, current


def heading(now, then):
    if then is None:
        return None
    dx, dz = now[0] - then[0], now[1] - then[1]
    length = math.hypot(dx, dz)
    return [round(dx / length, 3), round(dz / length, 3)] if length >= MOVED else None


def live_outcome(path, header, frame, ours, theirs, after):
    """Value left of each side's cut units `after` seconds past `frame`, as the duel harness scores a fight."""
    metal = {d["name"]: d["metal"] for d in header["unit_defs"]}
    start = lambda units: sum(metal[u["type"]] * u.get("health", 1.0) for u in units)
    ends = sorted(frame + int(s * 30) for s in after)
    own_at, truth_at = {}, {}
    with open(path) as f:
        f.readline()
        pending = list(ends)
        for line in f:
            if not pending:
                break
            if '"t":"s"' not in line[:40]:
                continue
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                break
            while pending and r["f"] >= pending[0]:
                own_at[pending.pop(0)] = {u: hp / 100 for u, d, x, z, hp, fl in r["own"]}
    truth = os.path.join(os.path.dirname(path), f"truth-{header['ai_id']}.jsonl")
    if os.path.isfile(truth):
        with open(truth) as f:
            pending = list(ends)
            for line in f:
                if not pending:
                    break
                try:
                    r = json.loads(line)
                except json.JSONDecodeError:
                    break
                while pending and r["f"] >= pending[0]:
                    truth_at[pending.pop(0)] = {e[0]: (e[4] / 100 if e[4] >= 0 else 1.0) for e in r["enemy"]}
    rows = []
    for end in ends:
        if end not in own_at:
            continue
        left = lambda units, now: sum(metal[u["type"]] * now.get(u["id"], 0.0) for u in units)
        row = {"seconds": (end - frame) / 30, "ours": round(left(ours, own_at[end]) / max(1.0, start(ours)), 3), "theirs": None}
        if end in truth_at and theirs:
            row["theirs"] = round(left(theirs, truth_at[end]) / max(1.0, start(theirs)), 3)
        rows.append(row)
    return rows


def enemy_track(path, header, frame, theirs, source, seconds, centre=None, radius=None):
    """The cut's enemies' places from just before `frame` to `seconds` after it: truth lines, or the record's `en`.
    From the record, armed enemies first seen inside `radius` of `centre` after the clock are added (`later`)."""
    index = {e["id"]: i for i, e in enumerate(theirs)}
    defs = header["unit_defs"]
    later = []
    end = frame + int(seconds * 30)
    lines = []
    if source == "truth":
        with open(os.path.join(os.path.dirname(path), f"truth-{header['ai_id']}.jsonl")) as f:
            for line in f:
                try:
                    r = json.loads(line)
                except json.JSONDecodeError:
                    break
                if r["f"] > end:
                    break
                lines.append((r["f"], [(e[0], e[2], e[3]) for e in r["enemy"]]))
    else:
        with open(path) as f:
            f.readline()
            for line in f:
                if '"t":"s"' not in line[:40]:
                    continue
                try:
                    r = json.loads(line)
                except json.JSONDecodeError:
                    break
                if r["f"] > end:
                    break
                lines.append((r["f"], [(e[0], e[2], e[3]) for e in r["en"]]))
                if centre is None or r["f"] <= frame:
                    continue
                for e, d, x, z, hp, *_ in r["en"]:
                    if e in index or d < 0 or not (defs[d].get("reach", 0) > 0 and defs[d]["class"] != "commander"):
                        continue
                    if math.hypot(x - centre[0], z - centre[1]) <= radius:
                        index[e] = len(theirs) + len(later)
                        later.append({"type": defs[d]["name"], "t": round((r["f"] - frame) / 30, 2), "x": x, "z": z,
                                      **({"hp": hp} if hp >= 0 else {}), "source_id": e})
    # From the last line at or before the clock on.
    start = max((i for i, (f, _) in enumerate(lines) if f <= frame), default=0)
    rows = []
    for f, units in lines[start:]:
        for e, x, z in units:
            if e in index:
                rows.append({"t": round((f - frame) / 30, 2), "unit": index[e], "x": x, "z": z})
    return rows, later


def within_field(x, z, centre):
    dx, dz = x - centre[0], z - centre[1]
    d = math.hypot(dx, dz)
    if d <= FIELD:
        return x, z
    return round(centre[0] + dx / d * FIELD), round(centre[1] + dz / d * FIELD)


def recorded_script(path, frame, ours, theirs, last_seen, seconds, centre):
    """Our cut units' recorded orders in the `seconds` after `frame`, as duel script rows."""
    ours_index = {u["id"]: i for i, u in enumerate(ours)}
    theirs_index = {e["id"]: i for i, e in enumerate(theirs)}
    end = frame + int(seconds * 30)
    seen = dict(last_seen)
    rows, last = [], {}
    with open(path) as f:
        f.readline()
        for line in f:
            if '"t":"s"' in line[:40]:
                try:
                    r = json.loads(line)
                except json.JSONDecodeError:
                    break
                if r["f"] > end:
                    break
                if r["f"] >= frame:
                    seen.update({e: (x, z) for e, d, x, z, *_ in r["en"]})
                continue
            if not (line.startswith('{"c":') or '"t":"cmd"' in line[-40:]):
                continue
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                break
            if r.get("t") != "cmd" or r["f"] < frame:
                continue
            if r["f"] > end:
                break
            t = round((r["f"] - frame) / 30, 2)
            this_tick = set()
            for c in r["c"]:
                if len(c) < 2 or c[1] not in ours_index:
                    continue
                unit = ours_index[c[1]]
                row = None
                if c[0] in ("move", "fight"):
                    x, z = within_field(c[2], c[3], centre)
                    row = {"cmd": c[0], "x": x, "z": z}
                elif c[0] == "attack":
                    if c[2] in theirs_index:
                        row = {"cmd": "attack", "target": theirs_index[c[2]]}
                    elif c[2] in seen:
                        row = {"cmd": "fight", "x": seen[c[2]][0], "z": seen[c[2]][1]}
                elif c[0] == "stop":
                    row = {"cmd": "stop"}
                elif c[0] == "guard" and c[2] in ours_index:
                    row = {"cmd": "guard", "target": ours_index[c[2]]}
                if row is None:
                    continue
                queued = unit in this_tick
                if not queued and last.get(unit) == row:
                    continue
                this_tick.add(unit)
                last[unit] = row if not queued else {"queued": True}
                rows.append({"t": t, "unit": unit} | row | ({"queue": True} if queued else {}))
    return rows


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("record")
    ap.add_argument("clock")
    ap.add_argument("--radius", type=float, default=900.0)
    ap.add_argument("--at")
    ap.add_argument("--enemies", choices=("sight", "known", "truth"), default="sight")
    ap.add_argument("--orders", type=float, default=20.0, help="seconds before the clock in which a command still counts")
    ap.add_argument("--after", default="30,60", help="seconds after the clock at which to score the live outcome")
    ap.add_argument("--out")
    ap.add_argument("--track", action="store_true", help="write the enemies' positions after the clock as sides[1].track")
    ap.add_argument("--track-seconds", type=float, default=60.0, help="how long after the clock the track and the script run")
    ap.add_argument("--script", help="write our units' recorded orders after the clock as a duel script to this file")
    a = ap.parse_args()

    path = record_of(a.record)
    frame = frame_of(a.clock)
    header, before, at, commands = samples(path, frame)
    defs = header["unit_defs"]
    by_name = {d["name"]: d for d in defs}

    def armed(d):
        return d is not None and d.get("reach", 0) > 0 and d["class"] != "commander"

    earlier = {u: (x, z) for u, d, x, z, *_ in (before or {"own": []})["own"]}
    ours = []
    for u, d, x, z, hp, flags in at["own"]:
        if armed(defs[d]) and not flags & 1 and hp >= 0:
            ours.append({"id": u, "type": defs[d]["name"], "x": x, "z": z, "health": round(hp / 100, 3), "heading": heading((x, z), earlier.get(u))})

    theirs = []
    if a.enemies == "truth":
        truth = os.path.join(os.path.dirname(path), f"truth-{header['ai_id']}.jsonl")
        if not os.path.isfile(truth):
            sys.exit(f"no {truth}: the match ran without WITHIN_REASON_OBSERVE")
        previous, current = truth_lines(truth, frame)
        was = {e[0]: (e[2], e[3]) for e in (previous or {"enemy": []})["enemy"]}
        for e, name, x, z, hp, built in current["enemy"]:
            if not built and armed(by_name.get(name)):
                # -1: the truth line had no health for it; full health then.
                health = round(hp / 100, 3) if hp >= 0 else None
                theirs.append({"id": e, "type": name, "x": x, "z": z, "health": health, "heading": heading((x, z), was.get(e))})
        truth_frame = current["f"]
    else:
        was = {e: (x, z) for e, d, x, z, hp, *_ in (before or {"en": []})["en"]}  # five fields before 2026-09-28, six (being built) after
        for e, d, x, z, hp, *_ in at["en"]:
            if d < 0 or not armed(defs[d]) or (a.enemies == "sight" and hp < 0):
                continue
            unit = {"id": e, "type": defs[d]["name"], "x": x, "z": z, "heading": heading((x, z), was.get(e))}
            if hp >= 0:
                unit["hp"] = hp
            theirs.append(unit)
        truth_frame = None

    if a.at:
        cx, cz = (float(v) for v in a.at.split(","))
    else:
        engaged_ours = [u for u in ours if any(math.hypot(u["x"] - e["x"], u["z"] - e["z"]) < CONTACT for e in theirs)]
        engaged_theirs = [e for e in theirs if any(math.hypot(u["x"] - e["x"], u["z"] - e["z"]) < CONTACT for u in ours)]
        if not engaged_ours:
            sys.exit("no unit of either side has one of the other within 800 at this clock: give --at X,Z (or --enemies truth)")
        mean = lambda units: (sum(u["x"] for u in units) / len(units), sum(u["z"] for u in units) / len(units))
        (ax, az), (bx, bz) = mean(engaged_ours), mean(engaged_theirs)
        cx, cz = (ax + bx) / 2, (az + bz) / 2
    inside = lambda u: math.hypot(u["x"] - cx, u["z"] - cz) <= a.radius
    ours, theirs = [u for u in ours if inside(u)], [u for u in theirs if inside(u)]

    # Our first orders: the last move / fight / attack / stop per unit in the window.
    index_of = {e["id"]: i for i, e in enumerate(theirs)}
    last_seen = {e: (x, z) for e, d, x, z, hp, *_ in at["en"]}
    wanted = {u["id"] for u in ours}
    orders = {}
    for r in commands:
        if r["f"] < frame - a.orders * 30:
            continue
        for c in r["c"]:
            if len(c) < 2 or c[1] not in wanted:
                continue
            if c[0] in ("move", "fight"):
                orders[c[1]] = {"kind": c[0], "x": c[2], "z": c[3]}
            elif c[0] == "attack":
                if c[2] in index_of:
                    orders[c[1]] = {"kind": "attack", "target": index_of[c[2]]}
                elif c[2] in last_seen:
                    x, z = last_seen[c[2]]
                    orders[c[1]] = {"kind": "fight", "x": x, "z": z}
            elif c[0] == "stop":
                orders.pop(c[1], None)
    for u in ours:
        if u["id"] in orders:
            u["order"] = orders[u["id"]]

    def clean(units):
        out = []
        for u in units:
            u = {k: v for k, v in u.items() if v is not None and k != "id"} | {"source_id": u["id"]}
            out.append(u)
        return out

    live = live_outcome(path, header, frame, ours, theirs, [float(x) for x in a.after.split(",") if x])
    track, later = enemy_track(path, header, frame, theirs, a.enemies, a.track_seconds, (cx, cz), a.radius) if a.track else (None, [])
    if a.script:
        rows = recorded_script(path, frame, ours, theirs, last_seen, a.track_seconds, (cx, cz))
        with open(a.script, "w") as f:
            json.dump({"format": "within-reason-script", "version": 1, "source": {"record": os.path.abspath(path), "clock": a.clock, "frame": at["f"]},
                       "rows": rows}, f, indent=None, separators=(",", ":"))
            f.write("\n")
        print(f"  script: {len(rows)} rows over {a.track_seconds:.0f} s -> {a.script}", file=sys.stderr)

    scenario = {
        "format": "within-reason-scenario",
        "version": 1,
        "map": header["map"]["name"],
        "map_size": [header["map"]["width"], header["map"]["height"]],
        "centre": [round(cx), round(cz)],
        "source": {
            "record": os.path.abspath(path), "clock": a.clock, "frame": at["f"], "radius": a.radius,
            "enemies": a.enemies, "truth_frame": truth_frame, "orders_window": a.orders, "live": live,
        },
        "sides": [{"name": "ours", "units": clean(ours)}, {"name": "theirs", "units": clean(theirs)}],
    }
    if track is not None:
        scenario["sides"][1]["track"] = track
    if later:
        scenario["sides"][1]["later"] = later
    text = json.dumps(scenario, indent=1)
    if a.out:
        with open(a.out, "w") as f:
            f.write(text + "\n")
    else:
        print(text)
    summary = lambda units: ", ".join(f"{n} {t}" for t, n in sorted(((t, sum(1 for u in units if u["type"] == t)) for t in {u["type"] for u in units}), key=lambda p: -p[1]))
    print(f"{a.clock} at ({cx:.0f}, {cz:.0f}) r {a.radius:.0f}: ours {len(ours)} ({summary(ours)}; {len(orders)} with orders), "
          f"theirs {len(theirs)} ({summary(theirs)}) [{a.enemies}]", file=sys.stderr)
    for row in live:
        print(f"  live, {row['seconds']:.0f} s later: value left ours {row['ours']:.3f}, theirs " + ("-" if row["theirs"] is None else f"{row['theirs']:.3f}"), file=sys.stderr)


if __name__ == "__main__":
    main()
