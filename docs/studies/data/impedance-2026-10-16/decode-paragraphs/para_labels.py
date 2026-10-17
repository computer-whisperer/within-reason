"""The thirty paragraphs and the main session's labels (2026-10-16): for a constructor the packet does not name,
does the paragraph give it the job, now or under some condition it states? yes/no per job; a job only on a list,
only for named constructors, or by another unit type (a Beamer, not the light turret) is no. Notes say why."""
import json
S = "docs/studies/data/impedance-2026-10-16/decode-paragraphs"
sample = json.load(open(S + "/para_sample.json"))
probe = json.load(open(S + "/decode_probe_packets.json"))["prio-1037"]
sample.append({"id": "prio1037x", "game": "prio-player-1", "seat": "strategist-0", "clock": "10:37", "paragraph": probe["paragraph"], "packet": probe["packet"]})
# index: (faction, mex, llt, solar, help, wreck, note)
L = {
 0: ("arm", 1, 1, 0, 0, 0, "wind, not a solar"),
 1: ("arm", 1, 0, 1, 1, 0, "a Beamer beside extractors, not the light turret; helps the fusion first"),
 2: ("arm", 1, 1, 1, 1, 0, "'with nothing to do help the plants'"),
 3: ("arm", 1, 1, 0, 1, 0, "the ones at home help the fusion; the others take spots; advanced solar only"),
 4: ("arm", 1, 1, 0, 0, 1, "after the list: spots with a Sentry each, the wreck field at G4/H4"),
 5: ("arm", 1, 1, 0, 1, 1, "wind only; wrecks or the plant without a list; spots when a list is done"),
 6: ("arm", 0, 0, 0, 0, 0, "only a walk away from raiders, otherwise 'works its list'"),
 7: ("cor", 1, 1, 1, 0, 1, "'never helps the plant'; 'no solar while energy is above half' read as a solar under half"),
 8: ("arm", 1, 1, 1, 0, 0, ""),
 9: ("arm", 1, 1, 1, 0, 0, ""),
 10: ("arm", 0, 0, 1, 0, 1, "wrecks and turret repair; no spots"),
 11: ("cor", 0, 0, 0, 0, 1, "extractors and Guards 'on their lists' only; a corrl (anti-air), not the light turret"),
 12: ("cor", 1, 1, 1, 1, 0, "helping the plant only while metal is stored"),
 13: ("arm", 1, 0, 0, 1, 0, "rebuilds a destroyed extractor at named spots; helps the aircraft plant"),
 14: ("arm", 1, 1, 0, 0, 0, "advanced solar only"),
 15: ("leg", 1, 1, 1, 0, 1, ""),
 16: ("arm", 1, 1, 0, 0, 1, "advanced solar only"),
 17: ("arm", 1, 1, 1, 0, 0, ""),
 18: ("arm", 1, 1, 1, 0, 0, "'when Pawns ... wait. Otherwise their job is taking free metal spots'"),
 19: ("arm", 1, 1, 1, 0, 0, "'free extractor spots'"),
 20: ("arm", 1, 1, 1, 0, 0, ""),
 21: ("arm", 1, 1, 0, 0, 0, "three named constructors take wrecks; 'the others take free spots'"),
 22: ("arm", 1, 1, 0, 0, 0, "the heading lists three constructors; read as all of them; Nettles then Sentries, then extractors"),
 23: ("arm", 1, 1, 0, 1, 0, "as 3, plus the advanced extractors"),
 24: ("arm", 1, 1, 1, 0, 0, ""),
 25: ("cor", 0, 0, 0, 0, 0, "a corhllt (heavy turret) at home only"),
 26: ("cor", 1, 1, 1, 0, 1, ""),
 27: ("arm", 0, 0, 0, 0, 0, "two named constructors only"),
 28: ("arm", 1, 1, 0, 0, 0, "Sentries, and extractors rebuilt when no enemy is in sight"),
 29: ("arm", 1, 1, 1, 0, 0, "'first a light turret beside every extractor that has none; then free metal spots'"),
}
for i, c in enumerate(sample):
    f, mex, llt, solar, help_, wreck, note = L[i]
    c.update({"index": i, "faction": f, "label": {"mex": mex, "llt": llt, "solar": solar, "help": help_, "wreck": wreck}, "note": note})
json.dump(sample, open(S + "/para_labelled.json", "w"), indent=1)
print(len(sample), "labelled; yes counts:", {k: sum(c["label"][k] for c in sample) for k in ("mex", "llt", "solar", "help", "wreck")})
