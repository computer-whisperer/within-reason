#!/bin/bash
# The distilled bot's bench (docs/design/2026-10-25-distilled-air.md, "The bench"): three maps, eight games each, one
# arena batch a map labelled distN-TAG-MAP, N counting up from FIRST.
#
#   run/distilled_bench.sh FIRST TAG [arena args for every batch...]
#
# The opponent and its options come in the arena args (e.g. --opponent Apex:v0.3.3 --profile standard
# --opponent-option apex_plan_force=5). The bot's arms come from WITHIN_REASON_DISTILLED. Before every batch Apex
# v0.3.3's exchange record is put back to the saved copy RECORD (by default the 165 lines it held on 2026-10-25), so
# that no batch faces an opponent that learned from the one before (terr318-331 met 72 cells at the first batch and
# 158 at the last). Each batch takes its own row in docs/experiments.md.
set -u
cd "$(dirname "$0")/.." || exit 1
if [ $# -lt 2 ]; then
  sed -n '5p' "$0" | sed 's/^# *//'
  exit 2
fi
first=$1
tag=$2
shift 2
record=${RECORD:-run/data/opponent-state/apex-v0.3.3-records/warm-165-lines-2026-10-25.txt}
live=run/data/opponent-state/Apex-v0.3.3/skirmish/apex-record.txt
[ -f "$record" ] || { echo "no saved record at $record" >&2; exit 1; }
# Build and install once, then keep what is installed for every batch (an edit made during a bench otherwise reaches
# its later batches: terr150, terr166-168).
run/install_ai.sh > /dev/null || exit 1
export WITHIN_REASON_KEEP_INSTALLED=1
n=$first
run() {
  name=$1
  map=$2
  shift 2
  label="dist$n-$tag-$name"
  n=$((n + 1))
  cp "$record" "$live"
  echo "=== $label on $map $(date +%T)"
  target/release/arena --our-ai WRDistilled:0.1 --matches 8 --parallel 8 --speed 50 --map "$map" --max-minutes 30 \
    --play-out --seed-base 1 --base-port 9500 --label "$label" "$@" 2>&1 | grep -E "^match|^==|->"
}
run comet "Comet Catcher Remake 1.8" "$@"
run glitters "All That Glitters v2.2.3" "$@"
run quicksilver "Quicksilver Remake 1.24" "$@"
cp "$record" "$live"
echo "=== done $(date +%T)"
