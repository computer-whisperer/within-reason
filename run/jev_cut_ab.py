#!/usr/bin/env python3
"""Offline, on a sample of a game's gate calls (`docs/studies/2026-10-02-jev-question-cuts.md`): `control`, the
recorded questions asked again (the noise floor); `short`, an actor's own moves asked as their words alone with the
framing said once in `rules`, and `short2`, the same with "Rather than: {course}" kept; `places:vN`, one noul per
place asking whether anyone has a reason to go there (the wordings in `WORDINGS`), scored against the recorded
per-actor answers: the places kept at each bar and the recorded moves at 0.5 whose place it kept. The sample is
two thirds spread over the game and one third its largest gates. The key is read, never printed.

    run/jev_cut_ab.py <match> <n gates> <out.jsonl> [control,short,short2,places:v0,...]
"""
import json, random, sys, collections, re, concurrent.futures, statistics, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from match_read import Match

PRE = re.compile(r"^Given `actors\.[^`]+`, `economy`, `ours` and the player's `instructions`: is this what \S+ should do now, rather than (.*?)\? The move: (.*)$", re.S)
NOTE = ("\n\nEach question that begins with the name of one of ours is a move for it: it asks whether that move is what the actor "
        "should do now, rather than what it does now (its `doing` line in `actors`).")
PLACE = re.compile(r"^(go|fight_to|build_[a-z0-9]+|take_apart|shell)_(.+)$")

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

def short_arm(state, questions, standing=False):
    st = dict(state); st["rules"] = st.get("rules", "") + NOTE
    qs = {}
    for k, q in questions.items():
        m = PRE.match(q.get("instructions", "")) if isinstance(q.get("instructions"), str) else None
        qs[k] = {"type": "noul", "instructions": m.group(2) if not standing else f"{m.group(2)} Rather than: {m.group(1)}."} if m else q
    return st, qs

def place_of(k):
    actor, key = k.split(".", 1)
    if key.endswith(".forbidden") or ".attack_party" in k or re.search(r"\.send_\d+_party", k):
        return None
    m = PLACE.match(key)
    if not m: return None
    verb, rest = m.groups()
    if verb.startswith("build_"):
        mm = re.match(r"^build_[a-z0-9]+_(.+)$", key)
        return mm.group(1) if mm else None
    return rest

WORDINGS = {
    "v0": lambda p, facts, who: f"Given `actors`, `enemy`, `economy`, `ours` and the player's `instructions`: does any of ours have a reason to go to, fight at or build at {p} now ({facts}), rather than carry on with what it does? Those with a move there: {who}. No when nothing there needs doing now or the instructions send everyone elsewhere.",
    "v1": lambda p, facts, who: f"Given `actors`, `enemy`, `economy`, `ours` and the player's `instructions`: should one of ours drop what it does now to go to, fight at or build at {p} ({facts})? Those who could: {who}. Yes only when the instructions send someone there or something there needs doing this minute (a spot to take, an enemy to meet, a building to raise or repair) and nothing of ours is already seeing to it; no when the place is merely reachable, or what stands there is fine as it is.",
    "v2": lambda p, facts, who: f"Read the player's `instructions` first, then `actors`, `enemy`, `economy` and `ours`: is {p} ({facts}) a place the instructions or the game call one of ours to right now? Those who could go: {who}. Most places are not: say no unless you can name what would be done there and why now.",
    "v3": lambda p, facts, who: f"Given `actors`, `enemy`, `economy`, `ours` and the player's `instructions`: would the player, looking at the picture now, send any of ours to {p} ({facts}) this second, taking it off what it does? Those who could: {who}. No when the instructions say nothing about it and nothing there is changing.",
    "v4": lambda p, facts, who: f"{p} ({facts}): does anyone of ours need to go there, fight there or build there now? Could: {who}. Yes when the instructions or the game ask for it now; no otherwise.",
    "v5": lambda p, facts, who: f"Given `actors`, `enemy`, `economy`, `ours` and the player's `instructions`: of the places our actors could go to this second, is {p} ({facts}) one where a move would be worth its cost in walking and in what the actor leaves? Those who could: {who}. A move is worth it when the instructions name the place or ask for what it gives (a free spot, a wreck, a turret's cover, an enemy within reach), and nothing of ours already covers it; otherwise no.",
}

