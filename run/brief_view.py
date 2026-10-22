#!/usr/bin/env python3
"""Show the whole brief set each model reads, as the bot assembles it, from the checkout, a git revision or a game's
snapshot, as one self-contained HTML page.

    run/brief_view.py [--rev REV | --match DIR] [--out FILE] [--open] [--annotations DIR] [--bot PATH]

One tab per role: the player on the standing brief (what `run/human_game.sh` and the arena without `--brief` run),
the player on the experience brief (`arena --brief experience`), the commander, and the hands (the standing texts Jev
reads; the per-call request is built from the game by `brain/pianist/picture.rs`). The text of each tab comes from
the bot itself, `bot --print-prompt ROLE --sources` (`crates/bot/src/strategist/mod.rs`), under the variables a game
sets, so the page shows exactly what a session gets and never re-implements the composition. Each tab shows the
composition (which file, in which order, how many words, when last committed, whether the working tree has changed
it), then the text with a band per source file, a table of contents from the headings, and under every heading the
provenance found in that section: the games it cites, the dates, the claims, the opponents, the user's rulings. The
search box lists the sections that match a regular expression and marks the matches.

Without `--rev` or `--match` the working tree is shown with `target/release/bot` (build first). `--rev REV` extracts
the revision's texts with `git archive` and runs the current binary over them (the composition of now over the texts
of then; a revision from before the manifests shows the role prompts and says the brief could not be assembled).
`--match DIR` (a batch directory under run/matches, or a match directory in one) reads the batch's `snapshot/` (`bot
--snapshot`: the texts a game was played with, with the checkout's git head, status and diff beside them), with the
snapshot's own binary when it can print prompts, else the current one over the snapshot's texts; the page says the
head and how many files were modified at launch.

`--annotations DIR` (the default is docs/studies/2026-10-21-brief-annotation when it holds part-*.jsonl) overlays the
reading pass's rows (that directory's README has the schema): under the first line of each annotated passage a chip
with its fate, layer, derivability and era, the passage's lines marked in the fate's colour, a fate filter in the
header that dims everything else, and an Annotations tab with every row and a text filter. A row is placed by its
file and line range, mapped onto the text the model reads through a line diff, so passages dropped by the role text's
brief markers or stripped comments still land where they stand. The page is written to
`run/data/brief-view/<rev>.html` unless `--out` says where, and `--open` opens it in the browser.
"""
import difflib
import glob
import html
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import webbrowser

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(REPO)

# The directories `git archive` extracts for a revision: everything `crates/bot/src/texts.rs` reads.
TEXT_DIRS = ["docs/briefs", "crates/bot/src/strategist", "crates/bot/src/brain/pianist"]

# The tabs: the role the bot prints, and the variables a game of that kind sets.
ROLES = [
    {"id": "player-standard", "name": "Player, standing brief", "role": "player",
     "env": {"WITHIN_REASON_COMMANDER": "claude-opus-5-5", "WITHIN_REASON_MODEL": "claude-opus-5-5"},
     "launch": "run/human_game.sh; arena without --brief (WITHIN_REASON_BRIEF unset); shown with a commander above and an Opus model"},
    {"id": "player-experience", "name": "Player, experience brief", "role": "player",
     "env": {"WITHIN_REASON_BRIEF": "experience", "WITHIN_REASON_COMMANDER": "claude-opus-5-5", "WITHIN_REASON_MODEL": "claude-opus-5-5"},
     "launch": "arena --brief experience (WITHIN_REASON_BRIEF=experience); shown with a commander above and an Opus model"},
    {"id": "commander", "name": "Commander", "role": "commander", "env": {},
     "launch": "any game with WITHIN_REASON_COMMANDER set: the role text, then the experience brief whole"},
    {"id": "hands", "name": "Hands (Jev)", "role": "hands", "env": {},
     "launch": "every pick; the request itself is built from the game by brain/pianist/picture.rs"},
]
GAME_TIME = ["the game's bonus paragraph (from the lobby's income bonuses; none when no seat has one)", "the objective (WITHIN_REASON_OBJECTIVE)",
             "then the game's own lines: the first report with the map sheet"]

