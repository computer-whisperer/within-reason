#!/bin/sh
# Installs @NAME@ @VERSION@ into a Beyond All Reason install on Linux.
#   ./install.sh                  find BAR, install
#   ./install.sh /path/to/data    BAR's data folder (the one with engine/, games/ and maps/ inside)
#   ./install.sh -y ...           answer yes to every question
# The AI goes to <data>/AI/Skirmish/@SHORT@/@VERSION@/, the data folder and not the engine folder: the engine's AI
# interface is stable across engine versions, so a BAR engine update keeps it (docs/design/2026-11-07-bot-releases.md).
set -eu

SHORT="@SHORT@"
VERSION="@VERSION@"
NAME="@NAME@"
HERE=$(cd "$(dirname "$0")" && pwd)
PAYLOAD="$HERE/payload/$SHORT/$VERSION"
YES=0
ARG=""
for a in "$@"; do
    case "$a" in
        -y|--yes) YES=1 ;;
        *) ARG="$a" ;;
    esac
done

say() { echo "  $*"; }
fail() { echo; echo "  INSTALL FAILED: $*" >&2; exit 1; }
ask() {
    [ "$YES" = 1 ] && return 0
    printf '  %s [y/N] ' "$1"
    read -r r
    case "$r" in y|Y|yes|YES|Yes) return 0 ;; *) return 1 ;; esac
}

# The data folder of a BAR install: the folder holding engine/ (the launcher's data folder on Linux, <install>/data
# of a Windows-style layout). A folder given on the command line is taken as the data folder when neither it nor
# its data/ holds engine/ (a data folder whose engine lives elsewhere, as a headless setup's does).
data_dir_of() {
    for d in "$1/data" "$1"; do
        [ -d "$d/engine" ] && { (cd "$d" && pwd); return 0; }
    done
    [ "${2:-}" = given ] && [ -d "$1" ] && { (cd "$1" && pwd); return 0; }
    return 1
}

find_bar() {
    for r in \
        "${BAR_DATA:-}" \
        "${XDG_DATA_HOME:-$HOME/.local/share}/BeyondAllReason" \
        "${XDG_STATE_HOME:-$HOME/.local/state}/Beyond All Reason" \
        "${XDG_DATA_HOME:-$HOME/.local/share}/Beyond All Reason" \
        "$HOME/.var/app/info.beyondallreason.bar/data/BeyondAllReason" \
        "$HOME/.var/app/info.beyondallreason.bar/data/Beyond All Reason" \
        "$HOME/.var/app/info.beyondallreason.bar/state/Beyond All Reason" \
        "$HOME/Documents/Beyond All Reason" \
        "$HOME/.steam/steam/steamapps/common/Beyond All Reason" \
        "$HOME/.local/share/Steam/steamapps/common/Beyond All Reason"; do
        [ -n "$r" ] && data_dir_of "$r" && return 0
    done
    return 1
}

bar_running() {
    pgrep -f -i 'spring|Beyond-All-Reason|beyond-all-reason' >/dev/null 2>&1
}

# The lobby hides every AI it has no simple name for while simpleAiList is on (its default), and the checkbox sits
# on the Developer tab, which only devmode.txt shows. The lobby rewrites IGL_data.lua when it closes, so this runs
# with BAR closed.
show_full_ai_list() {
    data="$1"; igl="$1/LuaMenu/Config/IGL_data.lua"
    touch "$data/devmode.txt"
    if [ -f "$igl" ]; then
        grep -Eq 'simpleAiList[[:space:]]*=[[:space:]]*false' "$igl" && return 0
        cp -f "$igl" "$igl.before-$SHORT"
        if grep -Eq 'simpleAiList[[:space:]]*=[[:space:]]*true' "$igl"; then
            sed -i -E 's/simpleAiList[[:space:]]*=[[:space:]]*true/simpleAiList = false/' "$igl"
        elif grep -Eq '\["Chili lobby"\][[:space:]]*=[[:space:]]*\{' "$igl"; then
            sed -i -E '0,/\["Chili lobby"\][[:space:]]*=[[:space:]]*\{/s//&\n\t\tsimpleAiList = false,/' "$igl"
        elif grep -Eq 'return[[:space:]]*\{' "$igl"; then
            sed -i -E '0,/return[[:space:]]*\{/s//&\n\t["Chili lobby"] = {\n\t\tsimpleAiList = false,\n\t},/' "$igl"
        else
            return 1
        fi
    else
        mkdir -p "$(dirname "$igl")"
        printf 'return {\n\t["Chili lobby"] = {\n\t\tsimpleAiList = false,\n\t},\n}\n' > "$igl"
    fi
    grep -Eq 'simpleAiList[[:space:]]*=[[:space:]]*false' "$igl" \
        && ! grep -Eq 'simpleAiList[[:space:]]*=[[:space:]]*true' "$igl"
}

