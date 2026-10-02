#!/usr/bin/env python3
"""Offline: can Jev say which of the places a packet names are a given actor's? The hands today find a group's
places by matching names in "its" paragraph (a heading that is exactly its name, else the whole packet); the
rebuild's proposal (docs/design/2026-10-01-hands-rebuild.md) is to ask Jev once per packet instead. For each
recorded packet of a game and each actor in the picture then, every place the packet names is asked as one noul.
Nothing is played.

usage: run/jev_read_ab.py run/matches/<batch>/<NN> --out <file.jsonl> [--every N] [--form about|business]
       run/jev_read_ab.py --read <file.jsonl> [--show m:ss]
Rows: f, actor, place, noul, own (the place is named in a paragraph headed by the actor's name alone: the code's
reading today; null when the actor has no such paragraph), heading (the heading of the paragraph naming the place).
The key is read by run/jev_ab.py's api_key() and never printed or written. Jev: $0.042 a million input tokens."""
import argparse, collections, concurrent.futures, json, os, re, statistics, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from match_read import Match

PLACE = re.compile(r"spot_\d+|passage_\d+|\bhome\b")
FORMS = {
    "about": "Read the player's `instructions` alone. Do they name {place} in anything they say to or about {actor} ({what}): a place it goes to, stands at, fights at, builds at, falls back to, or must keep away from?",
    "business": "Read the player's `instructions` alone. Is {place} a place they send {actor} ({what}) to, now or later: a stop of its route, its post, its target, or where it falls back to?",
}


def clock(f):
    return f"{f // 1800}:{f // 30 % 60:02d}"


def paragraphs(text):
    return [(p.split(":")[0].strip(), p) for p in text.split("\n\n")]


def own_paragraph(text, actor):
    return next((p for h, p in paragraphs(text) if h.split("(")[0].strip() == actor), None)


def what_of(entry):
    if isinstance(entry, dict):
        return (entry.get("units") or entry.get("is") or "")[:120]
    return str(entry)[:120]


def run(args):
    key = api_key()
    m = Match(args.match)
    jobs, last, n = [], None, 0
    for c, state, _ in requests_of(m):
        text = state.get("instructions")
        if not isinstance(text, str) or text == last:
            continue
        last = text
        n += 1
        if n % args.every:
            continue
        actors = {k: v for k, v in (state.get("actors") or {}).items() if k.startswith("group_") or k == "commander"}
        builders = [k for k in (state.get("actors") or {}) if k.startswith("constructor_")][:2]
        actors.update({k: state["actors"][k] for k in builders})
        places = sorted(set(PLACE.findall(text)))
        questions = {}
        for a, entry in actors.items():
            for p in places:
                questions[f"{a}|{p}"] = {"type": "noul", "instructions": FORMS[args.form].format(place=p, actor=a, what=what_of(entry))}
        jobs.append((c, {"instructions": text, "actors": state.get("actors")}, questions, text))
    print(f"{len(jobs)} packets, {sum(len(q) for _, _, q, _ in jobs)} questions", file=sys.stderr)

    def one(job):
        c, state, questions, text = job
        answer = ask(key, state, questions, c["model"])
        rows = []
        for k, a in (answer.get("answers") or {}).items():
            actor, place = k.split("|")
            own = own_paragraph(text, actor)
            heading = next((h for h, p in paragraphs(text) if re.search(re.escape(place) + r"(?![\d_])", p)), "")
            rows.append({"f": c["f"], "actor": actor, "place": place, "noul": a.get("noul"), "own": None if own is None else bool(re.search(re.escape(place) + r"(?![\d_])", own)), "heading": heading[:80]})
        return rows, (answer.get("usage") or {}).get("input_tokens", 0), c["f"], text

    tokens = 0
    with open(args.out, "w") as out, concurrent.futures.ThreadPoolExecutor(4) as pool:
        for rows, used, f, text in pool.map(one, jobs):
            tokens += used
            out.write(json.dumps({"f": f, "packet": text}) + "\n")
            for row in rows:
                out.write(json.dumps(row) + "\n")
    print(f"{tokens} input tokens, ${tokens * 0.042 / 1e6:.3f}", file=sys.stderr)


def read(args):
    rows = [json.loads(line) for line in open(args.read)]
    packets = {r["f"]: r["packet"] for r in rows if "packet" in r}
    rows = [r for r in rows if "noul" in r and r["noul"] is not None]
    v = [r["noul"] for r in rows]
    print(f"{len(rows)} answers over {len(packets)} packets; under 0.2: {sum(x < 0.2 for x in v)}, 0.2 to 0.8: {sum(0.2 <= x < 0.8 for x in v)}, 0.8 and over: {sum(x >= 0.8 for x in v)}")
    known = [r for r in rows if r["own"] is not None]
    yes = [r["noul"] for r in known if r["own"]]
    no = [r["noul"] for r in known if not r["own"]]
    if yes and no:
        print(f"actors with a paragraph headed by their name alone: places in it {len(yes)}, noul median {statistics.median(yes):.2f}, at 0.5 or over {sum(x >= 0.5 for x in yes)}; places elsewhere in the packet {len(no)}, median {statistics.median(no):.2f}, at 0.5 or over {sum(x >= 0.5 for x in no)}")
    per = collections.defaultdict(list)
    for r in rows:
        per[(r["f"], r["actor"])].append(r)
    counts = [sum(r["noul"] >= 0.5 for r in rs) for (f, a), rs in per.items() if a.startswith("group_")]
    named = [len(rs) for (f, a), rs in per.items() if a.startswith("group_")]
    print(f"a group's places at 0.5 or over: median {statistics.median(counts):.0f}, p90 {sorted(counts)[len(counts) * 9 // 10]}, of a median {statistics.median(named):.0f} named in the packet")
    if args.show:
        mm, ss = args.show.split(":")
        f = min(packets, key=lambda x: abs(x - (int(mm) * 60 + int(ss)) * 30))
        print(f"\n--- the packet in force at {clock(f)} ---\n{packets[f]}\n")
        for (ff, a), rs in sorted(per.items()):
            if ff != f:
                continue
            yes = sorted((r for r in rs if r["noul"] >= 0.5), key=lambda r: -r["noul"])
            mid = [r for r in rs if 0.2 <= r["noul"] < 0.5]
            print(f"{a}: " + ", ".join(f"{r['place']} {r['noul']:.2f}" for r in yes) + (" | 0.2 to 0.5: " + ", ".join(f"{r['place']} {r['noul']:.2f}" for r in mid) if mid else ""))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("match", nargs="?")
    ap.add_argument("--out")
    ap.add_argument("--read")
    ap.add_argument("--every", type=int, default=4)
    ap.add_argument("--form", default="about", choices=sorted(FORMS))
    ap.add_argument("--show")
    args = ap.parse_args()
    if args.read:
        read(args)
    elif args.match and args.out:
        run(args)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
