#!/usr/bin/env python3
"""The kill-tests of the derived articles on metal (docs/briefs/derived/metal.md, D-metal-is-the-game, and
docs/briefs/derived/lead.md, D-lead-convert-it).

  run/metal_killtest.py extract <cache.jsonl> <matches root> [<root> ...]
      Our games. Every arena match under the roots with a Win/Loss row in its batch's results.jsonl, a record and a
      truth file: both sides at every minute from 1:00 to 24:00. One JSON line per match, appended to the cache.
  run/metal_killtest.py score <cache.jsonl> [<cache.jsonl> ...]
      The metal test: does the income gap at 8:00, 10:00, 12:00 and 16:00 call the result better than the army gap?
      The table by population (1v1 played by the model, 1v1 played by the old heuristic code, team games) and the
      games at 12:00 where the income gap and the result disagree.
  run/metal_killtest.py pool [--min-os N]
      The metal test on the pool's duels, from the replay cards under run/matches/*-replay-*/card.json (both sides'
      income exact, from the engine's team statistics).
  run/metal_killtest.py windows <cache.jsonl> [<cache.jsonl> ...] [--ratio 1.5] [--margin 500] [--fall 3]
      The lead test on our games: the first window between 6:00 and 14:00 in which our army metal is at least --ratio
      times his and at least --margin metal above it; whether his extractor count fell by --fall or more from the
      window's first minute to its lowest point by two minutes after its end ("converted"); whether at least 30% of
      our army metal stood in his half during it ("went"); the window's size (its largest ratio), surplus and length;
      and which of these calls the result.
  run/metal_killtest.py pool-windows [--min-os N] [--ratio 1.5] [--margin 500] [--fall 3]
      The lead test on the pool's duels from the cards (the cards' army_metal_in_his_half is the "went").

What is measured for our games. Our side is the first record's `own` and `al` lists at the sample nearest the minute;
his is the truth file's list (every enemy unit, every two seconds). Income is the extractor income: each finished
extractor's nearest spot's worth (the header's `metal_spots`) times its `extracts_metal`, times the side's bonus
(`--enemy-bonus` / `--our-bonus` from batch.json; the game's handicap multiplies income by that per cent). Reclaim and
conversion are left out of both sides alike; against our seat's exact income the proxy runs at 0.96 (quartiles 0.93 to
0.97 over 63 duels). Army is the metal cost of the finished units of class army, aircraft, hover and ship; "in his half"
is the part of it past the midpoint of the line between the two sides' commanders at 0:00. Records before 2026-09-20
list spots without a worth and units without `extracts_metal`: there every spot counts one extractor's worth and the
advanced kinds four, so the income is a weighted count and only the gap's sign and ratio mean anything. The archive on
the ceph mount has files that hang in an uninterruptible read; every file is read through `timeout`, and a match whose
file does not answer is skipped and counted.

How the metal test is scored. For each minute and population: the share of games the sign of each gap calls right (a
tie counts half), the AUC of each log-ratio (the chance a won game's ratio exceeds a lost game's), how often the two
gaps agree and are right, and when they disagree, which was right. Then every game at 12:00 where income said one
thing and the result the other, for the reader to check against the ledger.
"""
import glob
import json
import math
import os
import re
import statistics as st
import subprocess
import sys
from collections import Counter
from multiprocessing import Pool

MINUTES = list(range(1, 25))
SCORE_MINUTES = [8, 10, 12, 16]
FPS = 30
ARMY = {"army", "aircraft", "hover", "ship"}
SAMPLE_RE = re.compile(r'^\{"t": ?"s", ?"f": ?(\d+)')
TRUTH_RE = re.compile(r'^\{"f": ?(\d+)')


