"""One call to a chat model through the CLI that carries its subscription: Claude models through `claude -p` on the
claude2 account, OpenAI models through `codex exec`. Both start a process per call; `ask` reports the process wall
time and, separately, the model's own time as the CLI reports it (`duration_api_ms` from claude; for codex the span
from its `turn.started` event to the final `agent_message`, timed here as the events stream out).

    ask(system, user, model) -> {"text", "wall", "api_s", "usage": {"input_tokens", "output_tokens", ...}, "cost_usd", "error"}

No auth is read or printed here: each CLI carries its own. Model output is data; nothing in it is executed.
"""
import json
import os
import subprocess
import tempfile
import time

CLAUDE_CONFIG_DIR = os.path.join(os.path.expanduser("~"), ".claude2")
CODEX_PREAMBLE = ("Answer directly in your final message. Do not run commands, read or write files, or use any tool: "
                  "everything you need is in this message.\n\n")


def backend_of(model):
    return "codex" if model.startswith(("gpt", "o1", "o3", "o4", "codex")) else "claude"


def ask(system, user, model, effort="low", config_dir=CLAUDE_CONFIG_DIR, timeout=900, thinking=None):
    """`effort` is the CLI's reasoning setting (claude --effort low..max; codex model_reasoning_effort none/low/...,
    what each model accepts differs). `thinking` (claude only) caps the thinking tokens; 0 turns thinking off, which
    `--effort low` does not do for haiku."""
    if backend_of(model) == "codex":
        return ask_codex(system, user, model, effort, timeout)
    return ask_claude(system, user, model, effort, config_dir, timeout, thinking)


def ask_claude(system, user, model, effort, config_dir, timeout, thinking=None):
    env = {**os.environ, "CLAUDE_CONFIG_DIR": config_dir}
    if thinking is not None:
        env["MAX_THINKING_TOKENS"] = str(thinking)
    cmd = ["claude", "-p", "--model", model, "--effort", effort, "--tools", "", "--strict-mcp-config", "--output-format", "json", "--system-prompt", system]
    started = time.time()
    try:
        proc = subprocess.run(cmd, input=user, capture_output=True, text=True, env=env, timeout=timeout)
    except subprocess.TimeoutExpired:
        return {"error": f"timeout after {timeout} s", "wall": time.time() - started, "text": ""}
    wall = time.time() - started
    if proc.returncode != 0:
        return {"error": proc.stderr.strip()[:500], "wall": wall, "text": proc.stdout[:2000]}
    try:
        out = json.loads(proc.stdout)
    except ValueError:
        return {"text": proc.stdout, "wall": wall, "error": "claude printed no JSON"}
    usage = out.get("usage") or {}
    return {"text": out.get("result") or "", "wall": wall, "api_s": (out.get("duration_api_ms") or 0) / 1000.0,
            "cost_usd": out.get("total_cost_usd"), "usage": usage, "is_error": out.get("is_error"),
            "overage": '"isUsingOverage":true' in proc.stdout.replace(" ", ""), "error": (out.get("result") or "")[:300] if out.get("is_error") else None}


def ask_codex(system, user, model, effort, timeout):
    """`codex exec` with a read-only sandbox, no session file, an empty working directory (so no repo AGENTS.md is
    read), the prompt on stdin and the events on stdout as JSONL. The model's own time is the span from the
    `turn.started` event to the last `agent_message`."""
    prompt = CODEX_PREAMBLE + system + "\n\n" + user
    with tempfile.TemporaryDirectory(prefix="codex-") as scratch:
        last = os.path.join(scratch, "last.txt")
        cmd = ["codex", "exec", "-m", model, "-c", f"model_reasoning_effort={effort}", "-s", "read-only", "--skip-git-repo-check",
               "--ephemeral", "--json", "-C", scratch, "-o", last, "-"]
        started = time.time()
        proc = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            proc.stdin.write(prompt)
            proc.stdin.close()
        except BrokenPipeError:
            pass
        turn_started = None
        message_at = None
        text = ""
        usage = {}
        events = 0
        try:
            for line in proc.stdout:
                now = time.time()
                if now - started > timeout:
                    proc.kill()
                    break
                try:
                    ev = json.loads(line)
                except ValueError:
                    continue
                events += 1
                kind = ev.get("type")
                if kind == "turn.started":
                    turn_started = now
                elif kind == "item.completed" and (ev.get("item") or {}).get("type") == "agent_message":
                    text = ev["item"].get("text") or ""
                    message_at = now
                elif kind == "turn.completed":
                    usage = ev.get("usage") or {}
                elif kind == "error":
                    usage.setdefault("error", str(ev.get("message"))[:300])
            proc.wait(timeout=30)
        except Exception as e:  # noqa: BLE001
            proc.kill()
            return {"error": f"{type(e).__name__}: {e}"[:300], "wall": time.time() - started, "text": text}
        wall = time.time() - started
        stderr = proc.stderr.read()
        if os.path.exists(last) and not text:
            text = open(last).read()
    error = usage.pop("error", None)
    if proc.returncode != 0 and not text:
        return {"error": (error or stderr.strip() or f"codex exited {proc.returncode}")[-500:], "wall": wall, "text": ""}
    api_s = (message_at - turn_started) if (turn_started and message_at) else None
    return {"text": text, "wall": wall, "api_s": api_s, "usage": usage, "cost_usd": None, "error": error,
            "startup_s": (turn_started - started) if turn_started else None, "events": events}
