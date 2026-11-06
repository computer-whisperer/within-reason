#!/usr/bin/env python3
"""The waste audit of one game: every second of every unit of both sides accounted for, from the audit dump
(`run/replay_match.py <demo> --audit` writes audit.jsonl; docs/harness/audit.md).

    run/audit.py <replay match dir> [--team N] [--sections a,b,...] [--top N]
    run/audit.py <dir> --unit ID                      one unit's life, run by run
    run/audit.py <dir> --window M:SS M:SS [X Z R]     events and units in a span (and a place)
    run/audit.py <dir> --map M:SS                     both sides on a character map

A review reads the last fight; this reads the ledgers a player keeps without thinking: where the metal sat unspent,
which builders and plants stood or walked instead of building, which frames nobody worked on, which soldiers walked
out and back without a shot or died without dealing their cost, what was lost to raiders and how long they were in
sight first, which spots stood free. Every table is printed for both sides so the opponent is the yardstick.
Sections: summary, economy, floats, power, builders, plants, frames, army, front, cohesion, types, trips, fights, raids,
spots, wrecks, orders, sight. `ours` is the seat whose AI name starts with WR (else team 0, or --team).
"""
import json, math, os, re, sys
from collections import Counter, defaultdict

FPS = 30
CMD = {0: "stop", 5: "wait", 10: "move", 15: "patrol", 16: "fight", 20: "attack", 25: "guard", 40: "repair", 90: "reclaim", 125: "resurrect", 130: "capture", 105: "dgun"}
SETTINGS = {45, 50, 115, 85, 95, 135, 145, 120, 70, 100, 110, 150}  # fire state, move state, repeat, on/off and the like: not an order to act
(F, X, Z, HP, PROG, QUEUE, CMDID, TX, TZ, TARGET, BUILDS, POWER, LINED, SEEN, XP) = range(15)
EXPERIENCE = 0.03  # the engine: 0.1 x the game's experienceMult (BAR 0.3) x the share of the victim's health x the victim's power over the attacker's


def clock(f):
    s = int(f) // FPS
    return f"{s // 60}:{s % 60:02d}"


def frames(text):
    m, s = text.split(":")
    return (int(m) * 60 + int(s)) * FPS


def role(d):
    n = d["name"]
    mobile = d["speed"] > 0
    if n.endswith("com") and mobile and d["build_speed"] > 0:
        return "commander"
    if d["factory"]:
        return "plant"
    if d["build_speed"] > 0 and not mobile:
        return "nano"
    if d["build_speed"] > 0 and mobile:
        return "constructor" if d["options"] else "support"
    if d["extracts"] > 0:
        return "extractor"
    if not mobile:
        if d["weapons"] and d["reach"] > 0:
            return "turret"
        if d["converter"] > 0:
            return "converter"
        if d["energy_make"] > 0 or d["energy_upkeep"] < 0 or d["wind"] > 0 or d["tidal"] > 0:
            return "energy"
        return "building"
    if d["weapons"] and d["reach"] > 0:
        if d["air"]:
            return "air"
        return "scout" if d["metal"] < 40 else "army"
    return "air" if d["air"] else "other"


ARMED = ("army", "scout", "air")
BUILDERS = ("commander", "constructor", "nano", "plant")


class Unit:
    __slots__ = ("id", "d", "role", "team", "created", "finished", "died", "by", "killer", "killer_d", "killer_team", "death_progress", "rows", "x", "z", "dealt", "taken", "hit_frames", "orders")

    def __init__(self, uid, d, team):
        self.id, self.d, self.role, self.team = uid, d, role(d), team
        self.created = self.finished = self.died = None
        self.by = self.killer = self.killer_d = 0
        self.killer_team = -1
        self.death_progress = 100
        self.rows = []
        self.x = self.z = 0
        self.dealt = self.taken = 0.0  # the value (power: metal + energy / 60) of the damage dealt and taken
        self.hit_frames = set()  # sample frames in which it dealt or took damage
        self.orders = 0

    @property
    def name(self):
        return self.d["name"]

    @property
    def worth(self):
        """The engine's `power` of the type: its metal and a sixtieth of its energy, what experience is counted in."""
        return self.d.get("power") or self.d["metal"]

    def alive_at(self, f):
        return (self.created or 0) <= f and (self.died is None or self.died > f)

    def row_at(self, f):
        rows = self.rows
        if not rows:
            return None
        i = (f - rows[0][F]) // FPS
        if i < 0:
            return None
        if i >= len(rows):
            return rows[-1] if self.died is None or self.died > f else None
        return rows[i]


class Game:
    def __init__(self, path, team=None):
        if os.path.isdir(path):
            self.dir = path
            path = os.path.join(path, "audit.jsonl")
            if not os.path.exists(path):
                path = os.path.join(self.dir, "LuaUI", "Config", "within-reason", "audit.jsonl")
        else:
            self.dir = os.path.dirname(path)
        self.defs, self.units, self.frames, self.sides, self.events, self.cmds, self.wrecks, self.hits = {}, {}, [], {}, [], [], [], []
        self.winners = None
        for line in open(path):
            try:
                r = json.loads(line)
            except ValueError:
                continue
            t = r["t"]
            if t == "header":
                self.header = r
            elif t == "def":
                self.defs[r["d"]["id"]] = r["d"]
            elif t == "s":
                f = r["f"]
                self.frames.append(f)
                self.sides[f] = {s["team"]: s for s in r["sides"]}
                for u in r["u"]:
                    unit = self.units.get(u[0])
                    if unit is None:
                        unit = self.units[u[0]] = Unit(u[0], self.defs[u[1]], u[2])
                        unit.created = f
                    unit.rows.append((f, u[3], u[4], u[5], u[6], u[7], u[8], u[9], u[10], u[11], u[12], u[13], u[14], u[15], u[16] if len(u) > 16 else 0.0))
                    unit.x, unit.z = u[3], u[4]
                for hit in r.get("hits", ()):
                    self.hits.append((f, hit[-2], hit[-1]))
            elif t == "ev":
                k = r["k"]
                unit = self.units.get(r["u"])
                if unit is None and k in ("created", "finished", "destroyed"):
                    unit = self.units[r["u"]] = Unit(r["u"], self.defs[r["d"]], r["team"])
                    unit.x, unit.z = r.get("x", 0), r.get("z", 0)
                if k == "created":
                    unit.created, unit.by = r["f"], r.get("by", 0)
                    unit.x, unit.z = r["x"], r["z"]
                elif k == "finished":
                    unit.finished = r["f"]
                elif k == "destroyed":
                    unit.died, unit.killer, unit.killer_d, unit.killer_team, unit.death_progress = r["f"], r.get("by", 0), r.get("by_d", 0), r.get("by_team", -1), r.get("progress", 100)
                    unit.x, unit.z = r["x"], r["z"]
                self.events.append(r)
            elif t == "cmd":
                self.cmds.append(r)
            elif t == "wrecks":
                self.wrecks.append(r)
            elif t == "over":
                self.winners = r.get("winners")
        self.end = self.frames[-1]
        for unit in self.units.values():
            if unit.created is None:
                unit.created = unit.rows[0][F] if unit.rows else 0
            if unit.finished is None and unit.rows and unit.rows[0][PROG] >= 100:
                unit.finished = unit.created
        taken_all = 0.0
        for f, v, amount in self.hits:
            victim = self.units.get(v)
            if victim is None:
                continue
            value = amount / max(1.0, victim.d["health"]) * victim.worth
            victim.taken += value
            taken_all += value
            victim.hit_frames.add(f)
        gained_all = 0.0
        for unit in self.units.values():
            for a, b in zip(unit.rows, unit.rows[1:]):
                if b[XP] > a[XP]:
                    unit.dealt += (b[XP] - a[XP]) * unit.worth / EXPERIENCE
                    unit.hit_frames.add(b[F])
            gained_all += unit.dealt
        # The constant's check: the value dealt by experience over the value taken by damage, 1 when it holds. It
        # read 0.58 on its first game (dist135-audit-apex: friendly fire, death blasts and a constant that may differ
        # in the game's version all sit in it), so the dealt values are scaled to the taken total: a unit's `dealt`
        # is its share of everything its side's enemies took, and both ledgers add to the same sum.
        self.experience_check = gained_all / taken_all if taken_all else None
        if self.experience_check:
            for unit in self.units.values():
                unit.dealt /= self.experience_check
        for c in self.cmds:
            unit = self.units.get(c["u"])
            if unit is not None and c["c"] not in SETTINGS:
                unit.orders += 1
        self.teams = sorted(s["team"] for s in self.header["seats"])
        self.names = self.seat_names()
        if team is None:
            team = next((t for t in self.teams if self.names[t].startswith("WR")), self.teams[0])
        self.us = team
        self.them = next(t for t in self.teams if t != team)
        self.start = {}
        for t in self.teams:
            com = next((u for u in self.units.values() if u.team == t and u.role == "commander" and u.rows), None)
            self.start[t] = (com.rows[0][X], com.rows[0][Z]) if com else (0, 0)
        self.has_hits = bool(self.hits) or any(u.dealt for u in self.units.values())

    def seat_names(self):
        names = {s["team"]: s["name"] for s in self.header["seats"]}
        script = os.path.join(self.dir, "_script.txt")
        if os.path.exists(script):
            text = open(script, errors="replace").read().lower()
            for block in re.findall(r"\[ai\d+\]\s*\{(.*?)\}", text, re.S):
                fields = dict(re.findall(r"(\w+)\s*=\s*([^;]*);", block))
                if "team" in fields and "shortname" in fields:
                    names[int(fields["team"])] = fields["shortname"].upper()[:2] + fields["shortname"][2:] if fields["shortname"].startswith("wr") else fields["shortname"]
        return names

    def of(self, team, *roles):
        return [u for u in self.units.values() if u.team == team and (not roles or u.role in roles)]

    def axis(self, team, x, z):
        """0 at this team's start, 1 at the enemy's, along the line between them."""
        (ax, az), (bx, bz) = self.start[team], self.start[self.them if team == self.us else self.us]
        dx, dz = bx - ax, bz - az
        return ((x - ax) * dx + (z - az) * dz) / max(1.0, dx * dx + dz * dz)

    def zone(self, team, x, z):
        p = self.axis(team, x, z)
        return "home" if p < 1 / 3 else ("his" if p > 2 / 3 else "mid")

    def label(self, team):
        return f"{self.names.get(team, '?')} (team {team}{', ours' if team == self.us else ''})"


