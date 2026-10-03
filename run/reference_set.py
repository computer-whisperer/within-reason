#!/usr/bin/env python3
"""The reference set of a map: what the experienced players' games look like there, as tables with the derivation
of each number against the map sheet (docs/design/2026-10-03-brief-rewrite-and-commander-pack.md, §3 part 1).

    run/reference_set.py --map "Comet Catcher Remake 1.8" --faction arm --sheet-from run/matches/<batch>/<NN> [--floor 40]

The pool is every carded side of the manifest on the map with every player at `--floor` OS and above, playing
`--faction` (arm, cor, any). `--sheet-from` is a recorded game of ours on the map: its header gives the unit
definitions and the spots, and `target/release/mapsheet` its sheet. Markdown on stdout.
"""
import argparse, collections, json, math, os, statistics as st, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "run"))
from pro_baseline import milestones, MILESTONE_ROWS, quartiles, secs, clock  # noqa: E402

MINUTES = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 16, 18, 20]
CURVES = [("extractors", "extractors"), ("constructors", "constructors"), ("metal_income", "metal income"), ("energy_income", "energy income"),
          ("army_metal", "army metal"), ("soldiers", "soldiers"), ("army_metal_in_his_half", "army metal in his half"), ("metal_stored", "metal stored")]


def med(xs):
    xs = [x for x in xs if x is not None]
    return st.median(xs) if xs else None


def num(v, digits=0):
    if v is None:
        return "-"
    return f"{v:.{digits}f}" if digits else f"{v:.0f}"


def q3(xs, kind="n", digits=0):
    q = quartiles(xs)
    if not q:
        return "- / - / -"
    f = clock if kind == "clock" else (lambda v: num(v, digits))
    return f"{f(q['p25'])} / {f(q['median'])} / {f(q['p75'])}"


