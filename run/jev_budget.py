#!/usr/bin/env python3
"""Offline: where a game's Jev tokens went, and what each way of asking less would have saved, counted on the
recorded calls (no Jev calls). Tokens are estimated from characters with the rates measured against the service on
2026-10-01 (state 0.34 a character, gate questions 0.28, world lines 0.34, 280 a request).

The levers, applied one after another to the calls as recorded:
  forbidden  the per-hunt "do the instructions forbid this" question asked once per packet per group instead
  reading    a group's walks and advances only to the places Jev read as its own (--reading <file from jev_read_ab.py>)
  openers    an actor's move questions only when its `change` was 0.5 or over at its last ask (or it is idle); a
             party's only when its `answer` was
  unchanged  a question whose words (clocks taken out) are those of its last ask within 20 s is not sent; a gate
             with nothing left to send is not sent
  split pick the pick asked per component in one request (run/jev_split_ab.py: 0.58 of the two calls' tokens)
  numbers    as `unchanged`, the numbers in the words taken out too: the ceiling of what lines written in word
             buckets ("far", "about 20 s") would save; answers to such questions crossed 0.5 in 1 to 3%
  two s      a gate with no event that second (a hit, an arrival, a party appearing, new orders) waits until 2 s
             have passed since the last gate; what it would have asked goes with the next one; picks scale with gates
  rules      the standing rules text taken out of every call's state (1,019 tokens); it moves answers
             (run/jev_trim_ab.py), so it is a decision about play, not a free saving

With --rebuilt the groups' and parties' questions are replaced by a model of the rebuilt menu
(docs/design/2026-10-01-hands-rebuild.md) under the same levers: for each group the gate asked, when open, a walk
and an advance to each place read as its own, a join to every other group, hold, gather and scout; an opener for
every actor asked and for every party in the picture; for each party whose recorded `answer` was 0.5 or over, every
group's attack and its detachments of 1, 2, 4, 8 (or --ladder N sizes). Question lengths are the recorded ones.

usage: run/jev_budget.py run/matches/<batch>/<NN> [--reading <file.jsonl>] [--by-minute] [--rebuilt] [--ladder N]"""
import collections, json, os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import requests_of
from match_read import Match

STATE, QUESTION, LINES, REQUEST = 0.34, 0.28, 0.34, 280
RE_ASK = 600


def kind(k):
    if k.endswith((".change", ".answer")):
        return "openers"
    if k.endswith(".forbidden"):
        return "forbidden"
    if ".walk_" in k or ".advance_" in k:
        return "group walks"
    if k.startswith("party_"):
        return "party answers"
    if k.startswith("group_"):
        return "group other"
    if k.startswith(("plant_", "lab_", "factory_")):
        return "factory"
    return "builder"


def strip_numbers(t):
    return re.sub(r"(?<![a-z_])\d+(\.\d+)?", "#", t)


def strip_clocks(t):
    t = re.sub(r"for \d+ s\b", "for N s", t)
    t = re.sub(r"\d+ s ago", "N s ago", t)
    t = re.sub(r"since \d+ s", "since N s", t)
    return re.sub(r"\d+:\d\d", "M:SS", t)


