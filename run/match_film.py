#!/usr/bin/env python3
"""A film strip of a match: one map picture a minute, both sides drawn from the record and the truth file, so a
reader of text can look at where everything stood (a bot of bots/ writes no record: play its demo through
run/replay_match.py first, which writes record-<team>.jsonl and truth-<team>.jsonl for every team).

    run/match_film.py MATCH [--from M] [--to M] [--step M] [--out PNG] [--team T]

Ours blue, his red; a building a square (a plant larger, an extractor on its spot), a turret a triangle, a soldier a
dot, a builder a hollow ring, the commander a star. Metal spots grey. The A-H / 1-8 grid as the harness names cells.
Under each frame: the minute, each side's army metal, and the last minute's deaths of ours (where). Written
2026-10-20 to watch the distilled bot's games against Apex and BARb.
"""
import argparse
import glob
import json
import os
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

FPS = 30


def load(path):
    rows = []
    for line in open(path, errors="replace"):
        try:
            rows.append(json.loads(line))
        except ValueError:
            pass
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("match")
    ap.add_argument("--from", dest="start", type=int, default=2)
    ap.add_argument("--to", type=int, default=None)
    ap.add_argument("--step", type=int, default=2)
    ap.add_argument("--out", default=None)
    ap.add_argument("--team", type=int, default=None, help="which record-<team>.jsonl is ours (default: the lowest)")
    args = ap.parse_args()
    match = args.match
    records = sorted(glob.glob(os.path.join(match, "record-*.jsonl")))
    if not records:
        sys.exit(f"{match}: no record-*.jsonl")
    record = records[0] if args.team is None else os.path.join(match, f"record-{args.team}.jsonl")
    team = os.path.basename(record)[7:-6]
    rows = load(record)
    header = next(r for r in rows if r.get("t") == "header")
    # A record indexes unit defs by their position in the header's list (docs/harness/record-format.md).
    defs = {i: d for i, d in enumerate(header["unit_defs"])}
    samples = {r["f"]: r for r in rows if r.get("t") == "s"}
    deaths = [(r["f"], r) for r in rows if r.get("t") == "ev" and r.get("k") in ("destroyed", "unit_destroyed")]
    truth_path = os.path.join(match, f"truth-{team}.jsonl")
    truth = {r["f"]: r for r in load(truth_path)} if os.path.exists(truth_path) else {}
    width, height = header["map"]["width"], header["map"]["height"]
    spots = header.get("metal_spots", [])
    last = max(samples) // (60 * FPS)
    end = min(args.to, last) if args.to else last
    minutes = list(range(args.start, end + 1, args.step))
    if not minutes:
        sys.exit("no minutes")
    cols = min(5, len(minutes))
    rows_n = (len(minutes) + cols - 1) // cols
    aspect = height / width
    fig, axes = plt.subplots(rows_n, cols, figsize=(cols * 3.6, rows_n * 3.6 * aspect + rows_n * 0.5), squeeze=False)
    by_name = {d["name"]: d for d in defs.values()}

    def cls(d):
        return d.get("class", "other")

    def nearest_sample(frame, table):
        if not table:
            return None
        keys = [k for k in table if k <= frame]
        return table[max(keys)] if keys else None

    for ax_i, minute in enumerate(minutes):
        ax = axes[ax_i // cols][ax_i % cols]
        frame = minute * 60 * FPS
        s = nearest_sample(frame, samples)
        t = nearest_sample(frame, truth)
        ax.set_xlim(0, width)
        ax.set_ylim(height, 0)
        ax.set_aspect("equal")
        ax.set_xticks([])
        ax.set_yticks([])
        # The grid: 8 columns A-H, 8 rows 1-8.
        for i in range(1, 8):
            ax.axvline(i * width / 8, color="#dddddd", lw=0.4)
            ax.axhline(i * height / 8, color="#dddddd", lw=0.4)
        for i in range(8):
            ax.text((i + 0.5) * width / 8, height * 0.02, "ABCDEFGH"[i], ha="center", va="top", fontsize=5, color="#999999")
            ax.text(width * 0.01, (i + 0.5) * height / 8, str(i + 1), ha="left", va="center", fontsize=5, color="#999999")
        if spots:
            ax.scatter([p[0] for p in spots], [p[1] for p in spots], s=4, c="#bbbbbb", marker="o", linewidths=0)
        our_army = his_army = 0.0
        if s:
            for u in s.get("own", []):
                d = defs.get(u[1], {})
                x, z, flags = u[2], u[3], u[5] if len(u) > 5 else 0
                being_built = flags & 1
                c = cls(d)
                col = "#1f5fbf" if not being_built else "#9fbbe8"
                if c == "factory":
                    ax.scatter([x], [z], s=34, c=col, marker="s", linewidths=0)
                elif c == "extractor":
                    ax.scatter([x], [z], s=10, c=col, marker="s", linewidths=0)
                elif c == "turret":
                    ax.scatter([x], [z], s=14, c=col, marker="^", linewidths=0)
                elif c == "building":
                    ax.scatter([x], [z], s=6, c=col, marker="s", linewidths=0)
                elif c == "commander":
                    ax.scatter([x], [z], s=40, c=col, marker="*", linewidths=0)
                elif c == "builder":
                    ax.scatter([x], [z], s=12, facecolors="none", edgecolors=col, marker="o", linewidths=0.7)
                else:
                    ax.scatter([x], [z], s=7, c=col, marker="o", linewidths=0)
                    if not being_built:
                        our_army += d.get("metal", 0)
        his_rows = t.get("enemy", []) if t else (s.get("en", []) if s else [])
        for u in his_rows:
            if t:
                name, x, z, built = u[1], u[2], u[3], u[5]
                d = by_name.get(name, {})
            else:
                d = defs.get(u[1], {})
                x, z, built = u[2], u[3], (u[5] if len(u) > 5 else 0)
            c = cls(d)
            col = "#c0392b" if not built else "#eaa39a"
            if c == "factory":
                ax.scatter([x], [z], s=34, c=col, marker="s", linewidths=0)
            elif c == "extractor":
                ax.scatter([x], [z], s=10, c=col, marker="s", linewidths=0)
            elif c == "turret":
                ax.scatter([x], [z], s=14, c=col, marker="^", linewidths=0)
            elif c == "building":
                ax.scatter([x], [z], s=6, c=col, marker="s", linewidths=0)
            elif c == "commander":
                ax.scatter([x], [z], s=40, c=col, marker="*", linewidths=0)
            elif c == "builder":
                ax.scatter([x], [z], s=12, facecolors="none", edgecolors=col, marker="o", linewidths=0.7)
            else:
                ax.scatter([x], [z], s=7, c=col, marker="o", linewidths=0)
                if not built:
                    his_army += d.get("metal", 0)
        # Deaths of ours in the last minute, as small crosses.
        lost = 0
        for f, ev in deaths:
            if frame - 60 * FPS < f <= frame and "x" in ev:
                ax.scatter([ev["x"]], [ev["z"]], s=10, c="#1f5fbf", marker="x", linewidths=0.6)
                lost += 1
        ax.set_title(f"{minute}:00  army {our_army:.0f} / {his_army:.0f}" + (f"  lost {lost}" if lost else ""), fontsize=7)
    for ax_i in range(len(minutes), rows_n * cols):
        axes[ax_i // cols][ax_i % cols].axis("off")
    out = args.out or os.path.join(match, "film.png")
    fig.tight_layout(pad=0.3)
    fig.savefig(out, dpi=130)
    print(out)


main()