def load(manifest, map_name, floor, faction):
    sides = []
    for line in open(manifest):
        r = json.loads(line)
        if not (r.get("keep") and r.get("card")) or r.get("map_script") != map_name:
            continue
        if any(p.get("os") is None or p["os"] < floor for p in r["players"]):
            continue
        path = os.path.join(REPO, r["match"], "card.json")
        if not os.path.exists(path):
            continue
        card = json.load(open(path))
        for i, t in enumerate(card["teams"]):
            if faction != "any" and t.get("side") != faction:
                continue
            other = card["teams"][1 - i] if len(card["teams"]) == 2 else None
            sides.append({**t, "id": r["id"], "duration": secs(card["duration"]), "his_start": other and other.get("start"), "his_side": other and other.get("side"),
                          "os": next((p["os"] for p in r["players"] if p["team"] == t["team"]), None), "curve": {c["minute"]: c for c in t.get("curves") or []}})
    return sides


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--map", required=True)
    ap.add_argument("--faction", default="arm")
    ap.add_argument("--floor", type=float, default=40)
    ap.add_argument("--sheet-from", required=True)
    ap.add_argument("--manifest", default=os.path.join(REPO, "run/data/replays/manifest.jsonl"))
    ap.add_argument("--json-out", help="also write the by-minute table as JSON here: the bot's report reads it (`brain/reference.rs`)")
    a = ap.parse_args()
    sides = load(a.manifest, a.map, a.floor, a.faction)
    if not sides:
        sys.exit("no carded side matches")
    if a.json_out:
        table = {}
        for minute in MINUTES:
            have = [s for s in sides if minute in s["curve"]]
            if len(have) < 6:
                break
            table[str(minute)] = {key: {**{k: round(v, 1) for k, v in (quartiles([s["curve"][minute].get(key) for s in have]) or {}).items()},
                                        "won": med([s["curve"][minute].get(key) for s in have if s.get("won")])} for key, _ in CURVES}
        apart = med([math.hypot(s["start"]["x"] - s["his_start"]["x"], s["start"]["z"] - s["his_start"]["z"]) for s in sides if s["his_start"]])
        json.dump({"map": a.map, "faction": a.faction, "floor": a.floor, "sides": len(sides), "games": len({s["id"] for s in sides}), "starts_apart": round(apart), "minutes": table}, open(a.json_out, "w"), indent=1)
    record = sorted(f for f in os.listdir(a.sheet_from) if f.startswith("record-"))[0]
    header = json.loads(open(os.path.join(a.sheet_from, record)).readline())
    defs = {d["name"]: d for d in header["unit_defs"]}
    spots = header["metal_spots"]
    classify = lambda name: defs.get(name, {}).get("class", "?")
    cost = lambda name: defs.get(name, {}).get("metal", 0)
    sheet = json.loads(subprocess.run([os.path.join(REPO, "target/release/mapsheet"), a.sheet_from, "--json"], capture_output=True, text=True, check=True).stdout)
    won, lost = [s for s in sides if s.get("won")], [s for s in sides if not s.get("won")]
    out = []
    P = out.append

    P(f"## The reference game: {a.map}, {'both factions' if a.faction == 'any' else {'arm': 'Armada', 'cor': 'Cortex'}[a.faction]}\n")
    oss = sorted(s["os"] for s in sides if s["os"] is not None)
    durs = [s["duration"] for s in sides]
    starts = collections.Counter(s["start"]["grid"] for s in sides)
    P(f"- **The pool:** {len(sides)} sides of {len({s['id'] for s in sides})} duels between people, every player at OS {a.floor:.0f} or above (OS {oss[0]:.0f} to {oss[-1]:.0f}, median {st.median(oss):.0f}); "
      f"{len(won)} of the sides won. Games lasted {q3(durs, 'clock')} (lower quartile / median / upper quartile, as every triple below).")
    P(f"- **Starts:** {', '.join(f'{c} ({n})' for c, n in starts.most_common())}; the starts stood {q3([math.hypot(s['start']['x'] - s['his_start']['x'], s['start']['z'] - s['his_start']['z']) for s in sides if s['his_start']])} apart in a straight line.")
    P(f"- **Against:** {', '.join(f'{k} ({n})' for k, n in collections.Counter(s['his_side'] for s in sides).most_common())}.")
    P("- A number here is what these players did, on this map. The derivation under each table says what on the map it came from; the map sheet gives the same quantities for another map.\n")

    # 1. the opening
    P("### The opening\n")
    seqs = collections.Counter()
    before = collections.defaultdict(list)
    for s in sides:
        seq = []
        for step in s["build_order"]:
            if step.get("by") != "commander":
                continue
            seq.append(step["unit"])
            if classify(step["unit"]) == "factory" or step["unit"] in ("armvp", "armlab", "armap", "corvp", "corlab", "corap"):
                break
        seqs[" → ".join(seq)] += 1
        for kind in ("extractor", "energy"):
            names = {"extractor": ("armmex", "cormex"), "energy": ("armsolar", "corsolar", "armwin", "corwin")}[kind]
            before[kind].append(sum(1 for u in seq[:-1] if u in names))
    P("The commander's buildings up to the first factory, the orders most played:\n")
    P("| Sides | Order |\n|---|---|")
    for seq, n in seqs.most_common(5):
        P(f"| {n} | {seq} |")
    P(f"\nBefore the factory: {q3(before['extractor'])} extractors and {q3(before['energy'])} generators.\n")
    # The most played order is not the best one: the same orders by who won, and what the sides that expanded
    # fastest did (the user, 2026-10-03, after player-55 copied the commonest order and stood behind our own).
    def opening(s):
        seq = []
        for step in s["build_order"]:
            if step.get("by") != "commander":
                continue
            seq.append(step["unit"])
            if step["unit"] in ("armvp", "armlab", "armap", "corvp", "corlab", "corap"):
                break
        return " → ".join(seq)

    def first_units(s, n=7):
        made = [b["unit"] for b in s["build_order"] if b.get("by", "").startswith("factory")][:n]
        out, last, count = [], None, 0
        for u in made + [None]:
            if u == last:
                count += 1
                continue
            if last:
                out.append(f"{count} {last}" if count > 1 else last)
            last, count = u, 1
        return ", ".join(out)

    at4 = lambda s: (s["curve"].get(4) or {}).get("extractors", 0)
    P("**The same orders by result**, with what each side's extractors and army were at 4:00 (medians):\n")
    P("| Order | Sides | Won | Extractors at 4:00 | Constructors at 4:00 | Army metal at 4:00 |\n|---|---|---|---|---|---|")
    groups = collections.defaultdict(list)
    for s in sides:
        groups[opening(s)].append(s)
    for seq, group in sorted(groups.items(), key=lambda g: -len(g[1]))[:5]:
        c4 = lambda key: num(med([(s["curve"].get(4) or {}).get(key) for s in group]))
        P(f"| {seq} | {len(group)} | {sum(1 for s in group if s.get('won'))} | {c4('extractors')} | {c4('constructors')} | {c4('army_metal')} |")
    P("\n**What the factory made first** (its first seven units), most played and by result:\n")
    P("| The factory's first units | Sides | Won | Extractors at 4:00 | Army metal in his half at 4:00 |\n|---|---|---|---|---|")
    units = collections.defaultdict(list)
    for s in sides:
        units[first_units(s)].append(s)
    for seq, group in sorted(units.items(), key=lambda g: -len(g[1]))[:6]:
        P(f"| {seq} | {len(group)} | {sum(1 for s in group if s.get('won'))} | {num(med([at4(s) for s in group]))} | {num(med([(s['curve'].get(4) or {}).get('army_metal_in_his_half') for s in group]))} |")
    fast = sorted(sides, key=at4, reverse=True)[: max(4, len(sides) // 4)]
    P(f"\n**The quarter of sides with the most extractors at 4:00** ({len(fast)} sides, {num(med([at4(s) for s in fast]))} extractors at the median, {sum(1 for s in fast if s.get('won'))} of them won):\n")
    P("| Side | Result | Commander's order | The factory's first units | Extractors at 4:00 / 8:00 | First constructor |\n|---|---|---|---|---|---|")
    for s in fast[:8]:
        first_con = next((b["clock"] for b in s["build_order"] if b.get("by", "").startswith("factory") and classify(b["unit"]) == "builder"), "-")
        P(f"| {s['player']} | {'won' if s.get('won') else 'lost'} at {clock(s['duration'])} | {opening(s)} | {first_units(s)} | {at4(s)} / {(s['curve'].get(8) or {}).get('extractors', '-')} | {first_con} |")
    P("")
    ms = [(s, milestones(s["build_order"], s.get("factories") or [], s.get("first_extractor_lost"), classify, s.get("pressure") or {})) for s in sides]
    kinds = collections.Counter(m["first_factory_kind"] for _, m in ms)
    P(f"First factory: {', '.join(f'{k} ({n})' for k, n in kinds.most_common())}.\n")
    P("| Milestone | Lower quartile / median / upper quartile | Winners' median | Losers' median |\n|---|---|---|---|")
    for key, label, kind in MILESTONE_ROWS:
        f = clock if kind == "clock" else num
        w, l = med([m[key] for s, m in ms if s.get("won")]), med([m[key] for s, m in ms if not s.get("won")])
        P(f"| {label} | {q3([m[key] for _, m in ms], kind)} | {f(w) if w is not None else '-'} | {f(l) if l is not None else '-'} |")
    # derivations
    first_units = collections.Counter((s.get("pressure") or {}).get("first_soldier_in_his_half", {}).get("unit") for s in sides)
    first_units.pop(None, None)
    travel = [m["first_soldier_in_his_half"] - m["first_soldier"] for _, m in ms if m.get("first_soldier_in_his_half") is not None and m.get("first_soldier") is not None]
    veh = next((c for c in sheet["classes"] if c["class"] == "vehicles"), None)
    to = veh and veh.get("to_each_other_start") and veh["to_each_other_start"][0]
    P("\n**Where the numbers come from.**")
    P(f"- *The factory at {clock(med([m['first_factory_started'] for _, m in ms]))}:* it is started after {num(med(before['extractor']))} extractors and {num(med(before['energy']))} generators, every one within the commander's reach of the start (the sheet: {sheet['expansion'][0]['spots_within_seconds'][0]} spots within 30 s of the commander).")
    if to and travel:
        apart = med([math.hypot(s["start"]["x"] - s["his_start"]["x"], s["start"]["z"] - s["his_start"]["z"]) for s in sides if s["his_start"]])
        pre = "arm" if a.faction == "any" else a.faction
        half = ", ".join(f"{u} {apart / 2 / defs[u]['speed']:.0f} s" for u in to["seconds"] if u.startswith(pre) and defs.get(u, {}).get("speed"))
        ours = sheet["starts"][1]["straight_from_ours"] if len(sheet["starts"]) > 1 else None
        P(f"- *The first soldier in his half:* the unit was {', '.join(f'{u} ({n})' for u, n in first_units.most_common(4))}; it crossed into his half {q3(travel)} s after it was made. "
          f"The pool's starts are {apart:.0f} apart at the median, and half of that at full speed is {half}." + (f" The start the sheet was made from is {ours} from his ({100 * ours / apart - 100:+.0f}%): every crossing there is that much longer." if ours else ""))
    inc = [(c["metal_income"] - 2) / c["extractors"] for s in sides for m, c in s["curve"].items() if 3 <= m <= 8 and c["extractors"] >= 3]
    P(f"- *Income per extractor:* {q3(inc, digits=2)} metal a second over minutes 3 to 8 (income less the commander's 2, over the extractors standing); the sheet's figure for a spot here is {num(sheet['spots']['metal_a_second_each_low_median_high'][1], 2)}. Below it means energy short or extractors still going up.")
    P("")

    # 2. curves
    P("### The game by the minute\n")
    P("Lower quartile / median / upper quartile of the pool; then the winners' and the losers' medians. A side counts at a minute only if its game reached it, so the late minutes are the longer games.\n")
    for keys in (CURVES[:4], CURVES[4:]):
        P("| Minute | Sides | " + " | ".join(f"{label} | won / lost" for _, label in keys) + " |\n|---|---|" + "---|---|" * len(keys))
        for minute in MINUTES:
            have = [s for s in sides if minute in s["curve"]]
            if len(have) < 6:
                break
            row = [str(minute), str(len(have))]
            for key, _ in keys:
                row.append(q3([s["curve"][minute].get(key) for s in have]))
                row.append(f"{num(med([s['curve'][minute].get(key) for s in have if s.get('won')]))} / {num(med([s['curve'][minute].get(key) for s in have if not s.get('won')]))}")
            P("| " + " | ".join(row) + " |")
        P("")
    # where the extractors are
    def share_of_way(s, spot):
        if spot is None or spot >= len(spots) or not s["his_start"]:
            return None
        x, z = spots[spot][0], spots[spot][1]
        d0 = math.hypot(x - s["start"]["x"], z - s["start"]["z"])
        d1 = math.hypot(x - s["his_start"]["x"], z - s["his_start"]["z"])
        return d0 / (d0 + d1)
    P("**Where the extractors are.** Each spot taken is placed by how far it is from the side's start against his (0 at ours, 50% equally far, 100% at his):\n")
    P("| Taken by | Spots taken | Under 25% | 25-40% | 40-50% | Past 50% (nearer him) |\n|---|---|---|---|---|---|")
    for limit in (240, 480, 720):
        taken, buckets = [], collections.Counter()
        for s in sides:
            if s["duration"] < limit:
                continue
            mine = [share_of_way(s, t.get("spot")) for t in s.get("spots_taken") or [] if secs(t["clock"]) <= limit]
            mine = [m for m in mine if m is not None]
            taken.append(len(mine))
            for m in mine:
                buckets["a" if m < 0.25 else "b" if m < 0.4 else "c" if m < 0.5 else "d"] += 1
        total = sum(buckets.values()) or 1
        P(f"| {clock(limit)} | {q3(taken)} | {100 * buckets['a'] / total:.0f}% | {100 * buckets['b'] / total:.0f}% | {100 * buckets['c'] / total:.0f}% | {100 * buckets['d'] / total:.0f}% |")
    lost_rate = [((s.get("losses") or {}).get("armmex", 0) + (s.get("losses") or {}).get("cormex", 0)) / (s["duration"] / 60) for s in sides]
    P(f"\nExtractors lost: {q3(lost_rate, digits=2)} a game-minute (winners {num(med([r for r, s in zip(lost_rate, sides) if s.get('won')]), 2)}, losers {num(med([r for r, s in zip(lost_rate, sides) if not s.get('won')]), 2)}).\n")

    # 3. factories and tier 2
    P("### More factories and tier 2\n")
    second = collections.Counter()
    for s in sides:
        f = s.get("factories") or []
        if len(f) > 1:
            second[f[1]["unit"]] += 1
    P(f"- **The second factory:** {sum(second.values())} of {len(sides)} sides built one: {', '.join(f'{k} ({n})' for k, n in second.most_common())}.")
    t2 = [(s, secs(s["first_tier2"]["clock"])) for s in sides if s.get("first_tier2") and s["first_tier2"].get("clock")]
    P(f"- **Tier 2:** {len(t2)} of {len(sides)} sides started an advanced factory ({', '.join(f'{k} ({n})' for k, n in collections.Counter(s['first_tier2']['unit'] for s, _ in t2).most_common())}), at {q3([t for _, t in t2], 'clock')}. "
      f"Of the sides whose game lasted past 15:00, {sum(1 for s, _ in t2 if s['duration'] > 900)} of {sum(1 for s in sides if s['duration'] > 900)} did.")
    if t2:
        at = [(s, s["curve"].get(t // 60)) for s, t in t2]
        at = [(s, c) for s, c in at if c]
        P(f"- **What they had when they started it:** {q3([c['extractors'] for _, c in at])} extractors, {q3([c['metal_income'] for _, c in at])} metal income, {q3([c['energy_income'] for _, c in at])} energy income, {q3([c['army_metal'] for _, c in at])} army metal.")
        first = [s for s, t in t2 if s["his_start"] and not any(o["id"] == s["id"] and o is not s and o.get("first_tier2") and secs(o["first_tier2"]["clock"]) < t for o in sides)]
        P(f"- **Who won:** the sides that started tier 2 won {sum(1 for s, _ in t2 if s.get('won'))} of {len(t2)}; the sides that never did won {sum(1 for s in sides if not s.get('first_tier2') and s.get('won'))} of {sum(1 for s in sides if not s.get('first_tier2'))} (most of those games ended before anyone would have).")
    P("")

    # 4. composition
    P("### What the armies were made of\n")
    P("The soldiers standing at the minute, summed over the pool's sides, as shares of the army's metal (types under 3% left out):\n")
    P("| Minute | Sides | Median army metal | Made of |\n|---|---|---|---|")
    for minute in (2, 4, 6, 8, 10, 14, 18):
        have = [(s, next((x["army"] for x in s.get("army_by_type") or [] if x["minute"] == minute), None)) for s in sides if minute in s["curve"]]
        have = [(s, army) for s, army in have if army is not None]
        if len(have) < 6:
            continue
        total = collections.Counter()
        for _, army in have:
            for unit, n in army.items():
                total[unit] += n * cost(unit)
        metal = sum(total.values()) or 1
        mix = ", ".join(f"{u} {100 * m / metal:.0f}%" for u, m in total.most_common() if m / metal >= 0.03)
        P(f"| {minute} | {len(have)} | {num(med([s['curve'][minute]['army_metal'] for s, _ in have]))} | {mix} |")
    P("")
    print("\n".join(out))


if __name__ == "__main__":
    main()
