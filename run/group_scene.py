#!/usr/bin/env python3
"""Cut one of our groups on the march into a duel scenario: its members where they stood, his buildings near the
place it was ordered to, and the order itself, so the engine walks the same army to the same place again
(`target/release/duel --scenario FILE --lane on`, with `WITHIN_REASON_GROUP_DEBUG=1` for the lane's own account).

    run/group_scene.py <match dir> <seat> <group> <M:SS> <X,Z> [--radius 1700] [--cap 24] --out FILE

`run/engagement.py` cuts a fight: every armed unit within a radius, the other side as the party our side attacks.
This cuts a march: only the group's members (the record's `groups` sample: formed and exceptions), given the hands'
advance to the place with no party (the scenario's `advance`, `crates/arena/src/bin/duel/scenario.rs`), in companies
of `--cap`. His side is his buildings within `--radius` of the place from the truth file (`truth-<seat>.jsonl`, the
row at or before the clock), seen or not; none of his mobile units, no wrecks, nothing of our allies.
"""
import argparse, json, math, os


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("seat")
    ap.add_argument("group")
    ap.add_argument("clock")
    ap.add_argument("place")
    ap.add_argument("--radius", type=float, default=1700.0)
    ap.add_argument("--cap", type=int, default=24)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    second = int(args.clock.split(":")[0]) * 60 + int(args.clock.split(":")[1])
    place = [float(v) for v in args.place.split(",")]
    header, before, now = None, None, None
    for line in open(os.path.join(args.dir, f"record-{args.seat}.jsonl")):
        if header is None:
            header = json.loads(line)
        elif f'"t":"s","f":{(second - 1) * 30},' in line:
            before = json.loads(line)
        elif f'"t":"s","f":{second * 30},' in line:
            now = json.loads(line)
            break
    defs = header["unit_defs"]
    g = now["groups"][args.group]
    members = set(g["formed"]) | {e[0] for e in g["exceptions"]}
    was = {u[0]: (u[2], u[3]) for u in before["own"]} if before else {}
    ours = []
    for u in now["own"]:
        if u[0] not in members:
            continue
        unit = {"type": defs[u[1]]["name"], "x": u[2], "z": u[3], "health": max(0.05, u[4] / 100)}
        if u[0] in was:
            dx, dz = u[2] - was[u[0]][0], u[3] - was[u[0]][1]
            if math.hypot(dx, dz) >= 8:
                unit["heading"] = [round(dx / math.hypot(dx, dz), 2), round(dz / math.hypot(dx, dz), 2)]
        ours.append(unit)
    speed = {d["name"]: d["speed"] for d in defs}
    row = None
    for line in open(os.path.join(args.dir, f"truth-{args.seat}.jsonl")):
        r = json.loads(line)
        if r["f"] > second * 30:
            break
        row = r
    theirs = [
        {"type": e[1], "x": e[2], "z": e[3], "health": max(0.05, (e[4] if e[4] >= 0 else 100) / 100)}
        for e in row["enemy"]
        if speed.get(e[1], 1) <= 0 and math.hypot(e[2] - place[0], e[3] - place[1]) <= args.radius
    ]
    scenario = {
        "format": "within-reason-scenario",
        "version": 1,
        "map": header["map"]["name"],
        "map_size": [header["map"]["width"], header["map"]["height"]],
        "centre": place,
        "source": {"match": args.dir, "seat": args.seat, "group": args.group, "clock": args.clock, "cut": "run/group_scene.py: the group's members, his buildings from the truth file"},
        "sides": [{"name": "ours", "units": ours}, {"name": "theirs", "units": theirs}],
        "advance": {"to": place, "cap": args.cap},
    }
    json.dump(scenario, open(args.out, "w"), indent=1)
    print(f"{args.out}: {len(ours)} of ours, {len(theirs)} buildings of his; the group's companies then: " + ", ".join(f"{c['members']} at ({c['anchor'][0]:.0f},{c['anchor'][1]:.0f})" for c in g["companies"]))


if __name__ == "__main__":
    main()
