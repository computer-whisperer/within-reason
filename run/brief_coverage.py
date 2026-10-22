#!/usr/bin/env python3
"""Tally the rewrite's coverage of the old brief: every passage of the 2026-10-21 set that the reading pass marked
fact, meta, rederive or episode, against the new documents' `covers:` blocks and the recorded dispositions.

    run/brief_coverage.py [--manifest PATH ...] [--study DIR] [--archive DIR] [--open] [--by subject|file] [--json] [--strict]

The study (docs/studies/2026-10-21-brief-annotation/part-*.jsonl) gives the rows; a row's key is its file and line
range as the readers gave them, `docs/briefs/player.md:92-107`, with `/N` for the Nth part of a passage the readers
split. The new documents (the manifests given, else docs/briefs/rework.manifest when it exists) carry
`<!--meta ... -->` blocks (docs/design/2026-10-21-brief-rework.md §3) whose `covers:` lists such keys; a key without
`/N` covers every part. The archive's `dispositions.jsonl` (docs/briefs/archive/2026-10-21/) records the passages
given no home: `{"key": ..., "disposition": "discard" | "covered", "reason": ..., "id": ..., "by": ..., "date": ...}`.
Tool rows are left out: the role prompts stay in place. The readers' own `discard` fate is a suggestion and counts as
open until a disposition records it.

Prints the tally by fate (covered, discarded, open) and the open rows grouped by file, or by the merge pass's backlog
subjects (`--by backlog`: the `###` sections of the derivation backlog and the meta catalogue, §a and §b of the study's
backlog.md, whose prose names passages as `player.md:234-246 part 3`; rows named in neither are "(not in the
backlog)"), or by the readers' subjects (`--by subject`);
`--open` lists every open row with its key, fate and subject; `--json` prints the whole result; `--strict` exits 1
while any row is open, for the day the manifests switch.
"""
import glob
import json
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(REPO)
STUDY = "docs/studies/2026-10-21-brief-annotation"
ARCHIVE = "docs/briefs/archive/2026-10-21"
BLOCK = re.compile(r"<!--meta\n(.*?)-->", re.S)
COUNTED = ("fact", "meta", "rederive", "episode", "discard")


def rows(study):
    out = []
    for path in sorted(glob.glob(os.path.join(study, "part-*.jsonl"))):
        for line in open(path):
            if line.strip():
                r = json.loads(line)
                span = r.get("lines") or [0]
                lo, hi = (span[0], span[1]) if len(span) >= 2 else (span[0], span[0])
                r["key"] = f"{r['file']}:{lo}-{hi}"
                r["part_key"] = f"{r['key']}/{r['part']}" if r.get("part") else r["key"]
                out.append(r)
    return out


def covers(manifests):
    """Every key a new document's block covers, with the block's id."""
    found = {}
    for manifest in manifests:
        if not os.path.exists(manifest):
            continue
        for document in [l.strip() for l in open(manifest) if l.strip() and not l.strip().startswith("#")]:
            if not os.path.exists(document):
                continue
            text = open(document).read()
            for m in BLOCK.finditer(text):
                fields = dict(l.split(":", 1) for l in m.group(1).split("\n") if ":" in l)
                fields = {k.strip(): v.strip() for k, v in fields.items()}
                ident = fields.get("id", f"{document}:{text[:m.start()].count(chr(10)) + 1}")
                listed = fields.get("covers", "")
                for key in re.findall(r"[\w./-]+:\d+-\d+(?:/\d+)?", listed):
                    found.setdefault(key, []).append(ident)
    return found


