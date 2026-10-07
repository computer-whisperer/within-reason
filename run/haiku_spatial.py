#!/usr/bin/env python3
"""What spatial input Haiku 5.5 can work with at the one-second setting (thinking off, effort low), on mock scenarios
with code-computed ground truth.

Four scenario kinds, random instances by seed, each rendered four ways and scored by code:

  gap      a turret line with one uncovered stretch: pick the candidate spot that closes it, and give a free (x, z)
  cover    eight extractors, ten candidate turret sites: pick the two sites covering the most extractors
  rally    a group with stragglers and an enemy party near: name the stragglers and the rally point (the main body)
  route    four routes past enemy turrets to a target: pick the one that stays out of their reach

Encodings: `coords` (x, z numbers), `cells` (512-elmo grid cells like the picture's "G7"), `ascii` (coords plus a
32x32 character map), `facts` (coords plus the distances and coverages code computed: the "code computes, the model
picks" ceiling), `image` (a 1024-pixel PNG of the map with named markers, no numbers), `image_coords` (the PNG and the
numbers), `image_reach` (the PNG with the turrets' reach drawn as circles, and the numbers).

    run/haiku_spatial.py                      # 12 instances a kind, every encoding, 8 calls at once
    run/haiku_spatial.py --kinds gap,route --encodings coords,facts --n 6 --show
    run/haiku_spatial.py --thinking            # the same with adaptive thinking (slow; a ceiling check)

Key: `ANTHROPIC_API_KEY` or `~/.config/within-reason/anthropic.env`.
"""
import argparse, base64, concurrent.futures, io, json, math, random, re, statistics, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from haiku_rate_probe import api_key, stream, cost, DEFAULT_MODEL

MAP = 8192
CELL = 512
R = 450  # a Sentry's reach, roughly

SYSTEM = ("You are the hands of a player of Beyond All Reason, deciding placement and positioning on a map of "
          f"{MAP} by {MAP} elmos: x grows east, z grows south. A turret of reach R covers everything within R of it; a "
          "unit inside a turret's reach is shot. Answer with one JSON object and nothing else, in the shape the question gives.")


def dist(a, b):
    return math.hypot(a[0] - b[0], a[1] - b[1])


def cell(p):
    return f"{chr(ord('A') + min(15, int(p[0] // CELL)))}{min(16, int(p[1] // CELL) + 1)}"


def pos(p, enc):
    return cell(p) if enc == "cells" else {"x": int(p[0]), "z": int(p[1])}


