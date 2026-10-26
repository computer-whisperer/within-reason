#!/usr/bin/env python3
"""The distilled bot's tier 2 in team games (run/distilled_team_bench.sh).

    run/distilled_t2.py LABEL [LABEL...]

For the newest batch of each label, pooled over the games and our seats:
- the outcome and the minute it came;
- D-T2-GATE: when it opened and why (`t2 gate:` lines), when the advanced plant stood, when his tier 2 was first seen
  (`his tier 2 seen:`), and his first tier-2 thing on the map (the truth files, the combat tables' techlevel);
- at 15, 20, 25 and 30 minutes, medians over the seats still playing: our army's metal of tier 1 and of tier 2 (the
  `army:` line), the metal banked, the income, the advanced extractors standing.
"""
import glob, json, os, re, statistics, sys

MARKS = (15, 20, 25, 30)
LINE = re.compile(r"\[distilled ai=(\d+)\] f=(\d+) (.*)")
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
UNITS = json.load(open(f"{ROOT}/crates/combatsim/data/units.json"))["units"]


def tech(name):
    return UNITS.get(name, {}).get("tech", 1)


def seats(path):
    """Per seat of ours: {gate: (t, why), plant: t, seen: t, marks: {m: {...}}}."""
    out = {}
    for line in open(f"{path}/engine.log", errors="replace"):
        m = LINE.search(line)
        if not m:
            continue
        ai, t, text = int(m.group(1)), int(m.group(2)) / 1800.0, m.group(3)
        s = out.setdefault(ai, {"gate": None, "plant": None, "seen": None, "marks": {}})
        minute = round(t)
        if text.startswith("t2 gate: ") and s["gate"] is None:
            s["gate"] = (t, text.split(": ", 1)[1].split(" ")[0])
        elif text.startswith("t2: advanced plant standing") and s["plant"] is None:
            s["plant"] = t
        elif text.startswith("his tier 2 seen") and s["seen"] is None:
            s["seen"] = t
        elif text.startswith("army: ") and minute in MARKS and abs(t - minute) < 0.01:
            a = re.match(r"army: tier 1 (\d+), tier 2 (\d+)", text)
            s["marks"].setdefault(minute, {}).update(t1=int(a.group(1)), t2=int(a.group(2)))
        elif text.startswith("extractors=") and minute in MARKS and abs(t - minute) < 0.01:
            v = dict(re.findall(r"(\w+)=([\d.]+)", text))
            mm = re.search(r"metal (\d+)/(\d+) \+([\d.]+)", text)
            d = s["marks"].setdefault(minute, {})
            d["mohos"] = int(v.get("mohos", 0))
            if mm:
                d["bank"], d["income"] = int(mm.group(1)), float(mm.group(3))
    return out


def his_tier2(path):
    """The minute his first tier-2 thing stood on the map (any truth file)."""
    first = None
    for f in glob.glob(f"{path}/truth-*.jsonl"):
        for line in open(f):
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            if any(not framed and tech(n) >= 2 for _, n, _, _, _, framed in row["enemy"]):
                t = row["f"] / 1800.0
                first = t if first is None else min(first, t)
                break
    return first


def med(values, fmt="{:.0f}"):
    values = [v for v in values if v is not None]
    return fmt.format(statistics.median(values)) if values else "-"


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
        all_seats, his = [], []
        for r in results:
            g = f"{batch}/{r['index']:02d}"
            all_seats += seats(g).values()
            his.append(his_tier2(g))
        gates = [s["gate"] for s in all_seats]
        whys = {}
        for gate in gates:
            if gate:
                whys[gate[1]] = whys.get(gate[1], 0) + 1
        ended = [r["game_minutes"] for r in results if r["outcome"] != "Timeout"]
        print(f"=== {os.path.basename(batch)}: {o.count('Win')} won, {o.count('Loss')} lost, {o.count('Timeout')} to the cap of {len(o)} (ended at {med(ended, '{:.1f}')} min median)")
        print(f"  gate opened in {sum(1 for g in gates if g)} of {len(gates)} seats at {med([g[0] for g in gates if g], '{:.1f}')} min median, by {whys or '-'}; "
              f"plant standing {med([s['plant'] for s in all_seats], '{:.1f}')}; his tier 2 seen {med([s['seen'] for s in all_seats], '{:.1f}')}; his tier 2 on the map {med(his, '{:.1f}')}")
        for m in MARKS:
            rows = [s["marks"].get(m, {}) for s in all_seats if s["marks"].get(m)]
            if not rows:
                continue
            share = [r["t2"] / (r["t1"] + r["t2"]) for r in rows if r.get("t1", 0) + r.get("t2", 0) > 0]
            print(f"  {m}:00 ({len(rows)} seats) army tier 1 {med([r.get('t1') for r in rows])} tier 2 {med([r.get('t2') for r in rows])} (share {med(share, '{:.2f}')}), "
                  f"bank {med([r.get('bank') for r in rows])}, income {med([r.get('income') for r in rows], '{:.1f}')}, mohos {med([r.get('mohos') for r in rows])}")


if __name__ == "__main__":
    main()
