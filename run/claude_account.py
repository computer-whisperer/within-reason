#!/usr/bin/env python3
"""Which Claude account to put arena games on.

usage: claude_account.py [--games N] [--per-game POINTS] [--parallel P] [--dir] [CONFIG_DIR ...]
       (default dirs: ~/.claude ~/.claude2, those that exist)

The policy (the user, 2026-11-13): "The basic policy should be taking the soonest-resetting account, so long as it
has enough free opus tokens left. Assume we will be using all fable tokens in a given week, and leave ~10% of the
opus-only credits left at the end."

How the meters combine, as this tool reads them. An account has a 7-day window over every model (the week) and a
7-day cap scoped to Fable. Fable use counts in both, and the Fable cap is half of the week (`FABLE_SHARE`;
docs/harness/claude-p.md, the user 2026-09-19: "its own cap of up to half the weekly allotment"; the snapshots of
2026-11-11 to 13 on one account: 9 of the week at 18 of Fable with nothing else used, 47 at 83 after five Opus
games). So the other half of the week is the Opus-only credit: nothing but the games here spends it.

    Fable still to come this week = FABLE_SHARE x (100 - fable)          points of the week
    kept back at the end          = RESERVE x (1 - FABLE_SHARE) x 100    (10 % of the Opus-only half: 5 points)
    free for Opus now             = 100 - week - Fable still to come - kept back

An account has enough when that is at least `--games` x `--per-game` points, and when its 5-hour window, where the
reply carries one, has room for `--parallel` games started now (`FIVE_HOUR_PER_GAME` each, under `FIVE_HOUR_CEILING`:
four Opus players in parallel filled one account's window in 70 minutes on 2026-10-23 and tipped it into paid
credits). Of the accounts with enough, the one whose week resets soonest is chosen: what it has left is lost first.
An account that cannot be read (a stale token comes back as HTTP 429) is never chosen.

`--per-game` defaults to 1.5 points of the week for one 1v1 game with an Opus 5.5 player (the five games of
2026-11-11 to 13 on one account came to about 5.5 points, the week less half of Fable); a team game with a player a
seat costs a multiple and is the caller's to say. `--dir` prints the chosen directory alone, for
`--claude-config-dir "$(run/claude_account.py --dir)"`, and exits 1 with nothing printed when no account has enough.
Snapshot and `--since` with run/claude_usage.py around every game as before: one of the accounts can be charged.
"""
import os, sys
from datetime import datetime

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from claude_usage import extra_usage, fetch, flag_value, meters, resets_in  # noqa: E402

FABLE_SHARE = 0.5
RESERVE = 0.10
PER_GAME = 1.5
FIVE_HOUR_PER_GAME = 8.0
FIVE_HOUR_CEILING = 90.0


def free_for_opus(week, fable):
    """Points of the week free for Opus now: what is left of it, less the Fable still to come (all of Fable is
    used by the week's end; an account whose reply names no Fable cap has all of it to come) and the reserve."""
    to_come = FABLE_SHARE * (100.0 - (fable or 0.0))
    kept = RESERVE * (1.0 - FABLE_SHARE) * 100.0
    return 100.0 - week - to_come - kept


def judge(usage, need, parallel):
    """(free points or None, enough, why not, when the week resets)."""
    if "error" in usage:
        return None, False, usage["error"], None
    week, fable, five, resets_at = meters(usage)
    free = free_for_opus(week, fable)
    if free < need:
        return free, False, f"{free:.1f} points free for Opus, {need:.1f} needed", resets_at
    if five is not None and five + FIVE_HOUR_PER_GAME * parallel > FIVE_HOUR_CEILING:
        return free, False, f"its 5-hour window is at {five:.0f}%", resets_at
    return free, True, "", resets_at


def main():
    games = float(flag_value("--games") or 1)
    per_game = float(flag_value("--per-game") or PER_GAME)
    parallel = int(flag_value("--parallel") or 1)
    only_dir = "--dir" in sys.argv
    args = [a for a in sys.argv[1:] if a != "--dir"]
    dirs = args or [d for d in (os.path.expanduser("~/.claude"), os.path.expanduser("~/.claude2")) if os.path.isdir(d)]
    need = games * per_game
    rows = []
    for d in dirs:
        usage = fetch(d)
        free, enough, why, resets_at = judge(usage, need, parallel)
        rows.append((d, usage, free, enough, why, resets_at))
    # The soonest reset first; an account with no week begun yet has nothing to lose and goes last.
    far = datetime.max.isoformat()
    able = sorted((r for r in rows if r[3]), key=lambda r: r[5] or far)
    chosen = able[0][0] if able else None
    if only_dir:
        if chosen is None:
            sys.exit(1)
        print(chosen)
        return
    print(f"needed: {need:.1f} points of a week ({games:g} game{'s' if games != 1 else ''} at {per_game:g}); "
          f"Fable takes {FABLE_SHARE:.0%} of a week, {RESERVE:.0%} of the rest is kept back")
    for d, usage, free, enough, why, resets_at in rows:
        if free is None:
            print(f"  {d}: cannot be read: {why}")
            continue
        week, fable, five, _ = meters(usage)
        charged = "can be charged beyond the plan" if extra_usage(usage)[0] else "cannot be charged"
        fable_words = f"Fable {fable:.0f}%" if fable is not None else "no Fable cap in the reply (all of it counted as to come)"
        five_words = f", 5-hour {five:.0f}%" if five is not None else ""
        print(f"  {d}: week {week:.0f}%, {fable_words}{five_words}, {resets_in(resets_at) or 'no week begun'}; "
              f"free for Opus {free:.1f} points; {charged}{'' if enough else '; NOT ENOUGH: ' + why}{'  <- chosen' if d == chosen else ''}")
    if chosen is None:
        print("no account has enough: no game")
        sys.exit(1)
    print(f"use: --claude-config-dir {chosen}")


if __name__ == "__main__":
    main()
