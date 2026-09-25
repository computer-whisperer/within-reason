#!/usr/bin/env python3
"""Joint worlds offline (the user, 2026-09-25, after ../jev_experiments battery J: one Choice over the joined worlds of
several actors, each world carrying its computed consequence, picks the best world 93-100% of the time up to 255
worlds): at recorded moments of pianist games, the groups asked in one call are put to Jev as ONE question over the
worlds their candidate actions make, instead of one `do` question each, and the pick is scored against the detectors
of `run/jev_ab.py` (a raider ignored beside a group, a hold beside a base under attack, a forbidden detachment), the
collisions the per-actor form made (two groups on one lone raider in the same call), and the confident ordinary picks
that should not change.

    run/worlds_ab.py run/matches/<batch>/<NN> [...] [--per-detector N] [--control N] [--out FILE.jsonl] [--list]
                     [--summarize FILE.jsonl] [--dump]

A world is one action per asked group, from at most four candidates each (keep its course; fight the raider party at
our structure nearest it, whole or as a detachment sized to the party; the way back when the odds are against it).
Worlds where two groups take one small party are pruned; the count is kept under the Choice cap by trimming the
least concerned group's candidates. Each world's line says who answers which raider with what odds and arrival, what
stays unanswered and what it keeps killing, and what each group gives up. The key comes from TYPESAFE_API_KEY or
~/.config/within-reason/jev.env and is never printed or written.
"""
import collections
import datetime
import itertools
import json
import os
import re
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import GLOSSARY, api_key, ask, choice_of, moments_of, requests_of  # noqa: E402
from floor import NEVER_SPLIT  # noqa: E402
from jev_audit import Match, clock  # noqa: E402

FIGHT = ("engage", "send_against", "attack_unit")
BACK = ("fall_back", "retreat")
RAIDER_REACH = 1500.0
SMALL = 2
CAP = 64
# A question past this many characters was refused (400) at 104k: the lines are kept short and differential.
QUESTION_CHARS = 40000


