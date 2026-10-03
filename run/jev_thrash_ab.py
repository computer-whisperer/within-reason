#!/usr/bin/env python3
"""Offline: the thrash at a fortified place (human-10, group_O_t1 at the D4 outpost, 10:14-10:41 and 14:00-14:25:
fourteen course changes in 27 s between the advance and the walk back, `docs/studies/2026-10-02-thrash-words.md`)
asked again under three word changes and their sum: `odds` adds the whole group's weight against the party and its
turrets to the `enemies_near` line and to every move's fight facts; `override` makes the rules' step-back sentence
yield when the instructions speak to this very fight; `standing` adds a sentence that a course chosen in the last
ten seconds stands unless something has happened; `pair` is `odds` with `override`. Scored per call: the course in force, the `change` noul, the best
walk-back and the best advance, and whether a walk back is offered at the bar while the course is the advance (the
thrash's left foot). A sample of other group gates measures the drift of each arm against asking again. Streams
the log (a human game's is hundreds of MB). The key is read, never printed.

    run/jev_thrash_ab.py <match> <actor> <from_s>-<to_s>[,<from_s>-<to_s>...] <n controls> <out.jsonl> [arms]
"""
import json, random, re, sys, os, collections, concurrent.futures, statistics
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask

ODDS = re.compile(r"for the (\d+) of its (\d+) soldiers near it, (it outweighs us|we outweigh it[^;:,]*|[^;:]*outweigh[^;:]*)")
STEP_BACK_OLD = "whatever they said about keeping on: they could not see this."
STEP_BACK_NEW = "whatever they said about keeping on, unless the instructions speak to this very fight (this party, its place or the turrets covering it, by name): those they could see, and a group they told to go in against it goes in as one body."
# The places the advance was aimed at (human-10's outpost at spot_42); `WITHIN_REASON_TARGET` sets another list.
TARGET = os.environ.get("WITHIN_REASON_TARGET", "outpost,spot_42").split(",")
STANDING = " A course the group chose in the last ten seconds (its `last_pick` line) stands unless something has happened since that the instructions or these rules answer: a group that reverses its course every few seconds walks back and forth under fire and loses more than either course would."

def stream(match):
    """Every call row of jev-0.jsonl with the instructions and rules in force, without loading the file."""
    path = os.path.join(match, "jev-0.jsonl")
    rules = ""; packet = ""
    with open(path) as fh:
        for line in fh:
            if '"t":"header"' in line:
                rules = json.loads(line).get("rules", ""); continue
            if '"t":"call"' not in line: continue
            c = json.loads(line)
            if "instructions" in c: packet = c["instructions"]
            if "rules" in c: rules = c["rules"]
            if not isinstance(c.get("state"), dict) or "questions" not in c: continue
            yield c, dict(sorted({**c["state"], "instructions": packet, "rules": rules}.items()))

def whole_group_clause(state, actor):
    """"; the whole group, 51 soldiers worth 4887 metal, against the party and the turrets covering it (340 and 825, the turrets counted three times: 2815) outweighs it 1.7 to 1" from the entry and the party's `in_sight` line."""
    entry = state["actors"].get(actor) or {}
    near = entry.get("enemies_near", "")
    m = re.match(r"(party_\d+)", near)
    units = entry.get("units", "")
    g = re.search(r"(\d+) soldiers worth (\d+) metal", units)
    if not m or not g: return None
    party = m.group(1); n, metal = int(g.group(1)), int(g.group(2))
    line = next((s for s in state["enemy"].get("in_sight", []) if isinstance(s, str) and s.startswith(party + ":")), "")
    pm = re.search(r"worth (\d+) metal", line); tm = re.search(r"turrets?:[^()]*\((\d+) metal\)", line)
    if not pm: return None
    p = int(pm.group(1)); t = int(tm.group(1)) if tm else 0
    against = p + 3 * t
    ratio = metal / max(1, against)
    verdict = f"outweighs it {ratio:.1f} to 1" if ratio >= 1.05 else f"is outweighed by it ({ratio:.1f} to 1)" if ratio <= 0.95 else "matches it"
    turrets = f" and the turrets covering it ({p} and {t}, the turrets counted three times: {against})" if t else f" ({p})"
    return f"; the whole group, {n} soldiers worth {metal} metal, against the party{turrets} {verdict}"

