#!/usr/bin/env python3
"""Check a synthesis agent's counted claims against the cards (docs/design/2026-09-23-replay-survey.md, decision 6).
One row per side of every carded match in the manifest, with the numbers claims quote; `--count EXPR` evaluates a
Python expression over each row and prints how many sides (or games) satisfy it, so "the plant before 1:00 in 41 of
58 sides" is checked as `--count "ff_secs < 60"`.
usage: run/replays/check.py [--map FILE_NAME] [--table] [--count EXPR ...] [--by won|start|faction] [--games]
Row fields: id, team, player, os, faction (arm|cor), start (cell), won, higher_os, duration_secs, ff_kind, ff_secs,
  ff_at, sf_kind, sf_secs, t2_secs, t2_unit, ex4, inc4, ex6, inc6, ex8, inc8, ex10, ex12, inc12, cons5, cons10,
  rovers2 (Rovers at the 2:00 sample), army8, army12, lost_secs, lost_spot, lost_grid, lost_by, first_seen_secs,
  n_fights, fight_cells (the cells with most deaths, worst first), spots (the first eight spots taken, as cells).
`--games` counts games (a match where any side satisfies the expression) instead of sides.
"""
import collections, json, statistics, sys
MANIFEST = 'run/data/replays/manifest.jsonl'
def secs(c):
    if not c or '?' in c: return None
    m, s = c.split(':'); return int(m) * 60 + int(s)
def rows(map_file=None):
    out = []
    for l in open(MANIFEST):
        r = json.loads(l)
        if not (r.get('keep') and r.get('card')): continue
        if map_file and r.get('map_file') != map_file: continue
        c = json.load(open(r['match'] + '/card.json'))
        winners = set(r.get('winners') or [])
        won_team = {p['team'] for p in r['players'] if p['name'] in winners}
        pl = {p['team']: p for p in r['players']}
        oss = [p.get('os') for p in r['players'] if p.get('os') is not None]
        for t in c['teams']:
            team = int(t['team']); p = pl.get(team, {})
            def at(minute, key):
                for row in t['curves']:
                    if row['minute'] == minute: return row.get(key)
            def army(minute):
                for row in t['army_by_type']:
                    if row['minute'] == minute: return row['army']
                return {}
            fights = sorted(t.get('fights') or [], key=lambda f: -(f['ours_lost'] + f['theirs_lost']))
            cells = collections.Counter()
            for f in t.get('fights') or []: cells[f['grid']] += f['ours_lost'] + f['theirs_lost']
            f1 = t['factories'][0] if t['factories'] else {}
            f2 = t['factories'][1] if len(t['factories']) > 1 else {}
            lost = t.get('first_extractor_lost') or {}
            row = dict(id=r['id'][:8], team=team, player=p.get('name'), os=p.get('os'), faction=t['side'], start=t['start']['grid'],
                won=team in won_team, higher_os=(p.get('os') is not None and len(oss) == 2 and p['os'] == max(oss)),
                duration_secs=r['duration_ms'] // 1000, ff_kind=f1.get('unit'), ff_secs=secs(f1.get('started')), ff_at=f1.get('at'),
                sf_kind=f2.get('unit'), sf_secs=secs(f2.get('started')), t2_secs=secs((t.get('first_tier2') or {}).get('clock')),
                t2_unit=(t.get('first_tier2') or {}).get('unit'),
                ex4=at(4, 'extractors'), inc4=at(4, 'metal_income'), ex6=at(6, 'extractors'), inc6=at(6, 'metal_income'),
                ex8=at(8, 'extractors'), inc8=at(8, 'metal_income'), ex10=at(10, 'extractors'), ex12=at(12, 'extractors'), inc12=at(12, 'metal_income'),
                cons5=at(5, 'constructors'), cons10=at(10, 'constructors'), rovers2=army(2).get('armfav', 0) + army(2).get('corfav', 0),
                army8=at(8, 'army_metal'), army12=at(12, 'army_metal'),
                lost_secs=secs(lost.get('clock')), lost_spot=lost.get('spot'), lost_grid=lost.get('grid'), lost_by=lost.get('by'),
                first_seen_secs=secs((t.get('first_enemy_seen') or {}).get('clock')), n_fights=len(t.get('fights') or []),
                fight_cells=[c for c, _ in cells.most_common(4)], spots=[s['grid'] for s in (t.get('spots_taken') or [])[:8]])
            out.append(row)
    return out
def main():
    args = sys.argv[1:]
    def opt(name, default=None):
        if name in args:
            i = args.index(name); v = args[i + 1]; del args[i:i + 2]; return v
        return default
    map_file = opt('--map'); by = opt('--by'); games = '--games' in args; table = '--table' in args
    exprs = []
    while '--count' in args:
        i = args.index('--count'); exprs.append(args[i + 1]); del args[i:i + 2]
    rs = rows(map_file)
    if table or not exprs:
        keys = ['id', 'team', 'os', 'faction', 'start', 'won', 'ff_kind', 'ff_secs', 'sf_kind', 'sf_secs', 't2_secs', 'ex4', 'ex8', 'ex12', 'inc8', 'cons5', 'rovers2', 'lost_secs', 'lost_by', 'duration_secs', 'fight_cells']
        print('\t'.join(keys))
        for r in rs: print('\t'.join(str(r[k]) for k in keys))
    for e in exprs:
        def ok(r):
            try: return bool(eval(e, {}, dict(r)))
            except TypeError: return False
        groups = collections.defaultdict(list)
        for r in rs: groups[r[by] if by else 'all'].append(r)
        for g, rr in sorted(groups.items(), key=lambda kv: str(kv[0])):
            if games:
                ids = {r['id'] for r in rr}; hit = {r['id'] for r in rr if ok(r)}
                print(f'{e} [{g}]: {len(hit)} of {len(ids)} games')
            else:
                print(f'{e} [{g}]: {sum(ok(r) for r in rr)} of {len(rr)} sides')
if __name__ == '__main__':
    main()
