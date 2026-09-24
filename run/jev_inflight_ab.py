#!/usr/bin/env python3
"""Offline A/B of the hands' fall-back precedence (K-hands-precedence-wording): every moment a group lost a member
while the player's orders were on their way is put to Jev again as recorded (`base`), with the rules on disk, the
`do` question's precedence sentence, the player line, the odds on the group's `enemies_near` line and a 30 s `losses`
line synthesised from the record (`prompt`), and with the state cuts of 2026-09-23 approximated (`prompt+cuts`).
Written for logs from before those lines existed (wake-1); on later logs the synthesis overwrites what is there.
usage: run/jev_inflight_ab.py run/matches/<batch>/00 [repeat]      (the key is read from the env file, never printed)
"""
import json,sys,re,os,copy,collections,statistics as S
sys.path.insert(0,'/home/christian/workspace/playground/bar_bots/run')
from jev_ab import ask, URL
FPS=30
d=sys.argv[1]; REPEAT=int(sys.argv[2]) if len(sys.argv)>2 else 3
key=None
for line in open(os.path.expanduser('~/.config/within-reason/jev.env')):
    if line.startswith('TYPESAFE_API_KEY='): key=line.split('=',1)[1].strip().strip('"')
assert key
RULES=open('/home/christian/workspace/playground/bar_bots/crates/bot/src/brain/pianist/rules.md').read().strip()
def clock(f): return f"{f//FPS//60}:{f//FPS%60:02d}"
st=[json.loads(l) for l in open(d+'/strategist-0.jsonl')]
turns=[];cur=None
for e in st:
    if e.get('kind')=='turn': cur={'f':e['frame']}
    elif e.get('kind')=='turn_end' and cur: cur['land']=cur['f']+int(e['wall_seconds']*FPS); turns.append(cur); cur=None
raw=[json.loads(l) for l in open(d+'/jev-0.jsonl')]; hdr=raw[0]; calls=[c for c in raw if c.get('t')=='call']
rec=[json.loads(l) for l in open(d+'/record-0.jsonl')]; defs=rec[0]['unit_defs']; losses=[e for e in rec if e.get('t')=='ev' and e['k']=='destroyed']
# rules/instructions in force per call
instr=None; rules=hdr.get('rules','')
for c in calls:
    if c.get('instructions'): instr=c['instructions']
    if c.get('rules'): rules=c['rules']
    c['_instr']=instr; c['_rules']=rules
def group_at(u,f):
    prev=[c for c in calls if c['f']<=f]
    for g in (prev[-1]['groups'] if prev else []):
        if u in g['members']: return 'group_'+g['name']
moments=[]
for t in turns:
    seen=set()
    for e in losses:
        if t['f']<=e['f']<t['land']:
            g=group_at(e['u'],e['f'])
            if not g or g in seen: continue
            seen.add(g)
            for c in calls:
                if c['f']>e['f'] and g+'.do' in c.get('questions',{}):
                    moments.append((c,g,e['f'])); break
print(len(moments),'moments')
def new_state(c,g,e_f):
    s=copy.deepcopy(c['state']); s['instructions']=c['_instr']
    wind=c['_rules'].split('\n')[-1] if '\n' in c['_rules'] else ''
    s['rules']=RULES+('\n'+wind if wind.startswith('This map') else '')
    s['player']=re.sub(r"; and nothing new comes from it before then; the instructions are its last word|, and nothing new comes from it before then; the instructions are its last word", "; it has not seen what has happened since, so what the picture shows now outranks an instruction it contradicts", s.get('player',''))
    a=s['actors'][g]
    # odds from the recorded engage option
    eng=(c['questions'][g+'.do'].get('criteria') or {}).get('engage','')
    en=a.get('enemies_near','')
    m=re.match(r'(party_\d+) \(([^)]*)\) (\d+) away',en)
    if m:
        pname=m.group(1); odds=''
        for seg in eng.split('In sight: ')[-1].split('; party_'):
            seg='party_'+seg if not seg.startswith('party_') else seg
            if seg.startswith(pname+':'):
                odds=seg.rstrip('.').split(': ')[-1]
                kill=re.search(r'killing ([^:]*?) now',seg)
                a['enemies_near']=f"{pname} ({m.group(2)}) {m.group(3)} away: {odds}"+(f"; killing {kill.group(1)} now" if kill else '')
    # losses line: last 30 s
    members=set()
    for cc in calls:
        if cc['f']<=c['f']:
            for gg in cc['groups']:
                if 'group_'+gg['name']==g: members|=set(gg['members'])
    lost=[e for e in losses if c['f']-900<=e['f']<=c['f'] and e['u'] in members]
    if lost:
        lm=sum(defs[e['d']]['metal'] for e in lost)
        um=re.search(r'worth (\d+) metal',a.get('units','')); standing=float(um.group(1)) if um else 0.0
        n=re.search(r'\((\d+) soldiers',a.get('units','')); nn=int(n.group(1)) if n else 0
        share=lm/max(lm+standing,1.0)
        words='a few' if share<0.1 else 'a noticeable share' if share<0.25 else 'a large share: it is losing this fight' if share<0.5 else 'most of it: it is being wiped out'
        a['losses']=f"lost {len(lost)} of its {nn+len(lost)} soldiers ({lm:.0f} metal, {words}) in the last 30 s"
    elif 'losses' in a: del a['losses']
    return s