def dist(ax, az, bx, bz):
    return math.hypot(ax - bx, az - bz)


def runs(states):
    """[(state, first index, last index)] over a list of states."""
    out = []
    for i, s in enumerate(states):
        if out and out[-1][0] == s:
            out[-1][2] = i
        else:
            out.append([s, i, i])
    return out


def builder_states(u):
    """One state a sample for a builder once finished: build, starved (building at part power counts in both, see
    `power`), walk, wait (an order, standing, nothing built), idle (no order); plants: build, blocked (a queue and
    nothing under way), idle. Returns [(frame, state, power 0..1)]."""
    out = []
    prev = None
    for r in u.rows:
        if u.finished is None or r[F] < u.finished or r[PROG] < 100:
            prev = r
            continue
        power = r[POWER] / 100.0
        if u.role == "plant":
            state = "build" if r[BUILDS] else ("blocked" if r[LINED] > 0 else "idle")
        elif r[BUILDS] or power > 0:
            state = "build" if r[BUILDS] else "work"
        elif u.role == "nano":
            state = "idle"
        elif r[QUEUE] == 0:
            state = "idle"
        elif prev is not None and dist(r[X], r[Z], prev[X], prev[Z]) >= 6:
            state = "walk"
        else:
            state = "guard" if r[CMDID] == 25 else "wait"
        out.append((r[F], state, power if state in ("build", "work") else 0.0))
        prev = r
    return out


def fmt_secs(s):
    return f"{int(s) // 60}:{int(s) % 60:02d}"


def table(rows, header=None, indent="  "):
    rows = [[str(c) for c in r] for r in rows]
    if header:
        rows = [list(header)] + rows
    if not rows:
        return
    widths = [max(len(r[i]) for r in rows if i < len(r)) for i in range(max(len(r) for r in rows))]
    for r in rows:
        print(indent + "  ".join(c.rjust(widths[i]) if re.fullmatch(r"[-+0-9.,:%x/ ]*", c) else c.ljust(widths[i]) for i, c in enumerate(r)).rstrip())


# ------------------------------------------------------------------------------------------------ sections


def section_economy(g, top):
    print("\n== ECONOMY by minute (store at the minute / mean income, pull and spend over it; stall = seconds the energy store was under 5 %; waste = overflow) ==")
    for team in (g.us, g.them):
        print(f" {g.label(team)}")
        rows = []
        last_m = last_e = 0.0
        for start in range(0, g.end, 60 * FPS):
            fs = [f for f in g.frames if start <= f < start + 60 * FPS]
            if not fs:
                continue
            sides = [g.sides[f][team] for f in fs]
            n = len(sides)
            m_inc = sum(s["m"][3] for s in sides) / n
            m_pull = sum(s["m"][2] for s in sides) / n
            m_exp = sum(s["m"][4] for s in sides) / n
            e_inc = sum(s["e"][3] for s in sides) / n
            e_pull = sum(s["e"][2] for s in sides) / n
            e_exp = sum(s["e"][4] for s in sides) / n
            stall = sum(1 for s in sides if s["e"][0] < 0.05 * max(1, s["e"][1]))
            mfull = sum(1 for s in sides if s["m"][0] > 0.9 * max(1, s["m"][1]))
            st = sides[-1].get("st", {})
            m_x, e_x = st.get("metal_excess", 0) or 0, st.get("energy_excess", 0) or 0
            end = sides[-1]
            rows.append([clock(start), f"{end['m'][0]:.0f}/{end['m'][1]:.0f}", f"{m_inc:.1f}", f"{m_pull:.1f}", f"{m_exp:.1f}", f"{m_x - last_m:.0f}", mfull, f"{end['e'][0]:.0f}/{end['e'][1]:.0f}", f"{e_inc:.0f}", f"{e_pull:.0f}", f"{e_exp:.0f}", f"{e_x - last_e:.0f}", stall])
            last_m, last_e = m_x, e_x
        table(rows, ["min", "metal", "inc", "pull", "spent", "wasted", "full s", "energy", "inc", "pull", "spent", "wasted", "stall s"])


def float_spans(g, team, bar=0.5, floor=400):
    """Spans with the metal store over `bar` of its size and over `floor`."""
    spans, cur = [], None
    for f in g.frames:
        m = g.sides[f][team]["m"]
        high = m[0] > max(floor, bar * m[1])
        if high:
            if cur is None:
                cur = [f, f, m[0]]
            cur[1], cur[2] = f, max(cur[2], m[0])
        elif cur is not None:
            spans.append(cur)
            cur = None
    if cur is not None:
        spans.append(cur)
    return spans


def power_at(g, team, f0, f1):
    """Builder-seconds by role and state over [f0, f1]."""
    out = defaultdict(Counter)
    for u in g.of(team, *BUILDERS):
        for f, state, power in u._states:
            if f0 <= f <= f1:
                out[u.role][state] += 1
    return out


