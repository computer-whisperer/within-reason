# Would a roving group with a menu be pulled off its rove? (2026-10-08)

The rove rework (after human-27 1:43-2:34, the first Rascals' 50 s at spot_18): roving becomes a group task like
hold, move, engage, hunt and follow; `scout` becomes `rove_1` in the detachment ladder's shape; a roving group
keeps a menu so the way back is a pick, not the player's `lane` setting; and it is asked only on a packet or list
change and at the re-ask interval, never on its own sightings. The risk the user named: a packet that says "roves;
being seen is its job" has to keep the group roving, or Jev pulls scouts home (player-9-posing, the reason the
scout had no menu).

## Method
`run/jev_roving_menu_ab.py`. A roving group had no menu in the recorded games, so none was asked. For gates where a
group was roving, the tool adds the menu the rework would give it, in the hands' question shapes (`change`,
`hold`, `go_home`, a walk to the extractor nearest home, a join of the largest other group, an attack on the party
in its entry when one is there), with the group's entry reworded as the rework words it ("roving: its Rascal is on
its way to look at spot_62 ..." without "in code, beyond your hands' reach"), and asks Jev under four arms:

- `kept`: the recorded packet, the reworded entry (what the game would see).
- `recorded`: the recorded packet and the recorded entry words.
- `takeback`: the group's paragraph replaced by "its roving is over now: it walks to home and holds there as the
  picket; any raider at our home extractors it attacks".
- `silent`: the group's paragraph removed.

48 gates: 16 each from human-27 (group_A, 2:35-13:44, packet "roving; leave it to rove"), human-28 (group_B, a
lone Rover, "roves") and rove-move-1 (group_B and group_C, "Rovers in group_B and group_C: they rove"), spread over
each span. Data: `docs/studies/data/jev-roving-menu.jsonl`. Cost $0.04.

## Result
| arm | change median | change >= 0.5 | hold | go_home | go_spot | join | attack | any move >= 0.5 |
|---|---|---|---|---|---|---|---|---|
| kept | 0.31 | 3 of 48 | 0.16 | 0.20 | 0.12 | 0.20 | 0.13 | 0 of 48 |
| recorded | 0.22 | 1 of 48 | 0.10 | 0.10 | 0.08 | 0.12 | 0.11 | 0 of 48 |
| takeback | 0.84 | 48 of 48 | 0.23 | 0.75 | 0.42 | 0.29 | 0.14 | 48 of 48 |
| silent | 0.45 | 19 of 48 | 0.23 | 0.27 | 0.23 | 0.25 | 0.16 | 0 of 48 |

(medians of the move nouls; per game, kept 0 of 16 pulled in each, takeback 16 of 16 in each.)

- Under the packets as written no move came within 0.07 of the bar at any gate (best 0.43, a join at human-27
  2:35, the second the rove began and two of the group had just left as a hunt). The reworded entry rates a little
  higher than the recorded "beyond your hands' reach" words and still clears nothing.
- Under a take-back paragraph the walk home is over the bar at every gate (0.58 to 0.83) and `change` at 0.84: the
  way back as a pick works at the first ask.
- A roving group nobody mentions sits at 0.45 on `change` with no move over the bar: Jev leaves it alone, barely.
  The brief should keep telling the player to name its roving groups.

## What it settles
The rework's third point is safe to build: a roving group can carry a menu without Jev pulling it off its rove,
and the player's words bring it back. Not measured: the pick stage (the gate alone was replayed), the cost of the
extra questions (bounded by asking only on packet and list changes and the re-ask interval), and party moves on a
roving group's menu (left off it: its sightings stay the lane's business).