def backlog_subjects(study, files):
    """The backlog's subject for each passage key it names: the `###` section the key first appears under."""
    path = os.path.join(study, "backlog.md")
    if not os.path.exists(path):
        return {}
    by_short = {}
    for f in files:
        for i in range(len(f.split("/"))):
            by_short.setdefault("/".join(f.split("/")[i:]), []).append(f)
    subjects, heading, section = {}, "(before the first heading)", ""
    for line in open(path):
        if line.startswith("## "):
            section = line[3:].strip()
        if line.startswith("### "):
            heading = re.sub(r"\s*\(\d+ passages.*$", "", line[4:].strip())
        if not (section.startswith("(a)") or section.startswith("(b)")):
            continue
        for short, lo, hi, part in re.findall(r"`?([\w./-]+\.md):(\d+)-(\d+)(?: part (\d+))?`?", line):
            candidates = by_short.get(short) or []
            if len(candidates) > 1:
                candidates = [c for c in candidates if c.startswith("docs/briefs/")] or candidates
            if candidates:
                key = f"{candidates[0]}:{lo}-{hi}" + (f"/{part}" if part else "")
                subjects.setdefault(key, heading)
                subjects.setdefault(f"{candidates[0]}:{lo}-{hi}", heading)
    return subjects


def dispositions(archive):
    out = {}
    path = os.path.join(archive, "dispositions.jsonl")
    if os.path.exists(path):
        for line in open(path):
            if line.strip():
                d = json.loads(line)
                out[d["key"]] = d
    return out


def main():
    args = sys.argv[1:]
    manifests = [args[i + 1] for i, a in enumerate(args) if a == "--manifest"] or ["docs/briefs/rework.manifest"]
    study = next((args[i + 1] for i, a in enumerate(args) if a == "--study"), STUDY)
    archive = next((args[i + 1] for i, a in enumerate(args) if a == "--archive"), ARCHIVE)
    by = next((args[i + 1] for i, a in enumerate(args) if a == "--by"), "file")
    all_rows = [r for r in rows(study) if r.get("fate") in COUNTED]
    covered_keys, decided = covers(manifests), dispositions(archive)
    tally = {f: {"covered": 0, "discarded": 0, "open": 0} for f in COUNTED}
    open_rows, result_rows = [], []
    for r in all_rows:
        home = covered_keys.get(r["part_key"]) or covered_keys.get(r["key"])
        decision = decided.get(r["part_key"]) or decided.get(r["key"])
        if home:
            state, ref = "covered", ", ".join(home)
        elif decision and decision.get("disposition") == "covered":
            state, ref = "covered", decision.get("id", "(a disposition)")
        elif decision:
            state, ref = "discarded", decision.get("reason", "")
        else:
            state, ref = "open", ""
            open_rows.append(r)
        tally[r["fate"]][state] += 1
        result_rows.append({"key": r["part_key"], "fate": r["fate"], "subject": r.get("subject", ""), "state": state, "ref": ref})
    total = {s: sum(t[s] for t in tally.values()) for s in ("covered", "discarded", "open")}
    if "--json" in args:
        print(json.dumps({"manifests": manifests, "tally": tally, "total": total, "rows": result_rows}, indent=1))
    else:
        print(f"{len(all_rows)} rows counted (fates {', '.join(COUNTED)}; tool rows left out); new documents from {', '.join(manifests)}")
        print(f"{'fate':10} {'covered':>8} {'discarded':>10} {'open':>6}")
        for f, t in tally.items():
            print(f"{f:10} {t['covered']:8} {t['discarded']:10} {t['open']:6}")
        print(f"{'all':10} {total['covered']:8} {total['discarded']:10} {total['open']:6}")
        groups = {}
        subjects = backlog_subjects(study, sorted({r["file"] for r in all_rows})) if by == "backlog" else {}
        for r in open_rows:
            g = (r.get("subject", "") if by == "subject" else subjects.get(r["part_key"]) or subjects.get(r["key"]) or "(not in the backlog)" if by == "backlog" else r["file"])
            groups.setdefault(g, []).append(r)
        print(f"\nopen, by {by} ({len(groups)} groups, largest first):")
        for g, rs in sorted(groups.items(), key=lambda kv: -len(kv[1]))[:60]:
            print(f"  {len(rs):3}  {g[:110]}")
        if "--open" in args:
            print("\nevery open row:")
            for r in open_rows:
                print(f"  {r['part_key']:55} {r['fate']:9} {r.get('subject', '')[:90]}")
    if "--strict" in args and total["open"]:
        sys.exit(1)


if __name__ == "__main__":
    main()
