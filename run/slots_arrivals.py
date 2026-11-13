#!/usr/bin/env python3
"""How long a new soldier takes to be with a group (in some group's members at a hands' call), by the minute it
was made; and how its coming ended.

usage: run/slots_arrivals.py run/matches/<batch>/<NN> [...]
"""
import bisect, collections, json, os, statistics, subprocess, sys

ARMY = {"armfav", "armflash", "armstump", "armsam", "armjanus", "armart", "armpincer", "armbull", "armlatnk", "armmart", "armcroc", "armmanni", "armyork", "armseer", "armmerl", "armpw", "armrock", "armham", "armwar", "armjeth"}


def read(d):
    path = d + "/jev-0.jsonl"
    text = open(path).read() if os.path.exists(path) else subprocess.run(["zstd", "-dc", path + ".zst"], capture_output=True).stdout.decode()
    first = {}
    spells = collections.defaultdict(list)   # unit -> list of [start, end] frames outside every group
    inside = {}
    frames = []
    for line in text.splitlines():
        try:
            r = json.loads(line)
        except Exception:
            continue
        if r.get("t") != "call" or not r.get("groups"):
            continue
        frames.append(r["f"])
        for g in r["groups"]:
            for u in g["members"]:
                first.setdefault(u, r["f"])
    defs, ud, born, died, end = None, {}, {}, {}, 0
    for line in open(d + "/record-0.jsonl"):
        r = json.loads(line)
        if "unit_defs" in r:
            defs = r["unit_defs"]
        end = max(end, r.get("f", 0))
        if r.get("t") == "ev":
            if r.get("k") == "finished":
                dd = defs[r["d"]]
                ud[r["u"]] = dd if isinstance(dd, str) else dd.get("name")
                born[r["u"]] = r["f"]
            if r.get("k") == "destroyed":
                died[r["u"]] = r["f"]
    rows = collections.defaultdict(lambda: {"n": 0, "join": [], "died": 0, "open": 0})
    for u, b in born.items():
        if ud[u] not in ARMY:
            continue
        m = (b // 1800) // 2 * 2
        row = rows[m]
        row["n"] += 1
        if u in first:
            row["join"].append((first[u] - b) / 30)
        elif u in died:
            row["died"] += 1
        else:
            row["open"] += 1
    print(os.path.basename(os.path.dirname(d.rstrip("/"))), f"(ends {end // 1800}:{end // 30 % 60:02d})")
    print("  made in    n   joined  median s  p90 s  over 120 s  died before joining  never joined by the end")
    for m in sorted(rows):
        row = rows[m]
        j = sorted(row["join"])
        med = f"{statistics.median(j):.0f}" if j else "-"
        p90 = f"{j[int(0.9 * (len(j) - 1))]:.0f}" if j else "-"
        print(f"  {m:2}-{m + 2:<2} min {row['n']:4} {len(j):7} {med:>9} {p90:>6} {sum(x > 120 for x in j):>11} {row['died']:>19} {row['open']:>24}")


for d in sys.argv[1:]:
    read(d)
