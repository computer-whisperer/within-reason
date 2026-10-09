#!/usr/bin/env python3
"""The nearest-builder filter with class and availability, replayed on a game's logs. A builder is offered a spot
only when it is among the k nearest, by walking time in a straight line (distance over its speed), of the builders
of its own type that are available that second (idle or helping; one that is building, walking to a job or taking
something apart is busy), itself counted whatever it does. Counted per k: the builder questions at a spot not
asked, the decode readings not needed, and the moves the pick played that the filter shuts, with how much farther
the played builder stood than the nearest available one of its type.

    run/jev_builder_avail.py run/matches/<batch> [...]
"""
import jevlog
import bisect, collections, json, math, re, statistics, sys

SPOT = re.compile(r"_(spot_\d+)$")
FREE = ("idle", "help")


def unit_of(actor, commander):
    if actor == "commander": return commander
    m = re.match(r"constructor_(\d+)$", actor)
    return int(m.group(1)) if m else None


for arg in sys.argv[1:]:
    D = arg.rstrip("/") + "/00/"
    frames, where, spots, speed, commander, com = [], [], [], {}, None, set()
    for l in open(D + "record-0.jsonl"):
        try: r = json.loads(l)
        except ValueError: continue
        if r.get("format"):
            spots = [(s[0], s[1]) for s in r["metal_spots"]]
            for i, d in enumerate(r.get("unit_defs") or []):
                if isinstance(d, dict):
                    speed[i] = d.get("speed") or 1.0
                    if d.get("class") == "commander": com.add(i)
        elif r.get("t") == "s":
            pos = {u[0]: (u[2], u[3], u[1]) for u in r["own"]}
            for u in r["own"]:
                if u[1] in com: commander = u[0]
            frames.append(r["f"]); where.append((pos, commander))
    KS = (1, 2, 3, 4)
    total = q_tok = 0.0; q_cut = collections.Counter(); reads = 0; r_cut = collections.Counter(); r_yes = 0; r_yes_cut = collections.Counter()
    plays = 0; shut = collections.defaultdict(list); free = set(); pos = {}; shut_rows = []
    def walk(u, spot): x, z, d = pos[u]; return math.dist((x, z), spots[int(spot[5:])]) / speed.get(d, 1.0)
    def rank(u, spot):
        """How many available builders of u's type reach the spot sooner than u, and the soonest of them."""
        mine = walk(u, spot)
        others = [walk(b, spot) for b in free if b != u and b in pos and pos[b][2] == pos[u][2]]
        return sum(1 for t in others if t < mine), (min(others) if others else None), mine
    for l in jevlog.lines(jevlog.seat(D)):
        try: r = json.loads(l)
        except ValueError: continue
        t = r.get("t")
        if t in ("call", "decode"): total += (r.get("usage") or {}).get("input_tokens", 0) or 0
        if t not in ("pass", "call", "decode", "plan"): continue
        i = bisect.bisect_right(frames, r.get("f", 0)) - 1
        pos, commander = where[max(i, 0)]
        if t == "pass" and r.get("menus"):
            free = {u for m in r["menus"] if m.get("kind") == "builder" and (m.get("idle") or m.get("course") in FREE) and (u := unit_of(m["name"], commander)) is not None}
        elif t == "call" and "questions" in r and not any(k.startswith("worlds") for k in r["questions"]):
            q = r["questions"]; T = r["usage"]["input_tokens"]
            tot = (sum(len(json.dumps(v)) for v in (r.get("state") or {}).values()) + 9000) * (r.get("batches", 1) or 1) + sum(len(json.dumps(v)) for v in q.values())
            for k, v in q.items():
                actor, _, move = k.partition(".")
                u = unit_of(actor, commander); m = SPOT.search(move)
                if u is None or not m or u not in pos: continue
                c = T * len(json.dumps(v)) / tot; q_tok += c
                n, _, _ = rank(u, m.group(1))
                for kk in KS:
                    if n >= kk: q_cut[kk] += c
        elif t == "decode":
            for k, v in (r.get("reads") or {}).items():
                if k.count("|") != 2: continue
                a, kind, what = k.split("|"); u = unit_of(a, commander)
                if kind != "place" or u is None or u not in pos or not what.startswith("spot_"): continue
                reads += 1; r_yes += v >= 0.5
                n, _, _ = rank(u, what)
                for kk in KS:
                    if n >= kk: r_cut[kk] += 1; r_yes_cut[kk] += v >= 0.5
        elif t == "plan":
            for m_ in r.get("played") or []:
                actor, _, move = (m_.get("played") or "").partition(".")
                u = unit_of(actor, commander); m = SPOT.search(move)
                if u is None or not m or u not in pos or m_.get("source") != "plan": continue
                plays += 1
                n, best, mine = rank(u, m.group(1))
                for kk in KS:
                    if n >= kk: shut[kk].append(mine / max(best, 1.0))
                if n >= 2: shut_rows.append((r["f"] // 30, m_.get("played"), round(mine), round(best), n))
    print(f"== {arg.rstrip('/').split('/')[-1][11:]}: builder questions at a spot {q_tok/1e6:.2f}M ({100*q_tok/total:.1f}% of the game), {reads} decode readings of a spot for a builder ({r_yes} yes), {plays} such moves played by the pick")
    for kk in KS:
        far = shut[kk]
        print(f"   k={kk}: questions not asked {100*q_cut[kk]/max(1,q_tok):3.0f}% ({100*q_cut[kk]/total:.1f}% of the game); decode readings not needed {100*r_cut[kk]/max(1,reads):3.0f}% (yes readings among them {r_yes_cut[kk]}); played moves shut {len(far)} of {plays}" + (f", the played builder {statistics.median(far):.1f} times as far in time as the nearest available one at the median" if far else ""))
    for row in shut_rows[:8]: print("     shut at k=2: %d s %s: %d s of walking, the nearest available of its type %d s, %d nearer" % row)
