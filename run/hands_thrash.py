#!/usr/bin/env python3
"""How often the hands' picks change each actor's course (docs/design/2026-10-01-hands-rebuild.md §10: "a group's
course changes at most 4 a minute in its worst minute"). Per kind of actor and per actor: the picks that moved it,
its worst minute, and how many of its picks came within ten seconds of its last (a pick replaced before it could
have been carried out). Reads the `plan` lines of `jev-<ai>.jsonl` (versions 2 and 3: `played` with source `plan`).

    run/hands_thrash.py run/matches/<batch>/<NN> [--actors]
"""
import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from match_read import Match, clock  # noqa: E402


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if len(args) != 1:
        print(__doc__)
        sys.exit(2)
    m = Match(args[0].rstrip("/"))
    plays = collections.defaultdict(list)
    for c in m.calls:
        if c.get("t") != "plan":
            continue
        for p in c.get("played") or []:
            if p.get("source") == "plan":
                plays[(p.get("kind"), p.get("actor"))].append((c["f"], p.get("played"), p.get("did")))
    by_kind = collections.defaultdict(list)
    for (kind, actor), rows in plays.items():
        minutes = collections.Counter(f // 1800 for f, _, _ in rows)
        worst_minute, worst = max(minutes.items(), key=lambda kv: kv[1])
        soon = sum(1 for a, b in zip(rows, rows[1:]) if b[0] - a[0] <= 300)
        by_kind[kind].append((len(rows), worst, worst_minute, soon, actor))
    for kind in sorted(by_kind):
        rows = sorted(by_kind[kind], reverse=True)
        total = sum(r[0] for r in rows)
        print(f"{kind}: {len(rows)} actors, {total} picks moved one; within 10 s of the actor's last pick: {sum(r[3] for r in rows)} ({100 * sum(r[3] for r in rows) / max(1, total):.0f}%); the worst minute: {max(r[1] for r in rows)} picks on one actor")
        if "--actors" in sys.argv:
            for n, worst, minute, soon, actor in rows:
                print(f"  {actor}: {n} picks, {soon} within 10 s of its last, worst minute {minute}:00 with {worst}")


if __name__ == "__main__":
    main()
