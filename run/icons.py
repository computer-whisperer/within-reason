#!/usr/bin/env python3
"""Export the game's minimap icons for the viewer (docs/design/2026-09-24-viewer-overhaul.md).

    run/icons.py [--upstream upstream/Beyond-All-Reason] [--units crates/bot/data/units.json]

Reads `gamedata/icontypes.lua` (one entry per unit name: `bitmap`, `size`; `alldefs_post.lua` gives every unit the
entry of its own name) and copies, unmodified, the bitmaps the unit table names into `viewer/icons/`, with
`viewer/icons/LICENSE.txt` (CC BY-NC-ND, IceXuick, Floris, PtaQ: verbatim copies with attribution; the viewer tints
them on the canvas only) and `viewer/icons.json` {unit name: {"file", "size"}}. Run it again after a game update.
"""
import argparse, json, os, re, shutil

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--upstream", default=os.path.join(REPO, "upstream/Beyond-All-Reason"))
    ap.add_argument("--units", default=os.path.join(REPO, "crates/bot/data/units.json"))
    a = ap.parse_args()
    src = open(os.path.join(a.upstream, "gamedata/icontypes.lua")).read()
    entries = {}
    for name, body in re.findall(r"\n\t(\w+) = \{\n((?:\t\t.*\n)+?)\t\}", src):
        bitmap = re.search(r'bitmap = "([^"]+)"', body)
        size = re.search(r"size = ([0-9.]+)", body)
        if bitmap:
            entries[name] = (bitmap.group(1), float(size.group(1)) if size else 1.0)
    units = json.load(open(a.units))["units"]
    names = [u["name"] for u in units] if isinstance(units, list) else list(units)
    out_dir = os.path.join(REPO, "viewer/icons")
    os.makedirs(out_dir, exist_ok=True)
    table, missing, copied = {}, [], set()
    for name in sorted(names):
        e = entries.get(name)
        if not e or not os.path.exists(os.path.join(a.upstream, e[0])):
            missing.append(name)
            continue
        base = os.path.basename(e[0])
        if base not in copied:
            shutil.copyfile(os.path.join(a.upstream, e[0]), os.path.join(out_dir, base))
            copied.add(base)
        table[name] = {"file": base, "size": e[1]}
    with open(os.path.join(out_dir, "LICENSE.txt"), "w") as f:
        f.write("Minimap icons of Beyond All Reason (https://github.com/beyond-all-reason/Beyond-All-Reason, icons/),\n"
                "made by IceXuick, Floris and PtaQ, licensed CC BY-NC-ND (the game's license_icons.txt).\n"
                "Copied unmodified by run/icons.py; the viewer tints them on the canvas at draw time and ships no derivative.\n")
    json.dump(table, open(os.path.join(REPO, "viewer/icons.json"), "w"), indent=0, sort_keys=True)
    print(f"{len(table)} units mapped to {len(copied)} icon files in viewer/icons/; missing {missing}")


if __name__ == "__main__":
    main()
