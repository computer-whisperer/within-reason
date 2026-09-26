#!/usr/bin/env python3
"""Every extractor we lost in a recorded match, one row each: what came, what stood there, what answered and how
late, whether the raiders paid, and whether the spot was rebuilt. The reviewer's raid table
(`.claude/skills/bar-review/SKILL.md`; the pros lose their first at 3:31-4:26 and rebuild, K-map-comet-catcher-remake-1-8-first-extractor-lost).

    run/raid_ledger.py run/matches/<batch>/<NN> [...] [--from MIN] [--json]

Reads the record (`record-<ai>.jsonl`), the truth file (`truth-<ai>.jsonl`: every enemy unit every two seconds;
matches run with WITHIN_REASON_OBSERVE=1, without it the party column is what we saw) and the pianist's log
(`jev-<ai>.jsonl`, the hands' plays) through `run/jev_audit.py`'s reader.

Columns:
  clock, spot (the record's spot index, the cards' numbers on Comet Catcher) and cell; killer (the `destroyed`
  event's type); party: the opponent's armed mobile units within 900 of the spot ten seconds before the death;
  seen s: how long an enemy had been in our sight within 900 of the spot before it died (0 = it died to something
  never seen); ours: our soldiers within 900 and turrets within 700, standing, ten seconds before;
  answer: the first play of the hands against a party placed within 900 of the spot, from the first sight to
  120 s after the death (`hunt`, `attack`, `against` in the play's words, any actor), with its delay from the first
  sight; paid: raiders of that party dead within 120 s of the death (from the truth file); rebuilt: the first
  extractor of ours started within 130 of the spot after the death, and whether it stood.
Then the totals: lost and their metal, by killer type, unseen, undefended (no soldier within 900 and no turret
within 700), answered within 60 s, the answer delay's median, parties that paid, rebuilt and the rebuild delay's
median, spots lost more than once, and the losses by four-minute bucket.
"""
import json
import math
import os
import re
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_audit import Match, clock  # noqa: E402

FPS = 30
ANSWER = re.compile(r"\b(hunt|attack|against)\b.*?(party_\d+)")


def grid_of(header, x, z):
    g = header.get("grid") or {"columns": 8, "rows": 8}
    w, h = header["map"]["width"] / g["columns"], header["map"]["height"] / g["rows"]
    return f"{chr(65 + min(int(x // w), 25))}{int(z // h) + 1}"


def spot_of(header, x, z):
    spots = header.get("metal_spots") or []
    best = min(range(len(spots)), key=lambda i: (spots[i][0] - x) ** 2 + (spots[i][1] - z) ** 2, default=None)
    if best is None or math.dist((spots[best][0], spots[best][1]), (x, z)) >= 130:
        return None
    return best


def at_or_before(keys, f):
    """The largest key at or before f (keys sorted), or None."""
    import bisect
    i = bisect.bisect_right(keys, f)
    return keys[i - 1] if i else None


