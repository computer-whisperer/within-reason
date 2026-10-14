#!/usr/bin/env python3
"""Builder priority in the pro replays: the hand toggles of the game's low/high priority command, read from the demo's
packet stream and joined to the replay records for what was toggled and in what state (the study
docs/studies/2026-10-13-builder-priority.md).

    run/replays/priority.py demo <demo.sdfz>...                 the toggles of each demo by frame: by hand and by the widget
    run/replays/priority.py catalog [--manifest M] [--out DIR]  every kept replay with records: a row per hand toggle with
                                                                its context, and the tallies (default DIR docs/studies/2026-10-13-builder-priority)
    run/replays/priority.py stalls [--manifest M]               metal stalls in the records: which high-priority builders' targets
                                                                progressed, by the builders' creation order (K-mech-stall-order-is-creation-order)

The game's gadget (unit_builder_priority.lua) takes CMD_PRIORITY 34571 with one param, 0 low or 1 high, on commanders,
constructors, factories and construction turrets, and consumes it (AllowCommand returns false), so the widget callin
UnitCommand never sees it and the records cannot carry it: the demo stream is the only place it is written. A click on
the order menu's Low Prio / High Prio button goes out as NETMSG_COMMAND on the player's current NETMSG_SELECT; the
stock widget unit_builder_priority.lua (labs and turrets low at creation and again when finished, constructors high)
goes out through Spring.GiveOrderToUnit as NETMSG_AICOMMANDS with aiID 255, so the two are told apart by packet type.
"""
import collections, glob, gzip, json, os, re, struct, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CMD_PRIORITY = 34571
FPS = 30


def clock(frame):
    return f"{frame // (60 * FPS)}:{(frame % (60 * FPS)) // FPS:02d}"


# ---------------------------------------------------------------- the demo stream

