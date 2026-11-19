#!/usr/bin/env python3
"""Read a game's air patrol (H-MICRO-INTERCEPT, the hands' `air_patrol_<place>` move): the hands' log joined with the
game's tape.

    run/air_patrol.py <match dir> [--leash 3000] [--guard 1500]

From the hands' log (`jev-<ai>.jsonl`, or its `.zst`): each group on a patrol (a section a group), its members and
its place as each `call` line's `groups` say them, from when to when, the lane's word (`air_patrol` on a `pass` line: "scrambled N of its fighters at ...",
"all back over ... after N s") and the bounds as the picture said them. From the tape (`game.tape`, read through
`target/release/tape audit`): where every fighter of the group and every aircraft of his was each second, and what
was destroyed where. The tape names no attacker for an aircraft (it is destroyed when it has crashed, by nothing),
so an aircraft of his that went down is counted by where and when, not by who shot it. The engine uses a dead
unit's id again: in a game with hundreds of aircraft lost a new unit can stand as a dead member of a group until the
next call says the group again, so a lone far "furthest fighter" in such a game is to be checked against a trace.

It prints
  the patrol   the group, the place, from when, and the bounds the hands' words carried
  fighters     in the group while it patrolled (a fighter the player moved to another group is that group's from
               then on), lost (inside the leash and beyond it; those that went with our last
               commander, when the game destroys the side, apart), and his aircraft that went down in the game
  on the loop  with no scramble open: how far the furthest fighter was from the place (a fighter counts from the
               first second it is within 1,000 of the place: before that it is flying in from its plant)
  scrambles    one row each: the fighters sent at how many contacts how far out, how long until all were back, his
               aircraft that were inside the leash meanwhile and the nearest any came to the place while it still
               flew (a row's health above nothing: one shot down falls for some seconds), the furthest a
               fighter went, the fighter-seconds beyond the leash, fighters lost, his aircraft that went down
               inside the leash meanwhile (to the fighters or to our anti-air: the tape does not say), and what of
               ours his aircraft destroyed meanwhile
  unanswered   seconds his aircraft, still flying, were within the guard of the place with no scramble open, by
               episode, and whether our side saw them (the tape's sight flag)
  his bombs    everything of ours his aircraft destroyed in the game, how far from the place, and whether a scramble
               was open
"""
import bisect, collections, glob, json, math, os, re, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HOME = 1000.0
ID, DEF, TEAM, X, Z, HEALTH, SEEN = 0, 1, 2, 3, 4, 5, 15


def clock(frame):
    return f"{frame // 1800}:{frame // 30 % 60:02d}"


def lines(path):
    if path.endswith(".zst"):
        return subprocess.Popen(["zstd", "-dc", path], stdout=subprocess.PIPE, text=True).stdout
    return open(path)


def hands(match):
    """The patrols as the hands' log has them, a group each: {group: {"runs": [[from, to, place, (x, z)]] (to is
    None while the patrol is kept), "members": [(frame, {units})] as each call said them (the player moves fighters
    between groups), "said": [(frame, text)], "words": the picture's line}}."""
    logs = sorted(glob.glob(os.path.join(match, "jev-*.jsonl")) + glob.glob(os.path.join(match, "jev-*.jsonl.zst")))
    if not logs:
        sys.exit(f"no hands' log (jev-*.jsonl) in {match}")
    groups, seen = {}, set()
    entry = lambda name: groups.setdefault(name, {"runs": [], "members": [], "said": [], "words": None})
    for line in lines(logs[0]):
        if '"air_patrol"' not in line and '"t":"call"' not in line:
            continue
        r = json.loads(line)
        frame = r.get("f", 0)
        for text in r.get("air_patrol") or []:
            if text not in seen and text.startswith("group_"):
                seen.add(text)
                entry(text.split(" ", 1)[0][len("group_"):])["said"].append((frame, text))
        if r.get("t") != "call":
            continue
        for group in r.get("groups") or []:
            task = group.get("task") or {}
            on = task.get("kind") == "air_patrol"
            if not on and group["name"] not in groups:
                continue
            g = entry(group["name"])
            run = g["runs"][-1] if g["runs"] and g["runs"][-1][1] is None else None
            place = (task.get("place"), tuple(task.get("to") or (0, 0)))
            if run and (not on or (run[2], run[3]) != place):
                run[1] = frame
                run = None
            if on and run is None:
                g["runs"].append([frame, None, place[0], place[1]])
            members = frozenset(group.get("members") or [])
            if not g["members"] or g["members"][-1][1] != members:
                g["members"].append((frame, members))
        for actor_name, actor in ((r.get("state") or {}).get("actors") or {}).items():
            lane = actor.get("lane") if isinstance(actor, dict) else None
            if lane and "patrolling the air" in lane and actor_name.startswith("group_"):
                g = entry(actor_name[len("group_"):])
                g["words"] = g["words"] or lane
    return {name: g for name, g in groups.items() if g["runs"]}


