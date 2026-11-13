#!/usr/bin/env python3
"""Offline: the picks of a game in which a party had an answer or was lent at, asked again as recorded and with
their worlds' lines said as the joint pricing says them (`docs/design/2026-11-11-groups-as-slots.md` §5.1): the
answer to a party once and whole on every world's line, world 1 among them, its verdict set beside world 1's; a
lend in its short words; no world that both withdraws an answer and lends to it. Nothing is played. The key is
read, never printed.

    run/jev_answer_ab.py run/matches/<batch>/<NN> [...] [--per-kind N] [--workers W] [--out FILE.jsonl]
                         [--list] [--show N] [--summarize FILE.jsonl]

The lines are rebuilt from the log, not by the bot (the menus are not in it): a lend's soldiers, times, verdict
and catch from its recorded words; the standing answer's place from world 1's `Not met` and `Met`; its soldiers
by lender from its entry in the state or the withdrawal's words; its own verdict from the last lend line that
priced as many soldiers at that party, else from its entry's `enemies_near`. A pick whose lines cannot be read
that way is counted and left out. Two lenders at one party in one world are priced by the recorded verdict of as
many soldiers when one is known, else by the better of the two (marked `approx`).

Kinds of component: `top-up` (world 1 has an answer at the party and a world lends more to it), `withdraw` (a
world withdraws an answer), `first` (a world lends at a party nobody answers: the control, where the new lines
should move nothing).
"""
import collections
import concurrent.futures
import datetime
import json
import os
import random
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of  # noqa: E402
from match_read import Match, clock  # noqa: E402

PLINE = r"party_\d+\w* \((?:[^()]|\((?:[^()]|\([^()]*\))*\))*\)"
KINDS = r"\d+ \w+(?:, \d+ \w+)*"
CATCH = r"; they can catch it|; it outruns them: it leaves when they arrive, or stands and dies|; its speed is unknown \(radar contacts only\)"
LEND = re.compile(
    rf"(?P<kinds>{KINDS}) of (?P<lender>group_\w+) \(its soonest to it(?P<coming>, \d+ of them still on the way to group_\w+)?\) answer "
    rf"(?P<party>party_\d+\w*) \((?P<pwords>.*?)\), the first there in (?P<first>\d+) s and the last in (?P<last>\d+) s: "
    rf"(?:with the (?P<k>\d+) already answering it|against them alone), (?P<verdict>[^;,:]+).*?(?P<catch>{CATCH})"
    rf"; they stay of no group of their own and go back when it is over; the rest of (?P=lender) keeps its course"
    rf"(?P<failed>; the last hunt of it ended without a kill: .*?)?(?P<already>; already on it: .*?)?"
    rf"(?=; {KINDS} of group_\w+ \(its soonest|; group_answer_\w+ \(|$)"
)
WITHDRAW = re.compile(r"(?P<actor>group_answer_(?P<party>party_\d+\w*)) \((?P<what>[^;]*); (?P<from>[^)]*)\) withdraws from (?P=party): the answer ends and its soldiers go back to the groups with room for them")
NOT_MET = re.compile(
    rf"(?P<pline>{PLINE}): (?P<actor>group_answer_(?P<party>party_\d+\w*)) is told to attack it, but nothing of it has the party in reach "
    r"\((?:it is|the nearest of its (?P<k>\d+) soldiers is) (?P<way>.*?) away(?P<idle>, on its way|, standing with no order|, every one of them standing with no order|, some of them standing with no order)\)"
    r"(?P<catch>; the party outruns it: it reaches the party only where the party stands still|; it is faster than the party)?"
)
MET = re.compile(rf"(?P<pline>{PLINE}) met by (?P<actor>group_answer_(?P<party>party_\d+\w*))(?P<others>(?: and \w+)*)")
SENT = re.compile(rf"(?P<pline>{PLINE}) by (?P<who>group_\w+(?: and group_\w+)*)")
ARMS = ("recorded", "again", "joint", "joint2")
RANK = ["it outweighs us", "an even fight", "we outweigh it narrowly", "we outweigh it", "we outweigh it heavily"]
WHOLE = {"outweighs it heavily": "we outweigh it heavily", "outweighs it": "we outweigh it", "outweighs it narrowly": "we outweigh it narrowly", "matches it": "an even fight", "is outweighed by it": "it outweighs us"}


class Unread(Exception):
    pass


