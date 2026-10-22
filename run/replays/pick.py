#!/usr/bin/env python3
"""Pick people's replays worth learning from and keep the manifest (docs/design/2026-09-23-replay-survey.md, decision 1).

    run/replays/pick.py [--map "Comet Catcher Remake 1.8"]... [--preset duel] [--pages 3] [--min-os 25]
                          [--min-minutes 6] [--max-minutes 40] [--manifest run/data/replays/manifest.jsonl]
                          [--bots] [--any-ending] [--min-mean-os 0] [--sides 8v8]
    run/replays/pick.py --id <match id>... [the same floors]      judge the matches named instead of paging
    run/replays/pick.py --demo <file.sdfz>... [--manifest ...]     a demo from the user's client into the manifest

Pages BAR's replay API (https://api.bar-rts.com/replays) for each map given (every map when none), reads the detail
of each match not yet in the manifest, and appends one line per match: kept or not and why. Kept: the preset, no
bots (with `--bots`: bots, the AI seats written as players with `ai` their short name), ended normally (`--any-ending`
keeps a game the host stopped: how most games against AIs end), no tweaks (unranked games are kept and marked: a
high-OS custom lobby still teaches) (`tweakunits`, `tweakdefs` empty), the standard start (1000 metal), the duration
within the range, every player's OS at or above the floor and their mean at or above `--min-mean-os`, the ally teams'
sizes as `--sides` says ("8v8": players and AIs together) when given. The line carries what the card needs and the
records lack: the players (name, OS, rank, faction, team, ally team, start position), the map's script and file
name, the game and engine versions, the demo's file name, `sides` (the shape) and `humans_vs_ais` (people on one
side, AIs on the other). Re-running adds new matches and keeps the old lines; a `--bots` run judges again the lines
an earlier run rejected for their bots.

`--demo` reads a demo's header and start script (run/replays/startscript.py): the match id is the demo's game id (the
API's id for the same game), the players and AIs come from the script, the winner from the demo's tail when the game
ended, and the demo is copied to run/data/replays/<map file>/<file> where the fetcher expects it.
"""
import json, os, shutil, statistics, sys, time, urllib.parse, urllib.request

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import startscript  # noqa: E402

API = "https://api.bar-rts.com/replays"
DEMOS = "https://storage.uk.cloud.ovh.net/v1/AUTH_10286efc0d334efd917d476d7183232e/BAR/demos/"
PAUSE = 0.25


def get(url, tries=3):
    """One GET as JSON; the API is slow at times (a 50-row page timed out at 30 s), so a longer wait and retries."""
    req = urllib.request.Request(url, headers={"User-Agent": "within-reason replay survey (github.com/computer-whisperer/within-reason)"})
    for attempt in range(tries):
        try:
            with urllib.request.urlopen(req, timeout=90) as r:
                return json.load(r)
        except (TimeoutError, urllib.error.URLError, json.JSONDecodeError) as e:
            if attempt + 1 == tries:
                raise
            print(f"  retrying {url.split('?')[0].rsplit('/', 1)[-1]}: {e}", file=sys.stderr)
            time.sleep(2 * (attempt + 1))


def os_of(player):
    """The OS as a number: the API writes it as "[35.66]"."""
    try:
        return float(str(player.get("skill", "")).strip("[]"))
    except ValueError:
        return None


def shape(detail):
    """"8v8": the seats (players and AIs) per ally team, largest side first is not wanted: in the API's ally order."""
    return "v".join(str(len(at.get("Players", [])) + len(at.get("AIs", []))) for at in detail.get("AllyTeams", []))


def judge(detail, preset, min_os, min_minutes, max_minutes, bots=False, any_ending=False, min_mean_os=0.0, sides=None):
    if detail.get("hasBots") and not bots:
        return "bots"
    if bots and not detail.get("hasBots"):
        return "no bots"
    if detail.get("preset") != preset:
        return f"preset {detail.get('preset')}"
    if not detail.get("gameEndedNormally") and not any_ending:
        return "did not end normally"
    settings = detail.get("gameSettings") or {}
    if settings.get("tweakunits") or settings.get("tweakdefs"):
        return "tweaked units"
    if str(settings.get("startmetal", "1000")) != "1000":
        return f"start metal {settings.get('startmetal')}"
    minutes = (detail.get("durationMs") or 0) / 60000
    if not min_minutes <= minutes <= max_minutes:
        return f"{minutes:.0f} min"
    if sides and shape(detail) != sides:
        return f"sides {shape(detail)}"
    players = [p for at in detail.get("AllyTeams", []) for p in at.get("Players", [])]
    if not players:
        return "no players"
    low = [p for p in players if os_of(p) is None or os_of(p) < min_os]
    if low:
        return "OS " + ", ".join(f"{p['name']} {p.get('skill')}" for p in low)
    known = [os_of(p) for p in players if os_of(p) is not None]
    if min_mean_os and (not known or statistics.mean(known) < min_mean_os):
        return f"mean OS {statistics.mean(known):.1f}" if known else "no OS"
    return None


