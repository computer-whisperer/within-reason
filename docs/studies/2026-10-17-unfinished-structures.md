# Unfinished structures: what became of every frame in 23 games, and why (2026-10-17)

The user, 2026-10-17: "send a sweep of opus agents tallying up the causes for unfinished structures in recent
games." The same twenty-three games as the impedance study (2026-10-06 to 2026-10-16: 12 arena duels, 2 realtime
8v8s, 9 games with people; 48 seats of ours), one Opus 5.5 reviewer a game under one brief
(`data/unfinished-2026-10-17/BRIEF.md`), one instrument for the denominators (`run/frames.py`, written for this
sweep), one tally shape (`run/unfinished_tally.py`). Nothing was changed in the bot by this study; what to fix is the
user's call (§7). Data: `data/unfinished-2026-10-17/` (every game's `unfinished.json` and `unfinished-review.md`,
`denominators.txt`, `tally.json`, `games.txt`, the brief). **Built 2026-10-17 on the user's word** ("Let's fix that
two hands bugs first"): §7's items 1 and 2, the task keyed to its own frame (H-HANDS-STARTED amended) and the loss
line telling a kill from a decay at the peak share (H-HANDS-STALL amended); unit tests, a smoke, unmeasured in a game.
§8 reviews what the hands give the player for knowing about and resuming a frame; **its asks 1, 2 and 4 and the
first `stop` semantic were built 2026-10-17 on the user's word** ("Go with the first, build all three"): the report's
`unfinished structures` line, the `finish <handle>` step and `finish_<handle>` move, the "unfinished <type> at
<place>" form of a reclaim, and the stop that spares a frame begun after the report (H-HANDS-LISTS and H-HANDS-SCRIPT
amended, `docs/code-decisions.md` §2). Ask 3, the site search resuming our own frame, is not built.

**Definitions.** A *frame* is a structure of ours the engine created (the record's `created` event: anything
immobile, so extractors, generators, turrets, plants, nanos, radars, storage, converters, silos). Its *fate* is
`finished`, `decayed` (destroyed with no attacker named: the builder left and the nanoframe decayed, or something
out of sight shot it), `killed` (destroyed by a named enemy while unfinished) or `unfinished at end`. *Metal in* a
frame is its peak build share times its cost. A *cause* is one of U1 the pick sent the builder elsewhere, U2 the
builder's own list moved on, U3 the player re-listed or moved it, U4 the builder died or fled or the frame was
killed, U5 economy, U6 the site, U7 helpers withdrawn, U8 game end, U9 harness fault, U0 unknown.

## 1. The count

By the instrument, over every seat of the 23 games (`denominators.txt`):

