"""Round 4: the sentence alone, one word at a time. The decode's build question as written; Jev 1.14.0 and 1.13.0, three repeats."""
import json, sys, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask
S = "docs/studies/data/impedance-2026-10-16/decode-probe"
WHAT = "Construction Vehicle (armcv; constructor vehicle, 135 metal): tier-1 constructor vehicle, 90 build power; builds the base and the"[:120]
Q = f"Read the player's `instructions` alone. Do they have constructor_1412 ({WHAT}) build a Metal Extractor (armmex), now or under some condition, or say anything else to it about that kind of building?"
Q2 = Q.replace("build a Metal Extractor (armmex),", "build a Metal Extractor (armmex: taking a metal spot means building one there),")
spots = "spot_56, spot_47, spot_39"
cases = [
 ("s1 'their job is taking free metal spots in this order: ...'", f"Constructors: their job is taking free metal spots in this order: {spots}.", Q),
 ("s2 'their job is taking free extractor spots, in this order: ...'", f"Constructors: their job is taking free extractor spots, in this order: {spots}.", Q),
 ("s3 'they take free metal spots on our half, each the first one still free: ...'", f"Constructors: they take free metal spots on our half, each the first one still free: {spots}.", Q),
 ("s4 'their job is taking free metal spots: ...' (no 'in this order')", f"Constructors: their job is taking free metal spots: {spots}.", Q),
 ("s5 'their job is building extractors (armmex) on free metal spots, in this order: ...'", f"Constructors: their job is building extractors (armmex) on free metal spots, in this order: {spots}.", Q),
 ("s6 = s1, the question told what taking a spot means", f"Constructors: their job is taking free metal spots in this order: {spots}.", Q2),
 ("s7 'constructor_1412: its job is taking free metal spots in this order: ...' (named)", f"constructor_1412: its job is taking free metal spots in this order: {spots}.", Q),
 ("s8 = s1 + 'Every extractor away from home gets a light turret beside it.'", f"Constructors: their job is taking free metal spots in this order: {spots}. Every extractor away from home gets a light turret (armllt) beside it.", Q),
 ("s9 'their job is expansion: the free metal spots, in this order: ...'", f"Constructors: their job is expansion: the free metal spots, in this order: {spots}.", Q),
 ("s10 'they build a Metal Extractor on each free metal spot, in this order: ...'", f"Constructors: they build a Metal Extractor on each free metal spot, in this order: {spots}.", Q),
]
key = api_key()
def run(case, model, rep):
    label, packet, q = case
    a = ask(key, {"instructions": packet}, {"mex": {"type": "noul", "instructions": q}}, model)["answers"]
    return (label, model, rep, a["mex"].get("noul"))
jobs = [(c, m, rep) for c in cases for m in ("jev-1.14.0", "jev-1.13.0") for rep in range(3)]
with concurrent.futures.ThreadPoolExecutor(4) as ex:
    results = list(ex.map(lambda j: run(*j), jobs))
json.dump(results, open(S + "/decode_probe4_results.json", "w"), indent=1)
print(f"{'sentence alone':88} 1.14 (3 reps)      1.13 (3 reps)")
for c in cases:
    f = lambda m: "/".join(f"{r[3]:.2f}" for r in results if r[0] == c[0] and r[1] == m)
    print(f"{c[0][:88]:88} {f('jev-1.14.0'):18} {f('jev-1.13.0')}")
