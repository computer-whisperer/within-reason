#!/usr/bin/env python3
"""Scores the player's `predict` rows against the truth file (the user, 2026-10-17: "some way for the player to report
hidden state predictions throughout the game that we can easily score later"): every `predictions-*.jsonl` of a match
directory (one per session name: `predictions-strategist-N.jsonl`, `predictions-commander.jsonl`), each row's numbers
beside what the opponent really had at that frame (`truth-<ai>.jsonl`, through run/analyze_match.py's reader and its
army and extractor classes), what the report showed the player at that turn (the floor it could have copied), and
the BARb table's median for that minute where one exists (the prior it could have read off the brief).

usage: run/predictions.py MATCH [--json] [--quiet]
  MATCH is a match directory (run/matches/<batch>/00) or a batch (its 00 is taken).
Per row: the clock, predicted / true extractors, predicted / true army metal with the signed error, the advanced
factory as predicted and as the truth has it, the report's floor and the prior with their own errors, the basis.
Then the summary: rows; the median absolute error of the army and extractor predictions as a share of the truth
(lower is better) and the median signed error (the bias: over means the player saw more of him than there was); the
same for the floor and the prior, so a prediction that beats neither is a copy; the "ahead" rate: rows where our army
stood at twice the predicted one while the truth had it under one and a half times; the advanced-factory calls: how
many right, and the first "building" or "standing" call against the truth's first advanced factory.
An advanced factory is a factory whose metal cost is over 1,500 (the record's unit table has no tier: tier-1 plants
cost 600 to 1,000, the advanced ones 2,300 and more, gantries far more).
"""
import argparse
import glob
import json
import os
import re
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import analyze_match  # noqa: E402

FPS = 30
ADVANCED_FACTORY_METAL = 1500


def clock(frame):
    return f"{frame // (60 * FPS)}:{(frame // FPS) % 60:02d}"


def load_rows(directory):
    rows = []
    for path in sorted(glob.glob(os.path.join(directory, "predictions-*.jsonl"))):
        who = os.path.basename(path)[len("predictions-"):-len(".jsonl")]
        for line in open(path, errors="replace"):
            try:
                r = json.loads(line)
            except ValueError:
                continue
            r["who"] = who
            rows.append(r)
    rows.sort(key=lambda r: (r.get("frame", 0), r["who"]))
    return rows


def truth_at(match, frame):
    """(his extractors standing, his army metal alive, his advanced factory: none|building|standing)."""
    units = match.theirs_at(frame)
    extractors = sum(1 for u in units if match.cls(u[1]) == "extractor" and not u[5])
    army = analyze_match.summarise(units, match).get("army", [0, 0.0])[1]
    advanced = "none"
    for u in units:
        if match.cls(u[1]) == "factory" and match.metal(u[1]) > ADVANCED_FACTORY_METAL:
            advanced = "standing" if not u[5] else ("building" if advanced != "standing" else advanced)
    return extractors, round(army), advanced


def first_advanced(match):
    """The truth's first frame with an advanced factory being built, and the first with one standing."""
    building = standing = None
    for t in match.truth:
        for u in t["enemy"]:
            if match.cls(u[1]) == "factory" and match.metal(u[1]) > ADVANCED_FACTORY_METAL:
                if u[5] and building is None:
                    building = t["f"]
                if not u[5] and standing is None:
                    standing = t["f"]
                    building = building if building is not None else t["f"]
    return building, standing


def prior_table(match_dir):
    """The BARb table's medians by minute for this map and profile, from docs/briefs/reference, or None."""
    batch = os.path.join(os.path.dirname(match_dir.rstrip("/")), "batch.json")
    try:
        b = json.load(open(batch))
    except (OSError, ValueError):
        return None
    opponent = b.get("opponent") or ""
    m = re.match(r"BARb (\S+)", opponent)
    if not m:
        return None
    slug = (b.get("map") or "").lower().replace(" ", "-")
    path = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "docs", "briefs", "reference", f"{slug}-barb-{m.group(1)}.md")
    if not os.path.isfile(path):
        return None
    table = {}
    for line in open(path):
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 6 or not cells[0].isdigit():
            continue
        try:
            minute = int(cells[0])
            ext = [float(x) for x in cells[2].split("/")]
            army = [float(x) for x in cells[5].split("/")]
        except ValueError:
            continue
        table[minute] = (ext[1], army[1])
    return table or None


def prior_at(table, frame):
    minute = frame // (60 * FPS)
    at = max((m for m in table if m <= minute), default=None)
    return table[at] if at is not None else None


def pct(value, truth):
    return None if not truth else 100.0 * (value - truth) / truth


