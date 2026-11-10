#!/usr/bin/env python3
"""A walk drill's crossing times (`duel --drill walk`; docs/design/2026-11-10-restricted-ground.md §3).

    run/drill_cross.py ALONG <batch dir> [<batch dir> ...]

Per duel of each batch, from the per-second logs (`seconds/*.jsonl`): the second, from the first order, at which the
first, the median and the last mobile unit of side x stood ALONG elmos past the start along the axis to the goal, and
how many never did. A wall's or a yard's far side as ALONG is the pass itself, without the rest of the walk and the
gathering at the goal (five companies at one goal time out on open ground too)."""
import json, sys, glob, os, statistics
along = float(sys.argv[1])
for d in sys.argv[2:]:
    rows = []
    for f in sorted(glob.glob(os.path.join(d, 'seconds', '*.jsonl'))):
        meta = None; first = {}; start_pos = {}; moved = set(); t0 = None
        for line in open(f):
            r = json.loads(line)
            if 'meta' in r:
                meta = r['meta']; sx, sz = meta['start']; gx, gz = meta['goal']
                n = ((gx - sx) ** 2 + (gz - sz) ** 2) ** 0.5; ax, az = (gx - sx) / n, (gz - sz) / n
                continue
            t = r['t']
            if r.get('sent', [0, 0])[0] and t0 is None: t0 = t
            for u in r['x']:
                i, x, z = u[0], u[1], u[2]
                if i not in start_pos: start_pos[i] = (x, z)
                elif abs(x - start_pos[i][0]) + abs(z - start_pos[i][1]) > 60: moved.add(i)
                if i not in first and (x - sx) * ax + (z - sz) * az >= along: first[i] = t
        t0 = t0 or 0
        times = sorted(first[i] - t0 for i in moved if i in first)
        never = len([i for i in moved if i not in first])
        if times: rows.append((times[0], times[len(times) // 2], times[-1], never, len(moved)))
    name = os.path.basename(d.rstrip('/')).split('-', 2)[-1]
    if rows:
        print('%-34s past %4.0f: first %s  median %s  last %s  never %s of %d' % (name, along, '/'.join(str(r[0]) for r in rows), '/'.join(str(r[1]) for r in rows), '/'.join(str(r[2]) for r in rows), '/'.join(str(r[3]) for r in rows), rows[0][4]))
