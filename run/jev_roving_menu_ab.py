#!/usr/bin/env python3
"""Offline: would a roving group that keeps a menu be pulled off its rove by Jev? (the rove rework, 2026-10-08:
roving as a group task with a menu, the way back a pick instead of the player's `lane` setting.)

A roving group has no menu in the recorded games, so none was asked. For gates where a group was roving, this
tool synthesizes the menu the rework would give it (`change`, `hold`, `go_home`, a walk to the extractor nearest
home, a join of the largest other group, an attack on the party near it) in the shapes the hands use, with the
group's entry reworded as the rework would word it, and asks Jev under four arms:

    kept       the recorded packet, the entry in the rework's words (what the game would see)
    recorded   the recorded packet and the recorded entry words ("in code, beyond your hands' reach")
    takeback   the group's paragraph replaced by one that calls it home: the moves should come up
    silent     the group's paragraph removed: what Jev does with a roving group nobody mentions

The key is read by run/jev_ab.py's api_key() and never printed or written.

    run/jev_roving_menu_ab.py <match dir>... [--gates N per game] [--arms a,b] [--out file.jsonl] [--list]
    run/jev_roving_menu_ab.py --read file.jsonl
"""
import argparse, collections, concurrent.futures, json, math, os, random, re, statistics, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from match_read import Match

ARMS = ["kept", "recorded", "takeback", "silent"]
TAKEBACK = "{name}: its roving is over now: it walks to home and holds there as the picket; any raider at our home extractors it attacks."
LANE_WORDS = "roving: its soldiers look at the places nothing of ours has seen (his start box first), kill what they find unguarded (extractors, radars, lone constructors) and keep out of the reach of anything that can shoot it; a pick of yours brings them back"


def clock(f):
    return f"{f // 1800}:{f // 30 % 60:02d}"


def way_words(d):
    if d < 300:
        return "right here, a few seconds of walking"
    if d < 1000:
        return f"near, {d / 100:.0f}00 elmos of walking"
    return f"some way off, {d:.0f} elmos of walking"


def doing_words(entry):
    """The rove's own words from the recorded entry: after "roving, in code, beyond your hands' reach: "."""
    doing = entry.get("doing", "")
    return doing.split(": ", 1)[1] if ": " in doing else doing


def reworded(entry):
    e = dict(entry)
    e["doing"] = "roving: " + doing_words(entry)
    if "lane" in e:
        e["lane"] = LANE_WORDS
    return e


def roving_in(c, state):
    for name, e in state.get("actors", {}).items():
        if isinstance(e, dict) and str(e.get("doing", "")).startswith("roving"):
            yield name, e


def menu_for(c, state, name, entry):
    """The questions the rework's menu would ask of a roving group, in the hands' shapes."""
    course = f"{name} roves: {doing_words(entry)}"
    rather = f"Given `actors.{name}`, `economy`, `ours` and the player's `instructions`: is this what {name} should do now, rather than {course}? The move: "
    at = entry.get("at", "where it stands")
    q = {}
    q[f"{name}.change"] = {"type": "noul", "instructions": f"Given `actors.{name}`, `economy`, `enemy` and the player's `instructions`: should {name} do something other than what it does now ({course})? Yes when the instructions call for a different job now, when what it does is finished or pointless, or when something near it needs answering. No when its course is what the instructions want and nothing has changed."}
    q[f"{name}.hold"] = {"type": "noul", "instructions": rather + f"{name} holds: every soldier stops where it stands, at {at}, and the group stays there until told otherwise; leaves its roving."}
    groups = {g["name"]: g for g in c.get("groups", [])}
    me = groups.get(name.removeprefix("group_"))
    places = {p["name"]: p for p in c.get("places", [])}
    home = places.get("home")
    me_at = me["at"] if me else None
    dist = lambda p: math.dist(me_at, [p["x"], p["z"]]) if me_at else 1500.0
    if home:
        q[f"{name}.go_home"] = {"type": "noul", "instructions": rather + f"{name} walks to home ({way_words(dist(home))}) without stopping to fight on the way; leaves its roving."}
        spots = [p for p in places.values() if p.get("spot") is not None and p["name"] != "home"]
        if spots:
            nearest = min(spots, key=lambda p: math.dist([home["x"], home["z"]], [p["x"], p["z"]]))
            q[f"{name}.go_{nearest['name']}"] = {"type": "noul", "instructions": rather + f"{name} walks to {nearest['name']} ({way_words(dist(nearest))}) without stopping to fight on the way; leaves its roving."}
    def entry_of(n):
        e = state["actors"].get(f"group_{n}")
        return e if isinstance(e, dict) else {}
    others = [g for n, g in groups.items() if n != name.removeprefix("group_") and not str(entry_of(n).get("doing", "")).startswith("roving")]
    if others:
        big = max(others, key=lambda g: len(g["members"]))
        other = f"group_{big['name']}"
        oe = entry_of(big["name"])
        q[f"{name}.join_{other}"] = {"type": "noul", "instructions": rather + f"{name} ({entry.get('units', 'its soldiers')}) joins {other} ({oe.get('units', 'its soldiers')}, {way_words(math.dist(me_at, big['at']) if me_at else 1500.0)}; it {oe.get('doing', 'keeps its course')}): one body on {other}'s course; leaves its roving."}
    near = entry.get("enemies_near")
    if isinstance(near, str):
        m = re.match(r"(party_\d+) \(([^)]*)\) (\d+) from", near)
        if m:
            q[f"{name}.attack_{m.group(1)}"] = {"type": "noul", "instructions": f"Is this the move to make against {m.group(1)} now, rather than what stands (nobody moves for it)? The move: {name} attacks {m.group(1)} ({m.group(2)}) with the whole group ({way_words(float(m.group(3)))}); leaves its roving."}
    return q