def read_text(path, max_bytes=120_000_000, timeout=60):
    """The file's first max_bytes, or None when it does not answer within timeout seconds (a stuck ceph read)."""
    try:
        r = subprocess.run(["timeout", str(timeout), "head", "-c", str(max_bytes), path], capture_output=True, timeout=timeout + 5)
    except subprocess.TimeoutExpired:
        return None
    if r.returncode == 124:
        return None
    return r.stdout.decode("utf-8", errors="replace")


def nearest_spot(spots, x, z):
    best = None
    for sx, sz, a in spots:
        d = (sx - x) ** 2 + (sz - z) ** 2
        if best is None or d < best[0]:
            best = (d, a)
    return best[1] if best else 0.0


def side_numbers(units, defs_by, spots, bonus, forward):
    """units: [(name, x, z, being_built)]; forward(x, z) says whether a point is in the other side's half."""
    income = 0.0
    extractors = advanced = 0
    army = army_forward = 0.0
    for name, x, z, building in units:
        if building:
            continue
        d = defs_by.get(name)
        if not d:
            continue
        cls = d.get("class", "other")
        if cls == "extractor":
            extractors += 1
            # records before 2026-09-20 carry no extracts_metal: the tier-1 rate, four times it for the advanced kinds
            em = d.get("extracts_metal") or (0.004 if any(k in name for k in ("moho", "mexp", "uwmme", "shockwave")) else 0.001)
            if em > 0.002:
                advanced += 1
            income += nearest_spot(spots, x, z) * em
        elif cls in ARMY:
            army += d.get("metal", 0)
            if forward and forward(x, z):
                army_forward += d.get("metal", 0)
    return {"ext_income": round(income * (1 + bonus / 100.0), 2), "extractors": extractors, "advanced": advanced,
            "army": round(army), "army_forward": round(army_forward)}


def centre(points):
    return (sum(p[0] for p in points) / len(points), sum(p[1] for p in points) / len(points)) if points else None


