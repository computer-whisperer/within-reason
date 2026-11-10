#!/usr/bin/env python3
"""Where a declared group's members stood, minute by minute, from a seat's record (the sample's `groups` and `own`,
docs/harness/record-format.md): how many members, their metal, the cell of their centroid, the share of them within
1,200 of it, the westmost member's cell and the members by cell. A group that is one body reads near 100 %; a
string from the plants to the front reads a third (the lobby 10v8 on Erebos Lakes: never over 62 % from 12:00).

    run/group_where.py <record-N.jsonl> <group name as the lane has it, group_A_t14> [step seconds, default 60]
"""
import json, sys, math, collections
rec, name = sys.argv[1], sys.argv[2]
step = int(sys.argv[3]) if len(sys.argv) > 3 else 60
def cell(x, z): return 'ABCDEFGHIJ'[min(int(x // 1024), 9)] + str(min(int(z // 1024), 9) + 1)
f = open(rec); hdr = json.loads(f.readline()); defs = hdr['unit_defs']
for line in f:
    if not line.startswith('{"t":"s"') and '"t":"s"' not in line[:30]: continue
    r = json.loads(line); s = r['f'] // 30
    if s % step: continue
    g = (r.get('groups') or {}).get(name)
    if not g: continue
    own = {u[0]: u for u in r['own']}
    ids = [u for u in list(g['formed']) + [e[0] for e in g['exceptions']] if u in own]
    if not ids: continue
    pts = [(own[u][2], own[u][3]) for u in ids]
    cx = sum(p[0] for p in pts) / len(pts); cz = sum(p[1] for p in pts) / len(pts)
    near = sum(1 for p in pts if math.dist(p, (cx, cz)) < 1200)
    cells = collections.Counter(cell(*p) for p in pts)
    metal = sum(defs[own[u][1]].get('metal', 0) for u in ids)
    xs = sorted(p[0] for p in pts)
    print('%2d:%02d %s n=%3d metal %6d centroid %s, %3.0f%% within 1,200; westmost %s; cells %s' % (s // 60, s % 60, name, len(ids), metal, cell(cx, cz), 100 * near / len(pts), cell(*min(pts)), ' '.join('%s:%d' % kv for kv in sorted(cells.items(), key=lambda kv: -kv[1])[:9])))
