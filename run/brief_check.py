#!/usr/bin/env python3
"""Check the brief's provenance blocks: every `<!--meta ... -->` block in the documents the manifests name
(docs/briefs/*.manifest), their ids, and the references between them.

    run/brief_check.py [--manifest PATH ...] [--json]

The syntax (docs/design/2026-10-21-brief-rework.md §3): a block before a passage, `key: value` lines, lists in
brackets, a map in braces; the bot strips every HTML comment before a model sees the text and drops the passage of a
block whose status is deprecated (`crates/bot/src/texts.rs` `strip`). Keys read here:

    id: D-sprint-timing                        unique across the brief; prefixes F- (fact), M- (meta), D- (derived), E- (episode)
    layer: fact | meta | derived | episode
    premises: [F-armavp-cost, M-pool-tech-rate]   ids a derivation rests on
    tests: [E-sd129-2v3]                       episodes that test a derivation
    status: live | deprecated <date> (<reason>)

Reports: blocks and ids per document; duplicate ids; premises and tests that name no block; derivations with no
premises; facts and meta no derivation cites; live passages citing a deprecated id; blocks without an id. Exit 1 when
a reference is unresolved or an id is duplicated, so it can gate a commit.
"""
import json
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(REPO)
BLOCK = re.compile(r"<!--meta\n(.*?)-->", re.S)
LIST = re.compile(r"^\[(.*)\]$", re.S)


def documents(manifest):
    return [l.strip() for l in open(manifest) if l.strip() and not l.strip().startswith("#")]


def parse_block(body):
    fields = {}
    for line in body.split("\n"):
        if ":" not in line:
            continue
        key, value = line.split(":", 1)
        value = value.strip()
        m = LIST.match(value)
        fields[key.strip()] = [v.strip() for v in m.group(1).split(",") if v.strip()] if m else value
    return fields


def blocks_of(path):
    text = open(path).read()
    out = []
    for m in BLOCK.finditer(text):
        fields = parse_block(m.group(1))
        fields["_file"], fields["_line"] = path, text[:m.start()].count("\n") + 1
        out.append(fields)
    return out


def main():
    args = sys.argv[1:]
    manifests = [args[i + 1] for i, a in enumerate(args) if a == "--manifest"] or sorted(p for p in os.listdir("docs/briefs") if p.endswith(".manifest"))
    manifests = [m if "/" in m else f"docs/briefs/{m}" for m in manifests]
    seen, blocks = [], []
    for manifest in manifests:
        for document in documents(manifest):
            if document in seen:
                continue
            seen.append(document)
            blocks.extend(blocks_of(document))
    by_id, duplicates, no_id = {}, [], []
    for b in blocks:
        if "id" not in b:
            no_id.append(b)
        elif b["id"] in by_id:
            duplicates.append((b["id"], by_id[b["id"]]["_file"], by_id[b["id"]]["_line"], b["_file"], b["_line"]))
        else:
            by_id[b["id"]] = b
    deprecated = {i for i, b in by_id.items() if str(b.get("status", "live")).startswith("deprecated")}
    unresolved, cites_deprecated, no_premises = [], [], []
    cited = set()
    for b in blocks:
        refs = [(k, r) for k in ("premises", "tests") for r in (b.get(k) if isinstance(b.get(k), list) else [])]
        for key, ref in refs:
            cited.add(ref)
            if ref not in by_id:
                unresolved.append((b.get("id", "?"), key, ref, b["_file"], b["_line"]))
            elif ref in deprecated and not str(b.get("status", "live")).startswith("deprecated"):
                cites_deprecated.append((b.get("id", "?"), key, ref, b["_file"], b["_line"]))
        if b.get("layer") == "derived" and not b.get("premises"):
            no_premises.append((b.get("id", "?"), b["_file"], b["_line"]))
    uncited = sorted(i for i, b in by_id.items() if b.get("layer") in ("fact", "meta") and i not in cited and i not in deprecated)
    report = {"manifests": manifests, "documents": len(seen), "blocks": len(blocks), "ids": len(by_id), "deprecated": len(deprecated),
              "duplicates": duplicates, "unresolved": unresolved, "cites_deprecated": cites_deprecated, "derived_without_premises": no_premises,
              "uncited_facts_and_meta": uncited, "blocks_without_id": [(b["_file"], b["_line"]) for b in no_id]}
    if "--json" in args:
        print(json.dumps(report, indent=1))
    else:
        print(f"{len(seen)} documents, {len(blocks)} meta blocks, {len(by_id)} ids ({len(deprecated)} deprecated)")
        for name in ("duplicates", "unresolved", "cites_deprecated", "derived_without_premises", "uncited_facts_and_meta", "blocks_without_id"):
            items = report[name]
            if items:
                print(f"{name} ({len(items)}):")
                for item in items:
                    print("  " + " ".join(str(x) for x in (item if isinstance(item, tuple) else (item,))))
    sys.exit(1 if duplicates or unresolved or cites_deprecated else 0)


if __name__ == "__main__":
    main()
