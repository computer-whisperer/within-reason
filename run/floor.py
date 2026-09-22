#!/usr/bin/env python3
"""The fundamentals scorecard of a pianist or policy match (the user, 2026-09-22: "floor skill: sloppy execution,
missed evidence"): what the hands and the player did badly regardless of the result, from the record, the Jev log,
the transcript and the truth file. One row per match, the same columns every game, so a harness change is judged by
the scorecard over the recorded games rather than by wins.

usage: run/floor.py run/matches/<batch>/<NN> [...] [--ledger] [--json]
       run/floor.py --batch run/matches/<batch> [...]        one row per batch: median (min-max) of each column
       run/floor.py --arms run/matches/<A> run/matches/<B>    the two batches side by side, A then B
  --ledger prints one markdown row per match for docs/experiments.md; --json prints the numbers.

Columns (lower is better unless said):
  idle%      builder-seconds idle (commander and constructors, not being built) over their seconds alive
  e0%        seconds with the energy store under 2% of storage, over the game
  mfull%     seconds with the metal store over 98% of storage (income wasted)
  react s    median seconds from a party first seen at one of our extractors to the first order against it
  unanswered raider episodes with no order against the party within 60 s
  never      orders contradicting a "never splits" / "never advances to shelling" clause of the packet in force
  noop%      group asks answering hold on a group already holding (the hands' wasted asks)
  illegal    policy orders refused (option not offered, place unknown, actor on a list)
  stuck s    unit-seconds our mobile units could not move: from a move failure until the unit has moved 80 elmos
  yard min   minutes some factory of ours had a stuck unit in its exit lane (- for records without footprints)
  known%     the player's enemy-army metal as a share of the truth, median over its turns past minute five (higher is better)
  fac min    minute the picture first listed a factory of theirs seen (- if never)
  look min   minute one of our units first stood within 1,500 of the enemy commander's true start, from the truth file (- if never)
  turn s     the player's wall seconds a turn, median
"""
import json
import os
import re
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_audit import Match, clock  # noqa: E402

NEVER_SPLIT = re.compile(r"never (splits|sends? (a )?detachment|sends? detachments)|forbid(s|ding)? detachments|no detachments|never split", re.I)
NEVER_SHELLING = re.compile(r"never (advances|walks|goes)[^.]*shelling|shelling[^.]*is (banned|forbidden)|never[^.]*called shelling", re.I)


def med(xs, default=None):
    return statistics.median(xs) if xs else default


