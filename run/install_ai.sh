#!/bin/sh
# Build the AI libraries and install them where an engine discovers skirmish AIs:
# <data dir>/AI/Skirmish/<shortName>/<version>/{AIInfo.lua,libSkirmishAI.so}
# A bot of bots/ is installed at the version of its Cargo.toml (run/bot_payload.sh), the flagship's shim at 0.1.
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
# install <shortName>/<version> <AIInfo.lua> <library>: the flagship's shim, at its fixed version.
install() {
    dest="$data/AI/Skirmish/$1"
    mkdir -p "$dest"
    for pair in "$2:AIInfo.lua" "target/release/$3:libSkirmishAI.so"; do
        cp "$root/${pair%%:*}" "$dest/.${pair##*:}.new"
        mv -f "$dest/.${pair##*:}.new" "$dest/${pair##*:}"
    done
    echo "installed to $dest"
}
# install_bot <bot dir> <shortName> <library>: a bot of bots/, laid out by run/bot_payload.sh at the version of its
# Cargo.toml (docs/design/2026-11-07-bot-releases.md §3.1). The other versions of the bot are removed: a lobby lists
# each, a hosted game loads the highest, and the arena resolves a bare name to the one installed, so a version left
# behind is a trap.
install_bot() {
    mkdir -p "$data/AI/Skirmish/$2"
    stage=$(mktemp -d "$data/AI/Skirmish/$2/.new.XXXXXX")
    version=$("$root/run/bot_payload.sh" "$1" "$root/target/release/$3" "$stage")
    dest="$data/AI/Skirmish/$2/$version"
    mkdir -p "$dest"
    for file in AIInfo.lua libSkirmishAI.so; do
        mv -f "$stage/$file" "$dest/$file"
    done
    rmdir "$stage"
    for other in "$data/AI/Skirmish/$2"/*/; do
        [ "${other%/}" = "$dest" ] || rm -rf "$other"
    done
    echo "installed to $dest"
}
# The chat relay widget (docs/design/2026-11-04-chat-permissions.md) goes beside the AI in a BAR install, where the
# client hosting the AI loads it as a user widget; the arena's matches take it from run/match-template instead, and
# a copy in the arena's data dir would load twice.
if [ "$data" != "$root/run/data" ]; then
    mkdir -p "$data/LuaUI/Widgets"
    cp "$root/run/match-template/LuaUI/Widgets/wr_chat_relay.lua" "$data/LuaUI/Widgets/.wr_chat_relay.lua.new"
    mv -f "$data/LuaUI/Widgets/.wr_chat_relay.lua.new" "$data/LuaUI/Widgets/wr_chat_relay.lua"
    echo "installed the chat relay widget to $data/LuaUI/Widgets"
fi
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
        WRTemplate) install_bot template WRTemplate libbot_template.so ;;
        WRDistilled) install_bot distilled WRDistilled libbot_distilled.so ;;
        WRTerritory) install_bot territory WRTerritory libbot_territory.so ;;
        Apex:*) install_apex "${name#Apex:}" ;;
        *) echo "unknown AI: $name" >&2; exit 2 ;;
    esac
done
