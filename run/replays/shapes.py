#!/usr/bin/env python3
"""How forces are shaped in fights, pros against us (docs/studies/2026-09-24-pro-fight-shapes.md): the user, from a
replay the players flagged, our Blitzes muzzled by each other and shot by the Stouts behind them. What shapes do the
pros' forces have at contact, how do they layer the types, how do they arrive, and how does each shape trade?

    run/replays/shapes.py [--map "Comet Catcher Remake 1.8"] [--floor 40] [--ours <match dir>...] [--fights] [--orders] [--boot]

Pros: every carded game on the map with both players at OS `--floor` and above (the manifest, as recheck.py selects).
A replay's records see everything, so record-0 carries both sides: team 0 from `own` and its `destroyed` events, team 1
from `en` and `enemy_destroyed`; record-1 adds team 1's orders. Ours: `record-0.jsonl` of each `--ours` match, us from
`own`, the opponent (every BARb seat as one side) from `en` (what we saw of it). Damage for both sides is the drop in
health between consecutive samples (see read_side): replay records carry no `dmg`.

A hit: a mobile combatant (a soldier: the def table's class `army` or `hover` with a weapon, or a commander) taking
damage in a one-second sample. A fight: hits linked when within `LINK_DT` seconds and `LINK_R` elmos of each other
(single linkage over both sides), kept when each side's combatants took at least `MIN_DMG` and each side had at least
one soldier present. Contact `t0` is the fight's first hit; the contact place `C` is the centroid of its hits in the
first ten seconds. A side's body at a second is its soldiers within `PLACE_R` of `C` (commanders and aircraft left out).
Participants are the soldiers that stood within `PRESENT_R` of one of the fight's hits at any second of it.

Measures, per fight side (the rows `--fights` prints):
- shape at t0 and t0+10: units, extent across and along the line from the body's centroid to the enemy body's centroid
  (max minus min of the projections); a line when across >= 2 x along, a stream when along >= 2 x across, a blob
  otherwise (three units and more); nearest-friend spacing by type; the muzzling geometry of run/fire.py: of the
  units with an enemy (any listed unit) within reach + 20, those with a friendly soldier within `HULL` of the segment to
  the nearest one and nearer than it.
- layering: each unit's projection toward the enemy, ranked front (1) to back (0) within the body; short-range
  (reach <= 250: Blitz, Pawn, Grunt, Incisor, the scouts) against line and long-range (reach >= 300) as the gap of
  their median projections: ahead (> +50), beside, behind (< -50). Separate groups: the share of the side's short-range
  metal on the whole map that stood in the body at t0 against the same share of its long-range metal.
- arrival: each participant's first second within `PLACE_R` of C, from 60 s before contact; the share there by t0 and
  the spread (p90 - p10, arrivals before t0 - 30 clamped to -30); transit shape at t0 - 20 (participants' extent across
  and along their line to C, when their centroid was at least 150 elmos off C).
- trade: soldier and commander metal lost in the fight (a destroyed unit whose id took a hit in the fight, else within
  `PLACE_R` of C and the window t0 - 5 .. t1 + 5), each side; the exchange share = theirs / (ours + theirs).
- conduct: damage dealt that landed on units that died in the fight (focus); per second, the most-hit enemy's share
  of that second's damage when two or more were hit; units that fell below 40% health: share that survived, share that
  stepped back 150 elmos or more within 8 s; stand-off: distance to the nearest enemy combatant over own reach, for
  units with one within reach + 100, by reach class.
- orders (`--orders`): the last group order (three or more of the side's participants given move or fight in one
  frame) in the 30 s before contact: distinct targets (a formation drag gives one point per unit), the targets'
  extent across and along the order's direction, their straightness (minor over major principal extent) and spacing.
  BARb's orders are not in our records, so its column is empty.
- `--boot`: the pros' exchange differences between shapes with a 95% interval from resampling whole games.

Every pool counts fight sides: a pro fight gives two rows to "pros" (one to the winners' pool, one to the losers'), so
the pros' exchange is 50% by construction and the shape rows compare shapes against each other.
"""
import argparse, bisect, collections, json, math, os, statistics, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
MANIFEST = os.path.join(REPO, "run/data/replays/manifest.jsonl")
LINK_DT = 8        # seconds: hits this close in time ...
LINK_R = 600.0     # ... and space are one fight
MIN_DMG = 200.0    # each side's combatants take at least this
PLACE_R = 700.0    # a side's body: its soldiers this near the contact place
PRESENT_R = 500.0  # a participant stood this near one of the fight's hits
GROUP_LINK = 200.0  # soldiers this near each other (chained) are one group; the body's largest group is its core
HULL = 24.0        # run/fire.py: a friend this near the line of fire blocks it
SLACK = 20.0
SHORT = 250.0      # reach at or below: the short-range (raider) class
LONG = 300.0       # reach at or above: line and long-range
SIZES = ((2, 6, "2-6"), (7, 15, "7-15"), (16, 10 ** 6, "16+"))


def load_tables():
    """Unit numbers by name: class, reach, footprint from the newest arena header (it carries every def), health and
    metal from crates/bot/data/units.json. Replay headers predate `reach` and classify aircraft as `army`."""
    g = json.load(open(os.path.join(REPO, "crates/bot/data/units.json")))["units"]
    rows = g if isinstance(g, list) else [dict(v, name=k) for k, v in g.items()]
    gl = {r["name"]: r for r in rows}
    table = {}
    hdr = os.path.join(REPO, "run/matches/1790257668-bank-1/00/record-0.jsonl")
    if os.path.exists(hdr):
        for u in json.loads(open(hdr).readline())["unit_defs"]:
            table[u["name"]] = {"class": u["class"], "reach": u.get("reach") or 0.0, "hull": 8 * max(u.get("footprint") or [4, 4]),
                                "metal": u.get("metal") or 0.0, "health": (gl.get(u["name"]) or {}).get("health")}
    for n, r in gl.items():
        if n not in table:
            table[n] = {"class": "army" if "range" in r else "other", "reach": r.get("range") or 0.0, "hull": 32, "metal": r.get("metal") or 0.0, "health": r.get("health")}
    return table


T = None


def soldier(name):
    t = T.get(name)
    return bool(t) and t["class"] in ("army", "hover") and t["reach"] > 0


def combatant(name):
    t = T.get(name)
    return bool(t) and (soldier(name) or t["class"] == "commander")


def reach(name):
    return (T.get(name) or {}).get("reach") or 0.0


def metal(name):
    return (T.get(name) or {}).get("metal") or 0.0


def rclass(name):
    r = reach(name)
    return "short" if r <= SHORT else "long" if r >= LONG else "mid"


