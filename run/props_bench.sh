#!/bin/sh
# The property drills (docs/design/2026-11-15-army-control-loop.md, section 9): one batch a row, every label
# prefixed with $1 (the engine under test: `base` for the engine before the rebuild). Read with run/drill_props.py.
# usage: run/props_bench.sh PREFIX [DUEL_BINARY] [ONLY]   (ONLY: a pattern of row names to run, default all)
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
prefix="$1"
duel="${2:-$root/target/release/duel}"
only="${3:-}"
flat="Full Metal Plate 1.7"
glit="All That Glitters v2.2.3"
c5="armflash*26+armbull*7+armjanus*5+armmanni*7+armstump*29+armsam*18"
cd "$root"
row() {
    name="$1"
    shift
    case "$name" in *"$only"*) ;; *) return 0 ;; esac
    echo "== $prefix-$name"
    "$duel" --drill walk --lane on --cap 24 --reps 1 --label "$prefix-$name" "$@" 2>&1 | grep -E "^\[1/1\]|not run|error" || true
}
w48="--ours armpw --count 48 --distance 2000 --order move"
# P1 stand: to one point, then what the army does there.
row stand24 --map "$flat" --ours armpw --count 24 --distance 1500 --order move --run 80 --time-limit 120
row stand48 --map "$flat" $w48 --run 90 --time-limit 130
row stand96 --map "$flat" --ours armpw --count 96 --distance 2000 --order move --run 120 --time-limit 160
row stand-c5 --map "$glit" --ours "$c5" --from 3260,3665 --to 2600,4568 --order fight --run 150 --time-limit 190
row stand-door --map "$flat" --ours armpw --count 96 --distance 2000 --order move --wall armdrag@900,160,1400 --run 150 --time-limit 190
# P2 disturb: on the march (10 s) and at rest (60 s).
row join-march --map "$flat" $w48 --behind "armpw*6@400" --late 10 --run 90 --time-limit 130
row join-rest --map "$flat" $w48 --behind "armpw*6@400" --late 60 --run 100 --time-limit 140
row kill-march --map "$flat" $w48 --kill 2@10 --run 90 --time-limit 130
row kill-rest --map "$flat" $w48 --kill 2@60 --run 100 --time-limit 140
row build-off --map "$flat" $w48 --build armsolar@1000,900,10 --run 90 --time-limit 130
row build-way --map "$flat" $w48 --build armsolar@1300,0,10 --run 90 --time-limit 130
row build-place --map "$flat" $w48 --build armsolar@2000,0,60 --run 100 --time-limit 140
# P3 thrash: the place moved once, and alternated every second.
row shift49 --map "$flat" $w48 --shift 0,49@10 --run 90 --time-limit 130
row shift100 --map "$flat" $w48 --shift 0,100@10 --run 90 --time-limit 130
row shift300 --map "$flat" $w48 --shift 0,300@10 --run 90 --time-limit 130
row shift100-rest --map "$flat" $w48 --shift 0,100@60 --run 100 --time-limit 140
row alternate --map "$flat" $w48 --nudge 1 --nudge-by 300 --run 60 --time-limit 100
# P5 restart.
row restart-march --map "$flat" $w48 --restart 10 --run 90 --time-limit 130
row restart-rest --map "$flat" $w48 --restart 60 --run 100 --time-limit 140
# T11 straggler: pace-type members set behind on a long walk.
row straggler1 --map "$flat" --ours armpw --count 24 --behind "armpw*1@300" --distance 2600 --order move --run 80 --time-limit 120
row straggler3 --map "$flat" --ours armpw --count 24 --behind "armpw*3@300" --distance 2600 --order move --run 80 --time-limit 120
