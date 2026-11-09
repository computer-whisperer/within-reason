#!/usr/bin/env python3
"""The gantry's units (tier 3): what people build of them and what kills them, their weapons by reach, a duel batch's
matrix read with names, and one game of ours read for its gantry units. Written for
docs/studies/2026-11-08-tier-3.md; every number there is this script's output.

    run/tier3.py pool                 the public team games (builds run/data/tier3-team.jsonl on the first run, ~6 min)
    run/tier3.py weapons [unit ...]   each unit's weapons by reach: type, damage a second, against aircraft, energy a shot
    run/tier3.py matrix <duel dir>    a `duel --ours ... --theirs ...` batch: row against column, margin x 100, with names
    run/tier3.py game <match dir>     our seat's record of one game: our gantry units' lives and his as seen
"""
import collections, csv, glob, json, math, multiprocessing as mp, os, re, statistics as st, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ARCHIVE = os.environ.get("WR_ARCHIVE", "/ceph/christian/within-reason/run/matches-archive")
CACHE = os.path.join(REPO, "run", "data", "tier3-team.jsonl")
U = json.load(open(os.path.join(REPO, "crates/bot/data/units.json")))["units"]
T3 = {k for k, u in U.items() if u.get("speed") and u.get("tier") == 3 and "gantry" in (u.get("flags") or [])}
clock = lambda f: f"{f // 1800}:{(f // 30) % 60:02d}"
minutes = lambda m: f"{int(m)}:{int(m % 1 * 60):02d}"
named = lambda n: U.get(n, {}).get("name", n)
metal_of = lambda n: U.get(n, {}).get("metal", 0) or 0

def gantry_units_of(d):
    """One replay directory: per seat every gantry unit [name, finished, died, killer, x, z] by unit id, and every kill a
    gantry unit made, {killer id: [its type, {victim type: count}]}."""
    game={'dir':os.path.basename(d),'teams':{}, 'kills':{}}
    kills={}
    for rec in sorted(glob.glob(d+'/record-*.jsonl')):
        f=open(rec,errors='replace')
        try: hdr=json.loads(f.readline())
        except ValueError: continue
        defs=hdr.get('unit_defs') or []; name=[x.get('name') for x in defs]
        t3={i for i,n in enumerate(name) if n in T3}
        units={}; lost_all=0.0
        for line in f:
            head=line[:200]
            if '"k":"finished"' in head:
                r=json.loads(line)
                if r.get('d') in t3: units[r['u']]=[name[r['d']], r['f'], None, None, None, None]
            elif '"k":"destroyed"' in head:
                r=json.loads(line); di=r.get('d'); by=r.get('by_d')
                if r.get('c') in ('reclaimed','decayed','selfd'): continue
                if r.get('u') in units:
                    b=units[r['u']]; b[2]=r['f']; b[3]=name[by] if isinstance(by,int) and 0<=by<len(name) else (str(r.get('w') or r.get('c') or '')[:28] or None); b[4]=r.get('x'); b[5]=r.get('z')
                if isinstance(by,int) and by in t3 and di is not None and di<len(name):
                    k=kills.setdefault(str(r.get('by')),[name[by],{}]); k[1][name[di]]=k[1].get(name[di],0)+1
        game['teams'][hdr.get('team')]=list(units.items())
    game['kills']=kills
    return game

def build_cache():
    dirs = sorted(set(os.path.dirname(p) for p in glob.glob(ARCHIVE + "/*-replay-*/record-*.jsonl")
                      + glob.glob(os.path.join(REPO, "run/matches/*-replay-*/record-*.jsonl"))))
    dirs = [d for d in dirs if len(glob.glob(d + "/record-*.jsonl")) >= 15]
    with mp.Pool(10) as pool, open(CACHE, "w") as out:
        for game in pool.imap_unordered(gantry_units_of, dirs):
            out.write(json.dumps(game) + "\n")

