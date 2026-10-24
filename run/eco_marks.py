#!/usr/bin/env python3
"""One team's economy from a match record at the territory benchmark's marks (bots/territory/README.md: metal
income at 5 / 10 / 15 / 20 against InactiveAI), and the shape behind it: what stands at each mark, who built what,
the tier-2 clocks, the nano turrets' placing, the stalls, the metal made. Reads any record in the format of
docs/harness/record-format.md: a replay's (run/replay_match.py writes record-<team>.jsonl per team: a person's
game, thebluegecko against InactiveAI on 2026-10-21 for instance) or the flagship's. A bot of bots/ writes no
record; its side is run/territory_curves.py from its own lines.

    run/eco_marks.py <record-N.jsonl> [--marks 5,10,15,20] [--minutes] [--shape]

--minutes prints every minute (income, extractors by tier, constructors, converters, both stocks); --shape adds
the builders by type per minute, the kinds each builder type began, the tier-2 clocks (begun -> done, per minute),
how many nano turrets stood within 300 elmos of each tier-2 frame when it was placed, and what the builders and
turrets were building at the marks (the record's `bp` rows, replay records from 2026-10-13 on).
"""
import collections, json, math, sys

TIER2 = ("avp", "alab", "aap", "acv", "ack", "aca", "fus", "afus", "moho", "mmkr", "makr", "estor", "uwadves", "nanotc", "advsol", "ap")


def read(rec):
    header = None
    samples, events = [], []
    for line in open(rec, errors="replace"):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        t = r.get("t")
        if t == "header" and header is None:
            header = r
        elif t == "s":
            samples.append(r)
        elif t == "ev":
            events.append(r)
    return header, samples, events


def kind_of(d):
    n = d["name"]
    c = d.get("class", "")
    if n.endswith("moho") or n.endswith("uwmme"):
        return "mex2"
    if d.get("extracts_metal", 0) > 0 or n.endswith("mex"):
        return "mex1"
    if n.endswith("makr") or n.endswith("mmkr"):
        return "conv"
    if n.endswith("fus") or n.endswith("afus"):
        return "fusion"
    if n.endswith("advsol"):
        return "advsol"
    if n.endswith("solar") or n.endswith("win") or n.endswith("tide"):
        return "e1"
    if n.endswith("nanotc"):
        return "nano"
    if n.endswith("estor") or n.endswith("mstor") or n.endswith("uwadves") or n.endswith("uwadvms"):
        return "store"
    if c == "factory":
        return "plant"
    if c == "commander":
        return "com"
    if c == "builder":
        return "cons"
    if c == "turret":
        return "turret"
    if c == "army":
        return "army"
    return "other"


def is_kind(unit, suffix):
    """A unit name ends with the suffix and is not its advanced cousin (fus against afus, ap against aap)."""
    return unit.endswith(suffix) and not (suffix in ("fus", "ap") and unit.endswith("a" + suffix))


def clock(f):
    return "-" if f is None else f"{f // 1800}:{(f // 30) % 60:02d}"


