#!/usr/bin/env python3
"""Static defence over our match pool, from the records: per match and seat, the defence metal finished (ground / anti-air),
the metal its defences killed (the other side's units whose killer type is a defence), the defence metal lost, the army metal
finished, and who the seat was. Reads every record-<n>.jsonl of a match dir (bot records: our seat's fair view with
`destroyed`+`by_d` and `enemy_destroyed`+`by_d`; replay records from the dump widget: one per team, `destroyed`+`by`).
    run/defence_pool.py OUT.jsonl DIR...        (DIR is a batch dir or a match dir; rows appended)
    run/defence_pool.py --summary ROWS.jsonl...  quartiles by seat
"""
import json, os, re, sys, subprocess, glob, collections
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
U = json.load(open(f'{REPO}/crates/bot/data/units.json'))['units']
GROUND = {k for k, v in U.items() if v['class'] in ('defence turret', 'artillery turret', 'torpedo launcher', 'armed metal extractor', 'armed geothermal plant', 'long-range cannon')}
AA = {k for k, v in U.items() if v['class'] == 'anti-air turret'}
DEF = GROUND | AA
MOBILE = {k for k, v in U.items() if v.get('speed') and (v.get('dps') or v.get('dps_air')) and 'construct' not in v['class'] and 'commander' not in v['class']}
EV = re.compile(r'\{[^{}]*"k": ?"(?:created|finished|destroyed|enemy_destroyed|enemy_seen)"[^{}]*\}')

def events(path):
    p = subprocess.run(['grep', '-o', r'{[^{}]*"k": \?"\(created\|finished\|destroyed\|enemy_destroyed\|enemy_seen\)"[^{}]*}', path], capture_output=True, text=True)
    for line in p.stdout.splitlines():
        try: yield json.loads(line)
        except ValueError: pass

def header(path):
    with open(path) as fh: return json.loads(fh.readline())

def seat_name(h, team):
    for s in h.get('seats') or []:
        if s.get('team') == team:
            c = s.get('controller')
            if isinstance(c, dict):
                if 'Person' in c: return 'person:' + str(c['Person'].get('name'))
                if 'Ai' in c: return 'ai:' + str(c['Ai'].get('short_name') or c['Ai'].get('name'))
            return str(c)
    return '?'

