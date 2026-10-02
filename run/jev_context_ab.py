#!/usr/bin/env python3
"""Offline: recorded gates or picks asked again with one piece of context cut, against the same request asked again
as recorded (the ask-again noise every arm is read against). The key is read, never printed.

    run/jev_context_ab.py ask <match> gate|pick <requests> <out.jsonl> [arm,arm...]
    run/jev_context_ab.py more <match> gate|pick <requests> <out.jsonl> arm,arm...   the same requests, more arms
    run/jev_context_ab.py read <out.jsonl>...
    run/jev_context_ab.py sizes <match> gate|pick          the text each arm removes over the whole game, no requests
    run/jev_context_ab.py show <match> <out.jsonl> <arm> [n]   questions an arm moved across the bar, with their words

Arms on the picture: `no roving` (the entries of groups roving in code, which take no orders), `asked actors` (only
the actors the request's questions name), `no buildings` (`enemy.buildings_seen`), `no scouting` (`enemy.never_looked`,
`looked_long_ago`, `start_box`, and the `produce` hint under `ours.resurrection_bots`), `rules by kind` (the rules'
factory, commander, allies and wind sentences only when an actor of that kind is asked about), `no rules`.
Arms on the questions: `no given` (the "Given `actors.x`, ...:" opening), `facts once` (a fight's facts said in the
course and again in the move: the second replaced by "(as said above)"), `short forbidden` (the forbidden question
without the move's odds), `preamble in state` (the pick's shared paragraph once, under `choosing`).
Placebos, which remove nothing: `placebo order` (the picture's sections in reverse order), `placebo words` (one line
added that bears on nothing). An arm that moves answers no more than these does what any change of the request does.
"""
import json, random, re, sys, collections, concurrent.futures, difflib
sys.path.insert(0, "run")

FLAG, FORBIDDEN = 0.5, 0.7
GIVEN = re.compile(r"^Given [^:]*: (.)")
SEND = re.compile(r"^Read the player's `instructions` alone: do they forbid this move for the group that would make it\? The move: (\d+) \S+ of (\S+) \(.*?\) leave it as a group of their own and hunt (party_\d+ \(.*?\)) \((?:[^()]*walking|[^()]*)\): ", re.S)
PICK_HEAD = re.compile(r"^Given `economy`, `ours`, `enemy`, `actors`, `player` and the player's `instructions`, which plan is best this second\? (Each option is one world: .*? pick the world they would call for\.) ", re.S)
RULE_PARTS = {
    "factory": (re.compile(r" A factory standing idle builds now .*? beyond a full store is wasted\.", re.S), ("plant_", "lab_")),
    "commander": (re.compile(r" The commander's `enemies_near` line weighs it .*? whatever it was building\.", re.S), ("commander",)),
    "wind": (re.compile(r"\nThis map's wind averages [^\n]*"), ("commander", "constructor_", "plant_", "lab_")),
}
ALLIES = re.compile(r"`allies` are the other seats' groups of ours: .*? but they are theirs to order; ", re.S)
NO_ALLIES = "no other seat of ours has a group"

def named(text, name):
    return re.search(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9])", text) is not None

def no_given(q):
    out = {}
    for k, v in q.items():
        m = GIVEN.match(v["instructions"])
        out[k] = {**v, "instructions": m.group(1).upper() + v["instructions"][m.end():]} if m else v
    return out

def facts_once(q):
    out = {}
    for k, v in q.items():
        text = v["instructions"]
        head, sep, move = text.partition("? The move: ")
        if sep and not k.endswith(".forbidden"):
            a, b, size = difflib.SequenceMatcher(None, head, move, autojunk=False).find_longest_match(0, len(head), 0, len(move))
            # whole clauses only: from a word's start to the end of a clause
            while size and b and move[b - 1] not in " ": b += 1; a += 1; size -= 1
            while size and b + size < len(move) and move[b + size] not in ";": size -= 1
            if size >= 60:
                span = move[b:b + size]
                party = re.match(r"party_\d+", span)
                move = move[:b] + (f"{party.group(0)} (as said above)" if party else "(as said above)") + move[b + size:]
                text = head + sep + move
        out[k] = {**v, "instructions": text}
    return out

