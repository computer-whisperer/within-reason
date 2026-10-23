#!/usr/bin/env python3
"""The live edge (docs/design/2026-10-21-brief-rework.md §11): what the next game of a series against the same opponent
on the same map is shown of the last ones. One entry a match: the report in plain numbers from the record and the truth
file, and the words of the player and the commander who sat in it (their notes and closing sentences, and the player's
debrief for the next player from run/debrief.py). No inference, no categories: the derived articles' tests run offline.

  run/live_edge.py close <match dir> [--no-debrief] [--store DIR]
      Write the entry into the store (default run/data/live-edge/) and append its row to the store's index.jsonl.
      The arena runs this when a flagship player match ends; run/human_game.sh after the bot leaves.
  run/live_edge.py text <match dir> [--no-debrief]
      Print the entry, write nothing.
  run/live_edge.py key <match dir>
      The opponent key of the match as the bot computes it from the hello: the enemy seats sorted (a person's name;
      an AI's short name, version and profile; the seat's bonus when it has one), joined with ", ".
  run/live_edge.py show --map NAME --key KEY [--days 14] [--n 3] [--store DIR]
      What a bot starting now would be given: the entries of that map and key within --days, newest first, at most --n.

The bot's side of the loop is crates/bot/src/strategist/live_edge.rs: it reads the store named by WITHIN_REASON_LIVE_EDGE
at session start and appends the entries to the player's and the commander's prompts.
"""
import glob
import json
import os
import re
import subprocess
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "run"))
import analyze_match  # noqa: E402

FPS = 30
DAYS = 14
MOST = 3
ARMY = {"army", "aircraft", "hover", "ship"}
ADVANCED_FACTORIES = {"armalab", "armavp", "armaap", "armasy", "coralab", "coravp", "coraap", "corasy", "legalab", "legavp", "legaap", "legasy"}
AIR_PLANTS = {"armap", "corap", "legap", "armplat", "corplat"}
SILOS = {"armsilo", "corsilo", "legsilo"}
ANTI_NUKES = {"armamd", "corfmd", "legabm", "armscab", "cormabm"}
TABLE_MINUTES = [2, 4, 6, 8, 10, 12, 14, 16, 20, 24, 28, 32, 36, 40, 45, 50, 60]


def clock(frame):
    s = int(frame) // FPS
    return f"{s // 60}:{s % 60:02d}"


def store_dir(args):
    """`--store DIR`, else the store the arena and the bot use: WITHIN_REASON_LIVE_EDGE when set, else run/data/live-edge."""
    if "--store" in args:
        return os.path.abspath(args[args.index("--store") + 1])
    return os.path.abspath(os.environ.get("WITHIN_REASON_LIVE_EDGE") or os.path.join(REPO, "run", "data", "live-edge"))


def match_directory(path):
    path = os.path.abspath(path)
    if not glob.glob(os.path.join(path, "record-*.jsonl")) and os.path.isdir(os.path.join(path, "00")):
        path = os.path.join(path, "00")
    return path


def handicaps(match_dir):
    """Team number -> bonus in per cent, from the start script beside the record (`Handicap=35` in `[TEAMn]`)."""
    out = {}
    try:
        text = open(os.path.join(match_dir, "script.txt"), errors="replace").read()
    except OSError:
        return out
    for m in re.finditer(r"\[TEAM(\d+)\]\s*\{([^}]*)\}", text, re.I | re.S):
        h = re.search(r"Handicap\s*=\s*(-?\d+)", m.group(2), re.I)
        if h:
            out[int(m.group(1))] = int(h.group(1))
    return out


def seat_name(controller):
    if isinstance(controller, dict):
        if "Person" in controller:
            return controller["Person"].get("name") or "a person"
        if "Ai" in controller:
            ai = controller["Ai"]
            parts = [ai.get("short_name") or ai.get("name") or "an AI"]
            if ai.get("version"):
                parts.append(ai["version"])
            if ai.get("profile"):
                parts.append(ai["profile"])
            return " ".join(parts)
    return "unknown"


