#!/usr/bin/env python3
"""Replay of player-31's Lazarus seconds (6:21-6:52, K-hands-an-unarmed-party-taking-a-building-apart-reads-as-harmless):
the gate's nouls about party_9 as recorded, with the packet's unit names made internal (`names`), under the new
words (`words`: taking apart, unarmed), and under both (`words+names`). Then the 6:39 pick with the words of 6:39,
6:41, 6:44 and 6:48: stage one, and world 1 against the whole-group attack and against the hunt.

usage: run/jev_taking_ab.py   (writes docs/studies/data/taking-apart-2026-10-01.jsonl and prints the tables)
The key is read by run/jev_ab.py's api_key() and never printed or written."""
import collections, json, re, sys, os, copy, concurrent.futures, statistics
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from match_read import Match

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
M = os.path.join(ROOT, 'run/matches/1790828086-player-31-hard/00')
OUT = os.path.join(ROOT, 'docs/studies/data/taking-apart-2026-10-01.jsonl')
# The extractors' health by game second, from the record's samples: (spot_54) 26351, (spot_62) 22253.
H54 = {383: 100, 384: 93, 385: 83, 386: 72, 387: 64, 388: 54, 389: 44, 390: 34, 391: 23, 392: 16, 393: 4}
H62 = {400: 100, 401: 96, 402: 87, 403: 76, 404: 66, 405: 56, 406: 46, 407: 36, 408: 26, 409: 16, 410: 6}

def harm(sec):
    for table in (H54, H62):
        if sec in table and table[sec] < 100:
            t0 = min(table)
            h = table[sec]
            gone = h / ((100 - h) / (sec - t0))
            return f"taking apart our Metal Extractor (armmex), {h:.0f}% left and gone in about {gone:.0f} s"
    return None

NAMES = [(r'\bPawns?\b', 'armpw'), (r'\bTicks?\b', 'armflea'), (r'\bscout cars?\b', 'armfav'), (r'\bresurrection bots?\b', 'armrectr'), (r'\bRovers?\b', 'armfav'), (r'\bBlitz(es)?\b', 'armflash'), (r'\bStouts?\b', 'armstump')]

def internal(packet):
    for pat, name in NAMES:
        packet = re.sub(pat, name, packet)
    return packet

def new_words(text, what):
    """One question's or line's words as the new code writes them."""
    text = text.replace("it cannot hit us: nothing there shoots at what this group is", "it is unarmed and cannot fight back")
    text = re.sub(r"; its fastest \(\d+\) catch our slowest \(\d+\): it chooses the fight", "", text)
    text = text.replace("Yes when it is killing or about to kill", "Yes when it is killing, taking apart or about to kill")
    text = re.sub(r"met with (\d+) metal: (it outweighs us|an even fight|we outweigh it heavily|we outweigh it)", r"met with \1 metal: it is unarmed and cannot fight back", text) if 'party_9 (1 armrectr, 130 metal' in text else text
    if what:
        # the leave line, the answer question and the pick's lines: "party_9 (1 armrectr, at spot_62)"
        text = re.sub(r"party_9 \(1 armrectr, (at [a-z_0-9]+|in sight)\)", lambda m: f"party_9 (1 armrectr, {m.group(1)}, {what})", text)
        # the hunt: "hunt party_9 (1 armrectr)"
        text = text.replace("hunt party_9 (1 armrectr)", f"hunt party_9 (1 armrectr, {what})")
        # the commander: "(1 armrectr, 949 away)"
        text = re.sub(r"attacks party_9 \(1 armrectr, (\d+) away\)", lambda m: f"attacks party_9 (1 armrectr, {m.group(1)} away, {what} (50 metal) now)", text)
        # the pick's Met line
        text = re.sub(r"(party_9 \(1 armrectr, 130 metal, [^)]*\) met with \d+ metal: it is unarmed and cannot fight back)", lambda m: f"{m.group(1)}, {what}", text)
    return text

def new_state(state, what):
    s = json.dumps(state)
    s = s.replace("it cannot hit us: nothing there shoots at what this group is", "it is unarmed and cannot fight back")
    s = re.sub(r"; its fastest \(\d+\) catch our slowest \(\d+\): it chooses the fight", "", s)
    state = json.loads(s)
    if what:
        def walk(o):
            if isinstance(o, dict):
                return {k: walk(v) for k, v in o.items()}
            if isinstance(o, list):
                return [walk(v) for v in o]
            if isinstance(o, str) and o.startswith("party_9: "):
                return o + f"; {what} (50 metal) now"
            return o
        state = walk(state)
    return state

def variant(name, state, questions, packet, sec):
    state = copy.deepcopy(state)
    questions = copy.deepcopy(questions)
    if name == 'recorded':
        return state, questions
    if name == 'names':
        state['instructions'] = internal(packet)
        return state, questions
    what = harm(sec)
    state = new_state(state, what)
    for k, q in questions.items():
        if 'party_9' not in k and 'worlds' not in k:
            # other actors' questions that mention party_9 on their way keep the odds sentence change only
            if isinstance(q.get('instructions'), str):
                q['instructions'] = new_words(q['instructions'], None)
            continue
        if isinstance(q.get('instructions'), str) and 'worlds' not in k:
            q['instructions'] = new_words(q['instructions'], what)
        if 'criteria' in q:
            q['criteria'] = {w: new_words(line, what) for w, line in q['criteria'].items()}
    if name == 'words+names':
        state['instructions'] = internal(packet)
    return state, questions

