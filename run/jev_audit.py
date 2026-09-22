#!/usr/bin/env python3
"""What the hands (Jev) were asked and answered over a pianist match, and what the player saw against the truth:
the mechanical sweep behind a Jev deep dive (docs/design/2026-09-21-pianist.md). Reads `jev-<ai>.jsonl` (every
call), `record-<ai>.jsonl` (events, samples, the hands' decisions), `strategist-<ai>.jsonl` (the player's turns) and
`truth-<ai>.jsonl` (what the opponent really had).

usage: run/jev_audit.py run/matches/<batch>/<NN> [...] [--section NAME[,NAME]] [--top N]
  Several matches print each section per match and then pooled. Sections: summary, kinds, options, noop, flips,
  detach, didnot, unsure, globals, packets, view, turns, movefail (default: all).
"""
import collections
import json
import os
import re
import statistics
import sys


def clock(frame):
    seconds = int(frame) // 30
    return f"{seconds // 60}:{seconds % 60:02d}"


def first(match_dir, prefix):
    name = next((f for f in sorted(os.listdir(match_dir)) if f.startswith(prefix) and f.endswith(".jsonl")), None)
    return os.path.join(match_dir, name) if name else None


def rows(path):
    if not path:
        return
    for line in open(path):
        try:
            yield json.loads(line)
        except ValueError:
            continue


class Match:
    def __init__(self, match_dir):
        if not any(f.startswith("record-") for f in os.listdir(match_dir)) and os.path.isdir(os.path.join(match_dir, "00")):
            match_dir = os.path.join(match_dir, "00")
        self.dir = match_dir
        self.label = os.path.basename(os.path.dirname(match_dir))
        self.header = None
        self.samples = []
        self.events = []
        self.decisions = {}
        self.result = None
        self.home = None
        for r in rows(first(match_dir, "record-")):
            t = r.get("t")
            if t == "header":
                self.header = r
            elif t == "s":
                self.samples.append(r)
            elif t == "ev":
                self.events.append(r)
            elif t == "d" and r.get("source") in ("jev", "list", "policy") and isinstance(r.get("inputs"), dict):
                self.decisions[(r["f"], r["inputs"].get("actor"))] = r["outputs"] or {}
            elif t == "intent" and self.home is None:
                self.home = tuple(r["home"])
            elif t == "result":
                self.result = r["result"]
        self.defs = dict(enumerate(self.header["unit_defs"]))  # event `d` fields index this list
        self.calls = []
        self.jev_header = None
        for r in rows(first(match_dir, "jev-")):
            if r.get("t") == "header":
                self.jev_header = r
            else:
                self.calls.append(r)
        self.turns = []
        turn = None
        for r in rows(first(match_dir, "strategist-")):
            kind = r.get("kind")
            if kind == "turn":
                m = re.search(r"Woken because: ([^\n]*)", r["prompt"])
                turn = {"frame": int(r["frame"]), "prompt": r["prompt"], "why": m.group(1) if m else "?", "calls": [], "text": [], "wall": None}
                self.turns.append(turn)
            elif turn is None:
                continue
            elif kind == "tool_call":
                turn["calls"].append((r["tool"], r.get("arguments") or {}))
            elif kind == "assistant":
                for c in r["message"]["message"].get("content", []):
                    if c.get("type") == "text" and c["text"].strip():
                        turn["text"].append(c["text"].strip())
            elif kind == "turn_end":
                turn["wall"] = r.get("wall_seconds")
        self.truth = {}
        for r in rows(first(match_dir, "truth-")):
            self.truth[r["f"]] = r["enemy"]

    def def_name(self, d):
        return self.defs.get(d, {}).get("name", f"def{d}")

    def is_soldier(self, d):
        u = self.defs.get(d, {})
        return u.get("class") != "building" and u.get("weapons", 0) > 0 and u.get("build_speed", 0) == 0

    def losses(self, f0, f1, near=None):
        """Metal of our units destroyed by an enemy in [f0, f1), optionally within `near` of home."""
        total = 0.0
        for e in self.events:
            if e["k"] != "destroyed" or e.get("by") is None or not (f0 <= e["f"] < f1):
                continue
            if near is not None and self.home and ((e["x"] - self.home[0]) ** 2 + (e["z"] - self.home[1]) ** 2) ** 0.5 > near:
                continue
            total += self.defs.get(e["d"], {}).get("metal", 0)
        return total

    def def_by_name(self, name):
        if not hasattr(self, "_by_name"):
            self._by_name = {d["name"]: i for i, d in self.defs.items()}
        i = self._by_name.get(name)
        return self.defs.get(i) if i is not None else None

    def is_soldier_name(self, name):
        d = self.def_by_name(name)
        return d is not None and d.get("class") != "building" and d.get("weapons", 0) > 0 and d.get("build_speed", 0) == 0

    def metal_of(self, name):
        return (self.def_by_name(name) or {}).get("metal", 0)

    def scripted(self, f, actor):
        o = self.decisions.get((f, actor))
        return bool(o and o.get("scripted"))


