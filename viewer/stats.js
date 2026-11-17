// Within Reason match viewer: the game's numbers (docs/design/2026-11-16-viewer-stats.md). One model whatever the
// source: the sides (ally teams), each with its teams (players), and a series a metric, a value a game second, for a
// team or a side. A game with a tape has the tape's numbers, both sides (crates/tape/src/stats.rs, through the wasm
// reader); a game without one has what our record holds and, for the other side, the truth file's when there is one.
// What a source does not have is absent (null), never zero.
//
// Drawn here: the strip across the top (a line a side at the playhead, a click on a number showing its chart, a
// click on a side unfolding its players) and the Stats tab (the whole table at the playhead and each side's units by
// type). The charts are the timeline's (viewer/timeline.js). Loaded after app.js.
"use strict";

(() => {
  const { $, el, view, COLOR, seatColor } = WV;
  const FPS = WR.FPS;
  const NEVER = 0xffffffff;

  // ---------------------------------------------------------------- words for numbers

  /// A number in few characters: 1,678; 44.5k; 6.26M. Null is a dash.
  function short(v) {
    if (v == null || Number.isNaN(v)) return "–";
    const a = Math.abs(v);
    const trim = (text) => text.replace(/\.0+$/, "");
    if (a >= 1e6) return `${trim((v / 1e6).toFixed(a >= 1e7 ? 1 : 2))}M`;
    if (a >= 1e4) return `${trim((v / 1e3).toFixed(a >= 1e5 ? 0 : 1))}k`;
    if (a >= 100 || Number.isInteger(v)) return Math.round(v).toLocaleString("en-US");
    return v.toFixed(1);
  }

  /// A team's colour of the game ([r, g, b], 0 to 1) as one that reads on the dark ground: the darkest lifted.
  function legible(rgb) {
    if (!rgb) return COLOR["ink-2"];
    let [r, g, b] = rgb;
    const light = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if (light < 0.42) {
      const t = (0.42 - light) / (1 - light);
      [r, g, b] = [r + (1 - r) * t, g + (1 - g) * t, b + (1 - b) * t];
    }
    return `rgb(${[r, g, b].map((v) => Math.round(v * 255)).join(", ")})`;
  }

  // ---------------------------------------------------------------- the model

  let ids = 0;

  function sideName(teams) {
    const first = (teams[0].name || "").slice(0, 18);
    return teams.length > 1 ? `${first} +${teams.length - 1}` : first;
  }

  /// The tape's numbers: `tape.stats` (scene/tape.js) is `[team][metric][second]`.
  function fromTape(tape, match) {
    const s = tape.stats;
    // (A metric this game has no numbers for, a count its version does not keep, is absent: no row, no chart.)
    const absent = new Set(s.absent || []);
    const index = new Map(s.metrics.map((m, i) => [m.key, i]).filter(([key]) => !absent.has(key)));
    const heads = new Map((tape.head.teams || []).map((t) => [t.team, t]));
    const ourAlly = match.seats.length && match.header ? match.header.ally_team : null;
    const seatOf = new Map(match.seats.map((seat) => [seat.team, seat.index]));
    const teams = s.teams.map((t, i) => {
      const seat = seatOf.has(t.team) ? seatOf.get(t.team) : null;
      // A seat of ours keeps its colour of the page (its button, its units on the 2D map); the others the game's.
      return { at: i, team: t.team, ally: t.ally, name: seat != null ? `${match.seats[seat].name} ${t.name}`.trim() : t.name, css: seat != null ? seatColor(seat) : legible((heads.get(t.team) || {}).color), seat };
    });
    const sides = [...new Set(teams.map((t) => t.ally))].sort((a, b) => a - b).map((ally) => {
      const members = teams.filter((t) => t.ally === ally);
      const ours = ourAlly == null ? null : ally === ourAlly;
      return { ally, teams: members, ours, name: sideName(members.map((t) => ({ name: (heads.get(t.team) || {}).name || t.name }))), css: ours == null ? members[0].css : ours ? COLOR.ours : COLOR.theirs };
    });
    // Ours first.
    sides.sort((a, b) => (b.ours === true) - (a.ours === true));
    const stride = s.metrics.length * s.seconds;
    const of = (team, metric) => s.values.subarray(team * stride + metric * s.seconds, team * stride + (metric + 1) * s.seconds);
    const sums = new Map();
    const classOf = new Map((s.types || []).map((t) => [t.id, t.class]));
    return {
      id: ++ids, source: "tape", seconds: s.seconds, sides, metrics: s.metrics.filter((x) => !absent.has(x.key)), classes: s.classes || [], tape,
      /// The wind's strength a second (the game's, not a side's) and the range the map's wind blows in.
      wind: s.wind && s.wind.length ? s.wind : null, windRange: [tape.head.map.wind_min || 0, tape.head.map.wind_max || 0],
      metric: (key) => s.metrics[index.get(key)] || null,
      series(key, side, team = null) {
        const metric = index.get(key);
        if (metric == null || !sides[side]) return null;
        const members = sides[side].teams;
        if (team != null) return members[team] ? of(members[team].at, metric) : null;
        if (members.length === 1) return of(members[0].at, metric);
        const name = `${key}:${side}`;
        if (!sums.has(name)) {
          const sum = new Float32Array(s.seconds);
          for (const t of members) {
            const one = of(t.at, metric);
            for (let i = 0; i < sum.length; i++) sum[i] += one[i];
          }
          sums.set(name, sum);
        }
        return sums.get(name);
      },
      /// Each side's finished units at a frame, by type: [Map type id -> count], and the deaths near a frame.
      census(frame) {
        const ints = tape.unitInts, sideOf = new Map();
        sides.forEach((side, i) => side.teams.forEach((t) => sideOf.set(t.team, i)));
        const out = sides.map(() => new Map());
        for (let row = 0; row < tape.unitCount; row++) {
          const a = row * 8;
          if (ints[a + 4] === NEVER || ints[a + 4] > frame || ints[a + 5] <= frame) continue;
          const side = sideOf.get(ints[a + 2]);
          if (side != null) out[side].set(ints[a + 1], (out[side].get(ints[a + 1]) || 0) + 1);
        }
        return out.map((counts) => [...counts].map(([id, count]) => {
          const d = tape.unitDefs.get(id) || {};
          return { name: d.human || d.name || "unit", count, metal: d.metal || 0, cls: classOf.get(id) || "" };
        }));
      },
      /// What each side lost between two frames, as names: the units finished and then destroyed.
      lostBetween(from, to) {
        const ints = tape.unitInts, sideOf = new Map();
        sides.forEach((side, i) => side.teams.forEach((t) => sideOf.set(t.team, i)));
        const out = sides.map(() => []);
        for (let row = 0; row < tape.unitCount; row++) {
          const a = row * 8;
          if (ints[a + 5] < from || ints[a + 5] > to || ints[a + 4] === NEVER) continue;
          const side = sideOf.get(ints[a + 2]);
          if (side != null) out[side].push((tape.unitDefs.get(ints[a + 1]) || {}).human || "unit");
        }
        return out;
      },
    };
  }

  /// What our record has, under the tape's names for it.
  const RECORD_METRICS = [
    ["metal", "metal stored", "live", "level"], ["metal_store", "metal capacity", "live", "level"], ["metal_income", "metal income", "live", "rate"], ["metal_spend", "metal spend", "live", "rate"],
    ["energy", "energy stored", "live", "level"], ["energy_store", "energy capacity", "live", "level"], ["energy_income", "energy income", "live", "rate"], ["energy_spend", "energy spend", "live", "rate"],
    ["extractors", "extractors", "economy", "count"],
    ["killed_value", "metal worth killed", "combat", "total"], ["lost_value", "metal worth lost", "combat", "total"],
    ["value_army", "army worth", "composition", "value"], ["count_army", "army", "composition", "count"], ["count_builders", "builders", "composition", "count"],
  ].map(([key, label, group, kind]) => ({ key, label, group, kind }));
  const OF_SAMPLE = {
    metal: (s) => s.m[0], metal_income: (s) => s.m[1], metal_spend: (s) => s.m[2], metal_store: (s) => s.m[3],
    energy: (s) => s.e[0], energy_income: (s) => s.e[1], energy_spend: (s) => s.e[2], energy_store: (s) => s.e[3],
  };
  const OF_SERIES = { extractors: "extractors", value_army: "armyMetal", count_army: "army", count_builders: "builders" };
  const OF_TRUTH = { extractors: "extractors", value_army: "armyMetal", count_army: "army" };

  /// A game without a tape: ours from the record's samples, a seat a team; theirs from the truth file (extractors and
  /// army) when there is one, and what we killed of theirs from our own events.
  function fromRecord(match) {
    const seconds = Math.floor(match.lastFrame / FPS) + 1;
    const names = WR.sides(match);
    const multi = match.seats.length > 1;
    const sides = [
      { ally: match.header ? match.header.ally_team : 0, ours: true, name: `ours: ${names.oursWords || "us"}`, css: COLOR.ours, teams: match.seats.map((seat) => ({ at: seat.index, team: seat.team, name: seat.name, css: seatColor(seat.index), seat: seat.index })) },
      { ally: -1, ours: false, name: `theirs: ${names.theirsWords || "the other side"}`, css: COLOR.theirs, teams: [] },
    ];
    const index = new Map(RECORD_METRICS.map((m) => [m.key, m]));
    const cache = new Map();
    /// A value a second from rows in frame order: the last row at or before the second.
    const held = (rows, value, until) => {
      const out = new Float32Array(seconds);
      let i = -1;
      for (let s = 0; s < seconds; s++) {
        while (i + 1 < rows.length && rows[i + 1].f <= s * FPS) i++;
        out[s] = i >= 0 && s * FPS <= until ? value(rows[i], i) || 0 : 0;
      }
      return out;
    };
    /// A running total a second from events: each adds its worth at the second after it.
    const running = (events, worth) => {
      const out = new Float32Array(seconds);
      for (const e of events) {
        const s = Math.ceil(e.f / FPS);
        if (s < seconds) out[s] += worth(e);
      }
      for (let s = 1; s < seconds; s++) out[s] += out[s - 1];
      return out;
    };
    const metalOf = (defs, d) => (d >= 0 && defs[d] ? defs[d].metal || 0 : 0);
    const kills = () => {
      const seen = new Set(), out = [];
      for (const seat of match.seats) for (const e of seat.events) if (e.k === "enemy_destroyed" && !seen.has(e.u)) { seen.add(e.u); out.push({ f: e.f, worth: metalOf(seat.defs, e.d) }); }
      return out.sort((a, b) => a.f - b.f);
    };
    const seatSeries = (seat, key) => {
      const until = seat.lastFrame + 2 * FPS;
      if (OF_SAMPLE[key]) return held(seat.samples, OF_SAMPLE[key], until);
      if (OF_SERIES[key]) return held(seat.samples, (_, i) => seat.series[i][OF_SERIES[key]], until);
      if (key === "lost_value") return running(seat.events.filter((e) => e.k === "destroyed"), (e) => metalOf(seat.defs, e.d));
      return null;
    };
    const build = (key, side, team) => {
      if (!index.has(key)) return null;
      if (side === 0) {
        if (key === "killed_value") return team == null ? running(kills(), (e) => e.worth) : null;
        const seats = team != null ? [match.seats[team]].filter(Boolean) : match.seats;
        const each = seats.map((seat) => seatSeries(seat, key));
        if (!each.length || each.some((one) => !one)) return null;
        if (each.length === 1) return each[0];
        const sum = new Float32Array(seconds);
        for (const one of each) for (let i = 0; i < seconds; i++) sum[i] += one[i];
        return sum;
      }
      if (team != null) return null;
      if (key === "lost_value") return running(kills(), (e) => e.worth);
      if (OF_TRUTH[key] && match.truth.length) return held(match.truth, (row) => row[OF_TRUTH[key]], Infinity);
      return null;
    };
    return {
      id: ++ids, source: "record", seconds, sides, metrics: RECORD_METRICS, classes: [], multi, wind: null, windRange: null,
      metric: (key) => index.get(key) || null,
      series(key, side, team = null) {
        const name = `${key}:${side}:${team}`;
        if (!cache.has(name)) cache.set(name, build(key, side, team));
        return cache.get(name);
      },
      census(frame) {
        const tally = (rows) => {
          const counts = new Map();
          for (const r of rows) counts.set(r.name, { name: r.name, count: ((counts.get(r.name) || {}).count || 0) + 1, metal: r.metal, cls: r.cls });
          return [...counts.values()];
        };
        const ours = [];
        for (const seat of match.seats) {
          const i = WR.indexAt(seat.samples, frame);
          if (i < 0) continue;
          for (const u of seat.samples[i].own) {
            const d = u[1] >= 0 ? seat.defs[u[1]] : null;
            if (d && !(u[5] & WR.FLAG.beingBuilt)) ours.push({ name: d.name, metal: d.metal || 0, cls: d.class });
          }
        }
        const truth = match.truth[WR.indexAt(match.truth, frame)];
        const metal = match.metalByName || new Map();
        return [tally(ours), tally(truth ? truth.units.filter((u) => !u.building).map((u) => ({ name: u.name, metal: metal.get(u.name) || 0, cls: u.class })) : [])];
      },
      lostBetween(from, to) {
        const ours = [], theirs = [], seen = new Set();
        for (const seat of match.seats) {
          for (const e of WR.range(seat.events, from, to)) {
            const name = e.d >= 0 && seat.defs[e.d] ? seat.defs[e.d].name : "unidentified";
            if (e.k === "destroyed") ours.push(name);
            else if (e.k === "enemy_destroyed" && !seen.has(e.u)) { seen.add(e.u); theirs.push(name); }
          }
        }
        return [ours, theirs];
      },
    };
  }

  let cached = { key: null, model: null };

  /// The numbers of the match on the page: the tape's once its tape is open, the record's until then and without one.
  function model() {
    const match = view.match;
    if (!match) return null;
    const stats = view.scene && view.scene.tape && view.scene.tape.stats;
    if (!stats && !match.seats.length) return null;
    // (A growing tape's refresh hands in a new `stats`; a record's rows arriving raise the version.)
    const key = stats || `record:${view.version}`;
    if (cached.key !== key || cached.seats !== match.seats.length) cached = { key, seats: match.seats.length, model: stats ? fromTape(view.scene.tape, match) : fromRecord(match) };
    return cached.model;
  }

  const secondOf = (m, frame) => Math.min(m.seconds - 1, Math.max(0, Math.floor(frame / FPS)));

  /// A metric's value for a side (or one of its teams) at a frame; null where the source has none.
  function valueAt(m, key, side, team, frame) {
    const series = m.series(key, side, team);
    return series ? series[secondOf(m, frame)] : null;
  }

  /// The rows of a side as the page lists them: the side, then its players when it is unfolded.
  function rowsOf(m) {
    const rows = [];
    m.sides.forEach((side, i) => {
      rows.push({ side: i, team: null, name: side.name, css: side.css, folds: side.teams.length > 1, open: view.unfolded.has(i) });
      if (view.unfolded.has(i) && side.teams.length > 1) side.teams.forEach((t, j) => rows.push({ side: i, team: j, name: t.name, css: t.css }));
    });
    return rows;
  }

  function toggleSide(side) {
    if (view.unfolded.has(side)) view.unfolded.delete(side);
    else view.unfolded.add(side);
    WV.renderAll();
  }

  // ---------------------------------------------------------------- the strip

  /// The strip's columns. A cell is a metric; `of` the capacity beside what is stored, `count` how many units the
  /// worth is of, `sign` what stands before a flow. `ch` is the cell's width in digits, the longest its text gets
  /// ("99.9k / 99.9k", "1,234 · 123k" of an army, "123 · 45.6k" of the smaller classes: a number is five characters at most): every cell has its width from the start, so no column moves as the
  /// numbers change under the playhead.
  const STRIP = [
    { head: "metal", title: "metal stored of the capacity, then income and spend a second", cells: [{ key: "metal", of: "metal_store", ch: 13 }, { key: "metal_income", sign: "+", ch: 6 }, { key: "metal_spend", sign: "−", ch: 6 }] },
    { head: "energy", title: "energy stored of the capacity, then income and spend a second", cells: [{ key: "energy", of: "energy_store", ch: 13 }, { key: "energy_income", sign: "+", ch: 6 }, { key: "energy_spend", sign: "−", ch: 6 }] },
    { head: "build power", title: "the build power of every finished builder and factory, and the share of it in use", cells: [{ key: "build_power", share: "build_power_used", ch: 12 }] },
    { head: "army", title: "finished units that move on the ground and shoot: how many, and their metal", cells: [{ key: "value_army", count: "count_army", ch: 12 }] },
    { head: "air", title: "finished aircraft: how many, and their metal", cells: [{ key: "value_air", count: "count_air", ch: 10 }] },
    { head: "sea", title: "finished ships and submarines: how many, and their metal", some: true, cells: [{ key: "value_sea", count: "count_sea", ch: 10 }] },
    { head: "defence", title: "finished buildings that shoot: how many, and their metal", cells: [{ key: "value_defence", count: "count_defence", ch: 10 }] },
    { head: "strategic", title: "finished nuke silos, anti-nukes and long-range guns: how many, and their metal", some: true, cells: [{ key: "value_strategic", count: "count_strategic", ch: 10 }] },
    { head: "economy", title: "the metal of the finished extractors, generators, converters and stores", cells: [{ key: "value_economy", ch: 5 }] },
    { head: "extractors", title: "finished metal extractors", cells: [{ key: "extractors", ch: 3 }] },
    { head: "killed", title: "the metal of the other side's finished units this side destroyed, so far", cells: [{ key: "killed_value", ch: 5 }] },
    { head: "lost", title: "the metal of this side's finished units destroyed, so far", cells: [{ key: "lost_value", ch: 5 }] },
  ];

  /// Whether any side has anything of a metric at any second of the game: a column marked `some` (the sea, the
  /// strategic buildings) stands only in a game that has it, from the start, so the strip does not change mid-game.
  function ever(m, key) {
    return m.sides.some((_, i) => {
      const series = m.series(key, i);
      return !!series && series.some((v) => v > 0);
    });
  }

  let strip = { key: null, cells: [] };

  /// The wind and tide of the map: the wind's strength at a second where the tape has it, and the range it blows in.
  function mapWords(m, second) {
    const tape = view.scene && view.scene.tape;
    const map = (tape && tape.head.map) || (view.match.header && view.match.header.map) || {};
    if (map.wind_min == null) return "";
    const range = `${Math.round(map.wind_min)}–${Math.round(map.wind_max)}`;
    const wind = !(map.wind_max > 0) ? "no wind" : m.wind ? `wind ${m.wind[second].toFixed(1)} of ${range}` : `wind ${range}`;
    return `${wind}${map.tidal ? ` · tide ${Math.round(map.tidal)}` : ""}`;
  }

  function buildStrip(box, m) {
    box.textContent = "";
    const table = el("table");
    const head = el("tr");
    const who = el("th", "who", m.source === "tape" ? "" : "from our record: no tape of this game");
    if (m.source !== "tape") who.title = "A tape has both sides' numbers: run/tape.py <match dir> makes one from the game's demo";
    head.append(who);
    const columns = STRIP.map((column) => ({ ...column, cells: column.cells.filter((cell) => m.metric(cell.key) && (!column.some || ever(m, cell.key))) })).filter((column) => column.cells.length);
    for (const column of columns) {
      const th = el("th", "group", column.head);
      th.colSpan = column.cells.length;
      th.title = column.title;
      head.append(th);
    }
    const wind = el("th", "wind");
    wind.title = m.wind ? "the wind's strength now, of the range the map's wind blows in: click for its chart" : "the range the map's wind blows in (its strength through the game is on a tape)";
    if (m.wind) wind.addEventListener("click", () => WV.timeline.toggleChart("wind"));
    head.append(wind);
    table.append(head);
    const cells = [];
    cells.wind = wind;
    for (const row of rowsOf(m)) {
      const tr = el("tr", row.team == null ? "side" : "team");
      const name = el("td", "who");
      const swatch = el("i", "swatch");
      swatch.style.background = row.css;
      name.append(swatch, row.folds ? `${row.open ? "▾" : "▸"} ${row.name}` : row.name);
      if (row.folds) {
        name.classList.add("folds");
        name.title = row.open ? "fold its players away" : "a line a player, here and in every chart";
        name.addEventListener("click", () => toggleSide(row.side));
      }
      tr.append(name);
      for (const column of columns) {
        column.cells.forEach((cell, i) => {
          const td = el("td", `num${i === 0 ? " first" : ""}${cell.of ? " store" : ""}`);
          td.title = `${m.metric(cell.key).label}: click for its chart`;
          // (A digit and a half over the longest text: a letter of the font may be wider than its digits.)
          td.style.minWidth = `calc(${cell.ch + 1.5}ch + 14px)`;
          td.addEventListener("click", () => WV.timeline.toggleChart(cell.key));
          const text = el("span");
          td.append(text);
          let fill = null;
          if (cell.of) {
            const bar = el("i", `bar ${cell.key}`);
            fill = el("b");
            bar.append(fill);
            td.append(bar);
          }
          tr.append(td);
          cells.push({ td, text, fill, cell, row });
        });
      }
      tr.append(el("td"));
      table.append(tr);
    }
    box.append(table);
    return cells;
  }

  function renderStrip() {
    const box = $("strip"), m = model();
    box.hidden = !m;
    if (!m) return;
    const structure = `${m.id}:${[...view.unfolded].join(",")}`;
    if (strip.key !== structure) strip = { key: structure, cells: buildStrip(box, m), at: null };
    const charted = WV.timeline.charted();
    const at = `${secondOf(m, view.frame)}:${[...charted].join(",")}`;
    if (strip.at === at) return;
    strip.at = at;
    strip.cells.wind.textContent = mapWords(m, secondOf(m, view.frame));
    strip.cells.wind.classList.toggle("charted", charted.has("wind"));
    for (const { td, text, fill, cell, row } of strip.cells) {
      const v = valueAt(m, cell.key, row.side, row.team, view.frame);
      td.classList.toggle("charted", charted.has(cell.key));
      td.classList.toggle("absent", v == null);
      if (v == null) {
        text.textContent = "–";
        if (fill) fill.style.width = "0";
        continue;
      }
      if (cell.of) {
        const cap = valueAt(m, cell.of, row.side, row.team, view.frame);
        text.textContent = cap ? `${short(v)} / ${short(cap)}` : short(v);
        fill.style.width = `${cap > 0 ? Math.min(100, (100 * v) / cap) : 0}%`;
      } else if (cell.count) {
        const n = valueAt(m, cell.count, row.side, row.team, view.frame);
        text.textContent = n != null ? `${short(n)} · ${short(v)}` : short(v);
      } else if (cell.share) {
        const part = valueAt(m, cell.share, row.side, row.team, view.frame);
        text.textContent = part != null && v > 0 ? `${short(v)} · ${Math.round((100 * part) / v)}%` : short(v);
      } else {
        text.textContent = `${v ? cell.sign || "" : ""}${short(v)}`;
      }
    }
  }

  // ---------------------------------------------------------------- the Stats tab

  /// The tab's views, after the game's own statistics window, and the metric groups each shows.
  const VIEWS = [["live", "Live", ["live"]], ["economy", "Economy", ["economy"]], ["combat", "Combat", ["combat"]], ["composition", "Composition", ["units", "composition"]], ["activity", "Activity", ["activity"]], ["types", "Unit types", []]];
  let tab = { key: null, cells: [] };
  let shown = "live";
  try { shown = localStorage.getItem("wr-stats-view") || "live"; } catch (_) { /* no storage */ }

  function buildTab(box, m) {
    box.textContent = "";
    const nav = el("div", "views");
    for (const [name, label] of VIEWS) {
      const b = el("button", shown === name ? "active" : null, label);
      b.addEventListener("click", () => {
        shown = name;
        try { localStorage.setItem("wr-stats-view", name); } catch (_) { /* no storage */ }
        tab.key = null;
        renderTab();
      });
      nav.append(b);
    }
    box.append(nav);
    if (m.source !== "tape") box.append(el("div", "meta", "From our record: this game has no tape. A tape has both sides' numbers; run/tape.py <match dir> makes one from the game's demo."));
    if (shown === "types") {
      box.append(el("div", "types"));
      return [];
    }
    const groups = VIEWS.find((v) => v[0] === shown)[2];
    const rows = rowsOf(m);
    const table = el("table", "grid stats");
    const head = el("tr");
    head.append(el("th", null, ""));
    for (const row of rows) {
      const th = el("th", row.team == null ? "side" : "team", row.team == null && row.folds ? `${row.open ? "▾" : "▸"} ${row.name}` : row.name);
      th.style.color = row.css;
      if (row.team == null && row.folds) {
        th.classList.add("folds");
        th.addEventListener("click", () => toggleSide(row.side));
      }
      head.append(th);
    }
    table.append(head);
    const cells = [];
    for (const metric of m.metrics.filter((x) => groups.includes(x.group))) {
      const tr = el("tr");
      const label = el("td", "name", metric.label);
      label.title = "click for its chart";
      tr.addEventListener("click", () => WV.timeline.toggleChart(metric.key));
      tr.append(label);
      for (const row of rows) {
        const td = el("td", row.team == null ? "side" : "team");
        tr.append(td);
        cells.push({ td, tr, key: metric.key, row });
      }
      table.append(tr);
    }
    if (!table.rows[1]) box.append(el("div", "meta", "This game's source has none of these numbers."));
    box.append(table, el("div", "meta", m.source === "tape" ? "From the tape. A class is the game's own (its statistics window's); worth is the metal cost of the finished units; an idle constructor has no order, an idle plant an empty line." : ""));
    return cells;
  }

  /// Each side's units by type at the playhead, the costliest holdings first.
  function renderTypes(box, m) {
    box.textContent = "";
    m.census(Math.floor(view.frame)).forEach((types, i) => {
      const side = m.sides[i];
      const column = el("div", "side-types");
      const total = types.reduce((sum, t) => sum + t.count * t.metal, 0);
      const title = el("h2", null, `${side.name} `);
      title.style.color = side.css;
      title.append(el("span", "muted", types.length ? `${types.reduce((n, t) => n + t.count, 0)} units, ${short(total)} metal` : m.source === "tape" ? "none" : "not known: no truth file"));
      column.append(title);
      const list = el("table", "grid");
      for (const t of types.sort((a, b) => b.count * b.metal - a.count * a.metal)) {
        const tr = el("tr");
        tr.append(el("td", "name", t.name), el("td", null, `×${t.count}`), el("td", null, short(t.count * t.metal)), el("td", "cls", t.cls));
        list.append(tr);
      }
      column.append(list);
      box.append(column);
    });
  }

  function renderTab() {
    const box = $("stats"), m = model();
    if (!m) return void (box.textContent = "");
    const structure = `${m.id}:${shown}:${[...view.unfolded].join(",")}`;
    if (tab.key !== structure) tab = { key: structure, cells: buildTab(box, m), at: null };
    const charted = WV.timeline.charted();
    const at = `${secondOf(m, view.frame)}:${[...charted].join(",")}`;
    if (tab.at === at) return;
    tab.at = at;
    if (shown === "types") return renderTypes(box.querySelector(".types"), m);
    for (const { td, tr, key, row } of tab.cells) {
      const v = valueAt(m, key, row.side, row.team, view.frame);
      td.textContent = short(v);
      td.classList.toggle("absent", v == null);
      tr.classList.toggle("charted", charted.has(key));
    }
  }

  WV.stats = { model, valueAt, secondOf, short, renderStrip, renderTab };
})();