def line_of(detail, why):
    players = []
    winner = None
    kinds = {}
    for at in detail.get("AllyTeams", []):
        kinds[at.get("allyTeamId")] = {"players": len(at.get("Players", [])), "ais": len(at.get("AIs", []))}
        for p in at.get("Players", []):
            # A player's own allyTeamId is the database id of its ally team row; the ally team's allyTeamId is the
            # in-game number (which `winningTeam` names), so every seat takes the latter.
            players.append({"name": p.get("name"), "os": os_of(p), "rank": p.get("rank"), "faction": p.get("faction"), "team": p.get("teamId"), "ally": at.get("allyTeamId"), "start": p.get("startPos")})
        for a in at.get("AIs", []):
            # The API gives an AI its ally team and not its team number; the card reads the number from the start script.
            players.append({"name": a.get("name") or a.get("shortName"), "os": None, "rank": None, "faction": a.get("faction"), "team": None, "ally": at.get("allyTeamId"), "start": a.get("startPos"), "ai": a.get("shortName"), "handicap": a.get("handicap")})
        if at.get("winningTeam"):
            winner = at.get("allyTeamId")
    m = detail.get("Map") or {}
    settings = detail.get("gameSettings") or {}
    winners = [p["name"] for p in players if p["ally"] == winner]
    humans_vs_ais = len(kinds) == 2 and any(k["ais"] and not k["players"] for k in kinds.values()) and any(k["players"] and not k["ais"] for k in kinds.values())
    return {
        "id": detail["id"], "start_time": detail.get("startTime"), "duration_ms": detail.get("durationMs"), "preset": detail.get("preset"),
        "map_script": m.get("scriptName"), "map_file": m.get("fileName"), "map_size": [m.get("width"), m.get("height")],
        "game_version": detail.get("gameVersion"), "engine_version": detail.get("engineVersion"),
        "file": detail.get("fileName"), "url": DEMOS + urllib.parse.quote(detail.get("fileName") or ""),
        "players": players, "winner_ally": winner, "winners": winners, "ranked": settings.get("ranked_game") in ("1", 1, True),
        "sides": shape(detail), "humans_vs_ais": humans_vs_ais, "ended": bool(detail.get("gameEndedNormally")),
        "keep": why is None, "why": why or "kept",
    }


def line_of_demo(path):
    """A manifest line from a demo on disk: the user's client keeps every game it played or watched."""
    header, game = startscript.load_script(path)
    seats = startscript.seats(game)
    players = startscript.players_of(seats)
    for p in players:
        p["start"] = None
    sides = startscript.shape_of(seats)
    counts = {}
    for p in players:
        c = counts.setdefault(p["ally"], {"players": 0, "ais": 0})
        c["ais" if p.get("ai") else "players"] += 1
    humans_vs_ais = len(counts) == 2 and any(k["ais"] and not k["players"] for k in counts.values()) and any(k["players"] and not k["ais"] for k in counts.values())
    winner = header["winners"][0] if header.get("winners") else None
    n_ally = len(counts)
    preset = "duel" if sides == "1v1" else "ffa" if n_ally > 2 else "team"
    map_script = game.get("mapname") or "?"
    return {
        "id": header["game_id"], "start_time": time.strftime("%Y-%m-%dT%H:%M:%S.000Z", time.gmtime(header["unix_time"])), "duration_ms": header["game_seconds"] * 1000, "preset": preset,
        "map_script": map_script, "map_file": startscript.map_file_of(map_script), "map_size": None,
        "game_version": game.get("gametype"), "engine_version": header.get("engine"),
        "file": os.path.basename(path), "url": None, "source": "client",
        "players": players, "winner_ally": winner, "winners": [p["name"] for p in players if winner is not None and p["ally"] == winner], "ranked": False,
        "sides": sides, "humans_vs_ais": humans_vs_ais, "ended": bool(header.get("winners")),
        "keep": True, "why": "local demo",
    }


