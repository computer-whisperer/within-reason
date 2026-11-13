#!/usr/bin/env python3
"""Which Claude account to put arena and human games on.

usage: claude_account.py [--games N] [--per-game POINTS] [--dir] [CONFIG_DIR ...]
       claude_account.py --self-test
       (default dirs: ~/.claude ~/.claude2, those that exist)

The policy (the user, 2026-11-13): "The basic policy should be taking the soonest-resetting account, so long as it
has enough free opus tokens left. Assume we will be using all fable tokens in a given week, and leave ~10% of the
opus-only credits left at the end." And when no account has that: "In the case of both accounts having the
opus-only budget used up, I would defer to the soonest expiring, unless the 7d meter is fully used up, in which case
we will have to use the remaining account."

How the meters combine (the user, 2026-11-13, on this reading: "matches the anthropic marketing and is correct to
the best of my knowlege"). An account has a 7-day window over every model (the week) and a 7-day cap scoped to
Fable. Fable use counts in both, and the Fable cap is half of the week (`FABLE_SHARE`). So the other half of the
week is the Opus-only credit: nothing but the games here spends it.

    Fable still to come this week = FABLE_SHARE x (100 - fable)          points of the week
    kept back at the end          = RESERVE x (1 - FABLE_SHARE) x 100    (10 % of the Opus-only half: 5 points)
    free for Opus now             = 100 - week - Fable still to come - kept back

The need is `--games` x `--per-game` points. The choice, in order:

1. Of the accounts with that much free for Opus, the one whose week resets soonest: what it has left is lost first.
2. With none, of the accounts whose week has room for the need at all (100 - week), the one that resets soonest.
   The game then spends what Fable was counted on to use, and the output says so.
3. With none, no account: exit 1.

An account that cannot be read (a stale token comes back as HTTP 429) is never chosen. There is no 5-hour window
in the rule: it was retired on these accounts (the user, 2026-11-13).

`--per-game` defaults to 1.5 points of the week for one game with one Opus 5.5 player (the five games of
2026-11-11 to 13 on one account came to about 5.5 points, the week less half of Fable); a game with a player a seat
is as many games as it has seats. `--dir` prints the chosen directory alone (the table goes to stderr), for
`--claude-config-dir "$(run/claude_account.py --dir)"`, and exits 1 when there is no account. Snapshot and `--since`
with run/claude_usage.py around every game as before: one of the accounts can be charged beyond its plan.
"""
import os, sys
from datetime import datetime

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from claude_usage import extra_usage, fetch, flag_value, meters, resets_in  # noqa: E402

FABLE_SHARE = 0.5
RESERVE = 0.10
PER_GAME = 1.5


def free_for_opus(week, fable):
    """Points of the week free for Opus now: what is left of it, less the Fable still to come (all of Fable is
    used by the week's end; an account whose reply names no Fable cap has all of it to come) and the reserve."""
    to_come = FABLE_SHARE * (100.0 - (fable or 0.0))
    kept = RESERVE * (1.0 - FABLE_SHARE) * 100.0
    return 100.0 - week - to_come - kept


def choose(accounts, need):
    """The account for `need` points and the rule that chose it: `accounts` is [(name, week, fable, resets_at)],
    the readable ones. ("opus", name): it has the need free for Opus, the soonest to reset of those. ("week", name):
    none has, and it is the soonest to reset of those whose week has room for the need. (None, None): no account."""
    far = datetime.max.isoformat()
    soonest = lambda rows: min(rows, key=lambda a: a[3] or far)[0]
    with_opus = [a for a in accounts if free_for_opus(a[1], a[2]) >= need]
    if with_opus:
        return "opus", soonest(with_opus)
    with_room = [a for a in accounts if 100.0 - a[1] >= need]
    if with_room:
        return "week", soonest(with_room)
    return None, None


def self_test():
    """The choice on made-up meters: `claude_account.py --self-test`."""
    late, soon = "2026-11-18T23:00:00+00:00", "2026-11-13T19:00:00+00:00"
    cases = [
        ("both have it free for Opus: the sooner to reset", [("a", 47, 84, late), ("b", 20, 40, soon)], ("opus", "b")),
        ("the sooner is out of Opus: the other", [("a", 47, 84, late), ("b", 100, 42, soon)], ("opus", "a")),
        ("both out of Opus: the sooner with room in its week", [("a", 90, 60, late), ("b", 80, 20, soon)], ("week", "b")),
        ("both out of Opus and the sooner's week is full: the remaining one", [("a", 90, 60, late), ("b", 100, 42, soon)], ("week", "a")),
        ("both weeks full: none", [("a", 99.5, 99, late), ("b", 100, 42, soon)], (None, None)),
        ("an account with no week begun goes last", [("a", 47, 84, late), ("b", 0, None, None)], ("opus", "a")),
        ("inside the reserve of 5: by the week's room alone", [("a", 94, 100, late)], ("week", "a")),
        ("outside the reserve", [("a", 93, 100, late)], ("opus", "a")),
        ("nobody readable", [], (None, None)),
    ]
    for name, accounts, want in cases:
        got = choose(accounts, 1.5)
        assert got == want, (name, got, want)
    print(f"{len(cases)} cases pass")


def main():
    if "--self-test" in sys.argv:
        return self_test()
    games = float(flag_value("--games") or 1)
    per_game = float(flag_value("--per-game") or PER_GAME)
    only_dir = "--dir" in sys.argv
    args = [a for a in sys.argv[1:] if a != "--dir"]
    dirs = args or [d for d in (os.path.expanduser("~/.claude"), os.path.expanduser("~/.claude2")) if os.path.isdir(d)]
    need = games * per_game
    usages = {d: fetch(d) for d in dirs}
    rule, chosen = choose([(d, *meters(u)) for d, u in usages.items() if "error" not in u], need)
    # The table: to stderr under --dir, so that the directory is the only thing on stdout.
    out = sys.stderr if only_dir else sys.stdout
    print(f"needed: {need:.1f} points of a week ({games:g} game{'s' if games != 1 else ''} at {per_game:g}); "
          f"Fable takes {FABLE_SHARE:.0%} of a week, {RESERVE:.0%} of the rest is kept back", file=out)
    for d, usage in usages.items():
        if "error" in usage:
            print(f"  {d}: cannot be read: {usage['error']}", file=out)
            continue
        week, fable, resets_at = meters(usage)
        charged = "can be charged beyond the plan" if extra_usage(usage)[0] else "cannot be charged"
        fable_words = f"Fable {fable:.0f}%" if fable is not None else "no Fable cap in the reply (all of it counted as to come)"
        print(f"  {d}: week {week:.0f}%, {fable_words}, {resets_in(resets_at) or 'no week begun'}; "
              f"free for Opus {free_for_opus(week, fable):.1f} points, {100.0 - week:.0f} left of the week; {charged}{'  <- chosen' if d == chosen else ''}", file=out)
    if chosen is None:
        print("no account has room in its week for this: no game", file=out)
        sys.exit(1)
    if rule == "week":
        print("no account has this much free for Opus: the soonest to reset with room in its week is taken, and the game spends what Fable was counted on to use", file=out)
    if only_dir:
        print(chosen)
    else:
        print(f"use: --claude-config-dir {chosen}")


if __name__ == "__main__":
    main()