def opponent_key(header, match_dir):
    """The enemy seats sorted, each a name with its bonus: the same string crates/bot/src/strategist/live_edge.rs
    builds from the hello (an AI's short name, version and profile; a person's name; `+N%` from the income multiplier)."""
    bonus = handicaps(match_dir)
    names = []
    for seat in header.get("seats") or []:
        if seat.get("ally_team") == header.get("ally_team") or seat.get("controller") == "Gaia":
            continue
        name = seat_name(seat.get("controller"))
        p = bonus.get(seat.get("team"), 0)
        names.append(name + (f" {p:+d}%" if p else ""))
    return ", ".join(sorted(names))


def our_seats(header):
    return [seat_name(s.get("controller")) for s in header.get("seats") or [] if s.get("ally_team") == header.get("ally_team") and s.get("controller") != "Gaia"]


def end_row(match_dir, record):
    for line in open(os.path.join(match_dir, record), errors="replace"):
        if line.startswith('{"t": "end"') or line.startswith('{"t":"end"'):
            try:
                return json.loads(line)
            except ValueError:
                pass
    return None


def centre(points):
    return (sum(p[0] for p in points) / len(points), sum(p[1] for p in points) / len(points)) if points else None


def halves(match):
    """forward_ours(x, z), forward_his(x, z): past the midpoint of the line between the sides' commanders at the start."""
    first = match.samples[0] if match.samples else None
    ours = his = None
    if first:
        defs = match.defs
        pts = [(u[2], u[3]) for u in first.get("own", []) + first.get("al", []) if 0 <= u[1] < len(defs) and defs[u[1]].get("class") == "commander"]
        ours = centre(pts) or centre([(u[2], u[3]) for u in first.get("own", [])])
    if match.truth:
        t = match.truth[0]
        pts = [(u[2], u[3]) for u in t.get("enemy", []) if match.cls(u[1]) == "commander"]
        his = centre(pts) or centre([(u[2], u[3]) for u in t.get("enemy", [])])
    if not ours or not his or ours == his:
        return None, None
    mx, mz = (ours[0] + his[0]) / 2, (ours[1] + his[1]) / 2
    ax, az = his[0] - ours[0], his[1] - ours[1]
    return (lambda x, z: (x - mx) * ax + (z - mz) * az > 0), (lambda x, z: (x - mx) * ax + (z - mz) * az < 0)


def first_in_half(match, forward_ours, forward_his):
    """(clock, unit) of our first finished soldier in his half, and of his first in ours."""
    ours = his = None
    if forward_ours:
        for s in match.samples:
            for u in s.get("own", []) + s.get("al", []):
                if 0 <= u[1] < len(match.defs):
                    d = match.defs[u[1]]
                    building = (u[5] & 1) if isinstance(u[5], int) and len(u) > 5 else 0
                    if d.get("class") in ARMY and not building and forward_ours(u[2], u[3]):
                        ours = (clock(s["f"]), d["name"])
                        break
            if ours:
                break
    if forward_his:
        for t in match.truth:
            for u in t.get("enemy", []):
                if match.cls(u[1]) in ARMY and not (len(u) > 5 and u[5]) and forward_his(u[2], u[3]):
                    his = (clock(t["f"]), u[1])
                    break
            if his:
                break
    return ours, his


def first_of(match, names):
    """(ours, his): the clock the first unit of these names was begun by each side."""
    ours = next((clock(e["f"]) for e in match.events if e.get("k") == "created" and e.get("d") is not None and 0 <= e["d"] < len(match.defs) and match.defs[e["d"]]["name"] in names), None)
    his = next((clock(t["f"]) for t in match.truth if any(u[1] in names for u in t.get("enemy", []))), None)
    return ours, his