echo
echo "  Installing $NAME $VERSION for Beyond All Reason (Linux)"
echo

[ -f "$PAYLOAD/AIInfo.lua" ] || fail "the AI files are missing from $PAYLOAD. Extract the whole archive and run install.sh again."
[ -f "$PAYLOAD/libSkirmishAI.so" ] || fail "this package has no Linux library (libSkirmishAI.so)."

if [ -n "$ARG" ]; then
    DATA=$(data_dir_of "$ARG" given) || fail "no folder at $ARG (expected BAR's data folder, the one with engine/ inside)."
else
    DATA=$(find_bar) || fail "could not find Beyond All Reason. Run again with its data folder: ./install.sh \"/path/to/BeyondAllReason\""
fi
say "BAR found:  $DATA"

TARGET="$DATA/AI/Skirmish/$SHORT/$VERSION"
if bar_running && [ -e "$TARGET/libSkirmishAI.so" ]; then
    say "BAR is open. A running game may have this AI loaded, and a loaded library must not be replaced."
    ask "Install anyway?" || fail "cancelled. Close BAR and run install.sh again."
fi
# One version installed: a hosted game loads the highest version folder by name, and "0.10.0" sorts below "0.9.0".
for other in "$DATA/AI/Skirmish/$SHORT"/*/; do
    [ -d "$other" ] || continue
    [ "${other%/}" = "$TARGET" ] && continue
    say "Removing the older $(basename "$other")."
    rm -rf "$other" || fail "could not remove $other"
done
if [ -e "$TARGET" ]; then
    say "Replacing the copy already installed."
    rm -rf "$TARGET" || fail "could not remove the old copy at $TARGET"
fi
mkdir -p "$(dirname "$TARGET")"
cp -R "$PAYLOAD" "$TARGET" || fail "copying to $TARGET failed."
chmod 755 "$TARGET/libSkirmishAI.so"
[ -f "$TARGET/libSkirmishAI.so" ] || fail "the copy is incomplete at $TARGET."
say "AI files:   $TARGET"

LISTED=0
while true; do
    if ! bar_running; then
        if show_full_ai_list "$DATA"; then LISTED=1; else say "Could not change the lobby setting."; fi
        break
    fi
    echo
    say "BAR is open. Its lobby must be closed to switch on the full AI list"
    say "(it saves its settings when it closes and would undo the change)."
    [ "$YES" = 1 ] && break
    printf '  Close BAR completely, then press Enter (or type S to skip) '
    read -r r
    case "$r" in s|S|skip) break ;; esac
done

echo
echo "  Done. $NAME $VERSION is installed."
if [ "$LISTED" = 1 ]; then
    say "The lobby will show the full AI list (devmode.txt made, 'Simple AI list' off)."
else
    say "The lobby setting was NOT changed, so $NAME will be hidden in the AI list. By hand:"
    say "  1. make an empty file named devmode.txt in $DATA"
    say "  2. start BAR, open Settings > Developer and untick 'Simple AI list'"
fi
echo
say "Start BAR. In a skirmish or a lobby you host, Add AI and pick $SHORT $VERSION."
say "Only the player hosting the game needs this. The AI stays installed across BAR engine updates."
