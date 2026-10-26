#!/usr/bin/env python3
"""The distilled bot against air (docs/design/2026-10-25-distilled-air.md, "The bench").

    run/distilled_air.py TAG [TAG...]

For the newest batch of each map labelled distN-TAG-MAP (run/distilled_bench.sh), game by game and pooled:
- the outcome and the minute it came;
- our metal lost to his weapons by killer: his aircraft, his ground units, or a blast (a weapon no unit owns), from
  the bot's `died:` lines;
- his air: the metal of the distinct armed aircraft of his the truth file saw finished, and on the map at 10, 15, 20
  and 25 minutes;
- with the `air:` lines (D-AIR-SEEN to D-BOMB): when the aircraft plant stood (its `begun:` and the first `air:` line
  that says standing), the fighters and bombers at the end against those wanted, the Nettles, and the sorties: flown,
  bombers lost, his economy buildings destroyed and their worth as the bot saw them; and from the truth file, his
  economy buildings within 400 of each sortie's target that were gone 60 s after it set out (the bot sees only what is
  in sight when it dies, and a chain goes on after the bombers turn for home).
A TAG names the batches distN-TAG-MAP of the three maps, or one batch by its whole label.
Unit numbers (metal, whether it flies) are the combat tables' (crates/combatsim/data/units.json); his economy
buildings are the glossary's generators, converters and energy storage (docs/glossary/units.csv).
"""
import csv, glob, json, os, re, statistics, sys

MAPS = ["comet", "glitters", "quicksilver"]
MARKS = (10, 15, 20, 25)
LINE = re.compile(r"\[distilled ai=\d+\] f=(\d+) (.*)")
DIED = re.compile(r"died: (\w+) unit \d+ .*? killer=(\S+) kind=(\w+)")
AIR = re.compile(r"air: shown (\d+) metal.*?\| plant (\w+).*?\| fighters (\d+)/(\d+) \| bombers (\d+)/(\d+) \| nettles (\d+) \| sorties (\d+) \((\d+) bombers lost, (\d+) buildings worth (\d+)\)")
BEGUN = re.compile(r"begun: (\w+) by unit")
SORTIE = re.compile(r"sortie: (\d+) bombers .*? at \((-?\d+),(-?\d+)\)")
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
UNITS = json.load(open(f"{ROOT}/crates/combatsim/data/units.json"))["units"]
ECONOMY = {r["unit"]: float(r["metal"] or 0) + float(r["energy"] or 0) / 70 for r in csv.DictReader(open(f"{ROOT}/docs/glossary/units.csv"))
           if r["class"] in ("solar collector", "wind generator", "tidal generator", "energy converter", "energy storage", "fusion reactor", "geothermal plant")}
BOMBED_WITHIN = 400.0
BOMBED_AFTER = 60.0


def metal(name):
    return UNITS.get(name, {}).get("metal", 0)


def flies(name):
    return UNITS.get(name, {}).get("air", False)


def armed(name):
    return bool(UNITS.get(name, {}).get("weapons"))


def read(path):
    g = {"lost": {"aircraft": 0.0, "ground": 0.0, "blast": 0.0}, "air": [], "plant": None, "plant_begun": None, "sorties": []}
    log = f"{path}/engine.log"
    if os.path.exists(log):
        for line in open(log, errors="replace"):
            m = LINE.search(line)
            if not m:
                continue
            t, text = int(m.group(1)) / 1800.0, m.group(2)
            if text.startswith("died: "):
                d = DIED.match(text)
                if d and d.group(3) != "cause":
                    by = d.group(2)
                    kind = "blast" if by not in UNITS else ("aircraft" if flies(by) else "ground")
                    g["lost"][kind] += metal(d.group(1))
            elif text.startswith("air: shown"):
                d = AIR.match(text)
                if d:
                    g["air"].append((t, [int(x) if x.isdigit() else x for x in d.groups()]))
                    if d.group(2) == "standing" and g["plant"] is None:
                        g["plant"] = t
            elif text.startswith("sortie: "):
                d = SORTIE.match(text)
                if d:
                    g["sorties"].append((t * 60, float(d.group(2)), float(d.group(3))))
            elif text.startswith("begun: "):
                d = BEGUN.match(text)
                if d and d.group(1) in ("armap", "corap", "legap") and g["plant_begun"] is None:
                    g["plant_begun"] = t
    built, at = {}, {m: 0.0 for m in MARKS}
    # Per sortie, his economy buildings near the target when it set out, and those of them still there 60 s on.
    near = [None] * len(g["sorties"])
    after = [None] * len(g["sorties"])
    truth = glob.glob(f"{path}/truth-*.jsonl")
    if truth:
        for line in open(truth[0]):
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            t = row["f"] / 1800.0
            for k, (s, x, z) in enumerate(g["sorties"]):
                here_now = {uid: ECONOMY[n] for uid, n, ux, uz, _, framed in row["enemy"] if not framed and n in ECONOMY and (ux - x) ** 2 + (uz - z) ** 2 <= BOMBED_WITHIN ** 2}
                if near[k] is None and row["f"] / 30.0 >= s:
                    near[k] = here_now
                if after[k] is None and row["f"] / 30.0 >= s + BOMBED_AFTER:
                    after[k] = {uid for uid, _, _, _, _, _ in row["enemy"]}
            here = 0.0
            for uid, name, _, _, _, framed in row["enemy"]:
                if framed or not flies(name) or not armed(name):
                    continue
                built[uid] = name
                here += metal(name)
            for mark in MARKS:
                if abs(t - mark) < 1 / 60:
                    at[mark] = here
    g["bombed"] = [0, 0.0]
    for k in range(len(g["sorties"])):
        if near[k] is None:
            continue
        gone = [w for uid, w in near[k].items() if after[k] is None or uid not in after[k]]
        g["bombed"][0] += len(gone)
        g["bombed"][1] += sum(gone)
    g["his_air_built"] = sum(metal(n) for n in built.values())
    g["his_air_at"] = at
    return g