# What a section cites. Each pattern names a kind of provenance; the chips under a heading count the distinct matches.
PROVENANCE = [
    ("game", r"\b(?:player|human|onepass-player|bluegecko-3v1(?:-comet-catcher)?|bluegecko-2v1-[a-z-]+|escalate|fable|sonnet5-[a-z]+|opus55-[a-z]+|cmd-opus-[a-z]+|worlds|nouls|standing|rove-task|chase-player|predict-player|spending-player|prio-player|jev114-player|leg-player|apex[0-9a-z-]*|nuke-\d-[a-z]+|micro-ab\d?|matt-fan\d?|late|gecko-3v3)-\d+\b"),
    ("people's game", r"Center Command|Altored|human-8v8-\d|Great Divide|Cape Violet|Coast To Coast|Gasbag Grabens|SD-129|Glitters|replay-[a-z0-9-]+"),
    ("date", r"\b2026-\d\d-\d\d\b"),
    ("the user", r"\bthe user\b"),
    ("claim", r"\bK-[a-z0-9-]+"),
    ("person", r"\b(?:thebluegecko|u6bkep|Irishstud14|Covariance|computer_whisperer)\b"),
    ("BARb", r"\bBARb\b|\bhard_aggressive\b|\bmedium\b|\beasy\b"),
    ("Apex", r"\bApex\b"),
    ("pool", r"\bpool\b|\bOS \d\d\b|\bpros?\b"),
    ("never/always/must", r"\b(?:never|always|must)\b"),
]
FATES = ["fact", "meta", "rederive", "episode", "tool", "discard"]
FATE_COLOR = {"fact": "#2f6fd6", "meta": "#2e9e5b", "rederive": "#e08a1e", "episode": "#8a4fd6", "tool": "#888", "discard": "#d63a2f"}
PART_RE = re.compile(r"<<<source: (.*?)>>>\n(.*?)\n<<<end>>>\n", re.S)


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True)


def load_annotations(directory):
    """The reading pass's rows by file, with the lines that failed to parse counted."""
    rows, bad = {}, 0
    for path in sorted(glob.glob(os.path.join(directory, "part-*.jsonl"))):
        for line in open(path):
            if not line.strip():
                continue
            try:
                row = json.loads(line)
                row["_part"] = os.path.basename(path)
                rows.setdefault(row["file"], []).append(row)
            except (ValueError, KeyError):
                bad += 1
    for lst in rows.values():
        lst.sort(key=lambda r: (r.get("lines") or [0])[0])
    return rows, bad


def line_map(source_text, band_text):
    """For each line of the band, the 1-based line of the source file it came from (a diff, so dropped passages and
    stripped comments do not shift the rest); None where a line has no counterpart."""
    a, b = (source_text or "").split("\n"), band_text.split("\n")
    out = [None] * len(b)
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes():
        if tag == "equal":
            for k in range(j2 - j1):
                out[j1 + k] = i1 + k + 1
        elif tag == "replace":
            for k in range(j2 - j1):
                out[j1 + k] = min(i1 + k, i2 - 1) + 1
    return out


class Source:
    """A text file as the page's root has it: its contents, its last commit, its working-tree state."""

    def __init__(self, path, rev=None, root=None, root_name="the game's snapshot"):
        self.path = path
        self.text, self.state, self.last_commit = None, "absent", ""
        if root is not None:
            full = os.path.join(root, path)
            if os.path.exists(full):
                self.text, self.state = open(full).read(), f"in {root_name}"
            return
        if rev is None:
            if os.path.exists(path):
                self.text = open(path).read()
                status = git("status", "--porcelain", "--", path).stdout.strip()
                self.state = "untracked" if status.startswith("??") else "modified in the working tree" if status else "committed"
        else:
            r = git("show", f"{rev}:{path}")
            if r.returncode == 0:
                self.text, self.state = r.stdout, f"at {rev}"
        log = git("log", "-1", "--format=%h %ad", "--date=short", *([rev] if rev else []), "--", path).stdout.strip()
        self.last_commit = log or "(never committed)"