def tape(match):
    path = os.path.join(match, "game.tape")
    if not os.path.exists(path):
        sys.exit(f"no game.tape in {match}: run/tape.py {match}")
    header, defs, samples, events, winners = None, {}, [], [], None
    for line in subprocess.Popen([os.path.join(REPO, "target/release/tape"), "audit", path, "-o", "-"], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True).stdout:
        r = json.loads(line)
        kind = r.get("t")
        if kind == "header":
            header = r
        elif kind == "def":
            defs[r["d"]["id"]] = r["d"]
        elif kind == "s":
            samples.append((r["f"], r["u"]))
        elif kind == "ev" and r.get("k") == "destroyed":
            events.append(r)
        elif kind == "over":
            winners = r.get("winners")
    return header, defs, samples, events, winners


def scrambles(said, end):
    """The lane's lines as windows: {from, to (the frame all were back, or the game's end), sent, contacts, out, open}."""
    out, current = [], None
    for frame, text in said:
        began = re.search(r"scrambled (?:(\d+) of its fighters )?at (\d+) aircraft of his.*?(\d+) from (?:it|\S+)$", text)
        back = re.search(r"back over .* after (\d+) s", text)
        if began:
            if current:
                current["to"] = frame - 1
                out.append(current)
            current = {"from": frame, "to": end, "sent": began.group(1), "contacts": int(began.group(2)), "out": int(began.group(3)), "open": True, "text": text}
        elif back and current:
            current.update(to=frame, open=False)
            out.append(current)
            current = None
    if current:
        out.append(current)
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    option = lambda name, default: float(sys.argv[sys.argv.index(name) + 1]) if name in sys.argv else default
    if not args:
        sys.exit(__doc__)
    match = args[0]
    if os.path.isdir(os.path.join(match, "00")):
        match = os.path.join(match, "00")
    leash, guard = option("--leash", 3000.0), option("--guard", 1500.0)

    groups = hands(match)
    if not groups:
        sys.exit("no group was on an air patrol in this game")
    game = tape(match)
    for index, (group, g) in enumerate(sorted(groups.items(), key=lambda item: item[1]["runs"][0][0])):
        if index:
            print("\n" + "=" * 20 + "\n")
        report(group, g, game, leash, guard)


