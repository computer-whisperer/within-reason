#!/usr/bin/env python3
"""The shape of our groups on the ground, second by second, from a game's hands logs and records.

usage: run/group_shape.py <match dir> [--least N]

For every group at every gate the hands logged (once a second at most): its members' places from the seat's record,
how many of them the picture said were still on the way to join (the entry's `reinforcements`; a group whose entry
was not in that request counts none), and the parties in sight. The body is taken as the members nearest the
group's place once those on the way are left out (the log does not say which members they are: an estimate).

  groups      group-seconds counted (groups of `--least` soldiers or more, 20 unless given)
  on the way  share of soldiers the picture said were still joining
  all / body  width of all members and of the body: median, and the share wider than 600 (the picture's
              `STRUNG_OUT`) and than 1,500
  parted      share of group-seconds whose body stands in two clumps: chained within 300, a second clump of three or
              more and a fifth of the body, more than 600 from the first
  four fifths the distance from the middle of the body's largest clump that holds four fifths of the body (a width
              that one straggler does not set), over all and by what the group was doing
  two fights  share whose body has three or more members and a fifth of it within 800 of each of two parties that
              stand more than 1,200 apart
"""
import jevlog
import argparse, glob, json, math, os, re


def width(points):
    return max((math.dist(a, b) for a in points for b in points), default=0.0) if len(points) < 200 else math.hypot(max(p[0] for p in points) - min(p[0] for p in points), max(p[1] for p in points) - min(p[1] for p in points))


def clumps(points, radius=300.0):
    left, out = list(points), []
    while left:
        clump, i = [left.pop()], 0
        while i < len(clump):
            near = [p for p in left if math.dist(p, clump[i]) < radius]
            left = [p for p in left if math.dist(p, clump[i]) >= radius]
            clump += near
            i += 1
        out.append(clump)
    return sorted(out, key=len, reverse=True)


def centre(points):
    return (sum(p[0] for p in points) / len(points), sum(p[1] for p in points) / len(points))


def quantile(values, q):
    return sorted(values)[int(q * (len(values) - 1))] if values else 0.0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("--least", type=int, default=20)
    args = ap.parse_args()
    rows = []
    for log in jevlog.logs(args.dir):
        seat = re.search(r"jev-(\d+)", log).group(1)
        places = {}
        for line in open(os.path.join(args.dir, f"record-{seat}.jsonl")):
            if '"t":"s"' in line:
                r = json.loads(line)
                places[r["f"] // 30] = {u[0]: (u[2], u[3]) for u in r["own"]}
        seen = set()
        for line in jevlog.lines(log):
            if '"groups"' not in line:
                continue
            r = json.loads(line)
            second = r["f"] // 30
            if second in seen or second not in places:
                continue
            seen.add(second)
            actors = (r.get("state") or {}).get("actors") or {}
            parties = [(p["x"], p["z"]) for p in r.get("parties") or [] if "x" in p]
            for g in r["groups"]:
                points = [places[second][m] for m in g["members"] if m in places[second]]
                if len(points) < args.least:
                    continue
                entry = actors.get(f"group_{g['name']}")
                words = entry.get("reinforcements", "") if isinstance(entry, dict) else ""
                if not words and isinstance(entry, str):
                    found = re.search(r"(\d+) of its soldiers are on the way", entry)
                    words = found.group(1) if found else ""
                joining = min(int(words.split()[0]), len(points) - 1) if words else 0
                at = tuple(g["at"])
                body = sorted(points, key=lambda p: math.dist(p, at))[: len(points) - joining]
                cl = clumps(body)
                parted = len(cl) > 1 and len(cl[1]) >= 3 and len(cl[1]) >= 0.2 * len(body) and math.dist(centre(cl[0]), centre(cl[1])) > 600
                share = max(3, 0.2 * len(body))
                held = [p for p in parties if sum(math.dist(p, b) < 800 for b in body) >= share]
                two = any(math.dist(a, b) > 1200 for a in held for b in held)
                middle = centre(cl[0])
                most = sorted(math.dist(p, middle) for p in body)[int(0.8 * (len(body) - 1))]
                rows.append((len(points), joining, width(points), width(body), parted, two, most, g["task"]["kind"]))
    if not rows:
        print("no group that large")
        return
    n = len(rows)
    share = lambda test: f"{100 * sum(1 for r in rows if test(r)) / n:.0f}%"
    print(f"{args.dir}: {n} group-seconds of groups of {args.least} or more; soldiers median {quantile([r[0] for r in rows], 0.5):.0f}, most {max(r[0] for r in rows)}")
    print(f"  on the way: {100 * sum(r[1] for r in rows) / sum(r[0] for r in rows):.0f}% of soldiers; a group with any: {share(lambda r: r[1] > 0)}")
    for name, k in (("all", 2), ("body", 3)):
        print(f"  {name:4s} width: median {quantile([r[k] for r in rows], 0.5):.0f}; wider than 600 {share(lambda r: r[k] > 600)}, than 1,500 {share(lambda r: r[k] > 1500)}")
    print(f"  parted: {share(lambda r: r[4])}; two fights: {share(lambda r: r[5])}")
    print(f"  four fifths of the body within, of its largest clump's middle: median {quantile([r[6] for r in rows], 0.5):.0f}; over 600 {share(lambda r: r[6] > 600)}")
    for kind in sorted(set(r[7] for r in rows)):
        of = [r for r in rows if r[7] == kind]
        print(f"    {kind:10s} {100 * len(of) / n:3.0f}% of group-seconds; body width median {quantile([r[3] for r in of], 0.5):.0f}, four fifths within {quantile([r[6] for r in of], 0.5):.0f}, parted {100 * sum(r[4] for r in of) / len(of):.0f}%")


if __name__ == "__main__":
    main()
