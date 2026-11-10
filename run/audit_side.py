#!/usr/bin/env python3
"""The side-level truth of a team game from the audit dump (docs/harness/audit.md), where run/audit.py reads one
seat against one opponent: every ally team's armed metal by minute and by thirds of the map, our seats' metal
ledger (the engine's excess, sent and received: a full store's overflow goes to the side, so `sent` is what a seat
handed its allies and `excess` what was lost), orders by side, the people's losses by killer team, what the silos
killed, units given between teams, the unit counts of the last minutes, the engine's team statistics, the winners.
Written for the lobby 10v8 on Erebos Lakes (run/matches/1794293029-human/00/review.md).

    run/audit_side.py <replay match dir>/audit.jsonl [our teams, comma separated; default 14,16]

The thirds are of x (west, middle, east): a map whose sides face each other along z needs the line changed.
The seats' names read UNKNOWN for AIs in a replay; the team numbers are the game's (the match's script.txt names them).
"""
import json, sys, collections
path = sys.argv[1]
OURS = [int(x) for x in sys.argv[2].split(',')] if len(sys.argv) > 2 else [14, 16]
defs = {}; ally = {}; names = {}
def cell(x, z): return 'ABCDEFGHIJ'[max(0, min(int(x // 1024), 9))] + str(max(0, min(int(z // 1024), 9)) + 1)
army = collections.defaultdict(lambda: collections.defaultdict(float))      # minute -> key -> metal
eco = collections.defaultdict(lambda: collections.defaultdict(float))
orders = collections.defaultdict(lambda: collections.Counter())
kills = collections.defaultdict(lambda: collections.Counter())             # minute -> (killer team) -> metal of ally-0 things killed
lost = collections.defaultdict(lambda: collections.Counter())              # minute -> victim team -> metal
silo_kills = []; given = collections.Counter(); over = None; last_units = {}
unit_def = {}
teams_alive = {}
final_st = {}
for line in open(path):
    try: r = json.loads(line)
    except Exception: continue
    t = r['t']
    if t == 'header':
        for s in r['seats']: ally[s['team']] = s['ally']; names[s['team']] = s['name']
    elif t == 'def': defs[r['d']['id']] = r['d']
    elif t == 'cmd':
        orders[r['f'] // 1800][(r['team'], r['c'])] += 1
    elif t == 'ev':
        k = r['k']
        if k == 'created': unit_def[r['u']] = r['d']
        elif k == 'given': given[(r['f'] // 1800, r.get('from'), r['team'])] += 1
        elif k == 'destroyed':
            d = defs.get(r['d'], {}); m = d.get('metal', 0) * (r.get('progress', 100) or 0) / 100.0
            mn = r['f'] // 1800
            lost[mn][r['team']] += m
            bt = r.get('by_team', -1)
            if ally.get(r['team']) == 0: kills[mn][bt] += m
            bd = defs.get(r.get('by_d'), {}).get('name', '')
            if 'silo' in bd: silo_kills.append((r['f'], bt, r['team'], d.get('name'), m, r['x'], r['z']))
    elif t == 'over': over = r
    elif t == 's':
        f = r['f']; mn = f // 1800
        for s in r['sides']:
            tm = s['team']; m = s['m']
            if tm in OURS:
                e = eco[mn]; e[(tm, 'excess')] += m[5]; e[(tm, 'sent')] += m[6]; e[(tm, 'recv')] += m[7]; e[(tm, 'inc')] += m[3]; e[(tm, 'exp')] += m[4]; e[(tm, 'n')] += 1; e[(tm, 'bank')] += m[0]
            final_st[tm] = s.get('st')
        if f % 1800 == 0:
            a = army[mn]; cnt = collections.Counter()
            for u in r['u']:
                d = defs.get(u[1]);
                if not d: continue
                tm = u[2]; al = ally.get(tm, -1)
                if len(u) > 6 and u[6] < 100: continue
                cnt[tm] += 1
                armed = d.get('weapons', 0) > 0
                if d.get('building'):
                    if armed: a[('turrets', al)] += d['metal']
                    continue
                if not armed or d.get('build_speed', 0) > 0: continue
                a[('army', al)] += d['metal']; a[('army_t', tm)] += d['metal']
                third = 'W' if u[3] < 3413 else 'M' if u[3] < 6827 else 'E'
                a[('third', al, third)] += d['metal']
                if tm in OURS: a[('ours', tm, third)] += d['metal']
                if d.get('air'): a[('air', al)] += d['metal']
            teams_alive[mn] = cnt
print('seats:', ', '.join('%d %s (ally %d)' % (t, names[t], ally[t]) for t in sorted(names)))
print('over:', over)
print()
print('min | people army (W/M/E)  turrets | AI side army (W/M/E) turrets | ratio | t14 W/M/E | t16 W/M/E | other AI seats')
for mn in sorted(army):
    a = army[mn]
    oth = ' '.join('%d:%dk' % (tm, a[('army_t', tm)] / 1000) for tm in sorted(names) if ally[tm] == 1 and tm not in OURS)
    ppl = ' '.join('%d:%dk' % (tm, a[('army_t', tm)] / 1000) for tm in sorted(names) if ally[tm] == 0)
    print('%2d | %7d (%6d/%6d/%6d) %6d | %7d (%6d/%6d/%6d) %6d | %4.1f | %5d/%5d/%6d | %5d/%5d/%6d | %s || people %s' % (mn, a[('army', 0)], a[('third', 0, 'W')], a[('third', 0, 'M')], a[('third', 0, 'E')], a[('turrets', 0)], a[('army', 1)], a[('third', 1, 'W')], a[('third', 1, 'M')], a[('third', 1, 'E')], a[('turrets', 1)], a[('army', 1)] / max(a[('army', 0)], 1), a[('ours', OURS[0], 'W')], a[('ours', OURS[0], 'M')], a[('ours', OURS[0], 'E')], a[('ours', OURS[1], 'W')], a[('ours', OURS[1], 'M')], a[('ours', OURS[1], 'E')], oth, ppl))
print()
print('our seats metal by minute: bank, income, expense, excess (overflowed), sent, received')
tot = collections.Counter()
for mn in sorted(eco):
    e = eco[mn]; out = []
    for tm in OURS:
        n = max(e[(tm, 'n')], 1)
        out.append('t%d bank %5d inc %4d exp %4d excess %6d sent %6d recv %6d' % (tm, e[(tm, 'bank')] / n, e[(tm, 'inc')] / n, e[(tm, 'exp')] / n, e[(tm, 'excess')], e[(tm, 'sent')], e[(tm, 'recv')]))
        for k in ('excess', 'sent', 'recv'): tot[(tm, k)] += e[(tm, k)]
    print('%2d | %s' % (mn, ' | '.join(out)))
print('totals', dict(tot))
print()
print('orders by minute: people / Apex seats / t14 / t16 (all commands)')
ts = collections.Counter()
for mn in sorted(orders):
    o = orders[mn]; ppl = sum(v for (tm, c), v in o.items() if ally.get(tm) == 0); apex = sum(v for (tm, c), v in o.items() if ally.get(tm) == 1 and tm not in OURS)
    a = sum(v for (tm, c), v in o.items() if tm == OURS[0]); b = sum(v for (tm, c), v in o.items() if tm == OURS[1])
    ts['people'] += ppl; ts['apex'] += apex; ts['t14'] += a; ts['t16'] += b
    print('%2d | %6d | %6d | %6d | %6d' % (mn, ppl, apex, a, b))
print('totals', dict(ts))
print()
print('metal of the people\'s things killed, by killer team, by minute (-1 unknown/self)')
kt = collections.Counter()
for mn in sorted(kills):
    for k, v in kills[mn].items(): kt[k] += v
    print('%2d | %s' % (mn, ' '.join('%s:%dk' % (k, v / 1000) for k, v in sorted(kills[mn].items()) if v >= 500)))
print('totals', {k: int(v) for k, v in sorted(kt.items())})
print()
print('metal lost by team by minute (ours and the AI side)')
lt = collections.Counter()
for mn in sorted(lost):
    for k, v in lost[mn].items(): lt[k] += v
    print('%2d | %s' % (mn, ' '.join('%s:%dk' % (k, v / 1000) for k, v in sorted(lost[mn].items()) if v >= 500)))
print('totals', {k: int(v) for k, v in sorted(lt.items())})
print()
print('silo kills (grouped by 10 s and cell):')
g = collections.defaultdict(lambda: [0, 0.0, collections.Counter()])
for f, bt, vt, n, m, x, z in silo_kills:
    k = (f // 300, bt, ally.get(vt), cell(x, z)); g[k][0] += 1; g[k][1] += m; g[k][2][n] += 1
for (b, bt, va, c), (n, m, names_) in sorted(g.items()):
    s = b * 10
    print('%2d:%02d silo of t%s killed %d of ally %s at %s, %d metal: %s' % (s // 60, s % 60, bt, n, va, c, m, ', '.join('%s x%d' % kv for kv in names_.most_common(8))))
print()
print('units given between teams (minute, from, to): ', {k: v for k, v in sorted(given.items()) if v >= 5})
print('finished unit counts by team at the last minutes:')
for mn in sorted(teams_alive)[-4:]: print(mn, dict(sorted(teams_alive[mn].items())))
print('final team statistics:')
for tm in sorted(final_st):
    st = final_st[tm] or {}
    print(tm, names.get(tm), {k: int(v) for k, v in st.items()})