def places_arm(state, questions, wording="v0"):
    by = collections.defaultdict(list)
    for k, q in questions.items():
        p = place_of(k)
        if p: by[p].append((k, q["instructions"]))
    qs = {}
    for p, moves in by.items():
        who = []; facts = ""
        for k, text in moves:
            m = PRE.match(text); words = m.group(2) if m else text
            actor = k.split(".")[0]
            # "constructor_1 builds a Sentry (armllt) at spot_56 (far, about 50 s of walking, our extractor ...)"
            inside = ""
            i = words.find(" " + p + " (")
            if i >= 0:
                j = i + len(p) + 3; depth = 1; k2 = j
                while k2 < len(words) and depth:
                    depth += {"(": 1, ")": -1}.get(words[k2], 0); k2 += 1
                inside = words[j:k2 - 1]
            dist = inside.split(",")[0] if inside else ""
            if not facts and inside: facts = inside
            verb = re.sub(r" (at|to)$", "", words[len(actor) + 1:].split(" " + p)[0]) if words.startswith(actor) else ""
            who.append(f"{actor} ({verb}{', ' + dist if dist else ''})")
        seen = set(); who = [w for w in who if not (w in seen or seen.add(w))]
        who_s = '; '.join(who[:12]) + (' and others' if len(who) > 12 else '')
        qs[p] = {"type": "noul", "instructions": WORDINGS[wording](p, facts, who_s)}
    return state, qs, by