# ---------------------------------------------------------------- reading

def lines(path):
    with open(path) as f:
        for line in f:
            try:
                yield json.loads(line)
            except json.JSONDecodeError:
                continue


class Game:
    """Per second: units[side][t] = {id: (name, x, z, health fraction)}, hits (t, side, id, x, z, damage), deaths
    (t, side, id, name, x, z), orders[side] = [(t, kind, id, x, z)]. Sides 0 and 1; 0 is 'ours' in our games."""

    def __init__(self, label, kind):
        self.label, self.kind = label, kind
        self.units = [dict(), dict()]
        self.hits, self.deaths = [], []
        self.orders = [[], []]
        self.won = [None, None]
        self.players = [None, None]


def read_side(g, side, path, en_side=None, orders_only=False):
    """Read one record as side `side`, and its `en` as side `en_side`. Damage is the drop in health between consecutive
    samples while the unit stays listed, for both sides (a replay record's `dmg` is always empty: the dump widget does
    not hook UnitDamaged): our side's percent times the type's health, the enemy's absolute health. The drop is net of
    repair and regeneration within the second. `orders_only`: read the `cmd` lines alone (the other record of a replay)."""
    header = None
    last_en = {}
    last_own = {}
    for r in lines(path):
        t = r.get("t")
        if t == "header":
            header = r
            defs = [u["name"] for u in r["unit_defs"]]
            continue
        if header is None:
            continue
        if "f" not in r:
            continue
        sec = r["f"] // 30
        if orders_only and t != "cmd":
            continue
        if t == "s":
            own = {}
            for u in r["own"]:
                uid, d, x, z, hp, fl = u[:6]
                if fl & 1 or d < 0:
                    continue
                own[uid] = (defs[d], x, z, hp / 100.0)
            g.units[side][sec] = own
            for uid, (n, x, z, hp) in own.items():
                if uid in last_own and combatant(n):
                    drop = (last_own[uid] - hp) * ((T.get(n) or {}).get("health") or 0)
                    if drop > 0:
                        g.hits.append((sec, side, uid, x, z, drop))
            last_own = {uid: u[3] for uid, u in own.items()}
            if en_side is not None:
                en = {}
                for e in r["en"]:
                    uid, d, x, z, hp = e[:5]
                    if d < 0:
                        continue
                    name = defs[d]
                    mx = (T.get(name) or {}).get("health") or 0
                    en[uid] = (name, x, z, min(1.0, hp / mx) if mx else 1.0)
                    if uid in last_en and combatant(name):
                        drop = last_en[uid] - hp
                        if drop > 0:
                            g.hits.append((sec, en_side, uid, x, z, drop))
                last_en = {e[0]: e[4] for e in r["en"]}
                g.units[en_side][sec] = en
        elif t == "ev":
            k = r.get("k")
            if k == "destroyed" and r.get("d", -1) >= 0:
                g.deaths.append((sec, side, r["u"], defs[r["d"]], r.get("x", 0), r.get("z", 0)))
            elif k == "enemy_destroyed" and en_side is not None and r.get("d") is not None and r["d"] >= 0:
                g.deaths.append((sec, en_side, r["u"], defs[r["d"]], r.get("x", 0), r.get("z", 0)))
        elif t == "cmd":
            for c in r["c"]:
                kind = str(c[0]).split(":")[0]
                if kind in ("move", "fight") and len(c) >= 4:
                    x, z = (c[2], c[4]) if len(c) >= 5 else (c[2], c[3])
                    g.orders[side].append((sec, r["f"], kind, c[1], x, z))
    return header


def pro_games(map_name, floor):
    out = []
    for line in open(MANIFEST):
        r = json.loads(line)
        if not (r.get("keep") and r.get("card")) or r.get("map_script") != map_name:
            continue
        oss = [p.get("os") for p in r["players"]]
        if len(oss) != 2 or any(o is None or o < floor for o in oss):
            continue
        d = os.path.join(REPO, r["match"])
        if not all(os.path.exists(os.path.join(d, f"record-{i}.jsonl")) for i in (0, 1)):
            continue
        g = Game(r["id"][:8], "pro")
        winners = set(r.get("winners") or [])
        by_team = {p["team"]: p for p in r["players"]}
        for side in (0, 1):
            # record-0 carries both sides (full vision); record-1 only adds side 1's orders
            h = read_side(g, side, os.path.join(d, f"record-{side}.jsonl"), en_side=1 if side == 0 else None, orders_only=side == 1)
            p = by_team.get(h["team"], {})
            g.players[side] = p.get("name")
            g.won[side] = p.get("name") in winners
        out.append(g)
    return out


def our_game(path):
    rec = path if path.endswith(".jsonl") else os.path.join(path, "record-0.jsonl")
    g = Game(rec.split("/matches/")[-1].split("/")[0], "ours")
    read_side(g, 0, rec, en_side=1)
    res = [r for r in lines(rec) if r.get("t") == "result"]
    if res:
        won = res[-1]["result"].get("outcome") == "Win"
        g.won = [won, not won]
    g.players = ["us", "BARb"]
    return g


# ---------------------------------------------------------------- fights

def find_fights(g):
    hits = sorted(g.hits)
    parent = list(range(len(hits)))

    def root(i):
        while parent[i] != i:
            parent[i] = parent[parent[i]]
            i = parent[i]
        return i
    times = [h[0] for h in hits]
    for i, h in enumerate(hits):
        j = bisect.bisect_right(times, h[0] + LINK_DT)
        for k in range(i + 1, j):
            o = hits[k]
            if (h[3] - o[3]) ** 2 + (h[4] - o[4]) ** 2 <= LINK_R * LINK_R:
                a, b = root(i), root(k)
                if a != b:
                    parent[b] = a
    groups = collections.defaultdict(list)
    for i, h in enumerate(hits):
        groups[root(i)].append(h)
    fights = []
    for hs in groups.values():
        dmg = [sum(h[5] for h in hs if h[1] == s) for s in (0, 1)]
        if min(dmg) < MIN_DMG:
            continue
        hs.sort()
        t0, t1 = hs[0][0], hs[-1][0]
        first = [h for h in hs if h[0] <= t0 + 10]
        cx = sum(h[3] for h in first) / len(first)
        cz = sum(h[4] for h in first) / len(first)
        fights.append({"t0": t0, "t1": t1, "c": (cx, cz), "hits": hs, "dmg": dmg})
    fights.sort(key=lambda f: f["t0"])
    return fights


def at(g, side, t):
    """The side's units at second t (the nearest sample at or before it, within 2 s)."""
    for d in (0, 1, 2):
        u = g.units[side].get(t - d)
        if u is not None:
            return u
    return {}


