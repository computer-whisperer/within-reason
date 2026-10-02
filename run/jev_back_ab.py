#!/usr/bin/env python3
"""Offline: the recorded picks in which a group was sent to "fall back to home from party_N" (the threat slot's way
back), asked again with every world holding that line taken out: where the pick's mass goes, and what stage two then
chooses against "nothing changes". Nothing is played.

usage: run/jev_back_ab.py run/matches/<batch>/<NN> --out <file.jsonl>   then   --read <file.jsonl>
The key is read by run/jev_ab.py's api_key() and never printed or written. Jev: $0.042 a million input tokens."""
import argparse, collections, concurrent.futures, copy, json, os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from match_read import Match


def kind_of(line, group):
    """What a world's line has `group` do."""
    parts = [p for p in re.split(r"; (?=group_|constructor_|commander|\d+ arm|plant_)", line.replace("As w1, and: ", "")) if group in p]
    if not parts:
        return "another actor's move"
    p = parts[0]
    for words, kind in (("falls back to home from", "back home (the threat's)"), ("falls back to our base", "retreat home (its own)"), ("with the whole group", "attack with the whole group"), (" hunt ", "a detachment hunts"),
                        ("falls back to where it last held", "fall back to where it last held"), ("pulls out", "pull out of a shooter's reach"), ("gathers", "gather"), ("walks to", "walk to a place"), ("advances to", "advance to a place"),
                        ("shells", "shell from a standoff"), ("closes on the shooter", "close on a shooter"), ("merges into", "join"), ("stands at", "stand at the next extractor"), ("raids", "raid")):
        if words in p:
            return kind
    return "other"


def run(args):
    key = api_key()
    m = Match(args.match)
    plans = {c["f"]: c for c in m.calls if c.get("t") == "plan"}
    picks = collections.defaultdict(list)
    for c, state, _ in requests_of(m):
        if "worlds.pick" in c["questions"]:
            picks[c["f"]].append((c, state))
    jobs = []
    for f, plan in sorted(plans.items()):
        for p in plan.get("played") or []:
            if ".back_group_" not in (p.get("played") or ""):
                continue
            group = re.search(r"group_\w+", p["played"]).group(0)
            one = [(c, s) for c, s in picks[f] if "w1" not in c["questions"]["worlds.pick"]["criteria"]]
            two = [(c, s) for c, s in picks[f] if "w1" in c["questions"]["worlds.pick"]["criteria"]]
            if one and two:
                jobs.append((f, group, one[0], two[0]))
    print(f"{len(jobs)} picks", file=sys.stderr)

    def one_job(job):
        f, group, (c1, state), (c2, _) = job
        crit = c1["questions"]["worlds.pick"]["criteria"]
        recorded = c1["answers"]["worlds.pick"].get("probabilities") or {}
        gone = {w for w, line in crit.items() if f"{group} falls back to home from" in line}
        left = {w: line for w, line in crit.items() if w not in gone}
        row = {"f": f, "group": group, "worlds": len(crit), "removed": len(gone), "recorded_back_mass": sum(recorded.get(w, 0) for w in gone),
               "recorded": collections.Counter(), "tokens": 0}
        for w, pr in recorded.items():
            row["recorded"][kind_of(crit[w], group)] += pr
        if len(left) < 1:
            row["after"] = {}
            row["choice"] = "nothing changes (no other world)"
            return row
        q1 = copy.deepcopy(c1["questions"]["worlds.pick"])
        q1["criteria"] = left
        a1 = ask(key, state, {"worlds.pick": q1}, c1["model"]) if len(left) > 1 else {"answers": {"worlds.pick": {"choice": next(iter(left)), "probabilities": {next(iter(left)): 1.0}}}}
        row["tokens"] += (a1.get("usage") or {}).get("input_tokens", 0)
        probs = a1["answers"]["worlds.pick"].get("probabilities") or {}
        after = collections.Counter()
        for w, pr in probs.items():
            after[kind_of(left[w], group)] += pr
        row["after"] = dict(after)
        top = max(probs, key=probs.get)
        q2 = copy.deepcopy(c2["questions"]["worlds.pick"])
        w1 = q2["criteria"]["w1"]
        q2["criteria"] = {"w1": w1, top: left[top]}
        a2 = ask(key, state, {"worlds.pick": q2}, c2["model"])
        row["tokens"] += (a2.get("usage") or {}).get("input_tokens", 0)
        p2 = a2["answers"]["worlds.pick"].get("probabilities") or {}
        row["top"] = kind_of(left[top], group)
        row["top_line"] = left[top][:300]
        row["w1"] = w1[:200]
        row["choice"] = "nothing changes" if p2.get("w1", 0) >= p2.get(top, 0) else row["top"]
        row["p_candidate"] = p2.get(top, 0)
        row["recorded"] = dict(row["recorded"])
        return row

    with open(args.out, "w") as out, concurrent.futures.ThreadPoolExecutor(8) as pool:
        tokens = 0
        for row in pool.map(one_job, jobs):
            row["recorded"] = dict(row["recorded"])
            tokens += row.pop("tokens")
            out.write(json.dumps(row) + "\n")
    print(f"{tokens} input tokens, ${tokens * 0.042 / 1e6:.3f}", file=sys.stderr)


def read(args):
    rows = [json.loads(line) for line in open(args.read)]
    clock = lambda f: f"{f // 1800}:{f // 30 % 60:02d}"
    print(f"{len(rows)} picks; recorded mass on the fall back to home: median {sorted(r['recorded_back_mass'] for r in rows)[len(rows) // 2]:.2f}")
    after, top, choice = collections.Counter(), collections.Counter(), collections.Counter()
    for r in rows:
        for k, v in (r.get("after") or {}).items():
            after[k] += v / len(rows)
        top[r.get("top", "none")] += 1
        choice[r["choice"]] += 1
    print("stage one without it, the mean mass by what the group does:")
    for k, v in after.most_common():
        print(f"  {v:.2f} {k}")
    print("stage one's top world:", dict(top.most_common()))
    print("stage two's choice against 'nothing changes':", dict(choice.most_common()))
    for r in rows:
        print(clock(r["f"]), r["group"], f"back mass {r['recorded_back_mass']:.2f} ->", r["choice"], f"({r.get('p_candidate', 0):.2f})", "|", r.get("top_line", "")[:150])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("match", nargs="?")
    ap.add_argument("--out")
    ap.add_argument("--read")
    args = ap.parse_args()
    if args.read:
        read(args)
    elif args.match and args.out:
        run(args)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
