#!/usr/bin/env python3
"""Every D-gun shot the hands fired, replayed against the menu's warning: the old line and the cone.

For each `dgun` command in a record: the gate (the last Jev pass before the command whose menu offered the shot),
both sides' positions at that frame (interpolated between the state samples 30 frames apart), the target the menu's
check used (the party's nearest member inside the reach) against the unit the command named, the old check (half a
footprint and 24 beside the line to the checked target, `ours_on_the_line` until 2026-10-24), the cone check now in
`crates/bot/src/brain/pianist/menu.rs` `ours_in_the_shot` (half a footprint, the unit's own travel in the delay and
the splash of 28 beside any ray the target can swing the line through in DELAY frames at its speed, the whole circle
when the swing exceeds its distance), and the truth: what of ours the shooter's disintegrator destroyed within 60
frames after the command.

    run/dgun_line.py [record.jsonl ...]        default: every uncompressed record under run/matches with a dgun command

Findings (2026-10-24, the eleven shots of the uncompressed records; the compressed older records hold none):
  the two shots that killed ours carried no warning under the old check at the gate (1792809574-human seat 1 at 7:21,
  a Blitz 239 away: the plant and the constructor stood 17 and 2 outside its band; 1792547311-human seat 1 at 14:52,
  a Raider 78 away circling: the Moho and the advanced solar ninety degrees off the checked line); the cone names all
  four victims, and of the nine clean shots warns on two: the second shot at the same circling Raider a second later
  (everything inside the reach again) and the second Boreal shot, where it names an extractor beside the commander
  that the shot missed. In one of eight shots with a Jev file the command's target was not the
  unit the menu had checked (1792809574-human seat 0 at 10:45: the nearest had changed). From the gate to the first
  kill: 33 and 39 frames (DELAY 36). Jev, re-asked both moments with the cone's words (jev-1.14.0, six times each):
  the shot's probability 0.59 to 0.33 at Boreal Falls, 0.66 to 0.11 on SD-129.
"""
import glob
import json
import math
import subprocess
import sys

DELAY = 36          # frames from the picture Jev saw to the shot's impact (menu.rs DGUN_DELAY)
SPLASH = 18 + 10    # the disintegrator's damage radius (areaofeffect 36 is a diameter) plus the fireball's radius 10
MARGIN_OLD = 24
PARTY = 400         # the picture's party radius (H-HANDS-PARTY-NAMES)
REACH = {"armcom": 250.0, "corcom": 262.0, "armcomcon": 250.0, "corcomcon": 262.0}


def lines(path):
    if path.endswith(".zst"):
        return subprocess.run(["zstd", "-dc", path], capture_output=True, check=True).stdout.decode().splitlines()
    return open(path).read().splitlines()


def interp(samples, f):
    """Own and enemy tables at frame f from the bracketing samples: {id: (x, z, vx, vz, def)}."""
    before = [s for s in samples if s["f"] <= f]
    after = [s for s in samples if s["f"] > f]
    s0 = before[-1] if before else after[0]
    s1 = after[0] if after else s0
    span = max(1, s1["f"] - s0["f"])

    def table(rows0, rows1):
        later = {r[0]: r for r in rows1}
        out = {}
        for r in rows0:
            r1 = later.get(r[0])
            if r1 is not None and s1 is not s0:
                vx, vz = (r1[2] - r[2]) / span, (r1[3] - r[3]) / span
                out[r[0]] = (r[2] + vx * (f - s0["f"]), r[3] + vz * (f - s0["f"]), vx, vz, r[1])
            else:
                out[r[0]] = (r[2], r[3], 0.0, 0.0, r[1])
        return out

    return table(s0["own"], s1["own"]), table(s0.get("en", []), s1.get("en", []))


def check_old(shooter, tgt, reach, own, UD):
    dx, dz = tgt[0] - shooter[0], tgt[1] - shooter[1]
    length = math.hypot(dx, dz)
    if length < 1:
        return []
    ux, uz = dx / length, dz / length
    hit = []
    for oid, o in own.items():
        if oid == shooter[2]:
            continue
        ox, oz = o[0] - shooter[0], o[1] - shooter[1]
        along = ox * ux + oz * uz
        half = max(UD[o[4]]["footprint"]) * 4.0
        if 0 < along < reach + half and abs(ox * uz - oz * ux) < half + MARGIN_OLD:
            hit.append((oid, UD[o[4]]["name"], round(along), round(abs(ox * uz - oz * ux))))
    return hit


def check_cone(shooter, tgt, speed, reach, own, UD):
    """The cone: a unit of ours is at risk when some ray the target can swing the line through passes within W of its
    centre (W: half a footprint, its own travel in the delay, the splash). Returns the hits, the swing, the half-angle."""
    dx, dz = tgt[0] - shooter[0], tgt[1] - shooter[1]
    d = math.hypot(dx, dz)
    if d < 1:
        return [], 0.0, 0.0
    swing = speed * DELAY / 30.0
    theta = math.pi if swing >= d else math.asin(swing / d)
    bearing = math.atan2(dz, dx)
    hit = []
    for oid, o in own.items():
        if oid == shooter[2]:
            continue
        ox, oz = o[0] - shooter[0], o[1] - shooter[1]
        rho = math.hypot(ox, oz)
        fp = max(UD[o[4]]["footprint"]) * 4.0
        width = fp + math.hypot(o[2], o[3]) * DELAY + SPLASH
        if rho < 1 or rho > reach + fp:
            continue
        off = abs((math.atan2(oz, ox) - bearing + math.pi) % (2 * math.pi) - math.pi)
        delta = max(0.0, off - theta)
        nearest = rho * math.sin(delta) if delta < math.pi / 2 else rho
        if nearest <= width:
            hit.append((oid, UD[o[4]]["name"], round(rho), round(math.degrees(off))))
    return hit, swing, math.degrees(theta)


