#!/usr/bin/env python3
"""The late-game frontier of a map: the fastest orders the build-order simulator finds to an income of 50, 100 and
200 a second, to the first tier-2 extractor, fusion, advanced fusion, anti-nuke and nuclear missile, each under three
taxes (metal a second that must go to soldiers before any build: none, the standard one, a heavy one), as tables for the
brief (docs/design/2026-10-09-late-economy.md §4.1 and §9). Read every number as a ceiling: the simulator has no enemy
(nothing is raided, nothing dies, every spot on our half is free) and runs 15 to 25 % high on income by minute 8
against quiet recorded openings (docs/studies/build-order.md); the tax is the only cost of the enemy it knows.

    run/frontier_set.py [--maps "Great Divide V1,Comet Catcher Remake 1.8"] [--quick] [--workers 4] [--iterations 20000] [--restarts 4]

Writes docs/briefs/reference/frontier-<map slug>.md (the tables) and .json (every run: plan, firsts, state by minute)
for each map, from the latest recorded game on it (run/build_brief.py's roots). run/build_brief.py assembles the .md
files into the brief.
"""
import argparse, concurrent.futures, json, os, re, subprocess, sys, tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "run"))
from build_brief import latest_record  # noqa: E402  (chdirs to the repo)

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
        ("the first tier-2 extractor", f"target:{side}moho@{MINUTES * 60}", ("first_finished", f"{side}moho")),
        ("the first fusion", f"target:{side}fus@{MINUTES * 60}", ("first_finished", f"{side}fus")),
        ("the first advanced fusion", f"target:{side}afus@{MINUTES * 60}", ("first_finished", f"{side}afus")),
        ("the first anti-nuke", f"target:{antinuke}@{MINUTES * 60}", ("first_finished", antinuke)),
        ("the first missile", f"target:round:{side}silo@{MINUTES * 60}", ("first_round", f"{side}silo")),
    ], antinuke


def run_one(record, side, antinuke, objective, tax, iterations, restarts, seed_chain=None):
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        out = f.name
    cmd = ["target/release/buildorder", "optimize", "--game", record, "--palette", "roster", "--also", f"{side}silo,{antinuke}", "--objective", objective,
           "--minutes", str(MINUTES), "--iterations", str(iterations), "--restarts", str(restarts), "--seed", "1", "--json-out", out]
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


def tables(map_name, record, results, side, spots):
    objs, _ = objectives(side)
    out = [f"### The late-game frontier: {map_name}\n",
           f"{spots // 2} spots a side ({spots} on the map), the {side} side, from the recorded game `{record.split('/')[-3]}`; the "
           f"build-order simulator's best orders over {MINUTES} minutes (`run/frontier_set.py`). **A ceiling, not a target:** no enemy, "
           "nothing dies, every spot on our half is free, and the simulator runs 15 to 25 % high on income by minute 8 against quiet "
           "recorded openings. The tax is the enemy it knows: metal a second taken for soldiers before any build, from each time. "
           "Standard: 3 a second from 2:00, 5 from 4:00, 9 from 8:00, 12 from 12:00, 15 from 16:00 (the experienced players' standing "
           "army on Comet grows at that rate, before losses); heavy: one and a half times that. The arithmetic the frontier rests on is "
           "the engine's (a frame builds at the sum of the power on it until the store empties; a stockpile at its fixed rate, helped by "
           "nobody: K-t2-build-power-for-a-project-follows-the-spend, K-rules-a-stockpile-takes-no-build-power).\n",
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


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--maps", default=",".join(MAPS))
    ap.add_argument("--quick", action="store_true", help="2,000 iterations x 2 restarts: a smoke of the generator, not a frontier")
    ap.add_argument("--workers", type=int, default=3)
    ap.add_argument("--iterations", type=int, default=20000)
    ap.add_argument("--restarts", type=int, default=8)
    a = ap.parse_args()
    iterations, restarts = (2000, 2) if a.quick else (a.iterations, a.restarts)
    for map_name in [m.strip() for m in a.maps.split(",") if m.strip()]:
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
        results = {}
        for (tax_name, label, chain), r in found.items():
            key = (tax_name, label)
            objective = next(o for l, o, _ in objs if l == label)
            if objective.startswith("reach:"):
                n = objective.split(":")[1]
                earlier = results.get(key)
                mine, theirs = r["reach"].get(n), earlier["reach"].get(n) if earlier else None
                if earlier is None or (mine is not None and (theirs is None or mine < theirs)):
                    results[key] = r
            else:
                results[key] = r
        name = slug(map_name)
        with open(f"{REF}/frontier-{name}.json", "w") as f:
            json.dump({"map": map_name, "record": record, "side": side, "spots": spots, "minutes": MINUTES, "iterations": iterations, "restarts": restarts,
                       "taxes": dict(TAXES), "runs": {f"{tax}|{label}": r for (tax, label), r in results.items()}}, f, indent=1)
        with open(f"{REF}/frontier-{name}.md", "w") as f:
            f.write(tables(map_name, record, results, side, spots))
        print(f"{REF}/frontier-{name}.md written ({len(results)} runs)")


if __name__ == "__main__":
    main()
