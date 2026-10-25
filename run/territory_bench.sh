#!/bin/bash
# The territory bot's bench (docs/design/2026-10-25-territory-regions.md, "The bench"): six starts, four seeds each,
# against NullAI to 20 minutes, one arena batch a start labelled terrN-TAG-START, N counting up from FIRST.
#
#   run/territory_bench.sh FIRST TAG [arena args for every batch...]
#
# The bot's arms come from WITHIN_REASON_TERRITORY, by default the arms terr102-132 ran with. Read the result with
# run/territory_bench.py TAG [TAG...]; each batch still takes its own row in docs/experiments.md.
set -u
cd "$(dirname "$0")/.." || exit 1
if [ $# -lt 2 ]; then
  sed -n '5p' "$0" | sed 's/^# *//'
  exit 2
fi
first=$1
tag=$2
shift 2
export WITHIN_REASON_TERRITORY=${WITHIN_REASON_TERRITORY:-spots=all,factories=4,constructors=40}
# Build and install once, then keep what is installed for all six batches (the arena's install step otherwise builds
# the checkout at each batch's start, and an edit made during the bench reached its later batches: terr150,
# terr166-168).
run/install_ai.sh > /dev/null || exit 1
export WITHIN_REASON_KEEP_INSTALLED=1
n=$first
run() {
  start=$1
  map=$2
  shift 2
  label="terr$n-$tag-$start"
  n=$((n + 1))
  echo "=== $label on $map $(date +%T)"
  target/release/arena --our-ai WRTerritory:0.1 --opponent NullAI:0.1 --matches 4 --parallel 4 --speed 50 \
    --map "$map" --max-minutes 20 --play-out --seed-base 1 --base-port 9400 --label "$label" "$@" 2>&1 | grep -E "^match|^==|->"
}
# The starts, as run/territory_bench.py names them. Comet from thebluegecko's start fixes both starts in the script;
# the others take the lobby's boxes, the corner and the faction alternating by seed.
run comet-his-start "Comet Catcher Remake 1.8" --starts 941,2534:6899,684 "$@"
run comet-boxes "Comet Catcher Remake 1.8" "$@"
run great-divide "Great Divide V1" "$@"
run glitters "All That Glitters v2.2.3" "$@"
run mithril "Mithril Mountain v2.0.1" "$@"
run quicksilver "Quicksilver Remake 1.24" "$@"
echo "=== done $(date +%T)"
