#!/usr/bin/env python3
"""Curves of a batch played by a bot of bots/ (no match record): the bot's own line a game minute in engine.log.
usage: run/bot_curves.py <batch dir or label> [minute ...]   (default minutes 5 8 10 12 15 20)
Per minute, over the batch's games: the range and median of each number the line carries, and the results; then the
tier-2 milestones of each game from the bot's `t2:` lines (the advanced plant, constructors, extractors, fusions)."""
import glob, json, re, statistics, sys

def main():
    name = sys.argv[1]
    batch = name if '/' in name else sorted(glob.glob(f'run/matches/*-{name}'))[-1]
    minutes = [int(m) for m in sys.argv[2:]] or [5, 8, 10, 12, 15, 20]
    games = sorted(glob.glob(f'{batch}/[0-9][0-9]'))
    lines, milestones, begun = {}, {}, {}
    for game in games:
        for line in open(f'{game}/engine.log', errors='replace'):
            if '[distilled ai=' not in line or ' f=' not in line:
                continue
            frame = int(re.search(r' f=(\d+)', line).group(1))
            if ' t2: ' in line:
                what = re.search(r' t2: (.*?) (standing|begun), (\d+) of them', line)
                if what and what.group(2) == 'standing':
                    milestones.setdefault(game[-2:], {}).setdefault(what.group(1), []).append((frame / 1800, int(what.group(3))))
                elif what:
                    begun.setdefault(game[-2:], {}).setdefault(what.group(1), []).append(frame / 1800)
                continue
            if 'extractors=' not in line:
                continue
            row = {k: float(v) for k, v in re.findall(r'(\w+)=([\d.]+)', line) if k not in ('ai', 'f')}
            row['solars'] = sum(float(x) for x in re.search(r'solars=([\d+]+)', line).group(1).split('+'))
            m = re.search(r'metal ([\d.]+)/([\d.]+) \+([\d.]+)(?: -([\d.]+))? energy ([\d.]+)/([\d.]+) \+([\d.]+)', line)
            row.update(metal_stored=float(m.group(1)), metal_income=float(m.group(3)), metal_spent=float(m.group(4) or 0), energy_stored=float(m.group(5)), energy_income=float(m.group(7)))
            row['army'] = row.get('home_guard', 0) + row.get('strike', 0)
            lines.setdefault(frame // 1800, []).append(row)
    results = [json.loads(l) for l in open(f'{batch}/results.jsonl')] if glob.glob(f'{batch}/results.jsonl') else []
    print(batch, '|', ' '.join(f"{r['outcome'][0]}{r['game_minutes']:.0f}" for r in results))
    keys = ['extractors', 'solars', 'plants', 'turrets', 'constructors', 'can_spend', 'metal_income', 'metal_spent', 'metal_stored', 'energy_stored', 'energy_income', 'made', 'army', 'advplant', 'acv', 'mohos', 'fusions', 'converters', 'assist', 't2_made', 'paused']
    print('min  n  ' + '  '.join(f'{k:>16}' for k in keys))
    for minute in minutes:
        rows = lines.get(minute, [])
        if not rows:
            continue
        cell = lambda k: (lambda v: f'{min(v):.0f}-{max(v):.0f} ({statistics.median(v):.0f})')([r.get(k, 0) for r in rows])
        print(f'{minute:>3} {len(rows):>2}  ' + '  '.join(f'{cell(k):>16}' for k in keys))
    if milestones:
        print('tier 2 by game (the minute each first stood; tier-2 extractors: first, and how many by the end):')
        for game in sorted(milestones):
            m = milestones[game]
            first = lambda what: f"{m[what][0][0]:.1f}" if what in m else '-'
            mohos = f"{m['tier-2 extractor'][0][0]:.1f}, {m['tier-2 extractor'][-1][1]} by the end" if 'tier-2 extractor' in m else '-'
            def took(what):
                # A project's build: from its frame's first appearance to its standing, in minutes.
                starts, ends = begun.get(game, {}).get(what, []), [t for t, _ in m.get(what, [])]
                return ', '.join(f"{e - s:.1f}" for s, e in zip(starts, ends)) or '-'
            print(f"  {game}: advanced plant {first('advanced plant')} | advanced constructor {first('advanced constructor')} | tier-2 extractor {mohos} | fusion {first('fusion')} | advanced fusion {first('advanced fusion')} | converter {first('converter')} | fast assist {first('fast assist unit')}"
                  f" | took (min): plant {took('advanced plant')}, fusion {took('fusion')}, advanced fusion {took('advanced fusion')}")

main()
