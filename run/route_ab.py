#!/usr/bin/env python3
"""Can Jev follow a stateful route written in prose? Recorded moments of player-9-posing (the 6:00 raid loop's five
arrivals, the scout's arrival at 3:53 and its fork at 3:58 with the enemy commander in sight) are put to Jev again
in five variants (the study: docs/studies/2026-09-28-route-in-prose.md):
  A  as recorded: the route is the `station` list (the hands' mechanic); the next leg is the base world's default.
  B  the route in the packet's prose, no station rule; the picture's `recent` says which stops were reached and when;
     the menu offers an advance to every remaining stop.
  C  as B without the `recent` progress lines: Jev must infer the next stop from where the group stands.
  D  as B, and the group's own entry says "reached X, its stop done; the instructions' next stop is Y, not yet ordered".
  E  as B, and the "nothing changes" world's line names the cost: the group holds at a reached stop with the route's
     next stop not ordered (the composer's convention for an idle lab).
Each variant asks the gate (nouls) over the group's candidates, then the worlds choice over the flagged ones, the way
the bot does. Measured: the next leg's noul, whether it is the top candidate and at or above the flag, the pick.
The key comes from TYPESAFE_API_KEY or ~/.config/within-reason/jev.env and is never printed or written.
usage: run/route_ab.py run/matches/<batch>/<NN> [--repeat N] [--out FILE.jsonl] [--variants A,B,C,D,E]
       run/route_ab.py --summarize FILE.jsonl
"""
import datetime
import json
import math
import os
import re
import statistics
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask  # noqa: E402

FLAG = 0.5
WORLDS_INSTRUCTIONS = ("Given `economy`, `ours`, `enemy`, `actors`, `player` and the player's `instructions`, which plan is best this second? "
    "Each option is one world: who changes course to do what, with what it costs and gives up, which enemy parties are met and which are left to nobody, who stays idle. "
    "w1 changes nothing: every actor keeps its course and the idle ones stay idle; every other world is w1 with one actor's course changed (a pair, when two hands go to one build), and its line says only that change. "
    "An idle factory or builder with metal in the store is a cost, not a course, unless the instructions say to wait. "
    "The instructions were written before this picture: where they name a place, a party, a building, a unit or a rule, follow them; where the situation has changed, pick the world they would call for.")

LOOP = ["spot_49", "spot_46", "spot_40", "spot_55", "spot_58", "spot_64"]
SCOUT = ["spot_72", "spot_60", "spot_42", "spot_34", "spot_25", "spot_17", "spot_10", "spot_5"]
LOOP_PROSE = ("group_A (Blitzes): one raid loop through the east-middle spots: spot_49, then spot_46, spot_40, spot_55, spot_58, and back to spot_64, "
    "in that order, advancing (it fights what it meets on the way and kills any undefended building of his it finds), then holds at spot_64.")
SCOUT_PROSE = ("group_B (the scout car): scouts spot_72, then spot_60, spot_42, spot_34, spot_25, spot_17, spot_10, spot_5, in that order, to find the enemy base. "
    "It is a scout: it runs from what can catch it and otherwise keeps to its route; being seen is its job.")

# (clock, group, route, reached [(clock, stop)], next stop, current words for the hold)
MOMENTS = [
    ("3:53", "group_B", SCOUT, [("3:37", "spot_72"), ("3:46", "spot_60"), ("3:53", "spot_42")], "spot_34"),
    ("3:58", "group_B", SCOUT, [("3:37", "spot_72"), ("3:46", "spot_60"), ("3:53", "spot_42")], "spot_34"),
    ("6:25", "group_A", LOOP, [("6:24", "spot_49")], "spot_46"),
    ("6:30", "group_A", LOOP, [("6:24", "spot_49"), ("6:30", "spot_46")], "spot_40"),
    ("6:36", "group_A", LOOP, [("6:24", "spot_49"), ("6:30", "spot_46"), ("6:36", "spot_40")], "spot_55"),
    ("6:46", "group_A", LOOP, [("6:24", "spot_49"), ("6:30", "spot_46"), ("6:36", "spot_40"), ("6:46", "spot_55")], "spot_58"),
    ("6:59", "group_A", LOOP, [("6:24", "spot_49"), ("6:30", "spot_46"), ("6:36", "spot_40"), ("6:46", "spot_55"), ("6:57", "spot_58")], "spot_64"),
]
SPEED = {"group_A": 101.0, "group_B": 168.0}


