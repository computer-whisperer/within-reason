#!/usr/bin/env python3
"""Measure how a side lays its base out, from a match record (ours, or a person's replay played through
run/replay_match.py). The study: docs/studies/2026-10-29-base-layout.md; the derivation: docs/briefs/derived/base.md.

    run/base_layout.py pool <min_os> <match dir>...     one JSON line a seat: every seat of a carded replay at or above
                                                        the OS (card.json names the players; record-<team>.jsonl beside it)
    run/base_layout.py pool 0 <record.jsonl>...         the same for bare records (ours: run/matches/<batch>/<NN>/record-0.jsonl)
    run/base_layout.py block <record.jsonl>             the construction-turret block of one side: pitch, bars, what stands
                                                        in it with the turrets in reach when placed, the chain bill
    run/base_layout.py pen [--allies] <record.jsonl>... per land factory (with --allies, allied structures block too), the first minute a unit of each footprint (2 to 7
                                                        squares) can no longer walk from its exit to ground 1,200 away through
                                                        our own buildings (and the terrain where the record has it)
    run/base_layout.py summary <pool.jsonl>             the pooled medians of a pool file

Footprints, blasts and health come from the game's unit files under upstream/Beyond-All-Reason (cached as
docs/studies/data/base-layout-2026-10-29/unitfacts.json when the checkout is missing). Death blasts: radius is half
the file's areaOfEffect, damage falls linearly to the radius, measured to the victim's surface (F-fight-death).
"""
import json, sys, os, glob, math, re, collections, statistics as st

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(REPO, 'docs', 'studies', 'data', 'base-layout-2026-10-29', 'unitfacts.json')

def unit_facts():
    units = os.path.join(REPO, 'upstream', 'Beyond-All-Reason', 'units')
    if not os.path.isdir(units):
        return json.load(open(CACHE))
    out = {}
    for p in glob.glob(units + '/**/*.lua', recursive=True):
        s = open(p, errors='ignore').read()
        m = re.search(r'return\s*\{\s*(\w+)\s*=\s*\{', s)
        if not m: continue
        def g(k, conv=float):
            mm = re.search(r'\n\t\t' + k + r'\s*=\s*([^,\n]+)', s)
            if not mm: return None
            v = mm.group(1).strip().strip('"')
            try: return conv(v)
            except Exception: return v
        out[m.group(1)] = {'fx': g('footprintx'), 'fz': g('footprintz'), 'explodeas': g('explodeas', str), 'health': g('health'),
                           'bd': g('builddistance'), 'mass': g('mass'), 'notransport': g('cantbetransported', str), 'yardmap': g('yardmap', str)}
    try:
        os.makedirs(os.path.dirname(CACHE), exist_ok=True); json.dump(out, open(CACHE, 'w'))
    except OSError: pass
    return out

FACTS = unit_facts()
COMMANDERS = {'armcom', 'corcom', 'legcom'}
NANOS = {'armnanotc', 'cornanotc', 'legnanotc', 'armnanotcplat', 'cornanotcplat'}
FACTORIES = {'armlab', 'armvp', 'armap', 'armhp', 'armalab', 'armavp', 'armaap', 'armahp', 'armshltx', 'armsy', 'armasy',
             'corlab', 'corvp', 'corap', 'corhp', 'coralab', 'coravp', 'coraap', 'corahp', 'corgant', 'corsy', 'corasy',
             'leglab', 'legvp', 'legap', 'leghp', 'legalab', 'legavp', 'legaap', 'legahp', 'leggant'}
NO_LANE = {'armap', 'armaap', 'armsy', 'armasy', 'corap', 'coraap', 'corsy', 'corasy', 'legap', 'legaap'}
BUILDER_MOBILE = {'armck', 'armcv', 'armca', 'armack', 'armacv', 'armaca', 'armrectr', 'armcs', 'armacsub', 'armbeaver', 'armch', 'armfark', 'armconsul',
                  'corck', 'corcv', 'corca', 'corack', 'coracv', 'coraca', 'cornecro', 'corcs', 'coracsub', 'cormuskrat', 'corch', 'corfast',
                  'legck', 'legcv', 'legca', 'legack', 'legacv', 'legaca', 'legaceb', 'legotter'}
# death blast (radius, damage at the centre) by explodeas, weapons/Unit_Explosions.lua; radius = areaOfEffect / 2
BLAST = {'customfusionexplo': (640, 10600), 'fusionexplosion': (240, 2650), 'fusionexplosion-uw': (240, 2650), 'energystorage': (210, 980),
         'metalmaker': (105, 660), 'advmetalmaker': (160, 1220), 'geo': (260, 1420), 'largebuildingexplosiongeneric': (170, 670),
         'hugebuildingexplosiongeneric': (210, 1330), 'mediumbuildingexplosiongeneric': (130, 220), 'smallbuildingexplosiongeneric': (90, 89),
         'commanderexplosion': (350, 5000), 'nanoboom': (64, 11), 'windboom': (90, 89)}
