"""What the player (or the commander) ordered, turn by turn, beside what the harness said it did with it: the
reviewer's sweep for the Opus-to-hands impedance question (the user, 2026-10-16: "how many times did the harness not
follow what opus wanted, or was opus missing a key piece of game context that should have been available to it").

For every turn of a seat's transcript (`strategist-<N>.jsonl`, or `commander.jsonl` with --commander): the clock, why
the player was woken, when the previous orders landed (the think penalty), every order it gave in compact form, the
tool results that were refused (REFUSED, a tool that cannot be batched, a name the picture does not know, a
malformed argument, a call after the turn ended), its note, and then, from the next report, the lines of "what your hands did since
your last turn" that say an order did not happen as written (the engine refused a site, a blocked site, a skipped
list step, a hunt ended on its leash or with no hunters, a build abandoned, a follow ended, a list step waiting).
`--all` prints every hands' line and every result line instead of the flagged ones. At the end the counts, so a
seat's tally has its denominator: turns, orders by tool, refusals by tool, flagged hands' lines by kind, session
replacements, and the landing delays.

    run/impedance_sweep.py run/matches/<batch>/<NN> [--seat N] [--commander] [--all] [--from m:ss] [--to m:ss]
    run/impedance_sweep.py run/matches/<batch>/<NN> --counts          the counts alone, every seat and the commander

Reads the transcripts only (docs/harness/record-format.md says nothing of them; `crates/bot/src/strategist/transcript.rs`
writes them): `turn` rows carry the prompt the model read, `tool_call` rows what it called (`orders` carries the
turn's calls in `arguments.calls`, its `result` the per-tool answers), `turn_end` how the turn ended. What the hands
did between two turns is in the second turn's prompt, so a turn's hands' lines come from the turn after it.
"""
import argparse
import collections
import glob
import json
import os
import re
import sys

FLAGS = [
    ("engine-refused", re.compile(r"the engine refused")),
    ("site-blocked", re.compile(r"never started in the")),
    ("step-skipped", re.compile(r"skipped the step")),
    ("hunt-ended-leash", re.compile(r"hunt of \S+ ended after \d+ s: the leash")),
    ("hunt-ended-no-hunters", re.compile(r"hunt of \S+ ended after \d+ s: no hunters")),
    ("hunt-ended-other", re.compile(r"hunt of \S+ ended after")),
    ("abandoned", re.compile(r"abandoned an unfinished")),
    ("follow-ended", re.compile(r"followed \S+ for \d+ s")),
    ("stopped-on-stop", re.compile(r"on your `stop`")),
    ("step-waits", re.compile(r"waits for it")),
    ("other-fault", re.compile(r"\b(refused|cannot|could not|failed|is gone|dropped|stuck|gave up|ran from)\b")),
]
REFUSED = re.compile(r"REFUSED|is not a tool|not a tool that can be batched|not a (group|factory|builder|party|place)|nothing of ours has that name|no missile ready|lists are by (builder|actor) name|a grid cell is A1 to H8|not a builder standing now|Your turn is over and the game is running")


def clock(frame):
    s = max(0, int(frame)) // 30
    return f"{s // 60}:{s % 60:02d}"


def frame_of(text):
    m, s = text.split(":")
    return (int(m) * 60 + int(s)) * 30


def hands_block(prompt):
    """The 'what your hands did since your last turn' lines of a prompt, stripped."""
    m = re.search(r"what your hands did since your last turn:\n((?:  .*\n?)+)", prompt)
    return [line.strip() for line in m.group(1).splitlines() if line.strip()] if m else []


def flag_of(line):
    for name, rx in FLAGS:
        if rx.search(line):
            return name
    return None


def takeover(prompt):
    """A session replaced mid-game (the phrase follows a short realtime preamble in the human games)."""
    return "taking over mid-game" in prompt[:600]


def woken(prompt):
    first = prompt.split("\n", 1)[0]
    if takeover(prompt):
        m = re.search(r"\n\[(\d+:\d\d)\] Woken because: (.*)", prompt)
        return "TAKEOVER; " + (m.group(2) if m else "(no wake line)")
    m = re.match(r"\[(\d+:\d\d|before the game)\] (?:Woken because: )?(.*)", first)
    return m.group(2) if m else first[:120]


