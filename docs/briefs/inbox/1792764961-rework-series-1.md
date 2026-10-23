# rework-series-1: the player's debrief

Match `run/matches/1792764961-rework-series-1/02`, commit f9602b1+, BARb hard_aggressive on Comet Catcher Remake 1.8: Win after 15.2 min, 65 turns. Debriefed by claude-opus-5 (effort low) in 42 s; since snapshot: 7-day +0.0 points, extra usage +0.00.

# Debrief — Comet Catcher Remake 1.8, Cortex, west box, vs BARb hard_aggressive

**Result: win at 15:12, 65 turns.** The game was decided by economy before 11:00 and finished by one 45-soldier body walking into his corner. Nothing clever happened. What cost time was the hands, not the opponent.

## What the map and seat gave us

- Our commander started at **B8**, the south-west corner of the west box. His base is the **north-east, G1–H2** (extractors G1/H2, solars G1/H1). That diagonal is the whole game: a straight line of metal spots runs from our corner up the west edge and then east across the middle.
- The west-edge spots **68 (A8), 62 (A7), 54 (A6), 69 (C8)** are the safe early claim; **50/45 (B5–B6), 52 (C5), 47 (D5)** are the contested ones and the ones his lone raiders will hit.
- A good route for the army, used and worth reusing: **spot_44 → 37 → 23 → 18** (E3, F3), then into his corner through **16, 11 (F2), 6, 10, 5 (G1), 12, 17, 25 (H2/H3)**.

## The opening that worked

Extractor, extractor, solar, solar, vehicle plant, third extractor, then commander helps the plant at low priority with one more solar between. Plant: constructor, 2 Rascals, constructor, Incisors with a constructor after every three. Plant stood at ~1:00. At 3:00 we held 7 extractors and 16 metal/s (strong-player median 6 and 14); at 4:00, 8 extractors and 22/s against their 18, army 1,012 vs 510, nothing lost. By 10:27 it was 33 extractors and 77/s. **Energy, not metal, is the early constraint** — the plant on Incisors with two helpers wants ~260 e/s against ~110 coming in, and we stalled at 2:20. Put the commander on two solars at full priority the moment that happens, or pre-empt it with one more solar in the opening.

**Send the two Rascals into his half immediately.** Roving E1–H8 and killing unguarded extractors and constructors, they killed three of his extractors (F1, F2, G1) by 3:40 and lost nothing. Highest-value metal we spent all game.

## The mistake to not repeat: the lone raider

BARb hard_aggressive sends **single Incisors** into our west between 4:00 and 6:00. They killed a light turret at A6, a half-built extractor at B6, two turret frames, and the extractor at B5 — all because:

1. The first three Incisors out of the plant **joined the roving group instead of staying home**, so nothing guarded. Name the home guard explicitly in the first packet and say that the first three Incisors are its, not the rovers'.
2. The home guard spent two minutes **following a constructor** that had wandered back to help the plant. A "guard the constructor" habit silently parks your whole guard at home. Write the guard as answering *raiders at our extractors and constructors*, in our half only, not as escorting anybody.
3. Our Incisors and his run at the **same speed**, so a chase never catches. Four of ours went after one of his in two pairs and caught nothing. **One pair, at most, and only to push it off a site** — and never let the guard follow a raider into his base (ours nearly walked 6,500 to F1).

Light turret beside each outer extractor is right, but a turret *frame* is free metal for him; build the extractor and the turret with a builder that stays until the turret stands.

## Tier 2 and the energy cliff

At 10:27, with energy banking (1982) and metal 1739, I went advanced vehicle plant at C8 with commander + 3 constructors — ahead of hard_aggressive's own T2 (~11:00). Right call. But **585 build power draws ~740 e/s**: we went to −150/s at 11:55 and were stalling at −300/s by 12:51 with 686 banked. Queue **8–10 plain solars and an energy store at the same time you start the AVP**, not after. I had to pull three constructors off the assist to fix it, which slowed the thing I was paying for.

AVP output order that served: 2 advanced constructors (mohos), 4 Tigers, another advanced constructor, then Tigers into the main army.

## The yard jam — plan for it

Builders boxed the AVP in with solars and a Guard at the yard: commander and three constructors couldn't move, a finished Tiger couldn't leave, and I spent four turns (13:57–14:52) reclaiming solars and stale frames to open a lane. **Keep a clear exit lane in front of the advanced plant and don't let builders site solars or defences in it.** Stale frames in the yard are the same problem; reclaim them early, not at 14:40.

## The army

One body, named route, closes as one. 38 soldiers / 8.1k at 13:26, 47 / 11k at 14:52, trading at roughly 4:1 or better the whole way in. Rules that earned their place in the packet:

- **Raids are the home guard's, never the main body's.** At 11:37 the push I had just ordered got pulled onto a *single* Incisor at C5. Write it as: the main group never turns back for raiders and never chases them; at most two soldiers after any raider party, never four or eight. Cap its detachments at two and have splits rejoin.
- **Commander is the first target** once his base is reached, ahead of buildings.
- **Kill the factories, then say so.** His AVP at G2 died 14:18; his vehicle plant at G1 (6912, 320) was only *found* at 14:32, beside a heavy laser tower at (6723, 264) that had been shelling us from out of sight. Expect two factories in that corner and sweep for the second before declaring the base dead.

### The failure that nearly cost the clean finish

Three separate times the body **oscillated between two waypoints** or kept attacking an already-dead building instead of taking the new order: between spot_18 and spot_16 for several minutes before 13:57, at the dead AVP site at 14:32, and worst at 15:02 — it shuttled between the dead plant sites at G1 and G2, still trying to hit the dead vehicle plant, while two Twin Guard towers wore it down for ~1,400 metal in a minute. **When a target dies, do not assume the body re-targets.** Give it a fresh named place (a position mark, not a unit) the same turn, and check the next report that its front actually moved; if the front is in the same cell two turns running, re-issue against a ground position rather than a target.

## If you sit here again

Run the same opening; it is comfortably ahead of strong-player pace on this map. Spend the first five minutes getting the home-guard wording right instead of reacting to each raider — that is where my turns went. Start the AVP around 10:30 *with its solars*, keep its yard open, and from 11:30 walk the 44→37→23→18 route into G1–H2 as one body, carrying the knowledge that there are two factories, a heavy laser tower out of sight, and Twin Guards around his commander. A tighter version of this game ends before 13:00.
