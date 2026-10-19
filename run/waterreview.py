"""waterreview.py <match/00>: the water spots of a game and what each side did with them (one record per seat)."""
import json, sys, struct, collections, os, re, glob
M=sys.argv[1].rstrip('/')
recs=sorted(glob.glob(f'{M}/record-*.jsonl'))
h=json.loads(open(recs[0]).readline()); t=h['terrain']; W,Hh=h['map']['width'],h['map']['height']
cell=t['cell']; cols=int(W/cell); rows=int(Hh/cell); data=open(f"{M}/{t['file']}",'rb').read(); n=cols*rows
heights=struct.unpack('<%dh'%n, data[:2*n])
def hgt(x,z): return heights[min(rows-1,int(z/cell))*cols+min(cols-1,int(x/cell))]
def cellname(x,z): return chr(ord('A')+min(7,int(x*8//W)))+str(min(7,int(z*8//Hh))+1)
def clk(f): return f'{f//1800}:{(f%1800)//30:02d}'
defs=h['unit_defs']; names=[d['name'] for d in defs]; isx={i: d.get('extracts_metal',0)>0 for i,d in enumerate(defs)}
def nm(i): return names[i] if isinstance(i,int) and 0<=i<len(names) else str(i)
spots=h['metal_spots']; water=[i for i,(x,z,a) in enumerate(spots) if hgt(x,z)<0]
print(f"map {h['map']['name']} {W:.0f}x{Hh:.0f}; side {h.get('side')} team {h.get('team')}; spots {len(spots)}, {len(water)} in water: " + ', '.join(f'spot_{i} {cellname(*spots[i][:2])} ({hgt(*spots[i][:2])})' for i in water))
for rec in recs:
    for l in open(rec):
        if '"t":"intent"' in l:
            r=json.loads(l); print(f"{os.path.basename(rec)} home {r.get('home')} {cellname(*r['home']) if r.get('home') else ''} enemy_start {r.get('enemy_start')} {cellname(*r['enemy_start']) if r.get('enemy_start') else ''}"); break
def nearest(x,z):
    best=None
    for i,(sx,sz,a) in enumerate(spots):
        d=((sx-x)**2+(sz-z)**2)**0.5
        if d<120 and (best is None or d<best[1]): best=(i,d)
    return best[0] if best else None
ours={}; theirs={}
for rec in recs:
    s=os.path.basename(rec); land=wat=0; fin=collections.Counter(); destroyed_water=[]; nb_water=0; nos=0; dw=[]
    for l in open(rec):
        if '"t":"ev"' in l:
            r=json.loads(l)
            if r.get('k')=='finished':
                fin[nm(r['d'])]+=1
                if isx.get(r.get('d')):
                    i=nearest(r['x'],r['z'])
                    if i in water: wat+=1; ours.setdefault(i,(s,r['f']))
                    else: land+=1
            if r.get('k')=='destroyed' and isx.get(r.get('d')) and hgt(r.get('x',0),r.get('z',0))<0: destroyed_water.append((clk(r['f']), nm(r.get('by_d')) if r.get('by_d') is not None else '?'))
            if r.get('k') in ('no_site','rejected') and hgt(r.get('x',0),r.get('z',0))<0: nos+=1
        elif '"t":"cmd"' in l:
            r=json.loads(l)
            for c in r.get('c',[]):
                if c[0]=='build' and len(c)>=5 and hgt(c[3],c[4])<0: nb_water+=1
        elif '"t":"s"' in l:
            r=json.loads(l)
            for e in r.get('en',[]):
                if isx.get(e[1]):
                    i=nearest(e[2],e[3])
                    if i in water and i not in theirs: theirs[i]=r['f']
        elif '"t":"d"' in l and re.search(r'spot_(\d+)\b', l):
            r=json.loads(l); txt=json.dumps(r.get('outputs',''))
            if any(int(m) in water for m in re.findall(r'spot_(\d+)\b', txt)) and len(dw)<10: dw.append(f"{clk(r['f'])} {r.get('kind')}/{r.get('source')}: {str(r.get('outputs'))[:200]}")
    print(f"\n== {s}: extractors finished on land {land}, in water {wat}; build commands at water positions {nb_water}; no_site/rejected in water {nos}; water extractors destroyed {destroyed_water}")
    keys=[k for k in fin if any(w in k for w in ('mex','lab','vp','hp','sy','amsub','ap','ca','ck','cv','ch','muskrat','beaver','cs','alab','avp','nanotc','plat','tide','uw'))]
    print('   finished:', {k:fin[k] for k in sorted(keys)})
    print('   decisions naming a water spot (first 10):'); [print('     ', d) for d in dw]
    jev=rec.replace('record-','jev-')
    if os.path.exists(jev):
        cnt=collections.Counter()
        with open(jev) as f:
            for l in f:
                for m in re.findall(r'_spot_(\d+)\b', l):
                    if int(m) in water: cnt[int(m)]+=1
        print('   hands questions naming water spots:', dict(cnt.most_common(12)))
print('\nwater spots: who held them')
for i in water:
    o=ours.get(i); th=theirs.get(i)
    print(f"  spot_{i} {cellname(*spots[i][:2])}: ours={'%s at %s'%(o[0],clk(o[1])) if o else '-'}; theirs seen={clk(th) if th else '-'}")