def main():
    tags = sys.argv[1:]
    if not tags:
        print(__doc__.split("\n\n")[1].strip())
        sys.exit(2)
    for tag in tags:
        pooled = {"outcome": [], "aircraft": [], "ground": [], "his_built": [], "sorties": [0, 0, 0, 0], "bombed": [0, 0.0]}
        print(f"=== {tag}")
        whole = sorted(glob.glob(f"run/matches/*-{tag}"), key=lambda d: os.path.basename(d).split("-")[0])
        batches = [whole[-1]] if whole else []
        for name in MAPS:
            found = sorted(glob.glob(f"run/matches/*-dist*-{tag}-{name}"), key=lambda d: os.path.basename(d).split("-")[0])
            if found:
                batches.append(found[-1])
        for batch in batches:
            results = {}
            for line in open(f"{batch}/results.jsonl"):
                r = json.loads(line)
                results[r["index"]] = r
            print(f"-- {os.path.basename(batch)}")
            for i in sorted(results):
                r = results[i]
                g = read(f"{batch}/{i:02d}")
                pooled["outcome"].append(r["outcome"])
                pooled["aircraft"].append(g["lost"]["aircraft"])
                pooled["ground"].append(g["lost"]["ground"])
                pooled["his_built"].append(g["his_air_built"])
                air = ""
                if g["air"]:
                    _, last = g["air"][-1]
                    shown, plant, fighters, wanted, bombers, squad, nettles, flown, lost, killed, worth = last
                    plant_at = f"{g['plant']:.1f}" if g["plant"] is not None else "-"
                    air = f" | plant {plant_at} | fighters {fighters}/{wanted} bombers {bombers}/{squad} nettles {nettles} | sorties {flown}, {lost} lost, {killed} seen destroyed ({worth}), {g['bombed'][0]} gone by the truth ({g['bombed'][1]:.0f})"
                    for k, v in enumerate((flown, lost, killed, worth)):
                        pooled["sorties"][k] += v
                    pooled["bombed"][0] += g["bombed"][0]
                    pooled["bombed"][1] += g["bombed"][1]
                marks = " ".join(f"{g['his_air_at'][m]:.0f}" for m in MARKS)
                print(f"{i:02d} {r['outcome']:7} {r['game_minutes']:4.1f} {r['our_side'][:3]} | lost to aircraft {g['lost']['aircraft']:6.0f} ground {g['lost']['ground']:6.0f} blast {g['lost']['blast']:5.0f} | his air built {g['his_air_built']:6.0f}, on the map {marks}{air}")
        o = pooled["outcome"]
        if not o:
            continue
        print(f"pooled: {o.count('Win')} won, {o.count('Loss')} lost, {o.count('Timeout')} to the cap of {len(o)}; "
              f"lost to aircraft median {statistics.median(pooled['aircraft']):.0f}, all {sum(pooled['aircraft']):.0f}; to ground median {statistics.median(pooled['ground']):.0f}; "
              f"his air built median {statistics.median(pooled['his_built']):.0f}; sorties {pooled['sorties'][0]}, bombers lost {pooled['sorties'][1]}, his economy buildings seen destroyed {pooled['sorties'][2]} worth {pooled['sorties'][3]}, gone by the truth {pooled['bombed'][0]} worth {pooled['bombed'][1]:.0f}")


if __name__ == "__main__":
    main()