def med(values):
    values = [v for v in values if v is not None]
    return statistics.median(values) if values else None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("match")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--quiet", action="store_true")
    a = ap.parse_args()
    match_dir = a.match if glob.glob(os.path.join(a.match, "record-*.jsonl")) else os.path.join(a.match, "00")
    rows = load_rows(match_dir)
    if not rows:
        print(f"{match_dir}: no predictions-*.jsonl", file=sys.stderr)
        return 1
    match = analyze_match.Match(match_dir)
    if not match.truth:
        print(f"{match_dir}: no truth file; the rows are listed, nothing scored", file=sys.stderr)
    prior = prior_table(match_dir)
    first_building, first_standing = first_advanced(match) if match.truth else (None, None)
    scored = []
    for r in rows:
        frame = int(r.get("frame", 0))
        out = {"who": r["who"], "frame": frame, "clock": clock(frame), "pred_extractors": r.get("extractors"), "pred_army": r.get("army_metal"),
               "pred_advanced": r.get("advanced_factory"), "basis": r.get("basis", ""), "report": r.get("report") or {}}
        if match.truth:
            ext, army, adv = truth_at(match, frame)
            out.update({"true_extractors": ext, "true_army": army, "true_advanced": adv,
                        "err_extractors_pct": pct(r.get("extractors", 0), ext), "err_army_pct": pct(r.get("army_metal", 0), army)})
            seen = (r.get("report") or {}).get("enemy_soldiers_seen_metal")
            seen_ext = (r.get("report") or {}).get("enemy_extractors_seen")
            out["floor_army_pct"] = pct(seen, army) if seen is not None else None
            out["floor_extractors_pct"] = pct(seen_ext, ext) if seen_ext is not None else None
            ours = ((r.get("report") or {}).get("ours") or {}).get("army_metal")
            if ours and r.get("army_metal"):
                out["ahead_call"] = ours >= 2 * r["army_metal"] and (army == 0 or ours < 1.5 * army)
            if prior:
                p = prior_at(prior, frame)
                if p:
                    out["prior_extractors_pct"] = pct(p[0], ext)
                    out["prior_army_pct"] = pct(p[1], army)
        scored.append(out)

    summary = {"match": os.path.basename(os.path.dirname(match_dir.rstrip("/"))), "rows": len(scored), "by_who": {}}
    if match.truth:
        for who in sorted({s["who"] for s in scored}):
            mine = [s for s in scored if s["who"] == who]
            def ab(key):
                return med([abs(s[key]) for s in mine if s.get(key) is not None])
            def sg(key):
                return med([s[key] for s in mine if s.get(key) is not None])
            adv_right = sum(1 for s in mine if s.get("pred_advanced") == s.get("true_advanced"))
            first_call = next((s["frame"] for s in mine if s.get("pred_advanced") in ("building", "standing")), None)
            summary["by_who"][who] = {
                "rows": len(mine),
                "army_abs_err_pct": ab("err_army_pct"), "army_bias_pct": sg("err_army_pct"),
                "extractors_abs_err_pct": ab("err_extractors_pct"), "extractors_bias_pct": sg("err_extractors_pct"),
                "floor_army_abs_err_pct": ab("floor_army_pct"), "floor_extractors_abs_err_pct": ab("floor_extractors_pct"),
                "prior_army_abs_err_pct": ab("prior_army_pct"), "prior_extractors_abs_err_pct": ab("prior_extractors_pct"),
                "ahead_calls": sum(1 for s in mine if s.get("ahead_call")),
                "advanced_right": adv_right,
                "first_advanced_call": clock(first_call) if first_call is not None else None,
                "truth_first_advanced_building": clock(first_building) if first_building is not None else None,
                "truth_first_advanced_standing": clock(first_standing) if first_standing is not None else None,
            }
    if a.json:
        print(json.dumps({"summary": summary, "rows": scored}, indent=1))
        return 0
    if not a.quiet:
        e = lambda v: "-" if v is None else f"{v:+.0f}%"
        for s in scored:
            if "true_army" in s:
                fa = s.get("floor_army_pct")
                pa = s.get("prior_army_pct")
                print(f"{s['clock']:>6} {s['who']:<14} ext {s['pred_extractors']:>3} / {s['true_extractors']:>3} ({e(s['err_extractors_pct'])})  army {s['pred_army']:>6} / {s['true_army']:>6} ({e(s['err_army_pct'])})  adv {s['pred_advanced']:<8} / {s['true_advanced']:<8}"
                      + (f"  floor {e(fa)}" if fa is not None else "") + (f"  prior {e(pa)}" if pa is not None else "") + ("  AHEAD" if s.get("ahead_call") else "") + f"  | {s['basis'][:90]}")
            else:
                print(f"{s['clock']:>6} {s['who']:<14} ext {s['pred_extractors']} army {s['pred_army']} adv {s['pred_advanced']} | {s['basis'][:90]}")
    print(f"\n{summary['match']}: {summary['rows']} predictions")
    for who, v in summary["by_who"].items():
        f = lambda x: "-" if x is None else f"{x:.0f}"
        print(f"  {who}: {v['rows']} rows; army |err| {f(v['army_abs_err_pct'])}% (bias {f(v['army_bias_pct'])}%; the floor {f(v['floor_army_abs_err_pct'])}%, the prior {f(v['prior_army_abs_err_pct'])}%); "
              f"extractors |err| {f(v['extractors_abs_err_pct'])}% (bias {f(v['extractors_bias_pct'])}%; the floor {f(v['floor_extractors_abs_err_pct'])}%, the prior {f(v['prior_extractors_abs_err_pct'])}%); "
              f"ahead calls {v['ahead_calls']}; advanced factory right {v['advanced_right']} of {v['rows']}, first called {v['first_advanced_call'] or '-'} against the truth's building {v['truth_first_advanced_building'] or '-'} / standing {v['truth_first_advanced_standing'] or '-'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
