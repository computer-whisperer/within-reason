#!/usr/bin/env python3
"""A nuclear missile on one standing unit, in the engine: a silo alone, fed, fires its first missile at the place the
target stands (or a given offset from it), and the per-second log says what the target has left and in how many
blows. Written for docs/studies/2026-11-08-tier-3.md §5.

    run/nuke_direct.py [--silo armsilo,corsilo] [--offset 0,100,300] [--map comet|flat] unit [unit ...]

One engine start a case, about 20 s of wall each. The target's units stand by `--theirs track`; the aim is where the
unit stood a second before the launch in a first run of the same case (a big unit is spawned a little off its place).
Cortex's missile is metal-bound on a commander's income (ready near 250 s), so it is fired at 290 s.
"""
import glob, json, os, subprocess, sys, tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
U = json.load(open(os.path.join(REPO, "crates/bot/data/units.json")))["units"]
MAPS = {"comet": ("Comet Catcher Remake 1.8", [8192, 6144], (4096, 3072), (6600, 2992)),
        "flat": ("Full Metal Plate 1.7", [12288, 12288], (6144, 6144), (8644, 6144))}
FIRE = {"armsilo": (150.0, 230), "corsilo": (290.0, 350), "legsilo": (290.0, 350)}


def run(work, silo, unit, where, aim, label):
    name, size, centre, at = MAPS[where]
    scenario = {"format": "within-reason-scenario", "version": 1, "map": name, "map_size": size, "centre": list(centre),
                "source": {"note": f"one {silo} fires its first missile at one {unit}"},
                "sides": [{"name": "ours", "units": [{"type": silo, "x": centre[0], "z": centre[1]}]},
                          {"name": "theirs", "units": [{"type": unit, "x": at[0], "z": at[1]}],
                           "track": [{"t": 0.0, "unit": 0, "x": at[0], "z": at[1]}, {"t": 400.0, "unit": 0, "x": at[0], "z": at[1]}]}]}
    seconds, limit = FIRE[silo]
    json.dump(scenario, open(f"{work}/scenario.json", "w"))
    json.dump({"rows": [{"t": seconds, "unit": 0, "cmd": "fire", "x": aim[0], "z": aim[1]}]}, open(f"{work}/fire.json", "w"))
    subprocess.run([os.path.join(REPO, "target/release/duel"), "--scenario", f"{work}/scenario.json", "--script", f"{work}/fire.json",
                    "--theirs", "track", "--economy", "fed*24", "--time-limit", str(limit), "--reps", "1", "--label", label],
                   cwd=REPO, check=True, capture_output=True)
    batch = sorted(glob.glob(os.path.join(REPO, f"run/matches/*-duel-{label}")))[-1]
    rows = [json.loads(line) for line in open(glob.glob(batch + "/seconds/*.jsonl")[0])]
    wiped = open(batch + "/duels.csv").read().split("\n")[1].split(",")[13] == "wiped"
    return rows, wiped


def main():
    args, silos, offsets, where = sys.argv[1:], ["armsilo", "corsilo"], [0], "comet"
    units = []
    while args:
        a = args.pop(0)
        if a == "--silo":
            silos = args.pop(0).split(",")
        elif a == "--offset":
            offsets = [int(x) for x in args.pop(0).split(",")]
        elif a == "--map":
            where = args.pop(0)
        else:
            units.append(a)
    if not units:
        sys.exit(__doc__)
    with tempfile.TemporaryDirectory() as work:
        for unit in units:
            health = U[unit]["health"]
            for silo in silos:
                # Where it stands: the first run is aimed at the place it was given.
                rows, _ = run(work, silo, unit, where, MAPS[where][3], f"nuke-stand-{silo}-{unit}")
                stood = next(r["y"][0][1:3] for r in rows if r["y"])
                for off in offsets:
                    rows, wiped = run(work, silo, unit, where, (stood[0] + off, stood[1]), f"nuke-direct-{silo}-{unit}-{off}")
                    blows, was = [], 1.0
                    for r in rows:
                        now = r["y"][0][3] if r["y"] else 0.0
                        if now < was - 1e-4:
                            blows.append((r["t"], round((was - now) * health)))
                            was = now
                    left = 0.0 if wiped else was
                    print(f"{U[unit]['name']:14s} {health:>7,}  {silo}  aimed {off:3d} off: "
                          f"{'dead' if left == 0 else f'{100 * left:.1f} % left'}; blows (second, damage) {blows}", flush=True)


if __name__ == "__main__":
    main()
