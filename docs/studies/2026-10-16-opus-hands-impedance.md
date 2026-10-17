# Opus to the hands: how often the harness did not follow the player, and what the player was not told (2026-10-16)

The user, 2026-10-16: "I think first thing to focus on is overall opus to jev impedance matching -- how many times in
the last 20-ish games did the harness not follow what opus wanted, or was opus missing a key piece of game context
that should have been available to it? We can probably fork off a number of opus agents to chase down and tally up
that information."

Twenty-three games (2026-10-06 to 2026-10-16, every recorded game with a player transcript since the rove arc), one
Opus 5.5 reviewer each under one brief, one tally shape, verified on a random sample. Nothing was changed by this
study; the fixes it points at are the user's calls. Data: `data/impedance-2026-10-16/` (every game's `impedance.json`
and `impedance-review.md`, the sweep's counts, the decode scan, the brief, `tally.json`).

## 1. The count

| | |
|---|---|
| games | 23: 12 arena duels (BARb, Apex, the two nuke games, Legion, Jev 1.14), 2 realtime 8v8s, 9 games with people (1 to 4 seats) |
| transcripts | 63: 48 player seats (Opus 5.5 medium in 40, Sonnet 5.5 medium in 8), 15 commanders (Opus 5.5 high) |
| turns / orders | 5,434 / 10,877 (`run/impedance_sweep.py --counts`, every seat) |
| refused at the tool | 101 |
| hands' report lines ("what your hands did") | 37,258, of which 3,812 say an order did not happen as written |
| player sessions replaced mid-game | 109 |
| judgement moments examined | 209 (8 a seat in a duel, 12 a game across seats otherwise) |

**Items: 515.** A, the harness did not do what the player wanted: **412**. B, the player lacked context that was
available: **103**. C, the moment followed cleanly with context enough: **51 of 209 (24%)**.

| A: not followed | n | | B: context missing | n |
|---|---|---|---|---|
| A1 refused at the tool | 65 | | B1 in the hands' picture, not in the report | 19 |
| A2 accepted, not executed | 74 | | B2 in the record, in nobody's picture | 5 |
| A3 overruled by Jev's pick | 113 | | B3 in the report, wrong or stale | 15 |
| A4 overruled by a code rule or layer | 72 | | B4 in the report, missed by the player | 25 |
| A5 landed late | 9 | | B5 the hands' own lines misleading | 11 |
| A6 read differently from what was meant | 79 | | B6 lost on the commander-to-player hop | 28 |

Cost, as the reviewers measured it: 3 items plausibly decided a game (two in nuke-1-medium and nuke-2-hard, one
the missiles the report never named), 63 large (a raid unanswered, a group lost, a window spent), 282 small, 167
none. Where the fix would go, the reviewers' one-word guess: code-rule 131, tool 120, jev 94, report 88, words 27,
menu 25, brief 16, unknown 14.

Per game (`run/impedance_tally.py docs/studies/data/impedance-2026-10-16/17*`):

