#!/usr/bin/env python3
"""One Haiku exchange in place of a frame's Jev sequence (gate, split, pick), replayed offline from a `jev-<n>.jsonl`.

The request: the standing text and the player's packet as a cached system block (they change on 18 of 262 frames in
human-26); the picture as a compact JSON turn, then per actor one `keep` line and its options as code's own words
(the "The move: ..." part of each gate noul, the repeated preamble dropped); one JSON object back naming, per actor,
`keep` or the option id. The frame's Jev calls (their tokens, their milliseconds in sequence) and Jev's plan
(`changed` in the frame's `plan` record) are printed beside Haiku's answer.

    run/haiku_one_pass.py JEV_LOG                       # 8 consecutive frames from the middle of the game
    run/haiku_one_pass.py JEV_LOG --frames 12567,7695   # these frames
    run/haiku_one_pass.py JEV_LOG --sizes median,p90,max
    run/haiku_one_pass.py JEV_LOG --no-thinking --why   # thinking off; a short reason per changed actor

Key: `ANTHROPIC_API_KEY` or `~/.config/within-reason/anthropic.env` (run/haiku_rate_probe.py).
"""
import jevlog
import argparse, collections, json, re, statistics, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from haiku_rate_probe import api_key, stream, cost, DEFAULT_MODEL

INTRO = ("You are the hands of a player of Beyond All Reason: every second you read a picture of the game and decide, "
         "for every actor listed under `menus`, whether it keeps its course or takes one of the options code has enumerated "
         "for it. Every fact in an option's words (walking seconds, odds, counts) is computed by code: read them, do not "
         "recompute them. The player's standing instructions under `instructions` are what the actors serve.\n\n"
         "Answer with one JSON object and nothing else: for each actor name, either \"keep\" or the id of one of its options")

ACTORS = ("commander", "constructor_", "group_", "lab_")
KEEP_RE = re.compile(r"do something other than what it does now \((.*)\)\? Yes when", re.S)
MOVE_RE = re.compile(r"The move: (.*)", re.S)
VERSUS_RE = re.compile(r"\? The move: ")


def load(path):
    header, calls, plans = None, [], {}
    rules = instructions = ""
    for line in jevlog.lines(path):
        try:
            d = json.loads(line)
        except json.JSONDecodeError:
            break
        t = d.get("t")
        if t == "header":
            header = d
            rules = d.get("rules", "")
        elif t == "call":
            rules = d.get("rules", rules)
            instructions = d.get("instructions", instructions)
            if d.get("usage"):
                calls.append({**d, "rules": rules, "instructions": instructions})
        elif t == "plan":
            plans[d["f"]] = d
    frames = collections.OrderedDict()
    for c in calls:
        frames.setdefault(c["f"], []).append(c)
    return header, frames, plans


def menus_of(gate):
    """Per actor: the keep words and the options (id -> the move's words), from the gate's nouls."""
    menus = collections.OrderedDict()
    for qid, q in gate["questions"].items():
        actor, _, option = qid.rpartition(".")
        if option == "forbidden" or not actor.startswith(ACTORS):  # a party's or a place's nouls are not an actor's menu
            continue
        text = q["instructions"] if isinstance(q["instructions"], str) else json.dumps(q["instructions"])
        m = menus.setdefault(actor, {"keep": None, "options": collections.OrderedDict()})
        if option == "change":
            k = KEEP_RE.search(text)
            m["keep"] = k.group(1) if k else text
            continue
        mv = MOVE_RE.search(text)
        m["options"][option] = mv.group(1).strip() if mv else text
    for actor, m in list(menus.items()):
        if not m["options"]:
            del menus[actor]
        elif m["keep"] is None:
            m["keep"] = "its course as its entry under `actors` says"
    return menus


def render(gate, why):
    system_text = INTRO + (" and, for each actor that changes, a `why` of at most twelve words" if why else "") + \
        ". The shape: {\"<actor>\": \"keep\" | \"<option id>\"" + (", \"<actor>.why\": \"...\"" if why else "") + ", ...}.\n\n" + \
        "What you know about the game:\n" + gate["rules"] + "\n\nThe player's standing instructions:\n" + gate["instructions"]
    system = [{"type": "text", "text": system_text, "cache_control": {"type": "ephemeral"}}]
    menus = menus_of(gate)
    state = {k: v for k, v in gate["state"].items() if k != "player"}
    state["player"] = gate["state"].get("player")
    user = "Picture:\n" + json.dumps(state, separators=(",", ":")) + "\n\nMenus:\n" + \
        json.dumps({a: {"keep": m["keep"], "options": m["options"]} for a, m in menus.items()}, separators=(",", ":"))
    return system, user, menus


