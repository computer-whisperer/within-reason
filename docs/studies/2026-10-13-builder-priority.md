# Builder priority: the mechanics, and what the pros use it for

Written 2026-10-13 (the user: experienced players say "it is occasionally important to configure resource allocation
priority settings so that the important structure gets metal and energy when there is a shortage"; look into the game
code first, then catalog the pro games). Sources: the engine checkout `upstream/RecoilEngine` (the arena's 2026.07.04
line), the game checkout `upstream/Beyond-All-Reason` (54199a0d, 2026-09-19), the 75 kept pro duels of
`run/data/replays/manifest.jsonl` (Comet Catcher 68, Great Divide 4, Full Metal Plate 2, Gasbag Grabens 1; 41 players
at OS 25-51; game versions test-30922 to test-31405). Tools: `run/replays/priority.py` (the demo's packet stream, the
join to the replay records, the stall tallies) and the replay dump widget's new `bp` sample field
(docs/harness/record-format.md). Data: `docs/studies/2026-10-13-builder-priority/` (`toggles.jsonl` one row per hand
toggle and builder with its context, `widget-defaults.jsonl`, `games.jsonl`, `tallies.md` with every toggle in a
table, `stalls.md`).

## 1. The engine has no priority: creation order, whole step or nothing

Read from source; the one-second samples of the replays neither show nor refute the order (§5), and a per-frame test
is proposed in §6.

- **A build step is all or nothing.** Every frame, each builder or factory with a target calls
  `CUnit::AddBuildPower(builder, buildSpeed)`: the step is `buildSpeed / buildTime` of the target, its price the
  target's cost times the step, and the team either has the whole price (`CTeam::HaveResources`: `res >= amount`, both
  resources) and pays it, or the step is skipped and the demand is noted in `resPull`
  (`rts/Sim/Units/Unit.cpp` `AddBuildPower`, `rts/Sim/Misc/Team.cpp` `HaveResources`/`UseResources`). No partial
  steps, no sharing out.
- **First come, first served, in creation order.** `CUnitHandler::Update` runs every unit's `Update` in the order of
  `activeUnits`, which is `push_back` on creation (the randomised insertion is `#if 0`,
  `rts/Sim/Units/UnitHandler.cpp`). Income lands before that in the same frame (`SlowUpdateUnits` then
  `UpdateUnits`), from a fifteenth of the units each frame (`UNIT_SLOWUPDATE_RATE` 15). So within a frame the oldest
  builder takes its step first, and a cheap step (an extractor's, a solar's) slips through where an expensive one (a
  tier-2 lab's, a fusion's) does not: the expensive build starves first, whoever is building it.
  K-mech-upkeep-has-no-priority (docs/knowledge/mechanics.md) already measured the same queue for energy upkeep.
- **A stalled target does not decay** while a builder keeps trying: `lastNanoAdd` is stamped before the resource
  check. A builder whose build speed is zero never calls `AddBuildPower`, so its target decays: that is why the game's
  gadget lends one throttled builder per target a speed of 0.001 for one frame an interval.
- For the bot this means: with no priority flag set, our commander (the oldest unit) wins every frame's stall, then
  the first plant, then the constructors in the order they came out; the plant's step is dearer than a constructor's
  extractor step, so the plant is the first thing a metal stall slows. Code-read; over a second the samples show
  every high builder's target moving in 97 % of stall seconds (§5), so the per-frame order may matter less than it
  reads.

## 2. BAR's priority is a flag on the builder, never on the structure

`luarules/gadgets/unit_builder_priority.lua` ("Builder Priority", once "Passive Builders v3"), synced, on since 2024:

- **The command.** `CMD_PRIORITY` = 34571 (`modules/customcommands.lua`), an `ICON_MODE` command with one parameter:
  0 "Low Prio", 1 "High Prio"; the default is 1. It is inserted on every unit whose definition has `buildSpeed > 0`
  and (`canAssist` or build options): commanders, constructors, factories, construction turrets. Not rez bots.
- **The state** is the unit rules param `builderPriority` (0 or 1; unset means high). Readable by widgets, and by
  an AI through `Unit_getRulesParamFloat` (the C interface, `SSkirmishAICallback.h:1118`; the engine's own rules
  param `GetUnitRulesParamLosMask` gives a team's own units in full).
- **The command is consumed, not queued.** The gadget's `AllowCommand` records the flag and returns false, so the
  command never enters the unit's queue, the unsynced `UnitCommand` callin never fires (`CCommandAI::GiveCommand`
  calls `AllowCommand` first) and no record or widget sees it: the demo's packet stream is the only place a toggle is
  written. (So the replay dump widget samples the rules param instead, `bp`.) A unit without the command in the same
  selection gets the command too and the gadget lets it through to a queue that drops it: 10 such in the 75 games.
