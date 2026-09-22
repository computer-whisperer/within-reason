#!/usr/bin/env python3
"""A/B of the hands' standing rules on recorded moments: the questionable Jev answers of pianist games are found
in `jev-<ai>.jsonl` by detectors, the very request each one was answered from is rebuilt (the picture's state with
the packet and the rules in force, and the questions), and it is put to Jev again under each variant of the rules.
What changes is measured against what we judge the right answer to have been, and against a control set of ordinary
decisions that should not change (the user, 2026-09-22: "find questionable jev action moments, and run A/B tests on
some jev instruction variations to find what helps").

usage: run/jev_ab.py run/matches/<batch>/<NN> [...] [--variants NAME,NAME] [--per-detector N] [--control N]
                     [--repeat R] [--workers W] [--out FILE.jsonl] [--list] [--summarize FILE.jsonl]
  --list prints the moments found and asks nothing. --repeat R asks each request R times (Jev's own variance).
  --out defaults to docs/studies/data/jev-ab-<date>.jsonl; --summarize reads one back and prints the tables.
The key comes from TYPESAFE_API_KEY or ~/.config/within-reason/jev.env and is never printed or written.

Detectors (each names the options it judges right, `wanted`):
  hold_beside_attack  a group answered hold/continue with an enemy party in sight within 800 of it, when one of our
                      buildings within 800 of the group died in the next 30 s        wanted: engage, attack_unit, send_against
  whom_not_commander  the group attacked, their commander's party (one we outweigh) was on the `whom` list and
                      another party was chosen                                         wanted: that party
  shelling_over_named fight_to/move_to with `where` = shelling while the packet forbids it, or names places for
                      this actor and never mentions shelling                           wanted: any place but shelling
  never_split         send_against or split while the packet says never split / no detachments   wanted: anything else
  raider_ignored      a party stood at one of our extractors within 1,200 of this group, in its own entry, and it
                      answered hold/continue                                           wanted: send_against, engage, attack_unit
"""
import collections
import concurrent.futures
import datetime
import json
import os
import random
import re
import statistics
import sys
import time
import urllib.error
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from floor import NEVER_SHELLING, NEVER_SPLIT  # noqa: E402
from jev_audit import Match, clock  # noqa: E402

URL = "https://api.typesafe.ai/v1/systemone"
KEY_FILE = os.path.expanduser("~/.config/within-reason/jev.env")
AFTER_FRAMES = 900
NEAR = 800.0

# The variants: a function from the rules in force to the rules to send. Each is one added or changed sentence, so
# what helps can be told apart; `all` is every addition together.
NEVER_RULE = (
    ' A "never" in the instructions removes the choice: when they say an actor never does something, or never goes'
    " to a place, that option or place is not chosen whatever its own words on the menu say."
)
HOME_RULE = (
    " An enemy party in sight at one of our buildings, or within 600 of a group that outweighs it, is fought now"
    " (engage or attack_unit), whatever the instructions call the raiders: a group holding beside a base under"
    " attack loses the buildings."
)
COMMANDER_RULE = (
    " Their commander in sight is the target above every other when we outweigh its party: attack_unit on it wins"
    " the game, and a raid party far from the group is another group's business."
)
SHELLING_OLD = re.compile(r"`shelling` is where a weapon hitting us from out of sight likeliest stands, and advancing onto it kills it\.")
SHELLING_NEW = (
    "`shelling` is only where a weapon hitting us from out of sight likeliest stands; a group advances onto it only"
    " when the instructions say so, and otherwise goes to the place the instructions name."
)


def shelling_rule(rules):
    return SHELLING_OLD.sub(SHELLING_NEW, rules) if SHELLING_OLD.search(rules) else rules + " " + SHELLING_NEW


def commander_words(questions, moment):
    """The menu words H-HANDS-PARTY-NAMES gave a party with their commander in it (2026-09-22 night), put on the
    recorded `whom` and `engage` words of an older game: the party says THEIR COMMANDER and how far it is from the
    group. A words variant, not a rules one: it tests the harness change on the moments that motivated it."""
    actor = moment["actor"]
    group = next((g for g in moment.get("groups") or [] if f"group_{g['name']}" == actor), None)
    parties = {p["name"]: p for p in moment.get("parties") or []}
    out = json.loads(json.dumps(questions))
    whom = out.get(f"{actor}.whom")
    if not whom:
        return out
    for name, words in list((whom.get("criteria") or {}).items()):
        text = str(words)
        party = parties.get(name)
        elmos = dist((party["x"], party["z"]), group["at"]) if party and group and group.get("at") else None
        composition, _, rest = text.partition(" at ")
        place, _, odds = rest.partition(": ")
        far = f", {distance_words(elmos)} from this group ({elmos:.0f} away)" if elmos is not None else ""
        who = f"THEIR COMMANDER, the unit whose death wins the game ({composition})" if "armcom" in text or "corcom" in text else composition
        whom["criteria"][name] = f"{who} at {place}{far}: {odds}"
        do = out.get(f"{actor}.do", {}).get("criteria", {})
        if "engage" in do and who != composition:
            do["engage"] = str(do["engage"]).replace(f"{name} ({composition}", f"{name} ({who}")
    return out


