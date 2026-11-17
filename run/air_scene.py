#!/usr/bin/env python3
"""Cut a bombing run out of a game's tape into a duel scenario and its script, to fly the run again against a
defence we choose (`docs/design/2026-11-16-air-intercept.md`, `docs/harness/duels.md`).

    run/air_scene.py <match dir | audit.jsonl> --frame F --target X,Z --bombers corhurc
        [--ours 3,4,5] [--block 1200] [--anti-air 3000] [--near-path 1200] [--prelude X,Z[;X,Z...]]
        [--fighters armfig*12@X,Z] [--add armarad@X,Z ...] [--arm none|hold|idle|patrol|fight|spread|patrol_fight|patrol_spread]
        [--state 0|1|2] [--delay 0.5] [--loop 900] [--toward X,Z] [--lead 12]
        --out PREFIX            writes PREFIX.json (the scenario) and PREFIX.script.json

Side 1 is the bombers of the type named that were given an order from frame F on, where they were at F, and the
script's rows for it are their player's own orders from F to the attack (a move, a queued move, an attack at the
ground), `--lead` seconds late so that our fighters are up first. The bombers are given standing on the ground and
take off at their first order, and a Hailstorm takes fifteen seconds to its full speed: cut from a frame well before
the first contact (where they waited, or far out on their way), so that they come as they came. `--prelude` is the
order they were under at the frame, when it was given before it: moves given at the start, the first replacing, the
rest queued.

Side 0 is ours as it stood at F, finished units only: everything within `--block` of the target (what the bombs and
our own reactors' blasts can kill), every anti-air unit within `--anti-air` of it, every radar, and everything
within `--near-path` of where the bombers then flew (what saw them come). None of our aircraft, no commander. Then
what the drill adds: `--fighters` (given landed in ranks of six at the place) and `--add` (a building more, a radar).

`--arm` writes side 0's rows, all for the fighters added:
  none    no row: the anti-air as it stood
  hold    landed on hold position, as the hands hold an air group today; `--delay` after first contact all attack
          the first bomber, the others queued behind it in order
  idle    landed with move state `--state` (1 manoeuvre, 2 roam) and no order: the engine's own search
  patrol  move state `--state` and the engine's patrol round the target, four corners `--loop` out
  fight   landed on hold position; `--delay` after first contact move state `--state` and a fight order to `--toward`
  spread  landed on hold position; `--delay` after first contact each fighter attacks a bomber of its own (fighter k
          bomber k mod n), the others queued behind in turn
  patrol_fight, patrol_spread
          up on the patrol under manoeuvre until first contact, then as fight or spread
"""
import argparse, csv, json, math, os, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MOVE, ATTACK = 10, 20


def read(source, first, last):
    """The dump's header, definitions, the samples and the orders between two frames."""
    if os.path.isdir(source):
        tape = os.path.join(source, "game.tape") if os.path.exists(os.path.join(source, "game.tape")) else os.path.join(source, "00", "game.tape")
        stream = subprocess.Popen([os.path.join(REPO, "target/release/tape"), "audit", tape, "-o", "-"], stdout=subprocess.PIPE, text=True).stdout
    else:
        stream = open(source)
    header, defs, samples, orders = None, {}, [], []
    for line in stream:
        if '"t":"def"' in line:
            row = json.loads(line)
            defs[row["d"]["id"]] = row["d"]
            continue
        if header is None and '"seats"' in line:
            header = json.loads(line)
            continue
        # The frame without parsing the row (the keys are in order, `f` before any nested one): most rows are
        # outside the window.
        at = line.find('"f":')
        if at < 0:
            continue
        end = at + 4
        while line[end].isdigit():
            end += 1
        frame = int(line[at + 4 : end])
        if frame < first - 30 or frame > last:
            continue
        row = json.loads(line)
        if row.get("t") == "s":
            samples.append(row)
        elif row.get("t") == "cmd":
            orders.append(row)
    return header, defs, samples, orders


def anti_air_names():
    """The types the glossary sheet classes as anti-air."""
    with open(os.path.join(REPO, "docs/glossary/units.csv")) as f:
        return {r["unit"] for r in csv.DictReader(f) if "anti-air" in r["class"]}


