#!/usr/bin/env python3
"""The player's own debrief of a match, for the next player (docs/design/2026-09-21-pianist.md, "The player's
learnings"; the live edge, docs/design/2026-10-21-brief-rework.md §11): one `claude -p` over the match's notes, closing
sentences, last packet and result, no game held, asked for what decided the game, at most five lines for the next
player on this map and opponent, and what it wished it could order. The lines land in docs/briefs/inbox/<batch>.md
(match 00; a later match of a batch in <batch>__<NN>.md, so a series keeps every game's) for the review pass that
integrates them into the brief's layers, and run/live_edge.py puts them in the match's live-edge entry.

The call is the player's, not Claude Code's: an empty working directory, no settings sources (neither the checkout's
nor the account's CLAUDE.md is read), no tools, and a system prompt of its own, as the bot launches the player's own
sessions (crates/bot/src/strategist/mod.rs). Run from the checkout under the default system prompt (to 2026-10-23) the
model took itself for a coding agent: five of sixteen debriefs (rework-ladder-10-1/02, -35-1/00, -35-1/02,
1792807358-human, 1792810743-human) were a tool call written as text ("I'll look at how previous debriefs in this repo
are written ... <invoke name="Bash">"), accepted as non-empty output and shown to the next player in the live edge, and
the other eleven were long documents in a shape of their own, not the one asked for. The answer is checked before it
is written: tool-call markup, a code fence, a shell line or an "I'll look" preamble reject it; one retry with the
reason, then the rejected text goes to <match>/debrief-rejected.txt, nothing to the inbox, and the exit code is 2
(run/live_edge.py then writes "No debrief: ..." in the entry). The DECIDED / NEXT PLAYER / WISHED shape is asked for
and reported in the header line, not required: under the player's prompt ten of ten answers (the five repaired
2026-10-24, two each) were debriefs and none was in the shape, a retry asking for it included, so requiring it would
throw away the debrief for its headings.

usage: run/debrief.py <match dir> [--model ID] [--effort LEVEL] [--claude-config-dir DIR] [--dry-run]
  <match dir> is a match (run/matches/<batch>/00) or a batch (its 00 is taken).
  --dry-run prints the system prompt and the prompt and writes nothing.
The session runs on ~/.claude2 (weekly allotment only; a usage snapshot is taken before and reported after).
"""
import json, os, re, subprocess, sys, time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "run"))
import live_edge  # noqa: E402  (opponent_key, end_row, handicaps: the same reading of the record the entry uses)

INBOX = os.path.join(REPO, "docs", "briefs", "inbox")
MAX_NOTES = 40
MAX_CLOSING = 30
MAX_CHARS = 14000
CALL_TIMEOUT = 240  # seconds for one `claude -p`; two calls at most

SYSTEM_PROMPT = (
    "You are the player of a game of Beyond All Reason that has just ended: you directed a side through a pair of fast "
    "hands (Jev) by writing standing instructions, and you are now debriefing the game for the next player. You have no "
    "tools, no files and nothing to look up: everything you know of the game is in the message, and a debrief that says "
    "less is better than one that guesses. Answer in plain text in the shape the message asks for, and nothing else: no "
    "preamble, no headings, no markup, no code."
)

MARKUP = re.compile(r"<\s*/?\s*(invoke|function_calls|parameter|tool_use|tool_call|antml)", re.I)
SHELL = re.compile(r"^\s*(\$ |cd /|cd \.|ls |cat |grep |python3? )", re.M)
PREAMBLE = re.compile(r"^\s*(I'll|I will|Let me|I need to|I should)\s+(look|check|read|see|examine|inspect|find|open)\b", re.I)


def clock(frame):
    seconds = frame // 30
    return f"{seconds // 60}:{seconds % 60:02d}"


def match_directory(path):
    path = os.path.abspath(path)
    if not any(f.startswith("record-") for f in os.listdir(path)) and os.path.isdir(os.path.join(path, "00")):
        path = os.path.join(path, "00")
    return path


def inbox_path(match_dir):
    """docs/briefs/inbox/<batch>.md for a batch's match 00 (every debrief before 2026-10-24 is named so), <batch>__<NN>.md
    for its other matches, which before then overwrote the one file."""
    match_dir = match_directory(match_dir)
    batch = os.path.basename(os.path.dirname(match_dir))
    index = os.path.basename(match_dir)
    return os.path.join(INBOX, f"{batch}.md" if index == "00" else f"{batch}__{index}.md")


