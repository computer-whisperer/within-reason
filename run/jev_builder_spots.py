#!/usr/bin/env python3
"""Builders times spots under a nearest-builder filter, replayed on a game's logs: a builder is asked about a
spot (a build there, a walk there, the decode's "does the packet give it this place") only when it is among the k
builders nearest to it, in a straight line, at that second. Counted per k, over every builder and over the
builders asked at that gate: the gate questions and the decode readings that would not be sent, and the builder
moves at a spot the pick played that the filter would have shut.

    run/jev_builder_spots.py run/matches/<batch> [...]
"""
import bisect, collections, glob, json, math, re, sys

SPOT = re.compile(r"_(spot_\d+)$")


def load(D):
    D = D.rstrip("/") + "/00/"
    frames, where, spots, commander = [], [], [], None
    for l in open(D + "record-0.jsonl"):
        try: r = json.loads(l)
        except ValueError: continue
        t = r.get("t")
        if t is None and "metal_spots" in r or r.get("format"):
            spots = [(s[0], s[1]) for s in r["metal_spots"]]
            com = {i for i, d in enumerate(r.get("unit_defs") or []) if (d.get("class") if isinstance(d, dict) else None) == "commander"}
        elif t == "s":
            pos = {u[0]: (u[2], u[3]) for u in r["own"]}
            for u in r["own"]:
                if u[1] in com: commander = u[0]
            frames.append(r["f"]); where.append((pos, commander))
    return D, frames, where, spots


def at(frames, where, f):
    i = bisect.bisect_right(frames, f) - 1
    return where[max(i, 0)]


def unit_of(actor, commander):
    if actor == "commander": return commander
    m = re.match(r"constructor_(\d+)$", actor)
    return int(m.group(1)) if m else None


def nearest(spot_xy, builders, pos):
    """Builders by distance to the spot."""
    return sorted(builders, key=lambda b: math.dist(pos[b], spot_xy))