| game | result | A | B | clean / moments | orders | per category |
|---|---|---|---|---|---|---|
| human-26 (2 seats + commander) | ended 10:07, t2 out | 12 | 9 | 0/8 | 210 | A2 1 A3 5 A4 1 A5 1 A6 4 B1 2 B2 1 B3 1 B5 1 B6 4 |
| rove-move-1 | won 23:52 | 16 | 0 | 1/8 | 262 | A2 2 A3 7 A4 2 A6 5 |
| human-27 (+ commander) | lost 13:48 | 19 | 2 | 3/8 | 142 | A1 2 A2 3 A3 5 A4 6 A6 3 B5 1 B6 1 |
| human-28 (+ commander) | lost 15:24 | 12 | 3 | 2/8 | 155 | A2 2 A3 5 A4 1 A6 4 B4 1 B6 2 |
| rove-task-1 | lost 32:28 | 23 | 1 | 2/8 | 382 | A1 3 A2 2 A3 6 A4 6 A5 2 A6 4 B3 1 |
| rove-task-2 | won 21:11 | 18 | 3 | 2/8 | 236 | A1 1 A2 3 A3 6 A4 5 A6 3 B4 2 B5 1 |
| nuke-1-medium | undecided 40:00 | 25 | 6 | 4/9 | 439 | A1 7 A2 4 A3 7 A4 3 A6 4 B1 2 B3 2 B4 1 B5 1 |
| nuke-2-hard | lost 32:22 | 24 | 2 | 4/8 | 353 | A1 2 A2 10 A3 6 A4 4 A6 2 B1 1 B4 1 |
| human-29 (+ commander) | won 18:03 | 16 | 3 | 2/8 | 172 | A1 2 A2 2 A3 6 A4 1 A5 1 A6 4 B3 1 B6 2 |
| human-30 Comet (+ commander) | won 19:37 | 16 | 4 | 3/8 | 177 | A1 4 A2 2 A3 3 A4 4 A6 3 B4 1 B5 1 B6 2 |
| glitters-8v8-late-1 (8 Sonnet seats + commander) | lost 28:30 | 23 | 9 | 3/12 | 2,144 | A1 17 A2 4 A3 2 B1 1 B3 2 B4 4 B6 2 |
| glitters-8v8-opus-1 (8 seats + commander) | undecided 30:00 | 18 | 8 | 5/12 | 1,768 | A1 8 A2 1 A3 3 A4 1 A5 2 A6 3 B1 6 B2 1 B4 1 |
| human-30 Eclipsed (4 seats + commander) | lost about 21:45 | 15 | 8 | 3/12 | 672 | A1 2 A2 4 A3 6 A4 1 A6 2 B1 4 B4 1 B5 1 B6 2 |
| human-31 (3 seats + commander) | lost 19:50 | 13 | 3 | 4/12 | 588 | A1 3 A2 3 A3 2 A4 1 A5 1 A6 3 B6 3 |
| human-32 (3 seats + commander) | lost 27:20 | 14 | 3 | 1/12 | 795 | A1 2 A2 2 A3 7 A4 1 A6 2 B3 1 B6 2 |
| human-33 (4 seats + commander) | lost 22:45 | 23 | 5 | 2/12 | 863 | A1 6 A2 5 A3 1 A4 5 A5 1 A6 5 B3 2 B5 2 B6 1 |
| apex-v022-carrot-sym50-opus-1 (+ commander) | won 22:17 | 30 | 5 | 0/8 | 300 | A1 3 A2 7 A3 11 A4 7 A6 2 B4 3 B5 1 B6 1 |
| apex-v022-carrot-sym50-opus-2 (+ commander) | won 18:10 | 20 | 1 | 3/8 | 229 | A2 3 A3 4 A4 7 A6 6 B5 1 |
| apex-v022-carrot-sym0-opus-1 (+ commander) | won 10:57 | 13 | 6 | 1/8 | 104 | A2 1 A3 4 A4 2 A5 1 A6 5 B2 1 B4 1 B6 4 |
| prio-player-1 | won 18:02 | 14 | 5 | 2/8 | 192 | A1 1 A2 4 A3 4 A4 1 A6 4 B1 1 B2 1 B4 3 |
| leg-player-1 | won 25:54 | 24 | 8 | 0/8 | 295 | A2 5 A3 9 A4 6 A6 4 B1 1 B3 2 B4 5 |
| apex-v024-carrot-sym50-opus-1 (+ commander) | won 16:40 | 16 | 5 | 1/8 | 206 | A1 2 A2 3 A3 3 A4 3 A6 5 B3 1 B4 1 B5 1 B6 2 |
| jev114-player-1 | won 19:09 | 8 | 4 | 3/8 | 193 | A2 1 A3 1 A4 4 A6 2 B1 1 B2 1 B3 2 |

Wins and losses look alike in the count: the harness let the player down about as often in the games we won. The
items are the shape of the interface, not the cause of a result.

## 2. Method

