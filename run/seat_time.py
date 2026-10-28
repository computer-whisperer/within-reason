#!/usr/bin/env python3
"""What each AI seat of ours cost the engine thread (the host's `timing` lines; docs/design/2026-10-27-engine-thread.md).

    run/seat_time.py LABEL-OR-DIR [LABEL-OR-DIR...] [--marks 1,5,10,20]

A label is a batch's (the newest of that label in run/matches); a directory is a batch, a game in one, or any directory
holding an engine log (a client's log copied out). Per game and seat, which bot it is (from its own log lines), the
hello's reads and the bot's start, the seconds of engine thread a minute at the marks (reads / bot / apply), the slow
calls after frame 0 by size, and the worst three.
"""
import collections, glob, os, re, sys

MINUTE = re.compile(r"\[wreason ai=(\d+)\] timing f=(\d+) minute: reads ([\d.]+) s, bot ([\d.]+) s, apply ([\d.]+) s")
SLOW = re.compile(r"\[wreason ai=(\d+)\] timing f=(\d+) slow (tick|reads|apply) (\d+) ms")
HELLO = re.compile(r"\[wreason ai=(\d+)\] timing f=\d+ hello: reads (\d+) ms, bot start (\d+) ms")
NAME = re.compile(r"\[(territory|distilled|template) ai=(\d+)\]")
FRAMES_A_MINUTE = 1800


def logs(arg):
    if os.path.isfile(arg):
        return [arg]
    if not os.path.isdir(arg):
        batches = sorted(glob.glob(f"run/matches/*-{arg}"), key=lambda d: os.path.basename(d).split("-")[0])
        if not batches:
            sys.exit(f"no batch labelled {arg}")
        arg = batches[-1]
    found = sorted(glob.glob(f"{arg}/engine.log") + glob.glob(f"{arg}/*/engine.log") + glob.glob(f"{arg}/infolog.txt"))
    return found[:1] if f"{arg}/engine.log" in found else found


def bucket(ms):
    return "under 100" if ms < 100 else "100-250" if ms < 250 else "250-1000" if ms < 1000 else "1000+"


def report(path, marks):
    names, hello, minutes, slow = {}, {}, collections.defaultdict(dict), collections.defaultdict(list)
    for line in open(path, errors="replace"):
        if m := NAME.search(line):
            names.setdefault(int(m.group(2)), m.group(1))
        if m := MINUTE.search(line):
            minutes[int(m.group(1))][int(m.group(2)) // FRAMES_A_MINUTE] = tuple(float(m.group(i)) for i in (3, 4, 5))
        elif m := SLOW.search(line):
            slow[int(m.group(1))].append((int(m.group(2)), m.group(3), int(m.group(4))))
        elif m := HELLO.search(line):
            hello[int(m.group(1))] = (int(m.group(2)), int(m.group(3)))
    print(f"== {path}")
    for ai in sorted(set(minutes) | set(slow) | set(hello)):
        row = " ".join(f"{k}:{'/'.join(f'{v:.2f}' for v in minutes[ai][k])}" for k in marks if k in minutes[ai])
        first = [ms for f, kind, ms in slow[ai] if f == 0 and kind == "tick"]
        later = [(f, kind, ms) for f, kind, ms in slow[ai] if f > 0]
        sizes = dict(sorted(collections.Counter(bucket(ms) for _, _, ms in later).items()))
        worst = ", ".join(f"{ms} ms {kind} at {f / FRAMES_A_MINUTE:.1f} min" for f, kind, ms in sorted(later, key=lambda s: -s[2])[:3])
        reads, start = hello.get(ai, ("?", "?"))
        print(f"ai={ai} {names.get(ai, '?')}: hello reads {reads} ms, start {start} ms; first tick {first[0] if first else 'under 50'} ms")
        print(f"   engine thread a minute, reads/bot/apply s, at minute {row or '-'}")
        print(f"   slow calls after frame 0: {len(later)} {sizes or ''}{'; worst ' + worst if worst else ''}")


def main():
    args = sys.argv[1:]
    marks = [1, 5, 10, 15, 20, 30, 40]
    if "--marks" in args:
        i = args.index("--marks")
        marks = [int(m) for m in args[i + 1].split(",")]
        del args[i:i + 2]
    if not args:
        print(__doc__.split("\n\n")[1].strip())
        sys.exit(2)
    for arg in args:
        for path in logs(arg):
            report(path, marks)


if __name__ == "__main__":
    main()