def place(text):
    x, z = text.split(",")
    return float(x), float(z)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("source")
    ap.add_argument("--frame", type=int, required=True)
    ap.add_argument("--prelude")
    ap.add_argument("--target", type=place, required=True)
    ap.add_argument("--bombers", required=True)
    ap.add_argument("--ours", default="3,4,5")
    ap.add_argument("--block", type=float, default=1200.0)
    ap.add_argument("--anti-air", type=float, default=3000.0)
    ap.add_argument("--near-path", type=float, default=1200.0)
    ap.add_argument("--fighters")
    ap.add_argument("--add", action="append", default=[])
    ap.add_argument("--arm", default="none", choices=["none", "hold", "idle", "patrol", "fight", "spread", "patrol_fight", "patrol_spread"])
    ap.add_argument("--state", type=int, default=2)
    ap.add_argument("--delay", type=float, default=0.5)
    ap.add_argument("--loop", type=float, default=900.0)
    ap.add_argument("--toward", type=place)
    ap.add_argument("--lead", type=float, default=12.0)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    ours_teams = {int(t) for t in args.ours.split(",")}
    last = args.frame + 120 * 30
    header, defs, samples, orders = read(args.source, args.frame, last)
    name = lambda d: defs[d]["name"]
    start = max((s for s in samples if s["f"] <= args.frame), key=lambda s: s["f"])
    tx, tz = args.target

    # His bombers: of the type, not ours, and given the attack at the ground that ends the cut (the first one given
    # from the frame on; a bomber made meanwhile and sent to its plant's rally is not on the run).
    theirs = {u[0] for u in start["u"] if name(u[1]) == args.bombers and u[2] not in ours_teams and u[6] >= 100}
    attack = next((o["f"] for o in orders if o["u"] in theirs and o["f"] >= args.frame and o["c"] == ATTACK and "x" in o and not o.get("shift")), None)
    if attack is None:
        sys.exit(f"no {args.bombers} of theirs was sent at the ground from frame {args.frame}")
    on_the_run = {o["u"] for o in orders if o["u"] in theirs and o["f"] == attack and o["c"] == ATTACK}
    bombers = [u for u in start["u"] if u[0] in on_the_run]
    index = {u[0]: i for i, u in enumerate(bombers)}
    mine = [o for o in orders if o["u"] in index and args.frame <= o["f"] <= attack and o["c"] in (MOVE, ATTACK) and "x" in o]
    rows = []
    for k, at in enumerate(args.prelude.split(";") if args.prelude else []):
        x, z = place(at)
        rows.append({"t": args.lead, "side": 1, "unit": list(range(len(bombers))), "cmd": "move", "x": x, "z": z, "queue": k > 0})
    for o in mine:
        cmd = "move" if o["c"] == MOVE else "attack_ground"
        t = round(args.lead + (o["f"] - args.frame) / 30, 2)
        same = rows and all(rows[-1][k] == v for k, v in (("t", t), ("cmd", cmd), ("x", o["x"]), ("z", o["z"]), ("queue", bool(o.get("shift")))))
        if same:
            rows[-1]["unit"].append(index[o["u"]])
        else:
            rows.append({"t": t, "side": 1, "unit": [index[o["u"]]], "cmd": cmd, "x": o["x"], "z": o["z"], "queue": bool(o.get("shift"))})
    # Where they then flew: what of ours stood near it saw them come.
    track = [(u[3], u[4]) for s in samples if args.frame <= s["f"] <= attack + 300 for u in s["u"] if u[0] in index]

    anti_air = anti_air_names()
    ours, why, frames = [], {}, []
    for u in start["u"]:
        d = defs[u[1]]
        # Not our aircraft (they sat landed), and not a commander: one dying in a drill is the game's business.
        if u[2] not in ours_teams or d["air"] or d["name"].endswith("com"):
            continue
        reach = math.hypot(u[3] - tx, u[4] - tz)
        reason = (
            "block" if reach <= args.block
            else "anti-air" if d["name"] in anti_air and reach <= args.anti_air
            else "radar" if d["building"] and d["radar"] > 0
            else "near the path" if min(math.hypot(u[3] - x, u[4] - z) for x, z in track) <= args.near_path
            else None
        )
        if reason is None:
            continue
        if u[6] < 100:
            frames.append(f"{d['name']} {u[6]}% at {u[3]},{u[4]}")
            continue
        ours.append({"type": d["name"], "x": u[3], "z": u[4], "source_id": u[0]})
        why[reason] = why.get(reason, 0) + 1
    for extra in args.add:
        kind, at = extra.split("@")
        x, z = place(at)
        ours.append({"type": kind, "x": x, "z": z, "added": True})
    fighters = []
    if args.fighters:
        force, at = args.fighters.split("@")
        kind, count = force.split("*")
        fx, fz = place(at)
        for k in range(int(count)):
            fighters.append(len(ours))
            ours.append({"type": kind, "x": fx + 48 * (k % 6) - 120, "z": fz + 48 * (k // 6), "added": True})

    # Side 0's rows.
    every = fighters
    n = len(bombers)
    if args.arm != "none" and not fighters:
        sys.exit(f"--arm {args.arm} orders the fighters: give --fighters")
    def patrol(state):
        rows.append({"t": 0, "unit": every, "cmd": "movestate", "state": state})
        r = args.loop
        for k, (dx, dz) in enumerate([(-r, -r), (r, -r), (r, r), (-r, r)]):
            rows.append({"t": 0, "unit": every, "cmd": "patrol", "x": tx + dx, "z": tz + dz, "queue": k > 0})

    if args.arm == "hold":
        rows.append({"t": 0, "unit": every, "cmd": "movestate", "state": 0})
        rows += [{"t": args.delay, "from": "contact", "unit": every, "cmd": "attack", "target": b, "queue": b > 0} for b in range(n)]
    elif args.arm == "idle":
        rows.append({"t": 0, "unit": every, "cmd": "movestate", "state": args.state})
    elif args.arm == "patrol":
        patrol(args.state)
    elif args.arm in ("fight", "patrol_fight"):
        if args.toward is None:
            sys.exit(f"--arm {args.arm} needs --toward X,Z")
        if args.arm == "fight":
            rows.append({"t": 0, "unit": every, "cmd": "movestate", "state": 0})
        else:
            patrol(1)
        rows.append({"t": args.delay, "from": "contact", "unit": every, "cmd": "movestate", "state": args.state})
        rows.append({"t": args.delay, "from": "contact", "unit": every, "cmd": "fight", "x": args.toward[0], "z": args.toward[1]})
    elif args.arm in ("spread", "patrol_spread"):
        if args.arm == "spread":
            rows.append({"t": 0, "unit": every, "cmd": "movestate", "state": 0})
        else:
            patrol(1)
        for k, fighter in enumerate(fighters):
            rows += [{"t": args.delay, "from": "contact", "unit": fighter, "cmd": "attack", "target": (k + j) % n, "queue": j > 0} for j in range(n)]

    scenario = {
        "format": "within-reason-scenario", "version": 1,
        "map": header["map"]["name"], "map_size": [header["map"]["width"], header["map"]["height"]],
        "centre": [tx, tz],
        "source": {"tool": "run/air_scene.py", "from": os.path.abspath(args.source), "frame": args.frame, "attack_frame": attack, "arm": args.arm,
                   "fighters": fighters, "bombers": len(bombers), "lead": args.lead, "left_out_unfinished": frames},
        "sides": [
            {"name": "ours", "units": ours},
            {"name": "theirs", "units": [{"type": name(u[1]), "x": u[3], "z": u[4], "source_id": u[0]} for u in bombers]},
        ],
    }
    with open(args.out + ".json", "w") as f:
        json.dump(scenario, f)
    with open(args.out + ".script.json", "w") as f:
        # No contact counts before the bombers have flown eight seconds (`scenario.rs`, `contact_after`).
        json.dump({"format": "within-reason-script", "version": 1, "note": f"air_scene.py --arm {args.arm}", "contact_after": args.lead + 8, "rows": rows}, f)
    kinds = {}
    for u in ours:
        kinds[u["type"]] = kinds.get(u["type"], 0) + 1
    print(f"{args.out}.json: ours {len(ours)} ({', '.join(f'{k} {v}' for k, v in sorted(why.items()))}; added {len(args.add) + len(fighters)}), theirs {n} {args.bombers}")
    print("  ours by type:", ", ".join(f"{k} {v}" for k, v in sorted(kinds.items(), key=lambda kv: -kv[1])))
    print(f"  his orders: {sum(1 for r in rows if r.get('side') == 1)} rows from frame {args.frame} to {attack}; the attack at t = {round(args.lead + (attack - args.frame) / 30, 1) if attack else None} s")
    if frames:
        print("  left out, unfinished:", "; ".join(frames))


if __name__ == "__main__":
    main()
