#!/usr/bin/env python3
"""The distilled bot's silos on the team bench (run/distilled_team_bench.sh, docs/design/2026-10-27-distilled-nukes.md).

    run/distilled_nuke.py LABEL [LABEL...]

For the newest batch of each label, pooled over the games and our seats:
- the outcome and the minute it came;
- the silos: seats that began one, the minute the first was begun and stood, the silos standing at the marks;
- the strikes: ordered, launched, and what the truth file says each did (his buildings inside the blast before the
  impact against after: killed or damaged = landed; buildings inside and none harmed = stopped, which is an
  interception or a miss; none inside = empty), with his anti-nuke within 2,000 of the aim at the launch by the truth,
  against the bot's own verdict from the ground (the `nuke: strike` lines) and the ground's fall it read;
- at the marks, medians over the seats still playing: our bank, income and energy store, and the worth of his
  buildings standing by the truth (metal plus energy over 70).
"""
import glob, json, os, re, statistics, sys

MARKS = (35, 40, 45, 50)
LINE = re.compile(r"\[distilled ai=(\d+)\] f=(\d+) (.*)")
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
UNITS = json.load(open(f"{ROOT}/crates/combatsim/data/units.json"))["units"]
ANTI_NUKES = {"armamd", "corfmd", "legabm", "armscab", "cormabm"}
COVERAGE = 2000.0
ENERGY_A_METAL = 70.0
# The truth file has a row every 2 s; the warhead lands about 12 s after the launch.
BEFORE, AFTER = 10 * 30, 24 * 30
LANDED_SHARE = 0.25


def worth(name):
    u = UNITS.get(name, {})
    return u.get("metal", 0) + u.get("energy", 0) / ENERGY_A_METAL


def building(name):
    return name in UNITS and UNITS[name].get("speed", 1) == 0


def med(values, fmt="{:.0f}"):
    values = [v for v in values if v is not None]
    return fmt.format(statistics.median(values)) if values else "-"


def read_log(path):
    """Per seat: silo begun/standing minutes, silos standing by mark, launches [(frame, x, z, r)], strike lines seen, marks."""
    seats = {}
    pending = {}
    for line in open(f"{path}/engine.log", errors="replace"):
        m = LINE.search(line)
        if not m:
            continue
        ai, f, text = int(m.group(1)), int(m.group(2)), m.group(3)
        s = seats.setdefault(ai, {"begun": None, "standing": None, "silos": {}, "ordered": 0, "launches": [], "verdicts": {}, "marks": {}})
        minute = round(f / 1800)
        at_mark = minute in MARKS and abs(f / 1800 - minute) < 0.01
        if text.startswith("t2: silo begun") and s["begun"] is None:
            s["begun"] = f / 1800
        elif text.startswith("t2: silo standing") and s["standing"] is None:
            s["standing"] = f / 1800
        elif text.startswith("nuke: silo ") and " fires at " in text:
            g = re.search(r"silo (\d+) fires at \((-?\d+),(-?\d+)\) blast (\d+)", text)
            s["ordered"] += 1
            pending[(ai, int(g.group(1)))] = (int(g.group(2)), int(g.group(3)), float(g.group(4)))
        elif text.startswith("nuke: silo ") and " launched at " in text:
            silo = int(re.search(r"silo (\d+)", text).group(1))
            if (ai, silo) in pending:
                x, z, r = pending.pop((ai, silo))
                s["launches"].append((f, x, z, r))
        elif text.startswith("nuke: strike at "):
            g = re.search(r"strike at \((-?\d+),(-?\d+)\) by silo \d+: (\w+)(?: \(the ground down (-?[\d.]+) on the mean, (-?[\d.]+) at most\))?", text)
            s["verdicts"][(int(g.group(1)), int(g.group(2)))] = (g.group(3), float(g.group(4)) if g.group(4) else None)
        elif text.startswith("nuke: silos ") and at_mark:
            s["silos"][minute] = int(re.match(r"nuke: silos (\d+)", text).group(1))
        elif text.startswith("extractors=") and at_mark:
            mm = re.search(r"metal (\d+)/(\d+) \+([\d.]+).*?energy (\d+)/(\d+)", text)
            if mm:
                s["marks"][minute] = {"bank": int(mm.group(1)), "income": float(mm.group(3)), "estore": int(mm.group(4)) / max(1, int(mm.group(5)))}
    return seats


def read_truth(path):
    """The truth rows by frame: [(frame, [(id, name, x, z, health, framed)])]."""
    files = sorted(glob.glob(f"{path}/truth-*.jsonl"))
    if not files:
        return []
    rows = []
    for line in open(files[0]):
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue
        rows.append((row["f"], row["enemy"]))
    return rows


def row_at(rows, frame, after):
    """The last row at or before `frame`, or the first at or after it."""
    if after:
        return next((r for f, r in rows if f >= frame), None)
    best = None
    for f, r in rows:
        if f > frame:
            break
        best = r
    return best


