#!/usr/bin/env python3
"""The team-game set of a map: what the carded team games there look like, side by side and seat by seat, as
markdown tables with their derivation (the 8v8 mining of 2026-10-22; docs/briefs/meta/formats.md reads it).

    run/replays/team_set.py --map "All That Glitters v2.2.3" [--sides 8v8] [--ais any|none|NAME] [--min-mean-os 0]
                            [--manifest run/data/replays/manifest.jsonl] [--match DIR ...] [--chat 40]

The pool is every kept and carded manifest line on the map with the shape asked for, filtered by its AIs (`none`:
games of people only; `any`: games with an AI; a name: games with that AI), plus the match directories named with
`--match` (a card with no manifest line; given alone, without a map, shape or AI filter, they are the whole pool). A side is its ally team's seats; a side's kind is "people" or the AI's
short name ("mixed" when both). What the tables say:

- the set: games, dates, versions, durations, who won by side kind, the people's OS, the AI side's handicap;
- the side curves by minute, the seats summed (extractors, constructors, metal and energy income, army metal,
  soldiers, and when the cards carry transfers the metal and energy sent within the side by the engine's figure,
  which counts a full store's overflow, and the metal and energy given by hand, the player list's gift lines, which
  AIs never write), quartiles over sides by kind and result;
- the seats by lane (1 the west-most or north-most start of the side): what each lane's first plant was, how often
  it opened air, when it put up an advanced plant, its income and army share at 10:00 and 20:00, what it sent and
  received;
- the choreography of a side: the air seat's lane, the first advanced plant's lane and clock, how many seats had an
  advanced plant by 10:00, 15:00 and 20:00, the top receiver of metal and its role, the share of the army the three
  largest seats held, the gifts by hand and the largest flows by lane, the seats taken over after a player left;
- the unit selection by role (one role a seat: the first tech seat, the air seat, the economy seat with most
  converters and fusions, the lane seats): what each role built most, by count, over the pool;
- the players' chat lines that speak of roles and transfers (air, tech, t2, metal, energy, send, share), a sample.

Floors, not values, where the cards cut off: a side that lost a seat early sums fewer seats. Say so when quoting.
"""
import argparse, collections, json, os, re, statistics as st, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
MINUTES = [2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 24, 28]
ROLE_WORDS = re.compile(r"\b(air|tech|t2|t3|metal|energy|send|share|sent|shared|nuke|eco|front|lane|mex|con|cons|builder|builders|fus|fusion|afus)\b", re.I)


def secs(c):
    if not c:
        return None
    m, s = c.split(":")
    return int(m) * 60 + int(s)


def clock(s):
    return "-" if s is None else f"{int(s) // 60}:{int(s) % 60:02d}"


def q(xs, kind="n", digits=0):
    xs = sorted(x for x in xs if x is not None)
    if not xs:
        return "-"
    f = clock if kind == "clock" else (lambda v: f"{v:,.{digits}f}")
    if len(xs) <= 3:
        return " ".join(f(x) for x in xs)
    qs = st.quantiles(xs, n=4, method="inclusive")
    return f"{f(qs[0])} / {f(st.median(xs))} / {f(qs[2])}"


def at(team, minute, key):
    for row in team.get("curves", []):
        if row["minute"] == minute:
            return row.get(key)
    return None


def side_kind(seats):
    ais = {s.get("ai") for s in seats if s.get("ai")}
    people = [s for s in seats if not s.get("ai")]
    if ais and people:
        return "mixed"
    if len(ais) == 1:
        return next(iter(ais))
    if len(ais) > 1:
        return "+".join(sorted(ais))
    return "people"


