# Code decisions in play

The roster of what code decides in a live game without a pick by Jev or an order from the player. The rule
(`design/2026-09-28-no-live-heuristics.md`): a fix for a wrong move is information or an option for Jev, never a gate
in code; the micro engine is the standing exception. A code decision above the micro engine is added only on the
user's explicit word, as a layer with a switch and an audit, and is entered here in the same commit. The user,
2026-10-04: "We should refresh our documentation to keep a roster of the ones currently in play."

State: audited against the code on 2026-10-04 (commit `779b040`), every deciding line read by a review agent and
the constants of §1 to §6 spot-checked (26 of them, all as written). `P/` is `crates/bot/src/brain/pianist/`, `B/`
is `crates/bot/src/brain/`, `M/` is `crates/micro/src/`. Keep it current: a constant changed, a rule added or
removed changes its row here.

## 1. Sanctioned by name

| Decision | Values | Code | ID | By whose word |
|---|---|---|---|---|
| A builder's move at a metal spot is shut while two or more free builders of its type reach the spot sooner (not for a builder the packet names or one with a party beside it). Off unless `near` is among the hands' layers | `NEAR = 2` | `P/layers.rs` `near`, `P/mod.rs` `pass` | H-HANDS-NEAR-BUILDER | the user, 2026-10-04: "this is a code decision heuristic that I want to try" |
| An idle construction turret guards the nearest finished factory in its reach, told again every 10 s | `RETELL_FRAMES` 10 s, `FACTORY_REACH_SLACK = 100` | `B/nanos.rs` | H-ECO-NANO-GUARD | the user, 2026-09-28: stays as a mechanic |
| An idle resurrection bot works the best safe wreck field, raises a soldier's wreck worth 100 or more while energy is over half, walks home with nothing to do | `METAL_PER_WORKER = 300`, `QUEUE = 5`, `RAISE_MIN_METAL = 100`, `RAISE_ENERGY = 0.5` | `B/reclaim.rs`, `P/mod.rs` | H-REC-CREW, H-REC-RESURRECT | the user, 2026-09-28: stays as a mechanic |
| A new soldier joins its plant's group and is walked to the body | `ADOPT_RADIUS = 400`, `FOLLOW_STEP = 150` | `P/groups.rs` | H-HANDS-GROUPS, H-HANDS-JOINERS | the user, 2026-09-28: the newcomer walk stays |
| Every plant and construction turret of ours carries the game's builder-priority flag low (it draws only the team's surplus, full speed or nothing), as every human's do from the stock widget's birth defaults; the flag is read back each tick and set when it differs | none | `P/priority.rs` `tend_priorities` | H-HANDS-PRIORITY-BIRTH | the user, 2026-10-13: "the changed default sounds like a good thing to try" (the study `docs/studies/2026-10-13-builder-priority.md`) |
| The micro engine (§5) | | `M/` | H-MICRO-* | the ruling's own exception |

## 2. Orders given, changed or ended with no pick

