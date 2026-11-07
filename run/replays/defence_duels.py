#!/usr/bin/env python3
"""Static defence in the processed high-OS duels (the friend's bar-replays tool, run/replays/bar_replays.py fetches a game's
events and match JSON into run/data/bar-replays/): per side, the ground and anti-air defence metal built, the types, the
metal of his units whose last hit was a defence's, the defence lost, the army beside it, the engine's own defenseValue by
minute; summarised over the pool as quartiles for all sides, winners and losers; then where the defences stand (home, own
half, forward), what they kill and what kills them, damage dealt per metal of units that died, and the habits by map.
Written for docs/studies/2026-11-06-static-defence-survey.md.

    run/replays/defence_duels.py [--rows OUT.json]      every game cached under run/data/bar-replays/
"""
import json, glob, os, sys, collections, statistics as st, math
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
U = json.load(open(f'{REPO}/crates/bot/data/units.json'))['units']
CLS = {k: v['class'] for k, v in U.items()}
GROUND = {k for k, v in U.items() if v['class'] in ('defence turret', 'artillery turret', 'torpedo launcher', 'armed metal extractor', 'armed geothermal plant', 'long-range cannon')}
AA = {k for k, v in U.items() if v['class'] == 'anti-air turret'}
DEF = GROUND | AA
NANO = {k for k, v in U.items() if 'construction turret' in v['class']}
MOBILE = {k for k, v in U.items() if v.get('speed') and (v.get('dps') or v.get('dps_air')) and 'construct' not in v['class'] and 'commander' not in v['class']}
BUILDER = {k for k, v in U.items() if 'construct' in v['class'] or 'commander' in v['class']}
metal = lambda n: U.get(n, {}).get('metal', 0) or 0