def packet_for(packet, name, arm):
    if arm in ("kept", "recorded"):
        return packet
    paras = packet.split("\n\n")
    mine = [i for i, p in enumerate(paras) if p.startswith(name)]
    if arm == "takeback":
        text = TAKEBACK.format(name=name)
        if mine:
            paras[mine[0]] = text
        else:
            paras.append(text)
    elif arm == "silent":
        paras = [p for i, p in enumerate(paras) if i not in mine]
    return "\n\n".join(paras)


def one(key, game, c, state, packet, name, entry, arm):
    st = dict(state)
    actors = dict(st["actors"])
    actors[name] = entry if arm == "recorded" else reworded(entry)
    st["actors"] = actors
    st["instructions"] = packet_for(packet, name, arm)
    st = dict(sorted(st.items()))
    questions = menu_for(c, st, name, actors[name])
    r = ask(key, st, questions, "jev-latest")
    answers = {k.split(".", 1)[1]: a.get("noul") for k, a in (r.get("answers") or {}).items() if isinstance(a, dict)}
    return {"game": game, "f": c["f"], "clock": clock(c["f"]), "actor": name, "arm": arm, "answers": answers,
            "tokens": (r.get("usage") or {}).get("input_tokens", 0)}


def summarize(rows):
    by = collections.defaultdict(list)
    for r in rows:
        by[r["arm"]].append(r)
    print(f"{'arm':9s} {'n':>3s} {'change':>7s} {'ch>=.5':>6s} {'hold':>5s} {'home':>5s} {'spot':>5s} {'join':>5s} {'attack':>6s}  {'any move>=.5':>12s}")
    for arm in ARMS:
        rs = by.get(arm)
        if not rs:
            continue
        def med(key):
            xs = [r["answers"][k] for r in rs for k in r["answers"] if k == key or (key == "spot" and k.startswith("go_spot")) or (key == "join" and k.startswith("join_")) or (key == "attack" and k.startswith("attack_"))]
            xs = [x for x in xs if x is not None]
            return f"{statistics.median(xs):5.2f}" if xs else "    -"
        ch = [r["answers"].get("change") for r in rs if r["answers"].get("change") is not None]
        any_move = sum(any(v is not None and v >= 0.5 for k, v in r["answers"].items() if k != "change") for r in rs)
        print(f"{arm:9s} {len(rs):3d} {statistics.median(ch) if ch else float('nan'):7.2f} {sum(x >= 0.5 for x in ch):6d} {med('hold')} {med('go_home')} {med('spot')} {med('join')} {med('attack'):>6s}  {any_move:5d} of {len(rs)}")
    for game in sorted({r["game"] for r in rows}):
        print(f"  {game}:", ", ".join(f"{arm} any move>=.5 {sum(any(v is not None and v >= 0.5 for k, v in r['answers'].items() if k != 'change') for r in by[arm] if r['game'] == game)}/{sum(r['game'] == game for r in by[arm])}" for arm in ARMS if by.get(arm)))
    print("tokens", sum(r["tokens"] for r in rows), "≈ $", round(sum(r["tokens"] for r in rows) / 1e6 * 0.042, 2))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("matches", nargs="*")
    ap.add_argument("--gates", type=int, default=16, help="per game, spread over the roving span")
    ap.add_argument("--arms", default=",".join(ARMS))
    ap.add_argument("--out", default="docs/studies/data/jev-roving-menu.jsonl")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--read")
    ap.add_argument("--workers", type=int, default=4)
    a = ap.parse_args()
    if a.read:
        summarize([json.loads(l) for l in open(a.read)])
        return
    chosen = []
    for d in a.matches:
        game = os.path.basename(os.path.dirname(d.rstrip("/")))
        m = Match(d)
        gates = []
        for c, state, packet in requests_of(m):
            if any(k.startswith("worlds") for k in c["questions"]):
                continue
            for name, entry in roving_in(c, state):
                gates.append((game, c, state, packet, name, entry))
        step = max(1, len(gates) // a.gates)
        picked = gates[::step][: a.gates]
        print(f"{game}: {len(gates)} roving gates, replaying {len(picked)}")
        chosen.extend(picked)
    if a.list:
        for game, c, state, packet, name, entry in chosen:
            print(f"  {game} {clock(c['f'])} {name}: {list(menu_for(c, state, name, entry))}")
        return
    key = api_key()
    arms = a.arms.split(",")
    rows = []
    with concurrent.futures.ThreadPoolExecutor(a.workers) as ex, open(a.out, "w") as f:
        futures = [ex.submit(one, key, game, c, state, packet, name, entry, arm) for game, c, state, packet, name, entry in chosen for arm in arms]
        for fut in concurrent.futures.as_completed(futures):
            try:
                row = fut.result()
            except Exception as e:
                print("failed:", e, file=sys.stderr)
                continue
            rows.append(row)
            f.write(json.dumps(row) + "\n")
    summarize(rows)
    print("written", a.out)


if __name__ == "__main__":
    main()
