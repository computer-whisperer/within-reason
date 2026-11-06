#!/bin/sh
# The distilled bot's map bench (bots/distilled/README.md, "The maps"): eight games of WRDistilled on each of twelve
# 1v1 maps of different kinds against one opponent, two maps at a time, on the lobby's start boxes.
# usage: run/distilled_maps.sh <apex|barb> <first number> [tag]
#   e.g. run/distilled_maps.sh apex 148          rows dist148-onyx-apex .. dist159-titanduel-apex
#        run/distilled_maps.sh barb 160 reach    rows dist160-reach-onyx-barb ..
# apex is Apex v0.5.04 standard from a cold state (its learned state deleted once, before the first row); barb is
# BARb hard_aggressive. Seeds 1-8, sides and boxes alternating, a 30-minute cap, no referee calls. The library is
# built and installed once, before the first row (WITHIN_REASON_KEEP_INSTALLED for the rows: an edit during the
# bench reaches none of them). Read with `run/bot_maps.py <label> ...`.
# The maps (fetch with pr-downloader --download-map, docs/harness/engine.md): the lobby's 1v1 maps of two players
# (Onyx Cauldron, Quicksilver, Ravaged, Avalanche, Zed, Hooked, Flooded Valley, Argent Strata) and four small ones
# of other kinds (Red Comet flat, The Halite Maze a maze with little metal, Sertaleina wind, TitanDuel flat).
set -eu
who="$1"; n="$2"; tag="${3:+$3-}"
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
run/install_ai.sh
export WITHIN_REASON_KEEP_INSTALLED=1
if [ "$who" = apex ]; then
    rm -rf run/data/opponent-state/Apex-v0.5.04
    set -- --opponent Apex:v0.5.04 --profile standard
else
    set -- --profile hard_aggressive
fi
row() { # <map> <name> <port> <the opponent's flags...>
    map="$1"; name="$2"; port="$3"; shift 3
    target/release/arena --our-ai WRDistilled:0.1 --matches 8 --parallel 8 --speed 50 --map "$map" --seed-base 1 --max-minutes 30 --play-out \
        --base-port "$port" --label "dist$n-$tag$name-$who" "$@" 2>&1 | grep -E "^==|->|rror|usage"
}
pair() { # <map> <name> <map> <name>
    m1="$1"; l1="$2"; m2="$3"; l2="$4"; shift 4
    row "$m1" "$l1" 9700 "$@" & n=$((n + 1)); sleep 20
    row "$m2" "$l2" 9800 "$@" & n=$((n + 1)); wait
}
pair "Onyx Cauldron 2.2.2" onyx "Quicksilver Remake 1.24" quicksilver "$@"
pair "Ravaged Remake v1.2" ravaged "Avalanche 3.4" avalanche "$@"
pair "Zed Remake 3.3.1" zed "Red Comet Remake 1.8" redcomet "$@"
pair "The Halite Maze v1.4" halite "Hooked 1.1.1" hooked "$@"
pair "Flooded_Valley_V5" flooded "Argent Strata 1.1" argent "$@"
pair "Sertaleina_v9.2" sertaleina "TitanDuel_V3" titanduel "$@"
