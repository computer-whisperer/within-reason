# Why the player forbids detachments, and what the hands do with them (2026-10-17)

The user, after predict-sweep-1: "Why does opus decide to forbid detachments? I have seen that trend for a while now
and I don't think it's right. We usually need the freedom for jev to detach units and repel an enemy party, and if jev
is not using that power coherently then that is something to fix." Then: "As described the detachments don't seem like
a mistake. Sending a group after a group, then forking that group between two threats sounds completely reasonable.
Why was opus forbidding it in the first place?" And: "In total I see one mistake so far here: the hands not withdrawing
a chase when it encounters danger."

Games: the eight of `run/matches/1792265432-predict-sweep-1` (seeds 1-8, the arena target line, Opus 5.5 medium, BARb
hard_aggressive on Comet Catcher) and the three singles of the day on the same brief (finish-player-1,
sightings-player-1, predict-player-1). Instruments: `run/hunts.py` (written for this; the counts below from its
scratch forerunners agree with it on seed 8), the records' `group` rows, the hands' logs streamed, the players'
transcripts.

## 1. The rule and when it is written

An instruction packet with "it sends no detachments", "never splits", "as one body" appears in nine of the eleven
games; the timeout game (seed 8) carries it in 69 packets from 14:09, finish-player-1 in 54. The first such packet
and the player's own reason, read in full:

| game | first rule | the player's reason, in its words |
|---|---|---|
| seed 8 | 6:58, 7:08 | "The hands sent too much at one raider ... Instead of two, the hands sent 8 soldiers ... Group_F also had no place to come back to." "Of the 8 sent after the first raider, 4 then split off as group_H after a second one ... My last orders named only group_E as the army, so they no longer matched what's on the field." The push moved from 7:30 to 7:50. |
| seed 6 | 5:48 | "The Blitz army peeled off two more Blitzes after a lone enemy Rover, even though my orders said not to. It's down to 6 on the march." |
| seed 3 | 12:14, 12:24 | "My raid sentence ... let the hands keep splitting the body. Group after group peeled off to chase single Grunts and Incisors back home. As a result group_E1 has only 13 soldiers at spot_18." |
| seed 5 | 11:26 | three groups each fighting a post alone; "their best-rated moves were detachments sent home after his Rovers." |
| seed 7 | 13:54 | its own detachment-of-eight rule: "the army split three times in ten seconds chasing one Incisor and a block that had already left." |
| finish | 4:45, 4:55 | "lost a Blitz to a lone Pawn ... The Blitzes keep splitting into ones and twos to chase single raiders, and lose those fights alone." |
| sightings | 6:13, 6:23 | "We've lost five Blitzes ... Our Blitzes kept going in a few at a time after single Pawns, and the rule that steps soldiers out of a turret's reach kept the rest from closing on it." |
| predict | 6:59, 7:09 | "four Blitzes died at spot_18 and around his F2 turret ... our Blitzes keep breaking into groups of one and two, chasing single units, and those small groups die alone." |

Two kinds of reason. In seeds 8 and 6 nothing died: the complaint is the size (eight for two), the fork breaking the
packet's named body and the timed push. Where units died (sightings, predict, finish, seed 8 at 7:58, seed 3), the
detachment had followed its quarry under a turret or into a party, or the attacking body at his base had been pulled
apart by sends at raids in our middle.

The prompt primed the rule. It said "Defence is yours: nothing in the code answers a raider at a structure on its own,
and the hands answer only as your packet tells them. Left to a bare engage, they send the whole ball after one scout
car", "the ball never chases a lone raider", "send: one; two or four for a small party", and on attack "the whole army
together at its commander, not a detachment; a fifth of the army loses to what all of it would walk over". The hands'
standing rules (`rules.md`, read at every pick, never shown to the player) said "A raider at one of our extractors is
met by a small detachment (send), not the whole group ... the soonest wins, faster soldiers over slower". The player
read a detachment it had not asked for as disobedience and wrote "never". The absolute works for it: since 2026-10-16
the menu returns the whole-group attack only when every rung of the ladder is forbidden at the decode's ON.

## 2. What the hands did with sends

Eleven games, 323 send picks:

| what | count |
|---|---|
| sends picked by a group that was itself a detachment | 191 of 323 |
| sends at a party already sent at within the last 60 s | 119 of 323 |
| send words naming a group of ours already on the party (four games read) | 0 of 130 |
| sends taking at least half the group (four games sized) | 56 of 126 |
| hunts ended with a kill (four games) | 20 of 92 |
| median hunt length | 9 s |

Seed 8 at 6:53-7:14: group_E (11) sent 8 after one Incisor as group_F (0.64 against send_4 0.62, send_2 0.34, send_1
0.08: the two were two Rascals, "it outweighs us"), five seconds later 2 more after the same Incisor, group_F sent 4
after a second Incisor and 2 more after the same one, group_H then attacked it with the whole group and followed it
into two Pounders at C6 (7:58, two Rascals dead). Seed 3 at 12:12-12:17: the body of 31 at his base sent 8, 4 and 8 at
one five-unit raiding party and 4 and 4 at two others in five seconds; 13 were left at spot_18.

Hunt endings: seed 8 12 (1 dead, 4 lost, 7 leash), seed 3 24 (3, 9, 12), seed 1 24 (3, 14, 7), seed 7 32 (13, 5, 14).
Sends whose words said the quarry outruns the hunters: 15 of 20 in seed 8, 44 of 50 in seed 7: by design, the standing
rules make driving off the goal, and in seed 8 the eight did drive the raider off.

## 3. Verdict, with the user's

The forks are sound. What is not, in the user's order: the chase does not withdraw at danger (a hunt ends only on a
kill, six seconds out of sight or the 900 leash, hunters attack by id with raw footwork, and the whole-group attack
follows its party anywhere while seen); then a second send at a party already hunted with words that do not say so;
then the name left on the remnant when most of the group goes; then the ladder by speed. The player's remaining
reasons are its plan form (a named body and a timed signal).

## 4. Built (the user, 2026-10-17: "go ahead")

- The player's prompt carries `rules.md` whole where it has `<!--hands-rules-->` (`strategist/mod.rs`
  `with_hands_rules`, `brain::pianist::hands_rules_whole`), and its defence paragraph describes the ladder, the
  detachment's life, the name rule and the packet's levers; the sentences above are gone, "a fifth of the army"
  with them. The brief's hands bullet and the old brief's reworded.
- The `chase` line and the `chase meets` event (H-HANDS-CHASE), the join's "the group it split from", the turret ahead
  of a moving party on every move aimed at it.
- The send and attack words' "already on it"; the hunters keep the name when at least half the group goes.
- `run/hunts.py` for the ledger: sends by size and share, repeats, chains, hunts by ending, soldiers lost while
  hunting, chase asks and what was picked.

Not built: a detachment made of the units the player names (the user: "the naive solution will be far too fragile"),
and an exact-detachment tool for the player, later.
