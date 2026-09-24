# wake-3: regressions from the hands' prompt rework and the fall_back fixes (2026-09-23)

`run/matches/1790217337-wake-3/00`, read live at 17.6 game minutes, carries f3553c9 (A, the prompt rework and five
cuts) and 094fca5 (B, the fall_back fixes). Baselines: wake-1 (`1790212198-wake-1/00`, before both, won 18.5) and
wake-2 (`1790215463-wake-2/00`, A only, lost 24.8), compared at the same clock (to 12:00 or 17:00, as stated).

## Findings, most important first

### 1. `fall_back` is almost never offered: the station never forms (B)
- To 17:00, `fall_back` was offered on **2 of 724** group asks (wake-1: 337 of 809; wake-2: 138 of 435).
- Cause (code plus log): `hands.rs:285` answers a `hold` pick with `set_task(Hold { since: frame })` even for a group
  already holding. A holding group has no `continue` (`menu.rs:492`), so it keeps holding by re-picking `hold`,
  every 5 s median (p90 12 s), and each re-pick resets `since`. Replayed over the log, only 3 holds reached the 15 s
  `STATION_FRAMES` in 17 minutes; without `last_hold` the gate fails whatever the odds.
- The rules and the `do` question still say the group "falls back (fall_back)". With no `fall_back` on the menu,
  Jev takes `retreat`, which walks home: **73 of 724 group choices (10.1 %)**, against 5 of 809 (0.6 %) in wake-1
  and 29 of 435 (6.7 %) in wake-2.

### 2. The groups swing between retreat and the player's place (A1 precedence + B gating)
- Flip triples are three consecutive non-continue picks by one group within 30 s, going back, then forward, then
  back, or the other way round. To 17:00 there were **61 in wake-3**, 3 in wake-1 and 6 in wake-2. To 12:00: 33, of which
  32 alternate `retreat` and `move_to`.
- How it happens: a group retreats home. On its next asks it is "walking back", so B hides `retreat`. 64 asks came
  from groups walking back (to 12:00). **30 of them chose `move_to`**, and 26 of those 30 came within 1 s of a new
  player packet landing ("orders from 0-1 s ago"): the packet names the group's place, so Jev sends it there. On
  the next ask the group is no longer walking back, so `retreat` is offered again. The precedence sentence
  ("outweighs it ... falls back") then picks it.
- Frames: group_A alternated move_to and retreat six times from 6:51 to 7:16 while 6-8 Pawns raided the base.
  group_J alternated them nine times from 10:27 to 10:55 and lost 5 of its 6 soldiers (471 metal). group_L, group_M
  and group_Q did the same at 11:15-11:23.
- The player saw it. At 11:17 it noted "our Blitzes (~30, 3.3k) keep retreating in pieces", and at 13:44 "Hands
  appear to act one packet behind; every rewrite causes back-and-forth."

### 3. The "killing ... now" words on the group's own line read as the group's own losses (A4)
- `enemies_near` now carries the nearest party's "killing X now". That list covers everything of ours the party is
  killing, other groups' soldiers included.
- To 12:00, at "we outweigh it" or "heavily", with no `losses` line on the group:
  - without a killing clause: **0 of 69** asks chose retreat;
  - with "killing N of our Blitz/Rover" on the line: **7 of 51** chose retreat.
  - wake-2: 11 of 26 (fall_back or retreat); B damped it, not removed it.
