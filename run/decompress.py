#!/usr/bin/env python3
"""Jev as a decompression agent (the user, 2026-09-25): can Jev turn the player's prose packet into standing orders
from a fixed vocabulary, so the hands run those in code between player turns instead of re-reading the packet forty
times a minute? This is the offline test: every `instruct` packet of a recorded game is put to Jev once, with one
extraction question per (actor, rule) over the vocabulary below, and three things are measured. (1) Consistency: when
an actor's paragraphs are identical in two packets, the extracted rules should be identical. (2) Coverage: at the
detectors' moments (`run/jev_ab.py`: a raider ignored, a forbidden detachment, an advance onto shelling) the packet in
force is read back: would the standing order extracted from it have answered the moment? (3) A readable dump per game
for hand checking (`--dump`).

    run/decompress.py run/matches/<batch>/<NN> [...] [--limit N] [--out FILE.jsonl] [--dump] [--summarize FILE.jsonl]

The key comes from TYPESAFE_API_KEY or ~/.config/within-reason/jev.env and is never printed or written. A packet is
about 400 tokens and carries 60-150 small questions; a game of 90 packets costs well under a cent.

The vocabulary (v0, from reading the packets of family-1, bank-1, 2v1b-hard, 2v1b-hard_aggressive):
  group:   station (a named place or none); raiders (whole_group / detachment / not_said / forbidden);
           detachment_size (1/2/4/half/not_said); no_chase; no_detachments; fall_back (outweighed /
           never_while_even_or_better / not_said); fall_back_to (place); move (advance_and_fight / walk /
           engage_party / hold / scout_route / not_said); move_to (place); never_<place> per place in its paragraphs.
  builder (commander, constructors): job (help_factory / follow_list / expand / not_said); attack_raiders; no_chase;
           solar (only_when_stalling / never / freely / not_said); turrets (beside_each_outer_extractor /
           beside_each_extractor / none / not_said); rebuild_lost; retreat_when_enemy_near; never_<place>.
"""
import collections
import datetime
import json
import os
import re
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, moments_of  # noqa: E402
from jev_audit import Match, clock  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GROUP = re.compile(r"\bgroup_[A-Z][A-Z0-9]*\b")
PLACE = re.compile(r"\b(?:spot_\d+|passage_\d+|home|shelling|enemy_base)\b")
BUILDERS = ("commander", "constructors")


def choice(instructions, criteria):
    return {"type": "choice", "instructions": instructions, "criteria": criteria}


def noul(instructions):
    return {"type": "noul", "instructions": instructions}


def packets_of(match_dir):
    """The player's instruct packets and its marks, in game order."""
    out, marks, frame = [], [], 0
    path = os.path.join(match_dir, "strategist-0.jsonl")
    for line in open(path):
        try:
            r = json.loads(line)
        except json.JSONDecodeError:
            continue
        if r.get("kind") == "turn":
            frame = r.get("frame", frame)
        if r.get("kind") != "tool_call":
            continue
        a = r.get("arguments", {})
        calls = a.get("calls") if isinstance(a, dict) and "calls" in a else [{"tool": r.get("tool"), "arguments": a}]
        for c in calls:
            if c.get("tool") == "mark" and isinstance(c.get("arguments"), dict):
                marks.extend(k for k in c["arguments"] if k != "arguments")
            if c.get("tool") == "instruct":
                text = (c.get("arguments") or {}).get("text")
                if text:
                    out.append({"frame": frame, "text": text, "marks": list(dict.fromkeys(marks))})
    return out


def paragraphs_about(text, actor):
    """The packet's paragraphs that name the actor (constructors: any paragraph naming a constructor)."""
    pat = re.compile(r"\bconstructor" if actor == "constructors" else re.escape(actor), re.I if actor in BUILDERS else 0)
    return [p for p in re.split(r"\n\s*\n", text) if pat.search(p)]