- **The brief** (`data/impedance-2026-10-16/BRIEF.md`): the twelve categories above, the evidence standard (a clock,
  a seat, the player's words quoted, the harness's act quoted; "not followed" needs the order accepted and something
  else done; a pick against the packet needs the pick line and the decode's reading of the packet, which settles A3
  against A6), the cost scale, read-only rules, the tools.
- **The mechanical sweep**, `run/impedance_sweep.py` (new): every turn's orders in compact form beside the next
  report's "what your hands did" lines that say an order did not happen (the engine refused a site, a blocked site,
  a skipped step, a hunt ended on its leash or with no hunters, a frame abandoned, a follow ended), every refused
  tool result, the landing delay, and the counts per seat. Every seat of every game was swept; every refusal is an
  A1 item.
- **Judgement moments**, a fixed sample: the opening against what stood by 4:00; the raids in `run/raid_ledger.py`
  (up to four); the first 1.5x window and the first push; two mid-game packet changes against the hands' plays in
  the next 90 s and the `decode` row of the packet; the last turns against the truth. At each: what the player
  wanted, what the hands did (`run/hands_window.py`), what the player knew against the truth file or the record.
- **The reviewers**: 23 Opus 5.5 agents, one a game, 20 to 35 minutes and 340k to 470k tokens each (about 9.3M
  tokens in all), writing `impedance.json` and `impedance-review.md` into the match directory.
- **Verification by the main session.** A random sample of ten items (seed 20261016; A2/A3/A4/A6/B1/B3/B5 with a
  cost) checked against the hands' log and the transcripts: 10 of 10 held. Fourteen targeted checks of the
  reviewers' mechanisms, all in the raw data or the source: the boxed commander (232 failed moves from 6:18,
  jev114-player-1), the metal store's "full" in the picture and absent from the report (two games), the forbidden
  reading at 0.69 against the 0.70 bar (rove-task-2 15:50, `decode.rs`), `signal` outside `batchable()` (`mcp.rs`),
  the side's mark surviving the player's removal (apex-v024 10:27 to 10:48), the extractor readings falling under
  the 0.3 fuse on a reworded paragraph (prio-player-1 10:45; `FUSE = 0.3` in `decode.rs`), the dead labs listed as
  standing in sight (nuke-1-medium 28:09 to 29:05 against the truth's last `corlab` at 26:20), a negated place read
  as a destination (human-31 12:27, spot_8 at 0.59), the commander rule's missing exception (`rules.md`), the
  `no_site` events (11) with no report line, and the corpus counts in §3. One loose attribution found: jev114's
  "group_K detaching 4 or 8" was several small groups' detachments, the substance (six detachments after one Pawn
  against a packet that said one or two) right.

## 3. The mechanisms, by where the fix would go

The reviewers' 515 items cluster into about thirty recurring shapes. Counts labelled *corpus* are the main
session's own scans of all 23 games' transcripts or logs (complete); counts labelled *items* are the reviewers'
items naming the shape (a sample of the moments, so a lower bound). Verified means the main session saw it in the
raw data or the source.

### 3.1 The packet weights the pick; it does not command (A3, 113 items)

- **The mark landed, the pick kept its course.** The decode read the packet's place or instruction at 0.9 or over
  and the pick still kept the old course, because the move's own rating sat under the bar or the actor's `change`
  question stayed under 0.5 and the `news` layer kept it closed. 14 items naming the landed mark, 39 naming the
  change gate, 16 games; verified (rove-task-2 12:16, leg-player-1 17:24-18:27 "fight to spot_18" read at 0.98 and
  kept on spot_27 for 63 s, 9 Decurions; apex-v024 11:34). Three players wrote the wish into their notes: "a hard
  move order for a group that bypasses Jev's rating", "a way to force a group's course to end and re-ask".
- **The commander's standing rule beats the packet.** `rules.md`: the commander "never walks toward, builds toward
  or attacks a party that outreaches its D-gun or outweighs it", with no "unless the instructions speak to this very
  fight" clause, which the group rule has. human-28 12:06 "attacks his army at home beside group_K ... using its
  D-gun": walked to spot_12 at full health, the plant died at 12:12. Verified in the source.
