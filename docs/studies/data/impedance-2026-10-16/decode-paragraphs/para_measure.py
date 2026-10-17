"""The three forms over the thirty labelled paragraphs, each inside its full packet as the game sends it, on Jev
1.14.0 and 1.13.0, two repeats: F0 the decode's yes/no as written; F1 the yes/no reworded ("among the jobs ... at
any time"); F2 one Choice per actor over its jobs; F2b a three-way Choice per job (yes / other jobs / not spoken to)."""
import json, sys, concurrent.futures, collections
sys.path.insert(0, "run")
from jev_ab import api_key, ask
S = "docs/studies/data/impedance-2026-10-16/decode-paragraphs"
paras = json.load(open(S + "/para_labelled.json"))
WHAT = {"arm": "Construction Vehicle (armcv; constructor vehicle, 135 metal): tier-1 constructor vehicle, 90 build power; builds the base and the advanced vehicle plant"[:120],
        "cor": "Construction Vehicle (corcv; constructor vehicle, 145 metal): tier-1 constructor vehicle, 95 build power; builds the base and the advanced vehicle plant"[:120],
        "leg": "Legion Construction Vehicle (legcv; constructor vehicle, 125 metal): tier-1 constructor vehicle, 85 build power at speed 57; builds the base and the advanced vehicle plant"[:120]}
WORDS = {"mex": {"arm": "Metal Extractor (armmex)", "cor": "Metal Extractor (cormex)", "leg": "Metal Extractor (legmex)"},
         "llt": {"arm": "Sentry (armllt)", "cor": "Guard (corllt)", "leg": "Pharos (leglht)"},
         "solar": {"arm": "Solar Collector (armsolar)", "cor": "Solar Collector (corsolar)", "leg": "Solar Collector (legsolar)"}}
ACTOR = "constructor_1412"
def f0(what, words): return f"Read the player's `instructions` alone. Do they have {ACTOR} ({what}) build a {words}, now or under some condition, or say anything else to it about that kind of building?"
def f1(what, words): return f"Read the player's `instructions` alone. Is building a {words} among the jobs they give {ACTOR} ({what}) at any time: now, later, after other steps, or only when some other case they describe does not apply?"
def f2(what, fac):
    return {"type": "choice", "instructions": f"Read the player's `instructions` alone. Which of these jobs do they give {ACTOR} ({what}), now or under some condition they state? Weigh every job the instructions give it, not only the first.",
            "criteria": {"mex": f"building a {WORDS['mex'][fac]} on a free metal spot", "llt": f"building a {WORDS['llt'][fac]} beside an extractor", "solar": f"building a {WORDS['solar'][fac]} or another generator", "help": "helping a factory or a frame being built", "wreck": "taking apart wreck fields for their metal", "none": "none of these: the instructions give this constructor no such job"}}
def f2b(what, words):
    return {"type": "choice", "instructions": f"Read the player's `instructions` alone. About {ACTOR} ({what}) and building a {words}, which is true?",
            "criteria": {"yes": f"the instructions give it a {words} to build, now or under some condition they state", "no": "the instructions give it other jobs and not that", "silent": "the instructions do not speak to this constructor about building at all"}}
key = api_key()
def run(p, model, rep):
    fac, what = p["faction"], WHAT[p["faction"]]
    qs = {}
    for t in ("mex", "llt", "solar"):
        qs[f"f0_{t}"] = {"type": "noul", "instructions": f0(what, WORDS[t][fac])}
        qs[f"f1_{t}"] = {"type": "noul", "instructions": f1(what, WORDS[t][fac])}
        qs[f"f2b_{t}"] = f2b(what, WORDS[t][fac])
    qs["f2"] = f2(what, fac)
    a = ask(key, {"instructions": p["packet"]}, qs, model)["answers"]
    out = {"index": p["index"], "model": model, "rep": rep}
    for t in ("mex", "llt", "solar"):
        out[f"f0_{t}"] = a[f"f0_{t}"].get("noul"); out[f"f1_{t}"] = a[f"f1_{t}"].get("noul"); out[f"f2b_{t}"] = (a[f"f2b_{t}"].get("probabilities") or {}).get("yes")
    out["f2"] = a["f2"].get("probabilities") or {}
    return out
