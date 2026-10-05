#!/usr/bin/env python3
"""The viewer's index of a pianist's log (jev-<n>.jsonl, docs/harness/record-format.md): what run/view_match.py
serves of a log too big for the page (a team game's are 1 to 2.4 GB a seat).

The log is read once, line by line. Nine tenths of it is the `menus` of its `pass` rows, which the index does not
parse: a pass row is parsed with its menus cut out. Per line it keeps the byte offset, the frame and the kind; per
game minute a summary (tokens by request kind and by question family as run/jev_products.py counts them, calls,
latencies, gates, quiet seconds, plays, audits and faults by layer). The index is kept in memory and in a sidecar
`.viewer-jev-<n>.json` beside the log, and is continued from where it stopped when the log has grown.

A gate is a `pass` row with `gate` and every row after it up to the next such row (its calls, audits,
`worlds_gate` and `plan`: in a realtime game they carry the frame the answer landed, later than the pass's).

    run/viewer_index.py <jev-N.jsonl> [--fresh]          the summary's totals, and how long the read took (--fresh:
                                                         without the sidecar)
    run/viewer_index.py <jev-N.jsonl> --gate <frame>     the gate at or before the frame, as the server sends it
    run/viewer_index.py <jev-N.jsonl> --picture <frame>  the picture of the last call at or before the frame
"""
import bisect, hashlib, json, os, re, sys, threading, time

from jev_products import family

FORMAT = 2
PRICE = 0.042  # dollars a million input tokens (docs/studies/2026-10-04-jev-where-the-tokens-go.md)
MINUTE = 1800  # frames
TAIL_KIND = re.compile(rb'"t":"([a-z_]+)"')
# Line kinds: a pass that asked (G) or did not (p), the gate's call (c), a pick's call (k), decode, audit,
# worlds_gate, plan (n), error, header, shed, anything else (?).
KIND = {"decode": "d", "audit": "a", "worlds_gate": "w", "plan": "n", "error": "e", "header": "h", "shed": "s"}
GATE_ROWS = "ckawnes"


def new_minute():
    return {"tokens": {}, "calls": {}, "requests": {}, "families": {}, "ms": [], "gates": 0, "quiet": 0, "picks": 0, "taken": 0,
            "plays": 0, "errors": 0, "questions": 0, "audits": {}, "skipped": {}}


def add(counter, key, amount):
    counter[key] = counter.get(key, 0) + amount


