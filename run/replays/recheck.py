#!/usr/bin/env python3
"""Re-check the Comet Catcher synthesis claims (docs/knowledge/maps.md, the K-map-comet-catcher-remake-1-8-* section,
made on 29 games with both players at OS 25 and above) on a pool cut by OS, side by side: the players' advice
(2026-09-24) was that 25 is too low to learn from and 40 and above is where the lessons are. Also the pointed one:
more than three solars in the early opening is a major mistake.

    run/replays/recheck.py [--map "Comet Catcher Remake 1.8"] [--floors 25,40] [--manifest run/data/replays/manifest.jsonl]

One row per side of every carded game whose players are all at or above the floor; numbers as medians (with the
count) or shares. Reads the cards directly (`card.json`: build_order, factories, curves, spots_taken, first_extractor_lost,
fights) and the manifest (OS, winners).
"""
import argparse, collections, json, os, statistics

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SOLAR = ("armsolar", "corsolar")
LLT = ("armllt", "corllt")
PLANTS = ("armvp", "corvp", "armlab", "corlab", "armap", "corap", "armsy", "corsy", "armhp", "corhp")


def secs(c):
    if not c or "?" in str(c):
        return None
    m, s = str(c).split(":")
    return int(m) * 60 + int(s)


def clock(s):
    return "-" if s is None else f"{int(s) // 60}:{int(s) % 60:02d}"


def sides(manifest, map_name, floor):
    out = []
    for line in open(manifest):
        r = json.loads(line)
        if not (r.get("keep") and r.get("card")):
            continue
        if map_name and r.get("map_script") != map_name:
            continue
        oss = [p.get("os") for p in r["players"]]
        if any(o is None or o < floor for o in oss):
            continue
        c = json.load(open(os.path.join(r["match"], "card.json")))
        winners = set(r.get("winners") or [])
        by_team = {p["team"]: p for p in r["players"]}
        for t in c["teams"]:
            team = int(t["team"])
            p = by_team.get(team, {})
            bo = t.get("build_order") or []
            ff = t["factories"][0] if t.get("factories") else {}
            ff_s = secs(ff.get("started"))
            solars_before_plant = sum(1 for b in bo if b["unit"] in SOLAR and (ff_s is None or secs(b["clock"]) < ff_s))
            solars_by = lambda limit: sum(1 for b in bo if b["unit"] in SOLAR and secs(b["clock"]) is not None and secs(b["clock"]) < limit)
            llt_by = lambda limit: sum(1 for b in bo if b["unit"] in LLT and secs(b["clock"]) is not None and secs(b["clock"]) < limit)
            first_llt = next((secs(b["clock"]) for b in bo if b["unit"] in LLT and b.get("by") == "commander"), None)
            curves = {row["minute"]: row for row in t.get("curves") or []}
            at = lambda minute, key: (curves.get(minute) or {}).get(key)
            fights = t.get("fights") or []
            rows45 = sum(f["ours_lost"] + f["theirs_lost"] for f in fights if f["grid"][1:] in ("4", "5"))
            deaths = sum(f["ours_lost"] + f["theirs_lost"] for f in fights)
            after = []
            if ff_s is not None:
                seen_plant = False
                for b in bo:
                    if not seen_plant:
                        seen_plant = b["unit"] in PLANTS and secs(b["clock"]) == ff_s
                        continue
                    after.append(b["unit"])
                    if len(after) == 5:
                        break
            out.append(dict(
                after=tuple(after),
                id=r["id"][:8], player=p.get("name"), os=p.get("os"), won=p.get("name") in winners, faction=t["side"], start=t["start"]["grid"],
                higher_os=(p.get("os") is not None and p["os"] == max(o for o in oss if o is not None)),
                duration=r["duration_ms"] // 1000, ff_unit=ff.get("unit"), ff_s=ff_s,
                sf_s=secs(t["factories"][1].get("started")) if len(t.get("factories") or []) > 1 else None,
                t2_s=secs((t.get("first_tier2") or {}).get("clock")),
                first_llt=first_llt, llt8=llt_by(480),
                solars_before_plant=solars_before_plant, solars2=solars_by(120), solars3=solars_by(180), solars5=solars_by(300),
                ex4=at(4, "extractors"), ex8=at(8, "extractors"), ex12=at(12, "extractors"), inc8=at(8, "metal_income"), inc12=at(12, "metal_income"),
                army8=at(8, "army_metal"), army12=at(12, "army_metal"),
                lost_s=secs((t.get("first_extractor_lost") or {}).get("clock")),
                rows45=rows45, deaths=deaths,
            ))
    return out


