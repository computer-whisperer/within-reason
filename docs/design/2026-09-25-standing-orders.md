# Standing orders: the executor, the `standing` tool, and the variants (target design, 2026-09-25)

**Status (2026-09-25 evening).** Built (770bebd) and played twice (standing-1, standing-2: both lost, the ledger has the
reading). The `family` arm and the Lua policy are gone (`2026-09-25-one-decider.md`); the executor stays as the candidate
generator of the worlds question, which becomes the groups' one decider. The rules and the tool below stand.

The user (2026-09-25): "Build the executor and the standing tool. I now have several close favorite concepts out of
what we discussed, and we should probably try multiple variations on actual games ... something that uses jev to
decompress opus's prose, then applies a post filter to 'what should we do: a: the literal rule result, b: something
slightly different, c: something very different, d: panic' ... We may also try the family breakdown ... a flexible
system for us to try stuff in." Written before the code. The offline test behind it: `docs/studies/2026-09-25-decompression.md`
(K-jev-a-packet-decompresses-to-standing-orders); the outline it belongs to: `docs/design/2026-09-25-menus-from-scratch.md` §9d.

## 1. What a standing order is

A per-actor rule with a parameter, held in the hands and evaluated in code every second against the picture, giving
that actor's answer without a Jev ask when it fires. Orders are a map `actor -> rule -> value`, the same shape the
`standing` tool takes and the decompression returns, so one executor serves both.

Group rules (`group_X`):
- `station` = place: when the group holds and stands more than 300 from the place, it walks there (`move_to`);
  `station_mode` = `advance` makes that `fight_to`.
- `raiders_lone` and `raiders_party` = `whole_group` | `detachment:N` | `detachment` (N from the odds) | `ignore`: an
  enemy party (one unit for lone, two to six for party) at one of our structures within 1,200 of the group that no
  group of ours engages yet is answered by `engage` (whole group, when the odds are not "it outweighs us") or
  `send_against` with `how_many` N (when the menu offers it: four or more soldiers, the party three or fewer; else
  `engage`). A party the group already engages is `continue`.
- `no_chase` = yes: a group engaging a party that is now more than 1,200 from its station (or its last hold, or
  home) holds.
- `no_detachments` = yes: `send_against`, `split` and `scout` are taken off its menu (a removal, H-HANDS-FORBID).
- `hold_line` = yes ("never_while_even_or_better"): `retreat` and `fall_back` are taken off its menu while the
  nearest party's odds are not "it outweighs us".
- `fall_back_to` = place: when the odds against say the nearest party within 600 outweighs the group, or it has lost
  a quarter of its metal in 30 s, it walks there (`move_to`), else `fall_back`/`retreat` as the menu has them.
- `engage_party` = party name: while that party is in the picture, `engage` it (whole group).
- `never` = places (space-separated): those places are taken off its `where` candidates and no standing order sends
  it there.

Builder rules (`commander`, `constructors` for every constructor, `constructor_N` for one):
- `job` = `help_factory` | `expand` | `follow_list`: an idle builder with no list helps the nearest factory, or
  takes the nearest free spot (`extractor`, ground held by us first, never-places excluded); `follow_list` does
  nothing (lists play themselves).
- `attack_raiders` = yes: when the menu offers `attack` (a party within reach it outweighs, H-HANDS-COMMANDER-FIGHTS),
  it attacks.
- `retreat_when_enemy_near` = yes: a party within 600 that it does not outweigh sends it home (`retreat_home`, or
  `walk_to home`).
- `solar` = `only_when_stalling` | `never` | `freely`: `only_when_stalling` orders a solar when the energy line reads
  STALLING and the builder is idle; `never` takes the generators off its menu.
- `turrets` = `beside_each_outer_extractor` | `beside_each_extractor` | `none`: an idle builder with an extractor of
  ours within 1,200 (beyond 500 from home for `outer`) that has no turret within 200 builds the light turret at that
  extractor's place; `none` takes turrets off its menu.
- `no_chase` = yes: `attack` is not offered against a party farther than 320 (the busy-party extension is off).
- `never` = places: off its `where` candidates and its free spots.

Rules the vocabulary does not have yet, found in the packets: a boundary ("never east of spot_38"), a conditional
("never past spot_39 without soldiers near"), scout routes. They stay prose for the menus.

## 2. Where the orders come from

