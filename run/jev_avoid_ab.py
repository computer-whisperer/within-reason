#!/usr/bin/env python3
"""Offline: can Jev say which of the places a packet names for a group are places the packet forbids it? The
decode's place question reads yes for a place the group "must keep away from" as for one it is sent to, so a place
named only to forbid it stays on the group's walks (the impedance study §3.2, "a negated place is a destination").
For each packet of a recorded game and each (group, place) the game's own decode read at 0.3 or over, the forms
below are asked as nouls over the instructions alone, as the decode asks. Nothing is played.

usage: run/jev_avoid_ab.py run/matches/<batch>/<NN> --out <file.jsonl>
       run/jev_avoid_ab.py --read <file.jsonl> [--packets]
Rows: f (the packet's first frame), actor, place, place_read (the game's decode), form, noul.
The key is read by run/jev_ab.py's api_key() and never printed or written."""
import argparse, collections, glob, json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask

FORMS = {
    "plain": "Read the player's `instructions` alone. Do they forbid {actor} ({what}) to go to {place}, or tell it to keep away from it?",
    "edge": "Read the player's `instructions` alone. Do they forbid {actor} ({what}) to go to {place}, or tell it to keep away from it? No when they name {place} only as the edge of where it may go (\"never east of {place}\", \"no farther than {place}\").",
    "limits": "Read the player's `instructions` alone. Do they forbid {actor} ({what}) to go to {place}: tell it never to walk, advance or fight there, to keep away from it or to stay out of it? No when {place} is a place they send it to, have it stand, hold, fight or fall back at, or name only as the limit of how far it may go.",
}
NAMED = 0.3


def clock(f):
    return f"{f // 1800}:{f // 30 % 60:02d}"


def packets_of(log):
    """Each packet of the game: its first frame, its text, the model, the groups' words and the places read for them."""
    out, current = [], None
    for line in open(log):
        if '"t":"call"' in line[:4000] or '"t":"decode"' in line[:4000] or '"t": "call"' in line[:4000]:
            pass
        try:
            row = json.loads(line)
        except ValueError:
            continue
        if row.get("t") == "call" and isinstance(row.get("instructions"), str) and (current is None or row["instructions"] != current["text"]):
            current = {"f": row["f"], "text": row["instructions"], "model": row["model"], "what": {}, "reads": {}}
            out.append(current)
        if current is None:
            continue
        if row.get("t") == "call" and isinstance(row.get("state"), dict):
            for name, entry in (row["state"].get("actors") or {}).items():
                if name.startswith("group_") and isinstance(entry, dict):
                    current["what"][name] = str(entry.get("is") or entry.get("units") or "")[:120]
        if row.get("t") == "decode":
            for key, p in row["reads"].items():
                actor, kind, _, place = key.partition("|")[0], key.split("|")[1], None, key.split("|")[-1]
                if kind == "place" and actor.startswith("group_") and p >= NAMED:
                    current["reads"][(actor, place)] = p
    return out


def run(args):
    key = api_key()
    log = glob.glob(os.path.join(args.match, "jev-*.jsonl"))[0]
    tokens = 0
    with open(args.out, "w") as out:
        for packet in packets_of(log):
            if not packet["reads"]:
                continue
            questions = {}
            for (actor, place), p in packet["reads"].items():
                for form, text in FORMS.items():
                    questions[f"{actor}|{place}|{form}"] = {"type": "noul", "instructions": text.format(actor=actor, place=place, what=packet["what"].get(actor, "a group of ours"))}
            answer = ask(key, {"instructions": packet["text"]}, questions, packet["model"])
            tokens += (answer.get("usage") or {}).get("input_tokens", 0)
            out.write(json.dumps({"f": packet["f"], "packet": packet["text"]}) + "\n")
            for k, a in (answer.get("answers") or {}).items():
                actor, place, form = k.split("|")
                out.write(json.dumps({"f": packet["f"], "actor": actor, "place": place, "place_read": packet["reads"][(actor, place)], "form": form, "noul": a.get("noul")}) + "\n")
    print(f"{tokens} input tokens, ${tokens * 0.042 / 1e6:.3f}", file=sys.stderr)


def read(args):
    rows = [json.loads(line) for line in open(args.read)]
    packets = {r["f"]: r["packet"] for r in rows if "packet" in r}
    rows = [r for r in rows if r.get("noul") is not None]
    for form in FORMS:
        v = [r["noul"] for r in rows if r["form"] == form]
        print(f"{form}: {len(v)} answers; under 0.2: {sum(x < 0.2 for x in v)}, 0.2 to 0.8: {sum(0.2 <= x < 0.8 for x in v)}, 0.8 and over: {sum(x >= 0.8 for x in v)}")
    per = collections.defaultdict(dict)
    for r in rows:
        per[(r["f"], r["actor"], r["place"])][r["form"]] = r["noul"]
    last = None
    for (f, actor, place), forms in sorted(per.items()):
        if f != last:
            last = f
            print(f"\n--- the packet from {clock(f)} ---" + (f"\n{packets[f]}\n" if args.packets else ""))
        print(f"  {actor:12s} {place:12s} " + "  ".join(f"{form} {forms.get(form, float('nan')):.2f}" for form in FORMS))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("match", nargs="?")
    parser.add_argument("--out")
    parser.add_argument("--read")
    parser.add_argument("--packets", action="store_true")
    args = parser.parse_args()
    if args.read:
        read(args)
    elif args.match and args.out:
        run(args)
    else:
        parser.error("a match directory with --out, or --read")


if __name__ == "__main__":
    main()