- **The detachment's forbidden mark takes effect at 0.70.** Readings of 0.61 to 0.69 send the detachment. 33 items
  in 18 games; verified (rove-task-2 15:50: send_8 at 0.69, the others 0.71 to 0.85; three detachments of eight
  left a 53-soldier body in three seconds while the packet said "never splits"; human-29 6:28 to 6:39 at 0.61 to
  0.66; nuke-1-medium's whole-group attack withheld by `WHOLE_GROUP_OVER = 4` and every detachment forbidden at 0.83,
  so nothing attacked his commander at 39:31).
- **Standoff and shelling over the packet's stationing.** A group told to stand at a place or to go in as one body
  was played as a shell standoff, the screen at 0.55 of reach inside the turrets' reach: 27 items in 11 games
  (apex-v022-opus-2 13:41, 30 Blitzes; apex-v024 9:20 to 10:03, 6 Stouts one at a time).

### 3.2 The decode reads the packet into marks, and the marks are not what was written (A6, 79 items)

- **An ordered constructor paragraph reads as "no extractors".** `decode.rs` fuses a move off under 0.3. The
  constructors' `build|armmex` reading, by the paragraph in force (the decode scan, 12 arena games, 1,122 readings
  under a paragraph that begins "Constructors"):

  | the Constructors paragraph says | readings under 0.3 |
  |---|---|
  | "their job is taking free metal spots, in this order: ..." (rove-move-1, apex-v022-opus-1 twice) | 0 of 22, 0 of 39, 0 of 23 |
  | "their job is extractors ... Otherwise free metal spots in this order" (prio-player-1 after 14:25) | 6 of 42 |
  | "first a light turret beside every extractor ... then free metal spots in this order" (prio-player-1 10:45-14:25) | 24 of 35, 20 of 27 |
  | "the ones at home help build the fusion ...; the others take free metal spots" (apex-v024) | 28 of 33 |
  | "constructor_18073 and constructor_904 take apart the wreck fields ...; the others take free metal spots" (jev114) | 24 of 27, 20 of 20 |
  | "a constructor with an enemy party near it walks toward spot_61 ...; constructor_3166 ... take apart the wrecks" (leg-player-1) | 17 of 21 |

  prio-player-1 10:45: six constructors lost their extractor moves for four and a half minutes on the word "then";
  extractors sat at 17 to 20 against the pool's 22 to 27 (verified). rove-task-1 16:39 to 20:13 the same under
  "(rebuild)". The report shows the decode's readings on group entries, not on builder entries, so the player never
  saw that its words had shut the move (B1).
- **A negated place is a destination.** "never walks to spot_8" read as spot_8 at 0.59 to 0.97; "staying out of
  reach of the corexp at spot_0" as spot_0 at 0.92; "never goes toward the turret nest at spot_93, spot_92, spot_91"
  opened those places. 5 items, 5 games; verified in human-31.
- **"Join each other" with no target, a group named by description, a renamed group.** "the home guard ... join each
  other at spot_61" decoded to no join; "the armflash group from plant_19529" matched no actor until the packet said
  group_M (human-30 Comet, eight Blitzes never raided); a join or a pick's merge renamed the packet's group between
  the report and the landing, so the paragraph pointed at nothing (13 items, 11 games). `produce` with `"new"` gives
  the player no name to write before the group forms.
- **The forbidden mark is asked of detachments only.** "Aircraft ... are never chased by any group" holds nothing
  against `attack` or `shell` at an aircraft party, worded "it is unarmed and cannot fight back": 21 items, 9
  games, seven aircraft hunts in nuke-2-hard 28:05 to 28:37.

### 3.3 Code rules end or replace the order (A4, 72 items)

