#!/usr/bin/env python3
"""The late-game frontier of a map: the fastest orders the build-order simulator finds to an income of 50, 100 and
200 a second, to the first tier-2 extractor, fusion, advanced fusion, anti-nuke and nuclear missile, each under three
taxes (metal a second that must go to soldiers before any build: none, the standard one, a heavy one), as tables for the
brief (docs/design/2026-10-09-late-economy.md §4.1 and §9). Read every number as a ceiling: the simulator has no enemy
(nothing is raided, nothing dies, every spot on our half is free) and runs 15 to 25 % high on income by minute 8
against quiet recorded openings (docs/studies/build-order.md); the tax is the only cost of the enemy it knows.

    run/frontier_set.py [--maps "Great Divide V1,Comet Catcher Remake 1.8"] [--quick] [--workers 4] [--iterations 20000] [--restarts 4]
    run/frontier_set.py --brief        the brief's section from the frontier files already drawn, no run

Writes docs/briefs/reference/frontier-<map slug>.md (the tables) and .json (every run: plan, firsts, state by minute)
for each map, from the latest recorded game on it (run/map_sheets.py's roots), then docs/briefs/reference/frontier.md,
the one section across the maps that the experience brief's manifest names (the per-map files stay beside it for a
reader who wants every order).
"""
import argparse, concurrent.futures, json, os, re, subprocess, sys, tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "run"))
from map_sheets import latest_record  # noqa: E402  (chdirs to the repo)

REF = "docs/briefs/reference"
# The maps the frontier is drawn for: 11, 22 and 40 spots a side (the user's ruling: scaling off a few spots is the key skill).
MAPS = ["Great Divide V1", "Quicksilver Remake 1.24", "Comet Catcher Remake 1.8"]
# The tax: metal a second to soldiers from each time. Standard follows the growth of the experienced players' standing
# army on Comet (docs/briefs/reference/comet-catcher-remake-1.8-arm.md, the median side: 578 metal at 4:00, 1,080 at
# 6:00, 1,496 at 8:00, 2,530 at 10:00, 3,570 at 12:00, 4,718 at 14:00, 5,933 at 16:00: 3 to 10 metal a second, before
# what they lost), rounded up for the losses; heavy is one and a half times that.
TAXES = [("none", ""), ("standard", "120:3,240:5,480:9,720:12,960:15"), ("heavy", "120:5,240:8,480:14,720:18,960:22")]
MINUTES = 30


def slug(name):
    return re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")


def clock(seconds):
    if seconds is None:
        return f"not by {MINUTES}:00"
    return f"{int(seconds) // 60}:{int(seconds) % 60:02d}"


def objectives(side):
    antinuke = "armamd" if side == "arm" else "corfmd"
    return [
        ("income 50 a second", "reach:50", ("reach", "50")),
        ("income 100 a second", "reach:100", ("reach", "100")),
        ("income 200 a second", "reach:200", ("reach", "200")),
        # `:1`: the seed lays one of the goal, not the produce tool's six (six advanced fusions in a seed is 58,000 metal the
        # search must first delete).
        ("the first tier-2 extractor", f"target:{side}moho:1@{MINUTES * 60}", ("first_finished", f"{side}moho")),
        ("the first fusion", f"target:{side}fus:1@{MINUTES * 60}", ("first_finished", f"{side}fus")),
        ("the first advanced fusion", f"target:{side}afus:1@{MINUTES * 60}", ("first_finished", f"{side}afus")),
        ("the first anti-nuke", f"target:{antinuke}:1@{MINUTES * 60}", ("first_finished", antinuke)),
        ("the first missile", f"target:round:{side}silo:1@{MINUTES * 60}", ("first_round", f"{side}silo")),
    ], antinuke


def run_one(record, side, antinuke, objective, tax, iterations, restarts, seed_chain=None, seed=1):
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        out = f.name
    cmd = ["target/release/buildorder", "optimize", "--game", record, "--palette", "roster", "--also", f"{side}silo,{antinuke}", "--objective", objective,
           "--minutes", str(MINUTES), "--iterations", str(iterations), "--restarts", str(restarts), "--seed", str(seed), "--json-out", out]
    if tax:
        cmd += ["--tax", tax]
    if seed_chain:
        cmd += ["--seed-chain", seed_chain]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        os.unlink(out)
        raise RuntimeError(f"{' '.join(cmd)}\n{r.stderr.strip()}\n{r.stdout.strip()[:400]}")
    result = json.load(open(out))
    os.unlink(out)
    return result


def reached(result, key):
    kind, name = key
    value = result.get(kind, {}).get(name)
    return value


def state_at(result, t):
    """The state at the last whole minute at or before t."""
    rows = [m for m in result["by_minute"] if m["t"] <= (t if t is not None else MINUTES * 60)]
    return rows[-1] if rows else None