def summarize(rows):
    def clock(s):
        return f'{s // 60}:{s % 60:02d}'
    agg = collections.defaultdict(lambda: collections.defaultdict(list))
    for r in rows:
        if r['kind'] != 'gate':
            continue
        a = r['answers']
        best = lambda pat: max([x['noul'] for k, x in a.items() if pat in k and not k.endswith('.forbidden') and 'group_W' not in k] or [0])
        forbid = max([x['noul'] for k, x in a.items() if k.endswith('.forbidden') and 'group_W' not in k] or [0])
        for key, value in (('answer', a['party_9.answer']['noul']), ('hunt', best('.hunt_')), ('whole', best('.whole_')), ('commander', a.get('party_9.attack_commander', {}).get('noul', 0)), ('forbidden', forbid)):
            agg[r['variant']][key].append(value)
    for name, a in agg.items():
        print(f"{name:12s} n={len(a['answer'])} answer median {statistics.median(a['answer']):.2f}, at 0.5 or over in {sum(x >= 0.5 for x in a['answer'])}; "
              f"picket hunt {statistics.median(a['hunt']):.2f}; whole group {statistics.median(a['whole']):.2f}; commander {statistics.median(a['commander']):.2f}; "
              f"picket hunt forbidden {statistics.median(a['forbidden']):.2f}, at 0.7 or over in {sum(x >= 0.7 for x in a['forbidden'])}")
    for r in rows:
        if r['kind'].startswith('pick'):
            a = r['answers']['worlds.pick']
            print(clock(r['sec']), f"{r['variant']:12s} {r['kind']:18s}", a.get('choice'), {k: round(v, 2) for k, v in sorted(a['probabilities'].items())})

def main():
    key = api_key()
    m = Match(M)
    gates, picks = [], []
    for c, state, packet in requests_of(m):
        f = c.get('f', 0)
        if not (11430 <= f <= 12570):
            continue
        if 'party_9.answer' in c['questions']:
            gates.append((c, state, packet))
        elif f == 11970 and 'worlds.pick' in c['questions']:
            picks.append((c, state, packet))
    print(len(gates), 'gate calls,', len(picks), 'pick calls at 6:39', file=sys.stderr)
    jobs = []
    for c, state, packet in gates:
        sec = c['f'] // 30
        for v in ('recorded', 'names', 'words', 'words+names'):
            st, qs = variant(v, state, c['questions'], packet, sec)
            # only party_9's questions are asked again
            qs = {k: q for k, q in qs.items() if k.startswith('party_9.')}
            jobs.append(('gate', sec, v, st, qs, c['model'], {k: c['answers'][k].get('noul') for k in qs}))
    # The pick of 6:39 with the words of 6:41, 6:44 and 6:48 (96%, 66%, 26% left): stage one, and world 1 against
    # the whole-group attack and against the hunt.
    stage1 = [c for c, _, _ in picks if 'w1' not in c['questions']['worlds.pick']['criteria']][0]
    stage2 = [c for c, _, _ in picks if 'w1' in c['questions']['worlds.pick']['criteria']][0]
    state, packet = picks[0][1], picks[0][2]
    lines = dict(stage1['questions']['worlds.pick']['criteria'])
    lines['w1'] = stage2['questions']['worlds.pick']['criteria']['w1']
    for sec in (399, 401, 404, 408):
        for v in ('recorded', 'words', 'words+names'):
            for label, keep in (('stage1', [w for w in lines if w != 'w1']), ('w1_vs_whole', ['w1', 'w4']), ('w1_vs_hunt', ['w1', 'w2'])):
                q = copy.deepcopy(stage2['questions']['worlds.pick'] if label != 'stage1' else stage1['questions']['worlds.pick'])
                q['criteria'] = {w: lines[w] for w in keep}
                st, qs = variant(v, state, {'worlds.pick': q}, packet, sec)
                jobs.append(('pick:' + label, sec, v, st, qs, stage2['model'], None))
    def run(job):
        kind, sec, v, st, qs, model, recorded = job
        answer = ask(key, st, qs, model)
        return {'kind': kind, 'sec': sec, 'variant': v, 'recorded': recorded, 'answers': answer.get('answers', answer), 'usage': answer.get('usage')}
    rows = []
    with concurrent.futures.ThreadPoolExecutor(8) as pool:
        for row in pool.map(run, jobs):
            rows.append(row)
    with open(OUT, 'w') as out:
        for row in rows:
            out.write(json.dumps(row) + '\n')
    tokens = sum((r['usage'] or {}).get('input_tokens', 0) for r in rows)
    print(f"{len(rows)} calls, {tokens} input tokens, ${tokens * 0.042 / 1e6:.3f}", file=sys.stderr)
    summarize(rows)

if __name__ == '__main__':
    main()