- **The hunt's 900 leash counts from the hunters' own centre.** A detachment cut from a strung-out body, or sent at
  a raider 2,000 away, trips it at birth: *corpus* 395 hunts ended on the leash, **174 "after 0 s"**; 58 items in 18
  games. In leg-player-1 four hunts of lone Pawns ended at 0 s in a row and the Pawns went on (16:27 to 18:00).
  Every "never chase past spot_N" in a packet is this: a hunt has no bound but the leash.
- **A list step "taken as done" on a build that never started.** A list re-sent whose first step names the build the
  builder has been ordered but has not begun is matched to that order and marked done; the next step's order cancels
  the walk: *corpus* 104 lines in 18 games; apex-v022-opus-1 2:45 and 3:15 (two opening extractors 3.3 minutes late),
  human-27 6:56 and 11:07 (held to 4 extractors). Verified in the transcripts.
- **The turret step waits on a refused extractor that stands.** When another builder's frame took the spot first,
  the engine refuses this builder's extractor and the code then skips its `armllt` step ("the extractor at spot_N was
  refused lately; a defence there waits for it") though our extractor stands there: *corpus* 150 lines; leg-player-1's
  spot_14 undefended and lost to a Tick at 24:05.
- **Arrival, gather and follow.** A walk turns to a hold within 300 of the place and the player's route stalls at
  every stop until re-sent (leg-player-1 four stops, 116 s); a gather never ends while newcomers stream in as its
  tail (26 items, 16 games); a follow is a trail with no pace, so the follower arrives after its ward (33 items, 16
  games; rove-task-1 23:11, group_A2 844 to 1,283 behind group_F1 into the nest).
- **Sites chosen by code.** A bare `extractor` step took the plant's exit-lane spot (fixed 3aabc77 after
  apex-v022-opus-1); the `#id` shared by several builders built two plants on the same tick (fixed 4ed68d8 after
  human-33); the code's own solar sites boxed the commander and a constructor for 12.8 minutes (jev114-player-1,
  verified: 232 and 334 failed moves) and no report named the solars or that `remove` would free it (B2).

### 3.4 Refused or dropped at the tool (A1 65, A2 74 items)

- **`signal` inside `orders` is refused** ("not a tool that can be batched"; `batchable()` in `mcp.rs` omits it):
  *corpus* 38 times in 15 games, the signal re-sent alone a turn later (21 s late in glitters-8v8-late-1 t3).
- **`queue` accepts steps the builder cannot make** and the hands skip them silently: *corpus* 180 "this builder
  cannot build it" and 35 "needs a place" lines; nuke-2-hard ten episodes; the armacv steps in the 8v8s where the
  report names an advanced constructor "constructor_N" with no type (B1).
- **`no_site` is dropped with no line**: *corpus* 11 events in the human games, none in any report (the engine's own
  "rejected" is reported as "the engine refused ... the site was bad", 374 lines). human-30 Eclipsed: the tier-2
  plant "the priority" first stood at 15:21 after four silent failures from 13:23.
- **`remove` takes handles the picture does not give** for our buildings outside the exit-lane line: human-32 t4's
  commander penned by a wind turbine from 3:12 to 22:29 ("corwin: nothing of ours has that name"); 13 items, 9
  games. And a `remove` accepted and never carried out: human-33 13:23 (the other builders then repaired the frame
  to 100%), human-32 22:20, glitters-8v8-opus-1 15:58.
- **`transfer` refuses the whole list for one stale name** (a group merged or dead during the turn): 25 items, 7
  games; and the transfer runs mid-turn before the commander's words reach the player (human-33).
- **A mark removal does not reach the side's copy**: the player's `rally_east: null` answered "forgotten" while the
  commander's mark of the same name stood and the army walked to it (apex-v024 10:27 to 10:48, verified).
- **Argument-shape refusals** the Sonnet players hit (`{"arguments": "<json string>"}`, a malformed grid cell) and
  calls sent after `wait`, dropped: 9 in glitters-8v8-late-1.

### 3.5 The report (B1 19, B2 5, B3 15, B5 11 items)