def places_in(text, marks):
    found = list(dict.fromkeys(PLACE.findall(text)))
    found += [m for m in marks if re.search(rf"\b{re.escape(m)}\b", text) and m not in found]
    return found


def questions_for(packet):
    """One extraction question per (actor, rule), the packet as the state."""
    text, marks = packet["text"], packet["marks"]
    groups = list(dict.fromkeys(GROUP.findall(text)))
    qs = {}
    for g in groups:
        paras = paragraphs_about(text, g)
        places = places_in(" ".join(paras), marks)
        place_opts = {p: f"the place {p}" for p in places}
        qs[f"{g}.station"] = choice(f"Read `packet`. Where does it tell {g} to stand, gather, hold or be stationed? `none` when it names no such place for {g}.", {**place_opts, "none": f"the packet names no place for {g} to stand"})
        qs[f"{g}.raiders"] = choice(
            f"Read `packet`. When an enemy raider party (a scout car, Tick, Pawn or the like) appears at one of our extractors, turrets or constructors near {g}, what does the packet tell {g} to do?",
            {"whole_group": f"{g} attacks it (engages, attacks on sight) as a group", "detachment": f"{g} sends a detachment (send_against, a few soldiers) against it and the rest stay", "forbidden": f"the packet tells {g} not to answer raiders (it holds, keeps its walk, or never chases them)", "not_said": f"the packet says nothing about raiders for {g}"},
        )
        qs[f"{g}.detachment_size"] = choice(f"Read `packet`. If it tells {g} to send a detachment against a raider, how many soldiers does it say go?", {"1": "one soldier", "2": "two", "4": "four", "half": "half the group", "not_said": "no size is said, or no detachment is ordered"})
        qs[f"{g}.no_chase"] = noul(f"Read `packet`. Does it forbid {g} to chase raiders far, or tell it never to leave its place after them?")
        qs[f"{g}.no_detachments"] = noul(f"Read `packet`. Does it forbid {g} to send detachments or to split?")
        qs[f"{g}.fall_back"] = choice(f"Read `packet`. When does it let or tell {g} to fall back or retreat?", {"outweighed": f"when a party outweighs {g}", "never_while_even_or_better": f"it tells {g} not to fall back while the fight is even or better", "not_said": "the packet does not say"})
        qs[f"{g}.fall_back_to"] = choice(f"Read `packet`. Where does it tell {g} to fall back to, if it says?", {**place_opts, "home": "home", "not_said": "not said"})
        qs[f"{g}.move"] = choice(
            f"Read `packet`. What movement, if any, does it give {g} now?",
            {"advance_and_fight": f"{g} advances to a place fighting on the way (fight_to, attacks a place)", "walk": f"{g} walks to a place without fighting (move_to, runs)", "engage_party": f"{g} engages a named enemy party", "hold": f"{g} stands where it is", "scout_route": f"{g} walks a route of places to look (a scout)", "not_said": f"no movement is given to {g}"},
        )
        qs[f"{g}.move_to"] = choice(f"Read `packet`. To which place is {g} sent first, if it is sent anywhere?", {**place_opts, "not_said": "not sent anywhere, or no place named"})
        for p in places:
            qs[f"{g}.never_{p}"] = noul(f"Read `packet`. Does it forbid {g} to go to, stand at or advance to {p} (never, no soldier stands at, does not go)?")
    for b in BUILDERS:
        paras = paragraphs_about(text, b)
        if not paras:
            continue
        who = "the commander" if b == "commander" else "a constructor"
        places = places_in(" ".join(paras), marks)
        qs[f"{b}.job"] = choice(f"Read `packet`. What is {who}'s standing job when it has nothing else to do?", {"help_factory": "help (assist, guard) the factory or plant", "follow_list": "follow its build list from the player", "expand": "take free metal spots, build extractors", "not_said": "the packet does not say"})
        qs[f"{b}.attack_raiders"] = noul(f"Read `packet`. Does it tell {who} to attack a raider party at one of our buildings near it (one it outweighs)?")
        qs[f"{b}.no_chase"] = noul(f"Read `packet`. Does it tell {who} never to chase scout cars, Ticks or raiders?")
        qs[f"{b}.solar"] = choice(f"Read `packet`. When may {who} build a solar collector or generator?", {"only_when_stalling": "only when energy reads STALLING or is low", "never": "never, no more solars", "freely": "as it sees fit, or a number of them", "not_said": "the packet does not say"})
        qs[f"{b}.turrets"] = choice(f"Read `packet`. Where does it tell {who} to build light turrets?", {"beside_each_outer_extractor": "beside each outer or far extractor, or each pair", "beside_each_extractor": "beside every extractor", "none": "no turrets, or it forbids them", "not_said": "the packet does not say"})
        qs[f"{b}.rebuild_lost"] = noul(f"Read `packet`. Does it tell {who} to rebuild lost extractors?")
        qs[f"{b}.retreat_when_enemy_near"] = noul(f"Read `packet`. Does it tell {who} to walk back toward home or the commander when enemy soldiers are near?")
        for p in places:
            qs[f"{b}.never_{p}"] = noul(f"Read `packet`. Does it forbid {who} to go to, walk past or build at {p}?")
    return qs


