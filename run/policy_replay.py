#!/usr/bin/env python3
"""Opus writes the policy, offline (docs/design/2026-09-22-policy-replay.md): for each recorded player turn of a
match, Opus gets the report it saw in the game and a role text whose lever is a Lua policy (`run/policy/policy-player.md`
+ the brief), answers with a script, and the script is run in Lua 5.4 over every hands' call recorded between that
turn and the next (`jev-<ai>.jsonl`): its orders are scored against the menu, Jev's play and the packet's prohibitions.

usage: run/policy_replay.py <match dir> [--turns A-B] [--mode rewrite|amend] [--skip-opus] [--policy FILE] [--summarize] [--model ID] [--effort LEVEL]
  Scripts and results land in <match dir>/policy/ (turn-NN.lua, turn-NN.json, replay.jsonl), or policy-amend/ in
  amendment mode, where a turn answers with only the functions it changes and the harness appends them.
  --skip-opus replays the saved scripts; --policy FILE replays one script for every turn (a harness test).
  Opus runs on ~/.claude2 (weekly allotment only): a usage snapshot is taken before and reported after.
"""
import json
import os
import re
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_audit import Match, clock  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROLE = os.path.join(REPO, "run", "policy", "policy-player.md")
BRIEF = os.path.join(REPO, "docs", "briefs", "player.md")
LUA = "lua5.4"

PRELUDE = r'''
local function has(s, w) if s == nil then return false end if type(s) ~= "string" then s = tostring(s) end return string.find(s, w, 1, true) ~= nil end
local function starts(s, w) if s == nil then return false end if type(s) ~= "string" then s = tostring(s) end return string.sub(s, 1, #w) == w end
local env = { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math, tostring = tostring, tonumber = tonumber,
  type = type, next = next, select = select, has = has, starts = starts, print = function() end, error = error, pcall = pcall }
env._G = env
local chunk, err = load(POLICY, "policy", "t", env)
if not chunk then print("LOAD_ERROR\t" .. tostring(err)) os.exit(0) end
local ok, err2 = pcall(chunk)
if not ok then print("LOAD_ERROR\t" .. tostring(err2)) os.exit(0) end
local decide = env.decide
if type(decide) ~= "function" then print("LOAD_ERROR\tno decide function defined") os.exit(0) end
for _, S in ipairs(STATES) do
  local ok3, res = pcall(decide, S)
  if not ok3 then
    print(S.frame .. "\tERROR\t" .. tostring(res))
  else
    local parts = {}
    if type(res) == "table" then
      for actor, o in pairs(res) do
        if type(o) == "table" then
          local fields = {}
          for k, v in pairs(o) do fields[#fields + 1] = tostring(k) .. "=" .. tostring(v) end
          table.sort(fields)
          parts[#parts + 1] = tostring(actor) .. ":" .. table.concat(fields, ",")
        elseif type(o) == "string" then
          parts[#parts + 1] = tostring(actor) .. ":do=" .. o
        end
      end
    end
    table.sort(parts)
    print(S.frame .. "\tOK\t" .. table.concat(parts, ";"))
  end
end
'''


# ---------------------------------------------------------------- Lua literals

def lua_str(s):
    out = ['"']
    for ch in s:
        o = ord(ch)
        if ch == '"':
            out.append('\\"')
        elif ch == "\\":
            out.append("\\\\")
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\r":
            out.append("\\r")
        elif o < 32 or o == 127:
            out.append(f"\\{o:03d}")
        else:
            out.append(ch)
    out.append('"')
    return "".join(out)


def lua_literal(v):
    if v is None:
        return "nil"
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, (int, float)):
        return repr(v)
    if isinstance(v, str):
        return lua_str(v)
    if isinstance(v, (list, tuple)):
        return "{" + ",".join(lua_literal(x) for x in v) + "}"
    if isinstance(v, dict):
        items = [f"[{lua_str(str(k))}]={lua_literal(x)}" for k, x in v.items() if x is not None]
        return "{" + ",".join(items) + "}"
    return lua_str(str(v))


def kind_of(name):
    if name.startswith("group_"):
        return "group"
    if name.startswith(("lab_", "plant_")):
        return "lab"
    return "builder"


