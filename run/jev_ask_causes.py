#!/usr/bin/env python3
"""Why the hands' gates went and why each group was asked, from `jev-<ai>.jsonl`: every gate by the event that called
it, every group ask by the news that named the group, with the question tokens each cause carried, and the group's
losses between asks (members gone, as a share of the body at the last ask).

    run/jev_ask_causes.py run/matches/<batch> [...]
"""
import jevlog
import collections, glob, json, sys


def cause(events, name):
    mine = [e[len(name) + 1:] for e in events if e.startswith(name + " ")]
    for word in ("reached", "met", "sighted", "opened", "hit"):
        if any(m.startswith(word) for m in mine):
            return word
    return "no event (course, party, 20 s)"


for D in sys.argv[1:]:
    rows = []
    for fn in jevlog.logs(D.rstrip("/") + "/00")[:1]:
        for l in jevlog.lines(fn):
            try: rows.append(json.loads(l))
            except ValueError: pass
    gate_cause = collections.Counter(); ask = collections.Counter(); ask_tok = collections.Counter(); ask_q = collections.Counter()
    loss_bins = collections.Counter(); loss_tok = collections.Counter()
    last_gate = None; pending = None; size_at_ask = {}; total = 0
    for r in rows:
        t = r.get("t")
        if t in ("call", "decode"): total += (r.get("usage") or {}).get("input_tokens", 0) or 0
        if t == "pass" and r.get("gate"):
            ev = r.get("events") or []
            gap = (r["f"] - last_gate) // 30 if last_gate is not None else 99
            c = ("opened" if any(e.endswith(" opened") for e in ev) else "reached" if any(" reached " in e for e in ev)
                 else "met" if any(" met " in e for e in ev) else "an event after 5 s" if ev and gap >= 5 else "picture changed or re-ask due")
            gate_cause[c] += 1; last_gate = r["f"]; pending = r
        elif t == "call" and pending is not None and "questions" in r and not any(k.startswith("worlds") for k in r["questions"]):
            q = r["questions"]; T = r["usage"]["input_tokens"]; b = r.get("batches", 1) or 1
            st = sum(len(json.dumps(v)) for v in (r.get("state") or {}).values()) + 9000
            tot = st * b + sum(len(json.dumps(v)) for v in q.values())
            sizes = {"group_" + g["name"]: len(g["members"]) for g in r.get("groups") or [] if "name" in g}
            per = collections.Counter(); n = collections.Counter()
            for k, v in q.items():
                a = k.split(".")[0]
                if a.startswith("group_"): per[a] += T * len(json.dumps(v)) / tot; n[a] += 1
            ev = pending.get("events") or []
            for a in pending.get("open") or []:
                if not a.startswith("group_"): continue
                c = cause(ev, a); ask[c] += 1; ask_tok[c] += per[a]; ask_q[c] += n[a]
                now = sizes.get(a); before = size_at_ask.get(a)
                if c == "hit" and now and before:
                    lost = max(0, before - now) / before
                    b_ = "none lost" if lost == 0 else "under 5%" if lost < 0.05 else "5-10%" if lost < 0.10 else "10-20%" if lost < 0.2 else "20% or more"
                    loss_bins[b_] += 1; loss_tok[b_] += per[a]
                if now: size_at_ask[a] = now
            pending = None
    print(f"== {D.rstrip('/').split('/')[-1][11:]}: {total/1e6:.1f}M tokens")
    print("  gates by what called them: " + ", ".join(f"{k} {v}" for k, v in gate_cause.most_common()))
    print("  group asks by the news that named the group:")
    for k, v in ask.most_common():
        print(f"    {k:32s} {v:5d} asks, {ask_q[k]/v:5.1f} questions an ask, {ask_tok[k]/1e6:5.2f}M question tokens ({100*ask_tok[k]/total:.0f}% of the game)")
    print("  asks on a hit alone, by the body's losses since its last ask: " + ", ".join(f"{k} {loss_bins[k]} ({loss_tok[k]/1e6:.2f}M)" for k in ("none lost", "under 5%", "5-10%", "10-20%", "20% or more")))


