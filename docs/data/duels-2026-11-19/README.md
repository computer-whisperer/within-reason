# The tier-1 duel tables of 2026-11-19

Every pairing of 23 tier-1 land units at equal metal (EqualMetal(1200.0)), no lane, both sides attack-moving
(docs/harness/duels.md), on `Beyond All Reason test-32018-ad135b9`, engine 2026.09.03, Mithril Mountain v2.0.1.

- **tight**: ranks 56 apart, 2184 duels, batch `1795142939-duel-mu-nov-tight` (commit c4d13943): `target/release/duel --units armart,armfav,armflash,armflea,armham,armjanus,armjeth,armllt,armpw,armrock,armstump,armwar,corak,corcrash,corfav,corgator,corlevlr,corllt,cormist,corraid,corstorm,corthud,corwolv --reps 8 --spacing 56 --speed 200 --parallel 6 --map 'Mithril Mountain v2.0.1' --label mu-nov-tight --base-port 9800`
- **wide**: ranks 100 apart, 1092 duels, batch `1795143229-duel-mu-nov-wide` (commit c4d13943+): `target/release/duel --units armart,armfav,armflash,armflea,armham,armjanus,armjeth,armllt,armpw,armrock,armstump,armwar,corak,corcrash,corfav,corgator,corlevlr,corllt,cormist,corraid,corstorm,corthud,corwolv --reps 4 --spacing 100 --speed 200 --parallel 6 --map 'Mithril Mountain v2.0.1' --label mu-nov-wide --base-port 9800`

`*-duels.csv` carries the columns of the first set (2026-09-19); the batch directories under `run/matches` have the rest
(the fire and shape instruments, the per-second logs). Written by `run/matchup_table.py`, which also wrote
`crates/bot/data/matchups.csv` from the two `*-pairs.csv`.
