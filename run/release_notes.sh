#!/bin/sh
# The release notes of a bot's version: the top section of bots/<bot>/CHANGELOG.md, which must be that version's,
# then the fixed download-and-install text. The release workflow feeds this to `gh release create`; run it by hand
# to see the notes before tagging. usage: release_notes.sh <bot> <version>
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
bot=${1:?usage: release_notes.sh <bot> <version>}
version=${2:?usage: release_notes.sh <bot> <version>}
changelog="$root/bots/$bot/CHANGELOG.md"
[ -f "$changelog" ] || { echo "no $changelog: a release needs the player-facing changelog" >&2; exit 1; }
heading=$(grep -m 1 '^## ' "$changelog" || true)
case "$heading" in
    "## $version"|"## $version "*) ;;
    *) echo "the top section of $changelog is '$heading', not '## $version': write this version's section first" >&2; exit 1 ;;
esac
short=$(python3 -c "import re,sys; print(re.search(r\"key\s*=\s*'shortName'\s*,\s*value\s*=\s*'([^']*)'\", open(sys.argv[1]).read(), re.S).group(1))" "$root/bots/$bot/data/AIInfo.lua")
# The section's body: after the first '## ' line, up to the next one.
awk 'f && /^## /{exit} f{print} /^## /{f=1}' "$changelog" | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}'
cat <<NOTES

## Download and install

- **Windows:** \`$short-$version-INSTALL.zip\`. Close BAR, right-click the zip, **Extract All**, and double-click \`Install.bat\` in the extracted folder.
- **Linux:** \`$short-$version-LINUX.tar.gz\`, then \`tar xzf $short-$version-LINUX.tar.gz && sh install.sh\`.

Not the "Source code" archives GitHub adds below: those are the whole repository and have no installer.

Only the player hosting the game needs it. The installer puts the AI into BAR's data folder, where a BAR engine update keeps it, and switches the lobby to its full AI list (the simple list hides every custom AI; BAR must be closed for that step). Then, in a skirmish or a lobby you host, **Add AI** and pick **$short $version**. \`INSTALL.txt\` inside has the steps by hand.

Tested on BAR engine 2026.07.04 on Linux. The Windows build has not been played yet: please say in an issue whether it loads for you.
NOTES