def extract_one(job):
    d, row, batch = job
    try:
        records = sorted(glob.glob(d + "/record-*.jsonl"))
        truths = sorted(glob.glob(d + "/truth-*.jsonl"))
        if not records or not truths:
            return {"dir": d, "skip": "no record or truth"}
        text = read_text(records[0])
        if text is None:
            return {"dir": d, "skip": "record unreadable (timeout)"}
        lines = iter(text.splitlines(True))
        header = json.loads(next(lines))
        spots = header.get("metal_spots") or []
        if not spots:
            return {"dir": d, "skip": "no metal_spots in header"}
        if len(spots[0]) == 2:  # [x, z] with no worth: every spot one extractor's worth, an advanced extractor four
            spots = [[x, z, 1000.0] for x, z in spots]
            header["metal_spots_amount"] = "none"
        defs = header["unit_defs"]
        defs_by = {u["name"]: u for u in defs}
        targets = {m: m * 60 * FPS for m in MINUTES}
        last = max(targets.values()) + 2 * FPS
        samples = {}
        first_sample = None
        for line in lines:
            m = SAMPLE_RE.match(line)
            if not m:
                continue
            f = int(m.group(1))
            if f > last:
                break
            if first_sample is None:
                try:
                    first_sample = json.loads(line)
                except ValueError:
                    pass
            for mn, tf in targets.items():
                if abs(f - tf) <= 15 and mn not in samples:
                    try:
                        samples[mn] = json.loads(line)
                    except ValueError:
                        pass
        ttext = read_text(truths[0])
        if ttext is None:
            return {"dir": d, "skip": "truth unreadable (timeout)"}
        truth = {}
        first_truth = None
        for line in ttext.splitlines(True):
            m = TRUTH_RE.match(line)
            if not m:
                continue
            f = int(m.group(1))
            if f > last:
                break
            if first_truth is None:
                try:
                    first_truth = json.loads(line)
                except ValueError:
                    pass
            for mn, tf in targets.items():
                if abs(f - tf) <= 30 and mn not in truth:
                    try:
                        truth[mn] = json.loads(line)
                    except ValueError:
                        pass
        # the halves: the midpoint of the line between the sides' commanders at the start, from the first sample and truth line
        our_start = his_start = None
        if first_sample:
            pts = [(u[2], u[3]) for u in first_sample.get("own", []) + first_sample.get("al", []) if 0 <= u[1] < len(defs) and defs[u[1]].get("class") == "commander"]
            our_start = centre(pts) or centre([(u[2], u[3]) for u in first_sample.get("own", [])])
        if first_truth:
            pts = [(u[2], u[3]) for u in first_truth.get("enemy", []) if defs_by.get(u[1], {}).get("class") == "commander"]
            his_start = centre(pts) or centre([(u[2], u[3]) for u in first_truth.get("enemy", [])])
        forward_ours = forward_his = None
        if our_start and his_start and our_start != his_start:
            mx, mz = (our_start[0] + his_start[0]) / 2, (our_start[1] + his_start[1]) / 2
            ax, az = his_start[0] - our_start[0], his_start[1] - our_start[1]
            forward_ours = lambda x, z: (x - mx) * ax + (z - mz) * az > 0
            forward_his = lambda x, z: (x - mx) * ax + (z - mz) * az < 0
        out = {"dir": d, "batch": os.path.basename(os.path.dirname(d)), "nn": os.path.basename(d),
               "label": batch.get("label"), "opponent": batch.get("opponent"), "map": (header.get("map") or {}).get("name"),
               "ours": batch.get("ours"), "allies": batch.get("allies"), "enemies": batch.get("enemies"),
               "enemy_bonus": batch.get("enemy_bonus") or 0, "our_bonus": batch.get("our_bonus") or 0,
               "outcome": row["outcome"], "minutes": row["game_minutes"], "spots_amount": header.get("metal_spots_amount"),
               "mode": header.get("mode"), "seats": len(header.get("seats") or []),
               "starts": {"ours": our_start, "his": his_start}, "at": {}}
        for mn in MINUTES:
            s, t = samples.get(mn), truth.get(mn)
            if not s or not t:
                continue
            ours = [(defs[u[1]]["name"], u[2], u[3], (u[5] & 1) if isinstance(u[5], int) else 0) for u in s.get("own", []) if 0 <= u[1] < len(defs)]
            ours += [(defs[u[1]]["name"], u[2], u[3], u[5] if len(u) > 5 else 0) for u in s.get("al", []) if 0 <= u[1] < len(defs)]
            theirs = [(u[1], u[2], u[3], u[5] if len(u) > 5 else 0) for u in t.get("enemy", [])]
            o = side_numbers(ours, defs_by, spots, out["our_bonus"], forward_ours)
            e = side_numbers(theirs, defs_by, spots, out["enemy_bonus"], forward_his)
            o["exact_income_seat"] = round(s["m"][1], 1)
            out["at"][str(mn)] = {"ours": o, "theirs": e}
        return out
    except Exception as ex:  # one bad match must not end the batch
        return {"dir": d, "skip": f"error {type(ex).__name__}: {ex}"}


def extract(cache, roots):
    jobs = []
    for root in roots:
        for res in sorted(glob.glob(root + "/*/results.jsonl")):
            b = os.path.dirname(res)
            batch = {}
            if os.path.exists(b + "/batch.json"):
                bt = read_text(b + "/batch.json", timeout=10)
                try:
                    batch = json.loads(bt) if bt else {}
                except ValueError:
                    pass
            rt = read_text(res, timeout=10)
            if rt is None:
                print("unreadable", res, file=sys.stderr)
                continue
            for line in rt.splitlines():
                try:
                    r = json.loads(line)
                except ValueError:
                    continue
                if r.get("outcome") in ("Win", "Loss"):
                    jobs.append((f"{b}/{r['index']:02d}", r, batch))
    print(len(jobs), "Win/Loss rows", file=sys.stderr)
    skips = Counter()
    with Pool(8) as p, open(cache, "a") as fo:
        for i, r in enumerate(p.imap_unordered(extract_one, jobs, chunksize=4)):
            fo.write(json.dumps(r) + "\n")
            skips[r.get("skip", "ok").split(":")[0]] += 1
            if i % 100 == 0:
                print(i, file=sys.stderr)
    print(dict(skips), file=sys.stderr)


