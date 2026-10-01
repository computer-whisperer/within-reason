#!/usr/bin/env python3
"""The hands' Jev load of one game, from `jev-<ai>.jsonl`: tokens and cost, and per game minute the gates, the actors
asked and closed per gate, the questions and tokens a gate, its batches, and the seconds of gate and pick calls; then
the actor-asks by kind, the forbidden mark on hunts (`plan::FORBIDDEN`), the escorts, and the factory plays by source.

usage: run/jev_load.py run/matches/<batch>/<NN> [...]   (or a jev-<ai>.jsonl)
Jev's price: $0.042 a million input tokens, output free.
"""
import collections
import glob
import json
import os
import statistics
import sys

PRICE = 0.042 / 1e6
FORBIDDEN = 0.7


def kind_of(name):
    for prefix, kind in (("party_", "threat"), ("group_", "group"), ("constructor", "builder"), ("commander", "builder")):
        if name.startswith(prefix):
            return kind
    return "lab"


def read(path):
    for line in open(path):
        try:
            yield json.loads(line)
        except json.JSONDecodeError:
            continue


def report(path):
    minutes = collections.defaultdict(lambda: collections.Counter())
    asks = collections.Counter()
    tokens = calls_ms = errors = 0
    forbidden = []
    marked_offered = marked_candidate = marked_played = hunts_played = picks = changed = 0
    escort_asks, escort_plays = [], 0
    labs = collections.Counter()
    last_frame = 0
    marked_now = set()
    for r in read(path):
        t, f = r.get("t"), r.get("f", 0)
        last_frame = max(last_frame, f)
        m = minutes[f // 1800]
        if t == "error":
            errors += 1
        elif t == "pass":
            for name in r.get("open") or []:
                asks["asked_" + kind_of(name)] += 1
            for name in r.get("closed") or []:
                asks["closed_" + kind_of(name)] += 1
            if r.get("gate"):
                m["gates"] += 1
                m["asked"] += len(r.get("open") or [])
                m["closed"] += len(r.get("closed") or [])
        elif t == "call":
            used = (r.get("usage") or {}).get("input_tokens", 0)
            tokens += used
            calls_ms += r.get("ms", 0)
            questions = r.get("questions") or {}
            if "worlds.pick" in questions:
                m["pick_ms"] += r.get("ms", 0)
            else:
                m["gate_ms"] += r.get("ms", 0)
                m["gate_tokens"] += used
                m["questions"] += len(questions)
                m["batches"] += r.get("batches", 1)
                m["gate_calls"] += 1
            for key, answer in (r.get("answers") or {}).items():
                if ".escort_" in key and isinstance(answer, dict) and "noul" in answer:
                    escort_asks.append(answer["noul"])
        elif t == "worlds_gate":
            flags = r.get("flags") or {}
            marked_now = {k[: -len(".forbidden")] for k, v in flags.items() if k.endswith(".forbidden") and v >= FORBIDDEN}
            forbidden.extend(v for k, v in flags.items() if k.endswith(".forbidden"))
            marked_offered += sum("The player's instructions forbid this hunt" in str(line) for line in r.get("lines") or [])
        elif t == "plan":
            picks += 1
            changed += bool(r.get("changed"))
            for p in r.get("played") or []:
                played = p.get("played", "")
                if ".hunt_" in played:
                    hunts_played += 1
                    marked_played += played in marked_now
                if ".escort_" in played:
                    escort_plays += 1
        if t in ("pass", "plan"):
            for p in r.get("played") or []:
                if p.get("kind") == "lab":
                    labs[p.get("source")] += 1
    print(f"== {path}")
    print(f"through {last_frame // 1800}:{last_frame // 30 % 60:02d}; tokens {tokens / 1e6:.1f}M (${tokens * PRICE:.2f}), seconds of calls {calls_ms / 1000:.0f}, errors {errors}")
    print("min gates | asked closed per gate | q/gate tok/gate batches | gate s  pick s per game minute")
    for minute in sorted(minutes):
        m = minutes[minute]
        g, c = max(m["gates"], 1), max(m["gate_calls"], 1)
        print(f"{minute:3d} {m['gates']:4d} | {m['asked'] / g:5.1f} {m['closed'] / g:5.1f} | {m['questions'] / c:5.0f} {m['gate_tokens'] / c:6.0f} {m['batches'] / c:4.2f} | {m['gate_ms'] / 1000:5.1f} {m['pick_ms'] / 1000:5.1f}")
    print("actor-asks asked/closed:", dict(sorted(asks.items())))
    if forbidden:
        print(f"forbidden nouls: {len(forbidden)}, median {statistics.median(forbidden):.2f}, at {FORBIDDEN} or over {sum(v >= FORBIDDEN for v in forbidden)} ({100 * sum(v >= FORBIDDEN for v in forbidden) // len(forbidden)}%)")
    print(f"marked hunt lines offered {marked_offered}; a marked hunt played {marked_played}; hunts played in all {hunts_played}; picks {picks}, changed {changed}")
    if escort_asks:
        print(f"escort questions {len(escort_asks)}, median {statistics.median(escort_asks):.2f}; escorts played {escort_plays}")
    print("factory plays by source:", dict(labs))


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    for target in sys.argv[1:]:
        paths = [target] if target.endswith(".jsonl") else sorted(glob.glob(os.path.join(target, "jev-*.jsonl")))
        if not paths:
            print(f"no jev log at {target}")
        for path in paths:
            report(path)


if __name__ == "__main__":
    main()