def print_prompt(binary, role, env, texts_root):
    """The role's prompt from the bot, as parts (source, text); or the error the bot gave."""
    environment = {k: v for k, v in os.environ.items() if not k.startswith("WITHIN_REASON_")}
    environment.update(env)
    if texts_root:
        environment["WITHIN_REASON_TEXTS"] = texts_root
    r = subprocess.run([binary, "--print-prompt", role, "--sources"], capture_output=True, text=True, env=environment)
    if r.returncode != 0:
        return None, (r.stderr.strip().split("\n")[-1] if r.stderr.strip() else f"exit {r.returncode}")
    return [(m.group(1), m.group(2)) for m in PART_RE.finditer(r.stdout)], None


def provenance(text):
    out = []
    for kind, pat in PROVENANCE:
        found = sorted(set(m.group(0) for m in re.finditer(pat, text)))
        if found:
            out.append((kind, len(re.findall(pat, text)), found))
    return out


def render_role(role, palette, sources, ann):
    """The role's text as lines, headings as anchors, a band per source, provenance chips under each heading, the
    reading pass's chips under each annotated passage."""
    lines_html, toc = [], []
    sec_no = 0
    if role.get("error"):
        lines_html.append(f'<p class="stale">The bot could not print this prompt: {html.escape(role["error"])}</p>')
    for label, text in role["bands"]:
        color = palette.setdefault(label, f"hsl({(len(palette) * 47) % 360},55%,45%)")
        lines_html.append(f'<div class="band" style="border-color:{color}"><div class="bandlabel" style="background:{color}">{html.escape(label)}</div>')
        if not text.strip():
            lines_html.append('<div class="l empty">(empty)</div></div>')
            continue
        src = sources.get(label)
        rows = ann.get(label, []) if src and src.text is not None else []
        lmap = line_map(src.text, text) if rows else []
        by_line = {}
        for row in rows:
            span = row.get("lines") or [0]
            lo, hi = (span[0], span[1]) if len(span) >= 2 else (span[0], span[0])
            first = True
            for j, fl in enumerate(lmap):
                if fl is not None and lo <= fl <= hi:
                    by_line.setdefault(j, []).append((row, first))
                    first = False
        lines = text.split("\n")
        heading = re.compile(r"(#{1,6}) (.*)")
        starts = [0] + [i for i, l in enumerate(lines) if i > 0 and heading.match(l)]
        ends = starts[1:] + [len(lines)]
        for start, end in zip(starts, ends):
            m = heading.match(lines[start])
            sec_no += 1
            sid = f"{role['id']}-s{sec_no}"
            if m:
                level, title = len(m.group(1)), m.group(2).strip()
                toc.append((level, title, sid, color))
                lines_html.append(f'<h{min(level + 1, 6)} class="hd" id="{sid}" style="color:{color}">{html.escape(title)}</h{min(level + 1, 6)}>')
            else:
                lines_html.append(f'<div id="{sid}" class="anon"></div>')
            prov = provenance("\n".join(lines[start:end]))
            if prov:
                chips = " ".join(f'<span class="chip" title="{html.escape(", ".join(f[:40]))}">{html.escape(k)} {n}</span>' for k, n, f in prov)
                lines_html.append(f'<div class="prov">{chips}</div>')
            lines_html.append(f'<div class="sec" data-sid="{sid}">')
            for idx in range(start, end):
                line = lines[idx]
                marks = by_line.get(idx, [])
                for row, first in marks:
                    if first:
                        fate = row.get("fate", "")
                        chip = (f'{fate} · {row.get("layer", "")} · derivable {row.get("derivable", "")} · {", ".join(row.get("era") or [])}'
                                f' · {row.get("subject", "")}' + (" · conflicts " + str(len(row["conflicts"])) if row.get("conflicts") else ""))
                        lines_html.append(f'<div class="ann" data-fate="{html.escape(fate)}" style="border-color:{FATE_COLOR.get(fate, "#999")}" '
                                          f'title="{html.escape(row.get("note", ""))}">{html.escape(chip)}</div>')
                if m and idx == start:
                    continue  # the heading line is the h element above
                fates = " ".join("a-" + html.escape(r.get("fate", "")) for r, _ in marks)
                style = f' style="border-left-color:{FATE_COLOR.get(marks[0][0].get("fate", ""), "#999")}"' if marks else ""
                lines_html.append(f'<div class="l {fates}"{style}>{html.escape(line) or "&nbsp;"}</div>')
            lines_html.append("</div>")
        lines_html.append("</div>")
    return "\n".join(lines_html), toc