def lr(a, b):
    return math.log((a + 1.0) / (b + 1.0))


def auc(pairs):
    w = [s for s, y in pairs if y]
    l = [s for s, y in pairs if not y]
    if not w or not l:
        return float("nan")
    return sum(1.0 if a > b else (0.5 if a == b else 0.0) for a in w for b in l) / (len(w) * len(l))


def right(pairs):
    return sum(1.0 if (s > 0) == y and s != 0 else (0.5 if s == 0 else 0.0) for s, y in pairs)


def table(label, gaps_by_minute):
    """gaps_by_minute: {minute: [(income lr, army lr, extractors lr, won)]}."""
    print(f"{label}: " + ", ".join(f"{n} games at {mn}" for mn, g in gaps_by_minute.items() for n in [len(g)]))
    print("minute |    n (W-L)    | income sign | army sign | extractors sign | AUC income / army / extractors | agree & right | disagree: income right / army right")
    for mn, g in gaps_by_minute.items():
        if not g:
            continue
        inc = [(gi, w) for gi, ga, ge, w in g]
        army = [(ga, w) for gi, ga, ge, w in g]
        ext = [(ge, w) for gi, ga, ge, w in g]
        ag = ar = di = da = 0
        for gi, ga, ge, w in g:
            if gi == 0 or ga == 0:
                continue
            if (gi > 0) == (ga > 0):
                ag += 1
                ar += (gi > 0) == w
            else:
                di += (gi > 0) == w
                da += (ga > 0) == w
        n = len(g)
        won = sum(1 for *_, w in g if w)
        print(f"{mn:>6} | {n:>4} ({won}-{n - won}) | {right(inc) / n:6.0%}      | {right(army) / n:6.0%}    | {right(ext) / n:6.0%}          | "
              f"{auc(inc):.2f} / {auc(army):.2f} / {auc(ext):.2f}             | {ar} of {ag}     | {di} / {da} of {di + da}")
    print()


def family(r):
    o = (r.get("opponent") or "").lower()
    if "apex" in o:
        return "Apex"
    if "barb" in o or "bar" in o:
        return "BARb"
    return r.get("opponent") or "?"


def population(r):
    if (r.get("ours") or 1) == 1 and not r.get("allies") and (r.get("enemies") or 1) == 1:
        return "1v1 heuristic" if r.get("mode") == "heuristic" else "1v1 model"
    return "team"


def load(caches):
    rows = []
    for p in caches:
        for l in open(p):
            r = json.loads(l)
            if "at" in r:
                rows.append(r)
    return rows


