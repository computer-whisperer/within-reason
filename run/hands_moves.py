#!/usr/bin/env python3
"""What the rebuilt hands picked, by verb, and how the picks stood to the fights the actors were in
(docs/design/2026-10-01-hands-rebuild.md §10: a named fall-back place taken in the seconds the odds are against the
group; no walk home that the packet did not order). Reads `jev-<ai>.jsonl` of version 3: each `plan` line's played
moves, with the move's words from the `pass` line of the same second.

    run/hands_moves.py run/matches/<batch>/<NN> [--show VERB]

Prints, per kind of actor, the picks by verb; for a group's walks (`go`) how many stepped back from a party and
with what odds; for its attacks the odds it attacked at; the groups made and merged; and with `--show VERB` each
pick of that verb with its words.
"""
import collections
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from match_read import Match, clock  # noqa: E402

VERBS = ("fight_to", "take_apart", "go", "attack", "send", "shell", "join", "follow", "build", "help", "repair", "dgun", "make", "hold", "gather", "scout")
ODDS = ("we outweigh it heavily", "we outweigh it", "an even fight", "it outweighs us", "it is unarmed", "we cannot hit it", "it cannot hit us")


def verb_of(move_id):
    key = move_id.split(".", 1)[1] if "." in move_id else move_id
    return next((v for v in VERBS if key.startswith(v)), key)


def odds_of(words):
    return next((o for o in ODDS if o in words), "no party in its entry")


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    show = sys.argv[sys.argv.index("--show") + 1] if "--show" in sys.argv else None
    if show in args:
        args.remove(show)
    if len(args) != 1:
        print(__doc__)
        sys.exit(2)
    m = Match(args[0].rstrip("/"))
    words = {}
    picks = collections.defaultdict(collections.Counter)
    back, attacks, homes = collections.Counter(), collections.Counter(), 0
    made = merged = 0
    for c in m.calls:
        if c.get("t") == "pass" and c.get("menus"):
            words = {mv["id"]: mv["words"] for menu in c["menus"] for mv in menu["moves"]}
            packet_home = None
        if c.get("t") != "plan":
            continue
        for p in c.get("played") or []:
            if p.get("source") != "plan":
                continue
            verb = verb_of(p.get("played", ""))
            picks[p.get("kind")][verb] += 1
            said = words.get(p.get("played"), "")
            if p.get("kind") == "group":
                if verb == "go":
                    step = re.search(r"; (stepping back from|closing on|staying as near to) (party_\d+)(.*)", said)
                    back[(step.group(1), odds_of(step.group(3))) if step else ("no party in its entry", "")] += 1
                    homes += said.split(" walks to ")[-1].startswith("home ")
                elif verb == "attack":
                    attacks[odds_of(said.split("):", 1)[-1])] += 1
                elif verb == "send":
                    made += 1
                elif verb == "join":
                    merged += 1
            if show and verb == show:
                print(f"{clock(c['f'])} {p.get('actor')}: {said[:400]}")
    for kind in sorted(picks, key=str):
        print(f"{kind}: " + ", ".join(f"{v} {n}" for v, n in picks[kind].most_common()))
    print(f"groups' walks: " + (", ".join(f"{how} ({odds}) {n}" if odds else f"{how} {n}" for (how, odds), n in back.most_common()) or "none") + f"; to home {homes}")
    print("groups' attacks on a party, by the odds on the line: " + (", ".join(f"{o} {n}" for o, n in attacks.most_common()) or "none"))
    print(f"detachments sent {made}, joins {merged}")


if __name__ == "__main__":
    main()
