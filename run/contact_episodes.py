#!/usr/bin/env python3
"""Contact episodes of declared groups from a match record: runs of seconds with `engaged` set (gaps of 3 s or less
merged), for groups of --least members or more. Per episode: start, seconds, members, halted seconds (pace 0, not
arrived), share of members with an armed enemy in sight within their reach (the stand rule's test, without
hysteresis), formed share, anchor travel, what was engaged, our members' metal lost and his metal killed within 900 of
the anchor while it lasted.

usage: run/contact_episodes.py <match dir> [--least N] [--top N]

The record does not carry the halt itself (`Group::stopped`): "halted" is a pace of 0 short of arrival, which the drag
alone also gives. "in reach" leaves out the stand rule's hysteresis (reach + `STAND_KEEP` once standing).
"""
import sys, json, math, collections, argparse, glob

ap = argparse.ArgumentParser()
ap.add_argument('dir'); ap.add_argument('--least', type=int, default=6); ap.add_argument('--top', type=int, default=25)
a = ap.parse_args()
hdr = None; samples = []; deaths = []
for l in open(sorted(glob.glob(a.dir + '/record-*.jsonl'))[0]):
    try: r = json.loads(l)
    except ValueError: continue
    t = r.get('t')
    if t == 'header' and hdr is None: hdr = r
    elif t == 's': samples.append(r)
    elif t == 'ev' and r.get('k') in ('destroyed', 'enemy_destroyed') and r.get('c') != 'decayed': deaths.append(r)
defs = hdr['unit_defs']
name = lambda d: defs[d]['name'] if 0 <= d < len(defs) else '?'
metal = lambda d: defs[d]['metal'] if 0 <= d < len(defs) else 0
armed = lambda d: 0 <= d < len(defs) and defs[d]['reach'] > 0 and defs[d]['weapons'] > 0
ours = set()
for s in samples:
    for u in s['own']: ours.add(u[0])
deaths.sort(key=lambda e: e['f'])
clock = lambda f: f'{f // 1800}:{f // 30 % 60:02d}'

open_ = {}; done = []
def close(g):
    ep = open_.pop(g)
    done.append(ep)
for s in samples:
    f = s['f']; own = {u[0]: u for u in s['own']}
    en = [e for e in s.get('en', []) if e[1] >= 0 and armed(e[1]) and not e[5] and not (len(e) > 6 and e[6] and e[6] > 0)]
    for g, st in (s.get('groups') or {}).items():
        mem = list(st.get('formed', [])) + [x[0] for x in st.get('exceptions', [])]
        if len(mem) < a.least or st.get('engaged') is None:
            if g in open_ and f - open_[g]['last'] > 90: close(g)
            continue
        if g in open_ and f - open_[g]['last'] > 90: close(g)
        ep = open_.setdefault(g, {'g': g, 'f0': f, 'last': f, 'secs': 0, 'halt': 0, 'n': [], 'inreach': [], 'formed': [], 'anchor0': st['anchor'], 'anchor': st['anchor'], 'eng': collections.Counter(), 'members': set()})
        ep['last'] = f; ep['secs'] += 1; ep['anchor'] = st['anchor']
        if st.get('pace', 1) == 0 and not st.get('arrived'): ep['halt'] += 1
        ep['n'].append(len(mem)); ep['formed'].append(len(st.get('formed', [])) / len(mem)); ep['members'] |= set(mem)
        eng = next((e for e in s.get('en', []) if e[0] == st['engaged']), None)
        if eng: ep['eng'][name(eng[1])] += 1
        k = 0
        for m in mem:
            u = own.get(m)
            if not u: continue
            reach = defs[u[1]]['reach']
            if any(math.dist((u[2], u[3]), (e[2], e[3])) < reach - 20 for e in en): k += 1
        ep['inreach'].append(k / len(mem))
for g in list(open_): close(g)

for ep in done:
    f0, f1 = ep['f0'], ep['last'] + 30
    ax, az = ep['anchor0']
    lost = sum(metal(e['d']) for e in deaths if f0 <= e['f'] <= f1 and e['u'] in ep['members'])
    nlost = sum(1 for e in deaths if f0 <= e['f'] <= f1 and e['u'] in ep['members'])
    killed = sum(metal(e['d']) for e in deaths if f0 <= e['f'] <= f1 and e['k'] == 'enemy_destroyed' and (math.dist((e['x'], e['z']), ep['anchor']) < 900 or math.dist((e['x'], e['z']), ep['anchor0']) < 900))
    ep.update(lost=lost, nlost=nlost, killed=killed, travel=math.dist(ep['anchor0'], ep['anchor']))
tot = sum(e['secs'] for e in done); halt = sum(e['halt'] for e in done)
print(f"{a.dir}: {len(done)} contact episodes of groups of {a.least}+, {tot} s engaged, {halt} s of them halted; "
      f"lost {sum(e['lost'] for e in done):.0f} metal ({sum(e['nlost'] for e in done)} units), killed near {sum(e['killed'] for e in done):.0f}")
print('| group | at | s | halted s | members | in reach mean/max | formed | anchor moved | engaged | lost (units) | killed near |')
print('|---|---|---|---|---|---|---|---|---|---|---|')
for ep in sorted(done, key=lambda e: -e['secs'])[:a.top]:
    n = sum(ep['n']) / len(ep['n'])
    print(f"| {ep['g']} | {clock(ep['f0'])} | {ep['secs']} | {ep['halt']} | {n:.0f} | {sum(ep['inreach'])/len(ep['inreach']):.2f} / {max(ep['inreach']):.2f} | {sum(ep['formed'])/len(ep['formed']):.2f} | {ep['travel']:.0f} | "
          f"{', '.join(f'{k} {v}' for k, v in ep['eng'].most_common(3))} | {ep['lost']:.0f} ({ep['nlost']}) | {ep['killed']:.0f} |")
