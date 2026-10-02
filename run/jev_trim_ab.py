#!/usr/bin/env python3
"""Offline: do Jev's answers move when a recorded call is sent with less? Sampled gates and picks of a game are asked
again under each arm and set beside the recorded answers; `same` (the call as recorded) is the noise floor.

Gate arms:
  same         as recorded
  no_rules     the standing rules text taken out of the state
  short_rules  the rules text cut to what the words mean (SHORT_RULES below), the judgments about how to play left out
  bare         every move question cut to its actor and its move ("group_A: <the move>"; a party's: "against party_7:
               <the move>"), the common preamble said once in the state's `asking` line
Pick arms:
  same, no_rules, and
  slim         the state cut to the actors the worlds name, `enemy`, `instructions`, `economy`, `ours`, `player`

usage: run/jev_trim_ab.py run/matches/<batch>/<NN> --out <file.jsonl> [--gates N] [--picks N] [--arms a,b,...]
       run/jev_trim_ab.py --read <file.jsonl>
The key is read by run/jev_ab.py's api_key() and never printed or written. Jev: $0.042 a million input tokens."""
import argparse, collections, concurrent.futures, copy, json, os, re, statistics, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from match_read import Match

SHORT_RULES = ("We win by killing the enemy commander and lose when ours dies. Extractors on metal spots are the income. Places are named: home, spot_N, passage_N and the player's own names; "
               "where the enemy is comes only from what our units have seen. `shelling` is only where a weapon hitting us from out of sight likeliest stands. The words beside the numbers say what matters. "
               "A group advancing fights through everything on the way; a group moving without fighting runs from everything. The instructions were written before what the picture shows now; "
               "the `player` line says when its last orders arrived.")
ASKING = ("Each question below that names an actor and a move without more asks: is that what the actor should do now, rather than what its entry under `actors` says it is doing? "
          "A question that begins \"against party_N\" asks whether the move is the one to make against that party now, rather than what stands. "
          "Judge it from that actor's entry under `actors`, `economy`, `ours`, `enemy` and the player's `instructions`.")


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


def bare(k, q):
    """A move question cut to its actor and its move."""
    text = q["instructions"]
    if k.endswith((".change", ".answer", ".forbidden")):
        return q
    m = re.search(r"The move: (.*)$", text, re.S)
    if not m:
        m2 = re.match(r"^(\w+), rather than .*?: (\1 .*)$", text, re.S)
        return {"type": "noul", "instructions": m2.group(2)} if m2 else q
    actor = k.split(".")[0]
    return {"type": "noul", "instructions": (f"against {actor}: " if actor.startswith("party_") else "") + m.group(1)}


def gate_arm(arm, state, questions):
    state = copy.deepcopy(state)
    if arm == "no_rules":
        state.pop("rules", None)
    elif arm == "short_rules":
        state["rules"] = SHORT_RULES
    elif arm == "bare":
        state["asking"] = ASKING
        questions = {k: bare(k, q) for k, q in questions.items()}
    return state, questions


def pick_arm(arm, state, q):
    state = copy.deepcopy(state)
    if arm in ("no_rules", "slim"):
        state.pop("rules", None)
    if arm == "slim":
        named = set(re.findall(r"(?:group_\w+|constructor_\d+|commander\w*|plant_\d+|lab_\d+|factory_\d+)", json.dumps(q["criteria"])))
        state = {k: v for k, v in state.items() if k in ("actors", "enemy", "instructions", "economy", "ours", "player", "clock")}
        state["actors"] = {k: v for k, v in state.get("actors", {}).items() if k in named}
    return state