def report(group, g, game, leash, guard):
    header, defs, samples, events, winners = game
    end = samples[-1][0]
    seat = next((s for s in header["seats"] if s["name"].startswith("WReason")), header["seats"][0])
    ours = seat["team"]
    # A side that loses its last commander is destroyed by the game: what dies from then on is not the patrol's loss.
    fallen = [e["f"] for e in events if e.get("team") == ours and defs.get(e["d"], {}).get("name", "").endswith("com")]
    fell = max(fallen) if fallen and winners is not None and seat["ally"] not in winners else None
    name = lambda d: defs.get(d, {}).get("human") or defs.get(d, {}).get("name") or str(d)
    air = lambda d: defs.get(d, {}).get("air") == 1

    # The patrol's place at a frame (none while the group is on another task) and the group's members at a frame.
    runs = [(a, end if b is None else b, place, to) for a, b, place, to in g["runs"]]
    def place_at(frame):
        return next(((index, to) for index, (a, b, _, to) in enumerate(runs) if a <= frame <= b), None)
    frames = [f for f, _ in g["members"]]
    def members_at(frame):
        index = bisect.bisect_right(frames, frame) - 1
        return g["members"][index][1] if index >= 0 else frozenset()
    print(f"the patrol: group_{group}; " + "; ".join(f"over {place} ({to[0]}, {to[1]}) from {clock(a)}" + ("" if b >= end else f" to {clock(b)}") for a, b, place, to in runs) + f"; the game ends {clock(end)}")
    if g["words"]:
        print(f"the hands' words: {g['words'][:330]}")

    windows = scrambles(g["said"], end)
    open_at = lambda frame: any(w["from"] <= frame <= w["to"] for w in windows)

    # Second by second: the fighters' distances from the place, his aircraft's.
    arrived, loop_far, loop_seconds, loop_beyond = set(), (0.0, 0), 0, 0
    for w in windows:
        w.update(far=(0.0, 0), beyond=0, his=collections.Counter(), his_ids=set(), nearest=None)
    unanswered = []  # (frame, [(type, distance, seen)])
    ever = set()
    for frame, units in samples:
        at = place_at(frame)
        if at is None:
            continue
        run, (px, pz) = at
        members = members_at(frame)
        window = next((w for w in windows if w["from"] <= frame <= w["to"]), None)
        for u in units:
            d = math.hypot(u[X] - px, u[Z] - pz)
            if u[TEAM] == ours and u[ID] in members:
                ever.add(u[ID])
                # A fighter counts from the first second it is over this place: before, it is flying in.
                if d <= HOME:
                    arrived.add((u[ID], run))
                if (u[ID], run) not in arrived:
                    continue
                if window:
                    if d > window["far"][0]:
                        window["far"] = (d, frame)
                    window["beyond"] += d > leash
                else:
                    loop_seconds += 1
                    loop_beyond += d > HOME + 700
                    if d > loop_far[0]:
                        loop_far = (d, frame)
            elif u[TEAM] != ours and air(u[DEF]) and u[HEALTH] > 0:
                if window and d <= leash:
                    if u[ID] not in window["his_ids"]:
                        window["his_ids"].add(u[ID])
                        window["his"][name(u[DEF])] += 1
                    window["nearest"] = d if window["nearest"] is None else min(window["nearest"], d)
                if not window and d <= guard:
                    if not unanswered or unanswered[-1][0] != frame:
                        unanswered.append((frame, []))
                    unanswered[-1][1].append((name(u[DEF]), d, u[SEEN] if len(u) > SEEN else None))

    # What was destroyed where, while the patrol was up.
    lost, downed, bombed, with_the_side = [], [], [], 0
    for e in events:
        mine = e.get("team") == ours and (e["u"] in members_at(e["f"]) or e["u"] in members_at(e["f"] - 60))
        if fell is not None and e["f"] >= fell:
            with_the_side += mine
            continue
        at = place_at(e["f"])
        if at is None:
            continue
        by = e.get("by_d")
        d = math.hypot(e["x"] - at[1][0], e["z"] - at[1][1])
        if mine:
            lost.append((e["f"], d))
        elif e.get("team") == ours and by and air(by) and e.get("by_team") != ours:
            bombed.append((e["f"], name(e["d"]), d, name(by)))
        elif e.get("team") != ours and air(e["d"]):
            downed.append((e["f"], name(e["d"]), d))
    tally = lambda items: ", ".join(f"{n} {k}" for k, n in collections.Counter(items).most_common()) or "none"
    print(
        f"fighters: {len(ever)} in the group while it patrolled, {len(lost)} lost ({sum(d <= leash for _, d in lost)} within {leash:.0f} of the place, {sum(d > leash for _, d in lost)} beyond)"
        + (f", {with_the_side} more gone with the side when our last commander fell at {clock(fell)}" if fell is not None else "") + "; "
        f"his aircraft down while the patrol was up: {len(downed)} ({tally(k for _, k, _ in downed)}), {sum(d <= leash for _, _, d in downed)} of them within {leash:.0f} of the place"
    )
    print(f"on the loop, no scramble open: {loop_seconds} fighter-seconds, the furthest a fighter was from the place {loop_far[0]:.0f} (at {clock(loop_far[1])}), {loop_beyond} fighter-seconds beyond {HOME + 700:.0f}")

    print()
    print(f"| scramble | fighters sent | contacts | out | until all back s | his aircraft inside {leash:.0f} meanwhile | nearest of his to the place, still flying | furthest fighter | fighter-seconds beyond {leash:.0f} | fighters lost | his aircraft down inside {leash:.0f} | ours his aircraft destroyed |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|")
    inside = lambda w, frame: w["from"] <= frame <= w["to"]
    for w in windows:
        span = f"{(w['to'] - w['from']) // 30}" + (" (open at the end)" if w["open"] else "")
        nearest = "-" if w["nearest"] is None else f"{w['nearest']:.0f}"
        print(
            f"| {clock(w['from'])} | {w['sent'] or 'all'} | {w['contacts']} | {w['out']} | {span} | {tally_counter(w['his'])} | {nearest} | {w['far'][0]:.0f} ({clock(w['far'][1])}) | {w['beyond']} | "
            f"{sum(inside(w, f) for f, _ in lost)} | {tally(k for f, k, d in downed if inside(w, f) and d <= leash)} | {tally(k for f, k, _, _ in bombed if inside(w, f))} |"
        )
    came = sum(1 for w in windows if w["his"])
    print(f"\n{len(windows)} scrambles, {sum((w['to'] - w['from']) // 30 for w in windows)} s in all; in {came} an aircraft of his came inside {leash:.0f} of the place")

    print(f"\nhis aircraft, still flying, within {guard:.0f} of the place with no scramble open:")
    episodes = []
    for frame, there in unanswered:
        if episodes and frame - episodes[-1]["to"] <= 90:
            episodes[-1]["to"] = frame
        else:
            episodes.append({"from": frame, "to": frame, "types": collections.Counter(), "nearest": 1e9, "seen": set()})
        for kind, d, seen in there:
            episodes[-1]["types"][kind] = max(episodes[-1]["types"][kind], sum(1 for k, _, _ in there if k == kind))
            episodes[-1]["nearest"] = min(episodes[-1]["nearest"], d)
            episodes[-1]["seen"].add(seen)
    for e in episodes:
        seen = "in sight" if 1 in e["seen"] or 3 in e["seen"] else "on radar" if 2 in e["seen"] else "not seen by us"
        print(f"  {clock(e['from'])} to {clock(e['to'])}: {tally_counter(e['types'])}, nearest {e['nearest']:.0f}, {seen}")
    if not episodes:
        print("  none")

    print("\nwhat of ours his aircraft destroyed while the patrol was up:")
    for f, what, d, by in bombed:
        print(f"  {clock(f)} {what}, {d:.0f} from the place, by a {by}{', a scramble open' if open_at(f) else ''}")
    if not bombed:
        print("  nothing")


def tally_counter(counter):
    return ", ".join(f"{n} {k}" for k, n in counter.most_common()) or "none"


if __name__ == "__main__":
    main()