def short_forbidden(q):
    out = {}
    for k, v in q.items():
        m = SEND.match(v["instructions"]) if k.endswith(".forbidden") else None
        out[k] = {**v, "instructions": f"Read the player's `instructions` alone: do they forbid {m.group(2)} to send {m.group(1)} of its soldiers as a group of their own to hunt {m.group(3)}?"} if m else v
    return out

def preamble_in_state(q):
    out = {}
    for k, v in q.items():
        m = PICK_HEAD.match(v["instructions"])
        out[k] = {**v, "instructions": "Given `economy`, `ours`, `enemy`, `actors`, `player`, the player's `instructions` and `choosing`, which plan is best this second? " + v["instructions"][m.end():]} if m else v
    return out

def choosing(state, q, orig):
    for v in orig.values():
        m = PICK_HEAD.match(v["instructions"])
        if m: return {**state, "choosing": m.group(1)}
    return state

def no_roving(state, q, orig):
    return {**state, "actors": {a: v for a, v in (state.get("actors") or {}).items() if "roving, in code" not in json.dumps(v)}}

def asked_actors(state, q, orig):
    text = json.dumps(q)
    return {**state, "actors": {a: v for a, v in (state.get("actors") or {}).items() if named(text, a)}}

def no_buildings(state, q, orig):
    return {**state, "enemy": {k: v for k, v in (state.get("enemy") or {}).items() if k != "buildings_seen"}}

def no_scouting(state, q, orig):
    ours = dict(state.get("ours") or {})
    if isinstance(ours.get("resurrection_bots"), str): ours["resurrection_bots"] = re.sub(r" \(`produce` armrectr: [^)]*\)", "", ours["resurrection_bots"])
    return {**state, "ours": ours, "enemy": {k: v for k, v in (state.get("enemy") or {}).items() if k not in ("never_looked", "looked_long_ago", "start_box")}}

def rules_by_kind(state, q, orig):
    # A gate's questions are keyed by their actor; a pick's worlds name their actors in the options' words.
    rules = state.get("rules", ""); text = json.dumps(q) if is_pick(q) else " ".join(q)
    for name, (pattern, kinds) in RULE_PARTS.items():
        if not pattern.search(rules): sys.exit(f"the rules' {name} part is not where this script looks for it")
        if not any(re.search(r"(?<![A-Za-z0-9_])" + kind, text) for kind in kinds): rules = pattern.sub("", rules)
    if state.get("allies") == NO_ALLIES:
        if not ALLIES.search(rules): sys.exit("the rules' allies part is not where this script looks for it")
        rules = ALLIES.sub("", rules)
    return {**state, "rules": rules}

def placebo_order(state, q, orig):
    """The same picture with its sections in another order: what a change that removes nothing does to the answers."""
    return dict(reversed(list(state.items())))

def bot_order(state, q, orig):
    """The picture's sections in the order the bot sends them (its JSON maps are sorted by key)."""
    return dict(sorted(state.items()))

def placebo_words(state, q, orig):
    """The same picture with one line added that bears on nothing asked."""
    return {**state, "map": "Comet Catcher Remake 1.8, 8192 by 8192"}

def no_rules(state, q, orig):
    return {k: v for k, v in state.items() if k != "rules"}

same = lambda q: q
keep = lambda state, q, orig: state
ARMS = {
    "recorded": (same, keep), "recorded 2": (same, keep), "no given": (no_given, keep), "facts once": (facts_once, keep), "short forbidden": (short_forbidden, keep),
    "preamble in state": (preamble_in_state, choosing), "no roving": (same, no_roving), "asked actors": (same, asked_actors),
    "no buildings": (same, no_buildings), "no scouting": (same, no_scouting), "rules by kind": (same, rules_by_kind), "no rules": (same, no_rules),
    "placebo order": (same, placebo_order), "bot order": (same, bot_order), "placebo words": (same, placebo_words),
}
DEFAULT = {
    "gate": ["recorded", "recorded 2", "no given", "facts once", "short forbidden", "no roving", "asked actors", "no buildings", "no scouting", "rules by kind", "no rules"],
    "pick": ["recorded", "recorded 2", "preamble in state", "no roving", "no buildings", "no scouting", "rules by kind"],
}
def combined(names):
    """An arm of several: `a+b+c` applies each in turn."""
    def qf(q):
        for n in names: q = ARMS[n][0](q)
        return q
    def sf(state, q, orig):
        for n in names: state = ARMS[n][1](state, q, orig)
        return state
    return qf, sf