def ledger(m, from_minute):
    h = m.header
    truth_keys = sorted(m.truth)
    sample_keys = [s["f"] for s in m.samples]
    by_frame = {s["f"]: s for s in m.samples}
    cls = lambda d: m.defs.get(d, {}).get("class")
    # the hands' plays with a frame, and the parties named on the nearest call before each
    calls = [(c["f"], c["parties"]) for c in m.calls if c.get("t") == "call" and isinstance(c.get("parties"), list)]
    call_keys = [f for f, _ in calls]
    parties_at = {f: {p["name"]: p for p in ps} for f, ps in calls}
    plays = []
    for c in m.calls:
        for p in c.get("played") or []:
            plays.append((c["f"], p.get("actor"), p.get("did") or "", p.get("source")))
    plays.sort(key=lambda p: p[0])
    created = [e for e in m.events if e["k"] == "created" and cls(e.get("d")) == "extractor"]
    finished = {e["u"] for e in m.events if e["k"] == "finished"}
    rows = []
    for e in m.events:
        if e["k"] != "destroyed" or cls(e.get("d")) != "extractor" or e["f"] < from_minute * 60 * FPS:
            continue
        x, z, f = e["x"], e["z"], e["f"]
        near = lambda ux, uz, r: math.dist((ux, uz), (x, z)) < r
        # how long an enemy had been in sight near the spot
        seen_from = None
        k = at_or_before(sample_keys, f)
        while k is not None:
            s = by_frame[k]
            if not any(near(u[2], u[3], 900) for u in s.get("en", [])):
                break
            seen_from = k
            i = sample_keys.index(k)
            k = sample_keys[i - 1] if i > 0 else None
        # the party from the truth, ten seconds before
        party, party_ids = {}, set()
        tk = at_or_before(truth_keys, f - 10 * FPS)
        if tk is not None:
            for u in m.truth[tk]:
                if m.is_soldier_name(u[1]) and near(u[2], u[3], 900):
                    party[u[1]] = party.get(u[1], 0) + 1
                    party_ids.add(u[0])
        elif seen_from is not None:
            s = by_frame[at_or_before(sample_keys, f)]
            for u in s.get("en", []):
                if near(u[2], u[3], 900) and u[1] >= 0 and m.is_soldier(u[1]):
                    party[m.def_name(u[1])] = party.get(m.def_name(u[1]), 0) + 1
        # ours, ten seconds before
        sk = at_or_before(sample_keys, f - 10 * FPS)
        soldiers = turrets = 0
        if sk is not None:
            for u in by_frame[sk]["own"]:
                if u[5] & 1:
                    continue
                if cls(u[1]) == "army" and near(u[2], u[3], 900):
                    soldiers += 1
                elif cls(u[1]) == "turret" and near(u[2], u[3], 700):
                    turrets += 1
        # the first play against a party near the spot
        answer = None
        start = seen_from if seen_from is not None else f
        for pf, actor, did, source in plays:
            if pf < start:
                continue
            if pf > f + 120 * FPS:
                break
            mm = ANSWER.search(did)
            if not mm:
                continue
            ck = at_or_before(call_keys, pf)
            p = parties_at.get(ck, {}).get(mm.group(2)) if ck is not None else None
            if p and near(p["x"], p["z"], 900):
                answer = {"clock": clock(pf), "delay_s": (pf - start) // FPS, "actor": actor, "did": did[:70], "source": source}
                break
        # did the raiders pay
        paid = 0
        if party_ids:
            tk2 = at_or_before(truth_keys, f + 120 * FPS)
            alive = {u[0] for u in m.truth[tk2]} if tk2 is not None else party_ids
            paid = len(party_ids - alive)
        # rebuilt
        rebuilt = None
        for c in created:
            if c["f"] > f and near(c["x"], c["z"], 130):
                rebuilt = {"clock": clock(c["f"]), "delay_s": (c["f"] - f) // FPS, "stood": c["u"] in finished}
                break
        spot = spot_of(h, x, z)
        rows.append({"clock": clock(f), "frame": f, "spot": spot, "grid": grid_of(h, x, z), "metal": m.defs.get(e["d"], {}).get("metal", 0),
                     "killer": m.def_name(e["by_d"]) if e.get("by_d") is not None else None, "party": party,
                     "seen_s": (f - seen_from) // FPS if seen_from is not None else 0, "soldiers_near": soldiers, "turrets_near": turrets,
                     "answer": answer, "raiders_paid": paid, "rebuilt": rebuilt})
    return rows


def totals(rows):
    lost = len(rows)
    delays = [r["answer"]["delay_s"] for r in rows if r["answer"]]
    rebuilds = [r["rebuilt"]["delay_s"] for r in rows if r["rebuilt"]]
    killers = {}
    for r in rows:
        killers[r["killer"] or "?"] = killers.get(r["killer"] or "?", 0) + 1
    spots = {}
    for r in rows:
        spots[r["spot"]] = spots.get(r["spot"], 0) + 1
    buckets = {}
    for r in rows:
        b = (r["frame"] // (240 * FPS)) * 4
        buckets[f"{b}-{b + 4}"] = buckets.get(f"{b}-{b + 4}", 0) + 1
    return {"lost": lost, "metal": sum(r["metal"] for r in rows), "by_killer": dict(sorted(killers.items(), key=lambda kv: -kv[1])),
            "unseen": sum(1 for r in rows if r["seen_s"] == 0), "undefended": sum(1 for r in rows if r["soldiers_near"] == 0 and r["turrets_near"] == 0),
            "with_turret": sum(1 for r in rows if r["turrets_near"] > 0), "answered": len(delays), "answered_within_60": sum(1 for d in delays if d <= 60),
            "answer_delay_median_s": statistics.median(delays) if delays else None, "parties_paid": sum(1 for r in rows if r["raiders_paid"] > 0),
            "rebuilt": len(rebuilds), "rebuilt_stood": sum(1 for r in rows if r["rebuilt"] and r["rebuilt"]["stood"]),
            "rebuild_delay_median_s": statistics.median(rebuilds) if rebuilds else None,
            "spots_lost_twice_or_more": {f"spot_{k}": v for k, v in spots.items() if v > 1 and k is not None}, "by_minute_bucket": buckets}


def print_ledger(m, rows, t):
    r = m.result or {}
    print(f"== {m.label}: {r.get('outcome', '?')} in {r.get('game_minutes', 0):.1f} min; {t['lost']} extractors lost ({t['metal']:.0f} metal)" + ("" if m.truth else "; NO TRUTH FILE: parties are what we saw"))
    print(f"   {'clock':>5} {'spot':>7} {'cell':>4} {'killer':<9} {'party (10 s before)':<26} {'seen':>5} {'sold':>4} {'tur':>3}  {'answer (delay from first sight)':<58} {'paid':>4}  rebuilt")
    for x in rows:
        party = ", ".join(f"{n} {k}" for k, n in sorted(x["party"].items(), key=lambda kv: -kv[1])) or "-"
        a = x["answer"]
        answer = f"{a['clock']} +{a['delay_s']}s {a['actor']}: {a['did'][:40]}" if a else "none"
        rb = x["rebuilt"]
        rebuilt = f"{rb['clock']} (+{rb['delay_s']}s{', stood' if rb['stood'] else ', not finished'})" if rb else "no"
        print(f"   {x['clock']:>5} {('spot_%d' % x['spot']) if x['spot'] is not None else '?':>7} {x['grid']:>4} {x['killer'] or '?':<9} {party[:26]:<26} {x['seen_s']:>4}s {x['soldiers_near']:>4} {x['turrets_near']:>3}  {answer:<58} {x['raiders_paid']:>4}  {rebuilt}")
    print(f"   totals: by killer {t['by_killer']}; unseen {t['unseen']}; undefended (no soldier within 900, no turret within 700) {t['undefended']}; with a turret {t['with_turret']}; "
          f"answered {t['answered']} ({t['answered_within_60']} within 60 s of first sight, median delay {t['answer_delay_median_s']} s); parties that paid {t['parties_paid']}; "
          f"rebuilt {t['rebuilt']} ({t['rebuilt_stood']} stood, median {t['rebuild_delay_median_s']} s); lost twice or more {t['spots_lost_twice_or_more']}; by minute {t['by_minute_bucket']}")
    print()


def main():
    args = sys.argv[1:]
    if not args or "-h" in args or "--help" in args:
        print(__doc__)
        sys.exit(2)
    as_json = "--json" in args
    from_minute = 0
    dirs = []
    i = 0
    while i < len(args):
        if args[i] == "--from":
            from_minute = float(args[i + 1]); i += 1
        elif not args[i].startswith("--"):
            dirs.append(args[i])
        i += 1
    for d in dirs:
        m = Match(d.rstrip("/"))
        rows = ledger(m, from_minute)
        t = totals(rows)
        if as_json:
            print(json.dumps({"match": m.dir, "rows": rows, "totals": t}))
        else:
            print_ledger(m, rows, t)


if __name__ == "__main__":
    main()