def ascii_map(marks):
    """marks: list of ((x, z), char). A 32x32 map, 256 elmos a character, later marks over earlier."""
    n = MAP // 256
    grid = [["." for _ in range(n)] for _ in range(n)]
    for (x, z), ch in marks:
        i, j = min(n - 1, int(z // 256)), min(n - 1, int(x // 256))
        grid[i][j] = ch
    return "\n".join("".join(row) for row in grid)


# --- scenarios ---------------------------------------------------------------------------------------------------

def gen_gap(rng):
    z = rng.uniform(3500, 5000)
    xs, x = [], rng.uniform(1200, 1800)
    k = rng.randint(5, 6)
    gap_at = rng.randint(1, k - 2)
    gap = rng.uniform(400, 850)
    for i in range(k):
        xs.append(x)
        x += (2 * R + gap) if i == gap_at else rng.uniform(1.2 * R, 1.8 * R)
    ta, tb = xs[gap_at], xs[gap_at + 1]
    uncovered = (ta + R, tb - R)
    mid = (uncovered[0] + uncovered[1]) / 2
    cands = {"s1": (mid + rng.uniform(-60, 60), z + rng.uniform(-40, 40))}
    others = [((xs[i] + xs[i + 1]) / 2, z) for i in range(k - 1) if i != gap_at] + [(xs[0] - 1.5 * R, z), (xs[-1] + 1.5 * R, z)]
    rng.shuffle(others)
    for i, p in enumerate(others[:5]):
        cands[f"s{i + 2}"] = (p[0] + rng.uniform(-40, 40), p[1] + rng.uniform(-40, 40))
    keys = list(cands)
    rng.shuffle(keys)
    cands = {f"s{i + 1}": cands[k] for i, k in enumerate(keys)}
    right = next(k for k, p in cands.items() if abs(p[0] - mid) < 100)
    return {"kind": "gap", "z": z, "turrets": xs, "cands": cands, "right": right, "uncovered": uncovered}


def render_gap(s, enc):
    turrets = {f"t{i + 1}": pos((x, s["z"]), enc) for i, x in enumerate(s["turrets"])}
    cands = {k: pos(p, enc) for k, p in s["cands"].items()}
    d = {"our_turrets": turrets, "turret_reach": R, "enemy_comes_from": "the north (smaller z)", "candidate_spots": cands}
    if enc == "facts":
        d["each_turret_covers_x_from_to"] = {f"t{i + 1}": [int(x - R), int(x + R)] for i, x in enumerate(s["turrets"])}
        d["candidate_x"] = {k: int(p[0]) for k, p in s["cands"].items()}
    text = json.dumps(d, separators=(",", ":"))
    if enc == "ascii":
        text += "\n\nMap (one character is 256 elmos; T a turret, digits the candidate spots s1..s6, . empty):\n" + \
            ascii_map([((x, s["z"]), "T") for x in s["turrets"]] + [(p, k[1]) for k, p in s["cands"].items()])
    q = ("Raiders will cross our turret line from the north where it is not covered. Which candidate spot, given a new turret of "
         f"reach {R}, closes the uncovered stretch? Also give the best free position for it. Shape: {{\"spot\": \"sN\", \"x\": number, \"z\": number}}")
    return text + "\n\n" + q


def score_gap(s, a):
    spot = 1.0 if a.get("spot") == s["right"] else 0.0
    free = 0.0
    try:
        x = float(a.get("x"))
        lo, hi = s["uncovered"]
        if x - R <= lo and x + R >= hi:
            free = 1.0
        elif lo < x < hi:
            free = 0.5
    except (TypeError, ValueError):
        pass
    return {"spot": spot, "free": free}


def gen_cover(rng):
    ox, oz = rng.uniform(800, 3500), rng.uniform(3500, 6000)
    ext = [(ox + rng.uniform(0, 2600), oz + rng.uniform(0, 2600)) for _ in range(8)]
    cands = {}
    for i in range(10):  # between two extractors, nudged: sites cover one to three each and differ
        a, b = rng.sample(ext, 2)
        t = rng.uniform(0.2, 0.8)
        cands[f"s{i + 1}"] = (a[0] + (b[0] - a[0]) * t + rng.uniform(-150, 150), a[1] + (b[1] - a[1]) * t + rng.uniform(-150, 150))
    covers = {k: {f"e{j + 1}" for j, e in enumerate(ext) if dist(p, e) <= R} for k, p in cands.items()}
    best, best_n = None, -1
    keys = list(cands)
    for i in range(len(keys)):
        for j in range(i + 1, len(keys)):
            n = len(covers[keys[i]] | covers[keys[j]])
            if n > best_n:
                best, best_n = (keys[i], keys[j]), n
    return {"kind": "cover", "ext": ext, "cands": cands, "covers": covers, "best_n": best_n}


def render_cover(s, enc):
    d = {"our_extractors": {f"e{j + 1}": pos(e, enc) for j, e in enumerate(s["ext"])}, "turret_reach": R,
         "candidate_sites": {k: pos(p, enc) for k, p in s["cands"].items()}}
    if enc == "facts":
        d["each_site_covers"] = {k: sorted(v) for k, v in s["covers"].items()}
    text = json.dumps(d, separators=(",", ":"))
    if enc == "ascii":
        text += "\n\nMap (one character is 256 elmos; E an extractor, digits 0-9 the candidate sites s1..s10 (0 is s10), . empty):\n" + \
            ascii_map([(e, "E") for e in s["ext"]] + [(p, str(int(k[1:]) % 10)) for k, p in s["cands"].items()])
    q = (f"We can afford two turrets of reach {R}. Which two candidate sites, together, cover the most extractors? "
         "Shape: {\"sites\": [\"sA\", \"sB\"]}")
    return text + "\n\n" + q


def score_cover(s, a):
    sites = a.get("sites") or []
    if not isinstance(sites, list) or len(sites) != 2 or any(x not in s["covers"] for x in sites):
        return {"ratio": 0.0}
    n = len(s["covers"][sites[0]] | s["covers"][sites[1]])
    return {"ratio": n / s["best_n"] if s["best_n"] else 1.0}


def gen_rally(rng):
    c = (rng.uniform(2500, 5500), rng.uniform(2500, 5500))
    units = {}
    for i in range(7):
        a, r = rng.uniform(0, 2 * math.pi), rng.uniform(0, 250)
        units[f"u{i + 1}"] = (c[0] + r * math.cos(a), c[1] + r * math.sin(a))
    strag = set()
    for i in range(7, 10):
        a, r = rng.uniform(0, 2 * math.pi), rng.uniform(900, 1800)
        units[f"u{i + 1}"] = (c[0] + r * math.cos(a), c[1] + r * math.sin(a))
        strag.add(f"u{i + 1}")
    keys = list(units)
    rng.shuffle(keys)
    units = {f"u{i + 1}": units[k] for i, k in enumerate(keys)}
    strag = {f"u{i + 1}" for i, k in enumerate(keys) if k in strag}
    ea = rng.uniform(0, 2 * math.pi)
    enemy = (c[0] + 2500 * math.cos(ea), c[1] + 2500 * math.sin(ea))
    base = (c[0] - 2200 * math.cos(ea) + rng.uniform(-300, 300), c[1] - 2200 * math.sin(ea) + rng.uniform(-300, 300))
    all_c = (sum(p[0] for p in units.values()) / 10, sum(p[1] for p in units.values()) / 10)
    cands = {"main": (c[0] + rng.uniform(-80, 80), c[1] + rng.uniform(-80, 80)), "all": all_c, "base": base,
             "toward_enemy": (c[0] + 1200 * math.cos(ea), c[1] + 1200 * math.sin(ea)),
             "straggler": units[sorted(strag)[0]]}
    keys = list(cands)
    rng.shuffle(keys)
    names = {k: f"p{i + 1}" for i, k in enumerate(keys)}
    return {"kind": "rally", "units": units, "strag": strag, "enemy": enemy, "base": base, "c": c,
            "cands": {names[k]: cands[k] for k in keys}, "right": names["main"]}


def render_rally(s, enc):
    d = {"our_group": {k: pos(p, enc) for k, p in s["units"].items()}, "enemy_party": pos(s["enemy"], enc),
         "our_base": pos(s["base"], enc), "candidate_rally_points": {k: pos(p, enc) for k, p in s["cands"].items()}}
    if enc == "facts":
        d["each_unit_distance_to_the_densest_centre"] = {k: int(dist(p, s["c"])) for k, p in s["units"].items()}
        d["each_rally_point_distance_to_the_densest_centre"] = {k: int(dist(p, s["c"])) for k, p in s["cands"].items()}
    text = json.dumps(d, separators=(",", ":"))
    if enc == "ascii":
        text += "\n\nMap (one character is 256 elmos; o one of our units, X the enemy party, B our base, digits the rally points p1..p5):\n" + \
            ascii_map([(p, "o") for p in s["units"].values()] + [(s["enemy"], "X"), (s["base"], "B")] + [(p, k[1]) for k, p in s["cands"].items()])
    q = ("An enemy party approaches and our group is not one body. Which units are strung out, away from the main body, and at which "
         "candidate point should the group gather so that it fights as one? Shape: {\"stragglers\": [\"uN\", ...], \"rally\": \"pN\"}")
    return text + "\n\n" + q


def score_rally(s, a):
    got = set(a.get("stragglers") or [])
    inter, union = len(got & s["strag"]), len(got | s["strag"]) or 1
    return {"stragglers": inter / union, "rally": 1.0 if a.get("rally") == s["right"] else 0.0}


def exposure(path, turrets):
    n, inside = 400, 0
    for i in range(len(path) - 1):
        a, b = path[i], path[i + 1]
        for t in range(n):
            p = (a[0] + (b[0] - a[0]) * t / n, a[1] + (b[1] - a[1]) * t / n)
            if any(dist(p, tp) <= R for tp in turrets):
                inside += 1
    total = n * (len(path) - 1)
    return inside / total * sum(dist(path[i], path[i + 1]) for i in range(len(path) - 1))


def gen_route(rng):
    while True:
        g = (rng.uniform(1000, 3000), rng.uniform(5000, 7000))
        t = (g[0] + rng.uniform(2500, 3500), g[1] - rng.uniform(2500, 3500))
        mx, mz = (g[0] + t[0]) / 2, (g[1] + t[1]) / 2
        turrets = [(mx + rng.uniform(-900, 900), mz + rng.uniform(-900, 900)) for _ in range(3)]
        ways = {}
        for i in range(4):
            a = rng.uniform(0, 2 * math.pi)
            r = rng.uniform(600, 2200)
            ways[f"r{i + 1}"] = (min(MAP - 100, max(100, mx + r * math.cos(a))), min(MAP - 100, max(100, mz + r * math.sin(a))))
        exp = {k: exposure([g, w, t], turrets) for k, w in ways.items()}
        clean = [k for k, e in exp.items() if e == 0]
        if len(clean) == 1 and sum(1 for e in exp.values() if e > 300) >= 2:
            return {"kind": "route", "g": g, "t": t, "turrets": turrets, "ways": ways, "exp": exp, "right": clean[0]}


def render_route(s, enc):
    d = {"our_group": pos(s["g"], enc), "target": pos(s["t"], enc), "enemy_turrets": [pos(p, enc) for p in s["turrets"]], "turret_reach": R,
         "routes_via_one_waypoint": {k: pos(w, enc) for k, w in s["ways"].items()}}
    if enc == "facts":
        d["elmos_of_each_route_inside_turret_reach"] = {k: int(e) for k, e in s["exp"].items()}
    text = json.dumps(d, separators=(",", ":"))
    if enc == "ascii":
        text += "\n\nMap (one character is 256 elmos; G our group, T the target, X an enemy turret, digits the waypoints r1..r4):\n" + \
            ascii_map([(s["g"], "G"), (s["t"], "T")] + [(p, "X") for p in s["turrets"]] + [(w, k[1]) for k, w in s["ways"].items()])
    q = ("The group walks from its position to the target by one of the routes, each a straight walk to its waypoint and then to the "
         "target. Which route stays out of every enemy turret's reach? Shape: {\"route\": \"rN\"}")
    return text + "\n\n" + q


def score_route(s, a):
    return {"route": 1.0 if a.get("route") == s["right"] else 0.0}


SCHEMAS = {
    "gap": {"type": "object", "properties": {"spot": {"type": "string"}, "x": {"type": "number"}, "z": {"type": "number"}}, "required": ["spot", "x", "z"], "additionalProperties": False},
    "cover": {"type": "object", "properties": {"sites": {"type": "array", "items": {"type": "string"}}}, "required": ["sites"], "additionalProperties": False},
    "rally": {"type": "object", "properties": {"stragglers": {"type": "array", "items": {"type": "string"}}, "rally": {"type": "string"}}, "required": ["stragglers", "rally"], "additionalProperties": False},
    "route": {"type": "object", "properties": {"route": {"type": "string"}}, "required": ["route"], "additionalProperties": False},
}
KINDS = {"gap": (gen_gap, render_gap, score_gap), "cover": (gen_cover, render_cover, score_cover),
         "rally": (gen_rally, render_rally, score_rally), "route": (gen_route, render_route, score_route)}
ENCODINGS = ["coords", "cells", "ascii", "facts", "image", "image_coords", "image_reach"]
IMAGE_PX = 1024  # the map at 8 elmos a pixel


def image_of(s, reach):
    """A PNG of the scenario: markers with their names; with `reach`, the turrets' reach as circles. Base64."""
    from PIL import Image, ImageDraw, ImageFont
    img = Image.new("RGB", (IMAGE_PX, IMAGE_PX), (235, 230, 215))
    d = ImageDraw.Draw(img)
    font = ImageFont.load_default(size=16)
    k = IMAGE_PX / MAP
    for g in range(0, MAP + 1, 1024):
        d.line([(g * k, 0), (g * k, IMAGE_PX)], fill=(210, 205, 190)); d.line([(0, g * k), (IMAGE_PX, g * k)], fill=(210, 205, 190))
    def dot(p, color, name=None, size=6):
        x, z = p[0] * k, p[1] * k
        d.ellipse([x - size, z - size, x + size, z + size], fill=color, outline=(0, 0, 0))
        if name:
            d.text((x + size + 2, z - 9), name, fill=(0, 0, 0), font=font)
    def circle(p, r, color):
        x, z = p[0] * k, p[1] * k
        d.ellipse([x - r * k, z - r * k, x + r * k, z + r * k], outline=color, width=2)
    kind = s["kind"]
    if kind == "gap":
        for i, x in enumerate(s["turrets"]):
            if reach:
                circle((x, s["z"]), R, (200, 60, 60))
            dot((x, s["z"]), (200, 60, 60), f"t{i + 1}")
        for name, p in s["cands"].items():
            dot(p, (60, 90, 220), name)
        d.text((10, 10), "north (the enemy's side) is up; x grows right, z grows down", fill=(0, 0, 0), font=font)
    elif kind == "cover":
        for j, e in enumerate(s["ext"]):
            dot(e, (60, 160, 60), f"e{j + 1}")
        for name, p in s["cands"].items():
            if reach:
                circle(p, R, (60, 90, 220))
            dot(p, (60, 90, 220), name)
    elif kind == "rally":
        for name, p in s["units"].items():
            dot(p, (60, 160, 60), name)
        dot(s["enemy"], (200, 60, 60), "enemy party", 9)
        dot(s["base"], (120, 120, 120), "our base", 9)
        for name, p in s["cands"].items():
            dot(p, (60, 90, 220), name, 5)
    elif kind == "route":
        for name, w in s["ways"].items():
            d.line([(s["g"][0] * k, s["g"][1] * k), (w[0] * k, w[1] * k), (s["t"][0] * k, s["t"][1] * k)], fill=(150, 150, 150), width=1)
            dot(w, (60, 90, 220), name)
        for p in s["turrets"]:
            if reach:
                circle(p, R, (200, 60, 60))
            dot(p, (200, 60, 60))
        dot(s["g"], (60, 160, 60), "our group", 8)
        dot(s["t"], (120, 120, 120), "target", 8)
    buf = io.BytesIO()
    img.save(buf, format="PNG")
    return base64.b64encode(buf.getvalue()).decode()


LEGEND = {"gap": "red dots are our turrets t1..t6, blue dots the candidate spots s1..s6",
          "cover": "green dots are our extractors e1..e8, blue dots the candidate sites s1..s10",
          "rally": "green dots are our units u1..u10, the red dot the enemy party, the grey dot our base, blue dots the rally points p1..p5",
          "route": "the green dot is our group, the grey dot the target, red dots the enemy turrets, blue dots the waypoints r1..r4 with each route drawn grey"}


def content_of(kind, s, enc, render):
    """The user turn: a string, or content blocks when the encoding carries an image."""
    if not enc.startswith("image"):
        return render(s, enc)
    text = render(s, "coords")
    if enc == "image":  # the picture alone: strip the positions, keep the question and the reach
        text = text[text.index("\n\n") + 2:]
        text = f"turret_reach: {R}\n\n" + text if kind in ("gap", "cover", "route") else text
    legend = (f"The image is the map, {MAP} by {MAP} elmos, {MAP // IMAGE_PX} elmos a pixel, grid lines every 1024 elmos; "
              f"{LEGEND[kind]}" + ("; each turret's reach is drawn as a circle" if enc == "image_reach" else "") + ".")
    return [{"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": image_of(s, enc == "image_reach")}},
            {"type": "text", "text": legend + "\n\n" + text}]


def parse(text):
    """The answer: the last JSON object in the text (the working before it may hold braces of its own)."""
    t = text.strip().strip("`")
    if t.startswith("json"):
        t = t[4:]
    try:
        return json.loads(t)
    except json.JSONDecodeError:
        pass
    for m in reversed(list(re.finditer(r"\{[^{}]*\}", t, re.S))):
        try:
            return json.loads(m.group(0))
        except json.JSONDecodeError:
            continue
    return {}


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--kinds", default=",".join(KINDS))
    ap.add_argument("--encodings", default=",".join(ENCODINGS))
    ap.add_argument("--n", type=int, default=12)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--model", default=DEFAULT_MODEL)
    ap.add_argument("--effort", default="low", choices=["low", "medium", "high"])
    ap.add_argument("--thinking", action="store_true", help="adaptive thinking (the default is off)")
    ap.add_argument("--json-only", action="store_true", help="structured output: the JSON alone, no working in the text")
    ap.add_argument("--max-tokens", type=int, default=1024)
    ap.add_argument("--parallel", type=int, default=8)
    ap.add_argument("--show", action="store_true", help="print the first instance's prompt and answer per kind and encoding")
    ap.add_argument("--out", help="write every call's record as JSON lines here")
    ap.add_argument("--save-images", help="write the first instance's PNG per kind into this directory")
    args = ap.parse_args()
    key = api_key()
    if args.save_images:
        os.makedirs(args.save_images, exist_ok=True)
        for kind in args.kinds.split(","):
            s0 = KINDS[kind][0](random.Random(f"{args.seed}-{kind}"))
            open(os.path.join(args.save_images, f"{kind}-reach.png"), "wb").write(base64.b64decode(image_of(s0, True)))

    jobs = []
    for kind in args.kinds.split(","):
        gen, render, score = KINDS[kind]
        rng = random.Random(f"{args.seed}-{kind}")
        instances = [gen(rng) for _ in range(args.n)]
        for enc in args.encodings.split(","):
            for i, s in enumerate(instances):
                jobs.append((kind, enc, i, s, content_of(kind, s, enc, render)))

    def run(job):
        kind, enc, i, s, prompt = job
        r = stream(key, args.model, [{"type": "text", "text": SYSTEM}], prompt, args.effort, args.thinking, args.max_tokens, SCHEMAS[kind] if args.json_only else None)
        answer = parse(r["text"]) if not r.get("error") else {}
        sc = KINDS[kind][2](s, answer if isinstance(answer, dict) else {})
        shown = prompt if isinstance(prompt, str) else "[image] " + prompt[1]["text"]
        return {"kind": kind, "enc": enc, "i": i, "prompt": shown, "text": r["text"], "answer": answer, "score": sc,
                "wall_s": r["wall_s"], "first_token_s": r["first_token_s"], "usage": r["usage"], "error": r.get("error"),
                "stop_reason": r["stop_reason"], "cost": cost(args.model, r["usage"])}

    with concurrent.futures.ThreadPoolExecutor(max_workers=args.parallel) as pool:
        results = list(pool.map(run, jobs))
    if args.out:
        with open(args.out, "w") as f:
            for r in results:
                f.write(json.dumps(r) + "\n")

    print(f"model {args.model}, effort {args.effort}, thinking {'adaptive' if args.thinking else 'off'}{', json only' if args.json_only else ''}, {args.n} instances a kind, "
          f"{args.parallel} calls at once, total ${sum(r['cost'] for r in results):.3f}\n")
    print(f"{'kind':6s} {'encoding':8s} {'score (mean over instances, by measure)':50s} {'wall ms':>8s} {'in tok':>7s} {'out':>5s} {'errors':>6s}")
    for kind in args.kinds.split(","):
        for enc in args.encodings.split(","):
            rs = [r for r in results if r["kind"] == kind and r["enc"] == enc]
            measures = sorted({m for r in rs for m in r["score"]})
            sc = "  ".join(f"{m} {statistics.mean(r['score'].get(m, 0.0) for r in rs):.2f}" for m in measures)
            print(f"{kind:6s} {enc:8s} {sc:50s} {statistics.median(r['wall_s'] for r in rs) * 1000:8.0f} "
                  f"{statistics.median(r['usage'].get('input_tokens', 0) for r in rs):7.0f} "
                  f"{statistics.median(r['usage'].get('output_tokens', 0) for r in rs):5.0f} {sum(1 for r in rs if r['error']):6d}")
        print()
    if args.show:
        for kind in args.kinds.split(","):
            for enc in args.encodings.split(","):
                r = next(r for r in results if r["kind"] == kind and r["enc"] == enc and r["i"] == 0)
                print(f"==== {kind} / {enc}\n{r['prompt']}\n--> {r['text'].strip()}\n    score {r['score']}\n")


if __name__ == "__main__":
    main()
