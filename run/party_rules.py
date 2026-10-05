#!/usr/bin/env python3
"""Rules for forming enemy parties, compared on a game's records.

usage: run/party_rules.py <match dir> [--seat N] [--from MIN] [--sight] [--every S]

The hands' rule today chains mobile enemies within 400 of each other into one party (picture.rs PARTY_RADIUS), and on
a continuous front that makes one party of the whole line. Each rule here is run over the same seconds of one seat's
record (`en`: what the seat saw; a replay's seat is a spectator and sees all of it, `--sight` keeps what stands within
1,200 of a unit of our side) and measured for what a party is used for:

  parties   mean a second, and the most
  biggest   the most units one party held
  widest    the widest party, corner to corner
  wide      share of units standing in a party wider than 800
  spread    share of a party's metal farther than 600 from its centre (the party is not "on the spot")
  left out  share of the enemy metal within 600 of a party's centre that the rule put in another party (the party is
            less than what stands there)
  renamed   share of units a second whose party's name is not the one they stood under a second before
  new       names made a minute

Rules: `chain 400` today's; `chain 250`; `split 800` today's chain, a party wider than 800 cut in two along its length
until none is; `cells` the map's 8 by 8 grid; `seeds 400` the unit with the most others within 400 takes them, again
on the rest; `kept 400` a party is kept from second to second: its members that still stand within 500 of its centre
stay, a unit without a party joins one whose centre is within 400, the rest seed as `seeds`, two parties whose centres
come within 250 become one. Names pass by the largest overlap, as in the picture (H-HANDS-PARTY-NAMES).
"""
import argparse, json, os
import numpy as np
from scipy.sparse.csgraph import connected_components

CAP, SPOT, SIGHT = 800.0, 600.0, 1200.0


def dist(a, b):
    return np.sqrt(((a[:, None, :] - b[None, :, :]) ** 2).sum(-1))


def chain(xy, radius):
    if len(xy) == 0:
        return np.zeros(0, int)
    return connected_components(dist(xy, xy) < radius, directed=False)[1]


def extent(xy):
    if len(xy) < 2:
        return 0.0
    if len(xy) > 400:
        span = xy.max(0) - xy.min(0)
        return float(np.hypot(*span))
    return float(dist(xy, xy).max())


def split(xy, labels):
    out, n = labels.copy(), labels.max() + 1 if len(labels) else 0
    todo = list(range(n))
    while todo:
        p = todo.pop()
        idx = np.where(out == p)[0]
        if len(idx) < 2 or extent(xy[idx]) <= CAP:
            continue
        pts = xy[idx] - xy[idx].mean(0)
        axis = np.linalg.svd(pts, full_matrices=False)[2][0]
        along = pts @ axis
        centres = np.array([pts[along.argmin()], pts[along.argmax()]])
        for _ in range(8):
            side = dist(pts, centres).argmin(1)
            if side.min() == side.max():
                break
            centres = np.array([pts[side == k].mean(0) for k in (0, 1)])
        if side.min() == side.max():
            side = (along > np.median(along)).astype(int)
            if side.min() == side.max():
                continue
        out[idx[side == 1]] = n
        todo += [p, n]
        n += 1
    return out


def seeds(xy, radius):
    labels = np.full(len(xy), -1)
    free, first = np.arange(len(xy)), 0
    if len(free) == 0:
        return labels
    near = dist(xy[free], xy[free]) < radius
    left = np.ones(len(free), bool)
    while left.any():
        counts = (near & left[None, :]).sum(1) * left
        s = counts.argmax()
        take = near[s] & left
        labels[free[take]] = first
        left &= ~take
        first += 1
    return labels


class Kept:
    """Parties as things that last: [name, member ids]."""

    def __init__(self):
        self.parties, self.next = {}, 1

    def step(self, ids, xy):
        where = {u: i for i, u in enumerate(ids)}
        labels = np.full(len(ids), -1)
        centres = {}
        for name, members in self.parties.items():
            here = [where[u] for u in members if u in where]
            if not here:
                continue
            centre = xy[here].mean(0)
            stay = [i for i in here if np.hypot(*(xy[i] - centre)) < 500.0]
            if stay:
                labels[stay] = name
                centres[name] = xy[stay].mean(0)
        names = sorted(centres)
        for a in names:
            for b in names:
                if a < b and a in centres and b in centres and np.hypot(*(centres[a] - centres[b])) < 250.0:
                    big, small = (a, b) if (labels == a).sum() >= (labels == b).sum() else (b, a)
                    labels[labels == small] = big
                    centres[big] = xy[labels == big].mean(0)
                    del centres[small]
        if centres:
            names = sorted(centres)
            c = np.array([centres[n] for n in names])
            free = np.where(labels < 0)[0]
            if len(free):
                d = dist(xy[free], c)
                best = d.argmin(1)
                for k, i in enumerate(free):
                    if d[k, best[k]] < 400.0:
                        labels[i] = names[best[k]]
        free = np.where(labels < 0)[0]
        fresh = seeds(xy[free], 400.0)
        for k in range(fresh.max() + 1 if len(fresh) else 0):
            labels[free[fresh == k]] = self.next
            self.next += 1
        self.parties = {int(n): [ids[i] for i in np.where(labels == n)[0]] for n in set(labels.tolist())}
        return labels