def judge(rows, launch):
    """What the truth says a launch did: (verdict, killed, worth killed, covered)."""
    f, x, z, r = launch
    # Buildings only: his mobile units walk in and out of the circle in the seconds around the impact.
    inside = lambda units: {u[0]: u for u in units if building(u[1]) and (u[2] - x) ** 2 + (u[3] - z) ** 2 <= r * r}
    before, after = row_at(rows, f + BEFORE, False), row_at(rows, f + AFTER, True)
    if before is None or after is None:
        return "unread", 0, 0.0, False
    covered = any(u[1] in ANTI_NUKES and not u[5] and (u[2] - x) ** 2 + (u[3] - z) ** 2 <= COVERAGE ** 2 for u in before)
    b, a = inside(before), inside(after)
    if not b:
        return "empty", 0, 0.0, covered
    # A building gone with a new one standing on its place was replaced (an extractor upgraded), not killed.
    # (The new one may stand already as a frame before it.)
    replaced = lambda u: any(i != u[0] and (v[2] - u[2]) ** 2 + (v[3] - u[3]) ** 2 < 50 ** 2 for i, v in a.items())
    killed = [u for i, u in b.items() if i not in a and not replaced(u)]
    damaged = [u for i, u in b.items() if i in a and a[i][4] < u[4] - 5]
    # A warhead kills or hurts most of what stands in its circle; a few hurt buildings among many are a fight nearby
    # (dist57: an intercepted strike over 96 buildings, three worn solars dying in the window).
    if len(killed) + len(damaged) >= max(1, LANDED_SHARE * len(b)):
        return "landed", len(killed), sum(worth(u[1]) for u in killed), covered
    return "stopped", 0, 0.0, covered


def main():
    labels = sys.argv[1:]
    if not labels:
        print(__doc__.split("\n\n")[1].strip())
        sys.exit(2)
    for label in labels:
        found = sorted(glob.glob(f"run/matches/*-{label}"), key=lambda d: os.path.basename(d).split("-")[0])
        if not found:
            print(f"=== {label}: no batch")
            continue
        batch = found[-1]
        results = [json.loads(l) for l in open(f"{batch}/results.jsonl")]
        o = [r["outcome"] for r in results]
        ended = [r["game_minutes"] for r in results if r["outcome"] != "Timeout"]
        print(f"=== {os.path.basename(batch)}: {o.count('Win')} won, {o.count('Loss')} lost, {o.count('Timeout')} to the cap of {len(o)} (ended at {med(ended, '{:.1f}')} min median)")
        all_seats, verdicts, his = [], {}, {m: [] for m in MARKS}
        killed_worth, covered_by_verdict, against = [], {}, {}
        for r in results:
            g = f"{batch}/{r['index']:02d}"
            seats = read_log(g)
            rows = read_truth(g)
            all_seats += seats.values()
            for s in seats.values():
                for launch in s["launches"]:
                    v, _, w, covered = judge(rows, launch)
                    verdicts[v] = verdicts.get(v, 0) + 1
                    covered_by_verdict.setdefault(v, [0, 0])[0 if covered else 1] += 1
                    if v == "landed":
                        killed_worth.append(w)
                    bot, drop = s["verdicts"].get((launch[1], launch[2]), ("none", None))
                    against.setdefault((v, bot), []).append(drop)
            for m in MARKS:
                row = row_at(rows, m * 1800, False)
                if row is not None and rows and rows[-1][0] >= m * 1800:
                    his[m].append(sum(worth(u[1]) for u in row if building(u[1]) and not u[5]))
        began = [s for s in all_seats if s["begun"] is not None]
        launches = sum(len(s["launches"]) for s in all_seats)
        print(f"  silos: begun in {len(began)} of {len(all_seats)} seats, the first begun at {med([s['begun'] for s in began], '{:.1f}')} and standing at {med([s['standing'] for s in all_seats], '{:.1f}')} min median; "
              f"standing at {', '.join(f'{m}:00 {med([s['silos'].get(m) for s in all_seats if m in s['silos']])}' for m in MARKS)}")
        print(f"  strikes: {sum(s['ordered'] for s in all_seats)} ordered, {launches} launched; by the truth {verdicts or '-'} "
              f"(his anti-nuke within {COVERAGE:.0f} of the aim [covered, not]: {covered_by_verdict or '-'}); worth killed a landed strike {med(killed_worth)}, all {sum(killed_worth):.0f}")
        for (truth, bot), drops in sorted(against.items()):
            known = sorted(d for d in drops if d is not None)
            span = f", the ground down {known[0]:.1f} to {known[-1]:.1f} on the mean" if known else ""
            print(f"  the truth says {truth}, the bot said {bot}: {len(drops)}{span}")
        for m in MARKS:
            rows_m = [s["marks"][m] for s in all_seats if m in s["marks"]]
            if rows_m or his[m]:
                print(f"  {m}:00 ({len(rows_m)} seats) bank {med([x['bank'] for x in rows_m])}, income {med([x['income'] for x in rows_m], '{:.1f}')}, energy store {med([x['estore'] for x in rows_m], '{:.2f}')}; "
                      f"his buildings worth {med(his[m])} ({len(his[m])} games)")


if __name__ == "__main__":
    main()