def body(g, side, t, c):
    return {uid: u for uid, u in at(g, side, t).items() if soldier(u[0]) and (u[1] - c[0]) ** 2 + (u[2] - c[1]) ** 2 <= PLACE_R ** 2}


def centroid(pts):
    return (sum(p[0] for p in pts) / len(pts), sum(p[1] for p in pts) / len(pts))


def unit_dir(a, b):
    dx, dz = b[0] - a[0], b[1] - a[1]
    n = math.hypot(dx, dz)
    return (dx / n, dz / n) if n > 1e-6 else None


def near_segment(px, pz, ax, az, bx, bz):
    dx, dz = bx - ax, bz - az
    l2 = dx * dx + dz * dz
    if l2 <= 0.0:
        return math.hypot(px - ax, pz - az)
    t = max(0.0, min(1.0, ((px - ax) * dx + (pz - az) * dz) / l2))
    return math.hypot(px - (ax + t * dx), pz - (az + t * dz))


def extents(pts, d):
    along = [p[0] * d[0] + p[1] * d[1] for p in pts]
    across = [-p[0] * d[1] + p[1] * d[0] for p in pts]
    return max(across) - min(across), max(along) - min(along), along


def shape_class(across, along, n):
    if n < 3:
        return None
    if across >= 2 * max(along, 1):
        return "line"
    if along >= 2 * max(across, 1):
        return "stream"
    return "blob"


def components(units):
    """Single-linkage groups of the units at `GROUP_LINK`, largest first (lists of ids)."""
    ids = list(units)
    seen, out = set(), []
    for i in ids:
        if i in seen:
            continue
        stack, grp = [i], []
        seen.add(i)
        while stack:
            a = stack.pop()
            grp.append(a)
            ua = units[a]
            for b in ids:
                if b not in seen and (ua[1] - units[b][1]) ** 2 + (ua[2] - units[b][2]) ** 2 <= GROUP_LINK ** 2:
                    seen.add(b)
                    stack.append(b)
        out.append(grp)
    out.sort(key=len, reverse=True)
    return out


def snapshot(g, side, t, c):
    """Shape of the side's body at second t against the enemy's combatants near C."""
    mine = body(g, side, t, c)
    enemy_all = at(g, 1 - side, t)
    foes = [(u[1], u[2]) for u in enemy_all.values() if combatant(u[0]) and (u[1] - c[0]) ** 2 + (u[2] - c[1]) ** 2 <= PLACE_R ** 2]
    out = {"n": len(mine), "metal": sum(metal(u[0]) for u in mine.values()), "types": collections.Counter(u[0] for u in mine.values())}
    if not mine:
        return out
    groups = components(mine)
    core = {uid: mine[uid] for uid in groups[0]}
    out["groups"] = sum(1 for gr in groups if len(gr) >= 2)
    out["core_n"] = len(core)
    pts = [(u[1], u[2]) for u in core.values()]
    me = centroid(pts)
    d = unit_dir(me, centroid(foes) if foes else c)
    if d is None:
        d = unit_dir(me, c) or (1.0, 0.0)
    out["dir"] = d
    across, along, proj = extents(pts, d)
    out["across"], out["along"] = across, along
    out["shape"] = shape_class(across, along, len(pts))
    # spacing
    nn = {}
    ids = list(mine)
    for uid in ids:
        n, x, z, _ = mine[uid]
        best = min((math.hypot(x - v[1], z - v[2]) for w, v in mine.items() if w != uid), default=None)
        if best is not None:
            nn[uid] = best
    out["nn"] = [(mine[u][0], v) for u, v in nn.items()]
    # muzzling geometry against every listed enemy unit
    targets = [(u[1], u[2]) for u in enemy_all.values()]
    inreach = blocked = 0
    by_type = collections.Counter()
    for uid, (n, x, z, _) in mine.items():
        rr = reach(n) + SLACK
        near = [(ex, ez) for ex, ez in targets if (x - ex) ** 2 + (z - ez) ** 2 < rr * rr]
        if not near:
            continue
        ex, ez = min(near, key=lambda e: (x - e[0]) ** 2 + (z - e[1]) ** 2)
        inreach += 1
        by_type[(n, "in")] += 1
        dd = (x - ex) ** 2 + (z - ez) ** 2
        if any(w != uid and near_segment(v[1], v[2], x, z, ex, ez) < HULL and (v[1] - x) ** 2 + (v[2] - z) ** 2 < dd for w, v in mine.items()):
            blocked += 1
            by_type[(n, "blocked")] += 1
    out["inreach"], out["blocked"], out["muzzle_by_type"] = inreach, blocked, by_type
    # layering, within the core
    lo, hi = min(proj), max(proj)
    ranks = []
    for (uid, u), p in zip(core.items(), proj):
        ranks.append((u[0], (p - lo) / (hi - lo) if hi - lo > 1 else 0.5, p))
    out["ranks"] = ranks
    sh = [p for n, r, p in ranks if rclass(n) == "short"]
    lg = [p for n, r, p in ranks if rclass(n) == "long"]
    if sh and lg:
        out["short_gap"] = statistics.median(sh) - statistics.median(lg)
    # where the side's short and long metal was: in the body or elsewhere on the map
    out["core_types"] = collections.Counter(u[0] for u in core.values())
    allmine = [u for u in at(g, side, t).values() if soldier(u[0])]
    out["owns_both"] = sum(1 for u in allmine if rclass(u[0]) == "short") >= 2 and sum(1 for u in allmine if rclass(u[0]) == "long") >= 2
    for cls in ("short", "long"):
        tot = sum(metal(u[0]) for u in allmine if rclass(u[0]) == cls)
        here = sum(metal(u[0]) for u in mine.values() if rclass(u[0]) == cls)
        out[f"{cls}_here"] = (here, tot)
    return out


