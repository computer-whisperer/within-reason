# The changes the games with people call for: to be made together, then the arena resumes

Status: APPLYING from 2026-09-27 evening (the struck items below were removed by the user's ruling before application began: "we want to empower jev and the player to work the game, and we want limited rigid requirements or heuristics that fight that when the original problem may be resolved by something else we changed"). Before that: PROPOSED, nothing applied. The user, 2026-09-27: "I don't want to start making changes until we fully
understand everything we can from these matches. Once we understand all the mistakes we can see, we can apply the
changes at once and resume arena matches." The findings are in `docs/studies/2026-09-27-jev-posing/README.md`
(six games mined; the expansion study `expansion.md` is being written and will add to this). The code, rules and
brief stand at 6151a89. Each change names the finding it answers (the study's numbering: I = missing information,
A = missing action, W = misleading words, X = unfortunate interaction) and the games it was seen in. The user's
frame: the answers are information and offered actions, not hard limits or enforcement.

## 1. The picture: what a group is (I1, the costliest mechanism, seen in every game) — APPLIED 2026-09-27 evening (H-HANDS-GROUP-BODY), with 9b.6 and 6.6 (e)

1.1 **A group is a body with a front and a tail, not a point.** Every group entry says where its front is (the
member nearest the nearest enemy), where its tail is, how long it is, and how many of its members are in contact
(within their own reach of an enemy, or under fire). Distances to places and parties are given from the front for
what is ahead and from the tail for what is behind. `enemies_near` names every party within NEAR of any member, not
of the centroid, and says which members it touches. The odds of a fight are priced for the part that is in contact
when the group is strung out, with the words saying so ("17 of 48 are in the fight, 31 are 700-2,900 behind").
Files: `pianist/picture.rs` (the group entry, `party_words`, `NEAR`), `pianist/groups.rs` (a `front()`/`tail()`),
`pianist/plan.rs` (states priced on the front).
1.2 **"N to go" and "advancing/walking" from the front**, with "stalled" only when the front has not moved; a
group whose units hold formation slots at home is said so, not "stalled" (Cape Violet 5:17).
1.3 **Health as a distribution**, not "full on average" at 22 of 39 lost (game 9): "N of M at full, K under half,
L lost since the last orders", with the loss count running from the last orders, not the last turn (I6).
1.4 **A plant's output is said as a body**: the automatic gather-then-walk is ~~STRUCK~~ (2026-09-27, the user: empower Jev and the player; no rigid rule where another change resolves the cause): 6.2's gather state covers the action when Jev or the player wants it. Kept: the picture says "N of its soldiers are on the way
from plant_X, K behind" (game 3 11:34, game 9 15:44). Files: `pianist/groups.rs` (join), `execute.rs`.

## 2. The picture: the other seats (I2, games 4, 6, 9) — APPLIED 2026-09-27 evening (H-HANDS-SIDE-PARTIES, H-HANDS-ALLIED-GROUPS)

2.1 **Every seat's picture carries the other seats' groups** as `group_A_t2` entries with position, size, task and
what they engage, so "together" is a fact Jev can see, and the packet's "as one body" has something to hold to.
Files: `strategist/seats.rs` (the shared hands already merge; the per-seat picture takes the others' group lines
from `shared.hands`), `pianist/picture.rs`.
2.2 **One name for one enemy party across seats.** Parties are named by the side, not the seat: a party seen by two
seats has one name in both pictures (party ids keyed on the member ids, allocated in `Shared`). Files:
`pianist/threats.rs` (party naming), `strategist/shared.rs`.
2.3 **The combined odds**: when another seat's group stands within its reach of the same party, the odds line says
"with group_A_t2 beside it, N metal: we outweigh it" and the whole-group attack state carries the same.
2.4 **The standing tool checks party names against every seat's sightings** (the engage refusals: games 3, 4, 6, 9).
File: `strategist/mcp.rs` (the standing branch reads `hands_merged` parties, not the lead's `enemy.in_sight`).

## 3. The report: the numbers (I3, I6) — APPLIED 2026-09-27 evening (H-PLAYER-TRADE-ONCE: 3.1, 3.2, 3.4; 3.3 was one source already, 3.5 is covered by the group entry's losses since the player's orders; the ledger rows' correction still to do)

3.1 **One count of enemy deaths across seats**: the merged `traded` and `traded_3_min` dedupe by unit id (the
seats' `enemy_destroyed` events name the id), so the trade line is true. Every ledger row's trade figure of
tonight is re-derived from the records and corrected (a separate cleanup).
3.2 **An income estimate for the opponent** from what of his we have seen die and what stands ("he has lost 37k by
13:24: an income of about 46 a second, about 18 extractors"), beside "known to hold N" with the age of the count.
3.3 **The score and eco extractor counts agree** (one source), and the seats line carries each seat's count so "no
growth: 12" is never read as the side's.
3.4 **Builds in progress show their percentage** in the actor entry and the report ("armavp at 45 %, one builder,
about 200 s at this rate"), and the seats line says the seat's metal beside the build so the player sees a plant
starving on one seat with metal banked on another (games 7, 9).
3.5 **Wake lines say the loss since the last orders and since the turn before**, never a per-turn count that resets
(games 4, 7, Cape Violet).

## 4. The odds: what shoots (I4)

4.1 **Reach, speed and tier beside the metal** in every odds line for groups (the commander lines have reach already;
the reach addendum of d265666 is a start): "3 Bulls (tier 2, reach 460, 950 metal each) against 13 Stouts (reach
350): even by metal; they outrange us by 110 and outclass tier 1". The matchup table gains tier-2 and turret rows
from the duel harness (an arena job) so the metal verdict itself is right.
4.2 **A party's speed is its slowest member's for "can we catch it" and its fastest for "can it catch us"**, and an
unidentified contact is not priced as a Pawn for speed (Cape Violet 10:20, game 7).
4.3 **The unseen shooter is in the odds**: a group under fire from out of sight has the estimated shooter (range,
kind, likeliest place) counted against it in `enemies_near`, not only on `under_fire` (game 3 19:30).
4.4 **The enemy commander as a fighter**: its D-gun reach and its death blast (radius, damage, what of ours stands
inside) in the party line whenever it is in a party, and the odds words "it D-guns anything within 262; its death
takes everything within 350" (game 7: eleven Incisors, then thirteen Brutes).
4.5 **Ground groups are never offered against air parties**, and a mixed party's air and ground parts are named
apart, with "nothing in this group hits its air" said (game 3 14:21).
4.6 **Turrets covering a party**: their reach named in the group's line as it is in the commander's.

## 5. The enemy: where and what (I5) — APPLIED 2026-09-27 evening (H-HANDS-ENEMY-MEMORY: 5.1, 5.3; 5.2's scouts are groups with entries already, 5.4 left)

5.1 **Last-known positions with age** for every enemy party that left sight ("party_46 (10 Stouts, 3 Janus) last
seen 40 s ago at F3 heading north"), and for the block: "his main block, N units, last seen at X, T ago".
5.2 **Scouts are actors**: a scout plane or car is an actor entry with a route the packet can name, its progress
said ("at spot_36, next spot_19"), and its idleness said (game 3: three Blinks idle at home for 17 minutes in no
entry at all).
5.3 **A line the moment a tier-2 or air unit of his is first seen** ("first Bull seen at 15:16 at D4"; "first Liche
at 17:02"), in the report's front and as a wake reason.
5.4 **The resurrection of our wrecks** said when his rez bots are seen near a wreck field (game 4).

## 6. The states: what a group can do (A1, A2)

6.1 **Against a raid: "stand at the next extractor on its heading"** as a state, with the raid's heading in the
party line ("heading south-east along F6-G6, next extractor spot_48 in 40 s"), beside the chase and the leave.
6.2 **"Gather at X before contact"**: the group holds at a place until its tail arrives (or N of M stand), then goes
on; the state says how long the gather takes.
6.3 **"Close on the shooter as one body"** for a group under unseen fire when it outweighs the estimated shooter;
and **"pull out of its reach"** to the nearest place beyond the shooter's range, with the walk said.
6.4 **The artillery state**: a group with long-reach units (Shellshockers 710, Mausers 820) is offered "shell X
from Y" with the screen standing between (game 9's four Shellshockers never used).
6.5 **The fall-back state says what is at its point** ("fall back to spot_10 (the Bulls are entering it)"; "no way back that is not into fire" when that is the truth, with the gather offered beside it). The three prunes (never where the group stands, never `shelling`, never a place a party is entering) are ~~STRUCK~~ (2026-09-27, the user: empower Jev and the player; no rigid rule where another change resolves the cause) (games 3, 6, 9).
6.6 **A group can be sent at buildings, and can sweep** (the user, game 10 at 5:58: "both top and middle seats
have sizable army groups in position and could have a field day with undefended structures but put no effort
into exploring ... why nothing takes the initiative with those groups"). Verified: over the whole game a group
was offered only keep, walks to passages/home/named places, splits, station, fall back, retreat, join, the
one-soldier scout, and through the party slots hunts and attacks on parties (all three jev logs, every group slot).
No state names a building: his buildings enter the picture only as `buildings_seen` per cell ("B3: 1 armllt, 1
armmex") and as "under N turrets" on a party. At 6:00 the north group (15 Incisors, 1,800 metal) was walking to
its station spot_9 with 22 of his buildings unguarded within 3,000 (his truth: 36 standing, 4 turrets); at 6:30 it
arrived and held, "fighting everything here, turrets included", for a minute with 27 unguarded within 3,000 and
the nearest an extractor 1,184 away; it killed nothing until the player moved its station by hand (spot_2 at
7:20, spot_4 at 8:00, spot_9 at 9:00). The player had asked at 4:30 for "group_A_t1 raids his north corner
(spot_9, then spot_2, spot_7, spot_14)"; the decompression kept the first place as a station and dropped the
rest. The scout state was played 5 times all game. Changes: (a) a `raid` state per group, on the nearest of his
buildings that are known (seen, or the extractor implied by a spot he holds), worded with the walk, the turrets
within reach of it and the metal it yields ("kill his extractor at spot_7, 1,184 away, 23 s, no turret within
450: 50 metal of his income"); making it the base world for an idle station is ~~STRUCK~~ (2026-09-27, the user: empower Jev and the player; no rigid rule where another change resolves the cause) (the missing piece was the option, not a default); (b) a `sweep` state: the group walks the never-looked spots nearest it in
order, as a body, with the fight on the way, and says which; (c) the `standing` tool's `station` takes a list ("spot_9, spot_2, spot_7, spot_14") the hands advance through as each is reached or found empty (the decompression path is ~~STRUCK~~ (2026-09-27, the user: empower Jev and the player; no rigid rule where another change resolves the cause), 13b);
(d) the enemy section lists his buildings by place with when seen and what guards them, not by cell.
Why the hands did not run the raid, traced (the user: "did Jev not have those destinations available?"): they
did not, on either path. (1) The per-second menu: a group's walk and split states go only to home and to places
that are not spots within 3,000 (`plan.rs`, "walks to named places": `p.name == "home" || p.spot.is_none()`), so
group_A_t1 was never offered a walk to spot_2, spot_7 or spot_14 at any second of the game (its states 4:30-9:30:
keep, walk/split to passage_1, passage_2, shelling, home, fall back, retreat, scout, join, and the one
`station_spot_N` its standing rule named). A spot reaches a group only as its `station`, one place. The `instruct`
tool's description promises "a spot or passage you name here is always on your hands' menu, however far": true of
builders (the extractor states), false of groups. (2) The decompression: its vocabulary for a group is station
(one place), station_mode, fall_back_to, never, no_chase, raiders_*, hold_line, engage_party, join; the station
question asks "where do they tell group_A_t1 to stand, gather, hold or be stationed?" with spot_9, spot_2, spot_7
and spot_14 among its options, and Jev answered spot_11 at 0.96 (jev-0 decompress row f8379). The 4:30 packet named spot_11 only as the place the raiders "fall back to their picket spot (spot_11)", so a question with no way to take a route was answered with the one standing place the paragraph had. Nothing
asks for a route, an objective or a target. So the player set `station spot_9` by the tool at 4:18 (outranking
the packet), and then advanced the station by hand at 7:10 (spot_2), 7:35 (spot_4), 8:16 (spot_7) and 9:02
(spot_13): the packet's "then" list was played by the player, one turn per step, 30-40 s apart, and at each
station the group held with nothing to hold against. Changes (c) and (a) above are the answer: a route in the
packet becomes a station list the hands advance, and a group at a station with a known unguarded building in
reach has the raid as its base world; add (e), ruled firm by the user ("definitely something to fix"): a group's
walk states include the spots the packet names for it, however far, as the tool's description already promises.

## 7. The packet and the standing orders (A3)

7.1 ~~STRUCK~~ (2026-09-27, the user: empower Jev and the player; no rigid rule where another change resolves the cause): the decompression is deleted (13b); there is no vocabulary to grow.
7.2 **A refusal never takes the actor's other rules with it**: the tool applies what it can and names what it
refused (game 6 16:42).
7.3 **A rename never refuses an order**: a party name from the last report resolves to the party those units are
in now (2.2 makes the names stable; until then, the tool resolves by member ids).
7.4 **A place accepted is a place** ("corsolar spot_25" accepted for a spot not in the picture, game 9).
7.5 **A spot named in the packet is an extractor candidate only where the packet names it for building** (the
user, game 10, watching the replay: "2:00 we send the first constructor out of the plant to a very far away mex
location on the wrong side of the map ... across all seats"). Verified: nothing is hardcoded to a side; every
Comet game with people had us in the east strip and the player's first packet named the east spots. The hands'
extractor states offer "every free spot the instructions name, nearest first" (`plan.rs`, `diet::names` on the
whole packet text), and game 10's packet wrote the seats' spots as "spot_10, 5, 6, 12, 17, 11, 3, 16, 18" (the
brief's own shorthand, docs/briefs/player.md "spot_2, 7, 14, 19, ...") so only `spot_10`, `spot_34` and `spot_72`
matched, all three already the commanders' list steps; the only other tokens were the scouting sentence's
"spot_36, spot_19, spot_45, spot_2". So the north constructor's slot at 1:36 held spot_2 (107 s of walking) and
spot_19 (115 s) and nothing nearer, the `job expand` default took spot_2 by rule at 1:37 (jev-0 f2880-2910), and
the middle seat's took spot_36 at 1:29 and spot_45, the person's start, at 1:35 (jev-2 f2640-2850). Game 9's packet
named the same four scouting spots but wrote the seats' spots in full, so the nearest named were near, and no
Comet game before 10 sent a constructor west (all nine records scanned). Changes: the offer always carries the
nearest free spots beside the named ones, each with its walking time, nearest first, and Jev chooses; and the
brief writes spot names in full and says why. The sentence classification ("a spot named in a sentence about
scouts is not a build candidate") is ~~STRUCK~~ (2026-09-27, the user: empower Jev and the player; no rigid rule where another change resolves the cause): it needs a reader of prose that 13b deletes.

## 8. The commander (A4)

8.1 **The D-gun as a state**: "D-gun party_N (within 262, energy 1,415)" with the energy said.
8.2 **Time-to-contact in every builder threat line and on every step-away place** ("Brutes at 87 against its 38: contact in 9 s if it walks
away, 4 s if it stays"; "step away to spot_5: contact before it arrives"), and "no way out on
foot: fight here with the D-gun" or the turret's cover when none is reached in time. Pruning the places it would not reach before contact is ~~STRUCK~~ (2026-09-27, the user: empower Jev and the player; no rigid rule where another change resolves the cause).
8.3 **Anti-air and evasion**: a builder under bombers is offered "under the flak at X" or "spread from the plant",
and the plants' lines say the flak and Nettles they could make when bombers are first seen.

## 9. The map and the sea (I7, A5)

9.1 **Water as it is**: the terrain read classes ground as walkable, wadeable (within each class's depth) and
deep; the map tool and the party lines say "coming through the E4 ford (wadeable)" and "in deep water at spot_20:
ships and amphibious units only"; the water note lists what of ours crosses (by the commander's roster, not the
lobby side's prefix), counts the under-water spots, and says what an enemy does from the sea.
9.2 **Sea actors and states**: a shipyard, construction ships and warships as actors; states for a ship group
(patrol a coast, escort, hunt subs) and for coastal weapons at sea-facing spots; a contact in deep water is named
as a sea unit, never offered to tanks to chase.
9.3 **"Not free" says why**: unreachable for this builder, held by the enemy, covered by wrecks, claimed by
constructor_N.

## 9b. Distance is walking distance, and reach is by class (the user, 2026-09-27, on Cape Violet: "movement was nonsensical due to the topology of the map. There are a couple of alcoves with mexes that are in the middle of things by euclidean measure, but not in terms of connectivity. Armies huddled off in odd places while the main fight was elsewhere, and on several occasions spent many seconds trying to reach terrain that was not traversable by that unit type")

Today the picture and the states measure almost everything by the straight line (`dist2d` in 56 places in `plan.rs`
alone against one use of the walking fields; those fields serve spots and home only: `walk_from_home`, `seconds_to_spot`), and a
walk state is offered to a group whatever its class can reach: the routes' fields exist per movement class from
home and per spot, not from where a group stands, and `snap_for` moves an unreachable target to the nearest ground
the class reaches without saying so. Not yet measured; the measurement to make on Cape Violet's records: the
seconds each group spent with a move target its class could not reach, or whose walk was over twice its straight
line, and the places it gathered at against where the fight was.
9b.1 **Every distance a decider sees is a walk for that actor's class**: "N away" and "N to go" in the picture, the
"K s of walking" in the states, the ordering of "nearest" places and spots, the raid answers' "reaches it in N s",
the fall-back and gather choices. A walk that is more than twice the straight line says so ("1,900 on foot, 700
across the water: an alcove that opens from the north"). Files: `pianist/picture.rs`, `plan.rs` (the walk words),
`routes.rs` (fields from a group's position: a coarse region graph per class, built at the survey, with the
region-to-region walk precomputed, so a distance from anywhere costs a lookup; the fine fields stay for spots).
9b.2 **A place a class cannot reach is never a state for it**, and the place's entry says which classes reach it
("ground from the north only; hover and amphibious from the water"). A group told to go where it cannot is told
so in the report ("group_A_t1 cannot reach spot_20: deep water; the nearest ground it reaches is X"), instead of
walking to the shore and standing (the picture's stalls of Cape Violet).
9b.3 **Gather points are chosen on the ways**: a gather or station for several groups is the place with the least
total walk for all of them by their classes, on the way to the objective, never the straight-line centre or the
nearest place to one group's centroid ("armies huddled off in odd places while the main fight was elsewhere").
9b.4 **The map tool says the topology**: the passages as now, plus the alcoves and pockets (regions with one way
in), which spots lie in them and from which side they open, and the wadeable fords apart from deep water (9.1),
so the player's read of "the middle" is by connectivity.
9b.5 **Mixed groups**: a group's reach is its slowest and least capable member's; a group with hovers and tanks says
"the tanks stop at the shore" and the hover part is offered as its own detachment across the water.

Measured (the Opus review `run/matches/1790481450-bluegecko-3v1-cape-violet/00/topology-review.md`, its two headline
claims re-checked against jev-2 f20310-20355 and jev-0 f17400-20300): the observation holds, but most of the cost
is not where 9b.1 puts it. Every walk or advance target a group was given sat on ground its class reaches; the
unreachable orders came from newcomers sent to the group's raw average position (in deep water when the group
was split between the spot_19 plateau and the shore below), from attack orders at parties in the sea (11:17: the
whole group_A_t3, 26 soldiers, sent at a floating radar "938 away, 13 s of walking" while 10 Welders killed our
Sentry at spot_25 "left to nobody" in the same list), and from formation or gate slots on cliffs: 726 unit-seconds
short of unreachable targets and 742 on cells too deep or steep. The two-minute huddle on the spot_19 plateau
(9:40-11:17, 1.8k and 1.6k of soldiers idle while seat t2 fought the Welders alone and lost 3,246 for 1,400) came
from arrival judged by the group's centre (a group split by a cliff never gets within 300 of its goal), so the
fall-back walk in progress stayed the base world and the player's stations spot_25 (8:47) and spot_23 (10:01) were
shown as the default 59 seconds and never played; and from "at spot_29 (E3)" said of a group standing on the
plateau, on which the player stationed everyone at spot_29 "(where they are)", 2,930 on foot through the fight.
Four spots sit in pockets with one way in: spot_19 (E2 plateau, opens west; the gather point the player chose
twice as "east of the bridge"), spot_18 (G2, opens east; 6,838 on foot from spot_29 against 1,605 straight),
spot_52 (D7, east), spot_53 (B7, west); spot_12 and spot_59 are islands. Ranked by cost: 9b.4 and 9b.3 first, then
9b.2, then 9b.1; 9b.5 had no case (every group one class). Added:
9b.6 **A newcomer joins its group at the nearest member on reachable ground, never at the arithmetic centre**;
**arrival and "N to go" are judged by the members, not the centre** ("16 of 21 arrived; 5 below the cliff"), and a
walk whose members have arrived ends, so the next default can play.
9b.7 **An attack on a party the class cannot reach is not offered**; it says "at sea, out of our reach; N of ours
outrange it from the shore at X" when some can.
9b.8 **Formation and gate slots are on reachable, standable cells** (the ford's own depth is the tank limit, so
slots at -18 to -24 along its edge jammed the group).

## 10. The words (W)

10.1 "Outruns this group ... a chase drives it off" only of a party that is moving; of one holding ground:
"holds its ground at X and shells". 10.2 "Out of its reach" only of a place the builder reaches before contact.
10.3 "The quarry is dead" only when it is. 10.4 `shelling` is never a place to walk to in any state's words.
10.5 A hunt that failed says so when the same units are offered again.

## 11. The crossing orders (X1)

11.1 **An order landing carries its time**, and the picture says "the player's order of 16:07 is in force, 12 s old" on the group's line (four reversals in 45 s, game 9). The landing grace (a pick never reverses an order that landed in the last N seconds) is ~~STRUCK~~ (2026-09-27, the user: empower Jev and the player; no rigid rule where another change resolves the cause): the reversals came from wrong odds on a strung-out group (1.1) and from the player not knowing what happened (11.2).
11.2 **The player is told what the hands did with each order in the next report** ("your advance of 16:07 stood
1 s; fall back picked at 16:08 on losses"), so a standing station shown as set is not read as being played
(game 6 9:52).
11.3 **A list step that repeats the build the actor was just given adopts it** (the user, game 10: "0:23 we start
a solar collector, then abandon it to build one immediately adjacent"). Verified in record-0: Jev's pick at f585
(19.5 s, "build a corsolar at home", 0.72) ordered the solar; the player's first turn landed its commander list at
f600 and its `corsolar` step re-issued the build 15 frames later; the executor skips a step only when the started
build's type matches (`started_def`, execute.rs), and nothing had started yet, so the fresh order planned a fresh
site 96 elmos away: the first nanoframe (18150, 1 % built) stood until it decayed at f909 ("abandoned an
unfinished corsolar", bot.log f1800). Change: a step naming the type of the actor's current order, started or not,
keeps that order (the list takes it over, as a started one is); a step with a site adopts a nanoframe of its type
within the placement radius.

## 12. Defaults that stack (X2, X5)

12.1 One default per actor per second: `raiders_party whole_group` and `attack_raiders yes` make one state the
default, not one per threat slot. 12.2 `hold_line yes` reads the front's odds (1.1). 12.3 A builder on a list is
offered the solar default when the store drains (game 7 8:00-10:00). 12.4 A station change ends a walk to the old
station (Cape Violet 8:05). 12.5 (the hunt's leash from where the hunt began) ~~STRUCK~~ (2026-09-27, the user: empower Jev and the player; no rigid rule where another change resolves the cause): marginal, no cost named.

## 12b. Transfers between seats (the user, 2026-09-27: "units and resources can be transferred between seats when humans are playing; not sure if we can do that, but we should check")

Checked, and we can. The AI interface has both commands: `COMMAND_SEND_RESOURCES` (`SSendResourcesCommand`:
resource, amount, receiving team; capped to the sender's store; "LuaRules might not allow resource transfers, AI's
must verify the deduction") and `COMMAND_SEND_UNITS` (`SSendUnitsCommand`: unit ids, receiving team; returns how
many went; "AI's should check each unit ... via UnitTaken() and UnitGiven(), since LuaRules might block part of
it") in `AISCommands.h`, vendored in `crates/recoil-ai-sys` and bound by bindgen, so the shim only needs the two
handlers. The game allows both between allies: `game_no_share_to_enemy.lua` allows any transfer between allied
teams (and anything from an AI team), `game_prevent_excessive_share.lua` caps a resource transfer to what the
receiver can hold, `unit_cancel_orders_on_share.lua` clears a given unit's orders; the lobby options are
`tax_resource_sharing_amount` (default 0) and `disable_unit_sharing` (default false). What it answers: a seat with
1,224 banked while the advanced plant's seat sat at 0 (games 7, 9: "the player has no way to move metal between our
seats"), three armies that cannot be one group because they belong to three seats (I2), and the seat whose
commander died keeping its plants and constructors nobody can order well.
12b.1 **Protocol**: `Command::SendResource { metal, energy, to_team }` and `Command::SendUnits { units, to_team }`;
the shim handles them and reports `ret_isExecuted`/`ret_sentUnits`; the `UnitGiven`/`UnitTaken` events reach the
bot so a seat's own-unit set is right after a transfer (re-run `run/install_to_bar.sh` after: a shim change).
12b.2 **Tools**: `transfer {"metal": 800, "from": "t2", "to": "t1"}` and `transfer {"units": ["group_A_t2"], "to":
"t1"}` (a group, a constructor, a plant by actor name); the receiving seat's hands take the units into a group of
its own and say so; the report's seats line shows each seat's store so the player sees where the metal sits.
12b.3 **What it makes possible without a rule**: one army under one seat (the groups given to the seat nearest the
front), the advanced plant's seat fed by the others, a dead seat's plants and constructors given to a live one,
and a person's own habit (Irishstud14, game 5: "orange and red both need nanos", said of seats by colour).
12b.4 **Caps**: the receiver's storage caps metal (the gadget), so a transfer says what arrived; a tax option, when
set, is read from the mod options and said in the report. A transfer larger than the receiver's free storage is
sent in chunks as the receiver spends, the hands' task keeping the remainder ("1,800 of 2,600 sent; the rest as
t1's store empties").
12b.5 **Tier 2 for all seats from one plant** (the user: "one seat getting to t2 can allow all three seats to gain a
t2 build unit"). The advanced plant's seat makes an advanced constructor per seat and gives them (`transfer
{"units": ["constructor_N"], "to": "t3"}`); the received constructor joins the seat's builders with the tier-2
menu, and the brief says the arrangement as the team play it is: one seat teches, the others feed it metal from
12:00 and get their tier-2 builder back.
12b.6 **Eating a commander** (the user: "many multiplayer matches start by one player eating their own commander to
use the metal on something"). The `remove` tool already takes `commander_tN` as a reclaim handle (by another builder;
a builder cannot take itself apart), so the action exists; what is missing is the bank: a starting seat's store
(1,000) cannot hold a commander's 2,700, so the reclaim is done in chunks (the user: "take a chunk of it, cancel the
resource recovery long enough for the bank to empty, then continue"). The reclaim task pauses when the store is
within a margin of full and resumes as it drains, or the surplus is sent to another seat as it comes (12b.4), and
the picture says "reclaiming commander_t2: 900 of 2,700 taken, paused for the store" so the player sees the pace.
The brief carries the team pattern: which seat eats its commander (the one whose start is safest), what the metal
buys (a second plant and constructors at once), and that the seat then plays without a D-gun and with its other
builders as its only commander.
12b.7 **Sharing as the answer to a dead seat**: when a seat's commander dies its plants, constructors and groups are
given to the nearest live seat before the game's own ending rules take them (the report says what a dead seat
still holds and offers the transfer).

## 13. The expansion (`docs/studies/2026-09-27-jev-posing/expansion.md`)

The study's verdict: going home is small (2 % of constructor time); the thrash is between seats (114 extractor
orders abandoned, 57 at spots another seat took), the seats have too few constructors, and the side stops at the
midline. The side's extractor count was level with his; his tier-2 income per extractor decided it after 14:00.
13.1 **The seats share their spot claims and their extractors.** `free_spots` counts allied extractors and allied
orders as taken (the team board's `spot_claims` is filled and read: today it is neither, though H-TEAM-BOARD says
it is), the picture's spot entry says "constructor_12_t2 is on its way to take it" for another seat's claim, and
the extractor state's words say what the spot returns and who else is near it.
13.2 **The packet names spots however they are written**: "spot_34, 29, 35" names three spots; a seat's list of
spots is echoed back as read.
13.3 **A list step never vanishes**: a step `execute_builder` cannot play is put back or reported skipped with the
reason, never popped silently (39 of 139 lists lost extractor steps).
13.4 **Constructors per seat as the pace**: brief text and allowance defaults only (a constructor a minute from each plant to
four by 4:00 and eight by 8:00, his count; `armcv` not capped at one or two), and the
report's seats line says each seat's constructors beside its extractors with the pool's count. Nothing the hands enforce.
13.5 **The midline is not a wall**: the free spots west of the middle are named with their walk and their risk
(the nearest enemy party, the nearest turret of his), and the `expand` job takes them by walk, nearest first; the brief's "never go to the enemy's strip" says
where the strip is, not the middle. The turret cadence ("a turret every one or two spots") is ~~STRUCK~~ (2026-09-27, the user: empower Jev and the player; no rigid rule where another change resolves the cause): brief advice, not the job's.
13.6 **"Help the plant" says what it is worth**: the assist state's words carry the plant's draw against the store
("the plant is starved: helping adds nothing"; "the store is full: helping spends it") so a pick at 0.32 confidence
over an extractor is not the default answer of an idle constructor.
13.7 ~~STRUCK~~ (2026-09-27, the user: empower Jev and the player; no rigid rule where another change resolves the cause): a hard rule for what 7.5 solves by offering the nearest spots (two first constructors walked 108 s toward the person in game 9).

## 13b. The decompression stage (the user, 2026-09-27: "is there anywhere the decompression stage actually helped in those games? I am inclined to drop it, and hadn't intended for it to be active there")

The stage: a changed packet goes to Jev once as extraction questions over the standing vocabulary, and the answers
become the packet's standing rules (docs/design/2026-09-25-standing-orders.md; `decompress_packet` in
pianist/mod.rs). It is off under `WITHIN_REASON_RULES=off`, the arena's `--no-rules`, which the arena target has run
since onepass-player-8; the flag gates only the packet's rules (mod.rs:814), so the `standing` tool's rules, their
defaults and their pruning stay with it off. **Why it was on in the games with people:** `run/human_game.sh` sets
the realtime, record, log and model variables and not `WITHIN_REASON_RULES`, so every human game ran with the
stage the arena target does without. To settle: three Opus reviewers are auditing every decompression of the
eleven games (per game `decompression-review.md`: each extraction against the packet's text, the packet rules that
were in force and the rule-source plays they caused, their harm, and the counterfactual of dropping it).
**The audit's answer (three Opus reviewers over the eleven games, per game `decompression-review.md`; their
costliest claims re-checked in the logs): drop it.** Where it helped, in all eleven games: two commander kills
of raiders in game 10 (141 metal; the sentence "a commander attacks raiders at our own buildings within its short
walk", correctly read; Jev's gate had declined the raiders in 4 of 5 cases), t3's constructors going home from
the 14:43-15:26 raid in game 4 (8 rule plays, no damage taken), `job expand` extractors in games 2 and 5 (an
extractor was rated 0.48-0.51 the second before anyway), turrets in game 6, one commander help in game 4. Every one
of these is a rule the `standing` tool carries, and in every case but game 4's retreat the player set the same
rule by the tool in the same or the next turn; the packet only bridged the gap. Of 218 rule plays in games 9-11,
7 came from packet rules and 211 from the tool.
Where it hurt: (1) **switch-offs that cannot work**: the tool's null, false and "no" clear only a tool rule
(`set_tool`, standing.rs), so a packet rule of the same name stays in force: 8 cases in games 4, 6, 7 and two in
game 10; in game 4 commander_t3's attack rule, from a paragraph naming only commander_t1 and commander_t2,
stayed on after the player's "no", Jev picked the attack at 19:10 and 19:11, the commander died at 19:18 and its
blast took three constructors, an advanced solar and a turret (record-2). (2) **Pruning read from another
actor's sentence or with its condition dropped**: in game 4 (comet-catcher-3) `raiders_lone ignore` from "Never
chase single units" spread to every group; at 21:52 and 21:57 the rule played "leaves party_217_t1 and holds" over
Jev's own picks of the hunt and the whole-group attack at 21:50, 21:51 and 21:53, `stop` went to all 11 Stouts
twice, and four to five Stouts died to that Bull between 21:55 and 22:07 (jev-0, record-0); `no_detachments` from
"never in ones and twos" or "never split" removed the hunt from 239 small-party slots in game 4 and 572 in Cape
Violet, silencing the player's `raiders_lone detachment` tool rule for minutes; "while enemies stand in E3/E4,
don't go to X" became a permanent `never` (8 of 10 wrong constructor rules, game 3); "No energy converters; a
solar only when energy reads STALLING" became `solar never` for up to 6 minutes (game 4, three decompressions).
(3) **A one-seat packet read into every seat**: game 1's `station spot_12`, written for the south-east seat,
walked the west army toward it for two minutes, and a station read from "when the enemy commander is dead,
gather at spot_15" with the condition dropped walked the east group into three Thug deaths after the player had
cleared it by the tool; a `commander:` paragraph naming particular commanders becomes every seat's rule
(`rules_for`). (4) **The re-read bug**: the "packet" event is cleared only on a second that asks (mod.rs:680, after
the quiet return), so a quiet seat decompresses the same packet every second: 53-60 calls per seat for five
packets in game 10, 470 on seat 0 in game 2 (403 of one unchanged packet, 88 different order sets, 4.6M tokens,
15 % of the game's Jev tokens), rules flickering between reads and the three seats disagreeing on most reads.
What the vocabulary could not take at all: routes, conditions, "attack the enemy commander", "never west of the
middle", "every group", ranges ("spot_0 to spot_11" read as its endpoints), the commanders' "never chases" missed
in nearly every packet, and 18 "keep away from enemy soldiers" sentences never read as `retreat_when_enemy_near`.
Readings graded against the text: game 1 138 correct, 11 wrong, 52 missing; game 4 314, 68, 129.
**The change (APPLIED 2026-09-27 evening, the first of the set)**: the stage is deleted outright, so no flag is needed: `decompress_packet`,
`take_decompression`, `extraction_questions`, `extraction_state`, `orders_from`, `set_packet`, the `packet` side of
`Standing` and the `rules`/`packet_rules` flag are deleted with the arena's `--no-rules` (the only mode); the
packet stands as prose in the picture for the per-second pass; the `standing` tool is the one source of rules.
What the audit exposed that outlives the stage, for the tool's rules: (i) `raiders_lone ignore` removes every
option against a lone raider, including one attacking the group: the ignore must not prune an attack on a party
that is hitting the group (6.x), and a rule is never played over a pick of the same second; (ii) class rules
(`constructors`, `commander`) are not marked "(tool)" in the standing line (`words` checks the actor's own entry
only): mark them; (iii) the tool's refusal is all-or-nothing (one unknown place refuses the actor's whole set):
apply what checks and name what was refused (7.2 already); (iv) `no_chase` and `job follow_list` are read by
nothing: delete them from the vocabulary; (v) the only way a group reaches a spot is a station rule, so 6.6 (e)
is what keeps the player's spot walks possible once the packet's stations go. The user's second ruling, firm:
6.6 (e), a group's walk states include the spots the packet names for it.

## 14. Tool debt (the analysis tools, not the bot)

`run/raid_ledger.py` accepts `party_N_tK`; `analyze_match.py`, `hands_window.py`, `raid_ledger.py`, `floor.py`
and `fire.py` take a seat argument or read every seat; `hands_window.py` prints the states' words; the reviewers
get their own scratch folders in the brief.

`run/jev_audit.py` is broken on the worlds-era log (2026-09-27, on the Cape Violet game: its summary line reads
"None in 0.0 min vs None", and `--section kinds` dies with `KeyError: 'busy'` because a `played` entry no longer
carries `busy`), and it reads the first seat's `jev-` file only. No audit summary was produced for any of the
human games; the reviewers read the jsonl directly. Either rewrite it for the worlds log (calls, worlds, picks,
the states' words, per seat and pooled) or delete it and let `hands_window.py` be the audit.

## Order of application

First 13b (the deletion, so nothing after is built on the packet's rules); then the picture and the report (1, 2, 3, 5), because every other change is judged by what the deciders were
shown; then the odds (4) and the states (6, 8, 9.1, 9.3, 9b.3, 9b.4, 9b.6-9b.8); then the tool refusals and places (7.2-7.5); then the
interactions (11.2, 11.3, 12.1-12.4, 13.1-13.3, 13.6); the words (10) throughout; 12b and 14 beside. Held for after the arena has run on the rest (scope, not principle): 9.2 (sea actors and states) and 9b.1 (the region graph for walking distances from anywhere; the review ranked it last by cost). Each change registers its heuristic and claim as the docs loop
asks, and the arena resumes on the whole set (the arena target: Opus 5.5 medium, hard_aggressive on Comet Catcher,
`--map` always passed), with the games with people as the standard the arena is read against.