def landed(prompt):
    m = re.search(r"Your orders of (\d+:\d\d) came into force at (\d+:\d\d), (\d+) s after", prompt)
    return (m.group(1), m.group(2), int(m.group(3))) if m else None


def compact(value, n=160):
    s = json.dumps(value, ensure_ascii=False) if not isinstance(value, str) else value
    s = s.replace("\n", " ")
    return s if len(s) <= n else s[: n - 1] + "…"


def order_lines(tool, args):
    """One or more short lines for an order."""
    if tool == "instruct":
        text = args.get("text", "") if isinstance(args, dict) else str(args)
        return [f"instruct ({len(text)} chars): {compact(text, 200)}"]
    if tool == "queue" and isinstance(args, dict):
        out = []
        for builder, steps in args.items():
            if steps is None or steps == []:
                out.append(f"queue {builder}: cancelled")
            elif isinstance(steps, list):
                out.append(f"queue {builder}: {len(steps)} steps {compact(steps, 140)}")
            else:
                out.append(f"queue {builder}: {compact(steps, 80)}")
        return out
    if tool == "produce" and isinstance(args, dict):
        return [f"produce {actor}: {compact(units, 140)}" for actor, units in args.items()]
    if tool == "direct" and isinstance(args, dict):
        out = []
        if args.get("plan"):
            out.append(f"direct plan ({len(args['plan'])} chars): {compact(args['plan'], 200)}")
        for seat, text in (args.get("seats") or {}).items():
            out.append(f"direct {seat} ({len(text)} chars): {compact(text, 160)}")
        return out or ["direct (empty)"]
    if tool == "wait":
        return [f"wait {compact(args, 120)}"]
    if tool == "note":
        return []
    return [f"{tool} {compact(args, 160)}"]


def turns_of(path):
    """The turns of a transcript: [{frame, prompt, calls: [(tool, args)], results: [str], note, ended_by, wall}]."""
    turns = []
    cur = None
    for line in open(path):
        try:
            r = json.loads(line)
        except json.JSONDecodeError:
            continue
        kind = r.get("kind")
        if kind == "turn":
            cur = {"frame": r.get("frame", 0), "prompt": r.get("prompt", ""), "calls": [], "results": [], "notes": [], "ended_by": None, "wall": None}
            turns.append(cur)
        elif kind == "tool_call" and cur is not None:
            tool, args, res = r.get("tool"), r.get("arguments") or {}, r.get("result")
            if tool == "orders":
                for c in args.get("calls", []):
                    cur["calls"].append((c.get("tool"), c.get("arguments") or {}))
            else:
                cur["calls"].append((tool, args))
            if isinstance(res, str):
                cur["results"].append(res)
            elif res is not None:
                cur["results"].append(json.dumps(res, ensure_ascii=False))
        elif kind == "turn_end" and cur is not None:
            cur["ended_by"], cur["wall"] = r.get("ended_by"), r.get("wall_seconds")
    for t in turns:
        t["notes"] = [a.get("text", "") for tool, a in t["calls"] if tool == "note" and isinstance(a, dict)]
    return turns


def result_lines(results, every):
    out = []
    for res in results:
        if res.lstrip().startswith("{"):
            continue  # a tool's answer in JSON (situation, overview, plan): not an order's result
        for line in re.split(r"\n|(?<=\S) (?=[a-z_]+: (?:REFUSED|marked|footwork|noted|instructions|Turn over))", res):
            line = line.strip()
            if not line:
                continue
            if every or REFUSED.search(line):
                out.append(compact(line, 260))
    return out