def measure(g, f):
    t0, t1, c = f["t0"], f["t1"], f["c"]
    by_t = collections.defaultdict(list)
    for h in f["hits"]:
        by_t[h[0]].append(h)
    hit_ids = [set(h[2] for h in f["hits"] if h[1] == s) for s in (0, 1)]
    res = {"game": g.label, "kind": g.kind, "t0": t0, "t1": t1, "c": c, "sides": []}
    # deaths in the fight
    dead = [dict(), dict()]
    for (sec, side, uid, name, x, z) in g.deaths:
        if not combatant(name) or sec < t0 - 5 or sec > t1 + 5:
            continue
        if uid in hit_ids[side] or (x - c[0]) ** 2 + (z - c[1]) ** 2 <= PLACE_R ** 2:
            dead[side][uid] = (sec, name)
    lost = [sum(metal(n) for s, n in dead[s].values()) for s in (0, 1)]
    for side in (0, 1):
        s = {"side": side, "won_game": g.won[side], "player": g.players[side], "lost": lost[side], "killed": lost[1 - side]}
        s["exchange"] = lost[1 - side] / (lost[0] + lost[1]) if lost[0] + lost[1] > 0 else None
        s["s0"] = snapshot(g, side, t0, c)
        s["s10"] = snapshot(g, side, t0 + 10, c)
        # participants
        part = set()
        for t in range(t0, t1 + 1):
            hs = by_t.get(t, []) + by_t.get(t - 1, []) + by_t.get(t + 1, [])
            if not hs:
                continue
            for uid, u in at(g, side, t).items():
                if uid not in part and soldier(u[0]) and any((u[1] - h[3]) ** 2 + (u[2] - h[4]) ** 2 <= PRESENT_R ** 2 for h in hs):
                    part.add(uid)
        s["participants"] = len(part)
        # arrival
        arr = {}
        for t in range(t0 - 60, t1 + 1):
            for uid in part:
                if uid in arr:
                    continue
                u = at(g, side, t).get(uid)
                if u and (u[1] - c[0]) ** 2 + (u[2] - c[1]) ** 2 <= PLACE_R ** 2:
                    arr[uid] = t - t0
        a = sorted(max(-30, v) for v in arr.values())
        if len(a) >= 3:
            q = lambda p: a[min(len(a) - 1, int(round(p * (len(a) - 1))))]
            s["arrived_by_t0"] = sum(1 for v in a if v <= 0) / len(a)
            s["arrival_spread"] = q(0.9) - q(0.1)
        # transit at t0 - 20
        tr = [(u[1], u[2]) for uid, u in at(g, side, t0 - 20).items() if uid in part]
        if len(tr) >= 3:
            ce = centroid(tr)
            if math.hypot(ce[0] - c[0], ce[1] - c[1]) >= 150:
                d = unit_dir(ce, c)
                across, along, _ = extents(tr, d)
                s["transit"] = (across, along, shape_class(across, along, len(tr)))
        # conduct: focus of what this side dealt (hits on the other side)
        theirs = [h for h in f["hits"] if h[1] == 1 - side]
        dealt = sum(h[5] for h in theirs)
        s["dealt"] = dealt
        s["on_killed"] = sum(h[5] for h in theirs if h[2] in dead[1 - side]) / dealt if dealt else None
        tops = []
        per = collections.defaultdict(lambda: collections.Counter())
        for h in theirs:
            per[h[0]][h[2]] += h[5]
        for t, cnt in per.items():
            if len(cnt) >= 2:
                tot = sum(cnt.values())
                tops.append((cnt.most_common(1)[0][1] / tot, tot))
        s["top_share"] = sum(a * w for a, w in tops) / sum(w for a, w in tops) if tops else None
        # damaged units: survive, step back
        low = {}
        for t in range(t0, t1 + 1):
            for uid, u in at(g, side, t).items():
                if uid in part and uid not in low and u[3] <= 0.4:
                    low[uid] = t
        back = 0
        for uid, t in low.items():
            u, v = at(g, side, t).get(uid), at(g, side, t + 8).get(uid)
            foes = [(e[1], e[2]) for e in at(g, 1 - side, t).values() if combatant(e[0])]
            if not (u and v and foes):
                continue
            ex, ez = min(foes, key=lambda e: (e[0] - u[1]) ** 2 + (e[1] - u[2]) ** 2)
            if math.hypot(v[1] - ex, v[2] - ez) - math.hypot(u[1] - ex, u[2] - ez) >= 150:
                back += 1
        s["low"], s["low_survived"], s["low_back"] = len(low), sum(1 for uid in low if uid not in dead[side]), back
        # stand-off: distance to the nearest enemy combatant over own reach
        so = collections.defaultdict(list)
        for t in range(t0, t1 + 1):
            foes = [(e[1], e[2]) for e in at(g, 1 - side, t).values() if combatant(e[0])]
            if not foes:
                continue
            for uid, u in at(g, side, t).items():
                if uid not in part:
                    continue
                r = reach(u[0])
                dmin = min(math.hypot(u[1] - x, u[2] - z) for x, z in foes)
                if dmin <= r + 100:
                    so["short" if r <= SHORT else "line" if r < 450 else "long"].append(dmin / r)
        s["standoff"] = so
        # orders: last group order to participants in the 30 s before contact
        s["order"] = last_group_order(g, side, part, t0)
        s["commander_hit"] = any(h[1] == side and not soldier(at(g, side, h[0]).get(h[2], ("?",))[0]) for h in f["hits"])
        res["sides"].append(s)
    res["size"] = res["sides"][0]["participants"] + res["sides"][1]["participants"]
    return res


