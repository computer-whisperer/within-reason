# Routes in prose: Jev owns the sequence and the fork, the hands keep the facts

**Status.** Written 2026-09-28 after player-9-posing (lost 36:32: his base at G1-G2 never found) and the study
`docs/studies/2026-09-28-route-in-prose.md`. Agreed direction with the user (2026-09-28): three changes, quoted in §1.
Not built. §5 is the user's open idea, with one proposal, not a decision.

## 1. The user's three changes

"Jev needs to be able to react quickly when a group reaches the destination so it can keep momentum moving to the next
location. Opus needs to be able to queue longer tasks for jev and rely on it to bail if it needs to. In other words,
opus needs to be able to trust jev for 'proceed x, y, z and hold/fight/retreat as appropriate if you find the enemy'.
Thirdly, jev needs to be able to track some kind of state on its own allowing it to follow sequences of orders with a
possible fork in the middle."

And on the station list, once it was read as a mechanic: "we should consider letting jev own it naturally."

## 2. What the game showed (the evidence, from the review and the traces)

- The 6:00 raid loop: six stations queued at once, walked at the Blitz's top speed, five halts of 1-3 s at each stop
  while the hands re-picked the next, the sixth stop being home. Nothing of his came into sight because every stop
  was on our side of the midline. The lead of 2.4:1 at 6:00 was gone by 8:00.
- The scout routes: 3:27 an eight-stop route past the lab; at 3:58, one Rover sighting the enemy commander, the pick
  chose "falls back to our base" (0.62) over the route's four remaining stops, whose state read "walks to its station
  spot_34 (right here away), then on to spot_25, spot_17, spot_10, spot_5". 4:22 a seven-stop route; killed 500 from
  the lab. 7:05 the hands' own Blitz scout, diverted onto a Pawn and killed 1,300 short.
- The pushes at E3: a group on an advance station re-decided every second against fall-back, pull-out and gather
  states at 0.4-0.7, the advance's whole text being "advances to its station spot_23 (near away)"; 13 task changes in
  53 s, never past the nest's edge; the player read the flip-flop as the enemy's strength.
- Lag: turns every 9 s of game time, orders landing about 5 s after the report they answer, 61% of the game with
  orders in flight; the player queued more than one place in 14 of 101 station orders and wrote "letting it play"
  five times. The picture says nothing about when orders land.

## 3. What the study settled

With the route as prose in the packet, one `recent` line per stop reached, and the "nothing changes" world's line
naming the cost of standing at a reached stop, Jev picked the next leg at 24 of 24 arrivals (0.56-0.75) and kept the
scout on its route at the commander sighting 4 of 4 (0.82-0.90 against the retreat). Without the cost sentence
"nothing changes" won 22 of 24 with the leg flagged; without the reached lines the leg fell under the 0.5 flag. The
state Jev needs is words in the picture, and the fork is answered from the packet's prose about the group's role.

## 4. The design

### 4.1 Delete the mechanic

`standing`'s `station` list (its `stations_done`, the station state's "then on to ..." text, `station_walk`, the
list's advance-by-arrival in `keep_groups`) goes. `station` stays as one place, the group's post, as it was before
the list (game 10). A route is prose in the packet: "group_A: spot_49, then spot_46, spot_40, spot_55, spot_58, back
to spot_64, in that order, advancing; on his buildings in sight, kills what is undefended; on a party it does not
outweigh, holds out of its reach and says so." The `instruct` tool already promises a walk to every spot the packet
names, however far (12.3); a route's stops are those walks.

### 4.2 The facts the hands keep (the state, as observation)

Per group, in code, said in the picture and never acted on by code:

- **reached**: each named place the body came within 300 of, with the clock: `recent` gets "6:24 group_A reached
  spot_49"; the group's entry gets `route_seen: "reached spot_49 (6:24), spot_46 (6:30); at spot_46 now"`.
- **seen on the way**: the first sighting of his buildings or a party since the last stop, with the clock and place,
  in the group's entry, so the fork has its premise in the same lines as the route.
