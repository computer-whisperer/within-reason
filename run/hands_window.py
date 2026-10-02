#!/usr/bin/env python3
"""What the hands did in a window of game time, second by second, from the pianist's log: the events the pass saw,
the parties in the picture, the named actors' groups and open states with the gate's flags, the plays of every
source (the rules' defaults, the lists, the pick) and the pick itself with its probability. The reviewer's
close-up of a raid or a fight (`.claude/skills/bar-review/SKILL.md`), after `run/raid_ledger.py` or
`run/analyze_match.py` has named the clock.

    run/hands_window.py run/matches/<batch>/<NN> <from m:ss> <to m:ss> [actor ...]

Actors are the picture's names (`commander`, `constructor_N`, `plant_N`, `group_A`); by default the groups and
the commander. Reads `jev-<ai>.jsonl` (docs/harness/record-format.md, "The pianist's log"): `pass` lines (events,
quiet; the menus of version 3, the slots of version 2), `worlds_gate` lines (the gate's ratings by id), `plan`
lines (the pick) and `call` lines (parties and groups). Of version 3 every party's `answer` is printed and each
named actor's menu with its six best-rated moves; of version 2 a threat slot is printed whoever it belongs to.
Seconds with nothing to say for the actors named are skipped.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from match_read import Match, clock  # noqa: E402


def frame_of(text):
    m, s = text.split(":")
    return int(m) * 1800 + int(s) * 30


def main():
    args = sys.argv[1:]
    if len(args) < 3 or "-h" in args or "--help" in args:
        print(__doc__)
        sys.exit(2)
    m = Match(args[0].rstrip("/"))
    lo, hi = frame_of(args[1]), frame_of(args[2])
    actors = set(args[3:]) or None
    want = lambda a: (a.startswith("group") or a == "commander") if actors is None else a in actors
    by_frame = {}
    for line in m.calls:
        if line.get("f") is not None:
            by_frame.setdefault(line["f"], []).append(line)
    last_parties = None
    for f in sorted(k for k in by_frame if lo <= k <= hi):
        lines = by_frame[f]
        pas = next((x for x in lines if x.get("t") == "pass"), None)
        gate = next((x for x in lines if x.get("t") == "worlds_gate"), None)
        plan = next((x for x in lines if x.get("t") == "plan"), None)
        is_pick = lambda x: any(q.startswith("worlds.") for q in (x.get("questions") or {}))
        call = next((x for x in lines if x.get("t") == "call" and not is_pick(x)), None)
        pick_call = next((x for x in lines if x.get("t") == "call" and is_pick(x)), None)
        out = []
        if pas and pas.get("events"):
            out.append("events: " + "; ".join(pas["events"]))
        if call:
            parties = [f"{p['name']}={p['composition']}@({p['x']},{p['z']})" for p in call.get("parties") or []]
            if parties != last_parties:
                out.append("parties: " + (", ".join(parties) or "none"))
                last_parties = parties
            for g in call.get("groups") or []:
                name = f"group_{g['name']}"
                if actors is None or name in actors:
                    task = g.get("task") or {}
                    where = task.get("place") or task.get("party") or ""
                    out.append(f"{name}: {len(g.get('members') or [])} at {tuple(g.get('at') or ())}, task {task.get('kind')} {where}".rstrip())
        flags = (gate or {}).get("flags") or {}
        for s in (pas or {}).get("slots") or []:
            states = []
            for st in s.get("states") or []:
                short = st["id"].split(".", 1)[1] if "." in st["id"] else st["id"]
                p = flags.get(st["id"])
                if str(s.get("kind", "")).startswith("threat") or p is not None or st.get("current"):
                    states.append(f"{short}{'*' if st.get('current') else ''}={p if p is not None else '-'}")
            if str(s.get("kind", "")).startswith("threat"):
                out.append(f"THREAT {s['name']} ({s['kind']}) base={s.get('base')} " + " ".join(states))
            elif want(s["name"]) and states:
                out.append(f"slot {s['name']} base={s.get('base')} " + " ".join(states))
        # The rebuilt hands (log version 3): one menu per actor, a move's rating by its id; `~` marks an actor the
        # news layer closed, `?` one it closed and asked anyway for its audit.
        for name in sorted(k[: -len(".answer")] for k in flags if k.endswith(".answer")):
            out.append(f"PARTY {name} answer={flags[name + '.answer']}")
        for m in (pas or {}).get("menus") or []:
            if not want(m["name"]):
                continue
            rated = [(flags[mv["id"]], mv["id"].split(".", 1)[1]) for mv in m.get("moves") or [] if mv["id"] in flags]
            rated.sort(reverse=True)
            mark = "?" if m.get("audit") else "~" if m.get("quiet") else ""
            change = flags.get(m["name"] + ".change")
            if rated or change is not None:
                out.append(f"menu {m['name']}{mark} [{m.get('course')}{', idle' if m.get('idle') else ''}] change={change if change is not None else '-'} of {len(m.get('moves') or []) - 1}: " + " ".join(f"{k}={p}" for p, k in rated[:6]))
        plays = [f"{p['actor']}: {(p.get('did') or '')[:60]} [{p.get('source')}]" for x in lines for p in (x.get("played") or []) if want(p.get("actor", ""))]
        if plays:
            out.append("PLAYS " + " | ".join(plays))
        if plan and "split" in plan:
            parts = "; ".join(f"{'+'.join(c['actors'])}: {'took' if c.get('taken') else 'kept w1 over'} w{c['candidate']} of {c['changes']} at {c.get('confidence')}" for c in plan["split"])
            out.append(f"PICK split [{parts}] changed={[c[:50] for c in plan.get('changed') or []]}")
        elif plan:
            probs = ((pick_call or {}).get("answers") or {}).get("worlds.pick", {}).get("probabilities", {})
            out.append(f"PICK w{plan.get('pick')} p={probs.get('w%s' % plan.get('pick'))} confidence={plan.get('confidence')} changed={[c[:50] for c in plan.get('changed') or []]}")
        elif pas and pas.get("quiet") and not plays:
            continue
        if out:
            print(clock(f) + "\n     " + "\n     ".join(out))


if __name__ == "__main__":
    main()
