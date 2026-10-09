#!/usr/bin/env python3
"""The pianist's log on disk, for every reader (docs/harness/record-format.md, "The pianist's log"): `jev-<n>.jsonl`
while the match runs, `jev-<n>.jsonl.zst` once the arena has closed the match (one zstd frame a line, and last a seek
table in the Zstandard Seekable Format's skippable frame, so that one line is had without reading the rest). A reader
asks here for a seat's log or a match's logs, and reads lines or rows here, and sees no difference between the two forms.

    import jevlog
    for row in jevlog.rows(jevlog.seat(match_dir, 0)): ...     # one seat's rows (dicts; torn lines skipped)
    for path in jevlog.logs(match_dir): ...                    # every seat's log, by seat
    for line in jevlog.lines(path): ...                        # the lines as text, as `open(path)` gave them

    run/jevlog.py <jev-N.jsonl[.zst]>      the log's lines, frames and bytes, and that every frame reads back as its line
"""
import os, re, struct, sys, json
from compression import zstd

NAME = re.compile(r"jev-(\d+)\.jsonl(\.zst)?")
SKIPPABLE_MAGIC = 0x184D2A50  # the low nibble is free: 0x184D2A5E is written
SEEKABLE_MAGIC = 0x8F92EAB1


def seat_of(path):
    """The seat (ai_id) a log is of, from its name; None for a file that is not a seat's log."""
    m = NAME.fullmatch(os.path.basename(path))
    return int(m.group(1)) if m else None


def is_compressed(path):
    return path.endswith(".zst")


def logs(match_dir):
    """Every seat's log in a match directory, by seat; the raw log where both forms stand."""
    found = {}
    try:
        names = os.listdir(match_dir)
    except OSError:
        return []
    for name in names:
        m = NAME.fullmatch(name)
        if m and (int(m.group(1)) not in found or not m.group(2)):
            found[int(m.group(1))] = os.path.join(match_dir, name)
    return [found[n] for n in sorted(found)]


def seat(match_dir, n=0):
    """Seat n's log in a match directory: the raw one if it stands, else the compressed one, else the raw name (so that
    opening it fails as it always did)."""
    for name in (f"jev-{n}.jsonl", f"jev-{n}.jsonl.zst"):
        path = os.path.join(match_dir, name)
        if os.path.exists(path):
            return path
    return os.path.join(match_dir, f"jev-{n}.jsonl")


def raw_lines(path):
    """The lines as bytes, newline included (the last may lack it: a torn line). A compressed log is read as a stream,
    frame after frame, the seek table skipped."""
    if is_compressed(path):
        with zstd.ZstdFile(path, "rb") as f:
            yield from f
    else:
        with open(path, "rb") as f:
            yield from f


def lines(path):
    """The lines as text, what `for line in open(path)` gave a reader of the raw log."""
    for line in raw_lines(path):
        yield line.decode("utf-8", "replace")


def rows(path):
    """The log's rows: a dict a line, torn or broken lines skipped."""
    for line in raw_lines(path):
        try:
            row = json.loads(line)
        except ValueError:
            continue
        if isinstance(row, dict):
            yield row


def frames(path):
    """A compressed log's lines as (offset, compressed length, line length) from its seek table, for a reader that wants
    one line without the rest (`read_frame`); None for a raw log, or a compressed one without the table."""
    if not is_compressed(path):
        return None
    with open(path, "rb") as f:
        size = f.seek(0, os.SEEK_END)
        if size < 17:
            return None
        f.seek(size - 9)
        n, descriptor, magic = struct.unpack("<IBI", f.read(9))
        if magic != SEEKABLE_MAGIC:
            return None
        entry = 12 if descriptor & 0x80 else 8  # a checksum a frame when the high bit is set (the arena writes none)
        table = n * entry + 9
        if size < table + 8:
            return None
        f.seek(size - table - 8)
        head_magic, head_size = struct.unpack("<II", f.read(8))
        if head_magic & 0xFFFFFFF0 != SKIPPABLE_MAGIC or head_size != table:
            return None
        body = f.read(n * entry)
    out, offset = [], 0
    for i in range(n):
        compressed, uncompressed = struct.unpack_from("<II", body, i * entry)
        out.append((offset, compressed, uncompressed))
        offset += compressed
    return out


def read_frame(f, offset, length):
    """One line of a compressed log open as `f` (binary), by its entry in `frames`; newline included."""
    f.seek(offset)
    return zstd.decompress(f.read(length))


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    path = sys.argv[1]
    size = os.path.getsize(path)
    n = raw = 0
    streamed = raw_lines(path)
    table = frames(path)
    if table is None:
        for line in streamed:
            n += 1
            raw += len(line)
        print(f"{path}: {n} lines, {raw / 1e6:.0f} MB{' raw' if not is_compressed(path) else ' compressed without a seek table'} ({size / 1e6:.1f} MB on disk)")
        return
    with open(path, "rb") as f:
        for (offset, length, uncompressed), line in zip(table, streamed, strict=True):
            n += 1
            raw += len(line)
            if len(line) != uncompressed or read_frame(f, offset, length) != line:
                sys.exit(f"{path}: frame {n} does not read back as line {n}")
    print(f"{path}: {n} lines in {n} frames, {raw / 1e6:.0f} MB in {size / 1e6:.1f} MB ({raw / max(size, 1):.0f} to 1); every frame reads back as its line")


if __name__ == "__main__":
    main()
