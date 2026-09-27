#!/usr/bin/env python3
"""The queue instrument: soldier-seconds spent behind a firing front, with an enemy in reach of the friend in front
but not of the unit itself (the user, 2026-09-25: "a body of units where the front line stops and fires and prevents
the rest from reaching the enemy").

    run/queued.py <match dir or record>... [--pros MAP --floor 40 --repo DIR] [--body 300] [--fight 4] [--top 8]

Per once-a-second sample (`docs/harness/record-format.md`, the `s` line): a soldier of ours (class `army`, a reach,
not being built) with no enemy within its reach + 20 is **queued** when a friendly soldier within `--body` of it has
an enemy within its own reach + 20, stands nearer that enemy than the unit does, and fired this second (`shots`).
**Blocked** is the queued unit whose firing friend stands within `HULL` (40, a hull with clearance) of its line to
its nearest enemy: physically in the way, as against a unit spread out at the flank with nothing to shoot. The
**geometric** count drops the shot condition (a friend in reach and nearer), which is what a replay record can
show, so the pros (`--pros`, the carded games `run/replays/shapes.py` selects) are counted that way and ours both
ways. Blocked seconds are split by whether the unit moved 10 elmos in the second (walking up behind the front) or
stood (held behind it), and by how long it has been engaged. **Engaged** soldier-seconds are those with `--fight` or more armed enemies within 700 (a fight, not a mop-up
of a scattered few: a body of 70 against eleven Rectifiers is "queued" by geometry and has nothing to reach); the
shares are over them. Per game: the totals, by type, by how many friends stand within 120 (the ball), and the top
`--top` ten-second windows by queued seconds with their clock and place, for `run/engagement.py`. `--repo` is where
the pro replay records live (the main checkout, read-only, when run from a worktree).
"""
import argparse, collections, json, math, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "replays"))

SLACK = 20.0
ENGAGED = 700.0
BALL = 120.0
HULL = 40.0


def near_segment(px, pz, ax, az, bx, bz):
    dx, dz = bx - ax, bz - az
    l2 = dx * dx + dz * dz
    if l2 <= 0.0:
        return math.hypot(px - ax, pz - az)
    t = max(0.0, min(1.0, ((px - ax) * dx + (pz - az) * dz) / l2))
    return math.hypot(px - (ax + t * dx), pz - (az + t * dz))


def record_of(path):
    if os.path.isfile(path):
        return path
    for cand in (os.path.join(path, "record-0.jsonl"), os.path.join(path, "00", "record-0.jsonl")):
        if os.path.isfile(cand):
            return cand
    return None


def count(seconds, body, top, with_shots, fight):
    """`seconds`: iterable of (t, soldiers [(id, type, x, z, reach, shots)], enemies [(x, z, armed)])."""
    out = {"engaged": 0, "in_reach": 0, "queued": 0, "blocked": 0, "geo": 0, "geo_blocked": 0, "by_type": collections.Counter(),
           "blocked_split": collections.Counter(), "engaged_age": collections.Counter(), "by_type_engaged": collections.Counter(),
           "ball": collections.Counter(), "ball_engaged": collections.Counter(), "windows": collections.Counter(), "places": {}}
    prev = {}
    since, last = {}, {}
    for t, soldiers, enemies in seconds:
        if not enemies:
            prev = {u[0]: (u[2], u[3]) for u in soldiers}
            continue
        near = []
        for u in soldiers:
            uid, name, x, z, reach, shots = u
            d2 = min(((x - ex) ** 2 + (z - ez) ** 2, ex, ez) for ex, ez, _ in enemies)
            armed = sum(1 for ex, ez, a in enemies if a and (x - ex) ** 2 + (z - ez) ** 2 < ENGAGED * ENGAGED)
            near.append((u, d2[0], (d2[1], d2[2]), armed))
        for (u, d2, (ex, ez), armed) in near:
            uid, name, x, z, reach, shots = u
            if armed < fight:
                continue
            out["engaged"] += 1
            out["by_type_engaged"][name] += 1
            if last.get(uid) is None or t - last[uid] > 5:
                since[uid] = t
            last[uid] = t
            age = t - since[uid]
            age_bucket = "0-10s" if age < 10 else "10-30s" if age < 30 else "30s+"
            out["engaged_age"][age_bucket] += 1
            friends = sum(1 for v, _, _, _ in near if v[0] != uid and (v[2] - x) ** 2 + (v[3] - z) ** 2 < BALL * BALL)
            bucket = "alone" if friends == 0 else "1-2" if friends <= 2 else "3-5" if friends <= 5 else "6+"
            out["ball_engaged"][bucket] += 1
            r = reach + SLACK
            if d2 < r * r:
                out["in_reach"] += 1
                continue
            geo = fired = geo_blocked = blocked = False
            for (v, vd2, (vx, vz), _) in near:
                if v[0] == uid or (v[2] - x) ** 2 + (v[3] - z) ** 2 > body * body:
                    continue
                vr = v[4] + SLACK
                if vd2 >= vr * vr:
                    continue
                # the friend stands nearer its nearest enemy than the unit does
                if (x - vx) ** 2 + (z - vz) ** 2 <= vd2:
                    continue
                geo = True
                in_the_way = near_segment(v[2], v[3], x, z, ex, ez) < HULL
                geo_blocked = geo_blocked or in_the_way
                if v[5] > 0:
                    fired = True
                    blocked = blocked or in_the_way
            if geo:
                out["geo"] += 1
            if geo_blocked:
                out["geo_blocked"] += 1
            if (blocked if with_shots else geo_blocked):
                out["blocked"] += 1
                p = prev.get(uid)
                moved = p is not None and math.hypot(x - p[0], z - p[1]) > 10
                out["blocked_split"][(age_bucket, "moving" if moved else "still")] += 1
            if (fired if with_shots else geo):
                out["queued"] += 1
                out["by_type"][name] += 1
                out["ball"][bucket] += 1
                w = int(t) // 10
                out["windows"][w] += 1
                px, pz, n = out["places"].get(w, (0.0, 0.0, 0))
                out["places"][w] = (px + x, pz + z, n + 1)
        prev = {u[0]: (u[2], u[3]) for u in soldiers}
    return out


