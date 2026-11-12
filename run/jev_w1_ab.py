#!/usr/bin/env python3
"""Offline: is world 1 privileged by how the pick says it? The recorded picks of a game asked again as recorded
and with world 1's marks taken off, one at a time (the user, 2026-10-24: "allowing the current control state to
exist in the world set. We should make sure the wordings don't privilege that option"; 2026-11-11: "'nothing
changes' is exactly that privilege I had hoped to eliminate"). Nothing is played. The key is read, never printed.

    run/jev_w1_ab.py run/matches/<batch>/<NN> [--picks N] [--workers W] [--out FILE.jsonl] [--summarize FILE.jsonl]

World 1's marks in the pick as the bot asks it: its line begins "Nothing changes:"; the question has a sentence of
its own for it ("w1: every actor keeps the course its entry under `actors` describes, and the idle ones stay as
they are. Every other option changes ..."); it is the first option; every other world is written as a difference
from it ("As w1, and: ...").

Arms: `again` (as recorded, the noise floor); `unlabelled` (w1's line without "Nothing changes:", the question
without its sentence about w1); `last` (w1 listed after the other worlds, its key and its words unchanged);
`both`. The difference form of the other worlds stays in every arm: undoing it needs each actor's course in words
on every line, which the log does not hold for a closed actor.
"""
import collections
import concurrent.futures
import datetime
import json
import os
import random
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of  # noqa: E402
from match_read import Match  # noqa: E402

ARMS = ("recorded", "again", "unlabelled", "last", "both")
LABEL = "Nothing changes: every actor keeps the course its entry under `actors` describes"
PLAIN = "Every actor keeps the course its entry under `actors` describes"
SENTENCE = " w1: every actor keeps the course its entry under `actors` describes, and the idle ones stay as they are. Every other option changes one or more actors' courses, its line saying only what it changes from w1 and what that alters."


def arm_of(questions, arm):
    out = {}
    for qid, q in questions.items():
        if not (qid.startswith("worlds.c") and qid.endswith(".pick")) or arm == "again":
            out[qid] = q
            continue
        lines = dict(q["criteria"])
        instructions = q["instructions"]
        if arm in ("unlabelled", "both"):
            if not lines["w1"].startswith(LABEL) or SENTENCE not in instructions:
                raise ValueError("the pick's words are not the ones this tool knows")
            lines["w1"] = PLAIN + lines["w1"][len(LABEL):]
            instructions = instructions.replace(SENTENCE, "")
        if arm in ("last", "both"):
            lines = {**{k: v for k, v in lines.items() if k != "w1"}, "w1": lines["w1"]}
        out[qid] = {**q, "instructions": instructions, "criteria": lines}
    return out


def summarize(rows):
    cells = collections.defaultdict(lambda: collections.defaultdict(list))
    for r in rows:
        for qid, k in r["worlds"].items():
            for arm in ARMS:
                p = ((r["answers"].get(arm) or {}).get(qid) or {}).get("probabilities") or {}
                if p:
                    total = sum(p.values()) or 1.0
                    cells[min(k, 4)][arm].append((p.get("w1", 0.0) / total, max(p, key=p.get) == "w1"))
    for k in sorted(cells):
        print(f"\n{k}{'+' if k == 4 else ''} worlds: {len(cells[k]['again'])} components (an even split gives world 1 {1 / k:.2f}{' or less' if k == 4 else ''})")
        print(f"  {'arm':11} {'world 1 mean share':>19} {'median':>7} {'the top world':>14}")
        for arm in ARMS:
            c = cells[k][arm]
            if c:
                print(f"  {arm:11} {statistics.mean(x[0] for x in c):19.2f} {statistics.median(x[0] for x in c):7.2f} {sum(x[1] for x in c) / len(c):14.0%}")


def main():
    args = sys.argv[1:]
    if "--summarize" in args:
        summarize([json.loads(line) for line in open(args[args.index("--summarize") + 1])])
        return
    value = lambda flag, default: args[args.index(flag) + 1] if flag in args else default  # noqa: E731
    count, workers = int(value("--picks", "200")), int(value("--workers", "8"))
    out = value("--out", os.path.join("docs", "studies", "data", f"w1-framing-{datetime.date.today().isoformat()}.jsonl"))
    match = Match(args[0])
    picks = [(c, state) for c, state, _ in requests_of(match) if any(k.startswith("worlds.c") for k in c["questions"])]
    random.seed(11)
    random.shuffle(picks)
    picks = sorted(picks[:count], key=lambda p: p[0]["f"])
    key = api_key()

    def run(job):
        c, state, arm = job
        reply = ask(key, state, arm_of(c["questions"], arm), c.get("model", "jev-1.14.0")) or {}
        return c["f"], arm, reply.get("answers") or {}

    rows = {c["f"]: {"game": match.label, "f": c["f"], "worlds": {qid: len(q["criteria"]) for qid, q in c["questions"].items() if qid.startswith("worlds.c")}, "answers": {"recorded": c["answers"]}} for c, _ in picks}
    with concurrent.futures.ThreadPoolExecutor(workers) as pool:
        for f, arm, answers in pool.map(run, [(c, state, arm) for c, state in picks for arm in ARMS[1:]]):
            rows[f]["answers"][arm] = answers
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w") as f:
        for row in rows.values():
            f.write(json.dumps(row) + "\n")
    print(f"{len(picks)} picks of {match.label}; written to {out}")
    summarize(list(rows.values()))


if __name__ == "__main__":
    main()
