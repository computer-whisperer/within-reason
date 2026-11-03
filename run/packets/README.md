# Fixed packets for the pianist without a player

`arena --packet run/packets/<file>` gives the pianist this text as the player's packet for the whole game (the bot reads
`WITHIN_REASON_PACKET`): the arena instrument for A/Bs of the hands and the micro lane, at Jev's cents a game and with no
LLM session (`docs/design/2026-09-25-one-decider.md`, §2). A packet is the `instruct` text of a real player turn, so its
place names are the map's and the strip's: `comet-west-standing-2.md` is the first packet Opus 5.5 wrote in standing-2
(Comet Catcher Remake 1.8, the west strip, Armada, `--corner nw --mirror --place --side armada`).

`comet-west-hold-1.md` (2026-09-25 late, written by the micro agent for the lane A/Bs, same settings) is a packet that
holds the west strip against medium into the mid game: every count the packet keys on is a lookup table in the picture's
own words (K-jev-a-lookup-table-counts), the commander opens by table (extractor, extractor, solar, plant at 0:23) and
then helps the plant with a turret beside each extractor and a solar when STALLING, the constructors expand along the
strip under `job expand` with the middle spots forbidden, the plant makes four constructors then Blitzes then Stouts,
and group_A stands at spot_45 beside the commander with one-soldier pickets against lone Ticks. Check games hold-1 to
hold-9 (16 games, `docs/experiments.md`): against medium it reaches minute 20 in about half the games, with a body of
15-50 Stouts by minute 10-12 either way; the games end when medium's block of Maces and Rocketeers kills the commander at
home.


`comet-west-hold-table-1.md` (2026-09-27) is `comet-west-hold-1.md` with the constructors' paragraph rewritten as a table
keyed on our extractor count, the way the plant's paragraph is ("with 4 extractors: an extractor at spot_28"), with rows
for a lost spot, the turret, the solar (STALLING only) and no converters: the test of whether a job written as a table is
followed with no standing rules where the job sentence was not (K-jev-a-job-sentence-is-not-a-per-second-signal).
onepass-norules-hard-7: it is, in the table's order, 10 extractors at 8:00 against 2-4 with the sentence.

`comet-west-rebuild-1.md` (2026-10-02) is `comet-west-hold-table-1.md` with group_A's paragraph written for the rebuilt
hands (`docs/design/2026-10-01-hands-rebuild.md`): its station as a place it walks to and holds at, one-soldier
detachments (`send 1`) and no larger, a named place to step back to (`home`), and the detachments told to join the
group again. The first pianist-alone game of the rebuilt menu, rebuild-smoke-1.

`speedmetal-cells-1.md` (2026-11-02, the metal-maps design's step 2 judge) is a packet for SpeedMetal BAR V2 (Armada
from the west box, 13 by 2 cells of 1,024), the first written in the grid's vocabulary and on a map that publishes no
spots: the commander and the constructors build extractors "on the metal nearest" them (the `extractor` step with no
place, sited over the coverage), the constructors are told to stay in A1, A2, B1 and B2 and the commander never east of
C1, and group_A's places are cells: B1 to hold, D1 and F1 to advance to. Energy is the game there (30.6 metal a mex), so
the solars are keyed on STALLING and no converters are built. First game metal-step2-speedmetal-1.