# ---------------------------------------------------------------- sections

def sec_summary(m, out):
    r = m.result or {}
    ms = sorted(c.get("ms", 0) for c in m.calls)
    retries = sum(c.get("retries", 0) for c in m.calls)
    tokens = sum((c.get("usage") or {}).get("input_tokens", 0) or (c.get("usage") or {}).get("prompt_tokens", 0) for c in m.calls)
    minutes = r.get("game_minutes", 0)
    played = sum(len(c.get("played", [])) for c in m.calls)
    scripted = sum(1 for c in m.calls for p in c["played"] if m.scripted(c["f"], p["actor"]))
    packets = sum(1 for c in m.calls if "instructions" in c)
    out(f"{m.label}: {r.get('outcome')} in {minutes:.1f} min vs {r.get('opponent')}; {len(m.calls)} calls "
        f"({len(m.calls) / max(minutes, 0.1):.1f}/min), {played} menus answered ({scripted} scripted), {packets} packets; "
        f"Jev median {ms[len(ms) // 2] if ms else 0} ms, max {ms[-1] if ms else 0}, retries {retries}; "
        f"{len(m.turns)} player turns; input tokens {tokens}")


def sec_kinds(m, out):
    by = collections.defaultdict(lambda: collections.Counter())
    conf = collections.defaultdict(list)
    for c in m.calls:
        for p in c["played"]:
            k = p["kind"]
            by[k]["asks"] += 1
            if m.scripted(c["f"], p["actor"]):
                by[k]["scripted"] += 1
                continue
            if p["busy"]:
                by[k]["busy"] += 1
            if p["kept"]:
                by[k]["kept (continue held)"] += 1
            if p["played"] == "continue":
                by[k]["played continue"] += 1
            if p["did"] is None:
                by[k]["did nothing"] += 1
            if p["choice"] != p["played"]:
                by[k]["choice not played"] += 1
            conf[k].append(p["confidence"])
    for k, c in by.items():
        cs = sorted(conf[k])
        q = lambda x: cs[min(len(cs) - 1, int(len(cs) * x))] if cs else 0
        out(f"  {k:8s} " + ", ".join(f"{n} {v}" for v, n in c.items()) + f"; confidence quartiles {q(0.25):.2f}/{q(0.5):.2f}/{q(0.75):.2f}")


def option_table(matches):
    table = collections.defaultdict(lambda: {"offered": 0, "chosen": 0, "played": 0, "p": 0.0, "pmax": 0.0})
    for m in matches:
        for c in m.calls:
            for p in c["played"]:
                if m.scripted(c["f"], p["actor"]):
                    continue
                a = (c.get("answers") or {}).get(f"{p['actor']}.next" if p["kind"] == "lab" else f"{p['actor']}.do") or {}
                probs = a.get("probabilities") or {}
                for o in p["options"]:
                    row = table[(p["kind"], o)]
                    row["offered"] += 1
                    pr = probs.get(o, 0.0)
                    row["p"] += pr
                    row["pmax"] = max(row["pmax"], pr)
                    if p["choice"] == o:
                        row["chosen"] += 1
                    if p["played"] == o:
                        row["played"] += 1
    return table


def sec_options(matches, out, top):
    table = option_table(matches)
    out("  kind     option                 offered chosen played  mean p  max p   note")
    for (k, o), r in sorted(table.items(), key=lambda kv: (kv[0][0], -kv[1]["offered"])):
        mean = r["p"] / r["offered"]
        note = ""
        if r["offered"] >= 30 and r["chosen"] == 0 and mean < 0.03:
            note = "never chosen: dead weight"
        elif r["offered"] >= 30 and r["chosen"] == 0:
            note = "never chosen"
        elif r["chosen"] and r["played"] < r["chosen"] * 0.5:
            note = "chosen but mostly not played"
        out(f"  {k:8s} {o:22s} {r['offered']:7d} {r['chosen']:6d} {r['played']:6d}  {mean:6.2f}  {r['pmax']:5.2f}   {note}")


