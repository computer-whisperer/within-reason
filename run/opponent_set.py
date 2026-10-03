#!/usr/bin/env python3
"""What an AI opponent's games look like on a map, from the truth files of our recorded games against it: its curve
by the minute, when its soldiers come, what it builds and when it goes to tier 2
(docs/design/2026-10-03-brief-rewrite-and-commander-pack.md, §3 part 3). A description of the opponent, never our
target.

    run/opponent_set.py --map "Comet Catcher Remake 1.8" --profile hard_aggressive [--min-minutes 6]

Every match under run/matches with a record and a truth file on the map whose one opponent is BARb at `--profile`.
His half is the ground nearer his start than ours. Markdown on stdout.
"""
import argparse, collections, glob, json, math, os, statistics as st, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "run"))
from pro_baseline import quartiles, clock  # noqa: E402

MINUTES = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 16, 18, 20, 25, 30]
TIER2 = ("armalab", "armavp", "armaap", "armasy", "coralab", "coravp", "coraap", "corasy")


def med(xs):
    xs = [x for x in xs if x is not None]
    return st.median(xs) if xs else None


def num(v):
    return "-" if v is None else f"{v:.0f}"


def q3(xs, kind="n"):
    q = quartiles(xs)
    if not q:
        return "- / - / -"
    f = clock if kind == "clock" else num
    return f"{f(q['p25'])} / {f(q['median'])} / {f(q['p75'])}"