def scan_match(d):
    recs = sorted(glob.glob(os.path.join(d, 'record-*.jsonl')))
    if not recs: return []
    heads = {}
    for r in recs:
        try: heads[r] = header(r)
        except Exception: pass
    if not heads: return []
    h0 = next(iter(heads.values()))
    defs = h0.get('unit_defs', [])  # events index the header's list by position (viewer/record.js seat.defs[u[1]])
    name = lambda i: defs[i]['name'] if isinstance(i, int) and 0 <= i < len(defs) else '?'
    metal = lambda i: (defs[i].get('metal') or 0) if isinstance(i, int) and 0 <= i < len(defs) else 0
    mode = h0.get('mode')
    teams = {}
    owner = {}  # unit id -> team, from created events (replay records) or our own
    rows = []
    if mode == 'replay':
        per = {}
        for r, h in heads.items():
            t = h.get('team'); per[t] = list(events(r))
            for e in per[t]:
                if e['k'] == 'created': owner[e['u']] = t
        for t, evs in per.items():
            row = dict(match=d, mode=mode, team=t, seat=seat_name(heads[[r for r, h in heads.items() if h.get('team') == t][0]], t), ground=0, aa=0, army=0, def_lost=0, kills_by_def=0, kills_by_def_n=0, types=collections.Counter(), first_def=None)
            teams[t] = row
        for t, evs in per.items():
            for e in evs:
                n = name(e['d'])
                if e['k'] == 'finished':
                    if n in GROUND: teams[t]['ground'] += metal(e['d']); teams[t]['types'][n] += 1
                    elif n in AA: teams[t]['aa'] += metal(e['d']); teams[t]['types'][n] += 1
                    elif n in MOBILE: teams[t]['army'] += metal(e['d'])
                    if n in DEF and (teams[t]['first_def'] is None or e['f'] < teams[t]['first_def']): teams[t]['first_def'] = e['f']
                elif e['k'] == 'destroyed':
                    if n in DEF: teams[t]['def_lost'] += metal(e['d'])
                    kd = name(e['by_d']) if e.get('by_d') is not None else None
                    kt = owner.get(e.get('by'))
                    if kd in DEF and kt is not None and kt != t and kt in teams:
                        teams[kt]['kills_by_def'] += metal(e['d']); teams[kt]['kills_by_def_n'] += 1
        rows = list(teams.values())
    else:
        for r, h in heads.items():
            t = h.get('team')
            us = dict(match=d, mode=mode, team=t, seat='ours:' + str(mode), ground=0, aa=0, army=0, def_lost=0, kills_by_def=0, kills_by_def_n=0, types=collections.Counter(), first_def=None)
            his = dict(match=d, mode=mode, team='enemy', seat='enemy:' + (str(h.get('opponent')) if h.get('opponent') else '?'), ground=0, aa=0, army=0, def_lost=0, kills_by_def=0, kills_by_def_n=0, types=collections.Counter(), first_def=None, seen_only=True)
            seen = {}
            for e in events(r):
                n = name(e['d'])
                if e['k'] == 'finished':
                    if n in GROUND: us['ground'] += metal(e['d']); us['types'][n] += 1
                    elif n in AA: us['aa'] += metal(e['d']); us['types'][n] += 1
                    elif n in MOBILE: us['army'] += metal(e['d'])
                    if n in DEF and (us['first_def'] is None or e['f'] < us['first_def']): us['first_def'] = e['f']
                elif e['k'] == 'destroyed':
                    if n in DEF: us['def_lost'] += metal(e['d'])
                    kd = name(e['by_d']) if e.get('by_d') is not None else None
                    if kd in DEF: his['kills_by_def'] += metal(e['d']); his['kills_by_def_n'] += 1
                elif e['k'] == 'enemy_seen':
                    if n in DEF and e['u'] not in seen:
                        seen[e['u']] = n
                        if n in GROUND: his['ground'] += metal(e['d'])
                        else: his['aa'] += metal(e['d'])
                        his['types'][n] += 1
                elif e['k'] == 'enemy_destroyed':
                    if n in DEF: his['def_lost'] += metal(e['d'])
                    kd = name(e['by_d']) if e.get('by_d') is not None else None
                    if kd in DEF and (e.get('by_ours') or e.get('by') is not None): us['kills_by_def'] += metal(e['d']); us['kills_by_def_n'] += 1
            # the seats from the header name the opponent better
            for s in h.get('seats') or []:
                if s.get('ally_team') != h.get('ally_team') and s.get('controller') not in (None, 'Gaia', 'Unknown'):
                    his['seat'] = 'enemy:' + seat_name(h, s.get('team')); break
            rows += [us, his]
    return rows

if __name__ == '__main__' and sys.argv[1:2] != ['--summary']:
    out = open(sys.argv[1], 'a')
    n = 0
    for d in sys.argv[2:]:
        dirs = [d] if glob.glob(os.path.join(d, 'record-*.jsonl')) else sorted(glob.glob(os.path.join(d, '*/record-*.jsonl')))
        dirs = sorted({os.path.dirname(x) if x.endswith('.jsonl') else x for x in dirs})
        for m in dirs:
            try: rows = scan_match(m)
            except Exception as e: print('fail', m, e, file=sys.stderr); continue
            for r in rows: out.write(json.dumps(r, default=dict) + '\n')
            n += 1
    out.close(); print(n, 'matches scanned')


