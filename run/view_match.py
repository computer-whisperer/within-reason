#!/usr/bin/env python3
"""Open recorded matches in the web viewer (viewer/, format in docs/harness/record-format.md).

usage: run/view_match.py [run/matches | run/matches/<batch> | run/matches/<batch>/<NN>] [--port N] [--bind ADDRESS] [--no-browser]

Given the matches directory (the default), the page opens on a browser of every batch in it, newest first, with each
match's result; a match opens from there (`?match=matches/<batch>/<NN>/`). Given one batch, the browser lists that
batch's matches. Given one match, the page opens on it as before. The whole directory given is served, read-only, at
/matches/ until interrupted (a single match also at /match/, the old address). It listens on `::` by default: every
interface, IPv6 and IPv4, so the viewer can be opened from another machine on the network. That exposes the match
directories (logs, records, transcripts) to that network; `--bind 127.0.0.1` keeps it to this machine.

A match still being played can be watched: the viewer asks for each file's new bytes (`?from=<byte offset>`) every few
seconds and follows the newest sample. The browser lists a batch without a results line as still being played.

What the page asks for, beside the files themselves:

    /matches/index.json                    the browser's listing: every batch with its options, results and Jev bill
    <match>/index.json                     the match's files with their sizes, its batch's options, its result line
    <match>/<file>?from=<offset>           the file from a byte offset on (a tail: the header X-From repeats the offset)
    <match>/jev-<n>.jsonl?index            the log's index (run/viewer_index.py): per game minute the tokens by request
                                           kind and question family, calls, gates, audits; the frames of its gates.
                                           {"ready": false, "progress": 0..1} while a big log is first read
    <match>/jev-<n>.jsonl?gate=<frame>     the gate at or before the frame, whole: its pass row, the calls, audits,
                                           worlds_gate and plan that followed, the decode's readings in force
    <match>/jev-<n>.jsonl?picture=<frame>  the groups, places, parties and actors of the last call at or before the frame

A Jev log is never sent whole to the page (a team game's are gigabytes); its index is cached in a sidecar
`.viewer-jev-<n>.json` beside it, the only file this server writes.
"""
import argparse, gzip, http.server, json, os, re, socket, sys, time, webbrowser

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import viewer_index

VIEWER = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "viewer")


def is_match(path):
    return os.path.isdir(path) and any(f.startswith("record-") and f.endswith(".jsonl") for f in os.listdir(path))


def is_batch(path):
    return os.path.isdir(path) and any(f.isdigit() and os.path.isdir(os.path.join(path, f)) for f in os.listdir(path))


def read_json(path, default):
    try:
        with open(path) as f:
            return json.load(f)
    except (OSError, ValueError):
        return default


def read_jsonl(path):
    out = []
    try:
        with open(path) as f:
            for line in f:
                try:
                    out.append(json.loads(line))
                except ValueError:
                    continue  # a torn last line
    except OSError:
        pass
    return out


def jev_bill(match_dir):
    """A match's Jev bill in dollars from its bot.log (the hands' line each minute, every seat's): cheap to read
    where the logs themselves are gigabytes. None when the log has no such line."""
    tokens, found = 0, False
    try:
        with open(os.path.join(match_dir, "bot.log"), errors="replace") as f:
            for line in f:
                m = re.search(r"pianist this minute: .* (\d+) tokens in", line)
                if m:
                    tokens += int(m.group(1))
                    found = True
    except OSError:
        return None
    return tokens * viewer_index.PRICE / 1e6 if found else None