| Decision | Values | Code | ID |
|---|---|---|---|
| Arrival turns a walk into a hold (the group is then idle and asked); a gather ends when the tail is up | `ARRIVED = 300`, `BODY_SHARE = 0.6`, `STRUNG_OUT = 600` | `P/groups.rs` | none |
| A group told to attack is sent again when its party has moved 150; a party unseen 6 s puts it on hold; an air group searches once at 6 s and holds at 60 s | `LOST_FRAMES` 6 s, `AIR_LOST_FRAMES` 60 s | `P/groups.rs` | H-HANDS-ATTACK-FOLLOWS, H-HANDS-AIR-TARGET; the 6 s hold has none |
| A follower is sent after its ward each 150; the ward gone, it holds | | `P/groups.rs`, `P/mod.rs` | H-HANDS-FOLLOW |
| A build the engine holds without starting (the builder standing still within reach of the site 8 s, no frame of the type there, never idle) is stopped, its site refused 90 s and its list step put back, so the order goes again elsewhere (the user's word, 2026-10-08, after human-27 2:08-2:54) | `BLOCKED_WATCH` 8 s, `BLOCKED_MARGIN = 120`, `BLOCKED_STILL = 0.2` | `P/mod.rs` | H-HANDS-BLOCKED-SITE |
| A hunt ends when the quarry is dead, unseen 6 s (30 s for a quarry whose type cloaks: hidden where it stands, not gone, the hunters walking to its last place meanwhile; the end's word is "hidden"; the user, 2026-10-18: "a cloakable quarry out of sight is hidden, not lost", `docs/design/2026-10-18-sight-jammers-cloak-shields.md`) or a hunter is past the leash, measured for each hunter from where it stood when the hunt began (from the hunters' centre until 2026-10-16: 174 of 395 hunts in the impedance tally ended on their first tick); the failure is remembered 3 minutes | `HUNT_LEASH = 900`, `HUNT_LOST_FRAMES` 6 s, `HUNT_HIDDEN_FRAMES` 30 s | `M/lib.rs`, `P/groups.rs`, `B/micro.rs` | H-MICRO-HUNT, H-HUNT-HIDDEN |
| A builder attacking a party is sent again as it moves and stopped when the party leaves the picture | | `P/mod.rs` | none |
| A task is dropped by the clock: a build not started 1.5 s after its order is refused; a started job found idle is dropped; a walk within 150 is done | `ORDER_GRACE_FRAMES = 45` | `P/mod.rs` | H-HANDS-REFUSED (the refusal) |
| The player's lists, timed by code: the next step when the builder is free or its build is 60% done; an undoable step skipped; `assist` means the nearest factory; a bare `extractor` the soonest free spot | `QUEUE_AT = 0.6` | `P/lists.rs`, `P/execute.rs` | H-HANDS-SCRIPT, H-HANDS-LISTS |
| A factory makes its `produce` entries in order without asking | | `P/lists.rs` | H-HANDS-PRODUCE |
| A pick replaced at execution: a picked factory becomes help on a frame of that type within 900; a tier-2 extractor under way becomes a repair | `FRAME_HELP = 900` | `P/execute.rs` | H-HANDS-STARTED, H-HANDS-UPGRADE |
| Who and what inside a move: a detachment is the fastest armed, nearest first; the quarry the party's nearest unit; the scout the fastest; an air target his commander, else the dearest; the larger group keeps its name in a join | | `P/menu.rs`, `P/groups.rs`, `P/execute.rs` | mostly none |
| Shelling footwork: artillery to 0.85 of its reach, the screen to 0.55 | `STANDOFF = 0.85`, `SCREEN = 0.55` | `P/execute.rs` | H-HANDS-GROUP-STATES |
| A group picked to rove (`rove`, or `rove_1` for its fastest soldier as a group of its own) is the lane's from the next tick, and any later pick takes it elsewhere (the player's own `rove` setting, the one unit-control order with no pick, deleted 2026-10-07; the lane setting as the way back, a bypass in the other direction, deleted 2026-10-08) | | `B/micro.rs`, `P/groups.rs`, `P/execute.rs` | H-MICRO-ROVE |

| A list headed by `stop` drops the build in progress only when its frame was made before the report the player read; a frame begun after it finishes first | `Shared.last_turn_frame` against the frame's created frame (`Brain.frame_builders`) | `P/mod.rs` `take_lists` | the user, 2026-10-17 ("Go with the first"): a stop written against a builder seen walking met a frame begun during the turn in 84 of the 129 player drops of the unfinished-structures sweep; the hands' line says which it kept and what a stop dropped |

## 3. Options withheld before Jev sees them

