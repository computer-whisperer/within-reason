<!--meta
id: D-lead-convert-it
layer: derived
premises: [D-metal-is-the-game, F-eco-extractors, F-eco-consequences, F-fight-consequences, F-fight-range, M-pool-closing, M-pool-fights, M-pool-tier2-and-wrecks, M-barb-hard-aggressive-comet, M-barb-bonus]
lessons: [L-lead-use-it, L-lead-together]
sources: {people: [thebluegecko, Irishstud14, the user], dates: [2026-09-27, 2026-10-22], relay: the lessons' chat lines; the user's framing of the unconverted lead as a catch-all for harness bugs, poor army movement and wrong tempo recognition, to be kept apart by name}
status: live, checked (the kill-test run 2026-10-22 with run/metal_killtest.py windows and pool-windows over 2,467 games with a result: a surplus kept home wins a third to 41% in every population, sent 71% to 82%; the quantity was narrowed in the running, see the tests)
-->
### The unconverted lead

**The heuristic.** thebluegecko, at 6:49 of the 3v1, the side up almost four times in army and behind in income: "if
you dont use that army advantage this will soon go bad"; Irishstud14 at 8:35: "push the advantage. kill his base".
The plain line: a lead in army that stands in our half is not a lead; it becomes one in his half and it is gone once he
has built through it. Follow it from the first minute our army is half again his and the margin is a fight's worth of
metal, five hundred or more, without waiting for a better one.

**The quantity.** Where the surplus stands, and what it has done to his income. The surplus is our army metal above
his. It is converted when it stands in his half and his extractor count is falling; it is unconverted while it stands
in ours and his count holds or rises; and it is spent when the game ends, which in the pool is three minutes after it
appears. Not the ratio alone: twice the army at minute 4 is a few hundred metal, one fight, and reading that as a lead
is the tempo mistake. The margin is in metal, against what covers his nearest spots.

**The mechanism.** Three clocks run against a surplus that waits.

- *Two growing stocks.* Army is metal spent, and his production goes on while his spots stand (the root article).
  A surplus is the difference between two growing stocks; when his income is level with ours the difference does not
  grow by itself, and when his is larger it shrinks. In the pool the first window's surplus is 46 seconds of the
  other side's income at that minute (28 to 76; windows of 1.5 times and 500 metal, minutes 6 to 14); in our own
  model-played games it is 156 seconds (59 to 308), and half of those were lost. A fact of the economy, not of the
  opponent.
- *The price rises.* A light laser tower fires 2.5 seconds before a Blitz can answer and kills it in 5.7; a turret
  beside a spot turns the surplus's job from a raid into an assault (facts of the fight). What stands beside his spots
  is the opponent's habit, so this clock is meta: BARb hard_aggressive has 11 turrets at 8:00, 16 at 10:00, 21 at
  12:00 and 35 at 20:00, one beside nearly every extractor, and the same surplus buys less of his base every minute
  it waits.
- *The tier cliff.* A Bull kills a Stout in 7.3 seconds and the Stout the Bull in 58 (a fact): past his advanced
  factory a tier-1 surplus is spent on worse terms. When that factory comes is meta: BARb starts it at 10:11 / 11:04
  / 11:57 in 103 of the 106 games that last fifteen minutes; the pool never before 12:51 and in four of eighty sides,
  so the duels there are decided on tier 1 and the cliff is BARb's, not the game's.

The pool's close follows from the three: the winner first holds twice the army at minute 7 and the game ends 3.4
minutes later, with the loser's commander and every factory among the kills in 29 of 29. Nobody there holds a lead
long enough for the clocks to matter, which is the discipline.

**The three ways a lead goes unconverted.** Named here so the episode index can keep them apart; the article names
them, it does not find them in any game. (1) *Not seen:* the player's turns never name the surplus, the ratio read as
noise or the margin below the threshold, no reference line to read it against. (2) *Not sent:* named, and the orders
keep it home, or the hands do not carry them: the picket, the raid answer, "no detachments", a group that never
leaves. (3) *Not landed:* it went, three tenths or more of it in his half, and his extractor count did not fall:
groups arriving one at a time, a walk into his turrets, roving the near side of his half, the wrong instrument
against his cover ("your armies keep not fighting together", "not mobilizing fast enough is what lost you the game").
The curves separate (3) from the other two; only the transcripts separate (1) from (2). Harness bugs live in (2) and
(3) and are found there, not here.

**Where it bends.** When his base is past tier-1 reach (Gauntlets by 20:00 against BARb), the conversion is his outer
spots and the map, not the kill, while tier 2 comes. Against a bonus opponent his income runs 1.5 times by handicap
and every clock above runs faster, so the window is shorter than it looks. In a team the surplus is the side's and
has to arrive at one place at one time, which is the whole of Irishstud14's complaint in game 3. And the raid trade:
in the pool five sides killed three or more of his extractors during their window and lost anyway, the raiders
buying extractors while his main body took theirs.

**When to depart from the line.** When the surplus is the wrong instrument for what covers his spots (raiders against
turrets), spend it on his outer spots and bring the right one. When the neighbour's lane is breaking, the side's
surplus goes there first. When the window is a few hundred metal at minute 4, it is a raid, not a push. Never because
the lead is comfortable: that is the case the test below measures.

**The tests.** Run 2026-10-22 (`run/metal_killtest.py windows` and `pool-windows`): the first window of 1.5 times
his army and 500 metal between 6:00 and 14:00; "went" is three tenths or more of our army metal in his half during
it, "landed" is his extractor count falling by three or more from the window's first minute to two after its end.
Three populations agree on the heuristic: a surplus kept home wins 20 of 60 (33%) in the 266 duels the model played,
186 of 452 (41%) in the 1,535 the old code played, and 7 of 17 (41%) in the pool's duels seen from both sides; sent,
it wins 89 of 126 (71%), 242 of 317 (76%) and 46 of 56 (82%). A window that landed lost no game of ours (42 won, 0
lost) and 5 of 27 in the pool. Our model-played games with a window were lost 77 times: the surplus never went in 40
and went without landing in 37, the losses ending 12 to 22 minutes after the window where the wins ended two to ten
after; the old code's 341 such losses were 266 never sent and 75 not landed. Without a window, the model won 10 of
80 and the code 55 of 766.

The kill-test as named in the discussion was that his extractor count's fall during the window would call the
result better than the window's size or length; if size called better, the root would be "be bigger". It came out
split, and the quantity was narrowed because of it. In our model-played games the fall calls better (AUC 0.71, 0.75
by the curves' end, against 0.60 for size, 0.68 for the surplus and 0.61 for length); in the pool and the old code's
games the surplus's size calls best (0.84 and 0.86) and the fall worst (0.63 and 0.65), because those windows are
small, under a minute of his income, and the game ends within three minutes of them, before his count has had time
to fall. So his count's fall is the sign that the surplus landed, not the quantity itself; the quantity is where the
surplus stands, and its size matters until it can fight through his cover, which in our games it already could. The
next check, when the episode index exists: the 37 model-played losses that went and did not land, read one by one
for which of the three it was, against the transcripts.
