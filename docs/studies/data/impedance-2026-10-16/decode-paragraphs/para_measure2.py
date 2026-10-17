"""The extractor question in the players' words, over the thirty paragraphs: F3 'take free metal spots, build
extractors'; F4 the question as written with 'on a free metal spot (the instructions may call this taking a spot)'."""
import json, sys, concurrent.futures, statistics
sys.path.insert(0, "run")
from jev_ab import api_key, ask
S = "docs/studies/data/impedance-2026-10-16/decode-paragraphs"
paras = json.load(open(S + "/para_labelled.json"))
WHAT = {"arm": "Construction Vehicle (armcv; constructor vehicle, 135 metal): tier-1 constructor vehicle, 90 build power; builds the base and the advanced vehicle plant"[:120],
        "cor": "Construction Vehicle (corcv; constructor vehicle, 145 metal): tier-1 constructor vehicle, 95 build power; builds the base and the advanced vehicle plant"[:120],
        "leg": "Legion Construction Vehicle (legcv; constructor vehicle, 125 metal): tier-1 constructor vehicle, 85 build power at speed 57; builds the base and the advanced vehicle plant"[:120]}
MEX = {"arm": "Metal Extractor (armmex)", "cor": "Metal Extractor (cormex)", "leg": "Metal Extractor (legmex)"}
ACTOR = "constructor_1412"
def f3(what, mex): return f"Read the player's `instructions` alone. Do they have {ACTOR} ({what}) take free metal spots, that is build a {mex} on each, now or under some condition they state?"
def f4(what, mex): return f"Read the player's `instructions` alone. Do they have {ACTOR} ({what}) build a {mex} on a free metal spot (the instructions may call this taking a spot), now or under some condition, or say anything else to it about that kind of building?"
key = api_key()
def run(p, model, rep):
    fac, what = p["faction"], WHAT[p["faction"]]
    qs = {"f3": {"type": "noul", "instructions": f3(what, MEX[fac])}, "f4": {"type": "noul", "instructions": f4(what, MEX[fac])}}
    a = ask(key, {"instructions": p["packet"]}, qs, model)["answers"]
    return {"index": p["index"], "model": model, "rep": rep, "f3": a["f3"].get("noul"), "f4": a["f4"].get("noul")}
jobs = [(p, m, rep) for p in paras for m in ("jev-1.14.0", "jev-1.13.0") for rep in range(2)]
with concurrent.futures.ThreadPoolExecutor(4) as ex:
    results = list(ex.map(lambda j: run(*j), jobs))
json.dump(results, open(S + "/para_results2.json", "w"), indent=1)
old = json.load(open(S + "/para_results.json")); label = {p["index"]: p["label"]["mex"] for p in paras}
for model in ("jev-1.14.0", "jev-1.13.0"):
    for name, src, k in (("F0 as written", old, "f0_mex"), ("F3 'take free metal spots, that is build a Metal Extractor on each'", results, "f3"), ("F4 as written + '(the instructions may call this taking a spot)'", results, "f4")):
        per = {}
        for r in src:
            if r["model"] == model: per.setdefault(r["index"], []).append(r[k])
        yes = [statistics.mean(v) for i, v in per.items() if label[i]]; no = [statistics.mean(v) for i, v in per.items() if not label[i]]
        print(f"{model} {name:68} yes {len(yes)}: under 0.3 {sum(1 for v in yes if v < 0.3):2}, under 0.2 {sum(1 for v in yes if v < 0.2):2}, median {statistics.median(yes):.2f} | no {len(no)}: under 0.3 {sum(1 for v in no if v < 0.3)}, median {statistics.median(no):.2f}")