def arm_of(name):
    return ARMS[name] if name in ARMS else combined(name.split("+"))

def chunks_of(state, q):
    """The request's questions in parts under the client's size, a forbidden question never apart from its move."""
    base = len(json.dumps(state)); chunk = {}; size = base; out = []
    for k, v in q.items():
        L = len(json.dumps(v)) + len(k) + 6
        if chunk and size + L > 140_000 and not k.endswith(".forbidden"):
            out.append(chunk); chunk = {}; size = base
        chunk[k] = v; size += L
    if chunk: out.append(chunk)
    return out

def is_pick(q):
    return any(k.startswith("worlds.") for k in q)

def requests(match, kind):
    from jev_ab import requests_of
    from match_read import Match
    return [(c, s) for c, s, _ in requests_of(Match(match)) if is_pick(c["questions"]) == (kind == "pick")]

def value(a):
    if not isinstance(a, dict): return None
    return a.get("noul") if a.get("noul") is not None else a.get("choice")

def ask_arms(match, kind, n, out, arms, more=False):
    from jev_ab import api_key, ask
    key = api_key()
    rs = requests(match, kind)
    random.seed(11); random.shuffle(rs); rs = rs[:n]
    def run(job):
        c, state, name = job
        qf, sf = arm_of(name)
        q = qf(c["questions"]); answers = {}; tokens = 0
        for part in chunks_of(state, q):
            r = ask(key, sf(state, part, c["questions"]), part, c.get("model", "jev-latest"))
            answers.update(r.get("answers") or {}); tokens += (r.get("usage") or {}).get("input_tokens", 0)
        return {"f": c["f"], "kind": kind, "arm": name, "tokens": tokens, "answers": {k: value(a) for k, a in answers.items()}}
    with open(out, "a" if more else "w") as f, concurrent.futures.ThreadPoolExecutor(8) as ex:
        for c, _ in ([] if more else rs):
            f.write(json.dumps({"f": c["f"], "kind": kind, "arm": "as played", "tokens": (c.get("usage") or {}).get("input_tokens", 0), "answers": {k: value(a) for k, a in c["answers"].items()}}) + "\n")
        for row in ex.map(run, [(c, s, a) for c, s in rs for a in arms]):
            f.write(json.dumps(row) + "\n"); f.flush()

def family(k):
    if k.endswith(".forbidden"): return "forbidden"
    if k.endswith(".change") or k.endswith(".answer"): return "opener"
    if k.startswith("worlds."): return "pick: which change" if k.endswith(".pick") else "pick: change or not"
    return "move"

def crossed(k, a, b):
    """Whether the two answers fall on different sides of what the code does with them."""
    if isinstance(a, str) or isinstance(b, str): return a != b
    bar = FORBIDDEN if k.endswith(".forbidden") else FLAG
    return (a >= bar) != (b >= bar)

def reference(rows):
    """What the arms are read against and what gives the noise: two fresh asks when the file has them (the answers
    as played were given hours before, and the forbidden answers have drifted up since), else the played answers."""
    return ("recorded", "recorded 2") if any("recorded 2" in r for r in rows) else ("as played", "recorded")

