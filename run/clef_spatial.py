#!/usr/bin/env python3
"""The spatial battery of run/haiku_spatial.py asked of Cloudflare's Clef models and of Jev as typed questions, with the
pictures through Clef's `images` field or Jev's `assets` (under the `TypeSafe-Danger-Zone-Images` header, 2026-10-16).
Same instances (seed), same code-computed truth, same scores.

    run/clef_spatial.py                                  # both Clef models, every encoding, 12 instances a kind
    run/clef_spatial.py --models clef-flash --encodings coords,facts,image_reach --n 6 --show
    run/clef_spatial.py --models jev,jev-preview --box   # Jev 1.13 and 1.14 with the pictures; Box questions on 1.14

With `--box`, the image encodings on `jev-preview` (Jev 1.14, the first with Box) also ask a Box: the uncovered
stretch of the turret line (`gap`, scored as the free position: `free`) and the main body of the group (`rally`,
its centre's nearest candidate scored as `rally_box`).

How each ask becomes questions: `gap` a choice over the candidate spots (the free position is not asked: a decision
model gives no numbers); `cover` a noul per site ("one of the best two?"), the top two taken; `rally` a noul per unit
("strung out?") and a choice over the rally points; `route` a choice over the routes. The state is the encoding's
JSON (or the ASCII map text); the image encodings send the PNG (`image` alone, `image_coords` with the numbers,
`image_reach` with the reach circles drawn and the numbers).
"""
import argparse, concurrent.futures, json, random, statistics, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import haiku_spatial as H
from clef_replay import credentials, clef_ask, PRICES
import urllib.request, urllib.error, time, uuid

JEV_URL = "https://api.typesafe.ai/v1/systemone"
JEV_PRICE = 0.042


def jev_key():
    key = os.environ.get("TYPESAFE_API_KEY", "")
    path = os.path.expanduser("~/.config/within-reason/jev.env")
    if not key and os.path.exists(path):
        for line in open(path):
            if line.startswith("TYPESAFE_API_KEY="):
                key = line.split("=", 1)[1].strip().strip("\"'")
    if not key:
        sys.exit("no TYPESAFE_API_KEY")
    return key


JEV_MODELS = {"jev": "jev-latest", "jev-preview": "jev-preview"}


def jev_ask(key, model, state, questions, images=None):
    """Jev on the same state and questions. The images (data URLs) go as `assets` under the images header, and the
    state becomes an object with the picture under `map` and the text under `description`, so a question can name it."""
    body = {"model": JEV_MODELS.get(model, model), "state": state, "questions": questions}
    headers = {"Authorization": f"Bearer {key}", "Content-Type": "application/json", "User-Agent": "within-reason-spatial"}
    if images:
        refs = [str(uuid.uuid4()) for _ in images]
        body["assets"] = {ref: {"type": "image", "format": "png", "data_base64": img.split(",", 1)[1]} for ref, img in zip(refs, images)}
        body["state"] = {"map": refs[0], "description": state} if len(refs) == 1 else {"maps": refs, "description": state}
        headers["TypeSafe-Danger-Zone-Images"] = "true"
    req = urllib.request.Request(JEV_URL, data=json.dumps(body).encode(), method="POST", headers=headers)
    t = time.monotonic()
    try:
        with urllib.request.urlopen(req, timeout=120) as r:
            out = json.load(r)
    except urllib.error.HTTPError as e:
        raise RuntimeError(f"HTTP {e.code}: {e.read().decode(errors='replace')[:300]}")
    return out["answers"], out.get("usage", {}), time.monotonic() - t, 1, 0


BOXES = {"gap": "the uncovered stretch of our turret line in `map`: the part of the line, between two neighbouring red turrets, that no turret's reach covers and the raiders will cross",
         "rally": "the main body of our group in `map`: the dense cluster of green dots where the group should gather to fight as one body"}

ENCODINGS = ["coords", "cells", "ascii", "facts", "image", "image_coords", "image_reach"]


def state_and_images(kind, s, enc):
    """The state text (the rendered prompt without its question) and the images list."""
    text = H.KINDS[kind][1](s, "coords" if enc.startswith("image") else enc)
    state = text[:text.rindex("\n\n")]
    if not enc.startswith("image"):
        return state, None
    if enc == "image":
        state = f"turret_reach: {H.R}" if kind in ("gap", "cover", "route") else "see the image"
    legend = (f"The image is the map, {H.MAP} by {H.MAP} elmos, {H.MAP // H.IMAGE_PX} elmos a pixel, grid lines every 1024 elmos; {H.LEGEND[kind]}"
              + ("; each turret's reach is drawn as a circle" if enc == "image_reach" else "") + ".")
    return legend + "\n\n" + state, ["data:image/png;base64," + H.image_of(s, enc == "image_reach")]


def questions_of(kind, s, box=False):
    q = typed_questions_of(kind, s)
    if box and kind in BOXES:
        q[f"{kind}_box"] = {"type": "box", "instructions": BOXES[kind]}
    return q


def box_centre(answer):
    b = answer["box"]
    return ((b["left"] + b["right"]) / 2 * H.MAP, (b["top"] + b["bottom"]) / 2 * H.MAP)


