#!/usr/bin/env python3
"""Offline, the gate's half of run/jev_fallback_ab.py: would a step back reach the pick at all? In the recorded
seconds where a group was sent to "fall back to home from party_N" (and, as controls, where a group attacked a party
it outweighed) the gate is asked a noul for a step back, worded as the rebuild words it
(docs/design/2026-10-01-hands-rebuild.md), in the two frames the gate has today:
  actor  "... is this what group_X should do now, rather than <its course>? The move: ..."
  party  "Is this the move to make against party_N now, rather than <what stands>? The move: ..."
and for three moves:
  named    a walk to a spot, with a sentence in the packet naming it as the group's place to step back to
  unnamed  the same walk with the packet as recorded (the place is somebody else's, or nobody's)
  group    falling back onto our largest other group, the packet as recorded
Each noul is set beside the recorded nouls of the group's other moves that second: would it be among the group's two
best, and at 0.5 or over? Nothing is played.

usage: run/jev_fallback_gate_ab.py run/matches/<batch>/<NN> --out <file.jsonl> [--controls N]
       run/jev_fallback_gate_ab.py --read <file.jsonl>
The key is read by run/jev_ab.py's api_key() and never printed or written. Jev: $0.042 a million input tokens."""
import argparse, collections, concurrent.futures, copy, json, os, random, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from jev_fallback_ab import SENTENCE, dist, place_for, words
from match_read import Match


def run(args):
    key = api_key()
    m = Match(args.match)
    gates = {}
    for c, state, _ in requests_of(m):
        if "worlds.pick" not in c["questions"]:
            gates.setdefault(c["f"], (c, state))
    rows_at = {}
    for c in m.calls:
        if c.get("t") == "call" and c.get("groups") is not None:
            rows_at.setdefault(c["f"], c)
    jobs, controls = [], []
    for c in m.calls:
        if c.get("t") != "plan" or c["f"] not in gates:
            continue
        f = c["f"]
        gate, state = gates[f]
        for p in c.get("played") or []:
            hit = re.match(r"(party_\d+)\.(back|whole)_(group_\w+)", p.get("played") or "")
            if not hit:
                continue
            party, kind, group = hit.group(1), hit.group(2), hit.group(3)
            q = gate["questions"].get(f"{party}.{kind}_{group}")
            if not q:
                continue
            text = q["instructions"]
            if kind == "back":
                facts = re.search(re.escape(f"{group} falls back to home from {party} ") + r"\((.*?)\), \d+ from its front", text)
                said = facts.group(1) if facts else None
            else:
                facts = re.search(re.escape(f"{group} attacks {party} ") + r"\((.*?)\) with the whole group \(.*?: (.*?)\), \d+ from its front", text)
                said = f"{facts.group(1)}; {facts.group(2)}" if facts and "we outweigh it" in facts.group(2) else None
            where = place_for(rows_at.get(f) or {}, group, party)
            stands = re.search(r"now, rather than (.*?)\? The move: ", text, re.S)
            own = next((qq["instructions"] for k, qq in gate["questions"].items() if k.startswith(group + ".") and "rather than" in qq["instructions"]), None)
            course = re.search(r"rather than (.*?)(\? The move: |: " + re.escape(group) + " )", own, re.S) if own else None
            row = rows_at.get(f) or {}
            me = next((x for x in row.get("groups") or [] if "group_" + x["name"] == group), None)
            others = [x for x in row.get("groups") or [] if "group_" + x["name"] != group and x.get("at") and len(x["members"]) >= 2]
            if not said or not where or not stands or not course or not me or not others:
                continue
            friend = max(others, key=lambda x: len(x["members"]))
            (jobs if kind == "back" else controls).append((f, group, party, said, where, stands.group(1), course.group(1), ("group_" + friend["name"], len(friend["members"]), dist(tuple(me["at"]), tuple(friend["at"]))), "back" if kind == "back" else "control"))
    random.Random(1).shuffle(controls)
    jobs += controls[: args.controls]
    print(f"{sum(j[8] == 'back' for j in jobs)} fall-backs to home, {sum(j[8] == 'control' for j in jobs)} controls (of {len(controls)})", file=sys.stderr)

    def one_job(job):
        f, group, party, said, (place, d, d_foe, here), stands, course, (friend, n, d_friend), kind = job
        gate, state = gates[f]
        walk = f"{group} walks to {place} ({words(d)}: {d:.0f} away) without stopping to fight on the way, stepping back from {party} ({said}): {place} is {d_foe:.0f} from {party}, {d_foe - here:.0f} farther than {group} stands now"
        onto = f"{group} falls back onto {friend} ({n} soldiers of ours, {words(d_friend)}: {d_friend:.0f} away) without stopping to fight on the way, stepping back from {party} ({said})"
        actor = lambda move: {"type": "noul", "instructions": f"Given `actors.{group}`, `economy`, `ours` and the player's `instructions`: is this what {group} should do now, rather than {course}? The move: {move}."}
        against = lambda move: {"type": "noul", "instructions": f"Is this the move to make against {party} now, rather than {stands}? The move: {move}."}
        named = copy.deepcopy(state)
        named["instructions"] = state.get("instructions", "") + "\n\n" + SENTENCE.format(group=group, place=place)
        a = ask(key, named, {"named_actor": actor(walk), "named_party": against(walk)}, gate["model"])
        b = ask(key, state, {"unnamed_actor": actor(walk), "unnamed_party": against(walk), "group_actor": actor(onto), "group_party": against(onto)}, gate["model"])
        row = {"f": f, "group": group, "party": party, "kind": kind, "place": place, "friend": friend, "tokens": sum((x.get("usage") or {}).get("input_tokens", 0) for x in (a, b))}
        for x in (a, b):
            for k, v in (x.get("answers") or {}).items():
                row[k] = v.get("noul")
        # The group's other moves that second, as the game rated them.
        rated = {}
        for k, v in gate["answers"].items():
            if v.get("noul") is None or k.endswith((".change", ".answer", ".forbidden", ".told")):
                continue
            if k.startswith(group + ".") or (k.startswith("party_") and k.endswith("_" + group)):
                rated[k] = v["noul"]
        row["others"] = sorted(rated.values(), reverse=True)[:4]
        row["best_other"] = max(rated, key=rated.get) if rated else None
        row["change"] = (gate["answers"].get(f"{group}.change") or {}).get("noul")
        row["answer"] = (gate["answers"].get(f"{party}.answer") or {}).get("noul")
        row["recorded_home"] = (gate["answers"].get(f"{party}.back_{group}") or {}).get("noul")
        return row

    with open(args.out, "w") as out, concurrent.futures.ThreadPoolExecutor(8) as pool:
        tokens = 0
        for row in pool.map(one_job, jobs):
            tokens += row.pop("tokens")
            out.write(json.dumps(row) + "\n")
    print(f"{tokens} input tokens, ${tokens * 0.042 / 1e6:.3f}", file=sys.stderr)


