"""The decode's build question under the constructor paragraphs the impedance study flagged, with the paragraph
edited to isolate what pulls the reading down; and a reworded question beside it. Both Jev versions, three repeats."""
import json, re, sys, os, statistics, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask
S = "docs/studies/data/impedance-2026-10-16/decode-probe"
packets = json.load(open(S + "/decode_probe_packets.json"))
WHAT = "Construction Vehicle (armcv; constructor vehicle, 135 metal): tier-1 constructor vehicle, 90 build power; builds the base and the"[:120]
def q_build(actor):  # the decode's question as written (decode.rs)
    return f"Read the player's `instructions` alone. Do they have {actor} ({WHAT}) build a Metal Extractor (armmex), now or under some condition, or say anything else to it about that kind of building?"
def q_build2(actor):  # a reworded one: the job, at any time
    return f"Read the player's `instructions` alone. Is building a Metal Extractor (armmex) on a free metal spot among the jobs they give {actor} ({WHAT}) at any time: now, later, after other steps, or only when some other case they describe does not apply?"
def q_place(actor, place):
    return f"Read the player's `instructions` alone. Do they name {place} in anything they say to or about {actor} ({WHAT}): a place it goes to, stands at, fights at, builds at, falls back to, or must keep away from?"
def variant(name, para_edit=None):
    p = packets[name]; para = p["paragraph"]; new = para_edit(para) if para_edit else para
    assert para_edit is None or new != para, name
    return p["packet"].replace(para, new), new
cases = []
# prio 10:37: the conditional preamble and the "first turrets, then spots" order
cases.append(("prio-1037 as is", *variant("prio-1037"), "constructor_1412"))
cases.append(("prio-1037 without 'when Pawns ... Otherwise:'", *variant("prio-1037", lambda s: s.replace("when Pawns of his stand within reach, they walk to spot_63 and wait. Otherwise: first", "first")), "constructor_1412"))
cases.append(("prio-1037 spots before turrets", *variant("prio-1037", lambda s: re.sub(r"Otherwise: first a light turret \(armllt\) beside every extractor of ours that has none \(([^)]*)\); then free metal spots in this order: ([^.]*)\.", r"Otherwise: free metal spots in this order: \2, with a light turret (armllt) beside each extractor right after it (\1 first).", s)), "constructor_1412"))
cases.append(("prio-1425 as is ('their job is extractors')", *variant("prio-1425"), "constructor_1412"))
cases.append(("apexv024 as is ('the ones at home ...; the others ...')", *variant("apexv024"), "constructor_1412"))
cases.append(("apexv024 'all take free metal spots'", *variant("apexv024", lambda s: s.replace("the ones at home help build the fusion (armfus) at home until it stands; the others take free metal spots", "they take free metal spots")), "constructor_1412"))
cases.append(("jev114 as is, for constructor_18284 (one of 'the others')", *variant("jev114"), "constructor_18284"))
cases.append(("jev114 as is, for constructor_26470 (named for the wrecks)", *variant("jev114"), "constructor_26470"))
cases.append(("rovemove as is (plain 'their job is taking free extractor spots')", *variant("rovemove"), "constructor_1412"))
key = api_key()
def run(case, model, rep):
    label, packet, para, actor = case
    place = (re.findall(r"spot_\d+", para) or ["spot_47"])[0]
    qs = {"build": {"type": "noul", "instructions": q_build(actor)}, "build2": {"type": "noul", "instructions": q_build2(actor)}, "place": {"type": "noul", "instructions": q_place(actor, place)}}
    r = ask(key, {"instructions": packet}, qs, model)
    a = r["answers"]
    return (label, model, rep, {k: a[k].get("noul") for k in qs}, place)
jobs = [(c, m, rep) for c in cases for m in ("jev-1.14.0", "jev-1.13.0") for rep in range(3)]
with concurrent.futures.ThreadPoolExecutor(4) as ex:
    results = list(ex.map(lambda j: run(*j), jobs))
json.dump(results, open(S + "/decode_probe_results.json", "w"), indent=1)
print(f"{'case':66} {'model':10} build(as written)  build(reworded)  place")
for c in cases:
    for m in ("jev-1.14.0", "jev-1.13.0"):
        rs = [r for r in results if r[0] == c[0] and r[1] == m]
        f = lambda k: "/".join(f"{r[3][k]:.2f}" for r in rs)
        print(f"{c[0][:66]:66} {m:10} {f('build'):18} {f('build2'):16} {f('place')} ({rs[0][4]})")