def replay(D, threshold):
    """The same log with a hit as news only once the body has lost `threshold` of its members since its last ask
    (0: any member lost). An ask on a hit below that is dropped, and with it the `opened` asks that follow it second
    by second (they exist only because the dropped ask's opener said yes). Counted: question tokens dropped, gates
    left with nobody to ask, and the course changes the dropped asks played (what the replay would have delayed)."""
    rows = []
    for l in jevlog.lines(jevlog.seat(D.rstrip("/") + "/00")):
        try: rows.append(json.loads(l))
        except ValueError: pass
    total = sum((r.get("usage") or {}).get("input_tokens", 0) or 0 for r in rows if r.get("t") in ("call", "decode"))
    dropped_at = {}; size_at_ask = {}; pending = None; dropped_now = set()
    saved = 0.0; asks = 0; gates_empty = 0; gate_tok = 0.0; changes = []; kept_changes = 0
    for r in rows:
        t = r.get("t")
        if t == "pass" and r.get("gate"):
            pending = r; dropped_now = set()
        elif t == "call" and pending is not None and "questions" in r and not any(k.startswith("worlds") for k in r["questions"]):
            q = r["questions"]; T = r["usage"]["input_tokens"]; b = r.get("batches", 1) or 1
            st = sum(len(json.dumps(v)) for v in (r.get("state") or {}).values()) + 9000
            tot = st * b + sum(len(json.dumps(v)) for v in q.values())
            sizes = {"group_" + g["name"]: len(g["members"]) for g in r.get("groups") or [] if "name" in g}
            ev = pending.get("events") or []; opened = pending.get("open") or []
            for a in opened:
                if not a.startswith("group_"): continue
                c = cause(ev, a); now = sizes.get(a); before = size_at_ask.get(a)
                drop = False
                if c == "hit" and now and before:
                    lost = max(0, before - now) / before
                    drop = lost < threshold if threshold else lost == 0
                elif c == "opened" and dropped_at.get(a) is not None and r["f"] - dropped_at[a] <= 30:
                    drop = True
                if drop:
                    dropped_at[a] = r["f"]; dropped_now.add(a); asks += 1
                    saved += sum(T * len(json.dumps(v)) / tot for k, v in q.items() if k.split(".")[0] == a and ".send_" not in k and ".attack_" not in k and ".shell_party" not in k)
                elif now:
                    size_at_ask[a] = now
            if opened and all(a in dropped_now for a in opened) and not any(" reached " in e or " met " in e for e in ev):
                gates_empty += 1; gate_tok += T
            pending = None
        elif t == "plan":
            for c in r.get("changed") or []:
                a = c.split(":")[0]
                if a.startswith("group_"):
                    if a in dropped_now: changes.append((r["f"], c))
                    else: kept_changes += 1
    return total, saved, asks, gates_empty, gate_tok, changes, kept_changes


for D in sys.argv[1:]:
    print(f"== {D.rstrip('/').split('/')[-1][11:]}: a hit is news only at a share of the body lost")
    for th in (0, 0.05, 0.10, 0.20):
        total, saved, asks, ge, gt, changes, kept = replay(D, th)
        print(f"  {'any loss' if th == 0 else f'{th:.0%} lost':9s}: {asks:4d} group asks dropped, {saved/1e6:5.2f}M of their own questions ({100*saved/total:.1f}%); {ge} gates with nobody else to ask ({gt/1e6:.2f}M, {100*gt/total:.1f}%); course changes played at dropped asks {len(changes)} of {len(changes)+kept}")
    total, saved, asks, ge, gt, changes, kept = replay(D, 0.05)
    kinds = collections.Counter(c.split(": ", 1)[1].split(" spot_")[0].split(" party_")[0].split(" group_")[0] for _, c in changes)
    print("    at 5%, what the dropped asks changed: " + ", ".join(f"{k} {v}" for k, v in kinds.most_common(10)))