KINDS = {'solar': {'armsolar', 'corsolar', 'legsolar'}, 'wind': {'armwin', 'corwin', 'legwin'}, 'advsol': {'armadvsol', 'coradvsol', 'legadvsol'},
         'makr': {'armmakr', 'cormakr', 'legmakr'}, 'mmkr': {'armmmkr', 'cormmkr', 'legmmkr'}, 'fus': {'armfus', 'corfus', 'legfus', 'armckfus'},
         'afus': {'armafus', 'corafus', 'legafus'}, 'estor': {'armestor', 'corestor', 'legestor'}, 'mstor': {'armmstor', 'cormstor', 'legmstor'},
         'mex': {'armmex', 'cormex', 'legmex'}, 'moho': {'armmoho', 'cormoho', 'legmoho'}, 'nano': NANOS, 'fac': FACTORIES}

def kind_of(name):
    for k, s in KINDS.items():
        if name in s: return k
    return None

def half(name):
    """Half the footprint in elmos (a footprint square is 16 elmos): a 3x3 turret is 24, a 6x6 plant 48."""
    f = FACTS.get(name) or {}
    return int((f.get('fx') or 2) * 8), int((f.get('fz') or 2) * 8)

def dist(a, b): return math.hypot(a[0] - b[0], a[1] - b[1])

def kills(blast_name, victim_name, d_centres):
    """Whether a death blast of that kind at full damage kills a full-health victim whose centre is d away."""
    rad, dmg = BLAST.get((blast_name or '').lower(), (0, 0))
    hp = (FACTS.get(victim_name) or {}).get('health') or 1e9
    dsurf = max(0.0, d_centres - max(half(victim_name)))
    return rad > 0 and dsurf <= rad and dmg * (rad - dsurf) / rad >= hp

def load(record):
    f = open(record)
    h = json.loads(f.readline())
    defs = {i: d for i, d in enumerate(h['unit_defs'])}
    created, finished, destroyed, cmds, samples = {}, {}, {}, [], []
    for l in f:
        try: r = json.loads(l)
        except Exception: continue
        t = r.get('t')
        if t == 'ev':
            k = r['k']
            if k == 'created': created[r['u']] = r
            elif k == 'finished': finished[r['u']] = r
            elif k == 'destroyed': destroyed[r['u']] = r
        elif t == 'cmd': cmds.append(r)
        elif t == 's': samples.append(r)
    return h, defs, created, finished, destroyed, cmds, samples