def batch_entry(root, name):
    """One batch for the browser: its options from batch.json, each match's result from results.jsonl, and which match
    directories hold a record."""
    path = os.path.join(root, name)
    options = read_json(os.path.join(path, "batch.json"), {})
    results = {r.get("index"): r for r in read_jsonl(os.path.join(path, "results.jsonl")) if isinstance(r, dict)}
    matches = []
    for sub in sorted(f for f in os.listdir(path) if f.isdigit() and os.path.isdir(os.path.join(path, f))):
        r = results.get(int(sub)) or {}
        matches.append({
            "index": sub, "record": is_match(os.path.join(path, sub)),
            "outcome": r.get("outcome"), "minutes": r.get("game_minutes"), "corner": r.get("our_corner"), "side": r.get("our_side"),
            "arm": r.get("arm") or None, "called": r.get("called"), "jev_dollars": jev_bill(os.path.join(path, sub)),
        })
    try:
        started = os.path.getmtime(os.path.join(path, "batch.json")) if options else os.path.getmtime(path)
    except OSError:
        started = 0
    # A game with people has no results.jsonl (the arena writes that) and no result line (the engine drops the
    # connection at the end): its records go quiet. Quiet for three minutes is ended, and shown as "no result".
    newest_write = 0
    for sub in matches:
        for file in os.listdir(os.path.join(path, sub["index"])) if sub["record"] else []:
            if file.startswith("record-") and file.endswith(".jsonl"):
                try:
                    newest_write = max(newest_write, os.path.getmtime(os.path.join(path, sub["index"], file)))
                except OSError:
                    pass
    quiet = newest_write > 0 and time.time() - newest_write > 180
    return {
        "batch": name, "started": started, "label": options.get("label"), "commit": options.get("commit"),
        "opponent": options.get("opponent"), "map": options.get("map"), "profile": options.get("profile"),
        "pianist": options.get("pianist"), "player": options.get("player"), "packet": os.path.basename(options["packet"]) if options.get("packet") else None,
        "ours": options.get("ours"), "enemies": options.get("enemies"), "allies": options.get("allies"), "players": options.get("players"),
        "player_model": options.get("player_model"), "commander": options.get("commander"), "realtime": options.get("realtime"),
        "max_minutes": options.get("max_minutes"), "matches": matches, "finished": bool(results) or quiet,
    }


class Handler(http.server.SimpleHTTPRequestHandler):
    root = None        # the directory served at /matches/
    match_dir = None   # a single match, served at /match/ too (the old address)
    root_kind = "root"  # root | batch | match: what the page's browser should list

    def translate_path(self, path):
        # The base class resolves against `directory` and drops "..", so no root can be escaped.
        if path.startswith("/matches/"):
            self.directory = self.root
            path = path[len("/matches"):]
        elif path.startswith("/match/") and self.match_dir:
            self.directory = self.match_dir
            path = path[len("/match"):]
        else:
            self.directory = VIEWER
        return super().translate_path(path)

    def send_json(self, body):
        """`body`: a value, or JSON already as bytes. Compressed when it is big and the reader takes gzip (a gate is
        about a megabyte of words)."""
        if not isinstance(body, bytes):
            body = json.dumps(body).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        if len(body) > 4096 and "gzip" in self.headers.get("Accept-Encoding", ""):
            body = gzip.compress(body, compresslevel=1)
            self.send_header("Content-Encoding", "gzip")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def send_jev(self, path, query):
        """The index of a Jev log, a gate or a picture out of it (run/viewer_index.py)."""
        file = self.translate_path(path)
        if not os.path.isfile(file):
            return self.send_error(404)
        index, reading = viewer_index.index_of(file)
        if index is None:
            return self.send_json({"ready": False, "progress": reading.progress()})
        try:
            if query == "index":
                return self.send_json(index.summary())
            what, _, frame = query.partition("=")
            return self.send_json(index.gate(int(float(frame))) if what == "gate" else index.picture(int(float(frame))))
        except (OSError, ValueError):
            return self.send_error(400)

    def do_GET(self):
        path = self.path.split("?")[0]
        if path == "/matches/index.json":
            return self.send_json(self.listing())
        if path.endswith("/index.json") and (path.startswith("/matches/") or (path.startswith("/match/") and self.match_dir)):
            directory = self.translate_path(path[: -len("index.json")])
            if not os.path.isdir(directory):
                return self.send_error(404)
            files = sorted(f for f in os.listdir(directory) if os.path.isfile(os.path.join(directory, f)) and not f.startswith(".viewer-"))
            sizes = {f: os.path.getsize(os.path.join(directory, f)) for f in files}
            # The batch the match belongs to (its directory's parent): how it was played, and how it ended.
            parent = os.path.dirname(directory.rstrip("/"))
            number = os.path.basename(directory.rstrip("/"))
            result = next((r for r in read_jsonl(os.path.join(parent, "results.jsonl")) if isinstance(r, dict) and number.isdigit() and r.get("index") == int(number)), None)
            # The engine's own replay (demos/*.sdfz), for the viewer's download link; the match dir is served whole.
            demos = os.path.join(directory, "demos")
            replays = sorted(f"demos/{f}" for f in os.listdir(demos) if f.endswith(".sdfz")) if os.path.isdir(demos) else []
            return self.send_json({"dir": directory, "files": files, "sizes": sizes, "replays": replays, "batch": read_json(os.path.join(parent, "batch.json"), None), "result": result})
        query = self.path.partition("?")[2]
        if re.fullmatch(r".*/jev-[^/]*\.jsonl", path) and (query == "index" or re.fullmatch(r"(gate|picture)=-?[\d.]+", query)) and (path.startswith("/matches/") or path.startswith("/match/")):
            return self.send_jev(path, query)
        if (path.startswith("/matches/") or path.startswith("/match/")) and "?from=" in self.path:
            return self.send_tail()
        super().do_GET()

    def listing(self):
        """The browser's index: every batch under the root (or the one batch, or the one match), newest first."""
        if self.root_kind == "match":
            name = os.path.basename(self.root)
            return {"kind": "match", "root": self.root, "batches": [{"batch": ".", "label": name, "started": os.path.getmtime(self.root), "matches": [{"index": "", "record": True}], "finished": True}]}
        if self.root_kind == "batch":
            return {"kind": "batch", "root": self.root, "batches": [dict(batch_entry(os.path.dirname(self.root), os.path.basename(self.root)), batch=".")]}
        batches = [batch_entry(self.root, name) for name in os.listdir(self.root) if is_batch(os.path.join(self.root, name))]
        batches.sort(key=lambda b: b["started"], reverse=True)
        return {"kind": "root", "root": self.root, "batches": batches}

    def send_tail(self):
        """The file from a byte offset on: how the viewer follows a growing record without fetching it whole."""
        path, _, offset = self.path.partition("?from=")
        try:
            with open(self.translate_path(path), "rb") as f:
                f.seek(int(offset))
                body = f.read()
        except (OSError, ValueError):
            return self.send_error(404)
        self.send_response(200)
        self.send_header("Content-Type", "application/octet-stream")
        self.send_header("X-From", str(int(offset)))  # tells the viewer this is a tail, not the whole file
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")  # a match still being played grows between reloads
        super().end_headers()

    def log_message(self, *args):
        pass


