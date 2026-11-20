#!/usr/bin/env python3
"""The flagship's matchup table from the two duel matrices it is the mean of.

    run/matchup_table.py <tight batch dir> <wide batch dir> --date 2026-11-19     # write the data set and the table
    run/matchup_table.py <tight batch dir> <wide batch dir> --date ... --dry-run  # only say what would change

`crates/bot/data/matchups.csv` (`unit,against,margin`; the flagship's fight prediction reads it,
`crates/bot/src/brain/combat.rs`) is the mean of two engine-measured tables: every pairing of the tier-1 land units
at equal metal, fought with the armies in ranks 56 apart (tight) and 100 apart (wide). The two batches are flown
with the duel harness (docs/harness/duels.md); the commands are in docs/harness/engine.md, "Game data", and are
printed by `--commands`.

This takes the two batch directories, checks they are what the table wants (one game and engine for both, the
spacings 56 and 100, every ordered pairing there, nothing unspawned), and writes

  - docs/data/duels-<date>/{tight,wide}-duels.csv   one row a duel, the columns of the first set (2026-09-19)
  - docs/data/duels-<date>/{tight,wide}-pairs.csv   one row an ordered pairing, as the batch wrote it
  - docs/data/duels-<date>/{tight,wide}-matrix.md   the batch's matrix
  - docs/data/duels-<date>/README.md                what was fought, on what
  - crates/bot/data/matchups.csv                    the mean of the two tables' margins

and prints what moved against the table that stood. The combat simulator is judged against the same two tables
(`crates/combatsim/src/duels.rs` names the directory): point it at the new one by hand and run its tests.
"""
import argparse
import csv
import json
import os
import shlex
import shutil
import statistics
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TABLE = os.path.join(REPO, "crates", "bot", "data", "matchups.csv")
UNITS = ("armart armfav armflash armflea armham armjanus armjeth armllt armpw armrock armstump armwar "
         "corak corcrash corfav corgator corlevlr corllt cormist corraid corstorm corthud corwolv").split()
# The first set's columns: what every reader of a duels.csv of these sets may count on.
DUEL_COLUMNS = ("match site sequence x y rep x_end x_team n_x n_y metal_x metal_y winner reason seconds contact_seconds "
                "survivors_x survivors_y value_left_x value_left_y damage_taken_x damage_taken_y").split()
SPACING = {"tight": 56.0, "wide": 100.0}
REPS = {"tight": 8, "wide": 4}


def commands(map_name):
    units = ",".join(UNITS)
    for name in ("tight", "wide"):
        print(f'target/release/duel --units {units} --reps {REPS[name]} --spacing {SPACING[name]:g} --speed 200 '
              f'--parallel 6 --map "{map_name}" --label matchups-{name}')


def read_batch(directory, name):
    """A batch's facts, its duels and its pairs, with everything the table cannot stand on said as an error."""
    batch = json.load(open(os.path.join(directory, "batch.json"), encoding="utf-8"))
    problems = []
    if abs(float(batch.get("spacing", 0)) - SPACING[name]) > 0.01:
        problems.append(f"spacing {batch.get('spacing')}, the {name} table is fought at {SPACING[name]:g}")
    if batch.get("lanes") not in (["off", "off"], None):
        problems.append(f"lanes {batch.get('lanes')}: the table is fought with no lane")
    duels = list(csv.DictReader(open(os.path.join(directory, "duels.csv"), newline="", encoding="utf-8")))
    pairs = list(csv.DictReader(open(os.path.join(directory, "pairs.csv"), newline="", encoding="utf-8")))
    bad = [row["reason"] for row in duels if row["reason"] not in ("wiped", "timeout", "stalemate")]
    if bad:
        problems.append(f"{len(bad)} duels not fought ({', '.join(sorted(set(bad)))})")
    have = {(row["unit"], row["against"]) for row in pairs}
    want = {(a, b) for a in UNITS for b in UNITS if not (a.endswith("llt") and b.endswith("llt"))}
    if have != want:
        problems.append(f"pairings: {len(want - have)} missing ({sorted(want - have)[:4]}...), {len(have - want)} not of the table")
    short = sorted({(row["unit"], row["against"]) for row in pairs if int(row["duels"]) < REPS[name] and row["unit"] != row["against"]})
    if short:
        problems.append(f"{len(short)} pairings with fewer than {REPS[name]} duels ({short[:4]}...)")
    return batch, duels, pairs, problems


def margins(pairs):
    return {(row["unit"], row["against"]): float(row["mean_margin"]) for row in pairs}


