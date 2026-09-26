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
    if model.startswith(("fw:", "api:")):
        return "api"
    return "codex" if model.startswith(("gpt", "o1", "o3", "o4", "codex")) else "claude"


def api_config(model):
    """The endpoint, key and model id for `fw:<model>` (Fireworks, `~/.config/within-reason/fireworks.env`) or
    `api:<model>` (`api.env`): the environment variable of each name overrides the file. Never printed."""
    prefix, name = model.split(":", 1)
    var = "FIREWORKS" if prefix == "fw" else "API"
    path = os.path.join(os.path.expanduser("~"), ".config", "within-reason", "fireworks.env" if prefix == "fw" else "api.env")
    values = {}
    if os.path.exists(path):
        for line in open(path):
            line = line.strip()
            if "=" in line and not line.startswith("#"):
                k, v = line.split("=", 1)
                values[k.strip()] = v.strip().strip('"').strip("'")
    get = lambda k, default=None: os.environ.get(f"{var}_{k}") or values.get(f"{var}_{k}") or default
    base = get("BASE_URL", "https://api.fireworks.ai/inference/v1" if prefix == "fw" else None)
    key = get("API_KEY") if prefix == "fw" else get("KEY")
    if not key or not base:
        raise RuntimeError(f"no key or base url for {prefix}: put them in {path}")
    price = lambda k: float(get(k)) if get(k) is not None else None
    prices = {"input": price("PRICE_INPUT"), "output": price("PRICE_OUTPUT"), "cached": price("PRICE_CACHED")}
    if prices["cached"] is None:
        prices["cached"] = prices["input"]
    return base.rstrip("/"), key, name, int(get("MAX_TOKENS", 8192)), prices


def api_cost(prices, usage):
    """USD for one call from its usage, at the config's prices (USD per million tokens); None without prices."""
    if prices["input"] is None or prices["output"] is None:
        return None
    prompt, out = usage.get("prompt_tokens") or 0, usage.get("completion_tokens") or 0
    cached = min((usage.get("prompt_tokens_details") or {}).get("cached_tokens") or 0, prompt)
    return ((prompt - cached) * prices["input"] + cached * prices["cached"] + out * prices["output"]) / 1e6


def ask_api(system, user, model, effort, timeout):
    import urllib.request
    base, key, name, max_tokens, prices = api_config(model)
    body = {"model": name, "messages": [{"role": "system", "content": system}, {"role": "user", "content": user}], "max_tokens": max_tokens}
    if effort and effort != "medium":
        body["reasoning_effort"] = effort
    req = urllib.request.Request(base + "/chat/completions", data=json.dumps(body).encode(), headers={"Authorization": "Bearer " + key, "Content-Type": "application/json"})
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            data = json.loads(r.read())
    except urllib.error.HTTPError as e:
        return {"text": "", "wall": time.time() - t0, "api_s": None, "usage": {}, "cost_usd": None, "error": f"HTTP {e.code}: {e.read()[:300].decode(errors='replace')}"}
    except Exception as e:  # noqa: BLE001
        return {"text": "", "wall": time.time() - t0, "api_s": None, "usage": {}, "cost_usd": None, "error": str(e)[:300]}
    wall = time.time() - t0
    u = data.get("usage") or {}
    text = ((data.get("choices") or [{}])[0].get("message") or {}).get("content") or ""
    return {"text": text, "wall": wall, "api_s": wall, "usage": {"input_tokens": u.get("prompt_tokens"), "output_tokens": u.get("completion_tokens")}, "cost_usd": api_cost(prices, u), "error": ""}


def ask(system, user, model, effort="low", config_dir=CLAUDE_CONFIG_DIR, timeout=900, thinking=None):
    """`effort` is the CLI's reasoning setting (claude --effort low..max; codex model_reasoning_effort none/low/...,
    what each model accepts differs). `thinking` (claude only) caps the thinking tokens; 0 turns thinking off, which
    `--effort low` does not do for haiku."""
    if backend_of(model) == "api":
        return ask_api(system, user, model, effort, timeout)
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
