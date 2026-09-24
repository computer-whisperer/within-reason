# Target design: the viewer overhaul — the game's icons, zoom, and what each unit is doing

Written 2026-09-24 at the user's direction, before the code (the user: "I have a hard time inspecting building and
unit placements in the current inspection ui. Let's give that an overhaul first, including unit icons from the
official game ... I also need a way to zoom in on the map and see what each unit is doing."). The aim behind it: to
read Jev's decisions in the opening moments of a game (placements, lists, the plant's first units).

## What is true today (viewer/, `run/view_match.py`, `docs/harness/record-format.md`)
- The map is one canvas fitted to the pane, no zoom, no pan; every unit is a class glyph (circle, square, diamond)
  in a side colour; hover names the nearest unit within 10 px; nothing can be selected.
- The record has per unit only id, type, position, health and flags each second (`s.own`), the commands the bot sent
  (`cmd`: build/move/fight/guard/repair/reclaim/attack/stop with their targets), the events (`created` with its
  builder, `finished` with the facing, `destroyed`), and the header's unit table with `footprint` (engine squares of
  8 elmos) and `death_blast`. The Jev log's calls carry each actor's picture entry (`doing`, `list`, `at`), the
  groups with members and tasks, the played answers.
- BAR's minimap icon standard: `gamedata/icontypes.lua` (one entry per unit name: `bitmap` under `icons/`, a `size`
  0 to 5, median 1.47), assigned in `alldefs_post.lua` (`icontype = name` unless the unit file says otherwise); the
  bitmaps are 128 x 128 RGBA, white shapes the game tints with the team colour. Licence CC BY-NC-ND (IceXuick,
  Floris, PtaQ): verbatim redistribution with attribution is allowed, derivatives are not, so the files are copied
  unmodified and tinted only on the canvas at draw time. Every one of our 377 unit-table entries has an icon.

## The design
1. **Icons** (`run/icons.py`, `viewer/icons/`, `viewer/icons.json`): the script copies the bitmaps the unit table
   names from `upstream/Beyond-All-Reason/icons/` unmodified into `viewer/icons/` with a LICENSE.txt naming the
   authors and the licence, and writes `icons.json` {unit name: {file, size}}; both committed (about 3 MB), so a
   checkout without `upstream/` still has them. The viewer draws a unit as its icon tinted by side (ours, ally,
   enemy; being built at 40%; the commander's squad and the attacker wave keep their colours), from an offscreen
   cache per (icon, colour); the drawn size is the icon's `size` times a base that grows with the zoom, clamped so an
   icon never exceeds its footprint on the ground at high zoom. A unit without an icon keeps the class glyph.
2. **Zoom and pan**: a view transform (scale, offset in elmos) over the map; the wheel zooms about the cursor (1x to
   32x the fitted scale), drag pans, double-click and the `0` key reset, `+`/`-` step; the transform survives seeking
   and live refresh. Grid lines, spot rings, places, parties and group lines all go through the same transform. At
   4x and above, buildings draw their footprint rectangle (`footprint` x 8 elmos, oriented by the `finished`
   facing) under the icon, factories with their exit side marked, and unit names appear under icons; at 8x, each
   unit's id too.
3. **What each unit is doing**: clicking a unit selects it (a ring; `Escape` clears). The side pane gains a
   **Unit** tab: type and gloss, health, flags, position and grid cell; its standing order (the last command the bot
   sent it, with the clock, and the target drawn on the map: a line to the point or unit, a rectangle at a build
   site); its role in the pianist's picture at the playhead (the builder's `doing` and `list` entry, or the group it
   belongs to with the group's task) and its last decisions; its timeline of events (created by whom, finished,
   damage taken, destroyed by what). The map's `orders` layer draws every unit's standing order at 4x and above (a
   stop, an idle flag or a newer command ends it), and only the last 10 s of orders below that as today.
   `record.js` gains `orderAt(match, unit, frame)` and `unitHistory(match, unit)` for this; the tests cover both.
4. **The opening, inspectable**: the Unit tab's list entry and the Pianist tab's actor entries show the list with
   its done steps struck through (from the narration's "skipped" and "from its list" lines), and a **Build order**
   strip in the Pianist tab lists every `created` event of the first eight minutes with the clock, the builder,
   the type and the place, click to seek.

## Tests and docs
`viewer/test/smoke.js` covers the model additions; `viewer/test/browser.js` zooms with the wheel, pans, clicks a unit
and reads the Unit tab, at the same headless Chromium; `docs/harness/record-format.md`'s viewer section says what
changed. No record or protocol change: everything is drawn from what the record already holds.

## Status (2026-09-24, built the same day)
All four parts stand, with two departures from the text above: the list's done steps are not struck through (the
picture's `list` entry is the remaining steps, which says the same), and the build-order strip lists what was begun
rather than the planned steps. Icons are tinted by multiplying (the building icons are opaque squares whose shape
is in the shading; a source-in tint made them solid). Checked in headless Chromium (`viewer/test/browser.js`: wheel
zoom to 11x with the unit still under the cursor, a click opening the Unit tab with 5 facts and 80 history rows,
Escape, a double-click reset, 85 build-order rows, 377 icons) on 2v1b-hard, and by eye at 20x on its base at 1:35.
The DevTools protocol truncates event coordinates to whole pixels, so the test anchors its cursor on whole pixels.