def main():
    args = sys.argv[1:]
    reading = {}
    if "--reading" in args:
        i = args.index("--reading")
        for line in open(args[i + 1]):
            r = json.loads(line)
            if r.get("noul") is not None and "actor" in r:
                reading.setdefault(r["f"], {}).setdefault(r["actor"], set())
                if r["noul"] >= 0.5:
                    reading[r["f"]][r["actor"]].add(r["place"])
        del args[i:i + 2]
    by_minute = "--by-minute" in args
    rebuilt = "--rebuilt" in args
    ladder_n = 4
    if "--ladder" in args:
        i = args.index("--ladder")
        ladder_n = int(args[i + 1])
        del args[i:i + 2]
    args = [a for a in args if not a.startswith("--")]
    read_frames = sorted(reading)
    for match in args:
        m = Match(match)
        calls = [(c, s) for c, s, _ in requests_of(m)]
        minutes = max(c["f"] for c, _ in calls) / 1800
        recorded = sum((c.get("usage") or {}).get("input_tokens", 0) for c, _ in calls)
        levers = ["as recorded", "forbidden", "reading", "openers", "unchanged", "split pick", "numbers", "two s", "rules"]
        events_at = {c["f"]: c.get("events") or [] for c in m.calls if c.get("t") == "pass"}
        n_gates = sum(1 for c, _ in calls if "worlds.pick" not in c["questions"])
        pending = collections.defaultdict(dict)
        last_fired = -10_000
        fired = 0
        last_loose = {}
        total = {l: collections.Counter() for l in levers}
        last_words, last_open = {}, {}
        minute_tokens = collections.Counter()
        for c, state in calls:
            f = c["f"]
            b = c.get("batches") or 1
            state_chars = len(json.dumps(state))
            rules_chars = len(json.dumps(state.get("rules", "")))
            if "worlds.pick" in c["questions"]:
                q = c["questions"]["worlds.pick"]
                lines = len(json.dumps(q))
                named = set(re.findall(r"(?:group_\w+|constructor_\d+|commander\w*|plant_\d+|lab_\d+|factory_\d+)", json.dumps(q["criteria"])))
                actors = state.get("actors", {})
                slim = sum(len(json.dumps(v)) + len(k) + 4 for k, v in actors.items() if k in named) + sum(len(json.dumps(state.get(k, ""))) for k in ("enemy", "instructions", "economy", "ours", "player", "clock"))
                for l in levers:
                    s = state_chars - (rules_chars if l == "rules" else 0)
                    f_split = 0.58 if levers.index(l) >= levers.index("split pick") else 1.0
                    total[l]["pick state"] += s * STATE * f_split
                    total[l]["world lines"] += lines * LINES * f_split
                    total[l]["requests"] += REQUEST * f_split
                minute_tokens[f // 1800] += (c.get("usage") or {}).get("input_tokens", 0)
                continue
            minute_tokens[f // 1800] += (c.get("usage") or {}).get("input_tokens", 0)
            answers = c["answers"]
            at = max((x for x in read_frames if x <= f), default=None)
            sent = {l: collections.Counter() for l in levers}
            for k, q in c["questions"].items():
                n = len(json.dumps(q)) + len(k) + 8
                kd = kind(k)
                actor = k.split(".")[0]
                keep = True
                for l in levers:
                    if l == "forbidden" and kd == "forbidden":
                        keep = False
                    if l == "reading" and kd == "group walks" and reading and keep:
                        place = k.split(".", 1)[1].split("_", 1)[1]
                        mine = reading.get(at, {}).get(actor)
                        if mine is not None and place not in mine:
                            keep = False
                    if l == "openers" and keep and kd not in ("openers",):
                        opener = f"{actor}.change" if not actor.startswith("party_") else f"{actor}.answer"
                        was = last_open.get(opener)
                        if was is not None and f - was[0] <= RE_ASK and was[1] < 0.5:
                            keep = False
                    if l == "unchanged" and keep:
                        w = last_words.get(k)
                        if w is not None and f - w[0] <= RE_ASK and w[1] == strip_clocks(q["instructions"]):
                            keep = False
                    if l == "numbers" and keep:
                        w = last_loose.get(k)
                        if w is not None and f - w[0] <= RE_ASK and w[1] == strip_numbers(q["instructions"]):
                            keep = False
                    if keep:
                        sent[l][kd] += n
            if rebuilt:
                actors_now = state.get("actors", {})
                groups_now = [g for g in actors_now if g.startswith("group_")]
                parties_now = set(re.findall(r"party_\d+", json.dumps(state.get("enemy", {})) + json.dumps(actors_now)))
                asked_groups = {k.split(".")[0] for k in c["questions"] if k.startswith("group_")} | {x.group(1) for k in c["questions"] if k.startswith("party_") for x in [re.search(r"_(group_\w+?)(\.forbidden)?$", k)] if x}
                asked_groups &= set(groups_now)
                def size_of(g):
                    e = actors_now[g]
                    mm = re.search(r"\((\d+) soldiers?", e.get("units", "") if isinstance(e, dict) else str(e))
                    return int(mm.group(1)) if mm else 1
                def is_open(g, gated):
                    if not gated:
                        return True
                    a = (answers.get(f"{g}.change") or {}).get("noul")
                    was = last_open.get(f"{g}.change")
                    return a is None or (was is None) or was[1] >= 0.5
                def party_open(p, gated):
                    was = last_open.get(f"{p}.answer")
                    now = (answers.get(f"{p}.answer") or {}).get("noul")
                    if not gated:
                        return True
                    return (was is not None and was[1] >= 0.5) or (was is None and now is not None and now >= 0.5)
                for l in levers:
                    i = levers.index(l)
                    gated, cached, read = i >= levers.index("openers"), i >= levers.index("unchanged"), i >= levers.index("reading") and bool(reading)
                    for kd in ("group walks", "group other", "party answers", "forbidden"):
                        sent[l].pop(kd, None)
                    op = [k for k in c["questions"] if k.endswith(".change") and not k.startswith("group_")]
                    sent[l]["openers"] = (len(op) + len(asked_groups) + len(parties_now)) * 555 * (0.72 if cached else 1.0)
                    for g in asked_groups:
                        if not is_open(g, gated):
                            continue
                        mine = reading.get(at, {}).get(g) if read else None
                        n_places = len(mine) if mine is not None else (3 if read else len(set(re.findall(r"spot_\d+|passage_\d+|\bhome\b", state.get("instructions", "")))))
                        sent[l]["group walks"] += n_places * 2 * 300 * (0.6 if cached else 1.0)
                        sent[l]["group other"] += ((len(groups_now) - 1) * 566 + 3 * 470) * (0.75 if cached else 1.0)
                    for p in parties_now:
                        if not party_open(p, gated):
                            continue
                        for g in groups_now:
                            steps = sum(1 for sz in (1, 2, 4, 8)[:ladder_n] if sz < size_of(g))
                            sent[l]["party answers"] += 607 + steps * 499
                            if i < levers.index("forbidden") + 0 and False:
                                pass
                            if l == "as recorded":
                                sent[l]["forbidden"] += steps * 412
            for k, q in c["questions"].items():
                w = last_words.get(k)
                text = strip_clocks(q["instructions"])
                if w is None or w[1] != text or f - w[0] > RE_ASK:
                    last_words[k] = (f, text)
                w = last_loose.get(k)
                loose = strip_numbers(q["instructions"])
                if w is None or w[1] != loose or f - w[0] > RE_ASK:
                    last_loose[k] = (f, loose)
                if k.endswith((".change", ".answer")) and (answers.get(k) or {}).get("noul") is not None:
                    last_open[k] = (f, answers[k]["noul"])
            fire = bool(events_at.get(f)) or f - last_fired >= 60
            for l in levers:
                if l in ("two s", "rules"):
                    for kd, n in sent[l].items():
                        pending[l][kd] = pending[l].get(kd, 0) + n * (1.0 if fire or not pending[l].get(kd) else 0.5)
                    if not fire:
                        continue
                    sent[l] = collections.Counter(pending[l])
                    pending[l].clear()
                if not sent[l]:
                    continue  # nothing left to ask: the gate is not sent
                s = state_chars * b - (rules_chars * b if l == "rules" else 0)
                total[l]["gate state"] += s * STATE
                total[l]["requests"] += REQUEST * b
                total[l]["gates"] += 1
                for kd, n in sent[l].items():
                    total[l][kd] += n * QUESTION
            if fire:
                last_fired = f
                fired += 1
        for l in ("two s", "rules"):
            for part in ("pick state", "world lines"):
                total[l][part] *= total[l]["gates"] / max(total["numbers"]["gates"], 1)
        parts = ["gate state", "pick state", "group walks", "builder", "group other", "party answers", "openers", "forbidden", "factory", "world lines", "requests"]
        est = sum(total["as recorded"][p] for p in parts)
        print(f"{match.split('/')[2]}{' with the rebuilt menu modelled' if rebuilt else ''}: {minutes:.1f} min, {recorded / 1e6:.1f}M tokens recorded (${recorded * 0.042 / 1e6:.2f}); $1 a game is {1 / 0.042:.1f}M tokens, $1 for 40 minutes is 0.60M a minute")
        print(f"  {'millions of tokens':16s}" + "".join(f"{p[:11]:>12s}" for p in parts) + f"{'total':>9s}{'$':>7s}{'gates':>7s}{'M/min':>7s}")
        for l in levers:
            t = sum(total[l][p] for p in parts)
            print(f"  {('+ ' if l != 'as recorded' else '') + l:16s}" + "".join(f"{total[l][p] / 1e6:12.2f}" for p in parts) + f"{t / 1e6:9.1f}{t * 0.042 / 1e6:7.2f}{total[l]['gates']:7d}{t / 1e6 / minutes:7.2f}")
        if by_minute:
            print("  tokens a game minute, recorded (M): " + " ".join(f"{minute_tokens[i] / 1e6:.1f}" for i in range(int(minutes) + 1)))


if __name__ == "__main__":
    main()