- **Low means: only out of surplus, full speed or nothing.** Every interval (1 to 6 frames: the frames a team's
  income takes to fill its storage, capped at 6) the gadget takes the team's resources and computes, per resource,
  `stored − max(0.2 × income, 0.01 × storage) − 1 + interval × (income + received − sent − the high-priority
  builders' expense)`. Then it walks the low-priority builders that have a target (Lua `pairs` order: arbitrary,
  stable) and grants each its whole expense for the interval if the figure stays positive, else sets its build speed
  to 0 (`Spring.SetUnitBuildSpeed`). A low-priority builder therefore runs at full speed or stands still; it never
  takes what a high-priority builder would use in the next few frames, and it never competes with another low builder
  on even terms (the first in `pairs` order takes what there is).
- **High changes nothing.** The gadget only ever throttles low builders; among high builders the engine's creation
  order of §1 still decides. "The important structure gets the metal" is done by setting every *other* builder low,
  through the builders, not on the structure.
- `suspendbuilderpriority` (a team rules param) turns the gadget off; only the PvE cheat gadget sets it.

## 3. The stock UI: labs and construction turrets are low from birth

`luaui/Widgets/unit_builder_priority.lua` (on by default; the three settings are in the options menu): at every
builder's `UnitCreated` and again at `UnitFinished` it sends the unit `CMD_PRIORITY` with the class default,
**labs 0, construction turrets 0, constructors 1**; the commander is never touched. So in every human's game the
baseline is: the plant and the turrets yield in a stall, the commander and the constructors do not. A player
changes a unit by hand with the Low Prio / High Prio button of the order menu on the selected units (the engine's
`SetActiveCommand` path, sent as `NETMSG_COMMAND` on the selection), which is how the packet stream tells a hand
toggle from the widget's (`NETMSG_AICOMMANDS`, aiID 255, from `Spring.GiveOrderToUnit`).

**No AI gets the widget.** It runs in the player's own LuaUI for `myTeamID`; an AI team has no LuaUI. So our bot's
labs and turrets have always been high priority, BARb's too (CircuitAI defines `CMD_BAR_PRIORITY 34571` and never
sends it; Apex does not name it). Our plant competes with our constructors by creation order, where a human's
yields: a difference from every human game we have read, hidden until now.

## 4. Our bindings

