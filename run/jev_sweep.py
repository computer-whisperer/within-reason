#!/usr/bin/env python3
"""A Jev classifier swept over a pianist match's own decisions, in hindsight: every unscripted menu answer in
`jev-<ai>.jsonl` is put back to Jev with the packet, the menu, what was played and what happened in the next
half minute, and five yes/no questions are asked of it (docs/studies/2026-09-22-jev-deep-dive.md). Writes
`jev-sweep.jsonl` beside the match's record; `--summarize` reads that back and lists the moments each question
flags most strongly.

usage: run/jev_sweep.py run/matches/<batch>/<NN> [...] [--workers N] [--limit N] [--summarize] [--top N]
The key comes from TYPESAFE_API_KEY or ~/.config/within-reason/jev.env and is never printed or written.
"""
import concurrent.futures
import json
import os
import sys
import time
import urllib.error
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from match_read import Match, clock  # noqa: E402

URL = "https://api.typesafe.ai/v1/systemone"
KEY_FILE = os.path.expanduser("~/.config/within-reason/jev.env")
AFTER_FRAMES = 900

QUESTIONS = {
    "contradicts": "Does `played` go against what the player's `instructions` say `actor` should be doing in the situation described by `actor_state`, `enemy`, `ours` and `economy`?",
    "better_option": "Is there an option in `menu` that the `instructions` and the situation call for more clearly than `played`?",
    "missing_option": "Do the `instructions` or the situation call for something `actor` should do now that no option in `menu` can express?",
    "blind": "Does `afterwards` show a danger to `actor` or to our buildings that `actor_state`, `enemy` and `ours` gave no sign of?",
    "pointless_ask": "Was there no reason to ask at all: `actor` is already doing what the `instructions` want and nothing in the situation calls for a change?",
}


def api_key():
    key = os.environ.get("TYPESAFE_API_KEY")
    if not key and os.path.exists(KEY_FILE):
        for line in open(KEY_FILE):
            if line.startswith("TYPESAFE_API_KEY="):
                key = line.split("=", 1)[1].strip().strip("\"'")
    if not key:
        sys.exit(f"no API key: put TYPESAFE_API_KEY=... in {KEY_FILE}")
    return key


def ask(key, state):
    body = {"model": os.environ.get("TYPESAFE_DEFAULT_MODEL", "jev-latest"), "state": state,
            "questions": {q: {"type": "noul", "instructions": text} for q, text in QUESTIONS.items()}}
    request = urllib.request.Request(URL, data=json.dumps(body).encode(), method="POST", headers={
        "Authorization": f"Bearer {key}", "Content-Type": "application/json", "User-Agent": "within-reason-jev-sweep"})
    for attempt in range(3):
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return json.load(response)
        except urllib.error.HTTPError as e:
            if e.code in (429, 500, 502, 503) and attempt < 2:
                time.sleep(2)
                continue
            raise
    return None


def unit_positions(m):
    """unit id -> [(frame, x, z)] from the samples, for builders and labs."""
    by = {}
    for s in m.samples:
        for u in s["own"]:
            by.setdefault(u[0], []).append((s["f"], u[2], u[3]))
    return by


def position_at(track, f):
    if not track:
        return None
    best = min(track, key=lambda t: abs(t[0] - f))
    return best[1], best[2]


def afterwards(m, f, actor_pos, actor_ids):
    lines = []
    for e in m.events:
        if not (f <= e["f"] < f + AFTER_FRAMES):
            continue
        if e["k"] == "destroyed" and e.get("by") is not None:
            name = m.def_name(e["d"])
            killer = m.def_name(e["by_d"]) if e.get("by_d") is not None else "something unseen"
            d = ""
            if actor_pos:
                d = f", {((e['x'] - actor_pos[0]) ** 2 + (e['z'] - actor_pos[1]) ** 2) ** 0.5:.0f} from this actor"
            who = "this actor" if e["u"] in actor_ids else f"our {name}"
            lines.append(f"{clock(e['f'])}: {who} destroyed by {killer}{d}")
        elif e["k"] == "move_failed" and e["u"] in actor_ids:
            lines.append(f"{clock(e['f'])}: this actor could not move (blocked)")
    return lines[:12] or ["nothing of ours was destroyed in the next 30 s"]


