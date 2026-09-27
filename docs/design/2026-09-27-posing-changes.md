# The changes the games with people call for: to be made together, then the arena resumes

Status: PROPOSED, nothing applied. The user, 2026-09-27: "I don't want to start making changes until we fully
understand everything we can from these matches. Once we understand all the mistakes we can see, we can apply the
changes at once and resume arena matches." The findings are in `docs/studies/2026-09-27-jev-posing/README.md`
(six games mined; the expansion study `expansion.md` is being written and will add to this). The code, rules and
brief stand at 6151a89. Each change names the finding it answers (the study's numbering: I = missing information,
A = missing action, W = misleading words, X = unfortunate interaction) and the games it was seen in. The user's
frame: the answers are information and offered actions, not hard limits or enforcement.

## 1. The picture: what a group is (I1, the costliest mechanism, seen in every game)

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
1.4 **A plant's output joins the group as a body**: new units gather at the plant's rally (or the group's tail)
until a handful stand, then walk to the group together, and the picture says "N of its soldiers are on the way
from plant_X, K behind" (game 3 11:34, game 9 15:44). Files: `pianist/groups.rs` (join), `execute.rs`.

## 2. The picture: the other seats (I2, games 4, 6, 9)

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

## 3. The report: the numbers (I3, I6)

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

## 5. The enemy: where and what (I5)

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
6.5 **The fall-back point is never where the group stands, never `shelling`, never a place a party is entering**;
when none qualifies, the state says "no way back that is not into fire" and offers the gather instead (games 3,
6, 9).

## 7. The packet and the standing orders (A3)

7.1 **The decompression's vocabulary gains** `advance_together_with group_X`, `attack_commander_on_sight`, an escape
route (`escape_to place, place`), `station_by_front`; and every paragraph part it cannot read into a rule is
said back to the player at once ("not read: 'advance as one body'"), instead of dropped silently (game 6).
7.2 **A refusal never takes the actor's other rules with it**: the tool applies what it can and names what it
refused (game 6 16:42).
7.3 **A rename never refuses an order**: a party name from the last report resolves to the party those units are
in now (2.2 makes the names stable; until then, the tool resolves by member ids).
7.4 **A place accepted is a place** ("corsolar spot_25" accepted for a spot not in the picture, game 9).

## 8. The commander (A4)

8.1 **The D-gun as a state**: "D-gun party_N (within 262, energy 1,415)" with the energy said.
8.2 **Time-to-contact in every builder threat line** ("Brutes at 87 against its 38: contact in 9 s if it walks
away, 4 s if it stays"), and the step-away only to places it reaches before contact; when none, "no way out on
foot: fight here with the D-gun" or the turret's cover.
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

## 10. The words (W)

10.1 "Outruns this group ... a chase drives it off" only of a party that is moving; of one holding ground:
"holds its ground at X and shells". 10.2 "Out of its reach" only of a place the builder reaches before contact.
10.3 "The quarry is dead" only when it is. 10.4 `shelling` is never a place to walk to in any state's words.
10.5 A hunt that failed says so when the same units are offered again.

## 11. The crossing orders (X1)

11.1 **An order landing carries its time and the picture it was written on**; a pick within the landing grace
never reverses an order that landed in the last N seconds unless a loss or a new party justifies it, and the
picture says "the player's order of 16:07 is in force" on the group's line (four reversals in 45 s, game 9).
11.2 **The player is told what the hands did with each order in the next report** ("your advance of 16:07 stood
1 s; fall back picked at 16:08 on losses"), so a standing station shown as set is not read as being played
(game 6 9:52).

## 12. Defaults that stack (X2, X5)

12.1 One default per actor per second: `raiders_party whole_group` and `attack_raiders yes` make one state the
default, not one per threat slot. 12.2 `hold_line yes` reads the front's odds (1.1). 12.3 A builder on a list is
offered the solar default when the store drains (game 7 8:00-10:00). 12.4 A station change ends a walk to the old
station (Cape Violet 8:05). 12.5 A hunt's leash is measured from where the hunt began, not the station.

## 13. The expansion (the expansion study, pending)

(Filled from `docs/studies/2026-09-27-jev-posing/expansion.md` when it lands: the constructors' thrash, the
returns to base, the lists arriving after the hands moved, the pace against thebluegecko's.)

## 14. Tool debt (the analysis tools, not the bot)

`run/raid_ledger.py` accepts `party_N_tK`; `analyze_match.py`, `hands_window.py`, `raid_ledger.py`, `floor.py`
and `fire.py` take a seat argument or read every seat; `hands_window.py` prints the states' words; the reviewers
get their own scratch folders in the brief.

## Order of application (proposed)

First the picture and the report (1, 2, 3, 5), because every other change is judged by what the deciders were
shown; then the odds (4) and the states (6, 8, 9); then the packet vocabulary and the tool refusals (7); then the
interactions (11, 12); the words (10) throughout. Each change registers its heuristic and claim as the docs loop
asks, and the arena resumes on the whole set (the arena target: Opus 5.5 medium, hard_aggressive on Comet Catcher,
`--map` always passed), with the games with people as the standard the arena is read against.
