# Part e: the role texts and the hands' standing texts (reader E)

Files: `crates/bot/src/strategist/player.md` (445 lines, 71 rows), `commander.md` (261, 44), `player-under-command.md`
(41, 10), `packet-example.md` (56, 9), `docs/briefs/player-opus.md` (33, 7), `crates/bot/src/brain/pianist/rules.md`
(2 lines, 16 rows by part), `rules_gate.md` (1, 1), `default.md` (5, 5). 163 rows in `part-e.jsonl`; every line of
every file is in some row's range (checked by script), every row's quote occurs in its range, every line parses with
the schema's keys. The tree is d44e318's; none of these eight files is in the uncommitted change.

Where each text is live, from the code (`crates/bot/src/texts.rs`, `strategist/mod.rs`, `brain/pianist/*.rs`):
`player.md` is every player's role text, with the `<!--brief:standard-->` / `<!--brief:experience-->` passages kept one
at a time and `rules.md` pasted in whole at `<!--hands-rules-->`; `player-under-command.md` is added under a commander;
`packet-example.md` only under `arena --packet-example` (one commit, 2026-10-04, never revised); `player-opus.md` only
when the player model starts with `claude-opus` (one commit, 2026-10-07, never revised); `rules.md` is read at every
pick; `rules_gate.md` replaces it for gate requests at `--hands-effort lean` only (the human games run at normal);
`default.md` is the hands' packet when no player instructions and no `--packet` file stand (a pianist-alone game, or any
second before a packet lands).

## Counts

By layer (what the passage is now): tool 78, mixed 49, derived 23, episode 6, fact 4, meta 3.

By fate (what it would become): tool 106, rederive 34, meta 8, fact 7, episode 4, discard 4.

Doctrine flags: 60 rows carry a rule ("never", "always", "only", "the one case"): 45 from one case, 8 from a ruling,
7 from the pool. 35 rows name a conflict, 85 a duplicate, 29 checked a number (against `units.csv` or the reference sheet).

The tool share is the point of this part: 106 of 163 rows stay in the role prompt. The other 57 are the doctrine that
has grown inside the role texts and the hands' rules, listed in the last section.

## The derivation backlog (fate `rederive`), by subject

Each subject names the lessons the derivation must reproduce or explain away, with the rows that carry them.

1. **The going doctrine** (commander.md:56-58, 138-142, 155-160, 161-162, 172-174; packet-example.md:17-19; player.md:382-384).
   Must reproduce: a leading army in our half loses its lead (human-23 x2, packet-example-1); "goes now, never a place
   in our half, a count or a clock"; the raid protects the extractors by keeping his attention home. Must explain
   away or bound: the brief's "arrive together or the push waits" and the players' words ("your armies keep not fighting
   together"), K-army-piecemeal-midmap, going-1 (the going lost the economy it built), and player.md:318-325's "the
   ball comes home" when the economy crashes. Premises: K-army-an-army-lead-unspent-goes-bad; the pool's army in his
   half (winners 416 against 296 at 8:00, 1,540 against 361 at 12:00); unit speeds and the march rule; static defence's
   free damage.
2. **Gathering only for a defended position** (commander.md:163-167). Must reproduce human-23's 33 Incisors dead in
   six seconds at an unlooked base. Premises: K-units-static-defence-value, sight shorter than range, K-t2-tower-line-needs-mass.
3. **The tier-2 sprint's trigger and conduct** (commander.md:17-22, 22-30; player-under-command.md:31-39 as tool).
   Must reproduce: Center Command 2v4 (450 metal fed in nineteen minutes against three surplus asks; the plant after
   3:24 on 9 a second beside thebluegecko's 1:46); the +60 Altored 3v3 (two seats zero mohos for an hour). The trigger
   "first bank or the side's spots taken, not when his tier 2 is seen" is a ruling plus one game; derivable in part
   from the costs (armavp 2,600, moho 620/640, the reclaim of the tier-1 lab) and K-t2-the-gap-is-covered-by-allies.
4. **Tier 1 against tier 2 and 3** (commander.md:38-42). Must reproduce the 338 of ours killed by Juggernauts after
   44:00 in apex60-1; must bound "at any number" (the square law says numbers matter). Premises: the units' stats.
5. **The raid answer** (player.md:361-369, 370-373 as meta, 378-380, 431-432; rules.md:1 part 5; player-opus.md:22-24,
   25-27, 28-29; packet-example.md:20-22). Must reproduce: human-21 (the answer must stand in the packet before the raid,
   since the next packet lands 10-20 s later); K-army-distance-decides-a-raid-response (79% within 600, 23% from nobody);
   player-57's thrash (a body turning for a lone Rover kills nothing); human-26 (the raid rule last in a paragraph rated
   0.43, first 0.60). Must resolve: "units that can catch them" (player.md:431) against "a slower chaser still drives
   it off" (rules.md); the Blitz does not catch a Tick (101 against 132). Premises: speeds, distances, the raider's
   behaviour at a chaser's arrival (BARb's is code; a person's is unmeasured), the body's cost of turning.
6. **The 600 rule** (rules.md:1 part 10). Must reproduce its offline measurement (holds beside a base under attack
   2% to 28-32% right, 2026-09-22) and must be reconciled with the packet's prohibitions it overrides (the example
   packets' "NEVER", player-opus's "never the body's") and with the menu's 4x withholding. Premise: within 600 the fight
   is already on (tier-1 ranges 180-430).