def decisions(m):
    tracks = unit_positions(m)
    commander = next((e["u"] for e in m.events if e["k"] == "created" and m.def_name(e["d"]).endswith("com")), None)
    packet = ""
    for c in m.calls:
        if "instructions" in c:
            packet = c["instructions"]
        groups = {f"group_{g['name']}": g for g in c.get("groups", [])}
        for p in c["played"]:
            if m.scripted(c["f"], p["actor"]):
                continue
            actor = p["actor"]
            qid = f"{actor}.next" if p["kind"] == "lab" else f"{actor}.do"
            q = c["questions"].get(qid) or {}
            a = c["answers"].get(qid) or {}
            if actor in groups:
                pos = tuple(groups[actor]["at"])
                ids = set(groups[actor]["members"])
            else:
                uid = commander if actor == "commander" else int(actor.rsplit("_", 1)[1]) if actor.rsplit("_", 1)[-1].isdigit() else None
                pos = position_at(tracks.get(uid), c["f"]) if uid else None
                ids = {uid} if uid else set()
            played = p["played"]
            if p["kept"]:
                played = f"continue (Jev chose {p['choice']} at {p['probability']:.2f}, not clearly enough to change course)"
            where = {k.rsplit(".", 1)[1]: v.get("choice") for k, v in c["answers"].items() if k.startswith(actor + ".") and k != qid and v.get("type") == "choice"}
            state = {
                "clock": clock(c["f"]),
                "actor": actor,
                "actor_state": c["state"]["actors"].get(actor),
                "economy": c["state"].get("economy"),
                "enemy": {k: v for k, v in (c["state"].get("enemy") or {}).items() if k != "buildings_seen"},
                "ours": {k: v for k, v in (c["state"].get("ours") or {}).items() if k != "wrecks"},
                "instructions": packet,
                "menu": q.get("criteria") or {o: "" for o in p["options"]},
                "chosen": p["choice"],
                "chosen_probability": p["probability"],
                "parameters": where,
                "played": played,
                "did": p["did"] or "nothing",
                "afterwards": afterwards(m, c["f"], pos, ids),
            }
            yield {"f": c["f"], "actor": actor, "kind": p["kind"], "chosen": p["choice"], "played": p["played"], "kept": p["kept"], "did": p["did"], "probability": p["probability"]}, state


def sweep(m, key, workers, limit):
    out_path = os.path.join(m.dir, "jev-sweep.jsonl")
    items = list(decisions(m))
    if limit:
        items = items[:limit]
    done = 0
    tokens = 0
    started = time.time()
    with open(out_path, "w") as out, concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
        futures = {pool.submit(ask, key, state): meta for meta, state in items}
        for fut in concurrent.futures.as_completed(futures):
            meta = futures[fut]
            try:
                r = fut.result()
            except Exception as e:  # noqa: BLE001
                meta["error"] = str(e)[:200]
                out.write(json.dumps(meta) + "\n")
                continue
            meta["answers"] = {q: a.get("noul") for q, a in (r.get("answers") or {}).items()}
            meta["model"] = r.get("model")
            tokens += (r.get("usage") or {}).get("input_tokens", 0)
            out.write(json.dumps(meta) + "\n")
            done += 1
    print(f"{m.label}: {done} of {len(items)} decisions swept in {time.time() - started:.0f} s, {tokens} input tokens -> {out_path}")


def summarize(m, top):
    path = os.path.join(m.dir, "jev-sweep.jsonl")
    rows = [json.loads(line) for line in open(path)]
    rows = [r for r in rows if "answers" in r]
    print(f"\n===== {m.label}: {len(rows)} decisions")
    for q in QUESTIONS:
        vals = sorted((r["answers"].get(q, 0) for r in rows), reverse=True)
        high = sum(1 for v in vals if v >= 0.6)
        print(f"-- {q}: {high} at 0.6 or more ({100 * high / max(len(vals), 1):.0f}%), median {vals[len(vals) // 2]:.2f}")
        for r in sorted(rows, key=lambda r: -r["answers"].get(q, 0))[:top]:
            print(f"    {r['answers'][q]:.2f} {clock(r['f'])} {r['actor']}: {r['chosen']} -> {r['played']}" + (" (kept)" if r["kept"] else "") + f" | {(r['did'] or 'nothing')[:60]}")


def main():
    args = sys.argv[1:]
    workers, limit, top = 8, 0, 8
    for flag, name in (("--workers", "workers"), ("--limit", "limit"), ("--top", "top")):
        if flag in args:
            i = args.index(flag)
            val = int(args[i + 1])
            del args[i:i + 2]
            if name == "workers":
                workers = val
            elif name == "limit":
                limit = val
            else:
                top = val
    do_summary = "--summarize" in args
    dirs = [a for a in args if not a.startswith("--")]
    if not dirs:
        sys.exit(__doc__)
    matches = [Match(d) for d in dirs]
    if do_summary:
        for m in matches:
            summarize(m, top)
        return
    key = api_key()
    for m in matches:
        sweep(m, key, workers, limit)


if __name__ == "__main__":
    main()
