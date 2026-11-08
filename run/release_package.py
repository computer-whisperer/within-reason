#!/usr/bin/env python3
"""Package a bot of bots/ as the downloads of a release (docs/design/2026-11-07-bot-releases.md §3.2).

    run/release_package.py distilled --so target/release/libbot_distilled.so [--dll path/bot_distilled.dll] [--out dist]

Writes, for the libraries given:
  <out>/<Short>-<version>-INSTALL.zip   Install.bat, INSTALL.txt, README.txt, LICENSE, payload/install.ps1,
                                        payload/<Short>/<version>/{AIInfo.lua, SkirmishAI.dll}
  <out>/<Short>-<version>-LINUX.tar.gz  install.sh, INSTALL.txt, README.txt, LICENSE,
                                        payload/<Short>/<version>/{AIInfo.lua, libSkirmishAI.so}
The payload directory is laid out by run/bot_payload.sh, the same as install_ai.sh installs into the arena's data
dir, so the release and a development game hold the identical directory. The installers and texts come from
release/installer/ with @SHORT@, @VERSION@, @NAME@, @DESCRIPTION@, @URL@ and @ENGINE@ filled from the bot's AIInfo.lua
and --engine. Windows text files go out with CRLF endings (cmd.exe misreads a .bat with LF). Prints the files written.
"""
import argparse
import os
import re
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
INSTALLER = REPO / "release/installer"


def ai_info(path: Path) -> dict:
    """The key/value pairs of an AIInfo.lua (single-quoted values, one entry per `{ key = ..., value = ... }`)."""
    text = path.read_text(encoding="utf-8")
    return {k: v for k, v in re.findall(r"key\s*=\s*'([^']*)'\s*,\s*value\s*=\s*'([^']*)'", text)}


def fill(text: str, values: dict) -> str:
    for key, value in values.items():
        text = text.replace(f"@{key}@", value)
    left = re.findall(r"@[A-Z]+@", text)
    if left:
        raise SystemExit(f"unfilled placeholders: {', '.join(sorted(set(left)))}")
    return text


def crlf(text: str) -> bytes:
    return text.replace("\r\n", "\n").replace("\n", "\r\n").encode("utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("bot", help="a directory of bots/ (distilled, territory, template)")
    ap.add_argument("--so", help="the Linux library to ship (libbot_<bot>.so)")
    ap.add_argument("--dll", help="the Windows library to ship (bot_<bot>.dll)")
    ap.add_argument("--out", default="dist", help="output directory (default dist/)")
    ap.add_argument("--engine", default="2026.07.04", help="the BAR engine the release was tested on, for the texts")
    ap.add_argument("--url", help="the url written into AIInfo.lua and the texts (default: bot_payload.sh's, the repository)")
    args = ap.parse_args()
    if not args.so and not args.dll:
        raise SystemExit("give --so and/or --dll")

    bot_dir = REPO / "bots" / args.bot
    if not (bot_dir / "data/AIInfo.lua").exists():
        raise SystemExit(f"no bot at {bot_dir}")
    env = dict(os.environ)
    if args.url:
        env["WITHIN_REASON_BOT_URL"] = args.url
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    written = []

    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        stage = tmp / "payload-stage"
        version = None
        for lib in (args.so, args.dll):
            if not lib:
                continue
            if not Path(lib).exists():
                raise SystemExit(f"no library at {lib}")
            version = subprocess.run([str(REPO / "run/bot_payload.sh"), args.bot, lib, str(stage)], check=True, env=env,
                                     capture_output=True, text=True).stdout.strip()
        info = ai_info(stage / "AIInfo.lua")
        short = info["shortName"]
        values = {"SHORT": short, "VERSION": version, "NAME": info["name"], "DESCRIPTION": info["description"],
                  "URL": info["url"], "ENGINE": args.engine}
        if info["version"] != version:
            raise SystemExit(f"AIInfo.lua says {info['version']}, Cargo.toml {version}")
        payload_rel = f"payload/{short}/{version}"
        texts = {name: fill((INSTALLER / f"{name}.in").read_text(encoding="utf-8"), values) for name in ("INSTALL.txt", "README.txt")}
        license_text = (REPO / "LICENSE").read_bytes()

        if args.dll:
            zip_path = out_dir / f"{short}-{version}-INSTALL.zip"
            with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as z:
                z.writestr("Install.bat", crlf(fill((INSTALLER / "Install.bat").read_text(encoding="utf-8"), values)))
                z.writestr("payload/install.ps1", crlf(fill((INSTALLER / "install.ps1").read_text(encoding="utf-8"), values)))
                for name, text in texts.items():
                    z.writestr(name, crlf(text))
                z.writestr("LICENSE", license_text)
                z.write(stage / "AIInfo.lua", f"{payload_rel}/AIInfo.lua")
                z.write(stage / "SkirmishAI.dll", f"{payload_rel}/SkirmishAI.dll")
            written.append(zip_path)

        if args.so:
            tar_path = out_dir / f"{short}-{version}-LINUX.tar.gz"
            files = [
                ("install.sh", fill((INSTALLER / "install.sh").read_text(encoding="utf-8"), values).encode("utf-8"), 0o755),
                ("INSTALL.txt", texts["INSTALL.txt"].encode("utf-8"), 0o644),
                ("README.txt", texts["README.txt"].encode("utf-8"), 0o644),
                ("LICENSE", license_text, 0o644),
                (f"{payload_rel}/AIInfo.lua", (stage / "AIInfo.lua").read_bytes(), 0o644),
                (f"{payload_rel}/libSkirmishAI.so", (stage / "libSkirmishAI.so").read_bytes(), 0o755),
            ]
            import io
            with tarfile.open(tar_path, "w:gz") as t:
                for rel, data, mode in files:
                    ti = tarfile.TarInfo(rel)
                    ti.size = len(data)
                    ti.mode = mode
                    ti.mtime = int(os.environ.get("SOURCE_DATE_EPOCH", "0")) or int(Path(args.so).stat().st_mtime)
                    t.addfile(ti, io.BytesIO(data))
            written.append(tar_path)

    for path in written:
        print(f"{path}  ({path.stat().st_size / 1e6:.1f} MB)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