class JevIndex:
    def __init__(self, path, fresh=False):
        self.path = path
        self.lock = threading.Lock()
        self.fresh = fresh  # read the log anew, whatever the sidecar holds
        self.reset()

    def reset(self):
        self.off, self.length, self.frame, self.kind = [], [], [], ""
        self.minutes = {}
        self.header = None
        self.scanned = 0
        self.head = ""      # a hash of the file's first bytes: another file under the same name starts the index over
        self.decodes = []   # [line, epoch]: the decode rows, and which packet's readings they are
        self.packets = []   # the lines of the calls that carry `instructions` (the packet as the hands were sent it)
        self.epoch_keys = set()
        self.epoch = 0
        self.gates = []     # lines of the pass rows that asked
        self.calls = []     # lines of every call row
        self.saved = 0
        self.building = False
        self.total = 0

    # ------------------------------------------------------------ the sidecar

    def sidecar(self):
        directory, name = os.path.split(self.path)
        return os.path.join(directory, ".viewer-" + name[: -len(".jsonl")] + ".json")

    def load(self):
        try:
            with open(self.sidecar()) as f:
                saved = json.load(f)
        except (OSError, ValueError):
            return
        if saved.get("format") != FORMAT or saved.get("head") != self.file_head() or saved.get("scanned", 0) > os.path.getsize(self.path):
            return
        for key in ("off", "length", "frame", "kind", "header", "scanned", "head", "decodes", "packets", "epoch", "gates", "calls"):
            setattr(self, key, saved[key])
        self.minutes = {int(k): v for k, v in saved["minutes"].items()}
        self.epoch_keys = set(saved["epoch_keys"])
        self.saved = self.scanned

    def save(self):
        body = {key: getattr(self, key) for key in ("off", "length", "frame", "kind", "header", "scanned", "head", "decodes", "packets", "epoch", "gates", "calls")}
        body.update(format=FORMAT, minutes=self.minutes, epoch_keys=sorted(self.epoch_keys))
        try:
            with open(self.sidecar() + ".tmp", "w") as f:
                json.dump(body, f, separators=(",", ":"))
            os.replace(self.sidecar() + ".tmp", self.sidecar())
            self.saved = self.scanned
        except OSError:
            pass  # a directory we may not write to: the index lives in memory

    def file_head(self):
        try:
            with open(self.path, "rb") as f:
                return hashlib.sha1(f.read(4096)).hexdigest()
        except OSError:
            return ""

    # ------------------------------------------------------------ reading

    def update(self):
        """Reads what the log has gained since the last call. Whole lines only: the writer may be mid-line."""
        with self.lock:
            if not self.scanned and not self.fresh:
                self.load()
            try:
                size = os.path.getsize(self.path)
            except OSError:
                return
            if size < self.scanned or (self.scanned and self.head != self.file_head()):
                self.reset()
            if size == self.scanned:
                return
            if not self.head:
                self.head = self.file_head()
            self.total = size
            self.building = True
            try:
                with open(self.path, "rb") as f:
                    f.seek(self.scanned)
                    while True:
                        line = f.readline()
                        if not line.endswith(b"\n"):
                            break
                        self.add_line(line)
                        self.scanned += len(line)
            finally:
                self.building = False
            # Written when the first read is done, and then as a live log grows by 32 MB.
            if self.scanned - self.saved > (32 << 20) or (self.saved == 0 and self.scanned):
                self.save()

    def add_line(self, line):
        kind_word = None
        for found in TAIL_KIND.finditer(line, max(0, len(line) - 400)):
            kind_word = found.group(1).decode()
        row = None
        if kind_word == "pass":
            # The menus are the bulk of the log and nothing here needs them.
            a, b = line.find(b',"menus":['), line.rfind(b'],"open":[')
            try:
                row = json.loads(line[:a] + line[b + 1:]) if 0 <= a < b else json.loads(line)
            except ValueError:
                row = None
        if row is None:
            try:
                row = json.loads(line)
            except ValueError:
                return  # a torn line
        if not isinstance(row, dict):
            return
        t = row.get("t") or ("call" if "questions" in row else "?")
        f = row.get("f") or 0
        n = len(self.off)
        minute = self.minutes.setdefault(f // MINUTE, new_minute())
        code = KIND.get(t, "?")
        if t == "header":
            self.header = {k: v for k, v in row.items() if k != "t"}
        elif t == "pass":
            code = "G" if "gate" in row else "p"
            if code == "G":
                self.gates.append(n)
                minute["gates"] += 1
                minute["questions"] += len(row["gate"])
                for layer, counts in (row.get("layers") or {}).items():
                    add(minute["skipped"], layer, counts.get("skipped", 0))
            if "quiet" in row:
                minute["quiet"] += 1
            minute["plays"] += len(row.get("played") or [])
        elif t == "call":
            code = self.add_call(row, minute, n)
        elif t == "decode":
            tokens = (row.get("usage") or {}).get("input_tokens", 0) or 0
            add(minute["tokens"], "decode", tokens)
            add(minute["calls"], "decode", 1)
            add(minute["requests"], "decode", row.get("batches", 1) or 1)
            add(minute["families"], "the decode (actor x place, type, group)", tokens)
            minute["ms"].append(row.get("ms", 0))
            # The decode asks only what it does not hold: a key read twice is a new packet's first reading.
            keys = set(row.get("reads") or {})
            if keys & self.epoch_keys:
                self.epoch += 1
                self.epoch_keys = set()
            self.epoch_keys |= keys
            self.decodes.append([n, self.epoch])
        elif t == "audit":
            entry = minute["audits"].setdefault(row.get("layer", "?"), [0, 0])
            entry[0] += 1
            entry[1] += 1 if row.get("fault") else 0
        elif t == "plan":
            minute["picks"] += 1
            minute["plays"] += len(row.get("played") or [])
            minute["taken"] += sum(1 for part in row.get("split") or [] if part.get("taken")) if "split" in row else int(row.get("pick", 1) != 1)
        elif t == "error":
            minute["errors"] += 1
        self.off.append(self.scanned)
        self.length.append(len(line))
        self.frame.append(f)
        self.kind += code

    def add_call(self, row, minute, n):
        """A call's tokens by request kind and, for a gate, by question family: the picture's share and each
        question's by their characters, as run/jev_products.py splits them."""
        questions = row.get("questions") or {}
        tokens = (row.get("usage") or {}).get("input_tokens", 0) or 0
        batches = row.get("batches", 1) or 1
        pick = any(k.startswith("worlds") for k in questions)
        kind = "pick" if pick else "gate"
        add(minute["tokens"], kind, tokens)
        add(minute["calls"], kind, 1)
        add(minute["requests"], kind, batches)
        minute["ms"].append(row.get("ms", 0))
        if isinstance(row.get("instructions"), str):
            self.packets.append(n)
        self.calls.append(n)
        if pick:
            add(minute["families"], "the pick (its picture and questions)", tokens)
            return "k"
        sizes = {k: len(json.dumps(v)) for k, v in questions.items()}
        picture = (sum(len(json.dumps(v)) for v in (row.get("state") or {}).values()) + 9000) * batches
        whole = picture + sum(sizes.values())
        add(minute["families"], "the picture in the gate (actors x requests)", tokens * picture / whole)
        for k, size in sizes.items():
            add(minute["families"], family(k), tokens * size / whole)
        return "c"

    # ------------------------------------------------------------ what the server sends

    def raw(self, f, line):
        f.seek(self.off[line])
        return f.read(self.length[line]).rstrip(b"\n")

    def summary(self):
        with self.lock:
            minutes = []
            for m in sorted(self.minutes):
                row = dict(self.minutes[m], minute=m)
                ms = sorted(row.pop("ms"))
                row["medianMs"] = ms[len(ms) // 2] if ms else 0
                row["maxMs"] = ms[-1] if ms else 0
                row["families"] = {k: round(v) for k, v in row["families"].items()}
                minutes.append(row)
            tokens = sum(sum(m["tokens"].values()) for m in minutes)
            header = dict(self.header or {})
            header.pop("rules", None)
            return {
                "ready": True, "header": header, "version": (self.header or {}).get("version", 1), "size": self.scanned, "lines": len(self.off),
                "price": PRICE, "tokens": tokens, "dollars": tokens * PRICE / 1e6,
                "gates": [self.frame[n] for n in self.gates], "minutes": minutes,
            }

    def gate(self, frame):
        """The gate at or before the frame as one JSON object, the rows spliced in as they stand in the log."""
        with self.lock:
            frames = [self.frame[n] for n in self.gates]
            i = bisect.bisect_right(frames, frame) - 1
            if i < 0:
                return json.dumps({"gate": None, "next": frames[0] if frames else None}).encode()
            first = self.gates[i]
            end = self.gates[i + 1] if i + 1 < len(self.gates) else len(self.off)
            parts = {"c": [], "k": [], "a": [], "e": [], "s": [], "w": [], "n": []}
            with open(self.path, "rb") as f:
                passed = self.raw(f, first)
                for line in range(first + 1, end):
                    if self.kind[line] in GATE_ROWS:
                        parts[self.kind[line]].append(self.raw(f, line))
                # The decode's readings in force: every decode row of the packet's epoch written before this pass.
                epoch = max((e for n, e in self.decodes if n < first), default=None)
                reads = {}
                for n, e in self.decodes:
                    if e == epoch and n < first:
                        reads.update(json.loads(self.raw(f, n)).get("reads") or {})
                # The packet as the hands were sent it: the last call that carried one, up to this gate's own.
                own = next((line for line in range(first + 1, end) if self.kind[line] == "c"), first)
                at = bisect.bisect_right(self.packets, own) - 1
                packet = json.loads(self.raw(f, self.packets[at])) if at >= 0 else {}
            join = lambda rows: b"[" + b",".join(rows) + b"]"
            meta = {
                "f": self.frame[first], "line": first, "prev": frames[i - 1] if i > 0 else None, "next": frames[i + 1] if i + 1 < len(frames) else None,
                "reads": reads, "instructions": packet.get("instructions"), "instructions_f": packet.get("f"),
            }
            return (json.dumps(meta).encode()[:-1] + b',"pass":' + passed + b',"calls":' + join(parts["c"]) + b',"picks":' + join(parts["k"])
                    + b',"audits":' + join(parts["a"]) + b',"errors":' + join(parts["e"]) + b',"shed":' + join(parts["s"])
                    + b',"worlds_gate":' + (parts["w"][-1] if parts["w"] else b"null") + b',"plan":' + (parts["n"][-1] if parts["n"] else b"null") + b"}")

    def picture(self, frame):
        """The names of the last call at or before the frame: its groups, places and parties, and the actors'
        entries of its picture."""
        with self.lock:
            frames = [self.frame[n] for n in self.calls]
            i = bisect.bisect_right(frames, frame) - 1
            if i < 0:
                return json.dumps({"f": None}).encode()
            with open(self.path, "rb") as f:
                row = json.loads(self.raw(f, self.calls[i]))
            state = row.get("state") or {}
            return json.dumps({"f": row.get("f"), "groups": row.get("groups") or [], "places": row.get("places") or [], "parties": row.get("parties") or [],
                               "actors": state.get("actors") or {}, "signals": state.get("signals")}).encode()

    def progress(self):
        return self.scanned / self.total if self.total else 0.0


INDEXES = {}
INDEXES_LOCK = threading.Lock()


def index_of(path):
    """The index of a log, read in the background the first time: None while that first read runs (ask again)."""
    with INDEXES_LOCK:
        index = INDEXES.get(path)
        if index is None:
            index = INDEXES[path] = JevIndex(path)
            index.first = threading.Thread(target=index.update, daemon=True)
            index.first.start()
    if index.first.is_alive():
        index.first.join(timeout=0.2)  # a small log is ready within the request
        if index.first.is_alive():
            return None, index
    index.update()
    return index, index


def main():
    path = sys.argv[1]
    started = time.time()
    index = JevIndex(path, fresh="--fresh" in sys.argv)
    index.update()
    took = time.time() - started
    if "--gate" in sys.argv:
        sys.stdout.buffer.write(index.gate(int(sys.argv[sys.argv.index("--gate") + 1])) + b"\n")
    elif "--picture" in sys.argv:
        sys.stdout.buffer.write(index.picture(int(sys.argv[sys.argv.index("--picture") + 1])) + b"\n")
    else:
        s = index.summary()
        families = {}
        for m in s["minutes"]:
            for k, v in m["families"].items():
                add(families, k, v)
        print(f"{path}: {s['lines']} lines, {s['size'] / 1e6:.0f} MB read in {took:.1f} s; version {s['version']}, {len(s['gates'])} gates, {s['tokens'] / 1e6:.2f}M tokens, ${s['dollars']:.2f}")
        for k, v in sorted(families.items(), key=lambda kv: -kv[1]):
            print(f"   {k[:52]:52s} {100 * v / max(1, s['tokens']):5.1f}%")


if __name__ == "__main__":
    main()