def score(caches):
    rows = load(caches)
    print(f"{len(rows)} games of ours with a result and both sides' numbers: "
          + ", ".join(f"{k} {v}" for k, v in sorted(Counter(population(r) for r in rows).items()))
          + "; opponents " + ", ".join(f"{k} {v}" for k, v in sorted(Counter(family(r) for r in rows).items()))
          + "; spot worths " + ", ".join(f"{k} {v}" for k, v in sorted(Counter(str(r.get('spots_amount')) for r in rows).items())))
    print()
    for pop in ("1v1 model", "1v1 heuristic", "team", "all"):
        sel = [r for r in rows if pop == "all" or population(r) == pop]
        gaps = {}
        for mn in SCORE_MINUTES:
            g = []
            for r in sel:
                at = r["at"].get(str(mn))
                if not at:
                    continue
                o, t = at["ours"], at["theirs"]
                g.append((lr(o["ext_income"], t["ext_income"]), lr(o["army"], t["army"]), lr(o["extractors"], t["extractors"]), r["outcome"] == "Win"))
            gaps[mn] = g
        table(pop, gaps)
    chk = [(r["at"]["12"]["ours"]["exact_income_seat"], r["at"]["12"]["ours"]["ext_income"]) for r in rows
           if "12" in r["at"] and population(r) != "team" and r.get("spots_amount") == "worth"
           and r["at"]["12"]["ours"]["ext_income"] > 0 and r["at"]["12"]["ours"]["exact_income_seat"] > 0]
    if len(chk) > 3:
        ratios = sorted(e / x for x, e in chk)
        print(f"proxy check (1v1, worth-scaled spots, n={len(chk)}): our extractor-income proxy over our seat's exact income at 12:00, quartiles "
              + " / ".join(f"{q:.2f}" for q in st.quantiles(ratios, n=4))
              + f"; exact median {st.median(x for x, e in chk):.0f}, proxy median {st.median(e for x, e in chk):.0f}")
        print()
    for pop in ("1v1 model", "1v1 heuristic"):
        for outcome in ("Loss", "Win"):
            g = [r for r in rows if population(r) == pop and r["outcome"] == outcome and "12" in r["at"]]
            if not g:
                continue
            ahead = [r for r in g if r["at"]["12"]["ours"]["ext_income"] > r["at"]["12"]["theirs"]["ext_income"]]
            behind = [r for r in g if r["at"]["12"]["ours"]["ext_income"] < r["at"]["12"]["theirs"]["ext_income"]]
            both = [r for r in ahead if r["at"]["12"]["ours"]["army"] > r["at"]["12"]["theirs"]["army"]]
            q = lambda xs: " / ".join(f"{x:.0f}" for x in st.quantiles(sorted(xs), n=4)) if len(xs) > 3 else " ".join(f"{x:.0f}" for x in xs)
            print(f"{pop} {'losses' if outcome == 'Loss' else 'wins'} {len(g)}: income ahead at 12:00 in {len(ahead)} (army ahead too in {len(both)}), behind in {len(behind)}; "
                  f"game minutes when ahead {q([r['minutes'] for r in ahead]) or '-'}, when behind {q([r['minutes'] for r in behind]) or '-'}")
    print()
    print("income's inversions at 12:00 (the income gap and the result disagree; a tie listed as such):")
    for r in sorted(rows, key=lambda r: r["batch"]):
        if "12" not in r["at"]:
            continue
        o, t = r["at"]["12"]["ours"], r["at"]["12"]["theirs"]
        won = r["outcome"] == "Win"
        if o["ext_income"] == t["ext_income"] or (o["ext_income"] > t["ext_income"]) != won:
            tag = "TIE " if o["ext_income"] == t["ext_income"] else ("army agrees" if (o["army"] > t["army"]) == won else "army also wrong")
            print(f"  {r['batch'][11:]}/{r['nn']} | {family(r)} +{r['enemy_bonus']} | {population(r)} | {r['outcome']} @{r['minutes']:.0f} | "
                  f"income {o['ext_income']:.0f}/{t['ext_income']:.0f} | ext {o['extractors']}/{t['extractors']} | army {o['army']:,}/{t['army']:,} | {tag}")


def pool_duels(min_os):
    def secs(c):
        m, s = c.split(":")
        return int(m) * 60 + int(s)
    games = []
    for p in sorted(glob.glob("run/matches/*-replay-*/card.json")):
        try:
            card = json.load(open(p))
        except Exception:
            continue
        teams = card.get("teams") or []
        if len(teams) != 2 or not any(t.get("won") for t in teams) or not all(t.get("curves") for t in teams):
            continue
        os_by_team = {pl.get("team"): pl.get("os") or 0 for pl in card.get("players") or []}
        if min(os_by_team.get(t["team"], 0) for t in teams) < min_os:
            continue
        m = card.get("map")
        games.append((card, teams, m.get("name") if isinstance(m, dict) else m, os_by_team, secs(card["duration"])))
    print(f"{len(games)} duels of the pool with a result (min OS {min_os:g}); maps: "
          + ", ".join(f"{k} {v}" for k, v in sorted(Counter(g[2] for g in games).items())))
    return games