def typed_questions_of(kind, s):
    if kind == "gap":
        return {"spot": {"type": "choice", "instructions": f"Raiders will cross our turret line from the north where no turret's reach covers it. Which candidate spot, given a new turret of reach {H.R}, closes the uncovered stretch?",
                         "criteria": {k: f"a new turret at {k}" for k in s["cands"]}}}
    if kind == "cover":
        return {k: {"type": "noul", "instructions": f"We can afford two turrets of reach {H.R}. Is site {k} one of the two sites that together cover the most of our extractors?"} for k in s["cands"]}
    if kind == "rally":
        q = {f"strag.{u}": {"type": "noul", "instructions": f"An enemy party approaches and our group is not one body. Is unit {u} strung out, away from the main body of the group?"} for u in s["units"]}
        q["rally"] = {"type": "choice", "instructions": "At which candidate point should the group gather so that it fights as one body?", "criteria": {k: f"gather at {k}" for k in s["cands"]}}
        return q
    if kind == "route":
        return {"route": {"type": "choice", "instructions": "The group walks from its position to the target by one of the routes, each a straight walk to its waypoint and then to the target. Which route stays out of every enemy turret's reach?",
                          "criteria": {k: f"route {k}" for k in s["ways"]}}}


def answer_of(kind, s, answers):
    if kind == "gap":
        a = {"spot": answers.get("spot", {}).get("choice")}
        if "gap_box" in answers:
            a["x"], a["z"] = box_centre(answers["gap_box"])
        return a
    if kind == "cover":
        ranked = sorted(s["cands"], key=lambda k: -answers.get(k, {}).get("noul", 0))
        return {"sites": ranked[:2]}
    if kind == "rally":
        a = {"stragglers": [u for u in s["units"] if answers.get(f"strag.{u}", {}).get("noul", 0) >= 0.5], "rally": answers.get("rally", {}).get("choice")}
        if "rally_box" in answers:
            centre = box_centre(answers["rally_box"])
            a["rally_box"] = min(s["cands"], key=lambda k: H.dist(centre, s["cands"][k]))
        return a
    if kind == "route":
        return {"route": answers.get("route", {}).get("choice")}


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--kinds", default=",".join(H.KINDS))
    ap.add_argument("--encodings", default=",".join(ENCODINGS))
    ap.add_argument("--models", default="clef-flash,clef", help="clef-flash, clef, jev (jev-latest), jev-preview, or a versioned jev-1.x.y")
    ap.add_argument("--box", action="store_true", help="Box questions beside the typed ones on jev-preview's image encodings")
    ap.add_argument("--n", type=int, default=12)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--parallel", type=int, default=6)
    ap.add_argument("--show", action="store_true")
    ap.add_argument("--out")
    args = ap.parse_args()
    models = args.models.split(",")
    creds = credentials() if any(not m.startswith("jev") for m in models) else None
    jkey = jev_key() if any(m.startswith("jev") for m in models) else None

    jobs = []
    for kind in args.kinds.split(","):
        gen = H.KINDS[kind][0]
        rng = random.Random(f"{args.seed}-{kind}")
        instances = [gen(rng) for _ in range(args.n)]
        for enc in args.encodings.split(","):
            for i, s in enumerate(instances):
                state, images = state_and_images(kind, s, enc)
                for model in models:
                    box = args.box and images is not None and model in ("jev-preview", "jev-1.14.0")
                    jobs.append((model, kind, enc, i, s, state, images, box))

    def run(job):
        model, kind, enc, i, s, state, images, box = job
        try:
            if model.startswith("jev"):
                answers, usage, seconds, batches, retried = jev_ask(jkey, model, state, questions_of(kind, s, box), images)
            else:
                answers, usage, seconds, batches, retried = clef_ask(creds, model, state, questions_of(kind, s), images)
            a = answer_of(kind, s, answers)
            score = H.KINDS[kind][2](s, a)
            if "rally_box" in a:
                score["rally_box"] = 1.0 if a["rally_box"] == s["right"] else 0.0
            return {"model": model, "kind": kind, "enc": enc, "i": i, "answer": a, "score": score, "seconds": seconds, "usage": usage, "raw": answers}
        except (RuntimeError, urllib.error.HTTPError, urllib.error.URLError) as e:
            return {"model": model, "kind": kind, "enc": enc, "i": i, "error": str(e), "score": {}, "seconds": 0, "usage": {}}

    with concurrent.futures.ThreadPoolExecutor(max_workers=args.parallel) as pool:
        results = list(pool.map(run, jobs))
    if args.out:
        with open(args.out, "w") as f:
            for r in results:
                f.write(json.dumps(r) + "\n")
    for model in models:
        rs_m = [r for r in results if r["model"] == model]
        tokens = sum(r["usage"].get("input_tokens", 0) for r in rs_m)
        print(f"==== {model}: {len(rs_m)} calls, {sum(1 for r in rs_m if 'error' in r)} failed, {tokens / 1e6:.2f}M tokens (${tokens / 1e6 * PRICES.get(model, JEV_PRICE):.2f})")
        print(f"{'kind':6s} {'encoding':13s} {'score (mean over instances)':36s} {'wall ms':>8s} {'in tok':>7s}")
        for kind in args.kinds.split(","):
            for enc in args.encodings.split(","):
                rs = [r for r in rs_m if r["kind"] == kind and r["enc"] == enc and "error" not in r]
                if not rs:
                    continue
                measures = sorted({m for r in rs for m in r["score"]})
                sc = "  ".join(f"{m} {statistics.mean(r['score'].get(m, 0.0) for r in rs):.2f}" for m in measures)
                print(f"{kind:6s} {enc:13s} {sc:36s} {statistics.median(r['seconds'] for r in rs) * 1000:8.0f} {statistics.median(r['usage'].get('input_tokens', 0) for r in rs):7.0f}")
            print()
        for r in [r for r in rs_m if "error" in r][:3]:
            print("  error:", r["kind"], r["enc"], r["error"][:200])
    if args.show:
        for r in results:
            if r["i"] == 0 and "raw" in r:
                print(f"== {r['model']} {r['kind']}/{r['enc']}: {json.dumps(r['raw'])[:400]}\n   -> {r['answer']} {r['score']}")


if __name__ == "__main__":
    main()