def load_match(match_dir):
    match_dir = match_directory(match_dir)
    batch = {}
    batch_path = os.path.join(os.path.dirname(match_dir), "batch.json")
    if os.path.isfile(batch_path):
        batch = json.load(open(batch_path))
    records = sorted(f for f in os.listdir(match_dir) if f.startswith("record-") and f.endswith(".jsonl"))
    if not records:
        sys.exit(f"no record-*.jsonl in {match_dir}")
    result = header = None
    for line in open(os.path.join(match_dir, records[0]), errors="replace"):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        if r.get("t") == "header" and header is None:
            header = r
        if "result" in r and isinstance(r["result"], dict) and "outcome" in r["result"]:
            result = r
    transcripts = sorted(f for f in os.listdir(match_dir) if f.startswith("strategist-") and f.endswith(".jsonl"))
    if not transcripts:
        sys.exit(f"no strategist-*.jsonl in {match_dir}: not a player match")
    turns = []
    turn = None
    for line in open(os.path.join(match_dir, transcripts[0]), errors="replace"):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        kind = r.get("kind")
        if kind == "turn":
            turn = {"frame": int(r.get("frame", 0)), "notes": [], "closing": "", "instruct": None}
            turns.append(turn)
        elif kind == "tool_call" and turn is not None:
            calls = r.get("arguments", {}).get("calls") or [{"tool": r.get("tool"), "arguments": r.get("arguments", {})}]
            for c in calls:
                if c.get("tool") == "note":
                    turn["notes"].append(c.get("arguments", {}).get("text", ""))
                elif c.get("tool") == "instruct":
                    turn["instruct"] = c.get("arguments", {}).get("text", "")
        elif kind == "result" and turn is not None:
            text = r.get("message", {}).get("result")
            if isinstance(text, str) and text.strip():
                turn["closing"] = text.strip()
    try:
        build = open(os.path.join(os.path.dirname(match_dir), "snapshot", "git-head.txt")).read().strip()[:12]
    except OSError:
        build = batch.get("commit", "") or "?"
    header = header or {}
    outcome = result["result"] if result else {}
    if outcome:
        minutes = float(outcome.get("game_minutes") or 0.0)
        result_text = f"{outcome.get('outcome', '?')} after {minutes:.1f} min" + (" (called by the referee)" if outcome.get("called") else "")
    else:
        end = live_edge.end_row(match_dir, records[0])
        minutes = (end.get("f", 0) / 1800.0) if end else 0.0
        result_text = "not recorded (a game with people: no referee)"
        if end:
            result_text += f", the record ends at {clock(end.get('f', 0))} with {end.get('own_units', '?')} of our units standing"
    about = {
        "label": batch.get("label") or os.path.basename(os.path.dirname(match_dir)),
        "build": build,
        "map": (header.get("map") or {}).get("name") or batch.get("map") or "?",
        "opponent": live_edge.opponent_key(header, match_dir) or batch.get("opponent") or "?",
        "side": header.get("side", "?"),
        "corner": outcome.get("our_corner"),
        "result_text": result_text,
        "minutes": minutes,
        "seats": len(transcripts),
        "transcript": transcripts[0],
    }
    return match_dir, about, turns


