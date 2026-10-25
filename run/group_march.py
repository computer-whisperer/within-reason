#!/usr/bin/env python3
"""The lane's groups over a recorded match (the sample's `groups`, docs/harness/record-format.md): per declared group,
how long it was declared, how long it marched (a pace above zero), its formed share and pace share while marching,
its exceptions by cause in member-seconds, how often it was held (pace zero, not arrived, not engaged), engaged,
arrived, and the seconds a route point had no way on foot. The hands' migration measure (chunk 2,
docs/design/2026-10-25-hands-migration.md).

usage: run/group_march.py <match dir | record-*.jsonl> [...]
"""
import glob, json, os, sys
from collections import defaultdict


def records(path):
    if os.path.isdir(path):
        return sorted(glob.glob(os.path.join(path, "record-*.jsonl")))
    return [path]


def read(path):
    groups = defaultdict(lambda: {"seconds": 0, "marching": 0, "formed": 0.0, "pace": 0.0, "held": 0, "engaged": 0, "arrived": 0, "no_way": 0, "members": 0.0, "exceptions": defaultdict(float), "files": 0.0})
    frames = 0
    with open(path) as f:
        for line in f:
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            if row.get("t") != "s" or "groups" not in row:
                continue
            frames += 1
            for name, g in row["groups"].items():
                s = groups[name]
                s["seconds"] += 1
                members = len(g.get("formed", [])) + len(g.get("exceptions", []))
                s["members"] += members
                s["files"] += g.get("files", 0)
                if g.get("way") is None:
                    s["no_way"] += 1
                if g.get("engaged") is not None:
                    s["engaged"] += 1
                elif g.get("arrived"):
                    s["arrived"] += 1
                elif g.get("pace", 0) > 0:
                    s["marching"] += 1
                    s["formed"] += len(g.get("formed", [])) / max(1, members)
                    s["pace"] += g["pace"] / g["full_pace"] if g.get("full_pace") else 0.0
                else:
                    s["held"] += 1
                for _, cause, _ in g.get("exceptions", []):
                    s["exceptions"][cause] += 1
    return groups


def main():
    paths = [p for a in sys.argv[1:] for p in records(a)]
    if not paths:
        print(__doc__)
        sys.exit(2)
    for path in paths:
        groups = read(path)
        print(f"{path}: {len(groups)} declared groups")
        print("| group | declared s | marching s | formed share | pace share | held s | engaged s | arrived s | no way s | members | files | exceptions (member-s) |")
        print("|---|---|---|---|---|---|---|---|---|---|---|---|")
        for name, s in sorted(groups.items(), key=lambda kv: -kv[1]["seconds"]):
            m = s["marching"]
            exc = ", ".join(f"{c} {int(n)}" for c, n in sorted(s["exceptions"].items(), key=lambda kv: -kv[1])) or "-"
            print(f"| {name} | {s['seconds']} | {m} | {s['formed'] / m:.2f} | {s['pace'] / m:.2f} | {s['held']} | {s['engaged']} | {s['arrived']} | {s['no_way']} | {s['members'] / max(1, s['seconds']):.1f} | {s['files'] / max(1, s['seconds']):.1f} | {exc} |" if m else f"| {name} | {s['seconds']} | 0 | - | - | {s['held']} | {s['engaged']} | {s['arrived']} | {s['no_way']} | {s['members'] / max(1, s['seconds']):.1f} | {s['files'] / max(1, s['seconds']):.1f} | {exc} |")


if __name__ == "__main__":
    main()
