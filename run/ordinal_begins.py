#!/usr/bin/env python3
"""Where a territory batch falls behind thebluegecko's record, by ordinal: the n-th extractor, constructor, solar, plant
and turret each side began, side by side, with the seconds ours came after his (bots/territory/README.md, the
benchmark). His side from the record's `created` events (docs/harness/record-format.md; `d` indexes the header's
unit_defs), ours from the bot's `order:` and `begun:` lines in each game's engine.log.

    run/ordinal_begins.py <his record-N.jsonl> <batch dir> [--to S]

Read 2026-10-24 on terr97: the plant 16 s behind his from the opening's walk (1:17 against 1:01), his commander's pair
at 2:06 / 2:13 against our commander's solars, the Armada games' constructors closing the extractors to 2-12 s by
4:00-5:00 and the Cortex games' staying 20-40 s behind on a plant guarded less.
"""
import collections, json, re, sys

if len(sys.argv) < 3 or sys.argv[1].startswith("-"):
    print(__doc__); sys.exit(1)
rec, batch = sys.argv[1], sys.argv[2]
to = int(sys.argv[sys.argv.index("--to") + 1]) if "--to" in sys.argv else 330
header = None; events = []
for line in open(rec, errors="replace"):
    r = json.loads(line)
    if r.get("t") == "header": header = r
    elif r.get("t") == "ev" and r.get("k") == "created": events.append(r)
# find the def table
names = None
for k, v in header.items():
    if isinstance(v, dict) and all(isinstance(x, (dict, str)) for x in list(v.values())[:3]) and len(v) > 50:
        names = {int(i) if str(i).isdigit() else i: (x.get("name") if isinstance(x, dict) else x) for i, x in v.items()}
        print("def table:", k, len(v), file=sys.stderr); break
    if isinstance(v, list) and len(v) > 50 and isinstance(v[0], dict) and "name" in v[0]:
        names = {i: x["name"] for i, x in enumerate(v)}  # the events' d is the index into unit_defs
        print("def table (list):", k, len(v), file=sys.stderr); break
if names is None:
    print("header keys:", {k: (type(v).__name__, len(v) if hasattr(v, '__len__') else v) for k, v in header.items()}, file=sys.stderr); sys.exit(1)
def short(n): return re.sub(r"^(arm|cor|leg)", "", n or "?")
def side(evs):
    """evs: list of (seconds, unit kind, builder kind)."""
    out = collections.defaultdict(list)
    for s, u, b in sorted(evs):
        if s > to: break
        out[u].append((s, b))
    return out
his = []
builders = {}
for e in events:
    n = names.get(e["d"]); builders[e["u"]] = n
    his.append((e["f"] / 30, short(n), short(builders.get(e.get("by"), "?"))))
H = side(his)
games = {}
import glob, os
for g in sorted(os.path.basename(d) for d in glob.glob(f"{batch}/[0-9][0-9]") if os.path.exists(f"{d}/engine.log")):
    ours = []; kinds = {}
    for line in open(f"{batch}/{g}/engine.log", errors="replace"):
        m = re.search(r"f=(\d+) order: unit (\d+) (\w+):", line)
        if m: kinds[m.group(2)] = m.group(3); continue
        m = re.search(r"f=(\d+) begun: (\w+) by unit (\d+) at", line)
        if m: ours.append((int(m.group(1)) / 30, short(m.group(2)), short(kinds.get(m.group(3), "?"))))
    games[g] = side(ours)
def clock(s): return f"{int(s // 60)}:{s % 60:04.1f}"
for kind in ["mex", "vp", "cv", "solar", "avp", "nanotc", "acv", "win"]:
    rows = max([len(H.get(kind, []))] + [len(games[g].get(kind, [])) for g in games])
    if rows == 0: continue
    print(f"\n{kind}: ordinal | his (builder) | 00 | 01 | 02 | 03   (seconds behind his in brackets)")
    for i in range(rows):
        cells = []
        h = H.get(kind, [])
        hs = h[i][0] if i < len(h) else None
        cells.append(f"{clock(hs)} ({h[i][1]})" if hs is not None else "-")
        for g in games:
            o = games[g].get(kind, [])
            if i < len(o):
                d = f"[{o[i][0] - hs:+.0f}]" if hs is not None else ""
                cells.append(f"{clock(o[i][0])} ({o[i][1]}){d}")
            else: cells.append("-")
        print(f"  {i + 1:2d} | " + " | ".join(cells))