def report(match, match_dir):
    """The game in plain numbers: lines of text and a table, from the record and the truth file."""
    lines = []
    forward_ours, forward_his = halves(match)
    truth = bool(match.truth)
    if not truth:
        lines.append("No truth file: nothing of his side is known beyond what our units saw, and his columns below are floors.")
    ours_in, his_in = first_in_half(match, forward_ours, forward_his)
    deaths = match.deaths()
    def first_death(side, test):
        d = next((d for d in deaths if d[1] == side and test(d[2])), None)
        return f"{clock(d[0])} ({d[2]} at {match.grid(d[3], d[4])})" if d else "never"
    still = lambda name: match.by_name.get(name, {}).get("speed", 1) == 0
    lines.append("**His clocks:** first soldier in our half " + (f"{his_in[0]} ({his_in[1]})" if his_in else "never")
                 + f"; first building of ours killed {first_death('ours', still)}; first extractor of ours killed {first_death('ours', lambda n: match.cls(n) == 'extractor')}")
    for label, names in (("advanced factory", ADVANCED_FACTORIES), ("air plant", AIR_PLANTS), ("silo", SILOS), ("anti-nuke", ANTI_NUKES)):
        o, h = first_of(match, names)
        lines.append(f"- {label}: his {h or 'never'}, ours {o or 'never'}")
    lines.append("**Our clocks:** first soldier in his half " + (f"{ours_in[0]} ({ours_in[1]})" if ours_in else "never")
                 + f"; first building of his killed {first_death('theirs', still)}; first extractor of his killed {first_death('theirs', lambda n: match.cls(n) == 'extractor')}")
    # the sides by minute
    curve = analyze_match.curves(match)
    by_minute = {r["minute"]: r for r in curve}
    get = lambda r, side, cls: (r.get(side) or {}).get(cls, [0, 0])
    def share_forward(frame, units, forward):
        if not forward:
            return None
        total = fwd = 0.0
        for u in units:
            if match.cls(u[1]) in ARMY and not ((u[5] & 1) if isinstance(u[5], int) and len(u) > 5 else 0):
                m = match.metal(u[1])
                total += m
                if forward(u[2], u[3]):
                    fwd += m
        return (fwd / total) if total else 0.0
    rows = ["| minute | extractors ours / his | our income | army metal ours / his | soldiers ours / his | of our army in his half | of his in ours |", "|---|---|---|---|---|---|---|"]
    for mn in TABLE_MINUTES:
        r = by_minute.get(mn)
        if not r:
            continue
        f = mn * 60 * FPS
        ours_units = [(u[0], match.defs[u[1]]["name"], u[2], u[3], u[4], u[5] if len(u) > 5 else 0) for u in min(match.samples, key=lambda s: abs(s["f"] - f)).get("own", []) if 0 <= u[1] < len(match.defs)]
        so = share_forward(f, ours_units, forward_ours)
        sh = share_forward(f, match.theirs_at(f), forward_his) if truth else None
        pct = lambda v: "-" if v is None else f"{v * 100:.0f}%"
        rows.append(f"| {mn}:00 | {get(r, 'ours', 'extractor')[0]} / {get(r, 'theirs', 'extractor')[0]} | {r['metal_income']:.0f} | "
                    f"{get(r, 'ours', 'army')[1]:,} / {get(r, 'theirs', 'army')[1]:,} | {get(r, 'ours', 'army')[0]} / {get(r, 'theirs', 'army')[0]} | {pct(so)} | {pct(sh)} |")
    lines.append("")
    lines.append("The sides by minute (his income is not recorded; his extractors are):")
    lines.extend(rows)
    # extractors lost
    def lost(side):
        ds = [d for d in deaths if d[1] == side and match.cls(d[2]) == "extractor"]
        if not ds:
            return "none"
        buckets = {}
        cells = {}
        for d in ds:
            b = int(d[0] // (4 * 60 * FPS)) * 4
            buckets[b] = buckets.get(b, 0) + 1
            c = match.grid(d[3], d[4])
            cells[c] = cells.get(c, 0) + 1
        top = ", ".join(f"{c} {n}" for c, n in sorted(cells.items(), key=lambda kv: -kv[1])[:4])
        when = ", ".join(f"{n} in {b}:00-{b + 4}:00" for b, n in sorted(buckets.items()))
        return f"{len(ds)} ({when}; most at {top})"
    lines.append("")
    lines.append(f"**Extractors lost:** ours {lost('ours')}; his {lost('theirs')}.")
    # fights
    fights = sorted(analyze_match.engagements(match), key=lambda e: -(e["value_lost"]["ours"] + e["value_lost"]["theirs"]))[:4]
    if fights:
        lines.append("**The largest fights** (metal lost ours / his): " + "; ".join(
            f"{clock(e['start'])} at {e['grid']} ({e['where']}) {e['value_lost']['ours']:,} / {e['value_lost']['theirs']:,}" for e in fights) + ".")
    return lines


def words_of(path, who, first=8, last=20, closings=8):
    """The notes and closing sentences of a session transcript (strategist-*.jsonl, commander.jsonl)."""
    notes, closes = [], []
    frame = 0
    for line in open(path, errors="replace"):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        kind = r.get("kind")
        if kind == "turn":
            frame = int(r.get("frame", 0))
        elif kind == "tool_call":
            calls = (r.get("arguments") or {}).get("calls") or [{"tool": r.get("tool"), "arguments": r.get("arguments", {})}]
            for c in calls:
                if c.get("tool") == "note":
                    text = (c.get("arguments") or {}).get("text", "")
                    if text:
                        notes.append((frame, " ".join(text.split())))
        elif kind == "result":
            text = (r.get("message") or {}).get("result")
            if isinstance(text, str) and text.strip():
                closes.append((frame, " ".join(text.split())))
    out = []
    if notes:
        shown = notes if len(notes) <= first + last else notes[:first] + [(None, f"... {len(notes) - first - last} notes left out ...")] + notes[-last:]
        out.append(f"**{who}'s notes, by game time** ({len(notes)} in the game):")
        out.extend(f"- {clock(f) if f is not None else ''} {t[:300]}" for f, t in shown)
    if closes:
        out.append("")
        out.append(f"**{who}'s closing sentences, the last {min(closings, len(closes))} of {len(closes)}:**")
        out.extend(f"- {clock(f)} {t[:400]}" for f, t in closes[-closings:])
    return out


def debrief(match_dir):
    """Runs run/debrief.py on the match and returns its answer (the file it wrote, without the two header lines)."""
    env = dict(os.environ)
    cmd = [sys.executable, os.path.join(REPO, "run", "debrief.py"), match_dir]
    if env.get("WITHIN_REASON_CLAUDE_CONFIG_DIR"):
        cmd += ["--claude-config-dir", env["WITHIN_REASON_CLAUDE_CONFIG_DIR"]]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=420)
    except subprocess.TimeoutExpired:
        return None, "the debrief did not answer within 420 s"
    written = os.path.join(REPO, "docs", "briefs", "inbox", f"{os.path.basename(os.path.dirname(match_dir))}.md")
    if proc.returncode != 0 or not os.path.exists(written):
        return None, f"the debrief failed ({proc.returncode}): {(proc.stderr or proc.stdout).strip()[-300:]}"
    text = open(written, errors="replace").read().split("\n", 3)
    body = text[3] if len(text) > 3 else ""
    return body.strip()[:6000], None