def distance_words(elmos):
    """`pianist/picture.rs` `distance_words`."""
    return "right here" if elmos < 500 else "near" if elmos < 1200 else "some way off" if elmos < 2500 else "far"


NEVER_RULE_2 = (
    " The instructions' never-clauses are law: an option or a place they forbid for an actor is wrong even when its"
    " own words fit the moment, and the next best choice is taken instead."
)
HOME_RULE_2 = " An enemy party in sight within 600 of a group that outweighs it is fought now, whatever the instructions call it."
SHELLING_RULE_2 = re.compile(r", and advancing onto it kills it\.")

# name -> (rules transform, questions transform or None)
VARIANTS = {
    "base": (lambda rules: rules, None),
    "never": (lambda rules: rules + NEVER_RULE, None),
    "never2": (lambda rules: rules + NEVER_RULE_2, None),
    "home": (lambda rules: rules + HOME_RULE, None),
    "home2": (lambda rules: rules + HOME_RULE_2, None),
    "commander": (lambda rules: rules + COMMANDER_RULE, None),
    "shelling": (shelling_rule, None),
    # The old sentence with its last clause cut: the smallest change that stops calling shelling a target.
    "shelling2": (lambda rules: SHELLING_RULE_2.sub(".", rules), None),
    "all": (lambda rules: shelling_rule(rules) + NEVER_RULE + HOME_RULE + COMMANDER_RULE, None),
    "words": (lambda rules: rules, commander_words),
}


def api_key():
    key = os.environ.get("TYPESAFE_API_KEY")
    if not key and os.path.exists(KEY_FILE):
        for line in open(KEY_FILE):
            if line.startswith("TYPESAFE_API_KEY="):
                key = line.split("=", 1)[1].strip().strip("\"'")
    if not key:
        sys.exit(f"no API key: put TYPESAFE_API_KEY=... in {KEY_FILE}")
    return key


def ask(key, state, questions, model):
    body = {"model": model, "state": state, "questions": questions}
    request = urllib.request.Request(URL, data=json.dumps(body).encode(), method="POST", headers={
        "Authorization": f"Bearer {key}", "Content-Type": "application/json", "User-Agent": "within-reason-jev-ab"})
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


