#!/usr/bin/env python3
"""Whole games of the pool as timelines, minute by minute: what a side built, what it had, where its army stood and
what it fought (docs/design/2026-10-03-brief-rewrite-and-commander-pack.md, §3 part 2).

    run/timelines.py --map "Comet Catcher Remake 1.8" --faction arm [--floor 40] [--games ID,ID,...]

Without `--games` three are chosen from the pool's sides of the faction: the win nearest the pool's median length,
the shortest win, and the longest game the side lost. Markdown on stdout.
"""
import argparse, collections, json, os, statistics as st, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "run"))
from pro_baseline import secs  # noqa: E402


def counted(names):
    c = collections.Counter(names)
    return ", ".join(f"{n} {u}" if n > 1 else u for u, n in c.items())


def timeline(card, i, why):
    t, o = card["teams"][i], card["teams"][1 - i]
    out = [f"### {t['player']} ({t['side']}, start {t['start']['grid']}) against {o['player']} ({o['side']}, {o['start']['grid']}): {'won' if t.get('won') else 'lost'} at {card['duration']}\n", f"*{why}*\n",
           "| Minute | The commander and constructors built | The factories made | Extractors (his) | Constructors | Metal income (his) | Army metal (his) | Of it in his half | Army | Fights (cell: lost / killed) |", "|---|---|---|---|---|---|---|---|---|---|"]
    curve, his = {c["minute"]: c for c in t["curves"]}, {c["minute"]: c for c in o["curves"]}
    army = {a["minute"]: a["army"] for a in t.get("army_by_type") or []}
    built, made = collections.defaultdict(list), collections.defaultdict(list)
    for b in t["build_order"]:
        m = secs(b["clock"]) // 60 + 1
        (made if b.get("by", "").startswith("factory") else built)[m].append(b["unit"])
    fights = collections.defaultdict(list)
    for f in t.get("fights") or []:
        fights[f["minute"]].append(f"{f['grid']}: {f['ours_lost']} / {f['theirs_lost']}")
    tier2 = t.get("first_tier2") or {}
    for m in sorted(curve):
        if m == 0:
            continue
        c, h = curve[m], his.get(m, {})
        mix = ", ".join(f"{n} {u}" for u, n in sorted((army.get(m) or {}).items(), key=lambda x: -x[1])[:4])
        out.append(f"| {m} | {counted(built[m])} | {counted(made[m])} | {c['extractors']} ({h.get('extractors', '-')}) | {c['constructors']} | {c['metal_income']:.0f} ({h.get('metal_income', 0):.0f}) | "
                   f"{c['army_metal']:.0f} ({h.get('army_metal', 0):.0f}) | {c.get('army_metal_in_his_half', 0):.0f} | {mix} | {'; '.join(fights[m])} |")
    notes = []
    if tier2.get("clock"):
        notes.append(f"tier 2 ({tier2['unit']}) started {tier2['clock']}")
    lost = t.get("first_extractor_lost") or {}
    if lost.get("clock"):
        notes.append(f"first extractor lost {lost['clock']} at {lost.get('grid')} to {lost.get('by')}")
    p = (t.get("pressure") or {}).get("first_his_building_killed") or {}
    if p.get("clock"):
        notes.append(f"first building of his killed {p['clock']} ({p.get('unit')} at {p.get('grid')})")
    top = lambda d: ", ".join(f"{n} {u}" for u, n in sorted((d or {}).items(), key=lambda x: -x[1])[:6])
    notes.append(f"lost in all: {top(t.get('losses'))}")
    notes.append(f"killed in all: {top(t.get('kills'))}")
    out.append("\n" + "; ".join(notes) + ".\n")
    return "\n".join(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--map", required=True)
    ap.add_argument("--faction", default="arm")
    ap.add_argument("--floor", type=float, default=40)
    ap.add_argument("--games", default="")
    ap.add_argument("--manifest", default=os.path.join(REPO, "run/data/replays/manifest.jsonl"))
    a = ap.parse_args()
    sides = []
    for line in open(a.manifest):
        r = json.loads(line)
        if not (r.get("keep") and r.get("card")) or r.get("map_script") != a.map or any(p.get("os") is None or p["os"] < a.floor for p in r["players"]):
            continue
        path = os.path.join(REPO, r["match"], "card.json")
        if not os.path.exists(path):
            continue
        card = json.load(open(path))
        if len(card["teams"]) != 2:
            continue
        for i, t in enumerate(card["teams"]):
            if t.get("side") == a.faction:
                sides.append((r["id"], card, i, secs(card["duration"]), bool(t.get("won"))))
    if not sides:
        sys.exit("no carded side matches")
    if a.games:
        wanted = a.games.split(",")
        chosen = [(s, "chosen by hand") for s in sides if any(s[0].startswith(w) for w in wanted)]
    else:
        median = st.median(s[3] for s in sides)
        wins, losses = [s for s in sides if s[4]], [s for s in sides if not s[4]]
        chosen = [(min(wins, key=lambda s: abs(s[3] - median)), "A win of the usual length: the win nearest the pool's median game."),
                  (min(wins, key=lambda s: s[3]), "The shortest win of the pool: a game decided by the first minutes."),
                  (max(losses, key=lambda s: s[3]), "The longest game the side lost: a lead or a level game that turned late.")]
    print(f"## Whole games: {a.map}, {a.faction}\n\nNumbers in brackets are the opponent's at the same minute. A row is the minute ending at that clock.\n")
    for (gid, card, i, _, _), why in chosen:
        print(timeline(card, i, why))


if __name__ == "__main__":
    main()