def sweep(path, every=False, lo=None, hi=None, out=sys.stdout):
    turns = turns_of(path)
    counts = {"turns": len(turns), "takeovers": 0, "orders": collections.Counter(), "refused": collections.Counter(), "hands_flagged": collections.Counter(), "hands_lines": 0, "landing_s": [], "wall_s": []}
    for i, t in enumerate(turns):
        nxt = turns[i + 1] if i + 1 < len(turns) else None
        if takeover(t["prompt"]):
            counts["takeovers"] += 1
        for tool, _ in t["calls"]:
            if tool != "note":
                counts["orders"][tool] += 1
        refused = result_lines(t["results"], False)
        for line in refused:
            m = re.match(r"([a-z_]+): ", line)
            counts["refused"][m.group(1) if m else "?"] += 1
        hands = hands_block(nxt["prompt"]) if nxt else []
        counts["hands_lines"] += len(hands)
        flagged = [(flag_of(h), h) for h in hands]
        for f, _ in flagged:
            if f:
                counts["hands_flagged"][f] += 1
        land = landed(t["prompt"])
        if land:
            counts["landing_s"].append(land[2])
        if t["wall"] is not None:
            counts["wall_s"].append(t["wall"])
        if (lo is not None and t["frame"] < lo) or (hi is not None and t["frame"] > hi):
            continue
        head = f"== {clock(t['frame'])}  woken: {compact(woken(t['prompt']), 150)}"
        if land:
            head += f"  [orders of {land[0]} landed {land[1]}, +{land[2]} s]"
        if t["wall"] is not None:
            head += f"  (turn {t['wall']:.0f} s wall, ended by {t['ended_by']})"
        print(head, file=out)
        for tool, args in t["calls"]:
            for line in order_lines(tool, args):
                print("   " + line, file=out)
        for line in (result_lines(t["results"], True) if every else refused):
            print("   RESULT " + line, file=out)
        for n in t["notes"]:
            print("   note: " + compact(n, 300), file=out)
        if nxt:
            shown = [(f, h) for f, h in flagged if every or f]
            if shown:
                print(f"   hands, to {clock(nxt['frame'])} ({len(hands)} lines):", file=out)
                for f, h in shown:
                    print(f"     {compact(h, 220)}" + (f"  [{f}]" if f else ""), file=out)
    return counts


def print_counts(name, c, out=sys.stdout):
    land = sorted(c["landing_s"])
    wall = sorted(c["wall_s"])
    med = lambda xs: xs[len(xs) // 2] if xs else 0
    print(f"-- {name}: {c['turns']} turns, {c['takeovers']} session replacements, {sum(c['orders'].values())} orders, "
          f"{sum(c['refused'].values())} refused, {sum(c['hands_flagged'].values())} flagged of {c['hands_lines']} hands' lines; "
          f"landing median {med(land)} s (p90 {land[int(len(land) * 0.9)] if land else 0}), turn wall median {med(wall):.0f} s", file=out)
    print(f"   orders: {dict(c['orders'].most_common())}", file=out)
    if c["refused"]:
        print(f"   refused: {dict(c['refused'].most_common())}", file=out)
    print(f"   flagged: {dict(c['hands_flagged'].most_common())}", file=out)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("match")
    ap.add_argument("--seat", type=int, default=0)
    ap.add_argument("--commander", action="store_true")
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--from", dest="lo")
    ap.add_argument("--to", dest="hi")
    ap.add_argument("--counts", action="store_true")
    a = ap.parse_args()
    d = a.match
    if not any(f.startswith("strategist-") for f in os.listdir(d)) and os.path.isdir(os.path.join(d, "00")):
        d = os.path.join(d, "00")
    if a.counts:
        for p in sorted(glob.glob(os.path.join(d, "strategist-*.jsonl"))) + sorted(glob.glob(os.path.join(d, "commander.jsonl"))):
            c = sweep(p, lo=1, hi=0)  # print nothing
            print_counts(os.path.basename(p), c)
        return
    p = os.path.join(d, "commander.jsonl" if a.commander else f"strategist-{a.seat}.jsonl")
    if not os.path.exists(p):
        sys.exit(f"{p}: no such transcript")
    print(f"== {p}")
    c = sweep(p, every=a.all, lo=frame_of(a.lo) if a.lo else None, hi=frame_of(a.hi) if a.hi else None)
    print_counts(os.path.basename(p), c)


if __name__ == "__main__":
    main()
