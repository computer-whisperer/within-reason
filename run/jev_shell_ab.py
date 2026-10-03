#!/usr/bin/env python3
"""Offline: the attack-shell thrash (human-12, group_T against four Centurions at E4 9:07-9:32 and group_G1 against
two at home 12:34-13:25: the hands alternated attack, shell and walk at one-second turns, and the shell move's
"long-reach soldiers" were Crossbows that cannot hit the ground). Each recorded gate in the windows is asked again
under: `control` (as recorded); `noshell` (the shell question dropped: what the artillery fix does when the only
long-reach soldiers are anti-air); `named` (`noshell` plus one sentence in the instructions naming this very fight,
the shape the thrash pair's override answers). Scored per gate: the `change` noul, the top-rated of the actor's own
moves and its noul, and across the window the number of times the top move changes from one gate to the next (every
change is a new order to the group) by exact move and by kind (attack / shell / walk / other). A sample of other
group gates measures each arm's drift against asking again. The key is read, never printed.

    run/jev_shell_ab.py <match> <actor>:<from_s>-<to_s>:<party>[,<actor>:...] <n controls> <out.jsonl> [arms]
"""
import json, random, re, sys, os, collections, concurrent.futures, statistics
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key
from jev_thrash_ab import stream, ask_all

# Anti-air soldiers count nothing against a ground party (the bot's odds use `can_hit`; the picture's whole-group
# clause does not yet).
ANTI_AIR = {"armjeth", "corcrash", "armaak", "coraak"}
NEAR = re.compile(r"(near " + r"(party_\d+) \([^)]*\): (?:for the \d+ of its \d+ soldiers near it, )?(?:it outweighs us|an even fight|we outweigh it[^;:?]*))")

def whole_group(state, actor, party):
    """"; the whole group, 28 soldiers worth 3458 metal, against the party (540) outweighs it 6.4 to 1", the group's
    fighting metal (anti-air left out) against the party's."""
    e = state["actors"].get(actor) or {}
    units = e.get("units", "")
    n = m = 0
    for cnt, name in re.findall(r"(\d+) [A-Za-z ]+ \((\w+)\)", units):
        if name in ANTI_AIR: continue
        cost = {"armpw": 54, "armham": 125, "armjeth": 125, "armflea": 21}.get(name)
        if cost is None:
            g = re.search(r"(\d+) soldiers worth (\d+) metal", units); return None if not g else None
        n += int(cnt); m += int(cnt) * cost
    line = next((s for s in state["enemy"].get("in_sight", []) if isinstance(s, str) and s.startswith(party + ":")), "")
    pm = re.search(r"worth (\d+) metal", line)
    if not pm or n == 0: return None
    p = int(pm.group(1)); ratio = m / max(1, p)
    verdict = f"outweighs it {ratio:.1f} to 1" if ratio >= 1.05 else f"is outweighed by it ({ratio:.1f} to 1)" if ratio <= 0.95 else "matches it"
    return f"; the whole group, {n} soldiers worth {m} metal, against the party ({p}) {verdict}"

def arm_request(arm, state, questions, actor, party):
    st = json.loads(json.dumps(state)); qs = json.loads(json.dumps(questions))
    if arm in ("noshell", "named", "noshell_course"):
        qs = {k: v for k, v in qs.items() if not k.startswith(actor + ".shell_")}
    if arm == "named":
        st["instructions"] = st["instructions"].rstrip() + f"\n\n{actor} attacks {party} now with the whole group and keeps attacking until it is dead."
    if arm in ("tail", "dup"):
        # Position within the instructions: the actor's own paragraph said again at the end (`dup`), or only its
        # first sentence, the player's own words for the fight, as the last line (`tail`).
        paras = st["instructions"].split("\n\n")
        own = [p for p in paras if p.lstrip().startswith(actor)]
        if own:
            first = re.match(r"(.*?\.)(\s|$)", own[0].replace(f"{actor} is the army. ", f"{actor} ", 1))
            add = own if arm == "dup" else [first.group(1) if first else own[0]]
            st["instructions"] = "\n\n".join(paras + add)
    if arm == "last":
        # Section order: `instructions` after `rules`, the last key of the state (K-jev-answers-move-with-the-order-of-the-pictures-sections scored this order blind on 2026-10-02).
        ins = st.pop("instructions"); st["instructions"] = ins
    if arm in ("course", "noshell_course"):
        clause = whole_group(st, actor, party)
        if clause:
            for k, q in qs.items():
                if k.startswith(actor + ".") and isinstance(q.get("instructions"), str):
                    q["instructions"] = NEAR.sub(lambda mm: mm.group(1) + clause, q["instructions"], count=1)
    return st, qs

def kind(key):
    move = key.split(".", 1)[1]
    if move.startswith("attack_"): return "attack"
    if move.startswith("shell_"): return "shell"
    if move.startswith(("go_", "fight_to_")): return "walk"
    return "other"