def section_floats(g, top):
    print("\n== METAL FLOATS (store over half and over 400): what the builders did meanwhile ==")
    for team in (g.us, g.them):
        spans = [s for s in float_spans(g, team) if s[1] - s[0] >= 20 * FPS]
        banked = sum(g.sides[f][team]["m"][0] for f in g.frames) / len(g.frames)
        print(f" {g.label(team)}: mean bank {banked:.0f}, {len(spans)} spans of 20 s or more")
        for f0, f1, peak in spans[:top]:
            mix = power_at(g, team, f0, f1)
            parts = []
            for r in BUILDERS:
                total = sum(mix[r].values())
                if total:
                    parts.append(f"{r} " + " ".join(f"{k} {100 * v // total}%" for k, v in mix[r].most_common()))
            e = [g.sides[f][team]["e"] for f in g.frames if f0 <= f <= f1]
            stall = sum(1 for x in e if x[0] < 0.05 * max(1, x[1]))
            print(f"   {clock(f0)}-{clock(f1)} peak {peak:.0f}, energy stalled {stall} s of {len(e)}: " + "; ".join(parts))


def section_power(g, top):
    print("\n== BUILD POWER: every builder-second (delivered = build speed x the share of frames it actually built; starved = building below full power; walk, wait = an order but standing, guard = guarding idle, idle = no order) ==")
    for team in (g.us, g.them):
        print(f" {g.label(team)}")
        rows = []
        for r in BUILDERS:
            units = g.of(team, r)
            nominal = delivered = 0.0
            secs = Counter()
            for u in units:
                bs = u.d["build_speed"]
                for f, state, power in u._states:
                    nominal += bs
                    secs[state] += 1
                    if state in ("build", "work"):
                        delivered += bs * power
                        secs["starved_bp"] += bs * (1 - power)
            total = sum(v for k, v in secs.items() if k != "starved_bp")
            if not total:
                continue
            share = lambda k: f"{100 * secs[k] / total:.0f}%"
            rows.append([r, len(units), fmt_secs(total), f"{100 * delivered / max(1, nominal):.0f}%", share("build"), share("work"), f"{100 * secs['starved_bp'] / max(1, nominal):.0f}%", share("walk"), share("wait"), share("guard"), share("blocked"), share("idle")])
        table(rows, ["role", "n", "unit-time", "delivered", "build", "other work", "starved", "walk", "wait", "guard", "blocked", "idle"])
        # by five-minute blocks, the constructors' and plants' mix
        rows = []
        for start in range(0, g.end, 300 * FPS):
            row = [f"{clock(start)}-"]
            for r in ("commander", "constructor", "plant", "nano"):
                nominal = delivered = 0.0
                for u in g.of(team, r):
                    for f, state, power in u._states:
                        if start <= f < start + 300 * FPS:
                            nominal += u.d["build_speed"]
                            delivered += u.d["build_speed"] * power
                row.append(f"{delivered / 300:.0f}/{nominal / 300:.0f}" if nominal else "-")
            rows.append(row)
        table(rows, ["block", "commander", "constructors", "plants", "nanos  (build power delivered / standing, a second)"])