def parts(line):
    """A line's sentences by their head: the moves, then `Met`, `Sent at`, `Not met`, and the rest in order."""
    head, _, rest = line.rstrip(".").partition(". ")
    out = {"head": head, "Met": [], "Sent at": [], "Not met": [], "rest": []}
    # The sentences after the head: split where a known head begins.
    pieces = re.split(r"\. (?=Met: |Sent at: |Not met: |The player's instructions forbid )", rest) if rest else []
    for piece in pieces:
        for key in ("Met", "Sent at", "Not met"):
            if piece.startswith(key + ": "):
                out[key].append(piece[len(key) + 2:])
                break
        else:
            out["rest"].append(piece)
    return out


def soldiers_in(entry):
    """How many soldiers an actor's entry in the state says it has, and how many are coming to it."""
    text = entry if isinstance(entry, str) else json.dumps(entry)
    body = re.search(r"\((\d+) soldiers? worth", text)
    coming = re.search(r"(\d+) more soldiers? (?:is|are) coming to it", text)
    return (int(body.group(1)) if body else None, int(coming.group(1)) if coming else 0)


def plural(n):
    return f"{n} soldier" + ("" if n == 1 else "s")


class Game:
    """One game's picks, with what the lines of earlier picks said of an answer of so many soldiers at a party."""

    def __init__(self, path):
        self.match = Match(path)
        self.label = self.match.label
        self.priced = {}  # (party, soldiers) -> verdict, from the lend lines of the picks so far
        self.picks = []
        for c, state, _ in requests_of(self.match):
            comps = {k: q for k, q in c["questions"].items() if k.startswith("worlds.c") and k.endswith(".pick")}
            if not comps:
                continue
            self.picks.append((c, state, dict(self.priced)))
            for q in comps.values():
                for line in q["criteria"].values():
                    for m in LEND.finditer(parts(line)["head"]):
                        n = sum(int(x.split()[0]) for x in m["kinds"].split(", "))
                        self.priced[(m["party"], int(m["k"] or 0) + n)] = m["verdict"].strip()

    def rewritten(self, c, state, priced, variant="joint"):
        """The call's questions with every component's lines said the new way, and what each component is.
        `joint2`: the lend keeps the rung's own words for the party (its heading), a world that lends names the
        party by its name alone in the answer's clause, and the lent ones' arrival is set beside the arrival of
        those already on their way."""
        two = variant == "joint2"
        actors = state.get("actors") or {}
        questions, kinds = {}, {}
        for qid, q in c["questions"].items():
            if not (qid.startswith("worlds.c") and qid.endswith(".pick")):
                questions[qid] = q
                continue
            lines = q["criteria"]
            base = parts(lines["w1"])
            # The answers world 1 has: by party, the actor, its soldiers, where they are, whether they catch it.
            standing = {}
            kept_not_met, kept_met = [], []
            for entry in base["Not met"]:
                m = NOT_MET.fullmatch(entry) or (NOT_MET.match(entry) if "group_answer_" in entry else None)
                if not m:
                    kept_not_met.append(entry)
                    continue
                if m.end() != len(entry):
                    raise Unread("an answer and another actor not met at one party")
                k = int(m["k"] or 1)
                where = f"it has not the party in reach yet (it is {m['way']} away{m['idle']})" if k == 1 else f"none has the party in reach yet (the nearest is {m['way']} away{m['idle']})"
                seconds = re.search(r"about (\d+) s of walking", m["way"])
                eta = int(seconds.group(1)) if seconds else 3 if "a few seconds" in m["way"] else None
                standing[m["party"]] = {"actor": m["actor"], "pline": m["pline"], "k": k, "where": where, "catch": m["catch"] or "", "eta": eta}
            for entry in base["Met"]:
                m = MET.fullmatch(entry)
                if not m:
                    kept_met.append(entry)
                    continue
                if m["others"]:
                    kept_met.append(f"{m['pline']} met by {m['others'][5:]}")
                body, _ = soldiers_in(actors.get(m["actor"], ""))
                reach = re.search(rf"{m['actor']} on it: (?:(\d+) of its (\d+) soldiers have the party in reach|it has the party in reach)", json.dumps(self.gate_of(c)))
                if body is None and reach and reach.group(2):
                    body = int(reach.group(2))
                if body is None:
                    raise Unread("an answer in reach of unknown size")
                inside = int(reach.group(1)) if reach and reach.group(1) else body
                where = "it has the party in reach" if body == 1 else f"{inside} of them have the party in reach"
                standing[m["party"]] = {"actor": m["actor"], "pline": m["pline"], "k": body, "where": where, "catch": "", "eta": None}
            for party, a in standing.items():
                entry = actors.get(a["actor"])
                text = entry if isinstance(entry, str) else json.dumps(entry or "")
                lent = re.search(r"an answer made of lent soldiers \(([^)]*)\)", text)
                withdrawal = next((w for line in lines.values() for w in WITHDRAW.finditer(line) if w["party"] == party), None)
                a["from"] = lent.group(1) if lent else withdrawal["from"] if withdrawal else f"{a['k']} lent"
                whole = re.search(rf"{party} \([^)]*\)[^\"]*?the whole group, {a['k']} soldiers[^\"]*?against the party \([^)]*\) ({'|'.join(WHOLE)}) \(", text)
                near = re.search(rf"{party} \([^)]*\) \d+ from its nearest soldier, within 800 of {a['k']} of its {a['k']}: ([^;,:]+)", text)
                a["verdict"] = priced.get((party, a["k"])) or (WHOLE[whole.group(1)] if whole else None) or (near.group(1).strip() if near else None)
                if a["verdict"] is None:
                    raise Unread("an answer whose own verdict the log does not give")

            def clause(a, lends, approx):
                if a is None:
                    (lender, l), = lends if len(lends) == 1 else [(None, None)]
                    count = sum(l["n"] for _, l in lends)
                    first, last = min(l["first"] for _, l in lends), max(l["last"] for _, l in lends)
                    place = f"there in {first} s" if first == last else f"the first there in {first} s and the last in {last} s"
                    verdict = lends[0][1]["verdict"] if len(lends) == 1 else approx
                    who = ", ".join(f"{l['n']} from {lender}" for lender, l in lends)
                    name = lends[0][1]["party"] if two else lends[0][1]["pline"]
                    return f"The answer to {name}: {plural(count)} ({who}), {place}: {verdict}{lends[0][1]['catch']}"
                if not lends:
                    return f"{a['actor']}, the answer to {a['pline']}: {plural(a['k'])} ({a['from']}), {a['where']}: {a['verdict']}{a['catch']}"
                count = a["k"] + sum(l["n"] for _, l in lends)
                first, last = min(l["first"] for _, l in lends), max(l["last"] for _, l in lends)
                place = f"there in {first} s" if first == last else f"there in {first} to {last} s"
                verdict = lends[0][1]["verdict"] if len(lends) == 1 else approx
                beside = ", as in w1" if verdict == a["verdict"] else f" (in w1, with its {a['k']}: {a['verdict']})"
                who = ", ".join([a["from"]] + [f"{l['n']} more from {lender}" for lender, l in lends])
                if two and a["eta"] is not None:
                    place += ", sooner than those on their way" if first < a["eta"] - 2 else ", no sooner than those on their way"
                name = lends[0][1]["party"] if two else a["pline"]
                return f"{a['actor']}, the answer to {name}: {plural(count)} ({who}), {a['where']}; the ones lent now {place}: {verdict}{beside}{lends[0][1]['catch']}"

            def said(key, line):
                p = parts(line)
                if key == "w1":
                    out = [p["head"]] + [clause(a, [], None) for a in standing.values()]
                    if kept_met:
                        out.append("Met: " + "; ".join(kept_met))
                    out += ["Sent at: " + s for s in p["Sent at"]]
                    if kept_not_met:
                        out.append("Not met: " + "; ".join(kept_not_met))
                    return ". ".join(out + p["rest"]) + ".", set(), set(), 0
                head = p["head"]
                lends = collections.defaultdict(list)
                for m in list(LEND.finditer(head)):
                    n = sum(int(x.split()[0]) for x in m["kinds"].split(", "))
                    body, coming = soldiers_in(actors.get(m["lender"], ""))
                    keeps = f" with its other {body + coming - n}" if body is not None and body + coming >= n else ""
                    verb = "join the answer to" if m["party"] in standing else "answer"
                    pline = next((s["pline"] for sent in p["Sent at"] for s in [SENT.fullmatch(sent)] if s and s["pline"].startswith(m["party"] + " (")), None)
                    if pline is None:
                        raise Unread("a lend with no `Sent at` to take the party's line from")
                    lends[m["party"]].append((m["lender"], {"n": n, "first": int(m["first"]), "last": int(m["last"]), "verdict": m["verdict"].strip(), "catch": m["catch"], "pline": pline, "party": m["party"], "k": int(m["k"] or 0)}))
                    aimed = f"{m['party']} ({m['pwords']})" if two else m["party"]
                    head = head.replace(m.group(0), f"{m['kinds']} of {m['lender']} (its soonest to it{m['coming'] or ''}) {verb} {aimed}; {m['lender']} keeps its course{keeps}{m['failed'] or ''}")
                withdrawn = {w["party"] for w in WITHDRAW.finditer(head)}
                if withdrawn & set(lends):
                    return None, set(lends), withdrawn, 0  # a clash: the new hands offer no such world
                if "its soonest to it" in head and not lends:
                    raise Unread("a lend's words not read")
                out = [head]
                approx = 0
                for party, mine in lends.items():
                    a = standing.get(party)
                    pooled = None
                    if len(mine) > 1:
                        total = (a["k"] if a else 0) + sum(l["n"] for _, l in mine)
                        pooled = priced.get((party, total))
                        if pooled is None:
                            pooled = max((l["verdict"] for _, l in mine), key=lambda v: RANK.index(v) if v in RANK else -1)
                            approx += 1
                    out.append(clause(a, mine, pooled))
                out += [f"No answer to {party} then" for party in withdrawn]
                for sent in p["Sent at"]:
                    s = SENT.fullmatch(sent)
                    if not s:
                        raise Unread("a `Sent at` not read")
                    # Lenders are in the answer's clause; a whole group sent at the party stays.
                    lenders = {lender for mine in lends.values() for lender, _ in mine}
                    rest = [w for w in s["who"].split(" and ") if w not in lenders]
                    if rest:
                        out.append(f"Sent at: {s['pline']} by {' and '.join(rest)}")
                if p["Met"]:
                    out.append("Met: " + "; ".join(p["Met"]))
                if p["Not met"]:
                    out.append("Not met: " + "; ".join(p["Not met"]))
                return ". ".join(out + p["rest"]) + ".", set(lends), withdrawn, approx

            new, lends_of, withdraws_of, approx = {}, {}, {}, 0
            for key, line in lines.items():
                text, lent, withdrawn, a = said(key, line)
                lends_of[key], withdraws_of[key] = lent, withdrawn
                approx += a
                if text is not None:
                    new[key] = text
            lent_at = set().union(*lends_of.values())
            kind = []
            if lent_at & set(standing):
                kind.append("top-up")
            if set().union(*withdraws_of.values()):
                kind.append("withdraw")
            if lent_at - set(standing):
                kind.append("first")
            if kind:
                kinds[qid] = {"kinds": kind, "lends": {k: sorted(v) for k, v in lends_of.items()}, "withdraws": {k: sorted(v) for k, v in withdraws_of.items()}, "standing": sorted(standing), "approx": approx, "dropped": [k for k in lines if k not in new]}
            questions[qid] = {**q, "criteria": new}
        return questions, kinds

    def gate_of(self, c):
        """The gate's questions of the pick's second: the party's `answer` question says who is on it and how."""
        for other in self.match.calls:
            if other.get("f") == c["f"] and other.get("t") == "call" and isinstance(other.get("questions"), dict) and not any(k.startswith("worlds.") for k in other["questions"]):
                return other["questions"]
        return {}


