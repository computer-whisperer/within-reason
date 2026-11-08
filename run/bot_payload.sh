#!/bin/sh
# Lay out a bot of bots/ as the engine finds it: <out dir>/{AIInfo.lua, library}, the version and the url filled into
# AIInfo.lua. The crate's version in bots/<bot>/Cargo.toml is the only place a bot's version is written
# (docs/design/2026-11-07-bot-releases.md §3.1); install_ai.sh and the release packaging both go through here, so
# the arena's data dir and a release hold the same directory. Prints the version.
# usage: bot_payload.sh <bot> <library> <out dir>
#   bot:      a directory of bots/ (distilled, territory, template)
#   library:  the built library; by its extension the copy is named libSkirmishAI.so or SkirmishAI.dll
#   WITHIN_REASON_BOT_URL: the url written into AIInfo.lua (default: the repository)
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
[ $# -eq 3 ] || { echo "usage: bot_payload.sh <bot> <library> <out dir>" >&2; exit 2; }
bot=$1 lib=$2 out=$3
toml="$root/bots/$bot/Cargo.toml"
version=$(sed -n 's/^version *= *"\([^"]*\)".*/\1/p' "$toml" | head -n 1)
[ -n "$version" ] || { echo "no version in $toml" >&2; exit 2; }
url=${WITHIN_REASON_BOT_URL:-https://github.com/computer-whisperer/within-reason}
case $lib in
    *.so) name=libSkirmishAI.so ;;
    *.dll) name=SkirmishAI.dll ;;
    *) echo "the library must end in .so or .dll: $lib" >&2; exit 2 ;;
esac
[ -f "$lib" ] || { echo "no library at $lib" >&2; exit 2; }
mkdir -p "$out"
sed -e "s|@VERSION@|$version|" -e "s|@URL@|$url|" "$root/bots/$bot/data/AIInfo.lua" > "$out/AIInfo.lua"
if grep -q '@[A-Z]*@' "$out/AIInfo.lua"; then echo "unfilled placeholder in $out/AIInfo.lua" >&2; exit 2; fi
cp "$lib" "$out/$name"
echo "$version"
