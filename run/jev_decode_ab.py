#!/usr/bin/env python3
"""Offline: the decode's questions (the `fuse` layer's reading of a packet, `crates/bot/src/brain/pianist/decode.rs`)
asked again in shorter words. The log keeps a decode's readings and not its questions, so every arm is rebuilt from
the code's template and each reading's key, the actor's words taken from the nearest logged picture (`rebuilt` is
the template as the code has it; `short` drops "Read the player's `instructions` alone", the lists of what a place
may be for and "now or under some condition"; `no what` drops the actor's description). The key is read, never printed.

    run/jev_decode_ab.py <match> <decodes> [out.jsonl]
"""
import json, random, sys, collections, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask, requests_of, GLOSSARY
from match_read import Match
OFF, ON = 0.2, 0.8
ARMS = ("rebuilt", "rebuilt 2", "short", "no what")
HEAD = "Read the player's `instructions` alone. "

def question(key, what, arm):
    actor, kind, x = (key.split("|", 2) + [""])[:3]
    w = "" if arm == "no what" or not what else f" ({what})"
    if arm == "short":
        if kind == "place": return f"Do the `instructions` name {x} in what they say to or about {actor}{w}?"
        if kind == "build": return f"Do the `instructions` have {actor}{w} build a {(GLOSSARY.get(x) or {}).get('name', x)} ({x}), or say anything else to it about that kind of building?"
        if kind == "join": return f"Do the `instructions` tell {actor}{w} to join {x}? No when it is {x} they tell to join {actor}."
        if kind == "follow": return f"Do the `instructions` tell {actor}{w} to follow, escort or stay beside {x}?"
        if kind == "detach": return f"Do the `instructions` forbid {actor}{w} to send a detachment of {x} of its soldiers after an enemy party?"
        return f"Do the `instructions` say where {actor}{w} goes when it should not fight?"
    if kind == "place": return HEAD + f"Do they name {x} in anything they say to or about {actor}{w}: a place it goes to, stands at, fights at, builds at, falls back to, or must keep away from?"
    if kind == "build": return HEAD + f"Do they have {actor}{w} build a {(GLOSSARY.get(x) or {}).get('name', x)} ({x}), now or under some condition, or say anything else to it about that kind of building?"
    if kind == "join": return HEAD + f"Do they tell {actor}{w} to join {x}, now or under some condition: {actor} to merge into {x} and take {x}'s name and course? No when it is {x} they tell to join {actor}."
    if kind == "follow": return HEAD + f"Do they tell {actor}{w} to follow, escort or stay beside {x}, now or under some condition?"
    if kind == "detach": return HEAD + f"Do they forbid {actor}{w} to send a detachment of {x} of its soldiers after an enemy party?"
    return HEAD + f"Do they say where {actor}{w} goes when it should not fight: a place it walks to, steps back to or falls back to?"

def main():
    m = Match(sys.argv[1]); n = int(sys.argv[2]); out = open(sys.argv[3], "w") if len(sys.argv) > 3 else None
    key = api_key()
    pictures = [(c["f"], state) for c, state, _ in requests_of(m)]
    decodes = [c for c in m.calls if c.get("t") == "decode" and c.get("reads")]
    random.seed(11); random.shuffle(decodes); decodes = decodes[:n]
    def run(job):
        d, arm = job
        f, state = min(pictures, key=lambda p: abs(p[0] - d["f"]))
        def what(actor):
            e = (state.get("actors") or {}).get(actor)
            if not isinstance(e, dict): return ""
            units = (e.get("units") or e.get("is") or "")[:120]
            parent = (e.get("split_from") or "").split(",")[0]
            return f"{units}; a detachment that left {parent}" if parent else units
        q = {k: {"type": "noul", "instructions": question(k, what(k.split("|")[0]), arm)} for k in d["reads"]}
        st = {"instructions": state.get("instructions", "")}
        answers = {}; tokens = 0; chunk = {}; size = len(json.dumps(st)); parts = []
        for k, v in q.items():
            L = len(json.dumps(v)) + len(k) + 6
            if chunk and size + L > 140_000:
                parts.append(chunk); chunk = {}; size = len(json.dumps(st))
            chunk[k] = v; size += L
        if chunk: parts.append(chunk)
        for part in parts:
            r = ask(key, st, part, "jev-latest")
            answers.update({k: a.get("noul") for k, a in (r.get("answers") or {}).items() if isinstance(a, dict)}); tokens += (r.get("usage") or {}).get("input_tokens", 0)
        return d["f"], arm, answers, tokens
    res = collections.defaultdict(dict); toks = collections.Counter()
    with concurrent.futures.ThreadPoolExecutor(8) as ex:
        for f, arm, answers, tokens in ex.map(run, [(d, a) for d in decodes for a in ARMS]):
            res[f][arm] = answers; toks[arm] += tokens
    played = {d["f"]: d for d in decodes}
    for f, arms in res.items():
        if out: out.write(json.dumps({"f": f, "played": played[f]["reads"], **arms}) + "\n")
    print(f"{len(decodes)} decodes, {sum(len(d['reads']) for d in decodes)} readings; as played {sum(d['usage'].get('input_tokens', 0) for d in decodes)/1e6:.2f}M tokens")
    def side(k, v):
        """What the code does with a reading: fused off under OFF; a detachment's mark on at ON or over."""
        return ("off" if v < OFF else "on" if v >= ON else "asked") if "|detach|" in k else ("off" if v < OFF else "kept")
    for name, ref, arm in [("rebuilt against as played", "played", "rebuilt")] + [(f"{a} against rebuilt", "rebuilt", a) for a in ARMS[1:]]:
        c = collections.Counter(); kinds = collections.Counter()
        for f, arms in res.items():
            a_ = played[f]["reads"] if ref == "played" else arms[ref]
            for k, a in a_.items():
                b = arms[arm].get(k)
                if a is None or b is None: continue
                c["n"] += 1; c["abs"] += abs(a - b)
                if side(k, a) != side(k, b):
                    c["other side"] += 1; c["now off" if b < OFF else "no longer off" if a < OFF else "mark"] += 1; kinds[k.split("|")[1]] += 1
        print(f"  {name:28s} tokens {toks[arm]/1e6:.2f}M ({100*toks[arm]/max(1, toks['rebuilt']):.0f}%): readings {c['n']}, other side {c['other side']} ({100*c['other side']/max(1,c['n']):.2f}%): fused off only here {c['now off']}, fused off only there {c['no longer off']}, a detachment's mark {c['mark']}; mean |diff| {c['abs']/max(1,c['n']):.3f}; by kind {dict(kinds)}")

if __name__ == "__main__":
    main()