def sec_noop(m, out):
    """Asks whose answer changed nothing: a holding group told to hold, a busy actor kept on continue."""
    hold_on_hold = 0
    group_asks = 0
    tasks = {}
    for c in m.calls:
        for g in c.get("groups", []):
            tasks[f"group_{g['name']}"] = (g.get("task") or {}).get("kind")
        for p in c["played"]:
            if p["kind"] != "group":
                continue
            group_asks += 1
            if p["played"] == "hold" and tasks.get(p["actor"]) == "hold":
                hold_on_hold += 1
    kept = sum(1 for c in m.calls for p in c["played"] if p["kept"])
    cont = sum(1 for c in m.calls for p in c["played"] if p["played"] == "continue" and not p["kept"])
    out(f"  group asks {group_asks}: hold answered on a group already holding {hold_on_hold} ({100 * hold_on_hold / max(group_asks, 1):.0f}%); "
        f"all kinds: continue chosen outright {cont}, continue held by the switch margin {kept}")


DETACH = ("send_against", "split", "scout")  # these spawn a detachment and leave the group's own task alone


def sec_flips(m, out, top):
    """A, B, A within three consecutive unscripted asks of one actor: the hands changing their mind and back."""
    seq = collections.defaultdict(list)
    for c in m.calls:
        for p in c["played"]:
            if m.scripted(c["f"], p["actor"]):
                continue
            seq[p["actor"]].append((c["f"], p["played"], p["probability"]))
    flips = []
    for actor, s in seq.items():
        for i in range(len(s) - 2):
            a, b, c3 = s[i][1], s[i + 1][1], s[i + 2][1]
            if a == c3 and a != b and "continue" not in (a, b) and b not in DETACH and a not in DETACH:
                flips.append((s[i][0], actor, a, b, s[i][2], s[i + 1][2], s[i + 2][2]))
    out(f"  {len(flips)} flip-flops (A,B,A in three consecutive asks of one actor)")
    counts = collections.Counter((f[2], f[3]) for f in flips)
    for (a, b), n in counts.most_common(top):
        out(f"    {a} -> {b} -> {a}: {n}")
    for f in flips[:top]:
        out(f"    {clock(f[0])} {f[1]}: {f[2]} ({f[4]:.2f}) / {f[3]} ({f[5]:.2f}) / {f[2]} ({f[6]:.2f})")


def sec_detach(m, out, top):
    """A detachment (born of send_against or split) that folded back into its parent: how soon."""
    born = {}
    folded = []
    for c in m.calls:
        for p in c["played"]:
            did = p.get("did") or ""
            mm = re.search(r"as (group_\w+) ", did)
            if p["played"] in ("send_against", "split") and mm:
                born[mm.group(1)] = (c["f"], p["actor"], did)
            elif p["played"].startswith("join_") and p["actor"] in born:
                f0, parent, did0 = born.pop(p["actor"])
                folded.append((f0, p["actor"], parent, (c["f"] - f0) / 30, did0, p["played"]))
    sent = sum(1 for c in m.calls for p in c["played"] if p["played"] in ("send_against", "split"))
    quick = [x for x in folded if x[3] <= 30]
    out(f"  {sent} detachments sent; {len(folded)} later joined a group, {len(quick)} of them within 30 s")
    for f0, g, parent, secs, did0, join in sorted(folded, key=lambda x: x[3])[:top]:
        out(f"    {clock(f0)} {parent}: {did0[:70]} -> {g} {join} after {secs:.0f} s")


def sec_didnot(m, out, top):
    """Choices the hands could not turn into an order (`did` empty), by option."""
    c2 = collections.Counter()
    tot = collections.Counter()
    for c in m.calls:
        for p in c["played"]:
            if m.scripted(c["f"], p["actor"]) or p["played"] == "continue":
                continue
            tot[(p["kind"], p["played"])] += 1
            if p["did"] is None:
                c2[(p["kind"], p["played"])] += 1
    out("  played but nothing done: " + ", ".join(f"{k}/{o} {n} of {tot[(k, o)]}" for (k, o), n in c2.most_common(top)))