jobs = [(p, m, rep) for p in paras for m in ("jev-1.14.0", "jev-1.13.0") for rep in range(2)]
with concurrent.futures.ThreadPoolExecutor(4) as ex:
    results = list(ex.map(lambda j: run(*j), jobs))
json.dump(results, open(S + "/para_results.json", "w"), indent=1)
label = {p["index"]: p["label"] for p in paras}
def score(model, getter, bars, name):
    rows = [(label[r["index"]][t], getter(r, t)) for r in results if r["model"] == model for t in ("mex", "llt", "solar") if getter(r, t) is not None]
    yes = [v for l, v in rows if l]; no = [v for l, v in rows if not l]
    import statistics
    # rank quality: share of (yes, no) pairs with the yes reading higher
    pairs = sum(1 for y in yes for n in no if y > n) / max(1, len(yes) * len(no))
    line = f"{name:34} {model:10} n={len(rows):3} yes-median {statistics.median(yes):.2f} no-median {statistics.median(no):.2f} rank {pairs:.2f} |"
    for b in bars:
        false_off = sum(1 for v in yes if v < b) / len(yes); right_off = sum(1 for v in no if v < b) / len(no)
        line += f" bar {b}: wrongly off {false_off:.0%}, rightly off {right_off:.0%} |"
    print(line)
print("Over 30 paragraphs x 3 jobs (mex, llt, solar) x 2 repeats; 'wrongly off' = labelled yes but read under the bar (a job the fuse would remove); 'rightly off' = labelled no and under the bar (the pruning)")
for model in ("jev-1.14.0", "jev-1.13.0"):
    score(model, lambda r, t: r[f"f0_{t}"], (0.2, 0.3), "F0 yes/no as written")
    score(model, lambda r, t: r[f"f1_{t}"], (0.2, 0.3), "F1 yes/no reworded")
    score(model, lambda r, t: min(r[f"f0_{t}"], r[f"f1_{t}"]) if r[f"f0_{t}"] is not None and r[f"f1_{t}"] is not None else None, (0.3,), "both under (F0 and F1)")
    score(model, lambda r, t: r[f"f2b_{t}"], (0.2, 0.3), "F2b three-way per job, p(yes)")
    score(model, lambda r, t: r["f2"].get(t), (0.05, 0.1, 0.15), "F2 one Choice over jobs, p(job)")
    score(model, lambda r, t: (r["f2"].get(t, 0) - r["f2"].get("none", 0)) + 0.5, (0.5,), "F2 job above 'none' (bar 0.5 = equal)")
# help and wreck from the Choice
print("\nF2's help and wreck (labelled yes vs no), 1.14, p(job):")
for t in ("help", "wreck"):
    yes = [r["f2"].get(t, 0) for r in results if r["model"] == "jev-1.14.0" and label[r["index"]][t]]; no = [r["f2"].get(t, 0) for r in results if r["model"] == "jev-1.14.0" and not label[r["index"]][t]]
    import statistics
    print(f"  {t}: yes median {statistics.median(yes):.2f} (n={len(yes)}), no median {statistics.median(no):.2f} (n={len(no)})")
print("\nPer paragraph, 1.14, mean over repeats: index  label(mex,llt,solar)  F0  F1  F2b  F2")
for p in paras:
    rs = [r for r in results if r["model"] == "jev-1.14.0" and r["index"] == p["index"]]
    m = lambda k: sum(r[k] for r in rs) / len(rs)
    f2 = lambda t: sum(r["f2"].get(t, 0) for r in rs) / len(rs)
    print(f"  [{p['index']:2}] {p['game'][:14]:14} {p['label']['mex']}{p['label']['llt']}{p['label']['solar']}  F0 {m('f0_mex'):.2f}/{m('f0_llt'):.2f}/{m('f0_solar'):.2f}  F1 {m('f1_mex'):.2f}/{m('f1_llt'):.2f}/{m('f1_solar'):.2f}  F2b {m('f2b_mex'):.2f}/{m('f2b_llt'):.2f}/{m('f2b_solar'):.2f}  F2 {f2('mex'):.2f}/{f2('llt'):.2f}/{f2('solar'):.2f} none {f2('none'):.2f}")