def section_builders(g, top):
    print("\n== BUILDERS one by one (ours): seconds by state, the path walked, jobs, and the longest idle, wait and walk runs ==")
    team = g.us
    rows, episodes = [], []
    for u in sorted(g.of(team, "commander", "constructor"), key=lambda u: u.created):
        st = u._states
        if not st:
            continue
        secs = Counter(s for _, s, _ in st)
        path = sum(dist(a[X], a[Z], b[X], b[Z]) for a, b in zip(u.rows, u.rows[1:]))
        jobs = len({r[BUILDS] for r in u.rows if r[BUILDS]})
        total = len(st)
        rows.append([u.id, u.name, clock(u.finished or u.created), clock(u.died) if u.died else "-", fmt_secs(total)] + [f"{100 * secs[k] / total:.0f}%" for k in ("build", "work", "walk", "wait", "guard", "idle")] + [f"{path:.0f}", jobs, f"{path / max(1, jobs):.0f}", u.orders])
        # A run is one state under one order: a walk redirected on its way is two walks, each named by its own order.
        keyed = []
        for f, state, _ in st:
            r = u.row_at(f)
            keyed.append((state, (r[CMDID], r[TX] // 64, r[TZ] // 64, r[TARGET]) if state == "walk" else None))
        for (state, _), i, j in runs(keyed):
            if state in ("idle", "wait", "walk", "guard") and j - i + 1 >= {"idle": 8, "wait": 15, "walk": 40, "guard": 30}[state]:
                r0 = u.row_at(st[i][0])
                r1 = u.row_at(st[j][0])
                after = st[j + 1][1] if j + 1 < len(st) else "the end"
                episodes.append((j - i + 1, state, u, st[i][0], st[j][0], r0, r1, after))
    table(rows, ["id", "type", "from", "died", "time", "build", "work", "walk", "wait", "guard", "idle", "path", "jobs", "path/job", "orders"])
    for state in ("idle", "wait", "guard", "walk"):
        eps = sorted((e for e in episodes if e[1] == state), key=lambda e: -e[0])
        total = sum(e[0] for e in eps)
        print(f"  {state}: {len(eps)} runs, {fmt_secs(total)} in all; the longest:")
        for secs, _, u, f0, f1, r0, r1, after in eps[:top]:
            cmd = r0[CMDID]
            what = (g.defs.get(-cmd, {}).get("name", f"def {-cmd}") if cmd < 0 else CMD.get(cmd, str(cmd))) if r0[QUEUE] else "no order"
            moved = dist(r0[X], r0[Z], r1[X], r1[Z])
            print(f"     {clock(f0)}-{clock(f1)} {secs:4d} s  {u.name} {u.id} at ({r0[X]},{r0[Z]}) -> ({r1[X]},{r1[Z]}) {moved:.0f} elmos, order {what}" + (f" to ({r0[TX]},{r0[TZ]})" if r0[TX] or r0[TZ] else "") + f", then {after}")


def section_plants(g, top):
    print("\n== PLANTS: what each made and when it stood (blocked = a queue and nothing under way) ==")
    for team in (g.us, g.them):
        print(f" {g.label(team)}")
        for u in sorted(g.of(team, "plant"), key=lambda u: u.created):
            st = u._states
            if not st:
                print(f"   {u.name} {u.id} begun {clock(u.created)}, never finished" + (f", died {clock(u.died)}" if u.died else ""))
                continue
            secs = Counter(s for _, s, _ in st)
            total = len(st)
            mean_power = sum(p for _, s, p in st if s == "build") / max(1, secs["build"])
            made = sorted((v for v in g.units.values() if v.by == u.id and v.team == team), key=lambda v: v.created)
            kinds = Counter(v.role for v in made)
            print(f"   {u.name} {u.id} at ({u.x},{u.z}) up {clock(u.finished)}" + (f" died {clock(u.died)}" if u.died else "") + f": build {100 * secs['build'] // total}% at {100 * mean_power:.0f}% power, blocked {100 * secs['blocked'] // total}%, idle {100 * secs['idle'] // total}% ({fmt_secs(secs['idle'])}); made {len(made)}: " + ", ".join(f"{n} {k}" for k, n in kinds.most_common()))
            if team == g.us:
                by_min = defaultdict(list)
                for v in made:
                    by_min[v.created // (60 * FPS)].append(v.name.replace("arm", "").replace("cor", ""))
                line = "     "
                for m in sorted(by_min):
                    c = Counter(by_min[m])
                    line += f"{m}: " + " ".join(f"{n}{k}" if n > 1 else k for k, n in c.items()) + " | "
                print(line[:400])
                states = [s for _, s, _ in st]
                gaps = [(j - i + 1, st[i][0], st[j][0], s) for s, i, j in runs(states) if s in ("idle", "blocked") and j - i + 1 >= 10]
                for secs_, f0, f1, s in sorted(gaps, reverse=True)[:top // 2]:
                    m = g.sides[f0][team]["m"]
                    e = g.sides[f0][team]["e"]
                    print(f"     {s} {clock(f0)}-{clock(f1)} ({secs_} s), metal {m[0]:.0f} energy {e[0]:.0f} at its start")


def section_frames(g, top):
    print("\n== FRAMES: built slowly, left alone, lost unfinished (orphan = seconds a frame stood with nobody building it) ==")
    for team in (g.us, g.them):
        working = defaultdict(set)  # frame id -> sample frames somebody built it
        for u in g.of(team, *BUILDERS):
            for r in u.rows:
                if r[BUILDS]:
                    working[r[BUILDS]].add(r[F])
        slow, lost, unfinished = [], [], []
        orphan_total = 0
        for u in g.of(team):
            if u.by == 0 and u.role == "commander":
                continue
            if u.d["speed"] > 0 and u.role not in ("commander",):
                # a plant's product: the plant works on it; orphan seconds show a blocked plant
                pass
            start = u.created
            end = u.finished if u.finished is not None else (u.died if u.died is not None else g.end)
            if end is None or start is None or end <= start:
                continue
            span = [f for f in g.frames if start <= f < end]
            orphan = sum(1 for f in span if f not in working[u.id])
            orphan_total += orphan
            took = (end - start) / FPS
            if u.finished is None and u.died is not None:
                lost.append((u.death_progress * u.d["metal"] / 100, u, took, orphan))
            elif u.finished is None:
                last = u.rows[-1][PROG] if u.rows else 0
                unfinished.append((last * u.d["metal"] / 100, u, took, orphan, last))
            elif orphan >= 15:
                slow.append((orphan, u, took))
        print(f" {g.label(team)}: {fmt_secs(orphan_total)} frame-time with nobody building; {len(lost)} frames lost unfinished holding {sum(x[0] for x in lost):.0f} metal; {len(unfinished)} unfinished at the end holding {sum(x[0] for x in unfinished):.0f}")
        if team != g.us:
            continue
        print("   left alone longest (finished later):")
        for orphan, u, took in sorted(slow, key=lambda x: -x[0])[:top]:
            print(f"     {u.name} {u.id} at ({u.x},{u.z}) begun {clock(u.created)} done {clock(u.finished)} ({took:.0f} s), alone {orphan} s")
        if lost:
            print("   lost unfinished:")
            for metal, u, took, orphan in sorted(lost, key=lambda x: -x[0])[:top]:
                killer = g.defs.get(u.killer_d, {}).get("name", "?") if u.killer_team not in (-1, u.team) else ("self/decay" if u.killer_team == -1 else "own")
                print(f"     {u.name} {u.id} at ({u.x},{u.z}) begun {clock(u.created)} died {clock(u.died)} at {u.death_progress}% ({metal:.0f} metal), alone {orphan} s, by {killer}")
        if unfinished:
            print("   unfinished at the end: " + ", ".join(f"{u.name} {last}% (begun {clock(u.created)}, alone {orphan} s)" for metal, u, took, orphan, last in sorted(unfinished, key=lambda x: -x[0])[:top]))


def army_state(g, u, i):
    r = u.rows[i]
    f = r[F]
    if any((f + k * FPS) in u.hit_frames for k in (-3, -2, -1, 0, 1, 2, 3)):
        return "fight"
    if i and dist(r[X], r[Z], u.rows[i - 1][X], u.rows[i - 1][Z]) >= 6:
        return "move"
    return "still"


def section_army(g, top):
    print("\n== ARMY by minute: metal alive on each side, where ours stood (home third / middle / his third) and what it was doing ==")
    mins = defaultdict(lambda: defaultdict(float))
    for team in (g.us, g.them):
        for u in g.of(team, *ARMED):
            if u.finished is None:
                continue
            for i, r in enumerate(u.rows):
                if r[F] < u.finished or r[F] % (10 * FPS):
                    continue
                m = r[F] // (60 * FPS)
                metal = u.d["metal"] / 6.0
                mins[m][("alive", team)] += metal
                if u.dealt == 0 and u.taken == 0:
                    mins[m][("unused", team)] += metal
                if team == g.us:
                    mins[m][g.zone(team, r[X], r[Z])] += metal
                    mins[m][army_state(g, u, i)] += metal
    rows = []
    first = {}
    for m in sorted(mins):
        v = mins[m]
        ours, his = v[("alive", g.us)], v[("alive", g.them)]
        f = m * 60 * FPS
        turrets = sum(u.d["metal"] for u in g.of(g.them, "turret") if u.finished is not None and u.finished <= f and u.alive_at(f))
        pct = lambda k: f"{100 * v[k] / ours:.0f}%" if ours else "-"
        for bar in (2, 3):
            if his and ours / (his + turrets) >= bar and ours > 1000:
                first.setdefault(bar, m)
        rows.append([f"{m}:00", f"{ours:.0f}", f"{his:.0f}", f"{turrets:.0f}", f"{ours / his:.1f}x" if his else "-", f"{ours / (his + turrets):.1f}x" if his + turrets else "-", pct("home"), pct("mid"), pct("his"), pct("still"), pct("move"), pct("fight"), pct(("unused", g.us))])
    table(rows, ["min", "ours", "his", "his turrets", "ratio", "with turrets", "home", "mid", "his third", "still", "move", "fight", "never fought"])
    if first:
        print("  the lead: " + ", ".join(f"{bar}x his army and turrets together from {m}:00" for bar, m in sorted(first.items())) + f"; the game ended {clock(g.end)}")
    if not g.has_hits:
        print("  (no damage rows in this dump: the fight column is empty and the still column holds it)")


def section_types(g, top):
    print("\n== UNIT TYPES: made, lost, the value of the damage each dealt against its own cost (dealt/cost under 1 = did not pay for itself) ==")
    if g.experience_check:
        print(f"  (dealt is read from each unit's experience and scaled so that it sums to the damage taken: the raw sum was {g.experience_check:.2f} of it)")
    for team in (g.us, g.them):
        print(f" {g.label(team)}")
        by = defaultdict(list)
        for u in g.of(team):
            if u.finished is not None and (u.role in ARMED or u.role in ("turret", "commander")):
                by[u.name].append(u)
        rows = []
        for name, us in sorted(by.items(), key=lambda kv: -len(kv[1]) * kv[1][0].d["metal"]):
            cost = sum(u.d["metal"] for u in us)
            dead = [u for u in us if u.died]
            dealt = sum(u.dealt for u in us)
            nothing = [u for u in dead if u.dealt < 0.1 * u.worth]
            life = sum(((u.died or g.end) - u.finished) / FPS for u in us) / len(us)
            rows.append([name, us[0].role, len(us), f"{cost:.0f}", len(dead), f"{sum(u.d['metal'] for u in dead):.0f}", f"{dealt:.0f}", f"{dealt / max(1, cost):.2f}", len(nothing), f"{sum(u.d['metal'] for u in nothing):.0f}", fmt_secs(life)])
        table(rows, ["type", "role", "made", "metal", "died", "metal", "dealt", "dealt/cost", "died for <10%", "metal", "mean life"])


def section_trips(g, top):
    print("\n== MOVEMENT (ours): the army's walking, and trips out of the home third and back without a shot dealt or taken ==")
    team = g.us
    total_path = fight = still = move = 0
    wasted, yoyo = [], Counter()
    for u in g.of(team, "army", "scout"):
        if u.finished is None:
            continue
        rows = [r for r in u.rows if r[F] >= u.finished]
        out_from = None
        peak = 0.0
        reversals = 0
        last_dir, anchor = 0, None
        for i, r in enumerate(rows):
            if i:
                total_path += dist(r[X], r[Z], rows[i - 1][X], rows[i - 1][Z])
            p = g.axis(team, r[X], r[Z])
            if anchor is None:
                anchor = p
            if abs(p - anchor) > 0.06:
                d = 1 if p > anchor else -1
                if last_dir and d != last_dir:
                    reversals += 1
                last_dir, anchor = d, p
            if out_from is None and p > 0.4:
                out_from, peak = i, p
            elif out_from is not None:
                peak = max(peak, p)
                if p < 0.3:
                    span = rows[out_from:i + 1]
                    if not any(x[F] in u.hit_frames for x in span):
                        path = sum(dist(a[X], a[Z], b[X], b[Z]) for a, b in zip(span, span[1:]))
                        wasted.append((len(span), u, span[0][F], span[-1][F], peak, path))
                    out_from = None
        yoyo[u.id] = reversals
    units = [u for u in g.of(team, "army", "scout") if u.finished is not None]
    waits, never = [], []
    for u in units:
        if not u.hit_frames:
            never.append(u)
            continue
        f1 = min(u.hit_frames)
        before = [r for r in u.rows if u.finished <= r[F] <= f1]
        if len(before) < 2:
            continue
        path = sum(dist(a[X], a[Z], b[X], b[Z]) for a, b in zip(before, before[1:]))
        straight = dist(before[0][X], before[0][Z], before[-1][X], before[-1][Z])
        waits.append(((f1 - u.finished) / FPS, path, straight))
    if waits:
        waits.sort()
        mid = waits[len(waits) // 2]
        paths = sorted(w[1] / max(1.0, w[2]) for w in waits)
        print(f"  from the plant to the first shot dealt or taken ({len(waits)} units): median {fmt_secs(mid[0])}, quartiles {fmt_secs(waits[len(waits) // 4][0])} and {fmt_secs(waits[3 * len(waits) // 4][0])}; the path walked over the straight line to that fight: median {paths[len(paths) // 2]:.1f}x, upper quartile {paths[3 * len(paths) // 4]:.1f}x")
    print(f"  never dealt or took a shot: {len(never)} of {len(units)} units, {sum(u.d['metal'] for u in never):.0f} metal ({sum(1 for u in never if u.died is None)} alive at the end)")
    unit_secs = sum(len([r for r in u.rows if r[F] >= u.finished]) for u in units)
    print(f"  {len(units)} soldiers and scouts, {fmt_secs(unit_secs)} unit-time, {total_path / 1000:.0f} thousand elmos walked ({total_path / max(1, len(units)):.0f} a unit), {sum(u.orders for u in units)} orders ({sum(u.orders for u in units) / max(1, unit_secs / 60):.1f} a unit-minute)")
    print(f"  direction reversals along the axis between the starts (over 6 % of it each): {sum(yoyo.values())} in all, {sum(yoyo.values()) / max(1, len(units)):.1f} a unit; most: " + ", ".join(f"{g.units[i].name} {i} x{n}" for i, n in yoyo.most_common(6)))
    print(f"  trips out past 40 % and back under 30 % with no shot either way: {len(wasted)}, {fmt_secs(sum(w[0] for w in wasted))} unit-time, {sum(w[5] for w in wasted) / 1000:.0f} thousand elmos")
    by_minute = Counter()
    for secs, u, f0, f1, peak, path in wasted:
        by_minute[f0 // (60 * FPS)] += 1
    print("    leaving in minute: " + " ".join(f"{m}:{n}" for m, n in sorted(by_minute.items())))
    for secs, u, f0, f1, peak, path in sorted(wasted, key=lambda w: -w[0])[:top]:
        print(f"    {clock(f0)}-{clock(f1)} {u.name} {u.id} out to {100 * peak:.0f}% of the way, {path:.0f} elmos")


def section_front(g, top):
    print("\n== FRONT: armed metal along the line between the starts, in tenths of the way from ours (left) to his (right), hundreds of metal; ours is the army, his the army and turrets ==")
    step = 30 * FPS
    rows = []
    for f in range(0, g.end + 1, step):
        if f % (60 * FPS) and f < 6 * 60 * FPS:
            continue
        ours, his = [0.0] * 10, [0.0] * 10
        for u in g.units.values():
            if u.finished is None or u.finished > f or not u.alive_at(f):
                continue
            if u.role not in ARMED and not (u.team == g.them and u.role in ("turret", "commander")):
                continue
            r = u.row_at(f)
            if r is None:
                continue
            b = min(9, max(0, int(10 * g.axis(g.us, r[X], r[Z]))))
            (ours if u.team == g.us else his)[b] += u.d["metal"]
        cell = lambda v: f"{v / 100:4.0f}" if v >= 50 else "   ."
        rows.append(f"  {clock(f):>5} ours|" + "".join(cell(v) for v in ours) + " |  his|" + "".join(cell(v) for v in his) + " |")
    for row in rows:
        print(row)


def section_cohesion(g, top):
    print("\n== COHESION (ours): the army outside the home third every 30 s: how many, how many within 700 of the foremost unit (the head), his armed metal within 900 of the head; and how alone our soldiers died ==")
    team = g.us
    army = [u for u in g.of(team, "army") if u.finished is not None]
    rows = []
    for f in range(0, g.end + 1, 30 * FPS):
        out = []
        for u in army:
            r = u.row_at(f) if u.alive_at(f) and u.finished <= f else None
            if r is not None and g.axis(team, r[X], r[Z]) > 1 / 3:
                out.append((g.axis(team, r[X], r[Z]), r, u))
        if len(out) < 4:
            continue
        out.sort(key=lambda o: -o[0])
        head = out[0][1]
        with_head = [o for o in out if dist(o[1][X], o[1][Z], head[X], head[Z]) <= 700]
        ps = [o[0] for o in out]
        his, _ = strength(g, g.them, f, head[X], head[Z], 900, ARMED + ("turret", "commander"))
        rows.append([clock(f), len(out), f"{sum(o[2].d['metal'] for o in out):.0f}", len(with_head), f"{sum(o[2].d['metal'] for o in with_head):.0f}", f"{his:.0f}", f"{100 * ps[0]:.0f}%", f"{100 * ps[len(ps) // 2]:.0f}%", f"{100 * ps[-1]:.0f}%", f"({head[X]},{head[Z]})"])
    table(rows, ["clock", "out", "metal", "with the head", "metal", "his at the head", "head at", "median", "tail", "head"])
    bins = Counter()
    metal = Counter()
    for u in army:
        if u.died is None or u.killer_team in (-1, team):
            continue
        f = u.died - u.died % FPS
        friends = sum(1 for v in army if v is not u and v.alive_at(f) and v.finished <= f and (r := v.row_at(f)) is not None and dist(r[X], r[Z], u.x, u.z) <= 600)
        key = "alone (0-2 friends within 600)" if friends <= 2 else ("few (3-9)" if friends <= 9 else "in a group (10+)")
        bins[key] += 1
        metal[key] += u.d["metal"]
    print("  our soldiers' deaths: " + "; ".join(f"{k}: {n}, {metal[k]:.0f} metal" for k, n in bins.most_common()))


def fights(g):
    """Deaths of finished things of both sides clustered in time (25 s) and place (900 elmos)."""
    deaths = sorted((u for u in g.units.values() if u.died is not None and u.killer_team not in (-1, u.team)), key=lambda u: u.died)
    clusters = []
    for u in deaths:
        for c in reversed(clusters):
            if u.died - c["f1"] > 25 * FPS:
                continue
            if dist(u.x, u.z, c["x"], c["z"]) <= 900:
                c["units"].append(u)
                c["f1"] = u.died
                n = len(c["units"])
                c["x"] += (u.x - c["x"]) / n
                c["z"] += (u.z - c["z"]) / n
                break
        else:
            clusters.append({"f0": u.died, "f1": u.died, "x": u.x, "z": u.z, "units": [u]})
    return clusters


def strength(g, team, f, x, z, radius, roles):
    total = 0.0
    names = Counter()
    for u in g.of(team, *roles):
        if u.finished is None or u.finished > f or not u.alive_at(f):
            continue
        r = u.row_at(f)
        if r is not None and dist(r[X], r[Z], x, z) <= radius:
            total += u.d["metal"]
            names[u.name] += 1
    return total, names


def section_fights(g, top):
    print("\n== FIGHTS: deaths of both sides clustered (25 s, 900 elmos), ranked by what we lost over what he lost; 'there' = armed metal within 1,000 of the place 10 s before the first death ==")
    rows = []
    for c in fights(g):
        ours = [u for u in c["units"] if u.team == g.us]
        his = [u for u in c["units"] if u.team == g.them]
        value = lambda us: sum(u.d["metal"] * (u.death_progress if u.finished is None else 100) / 100 for u in us)
        lo, lh = value(ours), value(his)
        if lo + lh < 150:
            continue
        rows.append((lo - lh, c, ours, his, lo, lh))
    rows.sort(key=lambda r: -r[0])
    total_o = sum(r[4] for r in rows)
    total_h = sum(r[5] for r in rows)
    print(f"  {len(rows)} fights of 150 metal or more: ours lost {total_o:.0f}, his {total_h:.0f}")
    shown = rows if len(rows) <= 2 * top else rows[:top] + [None] + rows[-top:]
    for row in shown:
        if row is None:
            print("   ...")
            continue
        net, c, ours, his, lo, lh = row
        f = max(0, c["f0"] - 10 * FPS)
        f -= f % FPS
        x, z = c["x"], c["z"]
        so, no = strength(g, g.us, f, x, z, 1000, ARMED + ("commander",))
        sh, nh = strength(g, g.them, f, x, z, 1000, ARMED + ("commander",))
        to, _ = strength(g, g.us, f, x, z, 1000, ("turret",))
        th, nth = strength(g, g.them, f, x, z, 1000, ("turret",))
        killers = Counter(g.units[u.killer].role if u.killer in g.units else "?" for u in ours)
        what = lambda us: " ".join(f"{n}{k.replace('arm', '').replace('cor', '')}" for k, n in Counter(u.name for u in us).most_common(5))
        print(f"   {clock(c['f0'])}-{clock(c['f1'])} at ({x:.0f},{z:.0f}) {g.zone(g.us, x, z)}: ours -{lo:.0f} ({what(ours)}) his -{lh:.0f} ({what(his)}); there: ours {so:.0f}+{to:.0f} turrets, his {sh:.0f}+{th:.0f} turrets; ours killed by {dict(killers)}")


def section_raids(g, top):
    print("\n== LOSSES OUTSIDE THE ARMY (ours): each thing lost with its killer, how long the killer had been in our sight (los or radar) and in our two thirds, and the nearest soldier and turret of ours ==")
    team = g.us
    lost = sorted((u for u in g.of(team) if u.died is not None and u.role not in ARMED and u.killer_team not in (-1, team)), key=lambda u: u.died)
    print(f"  {len(lost)} lost, {sum(u.d['metal'] * (u.death_progress if u.finished is None else 100) / 100 for u in lost):.0f} metal: " + ", ".join(f"{n} {k}" for k, n in Counter(u.role for u in lost).most_common()))
    by_killer = defaultdict(list)
    for u in lost:
        by_killer[u.killer].append(u)
    print("  by killer (the raiders that took the most):")
    ranked = sorted(by_killer.items(), key=lambda kv: -sum(u.d["metal"] for u in kv[1]))
    for kid, victims in ranked[:top]:
        k = g.units.get(kid)
        if k is None:
            print(f"    unknown killer: {len(victims)} things, {sum(u.d['metal'] for u in victims):.0f} metal")
            continue
        first = victims[0].died
        seen_since = inside_since = None
        for r in reversed([r for r in k.rows if r[F] <= first]):
            if seen_since is None and r[SEEN] == 0:
                seen_since = r[F]
            if inside_since is None and g.axis(team, r[X], r[Z]) > 2 / 3:
                inside_since = r[F]
            if seen_since is not None and inside_since is not None:
                break
        born = k.rows[0][F] if k.rows else first
        seen_for = (first - (seen_since if seen_since is not None else born)) / FPS
        inside_for = (first - (inside_since if inside_since is not None else born)) / FPS
        fate = f"died {clock(k.died)}" if k.died else "lived"
        print(f"    {k.name} {kid} ({k.role}): {len(victims)} things, {sum(u.d['metal'] for u in victims):.0f} metal, {clock(first)}-{clock(victims[-1].died)}; in sight {seen_for:.0f} s and out of his third {inside_for:.0f} s before its first kill; {fate}; took {k.taken:.0f} metal of damage")
    print("  one by one:")
    rows = []
    for u in lost[:400]:
        f = u.died - u.died % FPS
        k = g.units.get(u.killer)
        soldier = min((dist(r[X], r[Z], u.x, u.z) for v in g.of(team, "army", "commander") if v.finished is not None and v.alive_at(f) for r in [v.row_at(f)] if r), default=-1)
        turret = min((dist(v.x, v.z, u.x, u.z) for v in g.of(team, "turret") if v.finished is not None and v.finished <= f and v.alive_at(f)), default=-1)
        rows.append([clock(u.died), u.name + ("" if u.finished is not None else f" {u.death_progress}%"), f"({u.x},{u.z})", g.zone(team, u.x, u.z), f"{k.name} {k.id}" if k else "?", f"{soldier:.0f}", f"{turret:.0f}"])
    if len(rows) > 3 * top:
        print(f"    (first {3 * top} of {len(rows)})")
    table(rows[:3 * top], ["clock", "lost", "at", "zone", "killer", "nearest soldier", "nearest turret"], indent="    ")


def spot_owners(g):
    """For each spot, [(frame from, frame to, team)] of the extractors that stood on it (finished)."""
    spots = g.header["spots"]
    own = defaultdict(list)
    for u in g.units.values():
        if u.role != "extractor" or u.finished is None:
            continue
        i = min(range(len(spots)), key=lambda i: dist(spots[i][0], spots[i][1], u.x, u.z)) if spots else None
        if i is None or dist(spots[i][0], spots[i][1], u.x, u.z) > 120:
            continue
        own[i].append((u.finished, u.died if u.died is not None else g.end + 1, u.team))
    return own


def section_spots(g, top):
    spots = g.header["spots"]
    if not spots:
        print("\n== SPOTS: the map publishes none (a metal map) ==")
        return
    own = spot_owners(g)
    print(f"\n== SPOTS ({len(spots)} on the map): held by minute, and the spots of our half that stood free ==")
    rows = []
    for m in range(0, g.end // (60 * FPS) + 1):
        f = m * 60 * FPS
        held = Counter()
        for i in range(len(spots)):
            for a, b, team in own.get(i, ()):
                if a <= f < b:
                    held[team] += 1
        rows.append((m, held[g.us], held[g.them]))
    print("  min:ours/his  " + "  ".join(f"{m}:{a}/{b}" for m, a, b in rows))
    team = g.us
    free_rows = []
    begin = 180 * FPS
    for i, (x, z, _) in enumerate(spots):
        p = g.axis(team, x, z)
        if p > 0.55:
            continue
        taken = sorted((a, b) for a, b, t in own.get(i, ()) if t == team)
        free = []
        cur = begin
        for a, b in taken:
            if a > cur:
                free.append((cur, a))
            cur = max(cur, b)
        if cur < g.end:
            free.append((cur, g.end))
        free_secs = sum(b - a for a, b in free) / FPS
        his = sum(min(b, g.end) - a for a, b, t in own.get(i, ()) if t != team) / FPS
        first = taken[0][0] if taken else None
        lost = sum(1 for a, b in taken if b <= g.end)
        free_rows.append((free_secs, i, x, z, p, first, lost, his, free))
    free_rows.sort(reverse=True)
    total = sum(r[0] for r in free_rows)
    print(f"  spots within 55 % of the way to him: {len(free_rows)}; free of ours from 3:00 on for {total / 60:.0f} spot-minutes of {len(free_rows) * (g.end - begin) / FPS / 60:.0f}")
    out = []
    for free_secs, i, x, z, p, first, lost, his, free in free_rows[:2 * top]:
        out.append([i, f"({x},{z})", f"{100 * p:.0f}%", f"{dist(x, z, *g.start[team]):.0f}", clock(first) if first is not None else "never", lost, fmt_secs(free_secs), fmt_secs(his), " ".join(f"{clock(a)}-{clock(b)}" for a, b in free[:4])])
    table(out, ["spot", "at", "way", "from home", "first taken", "times lost", "free", "his", "free spans"], indent="    ")


def section_wrecks(g, top):
    print("\n== WRECKS: metal lying on the ground by thirds of the way from our start (unreclaimed metal is income not taken) ==")
    rows = []
    for w in g.wrecks:
        if w["f"] % (120 * FPS):
            continue
        z = Counter()
        for fid, name, x, zz, metal in w["w"]:
            z[g.zone(g.us, x, zz)] += metal
        rows.append([clock(w["f"]), z["home"], z["mid"], z["his"], len(w["w"])])
    table(rows, ["clock", "home third", "middle", "his third", "features"])
    if g.wrecks:
        last = g.wrecks[-1]
        big = sorted(last["w"], key=lambda r: -r[4])[:top]
        print("  largest at the end: " + ", ".join(f"{name} {metal} at ({x},{z})" for fid, name, x, z, metal in big))


def section_orders(g, top):
    print("\n== ORDERS: what each side told its units (settings left out), and build orders that never became a frame ==")
    for team in (g.us, g.them):
        kinds = Counter()
        for c in g.cmds:
            if c["team"] == team and c["c"] not in SETTINGS:
                kinds["build" if c["c"] < 0 else CMD.get(c["c"], str(c["c"]))] += 1
        print(f" {g.label(team)}: " + ", ".join(f"{k} {n}" for k, n in kinds.most_common(8)))
    team = g.us
    units = sorted(g.of(team), key=lambda u: -u.orders)[:top]
    print("  most ordered (ours): " + ", ".join(f"{u.name} {u.id} {u.orders} ({u.orders / max(1, ((u.died or g.end) - u.created) / FPS / 60):.0f}/min)" for u in units))
    # build orders by mobile builders against frames begun
    created = defaultdict(list)
    for u in g.of(team):
        created[u.by].append(u)
    failed = Counter()
    repeats = Counter()
    last = {}
    for c in g.cmds:
        if c["team"] != team or c["c"] >= 0 or "x" not in c:
            continue
        builder = g.units.get(c["u"])
        if builder is None or builder.role == "plant":
            continue
        name = g.defs.get(-c["c"], {}).get("name", str(-c["c"]))
        key = (c["u"], c["c"], c["x"], c["z"])
        if last.get(c["u"]) == key:
            repeats[name] += 1
        last[c["u"]] = key
        began = any(v.d["id"] == -c["c"] and dist(v.x, v.z, c["x"], c["z"]) < 40 and v.created >= c["f"] for v in created.get(c["u"], ()))
        assisted = any(v.d["id"] == -c["c"] and dist(v.x, v.z, c["x"], c["z"]) < 40 for v in g.of(team))
        if not began and not assisted:
            failed[name] += 1
    print("  build orders with no frame ever at the site: " + (", ".join(f"{k} {n}" for k, n in failed.most_common(12)) or "none"))
    print("  the same build order given again to the same builder: " + (", ".join(f"{k} {n}" for k, n in repeats.most_common(12)) or "none"))


def section_sight(g, top):
    print("\n== SIGHT: how much of each side the other could see (los or radar), by minute, in % of its units ==")
    rows = []
    for m in range(0, g.end // (60 * FPS) + 1):
        f = m * 60 * FPS
        row = [f"{m}:00"]
        for team in (g.them, g.us):
            n = seen = los = army_n = army_seen = 0
            for u in g.of(team):
                r = u.row_at(f) if u.alive_at(f) else None
                if r is None or r[F] != f:
                    continue
                n += 1
                seen += r[SEEN] != 0
                los += r[SEEN] & 1
                if u.role in ARMED:
                    army_n += 1
                    army_seen += r[SEEN] != 0
            row += [f"{100 * los // max(1, n)}%", f"{100 * seen // max(1, n)}%", f"{100 * army_seen // max(1, army_n)}%" if army_n else "-"]
        rows.append(row)
    table(rows, ["min", "his in our los", "los+radar", "his army seen", "ours in his los", "los+radar", "our army seen"])
    first = None
    for u in g.of(g.them, "plant", "extractor", "energy", "building", "turret", "nano"):
        for r in u.rows:
            if r[SEEN] & 1:
                if first is None or r[F] < first[0]:
                    first = (r[F], u)
                break
    if first:
        print(f"  the first building of his in our line of sight: {first[1].name} at {clock(first[0])}")


def section_summary(g, top):
    print(f"== AUDIT {os.path.basename(os.path.abspath(g.dir))}: {g.header['map']['name']}, {clock(g.end)} recorded, winners {g.winners} ==")
    rows = []
    for team in (g.us, g.them):
        sides = [g.sides[f][team] for f in g.frames]
        st = sides[-1].get("st", {})
        n = len(sides)
        bank = sum(s["m"][0] for s in sides) / n
        full = sum(1 for s in sides if s["m"][0] > 0.9 * max(1, s["m"][1]))
        stall = sum(1 for s in sides if s["e"][0] < 0.05 * max(1, s["e"][1]))
        nominal = delivered = 0.0
        lost_secs = Counter()
        for u in g.of(team, *BUILDERS):
            for f, state, power in u._states:
                nominal += u.d["build_speed"]
                delivered += u.d["build_speed"] * power
                if u.role in ("commander", "constructor") and state in ("idle", "wait", "walk", "guard"):
                    lost_secs[state] += 1
                if u.role == "plant" and state != "build":
                    lost_secs["plant"] += 1
        army = [u for u in g.of(team, *ARMED) if u.finished is not None]
        dead = [u for u in army if u.died]
        nothing = [u for u in dead if u.dealt < 0.1 * u.worth]
        other = [u for u in g.of(team) if u.died is not None and u.role not in ARMED and u.killer_team not in (-1, team)]
        rows.append([
            g.label(team), f"{st.get('metal_produced', 0):.0f}", f"{st.get('metal_excess', 0):.0f}", f"{bank:.0f}", fmt_secs(full), f"{st.get('energy_excess', 0) / 1000:.0f}k", fmt_secs(stall),
            f"{100 * delivered / max(1, nominal):.0f}%", fmt_secs(lost_secs["idle"]), fmt_secs(lost_secs["wait"] + lost_secs["guard"]), fmt_secs(lost_secs["walk"]), fmt_secs(lost_secs["plant"]),
            f"{sum(u.d['metal'] for u in army):.0f}", f"{sum(u.d['metal'] for u in dead):.0f}", f"{sum(u.dealt for u in g.of(team)):.0f}", f"{sum(u.d['metal'] for u in nothing):.0f}", f"{sum(u.d['metal'] for u in other):.0f}",
        ])
    table(rows, ["side", "metal made", "overflowed", "mean bank", "store full", "energy thrown", "energy stalled", "build power used", "builders idle", "waiting", "walking", "plants not building", "army made", "army lost", "damage dealt (metal)", "died for <10%", "other losses"])


SECTIONS = {
    "summary": section_summary, "economy": section_economy, "floats": section_floats, "power": section_power, "builders": section_builders, "plants": section_plants,
    "frames": section_frames, "army": section_army, "front": section_front, "cohesion": section_cohesion, "types": section_types, "trips": section_trips, "fights": section_fights, "raids": section_raids, "spots": section_spots,
    "wrecks": section_wrecks, "orders": section_orders, "sight": section_sight,
}


# ------------------------------------------------------------------------------------------------ drill-downs


def show_unit(g, uid):
    u = g.units[uid]
    print(f"{u.name} {u.id} ({u.role}, team {u.team}): begun {clock(u.created)} by {u.by}" + (f", finished {clock(u.finished)}" if u.finished is not None else "") + (f", died {clock(u.died)} to {g.defs.get(u.killer_d, {}).get('name', '?')} {u.killer}" if u.died else "") + f"; dealt {u.dealt:.0f} taken {u.taken:.0f} metal of damage; {u.orders} orders")
    keyed = []
    prev = None
    for i, r in enumerate(u.rows):
        moved = prev is not None and dist(r[X], r[Z], prev[X], prev[Z]) >= 6
        cmd = r[CMDID]
        what = "-" if not r[QUEUE] else (("build " + g.defs.get(-cmd, {}).get("name", str(-cmd))) if cmd < 0 else CMD.get(cmd, str(cmd)))
        target = (r[TX] // 50, r[TZ] // 50) if (r[TX] or r[TZ]) else r[TARGET]
        builds = g.units[r[BUILDS]].name if r[BUILDS] in g.units else (r[BUILDS] or "")
        fighting = r[F] in u.hit_frames
        keyed.append(((what, target, builds, moved, fighting, r[PROG] < 100), r))
        prev = r
    for key, i, j in runs([k for k, _ in keyed]):
        r0, r1 = keyed[i][1], keyed[j][1]
        what, target, builds, moved, fighting, frame = key
        powers = [keyed[k][1][POWER] for k in range(i, j + 1)]
        print(f"  {clock(r0[F])}-{clock(r1[F])} {j - i + 1:4d} s ({r0[X]},{r0[Z]})->({r1[X]},{r1[Z]}) hp {r0[HP]}->{r1[HP]} {'frame ' if frame else ''}{what}" + (f" to ({r0[TX]},{r0[TZ]})" if r0[TX] or r0[TZ] else (f" target {r0[TARGET]}" if r0[TARGET] else "")) + (f" building {builds} at {sum(powers) // len(powers)}%" if builds else "") + (" moving" if moved else "") + (" FIGHT" if fighting else "") + f" queue {r0[QUEUE]}")


def show_window(g, f0, f1, place=None):
    print(f"window {clock(f0)}-{clock(f1)}" + (f" within {place[2]} of ({place[0]},{place[1]})" if place else ""))
    near = lambda x, z: place is None or dist(x, z, place[0], place[1]) <= place[2]
    for e in g.events:
        if f0 <= e["f"] <= f1 and e["k"] in ("created", "finished", "destroyed") and near(e.get("x", 0), e.get("z", 0)):
            u = g.units[e["u"]]
            extra = ""
            if e["k"] == "destroyed":
                k = g.units.get(e.get("by", 0))
                extra = f" by {k.name} {k.id}" if k else " (no attacker)"
            elif e["k"] == "created":
                b = g.units.get(e.get("by", 0))
                extra = f" by {b.name} {b.id}" if b else ""
            print(f"  {clock(e['f'])} team {e['team']} {e['k']} {u.name} {u.id} at ({e.get('x')},{e.get('z')}){extra}")
    for f in (f0 - f0 % FPS, f1 - f1 % FPS):
        print(f"  at {clock(f)}:")
        for team in g.teams:
            here = Counter()
            for u in g.of(team):
                r = u.row_at(f) if u.alive_at(f) else None
                if r is not None and near(r[X], r[Z]):
                    here[u.name + ("" if r[PROG] >= 100 else "*")] += 1
            print(f"    team {team}: " + ", ".join(f"{n} {k}" for k, n in here.most_common(30)))


def show_map(g, f, cols=96):
    w, h = g.header["map"]["width"], g.header["map"]["height"]
    cell = w / cols
    rows = int(h / cell / 2) + 1  # a character is twice as tall as wide
    grid = [["." for _ in range(cols)] for _ in range(rows)]
    marks = {"commander": "C", "plant": "F", "nano": "n", "constructor": "c", "support": "c", "extractor": "m", "turret": "T", "energy": "e", "converter": "e", "building": "b", "army": "a", "scout": "s", "air": "v", "other": "o"}
    rank = "ebmnTFcsvaC"
    for x, z, _ in g.header["spots"]:
        grid[min(rows - 1, int(z / cell / 2))][min(cols - 1, int(x / cell))] = "+"
    for u in g.units.values():
        r = u.row_at(f) if u.alive_at(f) else None
        if r is None:
            continue
        ch = marks.get(u.role, "o")
        cy, cx = min(rows - 1, int(r[Z] / cell / 2)), min(cols - 1, int(r[X] / cell))
        if u.team != g.us:
            ch = {"C": "@", "F": "#", "n": "N", "c": "K", "m": "M", "T": "X", "e": "E", "b": "B", "a": "A", "s": "S", "v": "V", "o": "O"}[ch]
        old = grid[cy][cx]
        if old in ".+" or (rank.find(ch.lower()) >= rank.find(old.lower())):
            grid[cy][cx] = ch
    print(f"map at {clock(f)}: ours lower case (C commander, F plant, n nano, c constructor, m extractor, T turret, e energy, b building, a army, s scout, v air); his: @ commander, # plant, N K M X E B A S V; + a free spot; a cell is {cell:.0f} x {2 * cell:.0f} elmos")
    for row in grid:
        print("  " + "".join(row))


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    team = int(args[args.index("--team") + 1]) if "--team" in args else None
    top = int(args[args.index("--top") + 1]) if "--top" in args else 12
    g = Game(args[0], team)
    for u in g.units.values():
        STATES[u.id] = builder_states(u) if u.role in BUILDERS else []
    if "--unit" in args:
        return show_unit(g, int(args[args.index("--unit") + 1]))
    if "--window" in args:
        i = args.index("--window")
        rest = [a for a in args[i + 3:i + 6] if re.fullmatch(r"-?\d+", a)]
        return show_window(g, frames(args[i + 1]), frames(args[i + 2]), tuple(int(a) for a in rest) if len(rest) == 3 else None)
    if "--map" in args:
        return show_map(g, frames(args[args.index("--map") + 1]))
    names = args[args.index("--sections") + 1].split(",") if "--sections" in args else list(SECTIONS)
    for name in names:
        SECTIONS[name](g, top)


STATES = {}
Unit._states = property(lambda self: STATES.get(self.id, []))

if __name__ == "__main__":
    main()