def dist(a, b):
    return ((a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2) ** 0.5


def requests_of(m):
    """Every call as (call, state_as_sent): the logged state with the packet and the rules in force put back."""
    rules = (m.jev_header or {}).get("rules", "")
    packet = ""
    for c in m.calls:
        if "instructions" in c:
            packet = c["instructions"]
        if "rules" in c:
            rules = c["rules"]
        state = dict(c["state"])
        state["instructions"] = packet
        state["rules"] = rules
        yield c, state, packet


def our_building_deaths(m, f, at, radius):
    """Our buildings destroyed by an enemy within `radius` of `at` in the 30 s after `f`."""
    out = []
    for e in m.events:
        if e["k"] == "destroyed" and e.get("by") is not None and f <= e["f"] < f + AFTER_FRAMES:
            if m.defs.get(e["d"], {}).get("class") == "building" and dist((e["x"], e["z"]), at) <= radius:
                out.append(m.def_name(e["d"]))
    return out


def choice_of(c, qid):
    a = c["answers"].get(qid) or {}
    return a.get("choice"), a.get("probabilities") or {}


def moments_of(m):
    """The questionable moments of one match, each with the request to replay and the options judged right."""
    for c, state, packet in requests_of(m):
        f = c["f"]
        groups = {f"group_{g['name']}": g for g in c.get("groups") or []}
        parties = c.get("parties") or []
        for qid, q in c["questions"].items():
            if not qid.endswith(".do"):
                continue
            actor = qid[:-3]
            chosen, probs = choice_of(c, qid)
            if chosen is None:
                continue
            options = set((q.get("criteria") or {}).keys())
            common = {"match": m.label, "f": f, "clock": clock(f), "actor": actor, "qid": qid, "chosen": chosen, "p_chosen": probs.get(chosen, 0.0), "state": state, "questions": c["questions"], "groups": c.get("groups") or [], "parties": parties}
            group = groups.get(actor)
            # hold_beside_attack
            if group and group.get("at") and chosen in ("hold", "continue") and parties:
                near = [p for p in parties if dist((p["x"], p["z"]), group["at"]) <= NEAR]
                if near:
                    deaths = our_building_deaths(m, f, tuple(group["at"]), NEAR)
                    if deaths:
                        wanted = [o for o in ("engage", "attack_unit", "send_against") if o in options]
                        if wanted:
                            yield {**common, "detector": "hold_beside_attack", "wanted": wanted, "note": f"{len(near)} party in sight within 800; lost {', '.join(deaths[:3])} in 30 s"}
            # whom_not_commander
            wq = c["questions"].get(f"{actor}.whom")
            if wq:
                crit = wq.get("criteria") or {}
                # Only when the group attacks something and the commander's party is one we outweigh: a whom answer
                # under hold changes nothing, and attacking a party that outweighs us is not the right answer.
                com = [name for name, words in crit.items() if ("armcom" in str(words) or "THEIR COMMANDER" in str(words) or "corcom" in str(words)) and "we outweigh" in str(words)]
                whom, _ = choice_of(c, f"{actor}.whom")
                if com and whom and whom not in com and chosen in ("engage", "attack_unit", "send_against"):
                    yield {**common, "qid": f"{actor}.whom", "chosen": whom, "p_chosen": choice_of(c, f"{actor}.whom")[1].get(whom, 0.0), "detector": "whom_not_commander", "wanted": com, "note": f"do={chosen}; commander in {com[0]}: {str(crit[com[0]])[:80]}"}
            # shelling_over_named
            if chosen in ("fight_to", "move_to"):
                where, wprobs = choice_of(c, f"{actor}.where")
                names_place = re.search(rf"{re.escape(actor)}[^\n]*(spot_\d+|passage_\d+|home)", packet) and "shelling" not in packet
                if where == "shelling" and (NEVER_SHELLING.search(packet) or names_place):
                    wanted = [o for o in (c["questions"].get(f"{actor}.where") or {}).get("criteria", {}) if o != "shelling"]
                    yield {**common, "qid": f"{actor}.where", "chosen": "shelling", "p_chosen": wprobs.get("shelling", 0.0), "detector": "shelling_over_named", "wanted": wanted, "note": "packet forbids shelling" if NEVER_SHELLING.search(packet) else "packet names places for this actor"}
            # never_split
            if chosen in ("send_against", "split") and NEVER_SPLIT.search(packet):
                yield {**common, "detector": "never_split", "wanted": [o for o in options if o not in ("send_against", "split")], "note": "packet says never split"}
            # raider_ignored
            entry = (state.get("actors") or {}).get(actor) or {}
            raiders = [t for t in entry.get("enemies_at_our_extractors") or [] if (mm := re.search(r"\((\d+)\) from this group", str(t))) and int(mm.group(1)) <= 1200]
            if raiders and chosen in ("hold", "continue"):
                wanted = [o for o in ("send_against", "engage", "attack_unit") if o in options]
                if wanted:
                    yield {**common, "detector": "raider_ignored", "wanted": wanted, "note": str(raiders[0])[:80]}


def controls_of(m, n, rng):
    """Ordinary decisions: group and builder .do answers not flagged, sampled; the right answer is what was chosen."""
    pool = []
    for c, state, packet in requests_of(m):
        for qid, q in c["questions"].items():
            if qid.endswith(".do"):
                chosen, probs = choice_of(c, qid)
                if chosen and probs.get(chosen, 0.0) >= 0.6:
                    pool.append({"match": m.label, "f": c["f"], "clock": clock(c["f"]), "actor": qid[:-3], "qid": qid, "chosen": chosen, "p_chosen": probs[chosen], "state": state, "questions": c["questions"], "detector": "control", "wanted": [chosen], "note": "ordinary decision, chosen at 0.6 or more"})
    rng.shuffle(pool)
    return pool[:n]


def replay(moment, variant, key, model, repeat):
    state = dict(moment["state"])
    rules_fn, questions_fn = VARIANTS[variant]
    state["rules"] = rules_fn(state.get("rules", ""))
    # Only this actor's questions (and the globals are left out): the answer to the moment's question is what counts.
    actor = moment["actor"]
    questions = {k: v for k, v in moment["questions"].items() if k.startswith(actor + ".")}
    if questions_fn:
        questions = questions_fn(questions, moment)
    runs = []
    for _ in range(repeat):
        response = ask(key, state, questions, model)
        answer = (response or {}).get("answers", {}).get(moment["qid"]) or {}
        runs.append({"choice": answer.get("choice"), "probabilities": answer.get("probabilities") or {}})
    return runs


def summarize(rows):
    by = collections.defaultdict(list)
    for r in rows:
        by[(r["detector"], r["variant"])].append(r)
    detectors = sorted({d for d, _ in by})
    variants = [v for v in VARIANTS if any((d, v) in by for d in detectors)]
    print(f"{'detector':20s} {'n':>4s} " + " ".join(f"{v:>16s}" for v in variants))
    print(f"{'':20s} {'':>4s} " + " ".join(f"{'P(wanted) right%':>16s}" for _ in variants))
    for d in detectors:
        cells = []
        n = 0
        for v in variants:
            rs = by.get((d, v), [])
            if not rs:
                cells.append(f"{'-':>16s}")
                continue
            n = len(rs)
            p = statistics.mean(statistics.mean(sum(run["probabilities"].get(w, 0.0) for w in r["wanted"]) for run in r["runs"]) for r in rs)
            right = statistics.mean(statistics.mean(1.0 if run["choice"] in r["wanted"] else 0.0 for run in r["runs"]) for r in rs)
            cells.append(f"{p:8.2f} {100 * right:6.0f}%")
        print(f"{d:20s} {n:4d} " + " ".join(cells))
    print("\nP(wanted): the probability Jev puts on the options judged right, averaged over the moments; right%: how often its pick is one of them.")
    print("For control the wanted option is the recorded choice, so right% is how much ordinary play a variant leaves alone.")


def main():
    args = sys.argv[1:]

    def opt(name, default=None):
        if name in args:
            i = args.index(name)
            value = args[i + 1]
            del args[i:i + 2]
            return value
        return default

    summarize_path = opt("--summarize")
    if summarize_path:
        summarize([json.loads(l) for l in open(summarize_path) if l.strip()])
        return
    variants = opt("--variants", ",".join(VARIANTS)).split(",")
    per_detector = int(opt("--per-detector", "40"))
    control_n = int(opt("--control", "40"))
    repeat = int(opt("--repeat", "1"))
    workers = int(opt("--workers", "6"))
    out = opt("--out", f"docs/studies/data/jev-ab-{datetime.date.today().isoformat()}.jsonl")
    list_only = "--list" in args
    dirs = [a for a in args if not a.startswith("--")]
    if not dirs:
        sys.exit(__doc__)
    rng = random.Random(1)
    moments = []
    controls = []
    for d in dirs:
        m = Match(d)
        if not m.calls:
            continue
        moments.extend(moments_of(m))
        controls.extend(controls_of(m, control_n, rng))
    # Cap per detector, spread over the matches: shuffle, then take the first N of each.
    rng.shuffle(moments)
    per = collections.defaultdict(list)
    for mo in moments:
        if len(per[mo["detector"]]) < per_detector:
            per[mo["detector"]].append(mo)
    rng.shuffle(controls)
    per["control"] = controls[:control_n]
    chosen = [mo for ms in per.values() for mo in ms]
    counts = collections.Counter(mo["detector"] for mo in moments)
    print(f"moments found: {dict(counts)}; replaying {sum(len(v) for v in per.values())} (cap {per_detector} a detector, {len(per['control'])} controls) under {len(variants)} variants x {repeat}", file=sys.stderr)
    if list_only:
        for mo in sorted(chosen, key=lambda x: (x["detector"], x["match"], x["f"])):
            print(f"{mo['detector']:20s} {mo['match']:32s} {mo['clock']:>6s} {mo['actor']:18s} {mo['qid'].rsplit('.', 1)[1]:5s} chose {mo['chosen']} ({mo['p_chosen']:.2f}) wanted {mo['wanted'][:3]}  {mo['note']}")
        return
    key = api_key()
    model = os.environ.get("TYPESAFE_DEFAULT_MODEL", "jev-latest")
    os.makedirs(os.path.dirname(out) or ".", exist_ok=True)
    jobs = [(mo, v) for mo in chosen for v in variants]
    started = time.time()
    rows = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool, open(out, "w") as sink:
        futures = {pool.submit(replay, mo, v, key, model, repeat): (mo, v) for mo, v in jobs}
        done = 0
        for fut in concurrent.futures.as_completed(futures):
            mo, v = futures[fut]
            try:
                runs = fut.result()
            except Exception as e:  # noqa: BLE001
                print(f"{mo['match']} {mo['clock']} {mo['actor']} {v}: {e}", file=sys.stderr)
                continue
            row = {k: mo[k] for k in ("match", "f", "clock", "actor", "qid", "chosen", "p_chosen", "detector", "wanted", "note")}
            row.update({"variant": v, "runs": runs})
            rows.append(row)
            sink.write(json.dumps(row) + "\n")
            done += 1
            if done % 50 == 0:
                print(f"{done}/{len(jobs)} in {time.time() - started:.0f} s", file=sys.stderr)
    print(f"{len(rows)} answers in {time.time() - started:.0f} s -> {out}", file=sys.stderr)
    summarize(rows)


if __name__ == "__main__":
    main()