def tables(map_name, record, results, side, spots, repaired=0):
    objs, _ = objectives(side)
    out = [f"### The late-game frontier: {map_name}\n",
           f"{spots // 2} spots a side ({spots} on the map), the {side} side, from the recorded game `{record.split('/')[-3]}`; the "
           f"build-order simulator's best orders over {MINUTES} minutes (`run/frontier_set.py`). **A ceiling, not a target:** no enemy, "
           "nothing dies, every spot on our half is free, and the simulator runs 15 to 25 % high on income by minute 8 against quiet "
           "recorded openings. The tax is the enemy it knows: metal a second taken for soldiers before any build, from each time. "
           "Standard: 3 a second from 2:00, 5 from 4:00, 9 from 8:00, 12 from 12:00, 15 from 16:00 (the experienced players' standing "
           "army on Comet grows at that rate, before losses); heavy: one and a half times that. The arithmetic the frontier rests on is "
           "the engine's (a frame builds at the sum of the power on it until the store empties; a stockpile at its fixed rate, helped by "
           "nobody: K-t2-build-power-for-a-project-follows-the-spend, K-rules-a-stockpile-takes-no-build-power). The search is "
           "stochastic (simulated annealing, 20,000 steps x 8 restarts a cell, the reach rows from two seeds); a cell a heavier tax beat "
           f"was searched again from other seeds ({repaired} cells improved so). A cell still out of order is the search's noise, not the game's.\n",
           "| the fastest order to | no tax | standard tax | heavy tax |", "|---|---|---|---|"]
    for label, _, key in objs:
        cells = [clock(reached(results[(tax, label)], key)) for tax, _ in TAXES]
        out.append(f"| {label} | " + " | ".join(cells) + " |")
    missile = {tax: reached(results[(tax, "the first missile")], ("first_round", f"{side}silo")) for tax, _ in TAXES}
    verdicts = []
    for tax, _ in TAXES:
        t = missile[tax]
        if t is None:
            verdicts.append(f"{tax}: no missile by {MINUTES}:00")
        else:
            inside = "inside" if t < 21 * 60 else "level with" if t <= 25 * 60 else "after"
            verdicts.append(f"{tax}: {clock(t)}, {inside} his anti-nuke's window")
    out.append("")
    out.append("**The nuke line against BARb hard_aggressive**, whose anti-nuke stands at minute 21 to 25 in 27 of 33 long games "
               "(K-barb-anti-nuke-by-minute; medium builds one in one long game of six): " + "; ".join(verdicts) + ". A missile is energy flow, "
               "1,042 a second for its 120 s, not build power.\n")
    out.append("**What stands at each milestone under the standard tax** (the state at the last whole minute before it).\n")
    out.append("| milestone | at | extractors | constructors | turrets | build power | energy a second | income | army metal paid | metal wasted |")
    out.append("|---|---|---|---|---|---|---|---|---|---|")
    for label, _, key in objs:
        r = results[("standard", label)]
        t = reached(r, key)
        s = state_at(r, t)
        if s is None:
            out.append(f"| {label} | {clock(t)} | | | | | | | | |")
            continue
        out.append(f"| {label} | {clock(t)} | {s['extractors']} | {s['constructors']} | {s['nanos']} | {s['build_power']:.0f} | {s['energy_income']:.0f} | {s['income']:.0f} | {s['tax_paid']:.0f} | {s['metal_wasted']:.0f} |")
    out.append("")
    out.append("**The orders under the standard tax**, one queue a line (`com` the commander, `facN` the factories as they stand, `conN` the "
               "constructors as they leave; `help` puts the builder's power on the dearest frame begun; `assist` on the first factory; "
               "`amex` the tier-2 extractor on a standing one's spot).\n")
    for label, _, key in objs:
        r = results[("standard", label)]
        lines = [l for l in r["plan"].strip().splitlines() if l.strip()]
        out.append(f"- **{label}** ({clock(reached(r, key))}): " + " · ".join(f"`{l}`" for l in lines))
    out.append("")
    return "\n".join(out)


