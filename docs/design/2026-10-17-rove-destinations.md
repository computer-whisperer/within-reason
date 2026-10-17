# Rove with a destination or a region (proposed 2026-10-17, not built)

**Status.** Proposed by Claude (Fable 5.1) after the impedance study's "no lever" list (the user, 2026-10-17: "The
one more change I think we should make now is adding the rove destinations option; let's design that"). **Ruled
and built 2026-10-17** (the user: "The 0.5 bar sounds fine, and I would just implement this as a cell-range region
for now rather than a beat or anything else more complicated. I think rove_1_<place> should be included, agreed on
nothing from produce"): regions only, as `mark` cell ranges (§3.3), the move family `rove_<region>` and
`rove_1_<region>` under the 0.5 bar (§3.1), the task's `within` and the engine's confinement (§3.2's region half);
no first place, no point beats (§3.2's `first` is not built). H-MICRO-ROVE and `docs/code-decisions.md` §3 carry
it. The first smoke (rove-region-smoke, `docs/experiments.md`) found the bar unreadable under the arena's default
layers (`news` alone: no `fuse` layer, no decode, no reading), while the player had marked `his_half` and written
`rove_his_half` unasked; so without the `fuse` layer every marked region is offered, and the bar applies where the
decode runs (the real player games, which run every layer). Unmeasured in a game otherwise.

## 1. The ask

Eight of the twenty-three games of the impedance tally (`docs/studies/2026-10-16-opus-hands-impedance.md` §4) had a
player or a commander write a rove with somewhere to go, and the hands had only the plain rove. The sentences, by
shape:

- **A place to look at first, then rove.** "its Rover leaves alone to rove (rove_1), his start box's south-west
  first (spot_55, spot_58)"; "one Rascal is sent alone to spot_12 and then h1_look ... the rest rove"; "its Rover
  leaves now as rove_1 up the east edge to spot_37 and spot_12; leave it to rove" (repeated in five packets);
  "Send one Rover alone, now, to look at A7 and A8".
- **A region to rove within.** "the look we need is columns A-C, rows 5-8 (spot_68, spot_74, ...)"; "look for it,
  and for any second factory, in H6-H8 and H1-H3"; "the Rovers rove the cells seen least (F7, E7, E3, D3)"; "they
  rove in his half"; "it roves; leave it to rove. It stays in his half."
- **A thing to look for.** "it is looking for his commander and any new factory of his"; "a Rover keeps looking at
  his base now and then for an air plant".
- **Plant output straight to rove.** A player's note: "Wished: a produce option that sends a plant's units straight
  to rove".

## 2. What exists

Roving is a group task (`GroupTask::Rove { since }`, the rework of `docs/design/2026-10-07-rove-approval.md`): the
pick puts a group on it (`rove`) or its fastest soldier as a group of one (`rove_1`), any later pick is the way back,
and the micro engine runs each rover whole (`crates/micro/src/rove.rs`, H-MICRO-ROVE). The engine's goals come from
the host once a tick per unit type (`View::rove_goals`): every metal spot the type can reach, with when it was last
in sight of ours and whether it is his (in his start box or within 600 of his found base), plus his base. A rover
with no goal takes, in order: his places never seen or not seen for a minute (tier 0), any place never seen (tier
1), the place longest out of sight (tier 2), nearest first within a tier, never a place another rover is going to,
never one whose way passes a reach. So "look at his base now and then" and "the cells seen least" are what the
rove does by itself; "a thing to look for" is answered by the rove log and the Found events the picture carries.
What it cannot take: a place to look at first, a region to stay in, a region to leave alone.

The place vocabulary: `scene.named` (every mark, every spot, passage or `home` the packet names, every spot a list
step names) gives a group's menu `go_<place>` and `fight_to_<place>`; the decode reads once a packet whether the
instructions name the place for the group (`<group>|place|<place>`), the `fuse` layer shuts a move reading under
0.3, the `places` layer blanks a place whose own question said no. Measured on three games (jev114-player-1,
prio-player-1, rove-task-1): 30 to 39 `go_` moves on a group's menu, 6 to 9 of them asked at a gate.

## 3. The design

### 3.1 One move family: `rove_<place>` and `rove_1_<place>`

A group's menu carries, beside `rove` and `rove_1`, a `rove_<place>` for each named place the decode reads as the
group's (its place reading at 0.5 or over: the bar the report already uses to say `reads: spot_27, spot_18`), and
`rove_1_<place>` likewise when the group has two or more soldiers. The words:

- a point place (a spot, a passage, a mark at a point): "group_A roves from spot_12: its soldiers go to look at
  spot_12 first and rove from there, looking at what we know least, killing unguarded constructors and extractors,
  and keeping out of every reach; a pick of yours brings it back";
- a region (§3.3): "group_A roves within east_half (E1-H8): its soldiers look only at places inside it, ...".

A plain `rove` pick clears the beat; `rove_<other>` changes it; a roving group's course words say its beat ("roves
within east_half"; "roves from spot_12, looked at 40 s ago"). The decode's place reading is shared with `go` and
`fight_to` (one key per group and place), so the family adds no decode question; it adds one to three gate
questions per group menu instead of the thirty a move for every named place would, and nothing for a group born
after the packet (no readings: the plain `rove` only, until the next packet names it).

Why a move and not a reading on the plain `rove`: a Choice asked of the decode ("which place do the instructions
give group_A to rove toward?") would cost nothing per second but would be the decode's word-fragile reading
(the study §3.2a-b) deciding where a rover goes, with no pick over it; a move per place puts the choice before the
pick with the full gate question and the place's words, which is how every other place is chosen.

### 3.2 The task and the engine

`GroupTask::Rove { since, beat: Option<Beat> }` with `Beat { name: String, first: Option<Vec3>, within:
Option<Region> }`. The micro engine's `View` gains `rove_beat(unit) -> Option<RoveBeat>`; the host answers from the
rover's group. In the rover's goal choice:

- `first`, while the place has not been in sight of ours since the beat was set (the host's `spot_seen` against
  `since`, so the engine keeps no memory of it), is the goal, subject to the same clear-way test as any goal; a
  way that passes a reach gives it up with the event "gives up spot_12: its way passes the reach of ...", the beat's
  `first` is dropped and the rove goes on free. Then free roving, as today.
- `within` filters the goals to the region: nothing outside it is looked at, and when nothing inside is reachable
  without passing a reach the rover waits where it is safe (the existing "waiting" state, said in the picture).
  Outside a region the rover may still pass through (its path is the engine's), but it looks at nothing there and
  attacks nothing there (`ATTACK_RADIUS` targets outside the region are left; the user's "stays in his half").

### 3.3 Regions are marks

`mark` takes a cell range as well as a point: `{"east_half": "E1-H8"}` is the rectangle from column E row 1 to
column H row 8; `{"his_corner": "H6-H8"}`, `{"left_band": "A5-C8"}`. A region mark is a place (its `at` is the
centre, so `go_<region>` and the rest work as for any mark) with `region: Some(rect)`, said in the picture's places
("east_half: the cells E1-H8, 14 spots inside") and in the scouting block's words. "His half" is for the player to
mark; the bot does not guess a half.

### 3.4 What is not proposed

- **Straight to rove from `produce`.** With `"group": "by_type"` the Rovers form a group of their own and the pick
  puts it on rove within seconds (no-lever-batch-smoke: group_A formed at 1:47, roving at 1:50). A newcomer group
  born roving would be a task set with no pick, the shape the rove-approval rework deleted. Not built unless the
  three seconds prove to matter.
- **A route for a rover** ("to spot_37 and spot_12", "spot_12 and on to H1"): `first` is one place; a second is
  the next packet's or the next pick's (`rove_spot_12` after spot_37 is seen). The route task of the no-lever list
  (`way`) is a separate design.
- **A thing to look for.** Covered: the rove walks his perimeter every minute, the Found events and the scouting
  block report what it sees; a wake when a rover finds a factory or his commander is a `wait` flag, cheap, not part
  of this.

## 4. Costs and risks

- Menu: one to three moves per group per gate (the 0.5 bar), none on groups the packet has not named. Thirty-odd
  if the bar is dropped.
- A wrong pick between `go_spot_12` and `rove_spot_12` under a packet that says "walk to spot_12": the words differ
  ("walks to" against "roves from ... and rove from there"); measured offline before a game (§5).
- A region that strands a rover: every goal inside behind a reach means it waits where it is safe, as a rover with
  no clear way does today; the picture says "waits: every way to a place worth a look inside east_half passes a
  reach".
- The `first` place walking a rover into a reach: the clear-way test and the give-up apply as to any goal.
- Code: menu.rs (the family, ~60 lines), execute.rs (the beat), groups.rs (`Beat`), micro.rs and
  `crates/micro/src/rove.rs` (`rove_beat`, the filter and the forced first, ~50 lines), picture.rs (words), mcp.rs
  (`mark` ranges, the place's region), the brief's rove passage; tests for each. About a day.

## 5. What to measure

Offline first, by the roving-menu study's method (`docs/studies/2026-10-08-roving-menu.md`, `run/jev_ab.py`): the
recorded gates of the eight games' rove asks replayed with the family on the menu, counting the picks of
`rove_<place>` and `rove_1_<place>` under packets that asked for a destination (should be taken) and under packets
that asked for a walk or a plain rove (should not). Then a player game with the brief's rove passage updated, read
by `run/impedance_sweep.py` and the study's brief: the rove rows of §4 should fall off the no-lever list.

## 6. The user's decisions

1. The 0.5 bar on the family (a withheld-options rule in the shape of the `places` layer; `docs/code-decisions.md`
   §3), against every named place, against a decode reading on the plain `rove`.
2. The two meanings: a point is looked at first and then free roving; a region is a hard scope. Or one meaning for
   both (a soft beat: prefer, not restrict).
3. `rove_1_<place>` in the family (the "one Rascal alone to spot_12" shape, four of the eight games), at one more
   move per read place.
4. Regions as `mark` ranges.
5. Nothing from `produce`.