def moments(games, per_kind, seed=11):
    found, unread = [], collections.Counter()
    for game in games:
        for c, state, priced in game.picks:
            try:
                questions, kinds = game.rewritten(c, state, priced)
                second, _ = game.rewritten(c, state, priced, "joint2")
            except Unread as why:
                unread[str(why)] += 1
                continue
            if kinds:
                found.append({"game": game.label, "f": c["f"], "call": c, "state": state, "questions": questions, "joint2": second, "kinds": kinds})
    random.seed(seed)
    random.shuffle(found)
    chosen, count = [], collections.Counter()
    for m in found:
        mine = {k for v in m["kinds"].values() for k in v["kinds"]}
        if any(count[k] < per_kind for k in mine):
            chosen.append(m)
            for k in mine:
                count[k] += 1
    chosen.sort(key=lambda m: (m["game"], m["f"]))
    return found, chosen, unread


def summarize(rows):
    """Per kind of component and arm: where the pick's mass went."""
    table = collections.defaultdict(lambda: collections.defaultdict(list))
    for r in rows:
        for qid, info in r["kinds"].items():
            for arm in ARMS:
                answer = (r["answers"].get(arm) or {}).get(qid) or {}
                p = answer.get("probabilities") or {}
                if not p:
                    continue
                total = sum(p.values()) or 1.0
                for kind in info["kinds"]:
                    at = set(info["standing"]) if kind == "top-up" else None
                    lend = sum(v for k, v in p.items() if (set(info["lends"].get(k, [])) & at if at is not None else info["lends"].get(k))) / total
                    withdraw = sum(v for k, v in p.items() if info["withdraws"].get(k)) / total
                    cell = table[kind][arm]
                    cell.append({"w1": p.get("w1", 0.0) / total, "lend": lend, "withdraw": withdraw, "top": answer.get("choice"), "top_lends": bool(info["lends"].get(answer.get("choice"))), "top_withdraws": bool(info["withdraws"].get(answer.get("choice")))})
    for kind in ("top-up", "withdraw", "first"):
        print(f"\n{kind}: {len(table[kind]['again'])} components")
        print(f"  {'arm':9} {'mass on w1':>11} {'on a lend':>10} {'on a withdrawal':>16} {'top is a lend':>14} {'top withdraws':>14}")
        for arm in ARMS:
            cells = table[kind][arm]
            if not cells:
                continue
            mean = lambda key: sum(c[key] for c in cells) / len(cells)  # noqa: E731
            print(f"  {arm:9} {mean('w1'):11.2f} {mean('lend'):10.2f} {mean('withdraw'):16.2f} {mean('top_lends'):14.2f} {mean('top_withdraws'):14.2f}")


