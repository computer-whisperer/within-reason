#!/usr/bin/env python3
"""What Jev's meter counts: small requests that differ in one thing, and the input tokens each reports.

    run/jev_token_probe.py
"""
import json, sys, urllib.request
sys.path.insert(0, "run")
from jev_ab import api_key, URL

KEY = api_key()

def tokens(state, questions, pretty=False, raw=None):
    body = raw if raw is not None else json.dumps({"model": "jev-latest", "state": state, "questions": questions}, indent=2 if pretty else None, separators=None if pretty else (",", ":"))
    req = urllib.request.Request(URL, data=body.encode(), method="POST", headers={"Authorization": f"Bearer {KEY}", "Content-Type": "application/json", "User-Agent": "within-reason-jev-probe"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.load(r)["usage"]["input_tokens"]

Q = {"a": {"type": "noul", "instructions": "Is it day?"}}
base = tokens({"x": "y"}, Q)
def show(name, n, note=""): print(f"{name:58s} {n:6d}  ({n - base:+d}) {note}")
show("base: state {x: y}, one short question", base)
show("the same body pretty-printed with spaces", tokens({"x": "y"}, Q, pretty=True))
prose = "The quick brown fox jumps over the lazy dog near the old stone bridge. " * 40
show(f"state x = {len(prose)} chars of prose", tokens({"x": prose}, Q), f"{len(prose) / (tokens({'x': prose}, Q) - base):.2f} chars a token")
parts = [prose[i:i + 71] for i in range(0, len(prose), 71)]
show("the same prose as a list of 40 strings", tokens({"x": parts}, Q))
show("the same prose as 40 keys k0..k39", tokens({"x": {f"k{i}": p for i, p in enumerate(parts)}}, Q))
show("the same prose under 40 keys like constructor_10655", tokens({"x": {f"constructor_{10655 + i}": p for i, p in enumerate(parts)}}, Q))
ids = " ".join(f"constructor_{10655 + i * 37}" for i in range(40))
show(f"40 names constructor_NNNNN ({len(ids)} chars)", tokens({"x": ids}, Q), f"{(tokens({'x': ids}, Q) - base) / 40:.1f} tokens a name")
ids = " ".join(f"c{i}" for i in range(40))
show("40 names c0..c39", tokens({"x": ids}, Q))
ids = " ".join(f"party_{291 + i}" for i in range(40))
show("40 names party_NNN", tokens({"x": ids}, Q))
ids = " ".join(f"spot_{i + 40}" for i in range(40))
show("40 names spot_NN", tokens({"x": ids}, Q))
ids = " ".join(f"group_K1_t{i % 8}" for i in range(40))
show("40 names group_K1_tN", tokens({"x": ids}, Q))
nums = " ".join(str(1234 + i * 17) for i in range(40))
show("40 four-digit numbers", tokens({"x": nums}, Q))
nums = " ".join(f"{0.1 * i + 13.7:.1f}" for i in range(40))
show("40 numbers like 13.7", tokens({"x": nums}, Q))
many = {f"a{i}": {"type": "noul", "instructions": "Is it day?"} for i in range(40)}
show("40 short questions a0..a39", tokens({"x": "y"}, many), f"{(tokens({'x': 'y'}, many) - base) / 39:.1f} tokens a question")
many = {f"constructor_{10655 + i}.build_armguard_spot_{i}": {"type": "noul", "instructions": "Is it day?"} for i in range(40)}
show("40 short questions with ids like the bot's", tokens({"x": "y"}, many), f"{(tokens({'x': 'y'}, many) - base) / 39:.1f} tokens a question")
show("state with 40 empty keys", tokens({"x": {f"k{i}": "" for i in range(40)}}, Q))
show("state: nested 5 deep", tokens({"x": {"a": {"b": {"c": {"d": "y"}}}}}, Q))
show("backticks: 40 x `go`", tokens({"x": " ".join(["`go`"] * 40)}, Q))
show("plain: 40 x go", tokens({"x": " ".join(["go"] * 40)}, Q))
show("parentheses: 40 x (armflash)", tokens({"x": " ".join(["(armflash)"] * 40)}, Q))
show("40 x armflash", tokens({"x": " ".join(["armflash"] * 40)}, Q))
show("40 x Blitz", tokens({"x": " ".join(["Blitz"] * 40)}, Q))
show("40 x corgator", tokens({"x": " ".join(["corgator"] * 40)}, Q))
show("non-ASCII: 40 x é", tokens({"x": " ".join(["é"] * 40)}, Q))
