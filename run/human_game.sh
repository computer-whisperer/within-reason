#!/bin/sh
# Start the bot for a game with people in the user's own BAR client: the pianist with the Opus player over it, in
# realtime (nothing pauses; docs/design/2026-09-21-pianist.md "Realtime"). The engine's shim finds the bot on
# $XDG_RUNTIME_DIR/within-reason.sock and retries about once a second, so the bot may be started before or after the
# game. The match's record, transcripts and Jev log land in run/matches/<unix time>-<label>/00, which
# run/view_match.py and run/debrief.py read like an arena match. The sessions run on ~/.claude2 (weekly allotment
# only) unless WITHIN_REASON_CLAUDE_CONFIG_DIR says otherwise; a usage snapshot is taken first for `--since`.
# usage: run/human_game.sh [label] [effort] [hands-effort] [model]     (defaults: human, medium, normal, claude-opus-5-5; hands-effort is the Jev
# token diet, lean | normal | full: a match worth every token takes full). The hands' layers are the full set
# (WITHIN_REASON_HANDS_LAYERS, set beforehand to choose otherwise; human-10 ran `news` alone for $6.91 over three seats).
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
label=${1:-human}
effort=${2:-medium}
hands=${3:-normal}
model=${4:-claude-opus-5-5}
dir="$root/run/matches/$(date +%s)-$label/00"
mkdir -p "$dir"
python3 "$root/run/claude_usage.py" --snapshot "$root/run/usage-before-$label.json" >/dev/null 2>&1 || true
echo "match dir $dir; usage snapshot run/usage-before-$label.json; socket ${XDG_RUNTIME_DIR:-/tmp}/within-reason.sock"
cd "$dir"
exec env WITHIN_REASON_REALTIME=1 WITHIN_REASON_RECORD=1 WITHIN_REASON_JEV_LOG=1 WITHIN_REASON_LOG_DIR="$dir" WITHIN_REASON_EFFORT="$effort" WITHIN_REASON_HANDS_EFFORT="$hands" WITHIN_REASON_MODEL="$model" \
    WITHIN_REASON_HANDS_LAYERS="${WITHIN_REASON_HANDS_LAYERS:-news,same,fuse,openers,split,tick,places}" \
    "$root/target/release/bot" --pianist --player 2>&1 | tee "$dir/bot.log"
