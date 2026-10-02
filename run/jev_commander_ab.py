#!/usr/bin/env python3
"""Replay the recorded commander asks with the commander weighed as a fighter (H-HANDS-COMMANDER-WORTH) before a
game is spent on it. Every call of a pianist game that asked `commander.do` with an enemy party on the commander's
`enemies_near` line is rebuilt as it was sent (`run/jev_ab.py` `requests_of`: the logged state with the packet and
the rules in force) and put to Jev again under variants:

  base   as recorded (odds by metal: "we outweigh it heavily" against a line of tanks)
  worth  the line's odds recomputed from the commander's fighting worth, with the reach words (what must walk into
         its D-gun, what outreaches it); the rules as recorded
  rule   the recorded line, the rules with the commander sentence appended
  both   the recomputed line and the sentence

The recomputation follows `combat.rs` `power` for a lone commander: its worth sqrt(dps x health) over the tier-1
median of that per metal, from the glossary (`crates/bot/data/units.json`; the bot reads the simulator's table, whose
numbers agree within a few percent), against the party's metal (110 for a radar blip); no matchup row involves a
commander, so the ratio is the bot's. What is measured: on the moments the new odds call outweighed or even, the
share of answers that take the commander away (retreat_home, or walk_to); on the moments it still outweighs the
party (raiders), the share that keeps its work. A death is not needed to judge these: the right answer is known from
the odds.

usage: run/jev_commander_ab.py run/matches/<batch>/<NN> [...] [--variants base,worth,rule,both] [--repeat R]
                               [--out FILE.jsonl] [--list] [--summarize FILE.jsonl] [--model ID]
The key comes from TYPESAFE_API_KEY or ~/.config/within-reason/jev.env and is never printed or written.
"""
import argparse, collections, json, math, os, re, statistics, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, choice_of, requests_of  # noqa: E402
from match_read import Match, clock  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
UNIDENTIFIED_METAL = 110.0
SENTENCE = (
    " The commander's `enemies_near` line weighs it as the fighter it is, not by its metal: it fights what comes to"
    " it and raiders that must walk into its D-gun, and it never walks toward, builds toward or attacks a party that"
    " outreaches its D-gun or outweighs it; away from home it walks back from such a party (retreat_home, or walk_to"
    " a place of ours behind it) before it is hit, whatever it was building."
)
LINE = re.compile(r"^(party_\d+) \((.*?)\) (\d+) away: against this unit alone, (.*)$")
AWAY = ("retreat_home", "walk_to")


def glossary():
    units = json.load(open(os.path.join(REPO, "crates/bot/data/units.json")))["units"]
    rows = units if isinstance(units, list) else [dict(v, name=k) for k, v in units.items()]
    return {r["name"]: r for r in rows}