def read(paths):
    rows = collections.defaultdict(dict)
    for p in paths:
        for line in open(p):
            r = json.loads(line); rows[(p, r["f"])][r["arm"]] = r
    ref, floor = reference(rows.values())
    arms = [a for a in dict.fromkeys(a for r in rows.values() for a in r) if a not in ("as played", "recorded", "recorded 2")]
    played_tokens = sum(r["recorded"]["tokens"] for r in rows.values() if "recorded" in r)
    print(f"{len(rows)} requests; asked again as recorded {played_tokens/1e6:.2f}M tokens; every arm is read against `{ref}`, and `again` is `{floor}` against it")
    rng = random.Random(5)
    for fam in ("opener", "move", "forbidden", "pick: which change", "pick: change or not"):
        def per_request(arm):
            """For each request: (answers compared, crossings with the arm, crossings asked again, sum |diff| arm, sum |diff| again, up, down)."""
            out = []
            for r in rows.values():
                if arm not in r or ref not in r or floor not in r: continue
                t = [0, 0, 0, 0.0, 0.0, 0, 0]
                for k, a in r[ref]["answers"].items():
                    if family(k) != fam or a is None: continue
                    b = r[arm]["answers"].get(k); again = r[floor]["answers"].get(k)
                    if b is None or again is None: continue
                    t[0] += 1; t[1] += crossed(k, a, b); t[2] += crossed(k, a, again)
                    if not isinstance(a, str) and not isinstance(b, str) and not isinstance(again, str):
                        t[3] += abs(b - a); t[4] += abs(again - a)
                        bar = FORBIDDEN if fam == "forbidden" else FLAG
                        t[5] += a < bar <= b; t[6] += b < bar <= a
                out.append(t)
            return out
        lines = []
        for arm in arms:
            per = per_request(arm); n = sum(t[0] for t in per)
            if not n: continue
            diffs = []
            for _ in range(2000):
                s = [per[rng.randrange(len(per))] for _ in per]; m = sum(t[0] for t in s)
                if m: diffs.append(100 * (sum(t[1] for t in s) - sum(t[2] for t in s)) / m)
            diffs.sort(); lo, hi = diffs[int(0.025 * len(diffs))], diffs[int(0.975 * len(diffs)) - 1]
            x, again = sum(t[1] for t in per), sum(t[2] for t in per)
            mark = "  MOVED" if lo > 0 or hi < 0 else ""
            numeric = f"; mean |diff| {sum(t[3] for t in per)/n:.3f} (again {sum(t[4] for t in per)/n:.3f}); up {sum(t[5] for t in per)}, down {sum(t[6] for t in per)}" if not fam.startswith("pick") else ""
            tokens = sum(r[arm]["tokens"] for r in rows.values() if arm in r)
            lines.append(f"  {arm:34s} tokens {100*tokens/max(1,played_tokens):5.1f}%  answers {n:6d}: other side {x:4d} ({100*x/n:.2f}%), again {again:4d} ({100*again/n:.2f}%), difference {100*(x-again)/n:+.2f} points (95% {lo:+.2f} to {hi:+.2f}){numeric}{mark}")
        if lines: print(fam); print("\n".join(lines))

def sizes(match, kind):
    rs = requests(match, kind)
    total = 0; cut = collections.Counter()
    for c, state in rs:
        full = sum(len(json.dumps({"state": state, "questions": part})) for part in chunks_of(state, c["questions"]))
        total += full
        for name in DEFAULT[kind][1:]:
            qf, sf = arm_of(name); q = qf(c["questions"])
            cut[name] += full - sum(len(json.dumps({"state": sf(state, part, c["questions"]), "questions": part})) for part in chunks_of(state, q))
    print(f"{len(rs)} {kind} requests, {total/1e6:.1f}M chars")
    for name, v in cut.most_common(): print(f"  {name:20s} {v/1e6:6.2f}M  {100*v/total:5.1f}%")

def show(match, out, arm, n):
    rows = collections.defaultdict(dict)
    for line in open(out):
        r = json.loads(line); rows[r["f"]][r["arm"]] = r
    kind = next(iter(rows.values()))["as played"]["kind"]
    by_f = {c["f"]: (c, s) for c, s in requests(match, kind)}
    qf, _ = arm_of(arm); shown = 0
    ref, floor = reference(rows.values())
    for f, r in rows.items():
        if arm not in r: continue
        c, state = by_f[f]; q = qf(c["questions"])
        for k, a in r[ref]["answers"].items():
            b = r[arm]["answers"].get(k); again = r[floor]["answers"].get(k)
            if a is None or b is None or again is None or not crossed(k, a, b) or crossed(k, a, again): continue
            print(f"## {state['clock']} (f {f}) {k}: {ref} {a}, {floor} {again}, {arm} {b}")
            print(f"as played: {c['questions'][k]['instructions']}")
            if q[k]["instructions"] != c["questions"][k]["instructions"]: print(f"{arm}: {q[k]['instructions']}")
            print(); shown += 1
            if shown >= n: return

if __name__ == "__main__":
    mode = sys.argv[1]
    if mode in ("ask", "more"):
        kind = sys.argv[3]
        ask_arms(sys.argv[2], kind, int(sys.argv[4]), sys.argv[5], sys.argv[6].split(",") if len(sys.argv) > 6 else DEFAULT[kind], mode == "more")
        read([sys.argv[5]])
    elif mode == "read": read(sys.argv[2:])
    elif mode == "sizes": sizes(sys.argv[2], sys.argv[3])
    elif mode == "show": show(sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5]) if len(sys.argv) > 5 else 20)
