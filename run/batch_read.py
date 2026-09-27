#!/usr/bin/env python3
"""One row per recorded game: the result, the economy at 8:00, 12:00 and 16:00, the army at 12:00, the player's
turns (count, wall seconds, abandoned, output tokens a turn, tool calls, refused calls, and the spend when the API
backend priced it; a subscription game shows none) and the hands' bill. The first table of a batch review (`.claude/skills/bar-review/SKILL.md`), to pick
the games worth reading in full; the fundamentals are `run/floor.py`'s.

    run/batch_read.py run/matches/<batch> [...] [--md]

A batch directory's `00` is read (a batch of one, as the player games are); a match directory is read as itself.
Reads `results.jsonl` (outcome, minutes, `reason` when the match was stopped), the record (counts by class at the
minute's sample, frames included), the transcript `strategist-<ai>.jsonl` (`turn`, `turn_end` with `ended_by`,
`tool_call`, `result` with the model's usage and `spent_usd`, `cost_cap`) and the pianist's log (`usage.input_tokens`
of every `call` and `decompress` line, priced at $0.042 a million). `--md` prints a markdown table.
"""
import glob
import json
import os
import statistics
import sys

FPS = 30
JEV_PRICE_PER_MILLION = 0.042


def rows(path):
    for line in open(path):
        try:
            yield json.loads(line)
        except ValueError:
            continue


def read(directory):
    d = directory.rstrip("/")
    if not glob.glob(os.path.join(d, "record-*.jsonl")) and os.path.isdir(os.path.join(d, "00")):
        d = os.path.join(d, "00")
    batch = os.path.dirname(d)
    record = sorted(glob.glob(os.path.join(d, "record-*.jsonl")))[0]
    header = json.loads(open(record).readline())
    defs = header["unit_defs"]
    cls = lambda i: defs[i]["class"] if 0 <= i < len(defs) else "?"
    extractors, soldiers = {}, {}
    for r in rows(record):
        if r.get("t") == "s" and r["f"] % (4 * 60 * FPS) == 0:
            minute = r["f"] // (60 * FPS)
            extractors[minute] = sum(1 for u in r["own"] if cls(u[1]) == "extractor")
            soldiers[minute] = sum(1 for u in r["own"] if cls(u[1]) == "army")
    walls, ended, outputs, tools, refused, turns, spent = [], [], [], 0, 0, 0, 0.0
    capped = False
    for r in rows(next(iter(sorted(glob.glob(os.path.join(d, "strategist-*.jsonl")))), "/dev/null")):
        kind = r.get("kind")
        if kind == "turn":
            turns += 1
        elif kind == "turn_end":
            walls.append(float(r.get("wall_seconds") or 0))
            ended.append(r.get("ended_by"))
        elif kind == "cost_cap":
            capped = True
        elif kind == "tool_call":
            tools += 1
            if "refused" in str(r.get("result", "")):
                refused += 1
        elif kind == "result":
            m = r.get("message")
            m = json.loads(m) if isinstance(m, str) else m
            if isinstance(m, dict):
                out = (m.get("usage") or {}).get("output_tokens")  # this turn's; `modelUsage` is the session's running total
                if out:
                    outputs.append(out)
                if m.get("spent_usd") is not None:
                    spent = max(spent, float(m["spent_usd"]))
    tokens = 0
    for r in rows(next(iter(sorted(glob.glob(os.path.join(d, "jev-*.jsonl")))), "/dev/null")):
        if r.get("t") in ("call", "decompress"):
            tokens += (r.get("usage") or {}).get("input_tokens", 0)
    result = {}
    if os.path.exists(os.path.join(batch, "results.jsonl")):
        index = int(os.path.basename(d)) if os.path.basename(d).isdigit() else 0
        result = next((r for r in rows(os.path.join(batch, "results.jsonl")) if r.get("index") == index), {})
    return {"label": os.path.basename(batch), "result": result.get("outcome"), "reason": result.get("reason") or "", "min": round(result.get("game_minutes", 0), 1),
            "mex8": extractors.get(8), "mex12": extractors.get(12), "mex16": extractors.get(16), "army12": soldiers.get(12),
            "turns": turns, "wall_med": round(statistics.median(walls), 1) if walls else None, "wall_max": round(max(walls)) if walls else None,
            "abandoned": ended.count("abandoned"), "out_med": round(statistics.median(outputs)) if outputs else None,
            "tools_per_turn": round(tools / turns, 2) if turns else None, "refused": refused,
            "jev_usd": round(tokens * JEV_PRICE_PER_MILLION / 1e6, 2), "player_usd": round(spent, 2) or None, "capped": capped}


def main():
    args = sys.argv[1:]
    if not args or "-h" in args or "--help" in args:
        print(__doc__)
        sys.exit(2)
    md = "--md" in args
    table = [read(a) for a in args if not a.startswith("--")]
    keys = list(table[0].keys())
    if md:
        print("| " + " | ".join(keys) + " |")
        print("|" + "---|" * len(keys))
        for r in table:
            print("| " + " | ".join("" if r[k] is None else str(r[k]) for k in keys) + " |")
    else:
        widths = {k: max(len(k), *(len("" if r[k] is None else str(r[k])) for r in table)) for k in keys}
        print("  ".join(k.ljust(widths[k]) for k in keys))
        for r in table:
            print("  ".join(("" if r[k] is None else str(r[k])).ljust(widths[k]) for k in keys))


if __name__ == "__main__":
    main()
