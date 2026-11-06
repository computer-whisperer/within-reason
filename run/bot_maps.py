#!/usr/bin/env python3
"""One block a batch for a bot of bots/ played over several maps (no match record): what the game on that map was,
ours beside his, from the census lines and the bot's own lines in engine.log. The cross-map companion of
`run/bot_sides.py` (one game by minute) and `run/bot_curves.py` (one batch by minute).

    run/bot_maps.py <batch dir or label> [<batch dir or label> ...]
    run/bot_maps.py --results <batch dir or label> ...      one line a batch and the sum

Per batch: the result and the wins' clock; when the first plant was begun and the first soldier stood; extractors,
soldiers and turrets of both sides and our income at 5, 10, 15 and 20 minutes (medians over the games still running);
the first body's clock and the bodies sent; the first minute we had twice his soldiers against the first body and
the end; the last minute of every game not won (extractors, soldiers, turrets,
our guard and body, the bank, and what was left of his when it was little); the types he fielded most and the types that killed most of ours.
"""
import collections, glob, json, re, statistics, sys

from bot_sides import classify

MINUTES = [5, 10, 15, 20]


def med(values):
    values = [v for v in values if v is not None]
    return statistics.median(values) if values else None


def show(value, digits=0):
    return "-" if value is None else f"{value:.{digits}f}"


def clock(frame):
    return "-" if frame is None else f"{int(frame) // 1800}:{(int(frame) // 30) % 60:02d}"


def batch_of(name):
    return name if '/' in name else sorted(glob.glob(f'run/matches/*-{name}'))[-1]


