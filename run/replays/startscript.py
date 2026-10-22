#!/usr/bin/env python3
"""The start script of a game: who sat where. Read from a demo's header (the script the lobby wrote, before the
stream) or from the `_script.txt` the engine writes beside a replay's records. Used by run/replays/pick.py (a local
demo into the manifest) and run/replays/card.py (the seats of a replayed game: the replay API names an AI's ally
team but not its team number, and a demo from the user's client is in no API at all).

    run/replays/startscript.py <demo.sdfz | _script.txt>      print the seats

The demo header (the engine's DemoFileHeader): magic[16], version, headerSize, versionString[256], gameID[16],
unixTime u64, then scriptSize, demoStreamSize, gameTime, wallclockTime, numPlayers, playerStatSize,
playerStatElemSize, numTeams, teamStatSize, teamStatElemSize, teamStatPeriod, winningAllyTeamsSize; the script
follows at headerSize, the stream after it, then the winning ally teams (winningAllyTeamsSize bytes: the recorder
writes them before the player and team statistics, DemoRecorder.cpp; none when the game was stopped before its
end), the player statistics and the team statistics (every teamStatPeriod seconds, cumulative: metal and energy
used, produced, excess, received and sent, damage dealt and received, units produced, died, received, sent,
captured, out-captured, killed; a reading of the transfers without a replay, not used yet).
"""
import gzip, json, os, re, struct, sys

HEADER_FIXED = 16 + 4 + 4 + 256 + 16 + 8


def read_demo(path, winners=False):
    """(header, script text) of a demo; with `winners`, the header carries `winners` (ally team numbers) too."""
    with gzip.open(path, "rb") as f:
        head = f.read(HEADER_FIXED + 12 * 4)
        version, header_size = struct.unpack_from("<ii", head, 16)
        engine = head[24:280].split(b"\0", 1)[0].decode("utf-8", "replace")
        game_id = head[280:296].hex()
        (unix_time,) = struct.unpack_from("<Q", head, 296)
        fields = struct.unpack_from("<12i", head, 304)
        names = ("script_size", "stream_size", "game_seconds", "wall_seconds", "num_players", "player_stat_size", "player_stat_elem_size", "num_teams", "team_stat_size", "team_stat_elem_size", "team_stat_period", "winning_ally_teams_size")
        header = {"version": version, "header_size": header_size, "engine": engine, "game_id": game_id, "unix_time": unix_time, **dict(zip(names, fields))}
        f.seek(header_size)
        script = f.read(header["script_size"]).decode("utf-8", "replace")
        if winners:
            n = header["winning_ally_teams_size"]
            header["winners"] = []
            if n > 0:
                f.seek(header_size + header["script_size"] + header["stream_size"])
                header["winners"] = list(f.read(n))
    return header, script


def parse(text):
    """The script's sections as nested dicts: keys lower-cased, values as strings, sections by their lower-cased
    name ("game" at the top holds the top-level keys; "player0", "ai0", "team0", "allyteam0", "modoptions" under it)."""
    tokens = re.findall(r"\[([^\]]+)\]|(\{)|(\})|([^=;{}\[\]\s][^=;{}]*)=([^;]*);", text)
    root, stack, pending = {}, [], None
    for section, open_, close, key, value in tokens:
        if section:
            pending = section.strip().lower()
        elif open_:
            node = {}
            (stack[-1] if stack else root)[pending or "?"] = node
            stack.append(node)
            pending = None
        elif close:
            if stack:
                stack.pop()
        elif key:
            (stack[-1] if stack else root)[key.strip().lower()] = value.strip()
    return root.get("game", root)


def _int(v, default=None):
    try:
        return int(float(v))
    except (TypeError, ValueError):
        return default


def _skill(v):
    try:
        return float(str(v).strip("[]"))
    except (TypeError, ValueError):
        return None


def seats(game):
    """One row per team of the script, in team order: team, ally (the script's ally team number), side, handicap,
    and the controller: {"person": {name, skill, rank}} for the first non-spectating player of the team, or
    {"ai": {name, short_name, version, profile}}; None for a team nobody plays."""
    teams = {}
    for name, body in game.items():
        m = re.fullmatch(r"team(\d+)", name)
        if m and isinstance(body, dict):
            n = int(m.group(1))
            teams[n] = {"team": n, "ally": _int(body.get("allyteam")), "side": body.get("side"), "handicap": _int(body.get("handicap"), 0), "controller": None}
    for name, body in sorted(game.items()):
        m = re.fullmatch(r"player(\d+)", name)
        if m and isinstance(body, dict) and body.get("spectator", "0") != "1":
            t = _int(body.get("team"))
            if t in teams and teams[t]["controller"] is None:
                teams[t]["controller"] = {"person": {"name": body.get("name"), "skill": _skill(body.get("skill")), "rank": _int(body.get("rank")), "account": body.get("accountid")}}
    for name, body in sorted(game.items()):
        m = re.fullmatch(r"ai(\d+)", name)
        if m and isinstance(body, dict):
            t = _int(body.get("team"))
            if t in teams:
                options = body.get("options") if isinstance(body.get("options"), dict) else {}
                teams[t]["controller"] = {"ai": {"name": body.get("name"), "short_name": body.get("shortname"), "version": body.get("version"), "profile": options.get("profile")}}
    return [teams[k] for k in sorted(teams)]


def players_of(seat_rows):
    """The manifest's `players` rows from seats: name, os, rank, faction, team, ally, ai (the AI's short name or
    None), handicap. The person's name, or the AI's instance name ("SMRTBARbstable(7)")."""
    out = []
    for s in seat_rows:
        c = s.get("controller") or {}
        if "person" in c:
            p = c["person"]
            out.append({"name": p["name"], "os": p.get("skill"), "rank": p.get("rank"), "faction": s["side"], "team": s["team"], "ally": s["ally"], "ai": None, "handicap": s["handicap"]})
        elif "ai" in c:
            a = c["ai"]
            out.append({"name": a.get("name") or a.get("short_name"), "os": None, "rank": None, "faction": s["side"], "team": s["team"], "ally": s["ally"], "ai": a.get("short_name"), "ai_version": a.get("version"), "ai_profile": a.get("profile"), "handicap": s["handicap"]})
    return out


def shape_of(seat_rows):
    """"8v8", "2v16": the seats per ally team, in ally order."""
    counts = {}
    for s in seat_rows:
        counts[s["ally"]] = counts.get(s["ally"], 0) + 1
    return "v".join(str(counts[k]) for k in sorted(counts, key=lambda a: (a is None, a)))


def map_file_of(script_name):
    """The replay API's map file name from a script name: lower case, spaces as underscores (how the API names
    every map we have met; the pool's archives agree for the fifteen maps on disk)."""
    return re.sub(r"\s+", "_", script_name.strip()).lower()


def load_script(path):
    """A demo or a `_script.txt`: (header or None, parsed game section)."""
    if path.endswith(".sdfz"):
        header, text = read_demo(path, winners=True)
    else:
        header, text = None, open(path, encoding="utf-8", errors="replace").read()
    return header, parse(text)


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    header, game = load_script(sys.argv[1])
    rows = seats(game)
    if header:
        print(json.dumps({k: v for k, v in header.items() if k != "script"}, indent=1))
    print(game.get("mapname"), "|", game.get("gametype"), "|", shape_of(rows))
    for s in rows:
        print(json.dumps(s))


if __name__ == "__main__":
    main()