def top(answer):
    if answer.get("type") == "noul" or "noul" in answer:
        return ("yes" if answer["noul"] >= 0.5 else "no", max(answer["noul"], 1 - answer["noul"]))
    probs = answer.get("probabilities") or {}
    c = answer.get("choice") or (max(probs, key=probs.get) if probs else None)
    return (c, probs.get(c, 0.0))


def extract(match_dir, key, model, limit):
    label = os.path.basename(os.path.dirname(match_dir.rstrip("/")))
    rows = []
    for i, packet in enumerate(packets_of(match_dir)):
        if limit and i >= limit:
            break
        qs = questions_for(packet)
        if not qs:
            continue
        response = ask(key, {"packet": packet["text"]}, qs, model)
        answers = response.get("answers") or {}
        rules = {qid: top(a) for qid, a in answers.items()}
        rows.append({"match": label, "frame": packet["frame"], "clock": clock(packet["frame"]), "text": packet["text"], "n_questions": len(qs), "rules": rules, "usage": response.get("usage")})
        print(f"  {label} {clock(packet['frame'])}: {len(qs)} questions", file=sys.stderr)
    return rows


def consistency(rows):
    """Same paragraphs for an actor in two consecutive packets: the same rules?"""
    same = agree = 0
    by_match = collections.defaultdict(list)
    for r in rows:
        by_match[r["match"]].append(r)
    for match_rows in by_match.values():
        for a, b in zip(match_rows, match_rows[1:]):
            actors = {k.split(".")[0] for k in b["rules"]}
            for actor in actors:
                if paragraphs_about(a["text"], actor) != paragraphs_about(b["text"], actor) or not paragraphs_about(b["text"], actor):
                    continue
                for qid, (val, _) in b["rules"].items():
                    if qid.startswith(actor + ".") and qid in a["rules"]:
                        same += 1
                        agree += a["rules"][qid][0] == val
    return agree, same


def coverage(rows, match_dirs):
    """At the detectors' moments, the rule extracted from the packet in force for that actor."""
    out = collections.defaultdict(lambda: collections.Counter())
    by_match = collections.defaultdict(list)
    for r in rows:
        by_match[r["match"]].append(r)
    for d in match_dirs:
        m = Match(d)
        label = os.path.basename(os.path.dirname(d.rstrip("/")))
        packets = by_match.get(label) or []
        for moment in moments_of(m):
            kind, actor, f = moment["detector"], moment["actor"], moment["f"]
            if kind not in ("raider_ignored", "never_split", "shelling_over_named", "hold_beside_attack"):
                continue
            in_force = [p for p in packets if p["frame"] <= f]
            if not in_force:
                out[kind]["no packet yet"] += 1
                continue
            rules = in_force[-1]["rules"]
            if kind in ("raider_ignored", "hold_beside_attack"):
                v = rules.get(f"{actor}.raiders", ("absent", 0))[0]
                out[kind][f"raiders={v}"] += 1
            elif kind == "never_split":
                v = rules.get(f"{actor}.no_detachments", ("absent", 0))[0]
                out[kind][f"no_detachments={v}"] += 1
            elif kind == "shelling_over_named":
                v = rules.get(f"{actor}.never_shelling", ("absent", 0))[0]
                out[kind][f"never_shelling={v}"] += 1
    return out