# ---------------------------------------------------------------- pool: one seat's layout numbers
def analyse(record, player):
    h, defs, created, finished, destroyed, cmds, samples = load(record)
    name = lambda d: defs.get(d, {}).get('name', '?')
    fps = h.get('frames_per_second', 30)
    out = {'record': record, 'player': player.get('name'), 'os': player.get('os'), 'map': h['map']['name'], 'faction': player.get('faction'),
           'won': player.get('won'), 'start_x': (player.get('start') or {}).get('x'), 'frames': max([r['f'] for r in samples] or [0])}
    out['minutes'] = out['frames'] / fps / 60
    pos_at = {s['f']: {o[0]: (o[2], o[3]) for o in s.get('own', [])} for s in samples}
    sample_frames = sorted(pos_at)
    def pos_near(uid, f):
        lo, hi, best = 0, len(sample_frames) - 1, None
        while lo <= hi:
            mid = (lo + hi) // 2
            if sample_frames[mid] <= f: best = sample_frames[mid]; lo = mid + 1
            else: hi = mid - 1
        return None if best is None else pos_at[best].get(uid)
    buildings = {u: r for u, r in created.items() if (defs.get(r['d'], {}).get('speed') or 0) == 0}
    com = next((u for u, r in created.items() if name(r['d']) in COMMANDERS), None)
    out['commander'] = name(created[com]['d']) if com else None
    P = lambda u: (created[u]['x'], created[u]['z'])

    # A. the opening: the commander until the first factory stands (plus 30 s), at most 150 s
    first_fac = min([finished[u]['f'] for u in buildings if u in finished and name(buildings[u]['d']) in FACTORIES] or [10**9])
    horizon = min(first_fac + 30 * fps, 150 * fps)
    path, still, prev = 0.0, 0, None
    for f in sample_frames:
        if f > horizon: break
        p = pos_at[f].get(com)
        if p and prev:
            d = dist(p, prev); path += d if d > 1 else 0; still += 1 if d <= 1 else 0
        prev = p
    out['open_horizon_s'] = horizon / fps
    out['open_com_path'] = round(path)
    out['open_com_still_s'] = still
    out['first_factory_s'] = first_fac / fps if first_fac < 10**9 else None
    com_builds = [(r['f'], c) for r in cmds for c in r['c'] if isinstance(c, list) and c[0] == 'build' and c[1] == com and r['f'] <= horizon]
    out['open_com_build_orders'] = len(com_builds)
    groups, last = [], None
    for f, c in com_builds:
        if last is not None and f - last <= 2 * fps: groups[-1].append((f, c))
        else: groups.append([(f, c)])
        last = f
    out['open_queue_groups'] = len(groups)
    out['open_queue_depths'] = [len(g) for g in groups]
    reach_hits, sites = 0, []
    for f, c in com_builds:
        p = pos_near(com, f); site = (c[3], c[4])
        if p is not None and dist(p, site) <= 145 + max(half(name(c[2]))): reach_hits += 1
        sites.append(site)
    out['open_sites_in_reach_at_order'] = reach_hits
    out['open_site_steps'] = [round(dist(a, b)) for a, b in zip(sites, sites[1:])]
    com_done = sorted([(finished[u]['f'], u) for u in buildings if u in finished and finished[u]['f'] <= first_fac
                       and pos_near(com, created[u]['f']) is not None and dist(pos_near(com, created[u]['f']), P(u)) <= 145 + 60 + 30])
    out['open_structures_before_factory'] = [name(buildings[u]['d']) for _, u in com_done]
    out['open_gaps_s'] = [round((created[u2]['f'] - f1) / fps, 1) for (f1, u1), (f2, u2) in zip(com_done, com_done[1:])]
    stands = []
    for _, u in com_done:
        p = pos_near(com, created[u]['f'])
        if p and not any(dist(p, q) <= 24 for q in stands): stands.append(p)
    out['open_stands'] = len(stands)

    # standing and ever-finished buildings by kind
    end_f = out['frames']
    standing = {u for u in buildings if u in finished and u not in destroyed}
    everb = {u: r for u, r in buildings.items() if u in finished}
    bykind = collections.defaultdict(list)
    for u, r in everb.items():
        k = kind_of(name(r['d']))
        if k: bykind[k].append(u)
    out['counts_ever'] = {k: len(v) for k, v in bykind.items()}

    # B. the turrets
    nanos, facs = bykind.get('nano', []), bykind.get('fac', [])
    def alive(u, f): return finished[u]['f'] <= f and (u not in destroyed or destroyed[u]['f'] > f)
    near_fac, nn = [], []
    for u in nanos:
        f = finished[u]['f']
        near_fac.append(min([dist(P(u), P(v)) - max(half(name(created[v]['d']))) for v in facs if alive(v, f)] or [9999]))
        others = [dist(P(u), P(v)) for v in nanos if v != u and finished[v]['f'] <= f]
        nn.append(min(others) if others else None)
    out['nano_n'] = len(nanos)
    out['nano_within_400_of_factory_share'] = round(sum(1 for d in near_fac if d <= 400) / len(nanos), 2) if nanos else None
    out['nano_to_factory_edge_median'] = round(st.median(near_fac)) if near_fac else None
    nnv = [d for d in nn if d is not None]
    out['nano_nn_median'] = round(st.median(nnv)) if nnv else None
    out['nano_nn_q1_q3'] = [round(st.quantiles(nnv, n=4)[0]), round(st.quantiles(nnv, n=4)[2])] if len(nnv) >= 4 else None
    per_fac = []
    for v in facs:
        fv = destroyed[v]['f'] if v in destroyed else end_f
        hv = max(half(name(created[v]['d'])))
        per_fac.append((name(created[v]['d']), sum(1 for u in nanos if alive(u, fv) and dist(P(v), P(u)) <= 400 + hv), round(finished[v]['f'] / fps / 60, 1)))
    out['nanos_per_factory'] = per_fac
    pts = [P(u) for u in nanos if u in standing]
    parent = list(range(len(pts)))
    def find(i):
        while parent[i] != i: parent[i] = parent[parent[i]]; i = parent[i]
        return i
    for i in range(len(pts)):
        for j in range(i + 1, len(pts)):
            if dist(pts[i], pts[j]) <= 120: parent[find(i)] = find(j)
    out['nano_clusters_standing'] = sorted(collections.Counter(find(i) for i in range(len(pts))).values(), reverse=True)[:8]

    # C. nearest-neighbour spacing, centre to centre, over everything ever finished of the kind
    def nn_med(kind_a, kind_b=None):
        A = [P(u) for u in bykind.get(kind_a, [])]
        B = A if kind_b is None else [P(u) for u in bykind.get(kind_b, [])]
        ds = []
        for i, p in enumerate(A):
            cand = [dist(p, q) for j, q in enumerate(B) if kind_b is not None or j != i]
            if cand: ds.append(min(cand))
        return (round(st.median(ds)), len(ds)) if ds else None
    out['spacing'] = {k: nn_med(k) for k in ['solar', 'wind', 'makr', 'mmkr', 'fus', 'afus', 'nano', 'estor']}
    out['spacing_cross'] = {'fus-nano': nn_med('fus', 'nano'), 'afus-nano': nn_med('afus', 'nano'), 'estor-fac': nn_med('estor', 'fac'),
                            'estor-nano': nn_med('estor', 'nano'), 'mmkr-nano': nn_med('mmkr', 'nano'), 'solar-fac': nn_med('solar', 'fac'),
                            'fus-fac': nn_med('fus', 'fac'), 'afus-fac': nn_med('afus', 'fac')}

    # D. own reclaims (the record's `c: reclaimed`, or a death by one of our own builders)
    own_ids = set(created)
    def reclaimed(r): return r.get('c') == 'reclaimed' or (r.get('by') in own_ids and name(r.get('by_d')) in (BUILDER_MOBILE | NANOS | COMMANDERS))
    recl, recl_min = collections.Counter(), []
    for u, r in destroyed.items():
        if u in everb and reclaimed(r): recl[name(r['d'])] += 1; recl_min.append(round(r['f'] / fps / 60, 1))
    out['own_reclaims'] = dict(recl)
    out['own_reclaim_minutes'] = sorted(recl_min)

    # E. chain deaths: a building that died within 3 s of an exploding neighbour whose blast at full damage reaches half its health
    deaths = sorted([(r['f'], u) for u, r in destroyed.items() if u in everb and not reclaimed(r)])
    out['building_deaths'] = len(deaths)
    chain_victims, chain_sources, victims = collections.Counter(), collections.Counter(), set()
    for f, u in deaths:
        nm = name(created[u]['d']); ex = ((FACTS.get(nm) or {}).get('explodeas') or '').lower()
        rad, dmg = BLAST.get(ex, (0, 0))
        if rad < 100: continue
        for f2, v in deaths:
            if v == u or f2 < f or f2 > f + 3 * fps or v in victims: continue
            vn = name(created[v]['d']); dsurf = max(0.0, dist(P(u), P(v)) - max(half(vn)))
            hp = (FACTS.get(vn) or {}).get('health') or 1e9
            if dsurf <= rad and dmg * (rad - dsurf) / rad >= 0.5 * hp:
                victims.add(v); chain_victims[vn] += 1; chain_sources[nm] += 1
    out['chain_victims'] = dict(chain_victims)
    out['chain_sources'] = dict(chain_sources)
    out['chain_share_of_building_deaths'] = round(len(victims) / len(deaths), 3) if deaths else None

    # F. factory lanes: the facing from where the first products went, what stood in a lane 320 deep, the turrets' side
    lanes = []
    for v in facs:
        nm = name(created[v]['d'])
        if nm in NO_LANE: continue
        pv, fv = P(v), finished[v]['f']; hx, hz = half(nm)
        prods = sorted([(created[u]['f'], u) for u in created if created[u]['f'] > fv and (defs.get(created[u]['d'], {}).get('speed') or 0) > 0
                        and dist(P(u), pv) <= max(hx, hz) + 120])[:8]
        vecs = []
        for f0, u in prods:
            if u not in finished: continue
            p1 = pos_near(u, finished[u]['f'] + 5 * fps)
            if p1 and dist(P(u), p1) > 20: vecs.append((p1[0] - pv[0], p1[1] - pv[1]))
        if not vecs: lanes.append({'fac': nm, 'facing': None}); continue
        mx, mz = st.mean(x for x, _ in vecs), st.mean(z for _, z in vecs)
        facing = (0 if mz > 0 else 2) if abs(mz) >= abs(mx) else (1 if mx > 0 else 3)   # 0 south, 1 east, 2 north, 3 west
        if facing in (0, 2):
            x0, x1 = pv[0] - hx, pv[0] + hx
            z0, z1 = (pv[1] + hz, pv[1] + hz + 320) if facing == 0 else (pv[1] - hz - 320, pv[1] - hz)
        else:
            z0, z1 = pv[1] - hz, pv[1] + hz
            x0, x1 = (pv[0] + hx, pv[0] + hx + 320) if facing == 1 else (pv[0] - hx - 320, pv[0] - hx)
        inlane = []
        for u, r in everb.items():
            if u == v or finished[u]['f'] < fv: continue
            q = P(u); qx, qz = half(name(r['d']))
            if q[0] + qx > x0 and q[0] - qx < x1 and q[1] + qz > z0 and q[1] - qz < z1:
                depth = {0: q[1] - qz - (pv[1] + hz), 2: (pv[1] - hz) - (q[1] + qz), 1: q[0] - qx - (pv[0] + hx), 3: (pv[0] - hx) - (q[0] + qx)}[facing]
                inlane.append((name(r['d']), round(depth)))
        sides = collections.Counter()
        for u in nanos:
            q = P(u)
            if dist(q, pv) > 400 + max(hx, hz): continue
            dx, dz = q[0] - pv[0], q[1] - pv[1]
            fwd = {0: dz, 2: -dz, 1: dx, 3: -dx}[facing]; side = {0: dx, 2: -dx, 1: -dz, 3: dz}[facing]
            sides['front' if fwd > max(abs(side), hz) else 'back' if -fwd > max(abs(side), hz) else 'side'] += 1
        lanes.append({'fac': nm, 'facing': facing, 'in_lane': inlane, 'nano_sides': dict(sides), 'finished_min': round(fv / fps / 60, 1)})
    out['lanes'] = lanes
    return out

