#!/bin/bash
# The distilled bot's team bench (the user, 2026-10-25: "2v2 against barb is a good option"): two seats of ours against
# two of BARb hard_aggressive on All That Glitters v2.2.3, sixteen games to 40:00 (MINUTES), played out; one arena batch an arm,
# labelled distN-TAG-ARM with the arm's '=' and ',' as '-', N counting up from FIRST. Every arm plays the same seeds.
#
#   run/distilled_team_bench.sh FIRST TAG ARMS [ARMS...] [-- arena args for every batch...]
#
# ARMS is a WITHIN_REASON_DISTILLED value ("gate=field"). MINUTES in the environment sets the cap (40 by default; 55 for
# the silos, which come from about 30:00). Read the result with run/distilled_t2.py (run/distilled_nuke.py for the silos).
set -u
cd "$(dirname "$0")/.." || exit 1
if [ $# -lt 3 ]; then
  sed -n '6p' "$0" | sed 's/^# *//'
  exit 2
fi
first=$1
tag=$2
shift 2
arms=()
while [ $# -gt 0 ] && [ "$1" != "--" ]; do
  arms+=("$1")
  shift
done
[ "${1:-}" = "--" ] && shift
# Build and install once, then keep what is installed for every batch (terr150, terr166-168).
run/install_ai.sh > /dev/null || exit 1
export WITHIN_REASON_KEEP_INSTALLED=1
n=$first
for arm in "${arms[@]}"; do
  label="dist$n-$tag-$(echo "$arm" | tr '=,' '--')"
  n=$((n + 1))
  echo "=== $label $(date +%T)"
  WITHIN_REASON_DISTILLED="$arm" target/release/arena --our-ai WRDistilled:0.1 --ours 2 --enemies 2 --profile hard_aggressive \
    --matches 16 --parallel 8 --speed 50 --map "All That Glitters v2.2.3" --max-minutes "${MINUTES:-40}" --play-out --seed-base 1 \
    --base-port 9500 --label "$label" "$@" 2>&1 | grep -E "^==|->"
done
echo "=== done $(date +%T)"