def scorecard(m):
    frames = m.header["frames_per_second"]
    sample = m.header.get("sample_frames", 30)
    defs = m.defs
    builders = {i for i, d in defs.items() if d.get("build_speed", 0) > 0 and d.get("class") != "building"}
    idle = alive = 0
    e0 = mfull = total = 0
    look = None
    positions = {}
    first_truth = min(m.truth) if m.truth else None
    enemy_start = next(((u[2], u[3]) for u in m.truth[first_truth] if u[1].endswith("com")), None) if first_truth is not None else None
    for line in open(next(os.path.join(m.dir, f) for f in os.listdir(m.dir) if f.startswith("record-"))):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        if r.get("t") != "s":
            continue
        total += 1
        e = r.get("e") or [0, 0, 0, 1]
        mm = r.get("m") or [0, 0, 0, 1]
        if e[3] and e[0] < 0.02 * e[3]:
            e0 += 1
        if mm[3] and mm[0] > 0.98 * mm[3]:
            mfull += 1
        for u in r["own"]:
            positions.setdefault(u[0], []).append((r["f"], u[2], u[3]))
            if u[1] in builders and not (u[5] & 1):
                alive += 1
                if u[5] & 2:
                    idle += 1
            if look is None and enemy_start and defs.get(u[1], {}).get("class") != "building" and ((u[2] - enemy_start[0]) ** 2 + (u[3] - enemy_start[1]) ** 2) ** 0.5 < 1500:
                look = r["f"]
    # raider episodes: a party first named at one of our extractors, and the first order against it
    episodes = {}
    answered = {}
    holds = group_asks = 0
    never = 0
    illegal = 0
    packet = ""
    fac = None
    tasks = {}
    for c in m.calls:
        if "instructions" in c:
            packet = c["instructions"]
        state = c.get("state") or {}
        if fac is None and isinstance((state.get("enemy") or {}).get("factories_seen"), list):
            fac = c["f"]
        for g in c.get("groups") or []:
            tasks[f"group_{g['name']}"] = (g.get("task") or {}).get("kind")
        for entry in (state.get("actors") or {}).values():
            for text in entry.get("enemies_at_our_extractors") or []:
                mm = re.match(r"(party_\d+)", text)
                if mm:
                    episodes.setdefault(mm.group(1), c["f"])
        for p in c.get("played") or []:
            did = p.get("did") or ""
            mm = re.search(r"(?:against|attack) (party_\d+)", did)
            if mm and mm.group(1) in episodes and mm.group(1) not in answered:
                answered[mm.group(1)] = c["f"]
            if p["kind"] == "group":
                group_asks += 1
                if p["played"] == "hold" and tasks.get(p["actor"]) == "hold":
                    holds += 1
            if packet and NEVER_SPLIT.search(packet) and p["played"] in ("send_against", "split"):
                never += 1
            if packet and NEVER_SHELLING.search(packet) and p["played"] in ("fight_to", "move_to") and "shelling" in did:
                never += 1
    # policy lines (the runtime's own log)
    for line in open(next(os.path.join(m.dir, f) for f in os.listdir(m.dir) if f.startswith("jev-"))):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        if r.get("t") == "policy":
            illegal += len(r.get("illegal") or [])
    delays = [(answered[k] - episodes[k]) / frames for k in answered]
    unanswered = sum(1 for k in episodes if k not in answered or answered[k] - episodes[k] > 60 * frames)
    stuck_frames, yard_min = stuck_and_yards(m, defs, positions, frames, sample)
    known = []
    for t in m.turns:
        if t["frame"] < 5 * 60 * frames:
            continue
        mk = re.search(r"not seen to die: (\d+) worth (\d+) metal", t["prompt"])
        tf = max((k for k in m.truth if k <= t["frame"]), default=None)
        if mk and tf is not None:
            true = sum(m.metal_of(u[1]) for u in m.truth[tf] if m.is_soldier_name(u[1]) and not u[5])
            if true > 0:
                known.append(100 * int(mk.group(2)) / true)
    walls = [t["wall"] or 0 for t in m.turns]
    r = m.result or {}
    return {
        "match": m.label, "result": r.get("outcome"), "minutes": round(r.get("game_minutes", 0), 1),
        "idle%": round(100 * idle / alive, 1) if alive else None,
        "e0%": round(100 * e0 / total, 1) if total else None,
        "mfull%": round(100 * mfull / total, 1) if total else None,
        "react_s": round(med(delays, 0), 0) if delays else None,
        "unanswered": unanswered, "episodes": len(episodes),
        "never": never,
        "noop%": round(100 * holds / group_asks, 0) if group_asks else None,
        "illegal": illegal,
        "stuck_s": round(stuck_frames / frames),
        "yard_min": yard_min,
        "known%": round(med(known, 0)) if known else None,
        "fac_min": round(fac / frames / 60, 1) if fac else None,
        "look_min": round(look / frames / 60, 1) if look else None,
        "turn_s": round(med(walls, 0), 1) if walls else None,
    }


COLUMNS = ["result", "minutes", "idle%", "e0%", "mfull%", "react_s", "unanswered", "never", "noop%", "illegal", "stuck_s", "yard_min", "known%", "fac_min", "look_min", "turn_s"]


STUCK_FREE = 80.0
LANE_DEPTH = 320.0
LANE_MARGIN = 48.0
SQUARE = 8.0