def sec_unsure(m, out, top):
    low = []
    diffuse = []
    for c in m.calls:
        for p in c["played"]:
            if m.scripted(c["f"], p["actor"]):
                continue
            if p["confidence"] < 0.4:
                low.append((c["f"], p["actor"], p["choice"], p["probability"], p["confidence"], len(p["options"])))
        for qid, a in (c.get("answers") or {}).items():
            if a.get("type") != "choice" or not qid.endswith((".where", ".where_extractor", ".where_scout", ".whom")):
                continue
            probs = a.get("probabilities") or {}
            actor = qid.rsplit(".", 1)[0]
            played = next((p["played"] for p in c["played"] if p["actor"] == actor), None)
            if probs and max(probs.values()) < 0.35 and played in ("fight_to", "move_to", "split", "extractor", "turret_at", "radar_at", "walk_to", "send_against", "scout"):
                diffuse.append((c["f"], qid, a["choice"], max(probs.values()), len(probs), played))
    asks = sum(1 for c in m.calls for p in c["played"] if not m.scripted(c["f"], p["actor"]))
    out(f"  {len(low)} of {asks} unscripted menu answers below confidence 0.4; {len(diffuse)} place/target answers with no option above 0.35 that were then played")
    for f, actor, ch, pr, cf, n in low[:top]:
        out(f"    {clock(f)} {actor}: {ch} p={pr:.2f} conf={cf:.2f} of {n} options")
    for f, qid, ch, mx, n, played in diffuse[:top]:
        out(f"    {clock(f)} {qid}: {ch} top={mx:.2f} of {n} places, played {played}")


def sec_globals(m, out):
    """Are the three global Nouls worth anything? Each binned against what happened in the next minute."""
    bins = [(0.0, 0.3), (0.3, 0.6), (0.6, 1.01)]
    stats = {q: {b: [] for b in bins} for q in ("attack_coming", "base_in_danger", "needs_player")}
    wake_frames = [t["frame"] for t in m.turns if "hands say" in t["why"]]
    all_turns = [t["frame"] for t in m.turns]
    for c in m.calls:
        f = c["f"]
        ans = c.get("answers") or {}
        outcomes = {
            "attack_coming": m.losses(f, f + 1800),
            "base_in_danger": m.losses(f, f + 1800, near=1500),
            "needs_player": 1.0 if any(f < t <= f + 1800 for t in all_turns) else 0.0,
        }
        for q, val in outcomes.items():
            a = ans.get(f"global.{q}")
            if not a:
                continue
            p = a["noul"]
            for b in bins:
                if b[0] <= p < b[1]:
                    stats[q][b].append((p, val))
    for q, per in stats.items():
        line = []
        for b, vals in per.items():
            if vals:
                mean = statistics.mean(v for _, v in vals)
                unit = "turn followed" if q == "needs_player" else "metal lost in 60 s"
                line.append(f"p {b[0]:.1f}-{min(b[1], 1.0):.1f}: {len(vals)} calls, mean {mean:.2f} {unit}")
            else:
                line.append(f"p {b[0]:.1f}-{min(b[1], 1.0):.1f}: none")
        out(f"  {q:15s} " + " | ".join(line))
    out(f"  the player was woken by the hands' needs_player {len(wake_frames)} times of {len(m.turns)} turns")


def sec_packets(m, out, top):
    packets = [(c["f"], c["instructions"]) for c in m.calls if "instructions" in c]
    out(f"  {len(packets)} packets; per packet the group choices until the next")
    for i, (f, text) in enumerate(packets):
        end = packets[i + 1][0] if i + 1 < len(packets) else 10 ** 9
        hist = collections.Counter()
        for c in m.calls:
            if not (f <= c["f"] < end):
                continue
            for p in c["played"]:
                if p["kind"] == "group":
                    hist[p["played"]] += 1
        words = len(text.split())
        out(f"    {clock(f)} ({words} words): " + ", ".join(f"{k} {n}" for k, n in hist.most_common(6)))