def main():
    m = Match(sys.argv[1]); n = int(sys.argv[2]); out = open(sys.argv[3], "w")
    arms = sys.argv[4].split(",") if len(sys.argv) > 4 else ["control", "short", "places"]
    key = api_key()
    gates = [(c, st) for c, st, _ in requests_of(m) if c.get("t") == "call" and any(q.get("type") == "noul" for q in c["questions"].values()) and not all(k.endswith(".answer") for k in c["questions"])]
    random.seed(7)
    gates.sort(key=lambda g: g[0]["f"])
    step = max(1, len(gates) // (n * 2 // 3))
    sample = gates[::step][: n * 2 // 3]
    big = sorted(gates, key=lambda g: -len(g[0]["questions"]))
    for g in big:
        if len(sample) >= n: break
        if g not in sample: sample.append(g)
    est = sum(len(json.dumps(c["state"])) * c["batches"] + sum(len(json.dumps(q)) for q in c["questions"].values()) for c, _ in sample) / 2.78
    print(f"{len(sample)} gates, {sum(len(c['questions']) for c, _ in sample)} questions; the control alone about {est/1e6:.2f}M tokens", flush=True)
    def run(job):
        (c, state), arm = job
        if arm == "control": answers, tokens = ask_all(key, state, c["questions"]); extra = None
        elif arm in ("short", "short2"):
            st, qs = short_arm(state, c["questions"], arm == "short2"); answers, tokens = ask_all(key, st, qs); extra = {"chars": sum(len(q["instructions"]) for q in qs.values())}
        else:
            st, qs, by = places_arm(state, c["questions"], arm.split(":")[1] if ":" in arm else "v0"); answers, tokens = ask_all(key, st, qs); extra = {"chars": sum(len(q["instructions"]) for q in qs.values()), "members": {p: [k for k, _ in v] for p, v in by.items()}}
        return c["f"], arm, answers, tokens, extra
    res = collections.defaultdict(dict); toks = collections.Counter(); extras = collections.defaultdict(dict)
    with concurrent.futures.ThreadPoolExecutor(6) as ex:
        for f, arm, answers, tokens, extra in ex.map(run, [(g, a) for g in sample for a in arms]):
            res[f][arm] = answers; toks[arm] += tokens; extras[f][arm] = extra
    recorded = {c["f"]: c for c, _ in sample}
    for f, arms_ in res.items():
        out.write(json.dumps({"f": f, "recorded": {k: a.get("noul") for k, a in recorded[f]["answers"].items()}, "extras": extras[f], **arms_}) + "\n")
    rec_tokens = sum(recorded[f]["usage"].get("input_tokens", 0) for f in res)
    rec_chars = sum(sum(len(q["instructions"]) for q in recorded[f]["questions"].values()) for f in res)
    print(f"recorded: {rec_tokens/1e6:.2f}M tokens, {rec_chars/1e6:.2f}M question chars; spent: " + ", ".join(f"{a} {toks[a]/1e6:.2f}M" for a in arms) + f"; total ${0.042*sum(toks.values())/1e6:.2f}")
    for arm in [a for a in arms if a in ("control", "short", "short2")]:
        c = collections.Counter(); diffs = []; by_verb = collections.Counter(); by_verb_n = collections.Counter()
        for f, arms_ in res.items():
            rec = {k: a.get("noul") for k, a in recorded[f]["answers"].items()}
            for k, a in rec.items():
                b = arms_[arm].get(k)
                if a is None or b is None: continue
                own = PRE.match(recorded[f]["questions"][k].get("instructions", "") or "") is not None
                c["n"] += 1; diffs.append(abs(a - b))
                if (a >= 0.5) != (b >= 0.5):
                    c["cross"] += 1; c["now yes" if b >= 0.5 else "now no"] += 1
                    if own: c["cross own"] += 1
                if own: c["own"] += 1
                if a >= 0.5: c["rec yes"] += 1
                if b >= 0.5: c["arm yes"] += 1
        chars = sum(extras[f][arm]["chars"] for f in res) if arm != "control" else rec_chars
        print(f"  {arm:8s} question chars {chars/1e6:.2f}M ({100*chars/rec_chars:.0f}%); nouls {c['n']} (own moves {c['own']}); crossing 0.5 against the record {c['cross']} ({100*c['cross']/max(1,c['n']):.2f}%; own moves {c['cross own']}): now yes {c['now yes']}, now no {c['now no']}; yes in the record {c['rec yes']}, in the arm {c['arm yes']}; mean |diff| {statistics.mean(diffs):.3f}")
    for parm in [a for a in arms if a.startswith("places")]:
        pairs = []; chars = sum(extras[f][parm]["chars"] for f in res)
        print(f"  {parm}:")
        for f, arms_ in res.items():
            rec = {k: a.get("noul") for k, a in recorded[f]["answers"].items()}
            for p, members in extras[f][parm]["members"].items():
                b = arms_[parm].get(p)
                if b is None: continue
                best = max((rec.get(k) for k in members if rec.get(k) is not None), default=None)
                ctl = max((arms_.get("control", {}).get(k) for k in members if arms_.get("control", {}).get(k) is not None), default=None) if "control" in arms_ else None
                pairs.append((f, p, len(members), b, best, ctl))
        n_q = sum(x[2] for x in pairs)
        print(f"    question chars {chars/1e6:.2f}M ({100*chars/rec_chars:.0f}% of the record's); {len(pairs)} place questions standing for {n_q} move questions")
        for bar in (0.1, 0.2, 0.3, 0.5):
            kept = [x for x in pairs if x[3] >= bar]
            kept_q = sum(x[2] for x in kept)
            hits = [x for x in pairs if x[4] is not None and x[4] >= 0.5]
            caught = [x for x in hits if x[3] >= bar]
            print(f"    pre-gate bar {bar}: keeps {len(kept)} places ({100*len(kept)/len(pairs):.0f}%), {kept_q} move questions ({100*kept_q/n_q:.0f}%); of {len(hits)} places with a recorded move at 0.5, caught {len(caught)} ({100*len(caught)/max(1,len(hits)):.0f}%)")
        missed = sorted([x for x in pairs if x[4] is not None and x[4] >= 0.5 and x[3] < 0.3], key=lambda x: -x[4])[:12]
        for f, p, nm, b, best, ctl in missed:
            print(f"    missed: {f//1800}:{f//30%60:02d} {p} pre-gate {b:.2f}, best recorded move {best:.2f}, control's best {ctl if ctl is None else round(ctl,2)}")
        same = [x for x in pairs if x[5] is not None]
        if same:
            print(f"    the control's own repeat: of {sum(1 for x in same if x[4] >= 0.5)} places with a recorded move at 0.5, the control repeats one at 0.5 in {sum(1 for x in same if x[4] >= 0.5 and x[5] >= 0.5)}")

if __name__ == "__main__":
    main()