for arg in sys.argv[1:]:
    D, frames, where, spots = load(arg)
    KS = (1, 2, 3, 4)
    q_tok = 0.0; q_cut = {(k, pool): 0.0 for k in KS for pool in ("all", "asked")}
    reads = 0; r_cut = collections.Counter(); r_yes = 0; r_yes_cut = collections.Counter()
    plays = 0; p_cut = collections.Counter(); total = 0
    NS = (3, 5, 8, 12); own_cut = collections.Counter(); own_play = collections.Counter(); offered = {}; r_own = collections.Counter(); r_own_yes = collections.Counter()
    asked_now = set(); builders_now = []
    for l in open(D + "jev-0.jsonl"):
        try: r = json.loads(l)
        except ValueError: continue
        t = r.get("t")
        if t in ("call", "decode"): total += (r.get("usage") or {}).get("input_tokens", 0) or 0
        if t not in ("call", "decode", "plan"): continue
        pos, commander = at(frames, where, r.get("f", 0))
        def ranks(spot, pool):
            return nearest(spots[int(spot[5:])], [b for b in pool if b in pos], pos)
        if t == "call" and "questions" in r and not any(k.startswith("worlds") for k in r["questions"]):
            q = r["questions"]; T = r["usage"]["input_tokens"]
            tot = (sum(len(json.dumps(v)) for v in (r.get("state") or {}).values()) + 9000) * (r.get("batches", 1) or 1) + sum(len(json.dumps(v)) for v in q.values())
            every = [u for a in (r.get("state") or {}).get("actors", {}) if (u := unit_of(a, commander)) is not None]
            asking = {u for k in q if (u := unit_of(k.split(".")[0], commander)) is not None}
            asked_now, builders_now = asking, every
            cache = {}
            # The builder's own side: the spots it is asked about this gate, nearest first.
            mine = collections.defaultdict(set)
            for k in q:
                u = unit_of(k.split(".")[0], commander); m = SPOT.search(k)
                if u is not None and m and u in pos: mine[u].add(m.group(1))
            for u, ss in mine.items():
                offered[u] = {sp: i for i, sp in enumerate(sorted(ss, key=lambda sp: math.dist(pos[u], spots[int(sp[5:])])))}
            for k, v in q.items():
                actor, _, move = k.partition(".")
                u = unit_of(actor, commander); m = SPOT.search(move)
                if u is None or not m or u not in pos: continue
                c = T * len(json.dumps(v)) / tot; q_tok += c
                for n in NS:
                    if offered[u][m.group(1)] >= n: own_cut[n] += c
                for name, pool in (("all", every), ("asked", asking)):
                    order = cache.setdefault((m.group(1), name), ranks(m.group(1), pool))
                    i = order.index(u) if u in order else 99
                    for kk in KS:
                        if i >= kk: q_cut[(kk, name)] += c
        elif t == "decode":
            T = r["usage"]["input_tokens"]; rd = r.get("reads") or {}
            if not rd: continue
            builders = sorted({u for k in rd if (u := unit_of(k.split("|")[0], commander)) is not None and u in pos})
            cache = {}
            theirs = collections.defaultdict(list)
            for k in rd:
                if k.count("|") == 2 and "|place|spot_" in k and (u := unit_of(k.split("|")[0], commander)) is not None and u in pos: theirs[u].append(k.split("|")[2])
            rank_r = {u: {sp: i for i, sp in enumerate(sorted(ss, key=lambda sp: math.dist(pos[u], spots[int(sp[5:])])))} for u, ss in theirs.items()}
            for k, v in rd.items():
                if k.count("|") != 2: continue
                a, kind, what = k.split("|")
                u = unit_of(a, commander)
                if kind != "place" or u is None or u not in pos or not what.startswith("spot_"): continue
                reads += 1; r_yes += v >= 0.5
                for n in NS:
                    if rank_r[u][what] >= n: r_own[n] += 1; r_own_yes[n] += v >= 0.5
                order = cache.setdefault(what, ranks(what, builders))
                i = order.index(u)
                for kk in KS:
                    if i >= kk: r_cut[kk] += 1; r_yes_cut[kk] += v >= 0.5
            r_cut["tokens"] += T; r_cut["all"] += len(rd)
        elif t == "plan":
            for m_ in r.get("played") or []:
                actor, _, move = (m_.get("played") or "").partition(".")
                u = unit_of(actor, commander); m = SPOT.search(move)
                if u is None or not m or u not in pos or m_.get("source") != "plan": continue
                plays += 1
                for n in NS:
                    if offered.get(u, {}).get(m.group(1), 0) >= n: own_play[n] += 1
                for name, pool in (("all", builders_now), ("asked", asked_now)):
                    order = ranks(m.group(1), pool); i = order.index(u) if u in order else 99
                    for kk in KS:
                        if i >= kk: p_cut[(kk, name)] += 1
    print(f"== {arg.rstrip('/').split('/')[-1][11:]}: {total/1e6:.1f}M tokens; builder questions at a spot {q_tok/1e6:.2f}M ({100*q_tok/total:.1f}%); {plays} such moves played by the pick")
    print(f"   the decode: {reads} readings of a spot for a builder, of {r_cut['all']} readings in all ({r_cut['tokens']/1e6:.2f}M tokens, {100*r_cut['tokens']/total:.1f}% of the game); {r_yes} read yes")
    for name, words in (("all", "the k nearest of every builder"), ("asked", "the k nearest of the builders asked that gate")):
        for kk in KS:
            print(f"   {words}, k={kk}: gate questions cut {100*q_cut[(kk,name)]/max(1,q_tok):3.0f}% ({100*q_cut[(kk,name)]/total:.1f}% of the game); played moves shut {p_cut[(kk,name)]} of {plays}" + (f"; decode readings cut {100*r_cut[kk]/max(1,reads):3.0f}% (yes readings cut {r_yes_cut[kk]} of {r_yes})" if name == "all" else ""))
    for n in NS:
        print(f"   each builder its {n:2d} nearest spots of those offered: gate questions cut {100*own_cut[n]/max(1,q_tok):3.0f}% ({100*own_cut[n]/total:.1f}% of the game); played moves shut {own_play[n]} of {plays}; decode readings cut {100*r_own[n]/max(1,reads):3.0f}% (yes readings cut {r_own_yes[n]} of {r_yes})")
