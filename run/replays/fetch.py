#!/usr/bin/env python3
"""Fetch what the manifest's kept replays need: the demos, the maps, and (asked for) the game versions
(docs/design/2026-09-23-replay-survey.md, decision 2).

    run/replays/fetch.py [--manifest run/data/replays/manifest.jsonl] [--limit N] [--maps] [--versions] [--dry]

Demos go to run/data/replays/<map file name>/<demo file name>; a line that has its demo is left alone. `--maps` fetches
every kept map not in run/data/maps with pr-downloader; `--versions` fetches every kept game version not in the
rapid pool, then `byar:test` again (docs/harness/pitfalls.md: fetching one version moves the tag the arena needs).
Never run `--versions` while an arena batch runs. Without `--maps`/`--versions` the script only says what is missing.
"""
import glob, gzip, json, os, subprocess, sys, urllib.request

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DATA = os.path.join(REPO, "run", "data")
ENV = {"PRD_RAPID_USE_STREAMER": "false", "PRD_RAPID_REPO_MASTER": "https://repos-cdn.beyondallreason.dev/repos.gz", "PRD_HTTP_SEARCH_URL": "https://files-cdn.beyondallreason.dev/find"}


def downloader():
    found = sorted(glob.glob(os.path.join(REPO, "run", "engines", "*", "pr-downloader")))
    if not found:
        sys.exit("no pr-downloader under run/engines")
    return found[-1]


def installed_versions():
    """Game versions in the rapid pool: the names in versions.gz whose package (.sdp) is present."""
    names = set()
    packages = {os.path.basename(p)[:-4] for p in glob.glob(os.path.join(DATA, "packages", "*.sdp"))}
    for path in glob.glob(os.path.join(DATA, "rapid", "*", "*", "versions.gz")):
        with gzip.open(path, "rt", errors="replace") as f:
            for line in f:
                parts = line.rstrip("\n").split(",")
                if len(parts) >= 4 and parts[1] in packages:
                    names.add(parts[3])
    return names


def installed_maps():
    return {os.path.basename(p).rsplit(".", 1)[0] for p in glob.glob(os.path.join(DATA, "maps", "*.sd[7z]"))}


def main():
    args = sys.argv[1:]
    def opt(name, default):
        return args[args.index(name) + 1] if name in args else default
    manifest = opt("--manifest", os.path.join(DATA, "replays", "manifest.jsonl"))
    limit = int(opt("--limit", "0")) or None
    dry = "--dry" in args
    rows = [json.loads(l) for l in open(manifest) if l.strip()]
    kept = [r for r in rows if r.get("keep")]
    changed = False
    fetched = 0
    for r in kept:
        path = os.path.join(DATA, "replays", r["map_file"], r["file"])
        if os.path.exists(path) and os.path.getsize(path) > 0:
            # A demo already on disk (a line re-added after a manifest rewrite lost it) is recorded too.
            if r.get("demo") != os.path.relpath(path, REPO):
                changed = True
            r["demo"] = os.path.relpath(path, REPO)
            continue
        if limit is not None and fetched >= limit:
            continue
        if dry:
            print("would fetch", r["file"])
            continue
        os.makedirs(os.path.dirname(path), exist_ok=True)
        req = urllib.request.Request(r["url"], headers={"User-Agent": "within-reason replay survey"})
        with urllib.request.urlopen(req, timeout=120) as src, open(path + ".part", "wb") as dst:
            dst.write(src.read())
        os.replace(path + ".part", path)
        r["demo"] = os.path.relpath(path, REPO)
        fetched += 1
        changed = True
    if changed:
        with open(manifest, "w") as f:
            for r in rows:
                f.write(json.dumps(r) + "\n")
    maps_missing = sorted({r["map_file"] for r in kept} - installed_maps())
    versions_missing = sorted({r["game_version"] for r in kept} - installed_versions())
    print(f"{fetched} demos fetched, {sum(1 for r in kept if r.get('demo'))} of {len(kept)} kept replays on disk")
    print(f"maps missing: {maps_missing or 'none'}")
    print(f"game versions missing: {versions_missing or 'none'}" + (f" (needed by {', '.join(f'{sum(1 for r in kept if r['game_version'] == v)} games of {v[-13:]}' for v in versions_missing)})" if versions_missing else ""))
    if "--maps" in args and maps_missing and not dry:
        scripts = {r["map_file"]: r["map_script"] for r in kept}
        for m in maps_missing:
            subprocess.run([downloader(), "--filesystem-writepath", DATA, "--download-map", scripts[m]], env={**os.environ, **ENV}, check=False)
    if "--versions" in args and versions_missing and not dry:
        for v in versions_missing:
            subprocess.run([downloader(), "--filesystem-writepath", DATA, "--download-game", v], env={**os.environ, **ENV}, check=False)
        subprocess.run([downloader(), "--filesystem-writepath", DATA, "--download-game", "byar:test"], env={**os.environ, **ENV}, check=False)
        print("versions fetched; byar:test refetched")


if __name__ == "__main__":
    main()
