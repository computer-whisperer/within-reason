#!/usr/bin/env python3
"""Every structure of ours a match started, and what became of each frame: finished, killed while unfinished, decayed
(destroyed by nobody after its builder left), or still unfinished when the record ended; beside it what the builder
was told next and when, the frame's progress and the stores while it stood, and who helped. The reviewer's table for
the unfinished-structures sweep (the user, 2026-10-17: "tallying up the causes for unfinished structures in recent
games"): the causes are the reviewer's reading; this lists the frames and the facts the same way for every game.

usage: run/frames.py run/matches/<batch>/<NN> [--seat N] [--all] [--json]
  --seat N   read record-N.jsonl (default 0); the seats of a match are its record-*.jsonl files
  --all      print the finished frames too (by default only the unfinished ones, with the finished counted)
  --json     the table as JSON (the sweep's agents write their reading beside it)

A `destroyed` event with no attacker is read as decayed; a frame that took damage in its last 15 s (the record's `dmg`
rows) is said to have been shot down by an attacker the event did not name (rove-task-2: two of five "decayed" frames
were killed by raiders in sight). The counts keep the record's fates; the words say the difference.

Columns: id, type, cell, started m:ss (by whom), fate (finished +m:ss | killed m:ss by <type> at N% | decayed m:ss at
N% | unfinished at end N%), metal put in (share x cost), progress (first metal at +s; last progress at m:ss; stalled s
= seconds from the last progress to the fate or the builder leaving), stores (the lowest metal and energy store share
while the frame stood), builder: what it was told after the frame began and when it first went elsewhere (the record's
`cmd` rows), its death if it died first, and the hands' decision rows for it in that time (`d` rows: list, plan,
heuristic), helpers (repair/guard commands at the frame). Then the build orders never started: a `build` command with
no `created` of that type by that builder within 300 of the site in the next 90 s, with the event that followed.
"""
import argparse
import json
import os
import sys

UNSTARTED_WINDOW = 90 * 30
SITE_RADIUS = 300.0
# The engine places a frame up to a building's width or more from the ordered point (600 seen): without a `placed`
# event (records before 2026-10-08) a build order is matched to its frame this far from the commanded point.
PLACED_RADIUS = 700.0
DAMAGE_WINDOW = 15 * 30  # a `destroyed` event with no attacker, after damage in these seconds, is a kill the record did not attribute