- **Decompression.** When the packet changes (the existing `packet_changed` point in `run_pianist`), the packet goes
  to Jev in its own request, state `{packet}`, one question per (actor, rule) for every group named in the packet and
  for the commander and the constructors (the questions of `run/decompress.py`, with the fixes the test found: the
  commander's no-chase asked in the packet's idiom; raiders asked for a lone raider and for a small party apart). The
  answers become the packet's orders (source `packet`, the packet's frame). In lockstep the request is made in place
  (one call, 130-250 ms); in realtime it goes through the worker as a tagged request and lands when it comes. Hedged
  answers (p_top under 0.6) set nothing.
- **The `standing` tool.** `{"set": {"group_B": {"station": "spot_61", "raiders_lone": "detachment:2", ...}}}` sets
  orders with source `tool`; `{"clear": ["group_B"]}` or `{"clear": "all"}` removes tool orders; no arguments shows
  what is in force and what fired since the last turn. A tool order outranks a packet order for the same
  (actor, rule) until cleared. Values are validated against the vocabulary and the picture's places.
- The picture shows each actor's orders in force on its entry (`standing`), so the residual Jev asks and the player
  both see them; the report says what fired since the last turn and how many asks it saved.

## 3. The executor and the variants

`standing_pass` runs where `policy_pass` runs, before it, on the menus of the second: for each actor with orders it
computes at most one order (the rules in the priority above: fall-back and retreat first, then raiders, no-chase,
attack, station, job, solar, turrets) and applies it exactly as a policy order is applied (the menu taken, the answer
at probability one, `same_as_current` as `continue`, the play through `play`); the source in the log and the record
is `standing`. Menu removals (`no_detachments`, `hold_line`, `never`, `solar=never`, `turrets=none`) are made when the
menu is built. A group with orders and nothing in sight, not hit and not on a new packet is not asked at all: the
review period of a quiet standing actor is the diet's quiet review, so the asks those actors used to make are the
saving (family-1: 1,529 group plays, 802 of them re-asks of busy groups).

`WITHIN_REASON_STANDING` (arena `--standing`): `off` (the executor never runs; the tool and the decompression still
fill the orders, so the log shows what would have fired), `on` (the executor takes the actors it decides for),
`filter` (the user's post filter: an actor whose order fired is still asked, with one more question beside its
menu: "The standing order for group_B says: send 2 Blitzes against party_15. What should happen: `rule`: exactly
that; `near`: the same kind of action, a different target, size or place (its own answer is played); `other`:
something else entirely (its own answer is played); `panic`: fall back and wake the player"; the order plays on
`rule`, the actor's own composed answer on `near` and `other`, `retreat` plus a wake on `panic`; every call logs
which). `WITHIN_REASON_FAMILY` (arena `--family`): `on` (H-HANDS-TWO-LEVEL as now) or `off` (the flat `do`). The
two are independent, so the arms are standing off/on/filter by family on/off.

The `standing` log line each second an order fires: `{"t": "standing", "f", "orders": {actor: {do, params, rule}},
"removed": {actor: [options]}, "skipped": [actors not asked]}`; the record's `played` entries carry source
`standing`. The scorecard (`run/floor.py`) gains `standing%` (plays by the executor over all plays) and the Jev bill
per game from the log's usage.

## 4. What is deleted, what is kept

Nothing of the menus is deleted in this step: the user wants the variants side by side in games. The policy runtime
stays (the executor reuses its application path, refactored into `apply_orders`). `run/decompress.py` keeps its own
copy of the questions for offline runs; the Rust questions are the ones in force and the script is checked against
them by a test that renders both for one packet.

## 5. Validation

- Unit tests: the extraction questions for a packet (actors found, places found, the ids); the executor's rules on
  synthetic pictures (a raider at an extractor with each `raiders_*` value; a station walk; the removals).
- One check game per arm on Comet Catcher against hard_aggressive with the player: `standing on, family on` first,
  read by `run/floor.py` (standing%, the Jev bill), `run/jev_ab.py --list` (the detectors' rates), `run/fire.py`, and
  the player's notes; then `filter`, then `off` as the control on the same brief. The user's review agents score the
  games' relative quality after.
- Success line for `on`: raider_ignored under 10% of group asks, a busy group's course kept above 50%, the Jev bill
  under half of family-1's $0.59, the game not obviously worse than bank-1.