def build_prompt(about, turns):
    seat = "" if about["seats"] == 1 else f" You were the player of the first of our {about['seats']} seats ({about['transcript']}); the others had players of their own."
    lines = [
        "You are debriefing a game of Beyond All Reason that you played as the player over a pair of fast hands (Jev), "
        "a game now over. Below are your notes and closing sentences from each turn, the last packet of standing "
        "instructions you wrote, and the result. Write for the next player who will sit where you sat, on this map "
        "against this opponent, and who has never seen this game.",
        "",
        f"Game: {about['label']}, map {about['map']}, against {about['opponent']}, our side {about['side']}"
        + (f", corner {about['corner']}" if about["corner"] else "") + "." + seat,
        f"Result: {about['result_text']}; {len(turns)} turns."
        + ("" if about["minutes"] and "not recorded" not in about["result_text"] else " Say in DECIDED how the game ended as far as your notes know it."),
        "",
        "Your notes, by game time:",
    ]
    notes = [(t["frame"], n) for t in turns for n in t["notes"]]
    if len(notes) > MAX_NOTES:
        notes = notes[: MAX_NOTES // 2] + notes[-MAX_NOTES // 2 :]
    lines += [f"  {clock(f)} {n}" for f, n in notes]
    closings = [(t["frame"], t["closing"]) for t in turns if t["closing"]]
    if len(closings) > MAX_CLOSING:
        closings = closings[: MAX_CLOSING // 2] + closings[-MAX_CLOSING // 2 :]
    lines += ["", "Your closing sentences, by game time:"] + [f"  {clock(f)} {c[:300]}" for f, c in closings]
    last_packet = next((t["instruct"] for t in reversed(turns) if t["instruct"]), None)
    if last_packet:
        lines += ["", "The last packet of standing instructions you wrote:", last_packet[:3000]]
    lines += [
        "",
        "Answer in exactly this shape, plain text, nothing else:",
        "DECIDED: one sentence on what decided the game.",
        "NEXT PLAYER: up to five lines, one lesson each, concrete and in the words of the packet where possible "
        "(places, groups, units, timings); only lessons this game supports.",
        "WISHED: up to three lines, each one thing you wished you could order or see and could not; 'none' if none.",
    ]
    text = "\n".join(lines)
    if len(text) > MAX_CHARS:
        text = text[:MAX_CHARS] + "\n[cut]"
    return text


def garbage(answer):
    """Why the answer is not a debrief at all (a coding agent's tool call written as text), or None."""
    if MARKUP.search(answer):
        return "tool-call markup in the text"
    if "```" in answer:
        return "a code fence in the text"
    if SHELL.search(answer):
        return "a shell command in the text"
    if PREAMBLE.match(answer):
        return "an 'I'll look' preamble instead of a debrief"
    return None


def in_shape(answer):
    """Whether the answer has the DECIDED and NEXT PLAYER lines asked for (reported, not required)."""
    return bool(re.search(r"^\s*DECIDED:", answer, re.M) and re.search(r"^\s*NEXT PLAYER:", answer, re.M))


def ask(prompt, model, effort, config_dir, cwd):
    """One `claude -p` as the player: the system prompt above, no tools, no settings sources, an empty working directory."""
    env = {**os.environ, "CLAUDE_CONFIG_DIR": config_dir}
    proc = subprocess.run(
        ["claude", "-p", "--model", model, "--effort", effort, "--tools", "", "--strict-mcp-config", "--setting-sources", "",
         "--system-prompt", SYSTEM_PROMPT, "--output-format", "text"],
        input=prompt, capture_output=True, text=True, env=env, cwd=cwd, timeout=CALL_TIMEOUT,
    )
    answer = proc.stdout.strip()
    if proc.returncode != 0 or not answer:
        sys.exit(f"claude -p failed ({proc.returncode}): {proc.stderr.strip()[:500]}")
    return answer


def main():
    args = sys.argv[1:]
    if not args or args[0].startswith("--"):
        sys.exit(__doc__)

    def flag(name, default):
        return args[args.index(name) + 1] if name in args else default

    model = flag("--model", "claude-opus-5")
    effort = flag("--effort", "low")
    config_dir = flag("--claude-config-dir", os.path.join(os.path.expanduser("~"), ".claude2"))
    match_dir, about, turns = load_match(os.path.abspath(args[0]))
    prompt = build_prompt(about, turns)
    if "--dry-run" in args:
        print("SYSTEM PROMPT:", SYSTEM_PROMPT, "", prompt, sep="\n")
        return
    os.makedirs(INBOX, exist_ok=True)
    out_path = inbox_path(match_dir)
    cwd = os.path.join(match_dir, "debrief-cwd")
    os.makedirs(cwd, exist_ok=True)
    snapshot = os.path.join(REPO, "run", f"usage-before-debrief-{int(time.time())}.json")
    usage_script = os.path.join(REPO, "run", "claude_usage.py")
    subprocess.run([sys.executable, usage_script, "--snapshot", snapshot, config_dir], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    started = time.time()
    rejected = os.path.join(match_dir, "debrief-rejected.txt")
    answer = ask(prompt, model, effort, config_dir, cwd)
    reason = garbage(answer)
    tries = 1
    if reason:
        tries = 2
        retry = (prompt + f"\n\nYour previous answer was rejected: {reason}. You have no tools and nothing to look up; everything "
                 "you know of the game is above. Answer in the shape asked, plain text, and nothing else.")
        first = answer
        answer = ask(retry, model, effort, config_dir, cwd)
        reason = garbage(answer)
        if reason:
            with open(rejected, "w") as f:
                f.write(f"first answer:\n{first}\n\nsecond answer ({reason}):\n{answer}\n")
            print(f"the debrief was rejected twice ({reason}); the answers are in {os.path.relpath(rejected, REPO)}", file=sys.stderr)
            sys.exit(2)
    if os.path.exists(rejected):
        os.remove(rejected)  # an earlier run's; this one answered
    shape = "in the asked shape" if in_shape(answer) else "in a shape of its own (DECIDED / NEXT PLAYER / WISHED was asked)"
    usage = subprocess.run([sys.executable, usage_script, "--since", snapshot, config_dir], capture_output=True, text=True)
    since = next((l.strip() for l in usage.stdout.splitlines() if "since snapshot" in l), usage.stdout.strip()[-200:])
    seat = "" if about["seats"] == 1 else f", the player of the first of our {about['seats']} seats"
    with open(out_path, "w") as f:
        f.write(f"# {about['label']}: the player's debrief\n\n")
        f.write(f"Match `{os.path.relpath(match_dir, REPO)}`, build {about['build']}, against {about['opponent']} on {about['map']}: "
                f"{about['result_text']}; {len(turns)} turns{seat}. "
                f"Debriefed by {model} (effort {effort}) in {time.time() - started:.0f} s"
                + (" (one answer rejected, the second kept)" if tries == 2 else "") + f", {shape}; {since}.\n\n")
        f.write(answer.rstrip() + "\n")
    print(f"wrote {os.path.relpath(out_path, REPO)}\n{since}")
    if usage.returncode == 3:
        print("STOP: the usage check reports a limit or overage", file=sys.stderr)
        sys.exit(3)
    print(answer)


if __name__ == "__main__":
    main()
