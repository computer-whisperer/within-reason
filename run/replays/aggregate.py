"""A first pass over the replay cards in run/data/replays/manifest.jsonl (kept lines with a card): openings, second
factory, tier 2, extractor and income curves by minute for winners and losers, early Rovers, first extractor lost,
start cells. Numbers for the design doc and for checking a synthesis agent's claims; not claims themselves."""
import json,statistics as st,collections,sys
def secs(c):
    m,s=c.split(':'); return int(m)*60+int(s)
sides=[];games=0;higher_won=0;os_known=0
for l in open('run/data/replays/manifest.jsonl'):
    r=json.loads(l)
    if not (r.get('keep') and r.get('card')): continue
    c=json.load(open(r['match']+'/card.json')); games+=1
    winners=set(r.get('winners') or [])
    won_team={p['team'] for p in r['players'] if p['name'] in winners}
    pl={p['team']:p for p in r['players'] if p.get('os') is not None}
    if len(pl)==2 and won_team:
        os_known+=1
        hi=max(pl,key=lambda a:pl[a]['os'])
        higher_won+= (hi in won_team)
    for t in c['teams']:
        won = int(t['team']) in won_team
        sides.append((t,won))
print(games,'games',len(sides),'sides; higher OS won',higher_won,'of',os_known)
first=[secs(t['factories'][0]['started']) for t,_ in sides if t['factories']]
print('first factory median %d:%02d'%divmod(int(st.median(first)),60))
kinds=collections.Counter(t['factories'][0]['unit'] for t,_ in sides if t['factories'])
print('first factory kind',dict(kinds))
second=[secs(t['factories'][1]['started']) for t,_ in sides if len(t['factories'])>1]
print('second factory in %d of %d, median %.1f min'%(len(second),len(sides),st.median(second)/60))
t2=[secs(t['first_tier2']['clock']) for t,_ in sides if t['first_tier2']]
print('tier 2 in %d sides, median %.1f min'%(len(t2), st.median(t2)/60 if t2 else 0))
def at(t,minute,key):
    for row in t['curves']:
        if row['minute']==minute: return row.get(key)
for minute in (4,8,12):
    ex=[at(t,minute,'extractors') for t,_ in sides if at(t,minute,'extractors') is not None]
    inc=[at(t,minute,'metal_income') for t,_ in sides if at(t,minute,'metal_income') is not None]
    exw=[at(t,minute,'extractors') for t,w in sides if w and at(t,minute,'extractors') is not None]
    exl=[at(t,minute,'extractors') for t,w in sides if not w and at(t,minute,'extractors') is not None]
    print('at %d:00 extractors median %s (winners %s, losers %s) income median %s, n=%d'%(minute,st.median(ex),st.median(exw) if exw else '-',st.median(exl) if exl else '-',st.median(inc),len(ex)))
def army_at(t,minute):
    for row in t['army_by_type']:
        if row['minute']==minute: return row['army']
    return {}
fav=sum(1 for t,_ in sides if army_at(t,2).get('armfav',0)+army_at(t,2).get('corfav',0)>=5)
print('5+ Rovers by 2:00 sample in %d sides'%fav)
raid=[secs(t['first_extractor_lost']['clock']) for t,_ in sides if t.get('first_extractor_lost')]
print('first extractor lost: %d of %d sides, median %d:%02d'%(len(raid),len(sides),*divmod(int(st.median(raid)),60)))
starts=collections.Counter(t['start']['grid'] for t,_ in sides)
print('start cells',dict(starts))