def pool():
    if not os.path.exists(CACHE):
        build_cache()
    man = {}
    for line in open(os.path.join(REPO, 'run/data/replays/manifest.jsonl')):
        row = json.loads(line)
        man[row['id']] = row
    statics={json.loads(l)['dir']:json.loads(l) for l in open(os.path.join(REPO, 'run/data/statics-team.jsonl'))}
    GANTRY={'armshltx','corgant','leggant','armshltxuw','corgantuw','leggantuw'}
    built=collections.Counter(); seats_with=collections.defaultdict(set); firsts=collections.defaultdict(list); died=collections.Counter(); lived=collections.defaultdict(list)
    killers=collections.defaultdict(collections.Counter); victims=collections.defaultdict(collections.Counter); kill_metal=collections.Counter(); per_unit=collections.defaultdict(list)
    people_seats=0; long_seats=0; gantry_seats=0; gantry_first=[]; seats_t3=0; fac_seats=collections.Counter(); ai_built=collections.Counter()
    first_t3=[]; n_games=0; counts_per_seat=collections.defaultdict(list)
    for l in open(CACHE):
        g=json.loads(l); m=re.search(r'replay-([0-9a-f]{32})$', g['dir']); mf=man.get(m.group(1)) if m else None
        if not mf: continue
        n_games+=1
        dur=mf['duration_ms']/60000
        ppl={p['team']:p for p in mf['players'] if p.get('team') is not None}
        kills=g['kills']
        st_g=statics.get(g['dir'],{'teams':{}})
        for t,units in g['teams'].items():
            p=ppl.get(int(t))
            if not p:
                for u,b in units: ai_built[b[0]]+=1
                continue
            people_seats+=1
            if dur>=30:
                long_seats+=1; fac_seats[p.get('faction')]+=1
                gs=[b[1] for b in st_g['teams'].get(t,[]) if b[0] in GANTRY]
                if gs: gantry_seats+=1; gantry_first.append(min(gs)/1800)
                if units: seats_t3+=1; first_t3.append(min(b[1] for u,b in units)/1800)
            c=collections.Counter(b[0] for u,b in units)
            for n,k in c.items(): counts_per_seat[n].append(k)
            for u,b in units:
                n=b[0]; built[n]+=1; seats_with[n].add((g['dir'],t))
                if dur>=30: pass
                if b[2] is not None:
                    died[n]+=1; lived[n].append((b[2]-b[1])/1800); killers[n][b[3]]+=1
                k=kills.get(str(u))
                km=sum(metal_of(v)*c2 for v,c2 in (k[1].items() if k else []))
                per_unit[n].append(km); kill_metal[n]+=km
                if k:
                    for v,c2 in k[1].items(): victims[n][v]+=metal_of(v)*c2
            for n in c: firsts[n].append(min(b[1] for u,b in units if b[0]==n)/1800)
    print(f"games {n_games}; people's seats {people_seats}; in games of 30+ min {long_seats}: with a gantry {100*gantry_seats/long_seats:.0f} % (first at quartiles {[minutes(x) for x in st.quantiles(gantry_first,n=4)]}), with a gantry unit finished {100*seats_t3/long_seats:.0f} % (first unit at {[minutes(x) for x in st.quantiles(first_t3,n=4)]})")
    print('by faction of long-game seats', dict(fac_seats))
    print(f"{'unit':16s} {'built':>6} {'seats':>5} {'each':>5} {'first':>6} {'died':>5} {'lived':>6} {'kill/cost':>9} {'no kill':>7}  killers | victims")
    def cls(n):
        if n is None: return 'unnamed'
        return U.get(n,{}).get('name', n)
    for n,b in built.most_common():
        pu=per_unit[n]; m=metal_of(n)
        kl=", ".join(f"{cls(k)} {100*v/died[n]:.0f}%" for k,v in killers[n].most_common(5)) if died[n] else ''
        vt=victims[n]; tot=sum(vt.values()) or 1
        byc=collections.Counter()
        for v,x in vt.items(): byc[U.get(v,{}).get('class','?')]+=x
        vs=", ".join(f"{k} {100*x/tot:.0f}%" for k,x in byc.most_common(5))
        print(f"{U[n]['name']:16s} {b:6d} {len(seats_with[n]):5d} {st.median(counts_per_seat[n]):5.0f} {minutes(st.median(firsts[n])):>6} {100*died[n]/b:4.0f}% {st.median(lived[n]) if lived[n] else 0:5.1f}m {kill_metal[n]/(b*m):9.2f} {100*sum(1 for x in pu if x==0)/len(pu):6.0f}%  {kl} | {vs}")
    print('AI seats built:', {U[k]['name']:v for k,v in ai_built.most_common(12)})