def stuck_and_yards(m, defs, positions, frames, sample):
    """Unit-frames our mobile units were stuck (from a move failure until they moved STUCK_FREE), and the minutes some
    factory had a stuck unit in its exit lane, as `brain/yards.rs` draws it (None without footprints in the header)."""
    spans = {}  # unit -> [(start frame, end frame)]
    for e in m.events:
        if e["k"] != "move_failed" or defs.get(e["d"], {}).get("speed", 0) <= 0:
            continue
        pos = positions.get(e["u"], [])
        if spans.get(e["u"]) and spans[e["u"]][-1][1] >= e["f"]:
            continue
        i0 = next((i for i, p in enumerate(pos) if p[0] >= e["f"]), None)
        if i0 is None:
            continue
        end = pos[-1][0]
        for f, x, z in pos[i0:]:
            if ((x - pos[i0][1]) ** 2 + (z - pos[i0][2]) ** 2) ** 0.5 > STUCK_FREE:
                end = f
                break
        spans.setdefault(e["u"], []).append((e["f"], end))
    stuck_frames = sum(end - start for spans_of in spans.values() for start, end in spans_of)
    if not any("footprint" in d for d in defs.values()):
        return stuck_frames, None
    lanes = []
    for e in m.events:
        d = defs.get(e["d"], {})
        if e["k"] != "finished" or not d.get("builds") or d.get("speed", 0) > 0 or "facing" not in e:
            continue
        xs, zs = d.get("footprint", [0, 0])
        fx, fz = [(0, 1), (1, 0), (0, -1), (-1, 0)][e["facing"] % 4]
        reach = zs * SQUARE / 2 + LANE_DEPTH
        lanes.append((e["u"], (e["x"], e["z"]), (e["x"] + fx * reach, e["z"] + fz * reach), xs * SQUARE / 2 + LANE_MARGIN))
    gone = {e["u"]: e["f"] for e in m.events if e["k"] == "destroyed"}

    def in_lane(lane, x, z):
        (fx, fz), (tx, tz), half = lane[1], lane[2], lane[3]
        dx, dz = tx - fx, tz - fz
        t = max(0.0, min(1.0, ((x - fx) * dx + (z - fz) * dz) / max(dx * dx + dz * dz, 1e-6)))
        return ((x - fx - t * dx) ** 2 + (z - fz - t * dz) ** 2) ** 0.5 <= half

    blocked = set()
    for unit, spans_of in spans.items():
        for f, x, z in positions.get(unit, []):
            if not any(start <= f <= end for start, end in spans_of):
                continue
            if any(gone.get(lane[0], float("inf")) > f and in_lane(lane, x, z) for lane in lanes):
                blocked.add(f)
    return stuck_frames, round(len(blocked) * sample / frames / 60, 1)


def batch_rows(batch):
    """The scorecards of every finished match in a batch directory."""
    rows = []
    for name in sorted(os.listdir(batch)):
        d = os.path.join(batch, name)
        if os.path.isdir(d) and any(f.startswith("record-") for f in os.listdir(d)):
            try:
                rows.append(scorecard(Match(d)))
            except (StopIteration, KeyError, ValueError) as e:
                print(f"{d}: skipped ({e})", file=sys.stderr)
    return rows


def batch_summary(batch):
    rows = batch_rows(batch)
    wins = sum(1 for r in rows if r["result"] == "Win")
    out = {"batch": os.path.basename(batch), "games": len(rows), "wins": wins}
    for c in COLUMNS[1:]:
        xs = [r[c] for r in rows if r[c] is not None]
        out[c] = f"{med(xs):.1f} ({min(xs):.0f}-{max(xs):.0f})" if xs else "-"
    return out


def print_batches(batches):
    rows = [batch_summary(b) for b in batches]
    width = max(len(r["batch"]) for r in rows)
    print(f"{'batch':{width}s} {'games':>6s} {'wins':>5s} " + " ".join(f"{c:>16s}" for c in COLUMNS[1:]))
    for r in rows:
        print(f"{r['batch']:{width}s} {r['games']:>6d} {r['wins']:>5d} " + " ".join(f"{r[c]:>16s}" for c in COLUMNS[1:]))


def main():
    args = sys.argv[1:]
    ledger = "--ledger" in args
    as_json = "--json" in args
    dirs = [a for a in args if not a.startswith("--")]
    if not dirs:
        sys.exit(__doc__)
    if "--batch" in args or "--arms" in args:
        print_batches(dirs)
        return
    rows = [scorecard(Match(d)) for d in dirs]
    if as_json:
        print(json.dumps(rows, indent=1))
        return
    if ledger:
        for r in rows:
            print(f"| {r['match']} | " + " | ".join(f"{c} {r[c] if r[c] is not None else '-'}" for c in COLUMNS[2:]) + " |")
        return
    width = max(len(r["match"]) for r in rows)
    print(f"{'match':{width}s} " + " ".join(f"{c:>10s}" for c in COLUMNS))
    for r in rows:
        print(f"{r['match']:{width}s} " + " ".join(f"{str(r[c]) if r[c] is not None else '-':>10s}" for c in COLUMNS))


if __name__ == "__main__":
    main()