def parse(text):
    t = text.strip()
    if t.startswith("```"):
        t = t.strip("`")
        t = t.split("\n", 1)[1] if "\n" in t else t
    try:
        return json.loads(t)
    except json.JSONDecodeError:
        m = re.search(r"\{.*\}", t, re.S)
        return json.loads(m.group(0)) if m else None


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("jev_log")
    ap.add_argument("--frames", help="comma-separated frame numbers")
    ap.add_argument("--sizes", help="comma-separated: median, p90, max (by the gate's Jev tokens)")
    ap.add_argument("--count", type=int, default=8, help="consecutive frames from the middle when neither is given")
    ap.add_argument("--model", default=DEFAULT_MODEL)
    ap.add_argument("--effort", default="low", choices=["low", "medium", "high"])
    ap.add_argument("--no-thinking", action="store_true")
    ap.add_argument("--why", action="store_true")
    ap.add_argument("--max-tokens", type=int, default=4096)
    ap.add_argument("--dry", action="store_true", help="render only: sizes, no call")
    args = ap.parse_args()

    header, frames, plans = load(args.jev_log)
    gated = [f for f, calls in frames.items() if any(q["type"] == "noul" for q in calls[0]["questions"].values())]
    if args.frames:
        chosen = [int(f) for f in args.frames.split(",")]
    elif args.sizes:
        toks = sorted((frames[f][0]["usage"]["input_tokens"], f) for f in gated)
        idx = {"median": len(toks) // 2, "p90": int(0.9 * (len(toks) - 1)), "max": len(toks) - 1}
        chosen = [toks[idx[s]][1] for s in args.sizes.split(",")]
    else:
        mid = len(gated) // 2
        chosen = gated[mid:mid + args.count]
    key = None if args.dry else api_key()
    print(f"model {args.model}, effort {args.effort}, thinking {'off' if args.no_thinking else 'adaptive'}{', why' if args.why else ''}\n")

    rows = []
    for f in chosen:
        calls = frames[f]
        gate = calls[0]
        system, user, menus = render(gate, args.why)
        jev_tokens = sum(c["usage"]["input_tokens"] for c in calls)
        jev_ms = sum(c["ms"] for c in calls)
        plan = plans.get(f, {})
        print(f"== frame {f} ({gate['state'].get('clock')}): Jev {len(calls)} calls, {jev_tokens} tokens, {jev_ms} ms in sequence; "
              f"{len(menus)} actors, {sum(len(m['options']) for m in menus.values())} options; "
              f"one-pass chars: system {len(system[0]['text'])}, turn {len(user)}")
        if args.dry:
            continue
        r = stream(key, args.model, system, user, args.effort, not args.no_thinking, args.max_tokens)
        if r.get("error"):
            print("  error:", r["error"])
            continue
        if r["stop_reason"] == "refusal":
            print("  refusal:", r["stop_details"])
            continue
        u = r["usage"]
        r["cost"] = cost(args.model, u)
        rows.append((r, jev_tokens, jev_ms))
        print(f"  haiku: in {u.get('input_tokens', 0)} fresh (+{u.get('cache_creation_input_tokens', 0)} cache write, "
              f"{u.get('cache_read_input_tokens', 0)} cache read), first token {1000 * (r['first_token_s'] or 0):.0f} ms, "
              f"out {u.get('output_tokens', 0)}, wall {1000 * r['wall_s']:.0f} ms, ${r['cost']:.5f}, {r['stop_reason']}")
        answer = parse(r["text"])
        if not isinstance(answer, dict):
            print("  unparsed:", r["text"][:400])
            continue
        changes = {a: v for a, v in answer.items() if not a.endswith(".why") and v != "keep"}
        unknown = [a for a, v in changes.items() if a not in menus or v not in menus[a]["options"]]
        print(f"  haiku changes {len(changes)} of {len(menus)}: " + "; ".join(
            f"{a} -> {v}" + (f" ({answer.get(a + '.why')})" if args.why and answer.get(a + '.why') else "") for a, v in changes.items()))
        if unknown:
            print("  not on the menu:", unknown)
        print(f"  jev plan: changed {plan.get('changed')}, played {[p.get('id') or p.get('actor') for p in plan.get('played', [])]}")
        print()

    if rows:
        med = lambda fn: statistics.median(fn(r, jt, jm) for r, jt, jm in rows)
        print(f"median over {len(rows)} frames: haiku wall {1000 * med(lambda r, jt, jm: r['wall_s']):.0f} ms "
              f"(first token {1000 * med(lambda r, jt, jm: r['first_token_s'] or 0):.0f}), fresh input "
              f"{med(lambda r, jt, jm: r['usage'].get('input_tokens', 0)):.0f}, cache read {med(lambda r, jt, jm: r['usage'].get('cache_read_input_tokens', 0)):.0f}, "
              f"out {med(lambda r, jt, jm: r['usage'].get('output_tokens', 0)):.0f}, ${med(lambda r, jt, jm: r['cost']):.5f} a frame; "
              f"Jev {med(lambda r, jt, jm: jt):.0f} tokens, {med(lambda r, jt, jm: jm):.0f} ms a frame")


if __name__ == "__main__":
    main()