def med(rows, key):
    vals = [r[key] for r in rows if r.get(key) is not None]
    return (statistics.median(vals), len(vals)) if vals else (None, 0)


def share(rows, pred):
    n = sum(1 for r in rows if pred(r))
    return f"{n} of {len(rows)} ({100 * n / max(1, len(rows)):.0f}%)"


def report(pools):
    names = list(pools)
    def line(label, f):
        print(f"{label:58} " + " | ".join(f"{f(pools[n]):>28}" for n in names))
    print(f"{'':58} " + " | ".join(f"{n:>28}" for n in names))
    line("sides (games)", lambda rs: f"{len(rs)} ({len({r['id'] for r in rs})})")
    line("first factory a vehicle plant", lambda rs: share(rs, lambda r: r["ff_unit"] in ("armvp", "corvp")))
    line("first factory started before 0:45 (the fast plant)", lambda rs: share(rs, lambda r: r["ff_s"] is not None and r["ff_s"] < 45))
    line("first factory started, median", lambda rs: "%s (n=%d)" % (clock(med(rs, 'ff_s')[0]), med(rs, 'ff_s')[1]))
    line("commander's first light turret before 0:30", lambda rs: share(rs, lambda r: r["first_llt"] is not None and r["first_llt"] < 30))
    line("commander's first light turret, median", lambda rs: "%s (n=%d)" % (clock(med(rs, 'first_llt')[0]), med(rs, 'first_llt')[1]))
    print("-- solars (the friend's claim: more than three in the early opening is a major mistake)")
    line("solars before the first factory: 0-1 / 2 / 3 / 4+", lambda rs: "/".join(str(sum(1 for r in rs if b(r["solars_before_plant"]))) for b in (lambda n: n <= 1, lambda n: n == 2, lambda n: n == 3, lambda n: n >= 4)))
    line("solars before the first factory, median", lambda rs: "%s (n=%d)" % med(rs, 'solars_before_plant'))
    line("solars by 2:00: 0-1 / 2 / 3 / 4+", lambda rs: "/".join(str(sum(1 for r in rs if b(r["solars2"]))) for b in (lambda n: n <= 1, lambda n: n == 2, lambda n: n == 3, lambda n: n >= 4)))
    line("solars by 2:00, median", lambda rs: "%s (n=%d)" % med(rs, 'solars2'))
    line("solars by 3:00, median", lambda rs: "%s (n=%d)" % med(rs, 'solars3'))
    line("solars by 5:00, median", lambda rs: "%s (n=%d)" % med(rs, 'solars5'))
    line("more than three solars by 2:00: winners / losers", lambda rs: f"{sum(1 for r in rs if r['won'] and r['solars2'] > 3)} of {sum(1 for r in rs if r['won'])} / {sum(1 for r in rs if not r['won'] and r['solars2'] > 3)} of {sum(1 for r in rs if not r['won'])}")
    line("more than three solars by 3:00: winners / losers", lambda rs: f"{sum(1 for r in rs if r['won'] and r['solars3'] > 3)} of {sum(1 for r in rs if r['won'])} / {sum(1 for r in rs if not r['won'] and r['solars3'] > 3)} of {sum(1 for r in rs if not r['won'])}")
    line("won with more than three solars by 3:00", lambda rs: share([r for r in rs if r["solars3"] > 3], lambda r: r["won"]))
    line("won with three or fewer solars by 3:00", lambda rs: share([r for r in rs if r["solars3"] <= 3], lambda r: r["won"]))
    print("-- the five actions after the first factory (build order entries: the commander's builds and the factory's units)")
    for i in range(5):
        line(f"action {i + 1} after the factory, most common", lambda rs, i=i: ", ".join(f"{u} {n}" for u, n in collections.Counter(r["after"][i] for r in rs if len(r["after"]) > i).most_common(3)))
    line("solars among the five, median (winners / losers)", lambda rs: f"{med([dict(r, k=sum(1 for u in r['after'] if u in SOLAR)) for r in rs if r['won']], 'k')[0]} / {med([dict(r, k=sum(1 for u in r['after'] if u in SOLAR)) for r in rs if not r['won']], 'k')[0]}")
    line("whole sequences, most common", lambda rs: "; ".join(f"{'-'.join(u.replace('arm', '').replace('cor', '') for u in seq)} x{n}" for seq, n in collections.Counter(r["after"] for r in rs if len(r["after"]) == 5).most_common(2)))
    print("-- economy and the rest")
    line("extractors at 4:00 / 8:00 / 12:00, medians", lambda rs: " / ".join(str(med(rs, k)[0]) for k in ("ex4", "ex8", "ex12")))
    line("extractors at 8:00: winners / losers", lambda rs: f"{med([r for r in rs if r['won']], 'ex8')[0]} / {med([r for r in rs if not r['won']], 'ex8')[0]}")
    line("extractors at 12:00: winners / losers", lambda rs: f"{med([r for r in rs if r['won']], 'ex12')[0]} / {med([r for r in rs if not r['won']], 'ex12')[0]}")
    line("income at 8:00: winners / losers", lambda rs: f"{med([r for r in rs if r['won']], 'inc8')[0]} / {med([r for r in rs if not r['won']], 'inc8')[0]}")
    line("army metal at 8:00: winners / losers", lambda rs: f"{med([r for r in rs if r['won']], 'army8')[0]} / {med([r for r in rs if not r['won']], 'army8')[0]}")
    line("light turrets by 8:00: winners / losers (median)", lambda rs: f"{med([r for r in rs if r['won']], 'llt8')[0]} / {med([r for r in rs if not r['won']], 'llt8')[0]}")
    line("second factory started (share; median)", lambda rs: f"{share(rs, lambda r: r['sf_s'] is not None)}; {clock(med(rs, 'sf_s')[0])}")
    line("tier 2 reached", lambda rs: share(rs, lambda r: r["t2_s"] is not None))
    line("first extractor lost, median; before 3:00", lambda rs: f"{clock(med(rs, 'lost_s')[0])}; {share(rs, lambda r: r['lost_s'] is not None and r['lost_s'] < 180)}")
    line("deaths in the fight lists on rows 4-5", lambda rs: f"{sum(r['rows45'] for r in rs)} of {sum(r['deaths'] for r in rs)} ({100 * sum(r['rows45'] for r in rs) / max(1, sum(r['deaths'] for r in rs)):.0f}%)")
    line("the higher-OS side won (games)", lambda rs: share([r for r in rs if r["higher_os"]], lambda r: r["won"]))
    line("game length, median", lambda rs: clock(med(rs, 'duration')[0]))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--map", default="Comet Catcher Remake 1.8")
    ap.add_argument("--floors", default="25,40")
    ap.add_argument("--manifest", default=os.path.join(REPO, "run/data/replays/manifest.jsonl"))
    a = ap.parse_args()
    pools = {}
    for f in a.floors.split(","):
        pools[f"OS {f}+"] = sides(a.manifest, a.map, float(f))
    report(pools)


if __name__ == "__main__":
    main()