def load(args):
    games = []
    rows = []
    # Match directories alone (no map, shape or AI filter): the manifest is not read.
    if os.path.exists(args.manifest) and not (args.match and not (args.map or args.sides or args.ais)):
        rows = [json.loads(l) for l in open(args.manifest) if l.strip()]
    for r in rows:
        if not (r.get("keep") and r.get("match")):  # the card on disk decides, not the batch's flag (a re-card after a failed one)
            continue
        if args.map and r.get("map_script") != args.map:
            continue
        if args.sides and r.get("sides") != args.sides:
            continue
        kinds = {p.get("ai") for p in r.get("players", []) if p.get("ai")}
        if args.ais == "none" and kinds or args.ais == "any" and not kinds or args.ais not in (None, "any", "none") and args.ais not in kinds:
            continue
        path = os.path.join(REPO, r["match"], "card.json")
        if os.path.exists(path):
            games.append((r, json.load(open(path))))
    for d in args.match or []:
        path = os.path.join(d, "card.json")
        if os.path.exists(path):
            games.append((None, json.load(open(path))))
    out = []
    for r, card in games:
        if not card.get("sides"):
            continue
        by_team = {t["team"]: t for t in card["teams"]}
        sides = []
        for s in card["sides"]:
            seats = [p for p in card["players"] if p.get("ally") == s["ally"]]
            teams = [by_team[p["team"]] for p in seats if p.get("team") in by_team]
            for p, t in zip([p for p in seats if p.get("team") in by_team], teams):
                t["_seat"] = p
            sides.append({"ally": s["ally"], "kind": side_kind(seats), "won": s.get("won"), "handicap": s.get("handicap", 0), "seats": seats, "teams": teams,
                          "oss": [p["os"] for p in seats if p.get("os") is not None]})
        oss = [p["os"] for p in card["players"] if p.get("os") is not None]
        if args.min_mean_os and (not oss or st.mean(oss) < args.min_mean_os):
            continue
        out.append({"id": (r or {}).get("id") or card.get("id") or card.get("match"), "date": ((r or {}).get("start_time") or "")[:10], "version": ((r or {}).get("game_version") or card.get("game_version") or "")[-13:],
                    "duration": secs(card.get("duration")), "sides": sides, "card": card, "match": card.get("match"), "ended": (r or {}).get("ended")})
    return out


def side_sum(side, minute, key):
    """The side's seats summed at the minute; `gift:metal` and `gift:energy` sum the player list's gift lines to the
    minute (None before the cards carried transfers, and after the game's end)."""
    if key.startswith("gift:"):
        if not any(t.get("transfers") for t in side["teams"]) or all(at(t, minute, "extractors") is None for t in side["teams"]):
            return None
        return sum(g["amount"] for t in side["teams"] for g in t.get("gifts") or [] if g["resource"] == key[5:] and secs(g["clock"]) <= minute * 60)
    vals = [at(t, minute, key) for t in side["teams"]]
    vals = [v for v in vals if v is not None]
    return sum(vals) if vals else None


def opened_air(t):
    return (t.get("factories") or [{}])[0].get("kind") in ("air", "advanced air", "seaplanes")