def pool(args):
    min_os = float(args[0])
    for d in args[1:]:
        if d.endswith('.jsonl'):
            print(json.dumps(analyse(d, {'name': os.path.relpath(d, REPO)}))); sys.stdout.flush(); continue
        cj = os.path.join(d, 'card.json')
        if not os.path.exists(cj): continue
        card = json.load(open(cj))
        for p in card.get('players', []):
            if (p.get('os') or 0) < min_os: continue
            rec = os.path.join(d, f"record-{p['team']}.jsonl")
            if not os.path.exists(rec): continue
            try: print(json.dumps(analyse(rec, p)))
            except Exception as e: print(json.dumps({'record': rec, 'error': repr(e)}))
            sys.stdout.flush()

# ---------------------------------------------------------------- block: one side's turret block
def block(record):
    h, defs, cr, fin, de, cmds, samples = load(record)
    name = lambda d: defs[d]['name']; fps = h.get('frames_per_second', 30)
    bld = {u: r for u, r in cr.items() if (defs.get(r['d'], {}).get('speed') or 0) == 0 and u in fin and r['d'] in defs}
    P = lambda u: (cr[u]['x'], cr[u]['z'])
    nanos = [u for u in bld if name(bld[u]['d']) in NANOS]
    end_nanos = [u for u in nanos if u not in de]
    print('turrets finished', len(nanos), 'standing at the end', len(end_nanos))
    pts = [P(u) for u in end_nanos]
    nn, bearings = [], collections.Counter()
    for i, p in enumerate(pts):
        ds = sorted((dist(p, q), q) for j, q in enumerate(pts) if j != i)
        if not ds: continue
        nn.append(ds[0][0]); q = ds[0][1]
        bearings[round(math.degrees(math.atan2(q[1] - p[1], q[0] - p[0])) / 45) * 45 % 180] += 1
    if nn:
        print('nearest neighbour: median', st.median(nn), 'quartiles', st.quantiles(nn, n=4) if len(nn) > 3 else nn,
              '| at 48 (touching)', sum(1 for d in nn if d <= 50), 'at 64', sum(1 for d in nn if 50 < d <= 66), 'over 100', sum(1 for d in nn if d > 100))
        print('bearing of the nearest neighbour (degrees mod 180; 0 = along x)', dict(bearings))
    par = list(range(len(pts)))
    def find(i):
        while par[i] != i: par[i] = par[par[i]]; i = par[i]
        return i
    for i in range(len(pts)):
        for j in range(i + 1, len(pts)):
            if dist(pts[i], pts[j]) <= 100: par[find(i)] = find(j)
    cl = collections.defaultdict(list)
    for i in range(len(pts)): cl[find(i)].append(pts[i])
    big = sorted(cl.values(), key=len, reverse=True)
    for c in big[:6]:
        xs = [p[0] for p in c]; zs = [p[1] for p in c]
        w, hg = max(xs) - min(xs) + 48, max(zs) - min(zs) + 48
        print(f'  cluster of {len(c)}: box {w} x {hg}, centre ({st.mean(xs):.0f}, {st.mean(zs):.0f}), {len(c) / (w * hg) * 1e6:.0f} per 1000x1000 (solid at 48 is 434)')
    reach_at = collections.defaultdict(list)
    for u, r in bld.items():
        n = name(r['d'])
        if n in NANOS or kind_of(n) in ('mex', 'moho'): continue
        hx, hz = half(n)
        def in_reach(f): return sum(1 for v in nanos if fin[v]['f'] <= f and (v not in de or de[v]['f'] > f) and dist(P(u), P(v)) <= 400 + max(hx, hz))
        reach_at[n].append((in_reach(cr[u]['f']), in_reach(fin[u]['f']), cr[u]['f'] / fps / 60, (fin[u]['f'] - cr[u]['f']) / fps))
    print('what stood with the turrets (turrets in reach when placed / when finished; the frame\'s seconds; the first one\'s clock):')
    for n, v in sorted(reach_at.items(), key=lambda x: -len(x[1])):
        print(f'  {n:12s} n={len(v):3d}  placed {st.median(a for a, _, _, _ in v):3.0f} (max {max(a for a, _, _, _ in v):3d})  finished {st.median(b for _, b, _, _ in v):3.0f}  frame {st.median(s for _, _, _, s in v):5.1f} s  first {min(m for _, _, m, _ in v):5.1f} min')
    groups = collections.defaultdict(list)
    for u, r in bld.items():
        if u in de and de[u].get('c') == 'reclaimed': continue
        k = kind_of(name(r['d']))
        if k: groups[k].append(u)
    print('spacing and the chain bill (a death at full health, every victim at full health):')
    for k in ['afus', 'fus', 'mmkr', 'makr', 'estor', 'wind', 'solar', 'advsol', 'nano']:
        us = groups.get(k, [])
        if len(us) < 2: continue
        nnk = [min(dist(P(u), P(v)) for v in us if v != u) for u in us]
        line = f'  {k:7s} n={len(us):3d} nearest neighbour median {st.median(nnk):.0f} min {min(nnk):.0f}'
        ex = ((FACTS.get(name(bld[us[0]]['d'])) or {}).get('explodeas') or '')
        if ex.lower() in BLAST and BLAST[ex.lower()][0] >= 100:
            tot = [sum(1 for v in bld if v != u and kills(ex, name(bld[v]['d']), dist(P(u), P(v)))) for u in us]
            same = [sum(1 for v in us if v != u and kills(ex, name(bld[v]['d']), dist(P(u), P(v)))) for u in us]
            line += f'; one death kills {st.median(tot):.0f} finished buildings median (max {max(tot)}), {st.median(same):.0f} of its own kind (max {max(same)})'
        print(line)
    recl = [(u, r) for u, r in de.items() if u in bld and r.get('c') == 'reclaimed']
    print('own reclaims', dict(collections.Counter(name(bld[u]['d']) for u, _ in recl)), 'at', sorted(round(r['f'] / fps / 60, 1) for _, r in recl))
    for u, r in bld.items():
        n = name(r['d'])
        if n in FACTORIES:
            hx, hz = half(n); fend = de[u]['f'] if u in de else 10**9
            inr = sum(1 for v in nanos if dist(P(u), P(v)) <= 400 + hx and fin[v]['f'] <= fend and (v not in de or de[v]['f'] > fend))
            print(f'  factory {n:8s} at {P(u)} finished {fin[u]["f"] / fps / 60:5.1f}', f'gone {de[u]["f"] / fps / 60:5.1f} ({de[u].get("c") or "killed"})' if u in de else 'standing at the end', f'turrets in reach then {inr}')


