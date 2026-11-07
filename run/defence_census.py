#!/usr/bin/env python3
"""Static defence in the heuristic bots' games from the engine's once-a-minute census of both sides (engine.log, lines
`[wreason ai=N] census f=F own|enemy <def>x<n>@x,z ...`): per match, both sides' defence metal and army metal at 5, 10, 15, 20
minutes and at the end, with the result from results.jsonl.
    run/defence_census.py OUT.jsonl BATCHDIR...     rows appended
    run/defence_census.py --summary ROWS.jsonl...  by our bot and opponent"""
import json, os, re, sys, glob, collections
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
U = json.load(open(f'{REPO}/crates/bot/data/units.json'))['units']
GROUND = {k for k, v in U.items() if v['class'] in ('defence turret', 'artillery turret', 'torpedo launcher', 'armed metal extractor', 'armed geothermal plant', 'long-range cannon')}
AA = {k for k, v in U.items() if v['class'] == 'anti-air turret'}
MOBILE = {k for k, v in U.items() if v.get('speed') and (v.get('dps') or v.get('dps_air')) and 'construct' not in v['class'] and 'commander' not in v['class']}
LINE = re.compile(r'\[wreason ai=(\d+)\] census f=(\d+) (own|enemy) (.*)')
TOK = re.compile(r'(\w+?)x(\d+)@')
MARKS = (5, 10, 15, 20)

def scan(m):
    log = os.path.join(m, 'engine.log')
    if not os.path.exists(log): return None
    rows = {}
    with open(log, errors='replace') as fh:
        for l in fh:
            if 'census f=' not in l: continue
            g = LINE.match(l.strip())
            if not g: continue
            ai, f, side, rest = g.groups()
            if ai != '0': continue
            minute = int(f) // 1800
            c = collections.Counter({d: int(n) for d, n in TOK.findall(rest)})
            gm = sum(U[d]['metal'] * n for d, n in c.items() if d in GROUND)
            am = sum(U[d]['metal'] * n for d, n in c.items() if d in AA)
            ar = sum(U[d]['metal'] * n for d, n in c.items() if d in MOBILE)
            ng = sum(n for d, n in c.items() if d in GROUND)
            rows[(side, minute)] = (gm, am, ar, ng, {d: n for d, n in c.items() if d in GROUND or d in AA})
    if not rows: return None
    last = max(mn for _, mn in rows)
    out = dict(match=m, end_minute=last)
    for side in ('own', 'enemy'):
        for mk in MARKS + (last,):
            key = mk if mk in MARKS else 'end'
            r = rows.get((side, mk))
            out[f'{side}_{key}'] = None if r is None else dict(ground=r[0], aa=r[1], army=r[2], n_ground=r[3])
        r = rows.get((side, last))
        out[f'{side}_types'] = r[4] if r else {}
    return out

if __name__ == '__main__' and sys.argv[1:2] != ['--summary']:
    out = open(sys.argv[1], 'a'); n = 0
    for b in sys.argv[2:]:
        res = {}
        rp = os.path.join(b, 'results.jsonl')
        if os.path.exists(rp):
            for l in open(rp):
                try: j = json.loads(l); res[j['index']] = j
                except Exception: pass
        bj = {}
        if os.path.exists(os.path.join(b, 'batch.json')):
            try: bj = json.load(open(os.path.join(b, 'batch.json')))
            except Exception: pass
        for m in sorted(glob.glob(os.path.join(b, '[0-9][0-9]'))):
            if glob.glob(os.path.join(m, 'record-*.jsonl')): continue  # the record scanner covers these
            r = scan(m)
            if not r: continue
            i = int(os.path.basename(m)); rr = res.get(i, {})
            r.update(batch=os.path.basename(b), label=bj.get('label'), our_ai=bj.get('our_ai') or bj.get('bot'), opponent=bj.get('opponent'), map=bj.get('map'), outcome=rr.get('outcome'), game_minutes=rr.get('game_minutes'), enemy_bonus=bj.get('enemy_bonus'))
            out.write(json.dumps(r) + '\n'); n += 1
    out.close(); print(n, 'matches')


def summary(paths):
    """By our bot and opponent: both sides' ground defence metal at 10 and 15 minutes, turret counts and defence over army at 15 (`--summary ROWS.jsonl...`)."""
    import statistics as st
    rows = [json.loads(l) for p in paths for l in open(p)]
    med = lambda xs: (lambda v: st.median(v) if v else float('nan'))([x for x in xs if x is not None])
    g = collections.defaultdict(list)
    for r in rows: g[(str(r['our_ai']).split(':')[0], r['opponent'])].append(r)
    print(f"{'us':13}{'opponent':24}{'n':>5}{'won':>5} | ours ground-def at 10 / 15 | his at 10 / 15 | ours n at 15 | his | ours def/army % at 15 | his")
    for (a, o), rs in sorted(g.items(), key=lambda kv: -len(kv[1])):
        if len(rs) < 10: continue
        f = lambda side, k, key='ground': med([(r.get(f'{side}_{k}') or {}).get(key) for r in rs])
        pct = lambda side: med([100 * r[f'{side}_15']['ground'] / r[f'{side}_15']['army'] for r in rs if r.get(f'{side}_15') and r[f'{side}_15']['army'] > 0])
        won = sum(1 for r in rs if r['outcome'] == 'Win')
        print(f"{a:13}{str(o)[:23]:24}{len(rs):>5}{won:>5} | {f('own',10):>6.0f} {f('own',15):>6.0f} | {f('enemy',10):>6.0f} {f('enemy',15):>6.0f} | {f('own',15,'n_ground'):>5.0f} | {f('enemy',15,'n_ground'):>5.0f} | {pct('own'):>5.0f} | {pct('enemy'):>5.0f}")
        to = collections.Counter(); te = collections.Counter()
        for r in rs: to.update(r['own_types']); te.update(r['enemy_types'])
        print('      ours: ' + ', '.join(f"{k} {v}" for k, v in to.most_common(6)) + '   |   his: ' + ', '.join(f"{k} {v}" for k, v in te.most_common(6)))


if __name__ == '__main__' and sys.argv[1:2] == ['--summary']:
    summary(sys.argv[2:])