class Server(http.server.ThreadingHTTPServer):
    """Listens on an IPv6 address when given one, and then on IPv4 too (dual stack) where the system allows."""

    def __init__(self, address, handler):
        self.address_family = socket.AF_INET6 if ":" in address[0] else socket.AF_INET
        super().__init__(address, handler)

    def server_bind(self):
        if self.address_family == socket.AF_INET6:
            try:
                self.socket.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_V6ONLY, 0)
            except OSError:
                pass  # IPv6 only, then
        super().server_bind()


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("path", nargs="?", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "matches"), help="the matches directory (default run/matches), one batch, or one match")
    parser.add_argument("--port", type=int, default=8137, help="first port to try (default 8137)")
    parser.add_argument("--bind", default="::", help="address to listen on (default ::, every interface; 127.0.0.1 for this machine only)")
    parser.add_argument("--no-browser", action="store_true")
    args = parser.parse_args()
    root = os.path.abspath(args.path)
    if not os.path.isdir(root):
        sys.exit(f"{root} is not a directory")
    if is_match(root):
        Handler.root_kind = "match"
        Handler.match_dir = root
        page = "?match=match/"
    elif is_batch(root):
        Handler.root_kind = "batch"
        page = ""
    else:
        Handler.root_kind = "root"
        page = ""
        if not any(is_batch(os.path.join(root, f)) for f in os.listdir(root)):
            sys.exit(f"{root} holds no match: no record-*.jsonl, no NN/ match directories, no batch directories (records are written when the bot runs with WITHIN_REASON_RECORD=1; the arena sets it)")
    Handler.root = root
    for port in range(args.port, args.port + 20):
        try:
            server = Server((args.bind, port), Handler)
            break
        except OSError:
            continue
    else:
        sys.exit(f"no free port in {args.port}-{args.port + 19}")
    everywhere = args.bind in ("::", "0.0.0.0")
    local = "127.0.0.1" if everywhere else args.bind
    url = (f"http://[{local}]:{port}/" if ":" in local else f"http://{local}:{port}/") + page
    print(f"{root} ({Handler.root_kind})\n{url}   (ctrl-c to stop)")
    if everywhere:
        print(f"listening on every interface: from another machine, http://{socket.gethostname()}:{port}/{page}")
    if not args.no_browser:
        webbrowser.open(url)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
