#!/usr/bin/env python3
"""The hands' layers in one game (docs/design/2026-10-01-hands-rebuild.md §8): per layer, the questions it did not
send, the actors it asked anyway for its audit, and the audit's faults (a closed actor whose `change` and whose best
move both came back at 0.5 or over: it would have gone to the pick). Beside it the game's Jev bill and what the gate
asked, by kind of question.

    run/layers_read.py run/matches/<batch>/<NN> [--faults]

Reads `jev-<ai>.jsonl` of version 3: the header's `layers`, the `pass` lines' `layers` and `gate`, the `audit`
lines, the `call` lines' usage. `--faults` prints each fault. Jev: $0.042 a million input tokens, output free.
"""
import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from match_read import Match, clock  # noqa: E402


def kind_of(qid):
    if qid == "worlds.pick":
        return "pick"
    tail = qid.rsplit(".", 1)[-1]
    if tail in ("answer", "change", "forbidden"):
        return tail
    verb = qid.split(".", 1)[1] if "." in qid else qid
    for name in ("fight_to", "take_apart", "go", "attack", "send", "shell", "join", "follow", "build", "help", "repair", "dgun", "make", "hold", "gather", "scout"):
        if verb.startswith(name):
            return name
    return "other"


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if len(args) != 1:
        print(__doc__)
        sys.exit(2)
    m = Match(args[0].rstrip("/"))
    header = m.jev_header or {}
    print(f"layers on: {', '.join(header.get('layers') or []) or 'none'} (log version {header.get('version')})")
    skipped, audited, gates = collections.Counter(), collections.Counter(), 0
    asked = collections.Counter()
    for c in m.calls:
        if c.get("t") == "pass" and c.get("gate"):
            gates += 1
            for layer, counts in (c.get("layers") or {}).items():
                skipped[layer] += counts.get("skipped", 0)
                audited[layer] += counts.get("audited", 0)
            for qid in c["gate"]:
                asked[kind_of(qid)] += 1
    audits = [c for c in m.calls if c.get("t") == "audit"]
    calls = [c for c in m.calls if c.get("t") == "call"]
    tokens = sum((c.get("usage") or {}).get("input_tokens", 0) for c in calls)
    picks = sum(1 for c in calls if "worlds.pick" in (c.get("questions") or {}))
    plans = [c for c in m.calls if c.get("t") == "plan"]
    print(f"{gates} gates, {picks} pick calls, {len(plans)} picks ({sum(1 for p in plans if p.get('pick', 1) != 1)} changed something); {tokens / 1e6:.2f}M input tokens, ${tokens * 0.042 / 1e6:.2f}")
    total = sum(asked.values())
    print(f"the gate asked {total} questions: " + ", ".join(f"{k} {n}" for k, n in asked.most_common()))
    for layer in sorted(set(skipped) | set(audited) | {a.get("layer") for a in audits}):
        mine = [a for a in audits if a.get("layer") == layer]
        faults = [a for a in mine if a.get("fault")]
        share = f"{100 * len(faults) / len(mine):.1f}%" if mine else "-"
        print(f"{layer}: {skipped[layer]} questions not sent ({100 * skipped[layer] / max(1, skipped[layer] + total):.0f}% of what the base would ask), {len(mine)} actors audited, {len(faults)} faults ({share})")
        if "--faults" in sys.argv:
            for a in faults:
                print(f"  {clock(a['f'])} {a['actor']}: change {a.get('change'):.2f}, {a.get('best')} {a.get('best_p'):.2f}")


if __name__ == "__main__":
    main()