- **the stop's cost**: while a group holds at a reached place and the packet names further places for it, the
  nothing-changes world's line says "group_A holds at spot_49, a stop of its route it has reached, with the route's
  next stop spot_46 not ordered" (K-jev-hold-words-carry-the-cost, applied to groups). The composer finds "next"
  by the packet's order of the named places after the last reached one; when it cannot, the line says "with further
  places named for it not ordered".

### 4.3 Momentum: the arrival is an event

A group's body arriving at a named place, a first sighting since the last stop, and a loss above a share are events
that ask at once (they go into `pianist.events`, which bypasses the signature and the 20-s re-ask; `EVENT_GAP` stays
for hits). A Jev call is about 235 ms, so the next leg starts within a second. The arrival hold has no seam for a
fall-back pick to slip into, because the pick that follows the arrival is the one that chooses the leg.

### 4.4 The fork

No new syntax. The packet's prose says what the group is and what it does on contact; the picture says what it met;
the menu carries the leg, the fight, the hold out of reach, the fall-back; Jev picks. What changes so the fight is
judged rather than flipped: the advance's words carry what stands ahead and the odds (the party line already does
this for `enemies_near`; the leg's state repeats it), and between events a group on a route is not re-asked every
second: its slot is open only on an event or the 20-s re-ask. This is the one departure from one-pass's "every actor
every second", and it is what stops the E3 flip-flop.

### 4.5 The player's side

- The brief: how to write a route with its exits, the scout's role sentence that held the 3:58 fork, and the lag: "your
  orders land about N s after the picture you read; a group that reaches a place waits for your next turn unless the
  packet names its next place: name the whole route."
- The picture: `player` already says "its last orders reached you N s ago"; add to the player's report the same
  figure the other way, "your last orders landed at m:ss, N s after the report you read".
- The sweep state sorts his side first when the packet names his box as the job; the raid reach of 4,000 becomes the
  route's problem, not a constant.

### 4.6 Measures

The kill test's numbers, rerun live: the 6:00 loop's halts (eight seconds of 52 today), a scout route's stops reached
before contact and after, the E3 push's task changes per minute (13 in 53 s today), and the base found by minute N
(never, in player-8 and player-9). `run/route_ab.py` reruns the offline test on any game's recorded arrivals.

## 5. The register set (the user's open idea)

"Some kind of register set for jev. The stateless hands are definitely smart enough to drive a state machine, we just
need some way to have opus's prose translate into a state machine that jev will instantly understand how to drive."

What the study says about it: the machine Jev drove was the packet's prose, and the registers it read were three
lines the hands wrote from facts (reached, since when, the cost of standing). It did not need the machine spelled out
as states and transitions; it needed the program counter said in words.

One proposal, to iterate on rather than adopt: a register is a named fact per actor that the hands maintain and print,
never act on. The player's prose defines the program; the registers are the only state, and the picture prints them
under the actor: `route_seen` (the places reached, in order, with clocks), `met` (what it met since the last stop),
`exit_taken` (the last fork: "6:40 held out of reach of party_11 (Jev)"), `since` (how long in the present state). A
transition is a pick; the hands record it as a fact. The player can name a register in prose ("after it has met
nothing for two stops, it goes on to spot_10") and Jev reads the register beside the sentence. What this avoids: a
transition table the player must write and the hands must parse, which is the decompression stage again (deleted
2026-09-27 for the harm it did). What it leaves open: whether prose alone carries a program with more than one fork,
which the study did not test; the next kill test is a route with two exits on recorded moments, the same harness.

## 6. Order

1. §4.2 and §4.3 (facts and the arrival event), with the list deleted (§4.1): one commit, tests on the composer's
   line and the events.
2. §4.4 (the slot open on events only for a group on a route) as its own commit, measured on the E3 replay window.
3. §4.5 the brief and the report line.
4. One arena game on the target (`docs/experiments.md`), then the register question with the two-exit kill test.
