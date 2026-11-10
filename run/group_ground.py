#!/usr/bin/env python3
"""How a game's formations sat on the ground while they marched (docs/design/2026-11-10-restricted-ground.md §0).

    run/group_ground.py <match seat dir> [hull pitch 1|0] [min members]

For every second the largest marching company of a declared group (members >= min, 8; an order in force, not arrived,
nobody engaged, not passing) was on the march: the six-file ranks are laid at the recorded anchor on the recorded
heading (the record keeps the travel's heading, not the ranks' facing, which lags it by up to 60 degrees: an
approximation), at the pitch from the members' hulls (`1`, the default, games from 46e3ce5b on; `0` for 64 by 96
before it), and judged against the ground the lane builds: the members' strictest class over the terrain, less every
structure of ours and of theirs in sight by its footprint (wrecks are not in the record).

Per slot: `off` its cell is not ground; `no ground` none within the snap's six cells; `crowded` after the snap
another slot stands nearer than the members' median hull; `detour` the walk from the anchor to the snapped slot is
over twice the straight line plus 32, or there is none within the window. A second is `clean` when no slot is any of
these, `some` up to a quarter of the slots, `many` above. Printed: the seconds, the anchor's pace as a share of full,
the share of seconds under half pace and the formed share, for all, within 1,500 of a plant of ours and further out,
by those three; and the seconds a company was passing a stretch in the relaxed posture (H-MICRO-RELAX), which are
left out of the rest. A record from before the unit definitions carried the pathing hull takes 40 for every type
(the crowded count alone reads it).
"""
import json, sys, math, heapq, collections
import numpy as np
from scipy import ndimage

seat = sys.argv[1]
HULL = (sys.argv[2] != '0') if len(sys.argv) > 2 else True
MIN = int(sys.argv[3]) if len(sys.argv) > 3 else 8
FILES, SNAP, WIN = 6, 6, 56

import glob, os
REC = sorted(glob.glob(seat + '/record-*.jsonl'))[0]
hdr = json.loads(open(REC).readline())
HULL_OF = {d['name']: d['move'][4] * 8.0 for d in hdr['unit_defs'] if d['move'] and len(d['move']) > 4}
T = hdr['terrain']; W, H, CELL = T['width'], T['height'], T['cell']
raw = open(seat + '/' + T['file'], 'rb').read()
heights = np.frombuffer(raw[:W * H * 2], dtype='<i2').reshape(H, W).astype(np.float32)
slopes = np.frombuffer(raw[W * H * 2:W * H * 3], dtype=np.uint8).reshape(H, W).astype(np.int32)
defs = hdr['unit_defs']

def class_grid(max_slope, depth, _cache={}):
    key = (round(max_slope * 255), round(depth * 10))
    if key not in _cache:
        _cache[key] = (slopes <= key[0]) & (heights >= -depth)
    return key, _cache[key]

