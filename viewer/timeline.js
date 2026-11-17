// Within Reason match viewer: the timeline under the map (docs/design/2026-11-16-viewer-stats.md). Rows chosen one at
// a time and remembered: lanes (the metal destroyed, the commander's and the players' turns, the hands' gates and
// plays) and charts (any metric of the game's numbers, viewer/stats.js: both sides as lines, a side unfolded in the
// strip as its players' lines; the worth of each side's classes as bands; the hands' bill a minute). A row is a
// label, a picture drawn once (again when the numbers, the choice or the width change) and the values at the
// playhead beside it; the playhead and the hover are two lines laid over the rows. Loaded after stats.js.
"use strict";

(() => {
  const { $, el, view, COLOR, seatColor, playerOf, seek, summarise } = WV;
  const FPS = WR.FPS;
  const LANE_H = 12, TRADE_H = 30, CHART_H = 84, AXIS_H = 18;
  /// Seconds a bar of the metal destroyed covers.
  const TRADE_SECONDS = 10;
  /// The classes' bands: softer than any side's colour, so a band is not read as a side.
  const CLASS_COLORS = { army: "#d98f7a", air: "#adcce6", defence: "#b8b8b2", factories: "#dbcc8a", builders: "#9ecc9e", economy: "#85c7c2", utility: "#d1adc2" };

  // ---------------------------------------------------------------- what is shown

  /// The rows chosen. A chart is a metric's key, `<key>/min` for a running total as what it gained in the last
  /// minute, `classes` (each side's worth by class) or `jev_cost`.
  const DEFAULT = { charts: ["value_army", "metal_income"], lanes: { trade: true, commander: true, player: true, gates: false, builders: false, labs: false, groups: false } };
  let prefs = JSON.parse(JSON.stringify(DEFAULT));
  try {
    const saved = JSON.parse(localStorage.getItem("wr-rows") || "null");
    if (saved && Array.isArray(saved.charts)) prefs = { charts: saved.charts.filter((c) => typeof c === "string"), lanes: { ...DEFAULT.lanes, ...(saved.lanes || {}) } };
  } catch (_) { /* no storage, or not ours */ }

  function changed() {
    try { localStorage.setItem("wr-rows", JSON.stringify(prefs)); } catch (_) { /* no storage */ }
    WV.renderAll();
  }

  const metricOf = (chart) => chart.replace(/\/min$/, "");
  /// The metrics with a chart shown (the strip and the Stats tab mark them).
  const charted = () => new Set(prefs.charts.map(metricOf));

  function toggleChart(key) {
    const has = prefs.charts.some((c) => metricOf(c) === key);
    prefs.charts = has ? prefs.charts.filter((c) => metricOf(c) !== key) : [...prefs.charts, key];
    // A chart asked for while the charts are folded away unfolds them.
    if (!has && document.body.dataset.compact === "1") $("compact").click();
    changed();
  }

  /// Whether a seat in scope has a hands' log, read yet or not.
  function expectsHands() {
    if (!view.match.seats.length) return false;
    return WR.scoped(view.match, view.scope).some((s) => s.jev || (view.live && view.live.files.includes(`jev-${s.header.ai_id}.jsonl`)));
  }

  /// The Jev bill a game minute of the seats in scope that have an index: [{minute, seats: Map seat -> tokens}].
  function costMinutes() {
    const hands = WR.scoped(view.match, view.scope).filter((s) => s.jev);
    return hands.length ? WR.costView(hands.map((s) => ({ seat: s.index, index: s.jev.index }))) : null;
  }

  /// The lanes this match has: [{name, label, items: [{f, to?, seat?, faint?}], ...}], the metal destroyed first.
  function lanesOf(m) {
    const match = view.match, list = [];
    if (m && m.sides.length >= 2 && m.sides.some((_, i) => m.series("lost_value", i))) list.push({ name: "trade", label: "metal destroyed", trade: true });
    if (!match.seats.length) return list;
    const lanes = WR.lanes(match, view.scope);
    const multi = match.seats.length > 1;
    const turns = (transcript, seat) => transcript.turns.map((t) => ({ f: t.f, to: WR.turnEnd(match, t), seat, turn: t }));
    if (match.commander) list.push({ name: "commander", label: "commander turns", color: "llm", items: turns(match.commander, null), bars: true });
    const players = new Map();
    for (const seat of WR.scoped(match, view.scope)) {
      const player = playerOf(seat);
      if (player && !players.has(player)) players.set(player, seat.index);
    }
    if (players.size) {
      const items = [...players].flatMap(([player, seat]) => turns(player, seat)).sort((a, b) => a.f - b.f);
      list.push({ name: "player", label: players.size > 1 ? "player turns, by seat" : "player turns", color: "llm", bySeat: players.size > 1, items, bars: players.size === 1 });
    }
    if (expectsHands()) {
      const hands = WR.scoped(match, view.scope).filter((s) => s.jev);
      const items = hands.flatMap((s) => s.jev.index.gates.map((f) => ({ f, seat: s.index }))).sort((a, b) => a.f - b.f);
      list.push({ name: "gates", label: "hands: gates asked", color: "jev", bySeat: multi, items });
    }
    for (const [name, label] of [["builders", "hands: builders"], ["labs", "hands: plants"], ["groups", "hands: army"]]) {
      const items = lanes[name].map((d) => ({ ...d, faint: d.source !== "plan" }));
      if (items.length) list.push({ name, label, color: "jev", bySeat: multi, items, plays: true });
    }
    return list;
  }

  // ---------------------------------------------------------------- a chart's lines

  /// A running total as what it gained in the last minute.
  function lastMinute(data) {
    const out = new Float32Array(data.length);
    for (let s = 0; s < data.length; s++) out[s] = data[s] - data[Math.max(0, s - 60)];
    return out;
  }

  /// The lines of a metric's chart: each side's, and before it its players' when it is unfolded in the strip.
  function linesOf(m, chart) {
    const key = metricOf(chart), minute = chart.endsWith("/min");
    const lines = [];
    m.sides.forEach((side, i) => {
      const whole = m.series(key, i);
      if (!whole) return;
      const of = (data) => (minute ? lastMinute(data) : data);
      if (view.unfolded.has(i) && side.teams.length > 1) {
        side.teams.forEach((t, j) => {
          const one = m.series(key, i, j);
          if (one) lines.push({ name: t.name, css: t.css, data: of(one), bold: false });
        });
      }
      lines.push({ name: side.name, css: side.css, data: of(whole), bold: true });
    });
    return lines;
  }

  /// A round number at or over `v`, and the step between the lines drawn under it.
  function axis(v) {
    const top = Math.max(v, 1e-9);
    const power = 10 ** Math.floor(Math.log10(top / 3));
    const step = [1, 2, 5, 10].map((k) => k * power).find((s) => top / s <= 3.5) || 10 * power;
    return { max: Math.ceil(top / step) * step, step };
  }

  /// A picture's columns: the value of each pixel column, a flow as the mean of the seconds it covers (a second's
  /// spike is a wreck reclaimed, a transfer), anything else as it stood at the column's end.
  function columns(data, w, mean) {
    const out = new Float32Array(w), n = data.length;
    for (let x = 0; x < w; x++) {
      const a = Math.min(n - 1, Math.floor((x / w) * n)), b = Math.max(a + 1, Math.min(n, Math.floor(((x + 1) / w) * n)));
      if (mean) {
        let sum = 0;
        for (let i = a; i < b; i++) sum += data[i];
        out[x] = sum / (b - a);
      } else out[x] = data[b - 1];
    }
    return out;
  }

  // ---------------------------------------------------------------- painting

  function fit(canvas) {
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth, h = canvas.clientHeight;
    canvas.width = Math.max(1, Math.round(w * dpr));
    canvas.height = Math.max(1, Math.round(h * dpr));
    const ctx = canvas.getContext("2d");
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);
    ctx.font = "10.5px system-ui";
    ctx.textBaseline = "middle";
    return { ctx, w, h };
  }

  const lastFrame = () => Math.max(view.match.lastFrame, 1);
  /// Minutes between the time axis' marks, however long the game.
  const everyMinutes = () => [1, 2, 5, 10, 20].find((n) => lastFrame() / (60 * FPS) / n <= 12) || 30;

  /// The minute marks behind a picture.
  function minuteLines(ctx, w, h) {
    ctx.strokeStyle = COLOR["surface-2"];
    ctx.lineWidth = 1;
    const every = everyMinutes();
    for (let minute = every; minute * 60 * FPS < lastFrame(); minute += every) {
      const x = Math.round((w * minute * 60 * FPS) / lastFrame()) + 0.5;
      ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, h); ctx.stroke();
    }
  }

  function paintAxis(canvas) {
    const { ctx, w } = fit(canvas);
    const every = everyMinutes();
    ctx.fillStyle = COLOR.muted;
    for (let minute = 0; minute * 60 * FPS <= lastFrame(); minute += every) {
      const x = Math.round((w * minute * 60 * FPS) / lastFrame()) + 0.5;
      ctx.strokeStyle = COLOR.line;
      ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, 4); ctx.stroke();
      ctx.textAlign = x < 20 ? "left" : x > w - 20 ? "right" : "center";
      ctx.fillText(`${minute}:00`, x, 11);
    }
  }

  function paintLane(canvas, lane) {
    const { ctx, w, h } = fit(canvas);
    ctx.fillStyle = COLOR["surface-2"];
    ctx.fillRect(0, 1, w, h - 2);
    for (const item of lane.items) {
      ctx.fillStyle = lane.bySeat && item.seat != null ? seatColor(item.seat) : COLOR[lane.color];
      ctx.globalAlpha = item.faint ? 0.35 : 1;
      const x = Math.round((w * item.f) / lastFrame());
      // A turn is as wide as it took, in a game that ran on meanwhile.
      const wide = lane.bars && item.to > item.f ? Math.max(2, Math.round((w * item.to) / lastFrame()) - x) : 2;
      ctx.fillRect(x - (wide === 2 ? 1 : 0), 2, wide, h - 4);
    }
    ctx.globalAlpha = 1;
  }

  /// What each side lost in each stretch of TRADE_SECONDS: [Float32Array a side].
  function tradeBars(m) {
    const n = Math.ceil(m.seconds / TRADE_SECONDS);
    return m.sides.map((_, i) => {
      const lost = m.series("lost_value", i), out = new Float32Array(n);
      if (lost) for (let b = 0; b < n; b++) out[b] = lost[Math.min(m.seconds - 1, (b + 1) * TRADE_SECONDS)] - lost[Math.min(m.seconds - 1, b * TRADE_SECONDS)];
      return out;
    });
  }

  /// The metal destroyed: the second side's losses up from the middle, the first side's (ours) down. A bar taller
  /// than the lane (one strike far over the rest) is cut and marked at its tip.
  function paintTrade(canvas, m) {
    const { ctx, w, h } = fit(canvas);
    ctx.fillStyle = COLOR["surface-2"];
    ctx.fillRect(0, 0, w, h);
    const bars = tradeBars(m);
    const all = bars.flatMap((b) => Array.from(b)).filter((v) => v > 0).sort((a, b) => a - b);
    if (!all.length) return;
    const cap = Math.max(1, Math.min(all[all.length - 1], all[Math.floor(0.97 * (all.length - 1))] * 1.5));
    const middle = Math.round(h / 2), half = middle - 1;
    const wide = Math.max(1, (w * TRADE_SECONDS * FPS) / lastFrame());
    [[1, -1], [0, 1]].forEach(([side, way]) => {
      if (!bars[side]) return;
      ctx.fillStyle = m.sides[side].css;
      bars[side].forEach((v, b) => {
        if (v <= 0) return;
        const tall = Math.max(1, Math.round((half * Math.min(v, cap)) / cap));
        const x = (w * b * TRADE_SECONDS * FPS) / lastFrame();
        ctx.fillRect(x, way < 0 ? middle - tall : middle, Math.max(1, wide - 0.5), tall);
        if (v > cap) {
          ctx.fillStyle = COLOR.ink;
          ctx.fillRect(x, way < 0 ? middle - tall : middle + tall - 1, Math.max(1, wide - 0.5), 1);
          ctx.fillStyle = m.sides[side].css;
        }
      });
    });
    ctx.fillStyle = COLOR.line;
    ctx.fillRect(0, middle, w, 1);
  }

  /// A metric's lines on a zero-based axis of its own.
  function paintChart(canvas, lines, mean) {
    const { ctx, w, h } = fit(canvas);
    minuteLines(ctx, w, h);
    const top = 5, bottom = h - 3;
    const drawn = lines.map((line) => ({ ...line, at: columns(line.data, w, mean) }));
    let peak = 0;
    for (const line of drawn) for (const v of line.at) if (v > peak) peak = v;
    if (mean) {
      // A flow's axis ends a little over what it nearly always stays under: a second's spike (a wreck reclaimed, a
      // gift) is cut off at the top rather than flattening the whole curve.
      const sorted = drawn.filter((line) => line.bold).flatMap((line) => Array.from(line.at)).filter((v) => v > 0).sort((a, b) => a - b);
      if (sorted.length) peak = Math.min(peak, sorted[Math.floor(0.985 * (sorted.length - 1))] * 1.25);
    }
    const { max, step } = axis(peak);
    const y = (v) => bottom - ((bottom - top) * Math.max(0, Math.min(v, max))) / max;
    ctx.textAlign = "left";
    for (let v = 0; v <= max + step / 2; v += step) {
      ctx.strokeStyle = v === 0 ? COLOR.line : COLOR["surface-2"];
      ctx.lineWidth = 1;
      ctx.beginPath(); ctx.moveTo(0, Math.round(y(v)) + 0.5); ctx.lineTo(w, Math.round(y(v)) + 0.5); ctx.stroke();
      if (v > 0) {
        ctx.fillStyle = COLOR.muted;
        ctx.fillText(WV.stats.short(v), 3, Math.min(bottom - 6, Math.max(top + 5, y(v) + 7)));
      }
    }
    for (const line of drawn) {
      ctx.strokeStyle = line.css;
      ctx.lineWidth = line.bold ? 1.75 : 1;
      ctx.globalAlpha = line.bold ? 1 : 0.8;
      ctx.beginPath();
      line.at.forEach((v, x) => (x ? ctx.lineTo(x + 0.5, y(v)) : ctx.moveTo(0.5, y(v))));
      ctx.stroke();
    }
    ctx.globalAlpha = 1;
  }

  /// The classes' series of a side, in the model's order, and the most any side's whole worth reaches.
  function classBands(m, side) {
    return m.classes.map((name) => ({ name, css: CLASS_COLORS[name] || COLOR.muted, data: m.series(`value_${name}`, side) })).filter((band) => band.data);
  }

  function classPeak(m) {
    let peak = 0;
    m.sides.forEach((_, i) => {
      const bands = classBands(m, i);
      for (let s = 0; s < m.seconds; s += 5) {
        let sum = 0;
        for (const band of bands) sum += band.data[s];
        if (sum > peak) peak = sum;
      }
    });
    return peak;
  }

  /// A side's worth by class, the bands stacked, on the axis every side's bands share.
  function paintClasses(canvas, m, side) {
    const { ctx, w, h } = fit(canvas);
    const top = 5, bottom = h - 3;
    const { max, step } = axis(classPeak(m));
    const y = (v) => bottom - ((bottom - top) * Math.min(v, max)) / max;
    const base = new Float32Array(w);
    for (const band of classBands(m, side)) {
      const at = columns(band.data, w, false);
      ctx.fillStyle = band.css;
      ctx.beginPath();
      ctx.moveTo(0, y(base[0]));
      for (let x = 0; x < w; x++) ctx.lineTo(x + 0.5, y(base[x] + at[x]));
      for (let x = w - 1; x >= 0; x--) ctx.lineTo(x + 0.5, y(base[x]));
      ctx.closePath();
      ctx.fill();
      for (let x = 0; x < w; x++) base[x] += at[x];
    }
    minuteLines(ctx, w, h);
    ctx.textAlign = "left";
    for (let v = step; v <= max + step / 2; v += step) {
      ctx.strokeStyle = "#00000055";
      ctx.beginPath(); ctx.moveTo(0, Math.round(y(v)) + 0.5); ctx.lineTo(w, Math.round(y(v)) + 0.5); ctx.stroke();
      ctx.fillStyle = COLOR["ink-2"];
      ctx.fillText(WV.stats.short(v), 3, Math.min(bottom - 6, Math.max(top + 5, y(v) + 7)));
    }
  }

  /// The hands' bill: a bar a game minute, stacked by seat.
  function paintCost(canvas, cost) {
    const { ctx, w, h } = fit(canvas);
    minuteLines(ctx, w, h);
    const top = 5, bottom = h - 3;
    const dollars = (tokens) => (tokens * WR.JEV_PRICE) / 1e6;
    const max = Math.max(0.01, ...cost.minutes.map((minute) => dollars(minute.tokens)));
    for (const minute of cost.minutes) {
      let base = 0;
      const [xa, xb] = [(w * minute.minute * 60 * FPS) / lastFrame(), Math.min(w, (w * (minute.minute + 1) * 60 * FPS) / lastFrame())];
      for (const [seat, tokens] of [...minute.seats].sort((a, b) => a[0] - b[0])) {
        const tall = ((bottom - top) * dollars(tokens)) / max;
        ctx.fillStyle = seatColor(seat);
        ctx.fillRect(xa + 0.5, bottom - base - tall, Math.max(1, xb - xa - 1), tall);
        base += tall;
      }
    }
    ctx.fillStyle = COLOR.muted;
    ctx.textAlign = "left";
    ctx.fillText(`$${max.toFixed(2)}`, 3, top + 5);
  }

  // ---------------------------------------------------------------- the rows

  /// The rows as built: [{box, canvas, values, paint(), value(second) -> [{css, name, text}], hover(second)}].
  let built = { key: null, rows: [], width: 0, at: null };
  let hoverFrame = null;

  function row(kind, height, label, buttons = []) {
    const box = el("div", `tl-row ${kind}`);
    box.style.height = `${height}px`;
    const name = el("div", "tl-label");
    name.append(el("span", null, label));
    for (const b of buttons) name.append(b);
    const canvas = el("canvas", "tl-plot");
    canvas.style.height = `${height}px`;
    const values = el("div", "tl-values");
    box.append(name, canvas, values);
    return { box, canvas, values };
  }

  function button(text, title, act) {
    const b = el("button", "tl-button", text);
    b.title = title;
    b.addEventListener("click", (e) => { e.stopPropagation(); act(); });
    return b;
  }

  function chartRow(m, chart) {
    const remove = button("×", "take this chart away", () => { prefs.charts = prefs.charts.filter((c) => c !== chart); changed(); });
    if (chart === "jev_cost") {
      const cost = costMinutes();
      if (!cost) return null;
      const r = row("chart", CHART_H, "Jev $ a minute", [remove]);
      return { ...r, paint: () => paintCost(r.canvas, cost), value: () => [] };
    }
    if (chart === "classes") return null; // (one row a side: see `build`)
    const metric = m.metric(metricOf(chart));
    const lines = metric ? linesOf(m, chart) : [];
    if (!lines.length) return null;
    const minute = chart.endsWith("/min");
    const buttons = [remove];
    if (metric.kind === "total") {
      buttons.unshift(button(minute ? "a minute" : "so far", minute ? "what it gained in the last minute; click for the running total" : "the running total; click for what it gained in the last minute", () => {
        prefs.charts = prefs.charts.map((c) => (c === chart ? (minute ? metric.key : `${metric.key}/min`) : c));
        changed();
      }));
    }
    // (Unfolded players make more lines than the usual height has room to name.)
    const r = row("chart", Math.max(CHART_H, Math.min(220, lines.length * 13 + 8)), metric.label, buttons);
    return {
      ...r, key: metric.key,
      paint: () => paintChart(r.canvas, lines, metric.kind === "rate"),
      value: (second) => lines.map((line) => ({ css: line.css, name: line.name, text: WV.stats.short(line.data[second]), bold: line.bold })),
    };
  }

  function classRow(m, side) {
    const bands = classBands(m, side);
    if (!bands.length) return null;
    const remove = button("×", "take these charts away", () => { prefs.charts = prefs.charts.filter((c) => c !== "classes"); changed(); });
    const r = row("chart", Math.max(CHART_H, bands.length * 13 + 8), `worth by class: ${m.sides[side].name}`, [remove]);
    return { ...r, paint: () => paintClasses(r.canvas, m, side), value: (second) => [...bands].reverse().map((band) => ({ css: band.css, name: band.name, text: WV.stats.short(band.data[second]) })) };
  }

  function tradeRow(m) {
    const r = row("lane trade", TRADE_H, "metal destroyed");
    r.box.title = `the metal of the finished units each side lost, ${TRADE_SECONDS} seconds a bar: ${m.sides[1].name} up, ${m.sides[0].name} down`;
    return {
      ...r, paint: () => paintTrade(r.canvas, m),
      // Beside it: what each side lost in the minute up to the playhead.
      value: (second) => [1, 0].map((side) => {
        const lost = m.series("lost_value", side);
        const gone = lost ? lost[second] - lost[Math.max(0, second - 60)] : null;
        return { css: m.sides[side].css, name: m.sides[side].name, text: gone == null ? "–" : `${gone ? "−" : ""}${WV.stats.short(gone)} /min` };
      }),
    };
  }

  function laneRow(lane) {
    const picked = lane.plays ? lane.items.filter((d) => !d.faint).length : 0;
    const r = row("lane", LANE_H, lane.plays ? `${lane.label} (${picked} pick, ${lane.items.length - picked} list)` : `${lane.label} (${lane.items.length})`);
    return { ...r, lane, paint: () => paintLane(r.canvas, lane), value: () => [] };
  }

  /// The menu of rows: the lanes this match has and every chart its numbers allow, a tick each. Built again only
  /// when what it lists or ticks changed (a match being played builds its rows again at every poll, and a tick
  /// under the mouse must not be swapped out).
  function buildMenu(m, lanes) {
    const box = $("rows-list");
    const key = JSON.stringify([lanes.map((lane) => lane.label), m ? [m.source, m.metrics.length, m.classes.length] : null, prefs, view.match.seats.length && expectsHands()]);
    if (box.dataset.key === key) return;
    box.dataset.key = key;
    box.textContent = "";
    const tick = (label, on, act) => {
      const line = el("label");
      const input = el("input");
      input.type = "checkbox";
      input.checked = on;
      input.addEventListener("change", () => act(input.checked));
      line.append(input, ` ${label}`);
      box.append(line);
    };
    const chart = (key, label) => tick(label, prefs.charts.some((c) => metricOf(c) === key), () => toggleChart(key));
    if (lanes.length) box.append(el("h2", null, "lanes"));
    for (const lane of lanes) tick(lane.label, !!prefs.lanes[lane.name], (on) => { prefs.lanes[lane.name] = on; changed(); });
    if (m) {
      for (const group of [...new Set(m.metrics.map((x) => x.group))]) {
        box.append(el("h2", null, `charts: ${group}`));
        for (const metric of m.metrics.filter((x) => x.group === group)) chart(metric.key, metric.label);
        if (group === "composition" && m.classes.length) chart("classes", "worth by class, a chart a side");
      }
    }
    if (view.match.seats.length && expectsHands()) {
      box.append(el("h2", null, "charts: cost"));
      chart("jev_cost", "Jev $ a minute");
    }
  }

  function build(m) {
    const box = $("timeline");
    box.textContent = "";
    const rows = [];
    const lanes = lanesOf(m);
    for (const lane of lanes) {
      if (!prefs.lanes[lane.name]) continue;
      rows.push(lane.trade ? tradeRow(m) : laneRow(lane));
    }
    if (document.body.dataset.compact !== "1") {
      for (const chart of prefs.charts) {
        if (chart === "classes" && m) m.sides.forEach((_, i) => rows.push(classRow(m, i)));
        else if (chart === "jev_cost" || m) rows.push(chartRow(m, chart));
      }
    }
    const axisRow = row("axis", AXIS_H, "");
    rows.push({ ...axisRow, paint: () => paintAxis(axisRow.canvas), value: () => [] });
    const kept = rows.filter(Boolean);
    for (const r of kept) box.append(r.box);
    buildMenu(m, lanes);
    return kept;
  }

  /// Where the pictures stand inside the timeline's box: their left edge and their width, in page pixels.
  function plot() {
    const first = built.rows[0];
    if (!first) return null;
    const c = first.canvas.getBoundingClientRect(), wrap = $("timeline-wrap").getBoundingClientRect();
    return { left: c.left - wrap.left, width: c.width, top: c.top - wrap.top };
  }

  function place(line, frame, p) {
    line.hidden = frame == null || !p;
    if (!line.hidden) line.style.left = `${Math.round(p.left + (p.width * frame) / lastFrame())}px`;
  }

  /// Called at every change of the page: builds the rows when what they show changed, paints them when they or the
  /// width did, then moves the playhead and writes the values at it.
  function draw() {
    const match = view.match;
    if (!match || (match.tapeOnly && !view.scene)) return;
    const m = WV.stats.model();
    const key = [m ? m.id : 0, view.version, view.scope, document.body.dataset.compact, JSON.stringify(prefs), [...view.unfolded].join(","), match.seats.length && expectsHands(), match.lastFrame].join("|");
    if (built.key !== key) built = { key, rows: build(m), width: 0, at: null };
    const p = plot();
    const width = p ? `${Math.round(p.width)}:${window.devicePixelRatio || 1}` : "";
    if (built.width !== width) {
      built.width = width;
      for (const r of built.rows) r.paint();
    }
    place($("playhead"), view.frame, p);
    place($("hoverhead"), hoverFrame, p);
    const second = m ? WV.stats.secondOf(m, view.frame) : Math.floor(view.frame / FPS);
    if (built.at === second) return;
    built.at = second;
    for (const r of built.rows) {
      const lines = r.value(second);
      r.values.textContent = "";
      for (const line of lines) {
        const div = el("div", line.bold ? "bold" : null);
        const dot = el("i");
        dot.style.background = line.css;
        div.append(dot, el("span", null, line.name), el("b", null, line.text));
        r.values.append(div);
      }
    }
  }

  // ---------------------------------------------------------------- the mouse

  /// The frame under the mouse, or null when it is not over the pictures (a label, a value, a button).
  function frameAt(event) {
    const p = plot();
    if (!p) return null;
    const x = event.clientX - $("timeline-wrap").getBoundingClientRect().left - p.left;
    if (x < -2 || x > p.width + 2) return null;
    return Math.round(Math.min(1, Math.max(0, x / p.width)) * lastFrame());
  }

  function hover(event) {
    const match = view.match;
    if (!match || (match.tapeOnly && !view.scene)) return;
    const frame = frameAt(event), tip = $("chart-tooltip");
    hoverFrame = frame;
    if (frame == null) {
      tip.hidden = true;
      return draw();
    }
    const m = WV.stats.model();
    const lines = [WR.clock(frame)];
    // The row under the mouse: its values there.
    const under = built.rows.find((r) => r.box.contains(event.target));
    if (under && m) for (const line of under.value(WV.stats.secondOf(m, frame))) lines.push(`${line.name}: ${line.text}`);
    const [from, to] = [frame - 5 * FPS, frame + 5 * FPS];
    if (m) m.lostBetween(from, to).forEach((names, i) => names.length && lines.push(`${m.sides[i].name} lost: ${summarise(names)}`.slice(0, 140)));
    if (match.seats.length) {
      const near = (items) => WR.range(items, from, to);
      for (const r of built.rows) {
        if (!r.lane || !r.lane.items.length || !r.lane.items[0].turn) continue;
        for (const item of near(r.lane.items).slice(0, 3)) {
          const who = item.seat != null && match.seats.length > 1 ? `player ${match.seats[item.seat].name}` : r.lane.label.replace(/ turns.*/, "");
          lines.push(`${WR.clock(item.f)} ${who}: ${item.turn.wake || "turn"}${item.turn.wallSeconds != null ? ` (${item.turn.wallSeconds.toFixed(0)} s)` : ""}`.slice(0, 110));
        }
      }
      const lanes = WR.lanes(match, view.scope);
      const plays = near(lanes.builders).concat(near(lanes.labs), near(lanes.groups)).sort((a, b) => a.f - b.f);
      for (const d of plays.slice(0, 6)) lines.push(`${WR.clock(d.f)} ${d.source}: ${d.inputs.actor}: ${d.outputs.did || ""}`.slice(0, 110));
      if (plays.length > 6) lines.push(`... ${plays.length - 6} more plays`);
    }
    tip.textContent = lines.join("\n");
    tip.hidden = false;
    const wrap = $("timeline-wrap").getBoundingClientRect();
    const x = event.clientX - wrap.left;
    tip.style.left = `${x > wrap.width - 440 ? Math.max(0, x - 420) : x + 14}px`;
    tip.style.bottom = `${wrap.bottom - event.clientY + 12}px`;
    if (event.buttons & 1) seek(frame);
    else draw();
  }

  function wire(leaveLive) {
    const box = $("timeline");
    box.addEventListener("mousedown", (e) => {
      const frame = view.match ? frameAt(e) : null;
      if (frame == null || e.button !== 0) return;
      leaveLive();
      seek(frame);
    });
    box.addEventListener("mousemove", hover);
    box.addEventListener("mouseleave", () => {
      hoverFrame = null;
      $("chart-tooltip").hidden = true;
      draw();
    });
    // The menu of rows closes at a click anywhere else.
    document.addEventListener("mousedown", (e) => {
      const menu = $("rows-menu");
      if (menu.open && !menu.contains(e.target)) menu.open = false;
    });
  }

  WV.timeline = { draw, wire, charted, toggleChart, lanesOf, plot, rows: () => built.rows };
})();