def read(args):
    rows = [json.loads(line) for line in open(args.read)]
    med = lambda xs: sorted(xs)[len(xs) // 2] if xs else float("nan")
    for kind, title in (("back", "the recorded fall-backs to home"), ("control", "controls: the group attacked a party it outweighed")):
        rs = [r for r in rows if r["kind"] == kind]
        if not rs:
            continue
        print(f"{title}: {len(rs)} seconds")
        ch = [r["change"] for r in rs if r["change"] is not None]
        print(f"  recorded: the group's own `change` median {med(ch):.2f} (asked in {len(ch)}, at 0.5 or over {sum(x >= 0.5 for x in ch)}); the party's `answer` median {med([r['answer'] for r in rs if r['answer'] is not None]):.2f}; the walk home's noul median {med([r['recorded_home'] for r in rs if r['recorded_home'] is not None]):.2f}; the group's best other move median {med([r['others'][0] for r in rs if r['others']]):.2f}")
        print(f"  {'move, frame':24s} {'median':>6s} {'>=0.5':>6s} {'among its two best':>19s} {'both':>5s}")
        for key in ("named_actor", "named_party", "unnamed_actor", "unnamed_party", "group_actor", "group_party"):
            v = [r[key] for r in rs if r.get(key) is not None]
            top2 = sum(1 for r in rs if r.get(key) is not None and sum(o > r[key] for o in r["others"]) < 2)
            both = sum(1 for r in rs if r.get(key) is not None and r[key] >= 0.5 and sum(o > r[key] for o in r["others"]) < 2)
            print(f"  {key.replace('_', ', '):24s} {med(v):6.2f} {sum(x >= 0.5 for x in v):6d} {top2:19d} {both:5d}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("match", nargs="?")
    ap.add_argument("--out")
    ap.add_argument("--read")
    ap.add_argument("--controls", type=int, default=38)
    args = ap.parse_args()
    if args.read:
        read(args)
    elif args.match and args.out:
        run(args)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
