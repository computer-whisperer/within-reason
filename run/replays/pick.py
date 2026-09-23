#!/usr/bin/env python3
"""Pick people's replays worth learning from and keep the manifest (docs/design/2026-09-23-replay-survey.md, decision 1).

    run/replays/pick.py [--map "Comet Catcher Remake 1.8"]... [--preset duel] [--pages 3] [--min-os 25]
                          [--min-minutes 6] [--max-minutes 40] [--manifest run/data/replays/manifest.jsonl]

Pages BAR's replay API (https://api.bar-rts.com/replays) for each map given (every map when none), reads the detail
of each match not yet in the manifest, and appends one line per match: kept or not and why. Kept: the preset, no
bots, ended normally, no tweaks (unranked games are kept and marked: a high-OS custom lobby still teaches) (`tweakunits`, `tweakdefs` empty), the standard start (1000 metal), the
duration within the range, every player's OS at or above the floor. The line carries what the card needs and the
records lack: the players (name, OS, rank, faction, team, ally team, start position), the map's script and file
name, the game and engine versions, the demo's file name. Re-running adds new matches and keeps the old lines.
"""
import json, os, sys, time, urllib.parse, urllib.request

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
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


def judge(detail, preset, min_os, min_minutes, max_minutes):
    if detail.get("hasBots"):
        return "bots"
    if detail.get("preset") != preset:
        return f"preset {detail.get('preset')}"
    if not detail.get("gameEndedNormally"):
        return "did not end normally"
    settings = detail.get("gameSettings") or {}
    if settings.get("tweakunits") or settings.get("tweakdefs"):
        return "tweaked units"
    if str(settings.get("startmetal", "1000")) != "1000":
        return f"start metal {settings.get('startmetal')}"
    minutes = (detail.get("durationMs") or 0) / 60000
    if not min_minutes <= minutes <= max_minutes:
        return f"{minutes:.0f} min"
    players = [p for at in detail.get("AllyTeams", []) for p in at.get("Players", [])]
    if not players:
        return "no players"
    low = [p for p in players if os_of(p) is None or os_of(p) < min_os]
    if low:
        return "OS " + ", ".join(f"{p['name']} {p.get('skill')}" for p in low)
    return None


def line_of(detail, why):
    players = []
    winner = None
    for at in detail.get("AllyTeams", []):
        for p in at.get("Players", []):
            players.append({"name": p.get("name"), "os": os_of(p), "rank": p.get("rank"), "faction": p.get("faction"), "team": p.get("teamId"), "ally": p.get("allyTeamId"), "start": p.get("startPos")})
            if at.get("winningTeam"):
                winner = p.get("allyTeamId")
    m = detail.get("Map") or {}
    settings = detail.get("gameSettings") or {}
    winners = [p["name"] for p in players if p["ally"] == winner]
    return {
        "id": detail["id"], "start_time": detail.get("startTime"), "duration_ms": detail.get("durationMs"), "preset": detail.get("preset"),
        "map_script": m.get("scriptName"), "map_file": m.get("fileName"), "map_size": [m.get("width"), m.get("height")],
        "game_version": detail.get("gameVersion"), "engine_version": detail.get("engineVersion"),
        "file": detail.get("fileName"), "url": DEMOS + urllib.parse.quote(detail.get("fileName") or ""),
        "players": players, "winner_ally": winner, "winners": winners, "ranked": settings.get("ranked_game") in ("1", 1, True),
        "keep": why is None, "why": why or "kept",
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
    manifest = opt("--manifest", os.path.join(REPO, "run", "data", "replays", "manifest.jsonl"))
    os.makedirs(os.path.dirname(manifest), exist_ok=True)
    known = set()
    if os.path.exists(manifest):
        with open(manifest) as f:
            for line in f:
                if line.strip():
                    known.add(json.loads(line)["id"])
    added = kept = 0
    with open(manifest, "a") as out:
        for map_name in maps or [None]:
            for page in range(1, pages + 1):
                query = {"page": page, "limit": 20, "preset": preset, "hasBots": "false"}
                if map_name:
                    query["maps"] = map_name
                rows = get(API + "?" + urllib.parse.urlencode(query)).get("data") or []
                time.sleep(PAUSE)
                if not rows:
                    break
                for row in rows:
                    if row["id"] in known:
                        continue
                    detail = get(f"{API}/{row['id']}")
                    time.sleep(PAUSE)
                    why = judge(detail, preset, min_os, min_minutes, max_minutes)
                    entry = line_of(detail, why)
                    out.write(json.dumps(entry) + "\n")
                    out.flush()
                    known.add(row["id"])
                    added += 1
                    kept += entry["keep"]
    print(f"{added} matches examined, {kept} kept; manifest {os.path.relpath(manifest, REPO)} now {len(known)} lines")


if __name__ == "__main__":
    main()