def entry(match_dir, with_debrief=True):
    """The entry's text and its index row."""
    match_dir = match_directory(match_dir)
    records = sorted(glob.glob(os.path.join(match_dir, "record-*.jsonl")))
    if not records:
        sys.exit(f"no record-*.jsonl in {match_dir}")
    match = analyze_match.Match(match_dir)
    header = match.header
    batch = {}
    batch_path = os.path.join(os.path.dirname(match_dir), "batch.json")
    if os.path.isfile(batch_path):
        try:
            batch = json.load(open(batch_path))
        except ValueError:
            pass
    label = batch.get("label") or os.path.basename(os.path.dirname(match_dir))
    key = opponent_key(header, match_dir)
    map_name = (header.get("map") or {}).get("name", "?")
    date = int(header.get("wall_start") or os.path.getmtime(records[0]))
    head = ""
    try:
        head = open(os.path.join(os.path.dirname(match_dir), "snapshot", "git-head.txt")).read().strip()[:12]
    except OSError:
        head = batch.get("commit", "")
    result_row = match.result
    end = end_row(match_dir, os.path.basename(records[0]))
    if result_row and isinstance(result_row.get("result"), dict):
        r = result_row["result"]
        outcome = r.get("outcome", "?")
        minutes = float(r.get("game_minutes") or 0.0)
        result_text = f"{outcome} after {minutes:.1f} minutes" + (" (called by the referee)" if r.get("called") else "")
    elif end:
        outcome = "unknown"
        minutes = end.get("f", 0) / (60.0 * FPS)
        result_text = f"result not recorded (a game with people: no referee); the record ends at {clock(end.get('f', 0))} with {end.get('own_units', '?')} of our units standing"
    else:
        outcome = "unknown"
        minutes = (match.samples[-1]["f"] / (60.0 * FPS)) if match.samples else 0.0
        result_text = f"result not recorded; the record ends at {clock(match.samples[-1]['f']) if match.samples else '0:00'}"
    when = time.strftime("%Y-%m-%d %H:%M", time.localtime(date))
    lines = [f"### {label}: {when}, {map_name}, against {key or 'nobody'}: {result_text}",
             f"Our side {header.get('side', '?')}, our seats {', '.join(our_seats(header)) or '?'}; build {head or '?'}. "
             "The report is a tool's reading of the record and the opponent's ground truth; the words are the player's and the commander's own.",
             "", "#### The report"]
    lines.extend(report(match, match_dir))
    lines.append("")
    lines.append("#### The words")
    transcripts = sorted(glob.glob(os.path.join(match_dir, "strategist-*.jsonl")))
    if transcripts:
        lines.extend(words_of(transcripts[0], "The player") or ["The player wrote no notes."])
    else:
        lines.append("No player sat in this game (the hands played alone).")
    commander = os.path.join(match_dir, "commander.jsonl")
    if os.path.isfile(commander):
        lines.append("")
        lines.extend(words_of(commander, "The commander") or ["The commander wrote no notes."])
    if transcripts and with_debrief:
        text, problem = debrief(match_dir)
        lines.append("")
        if text:
            lines.append("#### The player's debrief, written for you after the game")
            lines.append(text)
        else:
            lines.append(f"No debrief: {problem}.")
    row = {"id": f"{os.path.basename(os.path.dirname(match_dir))}/{os.path.basename(match_dir)}", "match": match_dir, "label": label,
           "date": date, "build": head, "map": map_name, "key": key, "result": outcome, "minutes": round(minutes, 1)}
    return "\n".join(lines) + "\n", row