def brief(maps):
    """The brief's frontier section (docs/briefs/reference/frontier.md) from the maps' frontier files, without a run: one
    table across the maps, the nuke line against BARb, what stood at the first missile and the missile's orders under the
    standard tax. The per-map files beside it keep every run, order and curve."""
    sets = []
    for map_name in maps:
        path = f"{REF}/frontier-{slug(map_name)}.json"
        if not os.path.exists(path):
            print(f"{map_name}: no frontier file ({path})", file=sys.stderr)
            continue
        sets.append(json.load(open(path)))
    if not sets:
        return
    never = lambda t: "never" if t is None else clock(t)
    heads = [f"{d['map']} ({d['spots'] // 2} spots a side)" for d in sets]
    out = ["## The late game by the simulator: the frontier\n",
           "The build-order simulator's fastest order to each late-game milestone, over thirty minutes, on maps of "
           + ", ".join(str(d["spots"] // 2) for d in sets) + " spots a side (`run/frontier_set.py`; every run's order and curve "
           "is in `docs/briefs/reference/frontier-<map>.md`). **A ceiling, not a target:** no enemy, nothing dies, every spot on "
           "our half is free, and the simulator runs 15 to 25 % high on income by minute 8 against quiet recorded openings. The one "
           "enemy it knows is the tax: metal a second taken for soldiers before any build. The standard tax is 3 a second from 2:00, "
           "5 from 4:00, 9 from 8:00, 12 from 12:00 and 15 from 16:00 (the experienced players' standing army on Comet Catcher grows "
           "at that rate, before losses); the heavy tax is one and a half times that. The arithmetic is the engine's: a frame builds "
           "at the sum of the power on it until the store empties, and a stockpile advances at its fixed rate, helped by nobody "
           "(K-t2-build-power-for-a-project-follows-the-spend, K-rules-a-stockpile-takes-no-build-power). The search is stochastic "
           "(simulated annealing, 20,000 steps x 8 restarts a cell); a cell out of order, a heavier tax ahead of a lighter one, is "
           "the search's noise, not the game's. `never`: not by 30:00.\n",
           "| the fastest order to (no tax / standard tax / heavy tax) | " + " | ".join(heads) + " |",
           "|---|" + "---|" * len(sets)]
    labels = [label for label, _, _ in objectives("arm")[0]]
    for label in labels:
        cells = []
        for d in sets:
            key = next(k for l, _, k in objectives(d["side"])[0] if l == label)
            cells.append(" / ".join(never(reached(d["runs"][f"{tax}|{label}"], key)) for tax, _ in TAXES))
        out.append(f"| {label} | " + " | ".join(cells) + " |")
    out.append("")
    line = []
    for d in sets:
        t = reached(d["runs"]["standard|the first missile"], ("first_round", f"{d['side']}silo"))
        line.append(f"{never(t)} on {d['map']}")
    out.append("**The nuke line.** BARb hard_aggressive's anti-nuke stands at minute 21 to 25 in 27 of 33 long games on Comet Catcher "
               "(K-barb-anti-nuke-by-minute; medium builds one in one long game of six, at 25 at the median), so a missile that is to land "
               "on him is stockpiled before about 21:00. Under the standard tax the first missile comes at " + ", ".join(line) + ". "
               "A missile is energy flow, 1,042 a second for its 120 s, and no build power.\n")
    out.append("**What stood at the first missile under the standard tax** (the state at the last whole minute before it).\n")
    out.append("| map | missile | silo stood | advanced lab | first advanced constructor | first tier-2 extractor | first fusion | extractors | constructors | turrets | build power | energy a second | income | army metal paid |")
    out.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for d in sets:
        side, r = d["side"], d["runs"]["standard|the first missile"]
        t = reached(r, ("first_round", f"{side}silo"))
        first = lambda *names: never(next((r["first_finished"][n] for n in names if n in r["first_finished"]), None))
        s = state_at(r, t)
        state = f"{s['extractors']} | {s['constructors']} | {s['nanos']} | {s['build_power']:.0f} | {s['energy_income']:.0f} | {s['income']:.0f} | {s['tax_paid']:.0f}" if s else "| | | | | | "
        out.append(f"| {d['map']} | {never(t)} | {first(f'{side}silo')} | {first(f'{side}alab', f'{side}avp', f'{side}aap')} | {first(f'{side}ack', f'{side}acv', f'{side}aca')} | "
                   f"{first(f'{side}moho')} | {first(f'{side}fus')} | {state} |")
    out.append("")
    out.append("**The missile's orders under the standard tax**, one queue a line (`com` the commander, `facN` the factories as they "
               "stand, `conN` the constructors as they leave; `help` puts the builder's power on the dearest frame begun; `assist` on the "
               "first factory; `moho` the tier-2 extractor on a standing one's spot; a constructor whose queue has run out upgrades the "
               "nearest standing extractor).\n")
    for d in sets:
        r = d["runs"]["standard|the first missile"]
        t = reached(r, ("first_round", f"{d['side']}silo"))
        lines = [l for l in r["plan"].strip().splitlines() if l.strip()]
        out.append(f"- **{d['map']}** ({never(t)}): " + " · ".join(f"`{l}`" for l in lines))
    out.append("")
    with open(f"{REF}/frontier.md", "w") as f:
        f.write("\n".join(out))
    print(f"{REF}/frontier.md written ({len(sets)} maps)")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--maps", default=",".join(MAPS))
    ap.add_argument("--brief", action="store_true", help="write the brief's frontier section from the frontier files already drawn, without a run")
    ap.add_argument("--quick", action="store_true", help="2,000 iterations x 2 restarts: a smoke of the generator, not a frontier")
    ap.add_argument("--workers", type=int, default=3)
    ap.add_argument("--iterations", type=int, default=20000)
    ap.add_argument("--restarts", type=int, default=8)
    a = ap.parse_args()
    maps = [m.strip() for m in a.maps.split(",") if m.strip()]
    if a.brief:
        brief(maps)
        return
    iterations, restarts = (2000, 2) if a.quick else (a.iterations, a.restarts)
    for map_name in maps:
        directory = latest_record(map_name)
        if not directory:
            print(f"{map_name}: no recorded game", file=sys.stderr)
            continue
        record = os.path.join(directory, "record-0.jsonl")
        header = json.loads(open(record).readline())
        side, spots = header.get("side", "arm"), len(header.get("metal_spots", []))
        objs, antinuke = objectives(side)
        # A reach row is searched twice, from the plain opening and from the tier-2 extractor's chain, and keeps the
        # earlier crossing (the plain seed finds the early incomes, the chain the late ones).
        jobs = {(tax_name, label, chain): (objective, tax) for tax_name, tax in TAXES for label, objective, _ in objs
                for chain in ([None, f"{side}moho:4"] if objective.startswith("reach:") else [None])}
        found = {}
        with concurrent.futures.ThreadPoolExecutor(a.workers) as ex:
            futures = {ex.submit(run_one, record, side, antinuke, objective, tax, iterations, restarts, chain): key for key, (objective, tax) in jobs.items() for chain in [key[2]]}
            for future in concurrent.futures.as_completed(futures):
                key = futures[future]
                found[key] = future.result()
                print(f"{map_name}: {key[0]:8s} {key[1]:28s} {'chain seed' if key[2] else 'plain seed'} score {found[key]['score']:.0f}", file=sys.stderr)
        def better(label, mine, theirs):
            """Whether run `mine` reaches the row's milestone earlier than run `theirs` (None: never)."""
            key = next(k for l, _, k in objs if l == label)
            a, b = reached(mine, key), reached(theirs, key) if theirs else None
            return theirs is None or (a is not None and (b is None or a < b))

        results = {}
        for (tax_name, label, chain), r in found.items():
            key = (tax_name, label)
            if better(label, r, results.get(key)):
                results[key] = r
        # The repair pass: the search is stochastic, and a cell a heavier tax beat is a cell the search missed, not a
        # fact about the game. Such a cell is searched again from other seeds and keeps its best.
        repaired = 0
        for attempt in (2, 3):
            retry = []
            for label, objective, key in objs:
                times = [reached(results[(tax_name, label)], key) for tax_name, _ in TAXES]
                for i in range(len(TAXES)):
                    if any(times[j] is not None and (times[i] is None or times[j] < times[i]) for j in range(i + 1, len(TAXES))):
                        retry.append((TAXES[i][0], label, objective, TAXES[i][1]))
            if not retry:
                break
            with concurrent.futures.ThreadPoolExecutor(a.workers) as ex:
                futures = {}
                for tax_name, label, objective, tax in retry:
                    for chain in ([None, f"{side}moho:4"] if objective.startswith("reach:") else [None]):
                        futures[ex.submit(run_one, record, side, antinuke, objective, tax, iterations, restarts, chain, attempt)] = (tax_name, label)
                for future in concurrent.futures.as_completed(futures):
                    key = futures[future]
                    r = future.result()
                    if better(key[1], r, results.get(key)):
                        results[key] = r
                        repaired += 1
                    print(f"{map_name}: repair {attempt} {key[0]:8s} {key[1]:28s} score {r['score']:.0f}", file=sys.stderr)
        print(f"{map_name}: {repaired} cells improved by the repair pass", file=sys.stderr)
        name = slug(map_name)
        with open(f"{REF}/frontier-{name}.json", "w") as f:
            json.dump({"map": map_name, "record": record, "side": side, "spots": spots, "minutes": MINUTES, "iterations": iterations, "restarts": restarts,
                       "repaired": repaired, "taxes": dict(TAXES), "runs": {f"{tax}|{label}": r for (tax, label), r in results.items()}}, f, indent=1)
        with open(f"{REF}/frontier-{name}.md", "w") as f:
            f.write(tables(map_name, record, results, side, spots, repaired))
        print(f"{REF}/frontier-{name}.md written ({len(results)} runs)")
    brief(maps)


if __name__ == "__main__":
    main()