| | frames | share | metal in |
|---|---|---|---|
| started | 5,553 | | |
| finished | 4,805 | 86.5% | |
| decayed | 311 | 5.6% | 22,270 |
| of which shot by an attacker the record did not name (damage in the frame's last 15 s) | 106 | 1.9% | 7,611 |
| killed while unfinished | 332 | 6.0% | 17,609 |
| unfinished when the record ended | 105 | 1.9% | 23,910 |
| build orders never started (a `build` command with no frame at the engine's placed site in 90 s) | 1,338 | | |

So one frame in seven did not finish, and 39,879 metal went into frames that were lost (the 23,910 standing at the
end is not a loss: the fusions and Pit Bulls of won games' last minutes are most of it). By kind: the 12 arena
duels 2,235 frames and 31,889 metal unfinished, the 9 human games 1,730 and 21,651, the two 8v8s 1,588 and 10,249.
Per game the lost metal runs from 172 (rove-task-2) to 5,603 (human-31); the median is about 800.

The reviewers' items (748 frames, every unfinished frame of every seat; 673 at high confidence, 72 medium, 3 low):

| cause | frames | metal in | the shape |
|---|---|---|---|
| U4 killed, or the builder died or fled | 432 | 21,356 | raids on forward spots, fire from out of sight, builders kept building under it, long builds crawling at an empty store in reach |
| U8 game end | 105 | 23,702 | no fault; 28 of them (12,888 metal) started more than two minutes before the end, mostly fusions in won games |
| U3 the player's own stop or re-list | 129 | 10,659 | the exit-lane wake answered with a leading `stop`; triage of advanced solars in an energy stall; stops that landed on frames begun after the report; stops on builders the entry called stuck |
| U9 a harness fault | 34 | 5,023 | the finish handler by type and distance (§2.1), the same-tick id race, the help step overrun, a stop cancelling a remove, the watchdog |
| U1 the pick | 33 | 818 | retreats under fire from a frame, the basic/advanced solar flip under a packet condition |
| U5 economy (first cause) | 8 | 1,771 | 53 frames and 10,026 metal with U5 as first or second cause |
| U2 the list | 5 | 420 | a queued step promoted early |
| U6 the site, U7 helpers | 2 | 40 | these live in the never-started list, not among frames |

The reviewers' fix classes, the game-end frames aside: `player` 246 frames (16,179 metal), `hands-rule` 205 (14,192),
`report` 48 (3,976), `tool` 27 (978), none 115 (4,713). What the player knew: 484 frames stood in a report before
the cause acted, 210 reached it only through the loss line afterwards, 48 were never said; it dropped 133 itself.

## 2. The causes, by what would have kept the metal

Ranked by the metal the mechanism touched across the 23 games (the keyword groupings over the items are in
`tally.json`; the numbers overlap where a frame had two causes).

### 2.1 The hands look a frame up by type and distance, not by the frame the builder's own order made

Named in 18 of the 23 games: 55 frames, 7,624 metal, of which 30 frames and 4,798 metal as the first cause and the
rest as the `stop` the player then sent at a builder whose entry read "finishing an order". Three lines of code,
all verified at HEAD by reading them:

- `pianist/mod.rs` `Event::UnitFinished` (line 1677): every builder whose started build is of the finished unit's
  type within 250 of it has its task ended or its queued step promoted. A neighbour's advanced solar finishing in a
  packed field takes the builder off its own frame at 62 to 98 percent (rove-move-1: three advanced solars, 914
  metal; glitters-8v8-opus-1: three, 823; human-30 second game: four, 893; nuke-2-hard: three, 637; apex-v024: 322;
  jev114-player-1: 270; prio-player-1: 177; human-30 first game: 152; rove-task-1: 152; human-31: 510).
- `pianist/mod.rs` line 1751, the blocked-site watchdog: a builder standing at an order's site with no frame of
  that type within 200 is "blocked". When the finish handler has promoted its queued step, the builder is still on
  its own frame 200 to 230 away from the queued site, so the watchdog stops it ("never started in the 8 s it stood
  at the site") at 76 to 92 percent and re-places the order elsewhere.
- `picture.rs` `task_course` (line 1037): a builder's entry shows the share of the most advanced frame of its type
  within 200 of its site, so the entry reads the neighbour's 98 percent while its own frame stands at 8, and turns
  to "finishing an order" when the neighbour finishes. The player read that as stuck and sent `stop` (human-30
  second game, nuke-2-hard, jev114-player-1, human-28: 11 frames, 2,348 metal).
- The created-event side does the same (jev114-player-1: any frame a builder makes is taken as its task's).

The `created` event names the builder (`by`), so the fix is one field: the build task keeps the unit id of the
frame its own order created, and the finish handler, the watchdog and the entry read that frame. A hands'
correctness fix, no decision in it.

### 2.2 Frames shot by something out of sight, reported as abandoned

106 of the 311 "decayed" frames (7,611 metal) took damage in their last fifteen seconds; the engine's destroyed
event names no attacker for a shooter out of sight, so the record, `floor.py`'s `aband` column and the hands' own
loss line all call them decays. The hands' line is "abandoned an unfinished X at Y (N% built): its builder was sent
elsewhere and the frame decayed" (`pianist/mod.rs` line 1693), said of frames whose builder stood on them while a
Banisher, a corvroc, an Ambusher or a Fatboy shot them (nuke-2-hard: 18 of 22 decays, 1,627 metal, eight to one
corvroc in sight once in eight minutes; glitters-8v8-late-1: 18 of 33; glitters-8v8-opus-1: 16 of 44; human-31: 10
of 38; leg-player-1: 6 of 12, where the player then sent a third constructor to the same spot). The players
believed the line ("two half-built Pit Bulls were abandoned", nuke-2-hard 28:30). The hands see every damage event
(the record's `dmg` is their sum), so the line can tell a kill from a decay.

The same line gives the share at destruction, not the share the frame reached: a nanoframe's progress decays once
left, so every decay reads "0% built" or "1% built" for frames left at 44 to 98 percent (said by 15 reviewers;
`briefing.rs` line 193 rebuilds the share map from the current health every tick). The player is never told what a
drop cost. `floor.py`'s `aband_m` had the same fault (6 metal against 984 in jev114-player-1); fixed in this
commit to the peak share.

### 2.3 Builds started into a raid the picture already showed, and builders kept building under fire

The largest count: 153 frames (8,658 metal) in 17 games began with an armed enemy within 600 to 900 of the site in
the report the player had just read or in the builder's own entry (glitters-8v8-late-1: 66 of its 96 killed frames;
prio-player-1: 11 of 18; human-32: Rascals sitting at the spot). The same raided outer spots were re-listed again
and again (spot_75 nine times across two seats of glitters-late-1). And 132 frames (8,227 metal) died with the
builder standing on them: the gate rated the builder's "change" at 0.65 to 0.90 under fire for 8 to 17 seconds,
but no single move cleared the world bar, so nothing moved it (human-29, rove-move-1, leg-player-1, apex-opus-2).
The hands' hold on a list step near a party was built and rejected on 2026-10-01 as a code gate (H-HANDS-SCRIPT);
the reviewers' proposals are a hold asked of Jev, or escorts from the player. Both are the user's call.

### 2.4 Long builds crawling at an empty store, in reach

53 frames with U5 as a cause, 10,026 metal, in 17 games: t4's advanced plant in human-31 built by eight
constructors for 177 s at a metal store of 0 and killed by Tigers at 91 percent (2,366 metal); the Agitator in
human-33 raised for 2:35 at metal 0 and killed at 98 percent by a lone Tzar (1,274); an Overwatch in rove-task-1
157 s at 1 percent progress every few seconds (400); twelve turret frames in glitters-late-1 at metal 0 for 30 to
89 s and lost at 22 to 90 percent (652). The reviewers' lever is the player's spending (one frame at a time when
the store is empty) and a report line giving a frame's time to finish at the current income; the hands have no
rule here and should not without the user's word.

### 2.5 The player's `stop`

129 frames, 10,659 metal, in four shapes:
- **The exit-lane wake answered with a leading `stop`** (39 frames, 3,583 metal, 13 games): the report said the
  builder stood in its plant's exit lane; the player's only lever was a `stop`-headed list, which drops the build
  in progress. In most of these the frame itself stood just outside the lane and the builder's working spot (about
  130 short of the frame) inside it (apex-sym0: 206 metal; apex-sym50-opus-1: 510; leg-player-1: only one of four
  dropped frames overlapped a lane). The building's keep-out covers the footprint, not where the builder stands.
- **Stops that landed on a frame begun after the report** (84 U3 items mention the landing, 6,882 metal, 19 games,
  overlapping the other shapes): under the think penalty or a realtime turn of 8 to 23 s, a stop written against a
  walk or a queued step met a frame at 49 to 97 percent (human-31: 445 metal; glitters-late-1: four frames in the
  6 to 10 s gap). The `queue` result says nothing about what the stop dropped.
- **Economy triage** (23 frames, 1,754 metal): advanced solars (5,000 energy each) begun into an empty energy store,
  crawling at a third of their rate, dropped for basic solars; informed and mostly sound.
- **Stops at builders the entry called stuck** (§2.1's "finishing an order": 11 frames, 2,348 metal).

### 2.6 The same-tick id race and what the player's remove then did

54 items, 7,855 metal, in 16 games: six builders given `armavp #avp1` in one `queue` call all saw the id free and
two plants rose (human-32: four duplicates, 1,738 metal; human-28, human-27, rove-move-1, nuke-1). Fixed on
2026-10-12 (`lists::nearest_first`, `same_tick_help`, after human-33), so these are the games before it; what
remains live is the player's answer: its `remove` of the duplicate was turned into "help plant_6196 build" by the
shared id (human-32, 1,040 metal), or cancelled a second later by the same turn's `queue` beginning with `stop`
(rove-move-1, 208 metal). Open: the order of `stop` and `remove` in one turn, and a remove on a frame an id still
claims.

### 2.7 Smaller harness mechanics, each a few hundred metal

- A list's `help`/repair step is a filler to the list, overrun by its next step a second later (`lists.rs`, the
  rule 5046f71 exempted `reclaim` from): the moho the player had dropped and listed a builder to finish was helped
  for one second (rove-move-1, 242 metal).
- A builder the pick walks off its frame has no way back: `menu.rs` offers repair for finished units and help for
  a frame another builder is on, so a frame nobody is on is on no builder's menu; the new list's unplaced step then
  starts a fresh frame 400 away (apex-opus-2: 113; rove-task-2: 37; human-26: the host's site search treated our own
  71 percent frame as an obstacle and placed a new one 64 away, 119).
- The 60 percent queue-ahead hides the frame still being built: once the next step is queued, the entry and the
  hands treat it as the builder's course, a re-list drops the frame without the "build in progress finishes"
  protection (human-33: two nanos, 402 metal).
- A `stop`-headed re-list of the same builder in the same second as its list's step, the pick's walk and the list's
  build in one second (leg-player-1), the watchdog firing the second a frame was created (human-33) or at arrival on
  a stale clock (rove-task-2; fixed de7dc80).
- The "taken as done" on an order not yet begun (fixed e9f5175 on 2026-10-16) accounts for most of the never-started
  extractors in the earlier games (3 to 4 minutes late each).

## 3. The report's faults, as the reviewers found them

- The loss line calls a shot frame abandoned and gives the share after decay (§2.2).
- A frame killed while unfinished is reported as a full loss ("lost armllt to armpw"), the same words as a finished
  building; the fights line counts a 0 percent frame at full cost.
- The exit-lane wake does not say whether the frame or the builder is in the lane (§2.5).
- A builder's entry shows a neighbour's share (§2.1) and never names the builder's list (human-28: none of 380
  entries did), so a builder waiting behind its build as designed was read as having "picked up a solar instead".
- No line gives a frame's time to finish at the current store (§2.4).
- A `no_site` on the first order of a long chain was never said (human-30 second game: fourteen orders for one
  advanced plant over two minutes; the no-site line was built 2026-10-16, after it).
- The threat line counts an unfinished turret frame as the extractor's turret (rove-task-1).
- Our own D-gun destroying our frame reads "lost to unseen" (human-26, apex-sym50-opus-1).

## 4. The orders never started

1,338 by the instrument (an order with no frame of that type by that builder at the engine's placed site within 90
s; the first reviewers ran a version that measured from the commanded point and over-counted by a third, and their
quoted lines say which). The reviewers classified 541 of them: U3 the player's new list 222, U6 the site 106, U4
the builder fled or died 71, U1 the pick 53, U8 the end 47, U2 23, U9 16. Nothing was put into them, so no metal
is lost, but the shapes cost time: two of the player's lists naming one spot, the second builder refused and its
turret step skipped, and a false "the engine refused ... the site was bad" that puts a 90 s refusal on the spot
(leg-player-1: 5; apex-sym50-opus-1); orders of one tick placed on one site, so advanced solars go up one at a time
instead of in parallel; builders walled in by their own solars (jev114-player-1: the commander's three second-plant
orders and its lab order, so no lab was ever built); stuck constructors for 3 to 6 minutes (glitters-opus-1,
rove-task-2); a build order that did not replace the pick's walk, the builder driving 2,900 the wrong way for 50 s
while its entry counted the distance up (apex-sym0).

## 5. Verification

- The three code lines of §2.1 were read at HEAD by me: the match is by `def` and a 250 radius in the finish
  handler, by a 200 radius in the watchdog's own-frame test, and by the best same-type frame within 200 in the
  entry. The share map of §2.2 is rebuilt from the current health every tick (`briefing.rs` line 193).
- The instrument's decayed count equals `floor.py`'s `aband` on every game checked (the reviewers checked all 23);
  its shot-frame count equals the nuke-2-hard reviewer's hand count (18 of 22, 1,627 metal) once the window took the
  sample after the destroyed event.
- A seeded random sample of eight reviewer items (causes U1 to U9) was put beside the instrument's entries: all
  eight hold (the stop, the list promotion, the pick's walk and the unseen fire each cite the line the instrument
  prints).

## 6. The instruments

- `run/frames.py MATCH [--seat N] [--all] [--json]`: every frame, its fate, metal in, the builder's next commands
  and when it left, the decision rows (list, plan, heuristic), the stores, the helpers, a shot frame flagged by its
  last hits; then the orders never started. 0.14 s a game.
- `run/unfinished_tally.py MATCH... [--items] [--cause U9] [--mechanisms] [--json OUT]`: sums the reviewers' files.
- `run/floor.py`: `aband_m` now uses the frame's peak share.
- The brief, `data/unfinished-2026-10-17/BRIEF.md`: one Opus reviewer a game, two output files into the match
  directory; a new game is reviewed the same way and the tally reads it beside the others.

## 7. What would keep the most metal (the user's call)

In order of metal per line of code, with the class of change:

1. **Key the build task to its own frame** (§2.1): a correctness fix in the hands, no decision; about 7,600 metal
   over 18 games, and the misleading entries with it. **Built 2026-10-17.**
2. **The loss line tells a kill from a decay and gives the peak share** (§2.2): a report fix; 106 frames the player
   was told the wrong story about, and the "0% built" of every decay. **Built 2026-10-17.**
3. **The exit-lane keep-out covers the builder's standing ring, and the wake says frame or builder** (§2.5): a
   placement rule in the same class as the exit-lane extractor fix of 2026-10-13, plus a report line; about 3,600
   metal.
4. **A `help`/repair list step is not a filler** (§2.7): the reclaim exemption's twin.
5. **A way back to one's own frame**: a menu move for the frame a builder left (the frame nobody is on), and the
   site search not treating our own frame as an obstacle; a menu option.
6. **A `stop` that spares what the player did not see**: the `queue` result naming the share and the seconds to
   finish of the build a leading `stop` drops, and the order of `remove` against `stop` in one turn; tool
   semantics.
7. **A frame's time to finish at the current store** on its builder's entry (§2.4): a report line; the lever for the
   10,000 metal of crawling builds is then the player's.
8. **A list step toward a party in the picture** (§2.3): the largest count and the one that needs a decision; a
   hold asked of Jev (the one-question shape, not a code gate) or the player's escorts.

None of these is built. The first two are cheap and carry no decision; the next player games' `frames.py` lines
measure whatever is built.

## 8. What the hands give the player for knowing about and resuming a frame (reviewed 2026-10-17)

The user, after the two fixes: "let's review what capabilities the players hands have for knowing about and resuming
partial structures." Read from the code (`picture.rs`, `menu.rs`, `lists.rs`, `execute.rs`, `remove.rs`, `mcp.rs`)
and the brief; nothing here is built.

**Knowing.** A frame reaches the player's report in five places, none of them a list of frames:
- *The builder's entry*, while the builder's task is that build: "building a Solar Collector at spot_73 (B8) (62 %
  built), ordered 40 s ago" (now its own frame's share). Once the hands queue the next step at 60 %, the entry's
  course is the queued step ("next, queued: build ..."), and the frame still rising is not in it (human-33: two nanos
  dropped by re-lists that never saw them).
- *A place's words*: "our extractor (being built)" at a spot, "our X (being built)" among the units beside a named
  place. Only frames at or beside a place the picture names.
- *A factory's `yard` entry*: the buildings standing in its exit lane, by handle (armsolar_31002), frames included.
- *The hands' lines on the way out*: a pick's move off a started build says "leaves the Solar Collector at about
  62%, which decays"; a `stop` says "stopped what it was doing on your `stop`" and does not name the frame or its
  share; a list replaced says what it dropped of the queue, not of the build in progress.
- *The loss line*, after the fact: now "abandoned an unfinished X at Y (N% built): its builder was sent elsewhere /
  died and the frame decayed", or "lost our unfinished X at Y (N% built) to fire from something out of sight / to
  <killer>", and the fights line's counts.

So a frame nobody is on, the orphan the sweep counted 205 of (14,659 metal), is in no report unless it stands at a
spot or in an exit lane: the player learns of it when it decays. The unit cards (`publish_cards`) carry every frame
with its handle and health, but no read tool shows them to the player; they feed `remove` and the side's view.

**Resuming.** What can put a builder back on a standing frame:
- *An id's frame.* A list step carrying `#id` whose building the engine has made finds it `UnderWay(frame)` and the
  builder helps that frame (a guard on it; `lists::tag_state`). Only for a frame begun under that id: a frame a pick
  started, or an id-less step, has no id, and a fresh id on a later step finds nothing and starts a new frame.
- *The same order again.* A list step naming the type of the builder's current order keeps the order (started or
  not), and a pick of the same type at the same site is the build going on (`at_the_orders_site`). Both need the
  builder's task to still be that build: once the pick, a `stop` or a re-list has replaced the task, the frame is
  no longer anyone's. A new `build X at place` for the frame's site goes through the host's site search, which
  treats our own frame as an obstacle and places the new frame beside it (human-26: a new Sentry 64 from the 71 %
  one, which decayed beside a working builder).
- *Factories only.* A build order for a factory type with an unfinished frame of that type within 900 helps that
  frame instead (`FRAME_HELP`), from any builder, list or pick.
- *The menu.* `help_<builder>` is offered for a frame another builder is on; `repair_<handle>` for finished units
  under 70 % health, frames excluded (`!u.being_built`). A frame nobody is on is on no builder's menu.
- *The engine.* `Command::Repair` on a frame builds it (a `Task::Repair`), which is what an id's helper and the
  `help` move send; nothing offers it for an orphan by handle.
- *`remove`.* A frame can be reclaimed by handle, or as "<type> at <place or cell>", for most of its metal back;
  the player must know the handle (the yard entry is the usual source).

**The gaps, as asks** (each the user's call; none a code decision in the no-live-heuristics sense, since every one
is a report line, a menu move or a tool, chosen by the player or the pick). Built 2026-10-17: 1, 2 and 4 (the
`stop` line names what it drops, and a `stop` spares a frame begun after the report; the queue-ahead entry is
unchanged); 3 is not built.
1. A report block of our unfinished frames nobody is building, with handles, share, how long since the builder
   left, and where (the cards have all of it).
2. A `finish <handle>` list step and a `finish_<handle>` menu move for a standing frame near a builder (the engine's
   repair command), so an orphan can be given to anyone; and the id mechanism to adopt a standing frame of the
   type at the step's place when the id is new.
3. The host's site search to resume our own unfinished frame of the type at the asked site rather than place a
   second beside it (the twin of the factory rule, for every type).
4. The `stop` line and the `queue` result to name the build they drop and its share, and the 60 % queue-ahead entry
   to keep the rising frame in view beside the queued step.
