#!/bin/sh
# Build the AI libraries and install them where an engine discovers skirmish AIs:
# <data dir>/AI/Skirmish/<shortName>/<version>/{AIInfo.lua,libSkirmishAI.so}
# usage: install_ai.sh [DATA_DIR [AI...]]   (default: the arena's run/data)
# The arena's data dir takes every AI. Another data dir (a BAR install) takes the flagship (WReason, the shim of the
# bot process) alone unless AIs are named: WReason, WRTemplate, WRDistilled, WRTerritory, or Apex:<version> (an
# installed build of the opponent, for the user's lobby: the user, 2026-10-26). A bot of bots/ goes to a BAR
# install only by name, so its lobby lists nothing that is not meant for people yet (the user, 2026-10-25, named the
# distilled and territory bots).
# Files are replaced by rename, never rewritten in place: a running game keeps the library it
# loaded. Overwriting a mapped .so crashes the process that has it loaded.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
data="${1:-$root/run/data}"
# WITHIN_REASON_KEEP_INSTALLED=1: neither build nor install; the libraries installed stay. Every arena and duel start
# runs this script, so without it a source edit during a batch is compiled into its later arms. run/territory_bench.sh
# builds and installs once and sets it for its batches, so an edit or a build in the checkout during the bench's six
# batches reaches none of them (terr150 and terr166-168 ran code edited after their bench began).
if [ -n "${WITHIN_REASON_KEEP_INSTALLED:-}" ]; then
    exit 0
fi
cargo build --release --manifest-path "$root/Cargo.toml"
# install_apex <version>: the Apex opponent as built into the arena's engine (its game-side and nets included, the
# recipe in run/engines/2026.07.04/AI/Skirmish/Apex/<version>/BUILD-NOTE.txt), copied whole and renamed in.
install_apex() {
    src="$root/run/engines/2026.07.04/AI/Skirmish/Apex/$1"
    dest="$data/AI/Skirmish/Apex/$1"
    [ -f "$src/libSkirmishAI.so" ] || { echo "no Apex build at $src" >&2; exit 2; }
    mkdir -p "$data/AI/Skirmish/Apex"
    rm -rf "$dest.new" "$dest.old"
    cp -r "$src" "$dest.new"
    [ -e "$dest" ] && mv "$dest" "$dest.old"
    mv "$dest.new" "$dest"
    rm -rf "$dest.old"
    echo "installed to $dest"
}
# install <shortName>/<version> <AIInfo.lua> <library>
install() {
    dest="$data/AI/Skirmish/$1"
    mkdir -p "$dest"
    for pair in "$2:AIInfo.lua" "target/release/$3:libSkirmishAI.so"; do
        cp "$root/${pair%%:*}" "$dest/.${pair##*:}.new"
        mv -f "$dest/.${pair##*:}.new" "$dest/${pair##*:}"
    done
    echo "installed to $dest"
}
if [ $# -eq 0 ]; then
    names="WReason WRTemplate WRDistilled WRTerritory"
elif [ $# -eq 1 ]; then
    names="WReason"
else
    shift
    names="$*"
fi
for name in $names; do
    case $name in
        WReason) install WReason/0.1 crates/ai-shim/data/AIInfo.lua libai_shim.so ;;
        WRTemplate) install WRTemplate/0.1 bots/template/data/AIInfo.lua libbot_template.so ;;
        WRDistilled) install WRDistilled/0.1 bots/distilled/data/AIInfo.lua libbot_distilled.so ;;
        WRTerritory) install WRTerritory/0.1 bots/territory/data/AIInfo.lua libbot_territory.so ;;
        Apex:*) install_apex "${name#Apex:}" ;;
        *) echo "unknown AI: $name" >&2; exit 2 ;;
    esac
done
