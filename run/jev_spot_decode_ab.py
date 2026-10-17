#!/usr/bin/env python3
"""Offline: the decode's builders-times-spots readings in two steps. Today every builder is read against every spot
("do the instructions name spot_N in anything they say to or about constructor_X"). The two-step arm reads each spot
once for all builders ("... to or about any builder") and reads builder by builder only the spots that come back at
`FUSE` or over. The pair questions are asked again as the code has them (`pair`), so the two-step arm is judged
against a fresh reading and the played one gives the noise.

    run/jev_spot_decode_ab.py <match> <decodes> [out.jsonl]      (the largest <decodes> decodes of the game)
"""
import collections, concurrent.futures, json, sys
sys.path.insert(0, "run")
from jev_ab import api_key, ask, requests_of
from match_read import Match
FUSE = 0.3
HEAD = "Read the player's `instructions` alone. "
USES = "a place it goes to, stands at, fights at, builds at, falls back to, or must keep away from?"


def builder(actor): return actor.startswith(("constructor", "commander"))


def main():
    m = Match(sys.argv[1]); n = int(sys.argv[2]); out = open(sys.argv[3], "w") if len(sys.argv) > 3 else None
    key = api_key()
    pictures = [(c["f"], state) for c, state, _ in requests_of(m)]
    decodes = sorted((c for c in m.calls if c.get("t") == "decode" and c.get("reads")), key=lambda c: -len(c["reads"]))[:n]

    def run(d):
        f, state = min(pictures, key=lambda p: abs(p[0] - d["f"]))
        def what(actor):
            e = (state.get("actors") or {}).get(actor)
            return (e.get("units") or e.get("is") or "")[:120] if isinstance(e, dict) else ""
        pairs = [k for k in d["reads"] if k.count("|") == 2 and k.split("|")[1] == "place" and builder(k.split("|")[0]) and k.split("|")[2].startswith("spot_")]
        spots = sorted({k.split("|")[2] for k in pairs})
        st = {"instructions": state.get("instructions", "")}
        def asked(q):
            answers, tokens, chunk, size, parts = {}, 0, {}, len(json.dumps(st)), []
            for k, v in q.items():
                L = len(json.dumps(v)) + len(k) + 6
                if chunk and size + L > 140_000:
                    parts.append(chunk); chunk = {}; size = len(json.dumps(st))
                chunk[k] = v; size += L
            if chunk: parts.append(chunk)
            for part in parts:
                r = ask(key, st, part, "jev-1.14.0")
                answers.update({k: a.get("noul") for k, a in (r.get("answers") or {}).items() if isinstance(a, dict)}); tokens += (r.get("usage") or {}).get("input_tokens", 0)
            return answers, tokens
        pair_q = {k: {"type": "noul", "instructions": HEAD + f"Do they name {k.split('|')[2]} in anything they say to or about {k.split('|')[0]} ({what(k.split('|')[0])}): " + USES} for k in pairs}
        spot_q = {s: {"type": "noul", "instructions": HEAD + f"Do they name {s} in anything they say to or about any builder of ours (a constructor or the commander): " + USES} for s in spots}
        pair, pair_tok = asked(pair_q); spot, spot_tok = asked(spot_q)
        # The second step's cost: the pairs at the open spots, asked as a request of their own.
        second = {k: v for k, v in pair_q.items() if (spot.get(k.split("|")[2]) or 0) >= FUSE}
        _, second_tok = asked(second) if second else ({}, 0)
        return d, pairs, pair, spot, pair_tok, spot_tok + second_tok, len(spots), len(second)

    c = collections.Counter()
    with concurrent.futures.ThreadPoolExecutor(6) as ex:
        for d, pairs, pair, spot, pair_tok, two_tok, n_spots, n_second in ex.map(run, decodes):
            if out: out.write(json.dumps({"f": d["f"], "played": {k: d["reads"][k] for k in pairs}, "pair": pair, "spot": spot}) + "\n")
            c["decodes"] += 1; c["pair tokens"] += pair_tok; c["two tokens"] += two_tok; c["spots"] += n_spots; c["second"] += n_second
            for k in pairs:
                a, b, s = d["reads"][k], pair.get(k), spot.get(k.split("|")[2])
                if b is None or s is None: continue
                c["pairs"] += 1
                c["noise"] += (a < FUSE) != (b < FUSE)
                kept = b >= FUSE; open_ = s >= FUSE
                c["kept"] += kept; c["yes"] += b >= 0.5
                c["kept, spot shut"] += kept and not open_
                c["yes, spot shut"] += b >= 0.5 and not open_
                c["yes in both, spot shut"] += b >= 0.5 and a >= 0.5 and not open_
                c["off, spot open"] += (not kept) and open_
    print(f"{c['decodes']} decodes, {c['pairs']} builder-spot readings over {c['spots']} spot readings")
    print(f"  as the code asks: {c['pair tokens']/1e6:.2f}M tokens; two steps: {c['two tokens']/1e6:.2f}M ({100*c['two tokens']/max(1,c['pair tokens']):.0f}%), {c['spots']} spot questions and {c['second']} pair questions")
    print(f"  the pair reading asked twice lands on the other side of {FUSE}: {c['noise']} ({100*c['noise']/max(1,c['pairs']):.1f}%)")
    print(f"  pairs kept by the fresh pair reading: {c['kept']}; of them at a spot the spot reading shuts: {c['kept, spot shut']} ({100*c['kept, spot shut']/max(1,c['kept']):.1f}%)")
    print(f"  pairs read yes (0.5 or over): {c['yes']}; at a shut spot: {c['yes, spot shut']} ({100*c['yes, spot shut']/max(1,c['yes']):.1f}%); yes in the played reading too: {c['yes in both, spot shut']}")
    print(f"  pairs fused off that the second step still asks (the spot is open for someone): {c['off, spot open']}")


if __name__ == "__main__":
    main()