def frame_of(clock):
    m, s = clock.split(":")
    return int(m) * 1800 + int(s) * 30


def distance_words(d):
    if d < 300:
        return "right here"
    if d < 1200:
        return "near"
    if d < 3000:
        return "some way off"
    return "far"


def load(match):
    rules = None
    instructions = {}
    calls = {}
    for line in open(os.path.join(match, "jev-0.jsonl")):
        r = json.loads(line)
        if r.get("t") == "header":
            rules = r["rules"]
        if r.get("t") != "call":
            continue
        if "instructions" in r:
            instructions[r["f"]] = r["instructions"]
        if "worlds.pick" not in r["questions"]:
            calls[r["f"]] = r
    return rules, instructions, calls


def in_force(instructions, f):
    keys = [k for k in instructions if k <= f]
    return instructions[max(keys)] if keys else ""


def group_of(row, name):
    return next(g for g in row["groups"] if g["name"] == name.split("_")[1])


def place_at(row, name):
    p = next(p for p in row["places"] if p["name"] == name)
    return (p["x"], p["z"])


def current_words(row, group):
    """The group's current course as the recorded questions phrase it ("rather than X")."""
    for qid, q in row["questions"].items():
        if qid.startswith(group + ".") and not qid.endswith(".change"):
            m = re.search(r"rather than (.*?)\? The move: ", q["instructions"], re.S)
            if m:
                return m.group(1)
    return None


def build(row, rules, instructions, moment, variant):
    clock, group, route, reached, nxt = moment
    state = json.loads(json.dumps(row["state"]))
    state["rules"] = rules
    text = instructions
    questions = {}
    g = group_of(row, group)
    at = tuple(g["at"])
    entry = state["actors"][group]
    if variant == "A":
        for qid, q in row["questions"].items():
            if qid.startswith(group + "."):
                questions[qid] = q
        state["instructions"] = text
        return state, questions, [qid.split(".", 1)[1] for qid in questions if not qid.endswith(".change")]
    # B and C: the route in prose, no station rule, the menu of remaining stops.
    entry["standing"] = re.sub(r";? ?station [^;]*(; ?station_mode [a-z]+)?", "", entry["standing"]).replace(":;", ":").strip()
    if entry["standing"].endswith(":"):
        entry["standing"] = "no standing orders in force"
    text = text.rstrip() + "\n\n" + (LOOP_PROSE if group == "group_A" else SCOUT_PROSE)
    state["instructions"] = text
    done = [s for _, s in reached]
    if variant in ("B", "D", "E"):
        state["recent"] = state.get("recent", []) + [f"{c} {group} reached {s}" for c, s in reached]
    if variant == "D" and entry["doing"].startswith("holding"):
        last_clock, last_stop = reached[-1]
        entry["doing"] = f"reached {last_stop} at {last_clock} and holds there, its stop done; the instructions' next stop is {nxt}, not yet ordered"
    # The current course said as the hold (or the walk) the picture shows.
    here = entry["at"]
    doing = entry["doing"]
    if doing.startswith("holding"):
        current = f"{group} at {here}, {doing}"
    else:
        current = f"{group} {doing}"
    unguarded = f", leaving {here} unguarded" if doing.startswith("holding") else f", abandoning its way"
    speed = SPEED[group]
    candidates = []
    for stop in route:
        if stop in done:
            continue
        px, pz = place_at(row, stop)
        d = math.hypot(px - at[0], pz - at[1])
        if d < 250:
            continue
        verb = "advances to" if group == "group_A" else "walks to"
        fight = ", fighting on the way" if group == "group_A" else " without stopping to fight on the way"
        words = f"{group} {verb} {stop} ({d:.0f} away, {d / speed:.0f} s of walking){fight}{unguarded}"
        candidates.append((f"walk_{stop}", words))
    # The recorded other candidates (retreat, sweep, raids, splits, joins, fall back), re-phrased against the hold.
    for qid, q in row["questions"].items():
        if not qid.startswith(group + ".") or qid.endswith(".change"):
            continue
        sid = qid.split(".", 1)[1]
        if sid.startswith("station_") or any(sid == c[0] for c in candidates):
            continue
        m = re.search(r"The move: (.*)$", q["instructions"], re.S)
        if m:
            candidates.append((sid, m.group(1).strip()))
    questions[f"{group}.change"] = {"type": "noul", "instructions": (
        f"Given `actors.{group}`, `economy`, `enemy` and the player's `instructions`: should {group} do something other than what it does now ({current})? "
        "Yes when the instructions call for a different job now, when what it does is finished or pointless, or when something near it needs answering. "
        "No when its course is what the instructions want and nothing has changed.")}
    for sid, words in candidates:
        questions[f"{group}.{sid}"] = {"type": "noul", "instructions": (
            f"Given `actors.{group}`, `economy`, `ours` and the player's `instructions`: is this what {group} should do now, rather than {current}? The move: {words}")}
    return state, questions, [c[0] for c in candidates]