7. **The turret with the spot** (player.md:305-313, 291-304 part 2; default.md:3 as the counter-case). Must reproduce
   going-1 (seven turret steps for twenty extractors by 8:00, three standing, 20 to 4 extractors by 12:30) and the pool's
   light turrets by 8:00 (7 / 10 / 13) and first extractor lost at 4:21. Premises: a spending seat's store near zero (so a
   bank condition is never true), Sentry 85 m / reach 430 against the raider's cost.
8. **Expansion pace as tempo** (player.md:285-290, 291-304 part 2). Must reproduce the pool's curve (7 / 15 / 22 at
   4 / 8 / 12) as cover-limited, not clock-limited; must source "four to six constructors on expansion" (the pool stands
   at 3 at 5:00, 5 made by 8:00). Premises: K-army-distance-decides-a-raid-response, K-maps-comet-middle-spots-die-first.
9. **The classic failure** (player.md:318-325). Must reproduce the early arcs' collapses (fifteen to one extractors
   while the ball fought in his half; undated). Must reconcile its "the ball comes home" with the going doctrine.
10. **The commander's conduct** (player.md:76-83 parts 2-3; rules.md:1 part 12). Must reproduce wake-4 (the commander
    dead rebuilding beside Stouts and Warriors while 26k against 3k ahead) and the early "a handful of Pawns die to it".
    Premises: K-rules-commander-death-ends-game, the D-gun, range 300 / speed 37.5 against tier-1 ranges and speeds, the
    fighting-worth odds (about 400 metal, code).
11. **The odds model** (player.md:380-382; rules.md:1 part 4). Must reproduce "a turret counting about three times its
    metal" (a code constant whose source no text gives; units.md calls it a nine-in-ten sim result) and "never into a
    turret line at parity, arrive together". Premises: the square law, free damage on the approach, K-army-groups-arrive-strung-out.
12. **The bank** (player.md:429-430; commander.md:121-129). Must reconcile three answers to a climbing bank (a second
    factory; construction turrets on the factory; "build power missing, not a plant missing") and the pool, whose
    winners bank more than the losers at 8:00-9:00 (1,148 against 614) before the second factory at 8:11. Premises:
    K-mech-build-power-ratio, nano 230 m / 200 bp against plant 590 m / 150 bp, K-mech-turrets-assist-in-radius.