- **Sending.** The C interface's `COMMAND_UNIT_CUSTOM` (78) with `SCustomUnitCommand { unitId, groupId -1,
  options 0, timeOut, cmdId 34571, params [0 or 1], params_size 1 }`
  (`crates/recoil-ai-sys/vendor/rts/ExternalAI/Interface/AISCommands.h:1425`), through `Engine::handle` in
  `crates/ai-host/src/engine.rs` like every other order. The engine sends it as `NETMSG_AICOMMAND`;
  `CSelectedUnitsHandler::AINetOrder` checks the team and calls `CCommandAI::GiveCommand`, whose `AllowCommand` the
  gadget answers: the flag is set, nothing is queued. Built the same night on the user's word: `Command::Priority
  { unit, low }`, the record's `["priority", unit, 0/1]` row and `bp` field, H-HANDS-PRIORITY-BIRTH and
  H-HANDS-LIST-LOW in the hands (`pianist/priority.rs`), D-PRIORITY in `bots/distilled`; the games prio-smoke-*,
  prio-player-1 and prio-distilled-*-comet-sym50-* in docs/experiments.md.
- **Reading.** `Unit_getRulesParamFloat(unit, "builderPriority", 1.0)`; `Game_getRulesParamFloat` is already bound
  the same way in `engine.rs` (the metal spots).
- The user's word (2026-10-13 evening): `low` as the player's one control word on a list step, the birth defaults
  (plants and turrets low) as an audited layer in `docs/code-decisions.md` §1, and the same in the heuristic bot.

## 5. The catalog of the pro games

The 75 demos were replayed 2026-10-13 20:08-22:40 with the dump widget sampling `bp` (ten engines, about 1.5 times
game speed; the earlier replay directories and cards were gone from `run/matches`, so the cards are re-cut too), the
hand toggles read from the packet streams and joined to the records by `run/replays/priority.py catalog`.

**The defaults are the setting.** All 41 players run the stock widget with labs low (41 of 41) and turrets low (40 of
41; I_AM_SKILLISSUE has turrets high); five (TehHardStuckOS25erer, [APM]ieNemo, [AMADO]epienriz, PRO_Jamerz,
RAM_Noctis) have constructors born low as well. **Nobody changes a lab or a turret by hand: 0 of 172.**

**134 hand toggles (172 builder-toggles) in 47 of 75 games, by 18 of the 41 players**; the median game has one;
TehHardStuckOS25erer's nine games carry 84. Two uses cover 133 of the 172:

1. **The commander goes low while it assists or builds energy** (52 toggles, 14 players, OS 25-51). Its job at the
   toggle: assisting the plant (a guard on the lab, or building the lab's unit) 30, a solar 7, the plant itself 3,
   idle 9. When: minute 1 in 21, minute 3 in 10, 42 of 52 before 5:00. Metal under 10 % of storage in 23 of 52,
   and the constructors on extractors in nearly every row. The reading: in the opening stall the commander's assist
   (the largest build power on the team) would take the constructors' metal; low makes it a sink for the surplus,
   and the extractors go first. 12 of the 52 are set high again later, median after 90 s, when the commander takes a
   job of its own: the 12 commander → high rows are an LLT, a radar, a lab, solars. thebluegecko (OS 48.5, 17
   games): 10 of his 12 toggles are the commander low, at 1:09-6:17, each time with the commander on the plant or a
   solar and his constructors on extractors.
2. **The constructor that expands goes high, in the school where constructors are born low** (81 toggles, 5 players;
   65 by TehHardStuckOS25erer, 7 ieNemo, 5 epienriz). Its job: nothing yet in 50 (a constructor just ordered:
   `build cormex` the last order in 29, 22 of the 81 under 30 s old, 10 still a nanoframe), an extractor 17, an LLT
   7, finishing the plant's unit 5. The pattern: every builder low, and the one carrying the next extractor (or the
   LLT, or the plant's unit at 80 %) flipped high as it gets the order. The important build is named through its
   builder; everything else waits.

The rest: **a constructor low** (27): assisting the plant (12: `guard armvp`, `repair`) or on a solar while the
others are on extractors, the commander's reading again (Baldric 8, Artur91 5, gecko, fins, modulo); mis-clicks (a
low then a high on the same units within a second, three pairs).

**What low does in play** (`stalls.md`: 16,299 one-second samples with metal under 2 % of storage, energy over 20 %,
the team spending; lone builders on a nanoframe, throughput against the same builder-and-target pair's median speed
with metal over 20 %): a high commander keeps 88 %, a high constructor 102 %; a low commander 45 %, a low constructor
78 %, a low turret 46 %; a low lab 106 % of a reference its own 2-20 % band puts at 148 %, so about 72 % of its
running speed (the factory's reference is unreliable: likely units at 100 % waiting to roll out; not chased). Outside
a hard stall the flag costs nothing (98-101 % at 2-20 %). The lab progressed in 81 % of hard-stall seconds and 99 %
otherwise. So the low flag costs its holder about half its speed in a hard metal stall, and the pros put it on the
builders whose work can wait a few seconds: the plant, the turrets, the commander's assist.

**The engine's creation order, watched** (§1): in the 193 stall seconds where one lone high builder's target
progressed and another's did not, the oldest builder's progressed in 124 and the youngest's in 114: no order visible
at one-second samples (steps are per frame, income arrives in lumps, and in 6,893 of the 7,086 such seconds every
high builder's target moved). The claim stays code-read.

### What this says for the bot

- Our builders are all high, all game; the engine gives them creation order, and over a second that mostly means
  everyone moves. Whether the plant's competing costs us anything is a measurement on our own records (metal stalls
  with the plant's unit and a constructor's extractor both open), not yet made.
- The pros use two levers, both per builder, both about once a game, both in minutes 1-5: the commander low while it
  assists in the opening (on top of labs and turrets low always), and, in one school, the constructor carrying the
  next extractor high with everything else low.
- The way in is one `Command` variant through `COMMAND_UNIT_CUSTOM` (§4). Who decides is the user's call under the
  no-live-heuristics ruling: the birth defaults (plant and turrets low) are a candidate for an audited default layer,
  "what every human's game has"; the commander low while assisting is a candidate for a player tool or an option on
  the assist move.

## 6. A test of §1 (proposed, not run)

The duel harness can settle the creation-order claim in one cheat game: spawn two constructors a few seconds apart,
give each a build of the same cost with metal at zero and income small, and read which target's progress moves
(records carry the targets' health). The same game with the older constructor set low shows the gadget's throttle.