def jev_path(record):
    seat = record.split("record")[-1].split(".")[0]  # "-1" or ""
    base = record[: record.rfind("/") + 1]
    for cand in (f"{base}jev{seat or '-0'}.jsonl", f"{base}jev{seat or '-0'}.jsonl.zst"):
        try:
            return lines(cand)
        except (OSError, subprocess.CalledProcessError):
            continue
    return []


def shots(path, tally):
    rows = [json.loads(l) for l in lines(path)]
    hdr, rows = rows[0], rows[1:]
    UD = hdr["unit_defs"]
    samples = [r for r in rows if r.get("t") == "s"]
    cmds = [(r["f"], c[1], c[2]) for r in rows if "c" in r for c in r["c"] if c[0] == "dgun"]
    if not cmds:
        return
    print("==", path)
    kills = [r for r in rows if r.get("k") == "destroyed" and str(r.get("w", "")).endswith("disintegrator")]
    jev = [json.loads(l) for l in jev_path(path)]
    for f, shooter_id, target_id in cmds:
        gate, words = None, ""
        for r in jev:
            if r.get("f") is None:
                continue
            if r["f"] > f:
                break
            for m in r.get("menus", []):
                for mv in m.get("moves", []):
                    if m.get("kind") == "builder" and mv["id"].split(".")[-1].startswith("dgun"):
                        gate, words = r["f"], mv["words"]
        g = gate if gate is not None else f
        own, en = interp(samples, g)
        sh = own.get(shooter_id)
        if sh is None:
            print(f"  f{f}: shooter {shooter_id} not in the samples")
            continue
        shooter = (sh[0], sh[1], shooter_id)
        reach = REACH.get(UD[sh[4]]["name"], 250.0)
        fired = en.get(target_id)
        party = {eid: e for eid, e in en.items() if fired is not None and math.hypot(e[0] - fired[0], e[1] - fired[1]) <= PARTY}
        inside = sorted((math.hypot(e[0] - sh[0], e[1] - sh[1]), eid) for eid, e in party.items() if math.hypot(e[0] - sh[0], e[1] - sh[1]) <= reach)
        checked_id = inside[0][1] if inside else target_id
        checked = en.get(checked_id)
        # A kill belongs to the latest command of this shooter at least 12 frames before it (the shot needs aim and
        # flight): one shot's two victims die up to 12 frames apart, and the next command follows within a second.
        later = [f2 for f2, s2, _ in cmds if s2 == shooter_id and f2 > f]
        truth = [(k["u"], UD[k["d"]]["name"]) for k in kills if k.get("by") == shooter_id and f + 12 <= k["f"] <= f + 60 and not any(f2 + 12 <= k["f"] for f2 in later)]

        def name(e):
            return UD[e[4]]["name"] if e is not None and isinstance(e[4], int) and 0 <= e[4] < len(UD) else "?"

        print(f"  f{f} ({f // 1800}:{f // 30 % 60:02d}) gate f{g} (+{f - g}) {UD[sh[4]]['name']} at ({sh[0]:.0f},{sh[1]:.0f}) reach {reach:.0f}")
        if fired is None:
            print(f"    fired at {target_id}: not in sight at the gate")
        else:
            print(f"    fired at {target_id} {name(fired)} at ({fired[0]:.0f},{fired[1]:.0f}), {math.hypot(fired[0] - sh[0], fired[1] - sh[1]):.0f} away, speed {UD[fired[4]]['speed'] if isinstance(fired[4], int) else '?'}")
        if checked is not None and checked_id != target_id:
            print(f"    THE MENU CHECKED {checked_id} {name(checked)} at ({checked[0]:.0f},{checked[1]:.0f}), {math.hypot(checked[0] - sh[0], checked[1] - sh[1]):.0f} away: the shot went at another unit")
        print(f"    inside the reach: {len(inside)} of the party's {len(party)}; the words: ...{words[words.find('inside the'):][:120] or '(no Jev file)'}")
        t = checked if checked is not None else fired
        if t is not None:
            speed = UD[t[4]]["speed"] if isinstance(t[4], int) else 101.0
            old = check_old(shooter, t, reach, own, UD)
            cone, swing, theta = check_cone(shooter, t, speed, reach, own, UD)
            print(f"    old line: {[(n, a, p) for _, n, a, p in old]}")
            print(f"    cone (swing {swing:.0f}, +-{theta:.0f} degrees): {[(n, r, dg) for _, n, r, dg in cone]}")
            killed = {u for u, _ in truth}
            tally.append((bool(truth), bool(old), bool(cone), all(u in {h[0] for h in old} for u in killed), all(u in {h[0] for h in cone} for u in killed)))
        print(f"    TRUTH: {truth}")


def main():
    paths = sys.argv[1:] or [p for p in sorted(glob.glob("run/matches/*/[0-9][0-9]/record*.jsonl")) if '["dgun"' in open(p).read()]
    tally = []
    for p in paths:
        shots(p, tally)
    if tally:
        killed = [t for t in tally if t[0]]
        clean = [t for t in tally if not t[0]]
        print(f"\n{len(tally)} shots: {len(killed)} killed ours, {len(clean)} clean.")
        print(f"  old line: warned on {sum(t[1] for t in killed)} of the {len(killed)} that killed (every victim named in {sum(t[3] for t in killed)}), on {sum(t[1] for t in clean)} of the {len(clean)} clean")
        print(f"  cone:     warned on {sum(t[2] for t in killed)} of the {len(killed)} that killed (every victim named in {sum(t[4] for t in killed)}), on {sum(t[2] for t in clean)} of the {len(clean)} clean")


if __name__ == "__main__":
    main()