def composition_table(role, sources, palette):
    rows = []
    for label, text in role["bands"]:
        src = sources.get(label)
        words = len(text.split())
        color = palette.get(label, "#888")
        rows.append(f'<tr><td><span class="sw" style="background:{color}"></span>{html.escape(label)}</td><td class="n">{words:,}</td><td class="n">{int(words * 1.3):,}</td>'
                    f'<td>{html.escape(src.last_commit if src else "")}</td><td>{html.escape(src.state if src else "")}</td></tr>')
    total = sum(len(text.split()) for _, text in role["bands"])
    rows.append(f'<tr class="tot"><td>whole</td><td class="n">{total:,}</td><td class="n">{int(total * 1.3):,}</td><td></td><td></td></tr>')
    game_time = "".join(f"<li>{html.escape(p)}</li>" for p in GAME_TIME) if role["role"] != "hands" else "<li>the picture: groups, places, parties, actors, the packet's rules per actor</li>"
    return (f'<table class="comp"><tr><th>source</th><th>words</th><th>≈ tokens</th><th>last commit</th><th>state</th></tr>{"".join(rows)}</table>'
            f'<p class="launch">Runs as: {html.escape(role["launch"])}</p><p>Game-time parts, empty here:</p><ul>{game_time}</ul>')


CSS = """
body{margin:0;font:14px/1.45 system-ui,sans-serif;color:#222;background:#fafafa}
header{position:sticky;top:0;background:#fff;border-bottom:1px solid #ddd;padding:8px 16px;z-index:5;display:flex;gap:12px;align-items:center;flex-wrap:wrap}
header h1{font-size:16px;margin:0 12px 0 0}
.tab{padding:4px 10px;border:1px solid #ccc;border-radius:4px;background:#f4f4f4;cursor:pointer}
.tab.on{background:#333;color:#fff;border-color:#333}
#q{padding:4px 8px;width:260px;border:1px solid #bbb;border-radius:4px}
#hits{color:#666;font-size:12px}
main{display:grid;grid-template-columns:320px 1fr;gap:0}
nav{position:sticky;top:46px;height:calc(100vh - 46px);overflow:auto;border-right:1px solid #ddd;padding:8px 12px;background:#fff;font-size:12px}
nav a{display:block;color:#333;text-decoration:none;padding:1px 0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
nav a.l1{font-weight:700;margin-top:6px}nav a.l2{padding-left:10px}nav a.l3{padding-left:20px}nav a.l4{padding-left:30px}
nav a.hit{background:#fff3b0}
.role{display:none;padding:12px 24px;max-width:1100px}.role.on{display:block}
.comp{border-collapse:collapse;font-size:12px;margin:8px 0}.comp td,.comp th{border:1px solid #ddd;padding:3px 8px;text-align:left;vertical-align:top}.comp .n{text-align:right;font-variant-numeric:tabular-nums}.comp .tot td{font-weight:700}
.sw{display:inline-block;width:10px;height:10px;border-radius:2px;margin-right:6px;vertical-align:middle}
.launch{color:#555}.note{color:#666;font-weight:400;font-size:12px}.stale{color:#b00}
.band{border-left:4px solid;margin:14px 0;padding-left:10px}.bandlabel{color:#fff;display:inline-block;padding:1px 8px;border-radius:0 4px 4px 0;font-size:12px;margin-left:-10px;margin-bottom:6px}
.hd{margin:14px 0 2px;font-size:15px}
.l{white-space:pre-wrap;font-family:ui-monospace,Menlo,Consolas,monospace;font-size:12.5px;line-height:1.45;border-left:3px solid transparent;margin-left:-6px;padding-left:3px}.l.empty{color:#999}
.prov{margin:0 0 4px}.chip{display:inline-block;font-size:11px;background:#eee;border:1px solid #ddd;border-radius:10px;padding:0 7px;margin:1px 2px 1px 0;color:#444}
.sec.hit{background:#fffbe6}mark{background:#ffd54a;padding:0}
.ann{font:11px system-ui,sans-serif;color:#333;background:#f3f3f3;border-left:3px solid;margin:6px 0 2px -6px;padding:1px 6px;border-radius:0 3px 3px 0}
body.dim .l:not(.a-F),body.dim .ann:not(.a-F){opacity:.25}
#fate{padding:4px 6px;border:1px solid #bbb;border-radius:4px}
.anntab{border-collapse:collapse;font-size:12px}.anntab td,.anntab th{border:1px solid #ddd;padding:2px 6px;vertical-align:top}.anntab td.n{text-align:right;white-space:nowrap}
#af{padding:4px 8px;width:320px;border:1px solid #bbb;border-radius:4px;margin:6px 0}
"""

