#!/bin/sh
# One row of the distilled bot's tier-2 study (bots/distilled/README.md, "Tier 2"): eight games of WRDistilled against
# a BARb profile with the arms of WITHIN_REASON_DISTILLED set for the whole batch.
# usage: run/distilled_row.sh "<arms>" <label> [arena flags...]
#   e.g. run/distilled_row.sh "t2=30" distilled-t2-30-hard --profile hard_aggressive --base-port 9500
# Defaults: Comet Catcher, eight games in parallel at speed 50, a 40-minute cap, seed base 1, sides alternating, no
# referee calls (--play-out), so a game is a game. The arena installs the freshly built library before it starts.
set -eu
arms="$1"; label="$2"; shift 2
root=$(cd "$(dirname "$0")/.." && pwd)
WITHIN_REASON_DISTILLED="$arms" exec "$root/target/release/arena" --our-ai WRDistilled:0.1 --matches 8 --parallel 8 --speed 50 --map "Comet Catcher Remake 1.8" --seed-base 1 --max-minutes 40 --play-out --label "$label" "$@"