def roles_of(side):
    """One role a seat: tech (the side's earliest advanced plant), air (the first plant an air plant, when it is not
    the tech seat), eco (the most converters and fusions, at least four, among the rest), lane (the rest). The sets
    hold one word each; the choreography counts air seats from the plants themselves, so an air seat that teched
    first is still an air opening there."""
    roles = {t["team"]: set() for t in side["teams"]}
    adv = [(secs(t["first_advanced_plant"]["started"]), t["team"]) for t in side["teams"] if t.get("first_advanced_plant")]
    if adv:
        roles[min(adv)[1]].add("tech")
    for t in side["teams"]:
        if not roles[t["team"]] and opened_air(t):
            roles[t["team"]].add("air")
    eco = []
    for t in side["teams"]:
        if roles[t["team"]]:
            continue
        b = t.get("built") or {}
        n = sum(v for k, v in b.items() if k.endswith(("makr", "mmkr", "fus", "afus", "ckfus", "geo", "ageo", "gmm")))
        eco.append((n, t["team"]))
    if eco and max(eco)[0] >= 4:
        roles[max(eco)[1]].add("eco")
    for team, r in roles.items():
        if not r:
            r.add("lane")
    return roles


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--map")
    ap.add_argument("--sides")
    ap.add_argument("--ais")
    ap.add_argument("--min-mean-os", type=float, default=0)
    ap.add_argument("--manifest", default=os.path.join(REPO, "run", "data", "replays", "manifest.jsonl"))
    ap.add_argument("--match", nargs="*")
    ap.add_argument("--chat", type=int, default=40)
    a = ap.parse_args()
    games = load(a)
    if not games:
        sys.exit("no carded game matches")
    out = []
    sides = [(g, s) for g in games for s in g["sides"]]
    kinds = collections.Counter(s["kind"] for _, s in sides)
    out.append(f"# Team set: {a.map or 'every map'}, {a.sides or 'any shape'}, AIs {a.ais or 'any'}")
    out.append("")
    out.append(f"{len(games)} games, {len(sides)} sides; side kinds {dict(kinds)}. Dates {min(g['date'] for g in games) or '?'} to {max(g['date'] for g in games) or '?'}; "
               f"versions {dict(collections.Counter(g['version'] for g in games))}. Duration p25 / median / p75: {q([g['duration'] for g in games], 'clock')}.")
    won = collections.Counter((s["kind"], s["won"]) for _, s in sides)
    out.append("Results by side kind: " + "; ".join(f"{k} won {won[(k, True)]}, lost {won[(k, False)]}" + (f", unknown {won[(k, None)]}" if won[(k, None)] else "") for k in kinds))
    people = [s for _, s in sides if s["kind"] == "people"]
    if people:
        means = [st.mean(s["oss"]) for s in people if s["oss"]]
        lows = [min(s["oss"]) for s in people if s["oss"]]
        highs = [max(s["oss"]) for s in people if s["oss"]]
        out.append(f"The people's sides: mean OS p25 / median / p75 {q(means, digits=1)}; the lowest seat {q(lows, digits=1)}, the highest {q(highs, digits=1)}.")
    hc = collections.Counter((s["kind"], s["handicap"]) for _, s in sides if s["kind"] != "people")
    if hc:
        out.append("The AI sides' handicap: " + ", ".join(f"{k} +{h} in {n}" for (k, h), n in sorted(hc.items())) + ".")
    groups = collections.OrderedDict()
    for g, s in sides:
        groups.setdefault((s["kind"], s["won"]), []).append(s)
    groups = collections.OrderedDict((k, v) for k, v in groups.items() if len(v) >= 2 or len(groups) <= 2)

    # Side curves
    out += ["", "## The sides by minute (seats summed; p25 / median / p75 over sides, or the values when three or fewer)", ""]
    for key, label in (("extractors", "extractors"), ("constructors", "constructors"), ("metal_income", "metal income"), ("energy_income", "energy income"), ("army_metal", "army metal"), ("soldiers", "soldiers"),
                       ("metal_sent", "metal sent within the side, cumulative (the engine's figure: gifts by hand and a full store's overflow)"), ("energy_sent", "energy sent within the side, cumulative (the engine's figure)"),
                       ("gift:metal", "metal given by hand, cumulative (the player list's gift lines; AIs write none)"), ("gift:energy", "energy given by hand, cumulative (the gift lines)")):
        cols = [f"{k} {'won' if w else 'lost' if w is False else 'no result'} (n={len(v)})" for (k, w), v in groups.items()]
        rows = []
        for m in MINUTES:
            cells = [q([side_sum(s, m, key) for s in v]) for v in groups.values()]
            if all(c == "-" for c in cells):
                continue
            rows.append(f"| {m}:00 | " + " | ".join(cells) + " |")
        if rows:
            out += [f"**{label}**", "", "| minute | " + " | ".join(cols) + " |", "|---|" + "---|" * len(cols)] + rows + [""]

    # Seats by lane
    out += ["", "## The seats by lane (1 the west-most or north-most start of its side)", ""]
    for (kind, w), v in groups.items():
        by_lane = collections.defaultdict(list)
        for s in v:
            for t in s["teams"]:
                if t.get("lane"):
                    by_lane[t["lane"]].append(t)
        if not by_lane:
            continue
        out += [f"**{kind} {'won' if w else 'lost' if w is False else 'no result'}** (n={len(v)} sides)", "",
                "| lane | first plant | air first | advanced plant (sides) | advanced plant clock | income 10:00 | income 20:00 | army 10:00 | army 20:00 | metal sent | by hand | metal received |", "|---|---|---|---|---|---|---|---|---|---|---|---|"]
        for lane in sorted(by_lane):
            ts = by_lane[lane]
            first = collections.Counter((t.get("factories") or [{}])[0].get("kind", "none") for t in ts)
            air = sum(1 for t in ts if opened_air(t))
            adv = [secs(t["first_advanced_plant"]["started"]) for t in ts if t.get("first_advanced_plant")]
            sent = [t["transfers"]["metal_sent"] for t in ts if t.get("transfers")]
            hand = [t["transfers"].get("metal_given", 0) for t in ts if t.get("transfers") and "metal_given" in t["transfers"]]
            recv = [t["transfers"]["metal_received"] for t in ts if t.get("transfers")]
            out.append(f"| {lane} | " + ", ".join(f"{k} {n}" for k, n in first.most_common(3)) + f" | {air} of {len(ts)} | {len(adv)} of {len(ts)} | {q(adv, 'clock')} | {q([at(t, 10, 'metal_income') for t in ts])} | {q([at(t, 20, 'metal_income') for t in ts])} | {q([at(t, 10, 'army_metal') for t in ts])} | {q([at(t, 20, 'army_metal') for t in ts])} | {q(sent)} | {q(hand)} | {q(recv)} |")
        out.append("")

    # Choreography
    out += ["", "## The choreography of a side", ""]
    for (kind, w), v in groups.items():
        air_lanes = collections.Counter()
        air_clock = []
        tech_lanes = collections.Counter()
        tech_clock = []
        n_adv = {10: [], 15: [], 20: []}
        top3 = {10: [], 20: []}
        receivers = collections.Counter()
        recv_share = []
        sent_total = {10: [], 20: []}
        sides_with_air = 0
        gifts_m, gifts_e, by_hand, seats_left = [], [], [], []
        flows = collections.Counter()
        for s in v:
            roles = roles_of(s)
            airs = [(secs(t["first_air_plant"]["started"]), t) for t in s["teams"] if t.get("first_air_plant")]
            if airs:
                c, t = min(airs, key=lambda x: x[0])
                air_clock.append(c)
                if opened_air(t):
                    sides_with_air += 1
                    air_lanes[t.get("lane")] += 1
            name_lane = {(t.get("_seat") or {}).get("name"): t.get("lane") for t in s["teams"]}
            gm = ge = 0
            for t in s["teams"]:
                for g in t.get("gifts") or []:
                    if g["resource"] == "metal":
                        gm += g["amount"]
                        flows[(t.get("lane"), name_lane.get(g["to"]))] += g["amount"]
                    else:
                        ge += g["amount"]
                for x in t.get("took_over") or []:
                    seats_left.append(secs(x["clock"]))
            if any("metal_given" in (t.get("transfers") or {}) for t in s["teams"]):
                gifts_m.append(gm)
                gifts_e.append(ge)
                by_hand.append(sum(t["transfers"].get("metal_sent_by_hand", 0) for t in s["teams"] if t.get("transfers")))
            adv = sorted(((secs(t["first_advanced_plant"]["started"]), t) for t in s["teams"] if t.get("first_advanced_plant")), key=lambda x: x[0])
            if adv:
                tech_clock.append(adv[0][0])
                tech_lanes[adv[0][1].get("lane")] += 1
            for m in n_adv:
                n_adv[m].append(sum(1 for c, _ in adv if c <= m * 60))
            for m in top3:
                armies = sorted((at(t, m, "army_metal") or 0 for t in s["teams"]), reverse=True)
                if sum(armies) > 0:
                    top3[m].append(sum(armies[:3]) / sum(armies))
            tr = [(t["transfers"]["metal_received"], t) for t in s["teams"] if t.get("transfers")]
            if tr:
                total_recv = sum(x for x, _ in tr)
                if total_recv > 0:
                    x, t = max(tr, key=lambda p: p[0])
                    receivers[(t.get("lane"), "+".join(sorted(roles[t["team"]])))] += 1
                    recv_share.append(x / total_recv)
                for m in sent_total:
                    sent_total[m].append(side_sum(s, m, "metal_sent"))
        gift_text = ""
        if gifts_m:
            top = ", ".join(f"lane {a} to lane {b} {n:,}" for (a, b), n in flows.most_common(4))
            gift_text = (f"; gifts by hand per side {q(gifts_m)} metal and {q(gifts_e)} energy (the player list's lines), metal sent with the store below nine tenths {q(by_hand)}"
                         + (f"; the largest metal flows by lane, summed over the sides: {top}" if top else "")
                         + (f"; seats taken over after a player left: {len(seats_left)} at {q(seats_left, 'clock')}" if seats_left else "; no seat taken over"))
        out.append(f"**{kind} {'won' if w else 'lost' if w is False else 'no result'}** (n={len(v)} sides): a seat opening on an air plant in {sides_with_air} of {len(v)}" + (f", at lane {dict(air_lanes.most_common(3))}" if air_lanes else "")
                   + f"; the side's first air plant at {q(air_clock, 'clock')}; the first advanced plant at {q(tech_clock, 'clock')}" + (f" by lane {dict(tech_lanes.most_common(4))}" if tech_lanes else "")
                   + f"; seats with an advanced plant by 10:00 {q(n_adv[10])}, by 15:00 {q(n_adv[15])}, by 20:00 {q(n_adv[20])}"
                   + f"; the three largest armies' share of the side's army at 10:00 {q(top3[10], digits=2)}, at 20:00 {q(top3[20], digits=2)}"
                   + (f"; metal sent within the side by 10:00 {q(sent_total[10])}, by 20:00 {q(sent_total[20])} (the engine's figure); the top receiver (lane, role) {dict(receivers.most_common(4))} with {q(recv_share, digits=2)} of what was received" if receivers else "; no transfer data (cards from before 2026-10-22)") + gift_text + ".")
    out.append("")

    # Unit selection by role
    out += ["", "## What each role built (counts summed over the pool's seats of the role; the ten most built)", ""]
    for (kind, w), v in groups.items():
        by_role = collections.defaultdict(collections.Counter)
        seats_in_role = collections.Counter()
        for s in v:
            roles = roles_of(s)
            for t in s["teams"]:
                for r in roles[t["team"]]:
                    seats_in_role[r] += 1
                    for k, n in (t.get("built") or {}).items():
                        by_role[r][k] += n
        for r in ("air", "tech", "eco", "lane"):
            if by_role.get(r):
                out.append(f"- **{kind} {'won' if w else 'lost' if w is False else 'no result'}, {r}** ({seats_in_role[r]} seats): " + ", ".join(f"{k} {n}" for k, n in by_role[r].most_common(10)))
    out.append("")

    # Chat
    lines = []
    for g in games:
        for s in g["sides"]:
            for t in s["teams"]:
                for c in t.get("chat") or []:
                    if ROLE_WORDS.search(c["text"]):
                        lines.append(f"{g['id'][:8]} {c['clock']} lane {t.get('lane')} ({(t.get('_seat') or {}).get('name')}): {c['text']}")
    if lines:
        out += ["", f"## The players' lines on roles and transfers ({len(lines)} of them; the first {a.chat})", ""]
        out += [f"- {l}" for l in lines[: a.chat]]
    out += ["", "## Games", ""]
    for g in games:
        out.append(f"- {g['id'][:8]} {g['date']} {g['version']} {clock(g['duration'])}" + (" (stopped before the end)" if g.get("ended") is False else "") + ": " + "; ".join(f"side {s['ally']} {s['kind']}" + (f" +{s['handicap']}" if s['handicap'] else "") + (" won" if s["won"] else " lost" if s["won"] is False else "") for s in g["sides"]) + f" ({g['match']})")
    print("\n".join(out))


if __name__ == "__main__":
    main()