# ---------------------------------------------------------------- pen: when a base fences its own factories in, by unit size
def pen(record, minutes_step=1, allies=False):
    """For each land factory of the record's side: at every minute, whether a unit of each movement footprint (2 squares = bots,
    3 = vehicles and T2 bots, 4 = Bull/Fatboy class, 5 = Bantha class, 6 = Vanguard, 7 = Thor/Juggernaut) can walk from the
    factory's exit to open ground 1,200 away, through our own standing buildings (and the terrain's slopes when the record
    has terrain-<ai>.bin). A size is penned when the eroded free grid has no path. Grid: 16-elmo squares."""
    import struct
    h, defs, cr, fin, de, cmds, samples = load(record)
    name = lambda d: defs.get(d, {}).get('name', '?'); fps = h.get('frames_per_second', 30)
    W, H = int(h['map']['width']) // 16, int(h['map']['height']) // 16
    base_block = bytearray(W * H)   # 1 = never walkable (steep ground), from the terrain file when present
    terr = h.get('terrain')
    tfile = os.path.join(os.path.dirname(record), f"terrain-{h.get('ai_id', 0)}.bin")
    slope_limit = None
    if terr and os.path.exists(tfile):
        tw, th = terr['width'], terr['height']
        raw = open(tfile, 'rb').read()
        slopes = raw[tw * th * 2: tw * th * 3]
        # a vehicle's limit: TANK3 maxslope 10 degrees... the record's move_classes give the slope units; use the largest ground class' bound
        mcs = terr.get('move_classes') or h.get('move_classes') or []
        tank = min([m['max_slope'] for m in mcs if m.get('kind') == 'tank'] or [0.109]) * 255
        bot = min([m['max_slope'] for m in mcs if m.get('kind') == 'bot'] or [0.412]) * 255
        slope_limit = {'tank': round(tank), 'bot': round(bot)}
        bot_block = bytearray(W * H)
        for z in range(min(H, th)):
            for x in range(min(W, tw)):
                sv = slopes[z * tw + x]
                if sv > tank: base_block[z * W + x] = 1
                if sv > bot: bot_block[z * W + x] = 1
    else:
        bot_block = bytearray(W * H)
    bld = {u: r for u, r in cr.items() if (defs.get(r['d'], {}).get('speed') or 0) == 0 and u in fin and r['d'] in defs}
    facs = [u for u in bld if name(bld[u]['d']) in FACTORIES and name(bld[u]['d']) not in NO_LANE]
    end_f = max([r['f'] for r in samples] or [0])
    # facing from the first products, as in analyse()
    pos_at = {s['f']: {o[0]: (o[2], o[3]) for o in s.get('own', [])} for s in samples}
    samples_at = {s['f']: s.get('al') or [] for s in samples}
    sf = sorted(pos_at)
    def pos_near(uid, f):
        lo, hi, best = 0, len(sf) - 1, None
        while lo <= hi:
            mid = (lo + hi) // 2
            if sf[mid] <= f: best = sf[mid]; lo = mid + 1
            else: hi = mid - 1
        return None if best is None else pos_at[best].get(uid)
    def facing_of(v):
        pv = (cr[v]['x'], cr[v]['z']); hx, hz = half(name(cr[v]['d']))
        prods = sorted([(cr[u]['f'], u) for u in cr if cr[u]['f'] > fin[v]['f'] and (defs.get(cr[u]['d'], {}).get('speed') or 0) > 0 and dist((cr[u]['x'], cr[u]['z']), pv) <= max(hx, hz) + 120])[:8]
        vecs = []
        for f0, u in prods:
            if u not in fin: continue
            p1 = pos_near(u, fin[u]['f'] + 5 * fps)
            if p1 and dist((cr[u]['x'], cr[u]['z']), p1) > 20: vecs.append((p1[0] - pv[0], p1[1] - pv[1]))
        if not vecs: return None
        mx, mz = st.mean(x for x, _ in vecs), st.mean(z for _, z in vecs)
        return (0 if mz > 0 else 2) if abs(mz) >= abs(mx) else (1 if mx > 0 else 3)
    results = []
    for v in facs:
        fv = fin[v]['f']; dv = de[v]['f'] if v in de else end_f
        nm = name(cr[v]['d']); pv = (cr[v]['x'], cr[v]['z']); hx, hz = half(nm)
        facing = facing_of(v)
        if facing is None: continue
        # the exit squares: the row of squares just outside the front edge, across the width
        def exits_for(n):
            k = n // 2 + 1   # a unit's centre stands at least half its footprint off the wall
            e = []
            for i in range(-hx // 16, hx // 16):
                if facing == 0: e.append((pv[0] // 16 + i, (pv[1] + hz) // 16 + k - 1))
                elif facing == 2: e.append((pv[0] // 16 + i, (pv[1] - hz) // 16 - k))
                elif facing == 1: e.append(((pv[0] + hx) // 16 + k - 1, pv[1] // 16 + i))
                else: e.append(((pv[0] - hx) // 16 - k, pv[1] // 16 + i))
            return e
        row = {'factory': nm, 'at': pv, 'facing': facing, 'finished_min': round(fv / fps / 60, 1), 'gone_min': round(dv / fps / 60, 1) if v in de else None, 'penned': {}}
        first_pen = {}
        for minute in range(int(fv / fps / 60) + 1, int(dv / fps / 60) + 1, minutes_step):
            f = minute * 60 * fps
            grid = bytearray(bot_block if 'lab' in nm else base_block)   # a lab's bots climb what a plant's vehicles cannot
            if allies:
                # allied structures standing at this minute, from the nearest sample's `al` rows ([id, def, x, z, team, ...])
                near_f = max([k for k in sf if k <= f] or [sf[0]]) if sf else None
                for a in (samples_at.get(near_f) or []):
                    if a[1] not in defs or (defs[a[1]].get('speed') or 0) != 0: continue
                    qx, qz = half(name(a[1]))
                    for z in range(max(0, (a[3] - qz) // 16), min(H, (a[3] + qz) // 16)):
                        base = z * W
                        for x in range(max(0, (a[2] - qx) // 16), min(W, (a[2] + qx) // 16)): grid[base + x] = 1
            for u, r in bld.items():
                if u == v or fin[u]['f'] > f or (u in de and de[u]['f'] <= f): continue
                qx, qz = half(name(r['d']))
                x0, x1 = max(0, (r['x'] - qx) // 16), min(W, (r['x'] + qx) // 16)
                z0, z1 = max(0, (r['z'] - qz) // 16), min(H, (r['z'] + qz) // 16)
                for z in range(z0, z1):
                    base = z * W
                    for x in range(x0, x1): grid[base + x] = 1
            # the factory's own footprint blocks too (its yard), the exits are outside it
            for z in range(max(0, (pv[1] - hz) // 16), min(H, (pv[1] + hz) // 16)):
                for x in range(max(0, (pv[0] - hx) // 16), min(W, (pv[0] + hx) // 16)): grid[z * W + x] = 1
            for n in (2, 3, 4, 5, 6, 7):
                if n in first_pen: continue
                # erosion: a square is open for size n when the n x n block it anchors (centre-ish) is free
                off = n // 2
                def free(x, z):
                    if x - off < 0 or z - off < 0 or x - off + n > W or z - off + n > H: return False
                    for zz in range(z - off, z - off + n):
                        base = zz * W
                        for xx in range(x - off, x - off + n):
                            if grid[base + xx]: return False
                    return True
                starts = [(x, z) for x, z in exits_for(n) if 0 <= x < W and 0 <= z < H and free(x, z)]
                if not starts:
                    # the door itself is narrower than the unit: look one more row out
                    first_pen[n] = minute; continue
                seen = set(starts); stack = list(starts); out = False
                far = (1200 // 16) ** 2
                while stack:
                    x, z = stack.pop()
                    if (x - pv[0] // 16) ** 2 + (z - pv[1] // 16) ** 2 >= far: out = True; break
                    for dx, dz in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        nx, nz = x + dx, z + dz
                        if (nx, nz) in seen or not (0 <= nx < W and 0 <= nz < H): continue
                        if free(nx, nz): seen.add((nx, nz)); stack.append((nx, nz))
                if not out: first_pen[n] = minute
        row['penned'] = first_pen
        results.append(row)
    return {'record': record, 'allies': allies, 'slope_limit': slope_limit, 'minutes': round(end_f / fps / 60, 1), 'factories': results}

# ---------------------------------------------------------------- summary of a pool file
def summary(path):
    rows = [json.loads(l) for l in open(path) if 'error' not in l]
    def med(vals):
        v = [x for x in vals if x is not None]; return (round(st.median(v), 2), len(v)) if v else None
    print(len(rows), 'seats; game minutes median', round(st.median(r['minutes'] for r in rows), 1))
    print('opening: first factory s', med(r['first_factory_s'] for r in rows), '| commander path', med(r['open_com_path'] for r in rows),
          'still s', med(r['open_com_still_s'] for r in rows), 'of horizon', med(r['open_horizon_s'] for r in rows))
    print('  stands before the factory', collections.Counter(r['open_stands'] for r in rows).most_common(5), '| build orders', med(r['open_com_build_orders'] for r in rows),
          'batches', med(r['open_queue_groups'] for r in rows), 'depths', collections.Counter(d for r in rows for d in r['open_queue_depths']).most_common(8))
    print('  sites in reach when ordered', round(sum(r['open_sites_in_reach_at_order'] for r in rows) / max(1, sum(r['open_com_build_orders'] for r in rows)), 2))
    steps = [s for r in rows for s in r['open_site_steps']]; gaps = [g for r in rows for g in r['open_gaps_s']]
    if len(steps) > 4: print('  site steps median', st.median(steps), 'quartiles', st.quantiles(steps, n=4)[0::2])
    if len(gaps) > 4: print('  gaps finished-to-next-order s median', st.median(gaps), 'quartiles', st.quantiles(gaps, n=4)[0::2])
    print('  first structures', collections.Counter(' '.join(x[3:] for x in r['open_structures_before_factory'][:6]) for r in rows).most_common(5))
    print('counts ever (median a seat)', {k: med(r['counts_ever'].get(k, 0) for r in rows) for k in ['solar', 'wind', 'mex', 'moho', 'nano', 'fac', 'fus', 'afus', 'makr', 'mmkr', 'estor']})
    print('turrets: n', med(r['nano_n'] for r in rows), 'within 400 of a factory', med(r['nano_within_400_of_factory_share'] for r in rows),
          'nearest neighbour', med(r['nano_nn_median'] for r in rows), 'to the factory edge', med(r['nano_to_factory_edge_median'] for r in rows))
    pf = collections.defaultdict(list)
    for r in rows:
        for nm, n, m in r['nanos_per_factory']: pf[nm[3:]].append(n)
    print('  per factory (median, max, n)', {k: (st.median(v), max(v), len(v)) for k, v in pf.items()})
    print('spacing (median of seat medians)', {k: med(r['spacing'][k][0] if r['spacing'][k] else None for r in rows) for k in rows[0]['spacing']})
    print('  cross', {k: med(r['spacing_cross'][k][0] if r['spacing_cross'][k] else None for r in rows) for k in rows[0]['spacing_cross']})
    rec = collections.Counter()
    for r in rows: rec.update(r['own_reclaims'])
    rm = [m for r in rows for m in r['own_reclaim_minutes']]
    print('own reclaims', rec.most_common(10), '| seats with any', sum(1 for r in rows if r['own_reclaims']), '| minute median', st.median(rm) if rm else None)
    cv, cs = collections.Counter(), collections.Counter()
    for r in rows: cv.update(r['chain_victims']); cs.update(r['chain_sources'])
    print('chain victims', cv.most_common(8), '| sources', cs.most_common(6), '| victims', sum(cv.values()), 'of', sum(r['building_deaths'] for r in rows), 'building deaths; seats with any', sum(1 for r in rows if r['chain_victims']))
    fac, inl, sides, toward, tot, withany, depths = collections.Counter(), collections.Counter(), collections.Counter(), 0, 0, 0, []
    for r in rows:
        sx = r.get('start_x')
        for L in r['lanes']:
            if L['facing'] is None: continue
            tot += 1; fac[L['facing']] += 1; sides.update(L['nano_sides']); withany += 1 if L['in_lane'] else 0
            if sx is not None and ((sx < 4096 and L['facing'] == 1) or (sx >= 4096 and L['facing'] == 3)): toward += 1
            for nm, dep in L['in_lane']: inl[nm] += 1; depths.append(dep)
    print('lanes: factories read', tot, 'facing (0 S, 1 E, 2 N, 3 W)', dict(fac), 'toward the enemy', toward, '| with a building in the 320 lane', withany)
    if len(depths) > 4: print('  in-lane near edges from the front: quartiles', st.quantiles(depths, n=4), 'within 100', round(sum(1 for d in depths if d < 100) / len(depths), 2), '| kinds', inl.most_common(6))
    print('  turrets round a plant by side', dict(sides))

if __name__ == '__main__':
    if len(sys.argv) < 3: sys.exit(__doc__)
    mode = sys.argv[1]
    if mode == 'pool': pool(sys.argv[2:])
    elif mode == 'block': block(sys.argv[2])
    elif mode == 'pen':
        allies = '--allies' in sys.argv
        for r in sys.argv[2:]:
            if r != '--allies': print(json.dumps(pen(r, allies=allies))); sys.stdout.flush()
    elif mode == 'summary': summary(sys.argv[2])
    else: sys.exit(__doc__)
