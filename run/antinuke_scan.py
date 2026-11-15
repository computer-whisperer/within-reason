#!/usr/bin/env python3
"""When the opponent builds an anti-nuke or a nuke silo, from the truth files
(K-barb-anti-nuke-by-minute, K-opp-ai-silo-follows-its-economy).

    run/antinuke_scan.py [--games] [ROOT...]     (default: run/matches and the archive)

Walks every match directory under the roots that holds a `truth-0.jsonl` and whose batch has a `batch.json` (the
opponent, the bonus to him, the format, our AI), and takes from the truth file the first frame his anti-nuke and
his silo appear in his unit list (a frame counts: begun, not standing), the frame the first silo stands, how many
silos he had, his extractors at 20:00 with an advanced one counted as four, and the game's last frame. Prints, for
BARb and Apex by format and by whether he had an income bonus: how many games passed 20, 25 and 30 minutes and how
many of those held an anti-nuke or a silo of his, when the silos were begun, and the share of silos by his economy
at 20:00. `--games` lists every game with a silo. About two minutes over the archive.
"""
import collections, glob, json, os, re, statistics as st, sys

ARGS = [a for a in sys.argv[1:] if not a.startswith("--")]
ROOTS = ARGS or ["run/matches", "/ceph/christian/within-reason/run/matches-archive"]
SILO = {"armsilo", "corsilo", "legsilo"}
ANTI = {"armamd", "corfmd", "legabm"}
BASIC = {"armmex", "cormex", "legmex"}
ADVANCED = {"armmoho", "cormoho", "legmoho"}
AT_20 = 20 * 60 * 30


def clock(frame):
    return "-" if frame is None else f"{frame // 1800}:{frame % 1800 // 30:02d}"


def family(opponent):
    o = opponent.lower()
    return "Apex" if "apex" in o else "Barb3" if "barb3" in o else "BARb" if "barb" in o else "other"


def read(truth):
    """One game's row from its truth file; lines without a silo or an anti-nuke are only parsed at 20:00."""
    row = dict(last=0, silo=None, stands=None, silos=set(), anti=None, economy=None)
    with open(truth, errors="replace") as lines:
        for line in lines:
            m = re.match(r'\{"f": *(\d+)', line)
            if not m:
                continue
            frame = row["last"] = int(m.group(1))
            if frame != AT_20 and "silo" not in line and 'amd"' not in line and 'fmd"' not in line and 'abm"' not in line:
                continue
            try:
                units = json.loads(line).get("enemy", [])
            except ValueError:
                continue
            if frame == AT_20:
                row["economy"] = sum(1 for u in units if u[1] in BASIC) + 4 * sum(1 for u in units if u[1] in ADVANCED)
            for u in units:
                if u[1] in SILO:
                    row["silos"].add(u[0])
                    row["silo"] = row["silo"] if row["silo"] is not None else frame
                    if row["stands"] is None and len(u) > 4 and u[4] >= 100:
                        row["stands"] = frame
                elif u[1] in ANTI and row["anti"] is None:
                    row["anti"] = frame
    row["silos"] = len(row["silos"])
    return row


rows, unlabelled = [], 0
for root in ROOTS:
    for truth in sorted(glob.glob(os.path.join(root, "*", "[0-9][0-9]", "truth-0.jsonl"))):
        batch_dir = os.path.dirname(os.path.dirname(truth))
        try:
            batch = json.load(open(os.path.join(batch_dir, "batch.json")))
        except (OSError, ValueError):
            unlabelled += 1
            continue
        row = read(truth)
        row.update(label=os.path.basename(batch_dir) + "/" + os.path.basename(os.path.dirname(truth)), opponent=str(batch.get("opponent") or ""),
                   bonus=batch.get("enemy_bonus") or 0, duel=(batch.get("ours") or 1) == 1 and (batch.get("enemies") or 1) == 1,
                   flagship=bool(batch.get("pianist")))
        rows.append(row)

print(f"{len(rows)} games with a truth file and a batch.json ({unlabelled} more without a batch.json left out)\n")


def table(games, title):
    if not games:
        return
    print(f"{title}: {len(games)} games")
    for minutes in (20, 25, 30):
        long = [g for g in games if g["last"] >= minutes * 1800]
        print(f"  past {minutes}:00: {len(long)}; an anti-nuke of his in {sum(g['anti'] is not None for g in long)}, a silo in {sum(g['silo'] is not None for g in long)}")
    begun = sorted(g["silo"] for g in games if g["silo"] is not None)
    if begun:
        print(f"  silos begun {clock(begun[0])} to {clock(begun[-1])}, median {clock(int(st.median(begun)))}")
    long = [g for g in games if g["last"] >= 28 * 1800 and g["economy"] is not None]
    for low, high in ((0, 20), (20, 30), (30, 45), (45, 10_000)):
        band = [g for g in long if low <= g["economy"] < high]
        if band:
            print(f"  past 28:00 with {low} to {high if high < 10_000 else 'more'} extractors at 20:00 (an advanced one counted as four): {len(band)}, a silo in {sum(g['silo'] is not None for g in band)}")


for name in ("BARb", "Apex", "Barb3"):
    for duel in (True, False):
        for bonus in (False, True):
            games = [g for g in rows if family(g["opponent"]) == name and g["duel"] == duel and (g["bonus"] > 0) == bonus]
            table(games, f"{name}, {'1v1' if duel else 'teams'}, {'an income bonus to him' if bonus else '+0'}")
            if games and bonus:
                print("  by bonus, past 25:00: " + ", ".join(
                    f"+{b} {sum(g['silo'] is not None for g in at)} of {len(at)}"
                    for b in sorted({g['bonus'] for g in games}) for at in [[g for g in games if g['bonus'] == b and g['last'] >= 25 * 1800]] if at))
            table([g for g in games if g["flagship"]], f"  of which the flagship's")

if "--games" in sys.argv:
    print()
    for g in sorted((g for g in rows if g["silo"] is not None), key=lambda g: (family(g["opponent"]), g["bonus"], g["silo"])):
        print(f"{g['opponent']} +{g['bonus']} {'1v1' if g['duel'] else 'teams'} {g['label']}: silo begun {clock(g['silo'])}, standing {clock(g['stands'])}, "
              f"{g['silos']} in all, his anti-nuke {clock(g['anti'])}, {g['economy']} extractors at 20:00, the game ends {clock(g['last'])}")