def main():
    args = sys.argv[1:]
    def opt(name, default):
        return args[args.index(name) + 1] if name in args else default
    maps = [args[i + 1] for i, a in enumerate(args) if a == "--map"]
    preset = opt("--preset", "duel")
    pages = int(opt("--pages", "3"))
    min_os = float(opt("--min-os", "25"))
    min_minutes = float(opt("--min-minutes", "6"))
    max_minutes = float(opt("--max-minutes", "40"))
    min_mean_os = float(opt("--min-mean-os", "0"))
    sides = opt("--sides", None)
    bots = "--bots" in args
    any_ending = "--any-ending" in args
    manifest = opt("--manifest", os.path.join(REPO, "run", "data", "replays", "manifest.jsonl"))
    os.makedirs(os.path.dirname(manifest), exist_ok=True)
    rows = []
    if os.path.exists(manifest):
        with open(manifest) as f:
            rows = [json.loads(line) for line in f if line.strip()]
    known = {r["id"] for r in rows}
    # A `--bots` run judges again what an earlier run turned away for its bots alone.
    rejudge = {r["id"] for r in rows if bots and r.get("why") == "bots"}
    known -= rejudge
    ids = [args[i + 1] for i, a in enumerate(args) if a == "--id"]
    demos = [args[i + 1] for i, a in enumerate(args) if a == "--demo"]
    added = kept = 0
    new_rows = []

    def take(entry):
        nonlocal added, kept
        new_rows.append(entry)
        known.add(entry["id"])
        added += 1
        kept += entry["keep"]
        print(f"{entry['id'][:8]} {entry['map_script']} {(entry['duration_ms'] or 0) // 60000} min {entry.get('sides', '')}: {entry['why']}")

    for path in demos:
        entry = line_of_demo(path)
        if entry["id"] in known:
            print(f"{entry['id'][:8]} already in the manifest ({os.path.basename(path)})")
            continue
        dest = os.path.join(REPO, "run", "data", "replays", entry["map_file"], entry["file"])
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        if not os.path.exists(dest):
            shutil.copy(path, dest)
        entry["demo"] = os.path.relpath(dest, REPO)
        take(entry)
    # Named matches (a player's games found by scanning the list: the API's player filter is a no-op, 2026-09-24).
    for match_id in ids:
        if match_id in known:
            print(f"{match_id[:8]} already in the manifest")
            continue
        detail = get(f"{API}/{match_id}")
        time.sleep(PAUSE)
        take(line_of(detail, judge(detail, preset, min_os, min_minutes, max_minutes, bots, any_ending, min_mean_os, sides)))
    for map_name in (maps or [None]) if not ids and not demos else []:
        for page in range(1, pages + 1):
            query = {"page": page, "limit": 50 if bots or sides else 20, "preset": preset, "hasBots": "true" if bots else "false"}
            if map_name:
                query["maps"] = map_name
            page_rows = get(API + "?" + urllib.parse.urlencode(query)).get("data") or []
            time.sleep(PAUSE)
            if not page_rows:
                break
            for row in page_rows:
                if row["id"] in known:
                    continue
                if sides and shape(row) != sides:
                    # The list row carries the ally teams: a game of the wrong shape needs no detail read.
                    take({"id": row["id"], "start_time": row.get("startTime"), "duration_ms": row.get("durationMs"), "preset": preset, "map_script": (row.get("Map") or {}).get("scriptName"), "map_file": (row.get("Map") or {}).get("fileName"), "players": [], "sides": shape(row), "keep": False, "why": f"sides {shape(row)}"})
                    continue
                detail = get(f"{API}/{row['id']}")
                time.sleep(PAUSE)
                take(line_of(detail, judge(detail, preset, min_os, min_minutes, max_minutes, bots, any_ending, min_mean_os, sides)))
    if rejudge or new_rows:
        # Written whole only when a rejected line is replaced; otherwise appended, so a fetcher's or runner's rewrite
        # of its own copy (docs/harness/pitfalls.md) meets the fewest surprises.
        replaced = {e["id"] for e in new_rows} & rejudge
        if replaced:
            rows = [r for r in rows if r["id"] not in replaced]
            with open(manifest, "w") as out:
                for r in rows + new_rows:
                    out.write(json.dumps(r) + "\n")
        else:
            with open(manifest, "a") as out:
                for r in new_rows:
                    out.write(json.dumps(r) + "\n")
    print(f"{added} matches examined, {kept} kept; manifest {os.path.relpath(manifest, REPO)} now {len(known)} lines")


if __name__ == "__main__":
    main()