def clock(frame):
    seconds = int(frame) // 30
    return f"{seconds // 60}:{seconds % 60:02d}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("match")
    ap.add_argument("--seat", type=int, default=0)
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    match = a.match
    if not os.path.exists(os.path.join(match, f"record-{a.seat}.jsonl")) and os.path.isdir(os.path.join(match, "00")):
        match = os.path.join(match, "00")
    path = os.path.join(match, f"record-{a.seat}.jsonl")
    if not os.path.exists(path):
        sys.exit(f"no {path}")

    header = None
    defs = {}
    frames = []  # every frame, in order of creation
    current = {}  # frame id -> the entry standing now (the engine reuses ids)
    builds = []  # build commands: (f, builder, def, x, z)
    cmds = {}  # builder id -> [(f, cmd)]
    decisions = {}  # actor name -> [(f, source, did)]
    events = []
    placed = []  # (f, builder, def, x, z): the site the engine chose for an order
    deaths = {}  # unit id -> [(f, killer type or None, by)] (ids are reused: a list)
    last_f = 0
    stores = []  # (f, metal share, energy share)
    damage = {}  # unit id -> [(f, damage)] (the record's `dmg` rows; a frame's last hit is sampled after its destroyed event)
    commander_name = None
    own_actor = {}

    def actor_name(unit):
        d = defs.get(own_actor.get(unit))
        if d is None:
            return f"constructor_{unit}"
        name = d["name"]
        if name.endswith("com"):
            return "commander"
        if d.get("builds") and d.get("speed", 0) <= 0:
            return f"plant_{unit}" if name.endswith("vp") else (f"lab_{unit}" if "lab" in name else f"factory_{unit}")
        return f"constructor_{unit}"

    with open(path) as fh:
        for line in fh:
            try:
                r = json.loads(line)
            except ValueError:
                continue
            if not isinstance(r, dict):
                continue
            t = r.get("t")
            if t == "header":
                header = r
                defs = dict(enumerate(r["unit_defs"]))
                continue
            f = r.get("f", 0)
            last_f = max(last_f, f)
            if t == "s":
                m, e = r.get("m") or [0, 0, 0, 0], r.get("e") or [0, 0, 0, 0]
                stores.append((f, m[0] / m[3] if m[3] else 0.0, e[0] / e[3] if e[3] else 0.0))
                for uid, dmg in r.get("dmg") or []:
                    damage.setdefault(uid, []).append((f, dmg))
                for u in r.get("own", []):
                    own_actor[u[0]] = u[1]
                    fr = current.get(u[0])
                    if fr is not None and fr["fate"] is None:
                        share = u[4] / 100.0
                        if share > fr["share"] + 1e-6:
                            if fr["share"] <= 1e-6 and share > 1e-6 and fr["first_metal"] is None:
                                fr["first_metal"] = f
                            fr["last_progress"] = f
                        fr["share"] = max(fr["share"], share)
                        fr["last_seen"] = f
            elif t == "ev":
                k = r.get("k")
                d = defs.get(r.get("d"), {})
                u = r.get("u")
                if k == "created":
                    by = r.get("by")
                    actor = actor_name(by) if by is not None else None
                    own_actor[u] = r.get("d")
                    if d.get("speed", 0) <= 0 and d.get("metal"):
                        fr = {"id": u, "def": r.get("d"), "type": d["name"], "cost": d["metal"], "x": r.get("x"), "z": r.get("z"), "started": f, "by": by, "by_actor": actor, "fate": None, "fate_f": None, "share": 0.0, "first_metal": None, "last_progress": None, "last_seen": f, "killer": None, "helpers": []}
                        frames.append(fr)
                        current[u] = fr
                elif k == "finished":
                    fr = current.pop(u, None)
                    if fr is not None:
                        if f == fr["started"]:
                            frames.remove(fr)  # given at the start, not built
                        else:
                            fr["fate"], fr["fate_f"], fr["share"] = "finished", f, 1.0
                elif k == "destroyed":
                    deaths.setdefault(u, []).append((f, defs.get(r.get("by_d"), {}).get("name") if r.get("by") is not None else None, r.get("by")))
                    fr = current.pop(u, None)
                    if fr is not None:
                        fr["fate_f"] = f
                        if r.get("by") is None:
                            fr["fate"] = "decayed"
                        else:
                            fr["fate"] = "killed"
                            fr["killer"] = defs.get(r.get("by_d"), {}).get("name", "?")
                elif k in ("no_site", "rejected", "placed", "move_failed"):
                    events.append((f, k, u, r.get("what"), r.get("code"), r.get("x"), r.get("z")))
                    if k == "placed":
                        placed.append((f, u, r.get("what"), r.get("x"), r.get("z")))
            elif t == "cmd":
                for c in r.get("c", []):
                    if not c or c[0] in ("say", "priority", "movestate", "firestate", "send_resources", "send_units", "stockpile"):
                        continue
                    unit = c[1]
                    cmds.setdefault(unit, []).append((f, c))
                    if c[0] == "build" and len(c) >= 5:
                        builds.append((f, unit, c[2], c[3], c[4]))
                    elif c[0] in ("repair", "guard") and len(c) >= 3:
                        fr = current.get(c[2])
                        if fr is not None:
                            fr["helpers"].append((f, c[0], actor_name(unit)))
            elif t == "d" and isinstance(r.get("inputs"), dict) and r["inputs"].get("actor"):
                did = (r.get("outputs") or {}).get("did")
                if did:
                    decisions.setdefault(r["inputs"]["actor"], []).append((f, r.get("source"), did))

    if header is None:
        sys.exit("no header")
    width, height = header["map"]["width"], header["map"]["height"]

    def cell(x, z):
        cx = min(max(int(x / width * 8), 0), 7)
        cz = min(max(int(z / height * 8), 0), 7)
        return f"{chr(65 + cx)}{cz + 1}"

    def store_min(f0, f1):
        ms = [s for s in stores if f0 <= s[0] <= f1]
        if not ms:
            return None, None
        return min(s[1] for s in ms), min(s[2] for s in ms)

    def same_site(c, fr):
        return c[0] == "build" and len(c) >= 5 and c[2] == fr["def"] and ((c[3] - fr["x"]) ** 2 + (c[4] - fr["z"]) ** 2) ** 0.5 < SITE_RADIUS

    def on_frame(c, fr):
        return c[0] in ("repair", "guard") and len(c) >= 3 and c[2] == fr["id"]

    rows = []
    for fr in frames:
        fate_f = fr["fate_f"] if fr["fate_f"] is not None else last_f
        # A decayed frame took no damage; one shot down by a shooter the event does not name did. The sample that
        # carries the last hit comes after the destroyed event, so the window runs two samples past it.
        fr["damage_last"] = round(sum(d for df, d in damage.get(fr["id"], []) if max(fr["started"], fate_f - DAMAGE_WINDOW) <= df <= fate_f + 2 * 30))
        if fr["fate"] is None:
            fr["fate"] = "unfinished at end"
        builder = fr["by"]
        left = None
        told = []
        if builder is not None:
            for f, c in cmds.get(builder, []):
                if f <= fr["started"] or f > fate_f:
                    continue
                if same_site(c, fr) or on_frame(c, fr):
                    continue
                told.append((f, c))
                if left is None:
                    left = f
        builder_death = next((x for x in deaths.get(builder, []) if fr["started"] <= x[0] <= fate_f), None) if builder is not None else None
        actor = fr["by_actor"]
        dec = [(f, s, did) for f, s, did in decisions.get(actor, [])] if actor else []
        dec = [x for x in dec if fr["started"] - 30 <= x[0] <= fate_f]
        end_of_work = min(x for x in [left, fate_f, builder_death[0] if builder_death else None] if x is not None)
        stalled = max(0.0, (end_of_work - (fr["last_progress"] or fr["started"])) / 30.0)
        mmin, emin = store_min(fr["started"], end_of_work)
        rows.append({
            "seat": a.seat, "id": fr["id"], "type": fr["type"], "cell": cell(fr["x"], fr["z"]), "x": fr["x"], "z": fr["z"],
            "started": clock(fr["started"]), "started_f": fr["started"], "builder": actor, "builder_id": builder,
            "fate": fr["fate"], "fate_clock": clock(fate_f), "fate_f": fate_f, "killer": fr["killer"], "damage_before_end": fr.get("damage_last", 0),
            "share": round(fr["share"], 3), "cost": fr["cost"], "metal_in": round(fr["share"] * fr["cost"]),
            "seconds": round((fate_f - fr["started"]) / 30.0),
            "first_metal_after_s": None if fr["first_metal"] is None else round((fr["first_metal"] - fr["started"]) / 30.0),
            "last_progress": None if fr["last_progress"] is None else clock(fr["last_progress"]),
            "stalled_s": round(stalled), "metal_store_min": None if mmin is None else round(mmin, 2), "energy_store_min": None if emin is None else round(emin, 2),
            "builder_left": None if left is None else clock(left),
            "builder_told": [(clock(f), c) for f, c in told[:6]],
            "builder_died": None if not builder_death else (clock(builder_death[0]), builder_death[1] or "nobody"),
            "decisions": [(clock(f), s, did) for f, s, did in dec[:8]],
            "helpers": [(clock(f), k, who) for f, k, who in fr["helpers"][:6]],
        })

    # Build orders never started: a build command with no created of that type by that builder at the site the engine
    # chose for it (the `placed` event within 2 s of the order; the commanded point, wider, without one) in 90 s.
    created_by = {}
    for fr in frames:
        created_by.setdefault(fr["by"], []).append(fr)
    unstarted = []
    for f, builder, d, x, z in builds:
        dd = defs.get(d, {})
        if dd.get("speed", 0) > 0 or not dd.get("metal"):
            continue
        site = next(((px, pz) for pf, pu, pd, px, pz in placed if pu == builder and pd == d and f <= pf <= f + 60), None)
        (sx, sz), radius = (site, SITE_RADIUS) if site else ((x, z), PLACED_RADIUS)
        hit = any(fr["def"] == d and f <= fr["started"] <= f + UNSTARTED_WINDOW and ((fr["x"] - sx) ** 2 + (fr["z"] - sz) ** 2) ** 0.5 < radius for fr in created_by.get(builder, []))
        if hit:
            continue
        # The same order re-sent within the window counts once: keep the first.
        if unstarted and unstarted[-1]["builder_id"] == builder and unstarted[-1]["def"] == d and ((unstarted[-1]["x"] - x) ** 2 + (unstarted[-1]["z"] - z) ** 2) ** 0.5 < SITE_RADIUS and f - unstarted[-1]["f"] <= UNSTARTED_WINDOW:
            unstarted[-1]["resent"] += 1
            continue
        after = [(clock(ef), k, defs.get(what, {}).get("name") if what is not None else None, code) for ef, k, eu, what, code, ex, ez in events if eu == builder and f <= ef <= f + UNSTARTED_WINDOW][:4]
        nxt = next(((clock(cf), c) for cf, c in cmds.get(builder, []) if cf > f), None)
        death = next((x for x in deaths.get(builder, []) if x[0] >= f), None)
        unstarted.append({"seat": a.seat, "f": f, "clock": clock(f), "builder": actor_name(builder), "builder_id": builder, "def": d, "type": dd.get("name", "?"), "cell": cell(x, z), "x": x, "z": z, "resent": 0, "events_after": after, "next_command": nxt, "builder_died": None if not death else clock(death[0])})

    tally = {}
    for r in rows:
        tally[r["fate"]] = tally.get(r["fate"], 0) + 1
    metal_lost = sum(r["metal_in"] for r in rows if r["fate"] != "finished")
    out = {"match": os.path.basename(os.path.dirname(match.rstrip("/"))) or match, "seat": a.seat, "last_clock": clock(last_f), "tally": tally, "metal_in_unfinished": metal_lost, "frames": rows, "unstarted": unstarted}
    if a.json:
        print(json.dumps(out, indent=1))
        return

    print(f"{out['match']} seat {a.seat}, record to {out['last_clock']}: frames {len(rows)} ({', '.join(f'{k} {v}' for k, v in sorted(tally.items()))}), metal in unfinished frames {metal_lost}, build orders never started {len(unstarted)}")
    for r in rows:
        if r["fate"] == "finished" and not a.all:
            continue
        fate = {"finished": f"finished {r['fate_clock']} (+{r['seconds']} s)", "killed": f"killed {r['fate_clock']} by {r['killer']} at {r['share'] * 100:.0f}%", "decayed": f"decayed {r['fate_clock']} at {r['share'] * 100:.0f}%", "unfinished at end": f"unfinished at end ({r['fate_clock']}) at {r['share'] * 100:.0f}%"}[r["fate"]]
        if r["fate"] == "decayed" and r["damage_before_end"] > 0:
            fate += f" (took {r['damage_before_end']} damage in its last {DAMAGE_WINDOW // 30} s: shot down by an attacker the record did not name, most likely)"
        print(f"\n{r['type']}_{r['id']} at {r['cell']} ({r['x']:.0f},{r['z']:.0f}): started {r['started']} by {r['builder']}; {fate}; metal in {r['metal_in']} of {r['cost']:.0f}")
        prog = "no metal ever put in" if r["first_metal_after_s"] is None else f"first metal after {r['first_metal_after_s']} s, last progress {r['last_progress']}"
        print(f"  progress: {prog}; stalled {r['stalled_s']} s before the builder left or the fate; stores min metal {r['metal_store_min']} energy {r['energy_store_min']}")
        if r["builder_left"]:
            told = "; ".join(f"{c0} {' '.join(str(x) for x in c)}" for c0, c in r["builder_told"])
            print(f"  builder left {r['builder_left']}: {told}")
        elif r["builder_died"]:
            print(f"  builder died {r['builder_died'][0]} to {r['builder_died'][1]}")
        else:
            print("  builder was told nothing else while the frame stood")
        if r["builder_died"] and r["builder_left"]:
            print(f"  builder died {r['builder_died'][0]} to {r['builder_died'][1]}")
        for c0, s, did in r["decisions"]:
            print(f"  {c0} ({s}) {did}")
        for c0, k, who in r["helpers"]:
            print(f"  helped by {who} ({k}) from {c0}")
    if unstarted:
        print(f"\nbuild orders never started ({len(unstarted)}):")
        for u in unstarted:
            after = "; ".join(f"{c0} {k}{' ' + w if w else ''}{' code ' + str(code) if code is not None else ''}" for c0, k, w, code in u["events_after"]) or "no site/rejected event"
            nxt = f"next command {u['next_command'][0]} {' '.join(str(x) for x in u['next_command'][1])}" if u["next_command"] else "no later command"
            died = f"; builder died {u['builder_died']}" if u["builder_died"] else ""
            resent = f" (re-sent {u['resent']}x)" if u["resent"] else ""
            print(f"  {u['clock']} {u['builder']}: {u['type']} at {u['cell']} ({u['x']:.0f},{u['z']:.0f}){resent}: {after}; {nxt}{died}")


if __name__ == "__main__":
    main()
