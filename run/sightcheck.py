"""sightcheck.py <match-dir>: the gone rule's firings against the truth, and the memory's recall in cells seen lately.
A: every "his X at C is gone" event per seat, matched to a building of that type in that cell that the truth shows
   dead by then (one truth death consumed per event); unmatched = misfire (something of that type still stands there
   or never stood there). B: for every report line "his, in C (<in sight now | seen 0:NN ago>): n type, ..." with the
   cell in sight or seen under a minute ago, reported buildings against the truth's buildings in that cell at the
   report's frame: recall = sum min(rep, truth) / sum truth; phantoms = sum max(rep - truth, 0) / sum rep. That recall
   counts buildings standing in tiles no unit of ours looked at (a cell is "all of it in sight now" when every tile was
   seen at some time and the median tile within ten seconds), so C: the same lines against only the buildings some
   seat's snapshot once held with a known type (the record's `s` rows, pooled as the team board pools them): what the
   memory holds of what the side has seen.
   Flags: --quiet (no per-event lines), --finished (truth buildings under construction left out of B)."""
import json, re, sys, glob, os
M = [a for a in sys.argv[1:] if not a.startswith('--')][0].rstrip('/')
QUIET = '--quiet' in sys.argv
FINISHED = '--finished' in sys.argv
MORE = [0]
if not os.path.exists(M + '/00'): sys.exit('no 00 in ' + M)
M += '/00'
hdr = json.loads(open(glob.glob(M + '/record-0.jsonl')[0]).readline())
W, H = hdr['map']['width'], hdr['map']['height']
defs = hdr['unit_defs']
is_building = {d['name']: d.get('class') == 'building' for d in defs}
def cell(x, z): return chr(ord('A') + min(7, int(x * 8 // W))) + str(min(7, int(z * 8 // H)) + 1)
seats = sorted(int(re.search(r'record-(\d+)', p).group(1)) for p in glob.glob(M + '/record-*.jsonl'))
ND = len(defs)
first_seen = {}  # enemy id -> first frame any seat's snapshot held it with a known type
for s_ in seats:
    for l in open(f'{M}/record-{s_}.jsonl'):
        if '"t":"s"' not in l: continue
        r = json.loads(l)
        for e in r.get('en', []):
            if 0 <= e[1] < ND: first_seen[e[0]] = min(first_seen.get(e[0], 10**9), r['f'])
SEEN = dict(expected=0, held=0, never=0)
COV = {}
tot = dict(events=0, matched=0, misfire=0, lag=[]); rec_tot = dict(rep=0, truth=0, hit=0, phantom=0, lines=0)
for s in seats:
    truth = {}
    first, last, where, kind = {}, {}, {}, {}
    for l in open(f'{M}/truth-{s}.jsonl'):
        r = json.loads(l); f = r['f']; ids = set()
        for e in r['enemy']:
            uid, name, x, z = e[0], e[1], e[2], e[3]
            ids.add(uid); first.setdefault(uid, f); last[uid] = f; where[uid] = (x, z); kind[uid] = name
        truth[f] = r['enemy']
    frames = sorted(truth)
    def truth_at(f):
        c = [g for g in frames if g <= f]
        return truth[c[-1]] if c else []
    end_frame = frames[-1]
    # A: gone events
    consumed = set(); n_ev = n_ok = n_mis = 0; lags = []
    for l in open(f'{M}/record-{s}.jsonl'):
        if '"is gone' not in l and 'is gone:' not in l: continue
        r = json.loads(l)
        if r.get('kind') != 'event': continue
        m = re.match(r'his (\S+) at ([A-H][1-8]) is gone', r['outputs'])
        if not m: print('  unparsed:', r['outputs'][:120]); continue
        typ, c = m.group(1), m.group(2)
        ev = re.search(r'\(ours (\d+) away, the grid (-?\d+) frames old\)', r['outputs'])
        dist, gridage = (int(ev.group(1)), int(ev.group(2))) if ev else ('?', '?')
        f = r['f']; n_ev += 1
        cands = [uid for uid in last if kind[uid] == typ and cell(*where[uid]) == c and last[uid] < f and uid not in consumed and last[uid] < end_frame]
        standing = [uid for uid in last if kind[uid] == typ and cell(*where[uid]) == c and first[uid] <= f and (last[uid] >= f)]
        if cands:
            uid = max(cands, key=lambda u: last[u]); consumed.add(uid); n_ok += 1; lag = (f - last[uid]) / 30
            lags.append(lag)
            if not QUIET: print(f'  seat {s} {f//1800}:{(f%1800)//30:02d} {typ} {c}: OK, died {lag:.0f} s before (ours {dist} away, grid {gridage} old; {len(standing)} of that type still in the cell)')
        else:
            n_mis += 1
            print(f'  seat {s} {f//1800}:{(f%1800)//30:02d} {typ} {c}: MISFIRE, {len(standing)} of that type standing there (ours {dist} away, grid {gridage} old)')
    print(f'seat {s}: {n_ev} gone events, {n_ok} matched a truth death, {n_mis} misfires')
    tot['events'] += n_ev; tot['matched'] += n_ok; tot['misfire'] += n_mis; tot['lag'] += lags
    # B: recall
    rep = tru = hit = ph = nl = 0
    for l in open(f'{M}/strategist-{s}.jsonl'):
        r = json.loads(l)
        if r.get('kind') != 'turn': continue
        f = r['frame']; tr = truth_at(f)
        for m in re.finditer(r'his, in ([A-H][1-8]) \(([^)]*)\): ([^\n"]*)', r['prompt']):
            c, how, items = m.groups()
            ago = re.search(r'seen (\d+):(\d+) ago', how)
            if 'in sight now' not in how and not (ago and int(ago.group(1)) == 0): continue
            counts = {}
            for it in items.split(','):
                mm = re.match(r'\s*(\d+) (\S+)', it)
                if mm and is_building.get(mm.group(2), False): counts[mm.group(2)] = counts.get(mm.group(2), 0) + int(mm.group(1))
            tcount = {}; once = {}
            for e in tr:
                if FINISHED and len(e) > 5 and e[5]: continue
                if is_building.get(e[1], False) and cell(e[2], e[3]) == c:
                    tcount[e[1]] = tcount.get(e[1], 0) + 1
                    if first_seen.get(e[0], 10**9) <= f: once[e[1]] = once.get(e[1], 0) + 1
                    else: SEEN['never'] += 1
            for t in set(once) | set(counts):
                SEEN['expected'] += once.get(t, 0); SEEN['held'] += min(once.get(t, 0), counts.get(t, 0))
            if ' more' in items: MORE[0] += 1
            if not counts and not tcount: continue
            nl += 1
            cls = 'all' if 'all of it' in how or how.strip()=='in sight now' else 'most' if 'most of it' in how else 'part' if 'in sight now' in how else 'seen<1m'
            cov = COV.setdefault(cls, [0, 0])
            for t in set(counts) | set(tcount):
                a, b = counts.get(t, 0), tcount.get(t, 0)
                rep += a; tru += b; hit += min(a, b); ph += max(a - b, 0); cov[0] += min(a, b); cov[1] += b
    print(f'seat {s}: recall {hit}/{tru} = {100*hit/max(tru,1):.0f}% over {nl} lines, phantoms {ph}/{rep} = {100*ph/max(rep,1):.0f}%')
    for k, v in zip(('rep', 'truth', 'hit', 'phantom', 'lines'), (rep, tru, hit, ph, nl)): rec_tot[k] += v
print(f"ALL: {tot['events']} gone events, {tot['matched']} matched, {tot['misfire']} misfires; lag median {sorted(tot['lag'])[len(tot['lag'])//2] if tot['lag'] else 0:.0f} s")
print(f"ALL: recall {rec_tot['hit']}/{rec_tot['truth']} = {100*rec_tot['hit']/max(rec_tot['truth'],1):.0f}% over {rec_tot['lines']} lines; phantoms {rec_tot['phantom']}/{rec_tot['rep']} = {100*rec_tot['phantom']/max(rec_tot['rep'],1):.0f}%")

print('ALL recall by how much of the cell is in sight: ' + ', '.join(f"{k} {v[0]}/{v[1]} = {100*v[0]/max(v[1],1):.0f}%" for k, v in sorted(COV.items())))
QUIET = '--quiet' in sys.argv
print('lines with "more" in them:', MORE[0])
print(f"ALL: of the truth's buildings in those cells {SEEN['never']} had never been in any seat's snapshot; of the {SEEN['expected']} once seen the memory held {SEEN['held']} = {100*SEEN['held']/max(SEEN['expected'],1):.0f}%")