def read(game):
    census, own = collections.defaultdict(dict), {}
    plant = soldier = launch = None
    launches = 0
    killers = collections.Counter()
    lost = collections.Counter()
    for line in open(f'{game}/engine.log', errors='replace'):
        m = re.search(r"census f=(\d+) (enemy|own) ?(.*)", line)
        if m:
            census[int(m[1]) // 1800][m[2]] = {n: int(c) for n, c in re.findall(r"(\w+?)x(\d+)@", m[3])}
            continue
        if '[distilled ai=' not in line or ' f=' not in line:
            continue
        f = int(re.search(r' f=(\d+)', line)[1])
        if 'extractors=' in line:
            row = dict(re.findall(r'(\w+)=([\d.]+)', line))
            mm = re.search(r'metal ([\d.]+)/([\d.]+) \+([\d.]+)', line)
            if mm:
                row.update(ms=float(mm[1]), mcap=float(mm[2]), mi=float(mm[3]))
            own[f // 1800] = row
            if soldier is None and float(row.get('made', 0)) > 0:
                soldier = f
        elif ' begun: ' in line and plant is None and re.search(r' begun: \w+(vp|lab|hp|ap|sy) ', line):
            plant = f
        elif ' launch: ' in line:
            if 'joins a body of n=0 ' in line:  # a new body, not followers for one that is out
                launches += 1
                launch = f if launch is None else launch
        elif ' died: ' in line:
            d = re.search(r' died: (\w+) unit \d+ .*? body=(\w+) .*? killer=(\w+)', line)
            if d:
                killers[d[3]] += 1
                lost[d[2]] += 1
    return dict(census=census, own=own, plant=plant, soldier=soldier, launch=launch, launches=launches, killers=killers, lost=lost)


def count(census, minute, side, label):
    c = census.get(minute)
    if not c or side not in c:
        return None
    return sum(n for name, n in c[side].items() if classify(name) == label)


def results_table(names):
    """`--results`: one line a batch (result, the wins' clock, unwon games we led two to one in soldiers at the end)
    and the sum, for laying benches side by side."""
    total = collections.Counter()
    for name in names:
        batch = batch_of(name)
        meta = json.load(open(f'{batch}/batch.json')) if glob.glob(f'{batch}/batch.json') else {}
        results = [json.loads(l) for l in open(f'{batch}/results.jsonl')]
        outcomes = collections.Counter(r['outcome'] for r in results)
        total.update(outcomes)
        wins = [r['game_minutes'] for r in results if r['outcome'] == 'Win']
        led = 0
        for r in results:
            if r['outcome'] == 'Win':
                continue
            g = read(f"{batch}/{r['index']:02d}")
            last = max((m for m in g['own'] if m in g['census']), default=None)
            if last is not None and (count(g['census'], last, 'own', 'army') or 0) >= max(5, 2 * (count(g['census'], last, 'enemy', 'army') or 0)):
                led += 1
        print(f"{batch.split('-', 1)[1]:40} {(meta.get('map') or '')[:22]:22} {outcomes['Win']:>2}-{outcomes['Loss']}-{outcomes['Timeout']}  wins at {show(med(wins), 1):>5}  unwon while two to one in soldiers: {led}")
    print(f"{'':63} {total['Win']}-{total['Loss']}-{total['Timeout']}")


def main():
    if sys.argv[1:2] == ['--results']:
        return results_table(sys.argv[2:])
    for name in sys.argv[1:]:
        batch = batch_of(name)
        meta = json.load(open(f'{batch}/batch.json')) if glob.glob(f'{batch}/batch.json') else {}
        results = {json.loads(l)['index']: json.loads(l) for l in open(f'{batch}/results.jsonl')}
        games = {int(g[-2:]): read(g) for g in sorted(glob.glob(f'{batch}/[0-9][0-9]'))}
        outcomes = collections.Counter(r['outcome'] for r in results.values())
        wins = [r['game_minutes'] for r in results.values() if r['outcome'] == 'Win']
        losses = [r['game_minutes'] for r in results.values() if r['outcome'] == 'Loss']
        print(f"== {meta.get('map') or batch.split('-', 1)[1]}: {outcomes['Win']}-{outcomes['Loss']}-{outcomes['Timeout']}"
              f" | wins at {show(med(wins), 1)} min, losses at {show(med(losses), 1)}"
              f" | plant begun {clock(med([g['plant'] for g in games.values()]))}, first soldier by {clock(med([g['soldier'] for g in games.values()]))}"
              f" | first body {clock(med([g['launch'] for g in games.values()]))}, bodies a game {show(med([g['launches'] for g in games.values()]))}")
        # The first minute our soldiers were twice his (five or more of ours), against the first body and the end.
        leads = []
        for index, g in games.items():
            lead = next((m for m in sorted(g['census']) if (count(g['census'], m, 'own', 'army') or 0) >= max(5, 2 * (count(g['census'], m, 'enemy', 'army') or 0))), None)
            leads.append((lead, g['launch'] / 1800 if g['launch'] is not None else None, results[index]['game_minutes'], results[index]['outcome']))
        had = [l for l in leads if l[0] is not None]
        print(f"   twice his soldiers from minute {show(med([l[0] for l in had]))} ({len(had)} of {len(leads)} games); from there to the first body {show(med([l[1] - l[0] for l in had if l[1] is not None]), 1)} min"
              f" ({sum(1 for l in had if l[1] is None)} never sent one), to the end {show(med([l[2] - l[0] for l in had]), 1)} min")
        print("   min  n | mex o/h | army o/h | turrets o/h | builders o/h | income  bank | guard+body")
        for minute in MINUTES:
            live = [g for g in games.values() if minute in g['own'] and minute in g['census']]
            if not live:
                continue
            cell = lambda label: f"{show(med([count(g['census'], minute, 'own', label) for g in live])):>3}/{show(med([count(g['census'], minute, 'enemy', label) for g in live])):<3}"
            own = lambda key: med([float(g['own'][minute][key]) for g in live if key in g['own'][minute]])
            print(f"   {minute:>3} {len(live):>2} | {cell('mex')} | {cell('army')}  | {cell('turret')}     | {cell('cons')}      | {show(own('mi')):>5} {show(own('ms')):>5} | {show(own('home_guard')):>3}+{show(own('strike'))}")
        for index, r in sorted(results.items()):
            if r['outcome'] == 'Win' or index not in games:
                continue
            g = games[index]
            last = max((m for m in g['own'] if m in g['census']), default=None)
            if last is None:
                print(f"   {index:02d} {r['outcome']:<7} {r['game_minutes']:.1f}: no minute read")
                continue
            c = lambda side, label: count(g['census'], last, side, label)
            o = g['own'][last]
            print(f"   {index:02d} {r['outcome']:<7} {r['our_side']} {r['game_minutes']:>4.1f} | at {last}: mex {c('own', 'mex')}/{c('enemy', 'mex')} army {c('own', 'army')}/{c('enemy', 'army')}"
                  f" turrets {c('own', 'turret')}/{c('enemy', 'turret')} builders {c('own', 'cons')}/{c('enemy', 'cons')} | guard {o.get('home_guard')} body {o.get('strike')} made {o.get('made')}"
                  f" income {o.get('mi', 0):.0f} bank {o.get('ms', 0):.0f} | bodies {g['launches']}")
            left = g['census'][last].get('enemy', {})
            if sum(left.values()) <= 30:  # what is left of a side we could not finish
                print("        his at that minute: " + " ".join(f"{u}x{n}" for u, n in sorted(left.items())))
        his, killers, lost = collections.Counter(), collections.Counter(), collections.Counter()
        for g in games.values():
            for minute, c in g['census'].items():
                for unit, n in c.get('enemy', {}).items():
                    if classify(unit) == 'army':
                        his[unit] = max(his[unit], 0) + n
            killers.update(g['killers'])
            lost.update(g['lost'])
        total = sum(his.values()) or 1
        print("   he fielded (share of his soldier-minutes): " + ", ".join(f"{u} {100 * n / total:.0f}%" for u, n in his.most_common(6)))
        print("   killed ours: " + ", ".join(f"{u} {n}" for u, n in killers.most_common(6)) + " | we lost: " + ", ".join(f"{u} {n}" for u, n in lost.most_common(6)))


main()