- **The metal store's cap is not on the eco line.** The picture says "metal: 2096 of 2100 stored (full: what comes
  in beyond this is wasted)"; the report's eco line says "metal 1794 (+43.3/-35.3)" with no store size while energy
  shows both. jev114-player-1 at the cap 21% of the game, leg-player-1 148 s; verified.
- **A dead enemy building stays listed, even with our army on the ground.** nuke-1-medium 28:09 to 29:05 "his, in
  E4 (all of it in sight now): 1 corlab" with no corlab in the truth after 26:20; two missiles fired at the dead
  labs, the 26:23 attack routed to one. Verified.
- **The enemy floor read as his army.** "Its soldiers seen in the last 3 min ...: 2146 metal; its army is at least
  that" was 62% of the truth when the push went (jev114 9:52); the 8v8 commander struck a lab on "armies even (20,350
  vs 20,200 seen)" against 28,420 in the truth. The line is labelled a floor; the players read it as the number.
- **Allied seats' spots are invisible.** *corpus* 185 extractor steps skipped "being taken by a builder of another
  seat" and most of the 552 "not free" in the team games; a seat's report shows no allied group either, so "one body
  with t4" has nothing to stand on (B6).
- **The missiles** (nuke-1, nuke-2: no report named a round; fixed b0d7213 after nuke-2-hard), **the recently-seen
  block** (human-26: none in any report; built 8a9788a the next day), **a stuck builder's cause** (never named), **the
  second advanced plant** (human-30 Comet: both builders' entries said "it will be plant_16012" for 2.8 minutes).
- **Moho upgrades counted as losses** on the "extractors lost in the last 3 min" line (apex-v024 15:22 to 16:36, nine
  of ours destroyed by nobody), so the commander ordered home guards on a false premise.

### 3.6 The commander hop (B6, 28 items)

Direction held back "until their next report after m:ss" while a transfer it explained ran at once (human-31 16:50:
t4 attacked under the old direction and lost 14,075 metal); the commander's report entry showed a group's count and
metal with 23 of 31 Blitzes 2,700 to 8,286 behind (apex-v024 7:58, the "now" signal and ten Blitzes); "join into one
body at meet_d5" across seats with no lever but transfer (human-30 Eclipsed 5:15 to 8:15, 3,900 for 2,555).

### 3.2a The decode probed (the same evening; `data/impedance-2026-10-16/decode-probe/`)

The user asked what the full problem is. The decode (`decode.rs`) is one lean request per packet: for each actor and
each thing its menu could do, a yes/no over the instructions alone ("Read the player's `instructions` alone. Do they
have constructor_1412 (Construction Vehicle (armcv; constructor vehicle, 135 metal): tier-1 constructor vehicle, 90
build power; builds the base and the) build a Metal Extractor (armmex), now or under some condition, or say anything
else to it about that kind of building?"); the actor's words are the first 120 characters of its picture entry's
gloss, never its position. A reading under 0.3 fuses every move of that kind off the actor's menu until the next
packet; a detachment's forbidden mark is on at 0.8 and off under 0.2; the band between is left to the pick's own
question each second. jev114-player-1: 260 decode requests, 22,259 readings, 16,769 of them under 0.2, 2.1M tokens
($0.09); the fuse withheld 45% of what the gate would have asked, the largest saving of the nine layers.

Five probe rounds (115 requests, both Jev versions, three repeats each; the answers drift by about 0.05) on the
extractor question under the paragraphs the study flagged:

| the Constructors sentence, asked for a constructor the packet does not name | reading (1.14 / 1.13) |
|---|---|
| "their job is taking free metal spots in this order: ..." alone | 0.03 / 0.07 |
| the same, inside rove-move-1's full packet | 0.36 |
| "their job is taking free extractor spots, in this order: ..." inside a full packet (rove-move-1's own, or prio-player-1's) | 0.80 / 0.76 |
| the same sentence alone | 0.14 |
| "their job is building extractors (armmex) on free metal spots" alone | 0.53 / 0.48 |
| prio-player-1 10:37, "first a light turret beside every extractor that has none (...); then free metal spots in this order" | 0.29 / 0.38 (the turret question on the same text: 0.68 / 0.74) |
| the same with the spots before the turrets | 0.52 / 0.68 |
| the same without "when Pawns ... Otherwise:" | 0.21 / 0.20 |
| prio-player-1 14:25, "their job is extractors. When Pawns ... Otherwise free metal spots in this order" | 0.52 / 0.40 (a question reworded to "among the jobs ... at any time": 0.74 / 0.79) |
| apex-v024, "they work their lists. When a list is done, the ones at home help the fusion; the others take free metal spots" | 0.30 / 0.31 (without the list framing: 0.38 / 0.27) |
| jev114, "constructor_A and constructor_B take apart the wrecks ...; the others take free metal spots", for one of the others | 0.13 / 0.06 ("every other constructor, constructor_18284 among them": 0.46 / 0.41) |
| the actor's words in the question: the gloss cut at 120 characters / "Construction Vehicle (armcv)" / "a constructor that builds extractors, generators, turrets and factories", on rove-move-1's sentence alone | 0.14 / 0.32 / 0.47 |

So: the reading of "does the packet give this constructor extractors" is not about one shape. It swings by 0.4 on
whether the prose says "extractor" rather than "metal spots", by 0.3 on the order of two clauses, by 0.3 on an
exclusion ("the others"), by 0.3 on the actor's description in the question, and by 0.4 on whether the sentence stands
in a full packet or alone; the question's own wording moves one case and not the others; 1.14 and 1.13 read alike.
A human reads every row of the table the same way. The fuse then takes a reading of 0.25 as a clear no and removes
the job for the packet's life. In prio-player-1 the audit sampled 42 fused extractor moves to spots the paragraph
lists by name and the gate's own fresh rating of each was 0.09 to 0.31: the audit measures agreement between two
Jev readings of the same prose, not whether the prose was read as meant, so it reports 0.5% faults while the
player's expansion stands for four minutes. The report shows the decode's readings for groups only
(`reads_words`), so the player cannot see that its words shut a builder's move. Places read better: the place
question is the one measured before the rebuild (72% of answers under 0.2, 1 to 2% cross-paragraph), and a place the
packet forbids reads yes by design (it is then the pick's own question each second), so the "negated place" items
of §3.2 are the pick's reading, not the decode's.

## 4. What the players asked for that nothing can do

Sentences in packets or direction that no menu move, list step, lane word or tool carries, by how many games asked:

| the ask | games | what exists instead |
|---|---|---|
| a constructor or the commander never stands or builds in a plant's exit lane | 13 | code places list steps and helpers; the yard watchdog wakes the player after 30 s |
| a route in order, never going back to a reached stop; "reached" within 150 | 6 | one place per move; arrival at 300; the route re-sent stop by stop |
| a hunt bounded by a place ("never chase past spot_N") | 5 | the 900 leash from where the hunt began |
| target order inside a fight (the constructor first, the Overwatch first, the armrectr first) | 6 | attack or shell a whole party; the micro engine's own targeting |
| surround, converge from several sides, go in as one body arriving at once | 7 | attack_party (soldiers arrive as they come), gather (never ends), march (waits for joiners too) |
| split a group by unit type (the Rovers stay, the Blitzes go; Stouts in front, Whistlers behind) | 8 | detachments of 1, 2, 4, 8 by the fastest; `form` ignores type |
| join the army in groups of six, never one at a time | 3 | newcomers walked singly to the body |
| a route that keeps out of a turret's or his commander's reach | 5 | the engine's path; the fuse shuts named places only |
| force a group to drop its course and re-ask; a direct move order | 4 | a packet reaches a busy group only through the change gate |
| rove with a destination, a region or a thing to look for; a rover sent alone to a place | 8 | rove chooses its own targets in code (his start box first, the least-known cells) |
| aircraft never chased; a radar contact named ("party_202 is a Shiva") | 4 | the forbidden mark on detachments only; the picture's "1 unidentified ... we outweigh it heavily" |
| cross-seat waits, joins and follows; spot ownership between seats | 4 | transfer, and prose each seat reads alone |
| attack-ground at a remembered building; a turret's target order | 2 | shelling fires at what is in sight; a turret has no menu |
| a constructor reclaims a named wreck field; tier-1 builders help a tier-2 frame by its id | 2 | the menu's three nearest fields; an id step means "build this type" |
| the whole group at a party it outweighs four times | 3 | `WHOLE_GROUP_OVER = 4` offers detachments only |

The first row is the loudest: thirteen games, every packet for minutes on end, and the players' notes show them
re-listing builders by hand to get them out of the lane.

## 5. Already answered since the game was played

Items the reviewers found that a later commit fixed: the exit-lane extractor (3aabc77, 2026-10-13: apex-v022-opus-1
1:08, rove-task-2, prio-player-1 1:56 are before it; jev114-player-1's reviewer reports one after it, unverified here);
the same-tick `#id` race (4ed68d8, 2026-10-12: human-30 Comet 14:36, human-33 12:41, rove-move-1, human-27 12:24);
the missiles line and the missile_ready wake (b0d7213, 2026-10-09: nuke-1-medium, nuke-2-hard); the recently-seen
block (8a9788a, 2026-10-07: human-26); the four human-27 fixes (6ea433b: the `same` layer voided on a packet
change, newcomers, the placed-site event, the blocked-site watchdog); the watchdog's clock (de7dc80, 08599ab).

## 6. What it says

1. In a judgement moment chosen for mattering, the harness did what the player wanted with the context it needed
   one time in four. The rest splits about evenly between Jev's pick going its own way (A3), the decode reading the
   packet into other marks (A6), a code rule ending or replacing the order (A4), and the tools refusing or dropping
   it (A1, A2). Missing context is a fifth of the items and carries two of the three game-deciding ones.
2. The packet is advice. The one-decider design wants that, and the reviewers' A3 items are the price: the player
   writes "goes now, whole, never splits" and the pick weighs it against a hunt rated 0.66. Three players asked in
   their notes for an order Jev cannot refuse.
3. The decode is where words turn into nothing: a "then" or a "the others" in the constructors' paragraph removes
   extractor building from the menu for minutes, a "never walks to" opens the place, a group described rather than
   named gets no mark. The report shows these readings for groups and not for builders, so the player cannot see it.
4. Five code faults are cheap and exact: the leash from the hunters' own centre (174 hunts dead at birth), "taken as
   done" on an unstarted build (104), the turret step waiting on a refused extractor that stands (150), `signal`
   outside the batch (38), `queue` accepting impossible steps (215). Five report lines are the same: the metal cap,
   the dead building in a seen cell, upgrades as losses, `no_site`, the builder's type.
5. The no-lever table is the design question. Half the rows are one shape: the player wants to say something about a
   group's *inside* (which units, which targets, where each stands) and the menu speaks only of whole groups and
   places.

## 7. What it does not say

- The 209 moments are a fixed sample, not the game; the item counts are lower bounds and the A/B split is the
  reviewers' judgement on the brief's categories. Two reviewers counted a moment clean only when no item at all came
  from it; the rest when nothing above cost none did.
- The per-game `refused` counts in the JSON files are the sweep's at the time (the pattern was widened twice during
  the run, for the batched `signal` and the argument-shape refusals); the 101 in §1 is the final sweep over every
  transcript and is the number to use.
- The human games have no truth file: their B items rest on the record alone, and the 8v8s' five unread seats are
  counted and not judged.
- Cost is the reviewers' measure from the tools; "large" was required to carry a number, "game" is three items at
  low or medium confidence.
- The mechanism clusters of §3 are the main session's reading of the items (keyword grouping checked by hand) and
  the corpus scans; a reviewer's item may sit in two.
- Nothing here says what to build. Each §3 shape is a candidate for the user's word: a code fix, a tool check, a
  report line, a menu move, or a change to how the packet reaches the pick.
