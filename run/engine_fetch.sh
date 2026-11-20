#!/bin/sh
# Fetch an engine release into run/engines/<release> and give it the opponents the checkout's engine carries.
# usage: engine_fetch.sh RELEASE [--use]      (e.g. 2026.09.03; --use also points run/engine at it)
#
# The release is the Linux archive of github.com/beyond-all-reason/RecoilEngine, the same file the BAR launcher
# unpacks (its `engine/recoil_<release>`; https://launcher-config.beyondallreason.dev/config.json names the release
# each of the launcher's channels runs). The engine's own AI/Skirmish holds BARb, CircuitAI and NullAI; the arena
# reads its opponents there too (`--opponent NAME:VERSION`), so what we installed beside them in the engine in use is
# copied across: every Apex build as it is, and Barb3's game side with the new release's BARb library, which is what
# Barb3's library is (its BUILD-NOTE.txt).
#
# A batch tries a release without moving the link: WITHIN_REASON_ENGINE=<release> before `arena` or `duel`
# (docs/harness/engine.md, "Moving to a new engine release"). A demo replays only on the engine that recorded it, and
# run/replay_match.py picks that engine from run/engines by the demo's header, so the older releases stay.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
release=${1:?usage: engine_fetch.sh RELEASE [--use]}
dest="$root/run/engines/$release"
from="$root/run/engine"
if [ ! -x "$dest/spring-headless" ]; then
    [ ! -e "$dest" ] || { echo "$dest exists without an engine in it; look at it first" >&2; exit 2; }
    mkdir -p "$root/run/engines"
    tmp=$(mktemp -d "$root/run/engines/.fetch.XXXXXX")
    trap 'rm -rf "$tmp"' EXIT
    archive="recoil_${release}_amd64-linux.7z"
    curl -fsSL -o "$tmp/$archive" "https://github.com/beyond-all-reason/RecoilEngine/releases/download/$release/$archive"
    mkdir "$tmp/engine"
    7z x -bso0 -bsp0 "-o$tmp/engine" "$tmp/$archive"
    [ -x "$tmp/engine/spring-headless" ] || { echo "no spring-headless in $archive" >&2; exit 2; }
    mv "$tmp/engine" "$dest"
    echo "unpacked $archive to $dest"
fi
# A first checkout has no engine yet, so nothing to carry: the globs then name nothing that exists.
for ai in "$from"/AI/Skirmish/*/; do
    name=$(basename "$ai")
    case $name in
        BARb|CircuitAI|NullAI|CampaignAI) continue ;;
    esac
    for version in "$ai"*/; do
        to="$dest/AI/Skirmish/$name/$(basename "$version")"
        [ -d "$version" ] && [ ! -e "$to" ] || continue
        mkdir -p "$dest/AI/Skirmish/$name"
        cp -a --reflink=auto "${version%/}" "$to"
        if [ "$name" = Barb3 ]; then
            cp "$dest/AI/Skirmish/BARb/stable/libSkirmishAI.so" "$to/.libSkirmishAI.so.new"
            mv -f "$to/.libSkirmishAI.so.new" "$to/libSkirmishAI.so"
            printf '\n%s, run/engine_fetch.sh: copied from the %s engine, the library replaced by the stock BARb stable library of engine %s.\n' \
                "$(date +%F)" "$(basename "$(readlink -f "$from")")" "$release" >> "$to/BUILD-NOTE.txt"
        fi
        echo "copied $name/$(basename "$version")"
    done
done
if [ "${2:-}" = --use ]; then
    ln -sfn "engines/$release" "$root/run/engine"
    echo "run/engine -> engines/$release"
else
    echo "try it: WITHIN_REASON_ENGINE=$release target/release/arena ...; adopt it: $0 $release --use"
fi
