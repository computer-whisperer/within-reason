#!/bin/sh
# The smoke before a tag (docs/design/2026-11-07-bot-releases.md §3.4): build, package the Linux tarball exactly as
# the release workflow does, install it with its own install.sh into the arena's data dir (the bot's directory
# removed first, so only the package can put it back), and play one short game against BARb easy with the installed
# files kept. The pass is the engine log naming the bot at the package's version.
# usage: release_smoke.sh <bot>        (distilled, territory, template)
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
bot=${1:?usage: release_smoke.sh <bot>}
short=$(python3 -c "import re,sys; print(re.search(r\"key\s*=\s*'shortName'\s*,\s*value\s*=\s*'([^']*)'\", open(sys.argv[1]).read(), re.S).group(1))" "$root/bots/$bot/data/AIInfo.lua")
data="$root/run/data"
work=$(mktemp -d "${TMPDIR:-/tmp}/release-smoke-$bot.XXXXXX")
trap 'rm -rf "$work"' EXIT

cd "$root"
cargo build --release --manifest-path "$root/Cargo.toml"
"$root/run/release_package.py" "$bot" --so "$root/target/release/libbot_$bot.so" --out "$work/dist"
tgz=$(ls "$work"/dist/*-LINUX.tar.gz)
version=$(basename "$tgz" | sed -E "s/^$short-(.*)-LINUX\.tar\.gz$/\1/")
mkdir -p "$work/unpacked"
tar xzf "$tgz" -C "$work/unpacked"

# Only the package may put the bot back; the rest of the data dir (the other AIs, the game, the maps) stays.
rm -rf "$data/AI/Skirmish/$short"
sh "$work/unpacked/install.sh" -y "$data"
[ -f "$data/AI/Skirmish/$short/$version/libSkirmishAI.so" ] || { echo "SMOKE FAILED: install.sh did not install $short/$version" >&2; exit 1; }

label="release-smoke-$bot"
WITHIN_REASON_KEEP_INSTALLED=1 "$root/target/release/arena" --our-ai "$short" --matches 1 --speed 50 --profile easy \
    --map "Comet Catcher Remake 1.8" --max-minutes 4 --label "$label"
match=$(ls -1dt "$root"/run/matches/*-"$label" | head -n 1)
log="$match/00/infolog.txt"
if grep -q "Short-Name: \"$short\", Version: \"$version\"" "$log" && grep -q "is $short $version (" "$log"; then
    echo "SMOKE PASSED: $short $version played from the package ($match)"
else
    echo "SMOKE FAILED: the engine log does not name $short $version ($log)" >&2
    exit 1
fi
