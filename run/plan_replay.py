#!/usr/bin/env python3
"""The engagement plan on a recorded moment (docs/design/2026-09-28-engagement-plan.md, validation): runs the bot's own
code (`bot --plan-replay`, crates/bot/src/brain/pianist/replay.rs) on the record and the Jev log at the clock: the
position, the battlefield in words, the candidates, and Jev's ranking.

    run/plan_replay.py <match dir> <m:ss> [--body group_X] [--dump] [--no-examples] [--both] [--repeat N]

--dump prints the request and asks nothing; --both asks with and without the examples block; --repeat asks N times.
The binary is $BOT_BIN, else $CARGO_TARGET_DIR/release/bot, else target/release/bot (build it with
`cargo build --release -p bot`). The Jev key is read by the Rust client (crates/jev) and never printed.
"""
import os
import sys


def binary():
    here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    candidates = [os.environ.get("BOT_BIN")]
    if os.environ.get("CARGO_TARGET_DIR"):
        candidates.append(os.path.join(os.environ["CARGO_TARGET_DIR"], "release", "bot"))
    candidates.append(os.path.join(here, "target", "release", "bot"))
    for c in candidates:
        if c and os.path.exists(c):
            return c
    sys.exit("no bot binary: cargo build --release -p bot (or set BOT_BIN)")


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    bot = binary()
    os.execv(bot, [bot, "--plan-replay"] + sys.argv[1:])


if __name__ == "__main__":
    main()