def at(samples, minute):
    s = min(samples, key=lambda s: abs(s["f"] - minute * 1800))
    return s if abs(s["f"] - minute * 1800) <= 90 else None


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    rec = args[0]
    marks = [int(m) for m in args[args.index("--marks") + 1].split(",")] if "--marks" in args else [5, 10, 15, 20]
    header, samples, events = read(rec)
    defs = header["unit_defs"]
    name = [d["name"] for d in defs]
    kind = [kind_of(d) for d in defs]
    created = [e for e in events if e.get("k") == "created"]
    finished = {e["u"]: e for e in events if e.get("k") == "finished"}
    udef = {e["u"]: e["d"] for e in created}
    last = samples[-1]["f"] / 1800
    print(f"{rec}: {len(samples)} samples to {last:.1f} min, {header.get('map', {}).get('name')}")
    cols = ["mex1", "mex2", "cons", "plant", "nano", "e1", "advsol", "fusion", "conv", "store", "army"]
    print(f"{'min':>4} {'m inc':>6} {'m use':>6} {'m stock':>8} {'e inc':>6} {'e use':>6} {'e stock':>8} " + " ".join(f"{c:>6}" for c in cols) + "  frames")
    for m in marks:
        s = at(samples, m)
        if s is None:
            continue
        count, frames = collections.Counter(), collections.Counter()
        for u in s["own"]:
            (frames if u[5] & 1 else count)[kind[u[1]]] += 1
        print(f"{m:>4} {s['m'][1]:>6.0f} {s['m'][2]:>6.0f} {s['m'][0]:>8.0f} {s['e'][1]:>6.0f} {s['e'][2]:>6.0f} {s['e'][0]:>8.0f} " + " ".join(f"{count[c]:>6}" for c in cols) + "  " + " ".join(f"{k}:{v}" for k, v in sorted(frames.items())))
    made, made_at = 0.0, {}
    for s in samples:
        made += s["m"][1]
        if s["f"] % 1800 == 0:
            made_at[s["f"] // 1800] = made
    print("metal made (the income integrated) at the marks: " + " / ".join(f"{made_at.get(m, 0):,.0f}" for m in marks))
    estall = [s for s in samples if s["e"][0] < 0.05 * max(1, s["e"][3])]
    mzero = sum(1 for s in samples if s["m"][0] < 0.05 * max(1, s["m"][3]))
    mcap = sum(1 for s in samples if s["m"][0] > 0.98 * s["m"][3])
    print(f"energy under 5 % of storage {len(estall)} s (minutes " + " ".join(sorted({str(s['f'] // 1800) for s in estall}, key=int)) + f"); metal under 5 % {mzero} s, at its cap {mcap} s")
    plants = [(e["f"], finished.get(e["u"], {}).get("f"), name[e["d"]]) for e in created if defs[e["d"]].get("class") == "factory"]
    print("factories: " + ", ".join(f"{n} {clock(b)}->{clock(d)}" for b, d, n in plants[:8]) + (" ..." if len(plants) > 8 else ""))
    print("first 30 begun: " + " ".join(f"{clock(e['f'])} {name[e['d']]}" for e in created[:30]))
    if "--minutes" in args or "--shape" in args:
        print("by minute: income, energy income, extractors tier 1 + tier 2, constructors, converters, nanos, stocks")
        for m in range(1, int(last) + 1):
            s = at(samples, m)
            if s is None:
                continue
            c = collections.Counter(kind[u[1]] for u in s["own"] if not (u[5] & 1))
            print(f"  {m:>2} inc {s['m'][1]:>5.0f} e {s['e'][1]:>6.0f} mex {c['mex1']:>2}+{c['mex2']:<2} cons {c['cons']:>3} conv {c['conv']:>3} nanos {c['nano']:>3} stock m {s['m'][0]:>6.0f}/{s['m'][3]:<6.0f} e {s['e'][0]:>6.0f}/{s['e'][3]:.0f}")
    if "--shape" not in args:
        return
    print("builders by type per minute:")
    btypes = sorted({name[u[1]] for s in samples for u in s["own"] if defs[u[1]].get("class") in ("builder", "commander")})
    for m in range(1, int(last) + 1):
        s = at(samples, m)
        if s is None:
            continue
        c = collections.Counter(name[u[1]] for u in s["own"] if not (u[5] & 1))
        print(f"  {m:>2} " + " ".join(f"{b}:{c[b]}" for b in btypes if c[b]))
    print("what each builder type began:")
    who = collections.defaultdict(collections.Counter)
    for e in created:
        b = e.get("by")
        who[name[udef[b]] if b in udef else "-"][name[e["d"]]] += 1
    for bt, c in who.items():
        print(f"  {bt:>10}: " + ", ".join(f"{k} {v}" for k, v in c.most_common(14)))
    print("tier-2 clocks (begun -> done; begun per minute):")
    for suffix in TIER2:
        items = sorted(((e["f"], finished.get(e["u"], {}).get("f")) for e in created if is_kind(name[e["d"]], suffix) and defs[e["d"]].get("class") != "army"), key=lambda t: (t[0], t[1] or 0))
        if not items:
            continue
        per = collections.Counter(b // 1800 for b, _ in items)
        print(f"  {suffix:>7} n={len(items):>3} " + " ".join(f"{clock(b)}->{clock(d)}" for b, d in items[:5]) + "  per minute " + " ".join(f"{m}:{per[m]}" for m in sorted(per)))
    nanos = [(e["x"], e["z"], e["f"]) for e in created if name[e["d"]].endswith("nanotc")]
    home = next(((e["x"], e["z"]) for e in created if defs[e["d"]].get("class") == "commander"), (0, 0))
    print(f"nano turrets {len(nanos)}; within 300 of each tier-2 frame when placed (min / median / max) and the frames' distance from home (median / max):")
    for suffix in ("fus", "afus", "mmkr", "makr", "moho", "nanotc"):
        rows = [(sum(1 for x, z, f in nanos if f <= e["f"] and math.hypot(x - e["x"], z - e["z"]) < 300), math.hypot(e["x"] - home[0], e["z"] - home[1])) for e in created if is_kind(name[e["d"]], suffix)]
        if not rows:
            continue
        w = sorted(r[0] for r in rows)
        d = sorted(r[1] for r in rows)
        print(f"  {suffix:>6} n={len(rows):>3} nanos {w[0]} / {w[len(w) // 2]} / {w[-1]}  distance {d[len(d) // 2]:.0f} / {d[-1]:.0f}")
    if any(s.get("bp") for s in samples):
        print("building at the marks (builder type -> target, count; then idle builders):")
        for m in marks:
            s = at(samples, m)
            if s is None:
                continue
            own = {u[0]: u[1] for u in s["own"]}
            target, idle = collections.Counter(), collections.Counter()
            for uid, _, tgt in s.get("bp", []):
                b = name[own[uid]] if uid in own else "?"
                if tgt:
                    target[(b, name[udef[tgt]] if tgt in udef else "?")] += 1
                else:
                    idle[b] += 1
            print(f"  {m:>2}:00 " + ", ".join(f"{b}->{t} {n}" for (b, t), n in target.most_common(8)) + " | idle " + ", ".join(f"{b} {n}" for b, n in idle.most_common(6)))


main()