def sec_view(m, out):
    """What the player's prompt said the enemy had against the truth, turn by turn."""
    out("  turn    known soldiers (metal)   true soldiers (metal)   commander stuck   prompt")
    fails = collections.defaultdict(list)
    for e in m.events:
        if e["k"] == "move_failed":
            fails[e["u"]].append(e["f"])
    commander_ids = {u for u, fs in fails.items()} & {e["u"] for e in m.events if e["k"] == "created" and m.def_name(e["d"]).endswith("com")}
    prev = 0
    for t in m.turns:
        f = t["frame"]
        mk = re.search(r"seen in the last 3 min and not seen to die: (\d+) worth (\d+) metal", t["prompt"])
        known = f"{mk.group(1)} ({mk.group(2)})" if mk else "?"
        tf = max((k for k in m.truth if k <= f), default=None)
        if tf is None:
            true = "?"
        else:
            sol = [u for u in m.truth[tf] if m.is_soldier_name(u[1]) and not u[5]]
            true = f"{len(sol)} ({sum(m.metal_of(u[1]) for u in sol):.0f})"
        stuck = sum(1 for u in commander_ids for x in fails[u] if prev <= x < f)
        told = "yes" if re.search(r"stuck|move fail|boxed|cannot move|blocked", t["prompt"], re.I) else "no"
        out(f"  {clock(f):>6}  {known:24s} {true:24s} {stuck:3d} fails, told {told:3s}  {t['why'][:60]}")
        prev = f


def sec_turns(m, out, top):
    why = collections.Counter(re.sub(r"\d+", "N", t["why"].split(";")[0])[:70] for t in m.turns)
    tools = collections.Counter(name for t in m.turns for name, _ in t["calls"])
    walls = sorted(t["wall"] or 0 for t in m.turns)
    idle = sum(1 for t in m.turns if all(name in ("wait", "situation", "overview", "map", "note") for name, _ in t["calls"]))
    out(f"  {len(m.turns)} turns, wall median {walls[len(walls) // 2] if walls else 0:.0f} s, max {walls[-1] if walls else 0:.0f}; "
        f"{idle} turns changed nothing (only wait/look/note)")
    out("  woken by: " + "; ".join(f"{w} x{n}" for w, n in why.most_common(top)))
    out("  tools: " + ", ".join(f"{k} {n}" for k, n in tools.most_common()))


def sec_movefail(m, out, top):
    by_unit = collections.defaultdict(list)
    for e in m.events:
        if e["k"] == "move_failed":
            by_unit[(e["u"], m.def_name(e["d"]))].append(e["f"])
    total = sum(len(v) for v in by_unit.values())
    out(f"  {total} move failures over {len(by_unit)} units")
    for (u, name), fs in sorted(by_unit.items(), key=lambda kv: -len(kv[1]))[:top]:
        out(f"    {name} {u}: {len(fs)} from {clock(fs[0])} to {clock(fs[-1])}")


def main():
    args = sys.argv[1:]
    top = 8
    sections = None
    if "--top" in args:
        i = args.index("--top")
        top = int(args[i + 1])
        del args[i:i + 2]
    if "--section" in args:
        i = args.index("--section")
        sections = set(args[i + 1].split(","))
        del args[i:i + 2]
    dirs = [a for a in args if not a.startswith("--")]
    if not dirs:
        sys.exit(__doc__)
    matches = [Match(d) for d in dirs]
    want = lambda s: sections is None or s in sections
    out = print
    for m in matches:
        out(f"\n===== {m.label}")
        if want("summary"):
            sec_summary(m, out)
        if want("kinds"):
            out("-- kinds"); sec_kinds(m, out)
        if want("noop"):
            out("-- no-op asks"); sec_noop(m, out)
        if want("flips"):
            out("-- flip-flops"); sec_flips(m, out, top)
        if want("detach"):
            out("-- detachments"); sec_detach(m, out, top)
        if want("didnot"):
            out("-- choices not carried out"); sec_didnot(m, out, top)
        if want("unsure"):
            out("-- unsure"); sec_unsure(m, out, top)
        if want("globals"):
            out("-- global nouls against the next minute"); sec_globals(m, out)
        if want("packets"):
            out("-- packets"); sec_packets(m, out, top)
        if want("view"):
            out("-- the player's view against the truth"); sec_view(m, out)
        if want("turns"):
            out("-- player turns"); sec_turns(m, out, top)
        if want("movefail"):
            out("-- move failures"); sec_movefail(m, out, top)
        if want("options") and len(matches) == 1:
            out("-- options"); sec_options(matches, out, top)
    if len(matches) > 1 and want("options"):
        out("\n===== pooled options over all matches")
        sec_options(matches, out, top)



if __name__ == "__main__":
    main()