def weapons(names):
    sys.path.insert(0, os.path.join(REPO, 'run'))
    import unit_stats as us
    defs, errors = us.load_defs(os.path.join(REPO, 'upstream/Beyond-All-Reason'))
    names = names or sorted(T3, key=metal_of)
    out={}
    for n in names:
        u=defs.get(n)
        if not u: print(n,'missing'); continue
        ws=[]
        for wd in us.used_weapons(u):
            g,a,bad=us.targets(wd)
            dps=us.weapon_dps(wd); dpa=us.weapon_dps(wd,'vtol')
            dmg=(wd.get('damage') or {})
            ws.append({'name':wd.get('name'),'type':wd.get('weapontype'),'range':float(wd.get('range')),'dps':round(dps) if dps else None,'dps_air':round(dpa) if (dpa and a) else None,'per':dmg.get('default'),'reload':us.reload_seconds(wd),'aoe':wd.get('areaofeffect'),'e':wd.get('energypershot'),'ground':g,'air':a,'air_bad':bad,'para':bool(wd.get('paralyzer')),'stock':bool(wd.get('stockpile')),'vel':wd.get('weaponvelocity'), 'cmds':dmg.get('commanders')})
        out[n]={'health':u.get('health'),'metal':u.get('metalcost'),'energy':u.get('energycost'),'bt':u.get('buildtime'),'speed':u.get('speed'),'sight':u.get('sightdistance'),'radar':u.get('radardistance'),'para_mult':(u.get('customparams') or {}).get('paralyzemultiplier'),'foot':u.get('footprintx'),'move':u.get('movementclass'),'explode':u.get('explodeas'),'selfd':u.get('selfdestructas'),'cancapture':u.get('capturable'),'transport':u.get('cantbetransported'),'weapons':ws}
        print(f"{n} hp {u.get('health')} m {u.get('metalcost')} spd {u.get('speed')} sight {u.get('sightdistance')} radar {u.get('radardistance')} para x{(u.get('customparams') or {}).get('paralyzemultiplier')} move {u.get('movementclass')} explode {u.get('explodeas')}")
        for w in sorted(ws, key=lambda w:-w['range']):
            print(f"    {str(w['name'])[:34]:34s} {w['type']:16s} range {w['range']:6.0f} dps {w['dps']!s:6} air {w['dps_air']!s:6} per {w['per']!s:6} reload {w['reload']} aoe {w['aoe']} e/shot {w['e']} {'ground' if w['ground'] else ''}{' air' if w['air'] else ''}{' (air last)' if w['air_bad'] else ''}{' STUN' if w['para'] else ''}{' stockpiled' if w['stock'] else ''}")