def our_seconds(path):
    with open(path) as f:
        header = json.loads(f.readline())
        defs = header["unit_defs"]
        if "reach" not in defs[0]:
            return None, None
        for line in f:
            if '"t":"s"' not in line[:12]:
                continue
            r = json.loads(line)
            shots = {u: n for u, n in r.get("shots", [])}
            soldiers = [(u, defs[d]["name"], x, z, defs[d]["reach"], shots.get(u, 0)) for u, d, x, z, hp, fl in r["own"] if defs[d]["class"] == "army" and defs[d]["reach"] > 0 and not fl & 1]
            enemies = [(u[2], u[3], u[1] < 0 or defs[u[1]]["reach"] > 0) for u in r["en"]]  # five fields before 2026-09-28, six after
            yield r["f"] / 30.0, soldiers, enemies


def pro_seconds(g, side):
    import shapes
    for t in sorted(g.units[side]):
        mine = shapes.at(g, side, t)
        theirs = shapes.at(g, 1 - side, t)
        soldiers = [(uid, u[0], u[1], u[2], shapes.reach(u[0]), 0) for uid, u in mine.items() if shapes.soldier(u[0]) and shapes.reach(u[0]) > 0]
        enemies = [(u[1], u[2], shapes.reach(u[0]) > 0) for u in theirs.values()]
        yield t, soldiers, enemies


def report(label, r, top, shots):
    e = max(1, r["engaged"])
    print(f"== {label}: engaged {r['engaged']} soldier-s, in reach {r['in_reach']} ({100 * r['in_reach'] / e:.0f}%), "
          f"queued {r['queued']} ({100 * r['queued'] / e:.0f}%), blocked {r['blocked']} ({100 * r['blocked'] / e:.0f}%){' [shots]' if shots else ' [geometry]'}; "
          f"geometric queued {r['geo']} ({100 * r['geo'] / e:.0f}%), blocked {r['geo_blocked']} ({100 * r['geo_blocked'] / e:.0f}%)")
    print("  queued by type: " + ", ".join(f"{t} {n} of {r['by_type_engaged'][t]} ({100 * n / max(1, r['by_type_engaged'][t]):.0f}%)" for t, n in r["by_type"].most_common(6)))
    print("  blocked by engagement age, moving / still: " + ", ".join(f"{b} {r['blocked_split'][(b, 'moving')]} / {r['blocked_split'][(b, 'still')]} of {r['engaged_age'][b]} engaged" for b in ("0-10s", "10-30s", "30s+")))
    print("  queued share by friends within 120: " + ", ".join(f"{b} {100 * r['ball'][b] / max(1, r['ball_engaged'][b]):.0f}% of {r['ball_engaged'][b]}" for b in ("alone", "1-2", "3-5", "6+")))
    if top:
        print("  top windows (clock, queued s, place):")
        for w, n in r["windows"].most_common(top):
            px, pz, k = r["places"][w]
            print(f"    {w * 10 // 60}:{w * 10 % 60:02d}  {n:4}  ({px / k:.0f}, {pz / k:.0f})")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("paths", nargs="*")
    ap.add_argument("--pros", help="map name: the carded pro games on it")
    ap.add_argument("--floor", type=int, default=40)
    ap.add_argument("--repo", help="the checkout whose run/matches holds the pro replay records")
    ap.add_argument("--body", type=float, default=300.0)
    ap.add_argument("--fight", type=int, default=4, help="armed enemies within 700 for a soldier-second to count as a fight")
    ap.add_argument("--top", type=int, default=8)
    a = ap.parse_args()
    for p in a.paths:
        rec = record_of(p)
        if rec is None:
            print(f"== {p}: no record", file=sys.stderr)
            continue
        secs = list(our_seconds(rec) or [])
        if not secs:
            print(f"== {p}: the record predates the fire instrument", file=sys.stderr)
            continue
        report(p, count(secs, a.body, a.top, True, a.fight), a.top, True)
    if a.pros:
        import shapes
        if a.repo:
            shapes.REPO = a.repo
            shapes.MANIFEST = os.path.join(a.repo, "run/data/replays/manifest.jsonl")
        shapes.T = shapes.load_tables()
        games = shapes.pro_games(a.pros, a.floor)
        total = None
        for g in games:
            for side in (0, 1):
                r = count(pro_seconds(g, side), a.body, 0, False, a.fight)
                if total is None:
                    total = r
                else:
                    for k in ("engaged", "in_reach", "queued", "blocked", "geo", "geo_blocked"):
                        total[k] += r[k]
                    for k in ("by_type", "by_type_engaged", "ball", "ball_engaged", "blocked_split", "engaged_age"):
                        total[k] += r[k]
        if total:
            report(f"pros on {a.pros} at OS {a.floor}+ ({len(games)} games)", total, 0, False)


if __name__ == "__main__":
    main()
