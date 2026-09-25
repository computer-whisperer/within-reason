#!/usr/bin/env python3
"""Replay the worlds questions a game recorded (`{"t":"standing", ...}` lines with `lines` and `worlds`, H-HANDS-WORLDS)
against Jev under a variant of the question's wording or pruning, and compare the picks with the recorded ones.

    run/worlds_replay.py run/matches/<match>/00 [--variant base|no_costs|rules_first|cap4] [--limit N] [--dump]

Variants: `base` the recorded lines as they were; `no_costs` the lines without the ", leaving ..." and ", abandoning
..." clauses; `rules_first` the instructions telling Jev to prefer w1 unless the situation calls otherwise (stronger
words); `cap4` the first four worlds only. Reports, per variant, the share of calls picking the recorded world, the
share picking w1 (the rules), and p(top). The key comes from TYPESAFE_API_KEY or ~/.config/within-reason/jev.env and
is never printed or written.
"""
import json, os, re, sys, time, urllib.error, urllib.request

URL = "https://api.typesafe.ai/v1/systemone"
MODEL = "jev-latest"
KEY_FILE = os.path.expanduser("~/.config/within-reason/jev.env")


def key():
    k = os.environ.get("TYPESAFE_API_KEY")
    if not k and os.path.exists(KEY_FILE):
        for line in open(KEY_FILE):
            if line.startswith("TYPESAFE_API_KEY="):
                k = line.split("=", 1)[1].strip()
    if not k:
        sys.exit(f"no API key: put TYPESAFE_API_KEY=... in {KEY_FILE}")
    return k


def ask(k, state, questions):
    body = {"model": MODEL, "state": state, "questions": questions}
    request = urllib.request.Request(URL, data=json.dumps(body).encode(), method="POST", headers={
        "Authorization": f"Bearer {k}", "Content-Type": "application/json", "User-Agent": "within-reason-worlds-replay"})
    for attempt in range(4):
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                return json.load(response)
        except urllib.error.HTTPError as e:
            if e.code in (429, 500, 502, 503) and attempt < 3:
                time.sleep(2 * (attempt + 1))
                continue
            raise
    return None


def moments(match):
    """(standing line with worlds, the call that asked worlds.pick at the same frame)"""
    standing = {}
    calls = {}
    for line in open(os.path.join(match, "jev-0.jsonl")):
        d = json.loads(line)
        if d.get("t") == "standing" and d.get("lines"):
            standing[d["f"]] = d
        elif d.get("t") == "call" and "worlds.pick" in (d.get("questions") or {}):
            calls[d["f"]] = d
    return [(standing[f], calls[f]) for f in sorted(standing) if f in calls]


def variant_question(q, lines, variant):
    q = json.loads(json.dumps(q))
    if variant == "no_costs":
        lines = [re.sub(r", (leaving|abandoning) [^.;]*", "", l) for l in lines]
    if variant == "rules_first":
        q["instructions"] = q["instructions"].replace("Pick the world", "w1 is the player's own rule and stands unless a party goes unanswered or a group is sent into a fight it loses. Pick the world")
    if variant == "cap4":
        lines = lines[:4]
    q["criteria"] = {f"w{i + 1}": l for i, l in enumerate(lines)}
    return q


def main():
    args = sys.argv[1:]
    match = next(a for a in args if not a.startswith("--"))
    variant = args[args.index("--variant") + 1] if "--variant" in args else "base"
    limit = int(args[args.index("--limit") + 1]) if "--limit" in args else 10 ** 9
    dump = "--dump" in args
    k = key()
    rows = moments(match)[:limit]
    agree = rules = 0
    ptop = []
    for standing, call in rows:
        q = variant_question(call["questions"]["worlds.pick"], standing["lines"], variant)
        recorded = call["answers"]["worlds.pick"]["choice"]
        answer = ask(k, call["state"], {"worlds.pick": q})
        a = (answer or {}).get("answers", {}).get("worlds.pick") or {}
        choice = a.get("choice")
        probs = a.get("probabilities") or {}
        ptop.append(max(probs.values()) if probs else 0.0)
        agree += choice == recorded
        rules += choice == "w1"
        if dump:
            print(f"{standing['f'] // 30 // 60}:{standing['f'] // 30 % 60:02d} recorded {recorded} now {choice} ({ptop[-1]:.2f})")
    n = len(rows)
    print(f"{variant}: {n} calls; agrees with the recorded pick {agree / max(n, 1):.0%}; picks the rules' world {rules / max(n, 1):.0%}; p(top) {sum(ptop) / max(n, 1):.2f}")


if __name__ == "__main__":
    main()