JS = """
const tabs=document.querySelectorAll('.tab'),roles=document.querySelectorAll('.role'),navs=document.querySelectorAll('nav .toc');
function show(id){tabs.forEach(t=>t.classList.toggle('on',t.dataset.id===id));roles.forEach(r=>r.classList.toggle('on',r.id===id));navs.forEach(n=>n.style.display=n.dataset.id===id?'block':'none');location.hash='';search();}
tabs.forEach(t=>t.onclick=()=>show(t.dataset.id));
const q=document.getElementById('q'),hits=document.getElementById('hits');
const esc=s=>s.replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));
function search(){const role=document.querySelector('.role.on');let re=null;try{re=q.value?new RegExp(q.value,'gi'):null}catch(e){hits.textContent='bad regexp';return}
 let n=0,secs=0;role.querySelectorAll('.sec').forEach(sec=>{let m=0;sec.querySelectorAll('.l').forEach(l=>{const t=l.textContent;if(re){const c=(t.match(re)||[]).length;m+=c;l.innerHTML=c?esc(t).replace(re,s=>'<mark>'+s+'</mark>'):esc(t)}else{l.textContent=t}});
  sec.classList.toggle('hit',m>0);n+=m;if(m)secs++;const a=document.querySelector('nav a[href="#'+sec.dataset.sid+'"]');if(a)a.classList.toggle('hit',m>0)});
 hits.textContent=re?n+' matches in '+secs+' sections':''}
q.oninput=search;
const fateSel=document.getElementById('fate');
if(fateSel){fateSel.onchange=()=>{const f=fateSel.value;document.body.classList.toggle('dim',!!f);document.querySelectorAll('.l').forEach(l=>l.classList.toggle('a-F',f&&l.classList.contains('a-'+f)));document.querySelectorAll('.ann').forEach(a=>a.classList.toggle('a-F',f&&a.dataset.fate===f))}}
const af=document.getElementById('af');if(af){af.oninput=()=>{const t=af.value.toLowerCase();document.querySelectorAll('.anntab tbody tr').forEach(tr=>tr.style.display=!t||tr.textContent.toLowerCase().includes(t)?'':'none')}}
show(tabs[0].dataset.id);
"""


def supports_print_prompt(binary):
    try:
        r = subprocess.run([binary, "--print-prompt", "hands"], capture_output=True, text=True, timeout=60)
        return r.returncode == 0 and "<<<" not in r.stdout
    except (OSError, subprocess.TimeoutExpired):
        return False


