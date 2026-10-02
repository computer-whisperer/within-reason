#!/usr/bin/env python3
"""Offline: the rebuild's bet (docs/design/2026-10-01-hands-rebuild.md §8.2). In the recorded picks where a group was
sent to "fall back to home from party_N", the walk home is taken out, the packet is given a sentence naming a place
for that group to step back to, and one world is added in which the group walks there. Asked again: does the pick
take the walk to the named place? And as a control, in recorded picks where a group attacked a party it outweighed,
the same sentence and the same world are added: does the pick now leave a fight it should keep? Nothing is played.

The place stands in for the player's choice: the spot nearest the group that is 500 to 1,800 from it and at least
400 farther from the party than the group is.

usage: run/jev_fallback_ab.py run/matches/<batch>/<NN> --out <file.jsonl> [--controls N]
       run/jev_fallback_ab.py --read <file.jsonl>
The key is read by run/jev_ab.py's api_key() and never printed or written. Jev: $0.042 a million input tokens."""
import argparse, collections, concurrent.futures, copy, json, math, os, random, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from jev_back_ab import kind_of
from match_read import Match

SENTENCE = "{group}: when a party outweighs the part of it that is in the fight, or outranges it, it steps back to {place} without fighting and waits there for the rest of its soldiers; it goes on from there when it outweighs what it meets. It does not walk home."


def dist(a, b):
    return math.hypot(a[0] - b[0], a[1] - b[1])


def words(d):
    return "right here" if d < 300 else "near" if d < 800 else "some way off" if d < 1600 else "far"


def place_for(row, group, party):
    """The stand-in for the player's fall-back place."""
    g = next((x for x in row.get("groups") or [] if "group_" + x["name"] == group), None)
    p = next((x for x in row.get("parties") or [] if x["name"] == party), None)
    if not g or not p or not g.get("at"):
        return None
    at, foe = tuple(g["at"]), (p["x"], p["z"])
    here = dist(at, foe)
    fit = [(dist(at, (pl["x"], pl["z"])), pl) for pl in row.get("places") or [] if pl.get("spot") is not None]
    fit = [(d, pl) for d, pl in fit if 500 <= d <= 1800 and dist((pl["x"], pl["z"]), foe) >= here + 400]
    if not fit:
        return None
    d, pl = min(fit, key=lambda x: x[0])
    return pl["name"], d, dist((pl["x"], pl["z"]), foe), here


