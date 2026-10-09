#!/usr/bin/env python3
"""Every tier-2 project frame of ours in recorded games against the build arithmetic the simulator uses: the frame's
recorded duration (its `created` to its `finished` event) beside the duration `buildtime / power` predicts from the
builders the samples show standing on it (a builder or construction turret of ours, not under construction, within its
build distance of the frame, in every sample while the frame was up), and the team's stock in those samples (a stall
shows as metal or energy near zero). The hands gave no repair orders in these games, so who helped is read from where
the builders stood, not from what they were told (docs/design/2026-10-09-late-economy.md §9, step 2).

    run/t2_frames.py <record.jsonl>...          one row a frame; a summary of recorded / predicted at the end
"""
import json, math, sys

PROJECTS = {"armavp", "armalab", "armaap", "armfus", "armafus", "armmmkr", "armsilo", "armamd", "armnanotc", "armadvsol",
            "coravp", "coralab", "coraap", "corfus", "corafus", "cormmkr", "corsilo", "corfmd", "cornanotc", "coradvsol"}
# A builder's reach is measured to the frame's edge: half the footprint (8 elmos a square) plus a little.
EDGE = 40.0


def frames_of(path):
    defs = None
    created, finished = {}, {}
    samples = []
    for line in open(path):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        t = r.get("t")
        if t == "header":
            defs = r["unit_defs"]
        elif t == "ev" and r.get("k") == "created" and isinstance(r.get("d"), int) and defs[r["d"]]["name"] in PROJECTS:
            created[r["u"]] = r
        elif t == "ev" and r.get("k") == "finished" and r.get("u") in created:
            finished[r["u"]] = r["f"]
        elif t == "s":
            samples.append(r)
    return defs, created, finished, samples


def main():
    rows = []
    for path in sys.argv[1:]:
        defs, created, finished, samples = frames_of(path)
        is_builder = [d.get("build_speed", 0) > 0 and (d.get("builds") or d.get("class") in ("builder", "commander")) for d in defs]
        reach = [d.get("build_distance", 0) + EDGE for d in defs]
        power = [d.get("build_speed", 0) for d in defs]
        for u, ev in sorted(created.items(), key=lambda kv: kv[1]["f"]):
            f0, f1 = ev["f"], finished.get(u)
            if f1 is None:
                continue
            name = defs[ev["d"]]["name"]
            site = (ev.get("x", 0), ev.get("z", 0))
            seen = [s for s in samples if f0 < s["f"] <= f1]
            if not seen:
                continue
            powers, stalls, counts = [], 0, []
            for s in seen:
                on = []
                for own in s.get("own", []):
                    oid, od, ox, oz, _health, flags = own[:6]
                    if oid == u or not is_builder[od] or flags & 1:
                        continue
                    if math.hypot(ox - site[0], oz - site[1]) <= reach[od]:
                        on.append(power[od])
                powers.append(sum(on))
                counts.append(len(on))
                m, e = s.get("m", [0, 0, 0, 0]), s.get("e", [0, 0, 0, 0])
                if m[0] < 20 or e[0] < 50:
                    stalls += 1
            mean_power = sum(powers) / len(powers)
            actual = (f1 - f0) / 30.0
            predicted = defs[ev["d"]]["build_time"] / mean_power if mean_power > 0 else float("inf")
            rows.append((path.split("/")[-3], name, f0 / 1800, actual, sum(counts) / len(counts), mean_power, predicted, stalls / len(seen)))
    print("| game | frame | begun (min) | recorded s | builders on it (mean) | power on it (mean) | buildtime / power s | recorded / predicted | samples with an empty stock |")
    print("|---|---|---|---|---|---|---|---|---|")
    for game, name, begun, actual, n, p, predicted, stall in rows:
        ratio = actual / predicted if predicted and predicted != float("inf") else float("nan")
        print(f"| {game} | {name} | {begun:.1f} | {actual:.0f} | {n:.1f} | {p:.0f} | {predicted:.0f} | {ratio:.2f} | {100 * stall:.0f} % |")
    good = [r for r in rows if r[6] != float("inf")]
    if good:
        ratios = sorted(r[3] / r[6] for r in good)
        print(f"\n{len(good)} frames: recorded over predicted, median {ratios[len(ratios) // 2]:.2f}, range {ratios[0]:.2f} to {ratios[-1]:.2f}")


if __name__ == "__main__":
    main()
