#!/bin/sh
# The page's copy of the tape's reader: crates/tape-wasm built for the browser and put where viewer/scene/tape.js loads
# it (viewer/scene/tape.wasm, git-ignored: it is a build product). run/view_match.py runs this with --if-stale at its
# start, which builds only when a source file of crates/tape or crates/tape-wasm is newer than the copy.
#
#     run/build_viewer.sh [--if-stale]
#
# Needs the wasm target once: rustup target add wasm32-unknown-unknown
set -e
repo=$(cd "$(dirname "$0")/.." && pwd)
out="$repo/viewer/scene/tape.wasm"
if [ "$1" = "--if-stale" ] && [ -f "$out" ]; then
    newer=$(find "$repo/crates/tape/src" "$repo/crates/tape-wasm/src" "$repo/crates/tape/Cargo.toml" "$repo/crates/tape-wasm/Cargo.toml" -newer "$out" -print -quit)
    [ -z "$newer" ] && exit 0
fi
cd "$repo"
cargo build --release --target wasm32-unknown-unknown -p tape-wasm >&2
mkdir -p "$repo/viewer/scene"
cp "$repo/target/wasm32-unknown-unknown/release/tape_wasm.wasm" "$out.new"
mv "$out.new" "$out"
