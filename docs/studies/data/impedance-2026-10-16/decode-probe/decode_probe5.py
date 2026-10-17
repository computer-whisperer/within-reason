"""Round 5: the actor's words in the question, and the packet around the sentence. Jev 1.14.0, three repeats."""
import json, sys, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask
S = "docs/studies/data/impedance-2026-10-16/decode-probe"
packets = json.load(open(S + "/decode_probe_packets.json"))
GLOSS = "Construction Vehicle (armcv; constructor vehicle, 135 metal): tier-1 constructor vehicle, 90 build power; builds the base and the"[:120]
WHATS = {"gloss120 (as in the game)": GLOSS, "short 'Construction Vehicle (armcv)'": "Construction Vehicle (armcv)", "descriptive": "a Construction Vehicle (armcv), a constructor that builds extractors, generators, turrets and factories"}
def q(what): return f"Read the player's `instructions` alone. Do they have constructor_1412 ({what}) build a Metal Extractor (armmex), now or under some condition, or say anything else to it about that kind of building?"
s1 = "Constructors: their job is taking free metal spots in this order: spot_56, spot_47, spot_39."
rm = packets["rovemove"]; prio = packets["prio-1037"]
cases = []
for wl, w in WHATS.items():
    cases.append((f"s1 alone | what = {wl}", s1, q(w)))
    cases.append((f"rove-move-1's own paragraph alone | what = {wl}", rm["paragraph"], q(w)))
cases.append(("s1 inside rove-move-1's full packet | gloss120", rm["packet"].replace(rm["paragraph"], s1), q(GLOSS)))
cases.append(("rove-move-1's own paragraph inside prio-1037's packet | gloss120", prio["packet"].replace(prio["paragraph"], rm["paragraph"]), q(GLOSS)))
cases.append(("rove-move-1's full packet as is | gloss120", rm["packet"], q(GLOSS)))
cases.append(("prio-1037 as is | what = descriptive", prio["packet"], q(WHATS["descriptive"])))
key = api_key()
def run(case, rep):
    label, packet, qq = case
    a = ask(key, {"instructions": packet}, {"mex": {"type": "noul", "instructions": qq}}, "jev-1.14.0")["answers"]
    return (label, rep, a["mex"].get("noul"))
jobs = [(c, rep) for c in cases for rep in range(3)]
with concurrent.futures.ThreadPoolExecutor(4) as ex:
    results = list(ex.map(lambda j: run(*j), jobs))
json.dump(results, open(S + "/decode_probe5_results.json", "w"), indent=1)
for c in cases:
    rs = [r[2] for r in results if r[0] == c[0]]
    print(f"{sum(rs)/3:.2f}  {'/'.join(f'{x:.2f}' for x in rs):16} {c[0]}")
