#!/usr/bin/env python3
"""Every ground defence of one of our matches, tower by tower: where it stood from its seat's start, when it was finished, what it
killed by last hit, whether and to what it died; bands by build minute, by distance, by type. run/defence_towers.py <match dir with record-N.jsonl>"""
import json,re,subprocess,glob,collections,sys
REPO='.'
U=json.load(open('crates/bot/data/units.json'))['units']
GROUND={k for k,v in U.items() if v['class'] in ('defence turret','artillery turret','torpedo launcher','armed metal extractor','armed geothermal plant','long-range cannon')}
D=sys.argv[1]
def events(path):
    p=subprocess.run(['grep','-o',r'{[^{}]*"k": \?"\(created\|finished\|destroyed\|enemy_destroyed\)"[^{}]*}',path],capture_output=True,text=True)
    for l in p.stdout.splitlines():
        try: yield json.loads(l)
        except ValueError: pass
allrows=[]
for r in sorted(glob.glob(D+'/record-*.jsonl')):
    seat=re.search(r'record-(\d+)',r).group(1)
    h=json.loads(open(r).readline()); defs=h['unit_defs']
    name=lambda i: defs[i]['name'] if isinstance(i,int) and 0<=i<len(defs) else '?'
    metal=lambda i: (defs[i].get('metal') or 0) if isinstance(i,int) and 0<=i<len(defs) else 0
    start=None; tw={}
    for e in events(r):
        k=e['k']
        if k=='created' and start is None and 'com' in name(e['d']): start=(e['x'],e['z'])
        if k=='finished' and name(e['d']) in GROUND:
            tw[e['u']]=dict(seat=seat,type=name(e['d']),metal=metal(e['d']),f=e['f'],x=e['x'],z=e['z'],died=None,kills=0,killer=None)
        if k=='destroyed' and e['u'] in tw:
            tw[e['u']]['died']=e['f']; tw[e['u']]['killer']=name(e.get('by_d',-1)) if e.get('by_d',-1)>=0 else 'unseen'
        if k=='enemy_destroyed' and e.get('by_ours') and e.get('by') in tw:
            tw[e['by']]['kills']+=metal(e['d'])
    for t in tw.values():
        t['dist']=int(((t['x']-start[0])**2+(t['z']-start[1])**2)**.5) if start else -1
        t['dz']=t['z']-start[1] if start else 0
    allrows+=list(tw.values())
    print(f"seat {seat} start {start} towers {len(tw)} metal {sum(t['metal'] for t in tw.values())} kills {sum(t['kills'] for t in tw.values())} lost {sum(t['metal'] for t in tw.values() if t['died'])}")
def band(d): return '<1000' if d<1000 else '1000-2000' if d<2000 else '2000-3000' if d<3000 else '3000-4000' if d<4000 else '4000+'
B=collections.defaultdict(lambda: dict(n=0,metal=0,kills=0,lost=0,first=[]))
for t in allrows:
    b=B[band(t['dist'])]; b['n']+=1; b['metal']+=t['metal']; b['kills']+=t['kills']; b['lost']+=t['metal'] if t['died'] else 0; b['first'].append(t['f']//1800)
print("\nby distance from own start: n metal kills yield lost  median build minute")
import statistics
for k in ['<1000','1000-2000','2000-3000','3000-4000','4000+']:
    b=B[k]
    if b['n']: print(f"{k:10} {b['n']:4} {b['metal']:6} {b['kills']:6} {b['kills']/b['metal']:.2f} {b['lost']:6}  {statistics.median(b['first']):.0f}")
print("\nby type:")
T=collections.defaultdict(lambda: dict(n=0,metal=0,kills=0,lost=0))
for t in allrows:
    b=T[t['type']]; b['n']+=1; b['metal']+=t['metal']; b['kills']+=t['kills']; b['lost']+=t['metal'] if t['died'] else 0
for k,b in sorted(T.items(),key=lambda kv:-kv[1]['metal']): print(f"{k:12} {b['n']:4} {b['metal']:6} {b['kills']:6} {b['kills']/b['metal']:.2f} lost {b['lost']}")
print("\nby build minute (3-min bins): n metal kills yield lost")
M=collections.defaultdict(lambda: dict(n=0,metal=0,kills=0,lost=0))
for t in allrows:
    b=M[(t['f']//1800)//3*3]; b['n']+=1; b['metal']+=t['metal']; b['kills']+=t['kills']; b['lost']+=t['metal'] if t['died'] else 0
for k in sorted(M): b=M[k]; print(f"{k:3}-{k+3:<3} {b['n']:4} {b['metal']:6} {b['kills']:6} {b['kills']/b['metal']:.2f} lost {b['lost']}")
print("\nkillers of towers:", collections.Counter(t['killer'] for t in allrows if t['died']).most_common(8))
print("\ntop earners:")
for t in sorted(allrows,key=lambda t:-t['kills'])[:12]: print(f"  t{t['seat']} {t['type']} at {t['f']//1800}:{(t['f']%1800)//30:02d} dist {t['dist']} dz {t['dz']} kills {t['kills']} died {t['died'] and f'{t['died']//1800}:{(t['died']%1800)//30:02d}'}")