| Decision | Values | Code | ID |
|---|---|---|---|
| No menu at all: a roving group; a builder on a list unless threatened (then its ways out only); a factory with a unit queued; anyone in the first 2 s | `THREATENED = 800`, `FIRST_ORDER_FRAME = 60` | `P/menu.rs`, `P/execute.rs` | H-HANDS-SCRIPT, H-MICRO-ROVE |
| Places a move may aim at: marks, and the spots, passages and home the packet or a list names; nothing within 300 of the actor; nothing its class cannot reach | `HERE = 300` | `P/menu.rs` | H-HANDS-NAMED-PLACES |
| Regions to rove within (`rove_<region>`, `rove_1_<region>`): only a marked region the decode reads as named for the group at 0.5 or over (the user, 2026-10-17: "the 0.5 bar sounds fine"; thirty moves a menu otherwise); without the `fuse` layer there is no reading and every marked region is offered (rove-region-smoke: the arena's default layers are `news` alone) | `ROVE_READ = 0.5` | `P/menu.rs` | H-MICRO-ROVE |
| Extractor spots: the named free ones and the one soonest reached; not free are another builder's, a team mate's claim, a spot refused in the last 90 s, a held one, ground nearer an ally's start for 3 minutes | `REFUSED_FRAMES` 90 s, `ALLY_GROUND_FRAMES` 3 min | `P/menu.rs`, `B/allies.rs` | H-HANDS-REFUSED, H-TEAM-ALLY-GROUND, H-TEAM-ALLIED-SPOTS |
| Building kinds: the `produce` allowance, else a fixed usual list with no converters; a construction turret only with a factory; a tier-2 extractor only over ours | | `P/menu.rs`, `B/roster.rs` | H-HANDS-PRODUCE, H-HANDS-ROSTER |
| Help only for factories and started builds; take apart only the first 3 wreck fields of 100 metal or more; repair only buildings and the commander under 70% | `WRECK_FIELDS = 3`, `FIELD_METAL = 100`, `DAMAGED = 0.7` | `P/menu.rs` | none |
| A group against a party: the whole-group attack withheld when we outweigh it four times with more than two armed, unless the decode reads every detachment of the ladder as forbidden by the instructions (then it is offered beside them; 2026-10-16); detachments of 1, 2, 4, 8 only; shelling only with artillery; a raid or shelling of a place only with his buildings within 500; no advance where his buildings stand | `WHOLE_GROUP_OVER = 4.0`, `LADDER = [1, 2, 4, 8]`, `HIS_AT = 500` | `P/menu.rs` | H-HANDS-DETACHMENTS-FOR-SMALL-PARTIES; the rest none |
| Air groups: no gather, advance, shelling, detachment, follow or scout | | `P/menu.rs` | H-HANDS-DOMAINS |
| A builder against a party: attack only if armed, D-gun only with a unit inside its reach | | `P/menu.rs` | H-HANDS-DGUN |
| What reaches the pick: 2 moves an actor, 255 worlds, 9 changes a component, clashing worlds dropped | `DEPTH = 2`, `CAP = 255`, `SPLIT_CAP = 9`, `LINE_CHARS = 40,000` | `P/compose.rs` | H-HANDS-ONE-PASS |
| What a party is: mobile enemies chained within 400; the party in an actor's entry is the nearest within 800 | `PARTY_RADIUS = 400`, `NEAR = 800` (menu) | `P/picture.rs`, `P/menu.rs` | H-HANDS-PARTY-NAMES |

## 4. Thresholds on Jev's answers and on when it is asked

The hands' layers (H-HANDS-LAYERS) are each a switch; which are on is set at launch (`--hands-layers`,
`run/human_game.sh`; unset is `news` alone; `again` and `near` are named explicitly).

| Decision | Values | Code |
|---|---|---|
| The gate: once a second at most; asked when the picture's signature changed, after 20 s, on an event no oftener than 5 s, at once on a place reached, a party met or an opening | `RE_ASK` 20 s, `EVENT_GAP` 5 s | `P/mod.rs`, `P/layers.rs` |
| `tick`: a gate no event calls for waits 2 s | `TICK` 2 s | `P/layers.rs` |
| Events: a party within 600 of a builder or group, a hit, a death; under fire lasts 3 s | `ALARM = 600`, `UNDER_FIRE_FRAMES` 3 s | `P/schedule.rs` |
| `news`: an actor with a course and no news is closed for up to 20 s | | `P/layers.rs` |
| `same`, `again`: an answer in the same words (numbers left out; for a builder or a plant, over the same counts) under 20 s old stands; after an opening, answers under 3 s old stand | `AGAIN` 3 s | `P/layers.rs` |
| `openers`: a move is asked only while its actor's `change` or its party's `answer` was at 0.5 | `FLAG = 0.5` | `P/layers.rs` |
| `places`: a place opens its moves at 0.3 and closes under 0.2 | `PLACE_BAR = 0.3`, `PLACE_CLOSE = 0.2` | `P/layers.rs` |
| `fuse`: a decode reading under 0.3 removes the move until the next packet (never the extractor move, 2026-10-16); the forbidden mark of a detachment, a rover of one, and the whole-group attack or shelling of a party of aircraft (the `Air` reading, 2026-10-16) is on at 0.8 and off under 0.2, by the highest of the move's readings | `FUSE = 0.3`, `ON = 0.8`, `OFF = 0.2` | `P/decode.rs` |
| What goes to the pick: the change and the move at 0.5; an idle actor's best at 0.3 | `FLAG = 0.5`, `IDLE_BAR = 0.3` | `P/compose.rs` |
| The pick: the candidate is sampled by probability, then asked against "nothing changes"; played at any confidence | | `P/compose.rs` |
| Audits: 2% of what a layer skips is asked anyway and never played | `AUDIT_SHARE = 0.02` | `P/layers.rs`, `P/decode.rs` |
| Failures: a failed gate leaves every course standing; in realtime an answer older than 3 s is dropped | `STALE_FRAMES` 3 s | `P/mod.rs` |