def side_rows(gid):
    ev = json.load(open(f'{REPO}/run/data/bar-replays/events/{gid}.json'))
    m = json.load(open(f'{REPO}/run/data/bar-replays/matches/{gid}.json'))
    defs = {d['definitionID']: d['definitionName'] for d in ev['unitDefinitions']}
    nm = lambda i: defs.get(i, '?')
    teams = {t['teamID']: t for t in m['teams']}
    players = {p['teamID']: p for p in m['players']}
    won = {a['allyTeamID']: a.get('won') for a in m.get('allyTeams', [])}
    end = ev['extraStats'][-1]['frame'] if ev['extraStats'] else 0
    # units by id: team, def, created frame, finished?
    unit = {}
    for u in ev['unitsCreated']:
        unit[u['unitID']] = (u['teamID'], nm(u['definitionID']), u['frame'], u['unitX'], u['unitZ'])
    out = {}
    for tid, t in teams.items():
        p = players.get(tid, {})
        r = dict(game=gid, map=m['map'], minutes=m['durationMs'] // 60000, team=tid, faction=t['faction'], name=p.get('name'), os=p.get('skill'), won=won.get(t['allyTeamID']),
                 def_by_min=collections.Counter(), def_types=collections.Counter(), aa_types=collections.Counter(), ground_metal=0, aa_metal=0, nano_metal=0, army_metal=0,
                 kills_by_def=0, kills_by_def_n=0, kills_by_type=collections.Counter(), def_lost=0, def_lost_to=collections.Counter(), def_first=None, first_def_clock=None,
                 def_value=[], army_value=[], total_value=[])
        out[tid] = r
    for uid, (tid, d, f, x, z) in unit.items():
        r = out.get(tid)
        if r is None: continue
        if d in DEF:
            r['def_by_min'][f // 1800] += metal(d)
            (r['def_types'] if d in GROUND else r['aa_types'])[d] += 1
            if d in GROUND: r['ground_metal'] += metal(d)
            else: r['aa_metal'] += metal(d)
            if r['def_first'] is None or f < r['def_first']: r['def_first'] = f
        elif d in NANO: r['nano_metal'] += metal(d)
        elif d in MOBILE: r['army_metal'] += metal(d)
    for k in ev['unitsKilled']:
        v = unit.get(k['unitID']); 
        if v is None: continue
        vt, vd = v[0], v[1]
        ad = nm(k['attackerDefinitionID']) if k['attackerDefinitionID'] else None
        at = k['attackerTeam']
        if ad in DEF and at in out and at != vt:
            out[at]['kills_by_def'] += metal(vd); out[at]['kills_by_def_n'] += 1; out[at]['kills_by_type'][ad] += metal(vd)
        if vd in DEF and vt in out:
            out[vt]['def_lost'] += metal(vd); out[vt]['def_lost_to'][ad or 'unattributed'] += 1
    for s in ev['extraStats']:
        r = out.get(s['teamID'])
        if r is None: continue
        r['def_value'].append((s['frame'], s['defenseValue'])); r['army_value'].append((s['frame'], s['armyValue'])); r['total_value'].append((s['frame'], s['totalValue']))
    for r in out.values():
        r['first_def_clock'] = None if r['def_first'] is None else f"{r['def_first'] // 1800}:{(r['def_first'] % 1800) // 30:02d}"
    return list(out.values())

def first_pass():
    ids = [os.path.basename(p)[:-5] for p in glob.glob(f'{REPO}/run/data/bar-replays/events/*.json')]
    rows = []; bad = 0
    for gid in ids:
        if not os.path.exists(f'{REPO}/run/data/bar-replays/matches/{gid}.json'): continue
        try: rows += side_rows(gid)
        except Exception as e: bad += 1
    print(f'{len(rows)} sides of {len(rows)//2} games ({bad} unreadable)')
    if '--rows' in sys.argv: json.dump(rows, open(sys.argv[sys.argv.index('--rows') + 1], 'w'), default=list)
    def q(xs): xs = sorted(xs); return f"{xs[len(xs)//4]:.1f} / {xs[len(xs)//2]:.1f} / {xs[3*len(xs)//4]:.1f}" if xs else '-'
    for label, rs in (('all', rows), ('winners', [r for r in rows if r['won']]), ('losers', [r for r in rows if r['won'] is False])):
        print(f"== {label}: {len(rs)} sides; quartiles (lower / median / upper)")
        print('  ground defence metal built   ', q([r['ground_metal'] for r in rs]))
        print('  anti-air defence metal built ', q([r['aa_metal'] for r in rs]))
        print('  army metal built             ', q([r['army_metal'] for r in rs]))
        print('  ground defence as % of army  ', q([100 * r['ground_metal'] / r['army_metal'] for r in rs if r['army_metal']]))
        print('  metal killed by defences     ', q([r['kills_by_def'] for r in rs]))
        print('  killed by defences / ground defence metal ', q([r['kills_by_def'] / r['ground_metal'] for r in rs if r['ground_metal']]))
        print('  defence metal lost           ', q([r['def_lost'] for r in rs]))
        print('  first defence at             ', q([r['def_first'] / 1800 for r in rs if r['def_first'] is not None]), 'min; none in', sum(1 for r in rs if r['def_first'] is None))
        tot = collections.Counter(); kt = collections.Counter()
        for r in rs: tot.update(r['def_types']); kt.update(r['kills_by_type'])
        print('  ground types: ' + ', '.join(f"{k} {v}" for k, v in tot.most_common(12)))
        print('  metal killed by type: ' + ', '.join(f"{k} {v:.0f}" for k, v in kt.most_common(10)))
    # by minute: defence value as share of total value across the pool, by game minute
    print('== defenseValue share of totalValue (median over sides alive at that minute), and median defenseValue')
    for minute in (3, 5, 8, 10, 12, 15, 20, 25, 30, 40):
        sh = []; dv = []; av = []
        for r in rows:
            pts = [(f, v) for f, v in r['def_value'] if f <= minute * 1800]
            tv = [(f, v) for f, v in r['total_value'] if f <= minute * 1800]
            am = [(f, v) for f, v in r['army_value'] if f <= minute * 1800]
            if not pts or r['minutes'] < minute: continue
            d = pts[-1][1]; t = tv[-1][1]; a = am[-1][1]
            dv.append(d); av.append(a)
            if t: sh.append(100 * d / t)
        if dv: print(f"  {minute:>2} min: sides {len(dv):>4}  defence {st.median(dv):>6.0f}  army {st.median(av):>6.0f}  defence/total {st.median(sh):>4.1f} %")


def second_pass():
    rows = []; place = collections.Counter(); place_m = collections.Counter(); victims = collections.Counter(); victims_n = collections.Counter()
    dmg = collections.defaultdict(lambda: [0.0, 0.0, 0])  # class -> damage dealt, metal, n (dead units only)
    bymap = collections.defaultdict(list)
    raider_names = {k for k, v in U.items() if 'raider' in v['class'] or 'scout' in v['class']}
    for p in sorted(glob.glob(f'{REPO}/run/data/bar-replays/events/*.json')):
        gid = os.path.basename(p)[:-5]
        mp = f'{REPO}/run/data/bar-replays/matches/{gid}.json'
        if not os.path.exists(mp): continue
        ev = json.load(open(p)); m = json.load(open(mp))
        defs = {d['definitionID']: d['definitionName'] for d in ev['unitDefinitions']}
        nm = lambda i: defs.get(i, '?')
        starts = {}
        for c in ev['commanderPositionUpdates']:
            if c['frame'] == 0: starts.setdefault(c['unitID'], (c['unitX'], c['unitZ']))
        team_of = {u['unitID']: u['teamID'] for u in ev['unitsCreated']}
        tstart = {}
        for uid, xz in starts.items():
            if uid in team_of: tstart[team_of[uid]] = xz
        if len(tstart) != 2: continue
        teams = list(tstart)
        dist = lambda a, b: math.hypot(a[0] - b[0], a[1] - b[1])
        unit = {u['unitID']: (u['teamID'], nm(u['definitionID']), u['frame'], u['unitX'], u['unitZ']) for u in ev['unitsCreated']}
        per = {t: dict(ground=0, first=None, kills=0) for t in teams}
        for uid, (t, d, f, x, z) in unit.items():
            if d in GROUND and t in tstart:
                other = [o for o in teams if o != t][0]
                do, dh = dist((x, z), tstart[t]), dist((x, z), tstart[other])
                gap = dist(tstart[t], tstart[other])
                k = 'home (<1500 of own start)' if do < 1500 else ('forward (nearer his start)' if dh < do else 'own half, out')
                place[k] += 1; place_m[k] += metal(d)
                per[t]['ground'] += metal(d)
                if per[t]['first'] is None or f < per[t]['first']: per[t]['first'] = f
        for k in ev['unitsKilled']:
            v = unit.get(k['unitID'])
            if not v: continue
            ad = nm(k['attackerDefinitionID']) if k['attackerDefinitionID'] else None
            if ad in DEF and k['attackerTeam'] != v[0]:
                victims[v[1]] += metal(v[1]); victims_n[v[1]] += 1
                if k['attackerTeam'] in per: per[k['attackerTeam']]['kills'] += metal(v[1])
        killed = {k['unitID'] for k in ev['unitsKilled']}
        for u in ev['unitDamage']:
            if u['unitID'] not in killed: continue
            d = nm(u['definitionID'])
            c = 'ground defence' if d in GROUND else ('soldier' if d in MOBILE else None)
            if c is None: continue
            if c == 'soldier' and d in raider_names: c = 'soldier: raider/scout'
            elif c == 'soldier': c = 'soldier: other'
            dmg[c][0] += u['damageDealt']; dmg[c][1] += metal(d); dmg[c][2] += 1
        for t in teams:
            bymap[m['map']].append(per[t])
    print('== where the ground defences stand (count, metal):', {k: (v, place_m[k]) for k, v in place.items()})
    print('== what defences kill (metal, count), top 15:')
    for k, v in victims.most_common(15): print(f'   {k:12} {v:>8.0f} {victims_n[k]:>5}  {U.get(k,{}).get("class","")}')
    print('== damage dealt per metal, dead units only (damage HP / metal; n):')
    for c, (dd, mm, n) in dmg.items(): print(f'   {c:24} {dd/mm if mm else 0:>5.1f}  (n {n}, metal {mm:.0f})')
    print('== by map (sides, median ground defence metal, median first defence min, median kills-by-defence / defence metal)')
    for mp, rs in sorted(bymap.items(), key=lambda kv: -len(kv[1]))[:14]:
        g = [r['ground'] for r in rs]; f = [r['first'] / 1800 for r in rs if r['first'] is not None]; e = [r['kills'] / r['ground'] for r in rs if r['ground']]
        print(f'   {mp[:34]:35} {len(rs):>4}  {st.median(g):>6.0f}  {st.median(f) if f else -1:>5.1f}  {st.median(e) if e else 0:>4.1f}')


if __name__ == '__main__':
    first_pass()
    print()
    second_pass()
