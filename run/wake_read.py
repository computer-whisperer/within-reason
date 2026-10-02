#!/usr/bin/env python3
"""How the player was scheduled in a pianist match: turns per minute, what woke each turn, the time its orders were on
their way (the think penalty), the idle after they landed, and how long a loss of ours, an extractor lost or an
attack the hands began waited for the next turn (K-player-idle-after-the-flight-costs-more-than-the-flight).

usage: run/wake_read.py run/matches/<batch>/00 [...]
"""
import json,sys,re,statistics as S,collections
FPS=30
def pct(xs,p): xs=sorted(xs); return xs[min(len(xs)-1,int(len(xs)*p))] if xs else 0
def read(d):
    st=[json.loads(l) for l in open(d+'/strategist-0.jsonl')]
    turns=[]
    cur=None
    for e in st:
        if e.get('kind')=='turn':
            m=re.search(r'Woken because: (.*)',e['prompt'])
            reasons=m.group(1).split('; ') if m else ['?']
            w=re.search(r'wake conditions in force: (\{.*\})\s*$',e['prompt'],re.M)
            cur={'f':e['frame'],'reasons':reasons,'wake':json.loads(w.group(1)) if w else None}
        elif e.get('kind')=='turn_end' and cur:
            cur['wall']=e['wall_seconds']; cur['land']=cur['f']+int(e['wall_seconds']*FPS); cur['ended']=e['ended_by']; turns.append(cur); cur=None
    jev=[json.loads(l) for l in open(d+'/jev-0.jsonl')]
    plans=[c for c in jev if c.get('t')=='plan']
    rec=[json.loads(l) for l in open(d+'/record-0.jsonl')]
    ev=[e for e in rec if e.get('t')=='ev']
    return turns,plans,ev
def classify(r):
    for key,name in [('capped at','timer-capped'),('s have passed','timer'),('has lost','loss-wake'),('has met','odds-wake'),('on their way','flight-review'),('your hands say','jev-needs'),('your hands sent','engaged'),('enemies within','near-mex'),('extractor was destroyed','mex-lost'),('said something','chat'),('unassigned soldiers','pool'),('no growth','stagnant'),('game begins','opening')]:
        if key in r: return name
    return 'other:'+r[:40]
for d in sys.argv[1:]:
    turns,plans,ev=read(d)
    end=turns[-1]['land']
    print(f"\n== {d}: {len(turns)} turns over {end/FPS/60:.1f} min = {len(turns)/(end/FPS/60):.2f}/min; wall median {S.median(t['wall'] for t in turns):.1f}s")
    # reasons
    kinds=collections.Counter(); nre=[]
    for t in turns:
        ks=[classify(r) for r in t['reasons']]; nre.append(len(ks))
        for k in set(ks): kinds[k]+=1
    print("  turns by reason kind (a turn may have several):",dict(kinds.most_common()))
    print(f"  reasons per turn: mean {S.mean(nre):.2f}, turns with >1 reason {sum(n>1 for n in nre)}")
    # wake settings chosen
    ms=collections.Counter(t['wake']['max_seconds'] for t in turns if t['wake'])
    print("  max_seconds in force at turn start:",dict(ms))
    # gaps
    gaps=[(turns[i+1]['f']-turns[i]['f'])/FPS for i in range(len(turns)-1)]
    idle=[(turns[i+1]['f']-turns[i]['land'])/FPS for i in range(len(turns)-1)]
    print(f"  turn-to-turn game gap s: median {S.median(gaps):.1f} p10 {pct(gaps,.1):.1f} p90 {pct(gaps,.9):.1f} max {max(gaps):.0f}")
    print(f"  landing-to-next-turn (idle after orders land) s: median {S.median(idle):.1f} p10 {pct(idle,.1):.1f} p90 {pct(idle,.9):.1f}; <=1 s: {sum(i<=1 for i in idle)} of {len(idle)} (woken by a pending reason at once)")
    busy_frac=sum(t['land']-t['f'] for t in turns)/end
    print(f"  fraction of game time the player was busy (orders in flight): {busy_frac:.0%}")
    # by reason kind: idle before this turn
    by=collections.defaultdict(list)
    for i in range(1,len(turns)):
        for k in set(classify(r) for r in turns[i]['reasons']): by[k].append((turns[i]['f']-turns[i-1]['land'])/FPS)
    print("  idle before a turn, by reason kind (median s, n):",{k:(round(S.median(v),1),len(v)) for k,v in by.items()})
    # event latency: our losses, extractor losses, engagements begun
    defs={}
    hdr=[json.loads(l) for l in open(d+'/record-0.jsonl')][0]
    for i,u in enumerate(hdr['unit_defs']): defs[i]=u
    losses=[e for e in ev if e['k']=='destroyed']
    mex=[e for e in losses if defs.get(e['d'],{}).get('extracts_metal',0)>0]
    def latency(events,label):
        lat=[];busy=0;quiet=0;lat_b=[];lat_q=[]
        for e in events:
            f=e['f']
            inflight=any(t['f']<=f<t['land'] for t in turns)
            nxt=[t['f'] for t in turns if t['f']>=f]
            if not nxt: continue
            l=(nxt[0]-f)/FPS; lat.append(l)
            (lat_b if inflight else lat_q).append(l)
        if not lat: print(f"  {label}: none"); return
        print(f"  {label}: {len(lat)}; next turn started after median {S.median(lat):.1f}s p90 {pct(lat,.9):.1f}s; during in-flight orders {len(lat_b)} (median {S.median(lat_b) if lat_b else 0:.1f}s), while idle {len(lat_q)} (median {S.median(lat_q) if lat_q else 0:.1f}s)")
    latency(losses,"our units destroyed")
    latency(mex,"our extractors destroyed")
    # first loss of a burst: losses with no loss in the previous 10 s
    bursts=[e for i,e in enumerate(losses) if i==0 or e['f']-losses[i-1]['f']>10*FPS]
    latency(bursts,"first loss of a burst (10 s quiet before)")
    eng=[{'f':f} for f in sorted({c['f'] for c in plans for p in c.get('played') or [] if re.search(r'^attack party_|hunt party_',p.get('did') or '')})]
    latency(eng,"hands began an attack or sent a detachment (seconds with one)")