def arm_request(arm, state, questions, actor):
    st = json.loads(json.dumps(state)); qs = json.loads(json.dumps(questions))
    if arm in ("odds", "pair", "all"):
        clause = whole_group_clause(st, actor)
        if clause:
            e = st["actors"][actor]
            e["enemies_near"] = ODDS.sub(lambda m: m.group(0) + clause, e["enemies_near"], count=1)
            for k, q in qs.items():
                if k.startswith(actor + ".") and isinstance(q.get("instructions"), str):
                    q["instructions"] = ODDS.sub(lambda m: m.group(0) + clause, q["instructions"], count=1)
    if arm in ("override", "pair", "all"):
        assert STEP_BACK_OLD in st["rules"], "the step-back sentence is not as expected"
        st["rules"] = st["rules"].replace(STEP_BACK_OLD, STEP_BACK_NEW)
    if arm in ("standing", "all"):
        st["rules"] = st["rules"].rstrip() + STANDING
    return st, qs

def chunks(state, questions):
    size = len(json.dumps(state)); chunk = {}; out = []
    for k, v in questions.items():
        L = len(json.dumps(v)) + len(k) + 6
        if chunk and size + L > 140_000:
            out.append(chunk); chunk = {}; size = len(json.dumps(state))
        chunk[k] = v; size += L
    if chunk: out.append(chunk)
    return out

def ask_all(key, state, questions):
    answers = {}; tokens = 0
    for part in chunks(state, questions):
        r = ask(key, state, part, "jev-latest")
        answers.update({k: a.get("noul") for k, a in (r.get("answers") or {}).items() if isinstance(a, dict)})
        tokens += (r.get("usage") or {}).get("input_tokens", 0)
    return answers, tokens

def course_of(questions, actor):
    q = questions.get(f"{actor}.change", {}).get("instructions", "")
    m = re.search(r"\(" + re.escape(actor) + r" (advances|walks|attacks|holds|stands|gathers|shells)", q)
    return {"advances": "fight_to", "walks": "go", "attacks": "attack"}.get(m.group(1), m.group(1)) if m else "?"

def score(answers, questions, actor):
    course = course_of(questions, actor)
    change = answers.get(f"{actor}.change")
    # The advance is toward the target the instructions named (`TARGET`); a walk or an advance anywhere else is back.
    forward = lambda k: any(t in k for t in TARGET)
    back = max(((v, k) for k, v in answers.items() if (k.startswith(actor + ".go_") or k.startswith(actor + ".fight_to_")) and not forward(k) and v is not None), default=(None, None))
    fwd = max(((v, k) for k, v in answers.items() if k.startswith(actor + ".fight_to_") and forward(k) and v is not None), default=(None, None))
    return {"course": course, "change": change, "back": back[0], "back_id": back[1], "fwd": fwd[0], "fwd_id": fwd[1]}