def close(match_dir, store, with_debrief):
    text, row = entry(match_dir, with_debrief)
    os.makedirs(store, exist_ok=True)
    path = os.path.join(store, row["id"].replace("/", "__") + ".md")
    with open(path, "w") as f:
        f.write(text)
    row["file"] = path
    with open(os.path.join(store, "index.jsonl"), "a") as f:
        f.write(json.dumps(row) + "\n")
    shown = os.path.relpath(path, REPO) if path.startswith(REPO + os.sep) else path
    print(f"live edge: {row['id']} ({row['result']}, {row['minutes']} min) on {row['map']} against {row['key'] or 'nobody'} -> {shown}")


def show(store, map_name, key, days, most):
    """What the bot's selection gives: the same rule as crates/bot/src/strategist/live_edge.rs."""
    path = os.path.join(store, "index.jsonl")
    if not os.path.isfile(path):
        print(f"no index at {path}")
        return
    now = time.time()
    rows = []
    for line in open(path):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        if r.get("map") == map_name and r.get("key") == key and now - r.get("date", 0) <= days * 86400:
            rows.append(r)
    rows.sort(key=lambda r: -r["date"])
    rows = rows[:most]
    print(f"{len(rows)} entries for {map_name} against {key} within {days} days")
    for r in rows:
        print(open(r["file"], errors="replace").read())


def main():
    args = sys.argv[1:]
    if not args or args[0] not in ("close", "text", "key", "show"):
        sys.exit(__doc__)
    cmd = args[0]
    if cmd == "show":
        opt = lambda name, default: args[args.index(name) + 1] if name in args else default
        show(store_dir(args), opt("--map", ""), opt("--key", ""), float(opt("--days", DAYS)), int(opt("--n", MOST)))
        return
    if len(args) < 2 or args[1].startswith("--"):
        sys.exit(__doc__)
    match_dir = match_directory(args[1])
    if cmd == "key":
        match = analyze_match.Match(match_dir)
        print(opponent_key(match.header, match_dir))
    elif cmd == "text":
        text, row = entry(match_dir, "--no-debrief" not in args)
        print(text)
        print(json.dumps(row))
    else:
        close(match_dir, store_dir(args), "--no-debrief" not in args)


if __name__ == "__main__":
    main()