def run(args):
    key = api_key()
    m = Match(args.match)
    rows_at = {}
    for c in m.calls:
        if c.get("t") == "call" and c.get("groups") is not None:
            rows_at.setdefault(c["f"], c)
    plans = {c["f"]: c for c in m.calls if c.get("t") == "plan"}
    picks = collections.defaultdict(list)
    for c, state, _ in requests_of(m):
        if "worlds.pick" in c["questions"]:
            picks[c["f"]].append((c, state))
    jobs, controls = [], []
    for f, plan in sorted(plans.items()):
        one = [(c, s) for c, s in picks[f] if "w1" not in c["questions"]["worlds.pick"]["criteria"]]
        two = [(c, s) for c, s in picks[f] if "w1" in c["questions"]["worlds.pick"]["criteria"]]
        if not one or not two or f not in rows_at:
            continue
        for p in plan.get("played") or []:
            played = p.get("played") or ""
            back = re.match(r"(party_\d+)\.back_(group_\w+)", played)
            whole = re.match(r"(party_\d+)\.whole_(group_\w+)", played)
            hit = back or whole
            if not hit:
                continue
            party, group = hit.group(1), hit.group(2)
            crit = one[0][0]["questions"]["worlds.pick"]["criteria"]
            if back:
                line = next((l for l in crit.values() if f"{group} falls back to home from {party}" in l), None)
                facts = re.search(re.escape(f"{group} falls back to home from {party} ") + r"\((.*?)\), \d+ from its front", line or "")
            else:
                line = next((l for l in crit.values() if f"{group} attacks {party}" in l and "we outweigh it" in l), None)
                facts = re.search(re.escape(f"{group} attacks {party} ") + r"\((.*?)\) with the whole group \(.*?: (.*?)\), \d+ from its front", line or "")
            where = place_for(rows_at[f], group, party)
            if not line or not facts or not where:
                continue
            said = facts.group(1) if back else f"{facts.group(1)}; {facts.group(2)}"
            (jobs if back else controls).append((f, group, party, said, where, one[0], two[0], "back" if back else "control"))
    random.Random(1).shuffle(controls)
    jobs += controls[: args.controls]
    print(f"{sum(j[7] == 'back' for j in jobs)} fall-backs to home, {sum(j[7] == 'control' for j in jobs)} controls (of {len(controls)})", file=sys.stderr)

    def one_job(job):
        f, group, party, said, (place, d, d_foe, here), (c1, state), (c2, _), kind = job
        state = copy.deepcopy(state)
        state["instructions"] = state.get("instructions", "") + "\n\n" + SENTENCE.format(group=group, place=place)
        crit = dict(c1["questions"]["worlds.pick"]["criteria"])
        recorded = c1["answers"]["worlds.pick"].get("probabilities") or {}
        gone = {w for w, l in crit.items() if f"{group} falls back to home from" in l}
        left = {w: l for w, l in crit.items() if w not in gone}
        new = f"w{max(int(w[1:]) for w in crit) + 1}"
        left[new] = f"As w1, and: {group} walks to {place} ({words(d)}: {d:.0f} away) without stopping to fight on the way, stepping back from {party} ({said}): {place} is {d_foe:.0f} from {party}, {d_foe - here:.0f} farther than {group} stands now."
        row = {"f": f, "group": group, "party": party, "kind": kind, "place": place, "line": left[new], "recorded_home": sum(recorded.get(w, 0) for w in gone), "tokens": 0}
        q1 = copy.deepcopy(c1["questions"]["worlds.pick"])
        q1["criteria"] = left
        a1 = ask(key, state, {"worlds.pick": q1}, c1["model"])
        row["tokens"] += (a1.get("usage") or {}).get("input_tokens", 0)
        probs = a1["answers"]["worlds.pick"].get("probabilities") or {}
        row["p_place"] = probs.get(new, 0)
        after = collections.Counter()
        for w, pr in probs.items():
            after["walk to the named place" if w == new else kind_of(left[w], group)] += pr
        row["after"] = dict(after)
        top = max(probs, key=probs.get)
        row["top"] = "walk to the named place" if top == new else kind_of(left[top], group)
        # Stage two twice: the top world against w1 (what the game would ask if it sampled the top), and the named
        # place against w1 whatever stage one preferred.
        for name, w in (("choice", top), ("place_against_w1", new)):
            q2 = copy.deepcopy(c2["questions"]["worlds.pick"])
            q2["criteria"] = {"w1": q2["criteria"]["w1"], w: left[w]}
            a2 = ask(key, state, {"worlds.pick": q2}, c2["model"])
            row["tokens"] += (a2.get("usage") or {}).get("input_tokens", 0)
            p2 = a2["answers"]["worlds.pick"].get("probabilities") or {}
            row[name] = p2.get(w, 0)
        row["w1"] = c2["questions"]["worlds.pick"]["criteria"]["w1"][:240]
        return row

    with open(args.out, "w") as out, concurrent.futures.ThreadPoolExecutor(8) as pool:
        tokens = 0
        for row in pool.map(one_job, jobs):
            tokens += row.pop("tokens")
            out.write(json.dumps(row) + "\n")
    print(f"{tokens} input tokens, ${tokens * 0.042 / 1e6:.3f}", file=sys.stderr)


def read(args):
    rows = [json.loads(line) for line in open(args.read)]
    clock = lambda f: f"{f // 1800}:{f // 30 % 60:02d}"
    med = lambda xs: sorted(xs)[len(xs) // 2] if xs else float("nan")
    for kind, title in (("back", "the recorded fall-backs to home, the walk home taken out and the named place put in"), ("control", "controls: the group attacked a party it outweighed, the named place put in beside it")):
        rs = [r for r in rows if r["kind"] == kind]
        if not rs:
            continue
        print(f"{title}: {len(rs)} picks")
        print(f"  stage one: the named place's share median {med([r['p_place'] for r in rs]):.2f}; it is the top world in {sum(r['top'] == 'walk to the named place' for r in rs)}")
        print(f"  stage one's top world: {dict(collections.Counter(r['top'] for r in rs).most_common())}")
        print(f"  stage two, the top world against 'nothing changes': the change wins in {sum(r['choice'] > 0.5 for r in rs)}; of them the named place {sum(r['choice'] > 0.5 and r['top'] == 'walk to the named place' for r in rs)}")
        print(f"  stage two, the named place against 'nothing changes': it wins in {sum(r['place_against_w1'] > 0.5 for r in rs)} (median {med([r['place_against_w1'] for r in rs]):.2f})")
        if args.lines:
            for r in rs:
                print(f"    {clock(r['f'])} {r['group']} p1 {r['p_place']:.2f} top {r['top'][:28]:28s} vs w1 {r['place_against_w1']:.2f} | {r['line'][12:230]}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("match", nargs="?")
    ap.add_argument("--out")
    ap.add_argument("--read")
    ap.add_argument("--controls", type=int, default=38)
    ap.add_argument("--lines", action="store_true")
    args = ap.parse_args()
    if args.read:
        read(args)
    elif args.match and args.out:
        run(args)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