def main():
    match, actor, windows, n_ctl, out = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), sys.argv[5]
    arms = sys.argv[6].split(",") if len(sys.argv) > 6 else ["control", "odds", "override", "standing", "all"]
    spans = [(int(a) * 30, int(b) * 30) for a, b in (w.split("-") for w in windows.split(","))]
    key = api_key()
    moments = []; others = []
    for c, state in stream(match):
        if not any(k.startswith("group_") and k.endswith(".change") for k in c["questions"]): continue
        if any(lo <= c["f"] <= hi for lo, hi in spans) and any(k.startswith(actor + ".") for k in c["questions"]):
            moments.append((c, state))
        else:
            others.append((c, state))
    random.seed(5); random.shuffle(others); controls = others[:n_ctl]
    est = sum((c.get("usage") or {}).get("input_tokens", 0) for c, _ in moments + controls) * len(arms)
    print(f"{len(moments)} moments, {len(controls)} controls, {len(arms)} arms: about {est/1e6:.1f}M tokens (${0.042*est/1e6:.2f})", flush=True)
    def run(job):
        (c, state), arm, kind = job
        st, qs = arm_request(arm, state, c["questions"], actor) if arm != "control" else (state, c["questions"])
        answers, tokens = ask_all(key, st, qs)
        return c["f"], arm, kind, answers, tokens
    res = collections.defaultdict(dict); toks = collections.Counter()
    jobs = [((c, s), a, "moment") for c, s in moments for a in arms] + [((c, s), a, "control") for c, s in controls for a in arms]
    with concurrent.futures.ThreadPoolExecutor(6) as ex:
        for f, arm, kind, answers, tokens in ex.map(run, jobs):
            res[(kind, f)][arm] = answers; toks[arm] += tokens
    recorded = {c["f"]: c for c, _ in moments + controls}
    with open(out, "w") as fh:
        for (kind, f), arms_ in res.items():
            fh.write(json.dumps({"f": f, "kind": kind, "recorded": {k: a.get("noul") for k, a in recorded[f]["answers"].items()}, **arms_}) + "\n")
    print("spent: " + ", ".join(f"{a} {toks[a]/1e6:.2f}M" for a in arms) + f"; ${0.042*sum(toks.values())/1e6:.2f}")
    # The moments: per arm, the left foot (a walk back at the bar while advancing) and the right (an advance at the bar while walking).
    print(f"\n{actor} at the moments (bar 0.5 on the move, with `change` at 0.5):")
    print("  arm       | advancing: n, walk back offered at the bar, best walk back (median), best advance | walking: n, advance offered, best advance, best walk back | change (median)")
    for arm in ["recorded"] + arms:
        rows = []
        for (kind, f), arms_ in res.items():
            if kind != "moment": continue
            a = arms_[arm] if arm != "recorded" else {k: v.get("noul") for k, v in recorded[f]["answers"].items()}
            rows.append(score(a, recorded[f]["questions"], actor))
        adv = [r for r in rows if r["course"] == "fight_to"]; walk = [r for r in rows if r["course"] == "go"]
        def med(xs): return f"{statistics.median(xs):.2f}" if xs else "-"
        lf = sum(1 for r in adv if (r["back"] or 0) >= 0.5 and (r["change"] or 0) >= 0.5)
        rf = sum(1 for r in walk if (r["fwd"] or 0) >= 0.5 and (r["change"] or 0) >= 0.5)
        print(f"  {arm:9s} | {len(adv)}, {lf}, {med([r['back'] for r in adv if r['back'] is not None])}, {med([r['fwd'] for r in adv if r['fwd'] is not None])} | {len(walk)}, {rf}, {med([r['fwd'] for r in walk if r['fwd'] is not None])}, {med([r['back'] for r in walk if r['back'] is not None])} | {med([r['change'] for r in rows if r['change'] is not None])}")
    print("\nthe controls (other group gates): answers crossing 0.5 against the record")
    for arm in arms:
        n = cross = 0; diffs = []
        for (kind, f), arms_ in res.items():
            if kind != "control": continue
            rec = {k: v.get("noul") for k, v in recorded[f]["answers"].items()}
            for k, a in rec.items():
                b = arms_[arm].get(k)
                if a is None or b is None: continue
                n += 1; diffs.append(abs(a - b)); cross += (a >= 0.5) != (b >= 0.5)
        print(f"  {arm:9s} nouls {n}, crossing {cross} ({100*cross/max(1,n):.2f}%), mean |diff| {statistics.mean(diffs) if diffs else 0:.3f}")

if __name__ == "__main__":
    main()
