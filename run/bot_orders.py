#!/usr/bin/env python3
"""The build orders of a bot of bots/ in a batch, by game clock: the host's build-trace lines in engine.log
(`[wreason ai=N] build unit=U def=D wanted=(x,z) placed=(x,z) ...` and `no site ...`, written when the arena sets
WITHIN_REASON_TRACE_BUILDS, which it always does). Those lines carry no frame of their own, so each is clocked by
the last framed line before it (the engine's `[f=N]` prefixes and the bot's own `f=N` lines), to within seconds.

    run/bot_orders.py <batch dir or label> [--full NN] [--unit U] [--defs RECORD]

Per game: the orderer of the first construction turret and whether it was the commander, the second and third
plant, the advanced plant, sites refused, and the orders by 10:00 by type. `--full NN` lists every order of game NN;
`--unit U` lists one unit's orders in every game (the commander is the first unit to order anything).
Unit-def names come from a flagship record's header (`--defs`, else the newest record-*.jsonl under run/matches):
the engine numbers unit defs the same way in every game of one game version.
"""
import collections, glob, json, os, re, sys


def def_names(path):
    for line in open(path, errors='replace'):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        if r.get('t') == 'header':
            return {d['id']: d['name'] for d in r['unit_defs']}
    return {}


def clock(frame):
    return f'{frame // 1800}:{(frame // 30) % 60:02d}'


def orders_of(game, names):
    """(frame, unit, def name, site refused, the raw line) for every build order of a game, in order."""
    frame, out = 0, []
    for line in open(f'{game}/engine.log', errors='replace'):
        m = re.match(r'\[t=[^\]]*\]\[f=(\d+)\]', line)
        if m:
            frame = max(frame, int(m[1]))
        m = re.search(r'\[distilled ai=\d\] f=(\d+)', line)
        if m:
            frame = max(frame, int(m[1]))
        if '[wreason ai=' in line and ('build unit=' in line or 'no site' in line):
            m = re.search(r'unit=(\d+) def=(\d+)', line)
            out.append((frame, int(m[1]), names.get(int(m[2]), m[2]), 'no site' in line, line.strip()))
    return out


def main():
    args = sys.argv[1:]
    name = args[0]
    batch = name if '/' in name else sorted(glob.glob(f'run/matches/*-{name}'))[-1]
    full = args[args.index('--full') + 1] if '--full' in args else None
    unit = int(args[args.index('--unit') + 1]) if '--unit' in args else None
    record = args[args.index('--defs') + 1] if '--defs' in args else max(glob.glob('run/matches/*/*/record-*.jsonl'), key=os.path.getmtime)
    names = def_names(record)
    results = {json.loads(l)['index']: json.loads(l) for l in open(f'{batch}/results.jsonl')} if os.path.exists(f'{batch}/results.jsonl') else {}
    for game in sorted(glob.glob(f'{batch}/[0-9][0-9]')):
        orders = orders_of(game, names)
        if not orders:
            continue
        r = results.get(int(game[-2:]), {})
        commander = orders[0][1]
        placed = [o for o in orders if not o[3]]
        first = lambda suffix, nth=1: next((o for i, o in enumerate(x for x in placed if x[2].endswith(suffix)) if i + 1 == nth), None)
        nano, plant2, plant3, avp = first('nanotc'), first('vp', 2), first('vp', 3), first('avp')
        at = lambda o: clock(o[0]) if o else '-'
        who = lambda o: ('the commander' if o[1] == commander else f'unit {o[1]}') if o else '-'
        by10 = collections.Counter(o[2] for o in placed if o[0] < 10 * 1800)
        print(f"== {game[-2:]} seed {r.get('seed', '?')} {r.get('our_side', '')} {r.get('outcome', '')} {r.get('game_minutes', 0):.1f} min: first construction turret {at(nano)} by {who(nano)}, "
              f"2nd plant {at(plant2)} by {who(plant2)}, 3rd {at(plant3)}, advanced plant {at(avp)}, sites refused {sum(o[3] for o in orders)}")
        print('   by 10:00:', ', '.join(f'{n} {c}' for n, c in by10.most_common()))
        for f, u, n, refused, raw in orders:
            if full == game[-2:] or (unit is not None and u == unit):
                print(f'   {clock(f)} unit={u} {"NO SITE " if refused else ""}{n} {re.sub(r"^.*wanted=", "wanted=", raw)[:90]}')


main()