def standing():
    out = {}
    for line in open(TABLE, encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        unit, against, margin = line.strip().split(",")
        out[(unit, against)] = float(margin)
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("tight", nargs="?", help="the batch fought at spacing 56")
    ap.add_argument("wide", nargs="?", help="the batch fought at spacing 100")
    ap.add_argument("--date", help="the data set's date, YYYY-MM-DD (docs/data/duels-<date>)")
    ap.add_argument("--dry-run", action="store_true", help="write nothing")
    ap.add_argument("--commands", metavar="MAP", help="print the two duel commands for this map, and stop")
    args = ap.parse_args()
    if args.commands:
        commands(args.commands)
        return
    if not (args.tight and args.wide and args.date):
        ap.error("two batch directories and --date")
    read = {name: read_batch(directory, name) for name, directory in (("tight", args.tight), ("wide", args.wide))}
    problems = [f"{name}: {p}" for name, (_, _, _, ps) in read.items() for p in ps]
    for key in ("game", "engine", "map", "sizing", "commit"):
        values = {name: read[name][0].get(key) for name in read}
        if key != "commit" and values["tight"] != values["wide"]:
            problems.append(f"the two batches differ in {key}: {values}")
    if problems:
        sys.exit("not a matchup table:\n  " + "\n  ".join(problems))
    tight, wide = margins(read["tight"][2]), margins(read["wide"][2])
    table = {pair: (tight[pair] + wide[pair]) / 2 for pair in tight}
    old = standing()
    moved = sorted(((table[p] - old[p], p) for p in table if p in old and p[0] < p[1]), key=lambda t: -abs(t[0]))
    shifts = [abs(d) for d, _ in moved]
    flips = [(d, p) for d, p in moved if abs(old[p]) >= 0.10 and abs(table[p]) >= 0.10 and (old[p] > 0) != (table[p] > 0)]
    print(f"{len(table)} ordered pairings; against the table that stood, over {len(moved)} pairings of two units: "
          f"median shift {statistics.median(shifts):.3f}, mean {statistics.fmean(shifts):.3f}, "
          f"beyond 0.10 in {sum(s > 0.10 for s in shifts)}, beyond 0.20 in {sum(s > 0.20 for s in shifts)}; "
          f"{len(flips)} changed sign with both margins beyond 0.10")
    for d, (unit, against) in moved[:15]:
        print(f"   {unit:9s} v {against:9s} {old[(unit, against)]:+.3f} -> {table[(unit, against)]:+.3f}")
    if args.dry_run:
        return
    out = os.path.join(REPO, "docs", "data", f"duels-{args.date}")
    os.makedirs(out, exist_ok=True)
    for name, directory in (("tight", args.tight), ("wide", args.wide)):
        batch, duels, _, _ = read[name]
        with open(os.path.join(out, f"{name}-duels.csv"), "w", newline="", encoding="utf-8") as fh:
            writer = csv.DictWriter(fh, fieldnames=DUEL_COLUMNS, extrasaction="ignore", lineterminator="\n")
            writer.writeheader()
            writer.writerows(duels)
        shutil.copyfile(os.path.join(directory, "pairs.csv"), os.path.join(out, f"{name}-pairs.csv"))
        shutil.copyfile(os.path.join(directory, "matrix.md"), os.path.join(out, f"{name}-matrix.md"))
    facts = read["tight"][0]
    with open(os.path.join(out, "README.md"), "w", encoding="utf-8") as fh:
        fh.write(f"# The tier-1 duel tables of {args.date}\n\n"
                 f"Every pairing of {len(UNITS)} tier-1 land units at equal metal ({facts.get('sizing')}), no lane, both sides attack-moving\n"
                 f"(docs/harness/duels.md), on `{facts.get('game')}`, engine {facts.get('engine')}, {facts.get('map')}.\n\n")
        for name in ("tight", "wide"):
            batch, duels, _, _ = read[name]
            fh.write(f"- **{name}**: ranks {SPACING[name]:g} apart, {len(duels)} duels, batch `{os.path.basename(os.path.normpath(args.tight if name == 'tight' else args.wide))}` "
                     f"(commit {batch.get('commit')}): `{shlex.join(batch.get('argv', []))}`\n")
        fh.write("\n`*-duels.csv` carries the columns of the first set (2026-09-19); the batch directories under `run/matches` have the rest\n"
                 "(the fire and shape instruments, the per-second logs). Written by `run/matchup_table.py`, which also wrote\n"
                 "`crates/bot/data/matchups.csv` from the two `*-pairs.csv`.\n")
    with open(TABLE, "w", encoding="utf-8") as fh:
        fh.write("# unit,against,margin: mean surviving-value margin of `unit` over `against` at equal metal (+1 wiped them untouched, -1 the reverse).\n")
        fh.write(f"# Mean of the tight and wide formation tables in docs/data/duels-{args.date} (the duel harness, docs/harness/duels.md; run/matchup_table.py).\n")
        for pair in sorted(table):
            fh.write(f"{pair[0]},{pair[1]},{table[pair]:.3f}\n")
    print(f"wrote {os.path.relpath(out, REPO)} and {os.path.relpath(TABLE, REPO)}")


if __name__ == "__main__":
    main()
