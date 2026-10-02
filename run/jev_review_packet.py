#!/usr/bin/env python3
"""Builds a blind review packet from a `run/jev_context_ab.py` out file: questions a reviewer answers from the
picture and the instructions without seeing Jev's answers (`run/jev_review_score.py` scores the verdicts).

    run/jev_review_packet.py moves <match> <out.jsonl> <arm> <path>      the arm's crossings that asking again did not make, with controls, by gate
    run/jev_review_packet.py forbidden <match> <out.jsonl> <path>         a sample of forbidden questions, by packet
    run/jev_review_packet.py disagree <match> <out.jsonl> <path> <packets>   questions the file's arms answer on different sides of the bar, as <path>-1.md ...

Writes <path>.md (what the reviewer reads) and <path>.key.json (each item's answers; not for the reviewer)."""
import json, sys, collections, random, re
sys.path.insert(0, "run")
import jev_context_ab as J
D = ""
mode, match, out = sys.argv[1:4]
rows = collections.defaultdict(dict)
for line in open(out):
    r = json.loads(line); rows[r["f"]][r["arm"]] = r
kind = next(iter(rows.values()))["as played"]["kind"]
by = {c["f"]: (c, s) for c, s in J.requests(match, kind)}
rng = random.Random(17)
if mode == "moves":
    arm, label = sys.argv[4:6]
    MAX_FLIPS, MAX_CHARS = 45, 330_000
    gates = []
    for f, r in rows.items():
        flips, controls = [], []
        for k, a in r["recorded"]["answers"].items():
            if a is None or k.endswith(".forbidden"): continue
            b, again = r[arm]["answers"].get(k), r["recorded 2"]["answers"].get(k)
            if b is None or again is None: continue
            if J.crossed(k, a, b) and not J.crossed(k, a, again): flips.append(k)
            elif not J.crossed(k, a, b) and (min(a, b, again) >= 0.65 or 0.25 <= max(a, b, again) <= 0.4): controls.append(k)
        if flips: gates.append((f, flips, controls))
    rng.shuffle(gates)
    md = []; key = {}; n = 0; chars = 0; i = 0
    for f, flips, controls in gates:
        c, state = by[f]
        st = {k: v for k, v in state.items() if k != "rules"}
        text = json.dumps(st, indent=1, ensure_ascii=False)
        if chars + len(text) > MAX_CHARS or n >= MAX_FLIPS: break
        ctl = rng.sample(controls, min(len(controls), max(1, len(flips) // 2)))
        items = flips + ctl; rng.shuffle(items)
        md.append(f"\n\n# Moment {state['clock']} (frame {f})\n\nThe picture and the player's instructions at this second:\n\n```json\n{text}\n```\n\n## Questions at this moment\n")
        for k in items:
            i += 1; iid = f"q{i:03d}"
            md.append(f"\n**{iid}** `{k}`\n\n{c['questions'][k]['instructions']}\n")
            key[iid] = {"f": f, "k": k, "flip": k in flips, "recorded": rows[f]["recorded"]["answers"][k], "recorded 2": rows[f]["recorded 2"]["answers"][k], "arm": rows[f][arm]["answers"][k]}
        n += len(flips); chars += len(text) + sum(len(c["questions"][k]["instructions"]) for k in items)
    rules = next(iter(by.values()))[1]["rules"]
    open(D + label + ".md", "w").write(f"# Review packet {label.rsplit('/', 1)[-1]}\n\nThe rules the hands are given with every question:\n\n> {rules}\n" + "".join(md))
    json.dump(key, open(D + label + ".key.json", "w"))
    print(label.rsplit("/", 1)[-1], "items", len(key), "flips", sum(v["flip"] for v in key.values()), "chars", chars)
elif mode == "disagree":
    # Questions on which the file's arms land on different sides of the bar, a few a gate, in <packets> packets.
    label, packets = sys.argv[4], int(sys.argv[5])
    MAX_CHARS, PER_GATE = 320_000, 7
    arms = [a for a in next(iter(rows.values())) if a != "as played"]
    gates = []
    for f, r in rows.items():
        items = []
        for k, a in r["recorded"]["answers"].items():
            if a is None or k.endswith(".forbidden"): continue
            vs = [r[x]["answers"].get(k) for x in arms]
            if None not in vs and len({v >= J.FLAG for v in vs}) == 2: items.append(k)
        if items: gates.append((f, items))
    rng.shuffle(gates)
    rules = next(iter(by.values()))[1]["rules"]
    for n in range(packets):
        md = []; key = {}; chars = 0; i = 0
        while gates:
            f, items = gates[-1]
            c, state = by[f]
            text = json.dumps({k: v for k, v in state.items() if k != "rules"}, indent=1, ensure_ascii=False)
            if chars + len(text) > MAX_CHARS: break
            gates.pop()
            openers = [k for k in items if J.family(k) == "opener"]; moves = [k for k in items if J.family(k) != "opener"]
            items = openers[:3] + rng.sample(moves, min(len(moves), PER_GATE - min(3, len(openers)))); rng.shuffle(items)
            md.append(f"\n\n# Moment {state['clock']} (frame {f})\n\nThe picture and the player's instructions at this second:\n\n```json\n{text}\n```\n\n## Questions at this moment\n")
            for k in items:
                i += 1; iid = f"q{i:03d}"
                md.append(f"\n**{iid}** `{k}`\n\n{c['questions'][k]['instructions']}\n")
                key[iid] = {"f": f, "k": k, "arms": {x: rows[f][x]["answers"][k] for x in arms}}
            chars += len(text) + sum(len(c["questions"][k]["instructions"]) for k in items)
        path = f"{label}-{n + 1}"
        open(path + ".md", "w").write(f"# Review packet {path.rsplit('/', 1)[-1]}\n\nThe rules the hands are given with every question:\n\n> {rules}\n" + "".join(md))
        json.dump(key, open(path + ".key.json", "w"))
        print(path.rsplit("/", 1)[-1], "items", len(key), "chars", chars)
else:
    label = sys.argv[4]
    arm = "short forbidden" if any("short forbidden" in r for r in rows.values()) else "recorded 2"
    items = [(f, k) for f, r in rows.items() for k in r["recorded"]["answers"] if k.endswith(".forbidden") and r[arm]["answers"].get(k) is not None]
    rng.shuffle(items); items = items[:160]
    bypk = collections.defaultdict(list)
    for f, k in items: bypk[by[f][1]["instructions"]].append((f, k))
    md = []; key = {}; i = 0
    for pk, its in bypk.items():
        md.append(f"\n\n# Instructions in force at {by[its[0][0]][1]['clock']}\n\n```\n{pk}\n```\n\n## Moves asked about under these instructions\n")
        for f, k in its:
            i += 1; iid = f"q{i:03d}"
            c, state = by[f]
            move = c["questions"][k]["instructions"].split("The move: ", 1)[1]
            group = k.split(".")[0]
            entry = (state.get("actors") or {}).get(group)
            what = entry.get("is") if isinstance(entry, dict) else None
            md.append(f"\n**{iid}** at {state['clock']}, {group}" + (f" ({what})" if what else "") + f": {move}\n")
            key[iid] = {"f": f, "k": k, "arms": {x: r["answers"].get(k) for x, r in rows[f].items()}}
    open(D + label + ".md", "w").write(f"# Review packet {label.rsplit('/', 1)[-1]}\n" + "".join(md))
    json.dump(key, open(D + label + ".key.json", "w"))
    print(label.rsplit("/", 1)[-1], "items", len(key), "packets", len(bypk), "chars", sum(len(x) for x in md))