def state_of(call):
    actors = {}
    for name, entry in (call["state"].get("actors") or {}).items():
        kind = kind_of(name)
        qid = f"{name}.next" if kind == "lab" else f"{name}.do"
        q = (call.get("questions") or {}).get(qid) or {}
        a = dict(entry)
        a["kind"] = kind
        a["options"] = q.get("criteria") or {}
        a.setdefault("enemies_at_our_extractors", [])
        actors[name] = a
    groups = {}
    for g in call.get("groups") or []:
        groups[f"group_{g['name']}"] = {"x": g["at"][0], "z": g["at"][1], "members": len(g.get("members") or []), "task": (g.get("task") or {}).get("kind")}
    places = {}
    for p in call.get("places") or []:
        places[p["name"]] = {"x": p["x"], "z": p["z"], "spot": p.get("spot")}
    for name, words in (call["state"].get("places") or {}).items():
        places.setdefault(name, {}).update(words)
    return {
        "clock": clock(call["f"]),
        "frame": call["f"],
        "actors": actors,
        "groups": groups,
        "economy": call["state"].get("economy") or {},
        "enemy": call["state"].get("enemy") or {},
        "ours": call["state"].get("ours") or {},
        "places": places,
    }


# ---------------------------------------------------------------- Lua run

def run_policy(policy, calls):
    level = 1
    while f"]{'=' * level}]" in policy:
        level += 1
    eq = "=" * level
    states = "STATES = {" + ",".join(lua_literal(state_of(c)) for c in calls) + "}\n"
    program = f"POLICY = [{eq}[\n{policy}\n]{eq}]\n" + states + PRELUDE
    with tempfile.NamedTemporaryFile("w", suffix=".lua", delete=False) as f:
        f.write(program)
        path = f.name
    try:
        proc = subprocess.run([LUA, path], capture_output=True, text=True, timeout=120)
    finally:
        os.unlink(path)
    if proc.returncode != 0 and not proc.stdout:
        return {"load_error": (proc.stderr or "lua failed").strip()[:300]}, {}
    results = {}
    load_error = None
    for line in proc.stdout.splitlines():
        parts = line.split("\t")
        if parts[0] == "LOAD_ERROR":
            load_error = parts[1][:300] if len(parts) > 1 else "?"
            break
        frame = int(parts[0])
        if parts[1] == "ERROR":
            results[frame] = {"error": parts[2][:200] if len(parts) > 2 else "?"}
            continue
        orders = {}
        body = parts[2] if len(parts) > 2 else ""
        for item in body.split(";"):
            if not item:
                continue
            actor, _, fields = item.partition(":")
            o = {}
            for kv in fields.split(","):
                k, _, v = kv.partition("=")
                if k:
                    o[k] = v
            orders[actor] = o
        results[frame] = {"orders": orders}
    return ({"load_error": load_error} if load_error else {}), results


# ---------------------------------------------------------------- Opus

def ask_opus(system, user, model, effort, config_dir):
    env = {**os.environ, "CLAUDE_CONFIG_DIR": config_dir}
    cmd = ["claude", "-p", "--model", model, "--effort", effort, "--tools", "", "--strict-mcp-config", "--output-format", "json", "--system-prompt", system]
    started = time.time()
    proc = subprocess.run(cmd, input=user, capture_output=True, text=True, env=env, timeout=900)
    wall = time.time() - started
    if proc.returncode != 0:
        return {"error": proc.stderr.strip()[:500], "wall": wall, "text": proc.stdout[:2000]}
    try:
        out = json.loads(proc.stdout)
        text = out.get("result") or ""
        return {"text": text, "wall": wall, "cost_usd": out.get("total_cost_usd"), "usage": out.get("usage"), "duration_ms": out.get("duration_ms"), "is_error": out.get("is_error")}
    except ValueError:
        return {"text": proc.stdout, "wall": wall}


def extract_lua(text):
    m = re.search(r"```lua\s*\n(.*?)```", text, re.S)
    if m:
        return m.group(1)
    m = re.search(r"```\s*\n(.*?)```", text, re.S)
    return m.group(1) if m else None


NEVER_SPLIT = re.compile(r"never (splits|sends? (a )?detachment|sends? detachments)|forbid(s|ding)? detachments|no detachments|never split", re.I)
NEVER_SHELLING = re.compile(r"never (advances|walks|goes)[^.]*shelling|shelling[^.]*is (banned|forbidden)|never[^.]*called shelling", re.I)