def worlds_of(questions, answers, group, idle_words=None):
    """w1 and one world per flagged candidate, best six by noul, the bot's shape."""
    rated = []
    for qid, a in answers.items():
        if qid.endswith(".change") or a.get("type") != "noul":
            continue
        rated.append((a["noul"], qid))
    rated.sort(reverse=True)
    worlds = {"w1": "Nothing changes: every actor keeps the course its entry under `actors` describes." + (f" {idle_words}" if idle_words else "")}
    ids = {"w1": None}
    for noul, qid in rated[:6]:
        if noul < FLAG:
            break
        m = re.search(r"The move: (.*)$", questions[qid]["instructions"], re.S)
        name = f"w{len(worlds) + 1}"
        worlds[name] = f"As w1, and: {m.group(1).strip()}."
        ids[name] = qid.split(".", 1)[1]
    return worlds, ids


def main():
    args = sys.argv[1:]
    match = args[0]
    repeat = int(args[args.index("--repeat") + 1]) if "--repeat" in args else 4
    out = args[args.index("--out") + 1] if "--out" in args else f"docs/studies/data/route-ab-{datetime.date.today().isoformat()}.jsonl"
    variants = args[args.index("--variants") + 1].split(",") if "--variants" in args else ["A", "B", "C", "D"]
    rules, instructions, calls = load(match)
    key = api_key()
    rows = []
    with open(out, "w") as fh:
        for moment in MOMENTS:
            clock, group, route, reached, nxt = moment
            f = frame_of(clock)
            row = calls.get(f)
            if row is None:
                near = [k for k in calls if abs(k - f) <= 30]
                if not near:
                    print(f"{clock}: no gate call recorded", file=sys.stderr)
                    continue
                row = calls[min(near, key=lambda k: abs(k - f))]
            text = in_force(instructions, row["f"])
            model = row["model"]
            for variant in variants:
                state, questions, candidates = build(row, rules, text, moment, variant)
                for rep in range(repeat):
                    gate = ask(key, state, questions, model)
                    answers = gate["answers"]
                    nouls = {qid.split(".", 1)[1]: a["noul"] for qid, a in answers.items() if a.get("type") == "noul"}
                    idle_words = None
                    if variant == "E" and state["actors"][group]["doing"].startswith("holding"):
                        idle_words = f"{group} holds at {state['actors'][group]['at']}, a stop of its route it has reached, with the route's next stop {nxt} not ordered."
                    worlds, ids = worlds_of(questions, answers, group, idle_words)
                    if variant == "E" and len(worlds) == 1 and f"{group}.walk_{nxt}" in questions:
                        m = re.search(r"The move: (.*)$", questions[f"{group}.walk_{nxt}"]["instructions"], re.S)
                        worlds["w2"] = f"As w1, and: {m.group(1).strip()}."
                        ids["w2"] = f"walk_{nxt}"
                    pick = "w1"
                    probs = {}
                    if len(worlds) > 1:
                        choice = ask(key, state, {"worlds.pick": {"type": "choice", "instructions": WORLDS_INSTRUCTIONS, "criteria": worlds}}, model)
                        pick = choice["answers"]["worlds.pick"]["choice"]
                        probs = choice["answers"]["worlds.pick"].get("probabilities", {})
                    picked = ids.get(pick)
                    next_id = f"walk_{nxt}" if variant != "A" else f"station_{nxt}"
                    cands = {k: v for k, v in nouls.items() if k != "change"}
                    top = max(cands, key=cands.get) if cands else None
                    r = {"clock": clock, "group": group, "variant": variant, "rep": rep, "next": nxt, "change": nouls.get("change"),
                         "next_noul": nouls.get(next_id), "top": top, "top_noul": cands.get(top), "flagged": sorted([k for k, v in cands.items() if v >= FLAG], key=lambda k: -cands[k]),
                         "pick": pick, "picked": picked, "probs": probs, "nouls": cands}
                    rows.append(r)
                    fh.write(json.dumps(r) + "\n")
                    fh.flush()
                    print(f"{clock} {variant} r{rep}: next {nxt} noul={r['next_noul']} top={top} ({r['top_noul']}) change={r['change']} flagged={r['flagged'][:4]} pick={pick}->{picked}", flush=True)
                    time.sleep(0.2)
    summarize(rows)


