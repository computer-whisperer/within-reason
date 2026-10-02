#!/usr/bin/env python3
"""The fundamentals scorecard of a pianist match (the user, 2026-09-22: "floor skill: sloppy execution,
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
  react s    median seconds from a party first named at one of our extractors to the first order against it (0 when one was already in force)
  unanswered raider episodes with no order against the party from 60 s before to 60 s after
  noop%      quiet passes (every question in the words of its last ask, nothing sent) over the passes that had something to ask
  jev$       the Jev bill from the log's usage, the packet's decode included (input tokens at $0.042 a million)
  stuck s    unit-seconds our mobile units could not move: from a move failure until the unit has moved 80 elmos
  yard min   minutes some factory of ours had a stuck unit in its exit lane (- for records without footprints)
  known%     the player's enemy-army metal as a share of the truth, median over its turns past minute five (higher is better)
  fac min    minute the picture first listed a factory of theirs seen (- if never)
  look min   minute one of our units first stood within 1,500 of the enemy commander's true start, from the truth file (- if never)
  turn s     the player's wall seconds a turn, median
  aband      structures of ours started and then abandoned: an unfinished frame destroyed by nobody (it decayed
             after its builder left, or an unseen enemy finished it); aband m is the metal put into them (the frame's
             last sampled build share times its cost), the user 2026-09-27: "it eats a lot of resources whenever it happens"
"""
import json
import os
import re
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from match_read import Match, clock  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


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
    built_share = {}
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
            if u[5] & 1:
                built_share[u[0]] = u[4] / 100.0
            if u[1] in builders and not (u[5] & 1):
                alive += 1
                if u[5] & 2:
                    idle += 1
            if look is None and enemy_start and defs.get(u[1], {}).get("class") != "building" and ((u[2] - enemy_start[0]) ** 2 + (u[3] - enemy_start[1]) ** 2) ** 0.5 < 1500:
                look = r["f"]
    # raider episodes: a party first named at one of our extractors, and the first order against it
    episodes = {}
    answered = {}
    orders = {}
    fac = None
    for c in m.calls:
        state = c.get("state") or {}
        if fac is None and isinstance((state.get("enemy") or {}).get("factories_seen"), list):
            fac = c["f"]
        for entry in (state.get("actors") or {}).values():
            if not isinstance(entry, dict):
                continue  # an unasked actor is one line since 2026-09-23
            for text in entry.get("enemies_at_our_extractors") or []:
                mm = re.match(r"(party_\d+)", text)
                if mm:
                    episodes.setdefault(mm.group(1), c["f"])
        for p in c.get("played") or []:
            did = p.get("did") or ""
            # An attack ("attack party_N"), a detachment ("2 of group_A hunt party_N"), shelling or a D-gun.
            mm = re.search(r"(?:attack|hunt|shell|D-gun) (party_\d+)", did)
            if mm:
                orders.setdefault(mm.group(1), []).append(c["f"])
    # Answered: an order against the party from a minute before it was first named at an extractor (a party
    # already under attack when it gets there) to a minute after.
    for party, first in episodes.items():
        mine = [f for f in orders.get(party, []) if first - 60 * frames <= f <= first + 60 * frames]
        if mine:
            answered[party] = max(first, min(mine))
    # The pass lines: a quiet pass had something to ask and sent nothing; a pass with a `gate` asked. The Jev bill
    # is every call's and every decode's usage.
    quiet_s = sum(1 for c in m.calls if c.get("t") == "pass" and c.get("quiet"))
    open_s = quiet_s + sum(1 for c in m.calls if c.get("t") == "pass" and "gate" in c and not c.get("quiet"))
    tokens = sum((c.get("usage") or {}).get("input_tokens", 0) for c in m.calls if c.get("t") in ("call", "decode"))
    delays = [(answered[k] - episodes[k]) / frames for k in answered]
    unanswered = sum(1 for k in episodes if k not in answered)
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
    abandoned, abandoned_metal = abandoned_builds(m, defs, built_share)
    r = m.result or {}
    return {
        "match": m.label, "result": r.get("outcome"), "minutes": round(r.get("game_minutes", 0), 1),
        "idle%": round(100 * idle / alive, 1) if alive else None,
        "e0%": round(100 * e0 / total, 1) if total else None,
        "mfull%": round(100 * mfull / total, 1) if total else None,
        "react_s": round(med(delays, 0), 0) if delays else None,
        "unanswered": unanswered, "episodes": len(episodes),
        "noop%": round(100 * quiet_s / open_s, 0) if open_s else None,
        "jev$": round(tokens * 0.042 / 1e6, 2),
        "stuck_s": round(stuck_frames / frames),
        "yard_min": yard_min,
        "known%": round(med(known, 0)) if known else None,
        "fac_min": round(fac / frames / 60, 1) if fac else None,
        "look_min": round(look / frames / 60, 1) if look else None,
        "turn_s": round(med(walls, 0), 1) if walls else None,
        "aband": abandoned, "aband_m": round(abandoned_metal),
        # The opening against the pros' (run/replays/assist.py; the OS 40+ pool on Comet Catcher: 19 factory units by
        # 4:00, the metal store below 20 in 9% of the seconds from 1:00 to 4:00, the commander guarding the plant 50 s).
        **opening(m),
    }


def opening(m):
    try:
        sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "replays"))
        from assist import side_of, glossary
        row = side_of(os.path.relpath(m.dir, REPO), m.header.get("ai_id", 0), 240.0, glossary())
    except Exception:
        row = None
    if not row:
        return {"fac4": None, "stall4": None, "assist4": None}
    return {"fac4": row["fac_units"], "stall4": row["stalled"], "assist4": row["assist_s"]}


COLUMNS = ["result", "minutes", "idle%", "e0%", "mfull%", "react_s", "unanswered", "noop%", "jev$", "stuck_s", "yard_min", "known%", "fac_min", "look_min", "turn_s", "aband", "aband_m", "fac4", "stall4", "assist4"]


def abandoned_builds(m, defs, built_share):
    """Structures of ours started (`created`) and never `finished`, then `destroyed` with no attacker: the frame decayed
    after its builder left it (or an unseen enemy finished it; the bot's own accounting, `briefing.rs`, has the same
    blind spot). The count, and the metal put in: the frame's last sampled build share times its metal cost."""
    unfinished = {}
    count = 0
    metal = 0.0
    for e in m.events:
        d = defs.get(e.get("d"), {})
        if e["k"] == "created" and d.get("speed", 0) <= 0 and d.get("metal"):
            unfinished[e["u"]] = d["metal"]
        elif e["k"] == "finished":
            unfinished.pop(e["u"], None)
        elif e["k"] == "destroyed" and e["u"] in unfinished and e.get("by") is None:
            count += 1
            metal += unfinished.pop(e["u"]) * built_share.get(e["u"], 0.0)
    return count, metal


STUCK_FREE = 80.0
LANE_DEPTH = 320.0
LANE_MARGIN = 48.0
SQUARE = 8.0


def stuck_and_yards(m, defs, positions, frames, sample):
    """Unit-frames our mobile units were stuck (from a move failure until they moved STUCK_FREE), and the minutes some
    factory had a stuck unit in its exit lane, as `brain/yards.rs` draws it (None without footprints in the header)."""
    spans = {}  # unit -> [(start frame, end frame)]
    for e in m.events:
        if e["k"] != "move_failed" or defs.get(e.get("d"), {}).get("speed", 0) <= 0:
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
        d = defs.get(e.get("d"), {})
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