def summary(paths):
    """Quartiles by seat over the rows the scan wrote (`defence_pool.py --summary ROWS.jsonl...`)."""
    import statistics as st
    rows = [json.loads(l) for p in paths for l in open(p)]
    q = lambda xs: (lambda s: f"{s[len(s)//4]:.0f}/{s[len(s)//2]:.0f}/{s[3*len(s)//4]:.0f}" if s else '-')(sorted(xs))
    # A replay record whose header names no seats is read from the match's card.json (team -> player with OS, or AI).
    cards = {}
    def card_seat(r):
        m = r['match']
        if m not in cards:
            # the card may sit beside the records, or under run/matches by the same name (the records of carded replays
            # were moved to the archive while the cards stayed, docs/harness/pitfalls.md)
            cp = os.path.join(m, 'card.json')
            if not os.path.exists(cp): cp = os.path.join(REPO, 'run', 'matches', os.path.basename(m.rstrip('/')), 'card.json')
            cards[m] = {}
            if os.path.exists(cp):
                try:
                    for t in json.load(open(cp))['teams']: cards[m][t['team']] = t
                except Exception: pass
        t = cards[m].get(r['team'])
        if not t: return None
        r['won'] = t.get('won')
        ai = t.get('ai')
        if ai: return 'ai:' + str(ai.get('short_name') if isinstance(ai, dict) else ai)
        if t.get('player'):
            os_ = t.get('os') or 0
            return 'person OS 35+' if os_ >= 35 else ('person OS 20-35' if os_ >= 20 else 'person OS <20')
        return None
    # Kills credited to our defences need `by_d` on `enemy_destroyed`, in records from 2026-10-18 on (unix 1792400000).
    recent = lambda r: int(os.path.basename(r['match'].rstrip('/').split('/')[-2 if r['match'].rstrip('/')[-2:].isdigit() and r['match'].rstrip('/')[-3] == '/' else -1]).split('-')[0] or 0) >= 1792400000 if r['mode'] != 'replay' else True
    # Game size: the teams in the card, else the seats in the first record's header (duel: 2 teams; team: more).
    sizes = {}
    def size(r):
        m = r['match']
        if m not in sizes:
            n = len(cards.get(m) or {})
            if not n:
                try:
                    rec = sorted(glob.glob(os.path.join(m, 'record-*.jsonl')))[0]
                    with open(rec) as fh: h = json.loads(fh.readline())
                    n = len([x for x in h.get('seats') or [] if x.get('controller') not in (None, 'Gaia')]) or len(glob.glob(os.path.join(m, 'record-*.jsonl')))
                except Exception: n = 0
            sizes[m] = n
        n = sizes[m]
        return 'duel' if n == 2 else ('team' if n > 2 else '?')
    g = collections.defaultdict(list)
    for r in rows:
        s = r['seat']
        if s == '?' and r['mode'] == 'replay': s = card_seat(r) or '? (no card)'
        elif s.startswith('person'): s = 'person (lobby, in our games)'
        elif s.startswith('enemy:person'): s = 'enemy:person'
        s = f"{s} [{size(r)}]"
        try: r['recent'] = recent(r)
        except Exception: r['recent'] = False
        g[s].append(r)
    print(f"{'seat':28}{'n':>5}{'won':>5} {'ground def built q':>20} {'army q':>20} {'def/army% med':>14} {'killed by def q':>20} {'killed/def med':>15} {'def lost/def med':>17} {'first def min':>14}")
    print("  (killed-by-defence columns: replay records all; bot records only from 2026-10-18, when the killer's type was added)")
    for s, rs in sorted(g.items(), key=lambda kv: -len(kv[1])):
        kr = [r for r in rs if r.get('recent')]
        kd = [r['kills_by_def'] / r['ground'] for r in kr if r['ground'] > 0]; dl = [r['def_lost'] / r['ground'] for r in rs if r['ground'] > 0]
        pct = [100 * r['ground'] / r['army'] for r in rs if r['army'] > 0]
        fd = [r['first_def'] / 1800 for r in rs if r['first_def'] is not None]
        won = sum(1 for r in rs if r.get('won'))
        print(f"{s[:27]:28}{len(rs):>5}{won:>5} {q([r['ground'] for r in rs]):>20} {q([r['army'] for r in rs]):>20} {st.median(pct) if pct else 0:>14.0f} {q([r['kills_by_def'] for r in kr]):>20} {st.median(kd) if kd else 0:>15.1f} {st.median(dl) if dl else 0:>17.1f} {st.median(fd) if fd else -1:>14.1f}")
        tot = collections.Counter()
        for r in rs: tot.update(r['types'])
        print('      types: ' + ', '.join(f"{k} {v}" for k, v in tot.most_common(7)))


if __name__ == '__main__' and sys.argv[1:2] == ['--summary']:
    summary(sys.argv[2:])