def summarize(rows):
    print("\nvariant | moment | next leg noul (median) | next is top | next flagged | continues the route (A: w1 plays the default; 3:58: w1 or a route stop) | pick = w1 | change noul")
    for variant in ("A", "B", "C", "D", "E"):
        for clock in [m[0] for m in MOMENTS]:
            rs = [r for r in rows if r["variant"] == variant and r["clock"] == clock]
            if not rs:
                continue
            nn = [r["next_noul"] for r in rs if r["next_noul"] is not None]
            med = f"{statistics.median(nn):.2f}" if nn else "n/a (not on the menu)"
            top = sum(1 for r in rs if r["top"] and r["top"].endswith(r["next"]))
            flagged = sum(1 for r in rs if any(k.endswith(r["next"]) for k in r["flagged"]))
            route = [m[2] for m in MOMENTS if m[0] == clock][0]
            picked = sum(1 for r in rs if (r["picked"] and r["picked"].endswith(r["next"])) or (variant == "A" and r["pick"] == "w1") or (clock == "3:58" and (r["pick"] == "w1" or (r["picked"] or "").replace("walk_", "") in route)))
            w1 = sum(1 for r in rs if r["pick"] == "w1")
            ch = statistics.median([r["change"] for r in rs if r["change"] is not None]) if any(r["change"] is not None for r in rs) else None
            print(f"{variant} | {clock} {rs[0]['group']} -> {rs[0]['next']} | {med} | {top}/{len(rs)} | {flagged}/{len(rs)} | {picked}/{len(rs)} | {w1}/{len(rs)} | {ch if ch is None else f'{ch:.2f}'}")


if __name__ == "__main__":
    if "--summarize" in sys.argv:
        rows = [json.loads(l) for l in open(sys.argv[sys.argv.index("--summarize") + 1])]
        summarize(rows)
    else:
        main()
