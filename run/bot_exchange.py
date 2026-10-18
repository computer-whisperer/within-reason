#!/usr/bin/env python3
"""The exchange by a minute of every game of a batch played by a bot of bots/ (no match record): his soldiers dead
of those seen, from the truth file (WITHIN_REASON_OBSERVE, which the arena always sets: every unit of his each
sample, so one seen before the minute and gone at it is dead), against ours, as the units the plants were told to
make (the bot's `made=` field) less the mobile units alive in the census at that minute. Soldiers are what is not an
extractor, generator, converter, store, plant, turret, constructor or commander (the classes of `run/bot_sides.py`).

    run/bot_exchange.py <batch dir or label>[,<label>...] [minute]        (default minute 10)

Written 2026-10-18 for the distilled bot's Apex batteries: by 10:00 the bot lost 10-25 soldiers for his 6-24 under
every rule set of the day, where the Opus player's record showed 7 lost for 9 of his, then a push with 43 against 13.
"""
import glob, json, re, sys

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


def exchange(game, minute):
    """(his dead, his seen, ours dead, ours made) by the minute, with None where a figure is missing."""
    truth = glob.glob(f'{game}/truth-*.jsonl')
    his_dead = his_seen = None
    if truth:
        seen, alive = {}, set()
        for line in open(truth[0]):
            t = json.loads(line)
            if t['f'] > minute * 1800:
                break
            soldiers = {u[0] for u in t['enemy'] if classify(u[1]) == 'army' and not u[5]}
            seen.update(dict.fromkeys(soldiers))
            alive = soldiers
        his_dead, his_seen = sum(1 for i in seen if i not in alive), len(seen)
    made = own_alive = None
    for line in open(f'{game}/engine.log', errors='replace'):
        m = re.search(r"census f=(\d+) own (.*)", line)
        if m and int(m[1]) == minute * 1800:
            own_alive = sum(int(c) for n, c in re.findall(r"(\w+?)x(\d+)@", m[2]) if classify(n) in ('army', 'cons'))
        m = re.search(r'\[distilled ai=\d\] f=(\d+) .* made=(\d+)', line)
        if m and int(m[1]) == minute * 1800:
            made = int(m[2])
    ours_dead = made - own_alive if made is not None and own_alive is not None else None
    return his_dead, his_seen, ours_dead, made


def main():
    minute = int(sys.argv[2]) if len(sys.argv) > 2 else 10
    for name in sys.argv[1].split(','):
        batch = name if '/' in name else sorted(glob.glob(f'run/matches/*-{name}'))[-1]
        rows = []
        for game in sorted(glob.glob(f'{batch}/[0-9][0-9]')):
            his_dead, his_seen, ours_dead, made = exchange(game, minute)
            fmt = lambda v: '?' if v is None else v
            rows.append(f"{game[-2:]}: his {fmt(his_dead):>2} of {fmt(his_seen):>2} / ours {fmt(ours_dead):>2} of {fmt(made)}")
        print(f"== {name} at {minute}:00 (his soldiers dead of seen / ours dead of made)")
        print('   ' + ' | '.join(rows))


main()