def parse_demo(path):
    """The demo's players (by player number: name, team, spectator) and every priority command: (frame, player, source,
    unit ids, value) with source "hand" (NETMSG_COMMAND on the selection) or "widget" (NETMSG_AICOMMAND(S) from LuaUI)."""
    data = gzip.open(path).read()
    assert data[:15] == b"spring demofile", path
    version, header_size = struct.unpack_from("<ii", data, 16)
    script_size, stream_size = struct.unpack_from("<ii", data, 16 + 8 + 256 + 16 + 8)
    script = data[header_size:header_size + script_size].decode("utf-8", "replace")
    players = {}
    for m in re.finditer(r"\[player(\d+)\]\s*\{([^}]*)\}", script, re.I | re.S):
        body = dict(kv.strip().split("=", 1) for kv in m.group(2).split(";") if "=" in kv)
        players[int(m.group(1))] = {"name": body.get("name"), "team": int(body["team"]) if "team" in body else None, "spectator": body.get("spectator") == "1"}
    pos = header_size + script_size
    end = pos + stream_size if stream_size else len(data)
    frame = 0
    selection = collections.defaultdict(list)
    events = []
    while pos + 8 <= end:
        _, length = struct.unpack_from("<fI", data, pos)
        pos += 8
        pkt = data[pos:pos + length]
        pos += length
        if not pkt:
            continue
        mid = pkt[0]
        if mid == 2:  # NEWFRAME
            frame += 1
        elif mid == 1:  # KEYFRAME carries the frame number
            frame = struct.unpack_from("<i", pkt, 1)[0]
        elif mid == 12:  # SELECT: the player's selection, which the next COMMAND applies to
            size, player = struct.unpack_from("<HB", pkt, 1)
            selection[player] = list(struct.unpack_from("<%dh" % ((size - 4) // 2), pkt, 4))
        elif mid == 11:  # COMMAND
            size, player, cmd, timeout, opts, nparams = struct.unpack_from("<HBiiBI", pkt, 1)
            if cmd == CMD_PRIORITY:
                params = struct.unpack_from("<%df" % nparams, pkt, 17)  # id, size, player, cmd, timeout, options, count
                events.append((frame, player, "hand", list(selection[player]), int(params[0]) if params else None))
        elif mid in (14, 76):  # AICOMMAND, AICOMMAND_TRACKED
            size, player, ai, aiteam, unit, cmd, timeout, opts, nparams = struct.unpack_from("<HBBBhiiBI", pkt, 1)
            if cmd == CMD_PRIORITY:
                params = struct.unpack_from("<%df" % nparams, pkt, 21 + (4 if mid == 76 else 0))  # id, size, player, ai, team, unit, cmd, timeout, options, count (, ai command id)
                events.append((frame, player, "widget" if ai == 255 else "ai%d" % ai, [unit], int(params[0]) if params else None))
        elif mid == 15:  # AICOMMANDS: LuaUI's batch form
            size, player, ai, pairwise, same_cmd, same_opt, same_psize, ucount = struct.unpack_from("<HBBBIBHh", pkt, 1)
            off = 15
            units = list(struct.unpack_from("<%dh" % ucount, pkt, off))
            off += 2 * ucount
            ccount = struct.unpack_from("<h", pkt, off)[0]
            off += 2
            for c in range(ccount):
                cmd = same_cmd
                if cmd == 0:
                    cmd = struct.unpack_from("<i", pkt, off)[0]
                    off += 4
                if same_opt == 0xFF:
                    off += 1
                pc = same_psize
                if pc == 0xFFFF:
                    pc = struct.unpack_from("<H", pkt, off)[0]
                    off += 2
                params = struct.unpack_from("<%df" % pc, pkt, off)
                off += 4 * pc
                if cmd == CMD_PRIORITY:
                    events.append((frame, player, "widget", [units[c]] if pairwise else list(units), int(params[0]) if params else None))
    return players, events, frame


# ---------------------------------------------------------------- the records

def builder_kind(d):
    """commander, lab, turret (a construction turret), con (a mobile constructor), or none (a unit without the
    command: a box-selection sends it to tanks and extractors too, and the gadget ignores them), by the def row."""
    if (d.get("build_speed") or 0) <= 0:
        return "none"
    if d["class"] == "commander":
        return "commander"
    if d["class"] == "factory":
        return "lab"
    if (d.get("speed") or 0) > 0:
        return "con"
    return "turret"


ENERGY = ("solar", "wind", "advsol", "fus", "geo", "tide", "mmkr", "makr", "estor", "uwes")
EXTRACTOR = ("mex", "moho", "exp", "uwmex", "uwmme", "mstor", "uwms")
DEFENCE = ("llt", "beamer", "hllt", "pulv", "guard", "dragon", "hlt", "claw", "maw", "rl", "ferret", "madsam", "vulcan", "chainsaw", "mercury", "screamer", "pb", "anni", "doom", "toaster", "amb", "tl", "ambusher")


def job(rec, building_def, last_cmd):
    """What the toggled unit was doing: assist (guarding or repairing another unit, or building a factory's mobile
    unit), energy, extractor, defence, nano, radar, lab, other building, or nothing."""
    if last_cmd and (last_cmd.startswith("guard") or last_cmd.startswith("repair")):
        return "assist"
    name = rec.name(building_def) if building_def is not None else None
    if not name or name == "?":
        return "nothing"
    d = rec.defs[building_def]
    if (d.get("speed") or 0) > 0:
        return "assist"
    core = name[3:]
    if any(core.startswith(x) for x in EXTRACTOR):
        return "extractor"
    if any(core.startswith(x) for x in ENERGY):
        return "energy"
    if core.startswith("nanotc"):
        return "nano"
    if core.startswith("rad") or core.startswith("arad") or core.startswith("jamm"):
        return "radar"
    if d["class"] == "factory":
        return "lab"
    if d["class"] == "turret" or any(core.startswith(x) for x in DEFENCE):
        return "defence"
    return "building"


class Record:
    def __init__(self, path):
        self.path = path
        lines = open(path).readlines()
        self.header = json.loads(lines[0])
        self.defs = self.header["unit_defs"]
        self.created, self.finished, self.destroyed, self.def_of = {}, {}, {}, {}
        self.samples = []
        self.cmds = collections.defaultdict(list)
        for line in lines[1:]:
            try:
                o = json.loads(line)
            except ValueError:
                continue
            t = o.get("t")
            if t == "ev":
                u = o.get("u")
                if o["k"] == "created":
                    self.created[u] = o["f"]
                    self.def_of[u] = o["d"]
                elif o["k"] == "finished":
                    self.finished[u] = o["f"]
                    self.def_of.setdefault(u, o["d"])
                elif o["k"] == "destroyed":
                    self.destroyed[u] = o["f"]
            elif t == "s":
                own = {r[0]: r for r in o.get("own", [])}
                bp = {r[0]: r for r in o.get("bp", [])}
                for r in o.get("own", []):
                    self.def_of.setdefault(r[0], r[1])
                self.samples.append({"f": o["f"], "m": o["m"], "e": o["e"], "own": own, "bp": bp})
            elif t == "cmd":
                for row in o.get("c", []):
                    if len(row) > 1 and isinstance(row[1], int):
                        self.cmds[row[1]].append((o["f"], row))

    def sample_at(self, frame, after=False):
        """The last sample at or before the frame (or the first at or after it)."""
        if after:
            for s in self.samples:
                if s["f"] >= frame:
                    return s
            return self.samples[-1] if self.samples else None
        last = None
        for s in self.samples:
            if s["f"] > frame:
                break
            last = s
        return last

    def name(self, d):
        return self.defs[d]["name"] if d is not None and 0 <= d < len(self.defs) else "?"

    def kind(self, u):
        d = self.def_of.get(u)
        return builder_kind(self.defs[d]) if d is not None else "?"

    def describe_cmd(self, row):
        verb = row[0]
        if verb == "build":
            return f"build {self.name(row[2])}" if len(row) > 2 else "build"
        if verb in ("guard", "repair") and len(row) > 2:
            return f"{verb} {self.name(self.def_of.get(row[2], None))}"
        return verb


def match_dir_for(row):
    """The row's replay directory: the manifest's, or the newest finished `<time>-replay-<id>` (run/replays/replay.py
    writes the manifest in submission order, so a running batch's finished games may not be in it yet)."""
    m = row.get("match")
    if m and os.path.exists(os.path.join(REPO, m, "record-0.jsonl")):
        return os.path.join(REPO, m)
    done = sorted(d for d in glob.glob(os.path.join(REPO, "run", "matches", "*-replay-" + row["id"])) if os.path.exists(os.path.join(d, "card.json")))
    return done[-1] if done else None


def context(rec, unit, frame):
    """What the unit was and was doing at the frame, and the team's resources then."""
    s = rec.sample_at(frame)
    d = rec.def_of.get(unit)
    kind = rec.kind(unit)
    own = s["own"].get(unit) if s else None
    bp = s["bp"].get(unit) if s else None
    target = bp[2] if bp else 0
    target_def = rec.def_of.get(target) if target else None
    target_pct = s["own"][target][4] if s and target in s["own"] else None
    before = [c for c in rec.cmds.get(unit, []) if c[0] <= frame]
    last_cmd = rec.describe_cmd(before[-1][1]) if before else None
    m, e = (s["m"], s["e"]) if s else (None, None)
    others = []
    if s:
        for ou, (oid, oprio, otarget) in sorted(s["bp"].items()):
            if ou != unit:
                others.append([rec.kind(ou), oprio, rec.name(rec.def_of.get(otarget)) if otarget else None])
    return {
        "def": rec.name(d), "kind": kind, "job": job(rec, target_def, last_cmd), "others": others,
        "age_s": round((frame - rec.created[unit]) / FPS) if unit in rec.created else None,
        "being_built": bool(own and own[5] & 1) if own else None,
        "prio_before": bp[1] if bp else None,
        "building": rec.name(target_def) if target else None, "building_pct": target_pct,
        "last_cmd": last_cmd,
        "metal": [round(x) for x in m] if m else None, "energy": [round(x) for x in e] if e else None,
    }


def after(rec, unit, frame, seconds):
    """The team's metal and the build target's progress `seconds` later."""
    s = rec.sample_at(frame + seconds * FPS, after=True)
    if not s:
        return None
    bp = s["bp"].get(unit)
    target = bp[2] if bp else 0
    return {"metal": [round(x) for x in s["m"]], "energy": [round(x) for x in s["e"]], "building": rec.name(rec.def_of.get(target)) if target else None,
            "building_pct": s["own"][target][4] if target in s["own"] else None, "prio": bp[1] if bp else None}


def catalog(manifest, out_dir):
    rows = [json.loads(l) for l in open(manifest) if l.strip()]
    toggles, defaults, games = [], [], []
    for row in rows:
        if not (row.get("keep") and row.get("demo")):
            continue
        match = match_dir_for(row)
        if not match:
            continue
        players, events, frames = parse_demo(os.path.join(REPO, row["demo"]))
        recs = {}
        for path in glob.glob(os.path.join(match, "record-*.jsonl")):
            team = int(os.path.basename(path)[7:-6])
            recs[team] = Record(path)
        os_of = {p["name"]: p.get("os") for p in row["players"]}
        game = {"id": row["id"], "map": row["map_script"], "minutes": round(row["duration_ms"] / 60000, 1), "players": {}, "hand": 0}
        # the widget's defaults per player: the first widget toggle of each builder, by kind
        first_widget = {}
        for f, p, src, units, value in events:
            if src != "widget":
                continue
            team = players.get(p, {}).get("team")
            rec = recs.get(team)
            if rec is None:
                continue
            for u in units:
                if (team, u) not in first_widget:
                    first_widget[(team, u)] = (rec.kind(u), value)
        by_player = collections.defaultdict(lambda: collections.defaultdict(collections.Counter))
        for (team, u), (kind, value) in first_widget.items():
            by_player[team][kind][value] += 1
        for p, info in players.items():
            if info["spectator"] or info["team"] not in recs:
                continue
            kinds = {k: dict(v) for k, v in by_player[info["team"]].items()}
            defaults.append({"game": row["id"], "player": info["name"], "os": os_of.get(info["name"]), "team": info["team"], "widget_first_values": kinds})
            game["players"][info["name"]] = {"team": info["team"], "os": os_of.get(info["name"]), "hand": 0}
        for f, p, src, units, value in events:
            if src != "hand":
                continue
            info = players.get(p, {})
            team = info.get("team")
            rec = recs.get(team)
            if rec is None:
                continue
            game["hand"] += 1
            if info.get("name") in game["players"]:
                game["players"][info["name"]]["hand"] += 1
            later = [e for e in events if e[2] == "hand" and e[0] > f and e[1] == p]
            for u in units:
                if rec.kind(u) == "none":
                    game["not_builders"] = game.get("not_builders", 0) + 1
                    continue
                revert = next((e[0] for e in later if u in e[3] and e[4] != value), None)
                toggles.append({
                    "game": row["id"], "map": row["map_script"], "player": info.get("name"), "os": os_of.get(info.get("name")), "team": team,
                    "frame": f, "clock": clock(f), "unit": u, "to": "low" if value == 0 else "high", "selected": len(units),
                    **context(rec, u, f),
                    "after_30s": after(rec, u, f, 30), "after_60s": after(rec, u, f, 60),
                    "reverted_at": clock(revert) if revert is not None else None,
                    "reverted_after_s": round((revert - f) / FPS) if revert is not None else None,
                })
        games.append(game)
    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, "toggles.jsonl"), "w") as out:
        for t in toggles:
            out.write(json.dumps(t) + "\n")
    with open(os.path.join(out_dir, "widget-defaults.jsonl"), "w") as out:
        for d in defaults:
            out.write(json.dumps(d) + "\n")
    with open(os.path.join(out_dir, "games.jsonl"), "w") as out:
        for g in games:
            out.write(json.dumps(g) + "\n")
    tallies(toggles, defaults, games, out_dir)


def tallies(toggles, defaults, games, out_dir):
    lines = []
    say = lines.append
    say(f"# Builder priority toggles by hand in {len(games)} replayed games ({sum(g['hand'] for g in games)} toggle commands, {len(toggles)} unit-toggles)")
    say("")
    say(f"Games with any hand toggle: {sum(1 for g in games if g['hand'])} of {len(games)}.")
    say("")
    say("## By kind and direction (unit-toggles)")
    c = collections.Counter((t["kind"], t["to"]) for t in toggles)
    say("| kind | to low | to high |")
    say("|---|---|---|")
    for kind in ("commander", "con", "lab", "turret", "?"):
        if c[(kind, "low")] or c[(kind, "high")]:
            say(f"| {kind} | {c[(kind, 'low')]} | {c[(kind, 'high')]} |")
    say("")
    say("## By kind, direction and the toggled unit's job at the time (unit-toggles)")
    say(f"(Units without the command that were in the same selection, ignored by the gadget: {sum(g.get('not_builders', 0) for g in games)}.)")
    say("")
    jobs = ("assist", "energy", "extractor", "defence", "nano", "radar", "lab", "building", "nothing")
    say("| kind -> to | " + " | ".join(jobs) + " | all |")
    say("|---|" + "---|" * (len(jobs) + 1))
    cj = collections.Counter((t["kind"], t["to"], t["job"]) for t in toggles)
    for kind in ("commander", "con", "lab", "turret"):
        for to in ("low", "high"):
            row = [cj[(kind, to, j)] for j in jobs]
            if sum(row):
                say(f"| {kind} -> {to} | " + " | ".join(str(n) for n in row) + f" | {sum(row)} |")
    say("")
    say("## By player")
    say("| player | OS | games | hand toggles | to low | to high | kinds |")
    say("|---|---|---|---|---|---|---|")
    per = collections.defaultdict(lambda: {"games": 0, "os": None, "low": 0, "high": 0, "kinds": collections.Counter()})
    for g in games:
        for name, info in g["players"].items():
            per[name]["games"] += 1
            per[name]["os"] = info["os"]
    for t in toggles:
        per[t["player"]][t["to"]] += 1
        per[t["player"]]["kinds"][f"{t['kind']}->{t['to']}"] += 1
    for name, v in sorted(per.items(), key=lambda kv: -(kv[1]["low"] + kv[1]["high"])):
        if v["low"] + v["high"]:
            say(f"| {name} | {v['os']} | {v['games']} | {v['low'] + v['high']} | {v['low']} | {v['high']} | {', '.join(f'{k} {n}' for k, n in v['kinds'].most_common())} |")
    say("")
    say("## When (game minute of the unit-toggles)")
    mins = collections.Counter(min(int(t["frame"] / (60 * FPS)), 20) for t in toggles)
    say(", ".join(f"{m}: {mins[m]}" for m in sorted(mins)))
    say("")
    say("## The widget's defaults (the first automatic toggle of each builder kind, per player and game)")
    say("The stock widget sends labs and turrets 0 (low) and constructors 1 (high) at creation; a player who changed its settings shows other values.")
    say("")
    say("| player | game | lab | turret | con |")
    say("|---|---|---|---|---|")
    for d in defaults:
        k = d["widget_first_values"]
        def show(kind):
            v = k.get(kind, {})
            return " ".join(f"{val}x{n}" for val, n in sorted(v.items())) or "-"
        say(f"| {d['player']} | {d['game'][:8]} | {show('lab')} | {show('turret')} | {show('con')} |")
    say("")
    say("## Every hand toggle")
    say("| game | clock | player | unit | kind | to | job | age | was building (pct) | last order | the other builders | metal cur/inc/use/store | energy cur/inc/use/store | prio before | 30 s later: building (pct), metal | reverted |")
    say("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for t in sorted(toggles, key=lambda t: (t["game"], t["frame"])):
        a = t["after_30s"] or {}
        others = ", ".join(f"{k}{'' if p != 0 else ' low'}: {b or 'idle'}" for k, p, b in t.get("others", []))
        say(f"| {t['game'][:8]} | {t['clock']} | {t['player']} | {t['def']} | {t['kind']} | {t['to']} | {t['job']} | {t['age_s']} s | {t['building'] or '-'} ({t['building_pct']}) | {t['last_cmd'] or '-'} | {others} | "
            f"{'/'.join(map(str, t['metal'])) if t['metal'] else '-'} | {'/'.join(map(str, t['energy'])) if t['energy'] else '-'} | {t['prio_before']} | "
            f"{a.get('building') or '-'} ({a.get('building_pct')}), {a.get('metal', ['-'])[0]} | {t['reverted_at'] or '-'} |")
    open(os.path.join(out_dir, "tallies.md"), "w").write("\n".join(lines) + "\n")
    print("\n".join(lines[:40]))
    print(f"... written to {out_dir}")


def stalls(manifest):
    """The engine's stall rule watched in the pro records: at every sample where a team's metal is under 2 % of its
    storage and it is spending, take the high-priority builders (priority 1 or never set) that are the only builder on
    their being-built target; a target progressed if its health rose by the next sample. Among samples where some of
    those targets progressed and some did not, count how often the oldest builder's (by creation frame; a unit alive at
    frame 0 is oldest) progressed and the youngest's did not, and the reverse. Also the throttle: every sample where a
    low-priority builder is the only builder on a nanoframe, whether the nanoframe progressed by the next sample, split
    by the team's metal (under 2 % of storage, 2-20 %, over 20 %), by builder kind; and the throughput of lone builders
    by kind, priority and metal band against the full speed of the same builder type on the same target type (its
    median gain with metal over 20 %). Samples with energy under 20 % of storage are left out of all of it."""
    rows = [json.loads(l) for l in open(manifest) if l.strip()]
    stall_samples = mixed = oldest_went = youngest_went = both = 0
    throttle = collections.defaultdict(collections.Counter)  # (kind, metal band) -> {progressed: n}
    deltas = collections.defaultdict(list)  # (builder def, target def, band, prio, kind) -> health % gained per sample
    for row in rows:
        match = match_dir_for(row) if row.get("keep") and row.get("demo") else None
        if not match:
            continue
        for path in glob.glob(os.path.join(match, "record-*.jsonl")):
            rec = Record(path)
            for i in range(len(rec.samples) - 1):
                s, nxt = rec.samples[i], rec.samples[i + 1]
                if not s["bp"] or s["m"][3] <= 0:
                    continue
                if s["e"][3] <= 0 or s["e"][0] < 0.2 * s["e"][3]:
                    continue  # an energy stall throttles every builder alike: keep to metal stalls
                targets_all = collections.Counter(bp[2] for bp in s["bp"].values() if bp[2])
                band = "under 2 %" if s["m"][0] < 0.02 * s["m"][3] else ("2-20 %" if s["m"][0] < 0.2 * s["m"][3] else "over 20 %")
                for u, (uid, prio, target) in s["bp"].items():
                    if not target or targets_all[target] != 1:
                        continue
                    own, own_next = s["own"].get(target), nxt["own"].get(target)
                    if not own or not own_next or not (own[5] & 1) or nxt["bp"].get(u, (0, 0, 0))[2] != target:
                        continue
                    delta = own_next[4] - own[4]
                    if prio == 0:
                        throttle[(rec.kind(u), band)][delta > 0] += 1
                    deltas[(rec.def_of.get(u), rec.def_of.get(target), band, "low" if prio == 0 else "high", rec.kind(u))].append(delta)
                if s["m"][0] >= 0.02 * s["m"][3] or s["m"][2] <= 0:
                    continue
                stall_samples += 1
                targets = collections.Counter(bp[2] for bp in s["bp"].values() if bp[2])
                cands = []
                for u, (uid, prio, target) in s["bp"].items():
                    if prio == 0 or not target or targets[target] != 1:
                        continue
                    own, own_next = s["own"].get(target), nxt["own"].get(target)
                    if not own or not own_next or not (own[5] & 1):
                        continue
                    cands.append((rec.created.get(u, -1), u, own_next[4] - own[4]))
                if len(cands) < 2:
                    continue
                went = [c for c in cands if c[2] > 0]
                stayed = [c for c in cands if c[2] <= 0]
                if not went or not stayed:
                    both += bool(went)
                    continue
                mixed += 1
                cands.sort()
                oldest_went += cands[0][2] > 0
                youngest_went += cands[-1][2] > 0
    print(f"stall samples (metal under 2 % of storage, energy over 20 %, spending): {stall_samples}")
    print(f"with two or more lone high-priority builders on nanoframes, all progressing: {both}; some but not all: {mixed}")
    print(f"of the mixed: the oldest builder's target progressed in {oldest_went}, the youngest's in {youngest_went}")
    print("the low flag's bite: lone low-priority builders on a nanoframe, samples where it progressed / all, by the team's metal")
    print("| kind | under 2 % | 2-20 % | over 20 % |")
    print("|---|---|---|---|")
    for kind in ("commander", "con", "lab", "turret"):
        cells = []
        for band in ("under 2 %", "2-20 %", "over 20 %"):
            c = throttle.get((kind, band), collections.Counter())
            n = c[True] + c[False]
            cells.append(f"{c[True]}/{n}" + (f" ({100 * c[True] // n} %)" if n else ""))
        print(f"| {kind} | " + " | ".join(cells) + " |")
    # throughput against the pair's full speed: the median gain of the same builder type on the same target type with metal over 20 %
    import statistics
    full = {}
    for (bd, td, band, prio, kind), ds in deltas.items():
        if band == "over 20 %":
            full.setdefault((bd, td), []).extend(ds)
    full = {k: statistics.median(v) for k, v in full.items() if len(v) >= 5 and statistics.median(v) > 0}
    through = collections.defaultdict(lambda: [0.0, 0.0, 0])  # (kind, prio, band) -> [gained, expected, samples]
    for (bd, td, band, prio, kind), ds in deltas.items():
        ref = full.get((bd, td))
        if ref is None:
            continue
        t = through[(kind, prio, band)]
        t[0] += sum(ds)
        t[1] += ref * len(ds)
        t[2] += len(ds)
    print("throughput: health gained per sample as a share of the pair's full speed (median with metal over 20 %), lone builders on nanoframes")
    print("| kind, priority | under 2 % | 2-20 % | over 20 % |")
    print("|---|---|---|---|")
    for kind in ("commander", "con", "lab", "turret"):
        for prio in ("high", "low"):
            cells = []
            for band in ("under 2 %", "2-20 %", "over 20 %"):
                g, x, n = through.get((kind, prio, band), (0, 0, 0))
                cells.append(f"{100 * g / x:.0f} % of {n}" if x else "-")
            if any(through.get((kind, prio, b), (0, 0, 0))[2] for b in ("under 2 %", "2-20 %", "over 20 %")):
                print(f"| {kind}, {prio} | " + " | ".join(cells) + " |")


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    if args[0] == "demo":
        for path in args[1:]:
            players, events, frames = parse_demo(path)
            names = {p: v["name"] for p, v in players.items()}
            hand = [e for e in events if e[2] == "hand"]
            widget = [e for e in events if e[2] == "widget"]
            print(f"{os.path.basename(path)[:19]} | {frames / (60 * FPS):4.1f} min | hand {len(hand)} | widget {len(widget)}")
            for f, p, src, units, value in hand:
                print(f"   {clock(f):>6} {names.get(p)} units={units[:10]} -> {'low' if value == 0 else 'high'}")
        return
    if args[0] == "stalls":
        manifest = args[args.index("--manifest") + 1] if "--manifest" in args else os.path.join(REPO, "run", "data", "replays", "manifest.jsonl")
        stalls(manifest)
        return
    if args[0] == "catalog":
        manifest = args[args.index("--manifest") + 1] if "--manifest" in args else os.path.join(REPO, "run", "data", "replays", "manifest.jsonl")
        out = args[args.index("--out") + 1] if "--out" in args else os.path.join(REPO, "docs", "studies", "2026-10-13-builder-priority")
        catalog(manifest, out)
        return
    sys.exit(__doc__)


if __name__ == "__main__":
    main()