def packets_by_turn(m):
    """The packet in force after each turn, from the recorded instruct calls."""
    packet = ""
    out = []
    for t in m.turns:
        for name, args in t["calls"]:
            if name == "orders":
                for entry in args.get("calls") or []:
                    if entry.get("tool") == "instruct":
                        packet = (entry.get("arguments") or {}).get("text") or packet
            elif name == "instruct":
                packet = args.get("text") or packet
        out.append(packet)
    return out


def score_turn(m, calls, results, packet):
    rows = []
    prev_do = {}
    prev_threat = {}
    reacted = episodes = 0
    for c in calls:
        r = results.get(c["f"]) or {}
        orders = r.get("orders") or {}
        for name, entry in (c["state"].get("actors") or {}).items():
            if kind_of(name) != "group":
                continue
            threat = bool(entry.get("enemies_at_our_extractors"))
            if threat and not prev_threat.get(name, False) and name in prev_do:
                episodes += 1
                if (orders.get(name) or {}).get("do", "continue") != prev_do.get(name, "continue"):
                    reacted += 1
            prev_threat[name] = threat
        for name in (c["state"].get("actors") or {}):
            prev_do[name] = (orders.get(name) or {}).get("do", "continue")
        for p in c["played"]:
            if m.scripted(c["f"], p["actor"]):
                continue
            actor = p["actor"]
            o = orders.get(actor) or {}
            do = o.get("do", "continue")
            options = set(p["options"]) | {"continue"}
            legal = do in options
            row = {"f": c["f"], "actor": actor, "kind": p["kind"], "jev": p["played"], "script": do, "legal": legal, "params": {k: v for k, v in o.items() if k != "do"}, "error": r.get("error")}
            row["agree"] = legal and do == p["played"]
            if packet and NEVER_SPLIT.search(packet) and do in ("send_against", "split"):
                row["violates"] = "never splits"
            if packet and NEVER_SHELLING.search(packet) and o.get("where") == "shelling":
                row["violates"] = "never shelling"
            if do in ("fight_to", "move_to", "split", "turret_at", "radar_at", "walk_to", "extractor") and o.get("where") and o.get("where") not in {pl["name"] for pl in c.get("places") or []} and o.get("where") not in (c["state"].get("places") or {}):
                row["bad_place"] = o.get("where")
            rows.append(row)
    return rows, reacted, episodes