def pool(min_os):
    games = pool_duels(min_os)
    gaps = {}
    inversions = []
    for mn in SCORE_MINUTES:
        g = []
        for card, teams, m, os_by, duration in games:
            if duration < mn * 60 + 30:
                continue
            rows = [next((r for r in t["curves"] if r["minute"] == mn), None) for t in teams]
            if None in rows:
                continue
            a, b = rows  # team 0's view; the sides are symmetric and every duel counts once
            won = bool(teams[0].get("won"))
            gi = lr(a["metal_income"], b["metal_income"])
            g.append((gi, lr(a["army_metal"], b["army_metal"]), lr(a["extractors"], b["extractors"]), won))
            if mn == 12 and (gi == 0 or (gi > 0) != won):
                w, l = (teams[0], teams[1]) if won else (teams[1], teams[0])
                wi, li = (a, b) if won else (b, a)
                inversions.append(f"  {card['id'][:8]} {m} {card['duration']} | winner {w.get('player')} OS {os_by.get(w['team'], 0):.0f}: income {wi['metal_income']:.0f}, "
                                  f"extractors {wi['extractors']}, army {wi['army_metal']:,} | loser {l.get('player')} OS {os_by.get(l['team'], 0):.0f}: income {li['metal_income']:.0f}, "
                                  f"extractors {li['extractors']}, army {li['army_metal']:,}")
        gaps[mn] = g
    table("the pool's duels", gaps)
    print("inversions at 12:00 (the winner behind on income, or level):")
    print("\n".join(inversions) or "  none")


# ---- the lead test -------------------------------------------------------------------------------------------------

WINDOW_FROM, WINDOW_TO = 6, 14


def first_window(series, ratio, margin):
    """series: {minute: (our army, his army, his extractors, our army in his half)} -> (start, end) of the first run of
    minutes in [WINDOW_FROM, WINDOW_TO] with ours >= ratio x his and ours - his >= margin, or None."""
    start = None
    for mn in range(WINDOW_FROM, WINDOW_TO + 1):
        s = series.get(mn)
        held = s is not None and s[0] >= ratio * s[1] and s[0] - s[1] >= margin
        if held and start is None:
            start = mn
        if not held and start is not None:
            return start, mn - 1
    return (start, WINDOW_TO) if start is not None else None


def window_facts(series, w, fall):
    """The window's size, surplus, length, whether his extractors fell by `fall` and whether 30% of our army went."""
    s, e = w
    mins = [m for m in range(s, e + 1) if m in series]
    size = max(series[m][0] / max(1, series[m][1]) for m in mins)
    surplus = max(series[m][0] - series[m][1] for m in mins)
    his_ext_start = series[s][2]
    after = [series[m][2] for m in range(s + 1, e + 3) if m in series]
    drop = his_ext_start - min(after) if after else 0
    later = [series[m][2] for m in series if m > s]
    drop_end = his_ext_start - min(later) if later else 0  # his lowest count from the window's start to the curves' end
    went = max((series[m][3] / series[m][0]) if series[m][0] else 0.0 for m in mins)
    return {"start": s, "end": e, "length": e - s + 1, "size": size, "surplus": surplus, "his_ext_start": his_ext_start,
            "drop": drop, "drop_end": drop_end, "converted": drop >= fall, "went": went >= 0.30, "went_share": went}


