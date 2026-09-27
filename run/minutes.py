#!/usr/bin/env python3
"""The game minute by minute, for the reviewer who has to find the cause behind the symptom: where each army's mass
stood, what share of ours was in his half, what of his we could see, what the plants made, what the player ordered,
what died where, and what he started building (from the truth file). Then the moments a reviewer must not miss: every
window (a minute at 1.5x his army or more) and how it was spent, his tier-2 factory and the minutes after it, when his
base entered the picture, and the fate of every unit of ours that went to his side early.

usage:
  run/minutes.py MATCH                 the table for the whole game and the sections below it
  run/minutes.py MATCH --to 15         the table to minute 15 (the sections always cover the whole game)
  run/minutes.py MATCH --early 10      the "units sent to his side" section covers units that crossed before minute 10 (default 12)

MATCH is `run/matches/<batch>/<NN>` (or the batch directory: its `00`). Reads the record, the truth file and the
player's transcript; never the Jev log. The opponent's numbers come from the truth file where there is one.
"""
import argparse
import json
import math
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from analyze_match import FPS, Match, clock, summarise  # noqa: E402
from commander_turns import calls_in  # noqa: E402

TIER2_FACTORIES = {"armalab", "armavp", "armaap", "armasy", "coralab", "coravp", "coraap", "corasy", "legalab", "legavp", "legaap"}
HIS_SIDE = ("in their half", "at their base")
OUR_SIDE = ("in our half", "at our base")


def turns_of(match_dir):
    """[(frame, [(tool, args)])] from the player's transcript, one per turn."""
    path = next((os.path.join(match_dir, f) for f in sorted(os.listdir(match_dir)) if f.startswith("strategist-") and f.endswith(".jsonl")), None)
    out, turn = [], None
    if not path:
        return out
    for line in open(path, errors="replace"):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        kind = r.get("kind")
        if kind == "turn":
            turn = (int(r["frame"]), [])
            out.append(turn)
        elif kind == "tool_call" and turn is not None:
            turn[1].extend(calls_in(r))
    return out


def soldiers(units, match):
    return [u for u in units if match.cls(u[1]) in ("army", "aircraft", "hover", "ship") and not ((u[5] & 1) if isinstance(u[5], int) else u[5])]


def mass(units, match, on):
    """(cell with most metal, metal, share of metal on the side `on`)."""
    by_cell, total, there = defaultdict(float), 0.0, 0.0
    for u in units:
        m = match.metal(u[1])
        by_cell[match.grid(u[2], u[3])] += m
        total += m
        if match.side_of_map(u[2], u[3]) in on:
            there += m
    if not total:
        return "-", 0, 0
    cell = max(by_cell, key=by_cell.get)
    return cell, total, there / total


def tool_summary(calls):
    counts = defaultdict(int)
    for tool, _ in calls:
        if tool != "wait":
            counts[tool] += 1
    return " ".join(f"{t}:{n}" if n > 1 else t for t, n in sorted(counts.items()))


