#!/usr/bin/env python3
"""How the hands used detachments in a game, from the record and the hands' log (the user, 2026-10-17: "we usually need
the freedom for jev to detach units and repel an enemy party, and if jev is not using that power coherently then that
is something to fix"): the sends by size and by the share of the group they took, the sends at a party already sent at
within the minute, the sends by a group that was itself a detachment, the hunts by how they ended and how long they ran,
the soldiers lost while hunting and while attacking a party, and the chase asks (a `chase meets` event: a turret's reach ahead on the quarry's line
or a party beside the chase that is not its quarry) with what the group picked in the fifteen seconds after each
(kept on, or withdrew: a walk, a join, a follow, an advance elsewhere).

usage: run/hunts.py MATCH... [--json OUT] [--quiet]
  MATCH is a match directory (run/matches/<game>/NN) or a batch directory (every NN in it).

The record's `group` rows carry the picks (the move and what was done); the hands' log (`jev-N.jsonl.zst`, streamed)
carries each row's groups with their members and tasks, its events, and the menus with the moves' words. A soldier
counted lost while chasing is a member of a hunting or attacking group in one row that is in no group at all in the
next; a merge or a reassignment moves it to another group and is not counted.
"""
import argparse
import collections
import glob
import json
import os
import re
import subprocess
import sys

WITHDRAW = ("go_", "join_group_", "follow_", "fight_to_", "gather", "hold", "shell_", "rove")
WINDOW = 15 * 30


def matches_of(path):
    if glob.glob(os.path.join(path, "record-*.jsonl")):
        return [path]
    return sorted(d for d in glob.glob(os.path.join(path, "[0-9][0-9]")) if glob.glob(os.path.join(d, "record-*.jsonl")))


def clock(f):
    return f"{f // 1800}:{f % 1800 // 30:02d}"


def read(match):
    rec = glob.glob(os.path.join(match, "record-*.jsonl"))[0]
    jevs = glob.glob(os.path.join(match, "jev-*.jsonl.zst")) + glob.glob(os.path.join(match, "jev-*.jsonl"))
    picks = []
    for l in open(rec):
        r = json.loads(l)
        if r.get("kind") != "group":
            continue
        mv = (r.get("inputs") or {}).get("move", "")
        picks.append((r["f"], mv, str((r.get("outputs") or {}).get("did", ""))))
    out = {"match": match, "sends": [], "hunts": [], "chase_asks": [], "lost_hunting": 0, "lost_engaging": 0, "attack_picks": 0, "join_picks": 0, "hunt_ends": collections.Counter(), "hunt_seconds": []}
    send_at = {}
    detachments = set()
    for f, mv, did in picks:
        m = re.match(r"(group_\w+)\.send_(\d+)_(party_\d+)", mv)
        if m:
            actor, n, party = m.group(1), int(m.group(2)), m.group(3)
            dup = party in send_at and f - send_at[party] <= 1800
            send_at[party] = f
            out["sends"].append({"f": f, "actor": actor, "n": n, "party": party, "chained": actor in detachments, "repeat": dup, "size": None, "words": "", "did": did})
            mm = re.search(r"as (group_\w+)", did)
            if mm:
                detachments.add(mm.group(1))
            mm = re.search(r"left behind are (group_\w+)", did)
            if mm:
                detachments.add(actor)
        elif ".attack_" in mv:
            out["attack_picks"] += 1
        elif ".join_group_" in mv:
            out["join_picks"] += 1
    if not jevs:
        return out
    by_frame = collections.defaultdict(list)
    for s in out["sends"]:
        by_frame[s["f"]].append(s)
    last_chasing = {}
    seen_ends = set()
    chase_events = []
    src = jevs[0]
    proc = subprocess.Popen(["zstd", "-dc", src], stdout=subprocess.PIPE, text=True) if src.endswith(".zst") else None
    stream = proc.stdout if proc else open(src)
    for l in stream:
        if '"groups"' not in l and '"events"' not in l and "hunt of" not in l:
            continue
        try:
            r = json.loads(l)
        except ValueError:
            continue
        f = r.get("f")
        for e in r.get("events") or []:
            m = re.match(r"group_(\w+) chase meets (.+)", e)
            if m:
                chase_events.append((f, "group_" + m.group(1), m.group(2)))
        # A hunt's end is said in the state for a while: each one counted once.
        for m in re.finditer(r"(group_\w+)'s hunt of (party_\d+) ended after (\d+) s: ([^;\"]+)", l):
            key = (m.group(1), m.group(2), m.group(3), m.group(4))
            if key in seen_ends:
                continue
            seen_ends.add(key)
            why = m.group(4)
            out["hunt_ends"]["dead" if "dead" in why else "lost" if "out of sight" in why else "leash" if "leash" in why else "other"] += 1
            out["hunt_seconds"].append(int(m.group(3)))
        groups = r.get("groups") or []
        if groups and f is not None:
            # The rows list the groups asked, not every group: a chasing group's members are compared with its own
            # next appearance, and a member gone from it and from every group listed then is counted lost.
            everyone = set()
            for g in groups:
                everyone.update(g.get("members") or [])
            for g in groups:
                name = "group_" + g["name"]
                members = set(g.get("members") or [])
                if name in last_chasing:
                    kind, before = last_chasing.pop(name)
                    out["lost_hunting" if kind == "hunt" else "lost_engaging"] += sum(1 for u in before if u not in members and u not in everyone)
                kind = (g.get("task") or {}).get("kind")
                if kind in ("hunt", "engage", "attack_unit"):
                    last_chasing[name] = (kind, members)
            for s in by_frame.get(f, []):
                for g in groups:
                    if "group_" + g["name"] == s["actor"]:
                        s["size"] = len(g.get("members") or [])
        if f in by_frame and r.get("menus"):
            for m in r["menus"]:
                for mo in m.get("moves") or []:
                    for s in by_frame[f]:
                        if mo.get("id") == f"{s['actor']}.send_{s['n']}_{s['party']}" and not s["words"]:
                            s["words"] = mo.get("words", "")
    if proc:
        proc.wait()
    for f, actor, what in chase_events:
        after = [(pf, mv) for pf, mv, _ in picks if mv.startswith(actor + ".") and f <= pf <= f + WINDOW]
        pick = after[0][1].split(".", 1)[1] if after else None
        out["chase_asks"].append({"f": f, "actor": actor, "what": what, "pick": pick, "withdrew": bool(pick) and pick.startswith(WITHDRAW)})
    return out