13. **The opening's bound and the energy rule** (player.md:35 part 3, 437-439; rules.md:1 part 2). Must reproduce
    "the plant's metal is the limit, the store never reads 0" (the pool's commander assisting the plant), the solar rule
    (STALLING and 150 metal: the 150 has no source), and must bound "energy only needs to keep the store from emptying"
    to tier 1 (mohos, converters and towers bind on energy later). Premises: the plant's draw, K-mech-stall-slows-everything,
    the pool's solars by 8:00 (12 / 14 / 17).
14. **Unseen fire** (player.md:263-269; rules.md:2 part 2). Must reproduce "a ball's worth lost in three games" (unnamed)
    and reconcile "see the shooter first, never make shelling a standing destination" (player) with "close on the shooter
    when the estimate says we outweigh it" (hands). Premises: artillery reach against sight, the bearing estimate's error.
15. **Scouting by sweep** (player.md:326-340). Must reproduce K-hands-scout-at-the-guess-does-nothing. Premise: K-scout-enemy-base-is-a-spot-cluster.
16. **The side's sight** (commander.md:86-95). Must reproduce human-8v8-1's twenty blind minutes. Premise: line of sight by terrain (engine).

## The meta catalogue (fate `meta`)

| entry | population | map | date | rows |
|---|---|---|---|---|
| the Comet pool's extractor curve: 7 / 15 / 22 at 4 / 8 / 12; winners 27, losers 18 at 12:00; income 39 against 34 at 8:00; the winners add 11 and the losers 1 from 8:00 to 12:00 (paired, 18 games) | 40 public duels, both sides OS 40+ | Comet Catcher Remake 1.8 | survey 2026-09-23 | player.md:278-284, 425-428 |
| constructors by the clock: first at 1:13 / 1:30 / 1:37; standing at 5:00 2 / 3 / 4; made by 8:00 4 / 5 / 7 | the same pool | Comet | 2026-09-23 | player.md:65-73; commander.md:110-120 (mis-stated as "about five by 5:00") |
| the first handful: first soldier in his half 1:22 / 1:40 / 1:49, first building killed 2:18 / 2:48 / 4:13; the players' words "the first handful runs to the other side" (relayed by the user 2026-10-03) | the pool; the experienced players | Comet | 2026-09-23, 2026-10-03 | player.md:433-436 |
| the commander set to low while assisting the plant or a solar, high for a build of its own; labs and nanos left low from birth | 14 of 41 players, OS 25-51, 75 pro duels | Comet | 2026-10-13 | player.md:35 part 5 |
| BARb's raid clock: squads of about four from the first minutes, outermost spots first, the army as one block later | BARb medium / hard_aggressive (code) | any | 2026-09-19 to 22 | player.md:370-373; rules.md:1 part 3 (stated to the hands as the opponent's nature, people included) |
| people build a silo very late or never (none in 21 minutes at 37 mohos; computer_whisperer's two after 30:00 at +100%); BARb hard_aggressive's anti-nuke at 21-25; Apex +60 a silo at 24:28 once, none the next game | thebluegecko, u6bkep, computer_whisperer; BARb; Apex v0.3.2 | Center Command, Cape Violet, Altored Divide | 2026-10-18 to 20 | commander.md:43-55 |
| how two people run mohos: one advanced constructor starts frames (19-33 s each with helpers; 90-135 s alone), the tier-1 pool finishes them, the constructor is passed between seats whatever the faction | thebluegecko, u6bkep | Center Command | 2026-10-19 | commander.md:32-38; player.md:35 part 6 |

## The episode index (games cited by these texts)

| game | date | opponent / arena | what the text draws from it | harness defects of its time | lens |
|---|---|---|---|---|---|
| human-7 | 2026-09-21 | a person (Matt), Cortex, realtime | "one constructor first, then raiders" got three constructors (player.md:184-200) | produce had no counts (built that day); pre-rebuild hands | ledger |
| hands-2-bulldogs | 2026-09-22 | BARb easy, Comet | five Bulls behind a solar six minutes (201-209); a Bulls-only search order (409-424) | no exit-lane keep-out; pre-rebuild | ledger |
| wake-3 | 2026-09-23 | the wake series (BARb) | 60 packets in 17 minutes swung the army (398-402) | hold clock restarted, retreat hidden, kill line read as own losses (all fixed after) | model (an Opus review, numbers verified) |
| wake-4 | 2026-09-23 | the wake series (BARb) | the commander died rebuilding the far north-east (76-83; rules.md commander clause) | odds by metal; builders not re-asked on a party in alarm reach; H-COM-RETREAT not under the pianist | ledger |
| player-31-hard | 2026-10-01 | BARb hard_aggressive, Comet | a reclaiming armrectr read as harmless (178-183) | unarmed party shown with no word of what it did (fixed in words that day) | person (the user's review) |
| human-10 | 2026-10-02 | thebluegecko, Gecko Isle, 3 seats | the hands played prose and built a lab at 0:10 (44-61) | factions set at the start; role words built that night | ledger |
| human-21 | 2026-10-03 | u6bkep, Comet | four Blitzes behind the base at 4:27 under "never comes home" (361-369; commander.md:130-137) | a party's name passed to a constructor (fixed 641cc72); the direction "do not bring them home" | person (the user's review ranks first) |
| human-22 | 2026-10-03 | thebluegecko, Isidis Crack, 2 Sonnet seats | "one constructor first, then raiders only": two builders at 4:00 (65-73; commander.md:110-120) | Sonnet players; the commander's opening direction | ledger (the user: Sonnet does not expand as Opus did) |
| human-19 | 2026-10-03 | thebluegecko, Comet, 2 Opus seats | the counter-case: five and seven builders at 6:00 (65-73) | the south seat's opening list to the wrong name | ledger |
| player-49 | 2026-10-03 | BARb hard_aggressive, Comet | the "now" in a route (92-100), replayed offline | none named | model (offline replay) |
| player-57-experience | 2026-10-03 | BARb hard_aggressive, Comet | the body left its advance for single Rovers (player-opus.md:25-27) | pruning cut 0.3; the forbidden mark read the detachment, not the whole-group attack | person (the user at 6:04) + ledger |
| duel-split-1 | 2026-10-03 | BARb hard_aggressive, Comet, Sonnet under Opus | energy, metal, energy: 11 extractors at 8:00 (commander.md:102-109) | Sonnet player; the commander corrected the minute on nine of ten turns | ledger |
| human-23 (first) | 2026-10-04 | a person, Comet, Sonnet under Opus | the gather held 23 in the middle to 7:20 (commander.md:143-148, 163-167) | commander not woken by aircraft or losses; wake lines "lost 2" at 33 dead; ten build orders never started | person (the host) + chat; review.md a model's |
| human-23 (second) | 2026-10-04 | the same | the raid held "until 8 extractors stand" (149-151, 161-162) | hunts against a forbidding packet; produce refused; six orders never started | person: the user read it as defence, the text uses it for going (a lens mismatch) |
| packet-example-1 | 2026-10-04 | BARb hard_aggressive, Comet, Sonnet under Opus | five signals set ahead; the body stood (152-154); the packet shape (packet-example.md) | Sonnet player; a won game | ledger |
| going-1 | 2026-10-04 | the same line, the commander's text rewritten | turrets gated on the bank came after the raiders (player.md:305-313) | Sonnet player; traced: not a harness fault | ledger |
| player-59 | before 2026-10-04 | BARb hard_aggressive, Comet, Opus | the two example packets (packet-example.md:26-56) | unknown | ledger |
| human-26 | 2026-10-07 | people, realtime 2v1 (no ledger row) | orders land late; the raid allowance first (player-opus.md) | turn-cost words in the hands; no recently-seen line (both changed that night) | person (the user's review at 3:30) + model (offline replay) |
| nuke-2-hard | 2026-10-09 | BARb hard_aggressive, Comet | BARb's anti-nuke and silo clock (commander.md:43-55) | the report never named the silo's rounds (fixed after) | ledger |
| human-8v8-1 | 2026-10-18 | five people + Apex allies, Altored Divide | the ridge hid the base from radar (player.md:252-257; commander.md:86-95); the sprint unnamed | no radar-now line or sight words (the sight arc came after) | chat (the user, drakcore) + ledger |
| Cape Violet 1v1 | 2026-10-18 | a person at +100% | no under-water spot taken, "a Muskrat later" (commander.md:65-74) | the review's author is not named in the texts | unknown |
| apex-v032-altored-3v3-apex60-1 / -2 | 2026-10-19 | Apex v0.3.2 at +60 | the lost sprint and the Juggernauts; the unseen silo (commander.md:38-55) | the sprint wording lost from the briefs (restored for -2); missile deaths unattributed | ledger + a model's debrief |
| Center Command 2v4 | 2026-10-19 | thebluegecko and u6bkep | feeding is production; the people's mohos (commander.md:22-38; player.md:35; player-under-command.md:31-39) | no standing transfer (built after); cross-faction constructor words; a tagged plant step fired twice | the user's reading + the replay |

## Numbers checked

Against `docs/glossary/units.csv` (one lookup each):

- commander 2,700 m, 300 build power, range 300, speed 37.5, 3,700 hp: match (armcom, corcom, legcom). The "about 400
  metal of tier-1 soldiers" is the odds code's square-law figure, not a glossary number.
- construction turret 200 build power (230 m): matches; "a constructor 90" is armcv (armck is 80); vehicle plant 590 m / 150 bp: matches.
- armavp 2,600 m: matches (coravp 2,600). Metal storage 330 / 340: matches.
- anti-nuke 1,500 m: matches (armamd, corfmd); 1,500 / 620 = 2.4 and 1,500 / 640 = 2.3, so "two and a half mohos" holds.
- advanced constructor 430 / 550 against moho 620 / 640: "a moho's worth of metal" is 70-90%; fusion 3,350 / 620 = 5.4
  and 3,600 / 640 = 5.6: "five mohos" holds. The moho's ~100 s repayment needs the spot's yield: not a lookup.
- Juno 660 m is Cortex's (armjuno 640). The camera's 32 m matches both; its "540 sight" is coreyes (armeyes 560).
- Pawn 54 m / 370 hp / range 180 / speed 87; Blitz 110 m / 730 hp / speed 101; Rover 31 m / speed 168; Tick speed 132;
  Stout 225 m / range 350 / speed 75: match. So "units that can catch them: Blitzes against Ticks" fails (101 < 132).
- Sentry 85 m / range 430, Guard 90 / 435; extractor 50 m; solar 155 / 150 m; wind 40 / 43 m: match.
- Not glossary columns, unchecked here: jammer radii (500 / 360 / 390 / 760 / 700 / 500), the Juno's 1,400, the shields'
  550 / 6,175, the silo's 9 s / 13 s / 12,500 / 640 / ten missiles / 1,042 energy a second, the anti-nuke's 2,000 and 90 s,
  the amphibious flags. late-game.md carries the silo and anti-nuke figures.

Against the reference sheet (`docs/briefs/reference/comet-catcher-remake-1.8-arm.md`): player.md's pool curve holds
(7 at 4:00, 15 at 8:00, winners 27 / losers 18 at 12:00, income 39 / 34; "23 at 12:00" is one off the median 22);
player.md's constructors (1:30; 3 at 5:00; 4-7 by 8:00) hold; commander.md's "about five by 5:00" does not (2 / 3 / 4
standing, winners 3); the first-handful times hold; the winners' bank at 8:00 (1,148 against 614) contradicts "never
that you are ahead".

## Conflicts

Within my files, and against the other readers' files by subject:

1. **Who answers a raider.** player.md:431-432 "units that can catch them ... never Hammers, Maces or Stouts on their
   own" against rules.md:1 "a chaser slower than the raider still drives it off, without catching it" (the user's own
   framing, 2026-09-27). The player is told one rule, the hands run another.
2. **The hands' 600 rule against the packet's prohibitions.** rules.md:1 "fought now, whatever the instructions call it"
   against packet-example.md:42 "NEVER answered by Blitzes or Stouts", player-opus.md:25-27 "never the body's", and the
   menu's 4x withholding (player.md:345-360). The mechanism under player-57's thrash is written into the shape players copy.
3. **Going against arriving together and coming home.** commander.md:155-160 "it does not come home ... Never give it a
   place in our half to stand, a count to reach, or a clock" against player.md:318-325 ("the ball comes home and the
   economy is rebuilt first"), player.md:380-382 ("arrive together"), and docs/briefs/player.md:299-300 and 893-894
   ("one gathering place ... arrive together or the push waits; new units join at the gathering place, not at the front").
4. **What pickets the spots.** player.md:370-373 (a soldier picket of one or two at each outer cluster from the first
   Blitzes, the user's 2026-09-22 observation of BARb) against player.md:433-436 ("the picket for our extractors is the
   turret beside each, not the first army", the players' words 2026-10-03). Both stand in the same file.
5. **Constructors by the clock.** commander.md:110-120 "about five by 5:00" and its rule "five constructors a seat by
   5:00" against player.md:65-73 "three stood at 5:00 (the sheet's median; four at its upper quartile)" and the sheet;
   docs/briefs/player.md:543 "four a seat by 4:00 and eight by 8:00" is a third number. player.md:291-304's "four to six
   on expansion" has no source.
6. **The bank.** player.md:429-430 (a second factory, soldiers or constructors; "never that you are ahead") against
   commander.md:121-129 (construction turrets, not another factory), docs/briefs/player.md:820 ("build power missing,
   not a plant missing"), and the pool (winners bank more at 8:00-9:00).
7. **The anti-nuke's timing.** commander.md:43-55 (the user 2026-10-20: on the opponent's clock) against
   docs/briefs/player.md:168 (the user 2026-10-19: due diligence once his tier 2 stands). late-game.md:172-183 carries both.
8. **The default packet against everything after it.** default.md:2-5 (a wind opening; a turret beside a raided
   extractor; "one constructor first, then raiders"; a group of four holds at passage_1; an outweighed group walks home)
   against player.md:437-439, 305-313, 65-73, 123-126, commander.md:110-120 and 155-160, and the 2026-10-01 ruling on
   fall-back places. It is live in pianist-alone games and in any second without a packet.
9. **The commander's clause, three ways.** rules_gate.md "never walks toward a fight"; rules.md "never toward a party
   that outreaches its D-gun or outweighs it; away from home walks to a named place before it is hit"; default.md:2
   "Walk home (go) at once when under fire".
10. **"A never sentence holds"** (player.md:162-163, 442-443) against K-jev-never-is-not-heard (supported 2026-09-22: a
    prohibition does not remove the option, chosen at 0.4-0.8 when its words fit). Which is current is unknown from the texts.
11. **Unseen fire.** player.md:263-269 (see the shooter first; never make shelling a standing destination) against
    rules.md:2 (close on the shooter when the estimate says we outweigh it).
12. **The pool's curve in two spellings.** player.md:278-284 (7 / 15-16 / 23) against docs/briefs/player.md:614 (7, 13, 17
    at 4:00, 6:00, 8:00: the 17 is the winners' median, 13 is nobody's).
13. **"Energy only needs to keep the store from emptying"** (rules.md:1) against the late-game part (energy binds the
    tier-2 economy; apex50's debrief) and docs/briefs/player.md:165.
14. **The energy rule past the opening.** player.md:437-439 (a basic solar when STALLING and 150 metal) against
    docs/briefs/player.md:209-212 (every later generator an advanced solar; basic only when STALLING or empty) and default.md's wind.
15. **BARb's raid clock as the opponent's nature** (rules.md:1, player.md:370-373) against the people's shapes
    (team-games.md:9-26, human-23's gunship rush).
16. **A lens mismatch, not a text conflict:** human-23's second game is cited for the going doctrine
    (commander.md:149-151) while the ledger's person-ranked reading of it is "poor defensive control".
17. **Two tool sentences:** player.md:385 "The game is paused while you take a turn" against 10-18 (with people it runs on).

## Era-bound passages

- **Pre-hands-rebuild (before 2026-10-02):** wake-3's 60 packets (player.md:398-402), wake-4's commander death (76-83;
  rules.md commander clause), hands-2's Bulls and search order (201-209, 409-424), human-7's three constructors
  (184-200), the 600 rule's measurement (rules.md:1 part 10, 2026-09-22), the odds with reach (rules.md:1 part 4,
  2026-09-26), the whole of default.md (2026-09-21, last touched 2026-10-02), "the classic failure" (318-325, undated
  but of the early arcs).
- **BARb medium:** K-army-distance-decides-a-raid-response behind player.md:378-380 (1,106 raids, 2026-09-20); the raid
  clock (370-373; rules.md:1 part 3).
- **BARb hard_aggressive:** going-1, packet-example-1, player-49, player-57, duel-split-1, nuke-2-hard, player-59's
  packets; the arena target from 2026-09-27.
- **The pool (Comet only):** player.md:35 (the commander on the plant; low priority), 65-73, 278-284, 425-428, 433-436;
  commander.md:110-120, 121-129. The "How games on this map are won" paragraph is Comet-specific inside a role text
  every map reads.
- **People:** human-10, -19, -21, -22, -23, -26, human-8v8-1, Cape Violet, Center Command; player-opus.md whole (one
  game, 2026-10-07, never revised).
- **Apex:** the +60 Altored 3v3s (commander.md:38-55).
- **Undated / unsourced:** "a ball's worth in three games" (263-269), the spot_62 Pawn (373-378), "four to six
  constructors" (291-304), the 150-metal solar threshold (437-439), "two to one" (431-432, default.md:4), the 3x turret
  constant (380-382, rules.md), "our games reach 5 at 4:00, 12 at 8:00" (278-284).

## The doctrine found inside the role texts and the hands' rules

By file and lines; `tool` rows are left out. What a copying player or the hands inherit as law:

**player.md** (the player's role text, every game):
- 35 part 3: the opening is metal-bound; the pros' commander guards the plant and the store never reads 0.
- 35 part 5: the commander low while assisting, high for its own build; plants and nanos low always "as every human's are".
- 35 part 6: one advanced constructor lays moho frames the tier-1 pool finishes, the way the people run theirs.
- 76-83 parts 2-3: the commander is the early army; later never toward what outreaches its D-gun or into unseen ground.
- 123-126: the step-back place is the player's; the hands have none (a ruling).
- 263-269: the four answers to unseen fire; never make shelling a standing destination.
- 278-284: "Metal is everything"; the Comet pool's curve as the pace (map-specific).
- 285-290: tempo, not schedule; the count by a minute is the turn's call.
- 291-304 part 2: name the next spots and the count; four to six constructors on expansion; middle spots need a turret; rebuild the minute the raider is gone.
- 305-313: the turret is the next step after each outer extractor; never gated on the bank (going-1).
- 318-325: the classic failure; the raid answer stands before the ball leaves; the ball comes home unless the kill is sure.
- 326-340: sweep the army through named spots when the evidence is thin.
- 361-369: the in-base rule written before the raid (human-21); name the enemy by what and where.
- 370-373: pickets of one or two at each outer cluster from the first Blitzes; BARb's raid clock.
- 373-378: an escort by follow; two from the plant are enough against a lone raider.
- 378-380: defence decided before the raid; a group at home protects home, at a passage everything behind it.
- 380-382: metal on the spot, a turret 3x; never into a turret line at parity; arrive together.
- 382-384: a clearly bigger army goes for his commander.
- 429-430: a bank is spending behind, never a lead; spend it that turn.
- 431-432: raiders answered by catchers; line to raiders two to one; a turret beside every outer extractor.
- 433-436: the first handful crosses the map; pressure stops a greedy opening; the picket is the turret.
- 437-439: a solar when STALLING and 150 metal; never "no solars, ever"; the opening's stall is the plant working.

**commander.md** (every commander):
- 17-42: the sprint whole: which seat and when; feeding is production (Center Command); the plant's first product by three cost ratios; one advanced constructor shared across seats and factions; the +60 Altored lesson; tier 1 trades badly against tier 2 and 3 "at any number".
- 43-55: the anti-nuke on the opponent's clock (the user 2026-10-20), with the people's, BARb's and Apex's clocks.
- 56-58, 138-174: the going doctrine (goes now; never a place in our half, a count or a clock; gathering for a defended position only; the raid protects the extractors; a signal ahead is a chosen delay; after a won fight the body goes on), from human-23 x2, packet-example-1 and going-1.
- 65-74: the under-water spots are the first direction's question (Cape Violet).
- 102-109: give the aim over minutes, never the minute's correction (duel-split-1).
- 110-120: constructors by the clock, "five a seat by 5:00, then one in every four or five units" (mis-reads the sheet).
- 121-129: a climbing bank with little power in reach wants construction turrets, not another factory.
- 130-137: do not send a body back out after a raid (human-21).

**player-under-command.md:** 9-14 (the going doctrine carried into the seat's packet: no place to stand or count of your own); 24-28 (never a clock or a count for a signal: a measurement of Jev); 31-39 (the feeder's conduct, Center Command).

**packet-example.md:** 17-19 (going is the standing order); 20-22 (raids written before they happen); 26-39 and 40-56 (two real packets whose embedded rules a copying player inherits: the solar rule, "never goes back to a place it has reached", "the whole army never chases one raider", "a single armfav is NEVER answered by Blitzes or Stouts", the commander's leash).

**player-opus.md** (Opus players only, one game's lessons from 2026-10-07): 22-24 (the interruption before the job); 25-27 (a lone Rover is the picket's or one soldier's, never the body's; a swarm is everyone's); 28-29 (no exclusions against the unseen raid); 30-33 (a party that burned one base is on its way to the next).

**rules.md** (the hands, every pick; also pasted into every player's role text): part 2 (metal is everything; energy only keeps the store from emptying); part 3 (BARb's raid clock as the opponent's nature); part 4 (the odds: metal on the spot, a turret 3x, reach a cost; an outranged group that outweighs goes in as one body); part 5 (a small detachment for a raider; the soonest to drive it off wins, the commander last; a slower chaser still drives it off); part 10 (a party within 600 that the group outweighs is fought now, whatever the instructions say); part 12 (the commander's conduct); line 2 part 2 (a group with nothing in sight is not idle; close on an unseen shooter when the estimate says we outweigh it).

**rules_gate.md** (lean effort only): the same in short, with "The commander never walks toward a fight".

**default.md** (no player connected): a complete pre-rebuild doctrine (a wind opening; the reactive turret; "one constructor first, then raiders"; hold at passage_1 with four; walk home when outweighed), all contradicted by the texts above.

## What I could not classify

- The "our games" curve in player.md:278-284 (5 / 12 / 16-17) and "the classic failure" (318-325): which games and
  which era they summarise is not in the text or the ledger rows I read; marked undated and pre-rebuild on the shape.
- "A never sentence holds" (153-177): the role text and K-jev-never-is-not-heard disagree and I cannot tell from the
  texts which describes today's Jev (the forbidden mark of 2026-09-30 sits between them).
- The author of the Cape Violet 1v1 review (commander.md:65-74) and of the three unnamed games behind "a ball's worth"
  (player.md:263-269): lens `unknown`.
- The source of the 3x turret constant, the 150-metal solar threshold, "two to one", "four to six constructors": no
  text, claim or ledger row names one.
- human-26 has no ledger row; its facts in `player-opus.md` rest on `docs/studies/2026-10-07-raid-packets.md`
  (the user's review and an offline replay) and nothing else.