def run(args):
    key = api_key()
    m = Match(args.match)
    gates, ones, twos = [], [], []
    for c, state, _ in requests_of(m):
        if "worlds.pick" not in c["questions"]:
            if (c.get("batches") or 1) == 1 and len(c["questions"]) >= 20:
                gates.append((c, state))
        elif "w1" in c["questions"]["worlds.pick"]["criteria"]:
            twos.append((c, state))
        elif len(c["questions"]["worlds.pick"]["criteria"]) >= 3:
            ones.append((c, state))
    take = lambda xs, n: xs[:: max(1, len(xs) // n)][:n]
    arms = args.arms.split(",")
    jobs = [("gate", a, c, s) for c, s in take(gates, args.gates) for a in arms if a in ("same", "no_rules", "short_rules", "bare")]
    jobs += [("one", a, c, s) for c, s in take(ones, args.picks) for a in arms if a in ("same", "no_rules", "slim")]
    jobs += [("two", a, c, s) for c, s in take(twos, args.picks) for a in arms if a in ("same", "no_rules", "slim")]
    print(f"{len(jobs)} calls", file=sys.stderr)

    def one(job):
        what, arm, c, state = job
        if what == "gate":
            st, qs = gate_arm(arm, state, c["questions"])
            a = ask(key, st, qs, c["model"])
            rows = [{"what": what, "arm": arm, "f": c["f"], "key": k, "kind": kind(k), "was": (c["answers"].get(k) or {}).get("noul"), "now": v.get("noul")} for k, v in (a.get("answers") or {}).items()]
        else:
            q = c["questions"]["worlds.pick"]
            a = ask(key, pick_arm(arm, state, q), {"worlds.pick": q}, c["model"])
            was, now = c["answers"]["worlds.pick"], a["answers"]["worlds.pick"]
            rows = [{"what": what, "arm": arm, "f": c["f"], "worlds": len(q["criteria"]), "was": was.get("choice"), "now": now.get("choice"), "was_p": was.get("probabilities"), "now_p": now.get("probabilities")}]
        return rows, (a.get("usage") or {}).get("input_tokens", 0), (c.get("usage") or {}).get("input_tokens", 0), what, arm

    tokens = 0
    used = collections.defaultdict(lambda: [0, 0])
    with open(args.out, "w") as out, concurrent.futures.ThreadPoolExecutor(6) as pool:
        for rows, t, was, what, arm in pool.map(one, jobs):
            tokens += t
            used[(what, arm)][0] += t
            used[(what, arm)][1] += was
            for r in rows:
                out.write(json.dumps(r) + "\n")
        for (what, arm), (t, was) in sorted(used.items()):
            out.write(json.dumps({"what": what, "arm": arm, "tokens": t, "recorded_tokens": was}) + "\n")
    print(f"{tokens} input tokens, ${tokens * 0.042 / 1e6:.3f}", file=sys.stderr)


def read(args):
    rows = [json.loads(line) for line in open(args.read)]
    used = {(r["what"], r["arm"]): r for r in rows if "tokens" in r}
    rows = [r for r in rows if "tokens" not in r]
    print("gates: an arm's nouls against the recorded ones")
    print(f"  {'arm':12s} {'kind':14s} {'n':>6s} {'median change':>14s} {'p90':>6s} {'crossed 0.5':>12s} {'tokens against recorded':>24s}")
    for arm in ("same", "no_rules", "short_rules", "bare"):
        rs = [r for r in rows if r["what"] == "gate" and r["arm"] == arm and r["was"] is not None and r["now"] is not None]
        if not rs:
            continue
        u = used.get(("gate", arm))
        for kd in ["all"] + sorted({r["kind"] for r in rs}):
            xs = [r for r in rs if kd == "all" or r["kind"] == kd]
            d = sorted(abs(r["now"] - r["was"]) for r in xs)
            cross = sum((r["now"] >= 0.5) != (r["was"] >= 0.5) for r in xs)
            print(f"  {arm:12s} {kd:14s} {len(xs):6d} {d[len(d) // 2]:14.3f} {d[len(d) * 9 // 10]:6.3f} {100 * cross / len(xs):11.1f}% " + (f"{100 * u['tokens'] / u['recorded_tokens']:22.0f}%" if u and kd == "all" else ""))
        # the best move of each asked actor: does it stay the best?
        best = collections.defaultdict(dict)
        for r in rs:
            if r["kind"] in ("openers", "forbidden"):
                continue
            actor = r["key"].split(".")[0]
            b = best[(r["f"], actor)]
            for side in ("was", "now"):
                if side not in b or r[side] > b[side][0]:
                    b[side] = (r[side], r["key"])
        agree = sum(b["was"][1] == b["now"][1] for b in best.values())
        strong = [b for b in best.values() if b["was"][0] >= 0.5]
        print(f"  {arm:12s} each actor's best-rated move is the same one in {agree} of {len(best)}; where it was 0.5 or over, {sum(b['was'][1] == b['now'][1] for b in strong)} of {len(strong)}, and still 0.5 or over in {sum(b['now'][0] >= 0.5 for b in strong)}")
    for what, title in (("one", "pick, stage one (the best of the changes)"), ("two", "pick, stage two (the change or nothing)")):
        print(title)
        for arm in ("same", "no_rules", "slim"):
            rs = [r for r in rows if r["what"] == what and r["arm"] == arm]
            if not rs:
                continue
            u = used.get((what, arm))
            tv = sorted(0.5 * sum(abs((r["now_p"] or {}).get(k, 0) - (r["was_p"] or {}).get(k, 0)) for k in set(r["now_p"] or {}) | set(r["was_p"] or {})) for r in rs)
            print(f"  {arm:10s} the same choice in {sum(r['was'] == r['now'] for r in rs)} of {len(rs)}; probability moved (half the sum of changes) median {tv[len(tv) // 2]:.2f}, p90 {tv[len(tv) * 9 // 10]:.2f}; tokens {100 * u['tokens'] / u['recorded_tokens']:.0f}% of recorded")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("match", nargs="?")
    ap.add_argument("--out")
    ap.add_argument("--read")
    ap.add_argument("--gates", type=int, default=50)
    ap.add_argument("--picks", type=int, default=60)
    ap.add_argument("--arms", default="same,no_rules,short_rules,bare,slim")
    args = ap.parse_args()
    if args.read:
        read(args)
    elif args.match and args.out:
        run(args)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
