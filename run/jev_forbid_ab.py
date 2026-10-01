#!/usr/bin/env python3
"""Offline: every arm a recorded gate offered is asked about a second time as a reading of the instructions ("do
they forbid this move"), the question hunts already get in the game (`plan::FORBIDDEN`, threats.rs). Nothing is
played; the answers are set beside the arm's own noul and what the pick played that second.

usage: run/jev_forbid_ab.py run/matches/<batch>/<NN> --out <file.jsonl> [--from m:ss --to m:ss] [--every N]
       run/jev_forbid_ab.py --read <file.jsonl> [--actor group_X4] [--from m:ss --to m:ss]
`--every N` takes every Nth gate call outside the window (0: none). The rows: f, key, kind, noul (the arm's own),
forbidden (the new noul), recorded (the game's forbidden noul, hunts only), played.
The key is read by run/jev_ab.py's api_key() and never printed or written. Jev: $0.042 a million input tokens."""
import argparse, collections, concurrent.futures, json, os, re, statistics, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from match_read import Match

REQUEST_CHARS = 150_000  # crates/jev/src/lib.rs
BAR = 0.7  # plan::FORBIDDEN


def frames(clock):
    m, s = clock.split(":")
    return (int(m) * 60 + int(s)) * 30


def clock(f):
    return f"{f // 1800}:{f // 30 % 60:02d}"


def kind_of(key):
    """An arm's kind: its state with the numbers and group names taken out, behind the kind of its actor."""
    actor, state = key.split(".", 1)
    actor = "party" if actor.startswith("party_") else "group" if actor.startswith("group_") else "plant" if actor.startswith(("plant_", "lab_")) else "builder"
    state = re.sub(r"group_[A-Z]+\d*", "group", re.sub(r"\d+", "N", state))
    if actor == "builder" and state.startswith("arm"):
        state = "build"
    if actor == "plant" and state.startswith("arm"):
        state = "make"
    return f"{actor}.{state}"


def move_of(key, text):
    """An arm's words out of its question: after "The move: " in the long form, after "<actor>, rather than
    <what stands>: " in the short one (a group's walks and advances)."""
    if "The move: " in text:
        return text.split("The move: ", 1)[1]
    actor = key.split(".", 1)[0]
    if text.startswith(f"{actor}, rather than ") and f": {actor} " in text:
        return f"{actor} " + text.split(f": {actor} ", 1)[1]
    return None


def forbid_question(key, move):
    """The hunt's question as the game asks it, for any arm: the mover is the group a threat's arm names, or the
    arm's own actor."""
    actor = key.split(".", 1)[0]
    who = "the group that would make it" if actor.startswith("party_") else actor
    return {"type": "noul", "instructions": f"Read the player's `instructions` alone: do they forbid this move for {who}? The move: {move}"}


def batches(state, questions):
    """The questions in batches whose bodies stay under the service's limit, as the bot sends them."""
    room = REQUEST_CHARS - len(json.dumps(state))
    out, batch, used = [], {}, 0
    for key, q in questions.items():
        size = len(json.dumps(q)) + len(key) + 8
        if batch and used + size > room:
            out.append(batch)
            batch, used = {}, 0
        batch[key] = q
        used += size
    if batch:
        out.append(batch)
    return out


