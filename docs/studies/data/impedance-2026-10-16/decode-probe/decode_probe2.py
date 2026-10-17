"""Round 2: which clause pulls the decode's reading down. The build question as written for extractors and for the
light turret, under clause-level edits of the three flagged paragraphs."""
import json, re, sys, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask
S = "docs/studies/data/impedance-2026-10-16/decode-probe"
packets = json.load(open(S + "/decode_probe_packets.json"))
WHAT = "Construction Vehicle (armcv; constructor vehicle, 135 metal): tier-1 constructor vehicle, 90 build power; builds the base and the"[:120]
def q(actor, words): return f"Read the player's `instructions` alone. Do they have {actor} ({WHAT}) build a {words}, now or under some condition, or say anything else to it about that kind of building?"
def variant(name, edit=None):
    p = packets[name]; para = p["paragraph"]; new = edit(para) if edit else para
    assert edit is None or new != para, name
    return p["packet"].replace(para, new)
P = packets["prio-1037"]["paragraph"]
turret_sentence = re.search(r"first a light turret \(armllt\) beside every extractor of ours that has none \([^)]*\); then ", P).group(0)
cases = [
 ("prio-1037 as is", variant("prio-1037"), "constructor_1412"),
 ("prio-1037: turret sentence without its spot list", variant("prio-1037", lambda s: re.sub(r"beside every extractor of ours that has none \([^)]*\);", "beside every extractor of ours that has none;", s)), "constructor_1412"),
 ("prio-1037: 'Their job: free metal spots in this order ... A light turret beside each extractor right after it'", variant("prio-1037", lambda s: s.replace(turret_sentence, "their job is taking ").replace("free metal spots in this order:", "free metal spots in this order (a light turret (armllt) beside each extractor right after it):", 1)), "constructor_1412"),
 ("prio-1037: only 'Constructors: their job is taking free metal spots in this order: <list>.'", variant("prio-1037", lambda s: "Constructors: their job is taking free metal spots in this order: " + ", ".join(re.findall(r"spot_\d+", s.split("then free metal spots in this order:")[1].split(".")[0])) + "."), "constructor_1412"),
 ("apexv024 as is", variant("apexv024"), "constructor_1412"),
 ("apexv024 without 'they work their lists. When a list is done,'", variant("apexv024", lambda s: s.replace("they work their lists. When a list is done, the ones at home help build the fusion (armfus) at home until it stands; the others take", "they take")), "constructor_1412"),
 ("jev114 as is (constructor_18284)", variant("jev114"), "constructor_18284"),
 ("jev114: 'The others' -> 'Every other constructor, constructor_18284 among them,'", variant("jev114", lambda s: s.replace("The others take free metal spots", "Every other constructor, constructor_18284 among them, takes free metal spots")), "constructor_18284"),
 ("jev114: the wreck sentence removed", variant("jev114", lambda s: re.sub(r"constructor_26470 and constructor_18073 take apart the wreck fields[^.]*\. The others take", "Their job is taking", s)), "constructor_18284"),
]
key = api_key()
def run(case, model, rep):
    label, packet, actor = case
    qs = {"mex": {"type": "noul", "instructions": q(actor, "Metal Extractor (armmex)")}, "llt": {"type": "noul", "instructions": q(actor, "Sentry (armllt)")}}
    a = ask(key, {"instructions": packet}, qs, model)["answers"]
    return (label, model, rep, {k: a[k].get("noul") for k in qs})
jobs = [(c, m, rep) for c in cases for m in ("jev-1.14.0", "jev-1.13.0") for rep in range(3)]
with concurrent.futures.ThreadPoolExecutor(4) as ex:
    results = list(ex.map(lambda j: run(*j), jobs))
json.dump(results, open(S + "/decode_probe2_results.json", "w"), indent=1)
print(f"{'case':100} {'model':10} extractor          light turret")
for c in cases:
    for m in ("jev-1.14.0", "jev-1.13.0"):
        rs = [r for r in results if r[0] == c[0] and r[1] == m]
        f = lambda k: "/".join(f"{r[3][k]:.2f}" for r in rs)
        print(f"{c[0][:100]:100} {m:10} {f('mex'):18} {f('llt')}")
