#!/usr/bin/env python3
"""The hands' layers in one game (docs/design/2026-10-01-hands-rebuild.md §8): per layer, the questions it did not
send, what it asked anyway for its audit, and the audit's faults (`news`: a closed actor whose `change` and whose
best move both came back at 0.5 or over, so it would have gone to the pick; `same`: a standing answer and the fresh
one on different sides of 0.5; `fuse`: a fused-off move rated 0.5 or over when asked anyway, or a decoded forbidden
mark the second's own reading puts on the other side of 0.7; `openers`: a held move at 0.5 or over with its opener open; `places`: a blanked move that would have gone to the pick, at 0.5 with its actor open or an idle actor's at 0.3). The decode's own calls are counted in the bill. Beside it the game's Jev bill and what the gate
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
    if qid.startswith("worlds."):
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
    decodes = [c for c in m.calls if c.get("t") == "decode"]
    calls = [c for c in m.calls if c.get("t") == "call"] + decodes
    tokens = sum((c.get("usage") or {}).get("input_tokens", 0) for c in calls)
    picks = sum(1 for c in calls if any(q.startswith("worlds.") for q in (c.get("questions") or {})))
    plans = [c for c in m.calls if c.get("t") == "plan"]
    print(f"{gates} gates, {picks} pick calls, {len(plans)} picks ({sum(1 for p in plans if (p.get('changed') if 'split' in p else p.get('pick', 1) != 1))} changed something); {tokens / 1e6:.2f}M input tokens, ${tokens * 0.042 / 1e6:.2f}")
    if decodes:
        reads = [p for d in decodes for p in (d.get("reads") or {}).values()]
        print(f"the decode: {len(decodes)} requests, {len(reads)} readings ({sum(p < 0.2 for p in reads)} under 0.2, {sum(p >= 0.8 for p in reads)} at 0.8 or over), {sum((d.get('usage') or {}).get('input_tokens', 0) for d in decodes) / 1e6:.2f}M tokens")
    total = sum(asked.values())
    print(f"the gate asked {total} questions: " + ", ".join(f"{k} {n}" for k, n in asked.most_common()))
    for layer in sorted(set(skipped) | set(audited) | {a.get("layer") for a in audits}):
        mine = [a for a in audits if a.get("layer") == layer]
        faults = [a for a in mine if a.get("fault")]
        share = f"{100 * len(faults) / len(mine):.1f}%" if mine else "-"
        print(f"{layer}: {skipped[layer]} questions not sent ({100 * skipped[layer] / max(1, sum(skipped.values()) + total):.0f}% of what the base would ask), {len(mine)} audited, {len(faults)} faults ({share})")
        if "--faults" in sys.argv:
            for a in faults:
                if "id" in a:
                    print(f"  {clock(a['f'])} {a['id']}: assumed {a.get('assumed')}, asked again {a.get('fresh'):.2f}")
                else:
                    print(f"  {clock(a['f'])} {a['actor']}: change {a.get('change'):.2f}, {a.get('best')} {a.get('best_p'):.2f}")


if __name__ == "__main__":
    main()
