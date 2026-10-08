import json, sys, glob, os, re
# Every switch of a group into roving in the recorded games (jev logs v3): what the group was doing at the last
# call before the switch, from the call line's `groups` task kind.
rows=[]
for path in sorted(glob.glob("run/matches/*/**/jev-*.jsonl", recursive=True)):
    try:
        with open(path, "rb") as f:
            if b"roving, in code" not in f.read(): continue
    except OSError: continue
    match=path.split("/")[2]
    last_tasks={}   # group name -> (frame, task kind, to/place)
    roving=set()
    with open(path) as f:
        for line in f:
            if "\"t\":\"call\"" not in line and "\"t\": \"call\"" not in line: continue
            try:o=json.loads(line)
            except: continue
            fr=o.get("f")
            st=o.get("state",{}); actors=st.get("actors",{}) if isinstance(st,dict) else {}
            now_roving={a for a,v in actors.items() if a.startswith("group_") and "roving, in code" in (v if isinstance(v,str) else json.dumps(v))}
            for g in now_roving-roving:
                prev=last_tasks.get(g)
                rows.append((match, os.path.basename(path), fr, g, prev))
            roving=now_roving
            for g in o.get("groups") or []:
                t=g.get("task",{})
                last_tasks[f"group_{g['name']}"]=(fr, t.get("kind"), t.get("place") or t.get("to") or t.get("party") or "")
for r in rows:
    m,f,fr,g,prev=r
    clock=f"{fr//1800}:{(fr%1800)//30:02d}"
    print(f"{m:40s} {f:12s} {clock:>6s} {g:14s} before: {prev[1] if prev else '?'} {prev[2] if prev else ''} (at {prev[0]//1800}:{(prev[0]%1800)//30:02d})" if prev else f"{m:40s} {f:12s} {clock:>6s} {g:14s} before: (no call before)")
print(len(rows),"switches")