def top(answers, actor):
    """The actor's best own move (its forbidden marks and the party questions aside) and its noul."""
    own = [(v, k) for k, v in answers.items() if k.startswith(actor + ".") and not k.endswith(".forbidden") and k != f"{actor}.change" and v is not None]
    return max(own, default=(None, None))

def main():
    match, windows, n_ctl, out = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
    arms = sys.argv[5].split(",") if len(sys.argv) > 5 else ["control", "noshell", "named"]
    spans = []
    for w in windows.split(","):
        actor, span, party = w.split(":")
        a, b = span.split("-"); spans.append((actor, int(a) * 30, int(b) * 30, party))
    key = api_key()
    moments = []; others = []
    for c, state in stream(match):
        if not any(k.startswith("group_") for k in c["questions"]): continue
        hit = next(((actor, party) for actor, lo, hi, party in spans if lo <= c["f"] <= hi and any(k.startswith(actor + ".") for k in c["questions"])), None)
        if hit: moments.append((c, state, hit))
        else: others.append((c, state, (None, None)))
    random.seed(5); random.shuffle(others); controls = others[:n_ctl]
    est = sum((c.get("usage") or {}).get("input_tokens", 0) for c, _, _ in moments + controls) * len(arms)
    print(f"{len(moments)} moments, {len(controls)} controls, {len(arms)} arms: about {est/1e6:.1f}M tokens (${0.042*est/1e6:.2f})", flush=True)
    def run(job):
        (c, state, (actor, party)), arm, k = job
        st, qs = arm_request(arm, state, c["questions"], actor, party) if arm != "control" and actor else (state, c["questions"])
        answers, tokens = ask_all(key, st, qs)
        return c["f"], arm, k, answers, tokens
    res = collections.defaultdict(dict); toks = collections.Counter()
    jobs = [(m, a, "moment") for m in moments for a in arms] + [(m, a, "control") for m in controls for a in arms]
    with concurrent.futures.ThreadPoolExecutor(6) as ex:
        for f, arm, k, answers, tokens in ex.map(run, jobs):
            res[(k, f)][arm] = answers; toks[arm] += tokens
    recorded = {c["f"]: (c, hit) for c, _, hit in moments + controls}
    with open(out, "w") as fh:
        for (k, f), arms_ in res.items():
            fh.write(json.dumps({"f": f, "kind": k, "actor": recorded[f][1][0], "recorded": {q: a.get("noul") for q, a in recorded[f][0]["answers"].items()}, **arms_}) + "\n")
    print("spent: " + ", ".join(f"{a} {toks[a]/1e6:.2f}M" for a in arms) + f"; ${0.042*sum(toks.values())/1e6:.2f}")
    for actor, lo, hi, party in spans:
        print(f"\n{actor} {lo//1800}:{lo//30%60:02d}-{hi//1800}:{hi//30%60:02d} against {party}: per arm, gates; top move changes gate to gate (by move, by kind); gates whose top is attack / shell / walk / other; median noul of change, attack, shell, best walk")
        for arm in ["recorded"] + arms:
            seq = []
            for (k, f), arms_ in sorted(res.items(), key=lambda kv: kv[0][1]):
                if k != "moment" or recorded[f][1][0] != actor: continue
                a = arms_[arm] if arm != "recorded" else {q: v.get("noul") for q, v in recorded[f][0]["answers"].items()}
                v, key_ = top(a, actor)
                walks = [x for q, x in a.items() if q.startswith(actor + ".") and kind(q) == "walk" and x is not None]
                seq.append((f, key_, v, a.get(f"{actor}.change"), a.get(f"{actor}.attack_{party}"), a.get(f"{actor}.shell_{party}"), max(walks, default=None)))
            if not seq: continue
            by_move = sum(1 for p, q in zip(seq, seq[1:]) if p[1] != q[1])
            by_kind = sum(1 for p, q in zip(seq, seq[1:]) if p[1] and q[1] and kind(p[1]) != kind(q[1]))
            kinds = collections.Counter(kind(s[1]) for s in seq if s[1])
            def med(i): xs = [s[i] for s in seq if s[i] is not None]; return f"{statistics.median(xs):.2f}" if xs else "-"
            print(f"  {arm:9s} | {len(seq)}; {by_move}, {by_kind}; {kinds['attack']} / {kinds['shell']} / {kinds['walk']} / {kinds['other']}; {med(3)}, {med(4)}, {med(5)}, {med(6)}")
    print("\nthe controls (other group gates): answers crossing 0.5 against the record")
    for arm in arms:
        n = cross = 0; diffs = []
        for (k, f), arms_ in res.items():
            if k != "control": continue
            rec = {q: v.get("noul") for q, v in recorded[f][0]["answers"].items()}
            for q, a in rec.items():
                b = arms_[arm].get(q)
                if a is None or b is None: continue
                n += 1; diffs.append(abs(a - b)); cross += (a >= 0.5) != (b >= 0.5)
        print(f"  {arm:9s} nouls {n}, crossing {cross} ({100*cross/max(1,n):.2f}%), mean |diff| {statistics.mean(diffs) if diffs else 0:.3f}")

if __name__ == "__main__":
    main()