def main():
    args = sys.argv[1:]

    def flag(name, default=None):
        if name not in args:
            return default
        i = args.index(name)
        value = args[i + 1]
        del args[i:i + 2]
        return value

    model = flag("--model", "claude-opus-5")
    effort = flag("--effort", "low")
    turns_arg = flag("--turns")
    policy_file = flag("--policy")
    mode = flag("--mode", "rewrite")
    config_dir = flag("--claude-config-dir", os.path.join(os.path.expanduser("~"), ".claude2"))
    skip_opus = "--skip-opus" in args or bool(policy_file)
    summarize = "--summarize" in args
    dirs = [a for a in args if not a.startswith("--")]
    if not dirs:
        sys.exit(__doc__)
    m = Match(dirs[0])
    out_dir = os.path.join(m.dir, "policy-amend" if mode == "amend" else "policy")
    os.makedirs(out_dir, exist_ok=True)
    if summarize:
        return print_summary(m, out_dir)
    turns = list(range(len(m.turns)))
    if turns_arg:
        a, _, b = turns_arg.partition("-")
        turns = list(range(int(a), int(b or a) + 1))
    system = open(ROLE).read() + open(BRIEF).read()
    packets = packets_by_turn(m)
    snapshot = None
    if not skip_opus:
        snapshot = os.path.join(REPO, "run", f"usage-before-policy-{int(time.time())}.json")
        subprocess.run([sys.executable, os.path.join(REPO, "run", "claude_usage.py"), "--snapshot", snapshot, config_dir], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    previous = None
    feedback = ""
    replay_path = os.path.join(out_dir, "replay.jsonl")
    with open(replay_path, "a") as replay:
        for i in turns:
            t = m.turns[i]
            f0 = t["frame"]
            f1 = m.turns[i + 1]["frame"] if i + 1 < len(m.turns) else 10 ** 9
            calls = [c for c in m.calls if f0 <= c["f"] < f1]
            lua_path = os.path.join(out_dir, f"turn-{i:02d}.lua")
            meta_path = os.path.join(out_dir, f"turn-{i:02d}.json")
            meta = {"turn": i, "frame": f0, "clock": clock(f0), "why": t["why"], "calls": len(calls)}
            if policy_file:
                policy = open(policy_file).read()
            elif skip_opus:
                if not os.path.exists(lua_path):
                    print(f"turn {i} {clock(f0)}: no saved script")
                    continue
                policy = open(lua_path).read()
                if os.path.exists(meta_path):
                    meta.update({k: v for k, v in json.load(open(meta_path)).items() if k in ("wall", "cost_usd", "usage", "closing")})
            else:
                if previous is None and i > 0 and os.path.exists(os.path.join(out_dir, f"turn-{i - 1:02d}.lua")):
                    previous = (m.turns[i - 1]["frame"], open(os.path.join(out_dir, f"turn-{i - 1:02d}.lua")).read())
                user = t["prompt"]
                if previous and feedback:
                    user += "\n\n" + feedback
                if previous and mode == "amend":
                    user += f"\n\nYour policy in force, written at {clock(previous[0])}:\n```lua\n{previous[1]}\n```\nAnswer with only the functions you change (complete top-level definitions), or the line `-- unchanged`."
                elif previous:
                    user += f"\n\nYour policy in force, written at {clock(previous[0])}:\n```lua\n{previous[1]}\n```\nKeep it, amend it or replace it; answer with the complete script."
                else:
                    user += "\n\nNo policy is in force yet: write the first one."
                r = ask_opus(system, user, model, effort, config_dir)
                meta.update({"wall": r.get("wall"), "cost_usd": r.get("cost_usd"), "usage": r.get("usage"), "opus_error": r.get("error")})
                text = r.get("text") or ""
                chunk = extract_lua(text)
                closing = re.sub(r"```.*?```", "", text, flags=re.S).strip()
                meta["closing"] = closing[:600]
                unchanged = chunk is None and "-- unchanged" in text or (chunk is not None and chunk.strip() in ("", "-- unchanged"))
                if previous and mode == "amend" and unchanged:
                    policy = previous[1]
                    meta["amend_lines"] = 0
                elif previous and mode == "amend" and chunk:
                    open(os.path.join(out_dir, f"turn-{i:02d}.amend.lua"), "w").write(chunk)
                    policy = previous[1] + f"\n\n-- amended at {clock(f0)}\n" + chunk
                    meta["amend_lines"] = chunk.count("\n") + 1
                elif chunk:
                    policy = chunk
                else:
                    meta["no_script"] = True
                    json.dump(meta, open(meta_path, "w"))
                    print(f"turn {i} {clock(f0)}: NO SCRIPT in the answer ({r.get('wall', 0):.0f} s): {closing[:120]}")
                    continue
                open(lua_path, "w").write(policy)
                previous = (f0, policy)
            load, results = run_policy(policy, calls)
            rows, reacted, episodes = score_turn(m, calls, results, packets[i])
            errors = sum(1 for r in results.values() if "error" in r)
            meta.update({"load_error": load.get("load_error"), "runtime_errors": errors, "decisions": len(rows), "legal": sum(1 for r in rows if r["legal"]),
                         "agree": sum(1 for r in rows if r["agree"]), "ordered": sum(1 for r in rows if r["script"] != "continue"),
                         "holds": sum(1 for r in rows if r["script"] == "hold"), "violations": sum(1 for r in rows if r.get("violates")),
                         "bad_places": sum(1 for r in rows if r.get("bad_place")), "reacted": reacted, "episodes": episodes, "lines": policy.count("\n") + 1})
            json.dump(meta, open(meta_path, "w"))
            given = {}
            for row in rows:
                given[row["script"]] = given.get(row["script"], 0) + 1
            first_error = next((r["error"] for r in results.values() if "error" in r), None)
            feedback = (f"Your policy since your last turn ran {len(calls)} times; {errors} of those runs raised an error"
                        + (f" (the first: {first_error})" if first_error else "") + "; the orders it gave, counted: "
                        + (", ".join(f"{k} {v}" for k, v in sorted(given.items(), key=lambda kv: -kv[1])) or "none") + ".")
            if load.get("load_error"):
                feedback = f"Your policy since your last turn did not load: {load['load_error']}. Every actor kept its course."
            for row in rows:
                row["turn"] = i
                replay.write(json.dumps(row) + "\n")
            replay.flush()
            sent = f", {meta['amend_lines']} sent" if "amend_lines" in meta else ""
            print(f"turn {i:2d} {clock(f0)} ({t['why'][:40]}): {meta['lines']} lines{sent} in {meta.get('wall') or 0:.0f} s; "
                  + (f"LOAD ERROR {meta['load_error']}" if meta["load_error"] else
                     f"{errors} runtime errors over {len(calls)} calls; {meta['ordered']}/{len(rows)} ordered, {meta['legal']} legal, {meta['agree']} agree with Jev, "
                     f"{meta['violations']} violate the packet, reacted {reacted}/{episodes}"))
    if snapshot:
        usage = subprocess.run([sys.executable, os.path.join(REPO, "run", "claude_usage.py"), "--since", snapshot, config_dir], capture_output=True, text=True)
        since = next((l.strip() for l in usage.stdout.splitlines() if "since snapshot" in l), usage.stdout.strip()[-200:])
        print("usage:", since)
        if usage.returncode == 3:
            print("STOP: the usage check reports a limit or overage", file=sys.stderr)


def print_summary(m, out_dir):
    metas = []
    for name in sorted(os.listdir(out_dir)):
        if name.startswith("turn-") and name.endswith(".json"):
            metas.append(json.load(open(os.path.join(out_dir, name))))
    rows = [json.loads(l) for l in open(os.path.join(out_dir, "replay.jsonl"))] if os.path.exists(os.path.join(out_dir, "replay.jsonl")) else []
    n = len(metas)
    walls = sorted(x.get("wall") or 0 for x in metas)
    print(f"{m.label}: {n} turns; scripts {sum(1 for x in metas if not x.get('no_script'))}, load errors {sum(1 for x in metas if x.get('load_error'))}, "
          f"turns with runtime errors {sum(1 for x in metas if x.get('runtime_errors'))}; wall median {walls[n // 2] if n else 0:.0f} s, max {walls[-1] if n else 0:.0f}, over 45 s: {sum(1 for w in walls if w > 45)}; "
          f"lines median {sorted(x.get('lines', 0) for x in metas)[n // 2] if n else 0}; cost ${sum(x.get('cost_usd') or 0 for x in metas):.2f}")
    sent = [x["amend_lines"] for x in metas if "amend_lines" in x]
    if sent:
        outs = sorted(((x.get("usage") or {}).get("output_tokens") or 0) for x in metas)
        print(f"  amendments: {len(sent)} turns, lines sent median {sorted(sent)[len(sent) // 2]}, unchanged {sum(1 for v in sent if v == 0)}; output tokens median {outs[len(outs) // 2]}")
    if rows:
        dec = len(rows)
        print(f"  decisions {dec}: ordered {sum(1 for r in rows if r['script'] != 'continue')}, legal {sum(1 for r in rows if r['legal'])}, illegal {sum(1 for r in rows if not r['legal'])}, "
              f"agree with Jev {sum(1 for r in rows if r['agree'])}, hold {sum(1 for r in rows if r['script'] == 'hold')}, violate the packet {sum(1 for r in rows if r.get('violates'))}, bad places {sum(1 for r in rows if r.get('bad_place'))}")
        by_kind = {}
        for r in rows:
            k = by_kind.setdefault(r["kind"], {"n": 0, "ordered": 0, "agree": 0, "illegal": 0})
            k["n"] += 1
            k["ordered"] += r["script"] != "continue"
            k["agree"] += r["agree"]
            k["illegal"] += not r["legal"]
        for k, v in by_kind.items():
            print(f"    {k}: {v}")
        print(f"  reacted {sum(x.get('reacted', 0) for x in metas)} of {sum(x.get('episodes', 0) for x in metas)} raider episodes")
    for x in metas:
        if x.get("load_error") or x.get("no_script"):
            print(f"  turn {x['turn']} {x['clock']}: {'no script' if x.get('no_script') else x['load_error']}")


if __name__ == "__main__":
    main()