def report_windows(label, items, fall):
    """items: [(facts or None, won, minutes, name)]."""
    n = len(items)
    with_w = [(f, w, mn, name) for f, w, mn, name in items if f]
    without = [(f, w, mn, name) for f, w, mn, name in items if not f]
    print(f"{label}: {n} games; a window in {len(with_w)}, none in {len(without)} "
          f"(won {sum(1 for f, w, mn, name in with_w if w)} of {len(with_w)} with a window, {sum(1 for f, w, mn, name in without if w)} of {len(without)} without)")
    if not with_w:
        print()
        return
    q = lambda xs: (" / ".join(f"{x:.1f}" for x in st.quantiles(sorted(xs), n=4)) if len(xs) > 3 else " ".join(f"{x:.1f}" for x in xs)) if xs else "-"
    print(f"  first window opens at minute {q([f['start'] for f, *_ in with_w])}, lasts {q([f['length'] for f, *_ in with_w])} minutes, "
          f"size {q([f['size'] for f, *_ in with_w])}, surplus {q([f['surplus'] for f, *_ in with_w])}")
    cells = Counter()
    for f, w, mn, name in with_w:
        cells[(f["converted"], f["went"], w)] += 1
    def cell(c, g):
        won, lost = cells[(c, g, True)], cells[(c, g, False)]
        return f"{won}-{lost}" + (f" ({won / (won + lost):.0%})" if won + lost else "")
    print(f"  his extractors fell by {fall}+ during the window (converted): won-lost {cell(True, True)} when 30% of our army went, {cell(True, False)} when it did not;")
    print(f"  did not fall:                                          won-lost {cell(False, True)} when it went, {cell(False, False)} when it did not")
    went_won = sum(1 for f, w, mn, name in with_w if f["went"] and w)
    went_n = sum(1 for f, w, mn, name in with_w if f["went"])
    stayed_won = sum(1 for f, w, mn, name in with_w if not f["went"] and w)
    stayed_n = len(with_w) - went_n
    pct = lambda a, b: f"{a} of {b}" + (f" ({a / b:.0%})" if b else "")
    print(f"  the surplus went (30%+ of our army in his half during the window): won {pct(went_won, went_n)}; stayed home: won {pct(stayed_won, stayed_n)}")
    conv = [(1.0 if f["converted"] else 0.0, w) for f, w, mn, name in with_w]
    drop = [(float(f["drop"]), w) for f, w, mn, name in with_w]
    drop_end = [(float(f["drop_end"]), w) for f, w, mn, name in with_w]
    size = [(f["size"], w) for f, w, mn, name in with_w]
    surplus = [(float(f["surplus"]), w) for f, w, mn, name in with_w]
    length = [(float(f["length"]), w) for f, w, mn, name in with_w]
    went = [(f["went_share"], w) for f, w, mn, name in with_w]
    acc = lambda ps: sum(1 for s, w in ps if (s > 0) == w) / len(ps)
    print(f"  calls the result: converted {acc(conv):.0%}; AUC of his extractors' fall within the window {auc(drop):.2f}, of their fall by the curves' end {auc(drop_end):.2f}, "
          f"of the window's size {auc(size):.2f}, of its surplus {auc(surplus):.2f}, of its length {auc(length):.2f}, of the share that went {auc(went):.2f}")
    wins = [mn - f["end"] for f, w, mn, name in with_w if w]
    print(f"  the won games end {q(wins)} minutes after the first window closes")
    losses = [(f, mn, name) for f, w, mn, name in with_w if not w]
    if losses:
        ns = sum(1 for f, mn, name in losses if not f["went"])
        nl = sum(1 for f, mn, name in losses if f["went"] and not f["converted"])
        nc = sum(1 for f, mn, name in losses if f["converted"])
        print(f"  the {len(losses)} losses with a window: the surplus never went (under 30% of it in his half) {ns}; went and his extractors did not fall {nl}; "
              f"fell and lost anyway {nc}; the losses end {q([mn - f['end'] for f, mn, name in losses])} minutes after the window closes")
    print()