def summarise(o):
    sends = o["sends"]
    sized = [s for s in sends if s["size"]]
    half = sum(1 for s in sized if s["n"] * 2 >= s["size"])
    kept = sum(1 for s in sends if "under its name" in s["did"])
    outrun = sum(1 for s in sends if "outruns them" in s["words"])
    told = sum(1 for s in sends if "already on it" in s["words"])
    ends = o["hunt_ends"]
    secs = sorted(o["hunt_seconds"])
    asks = o["chase_asks"]
    return {
        "match": o["match"], "sends": len(sends), "by_size": dict(collections.Counter(s["n"] for s in sends)), "half_or_more": half, "sized": len(sized), "kept_name": kept,
        "repeats": sum(1 for s in sends if s["repeat"]), "chained": sum(1 for s in sends if s["chained"]), "told_already_on_it": told, "quarry_outruns": outrun,
        "attack_picks": o["attack_picks"], "join_picks": o["join_picks"],
        "hunts": sum(ends.values()), "ended": dict(ends), "hunt_median_s": secs[len(secs) // 2] if secs else None, "lost_hunting": o["lost_hunting"], "lost_engaging": o["lost_engaging"],
        "chase_asks": len(asks), "withdrew": sum(1 for a in asks if a["withdrew"]), "kept_on": sum(1 for a in asks if not a["withdrew"]),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("matches", nargs="+")
    ap.add_argument("--json")
    ap.add_argument("--quiet", action="store_true")
    a = ap.parse_args()
    rows = []
    for path in a.matches:
        for m in matches_of(path):
            o = read(m)
            s = summarise(o)
            rows.append({"summary": s, "sends": o["sends"], "chase_asks": o["chase_asks"]})
            if not a.quiet:
                for x in o["sends"]:
                    print(f"  [{clock(x['f'])}] {x['actor']} sent {x['n']}{' of ' + str(x['size']) if x['size'] else ''} at {x['party']}{' (repeat)' if x['repeat'] else ''}{' (by a detachment)' if x['chained'] else ''}{' (kept the name)' if 'under its name' in x['did'] else ''}{' (told: already on it)' if 'already on it' in x['words'] else ''}{' (outruns them)' if 'outruns them' in x['words'] else ''}")
                for x in o["chase_asks"]:
                    print(f"  [{clock(x['f'])}] {x['actor']} chase meets {x['what']}: {'withdrew: ' + x['pick'] if x['withdrew'] else ('kept on: ' + x['pick'] if x['pick'] else 'kept on (no pick)')}")
            by_size = " ".join(f"{k}:{v}" for k, v in sorted(s["by_size"].items()))
            print(f"{m}: sends {s['sends']} (sizes {by_size}; at least half the group {s['half_or_more']} of {s['sized']}, kept the name {s['kept_name']}; repeats within 60 s {s['repeats']}, by a detachment {s['chained']}, told already on it {s['told_already_on_it']}, quarry outruns {s['quarry_outruns']}); whole-group attacks {s['attack_picks']}, joins {s['join_picks']}; hunts {s['hunts']} ended {s['ended']} median {s['hunt_median_s']} s; soldiers lost while hunting {s['lost_hunting']}, while attacking a party {s['lost_engaging']}; chase asks {s['chase_asks']}: withdrew {s['withdrew']}, kept on {s['kept_on']}")
    if a.json:
        json.dump(rows, open(a.json, "w"), indent=1)
    return 0


if __name__ == "__main__":
    sys.exit(main())