def tier1_scale(g):
    ratios = sorted(math.sqrt(r["dps"] * r["health"]) / r["metal"] for r in g.values()
                    if r.get("tier") == 1 and r.get("dps") and r.get("health") and r.get("metal") and (r.get("speed") or 0) > 0
                    and "constructor" not in (r.get("class") or "") and "com" not in r["name"] and "air" not in (r.get("class") or ""))
    return ratios[len(ratios) // 2] if ratios else 2.0


def odds_words(ratio):
    return "we outweigh it heavily" if ratio >= 2.5 else "we outweigh it" if ratio >= 1.3 else "an even fight" if ratio >= 0.8 else "it outweighs us"


def parse_composition(text):
    """"2 armstump, 3 armwar, 1 unidentified" -> [(count, name)]."""
    out = []
    for part in text.split(","):
        m = re.match(r"\s*(\d+)\s+(\S+)", part)
        if m:
            out.append((int(m.group(1)), m.group(2)))
    return out


def new_line(entry, g, scale):
    """The commander's `enemies_near` line with the fighting-worth odds and the reach words; None when no party."""
    m = LINE.match(str(entry.get("enemies_near", "")))
    if not m:
        return None, None
    party, composition, away, old_words = m.groups()
    ours = "corcom" if "corcom" in str(entry.get("is", "")) else "armcom"
    com = g[ours]
    worth = math.sqrt(com["dps"] * com["health"]) / scale
    dgun = 262.0 if ours == "corcom" else 250.0
    theirs, inside, beyond, unknown = 0.0, [], [], 0
    n_in = n_out = 0
    for n, name in parse_composition(composition):
        if name == "unidentified":
            theirs += n * UNIDENTIFIED_METAL
            unknown += n
            continue
        u = g.get(name)
        if not u:
            theirs += n * UNIDENTIFIED_METAL
            unknown += n
            continue
        theirs += n * (u.get("metal") or 0)
        if not u.get("dps"):
            continue
        reach = u.get("range") or 0
        label = f"{u.get('gloss') or name} {reach:.0f}"
        if reach <= dgun + 20:
            inside.append(label); n_in += n
        else:
            beyond.append(label); n_out += n
    ratio = worth / max(theirs, 1.0)
    # As `odds_words`: a party with nothing that shoots (constructors, a radar) cannot hit the commander.
    words = "it cannot hit us: nothing there shoots at what this group is" if not (n_in or n_out or unknown) else odds_words(ratio)
    line = f"{party} ({composition}) {away} away: against this unit alone, {words}"
    if n_in:
        line += f"; {n_in} of them must come inside its D-gun ({dgun:.0f}) to hit it ({', '.join(inside)})"
    if n_out:
        line += f"; {n_out} of them outreach its D-gun and it cannot answer them ({', '.join(beyond)})"
    if unknown:
        line += f"; {unknown} of unknown type"
    return line, {"ratio": ratio, "words": words, "old_words": old_words, "theirs": theirs, "worth": worth, "n_out": n_out, "n_in": n_in}


def moments_of(m, g, scale):
    for c, state, packet in requests_of(m):
        q = c["questions"].get("commander.do")
        if not q:
            continue
        entry = (state.get("actors") or {}).get("commander")
        if not isinstance(entry, dict) or "enemies_near" not in entry:
            continue
        line, odds = new_line(entry, g, scale)
        if line is None:
            continue
        chosen, probs = choice_of(c, "commander.do")
        yield {"match": m.label, "f": c["f"], "clock": clock(c["f"]), "chosen": chosen, "p_chosen": probs.get(chosen, 0.0) if chosen else 0.0,
               "state": state, "questions": c["questions"], "line": line, "old_line": entry["enemies_near"], "odds": odds,
               "doing": entry.get("doing", ""), "health": entry.get("health", "")}


def variant_request(moment, variant):
    state = json.loads(json.dumps(moment["state"]))
    if variant in ("worth", "both"):
        state["actors"]["commander"]["enemies_near"] = moment["line"]
    if variant in ("rule", "both"):
        state["rules"] = state.get("rules", "") + SENTENCE
    questions = {k: v for k, v in moment["questions"].items() if k.startswith("commander.")}
    return state, questions


def summarize(rows):
    by = collections.defaultdict(list)
    for r in rows:
        by[(r["variant"], r["class"])].append(r)
    variants = sorted({r["variant"] for r in rows}, key=lambda v: ["base", "worth", "rule", "both"].index(v) if v in ("base", "worth", "rule", "both") else 9)
    print(f"{'variant':8} {'moments (new odds)':22} {'away':>6} {'keeps work':>11}  answers")
    for cls in ("outweighed", "even", "outweighs"):
        for v in variants:
            rs = by.get((v, cls), [])
            if not rs:
                continue
            away = sum(r["choice"] in AWAY for r in rs)
            keep = sum(r["choice"] not in AWAY and r["choice"] not in ("attack",) for r in rs)
            counts = collections.Counter(r["choice"] for r in rs)
            print(f"{v:8} {cls:10} {len(rs):4} asks   {away:5} {100*away/len(rs):3.0f}% {keep:5} {100*keep/len(rs):3.0f}%  {dict(counts.most_common(5))}")
    print("recorded choices on the same moments:", dict(collections.Counter(r["chosen"] for r in rows if r["variant"] == variants[0]).most_common(6)))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("matches", nargs="*")
    ap.add_argument("--variants", default="base,worth,rule,both")
    ap.add_argument("--repeat", type=int, default=1)
    ap.add_argument("--model", default="jev-latest")
    ap.add_argument("--out", default=os.path.join(REPO, "docs/studies/data", f"jev-commander-ab-{time.strftime('%Y-%m-%d')}.jsonl"))
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--summarize")
    a = ap.parse_args()
    if a.summarize:
        summarize([json.loads(l) for l in open(a.summarize)])
        return
    g = glossary()
    scale = tier1_scale(g)
    moments = []
    for path in a.matches:
        m = Match(path if os.path.basename(path).isdigit() else os.path.join(path, "00"))
        moments.extend(moments_of(m, g, scale))
    print(f"{len(moments)} commander asks with a party on the line (scale {scale:.2f}, commander worth {math.sqrt(g['armcom']['dps'] * g['armcom']['health']) / scale:.0f})", file=sys.stderr)
    cls = lambda o: "outweighed" if o["words"] == "it outweighs us" else "even" if o["words"] == "an even fight" else "outweighs"
    if a.list:
        for mo in moments:
            print(f"{mo['match']} {mo['clock']} chose {mo['chosen']} ({mo['p_chosen']:.2f}) {mo['health']} | was: {mo['odds']['old_words']} | now: {mo['odds']['words']} ({mo['odds']['ratio']:.2f}; {mo['odds']['n_out']} outreach) | {mo['doing'][:60]}")
        counts = collections.Counter(cls(mo["odds"]) for mo in moments)
        print("by the new odds:", dict(counts), file=sys.stderr)
        return
    key = api_key()
    variants = a.variants.split(",")
    os.makedirs(os.path.dirname(a.out), exist_ok=True)
    rows = []
    with open(a.out, "w") as out:
        for i, mo in enumerate(moments):
            for v in variants:
                state, questions = variant_request(mo, v)
                for _ in range(a.repeat):
                    response = ask(key, state, questions, a.model)
                    answer = (response or {}).get("answers", {}).get("commander.do") or {}
                    where = (response or {}).get("answers", {}).get("commander.where") or {}
                    row = {"match": mo["match"], "f": mo["f"], "clock": mo["clock"], "variant": v, "class": cls(mo["odds"]), "chosen": mo["chosen"],
                           "choice": answer.get("choice"), "probabilities": answer.get("probabilities") or {}, "where": where.get("choice"),
                           "old_line": mo["old_line"], "line": mo["line"], "ratio": mo["odds"]["ratio"]}
                    rows.append(row)
                    out.write(json.dumps(row) + "\n")
            print(f"  {i + 1}/{len(moments)} {mo['match']} {mo['clock']}", file=sys.stderr)
    summarize(rows)
    print(f"wrote {a.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