def run(args):
    key = api_key()
    m = Match(args.match)
    lo, hi = frames(args.start), frames(args.end)
    played = collections.defaultdict(set)
    for c in m.calls:
        if c.get("t") in ("plan", "pass"):
            for p in c.get("played") or []:
                if p.get("source") == "plan":
                    played[c["f"]].add(p.get("played"))
    jobs, outside = [], 0
    for c, state, _packet in requests_of(m):
        if "worlds.pick" in c["questions"]:
            continue
        inside = lo <= c["f"] <= hi
        if not inside:
            outside += 1
            if not args.every or outside % args.every:
                continue
        moves = {k: move_of(k, str(q.get("instructions"))) for k, q in c["questions"].items() if not k.endswith(".forbidden")}
        questions = {k + ".forbidden": forbid_question(k, move) for k, move in moves.items() if move}
        for batch in batches(state, questions):
            jobs.append((c, state, batch))
    print(f"{len(jobs)} calls, {sum(len(b) for _, _, b in jobs)} questions", file=sys.stderr)

    def one(job):
        c, state, batch = job
        answer = ask(key, state, batch, c["model"])
        rows = []
        for k, a in (answer.get("answers") or {}).items():
            arm = k[: -len(".forbidden")]
            rows.append({"f": c["f"], "key": arm, "kind": kind_of(arm), "noul": (c["answers"].get(arm) or {}).get("noul"), "forbidden": a.get("noul"),
                         "recorded": (c["answers"].get(k) or {}).get("noul"), "played": arm in played[c["f"]]})
        return rows, (answer.get("usage") or {}).get("input_tokens", 0)

    tokens = 0
    with open(args.out, "w") as out, concurrent.futures.ThreadPoolExecutor(8) as pool:
        for rows, used in pool.map(one, jobs):
            tokens += used
            for row in rows:
                out.write(json.dumps(row) + "\n")
    print(f"{tokens} input tokens, ${tokens * 0.042 / 1e6:.3f}", file=sys.stderr)


def read(args):
    rows = [json.loads(line) for line in open(args.read)]
    rows = [r for r in rows if r["forbidden"] is not None and frames(args.start) <= r["f"] <= frames(args.end)]
    if args.actor:
        rows = [r for r in rows if args.actor in r["key"]]
    print(f"{len(rows)} arms over {len({r['f'] for r in rows})} seconds; forbidden at {BAR} or over: {sum(r['forbidden'] >= BAR for r in rows)} ({100 * sum(r['forbidden'] >= BAR for r in rows) // max(len(rows), 1)}%)")
    by = collections.defaultdict(list)
    for r in rows:
        by[r["kind"]].append(r)
    print(f"{'kind':34s} {'n':>6s} {'median':>6s} {'p10':>5s} {'p90':>5s} {'>=bar':>6s} | played n, of them >=bar | own noul: marked, unmarked")
    for kind, rs in sorted(by.items(), key=lambda kv: -len(kv[1])):
        v = sorted(r["forbidden"] for r in rs)
        p = [r for r in rs if r["played"]]
        marked = [r["noul"] for r in rs if r["forbidden"] >= BAR and r["noul"] is not None]
        free = [r["noul"] for r in rs if r["forbidden"] < BAR and r["noul"] is not None]
        med = lambda xs: f"{statistics.median(xs):.2f}" if xs else "  - "
        print(f"{kind:34s} {len(v):6d} {statistics.median(v):6.2f} {v[len(v) // 10]:5.2f} {v[len(v) * 9 // 10]:5.2f} {100 * sum(x >= BAR for x in v) // len(v):5d}% | {len(p):4d} {sum(r['forbidden'] >= BAR for r in p):4d} | {med(marked)} {med(free)}")
    hunts = [r for r in rows if r["recorded"] is not None]
    if hunts:
        agree = sum((r["forbidden"] >= BAR) == (r["recorded"] >= BAR) for r in hunts)
        print(f"hunts, the replay against the game's own forbidden noul: {len(hunts)}, same side of the bar {100 * agree // len(hunts)}%, median difference {statistics.median(abs(r['forbidden'] - r['recorded']) for r in hunts):.2f}")
    plays = [r for r in rows if r["played"]]
    print(f"played arms {len(plays)}, marked {sum(r['forbidden'] >= BAR for r in plays)}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("match", nargs="?")
    ap.add_argument("--out")
    ap.add_argument("--read")
    ap.add_argument("--from", dest="start", default="0:00")
    ap.add_argument("--to", dest="end", default="999:00")
    ap.add_argument("--every", type=int, default=0)
    ap.add_argument("--actor")
    args = ap.parse_args()
    if args.read:
        read(args)
    elif args.match and args.out:
        run(args)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