def read(directory, profile, map_name):
    lines = open(os.path.join(directory, "record-0.jsonl"))
    try:
        header = json.loads(lines.readline())
    except ValueError:
        return None
    if header.get("map", {}).get("name") != map_name:
        return None
    ais = [s["controller"]["Ai"] for s in header.get("seats", []) if isinstance(s.get("controller"), dict) and "Ai" in s["controller"] and s["controller"]["Ai"].get("short_name") == "BARb"]
    if len(ais) != 1 or ais[0].get("profile") != profile or any(isinstance(s.get("controller"), dict) and "Person" in s["controller"] for s in header.get("seats", [])):
        return None
    defs = header["unit_defs"]
    by_name = {d["name"]: d for d in defs}
    ours, lost, last = None, None, 0
    for line in lines:
        if line.startswith('{"t":"s"') or '"t":"s"' in line[:60]:
            r = json.loads(line)
            last = r["f"]
            if ours is None and r.get("own"):
                ours = (r["own"][0][2], r["own"][0][3])
        elif lost is None and '"destroyed"' in line[:80]:
            r = json.loads(line)
            if r.get("k") == "destroyed" and defs[r["d"]]["class"] == "extractor":
                lost = r["f"] / 30
    if ours is None or last < 1:
        return None
    his, minute, seen, first = None, {}, {}, {}
    for line in open(os.path.join(directory, "truth-0.jsonl")):
        line = line.strip()
        if not line:
            continue
        t = json.loads(line)
        units = t.get("enemy") or []
        if his is None and units:
            his = (units[0][2], units[0][3])
        if his is None:
            continue
        now = t["f"] / 30
        in_ours = lambda u: math.hypot(u[2] - ours[0], u[3] - ours[1]) < math.hypot(u[2] - his[0], u[3] - his[1])
        for u in units:
            d = by_name.get(u[1])
            if d is None:
                continue
            if d["class"] == "factory" and u[0] not in seen:
                seen[u[0]] = (now, u[1])
            if d["class"] == "army" and "soldier_in_our_half" not in first and in_ours(u):
                first["soldier_in_our_half"] = (now, u[1])
        m = t["f"] // 1800
        if t["f"] % 1800 < 60 and m not in minute and m > 0:
            built = [u for u in units if not u[5] and u[1] in by_name]
            army = [u for u in built if by_name[u[1]]["class"] == "army"]
            types = collections.Counter(u[1] for u in army)
            minute[m] = {
                "extractors": sum(1 for u in built if by_name[u[1]]["class"] == "extractor"),
                "constructors": sum(1 for u in built if by_name[u[1]]["class"] == "builder"),
                "turrets": sum(1 for u in built if by_name[u[1]]["class"] == "turret"),
                "army_metal": sum(by_name[u[1]]["metal"] for u in army), "soldiers": len(army),
                "soldiers_in_our_half": sum(1 for u in army if in_ours(u)),
                "types": {k: n * by_name[k]["metal"] for k, n in types.items()},
            }
    if his is None:
        return None
    factories = sorted(seen.values())
    result = None
    path = os.path.join(os.path.dirname(directory), "results.jsonl")
    if os.path.exists(path):
        index = int(os.path.basename(directory))
        for line in open(path):
            r = json.loads(line)
            if r.get("index") == index:
                result = r.get("outcome")
    return {"dir": directory, "minutes": last / 1800, "minute": minute, "factories": factories, "first": first, "our_first_extractor_lost": lost,
            "tier2": next(((t, n) for t, n in factories if n in TIER2), None), "side": factories[0][1][:3] if factories else "?", "result": result,
            "apart": math.hypot(ours[0] - his[0], ours[1] - his[1])}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--map", required=True)
    ap.add_argument("--profile", default="hard_aggressive")
    ap.add_argument("--min-minutes", type=float, default=6)
    a = ap.parse_args()
    games = []
    for d in sorted(glob.glob(os.path.join(REPO, "run/matches/*/[0-9][0-9]"))):
        if os.path.exists(os.path.join(d, "record-0.jsonl")) and os.path.exists(os.path.join(d, "truth-0.jsonl")):
            g = read(d, a.profile, a.map)
            if g and g["minutes"] >= a.min_minutes and g["factories"]:
                games.append(g)
    if not games:
        sys.exit("no game matches")
    P = print
    P(f"## What this opponent's games look like: BARb {a.profile} on {a.map}\n")
    # Our results are left out: the games span every version of our side, and a tally over them says nothing of now.
    P(f"- **The games:** {len(games)} of ours against it that lasted {a.min_minutes:.0f} minutes or more, across many versions of our side, {q3([g['minutes'] * 60 for g in games], 'clock')} long; "
      f"the starts {q3([g['apart'] for g in games])} apart. Read from the truth file: everything he had, seen by us or not.")
    P("- This is what he does, against the play we gave him. It is the opponent to expect, not a pace to match: the reference game is the people's.")
    first = collections.Counter(g["factories"][0][1] for g in games)
    P(f"- **His first factory:** {', '.join(f'{k} ({n})' for k, n in first.most_common())}, started {q3([g['factories'][0][0] for g in games], 'clock')}. "
      f"His second: {q3([g['factories'][1][0] for g in games if len(g['factories']) > 1], 'clock')} ({sum(1 for g in games if len(g['factories']) > 1)} of {len(games)} games; "
      f"{', '.join(f'{k} ({n})' for k, n in collections.Counter(g['factories'][1][1] for g in games if len(g['factories']) > 1).most_common(4))}).")
    s = [g["first"].get("soldier_in_our_half") for g in games]
    P(f"- **His first soldier in our half:** {q3([x[0] for x in s if x], 'clock')} ({', '.join(f'{k} ({n})' for k, n in collections.Counter(x[1] for x in s if x).most_common(4))}). "
      f"Our first extractor lost: {q3([g['our_first_extractor_lost'] for g in games], 'clock')}.")
    t2 = [g for g in games if g["tier2"]]
    long = [g for g in games if g["minutes"] >= 15]
    P(f"- **His tier 2:** an advanced factory in {len(t2)} of {len(games)} games ({sum(1 for g in long if g['tier2'])} of the {len(long)} that lasted 15 minutes), started {q3([g['tier2'][0] for g in t2], 'clock')}: "
      f"{', '.join(f'{k} ({n})' for k, n in collections.Counter(g['tier2'][1] for g in t2).most_common())}.")
    if t2:
        at = [(g, g["minute"].get(int(g["tier2"][0] // 60))) for g in t2]
        at = [c for _, c in at if c]
        P(f"  When he started it he had {q3([c['extractors'] for c in at])} extractors and {q3([c['army_metal'] for c in at])} army metal.")
    P("")
    P("| Minute | Games | Extractors | Constructors | Turrets | Army metal | Soldiers | Soldiers in our half | Army made of (share of metal, 5% and over) |\n|---|---|---|---|---|---|---|---|---|")
    for m in MINUTES:
        have = [g["minute"][m] for g in games if m in g["minute"]]
        if len(have) < 6:
            break
        total = collections.Counter()
        for c in have:
            total.update(c["types"])
        metal = sum(total.values()) or 1
        mix = ", ".join(f"{k} {100 * v / metal:.0f}%" for k, v in total.most_common() if v / metal >= 0.05)
        P(f"| {m} | {len(have)} | " + " | ".join(q3([c[k] for c in have]) for k in ("extractors", "constructors", "turrets", "army_metal", "soldiers", "soldiers_in_our_half")) + f" | {mix} |")


if __name__ == "__main__":
    main()