def main():
    args = sys.argv[1:]
    rev = out = match_dir = bot = None
    open_it = False
    ann_dir = "docs/studies/2026-10-21-brief-annotation"
    i = 0
    while i < len(args):
        if args[i] == "--rev":
            rev = args[i + 1]; i += 2
        elif args[i] == "--out":
            out = args[i + 1]; i += 2
        elif args[i] == "--open":
            open_it = True; i += 1
        elif args[i] == "--annotations":
            ann_dir = args[i + 1]; i += 2
        elif args[i] == "--match":
            match_dir = args[i + 1]; i += 2
        elif args[i] == "--bot":
            bot = args[i + 1]; i += 2
        elif args[i] in ("-h", "--help"):
            print(__doc__); sys.exit(0)
        else:
            print(__doc__); sys.exit(2)
    # The root the texts come from, and the binary that assembles them.
    texts_root = root_name = snap_info = None
    temp = None
    label = "working-tree"
    if match_dir:
        for candidate in (match_dir, os.path.dirname(match_dir.rstrip("/"))):
            if os.path.isdir(os.path.join(candidate, "snapshot")):
                texts_root = os.path.join(candidate, "snapshot")
                break
        if texts_root is None:
            print(f"{match_dir}: no snapshot/ in it or its batch (games before 2026-10-21 have none)"); sys.exit(1)
        root_name = "the game's snapshot"
        label = os.path.basename(os.path.dirname(texts_root.rstrip("/"))) + "-snapshot"
        try:
            snap_info = json.load(open(os.path.join(texts_root, "snapshot.json")))
        except (OSError, ValueError):
            snap_info = {}
        own = os.path.join(texts_root, "bot")
        if bot is None and os.path.isfile(own) and supports_print_prompt(own):
            bot = own
    elif rev:
        label = git("rev-parse", "--short", rev).stdout.strip() or rev
        temp = tempfile.mkdtemp(prefix="brief-view-")
        archive = subprocess.run(["git", "archive", "--format=tar", rev, *TEXT_DIRS], capture_output=True)
        if archive.returncode != 0:
            print(f"git archive {rev}: {archive.stderr.decode().strip()}"); sys.exit(1)
        subprocess.run(["tar", "-x", "-C", temp], input=archive.stdout, check=True)
        texts_root, root_name = temp, f"revision {label}"
    if bot is None:
        bot = "target/release/bot"
    if not os.path.isfile(bot):
        print(f"{bot}: no bot binary (cargo build --release)"); sys.exit(1)
    head = git("rev-parse", "--short", "HEAD").stdout.strip()
    ann, bad_rows = load_annotations(ann_dir) if os.path.isdir(ann_dir) else ({}, 0)
    # The roles, from the bot.
    role_list, sources, palette = [], {}, {}

    def source(path):
        return Source(path, root=texts_root, root_name=root_name) if texts_root else Source(path)
    for role in ROLES:
        parts, error = print_prompt(bot, role["role"], role["env"], texts_root)
        role_list.append({**role, "bands": parts or [], "error": error})
        for path, _ in parts or []:
            if path not in sources and not path.startswith("("):
                sources[path] = source(path)
    for manifest in ("docs/briefs/standard.manifest", "docs/briefs/experience.manifest"):
        if manifest not in sources:
            sources[manifest] = source(manifest)
    tabs = "".join(f'<span class="tab" data-id="{r["id"]}">{html.escape(r["name"])}</span>' for r in role_list)
    bodies, navs = [], []
    for r in role_list:
        text_html, toc = render_role(r, palette, sources, ann)
        comp = composition_table(r, sources, palette)
        bodies.append(f'<div class="role" id="{r["id"]}"><h2>{html.escape(r["name"])}</h2>{comp}{text_html}</div>')
        links = "".join(f'<a class="l{min(lv, 4)}" href="#{sid}" style="border-left:3px solid {c};padding-left:{4 + (lv - 1) * 10}px">{html.escape(t)}</a>' for lv, t, sid, c in toc)
        navs.append(f'<div class="toc" data-id="{r["id"]}">{links}</div>')
    ann_rows = [row for lst in ann.values() for row in lst]
    fate_opts = "".join(f'<option value="{f}">{f}</option>' for f in FATES)
    fate_sel = f'<select id="fate"><option value="">every fate</option>{fate_opts}</select>' if ann_rows else ""

    def cell(v):
        if isinstance(v, list):
            return html.escape("-".join(str(x) for x in v) if v and all(isinstance(x, int) for x in v) else ", ".join(str(x) for x in v))
        return html.escape(json.dumps(v) if isinstance(v, dict) else str(v or ""))
    ann_table = ""
    if ann_rows:
        cols = ["file", "lines", "subject", "layer", "fate", "derivable", "doctrine", "era", "grounding", "defects", "lens", "conflicts", "duplicates", "confidence", "note"]
        trs = "".join("<tr>" + "".join(f'<td class="{"n" if c == "lines" else ""}">{cell(r.get(c))}</td>' for c in cols) + "</tr>" for r in ann_rows)
        counts = {}
        for r in ann_rows:
            counts[r.get("fate", "?")] = counts.get(r.get("fate", "?"), 0) + 1
        ann_table = (f'<div class="role" id="annotations"><h2>The reading pass: {len(ann_rows):,} rows from {html.escape(ann_dir)}'
                     f'{f" ({bad_rows} lines failed to parse)" if bad_rows else ""}</h2><p>' + ", ".join(f"{k} {v}" for k, v in sorted(counts.items())) +
                     f'</p><input id="af" placeholder="filter rows (plain text)"><table class="anntab"><thead><tr>{"".join(f"<th>{c}</th>" for c in cols)}</tr></thead><tbody>{trs}</tbody></table></div>')
    files = "".join(f'<tr><td>{html.escape(p)}</td><td class="n">{len((s.text or "").split()):,}</td><td>{html.escape(s.last_commit)}</td><td>{html.escape(s.state)}</td></tr>' for p, s in sorted(sources.items()))
    where = f" (HEAD {html.escape(head)})" if not texts_root else ""
    if snap_info is not None:
        where = f" (the game's snapshot: head {html.escape(str(snap_info.get('head', '?'))[:9])}, {snap_info.get('dirty_files', '?')} files modified at launch)"
    page = f"""<!doctype html><html><head><meta charset="utf-8"><title>Brief set {html.escape(label)}</title><style>{CSS}</style></head><body>
<header><h1>The brief set at {html.escape(label)}{where}</h1>{tabs}<span class="tab" data-id="files">Files</span>{'<span class="tab" data-id="annotations">Annotations</span>' if ann_rows else ""}
{fate_sel}<input id="q" placeholder="search (regexp), e.g. medium|BARb"><span id="hits"></span></header>
<main><nav>{"".join(navs)}<div class="toc" data-id="files"></div><div class="toc" data-id="annotations"></div></nav><div>{"".join(bodies)}{ann_table}
<div class="role" id="files"><h2>Every text, at {html.escape(label)}</h2><table class="comp"><tr><th>file</th><th>words</th><th>last commit</th><th>state</th></tr>{files}</table>
<p>Tokens are estimated as words × 1.3. The text of each role is what <code>{html.escape(bot)} --print-prompt</code> printed{" over the texts of " + html.escape(root_name) if texts_root else ""}.</p></div></div></main>
<script>{JS}</script></body></html>"""
    if out is None:
        os.makedirs("run/data/brief-view", exist_ok=True)
        out = f"run/data/brief-view/{label}.html"
    open(out, "w").write(page)
    total = {r["name"]: sum(len(text.split()) for _, text in r["bands"]) for r in role_list}
    print(f"{out}: " + ", ".join(f"{k} {v:,} words" for k, v in total.items()))
    for r in role_list:
        if r["error"]:
            print(f"  {r['name']}: the bot could not print this prompt: {r['error']}")
    if temp:
        shutil.rmtree(temp, ignore_errors=True)
    if open_it:
        webbrowser.open("file://" + os.path.abspath(out))


if __name__ == "__main__":
    main()
