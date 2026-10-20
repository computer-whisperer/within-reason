#!/usr/bin/env python3
"""BARb's profile files for the game version the arena plays, pulled out of the rapid pool, and the differences
between two profiles as JSON key paths (the user, 2026-10-20: "I was told hard is potentially harder than
hard_aggressive, so let's try to measure it").

    run/barb_profiles.py "Beyond All Reason test-31677-a31a826" [--data run/data] [--out DIR] [--diff hard hard_aggressive]

BARb loads its configs and AngelScript from the GAME archive, not the engine's AI directory (K-opp-barb-tiers), so
the engine's copy says nothing about what it played; the version is the `GameType` line of a match's script.txt.
The files land under DIR (default `run/data/barb-profiles/<version>/`) as the archive names them
(`luarules/configs/barb/stable/config/<profile>/*.json`, `.../script/<profile>/...`). `--diff A B` then prints every
config key whose value differs between the two profiles (comments and trailing commas stripped) and names the
script files that differ.
"""
import argparse
import difflib
import glob
import gzip
import json
import os
import re
import struct
import sys

PREFIX = "luarules/configs/barb/stable/"


def package_hash(data, version):
    for path in glob.glob(os.path.join(data, "rapid", "*", "*", "versions.gz")):
        with gzip.open(path, "rt", encoding="utf-8") as fh:
            for line in fh:
                parts = line.rstrip("\n").split(",")
                if len(parts) >= 4 and parts[3] == version:
                    return parts[1]
    sys.exit(f"no rapid package named {version!r} under {data}/rapid (fetch it with pr-downloader first: docs/harness/pitfalls.md)")


def extract(data, version, out):
    sdp = os.path.join(data, "packages", package_hash(data, version) + ".sdp")
    with gzip.open(sdp, "rb") as fh:
        raw = fh.read()
    i, n = 0, 0
    while i < len(raw):
        ln = raw[i]
        name = raw[i + 1:i + 1 + ln].decode("utf-8", "replace")
        md5 = raw[i + 1 + ln:i + 17 + ln].hex()
        i += 25 + ln
        if not name.lower().startswith(PREFIX):
            continue
        dst = os.path.join(out, name.lower())
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        with gzip.open(os.path.join(data, "pool", md5[:2], md5[2:] + ".gz"), "rb") as src, open(dst, "wb") as fh:
            fh.write(src.read())
        n += 1
    return n


def load_json(path):
    text = open(path, encoding="utf-8").read()
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    text = re.sub(r"//[^\n]*", "", text)
    text = re.sub(r",(\s*[}\]])", r"\1", text)
    return json.loads(text)


def walk(a, b, path, out):
    if isinstance(a, dict) and isinstance(b, dict):
        for key in sorted(set(a) | set(b)):
            if key not in a:
                out.append((f"{path}/{key}", "<absent>", b[key]))
            elif key not in b:
                out.append((f"{path}/{key}", a[key], "<absent>"))
            else:
                walk(a[key], b[key], f"{path}/{key}", out)
    elif a != b:
        out.append((path, a, b))


def diff(out, first, second):
    base = os.path.join(out, PREFIX)
    for sub in ("config", "script"):
        da, db = os.path.join(base, sub, first), os.path.join(base, sub, second)
        files = lambda d: {os.path.relpath(os.path.join(root, f), d) for root, _, fs in os.walk(d) for f in fs}
        for name in sorted(files(da) | files(db)):
            pa, pb = os.path.join(da, name), os.path.join(db, name)
            if not (os.path.isfile(pa) and os.path.isfile(pb)):
                print(f"{sub}/{name}: only in {first if os.path.exists(pa) else second}")
                continue
            parsed = None
            if name.endswith(".json"):
                try:
                    parsed = load_json(pa), load_json(pb)
                except ValueError as e:
                    print(f"{sub}/{name}: not parseable as JSON ({e}), compared as text")
            if parsed:
                found = []
                walk(*parsed, "", found)
                for path, x, y in found:
                    print(f"{sub}/{name}{path}: {first}={json.dumps(x)}  {second}={json.dumps(y)}")
            else:
                lines = list(difflib.unified_diff(open(pa, encoding="utf-8", errors="replace").readlines(), open(pb, encoding="utf-8", errors="replace").readlines(), lineterm="", n=0))
                if lines:
                    print(f"{sub}/{name}: {sum(1 for l in lines[2:] if l[:1] in '+-')} changed lines")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("version", help='the game version, e.g. "Beyond All Reason test-31677-a31a826"')
    ap.add_argument("--data", default="run/data")
    ap.add_argument("--out")
    ap.add_argument("--diff", nargs=2, metavar=("A", "B"))
    args = ap.parse_args()
    out = args.out or os.path.join(args.data, "barb-profiles", args.version.replace(" ", "_"))
    print(f"{args.version}: {extract(args.data, args.version, out)} files -> {out}")
    if args.diff:
        diff(out, *args.diff)


if __name__ == "__main__":
    main()