## 5. The micro engine (10 Hz, ground soldiers; the player's `lane` tool sets the rules a group)

| Rule | What it decides | Main values | ID |
|---|---|---|---|
| Threat set | Which enemies a unit stands against, from its group's task: an advance everything; a hold or follow mobile units; an attack his commander only at 450 metal of group | `COMMANDER_PARTY_METAL = 450` | H-HANDS-GROUPS |
| Flee | Steps out of an unpriced threat, or when fire would kill it in 1.5 s and the side is outgunned | `HORIZON = 700`, `EXPOSURE_SECONDS = 1.5`, `ODDS_TO_STAY = 1.2` | H-MICRO-FLEE |
| Fan | Sidesteps out of a D-gun's line | `DGUN_MARGIN = 260`, `DGUN_SHADOW = 48` | H-MICRO-FAN |
| Kite | Out-reaching by 80 and no slower, steps back while reloading | `KITE_MARGIN = 80`, `KITE_EDGE = 15` | H-MICRO-KITE |
| Form | Group orders become per-unit slots; at contact picks the body's target and stands on an arc at reach | `HULL_SPACING = 64`, `SPACING_MAX = 160`, `FILES = 6` | H-MICRO-FORM, -ARC, -SPACING |
| March (inside form) | Forms up ahead, then ranks at the goal | `FORM_UP_SECONDS = 3`, `FORM_UP_LEAD = 300` | (H-ARMY-MARCH is deleted) |
| Pursuit let-go | Lets a chased target go | `PURSUIT = 600`, `LET_GO_FRAMES` 30 s | H-MICRO-FORM-PURSUIT |
| Hunt | Attack by id each tick; its ends are in §2 | | H-MICRO-HUNT |
| Rove | Evade, else kill what is unguarded, else look at the least-known place | `ATTACK_RADIUS = 1000`, `KILL_SECONDS = 45`, `LOOK_SHARE = 0.6` | H-MICRO-ROVE |

## 6. Where a building goes, and when the player is woken

Placement (`B/economy.rs`, `B/yards.rs`, `P/execute.rs`; H-ECO-BASE-LAYOUT, H-ECO-YARD-LANE, H-HANDS-DEFENCE-FORWARD): a
factory 350 ahead of home, an armed building on a line 650 ahead, generators in the back field, a stockpiler (a
nuclear silo, an anti-nuke: armed, but it reaches the whole map) in the back field too (the user, 2026-10-09), anything
else beside the builder turned in 45° steps until clear; factory exit lanes 320 deep kept free; a defence at a named place 120
toward his base; an extractor inside its patch nearest the builder; another builder's unstarted build kept out of
every site search by the two footprints (2026-10-08, H-HANDS-REFUSED).

Wakes (`B/wake.rs` and the triggers beside it; H-WAKE-*, H-HANDS-LOSS-WAKE, H-HANDS-STALL, H-HANDS-SHELLED): no turn
within 5 s of the last; the player's own timer (default 30 s) capped at 10 s while a fight is on; losses while orders
were on their way; a group that lost 2 soldiers or a quarter of its metal; an advance stalled 45 s; no new extractor
for 4 minutes with free spots; safe wreck metal each 500; 2 extractors lost in a minute; his first tier-2 or air
unit; shelled from out of sight 5 times in 15 s; a factory lane blocked 30 s.

## 7. Found by the audit, not yet mended

- Decisions with no ID in the registry: the arrival hold, the 6 s lost-party hold, the builder's attack re-send and
  stop, the task drops, who is picked inside a move, the help/take-apart/repair filters, the detachment ladder, the
  raid-not-advance rule, `HERE`, the stale-answer drop, the stagnation wake.
- Registry rows whose status opens "active" over code that is gone: H-HANDS-FORBID, H-HANDS-TURRETS-WITH-A-PARTY,
  H-ARMY-MARCH, H-HANDS-LAB-DEFAULT, H-HANDS-STEP-AWAY, H-HANDS-ESCORT, H-HANDS-ENGAGEMENT-PLAN,
  H-HANDS-ENGAGED-PICK-BAR, H-HANDS-SCOUTS-FIGHT-NOTHING-ARMED.
- Rows marked retired over code that runs: H-HANDS-MENU, H-MICRO-HUNT, H-REC-CREW.
- H-MAP-ENEMY-START and H-MAP-ENEMY-CLUSTER still feed the base guess that the placement rules use.
- The lane word `march` is carried and read by nothing; the march runs under `form`.
