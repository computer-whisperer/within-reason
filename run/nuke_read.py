#!/usr/bin/env python3
"""A game with a silo, read from its record: when the tier-2 plant, the first tier-2 extractor, the fusion, the silo and
our anti-nuke stood; the silo's frame against buildtime over the power standing on it (run/t2_frames.py's reading);
every round that came ready and every `fire_at` order with the clock; the energy store, income and usage while the
silo stockpiled; his anti-nukes, silos and advanced plants by the truth file (begun and stood; a truth entry's sixth
field is 1 while the building is under construction); what of his stood within 640 of each point fired at and was gone
40 s later, and how far the point was from his nearest standing anti-nuke against its 2,000 coverage; and the player's
`fire` calls with their answers (docs/design/2026-10-09-late-economy.md §10 (e)).

    run/nuke_read.py <match dir>...
"""
import json, math, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import match_read, t2_frames  # noqa: E402

BLAST = 640.0
# BAR's anti-nuke interceptor reaches this far (K-rules-the-silo-fires-on-an-attack-at-the-ground).
COVERAGE = 2000.0
HIS_LATE = ("armamd", "corfmd", "armsilo", "corsilo", "armalab", "coralab", "armavp", "coravp", "armaap", "coraap")


def main():
    for path in sys.argv[1:]:
        m = match_read.Match(path)
        record = match_read.first(m.dir, "record-")
        defs, created, finished, samples = t2_frames.frames_of(record)
        names = [d["name"] for d in defs]
        is_builder = [d.get("build_speed", 0) > 0 and (d.get("builds") or d.get("class") in ("builder", "commander")) for d in defs]
        reach = [d.get("build_distance", 0) + t2_frames.EDGE for d in defs]
        power = [d.get("build_speed", 0) for d in defs]
        rounds, fires, calls = [], [], []
        for r in match_read.rows(record):
            if r.get("t") == "ev" and r.get("k") == "round":
                rounds.append((r["f"], r["u"], r.get("n"), r.get("queued")))
            elif r.get("t") == "cmd":
                for c in r.get("c", []):
                    if c and c[0] == "fire_at":
                        fires.append((r["f"], c[1], c[2], c[3]))
        print(f"== {os.path.basename(os.path.dirname(m.dir) if os.path.basename(m.dir) == '00' else m.dir)}")
        firsts = {}
        for u, ev in created.items():
            name = names[ev["d"]]
            if u in finished and (name not in firsts or finished[u] < firsts[name][1]):
                firsts[name] = (ev["f"], finished[u], u)
        for name in sorted(firsts, key=lambda n: firsts[n][1]):
            begun, stood, _ = firsts[name]
            print(f"  first {name:10s} begun {match_read.clock(begun)} stood {match_read.clock(stood)} ({(stood - begun) / 30:.0f} s)")
        truth = match_read.first(m.dir, "truth-")
        truth_rows = list(match_read.rows(truth)) if truth else []
        his = {}
        for row in truth_rows:
            for e in row["enemy"]:
                if e[1] in HIS_LATE:
                    begun, stood, place = his.get(e[0], (row["f"], None, (e[2], e[3])))
                    if stood is None and len(e) > 5 and not e[5]:
                        stood = row["f"]
                    his[e[0]] = (begun, stood, place)
        for uid, (begun, stood, (x, z)) in sorted(his.items(), key=lambda kv: kv[1][0]):
            name = next(e[1] for row in truth_rows for e in row["enemy"] if e[0] == uid)
            print(f"  his {name:8s} {uid} at ({x}, {z}): begun {match_read.clock(begun)}, " + (f"stood {match_read.clock(stood)}" if stood else "never stood"))
        for u, ev in sorted(created.items(), key=lambda kv: kv[1]["f"]):
            if not names[ev["d"]].endswith("silo"):
                continue
            f0, f1 = ev["f"], finished.get(u)
            site = (ev.get("x", 0), ev.get("z", 0))
            seen = [s for s in samples if f0 < s["f"] <= (f1 or samples[-1]["f"])]
            on = [sum(power[od] for own in s.get("own", []) for oid, od, ox, oz, _h, flags in [own[:6]] if oid != u and is_builder[od] and not flags & 1 and math.hypot(ox - site[0], oz - site[1]) <= reach[od]) for s in seen]
            mean_power = sum(on) / max(1, len(on))
            empties = sum(1 for s in seen if s.get("m", [0])[0] < 20 or s.get("e", [0])[0] < 50)
            predicted = defs[ev["d"]]["build_time"] / mean_power if mean_power > 0 else float("inf")
            standing = f"stood {match_read.clock(f1)} after {(f1 - f0) / 30:.0f} s" if f1 else "never stood"
            print(f"  silo {u} at ({site[0]}, {site[1]}): begun {match_read.clock(f0)}, {standing}; mean power on it {mean_power:.0f} (buildtime / power {predicted:.0f} s); store empty in {100 * empties / max(1, len(seen)):.0f} % of its samples")
            if f1:
                after = [s for s in samples if f1 <= s["f"] <= f1 + 30 * 240]
                if after:
                    e = [s.get("e", [0, 0, 0, 0]) for s in after]
                    print(f"    the four minutes after it stood: energy store {min(x[0] for x in e):.0f} to {max(x[0] for x in e):.0f}, income {sum(x[1] for x in e) / len(e):.0f}, usage {sum(x[2] for x in e) / len(e):.0f} a second (mean)")
        kind = lambda u: names[created[u]["d"]] if u in created else "unit"
        for f, u, n, queued in rounds:
            print(f"  round ready {match_read.clock(f)}: {kind(u)} {u} holds {n}, {queued} stockpiling")
        for f, u, x, z in fires:
            print(f"  fire_at {match_read.clock(f)}: {kind(u)} {u} at ({x}, {z})")
            if truth_rows:
                before = max((r for r in truth_rows if r["f"] <= f), key=lambda r: r["f"], default=None)
                after = min((r for r in truth_rows if r["f"] >= f + 30 * 40), key=lambda r: r["f"], default=None)
                if before and after:
                    near = [e for e in before["enemy"] if math.hypot(e[2] - x, e[3] - z) <= BLAST]
                    gone = [e for e in near if not any(a[0] == e[0] for a in after["enemy"])]
                    kinds = lambda units: ", ".join(f"{units.count(k)} {k}" for k in sorted(set(units)))
                    print(f"    his within {BLAST:.0f} of the point at the order: {kinds([e[1] for e in near]) or 'nothing'}; gone 40 s later: {kinds([e[1] for e in gone]) or 'nothing'}")
                    antis = [(math.hypot(e[2] - x, e[3] - z), e) for e in before["enemy"] if e[1] in ("armamd", "corfmd") and (len(e) <= 5 or not e[5])]
                    if antis:
                        dist, e = min(antis)
                        print(f"    his nearest standing anti-nuke {e[0]} at ({e[2]}, {e[3]}) is {dist:.0f} from the point: " + ("inside its coverage" if dist <= COVERAGE else "outside its coverage"))
        strategist = match_read.first(m.dir, "strategist-")
        # A tool call carries no frame of its own: it belongs to the turn before it.
        turn_frame = 0
        for r in match_read.rows(strategist):
            if r.get("kind") == "turn":
                turn_frame = r.get("frame", turn_frame)
            if r.get("kind") != "tool_call":
                continue
            args = r.get("arguments") or {}
            if r.get("tool") == "fire":
                calls.append((turn_frame, args, r.get("result")))
            elif r.get("tool") == "orders":
                for c in args.get("calls", []):
                    if c.get("tool") == "fire":
                        calls.append((turn_frame, c.get("arguments"), r.get("result")))
        for frame, args, result in calls:
            print(f"  the player's fire call at {match_read.clock(frame or 0)}: {json.dumps(args)} -> {str(result)[:160]}")
        if not calls and not fires:
            print("  no fire call and no fire order")


if __name__ == "__main__":
    main()