def table(match, turns, to):
    last = match.samples[-1]["f"] if match.samples else 0
    end = min(to, last // (60 * FPS)) if to else last // (60 * FPS)
    seen_builds = defaultdict(list)
    for e in match.events:
        if e["k"] == "enemy_seen" and e.get("d", -1) >= 0 and match.cls(match.defs[e["d"]]["name"]) in ("factory", "turret", "extractor", "building"):
            seen_builds[e["f"] // (60 * FPS) + 1].append(match.defs[e["d"]]["name"])
    made = defaultdict(lambda: defaultdict(int))
    for e in match.events:
        if e["k"] == "created" and e.get("d", -1) >= 0:
            c = match.cls(match.defs[e["d"]]["name"])
            if c in ("army", "builder", "aircraft", "hover", "ship", "air_builder"):
                made[e["f"] // (60 * FPS) + 1]["soldiers" if c != "builder" and c != "air_builder" else "builders"] += 1
    lost = defaultdict(lambda: [0.0, defaultdict(float), 0.0])
    for e in match.events:
        if e.get("d", -1) < 0:
            continue
        minute = e["f"] // (60 * FPS) + 1
        if e["k"] == "destroyed" and e.get("by") is not None:
            m = match.metal(match.defs[e["d"]]["name"])
            lost[minute][0] += m
            lost[minute][1][match.grid(e["x"], e["z"])] += m
        elif e["k"] == "enemy_destroyed":
            lost[minute][2] += match.metal(match.defs[e["d"]]["name"])
    started = his_starts(match)
    by_min_turns = defaultdict(list)
    for f, calls in turns:
        by_min_turns[f // (60 * FPS) + 1].append(calls)
    print("min | army ours/his (x) | our mass (his half %) | his mass (our half %) | his in sight | made (b builders, s soldiers) | player turns | lost ours (where) / his seen | his buildings started | his buildings first seen")
    for minute in range(1, end + 1):
        f = minute * 60 * FPS
        ours, theirs = match.ours_at(f), match.theirs_at(f)
        os_, ts = soldiers(ours, match), soldiers(theirs, match)
        oc, om, oh = mass(os_, match, HIS_SIDE)
        tc, tm, th = mass(ts, match, OUR_SIDE)
        ratio = f"{om / tm:.1f}" if tm else "-"
        sample = min(match.samples, key=lambda s: abs(s["f"] - f))
        seen = len(sample.get("en", []))
        m = made[minute]
        made_s = " ".join(f"{k[0]}{v}" for k, v in sorted(m.items())) or "-"
        t = by_min_turns[minute]
        turns_s = f"{len(t)} ({tool_summary([c for calls in t for c in calls])})" if t else "0"
        L = lost[minute]
        where = max(L[1], key=L[1].get) if L[1] else "-"
        starts = starts_text(match, started.get(minute, []))
        firsts = " ".join(sorted(set(seen_builds[minute]))) or "-"
        print(f"{minute:>3} | {om:>6.0f}/{tm:<6.0f} ({ratio:>4}) | {oc:>3} ({oh * 100:>3.0f}%) | {tc:>3} ({th * 100:>3.0f}%) | {seen:>3} | {made_s:<6} | {turns_s:<28} | {L[0]:>5.0f} ({where}) / {L[2]:>5.0f} | {starts} | {firsts}")


def starts_text(match, items):
    named = [f"{n}@{c}" for n, c, _ in items if match.cls(n) in ("factory", "turret")]
    rest = defaultdict(int)
    for n, _, _ in items:
        if match.cls(n) not in ("factory", "turret"):
            rest["extractors" if match.cls(n) == "extractor" else "other"] += 1
    return " ".join(named + [f"{v} {k}" for k, v in sorted(rest.items())]) or "-"


def his_starts(match):
    """{minute: [(name, cell, frame)]} for every building of his that first appears in the truth (started, built or not)."""
    out, known = defaultdict(list), set()
    for t in match.truth:
        for u in t["enemy"]:
            if u[0] in known:
                continue
            known.add(u[0])
            if match.cls(u[1]) in ("factory", "turret", "extractor", "building") and t["f"] > 0:
                out[t["f"] // (60 * FPS) + 1].append((u[1], match.grid(u[2], u[3]), t["f"]))
    return out


def windows(match):
    print("\nWINDOWS: every run of minutes at 1.5x his army or more, where our mass stood through it, and how it ended")
    last = match.samples[-1]["f"] // (60 * FPS)
    rows = {}
    for minute in range(1, last + 1):
        f = minute * 60 * FPS
        os_, ts = soldiers(match.ours_at(f), match), soldiers(match.theirs_at(f), match)
        oc, om, oh = mass(os_, match, HIS_SIDE)
        _, tm, _ = mass(ts, match, OUR_SIDE)
        rows[minute] = (om, tm, oc, oh, (om / tm) if tm else 0)
    fac_min = first_factory_seen(match)
    runs, run = [], []
    for minute in range(1, last + 1):
        if rows[minute][4] >= 1.5:
            run.append(minute)
        elif run:
            runs.append(run)
            run = []
    if run:
        runs.append(run)
    if not runs:
        print("  none")
        return
    for run in runs:
        ratios = [rows[m][4] for m in run]
        cells = " ".join(f"{rows[m][2]}{'*' if rows[m][3] > 0.3 else ''}" for m in run)
        peak = max(rows[m][3] for m in run)
        end = run[-1] + 1
        if end <= last:
            ended = f"ended at {end}:00 at {rows[end][4]:.1f}x ({rows[end][0]:.0f}/{rows[end][1]:.0f})"
        else:
            ended = "open at the game's end"
        base = "not yet" if fac_min is None or fac_min > run[0] else f"since {fac_min}:00"
        print(f"  {run[0]}:00-{run[-1]}:00 ({min(ratios):.1f}-{max(ratios):.1f}x, {len(run)} min): our mass by minute {cells} "
              f"(* = over 30% of it in his half; peak {peak * 100:.0f}%); his factory in the picture: {base}; {ended}")


def first_factory_seen(match):
    for e in match.events:
        if e["k"] == "enemy_seen" and e.get("d", -1) >= 0 and match.cls(match.defs[e["d"]]["name"]) == "factory":
            return e["f"] // (60 * FPS) + 1
    return None


def tier2(match):
    print("\nHIS TIER 2: the factory in the truth file, his army and ours in the minutes after, and where our mass was")
    first = None
    for t in match.truth:
        for u in t["enemy"]:
            if u[1] in TIER2_FACTORIES:
                first = (t["f"], u)
                break
        if first:
            break
    if not first:
        print("  none in the truth file")
        return
    f0, u = first
    print(f"  {u[1]} started at {clock(f0)} at {match.grid(u[2], u[3])} ({u[2]:.0f}, {u[3]:.0f})")
    stood = next((t["f"] for t in match.truth if any(v[0] == u[0] and not v[5] for v in t["enemy"])), None)
    print(f"  standing at {clock(stood) if stood is not None else '- (never finished)'}")
    first_unit = None
    if stood is not None:
        seen_ids = set()
        for t in match.truth:
            for v in t["enemy"]:
                seen_ids.add(v[0]) if t["f"] < stood else None
        for t in match.truth:
            if t["f"] < stood:
                continue
            for v in t["enemy"]:
                if v[0] not in seen_ids and match.cls(v[1]) not in ("factory", "turret", "extractor", "building", "commander") and math.dist((v[2], v[3]), (u[2], u[3])) < 400:
                    first_unit = (t["f"], v[1])
                    break
                seen_ids.add(v[0])
            if first_unit:
                break
    if first_unit:
        print(f"  its first unit, {first_unit[1]}, at {clock(first_unit[0])}")
    seen = next((e for e in match.events if e["k"] == "enemy_seen" and e.get("d", -1) >= 0 and match.defs[e["d"]]["name"] in TIER2_FACTORIES), None)
    print(f"  in our picture: {clock(seen['f']) if seen else 'never'}")
    m0 = f0 // (60 * FPS)
    for minute in range(max(1, m0 - 1), m0 + 6):
        f = minute * 60 * FPS
        if f > match.samples[-1]["f"]:
            break
        os_, ts = soldiers(match.ours_at(f), match), soldiers(match.theirs_at(f), match)
        oc, om, oh = mass(os_, match, HIS_SIDE)
        tc, tm, _ = mass(ts, match, OUR_SIDE)
        print(f"  {minute}:00 army {om:.0f}/{tm:.0f} ({om / tm:.1f}x): our mass at {oc}, {oh * 100:.0f}% in his half; his mass at {tc}" if tm else f"  {minute}:00 army {om:.0f}/0")


def base_seen(match):
    print("\nHIS BASE IN THE PICTURE")
    if not match.enemy_home:
        print("  no enemy start known")
        return
    fac = next((e for e in match.events if e["k"] == "enemy_seen" and e.get("d", -1) >= 0 and match.cls(match.defs[e["d"]]["name"]) == "factory"), None)
    print(f"  his first factory seen: {clock(fac['f']) + ' at ' + match.grid(fac['x'], fac['z']) if fac else 'never'}")
    look = None
    for s in match.samples:
        for u in s["own"]:
            if math.dist((u[2], u[3]), match.enemy_home) < 1500:
                look = (s["f"], match.defs[u[1]]["name"], u[0])
                break
        if look:
            break
    print(f"  first of ours within 1,500 of his start ({match.grid(*match.enemy_home)}): {clock(look[0]) + ' ' + look[1] + '_' + str(look[2]) if look else 'never'}")


def early_visitors(match, before):
    print(f"\nUNITS OF OURS SENT TO HIS SIDE BEFORE {before}:00: how far each got and what became of it")
    if not match.enemy_home or not match.home:
        print("  no starts known")
        return
    limit = before * 60 * FPS
    track = {}
    names = {}
    for s in match.samples:
        for u in s["own"]:
            if (u[5] & 1) if isinstance(u[5], int) else u[5]:
                continue
            name = match.defs[u[1]]["name"]
            if match.cls(name) in ("factory", "turret", "extractor", "building", "commander", "builder"):
                continue
            d = math.dist((u[2], u[3]), match.enemy_home)
            side = match.side_of_map(u[2], u[3])
            t = track.get(u[0])
            if t is None:
                if side in HIS_SIDE and s["f"] < limit:
                    track[u[0]] = {"crossed": s["f"], "nearest": (d, s["f"], match.grid(u[2], u[3])), "last": s["f"], "last_d": d, "back": None}
                    names[u[0]] = name
                continue
            if d < t["nearest"][0]:
                t["nearest"] = (d, s["f"], match.grid(u[2], u[3]))
            if t["back"] is None and side in OUR_SIDE and s["f"] > t["nearest"][1]:
                t["back"] = s["f"]
            t["last"], t["last_d"] = s["f"], d
    deaths = {e["u"]: e for e in match.events if e["k"] == "destroyed"}
    if not track:
        print("  none")
        return
    for uid, t in sorted(track.items(), key=lambda kv: kv[1]["crossed"]):
        d = deaths.get(uid)
        fate = []
        if t["back"]:
            fate.append(f"back in our half at {clock(t['back'])}")
        if d:
            killer = match.defs[d["by_d"]]["name"] if d.get("by_d") is not None and d["by_d"] >= 0 else "something unseen"
            fate.append(f"died {clock(d['f'])} at {match.grid(d['x'], d['z'])} to {killer}")
        if not fate:
            fate.append(f"last seen {clock(t['last'])}, {t['last_d']:.0f} from his start")
        fate = "; ".join(fate)
        print(f"  {names[uid]}_{uid}: crossed {clock(t['crossed'])}; nearest his start {t['nearest'][0]:.0f} at {t['nearest'][2]} {clock(t['nearest'][1])}; {fate}")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("match")
    ap.add_argument("--to", type=int, default=0)
    ap.add_argument("--early", type=int, default=12)
    a = ap.parse_args()
    d = a.match
    if not any(f.startswith("record-") for f in os.listdir(d)) and os.path.isdir(os.path.join(d, "00")):
        d = os.path.join(d, "00")
    match = Match(d)
    print(f"MATCH {d}: we start {match.grid(*match.home) if match.home else '?'}, he starts {match.grid(*match.enemy_home) if match.enemy_home else '?'}; "
          f"'his half' is nearer his start than ours; the opponent's numbers are {'the truth file' if match.truth else 'what we saw'}")
    table(match, turns_of(d), a.to)
    windows(match)
    tier2(match)
    base_seen(match)
    early_visitors(match, a.early)


if __name__ == "__main__":
    main()