def cut_state(s,g):
    s=copy.deepcopy(s)
    en=s['enemy']
    for k in ('never_looked','looked_long_ago'):
        v=en.get(k)
        if isinstance(v,str): en[k]=re.sub(r'((?:spot_\d+ \([A-H]\d\)(?:, )?){3})[^.]*', r'\1, ...', v)
        elif isinstance(v,list): en[k]=v[:3]
    text=s['instructions']+json.dumps(en)+json.dumps(s['actors'].get(g))
    for name in list(s['places']):
        p=s['places'][name]
        if name.startswith('spot_') and not re.search(re.escape(name)+r'(?![\d_])',text) and (p.get('what','').startswith('metal spot never') or p.get('what','').startswith('free metal spot when last')):
            del s['places'][name]
    for name,a in s['actors'].items():
        if name!=g and isinstance(a,dict):
            parts=[a[k] for k in ('units',) if k in a]+([f"at {a['at']}"] if 'at' in a else [])+([a['doing']] if 'doing' in a else [])
            s['actors'][name]=', '.join(parts)
    s['recent']=[r for r in s.get('recent',[]) if ' finished ' not in r]
    return s
NEW_DO=lambda g: f"Given `actors.{g}`, `enemy`, `player` and the player's `instructions`, what should {g} do next? The instructions were written before this picture: when {g}'s own `enemies_near` line says the party outweighs it, or its `losses` line says it is losing this fight or being wiped out, that outranks an instruction to keep fighting and it falls back. A few losses, or a noticeable share, against a party it outweighs are the cost of fighting, even while that party is killing some of its soldiers: it keeps on."
ENGAGE="Attack the enemy party named in `whom` now and follow it: the parties in sight are under `enemy.in_sight`, and how this group weighs against the nearest is on its own `enemies_near` line."
def new_questions(c,g,cut):
    q={k:copy.deepcopy(v) for k,v in c['questions'].items() if k.startswith(g+'.')}
    q[g+'.do']['instructions']=NEW_DO(g)
    if 'engage' in q[g+'.do']['criteria']: q[g+'.do']['criteria']['engage']=ENGAGE
    if cut and g+'.where' in q:
        w=q[g+'.where']; w['criteria']={k:None for k in w['criteria']}; w['instructions']=w['instructions']+" Each option is a place; what stands there and who is near it is its entry under `places`."
    return q
rows=[]
for c,g,ef in moments:
    base_state=dict(c['state']); base_state['instructions']=c['_instr']; base_state['rules']=c['_rules']
    base_q={k:v for k,v in c['questions'].items() if k.startswith(g+'.')}
    variants={'base':(base_state,base_q),'prompt':(new_state(c,g,ef),new_questions(c,g,False)),'prompt+cuts':(cut_state(new_state(c,g,ef),g),new_questions(c,g,True))}
    out={}
    for name,(s,q) in variants.items():
        runs=[]
        for _ in range(REPEAT):
            r=ask(key,s,q,'jev-latest'); a=r['answers'][g+'.do']
            runs.append((a['choice'],a['probabilities'],r.get('usage',{}).get('input_tokens')))
        top=collections.Counter(x[0] for x in runs).most_common(1)[0][0]
        fb=S.mean(x[1].get('fall_back',0)+x[1].get('retreat',0) for x in runs)
        out[name]=(top,fb,runs[0][2])
    rec_choice=c['answers'][g+'.do']['choice']
    a=new_state(c,g,ef)['actors'][g]
    print(f"{clock(c['f'])} {g}: recorded {rec_choice}; "+'; '.join(f"{k}: {v[0]} (fb+ret {v[1]:.2f}, {v[2]} tok)" for k,v in out.items())+f"\n     near: {a.get('enemies_near','-')[:110]} | losses: {a.get('losses','-')[:90]}")
    rows.append((out,a))
for name in ('base','prompt','prompt+cuts'):
    print(name, 'fall_back/retreat top:', sum(1 for o,_ in rows if o[name][0] in ('fall_back','retreat')), 'of', len(rows), '; mean tokens', S.mean(o[name][2] for o,_ in rows if o[name][2]))