def windows(caches, ratio, margin, fall, list_losses=True):
    rows = load(caches)
    for pop in ("1v1 model", "1v1 heuristic", "team"):
        items = []
        for r in rows:
            if population(r) != pop:
                continue
            series = {}
            for mn, at in r["at"].items():
                o, t = at["ours"], at["theirs"]
                series[int(mn)] = (o["army"], t["army"], t["extractors"], o.get("army_forward", 0))
            if not all(m in series for m in range(WINDOW_FROM, WINDOW_FROM + 2)):
                continue  # the game did not reach the window's first minutes
            w = first_window(series, ratio, margin)
            f = window_facts(series, w, fall) if w else None
            items.append((f, r["outcome"] == "Win", r["minutes"], f"{r['batch'][11:]}/{r['nn']}"))
        report_windows(f"{pop} (windows of {ratio}x and {margin}+ metal between {WINDOW_FROM}:00 and {WINDOW_TO}:00)", items, fall)
        if list_losses and pop == "1v1 model":
            print("  the model-played losses with a window:")
            for f, w, mn, name in sorted(items, key=lambda i: i[3]):
                if f and not w:
                    kind = "never went" if not f["went"] else ("went, no fall" if not f["converted"] else "fell, lost anyway")
                    print(f"    {name} | lost @{mn:.0f} | window {f['start']}-{f['end']} size {f['size']:.1f} surplus {f['surplus']:,} | his extractors {f['his_ext_start']} fell {f['drop']} | "
                          f"{f['went_share']:.0%} of ours in his half | {kind}")
            print()


def pool_windows(min_os, ratio, margin, fall):
    games = pool_duels(min_os)
    items = []
    for card, teams, m, os_by, duration in games:
        for i, t in enumerate(teams):  # both views: each side's first window against the other
            other = teams[1 - i]
            by = {r["minute"]: r for r in t["curves"]}
            by_o = {r["minute"]: r for r in other["curves"]}
            series = {mn: (by[mn]["army_metal"], by_o[mn]["army_metal"], by_o[mn]["extractors"], by[mn].get("army_metal_in_his_half", 0))
                      for mn in by if mn in by_o and mn * 60 <= duration}
            if not all(mn in series for mn in range(WINDOW_FROM, WINDOW_FROM + 2)):
                continue
            w = first_window(series, ratio, margin)
            f = window_facts(series, w, fall) if w else None
            items.append((f, bool(t.get("won")), duration / 60.0, f"{card['id'][:8]} {t.get('player')}"))
    report_windows(f"the pool's duels, both sides' views (windows of {ratio}x and {margin}+ metal between {WINDOW_FROM}:00 and {WINDOW_TO}:00)", items, fall)
    print("  the sides that held a window and lost:")
    for f, w, mn, name in items:
        if f and not w:
            kind = "never went" if not f["went"] else ("went, no fall" if not f["converted"] else "fell, lost anyway")
            print(f"    {name} | lost @{mn:.0f} | window {f['start']}-{f['end']} size {f['size']:.1f} surplus {f['surplus']:,} | his extractors {f['his_ext_start']} fell {f['drop']} | "
                  f"{f['went_share']:.0%} of theirs in his half | {kind}")


def opt(name, default, cast=float):
    if name in sys.argv:
        return cast(sys.argv[sys.argv.index(name) + 1])
    return default


def main():
    if len(sys.argv) < 2 or sys.argv[1] not in ("extract", "score", "pool", "windows", "pool-windows"):
        print(__doc__)
        sys.exit(2)
    cmd = sys.argv[1]
    positional = [a for i, a in enumerate(sys.argv[2:], 2) if not a.startswith("--") and not sys.argv[i - 1].startswith("--")]
    if cmd == "extract":
        extract(positional[0], positional[1:])
    elif cmd == "score":
        score(positional)
    elif cmd == "pool":
        pool(opt("--min-os", 0.0))
    elif cmd == "windows":
        windows(positional, opt("--ratio", 1.5), opt("--margin", 500.0), opt("--fall", 3, int))
    else:
        pool_windows(opt("--min-os", 0.0), opt("--ratio", 1.5), opt("--margin", 500.0), opt("--fall", 3, int))


if __name__ == "__main__":
    main()
