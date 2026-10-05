#!/bin/sh
# Build the AI libraries and install them where an engine discovers skirmish AIs:
# <data dir>/AI/Skirmish/<shortName>/<version>/{AIInfo.lua,libSkirmishAI.so}
# usage: install_ai.sh [DATA_DIR]   (default: the arena's run/data)
# The flagship (WReason, the shim of the bot process) goes to any data dir; the bots of bots/ only to the arena's,
# so a BAR install's lobby lists nothing that is not meant for people yet.
# Files are replaced by rename, never rewritten in place: a running game keeps the library it
# loaded. Overwriting a mapped .so crashes the process that has it loaded.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
data="${1:-$root/run/data}"
cargo build --release --manifest-path "$root/Cargo.toml"
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
install WReason/0.1 crates/ai-shim/data/AIInfo.lua libai_shim.so
if [ $# -eq 0 ]; then
    install WRTemplate/0.1 bots/template/data/AIInfo.lua libbot_template.so
    install WRDistilled/0.1 bots/distilled/data/AIInfo.lua libbot_distilled.so
fi
