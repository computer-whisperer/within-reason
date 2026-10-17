"""Round 3: the rest of the packet. The plain constructor sentence inside each packet, with the other paragraphs
removed one at a time, against the sentence alone. Jev 1.14.0, three repeats."""
import json, re, sys, concurrent.futures
sys.path.insert(0, "run")
from jev_ab import api_key, ask
S = "docs/studies/data/impedance-2026-10-16/decode-probe"
packets = json.load(open(S + "/decode_probe_packets.json"))
WHAT = "Construction Vehicle (armcv; constructor vehicle, 135 metal): tier-1 constructor vehicle, 90 build power; builds the base and the"[:120]
def q(actor): return f"Read the player's `instructions` alone. Do they have {actor} ({WHAT}) build a Metal Extractor (armmex), now or under some condition, or say anything else to it about that kind of building?"
def paras(text): return [p for p in text.split("\n\n") if p.strip()]
cases = []
for name, actor, plain in [("prio-1037", "constructor_1412", "Constructors: their job is taking free metal spots in this order: spot_56, spot_47, spot_39, spot_28, spot_66, spot_78, spot_30, spot_19, spot_22, spot_33, spot_44, spot_58, spot_49."),
                           ("jev114", "constructor_18284", "Constructors: their job is taking free metal spots, in this order as they are free: spot_73, spot_45, spot_39, spot_47, spot_49, spot_28, spot_30.")]:
    p = packets[name]; full = p["packet"].replace(p["paragraph"], plain)
    ps = paras(full); ci = next(i for i, x in enumerate(ps) if x.startswith("Constructors:"))
    cases.append((f"{name}: plain sentence in the full packet", full, actor))
    cases.append((f"{name}: plain sentence alone", plain, actor))
    for i, x in enumerate(ps):
        if i == ci: continue
        rest = "\n\n".join(y for j, y in enumerate(ps) if j != i)
        cases.append((f"{name}: full minus paragraph {i} ({x[:40]!r})", rest, actor))
    cases.append((f"{name}: plain sentence + Commander paragraph only", "\n\n".join([plain, next(x for x in ps if x.startswith("Commander:"))]), actor))
key = api_key()
def run(case, rep):
    label, packet, actor = case
    a = ask(key, {"instructions": packet}, {"mex": {"type": "noul", "instructions": q(actor)}}, "jev-1.14.0")["answers"]
    return (label, rep, a["mex"].get("noul"))
jobs = [(c, rep) for c in cases for rep in range(3)]
with concurrent.futures.ThreadPoolExecutor(4) as ex:
    results = list(ex.map(lambda j: run(*j), jobs))
json.dump(results, open(S + "/decode_probe3_results.json", "w"), indent=1)
for c in cases:
    rs = [r[2] for r in results if r[0] == c[0]]
    print(f"{sum(rs)/len(rs):.2f}  {'/'.join(f'{x:.2f}' for x in rs):16} {c[0][:110]}")
