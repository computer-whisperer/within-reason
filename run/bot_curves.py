#!/usr/bin/env python3
"""Curves of a batch played by a bot of bots/ (no match record): the bot's own line a game minute in engine.log.
usage: run/bot_curves.py <batch dir or label> [minute ...]   (default minutes 5 8 10 12 15 20)
Per minute, over the batch's games: the range and median of each number the line carries, and the results."""
import glob, json, re, statistics, sys

def main():
    name = sys.argv[1]
    batch = name if '/' in name else sorted(glob.glob(f'run/matches/*-{name}'))[-1]
    minutes = [int(m) for m in sys.argv[2:]] or [5, 8, 10, 12, 15, 20]
    games = sorted(glob.glob(f'{batch}/[0-9][0-9]'))
    lines = {}
    for game in games:
        for line in open(f'{game}/engine.log', errors='replace'):
            if '[distilled ai=' in line and ' f=' in line:
                frame = int(re.search(r' f=(\d+)', line).group(1))
                row = {k: float(v) for k, v in re.findall(r'(\w+)=([\d.]+)', line) if k not in ('ai', 'f')}
                row['solars'] = sum(float(x) for x in re.search(r'solars=([\d+]+)', line).group(1).split('+'))
                m = re.search(r'metal ([\d.]+)/([\d.]+) \+([\d.]+)(?: -([\d.]+))? energy ([\d.]+)/', line)
                row.update(metal_stored=float(m.group(1)), metal_income=float(m.group(3)), metal_spent=float(m.group(4) or 0), energy_stored=float(m.group(5)))
                row['army'] = row.get('home_guard', 0) + row.get('strike', 0)
                lines.setdefault(frame // 1800, []).append(row)
    results = [json.loads(l) for l in open(f'{batch}/results.jsonl')] if glob.glob(f'{batch}/results.jsonl') else []
    print(batch, '|', ' '.join(f"{r['outcome'][0]}{r['game_minutes']:.0f}" for r in results))
    keys = ['extractors', 'solars', 'plants', 'turrets', 'constructors', 'can_spend', 'metal_income', 'metal_spent', 'metal_stored', 'made', 'energy_stored', 'army']
    print('min  n  ' + '  '.join(f'{k:>16}' for k in keys))
    for minute in minutes:
        rows = lines.get(minute, [])
        if not rows:
            continue
        cell = lambda k: (lambda v: f'{min(v):.0f}-{max(v):.0f} ({statistics.median(v):.0f})')([r.get(k, 0) for r in rows])
        print(f'{minute:>3} {len(rows):>2}  ' + '  '.join(f'{cell(k):>16}' for k in keys))

main()