def dist(a, b):
    return ((a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2) ** 0.5


def parse_units(line):
    """'a handful: 1 Rover (armfav), 3 Blitz (armflash) (4 soldiers worth 361 metal)' -> ({armfav: 1, armflash: 3}, 4, 361)."""
    comp = {m.group(2): int(m.group(1)) for m in re.finditer(r"(\d+) [A-Za-z' -]+? \(([a-z0-9]+)\)", line or "")}
    n = int(re.search(r"\((\d+) soldiers", line).group(1)) if line and re.search(r"\((\d+) soldiers", line) else sum(comp.values())
    metal = int(re.search(r"worth (\d+) metal", line).group(1)) if line and re.search(r"worth (\d+) metal", line) else 0
    return comp, n, metal


def speed_of(comp):
    speeds = [GLOSSARY.get(d, {}).get("speed") or 60 for d in comp]
    return min(speeds) if speeds else 60


def mean_unit_metal(comp, metal, n):
    return metal / max(n, 1)


def party_units(composition):
    return sum(int(m.group(1)) for m in re.finditer(r"(\d+) ", composition or "")) or 1


def odds_words(ours, theirs):
    ratio = ours / max(theirs, 1)
    if ratio >= 2.5:
        return "we outweigh it heavily"
    if ratio >= 1.3:
        return "we outweigh it"
    if ratio >= 0.8:
        return "an even fight"
    return "it outweighs us"


class Situation:
    """One call's groups, parties and raiders, read from the log line."""

    def __init__(self, c, packet):
        self.f = c["f"]
        self.packet = packet
        self.state = c["state"]
        self.questions = c["questions"]
        self.parties = {p["name"]: p for p in c.get("parties") or []}
        self.groups = {}
        actors = self.state.get("actors") or {}
        for g in c.get("groups") or []:
            name = f"group_{g['name']}"
            if f"{name}.do" not in self.questions or not g.get("at"):
                continue
            entry = actors.get(name) or {}
            comp, n, metal = parse_units(entry.get("units"))
            options = list(((self.questions[f"{name}.do"]).get("criteria") or {}).keys())
            self.groups[name] = {"name": name, "at": g["at"], "task": g.get("task") or {}, "comp": comp, "n": n, "metal": metal, "speed": speed_of(comp), "options": options, "entry": entry}
        # Raider parties at our structures: named on any group's `enemies_at_our_extractors` line, with the place.
        self.raiders = {}
        for g in self.groups.values():
            for text in g["entry"].get("enemies_at_our_extractors") or []:
                m = re.match(r"(party_\d+) \((.*?)\) at our extractor at (\S+)", text)
                if m and m.group(1) in self.parties:
                    self.raiders[m.group(1)] = {"place": m.group(3), "composition": m.group(2)}
        for g in self.groups.values():
            for p, party in self.parties.items():
                if p not in self.raiders and dist(g["at"], [party["x"], party["z"]]) <= RAIDER_REACH:
                    self.raiders[p] = {"place": None, "composition": party["composition"]}
        # What each party is killing, from the whom question's words.
        self.killing = {}
        for qid, q in self.questions.items():
            if qid.endswith(".whom"):
                for party, words in (q.get("criteria") or {}).items():
                    m = re.search(r"killing (.+?) now", str(words))
                    if m:
                        self.killing[party] = m.group(1)

    def candidates(self, g):
        """At most four: keep, one or two fights at the nearest raider, the way back."""
        opts = g["options"]
        out = []
        task = g["task"].get("kind", "hold")
        if "continue" in opts:
            out.append({"kind": "keep", "option": "continue", "words": self.course_words(g)})
        elif "hold" in opts:
            out.append({"kind": "keep", "option": "hold", "words": f"keeps holding at {g['entry'].get('at', 'where it stands')}"})
        raiders = sorted(((dist(g["at"], [self.parties[p]["x"], self.parties[p]["z"]]), p) for p in self.raiders if p in self.parties), key=lambda x: x[0])
        for d, p in raiders[:1]:
            if d > RAIDER_REACH:
                continue
            party = self.parties[p]
            units = party_units(party["composition"])
            cost = self.leave_words(g)
            if "engage" in opts:
                out.append({"kind": "fight", "option": "engage", "party": p, "metal": g["metal"], "words": f"attacks {p} with the whole group{cost}"})
            if "send_against" in opts and units <= 3 and g["n"] > 1 and not (self.packet and NEVER_SPLIT.search(self.packet)):
                n = 1 if units == 1 else (2 if units == 2 else 4)
                n = min(n, g["n"] - 1)
                out.append({"kind": "fight", "option": "send_against", "party": p, "n": n, "metal": n * mean_unit_metal(g["comp"], g["metal"], g["n"]), "words": f"sends {n} soldiers against {p}, the rest {self.course_words(g, keep=True)}"})
        against = "it outweighs us" in (g["entry"].get("enemies_near") or "") or "losing" in (g["entry"].get("losses") or "") or "wiped" in (g["entry"].get("losses") or "")
        if against:
            for o in BACK:
                if o in opts:
                    out.append({"kind": "back", "option": o, "words": ("falls back to where it last held" if o == "fall_back" else "falls back home") + self.leave_words(g)})
                    break
        if not out and opts:
            out.append({"kind": "keep", "option": opts[0], "words": f"{opts[0]}"})
        return out[:4]

    def leave_words(self, g):
        """What a group gives up by leaving its course: its hold at a place, or the walk the packet gave it."""
        task = g["task"]
        kind = task.get("kind", "hold")
        if kind in ("move_to", "fight_to"):
            return f", abandoning its {'advance' if kind == 'fight_to' else 'walk'} to {task.get('place', '?')} (the course it was given)"
        if kind == "hold":
            return f", leaving {g['entry'].get('at', 'its place')} unguarded"
        return ", leaving the party it was attacking"

    def course_words(self, g, keep=False):
        task = g["task"]
        at = g["entry"].get("at", "where it stands")
        kind = task.get("kind", "hold")
        if kind == "hold":
            return f"{'keep holding' if keep else 'keeps holding'} at {at}"
        if kind in ("move_to", "fight_to"):
            return f"{'keep' if keep else 'keeps'} {'advancing' if kind == 'fight_to' else 'walking'} to {task.get('place', '?')} (the course it was given)"
        return f"{'keep' if keep else 'keeps'} attacking its party"

    def worlds(self):
        names = list(self.groups)
        cands = {n: self.candidates(self.groups[n]) for n in names}
        # Trim the least concerned groups (no raider within reach) to their keep candidate while over the cap.
        concerned = {n: any(dist(self.groups[n]["at"], [self.parties[p]["x"], self.parties[p]["z"]]) <= RAIDER_REACH for p in self.raiders if p in self.parties) for n in names}
        order = sorted(names, key=lambda n: (concerned[n], -len(cands[n])))
        def count():
            total = 1
            for n in names:
                total *= max(1, len(cands[n]))
            return total
        for n in order:
            if count() <= CAP:
                break
            cands[n] = cands[n][:1]
        out = []
        for combo in itertools.product(*(cands[n] for n in names)):
            world = dict(zip(names, combo))
            # Two groups on one small party: pruned (the second adds nothing and abandons its own post).
            takers = collections.Counter(a["party"] for a in combo if a["kind"] == "fight")
            if any(cnt > 1 and party_units(self.parties[p]["composition"]) <= SMALL for p, cnt in takers.items()):
                continue
            out.append(world)
        return out[:CAP]

    def consequence(self, world):
        moves = []
        for name, action in world.items():
            if action["kind"] != "keep":
                moves.append(f"{name} {action['words']}")
        keeps = [n for n, a in world.items() if a["kind"] == "keep"]
        parts = []
        if moves:
            parts.append("; ".join(moves))
        if keeps:
            parts.append(f"{', '.join(keeps)} keep{'s' if len(keeps) == 1 else ''} {'its' if len(keeps) == 1 else 'their'} course")
        met, unmet = [], []
        for p, info in self.raiders.items():
            if p not in self.parties:
                continue
            party = self.parties[p]
            takers = [(n, a) for n, a in world.items() if a["kind"] == "fight" and a.get("party") == p]
            where = f"at {info['place']}" if info.get("place") else "in sight"
            if takers:
                metal = sum(a["metal"] for _, a in takers)
                arrival = max(int(dist(self.groups[n]["at"], [party["x"], party["z"]]) / max(self.groups[n]["speed"], 1)) for n, _ in takers)
                met.append(f"{p} ({party['composition']}, {party['metal']} metal, {where}) met with {metal:.0f} metal: {odds_words(metal, party['metal'])}, {arrival} s away")
            else:
                killing = self.killing.get(p)
                unmet.append(f"{p} ({party['composition']}, {where}{', killing ' + killing if killing else ''})")
        if met:
            parts.append("Met: " + "; ".join(met))
        if unmet:
            parts.append("Unanswered: " + "; ".join(unmet))
        return ". ".join(parts) + "."


LEVELS = ["a bad course: it loses something the instructions value, or sends a group into a party that outweighs it", "a poor course", "an acceptable course", "a good course", "the best course: raiders at our structures answered with enough and no more, the courses the instructions gave kept, no group into a party that outweighs it"]


def build_question(sit, form="choice"):
    """One Choice over the worlds, or (`form` score) one Score per world, argmax of the expected level in code."""
    worlds = sit.worlds()
    if len(worlds) < 2:
        return None, worlds
    criteria = {f"w{i + 1}": sit.consequence(w) for i, w in enumerate(worlds)}
    while len(json.dumps(criteria)) > QUESTION_CHARS and len(worlds) > 2:
        worlds = worlds[: max(2, len(worlds) * 2 // 3)]
        criteria = {f"w{i + 1}": sit.consequence(w) for i, w in enumerate(worlds)}
    names = ", ".join(sit.groups)
    if form == "score":
        qs = {}
        for key, line in criteria.items():
            qs[key] = {"type": "score", "instructions": f"Given `actors`, `enemy`, `player` and the player's `instructions`, how good is this joint course for {names} this second? The world: {line}", "criteria": LEVELS}
        return qs, worlds
    instructions = f"Given `actors`, `enemy`, `player` and the player's `instructions`, which joint course for {names} is best this second? Each option is one world: what every group does and what follows from it. Prefer the world that answers raiders at our structures with enough and no more, keeps the courses the instructions gave, and does not send a group into a party that outweighs it."
    return {"type": "choice", "instructions": instructions, "criteria": criteria}, worlds


def collision_calls(m):
    """Calls where two or more groups chose a fight option against the same party of two or fewer units."""
    out = {}
    for c, state, packet in requests_of(m):
        parties = {p["name"]: p for p in c.get("parties") or []}
        takers = collections.defaultdict(list)
        for qid in c["questions"]:
            if not qid.endswith(".do") or not qid.startswith("group_"):
                continue
            actor = qid[:-3]
            chosen, _ = choice_of(c, qid)
            if chosen in FIGHT:
                whom, _ = choice_of(c, f"{actor}.whom")
                if whom in parties and party_units(parties[whom]["composition"]) <= SMALL:
                    takers[whom].append(actor)
        if any(len(v) >= 2 for v in takers.values()):
            out[c["f"]] = {p: v for p, v in takers.items() if len(v) >= 2}
    return out


def gather(m, per_detector, control):
    """The calls to replay: the detectors' moments (grouped by call), the collision calls, and controls."""
    by_f = collections.defaultdict(list)
    for mo in moments_of(m):
        if mo["detector"] in ("raider_ignored", "hold_beside_attack", "never_split"):
            by_f[mo["f"]].append(mo)
    counts = collections.Counter()
    picked = {}
    for f in sorted(by_f):
        keep = [mo for mo in by_f[f] if counts[mo["detector"]] < per_detector]
        if keep:
            for mo in keep:
                counts[mo["detector"]] += 1
            picked[f] = {"moments": keep, "collisions": {}}
    for f, coll in collision_calls(m).items():
        if len([1 for p in picked.values() if p["collisions"]]) >= per_detector:
            break
        picked.setdefault(f, {"moments": [], "collisions": {}})["collisions"] = coll
    # Controls: calls with a group pick at 0.6 or more that no detector flagged.
    n = 0
    for c, state, packet in requests_of(m):
        if n >= control or c["f"] in picked:
            continue
        confident = [qid[:-3] for qid in c["questions"] if qid.endswith(".do") and qid.startswith("group_") and choice_of(c, qid)[1].get(choice_of(c, qid)[0], 0.0) >= 0.6]
        if confident:
            picked[c["f"]] = {"moments": [], "collisions": {}, "controls": confident}
            n += 1
    calls = {c["f"]: (c, state, packet) for c, state, packet in requests_of(m) if c["f"] in picked}
    return [(f, calls[f], picked[f]) for f in sorted(picked) if f in calls]


def action_kind(option):
    if option in FIGHT:
        return "fight"
    if option in BACK:
        return "back"
    if option in ("continue", "hold", "wait"):
        return "keep"
    return "go"


def replay(m, key, model, per_detector, control, dump, form="choice"):
    rows = []
    for f, (c, state, packet), what in gather(m, per_detector, control):
        sit = Situation(c, packet)
        question, worlds = build_question(sit, form)
        if question is None:
            continue
        sent = {k: v for k, v in state.items()}
        try:
            response = ask(key, sent, question if form == "score" else {"worlds": question}, model)
        except Exception as e:
            print(f"  {m.label} {clock(f)}: {len(worlds)} worlds, the call failed: {str(e)[:80]}", file=sys.stderr)
            continue
        if form == "score":
            # The expected level of each world; the pick is the highest, `p` its margin over the runner-up in levels.
            expected = {k: a.get("score", 0.0) for k, a in response["answers"].items()}
            ranked = sorted(expected.items(), key=lambda kv: -kv[1])
            pick = ranked[0][0]
            probs = {pick: (ranked[0][1] - ranked[1][1]) if len(ranked) > 1 else 1.0}
        else:
            answer = response["answers"]["worlds"]
            pick = answer["choice"]
            probs = answer.get("probabilities") or {}
        world = worlds[int(pick[1:]) - 1]
        row = {"match": m.label, "f": f, "clock": clock(f), "groups": list(sit.groups), "worlds": len(worlds), "raiders": list(sit.raiders), "pick": pick, "p": probs.get(pick, 0.0),
               "picked": {n: a["option"] + (f" {a.get('n', '')}@{a.get('party')}" if a["kind"] == "fight" else "") for n, a in world.items()},
               "recorded": {n: choice_of(c, f"{n}.do")[0] for n in sit.groups},
               "usage": response.get("usage"), "moments": [], "collisions": what.get("collisions") or {}, "controls": []}
        # A collision in the picked world: two groups on one small party (pruned, so it should never happen).
        takers = collections.Counter(a.get("party") for a in world.values() if a["kind"] == "fight")
        row["picked_collision"] = any(cnt > 1 and party_units(sit.parties[p]["composition"]) <= SMALL for p, cnt in takers.items() if p in sit.parties)
        for mo in what.get("moments") or []:
            actor = mo["actor"]
            a = world.get(actor)
            party = None
            mm = re.match(r"(party_\d+)", mo.get("note") or "")
            if mm:
                party = mm.group(1)
            answered = party is not None and any(x["kind"] == "fight" and x.get("party") == party for x in world.values())
            right = a is not None and a["option"] in mo["wanted"]
            row["moments"].append({"detector": mo["detector"], "actor": actor, "wanted": mo["wanted"], "recorded": mo["chosen"], "world_action": a["option"] if a else None, "right": right, "party_answered": answered})
        for actor in what.get("controls") or []:
            a = world.get(actor)
            rec = choice_of(c, f"{actor}.do")[0]
            row["controls"].append({"actor": actor, "recorded": rec, "world_action": a["option"] if a else None, "same_kind": a is not None and action_kind(a["option"]) == action_kind(rec)})
        rows.append(row)
        print(f"  {m.label} {clock(f)}: {len(sit.groups)} groups, {len(worlds)} worlds, {len(sit.raiders)} raiders -> {pick} ({probs.get(pick, 0):.2f}) {row['picked']}", file=sys.stderr)
        if dump:
            lines = question["criteria"].items() if form != "score" else ((k, q["instructions"].split("The world: ", 1)[-1]) for k, q in question.items())
            print(f"=== {m.label} {clock(f)}\n" + "\n".join(f"  {k}: {v}" for k, v in lines) + f"\n  -> {pick} {probs.get(pick, 0):.2f}\n", file=dump)
    return rows


def summarize(rows):
    n = len(rows)
    if not n:
        print("nothing")
        return
    toks = sum((r.get("usage") or {}).get("input_tokens", 0) for r in rows)
    print(f"{n} calls, worlds per call median {statistics.median(r['worlds'] for r in rows):.0f} (max {max(r['worlds'] for r in rows)}), groups median {statistics.median(len(r['groups']) for r in rows):.0f}; {toks} tokens (${toks * 0.042 / 1e6:.3f}); p(top) median {statistics.median(r['p'] for r in rows):.2f}")
    by = collections.defaultdict(lambda: [0, 0, 0])
    for r in rows:
        for mo in r["moments"]:
            b = by[mo["detector"]]
            b[0] += 1
            b[1] += mo["right"]
            b[2] += mo["party_answered"] or mo["right"]
    for det, (t, right, ans) in by.items():
        print(f"  {det:20} {t:3} moments: the flagged group does a wanted action {right} ({100 * right / t:.0f}%); the raider answered by some group or wanted done {ans} ({100 * ans / t:.0f}%)")
    coll = [r for r in rows if r["collisions"]]
    if coll:
        print(f"  collision calls {len(coll)}: the picked world repeats the collision in {sum(1 for r in coll if r['picked_collision'])}")
    ctrl = [c for r in rows for c in r["controls"]]
    if ctrl:
        print(f"  controls {len(ctrl)}: same kind of action as the recorded confident pick {sum(1 for c in ctrl if c['same_kind'])} ({100 * sum(1 for c in ctrl if c['same_kind']) / len(ctrl):.0f}%)")
    print(f"  any picked world with two groups on one small party: {sum(1 for r in rows if r['picked_collision'])}")


def main():
    args = sys.argv[1:]
    def opt(name, default=None):
        if name in args:
            i = args.index(name)
            v = args[i + 1]
            del args[i:i + 2]
            return v
        return default
    form = opt("--form", "choice")
    per_detector = int(opt("--per-detector", 25))
    control = int(opt("--control", 25))
    out = opt("--out", f"docs/studies/data/worlds-ab-{datetime.date.today().isoformat()}.jsonl")
    summ = opt("--summarize")
    listing = "--list" in args
    dumping = "--dump" in args
    for flag in ("--list", "--dump"):
        if flag in args:
            args.remove(flag)
    if summ:
        summarize([json.loads(l) for l in open(summ)])
        return
    dirs = [a for a in args if not a.startswith("--")]
    if listing:
        for d in dirs:
            m = Match(d)
            for f, (c, state, packet), what in gather(m, per_detector, control):
                sit = Situation(c, packet)
                q, worlds = build_question(sit)
                print(f"{m.label} {clock(f)}: groups {list(sit.groups)} raiders {list(sit.raiders)} worlds {len(worlds)} moments {[mo['detector'] for mo in what.get('moments') or []]} collisions {what.get('collisions')} controls {what.get('controls')}")
        return
    key = api_key()
    model = os.environ.get("TYPESAFE_DEFAULT_MODEL", "jev-latest")
    dump = open(out.replace(".jsonl", ".txt"), "w") if dumping else None
    rows = []
    for d in dirs:
        rows.extend(replay(Match(d), key, model, per_detector, control, dump, form))
    with open(out, "w") as f:
        for r in rows:
            f.write(json.dumps(r) + "\n")
    print(f"wrote {out}")
    summarize(rows)


if __name__ == "__main__":
    main()
