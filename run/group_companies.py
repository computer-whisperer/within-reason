#!/usr/bin/env python3
"""One declared group's companies over time, from a seat's record (the sample's `groups`,
docs/harness/record-format.md): the lead's cell (the anchor the group's top-level numbers and the report's place
are taken from: the last of the largest marching companies), its pace against the full pace, the way left, the
legs, ARR / ENG / PASSn (arrived, engaged, members through a relaxed pass), then every company as
cell:members, m marching or u mustering, formed (A arrived), and the exceptions by cause.

    run/group_companies.py <record-N.jsonl> <group name as the lane has it, group_A_t14> [step seconds, default 30]
"""
import json, sys
rec, name = sys.argv[1], sys.argv[2]
step = int(sys.argv[3]) if len(sys.argv) > 3 else 30
def cell(x, z): return 'ABCDEFGHIJ'[min(int(x // 1024), 9)] + str(min(int(z // 1024), 9) + 1)
f = open(rec); f.readline()
for line in f:
    if '"t":"s"' not in line[:20]: continue
    r = json.loads(line)
    s = r['f'] // 30
    if s % step: continue
    g = (r.get('groups') or {}).get(name)
    if not g: continue
    comps = g.get('companies') or []
    cs = ' '.join('%s:%d%s%d' % (cell(*c['anchor']), c['members'], 'm' if c['role'] == 'marching' else 'u', c['formed']) + ('A' if c['arrived'] else '') for c in comps)
    n = len(g['formed']) + len(g['exceptions'])
    ex = {}
    for e in g['exceptions']: ex[e[1]] = ex.get(e[1], 0) + 1
    print('%2d:%02d %s lead %s pace %2.0f/%2.0f way %s legs %s%s%s%s | %d comps: %s | %s' % (s // 60, s % 60, name, cell(*g['anchor']), g['pace'], g['full_pace'], '-' if g.get('way') is None else int(g['way']), g['legs_left'], ' ARR' if g['arrived'] else '', ' ENG' if g['engaged'] is not None else '', (' PASS%d' % g['passing']) if g.get('passing') is not None else '', len(comps), cs, ex))
