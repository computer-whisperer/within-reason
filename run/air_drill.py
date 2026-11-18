#!/usr/bin/env python3
"""Read air-drill batches (`run/air_scene.py` cuts flown by `target/release/duel --scenario ... --script ...`): one
row a batch, from the per-second logs.

    run/air_drill.py <batch dir> ... [--reps] [--places] [--leash]

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

`--places` adds a second table, where the fighters were from the merge (the first second a fighter is within 300 of
the rearmost bomber along its track) to the first bomb or the last bomber's death: the share of the fighter-seconds
in firing position (behind the rearmost bomber by no more than 550 and within 300 across), still flying at the
bombers (inbound), ahead of the rearmost while flying its way, wide (more than 300 across) and far (more than 550
behind), each with the hits a fighter-second landed there; and the seconds from first contact to the merge and from
the merge to the end.

`--leash` adds a table of how far the fighters went from the place they keep (the scenario's centre) and whether
they came back: the furthest any fighter was from it (the mean over the repetitions of each one's furthest, and the
furthest of all), the seconds in which some fighter was more than 1,500 from it, the share of the fighter-seconds
spent there, how far the furthest fighter was from it in the last second, the most fighters the wing had off its loop at once (flying to meet
something, turning in or attacking, by the lane's own report), the fighters lost and what of his died.
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


PLACES = ("firing", "inbound", "ahead", "wide", "far")


def places(batch, scenario):
    """Where the fighters were, second by second, against the bombers' rearmost and their mean heading."""
    fighters = set(scenario["source"].get("fighters", []))
    seconds, hits = dict.fromkeys(PLACES, 0), dict.fromkeys(PLACES, 0)
    leads, spans = [], []
    for log in sorted(glob.glob(os.path.join(batch, "seconds", "*.jsonl"))):
        before_theirs, before_ours, merge, contact = {}, {}, None, None
        for line in open(log):
            r = json.loads(line)
            if contact is None and any(o.get("contact") == 0 for o in r["orders"]):
                contact = r["t"]
            bombers = [u for u in r["y"] if u[3] > 0]
            if r["dealt"]["y"] or (merge is not None and not bombers):
                if merge is not None:
                    spans.append(r["t"] - merge)
                break
            moved = [(u[1] - before_theirs[u[0]][0], u[2] - before_theirs[u[0]][1]) for u in bombers if u[0] in before_theirs]
            if moved:
                vx, vz = sum(m[0] for m in moved) / len(moved), sum(m[1] for m in moved) / len(moved)
                speed = math.hypot(vx, vz)
                if speed > 50:
                    ux, uz = vx / speed, vz / speed
                    rear = min(bombers, key=lambda u: u[1] * ux + u[2] * uz)
                    here = []
                    for u in r["x"]:
                        if u[0] in fighters:
                            dx, dz = u[1] - rear[1], u[2] - rear[2]
                            was = before_ours.get(u[0])
                            its_way = (u[1] - was[0]) * ux + (u[2] - was[1]) * uz if was else 0.0
                            here.append((-(dx * ux + dz * uz), ux * dz - uz * dx, its_way, sum(1 for d in r["dealt"]["x"] if d[0] == u[0])))
                    if merge is None and any(behind > -300 and abs(across) < 700 for behind, across, _, _ in here):
                        merge = r["t"]
                        if contact is not None:
                            leads.append(merge - contact)
                    if merge is not None:
                        for behind, across, its_way, landed in here:
                            place = ("inbound" if its_way < 0 else "ahead") if behind < 0 else "far" if behind > 550 else "wide" if abs(across) > 300 else "firing"
                            seconds[place] += 1
                            hits[place] += landed
            before_theirs = {u[0]: (u[1], u[2]) for u in r["y"]}
            before_ours = {u[0]: (u[1], u[2]) for u in r["x"]}
    return seconds, hits, leads, spans


AWAY = 1500.0


def leash(batch, scenario):
    """How far the fighters went from the scenario's centre, repetition by repetition."""
    fighters = set(scenario["source"].get("fighters", []))
    tx, tz = scenario["centre"]
    his = len(scenario["sides"][1]["units"])
    runs = []
    for log in sorted(glob.glob(os.path.join(batch, "seconds", "*.jsonl"))):
        furthest, away, seconds, there, last, alive, his_left, off = 0.0, 0, 0, 0, 0.0, len(fighters), his, 0
        for line in open(log):
            r = json.loads(line)
            out = [math.hypot(u[1] - tx, u[2] - tz) for u in r["x"] if u[0] in fighters]
            if not out:
                alive = 0
                continue
            furthest = max(furthest, max(out))
            away += max(out) > AWAY
            seconds += len(out)
            there += sum(d > AWAY for d in out)
            last, alive, his_left = max(out), len(out), sum(1 for u in r["y"] if u[3] > 0)
            wing = r.get("wing") or {}
            off = max(off, wing.get("closing", 0) + wing.get("turning", 0) + wing.get("trailing", 0))
        runs.append({"furthest": furthest, "away": away, "share": there / max(1, seconds), "last": last, "lost": len(fighters) - alive, "his_lost": his - his_left, "off": off})
    return runs, his


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
    if "--places" in sys.argv:
        print()
        print("| batch | merge s after contact | merge to the end s | " + " | ".join(f"{p} (hits a second there)" for p in PLACES) + " | hits a fighter-second |")
        print("|---|---|---|" + "---|" * (len(PLACES) + 1))
        for batch in args:
            info = json.load(open(os.path.join(batch, "batch.json")))
            scenario = json.load(open(info["argv"][info["argv"].index("--scenario") + 1]))
            seconds, hits, leads, spans = places(batch, scenario)
            total = sum(seconds.values())
            if not total:
                continue
            cells = " | ".join(f"{100 * seconds[p] / total:.0f} % ({hits[p] / max(1, seconds[p]):.2f})" for p in PLACES)
            print(f"| {info['label']} | {show(mean(leads), 1)} | {show(mean(spans), 1)} | {cells} | {sum(hits.values()) / total:.2f} |")
    if "--leash" in sys.argv:
        print()
        print(f"| batch | reps | furthest from the place (mean, most) | seconds with a fighter over {AWAY:.0f} out | fighter-seconds out there | furthest at the end | most off the loop at once | fighters lost | his lost |")
        print("|---|---|---|---|---|---|---|---|---|")
        for batch in args:
            info = json.load(open(os.path.join(batch, "batch.json")))
            scenario = json.load(open(info["argv"][info["argv"].index("--scenario") + 1]))
            runs, his = leash(batch, scenario)
            if not runs:
                continue
            pick = lambda key: [r[key] for r in runs]
            print(
                f"| {info['label']} | {len(runs)} | {mean(pick('furthest')):.0f}, {max(pick('furthest')):.0f} | {mean(pick('away')):.1f} | {100 * mean(pick('share')):.0f} % | "
                f"{mean(pick('last')):.0f} | {mean(pick('off')):.1f} | {mean(pick('lost')):.1f} of {len(scenario['source'].get('fighters', []))} | {mean(pick('his_lost')):.1f} of {his} |"
            )


if __name__ == "__main__":
    main()
