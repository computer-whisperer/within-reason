#!/usr/bin/env python3
"""Both sides by minute for every game of a batch played by a bot of bots/ (no match record): the census lines in
engine.log (WITHIN_REASON_OBSERVE, which the arena always sets) for extractors, constructors, soldiers, plants with
construction turrets and defence turrets, ours against his; the bot's own line for its guard and body, units made,
the tier-2 pause and the stores; and from the truth file the clock of his first soldier within 1500 of our home.
The per-game companion of `run/bot_curves.py` (medians over a batch); `run/bot_orders.py` for what was ordered.

    run/bot_sides.py <batch dir or label> [minute ...]        (default minutes 3 4 5 6 8 10 12 15)
"""
import collections, glob, json, math, re, sys

CLASSES = [("mex", r"mex$|moho$"), ("energy", r"win$|solar$|advsol$|tide$|fus$|geo$"), ("conv", r"makr$"),
           ("store", r"mstor$|estor$"), ("lab", r"lab$|vp$|ap$|sy$|alab$|avp$|hp$"), ("nano", r"nanotc$"),
           ("turret", r"llt$|hllt$|hlt$|beamer$|rl$|guard$|pb$|dl$|ferret$|madsam$|mg$|rad$|jamt$|drag$|fort$"),
           ("com", r"^com$"), ("cons", r"^(ck|cv|ca|cs|ack|acv|aca|rectr|necro|fark|consul)$")]


def classify(name):
    suffix = name[3:]
    for label, pattern in CLASSES:
        if re.search(pattern, suffix):
            return label
    return "army"


def main():
    name = sys.argv[1]
    batch = name if '/' in name else sorted(glob.glob(f'run/matches/*-{name}'))[-1]
    minutes = [int(m) for m in sys.argv[2:]] or [3, 4, 5, 6, 8, 10, 12, 15]
    results = {json.loads(l)['index']: json.loads(l) for l in open(f'{batch}/results.jsonl')}
    for game in sorted(glob.glob(f'{batch}/[0-9][0-9]')):
        r = results[int(game[-2:])]
        census, own, home = collections.defaultdict(dict), {}, None
        for line in open(f'{game}/engine.log', errors='replace'):
            m = re.search(r"census f=(\d+) (enemy|own) ?(.*)", line)
            if m:
                census[int(m[1]) // 1800][m[2]] = {n: int(c) for n, c in re.findall(r"(\w+?)x(\d+)@", m[3])}
                continue
            if '[distilled ai=' not in line:
                continue
            h = re.search(r'home \((-?\d+), (-?\d+)\)', line)
            if h:
                home = (float(h[1]), float(h[2]))
            if ' f=' in line and 'extractors=' in line:
                f = int(re.search(r' f=(\d+)', line)[1])
                row = dict(re.findall(r'(\w+)=([\d.]+)', line))
                mm = re.search(r'metal ([\d.]+)/([\d.]+) \+([\d.]+)(?: -([\d.]+))? energy ([\d.]+)/([\d.]+) \+([\d.]+)', line)
                row.update(ms=float(mm[1]), mi=float(mm[3]), es=float(mm[5]), ei=float(mm[7]))
                own[f // 1800] = row
        first = None
        truth = glob.glob(f'{game}/truth-*.jsonl')
        if truth and home:
            for line in open(truth[0]):
                t = json.loads(line)
                near = [u for u in t['enemy'] if classify(u[1]) == 'army' and not u[5] and math.hypot(u[2] - home[0], u[3] - home[1]) < 1500]
                if near:
                    first = f"{t['f'] // 1800}:{(t['f'] // 30) % 60:02d} ({len(near)})"
                    break
        print(f"== {game[-2:]} seed {r['seed']} {r['our_side']} {r['outcome']} {r['game_minutes']:.1f} min; his first soldier within 1500 of our home: {first or 'never'}")
        print("  min | mex o/h | cons o/h | army o/h | lab+nano o/h | turr o/h | ours: guard+body made paused advp | metal st/inc | energy st/inc")
        for minute in minutes:
            c, o = census.get(minute, {}), own.get(minute)
            if not c and not o:
                continue
            cnt = lambda side, label: sum(n for name, n in c.get(side, {}).items() if classify(name) == label)
            cell = lambda label: f"{cnt('own', label):>2}/{cnt('enemy', label):<2}"
            labnano = f"{cnt('own', 'lab') + cnt('own', 'nano'):>2}/{cnt('enemy', 'lab') + cnt('enemy', 'nano'):<2}"
            ours = f"{o['home_guard']:>2}+{o['strike']:<2} {o['made']:>3} {o['paused']} {o['advplant']}" if o else "-"
            eco = f"{o['ms']:>5.0f}/{o['mi']:>4.0f} | {o['es']:>5.0f}/{o['ei']:>4.0f}" if o else "-"
            print(f"  {minute:>3} | {cell('mex')} | {cell('cons')} | {cell('army')} | {labnano} | {cell('turret')} | {ours:>20} | {eco}")


main()
