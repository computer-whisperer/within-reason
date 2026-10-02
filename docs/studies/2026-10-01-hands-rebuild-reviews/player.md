# The hands rebuilt, reviewed as the player who has to use it

One model's opinion (Opus 5.5) as the player, written from the design note (docs/design/2026-10-01-hands-rebuild.md),
the role text (crates/bot/src/strategist/player.md), the brief's group sections, rules.md, packets 12/30/45 of
player-33, and the recorded menu at f=29310 (packet 30's landing second). Nothing was run.

Overall: the direction is right and I would rather write for this menu than today's. Laws 2-5 remove the things that
made my paragraphs unpredictable (a group given the whole packet, "advanc" anywhere turning places into advances, the
party menu silently closing a group's own). But the design moves the two things I most depended on, *route order* and
*where a group goes when it should not fight*, from code into Jev's per-second reading of my prose, and gives me an
echo that shows which names registered but not what role they registered in. Below, concretely.

---

## 1. Packet 30's group paragraphs, rewritten for the new vocabulary

Packet 30 as written had three groups (V1, H5, I5) gathering at spot_38, attacking party_32 together with the Janus
and Stouts in front and the Blitzes on the armfido/armmav, then a seven-stop route to his base; I4 walking to spot_38
to join; no fall-back place anywhere. Under §3/§5 I would write:

> **group_V1** (the army: Stouts, Janus, Blitzes of the middle; every group that joins it is this paragraph): its post
> is spot_38. It holds at spot_38 while its entry says soldiers of it are still on the way. When none are on the way and
> party_32 is in sight, it attacks party_32 with the whole group. When party_32 is dead or gone, it fights to spot_33,
> then spot_37, then spot_20, then spot_18, spot_29, spot_25, spot_17, in that order, one at a time; it never fights to
> a stop before the one before it is reached. Its fall-back is spot_44: when its entry says it is losing the fight or the
> party it meets outweighs it, it goes to spot_44, and from spot_44 it goes back to spot_38. It never attacks a party of
> one or two raiders; it sends 1 or 2 at them.
>
> **group_H5, group_I5, group_I4** (and every new group from the plants): join group_V1. Never attack party_32 on their
> own. If party_32 comes within reach of one of them before it has joined, it goes to spot_44.

What I expect each menu to hold, move by move:

**group_V1.** stay; hold; scout; shell party_32 (if the Janus count as long-reach; armjanus is ~450, party_32's
armmav reaches 650, so probably not); join H5/I5/I4 (every group of its kind: three lines I do not want and must
out-word); go and fight-to for each place the reading gives it: spot_38, spot_33, spot_37, spot_20, spot_18, spot_29,
spot_25, spot_17, spot_44 = 18 place moves; attack and send-1/2/4/8 at every party in the picture it can reach (at
f=29310: party_32, party_41, party_45 = 15 lines). About 37 moves, of which two go on. Things I said with no move to carry
them:
- **"Janus and Stouts in front, Blitzes closing on the armfido and armmav."** No move. Formation and per-type
  targeting are micro-engine territory; `send N at` picks N soldiers by a rule the design does not state (today
  `hunters_for` picks the fastest; §4 Q2 drops "fewest that outweigh" but says nothing about *which* soldiers). I cannot
  say "the Blitzes".
- **"Hold until the others have arrived."** Carried only by `hold` plus Jev reading the body line ("the other 11 are
  800-1384 behind"). That works if the join really makes them one group; across *separate* groups ("until group_H5
  stands there") there is no move and no fact Jev can compare, so I rewrote it as joins. That is a real constraint on
  how I organise the army: anything synchronised must be one group.
- **"In that order."** No move. §4: world 1 shows "the places read for it that it has not been to since (no 'next
  stop': the order is in the prose)". All seven route stops are fight-to lines every second; nothing stops Jev rating
  `fight to spot_17` (his base, the destination of the whole paragraph) above `fight to spot_33`. Today the code took the
  order of names as a route, and the design lists that as a defect. For a route it is the feature.
- **"His commander, when seen, is attacked by every soldier until it dies."** Only as `attack` on the party that
  contains the commander. `attack_unit` stays a player tool; the hands have no focus-fire move. A commander inside a
  party of twelve is attacked as the party.
- **"From spot_44 it goes back to spot_38."** That one does have a move (`go spot_38`), as long as the reading keeps
  spot_38 as V1's once V1 has left it.

**group_H5 / group_I5 / group_I4.** stay; hold; scout; join V1 (and each other); attack/send at the parties; go/fight-to
spot_44 if the reading gives them spot_44 (they are named in a paragraph whose only place is spot_44, plus "join group_V1",
which §5 already reports as weak: "a group told only 'joins the nearer of group_V1 and group_T2' gets the other group's
post or nothing"). Missing: **"join at spot_38"**. `join` has a group as object, not a place: if V1 has already left
spot_38 for party_32 when I4 starts walking, I4 walks alone into the fight on the shortest path. The join line must say
where the group being joined is and what stands between (design §3 "Words" says "how it stands to each party near the
actor", which covers the end, not the path).

**New groups from the plants.** The reading runs "when a packet lands" (§5). A group that does not exist yet (every
new plant group, every merge result, every `send N at` detachment) has no reading until my next `instruct`. Packet 12
("whatever the big group is called after a merge, this paragraph is its") and packet 45 ("The new groups from the
plants: follow group_L5's route") show I write for groups that do not exist yet in almost every packet. The design must
say the reading is re-asked for a new actor, and what a merged group inherits (the union of its parts' places?). As
written, a new group's menu holds no places at all, only stay/hold/join/attack.

---

## 2. What I could not predict, and the echo

**Unpredictable, in order of how much it would hurt:**
1. **Which two moves reached the worlds.** "The two best per actor go on" (§4). I never see the per-move ratings, so
   when a group does something odd I cannot tell whether my fall-back was rated low, rated second and lost in
   composition, or not on the menu at all (not read). The echo only answers the third.
2. **Route order** (above). Jev re-derives "next stop" every second from prose plus a "not been to since" set. What
   does "since" mean after a fall-back through stops already reached, or after a new packet? If going back through
   spot_33 to spot_44 marks spot_33 as visited, the next advance can skip it.
3. **Attack lines for every party in the picture.** "every party in the picture, for every armed actor that can reach
   it. No threat test, no nearest three." With rules.md's "a party within 600 of a group that outweighs it is fought now,
   whatever the instructions call it", the main ball gets an attack line for every lone armpw. Today's record at
   f=29310 shows exactly that: "group_V1 attacks party_41 (1 armpw) with the whole group ... 14 s of walking, abandoning
   its way to spot_47". I can write "never attacks a party of one or two", but I cannot predict when a party's `answer`
   noul crosses 0.5 and opens that line regardless of the group's own `change` (§4 openers).
4. **How often a losing group is re-asked.** News for an actor (§4) is: course ended, an event names it, packet or
   list changed, a party came or went near it, store crossed. A group being ground down by a party that is *already*
   near has none of these unless "an event names it" includes every hit or loss. Otherwise it is asked every 20 s, and a
   fight is decided in 20 s. This is the condition the fall-back exists for.
5. **What `go` costs while engaged.** "go ... running from everything" means turning backs to a party that outranges
   us. Whether that is right depends on reach and speed, which only the line can tell Jev (see 4).

**The echo.** "group_L5: spot_27, spot_18, spot_29, spot_25, spot_17" is not enough, for three reasons:
- **No role.** The noul asks "a stop of its route, its post, its target, or where it falls back to?" and returns
  yes/no. If my fall-back spot_44 is read as a route stop (or the post), it shows in the echo exactly as if it were read
  right, and Jev will then rate it every second from my prose anyway. What I most need to check, *does this group have a
  place to go when it should not fight*, is invisible.
- **No margin.** §5's own measurement puts coverage lists at 0.42-0.68, both sides of the bar. A place at 0.52 and one
  at 0.97 look the same in the echo, and the 0.52 one may flip on my next unrelated rewrite (repeatability is check 1,
  not yet done).
- **Correction costs a full re-ask.** To fix a misreading I send a new packet, which (player.md: "Each `instruct`
  re-asks every group whose sentence changed") re-asks those groups and re-runs the reading. The echo reaches me a turn
  after the packet landed (5-15 s plus the think penalty), so a fall-back that did not register is found about a turn
  after the moment it was needed. Within a turn: yes for a route or a post; no for a fall-back during a fight.

What I would want the line to look like (one line per group, only when it changed, in the hands section):

```
group_V1 reads: post spot_38 | route spot_33 > spot_37 > spot_20 > spot_18 > spot_29 > spot_25 > spot_17 (reached: -)
               | fall-back spot_44 | weak: spot_31 (0.46, named as party_32's ground, not read)
group_I4 reads: join group_V1 | fall-back spot_44 | NO post
group_N7 (new 12:40): nothing read yet: stay/hold/join/attack only
```

and, separately, for groups that are fighting, the top two of the last pick: "group_V1 (fighting party_32): stay 0.71,
go spot_44 0.38". That costs four nouls per place instead of one at the reading (role questions), i.e. cents, and it
tells me the only thing I can act on: whether the group has a way out and whether it is being considered. The two
explicit warnings I would want even if nothing else is built: **"NO fall-back place read"** and **"nothing read yet"**.

---

## 3. What I lose, and what I still cannot order

**Lost by the design:**
- **Split to a place**: the one I would miss. "a picket of one or two at each outer spot cluster from the first
  Blitzes" (player.md) is the brief's own advice, and the design's answer is "the player forms groups with `produce ...
  group`". That only works for units not built yet. Re-posting two soldiers of an existing group to spot_64 because a
  raid route opened is now impossible. Low play count (0-14 a game) reflects how the option was offered, not demand:
  it was never a ladder, it was "half the group". Proposal: let `send N at` take a place as well as a party (same
  ladder, the detachment a group of its own, already ruled). That is not a special case, it is the same verb with the
  other object class, which law 3 argues for.
- **Sweep**: fine to drop; rove plus the scouting block covers it.
- **Fall-back to where it last held**: fine to drop *if* every group has a named fall-back. In practice it will not:
  packet 30 named none for any group, packet 45 told L5 "never holds, never walks west of spot_20 again and never walks
  home". Under the new menu that group, losing, has no backward move at all (`hold` keeps the front standing in the
  fight). That is my fault as the player, but the system should make it visible (the "NO fall-back" echo above), and
  the brief should state the rule: every group paragraph names where it goes when it should not fight.

**Still cannot order, most missed first:**
1. **Focus fire / target priority** inside a fight ("the Blitzes on the armmav", "kill the Overwatch first",
   commander snipes from the hands). attack_unit exists only as my tool, one call per turn.
2. **Sub-selection by type** ("the Blitzes of group_V1", "the Janus in front"). A person does this with one
   selection; I must pre-split with `produce` groups.
3. **Synchronised attack across groups** ("attack when group_H5 is also at spot_38"). Only `join` approximates it.
4. **Shell a place or a remembered building.** `shell`'s object is "a party"; the Overwatch at spot_27 or a Gauntlet
   (armguard, reach 1220) remembered out of sight is not a party. This is the one case where long-reach units matter most.
5. **Ordered waypoints** (shift-queue of fight-to). The route is now pure prose.
6. **Patrol / guard an area** between two or three spots (a person's patrol or fight-patrol for a picket).
7. **Unit stances** (hold fire, hold position vs. manoeuvre). `lane` raw/footwork is the nearest thing, per group.
8. **Retreat to repair**: go to a place where constructors or a nano repair it; `go` plus a builder line in my
   packet, but nothing ties the two.

---

## 4. Wording a losing fight

Case: group_V1, 18 soldiers, attacked party_32 at spot_33 and is losing; my packet names spot_44 as its fall-back.

The `go` line I think would make Jev choose it over `stay` **when it should**:

> group_V1 goes to spot_44, the place your instructions give it to fall back to, 900 behind it, 12 s, without fighting:
> it leaves the fight with party_32, which it is losing: in the last 20 s we lost 6 soldiers (1,350 metal) and it lost
> 280; for the 12 of ours in reach the odds are now 0.6 against us; party_32 is slower than our slowest (60 to 75) and
> stops shooting us after about 4 s of the walk; at spot_44 stand our light turret and nothing of his.

and the matching `stay` line must carry *the same facts*:

> group_V1 stays in the fight with party_32 at spot_33: losing: in the last 20 s we lost 6 soldiers (1,350) and it
> lost 280; at this rate the group is gone in about 40 s.

The line that would **wrongly** make it flee a fight it is winning (every fact true):

> group_V1 goes to spot_44, the place your instructions give it to fall back to, 12 s, without fighting: party_32
> outranges it (650 to our 380) and outclasses it (tier 2: 1 Gunslinger, 650 metal each); we lost 3 soldiers in the
> last 20 s.

Missing from it: what *they* lost (1,800 metal in the same 20 s), the odds of the part in reach (3 to 1 for us), that
leaving costs more fire than staying because we are already inside its reach.

Facts the lines must carry: **both sides' losses over the same recent window** (never ours alone); **odds for the part in
reach now**, not the whole group; **the trend** (losing / even / winning, as a word); **the cost of leaving** (seconds
under its fire on the way out, whether it can catch us); **what stands at the destination**; **that the place is the
packet's fall-back** (needs the role from the reading). Facts that mislead: reach and tier words without odds (today's
lines lead with "it outranges us", "it outclasses us", which read as alarm); whole-group metal while most of the group
is behind ("4095 metal against its 54" is honest only because today's line adds "for the 1 of its 18 in the fight");
"unidentified" members counted as weight; our losses as a bare count; and "less far than the base" style comparisons,
which describe the walk, not the fight. And the same facts must be on `stay`: if only `go` says "losing", the word
itself pulls.

---

## 5. Longer packets, slower turns

- **Every group paragraph gets a fall-back sentence and a route written positively.** About 20-40 words a group. Worth
  it; I should have written them anyway.
- **Placement discipline.** Places named in someone else's paragraph or the intro ("his army stands at spot_33 and
  spot_31") may now be read as a group's place (5 of 845 today: rare, but each one is an extra move line). I will write
  the intro without spot names where I can. Small cost.
- **Paragraphs for groups that do not exist yet** need a defined behaviour (re-read on new actors); otherwise I must
  re-instruct every time a plant group appears or groups merge, which is many packets a game, each re-asking the groups
  whose sentence changed: the wake-3 swinging the role text warns about. This one is not worth it as written; fix the
  design.
- **Reading the echo every turn**: a few lines; fine if it is only the changed groups and states role. Not worth it as a
  bare name list, because it does not answer my question.
- **Fewer special sentences.** I no longer need workarounds such as "the walk to spot_13 is abandoned" (brief) or
  "never walks home" (packet 45). That makes packets shorter. Net: about even on length, better on predictability,
  *provided* route order and new-group readings are settled.

## Short list of asks to the design

1. The reading returns a role per place (post / route stop / fall-back / target), and the echo shows it with the
   weak ones and an explicit "NO fall-back read".
2. Re-run the reading for a new actor (plant group, merge, detachment); state what a merged group inherits.
3. Route order: either the reading returns the order, or world 1's line says "next in your order: spot_X". "Not been
   to since" needs a definition that survives a fall-back.
4. A losing fight is news (losses within N s), so the group is asked in seconds, not 20.
5. `send N` takes a place as well as a party (keeps the picket without a special option).
6. `shell` takes a remembered building.
7. Say how `send N at` picks its N soldiers.
8. go/stay lines in a fight carry the same facts, both sides' losses, odds of the engaged part, and the cost of leaving.
