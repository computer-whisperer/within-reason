#!/usr/bin/env python3
"""Read air-drill batches (`run/air_scene.py` cuts flown by `target/release/duel --scenario ... --script ...`): one
row a batch, from the per-second logs.

    run/air_drill.py <batch dir> ... [--reps]

Per batch, the mean over its repetitions (and each repetition with `--reps`):
  warning    seconds from our side's first contact (sight or radar) to the first harm to anything of ours (a bomb's
             damage, or a unit gone); over the repetitions that had any
  out        how far the nearest bomber was from the target at first contact
  first hit  seconds from first contact to the fighters' first damage on a bomber (the fighters are the units the cut
             added as such); "-" with none
  fighters   the damage the fighters did before the first harm, as a share of the bombers' whole health
  bombed     bombers whose bombs hit something of ours, of those on the run
  saved      repetitions that kept every reactor
  lost       metal of ours destroyed, the fighters apart: the mean, and the least and the most
  fighters lost
"""
import csv, glob, json, math, os, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REACTORS = ("armfus", "armafus", "corfus", "corafus")


def table():
    with open(os.path.join(REPO, "docs/glossary/units.csv")) as f:
        rows = list(csv.DictReader(f))
    return {r["unit"]: float(r["metal"] or 0) for r in rows}, {r["unit"]: float(r["health"] or 0) for r in rows}


def read(log, scenario, metal, health):
    rows = [json.loads(line) for line in open(log)]
    ours, theirs = scenario["sides"][0]["units"], scenario["sides"][1]["units"]
    fighters = set(scenario["source"].get("fighters", []))
    tx, tz = scenario["centre"]
    contact = next((r["t"] for r in rows for o in r["orders"] if o.get("contact") == 0), None)
    out, harm, before, first_hit = None, None, None, None
    bombed, fighter_damage = set(), 0.0
    for r in rows:
        alive = [u for u in r["y"] if u[3] > 0]
        if r["t"] == contact and alive:
            out = min(math.hypot(u[1] - tx, u[2] - tz) for u in alive)
        bombed |= {d[0] for d in r["dealt"]["y"]}
        # The first harm: a bomb's damage (the bomber is named), or something of ours gone.
        now = sum(1 for u in r["x"] if u[0] not in fighters)
        if harm is None and (r["dealt"]["y"] or (before is not None and now < before)):
            harm = r["t"]
        before = now
        by_fighters = sum(d[2] for d in r["dealt"]["x"] if d[0] in fighters)
        if by_fighters and first_hit is None:
            first_hit = r["t"]
        if harm is None:
            fighter_damage += by_fighters
    end = {u[0] for u in rows[-1]["x"]}
    lost = [u["type"] for i, u in enumerate(ours) if i not in end and i not in fighters]
    whole = sum(health.get(u["type"], 0) for u in theirs)
    return {
        "warning": None if contact is None or harm is None else harm - contact,
        "out": out,
        "first_hit": None if first_hit is None or contact is None else first_hit - contact,
        "fighter_share": fighter_damage / whole if whole else 0.0,
        "bombed": len(bombed),
        "bombers": len(theirs),
        "lost": sum(metal.get(t, 0) for t in lost),
        "saved": not any(t in REACTORS for t in lost),
        "fighters_lost": sum(1 for i in fighters if i not in end),
        "fighters": len(fighters),
    }


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    each = "--reps" in sys.argv
    if not args:
        sys.exit(__doc__)
    metal, health = table()
    mean = lambda values: sum(values) / len(values) if values else None
    show = lambda v, digits=0: "-" if v is None else f"{v:.{digits}f}"
    print("| batch | reps | warning s | out | first hit s | fighters' damage before the first harm | bombed | saved | lost metal | fighters lost |")
    print("|---|---|---|---|---|---|---|---|---|---|")
    for batch in args:
        info = json.load(open(os.path.join(batch, "batch.json")))
        scenario = json.load(open(info["argv"][info["argv"].index("--scenario") + 1]))
        runs = [read(log, scenario, metal, health) for log in sorted(glob.glob(os.path.join(batch, "seconds", "*.jsonl")))]
        if not runs:
            continue
        pick = lambda key: [r[key] for r in runs if r[key] is not None]
        lost = pick("lost")
        print(
            f"| {info['label']} | {len(runs)} | {show(mean(pick('warning')), 1)} | {show(mean(pick('out')))} | {show(mean(pick('first_hit')), 1)} | "
            f"{100 * mean(pick('fighter_share')):.0f} % | {show(mean(pick('bombed')), 1)} of {runs[0]['bombers']} | {sum(r['saved'] for r in runs)} of {len(runs)} | "
            f"{show(mean(lost))} ({show(min(lost))}-{show(max(lost))}) | {show(mean(pick('fighters_lost')), 1)} of {runs[0]['fighters']} |"
        )
        if each:
            for k, r in enumerate(runs):
                print(f"|   rep {k} | | {show(r['warning'])} | {show(r['out'])} | {show(r['first_hit'])} | {100 * r['fighter_share']:.0f} % | {r['bombed']} | {'yes' if r['saved'] else 'no'} | {show(r['lost'])} | {r['fighters_lost']} |")


if __name__ == "__main__":
    main()