class Named:
    """A rule's labels given the picture's names: the last second's for the party of largest overlap."""

    def __init__(self):
        self.previous, self.next, self.made = {}, 1, 0

    def step(self, ids, labels):
        groups = {}
        for u, l in zip(ids, labels.tolist()):
            groups.setdefault(l, set()).add(u)
        claims = sorted(((len(m & old), l, name) for l, m in groups.items() for name, old in self.previous.items() if m & old), reverse=True)
        named, taken = {}, set()
        for _, l, name in claims:
            if l not in named and name not in taken:
                named[l] = name
                taken.add(name)
        for l in groups:
            if l not in named:
                named[l] = self.next
                self.next += 1
                self.made += 1
        self.previous = {named[l]: m for l, m in groups.items()}
        return {u: named[l] for u, l in zip(ids, labels.tolist())}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("--seat", type=int, default=0)
    ap.add_argument("--from", dest="start", type=float, default=0.0)
    ap.add_argument("--sight", action="store_true")
    args = ap.parse_args()
    path = os.path.join(args.dir, f"record-{args.seat}.jsonl")
    f = open(path)
    head = json.loads(f.readline())
    defs = head["unit_defs"]
    width, height = head["map"]["width"], head["map"]["height"]
    rules = ["chain 400", "chain 250", "split 800", "cells", "seeds 400", "kept 400"]
    kept, namers = Kept(), {r: Named() for r in rules}
    last = {r: {} for r in rules}
    stat = {r: dict(seconds=0, parties=0, most=0, biggest=0, widest=0.0, units=0, wide=0, metal=0.0, spread=0.0, there=0.0, left=0.0, both=0, renamed=0) for r in rules}
    for line in f:
        if '"t":"s"' not in line:
            continue
        r = json.loads(line)
        if r["f"] < args.start * 1800:
            continue
        en = [e for e in r.get("en", []) if (e[1] < 0 or defs[e[1]]["speed"] > 0) and (len(e) < 6 or not e[5])]
        if args.sight and en:
            ours = np.array([(u[2], u[3]) for u in r.get("own", []) + r.get("al", [])], float).reshape(-1, 2)
            pts = np.array([(e[2], e[3]) for e in en], float)
            seen = (dist(pts, ours) < SIGHT).any(1) if len(ours) else np.zeros(len(en), bool)
            en = [e for e, s in zip(en, seen) if s]
        ids = [e[0] for e in en]
        xy = np.array([(e[2], e[3]) for e in en], float).reshape(-1, 2)
        metal = np.array([defs[e[1]]["metal"] if e[1] >= 0 else 110.0 for e in en], float)
        base = chain(xy, 400.0)
        labelled = {
            "chain 400": base,
            "chain 250": chain(xy, 250.0),
            "split 800": split(xy, base),
            "cells": (np.clip((xy[:, 0] / width * 8).astype(int), 0, 7) * 8 + np.clip((xy[:, 1] / height * 8).astype(int), 0, 7)) if len(xy) else base,
            "seeds 400": seeds(xy, 400.0),
            "kept 400": kept.step(ids, xy),
        }
        for rule, labels in labelled.items():
            s = stat[rule]
            names = namers[rule].step(ids, labels) if rule not in ("cells", "kept 400") else dict(zip(ids, labels.tolist()))
            if rule in ("cells", "kept 400"):
                fresh = set(names.values()) - set(last[rule].values())
                namers[rule].made += len(fresh) if last[rule] else 0
            s["seconds"] += 1
            parts = set(labels.tolist())
            s["parties"] += len(parts)
            s["most"] = max(s["most"], len(parts))
            s["units"] += len(ids)
            for p in parts:
                idx = np.where(labels == p)[0]
                wide = extent(xy[idx])
                s["biggest"] = max(s["biggest"], len(idx))
                s["widest"] = max(s["widest"], wide)
                if wide > CAP:
                    s["wide"] += len(idx)
                centre = xy[idx].mean(0)
                close = np.hypot(*(xy - centre).T) < SPOT
                mine = labels == p
                s["metal"] += metal[mine].sum()
                s["spread"] += metal[mine & ~close].sum()
                # Weighted by the party's metal: what share of what stands at its centre it leaves out.
                there = metal[close].sum()
                if there > 0:
                    s["there"] += metal[mine].sum()
                    s["left"] += metal[mine].sum() * metal[close & ~mine].sum() / there
            for u, n in names.items():
                if u in last[rule]:
                    s["both"] += 1
                    s["renamed"] += n != last[rule][u]
            last[rule] = names
    print(f"{path}: {stat[rules[0]]['seconds']} seconds from minute {args.start:g}, {stat[rules[0]]['units'] / max(1, stat[rules[0]]['seconds']):.0f} mobile enemies a second{' in sight of our side' if args.sight else ''}")
    print(f"{'rule':10s} {'parties':>8s} {'most':>5s} {'biggest':>8s} {'widest':>7s} {'wide':>6s} {'spread':>7s} {'left out':>9s} {'renamed':>8s} {'new/min':>8s}")
    for rule in rules:
        s = stat[rule]
        n = max(1, s["seconds"])
        print(f"{rule:10s} {s['parties'] / n:8.1f} {s['most']:5d} {s['biggest']:8d} {s['widest']:7.0f} {100 * s['wide'] / max(1, s['units']):5.1f}% {100 * s['spread'] / max(1.0, s['metal']):6.1f}% {100 * s['left'] / max(1.0, s['there']):8.1f}% {100 * s['renamed'] / max(1, s['both']):7.2f}% {namers[rule].made / (n / 60):8.1f}")


if __name__ == "__main__":
    main()