def dump(rows, path):
    with open(path, "w") as f:
        for r in rows:
            f.write(f"=== {r['match']} {r['clock']} ({r['n_questions']} questions)\n{r['text']}\n--- rules\n")
            by_actor = collections.defaultdict(list)
            for qid, (val, p) in sorted(r["rules"].items()):
                actor, rule = qid.split(".", 1)
                if rule.startswith("never_") and val == "no":
                    continue
                by_actor[actor].append(f"{rule}={val} ({p:.2f})")
            for actor, items in by_actor.items():
                f.write(f"  {actor}: {', '.join(items)}\n")
            f.write("\n")


def summarize(rows, match_dirs):
    n = len(rows)
    qs = sum(r["n_questions"] for r in rows)
    toks = sum((r.get("usage") or {}).get("input_tokens", 0) for r in rows)
    print(f"{n} packets, {qs} questions, {toks} input tokens (${toks * 0.042 / 1e6:.3f})")
    hedged = collections.Counter()
    total = collections.Counter()
    for r in rows:
        for qid, (val, p) in r["rules"].items():
            rule = qid.split(".", 1)[1]
            rule = "never_<place>" if rule.startswith("never_") else rule
            total[rule] += 1
            hedged[rule] += p < 0.6
    print("hedged (p_top under 0.6) by rule:")
    for rule, t in sorted(total.items()):
        print(f"  {rule:24} {hedged[rule]:4} of {t:4} ({100 * hedged[rule] / t:.0f}%)")
    agree, same = consistency(rows)
    print(f"consistency: identical paragraphs in consecutive packets gave the same rule {agree} of {same} ({100 * agree / max(same, 1):.0f}%)")
    if match_dirs:
        print("coverage at the detectors' moments (the rule extracted from the packet in force):")
        for kind, counts in coverage(rows, match_dirs).items():
            print(f"  {kind:20} {dict(counts.most_common())}")


def main():
    args = sys.argv[1:]
    def opt(name, default=None):
        if name in args:
            i = args.index(name)
            v = args[i + 1]
            del args[i:i + 2]
            return v
        return default
    limit = int(opt("--limit", 0))
    out = opt("--out", f"docs/studies/data/decompress-{datetime.date.today().isoformat()}.jsonl")
    summ = opt("--summarize")
    do_dump = "--dump" in args
    if do_dump:
        args.remove("--dump")
    match_dirs = [a for a in args if not a.startswith("--")]
    if summ:
        rows = [json.loads(l) for l in open(summ)]
        summarize(rows, match_dirs)
        if do_dump:
            dump(rows, summ.replace(".jsonl", ".txt"))
        return
    key = api_key()
    model = os.environ.get("TYPESAFE_DEFAULT_MODEL", "jev-latest")
    rows = []
    for d in match_dirs:
        rows.extend(extract(d, key, model, limit))
    with open(out, "w") as f:
        for r in rows:
            f.write(json.dumps(r) + "\n")
    print(f"wrote {out}")
    if do_dump:
        dump(rows, out.replace(".jsonl", ".txt"))
        print(f"dump: {out.replace('.jsonl', '.txt')}")
    summarize(rows, match_dirs)


if __name__ == "__main__":
    main()
