#!/usr/bin/env python3
"""The reader for a pianist match directory, shared by the analysis scripts (floor, raid_ledger, hands_window and the
Jev replay arms): `record-<ai>.jsonl` (header, samples, events, the hands' decisions), `jev-<ai>.jsonl` (every call),
`strategist-<ai>.jsonl` (the player's turns) and `truth-<ai>.jsonl` (what the opponent really had). Formats in
docs/harness/record-format.md.
"""
import json
import os
import re


def clock(frame):
    seconds = int(frame) // 30
    return f"{seconds // 60}:{seconds % 60:02d}"


def first(match_dir, prefix):
    name = next((f for f in sorted(os.listdir(match_dir)) if f.startswith(prefix) and f.endswith(".jsonl")), None)
    return os.path.join(match_dir, name) if name else None


def rows(path):
    if not path:
        return
    for line in open(path):
        try:
            yield json.loads(line)
        except ValueError:
            continue


class Match:
    def __init__(self, match_dir):
        if not any(f.startswith("record-") for f in os.listdir(match_dir)) and os.path.isdir(os.path.join(match_dir, "00")):
            match_dir = os.path.join(match_dir, "00")
        self.dir = match_dir
        self.label = os.path.basename(os.path.dirname(match_dir))
        self.header = None
        self.samples = []
        self.events = []
        self.decisions = {}
        self.result = None
        self.home = None
        for r in rows(first(match_dir, "record-")):
            t = r.get("t")
            if t == "header":
                self.header = r
            elif t == "s":
                self.samples.append(r)
            elif t == "ev":
                self.events.append(r)
            elif t == "d" and r.get("source") in ("jev", "list", "policy") and isinstance(r.get("inputs"), dict):
                self.decisions[(r["f"], r["inputs"].get("actor"))] = r["outputs"] or {}
            elif t == "intent" and self.home is None:
                self.home = tuple(r["home"])
            elif t == "result":
                self.result = r["result"]
        self.defs = dict(enumerate(self.header["unit_defs"]))  # event `d` fields index this list
        self.calls = []
        self.jev_header = None
        for r in rows(first(match_dir, "jev-")):
            if r.get("t") == "header":
                self.jev_header = r
            else:
                self.calls.append(r)
        self.turns = []
        turn = None
        for r in rows(first(match_dir, "strategist-")):
            kind = r.get("kind")
            if kind == "turn":
                m = re.search(r"Woken because: ([^\n]*)", r["prompt"])
                turn = {"frame": int(r["frame"]), "prompt": r["prompt"], "why": m.group(1) if m else "?", "calls": [], "text": [], "wall": None}
                self.turns.append(turn)
            elif turn is None:
                continue
            elif kind == "tool_call":
                turn["calls"].append((r["tool"], r.get("arguments") or {}))
            elif kind == "assistant":
                for c in r["message"]["message"].get("content", []):
                    if c.get("type") == "text" and c["text"].strip():
                        turn["text"].append(c["text"].strip())
            elif kind == "turn_end":
                turn["wall"] = r.get("wall_seconds")
        self.truth = {}
        for r in rows(first(match_dir, "truth-")):
            self.truth[r["f"]] = r["enemy"]

    def def_name(self, d):
        return self.defs.get(d, {}).get("name", f"def{d}")

    def is_soldier(self, d):
        u = self.defs.get(d, {})
        return u.get("class") != "building" and u.get("weapons", 0) > 0 and u.get("build_speed", 0) == 0

    def losses(self, f0, f1, near=None):
        """Metal of our units destroyed by an enemy in [f0, f1), optionally within `near` of home."""
        total = 0.0
        for e in self.events:
            if e["k"] != "destroyed" or e.get("by") is None or not (f0 <= e["f"] < f1):
                continue
            if near is not None and self.home and ((e["x"] - self.home[0]) ** 2 + (e["z"] - self.home[1]) ** 2) ** 0.5 > near:
                continue
            total += self.defs.get(e["d"], {}).get("metal", 0)
        return total

    def def_by_name(self, name):
        if not hasattr(self, "_by_name"):
            self._by_name = {d["name"]: i for i, d in self.defs.items()}
        i = self._by_name.get(name)
        return self.defs.get(i) if i is not None else None

    def is_soldier_name(self, name):
        d = self.def_by_name(name)
        return d is not None and d.get("class") != "building" and d.get("weapons", 0) > 0 and d.get("build_speed", 0) == 0

    def metal_of(self, name):
        return (self.def_by_name(name) or {}).get("metal", 0)

    def scripted(self, f, actor):
        o = self.decisions.get((f, actor))
        return bool(o and o.get("scripted"))