- Frames: group_M at 11:12 (8 Blitz; "party_133 (1 armpw, 1 armrectr) 115 away: we outweigh it heavily; killing 3
  of our Blitz"; no losses line; the packet had sent it to spot_26) picked retreat 0.54. Likewise group_K at 9:35
  (0.53) and group_P at 11:27 (0.54).
- Retreat against a party we outweigh overall, to 12:00: 9 of 137 asks (6.6 %). wake-1: 1 of 87 (a fall_back).
  wake-2: 14 of 69.

### 4. What the precedence wording does where it was aimed (A1-A3): as designed
- `run/jev_inflight_ab.py` (run once, repeat 1) found 18 in-flight loss moments.
- The 5 retreats all had "losing this fight", "being wiped out" or "it outweighs us" on the lines.
- Groups losing "a few" or "a noticeable share" against a party we outweigh kept fighting (7:37 group_E engage,
  11:58 and 13:42 group_I continue).
- The replay matched the recorded choice at 17 of 18 moments (14:06 group_Z: replay move_to, recorded continue).

### 5. The economy: behind wake-1 from 6:00, but not because of A or B
- At 5:00 wake-3 was **ahead**: 13 extractors and 27.4 metal/s, against 8 and 17.5 in wake-1 and 5 and 9.5 in
  wake-2. Its first loss came at 4:13; wake-1's came at 2:40.
- From 5:00 to 8:00 it fell to 8 extractors (wake-1 rose to 17). At 10:00 it had 10 and 28.9/s against 25 and
  50.2/s, and at 17:00 7 and 14.8/s.
- **The plant made nothing from 5:13 to 7:45.** plant_17003 showed "building Construction Vehicle (armcv)" with no
  progress. Metal banked to 1,748 of 1,750 at 6:30.
  - constructor_30526, made at 4:45, stood on the pad (1281-1291, 2544-2552; units come out at 1296, 2554).
  - It built Construction Turrets from there, as the packet said ("each newly made constructor first builds a
    construction turret at the plant"); it left at 7:43 and the next armcv appeared at 7:50.
  - The yard check (`picture.rs`, `stuck_in_lane`) counts only units that cannot move, so neither Jev nor the player
    was told. The player's 6:10 note says "no progress since ~5:19 ... Unknown cause". It answered with plants two
    and three (6:50, 7:54; 1,180 metal).
  - This predates A and B.
- Raid pressure was also higher. Enemy units in our third of the map (truth file) were 13 and 11 in minutes 6-7, against 4
  and 2 in wake-1 (wake-2: 12, 13). 18 buildings died from 5:03 to 7:46.
- **The builders did not pick worse places.** Cut 5 applies only to the groups' `where`: `menu.rs` passes
  `names_only = false` for builders, and the log shows descriptions in all 220 builder `where` and `where_extractor`
  questions to 10:00.
  - Extractor picks to 10:00: 28, median walk 12 s, p90 28 s (wake-1: 27, 17 s, 39 s). Every pick was on ground held
    by us.
  - None took a spot another builder was claiming. 9 were the builder's own claim.

### 6. The unasked actors on one line (cut 5)
- To 12:00, all 3 `join_group` choices went to a target whose entry was still full (asked in the call or named in
  the packet).
- Over the game, a `join_group` option was offered 69 times with its target on one line. It got a mean P of 0.013,
  against 0.039 with a full entry, and was never chosen.
- I cannot separate the cut from selection: an unasked target is often far away. Not verified as harm.
- `assist_lab` targets are chosen in code; I found no answer that needed a dropped detail.

### 7. Three-name spot lists, `recent` without completions (cut 5)
- No answer plausibly needed them: all 70 group `where` answers for a walk (to 12:00) named a place in `places`;
  `scout` was picked once.

### 8. The player's side
- Turns to 12:00: 55 (wake-1 59, wake-2 64).
- Packet changes seen by Jev: **39** (wake-1 25, wake-2 27), shorter at a median of 1,375 characters (2,058; 1,705).
- The player rewrites packets in reaction to the swings (item 2), which feeds them; by its 13:44 note it then held
  the packet steady.

### 9. Mechanical health: clean
- 585 calls to 17:00, 0 errors, 0 "Jev unreachable" (wake-2 had 3).
- State median 10.1k characters, max 17.3k (well under 32k); 7.9k input tokens median.
- 0 questions with empty options, 0 choices outside their options.

## What changed cleanly
- The token cuts (state 10.1k against 14.3k characters in wake-1) and the names-only group `where`.
- The 30 s `losses` line with its share words, and the precedence sentence at in-flight loss moments (item 4).
- No group alternated fall_back and retreat (wake-2's group_L loop), though item 1 also removes fall_back.

## What I could not tell
- Whether the lab-pad block (item 5) recurs without these commits: it rests on the packet's nano order and the build
  site, not on A or B (not verified by a rerun).
- Whether one-line join targets lowered join quality (item 6).
- The swings' cost in metal. The costliest ask (11:54 group_I, 10 Blitz in 20 s) followed the packet's order to
  attack the Centurion block, not a swing.

## Fix first
1. `hands.rs` Pick::Hold: keep `since` when the group is already holding, so a 15 s station can form. Without it B
   disables `fall_back` and the rules point at an option that is not there.
2. Keep a group walking back on its way until the packet changes, or offer `continue` as the only way back. Stop a
   landing packet from re-sending a group that fell back for cause in the same breath.
3. On the group's own `enemies_near`, say "killing" only for its own soldiers, and name the rest as "elsewhere".
