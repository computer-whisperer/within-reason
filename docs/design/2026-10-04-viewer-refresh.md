# The viewer refresh: every seat, and the decisions as they run now (2026-10-04)

Status: built on branch `viewer-refresh`; see "Status" at the end for what is in and what is not.

The user: "It needs to handle multiplayer games better, and the introspection tools are several versions out of
date with how we are actually running various decisions now."

## 1. What is true today

- The viewer loads one seat: the first `record-*.jsonl`, that seat's truth file, one `jev-*.jsonl` and the
  `strategist-*.jsonl` transcripts. A team game writes all of these once per seat of ours (`record-0..N`,
  `jev-0..N`, `strategist-0..N` with `--players seat`, one `strategist-0` with `--players one`) and one
  `commander.jsonl`; the viewer shows the other seats only as "allies" from the first seat's samples, and never
  reads the commander's transcript.
- Every file is fetched whole as text and the record is parsed again from its first byte at every poll of a live
  match. A team game's Jev logs are 0.8 to 2.4 GB a seat (glitters-4v5-1: 4.1 GB; human-18: 5.9 GB): the page
  cannot open them. 84% of a Jev log is the `menus` of its `pass` rows (0.8 to 1.4 MB a row late in a game).
- The panels describe the hands of two rebuilds ago: `slots` and their states, the joint pick's worlds, sources
  `standing`, `policy`, `rule`, `jev`, `gate`, calls that carried `played`; the heuristic bot's waves, recalls,
  squads and `station`/`staging`/`target` intents. None of these are written any more. The layers (`news`,
  `same`, `again`, `fuse`, `openers`, `places`, `near`, `split`, `tick`), the decode, the audits, the commander's
  direction and the player's packet have no panel.
- Measured on the logs: the truth files of a match's seats are byte-identical (one ally team sees one enemy). In a
  realtime game the gate's `call`, `audit`, `worlds_gate` and `plan` rows carry the frame the answer landed, not
  the frame of the `pass` row that asked (10 to 30 frames later), and no second gate goes out while one is
  pending: a gate is the `pass` row with `gate` and every row after it up to the next such `pass` row. A record's
  `d` rows already hold every play of the hands (`source` `plan` or `list`) with its frame.
- The log does not say which layer shut a move the `near` layer shut: `log_menus` writes `fused`, `held`,
  `blanked`, `audit`, and `near` sets `fused` with its own `far`, which is not written.

## 2. The design

**The server indexes, the page asks.** `run/viewer_index.py` reads a Jev log once, line by line, without parsing
the menus (a `pass` row is parsed with its `menus` cut out), and keeps per line its byte offset, frame and kind,
and per game minute a summary: tokens by request kind (gate, pick, decode) and by question family (the families of
`run/jev_products.py`, the same function), calls, latencies, gates, quiet seconds, audits and faults by layer. The
index is cached in memory and in a sidecar `.viewer-jev-<n>.json` beside the log, and is continued from where it
stopped when the log has grown (a live match). `run/view_match.py` serves:

- `<match>/index.json`: the files with their sizes, the batch's options and the match's result line;
- `<file>?from=<offset>`: the file from a byte offset, as before;
- `jev-<n>.jsonl?index`: the summary (or `{"ready": false, "progress": ...}` while a big log is first read);
- `jev-<n>.jsonl?gate=<frame>`: the gate at or before the frame, whole: its `pass` row, the calls, audits,
  `worlds_gate` and `plan` that followed, and the decode's readings in force for it;
- `jev-<n>.jsonl?picture=<frame>`: the groups, places, parties and actors of the last call at or before the frame.

**The page holds every seat, incrementally.** Each file is a feed of complete lines; rows are parsed once and
appended to the model (`viewer/record.js`: one `seat` per record, a `transcript` per player and the commander's,
one truth). A sample's `al` rows that belong to a seat we load are dropped at parse time (they are three quarters
of a four-seat record). The match is the seats together: units carry their seat, the enemy is the truth (one
file) and what any seat sees, by unit id.

**A seat selector** (all, or one) scopes the stats, the charts, the lanes and the panels. With all seats the
charts are areas stacked by seat, so the top edge is the side's sum and each band a seat; with one seat its curve
is drawn against the others' faint lines.

**Panels by who decides**: Commander (the direction in force with each part's age, marks, signals, chat, notes),
Player (the packet in force, the lists and allowances as last given, the last turn, notes), Hands (the gate at the
playhead: layer counts, what opened it, the party and place questions, every actor's menu with each move's state
in words, the pick by component, what was played, the audits), Cost (dollars and tokens a minute by seat, kind and
family), Unit, Decisions, Log.

**A move's state** is worked out in `record.js` (`gateView`) from the gate's rows: asked and answered; asked for
an audit and not played; not asked because its actor is closed by news, because the decode fused it (with the
reading), because its opener holds it, because its place blanked it, or because an earlier answer stands (`same`
or `again`: the audit rows say which for the audited ones; for the rest the log holds only the two layers'
counts). A fused move with no decode reading under the fuse bar is said to be shut by `near` when it is a
builder's move at a spot and the layer is on, and the panel says that this is inferred.

**Rot goes.** Logs older than header `version` 3 get a one-line notice in place of the Hands panels. The census,
the heuristic bot's lanes and marks, the squad posts and the old sources are deleted (listed in the final report).

## 3. How it is judged

- `node viewer/test/smoke.js <match dir>` on the duel, the 4v5 and human-18: every seat loads, the parsers'
  invariants hold, the index and a gate come back from `run/viewer_index.py` and `gateView` accounts for every
  move of every menu.
- `curl` against the endpoints, with timings: the first index of human-18's three logs, a gate, a picture.
- `node viewer/test/browser.js` (headless Chromium) on the same matches and on a live one: no page error, the
  panels are not empty, scrubbing stays responsive (the time from a seek to the Hands panel showing its gate).
- The cost view's totals against `run/jev_products.py` on the same match.

## 4. Status

See the commit log of the branch and the final report; this section is rewritten when the branch lands.