def matrix(D):
    rows=list(csv.DictReader(open(D+'/duels.csv')))
    M=collections.defaultdict(list); info={}
    for r in rows:
        x,y=r['x'],r['y']
        try: mx=float(r['value_left_x'])-float(r['value_left_y'])
        except ValueError: continue
        M[(x,y)].append(mx); M[(y,x)].append(-mx)
        info[(x,y)]=(r['n_x'],r['n_y'],r['winner'],r['reason'],r['seconds'],r['muzzled_line_x'],r['muzzled_line_y'],r['metal_x'],r['metal_y'])
        info[(y,x)]=(r['n_y'],r['n_x'],{'x':'y','y':'x'}.get(r['winner'],r['winner']),r['reason'],r['seconds'],r['muzzled_line_y'],r['muzzled_line_x'],r['metal_y'],r['metal_x'])
    units=[]
    for r in rows:
        for k in ('x','y'):
            if r[k] not in units: units.append(r[k])
    order=sorted(units, key=lambda n:(n not in set(r['x'] for r in rows), U.get(n,{}).get('metal',0)))
    t3=[n for n in order if n in set(r['x'] for r in rows)]
    def cell(a,b):
        v=M.get((a,b))
        return f"{round(100*sum(v)/len(v)):+4d}" if v else '   .'
    print('margin x100 at about 30,000 metal a side: row against column, + the row won; counts in the row label')
    short=lambda n: named(n)[:5]
    print(' '*22+' '.join(f"{short(c):>5}" for c in order))
    for a in t3:
        cnt=next((info[(a,b)][0] for b in order if (a,b) in info), '?')
        print(f"{named(a):14s} x{cnt:>3}  "+' '.join(f"{cell(a,b):>5}" for b in order))
    # summary: average margin vs T3 field, vs T2 field
    print()
    for a in t3:
        v3=[sum(M[(a,b)])/len(M[(a,b)]) for b in t3 if b!=a and (a,b) in M]
        v2=[sum(M[(a,b)])/len(M[(a,b)]) for b in order if b not in t3 and (a,b) in M]
        wins=[named(b) for b in order if b!=a and (a,b) in M and sum(M[(a,b)])/len(M[(a,b)])>0.15]
        loses=[named(b) for b in order if b!=a and (a,b) in M and sum(M[(a,b)])/len(M[(a,b)])<-0.15]
        print(f"{named(a):14s} mean against gantry units {100*sum(v3)/len(v3):+4.0f}, against the tier-2 list {100*sum(v2)/max(len(v2),1):+4.0f} | beats {len(wins)}: {', '.join(wins)} | loses to {len(loses)}: {', '.join(loses)}")
    to=[(k,info[k]) for k in info if info[k][3]!='wiped']
    print('not decided by a wipe:', len(to)//2, sorted(set((named(a),named(b),i[3]) for (a,b),i in to))[:30])


def game(D):
    rows=[]
    for l in open(D+'/00/record-0.jsonl',errors='replace'):
        try: rows.append(json.loads(l))
        except ValueError: break
    hdr=rows[0]; defs=hdr['unit_defs']; name=[d['name'] for d in defs]; metal=[d.get('metal') or 0 for d in defs]; speed=[d.get('speed') or 0 for d in defs]; reach=[d.get('reach') or 0 for d in defs]; cls=[d.get('class') for d in defs]
    nm=lambda i: named(name[i])
    s=[r for r in rows if r.get('t')=='s']; ev=[r for r in rows if r.get('t')=='ev']
    home=(s[0]['own'][0][2], s[0]['own'][0][3])
    print('map', hdr['map']['name'], 'side', hdr.get('side'), 'home', home, 'to', clock(s[-1]['f']))
    # economy and army marks
    for m in (10,15,20,25,30,35,39):
        x=min(s,key=lambda r:abs(r['f']-m*1800))
        army=sum(metal[u[1]] for u in x['own'] if speed[u[1]]>0 and reach[u[1]]>0 and cls[u[1]] not in ('commander',) and not u[5]&1)
        his=sum(metal[e[1]] for e in x.get('en',[]) if e[1]>=0 and speed[e[1]]>0 and reach[e[1]]>0)
        t3o=collections.Counter(nm(u[1]) for u in x['own'] if name[u[1]] in T3 and not u[5]&1); t3h=collections.Counter(nm(e[1]) for e in x.get('en',[]) if e[1]>=0 and name[e[1]] in T3)
        print(f"  {m}:00 income {x['m'][1]:.0f} m/s {x['e'][1]:.0f} e/s store {x['m'][0]:.0f}/{x['e'][0]:.0f}; our army {army:,}; his army in sight or radar {his:,}; our gantry units {dict(t3o)}; his seen {dict(t3h)}")
    fin={e['u']:e for e in ev if e['k']=='finished'}; dead={e['u']:e for e in ev if e['k']=='destroyed'}
    print('gantry:', [(name[e['d']],clock(e['f'])) for e in ev if e['k']=='finished' and ('shltx' in name[e['d']] or 'gant' in name[e['d']])], 'created', [(name[e['d']],clock(e['f'])) for e in ev if e['k']=='created' and ('shltx' in name[e['d']] or 'gant' in name[e['d']])])
    kills=collections.defaultdict(collections.Counter)
    for e in ev:
        if e['k']=='enemy_destroyed' and e.get('by') is not None and e.get('d') is not None and e['d']>=0: kills[e['by']][e['d']]+=1
    mine=[u for u,e in fin.items() if name[e['d']] in T3]
    print(f"our gantry units: {len(mine)}")
    bys=collections.Counter(); 
    for u in sorted(mine,key=lambda u: fin[u]['f']):
        e=fin[u]; d=dead.get(u); f1=d['f'] if d else s[-1]['f']
        x=min(s,key=lambda r:abs(r['f']-(f1-60)))
        me=next((w for w in x['own'] if w[0]==u),None)
        comp=foes=dist=None; near_h=collections.Counter()
        if me:
            comp=sum(metal[w[1]] for w in x['own'] if w[0]!=u and speed[w[1]]>0 and reach[w[1]]>0 and math.hypot(w[2]-me[2],w[3]-me[3])<=700)
            foes=sum(metal[w[1]] for w in x.get('en',[]) if w[1]>=0 and math.hypot(w[2]-me[2],w[3]-me[3])<=1000 and reach[w[1]]>0)
            near_h=collections.Counter(nm(w[1]) for w in x.get('en',[]) if w[1]>=0 and math.hypot(w[2]-me[2],w[3]-me[3])<=1000 and reach[w[1]]>0)
            dist=math.hypot(me[2]-home[0],me[3]-home[1])
        km=sum(metal[v]*c for v,c in kills[u].items())
        how=f"died {clock(d['f'])} to {nm(d['by_d']) if d.get('by_d') is not None and d['by_d']>=0 else (d.get('w') or d.get('c'))}" if d else 'alive at the end'
        if d: bys[nm(d['by_d']) if d.get('by_d') is not None and d['by_d']>=0 else str(d.get('w') or d.get('c'))]+=1
        print(f"  {nm(e['d']):10s} up {clock(e['f'])} {how:40s} lived {(f1-e['f'])/1800:4.1f} min, killed {km:>6,} ({dict((nm(v),c) for v,c in kills[u].most_common(4))}); then {dist and round(dist)} from home, our army within 700: {comp}, his armed within 1000: {foes} {dict(near_h.most_common(5))}")
    print('our gantry units died to:', dict(bys))
    # his gantry units as seen: by sample, count and spread
    print('his gantry units in sight or radar, each minute from the first:')
    first=None
    for x in s:
        h=[e for e in x.get('en',[]) if e[1]>=0 and name[e[1]] in T3]
        if h and first is None: first=x['f']
        if first is not None and (x['f']-first)%1800<30:
            c=collections.Counter(nm(e[1]) for e in h)
            if h:
                cx=sum(e[2] for e in h)/len(h); cz=sum(e[3] for e in h)/len(h)
                spread=st.median([math.hypot(e[2]-cx,e[3]-cz) for e in h])
                esc=sum(metal[e[1]] for e in x['en'] if e[1]>=0 and name[e[1]] not in T3 and reach[e[1]]>0 and speed[e[1]]>0 and math.hypot(e[2]-cx,e[3]-cz)<=900)
                esc_t=collections.Counter(nm(e[1]) for e in x['en'] if e[1]>=0 and name[e[1]] not in T3 and reach[e[1]]>0 and speed[e[1]]>0 and math.hypot(e[2]-cx,e[3]-cz)<=900)
                print(f"  {clock(x['f'])} {dict(c)} centre ({cx:.0f},{cz:.0f}) {math.hypot(cx-home[0],cz-home[1]):.0f} from our home, median spread {spread:.0f}; other army of his within 900: {esc:,} {dict(esc_t.most_common(4))}")
    # damage by type pair involving gantry units
    xo=collections.Counter(); xi=collections.Counter()
    for x in s:
        for a,b,d in x.get('xo',[]):
            if a>=0 and b>=0 and (name[a] in T3 or name[b] in T3): xo[(nm(a),nm(b))]+=d
        for a,b,d in x.get('xi',[]):
            if b>=0 and (a<0 or name[a] in T3 or name[b] in T3): xi[(nm(a) if a>=0 else 'unseen',nm(b))]+=d
    print('our hits involving gantry units (ours -> his):', [(k,round(v)) for k,v in xo.most_common(14)])
    print('his hits (his -> ours):', [(k,round(v)) for k,v in xi.most_common(16)])
    ed=collections.Counter(); 
    for e in ev:
        if e['k']=='enemy_destroyed' and e.get('d') is not None and e['d']>=0 and name[e['d']] in T3: ed[(nm(e['d']), nm(e['by_d']) if e.get('by_d') is not None and e['by_d']>=0 else 'not ours')]+=1
    print('his gantry units seen destroyed:', dict(ed))


if __name__ == "__main__":
    mode = sys.argv[1] if len(sys.argv) > 1 else ""
    if mode == "pool":
        pool()
    elif mode == "weapons":
        weapons(sys.argv[2:])
    elif mode == "matrix" and len(sys.argv) > 2:
        matrix(sys.argv[2])
    elif mode == "game" and len(sys.argv) > 2:
        game(sys.argv[2])
    else:
        sys.exit(__doc__)
