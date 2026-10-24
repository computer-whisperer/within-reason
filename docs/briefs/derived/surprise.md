<!--meta
id: D-surprise-weapons
layer: derived
premises: [D-metal-is-the-game, D-lead-convert-it, D-reach-nests, D-energy-second, F-fight-big-weapons, F-fight-sight, F-eco-build, F-eco-energy, F-unit-armsilo, F-unit-corsilo, F-unit-armamd, F-unit-corfmd, F-unit-armkam, F-unit-armfig, F-unit-armrl, F-unit-armap, F-unit-armcom, F-unit-armamex, F-unit-armhp, M-barb-hard-aggressive-comet, M-barb-bonus, M-barb-sparring, M-format-team-small, M-format-8v8, M-gecko-vs-us-comet]
lessons: [L-gecko-nuke-surprise, L-sheet-surprises, L-sheet-labs, L-energy-converters, L-gecko-bots-vs-vehicles]
sources: {people: [thebluegecko, the user], dates: [2026-10-09, 2026-10-23, 2026-10-24], relay: thebluegecko's recording on nukes (docs/knowledge/_inbox/gecko-brief-review-2026-10-23.md, the second recording) and his sheet rows on the Banshee, the Twilight and the hovercraft platform; our two silo games and the +20 ladder game from the ledger (docs/experiments.md: nuke-1-medium, nuke-2-hard, rework-ladder-20-1); the Twilight's blast from the game's Unit_Explosions.lua (empblast: radius 400, paralyzer, 15 seconds, 4,450)}
status: draft, checked against the ledger and the facts; no kill-test tool of its own (the games are three)
-->
### Weapons of surprise

**The heuristic.** thebluegecko on the nuke (L-gecko-nuke-surprise): "It's possible to build enough nukes to overpower
a number of anti-nukes. It just scales. It just scales worse. The anti-nukes are denser in every way. They are cheaper,
they're smaller, they reload faster, they cost less to reload. So an anti-nuke versus nuke escalation results in
anti-nuke winning. So the nuke is a weapon of surprise." And, asked what else reaches past a defended front, "certain
uses of spybots and [bombers]". His sheet on the Banshee (L-sheet-surprises): "when loseing a game, rather than try
and recover by matching a stronger oponent, making 8+ banshee and sniping the oposing com is a strong option if
suprise can be maintained"; on the Twilight: "a well timed Twilight self destruct can turn the tide of a game if you
stun one of the first few oposing armies". The plain line: these weapons pay once, before the answer stands, and
never again; build one only with the answer's clock in hand, and build the answer when his clock says so.

**The quantity.** The time between the weapon's first blow and the answer's standing, on each side. Ours: the minutes
our silo, our air plant or our Twilight is unseen before it acts. His: the minutes between the sign that he has begun
one and the blow. Not the weapon's damage, which is the same whether it lands or not; the whole value is in the
difference of two clocks, and the answer in every case costs a small fraction of the weapon.

**The mechanism.**

- *The nuke's bill.* A silo is 8,100 metal, 90,000 energy and 178,500 of build: five minutes at 600 power, three at
  a thousand (F-fight-big-weapons, F-eco-build). Each missile is 1,000 metal and 125,000 energy over 120 seconds,
  which is 1,042 energy a second for the whole two minutes, fifty-two solars or a fusion and a half of draw, and the
  silo refills itself to ten without being told. The first missile is therefore seven minutes after the silo is begun
  at 600 power; ours came 5.5 and 5.3 minutes after (begun 14:29, ready 20:00; begun 13:19, ready 18:38). It falls
  anywhere on the map for 12,500 at the centre and nothing at 640.
- *The anti-nuke's bill.* 1,500 metal, 38,000 energy, 60,000 of build: 100 seconds at 600 power, 200 at the
  commander's 300, and 90 seconds more to its first interceptor (150 metal, 7,500 energy), to a stock of twenty,
  covering 2,000 around it. So the first silo and its first missile are 9,100 metal against the answer's 1,650, 5.5
  to 1; each missile after is 1,000 against 150, 6.7 to 1; the interceptor reloads in 90 seconds against the
  missile's 120, so one anti-nuke outlasts one silo for ever, and two silos against one anti-nuke empty its stock of
  twenty only after 20,000 metal of missiles. That is "denser in every way", and why the escalation goes to the
  anti-nuke.
- *The two clocks.* The anti-nuke is ready 3.2 to 4.8 minutes after it is begun; the first missile seven after the
  silo is. A defender who sees the silo begin has two to four minutes in hand and spends 1,650. The nuke's whole
  value is therefore the silo unseen (D-reach-nests: sight is what we lack), or a target outside the circle: in
  nuke-2-hard our three missiles at his base were all intercepted after his anti-nuke stood (begun 21:40, standing
  28 seconds later, its first interceptor by 23:38), and the four at his forward army, outside the 2,000, all landed.
  Two missiles at 20:38, before it, would have taken a plant, a fusion, a gantry, a Shiva and three advanced
  extractors. A missile takes 22 seconds to arrive and a commander walks: both of ours aimed at his lived.
- *His clocks.* BARb hard_aggressive begins an anti-nuke in 27 of 33 games past twenty minutes, at 21 to 25, standing
  near 24:40, so a nuke to land on it is stockpiled before about 21:00; its own silo is rare at +0 (14 of 848 long
  games, at 25 to 38) and at +20 one landed unseen on 14,075 metal of our home at 39:33, the first tier-3 unit having
  been seen at 28:50 (M-barb-hard-aggressive-comet, M-barb-bonus). The people we play have never built a silo in the
  two-to-four-a-side games, so an anti-nuke before 25:00 against them buys nothing (M-format-team-small); the AI teams
  of the 8v8 lobbies begin one at 8:30 to 9:21 in seven of forty games, standing 10:03 to 12:08, and the people answer
  with an anti-nuke at 13:47 at the median, which the chat calls "the 12 min nuke" (M-format-8v8).
- *The Banshee snipe.* A Banshee is 135 metal, 2,300 energy, 38.6 a second, 185 of speed: eight are 1,080 metal and
  18,400 energy after an air plant's 650, and together they put 309 a second into a commander's 3,700, twelve
  seconds. A Nettle (80 metal, 330 health) takes the commander six seconds to build and dies to eight Banshees in
  just over one, which is "a com cannot build aa fast enough to stop 8 banshees if you shoot the aa that is building
  then refocus the com each time"; the eight cross Comet corner to corner in 43 seconds. He fielded twenty against us
  at 14:00 in one replayed game and six at 18:00 in another (M-gecko-vs-us-comet). The answer before it exists is a
  Nettle beside the commander and Falcons (73 metal, 139 a second against air, "the strongest anti bomber per cost",
  L-sheet-labs); after it exists there is none but distance.
- *The Twilight.* An extractor at 200 metal that self-destructs as an EMP blast of radius 400 which stuns what it
  catches for up to fifteen seconds (the game's empblast: paralyzer, 4,450 of stun damage, more than any tier-1
  unit's health). A stunned army is a free fight for fifteen seconds; "giving you a chance to counter attack
  unblocked, or attack the frozen army". Its price is one extractor's place and the walk of his army over it, which
  the first big fight of a game supplies.
- *The others.* Bombers on his energy (D-energy-second: a solar closes when hit, the air plant "around 30m/s"), the
  hovercraft platform which "can supprise players who forget it exists because of how bad the units are per cost"
  (L-sheet-labs), spybots. Each is the same shape: a blow before the answer, and nothing after.

**Consider, and why it may work.**

1. *The anti-nuke on his clock, not on fear.* Against BARb at +0 his silo is 14 of 848, and a 1,500 insurance in a
   game that ends at 15:00 is a plant not built; with a bonus, the first tier-3 unit in sight is the sign (28:50
   before a 39:33 blow in the one game), and 1,650 against 14,075 is the trade; in the 8v8 lobbies against the AIs
   the tech seat's silo stands at 10:03 to 12:08, so the anti-nuke is a 12:00 build on the lane it will cover; against
   the people never before 25:00 unless one is seen.
2. *Our nuke only before his answer, at what stands still.* Against BARb hard_aggressive that is a silo begun by about
   14:00 at 600 power so the first missile is ready before his anti-nuke stands at 21 to 25; aimed at his lab, his
   fusion, his gantry, which do not walk, and never at his commander; and at his forward army when his anti-nuke
   stands at home, since the circle is 2,000 and the map is 8,000.
3. *The snipe as the losing side's move.* "Rather than try and recover by matching a stronger opponent": an air plant
   he has not seen, his commander found (D-reach-nests), eight Banshees in one flight; "might even work with as few as
   6, but 15 starts to be a pretty safe way". It is the lead article's third way to spend a game that is otherwise
   gone, and the one case where metal spent on nothing that compounds is right.
4. *The guard against the snipe is 80 metal, early.* A Nettle beside the commander from his air plant's first
   sighting, and the commander not alone at the front; the eight Banshees are 1,080 metal that his air plant's
   sighting announces four minutes ahead.
5. *The Twilight at the place of the first fight.* One of the forward extractors, where his first army will walk,
   held for its blast rather than its metal; fired when the body is over it and ours is a step away.

**Where it bends.** (1) The energy: a stockpiling silo draws 1,042 a second for twenty minutes, the largest single
draw in the game, so the nuke is paid in the energy article's prices before it is paid in metal. (2) A person's
anti-nuke is cheap for him too, and he reads the silo's sign where BARb reads nothing: against a person the surprise
is harder and the circle larger. (3) In a team one anti-nuke covers one seat's 2,000, so the side needs one per lane
or the tech seat's one where the silo would aim; the 8v8 people put theirs on the tech seat's lane in 11 of 37 and the
air seat's in 6. (4) The snipe's eight Banshees need his commander seen, and ours saw his in 8 percent of lost games
(D-reach-nests). (5) BARb is the sparring partner (M-barb-sparring): its anti-nuke clock and its silo habit are its
own, and the people's are above.

**When to depart from the line.** When his silo is seen before his anti-nuke clock says so: the anti-nuke now, the
1,650 against whatever stands within 640 of our plants. When our nuke is begun and his anti-nuke stands before the
first missile: stop the stockpile (the kept count) and fire at his forward army or nothing; the metal is the loan the
tier-2 article names. When the game is lost on the ground and his air is not up: the snipe, once.

**The tests.** No tool of its own yet. The ledger: nuke-1-medium (the silo begun 14:29, standing 17:31, the first
missile 20:00, fired 21:07 at his lab, everything within 640 gone but his commander; eight missiles by 37:37, two on
empty ground, two on a walking commander) and nuke-2-hard (begun 13:19, standing 16:32, the first missile 18:38, the
first firing asked for at 23:39 and three at his base intercepted after his anti-nuke of 21:40, four at his forward
army landed) carry the two clocks as measured; rework-ladder-20-1 the unseen silo at 39:33. The kill-test as named:
had the nuke been a weapon of attrition, missiles after his anti-nuke stood would land at a rate worth their 1,000;
of ours, three of three at his base after it were intercepted and four of four outside its circle landed. What the
tests cannot do: three games, all against BARb, no Banshee snipe of ours ever flown and no Twilight ever fired; the
sheet's figures for those are his.