def structures(units, _cache={}):
    key = frozenset(units)
    if key not in _cache:
        if len(_cache) > 64: _cache.clear()
        g = np.ones((H, W), dtype=bool)
        for (dx, dz, x, z) in units:
            hx, hz = max(dx, 1) * 4.0, max(dz, 1) * 4.0
            a, b = max(int((x - hx) // CELL), 0), min(int((x + hx - 0.01) // CELL), W - 1)
            c, e = max(int((z - hz) // CELL), 0), min(int((z + hz - 0.01) // CELL), H - 1)
            if a <= b and c <= e: g[c:e + 1, a:b + 1] = False
        _cache[key] = g
    return _cache[key]

def slots(anchor, h, count, across, deep):
    left = (-h[1], h[0]); ranks = max(-(-count // FILES), 1); front = min(FILES, count); out = []
    for rank in range(ranks):
        in_rank = min(count - rank * FILES, FILES)
        off = across / 2 if rank % 2 == 1 and in_rank % 2 == front % 2 else 0.0
        back = rank * deep
        for f in range(in_rank):
            side = (f - (in_rank - 1) / 2) * across + off
            out.append((anchor[0] + left[0] * side - h[0] * back, anchor[1] + left[1] * side - h[1] * back))
    return out

def snap(g, p):
    cx, cz = int(p[0] / CELL), int(p[1] / CELL); best = None
    for dz in range(-SNAP, SNAP + 1):
        for dx in range(-SNAP, SNAP + 1):
            x, z = cx + dx, cz + dz
            if 0 <= x < W and 0 <= z < H and g[z, x]:
                a = dx * dx + dz * dz
                if best is None or a < best[0]: best = (a, x, z)
    return None if best is None else (best[1], best[2])

def walks(g, start, targets):
    """Octile walking distance in elmos from cell `start` to each target cell, inside a window of WIN cells."""
    sx, sz = start; want = set(targets); dist = {start: 0.0}; heap = [(0.0, sx, sz)]; out = {}
    while heap and want:
        d, x, z = heapq.heappop(heap)
        if d > dist.get((x, z), 1e18): continue
        if (x, z) in want: out[(x, z)] = d; want.discard((x, z))
        for dx, dz, c in ((1, 0, 1), (-1, 0, 1), (0, 1, 1), (0, -1, 1), (1, 1, 1.4142), (1, -1, 1.4142), (-1, 1, 1.4142), (-1, -1, 1.4142)):
            nx, nz = x + dx, z + dz
            if abs(nx - sx) > WIN or abs(nz - sz) > WIN or not (0 <= nx < W and 0 <= nz < H) or not g[nz, nx]: continue
            if dx and dz and not (g[z, nx] and g[nz, x]): continue
            nd = d + c * CELL
            if nd < dist.get((nx, nz), 1e18): dist[(nx, nz)] = nd; heapq.heappush(heap, (nd, nx, nz))
    return out

rows = []
passing_s = 0
for line in open(REC):
    try: r = json.loads(line)
    except Exception: continue
    if r.get('t') != 's' or not r.get('groups'): continue
    own = {u[0]: u for u in r['own']}
    built = [(defs[u[1]]['footprint'][0], defs[u[1]]['footprint'][1], u[2], u[3]) for u in r['own'] if defs[u[1]]['speed'] == 0 and defs[u[1]]['move'] is None]
    built += [(defs[e[1]]['footprint'][0], defs[e[1]]['footprint'][1], e[2], e[3]) for e in r.get('en', []) if e[1] >= 0 and defs[e[1]]['speed'] == 0 and defs[e[1]]['move'] is None]
    plants = [(u[2], u[3]) for u in r['own'] if defs[u[1]]['class'] == 'factory']
    for name, g in r['groups'].items():
        full = g.get('full_pace') or 0
        if g.get('passing') is not None:
            passing_s += 1
            continue
        if not (full > 0 and not g.get('arrived') and g.get('engaged') is None and (g.get('legs_left') or 0) > 0): continue
        comp = next((c for c in g.get('companies', []) if c['role'] == 'marching' and c['anchor'] == g['anchor']), None)
        n = comp['members'] if comp else len(g['formed']) + len(g['exceptions'])
        if n < MIN: continue
        anchor = tuple(g['anchor']); h = tuple(g['heading'])
        hl = math.hypot(*h)
        if hl < 0.5: continue
        h = (h[0] / hl, h[1] / hl)
        ids = [u for u in list(g['formed']) + [e[0] for e in g['exceptions']] if u in own]
        ids.sort(key=lambda u: math.dist((own[u][2], own[u][3]), anchor))
        mine = ids[:n]
        moves = [defs[own[u][1]]['move'] for u in mine if defs[own[u][1]]['move']]
        if not moves: continue
        hulls = sorted((HULL_OF.get(defs[own[u][1]]['name'], 40.0) for u in mine if defs[own[u][1]]['move']), reverse=True)
        hull = hulls[0] if len(hulls) == 1 else (hulls[0] + hulls[1]) / 2
        med = hulls[len(hulls) // 2]
        across, deep = (max(64.0, 1.5 * hull), max(96.0, 1.5 * hull)) if HULL else (64.0, 96.0)
        _, cg = class_grid(min(m[1] for m in moves), min(m[2] for m in moves))
        ground = cg & structures(tuple(built))
        S = slots(anchor, h, n, across, deep)
        off = nocell = crowd = detour = 0; snapped = []
        for p in S:
            cx, cz = int(p[0] / CELL), int(p[1] / CELL)
            on = 0 <= cx < W and 0 <= cz < H and ground[cz, cx]
            if on: snapped.append((cx, cz)); continue
            off += 1
            s = snap(ground, p)
            if s is None: nocell += 1; snapped.append(None)
            else: snapped.append(s)
        pts = [None if s is None else ((s[0] + 0.5) * CELL, (s[1] + 0.5) * CELL) for s in snapped]
        raw_on = [int(p[0] / CELL) == s[0] and int(p[1] / CELL) == s[1] if s else False for p, s in zip(S, snapped)]
        use = [S[i] if raw_on[i] else pts[i] for i in range(n)]
        bad = [s is None for s in snapped]
        for i in range(n):
            if use[i] is None: continue
            if any(j != i and use[j] is not None and math.dist(use[i], use[j]) < med - 0.5 for j in range(n)):
                crowd += 1; bad[i] = True
        a = snap(ground, anchor)
        if a is not None:
            w = walks(ground, a, [s for s in snapped if s is not None])
            for i, s in enumerate(snapped):
                if s is None: continue
                straight = math.dist(((a[0] + 0.5) * CELL, (a[1] + 0.5) * CELL), ((s[0] + 0.5) * CELL, (s[1] + 0.5) * CELL))
                if s not in w or w[s] > 2 * straight + 32: detour += 1; bad[i] = True
        for i in range(n):
            if not raw_on[i]: bad[i] = True
        nbad = sum(bad)
        home = min((math.dist(anchor, p) for p in plants), default=1e9)
        exc = collections.Counter(e[1] for e in g['exceptions'] if e[0] in set(mine))
        rows.append(dict(s=r['f'] // 30, g=name, n=n, pace=g['pace'] / full, formed=(comp['formed'] / n if comp else len(g['formed']) / n), off=off, nocell=nocell, crowd=crowd, detour=detour, bad=nbad, home=home, exc=exc, across=across))

def show(label, rs):
    if not rs: print('%-44s none' % label); return
    k = len(rs); ms = sum(r['n'] for r in rs)
    print('%-44s %5d s  pace %.2f  under half %3.0f%%  formed %.2f  | slots off ground %4.1f%%  crowded %4.1f%%  detour %4.1f%%  no ground %4.1f%%' % (
        label, k, sum(r['pace'] for r in rs) / k, 100 * sum(r['pace'] < 0.5 for r in rs) / k, sum(r['formed'] * r['n'] for r in rs) / ms,
        100 * sum(r['off'] for r in rs) / ms, 100 * sum(r['crowd'] for r in rs) / ms, 100 * sum(r['detour'] for r in rs) / ms, 100 * sum(r['nocell'] for r in rs) / ms))

def kind(r):
    share = r['bad'] / r['n']
    return 'clean' if share == 0 else 'some' if share <= 0.25 else 'many'

print(seat, 'hull pitch' if HULL else 'pitch 64 x 96', 'members >=', MIN)
show('all marching seconds', rows)
for where, test in (('within 1,500 of a plant of ours', lambda r: r['home'] < 1500), ('further out', lambda r: r['home'] >= 1500)):
    part = [r for r in rows if test(r)]
    show(where, part)
    for k in ('clean', 'some', 'many'):
        show('  ranks %s' % k, [r for r in part if kind(r) == k])
exc = collections.Counter(); ms = 0
for r in rows: exc.update(r['exc']); ms += r['n']
print('the lane\'s own exceptions, share of member-seconds: ' + ', '.join('%s %.3f' % (k, v / ms) for k, v in exc.most_common()))
print('group-seconds passing a stretch in the relaxed posture: %d' % passing_s)