def last_group_order(g, side, part, t0):
    frames = collections.defaultdict(list)
    for sec, fr, kind, uid, x, z in g.orders[side]:
        if t0 - 30 <= sec <= t0 and uid in part:
            frames[fr].append((kind, uid, x, z))
    best = None
    for fr in sorted(frames):
        cs = frames[fr]
        if len({c[1] for c in cs}) >= 3:
            best = (fr, cs)
    if best is None:
        return None
    fr, cs = best
    last = {}
    for kind, uid, x, z in cs:
        last[uid] = (x, z)  # the last point per unit: a queued route keeps its end
    pts = list(last.values())
    distinct = len({(round(x / 16), round(z / 16)) for x, z in pts})
    units = at(g, side, fr // 30)
    starts = [(units[u][1], units[u][2]) for u in last if u in units]
    d = unit_dir(centroid(starts), centroid(pts)) if starts else None
    out = {"n": len(pts), "distinct": distinct}
    if d:
        across, along, _ = extents(pts, d)
        out["across"], out["along"] = across, along
    if len(pts) >= 3:
        # straightness: the minor over the major principal extent of the targets (0 = on one straight line)
        cx, cz = centroid(pts)
        sxx = sum((p[0] - cx) ** 2 for p in pts); szz = sum((p[1] - cz) ** 2 for p in pts); sxz = sum((p[0] - cx) * (p[1] - cz) for p in pts)
        tr, det = sxx + szz, sxx * szz - sxz * sxz
        l1 = tr / 2 + math.sqrt(max(0.0, tr * tr / 4 - det)); l2 = max(0.0, tr - l1)
        out["straight"] = math.sqrt(l2 / l1) if l1 > 0 else None
    if len(pts) >= 2:
        out["spacing"] = statistics.median(min(math.hypot(p[0] - q[0], p[1] - q[1]) for q in pts if q is not p) for p in pts)
    return out


# ---------------------------------------------------------------- report

def med(xs):
    xs = [x for x in xs if x is not None]
    return (statistics.median(xs), len(xs)) if xs else (None, 0)


def fmt(v, n=None, pct=False, d=0):
    if v is None:
        return "-"
    s = f"{100 * v:.0f}%" if pct else f"{v:.{d}f}"
    return f"{s} (n={n})" if n is not None else s


def pools(fights):
    rows = collections.defaultdict(list)
    for f in fights:
        for s in f["sides"]:
            if f["kind"] == "pro":
                rows["pros"].append((f, s))
                rows["pro winners" if s["won_game"] else "pro losers"].append((f, s))
            else:
                rows["us" if s["side"] == 0 else "BARb (vs us)"].append((f, s))
    return rows


def report(fights, show_orders):
    P = pools(fights)
    names = [n for n in ("pros", "pro winners", "pro losers", "us", "BARb (vs us)") if P.get(n)]

    def table(title, fn):
        print(f"\n## {title}")
        print(f"| measure | " + " | ".join(names) + " |")
        print("|---|" + "---|" * len(names))
        for label, get in fn:
            print(f"| {label} | " + " | ".join(get(P[n]) for n in names) + " |")

    def shape_share(rs, key, cls):
        ss = [s[key].get("shape") for f, s in rs if s[key].get("shape")]
        return f"{sum(1 for x in ss if x == cls)} of {len(ss)}" + (f" ({100 * sum(1 for x in ss if x == cls) / len(ss):.0f}%)" if ss else "")

    def medkey(rs, key, sub, d=0, min_n=3):
        v, n = med([s[key].get(sub) for f, s in rs if s[key].get("core_n", 0) >= min_n])
        return fmt(v, n, d=d)

    def ratio(rs, key):
        v, n = med([s[key]["across"] / max(s[key]["along"], 1) for f, s in rs if s[key].get("core_n", 0) >= 3])
        return fmt(v, n, d=2)

    def muzzle(rs, key):
        a = sum(s[key].get("inreach", 0) for f, s in rs)
        b = sum(s[key].get("blocked", 0) for f, s in rs)
        return f"{b} of {a} ({100 * b / a:.0f}%)" if a else "-"

    print(f"fights: {len(fights)} ({sum(1 for f in fights if f['kind'] == 'pro')} pro, {sum(1 for f in fights if f['kind'] == 'ours')} ours); "
          f"sides with 3+ soldiers in the body at contact: " + ", ".join(f"{n} {sum(1 for f, s in P[n] if s['s0'].get('n', 0) >= 3)}" for n in names))
    table("Shape at contact (t0) and 10 s in: extents of the body's core (its largest group), cores of 3+ soldiers", [
        ("soldiers in the body at t0, median (bodies of 3+)", lambda rs: fmt(*med([s["s0"]["n"] for f, s in rs if s["s0"]["n"] >= 3]))),
        ("soldiers in the core at t0, median (cores of 3+)", lambda rs: fmt(*med([s["s0"]["core_n"] for f, s in rs if s["s0"].get("core_n", 0) >= 3]))),
        ("groups of 2+ in the body at t0, median (bodies of 3+)", lambda rs: fmt(*med([s["s0"]["groups"] for f, s in rs if s["s0"]["n"] >= 3]))),
        ("bodies of 3+ in two or more groups", lambda rs: (lambda b: f"{sum(1 for x in b if x >= 2)} of {len(b)}")([s["s0"]["groups"] for f, s in rs if s["s0"]["n"] >= 3])),
        ("core's share of the body at t0, median", lambda rs: fmt(*med([s["s0"]["core_n"] / s["s0"]["n"] for f, s in rs if s["s0"]["n"] >= 3]), pct=True)),
        ("across at t0, median elmos", lambda rs: medkey(rs, "s0", "across")),
        ("along at t0, median elmos", lambda rs: medkey(rs, "s0", "along")),
        ("across / along at t0, median", lambda rs: ratio(rs, "s0")),
        ("line at t0", lambda rs: shape_share(rs, "s0", "line")),
        ("blob at t0", lambda rs: shape_share(rs, "s0", "blob")),
        ("stream at t0", lambda rs: shape_share(rs, "s0", "stream")),
        ("across at t0+10, median", lambda rs: medkey(rs, "s10", "across")),
        ("along at t0+10, median", lambda rs: medkey(rs, "s10", "along")),
        ("across / along at t0+10, median", lambda rs: ratio(rs, "s10")),
        ("line at t0+10", lambda rs: shape_share(rs, "s10", "line")),
        ("blob at t0+10", lambda rs: shape_share(rs, "s10", "blob")),
        ("stream at t0+10", lambda rs: shape_share(rs, "s10", "stream")),
        ("friend on the line (units with an enemy in reach), t0", lambda rs: muzzle(rs, "s0")),
        ("friend on the line, t0+10", lambda rs: muzzle(rs, "s10")),
    ])
    # spacing by type
    def spacing_rows():
        rows = []
        cnt = collections.Counter()
        for n in names:
            for f, s in P[n]:
                for key in ("s0", "s10"):
                    for name, v in s[key].get("nn", []):
                        cnt[name] += 1
        for name, _ in cnt.most_common(12):
            def get(rs, name=name):
                vs = [v for f, s in rs for key in ("s0", "s10") for nm, v in s[key].get("nn", []) if nm == name]
                if not vs:
                    return "-"
                hull = T[name]["hull"]
                return f"{statistics.median(vs):.0f}, {100 * sum(1 for v in vs if v < 1.5 * hull) / len(vs):.0f}% < {1.5 * hull:.0f} (n={len(vs)})"
            rows.append((f"{name} (hull {T[name]['hull']})", get))
        return rows
    def muzzle_rows():
        cnt = collections.Counter()
        for n in names:
            for f, s in P[n]:
                for k in ("s0", "s10"):
                    for (nm, what), v in s[k].get("muzzle_by_type", {}).items():
                        if what == "in":
                            cnt[nm] += v
        rows = []
        for nm, _ in cnt.most_common(10):
            def get(rs, nm=nm):
                a = sum(s[k].get("muzzle_by_type", {}).get((nm, "in"), 0) for f, s in rs for k in ("s0", "s10"))
                b = sum(s[k].get("muzzle_by_type", {}).get((nm, "blocked"), 0) for f, s in rs for k in ("s0", "s10"))
                return f"{b} of {a} ({100 * b / a:.0f}%)" if a else "-"
            rows.append((f"{nm} (reach {reach(nm):.0f})", get))
        return rows
    table("Friend on the line by type, t0 and t0+10 together (units with an enemy in reach)", muzzle_rows())

    def by_body(rs, lo, hi):
        a = b = n = 0
        for f, s in rs:
            for k in ("s0", "s10"):
                if lo <= s[k]["n"] <= hi:
                    a += s[k].get("inreach", 0)
                    b += s[k].get("blocked", 0)
                    n += 1
        return f"{b} of {a} ({100 * b / a:.0f}%; {n} bodies)" if a else "-"
    table("Friend on the line by the body's size, t0 and t0+10 together", [
        (f"bodies of {lo}-{hi}" if hi < 999 else f"bodies of {lo}+", lambda rs, lo=lo, hi=hi: by_body(rs, lo, hi)) for lo, hi in ((3, 6), (7, 12), (13, 999))])
    table("Nearest-friend spacing by type at t0 and t0+10 (median elmos; share closer than 1.5 hulls; unit-snapshots)", spacing_rows())

    def layer(rs, cls):
        gs = [s["s0"]["short_gap"] for f, s in rs if s["s0"].get("short_gap") is not None and s["s0"].get("core_n", 0) >= 3]
        if not gs:
            return "-"
        c = {"ahead": sum(1 for x in gs if x > 50), "beside": sum(1 for x in gs if -50 <= x <= 50), "behind": sum(1 for x in gs if x < -50)}
        return f"{c[cls]} of {len(gs)} ({100 * c[cls] / len(gs):.0f}%)"

    def here(rs, cls):
        h = sum(s["s0"].get(f"{cls}_here", (0, 0))[0] for f, s in rs if s["s0"].get("n", 0) >= 3)
        t = sum(s["s0"].get(f"{cls}_here", (0, 0))[1] for f, s in rs if s["s0"].get("n", 0) >= 3)
        return f"{100 * h / t:.0f}% of {t:.0f}" if t else "-"

    def both(ts):
        return any(rclass(n) == "short" for n in ts) and any(rclass(n) == "long" for n in ts)

    def mixed(rs, owners=False):
        b = [s for f, s in rs if s["s0"].get("core_n", 0) >= 3 and (not owners or s["s0"].get("owns_both"))]
        m = sum(1 for s in b if both(s["s0"]["core_types"]))
        return f"{m} of {len(b)} ({100 * m / max(1, len(b)):.0f}%)"

    table("Type layering at contact (within the core; the gap rows over cores holding both short-range and long-range soldiers)", [
        ("cores of 3+ holding both short and long", mixed),
        ("... when the side owned 2+ of each on the map", lambda rs: mixed(rs, True)),
        ("sides owning 2+ short and 2+ long at t0 (cores of 3+)", lambda rs: (lambda b: f"{sum(b)} of {len(b)}")([bool(s["s0"].get("owns_both")) for f, s in rs if s["s0"].get("core_n", 0) >= 3])),
        ("short-range ahead of long (median gap > +50)", lambda rs: layer(rs, "ahead")),
        ("beside (within 50)", lambda rs: layer(rs, "beside")),
        ("behind (< -50)", lambda rs: layer(rs, "behind")),
        ("median gap short - long, elmos", lambda rs: fmt(*med([s["s0"].get("short_gap") for f, s in rs if s["s0"].get("core_n", 0) >= 3]))),
        ("side's short-range metal in the body at t0", lambda rs: here(rs, "short")),
        ("side's long-range metal in the body at t0", lambda rs: here(rs, "long")),
    ])
    # rank by type
    def rank_rows():
        cnt = collections.Counter()
        for n in names:
            for f, s in P[n]:
                if s["s0"].get("n", 0) >= 3:
                    for name, r, p in s["s0"].get("ranks", []):
                        cnt[name] += 1
        rows = []
        for name, _ in cnt.most_common(12):
            def get(rs, name=name):
                vs = [r for f, s in rs if s["s0"].get("n", 0) >= 3 for nm, r, p in s["s0"].get("ranks", []) if nm == name]
                return f"{statistics.mean(vs):.2f} (n={len(vs)})" if vs else "-"
            rows.append((f"{name} (reach {reach(name):.0f})", get))
        return rows
    table("Where each type stands at t0: mean rank in its body, 1 = frontmost, 0 = rearmost (bodies of 3+)", rank_rows())

    def tr(rs, cls):
        ts = [s["transit"][2] for f, s in rs if s.get("transit")]
        return f"{sum(1 for x in ts if x == cls)} of {len(ts)}" if ts else "-"
    table("Arrival (participants; sides with 3+ participants)", [
        ("participants, median", lambda rs: fmt(*med([s["participants"] for f, s in rs if s["participants"] >= 3]))),
        ("share within 700 of the contact place by t0, median", lambda rs: fmt(*med([s.get("arrived_by_t0") for f, s in rs]), pct=True)),
        ("arrival spread p90-p10, s, median", lambda rs: fmt(*med([s.get("arrival_spread") for f, s in rs]))),
        ("arrival spread <= 10 s", lambda rs: (lambda xs: f"{sum(1 for x in xs if x <= 10)} of {len(xs)}")([s["arrival_spread"] for f, s in rs if s.get("arrival_spread") is not None])),
        ("transit at t0-20: across / along median", lambda rs: fmt(*med([s["transit"][0] / max(1, s["transit"][1]) for f, s in rs if s.get("transit")]), d=2)),
        ("transit line / blob / stream", lambda rs: "/".join(tr(rs, c).split(" of")[0] for c in ("line", "blob", "stream")) + (f" of {sum(1 for f, s in rs if s.get('transit'))}")),
    ])

    def xch(rs, pred=lambda f, s: True):
        sel = [(f, s) for f, s in rs if pred(f, s) and s["lost"] + s["killed"] > 0]
        if not sel:
            return "-"
        k = sum(s["killed"] for f, s in sel)
        l = sum(s["lost"] for f, s in sel)
        return f"{100 * k / (k + l):.0f}% ({len(sel)} sides, {k + l:.0f} m)"
    sh = lambda cls: (lambda f, s: s["s0"].get("shape") == cls)
    def is_mixed(f, s):
        ts = s["s0"].get("core_types", {})
        return s["s0"].get("core_n", 0) >= 3 and any(rclass(n) == "short" for n in ts) and any(rclass(n) == "long" for n in ts)
    def is_pure(f, s):
        ts = s["s0"].get("core_types", {})
        return s["s0"].get("core_n", 0) >= 3 and len({rclass(n) for n in ts}) == 1
    table("Trade: metal-weighted exchange share (their loss / both losses), by the side's shape at contact", [
        ("all fights", lambda rs: xch(rs)),
        ("line at t0", lambda rs: xch(rs, sh("line"))),
        ("blob at t0", lambda rs: xch(rs, sh("blob"))),
        ("stream at t0", lambda rs: xch(rs, sh("stream"))),
        ("core across >= 300 (cores of 3+)", lambda rs: xch(rs, lambda f, s: s["s0"].get("core_n", 0) >= 3 and s["s0"]["across"] >= 300)),
        ("core across < 300 (cores of 3+)", lambda rs: xch(rs, lambda f, s: s["s0"].get("core_n", 0) >= 3 and s["s0"]["across"] < 300)),
        ("body in one group (cores of 3+)", lambda rs: xch(rs, lambda f, s: s["s0"].get("core_n", 0) >= 3 and s["s0"]["groups"] <= 1)),
        ("body in two or more groups (cores of 3+)", lambda rs: xch(rs, lambda f, s: s["s0"].get("core_n", 0) >= 3 and s["s0"]["groups"] >= 2)),
        ("wider front than the enemy's (3+ each)", lambda rs: xch(rs, lambda f, s: wider(f, s) is True)),
        ("narrower front than the enemy's (3+ each)", lambda rs: xch(rs, lambda f, s: wider(f, s) is False)),
        ("more soldiers in the body than the enemy", lambda rs: xch(rs, lambda f, s: more(f, s) is True)),
        ("fewer soldiers than the enemy", lambda rs: xch(rs, lambda f, s: more(f, s) is False)),
        ("mixed core (short and long types)", lambda rs: xch(rs, is_mixed)),
        ("pure core (one reach class)", lambda rs: xch(rs, is_pure)),
        ("arrived together (spread <= 10 s)", lambda rs: xch(rs, lambda f, s: s.get("arrival_spread") is not None and s["arrival_spread"] <= 10)),
        ("trickled (spread > 10 s)", lambda rs: xch(rs, lambda f, s: s.get("arrival_spread") is not None and s["arrival_spread"] > 10)),
        ("friend on the line for 1/3+ of units in reach at t0", lambda rs: xch(rs, lambda f, s: s["s0"].get("inreach", 0) >= 3 and s["s0"]["blocked"] / s["s0"]["inreach"] >= 1 / 3)),
        ("friend on the line for under 1/3", lambda rs: xch(rs, lambda f, s: s["s0"].get("inreach", 0) >= 3 and s["s0"]["blocked"] / s["s0"]["inreach"] < 1 / 3)),
    ])

    def so(rs, cls):
        vs = [v for f, s in rs for v in s["standoff"].get(cls, [])]
        return f"{statistics.median(vs):.2f} (n={len(vs)})" if vs else "-"
    table("Conduct in the fight", [
        ("dealt damage that landed on units that died, median", lambda rs: fmt(*med([s["on_killed"] for f, s in rs if s["dealt"] >= 500]), pct=True)),
        ("... in even fights (exchange 35-65%)", lambda rs: fmt(*med([s["on_killed"] for f, s in rs if s["dealt"] >= 500 and s["exchange"] is not None and 0.35 <= s["exchange"] <= 0.65]), pct=True)),
        ("most-hit enemy's share of a second's damage (2+ hit), median", lambda rs: fmt(*med([s["top_share"] for f, s in rs if s["dealt"] >= 500]), pct=True)),
        ("units below 40% health: survived the fight", lambda rs: (lambda a, b: f"{b} of {a} ({100 * b / a:.0f}%)" if a else "-")(sum(s["low"] for f, s in rs), sum(s["low_survived"] for f, s in rs))),
        ("units below 40%: stepped back 150+ within 8 s", lambda rs: (lambda a, b: f"{b} of {a} ({100 * b / a:.0f}%)" if a else "-")(sum(s["low"] for f, s in rs), sum(s["low_back"] for f, s in rs))),
        ("stand-off / reach, short (<=250), median unit-seconds", lambda rs: so(rs, "short")),
        ("stand-off / reach, line (300-450)", lambda rs: so(rs, "line")),
        ("stand-off / reach, long (450+)", lambda rs: so(rs, "long")),
        ("commander took hits in the fight", lambda rs: f"{sum(1 for f, s in rs if s['commander_hit'])} of {len(rs)}"),
    ])
    if show_orders:
        def od(rs, fn):
            os_ = [s["order"] for f, s in rs if s.get("order")]
            return fn(os_)
        table("The last group order (3+ participants, one frame) in the 30 s before contact", [
            ("sides with such an order", lambda rs: f"{sum(1 for f, s in rs if s.get('order'))} of {len(rs)}"),
            ("one point per unit (distinct targets >= 80% of units)", lambda rs: od(rs, lambda o: f"{sum(1 for x in o if x['distinct'] >= 0.8 * x['n'])} of {len(o)}" if o else "-")),
            ("a single point (one distinct target)", lambda rs: od(rs, lambda o: f"{sum(1 for x in o if x['distinct'] == 1)} of {len(o)}" if o else "-")),
            ("targets across / along, median", lambda rs: od(rs, lambda o: fmt(*med([x['across'] / max(1, x['along']) for x in o if 'across' in x and x['distinct'] >= 3]), d=2))),
            ("targets across, median elmos", lambda rs: od(rs, lambda o: fmt(*med([x['across'] for x in o if 'across' in x and x['distinct'] >= 3])))),
            ("targets' minor / major principal extent, median (0 = one straight line)", lambda rs: od(rs, lambda o: fmt(*med([x.get('straight') for x in o if x['distinct'] >= 3]), d=2))),
            ("straight lines (minor/major < 0.2): at 60+ degrees to the travel / 30-60 / under 30", lambda rs: od(rs, lambda o: (lambda a: f"{sum(1 for x in a if x >= 60)} / {sum(1 for x in a if 30 <= x < 60)} / {sum(1 for x in a if x < 30)} of {len(a)}" if a else "-")(
                [math.degrees(math.atan2(x['across'], max(x['along'], 1))) for x in o if x['distinct'] >= 3 and 'across' in x and x.get('straight') is not None and x['straight'] < 0.2]))),
            ("target spacing, median elmos", lambda rs: od(rs, lambda o: fmt(*med([x.get('spacing') for x in o if x['distinct'] >= 3])))),
        ])
    # by size
    print("\n## By fight size (participants of both sides): fights; shape at t0 line/blob/stream; friend on the line at t0; exchange")
    print("| pool | size | fight sides | line/blob/stream at t0 | friend on the line t0 | exchange |")
    print("|---|---|---|---|---|---|")
    for n in names:
        for lo, hi, lab in SIZES:
            rs = [(f, s) for f, s in P[n] if lo <= f["size"] <= hi]
            if not rs:
                continue
            ss = [s["s0"].get("shape") for f, s in rs if s["s0"].get("shape")]
            a = sum(s["s0"].get("inreach", 0) for f, s in rs)
            b = sum(s["s0"].get("blocked", 0) for f, s in rs)
            print(f"| {n} | {lab} | {len(rs)} | {'/'.join(str(sum(1 for x in ss if x == c)) for c in ('line', 'blob', 'stream'))} of {len(ss)} | "
                  f"{b} of {a}{f' ({100 * b / a:.0f}%)' if a else ''} | {xch(rs)} |")


def bootstrap(fights, n=2000, seed=1):
    """Pros only: the exchange-share difference between shapes at contact, with a 95% interval from resampling games."""
    import random
    rnd = random.Random(seed)
    rows = [(f["game"], s) for f in fights if f["kind"] == "pro" for s in f["sides"] if s["lost"] + s["killed"] > 0]
    by = collections.defaultdict(list)
    for gl, s in rows:
        by[gl].append(s)
    games = sorted(by)

    def ex(ss):
        k = sum(s["killed"] for s in ss)
        l = sum(s["lost"] for s in ss)
        return k / (k + l) if k + l else None
    sel = {
        "line": lambda s: s["s0"].get("core_n", 0) >= 3 and s["s0"]["shape"] == "line",
        "blob": lambda s: s["s0"].get("core_n", 0) >= 3 and s["s0"]["shape"] == "blob",
        "stream": lambda s: s["s0"].get("core_n", 0) >= 3 and s["s0"]["shape"] == "stream",
        "friend on the line 1/3+": lambda s: s["s0"].get("inreach", 0) >= 3 and s["s0"]["blocked"] / s["s0"]["inreach"] >= 1 / 3,
        "friend on the line < 1/3": lambda s: s["s0"].get("inreach", 0) >= 3 and s["s0"]["blocked"] / s["s0"]["inreach"] < 1 / 3,
        "mixed core": lambda s: s["s0"].get("core_n", 0) >= 3 and any(rclass(t) == "short" for t in s["s0"]["core_types"]) and any(rclass(t) == "long" for t in s["s0"]["core_types"]),
        "pure core": lambda s: s["s0"].get("core_n", 0) >= 3 and len({rclass(t) for t in s["s0"]["core_types"]}) == 1,
    }
    print(f"\n## Exchange differences, pros, 95% interval over {n} resamples of the {len(games)} games")
    print("| a - b | difference | interval | fight sides a / b | unweighted mean exchange a / b |")
    print("|---|---|---|---|---|")
    for a, b in (("line", "blob"), ("line", "stream"), ("blob", "stream"), ("friend on the line 1/3+", "friend on the line < 1/3"), ("mixed core", "pure core")):
        A = [s for gl, s in rows if sel[a](s)]
        B = [s for gl, s in rows if sel[b](s)]
        diffs = []
        for _ in range(n):
            smp = [s for gl in rnd.choices(games, k=len(games)) for s in by[gl]]
            x, y = ex([s for s in smp if sel[a](s)]), ex([s for s in smp if sel[b](s)])
            if x is not None and y is not None:
                diffs.append(x - y)
        diffs.sort()
        lo, hi = diffs[int(0.025 * len(diffs))], diffs[int(0.975 * len(diffs))]
        print(f"| {a} - {b} | {100 * (ex(A) - ex(B)):+.1f} | {100 * lo:+.1f} .. {100 * hi:+.1f} | {len(A)} / {len(B)} | "
              f"{100 * statistics.mean(s['exchange'] for s in A):.0f}% / {100 * statistics.mean(s['exchange'] for s in B):.0f}% |")


def wider(f, s):
    o = f["sides"][1 - s["side"]]
    if s["s0"].get("core_n", 0) < 3 or o["s0"].get("core_n", 0) < 3:
        return None
    if abs(s["s0"]["across"] - o["s0"]["across"]) < 50:
        return None
    return s["s0"]["across"] > o["s0"]["across"]


def more(f, s):
    o = f["sides"][1 - s["side"]]
    if s["s0"].get("n", 0) == o["s0"].get("n", 0):
        return None
    return s["s0"].get("n", 0) > o["s0"].get("n", 0)


def print_fights(fights):
    print("| game | clock | size | side | n t0 | across x along t0 | shape t0 | n t0+10 | across x along t0+10 | friend on line t0 | arrival spread | lost | killed |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for f in fights:
        for s in f["sides"]:
            a, b = s["s0"], s["s10"]
            print(f"| {f['game']} | {f['t0'] // 60}:{f['t0'] % 60:02d} | {f['size']} | {s['player']}{' (W)' if s['won_game'] else ''} | {a['n']} | "
                  f"{a.get('across', 0):.0f} x {a.get('along', 0):.0f} | {a.get('shape') or '-'} | {b['n']} | {b.get('across', 0):.0f} x {b.get('along', 0):.0f} | "
                  f"{a.get('blocked', 0)}/{a.get('inreach', 0)} | {s.get('arrival_spread', '-')} | {s['lost']:.0f} | {s['killed']:.0f} |")


def main():
    global T
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--map", default="Comet Catcher Remake 1.8")
    ap.add_argument("--floor", type=float, default=40)
    ap.add_argument("--ours", nargs="*", default=[
        "run/matches/1790257668-bank-1/00", "run/matches/1790261454-2v1b-hard/00", "run/matches/1790263097-2v1b-hard_aggressive/00"])
    ap.add_argument("--fights", action="store_true", help="print every fight side")
    ap.add_argument("--orders", action="store_true", help="the group-order table")
    ap.add_argument("--no-pros", action="store_true")
    ap.add_argument("--boot", action="store_true", help="bootstrap intervals for the pros' exchange by shape")
    a = ap.parse_args()
    T = load_tables()
    games = [] if a.no_pros else pro_games(a.map, a.floor)
    print(f"pro games: {len(games)} ({a.map}, both players OS {a.floor:g}+)")
    for p in a.ours:
        path = p if os.path.isabs(p) else os.path.join(REPO, p)
        if os.path.exists(path):
            games.append(our_game(path))
        else:
            print(f"missing: {p}", file=sys.stderr)
    fights = []
    for g in games:
        for f in find_fights(g):
            fights.append(measure(g, f))
    if a.fights:
        print_fights(fights)
    report(fights, a.orders)
    if a.boot and not a.no_pros:
        bootstrap(fights)


if __name__ == "__main__":
    main()