def main():
    args = sys.argv[1:]
    if "--summarize" in args:
        summarize([json.loads(line) for line in open(args[args.index("--summarize") + 1])])
        return
    value = lambda flag, default: args[args.index(flag) + 1] if flag in args else default  # noqa: E731
    per_kind, workers = int(value("--per-kind", "120")), int(value("--workers", "8"))
    out = value("--out", os.path.join("docs", "studies", "data", f"answer-pricing-{datetime.date.today().isoformat()}.jsonl"))
    flags = {"--per-kind", "--workers", "--out", "--show"}
    paths = [a for i, a in enumerate(args) if not a.startswith("--") and (i == 0 or args[i - 1] not in flags)]
    games = [Game(p) for p in paths]
    found, chosen, unread = moments(games, per_kind)
    by_kind = collections.Counter(k for m in found for v in m["kinds"].values() for k in v["kinds"])
    print(f"{sum(len(g.picks) for g in games)} picks; {len(found)} with an answer or a lend in them: {dict(by_kind)}; not read: {dict(unread)}; asking {len(chosen)}")
    if "--show" in args:
        for m in chosen[: int(value("--show", "3"))]:
            print(f"\n=== {m['game']} {clock(m['f'])}")
            for qid, info in m["kinds"].items():
                print(f"  {qid}: {info['kinds']}, approx {info['approx']}, dropped {info['dropped']}")
                for key, line in m["call"]["questions"][qid]["criteria"].items():
                    print(f"    recorded {key}: {line}")
                    print(f"    joint    {key}: {m['questions'][qid]['criteria'].get(key, '(no such world)')}")
    if "--list" in args or "--show" in args:
        return
    key = api_key()

    def run(job):
        m, arm = job
        questions = {"again": m["call"]["questions"], "joint": m["questions"], "joint2": m["joint2"]}[arm]
        reply = ask(key, m["state"], questions, m["call"].get("model", "jev-1.14.0")) or {}
        return m, arm, reply.get("answers") or {}, (reply.get("usage") or {}).get("input_tokens", 0)

    rows = {(m["game"], m["f"]): {"game": m["game"], "f": m["f"], "kinds": m["kinds"], "answers": {"recorded": m["call"]["answers"]}, "lines": {qid: {"recorded": m["call"]["questions"][qid]["criteria"], "joint": m["questions"][qid]["criteria"], "joint2": m["joint2"][qid]["criteria"]} for qid in m["kinds"]}} for m in chosen}
    tokens = collections.Counter()
    with concurrent.futures.ThreadPoolExecutor(workers) as pool:
        for m, arm, answers, used in pool.map(run, [(m, arm) for m in chosen for arm in ARMS[1:]]):
            rows[(m["game"], m["f"])]["answers"][arm] = answers
            tokens[arm] += used
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w") as f:
        for row in rows.values():
            f.write(json.dumps(row) + "\n")
    print(f"tokens in: {dict(tokens)}; written to {out}")
    summarize(list(rows.values()))


if __name__ == "__main__":
    main()
