// Within Reason match viewer: loading, the map, the timeline and the panels. Parsing and the model are in record.js.
//
// URL parameters: match=<base URL of the match files, default "match/">, record=<file name>, t=<seconds>,
// bg=<image URL drawn under the map>. Without a server, use "Open files" and pick the record and its siblings.
// A record without a result line is a match still being played: its files are polled for new bytes
// (run/view_match.py serves `?from=<offset>`) and, with "follow live" ticked, the playhead stays on the newest sample.
"use strict";

(() => {
  const $ = (id) => document.getElementById(id);
  const css = (name) => getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  const COLOR = {};
  for (const name of ["surface", "surface-2", "line", "ink", "ink-2", "muted", "ours", "theirs", "good", "critical", "warning", "llm", "jev"]) COLOR[name] = css(`--${name}`);
  const OWN_ATTACKER = "#86b6ef";
  const ALLY = "#3fae8f";
  const ORDER_COLOR = { build: COLOR.good, fight: COLOR.critical, move: COLOR.muted, attack: COLOR.critical, guard: COLOR.jev, repair: COLOR.good, reclaim: COLOR.warning, reclaim_feature: COLOR.warning, resurrect: COLOR.warning };
  /// Zoom from which buildings show their footprints and units their names; the map's zoom limit over the fitted scale.
  const DETAIL_ZOOM = 4;
  const MAX_ZOOM = 32;
  /// How long an order line and a death mark stay on the map, in frames.
  const ORDER_FRAMES = 3 * WR.FPS;
  const DEATH_FRAMES = 20 * WR.FPS;

  const view = {
    match: null, lanes: null, posts: [], frame: 0, playing: false, speed: 10, lastTime: 0,
    layers: { terrain: true, nobots: true, notanks: false, metal: true, grid: true, spots: true, truth: true, census: true, orders: true, intent: true, deaths: true, pianist: true },
    terrain: null,
    background: null, mapBox: null, hoverFrame: null, decisionItems: [], currentDecision: -2,
    show: { heuristic: true, llm: true, event: false, jev: true, jevChangesOnly: true },
    /// The pianist's actor the decision list is narrowed to ("" for all), and the call drawn last.
    jevActor: "", callShown: null, tab: null,
    /// The map's view: zoom over the fitted scale (1 to MAX_ZOOM) and the elmo at the pane's centre; the selected
    /// unit; the game's icon table (viewer/icons.json).
    zoom: 1, centre: null, selected: null, icons: null,
  };
  window.viewer = view;
  view.seek = (frame) => seek(frame); // for the tests (viewer/test/browser.js), which drive the page from outside

  // ---------------------------------------------------------------- loading

  async function fetchText(url) {
    try {
      const response = await fetch(url);
      return response.ok ? await response.text() : null;
    } catch {
      return null;
    }
  }

  /// A file of the match being followed: the text so far and how many bytes of the file it is.
  const pulled = new Map();
  const POLL_MS = 3000;

  /// Fetches what a file has gained since the last pull. True if it grew.
  async function pull(base, name) {
    const file = pulled.get(name) || { bytes: 0, text: "", decoder: new TextDecoder() };
    try {
      const response = await fetch(`${base}${name}?from=${file.bytes}`);
      if (!response.ok) return false;
      const chunk = new Uint8Array(await response.arrayBuffer());
      // A server that does not know `from` (anything but run/view_match.py) sends the whole file every time.
      const tail = response.headers.get("X-From") === String(file.bytes);
      if (tail ? !chunk.length : chunk.length === file.bytes) return false;
      if (!tail) Object.assign(file, { bytes: 0, text: "", decoder: new TextDecoder() });
      file.text += file.decoder.decode(chunk, { stream: true });
      file.bytes += chunk.length;
      pulled.set(name, file);
      return true;
    } catch {
      return false;
    }
  }

  /// The text up to its last complete line: the writer may be half way through one.
  const complete = (name) => {
    const text = (pulled.get(name) || {}).text;
    return text ? text.slice(0, text.lastIndexOf("\n") + 1) : null;
  };

  /// Pulls every file of the match; returns the texts `open` takes, or null when nothing has changed.
  async function pullMatch(live) {
    const index = await fetchText(`${live.base}index.json`);
    const listing = index ? JSON.parse(index) : { files: [], replays: [] };
    const files = listing.files || [];
    showReplay(live.base, listing.replays || []);
    if (!live.record) live.record = files.find((f) => /^record-.*\.jsonl$/.test(f));
    if (!live.record) return { files };
    let grew = await pull(live.base, live.record);
    const record = complete(live.record);
    if (!record) return { files };
    const header = JSON.parse(record.slice(0, record.indexOf("\n")));
    const siblings = header.siblings || {};
    // Transcripts the header names, and any other the directory lists (a record and a transcript brought together by hand).
    const logs = [...new Set([...(siblings.decision_logs || []), ...files.filter((f) => /^(strategist|jev)-.*\.jsonl$/.test(f))])];
    const names = { engineLog: siblings.engine_log || "engine.log", botLog: siblings.bot_log || "bot.log", truth: `truth-${header.ai_id}.jsonl` };
    for (const name of [...logs, ...Object.values(names)]) grew = (await pull(live.base, name)) || grew;
    if (!grew) return null;
    return {
      files,
      texts: {
        record,
        strategist: logs.filter((f) => /^strategist/.test(f)).map(complete).filter(Boolean),
        jev: logs.filter((f) => /^jev/.test(f)).map(complete).find(Boolean) || null,
        engineLog: complete(names.engineLog), botLog: complete(names.botLog), truth: complete(names.truth),
      },
    };
  }

  /// The engine's replay of the match (demos/*.sdfz, written when the engine quits), as a download link in the header.
  function showReplay(base, replays) {
    const link = $("replay");
    if (!replays.length) return;
    const path = replays[replays.length - 1];
    link.href = base + path.split("/").map(encodeURIComponent).join("/");
    link.download = path.split("/").pop();
    link.hidden = false;
  }

  async function boot() {
    const params = new URLSearchParams(location.search);
    let base = params.get("match");
    if (!base) {
      // No match named: the server's listing decides. A single match served the old way opens itself; a batch or
      // the matches directory opens the browser; no listing (another server) means the old address.
      const listing = await fetchText("matches/index.json");
      const index = listing ? JSON.parse(listing) : null;
      if (index && index.kind !== "match") return renderBrowser(index);
      base = index ? "match/" : "match/";
    }
    if (!base.endsWith("/")) base += "/";
    $("browse").hidden = false;
    const live = { base, record: params.get("record") };
    const first = await pullMatch(live);
    if (!first.texts) {
      if (first.files.length) return status(`No record-*.jsonl in the match directory (was it played with WITHIN_REASON_RECORD=1?). Files: ${first.files.join(", ")}`);
      return status("No match loaded. Start with run/view_match.py <match dir>, or use Open files.");
    }
    open(first.texts, Number(params.get("t") || 0) * WR.FPS);
    loadIcons();
    const bg = params.get("bg") || `maps/${encodeURIComponent(view.match.header.map.name)}.png`;
    loadBackground(bg);
    loadTerrain(live.base, view.match.header.terrain);
    if (!view.match.result) follow(live, !params.get("t"));
  }

  /// The match browser: every batch the server lists, newest first, each match a link into the viewer.
  function renderBrowser(index) {
    document.body.dataset.browser = "1";
    const box = $("browser");
    box.hidden = false;
    box.textContent = "";
    $("subtitle").textContent = `${index.root}: ${index.batches.length} ${index.kind === "batch" ? "batch" : "batches"}`;
    const filter = el("input");
    filter.type = "search";
    filter.placeholder = "filter by label, opponent, map, commit";
    const table = el("table", "batches");
    const head = el("tr");
    for (const h of ["started", "label", "opponent", "map", "commit", "hands", "matches"]) head.append(el("th", null, h));
    table.append(head);
    const rows = [];
    for (const b of index.batches) {
      const tr = el("tr");
      const when = b.started ? new Date(b.started * 1000) : null;
      tr.append(el("td", "when", when ? `${when.toISOString().slice(0, 10)} ${when.toTimeString().slice(0, 5)}` : ""));
      tr.append(el("td", "label", b.label || b.batch));
      tr.append(el("td", null, [b.opponent, b.max_minutes ? `${b.max_minutes} min cap` : null].filter(Boolean).join(", ")));
      tr.append(el("td", null, b.map || ""));
      tr.append(el("td", "mono", b.commit || ""));
      tr.append(el("td", null, [b.player ? "player" : null, b.pianist ? "pianist" : null, b.packet ? `packet ${b.packet}` : null, b.rules === false ? "no rules" : null].filter(Boolean).join(", ")));
      const cell = el("td", "matches");
      for (const m of b.matches || []) {
        const path = b.batch === "." ? `matches/${m.index ? `${m.index}/` : ""}` : `matches/${encodeURIComponent(b.batch)}/${m.index}/`;
        const a = el("a", `match ${m.record ? "" : "norecord"} ${(m.outcome || "").toLowerCase()}`);
        a.href = `?match=${path}`;
        const words = m.outcome ? `${m.outcome}${m.minutes != null ? ` ${m.minutes.toFixed(1)} min` : ""}` : b.finished ? "ended, no result recorded" : "playing";
        a.textContent = `${m.index || "match"}: ${words}${m.arm ? ` (${m.arm})` : ""}${m.record ? "" : " · no record"}`;
        a.title = `${b.batch}/${m.index}`;
        cell.append(a);
      }
      tr.append(cell);
      table.append(tr);
      rows.push({ tr, text: `${b.batch} ${b.label || ""} ${b.opponent || ""} ${b.map || ""} ${b.commit || ""} ${b.packet || ""}`.toLowerCase() });
    }
    filter.addEventListener("input", () => {
      const q = filter.value.trim().toLowerCase();
      for (const r of rows) r.tr.hidden = !!q && !r.text.includes(q);
    });
    box.append(el("h2", null, "Matches"), filter, table);
    status("pick a match; a batch without a result is still being played");
  }

  /// Keeps a match that is still being played up to date, until its result line arrives.
  function follow(live, stayOnNewest) {
    $("follow-label").hidden = false;
    $("follow").checked = stayOnNewest;
    if (stayOnNewest) seek(view.match.lastFrame);
    const timer = setInterval(async () => {
      const next = await pullMatch(live);
      if (!next || !next.texts) return;
      open(next.texts, $("follow").checked ? Infinity : view.frame, true);
      if (view.match.result) {
        clearInterval(timer);
        $("follow-label").hidden = true;
      }
    }, POLL_MS);
  }

  // The record's terrain grid (docs/harness/record-format.md): heights then slopes, rendered once into three
  // canvases the map draws under everything else: relief with water, and where bots and vehicles cannot go.
  async function loadTerrain(base, terrain) {
    if (!terrain || !terrain.file) return;
    let bytes;
    try {
      const response = await fetch(base + terrain.file);
      if (!response.ok) return;
      bytes = await response.arrayBuffer();
    } catch (_) {
      return;
    }
    const { width, height } = terrain;
    const cells = width * height;
    if (bytes.byteLength < cells * 3) return;
    const heights = new Int16Array(bytes, 0, cells);
    const slopes = new Uint8Array(bytes, cells * 2, cells);
    // The raw metal map, from records of 2026-09-24 on: the patches the spots stand for.
    const metalValues = terrain.metal && bytes.byteLength >= cells * 4 ? new Uint8Array(bytes, cells * 3, cells) : null;
    const canvasOf = (paint) => {
      const canvas = document.createElement("canvas");
      canvas.width = width;
      canvas.height = height;
      const ctx = canvas.getContext("2d");
      const image = ctx.createImageData(width, height);
      for (let i = 0; i < cells; i++) paint(i, image.data, i * 4);
      ctx.putImageData(image, 0, 0);
      return canvas;
    };
    let top = 1;
    for (let i = 0; i < cells; i++) if (heights[i] > top) top = heights[i];
    const relief = canvasOf((i, out, o) => {
      const h = heights[i];
      // Light from the north-west: a cell brighter than its south-east neighbour faces the light.
      const x = i % width, z = (i / width) | 0;
      const other = heights[Math.min(z + 1, height - 1) * width + Math.min(x + 1, width - 1)];
      const shade = Math.max(-40, Math.min(40, (h - other) * 6));
      if (h < 0) {
        const depth = Math.min(1, -h / 120);
        out[o] = 18; out[o + 1] = 52 - 20 * depth; out[o + 2] = 96 - 36 * depth;
      } else {
        const t = h / top;
        out[o] = 46 + 96 * t + shade; out[o + 1] = 62 + 78 * t + shade; out[o + 2] = 40 + 60 * t + shade;
      }
      out[o + 3] = 255;
    });
    const blocked = (kind, rgb) => {
      const classes = (terrain.move_classes || []).filter((c) => c.kind === kind);
      if (!classes.length) return null;
      // The ordinary class of the kind: not the amphibians (any depth) or climbers (any slope), then the one most
      // unit types use.
      const ordinary = classes.filter((c) => c.depth < 1000 && c.max_slope < 0.99);
      const usual = (ordinary.length ? ordinary : classes).reduce((a, b) => ((b.units || 0) > (a.units || 0) ? b : a));
      const maxSlope = usual.max_slope * 255;
      const depth = usual.depth;
      return canvasOf((i, out, o) => {
        const no = slopes[i] > maxSlope || heights[i] < -depth;
        out[o] = rgb[0]; out[o + 1] = rgb[1]; out[o + 2] = rgb[2]; out[o + 3] = no ? 150 : 0;
      });
    };
    let metal = null;
    if (metalValues) {
      let top = Math.max(1, terrain.metal_max || 0);
      for (let i = 0; i < cells; i++) if (metalValues[i] > top) top = metalValues[i];
      metal = canvasOf((i, out, o) => {
        const v = metalValues[i];
        out[o] = 255; out[o + 1] = 205; out[o + 2] = 60; out[o + 3] = v ? 90 + Math.round((165 * v) / top) : 0;
      });
    }
    view.terrain = { relief, nobots: blocked("bot", [200, 40, 40]), notanks: blocked("tank", [230, 140, 30]), metal, metalValues, heights, width, height, cell: terrain.cell };
    drawMap();
  }

  // Terrain hook: any image of the whole map, north up. Missing is normal.
  function loadBackground(url) {
    const image = new Image();
    image.onload = () => {
      view.background = image;
      drawMap();
    };
    image.src = url;
  }

  async function openFiles(fileList) {
    const texts = { strategist: [] };
    for (const file of fileList) {
      const text = await file.text();
      if (/^strategist.*\.jsonl$/.test(file.name)) texts.strategist.push(text);
      else if (/^jev.*\.jsonl$/.test(file.name)) texts.jev = text;
      else if (/\.jsonl$/.test(file.name)) texts.record = text;
      else if (/engine/.test(file.name)) texts.engineLog = text;
      else if (/bot/.test(file.name)) texts.botLog = text;
    }
    if (!texts.record) return status("Pick a record-*.jsonl (and optionally strategist-*.jsonl, engine.log, bot.log).");
    open(texts, 0);
  }

  /// `refresh`: the same match with more of it; what the reader has open and where they have scrolled is kept.
  function open(texts, frame, refresh = false) {
    let match;
    try {
      match = WR.parseRecord(texts.record);
    } catch (e) {
      return status(`Cannot read the record: ${e.message}`);
    }
    const mode = match.header.mode;
    const source = mode === "heuristic" ? "llm" : `llm:${mode}`;
    for (const text of texts.strategist || []) match.decisions.push(...WR.parseStrategist(text, source));
    match.decisions.sort((a, b) => a.f - b.f);
    if (texts.engineLog) match.census = WR.parseCensus(texts.engineLog, match.classByName);
    if (texts.truth) match.truth = WR.parseTruth(texts.truth, match.classByName);
    // The opponent's curve: ground truth every two seconds when the match has it, else the once-a-minute census.
    match.theirs = match.truth.length ? match.truth : match.census;
    if (texts.botLog) match.botLog = WR.parseBotLog(texts.botLog);
    match.jev = texts.jev ? WR.parseJev(texts.jev) : null;
    document.body.dataset.pianist = match.jev ? "1" : "";
    document.querySelector('#tabs [data-tab="pianist"]').hidden = !match.jev;
    if (match.jev) buildPianistMinutes(match.jev);
    view.callShown = null;
    view.match = match;
    view.lanes = WR.lanes(match);
    view.posts = WR.squadPosts(match.decisions, match.lastFrame);
    view.frame = Math.min(frame, match.lastFrame);
    describeMatch();
    if (refresh) rebuildDecisionList();
    else buildDecisionList();
    if (!refresh) buildLegend();
    if (refresh) {
      renderAll();
    } else {
      // The tab last used, unless it is the pianist's and this match has no log; setTab renders everything.
      let saved = null;
      try { saved = localStorage.getItem("wr-tab"); } catch (_) { /* no storage */ }
      setTab(saved && (saved !== "pianist" || match.jev) ? saved : match.jev ? "pianist" : "decisions");
    }
    document.body.dataset.loaded = `${match.samples.length} samples`;
  }

  function status(text) {
    $("status").textContent = text;
  }

  function describeMatch() {
    const { header, result, samples, badLines, restarts, census } = view.match;
    const side = { arm: "Armada", cor: "Cortex", leg: "Legion" }[header.side] || header.side || "?";
    const parts = [`${header.map.name}`, `${side}, team ${header.team}`, header.mode];
    if (result) {
      const r = result.result;
      parts.push(`vs ${result.opponent}`, `${r.outcome} after ${r.game_minutes.toFixed(1)} min (${r.our_corner})`);
      if (result.replay) parts.push(`engine replay: ${result.replay}`);
    } else {
      parts.push("no result line (match unfinished or killed)");
    }
    $("subtitle").textContent = parts.join("  ·  ");
    const notes = [`${samples.length} samples`];
    if (badLines) notes.push(`${badLines} unreadable line(s) skipped`);
    if (restarts.length) notes.push(`bot restarted at ${restarts.map(WR.clock).join(", ")}`);
    if (view.match.jev) {
      const j = view.match.jev;
      const asks = j.passes.filter((p) => p.gate).length;
      notes.push(`pianist: ${j.calls.length} calls to ${j.header?.model || j.calls[0]?.model || "Jev"}${j.errors.length ? `, ${j.errors.length} failed` : ""}${j.version >= 2 ? ` · one pass: ${j.passes.length} pass lines, ${asks} asked, ${j.plans.length} picks` : ""}`);
    }
    notes.push(view.match.truth.length ? "opponent ground truth" : census.length ? `${census.length} census minutes` : "no census (play with WITHIN_REASON_OBSERVE=1 for the opponent's truth)");
    status(notes.join(" · "));
  }

  // ---------------------------------------------------------------- canvas helpers

  function fitCanvas(canvas) {
    const dpr = window.devicePixelRatio || 1;
    const { clientWidth: w, clientHeight: h } = canvas;
    if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
    }
    const ctx = canvas.getContext("2d");
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);
    return { ctx, w, h };
  }

  // One glyph per unit class; buildings are angular, mobile units round.
  function glyph(ctx, cls, x, y, color, scale = 1) {
    ctx.fillStyle = color;
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.5;
    const square = (r) => ctx.fillRect(x - r, y - r, 2 * r, 2 * r);
    ctx.beginPath();
    switch (cls) {
      case "commander":
        ctx.moveTo(x, y - 7 * scale); ctx.lineTo(x + 6 * scale, y); ctx.lineTo(x, y + 7 * scale); ctx.lineTo(x - 6 * scale, y);
        ctx.closePath(); ctx.fill();
        break;
      case "army": ctx.arc(x, y, 3 * scale, 0, 7); ctx.fill(); break;
      case "builder": ctx.arc(x, y, 3.5 * scale, 0, 7); ctx.stroke(); break;
      case "extractor":
        square(3.5 * scale);
        ctx.fillStyle = COLOR.surface; ctx.fillRect(x - 1.2 * scale, y - 1.2 * scale, 2.4 * scale, 2.4 * scale);
        break;
      case "factory": square(5.5 * scale); break;
      case "turret":
        ctx.moveTo(x, y - 5 * scale); ctx.lineTo(x + 4.5 * scale, y + 3.5 * scale); ctx.lineTo(x - 4.5 * scale, y + 3.5 * scale);
        ctx.closePath(); ctx.fill();
        break;
      case "building": square(2.5 * scale); break;
      case "unknown": ctx.arc(x, y, 2.5 * scale, 0, 7); ctx.setLineDash([2, 2]); ctx.stroke(); ctx.setLineDash([]); break;
      default: ctx.arc(x, y, 2 * scale, 0, 7); ctx.fill();
    }
  }

  function buildLegend() {
    const list = $("map-legend");
    list.textContent = "";
    const entry = (cls, color, text) => {
      const li = document.createElement("li");
      const icon = document.createElement("canvas");
      icon.width = icon.height = 16;
      glyph(icon.getContext("2d"), cls, 8, 8, color);
      li.append(icon, text);
      list.append(li);
    };
    for (const cls of ["commander", "army", "builder", "extractor", "factory", "turret", "building"]) entry(cls, COLOR.ours, cls);
    entry("army", OWN_ATTACKER, "attacker");
    entry("army", ALLY, "ally (another seat of ours, or another player)");
    entry("army", COLOR.theirs, "enemy seen");
    entry("unknown", COLOR.theirs, "radar contact");
  }

  // ---------------------------------------------------------------- icons

  // The game's minimap icons (viewer/icons.json and viewer/icons/, exported by run/icons.py from BAR's
  // gamedata/icontypes.lua): white shapes, tinted here by side as the game tints them by team.
  const iconImages = new Map();
  const tinted = new Map();
  async function loadIcons() {
    const text = await fetchText("icons.json");
    if (!text) return;
    try { view.icons = JSON.parse(text); } catch (_) { view.icons = null; }
    drawMap();
  }
  function iconOf(name) {
    const entry = view.icons && view.icons[name];
    if (!entry) return null;
    let image = iconImages.get(entry.file);
    if (!image) {
      image = new Image();
      image.onload = () => { tinted.clear(); drawMap(); };
      image.src = `icons/${entry.file}`;
      iconImages.set(entry.file, image);
    }
    return image.complete && image.naturalWidth ? { image, size: entry.size } : null;
  }
  function tintedIcon(icon, color, px) {
    const key = `${icon.image.src}|${color}|${px}`;
    let canvas = tinted.get(key);
    if (!canvas) {
      canvas = document.createElement("canvas");
      canvas.width = canvas.height = px;
      const ctx = canvas.getContext("2d");
      // As the engine draws them: the bitmap's colour times the team colour, its own alpha kept. The building icons
      // are opaque squares whose shape is in the shading; the mobile ones carry theirs in the alpha.
      ctx.drawImage(icon.image, 0, 0, px, px);
      ctx.globalCompositeOperation = "multiply";
      ctx.fillStyle = color;
      ctx.fillRect(0, 0, px, px);
      ctx.globalCompositeOperation = "destination-in";
      ctx.drawImage(icon.image, 0, 0, px, px);
      if (tinted.size > 400) tinted.clear();
      tinted.set(key, canvas);
    }
    return canvas;
  }
  /// Pixels across for a unit's icon: the icon's own size, growing with the zoom, and for a building never wider
  /// than its footprint on the ground.
  function iconPixels(d, icon, box) {
    let px = 11 * Math.max(0.8, icon.size) * Math.sqrt(box.zoom);
    if (d.footprint && d.speed === 0) px = Math.min(px, Math.max(9, Math.max(d.footprint[0], d.footprint[1]) * 8 * box.scale * 1.1));
    return Math.max(8, Math.round(Math.min(px, 96)));
  }
  /// A unit on the map: its icon tinted, or the class glyph while the icon is not there.
  function drawUnit(ctx, u, x, y, color, box) {
    const d = u.def >= 0 ? view.match.defs[u.def] : null;
    const icon = d ? iconOf(d.name) : null;
    if (!icon) return glyph(ctx, classOf(u.def), x, y, color);
    const px = iconPixels(d, icon, box);
    ctx.drawImage(tintedIcon(icon, color, px), x - px / 2, y - px / 2);
  }

  // ---------------------------------------------------------------- map

  function classOf(def) {
    const d = def >= 0 ? view.match.defs[def] : null;
    return d ? d.class : "unknown";
  }

  function defName(def) {
    const d = def >= 0 ? view.match.defs[def] : null;
    return d ? d.name : "unidentified";
  }

  function drawMap() {
    const { ctx, w, h } = fitCanvas($("map"));
    const match = view.match;
    if (!match) return;
    const map = match.header.map;
    const margin = 18;
    const fit = Math.min((w - 2 * margin) / map.width, (h - 2 * margin) / map.height);
    const scale = fit * view.zoom;
    // Fitted, the map sits centred; zoomed, the pane's centre is `view.centre`, kept on the map.
    if (!view.centre || view.zoom === 1) view.centre = [map.width / 2, map.height / 2];
    view.centre = [Math.min(map.width, Math.max(0, view.centre[0])), Math.min(map.height, Math.max(0, view.centre[1]))];
    const box = { x: w / 2 - view.centre[0] * scale, y: h / 2 - view.centre[1] * scale, w: map.width * scale, h: map.height * scale, scale, zoom: view.zoom, fit, paneW: w, paneH: h };
    view.mapBox = box;
    const px = (x) => box.x + x * scale;
    const pz = (z) => box.y + z * scale;

    ctx.fillStyle = "#121211";
    ctx.fillRect(box.x, box.y, box.w, box.h);
    if (view.background) {
      ctx.globalAlpha = 0.55;
      ctx.drawImage(view.background, box.x, box.y, box.w, box.h);
      ctx.globalAlpha = 1;
    }
    if (view.terrain) {
      ctx.imageSmoothingEnabled = true;
      if (view.layers.terrain) {
        ctx.globalAlpha = 0.75;
        ctx.drawImage(view.terrain.relief, box.x, box.y, box.w, box.h);
      }
      ctx.globalAlpha = 0.55;
      if (view.layers.notanks && view.terrain.notanks) ctx.drawImage(view.terrain.notanks, box.x, box.y, box.w, box.h);
      if (view.layers.nobots && view.terrain.nobots) ctx.drawImage(view.terrain.nobots, box.x, box.y, box.w, box.h);
      ctx.globalAlpha = 1;
      // The metal patches, sharp-edged cells: what an extractor at the spot draws from.
      if (view.layers.metal && view.terrain.metal) {
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(view.terrain.metal, box.x, box.y, box.w, box.h);
        ctx.imageSmoothingEnabled = true;
      }
    }

    if (view.layers.grid) {
      const { columns, rows } = match.header.grid;
      ctx.strokeStyle = COLOR.line;
      ctx.lineWidth = 1;
      ctx.fillStyle = COLOR.muted;
      ctx.font = "11px system-ui";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      for (let c = 0; c <= columns; c++) {
        const x = Math.round(box.x + (box.w * c) / columns) + 0.5;
        ctx.beginPath(); ctx.moveTo(x, box.y); ctx.lineTo(x, box.y + box.h); ctx.stroke();
        if (c < columns) ctx.fillText(String.fromCharCode(65 + c), x + box.w / columns / 2, box.y - 9);
      }
      for (let r = 0; r <= rows; r++) {
        const y = Math.round(box.y + (box.h * r) / rows) + 0.5;
        ctx.beginPath(); ctx.moveTo(box.x, y); ctx.lineTo(box.x + box.w, y); ctx.stroke();
        if (r < rows) ctx.fillText(String(r + 1), box.x - 9, y + box.h / rows / 2);
      }
    }

    if (view.layers.spots) {
      // Each spot: a ring, and the map's extractor radius around it (the game's own rule for a second extractor
      // there) once that circle is wider than the ring.
      ctx.lineWidth = 1;
      const radius = (map.extractor_radius || 0) * scale;
      for (const [x, z] of match.header.metal_spots) {
        ctx.strokeStyle = COLOR.muted;
        ctx.beginPath(); ctx.arc(px(x), pz(z), 5, 0, 7); ctx.stroke();
        if (radius > 7) {
          ctx.strokeStyle = COLOR.warning;
          ctx.globalAlpha = 0.5;
          ctx.setLineDash([3, 3]);
          ctx.beginPath(); ctx.arc(px(x), pz(z), radius, 0, 7); ctx.stroke();
          ctx.setLineDash([]);
          ctx.globalAlpha = 1;
        }
      }
    }

    // Ground truth: every enemy unit where it really was, faint; what our units could see is drawn solid on top.
    const truth = view.layers.truth ? match.truth[WR.indexAt(match.truth, view.frame)] : null;
    if (truth) {
      ctx.globalAlpha = 0.5;
      for (const u of truth.units) glyph(ctx, u.class, px(u.x), pz(u.z), COLOR.theirs, u.building ? 0.7 : 1);
      ctx.globalAlpha = 1;
    }
    const census = view.layers.census && !truth ? match.census[WR.indexAt(match.census, view.frame)] : null;
    if (census) {
      ctx.globalAlpha = 0.4;
      ctx.font = "10px system-ui";
      ctx.textAlign = "left";
      // Label the biggest groups first and skip labels that would land on one already drawn; hover names the rest.
      const labelled = [];
      for (const g of [...census.enemy].sort((a, b) => b.count - a.count)) {
        const [x, y] = [px(g.x), pz(g.z)];
        glyph(ctx, g.class, x, y, COLOR.theirs, 1.3);
        if (labelled.some(([lx, ly]) => Math.abs(lx - x) < 70 && Math.abs(ly - y) < 11)) continue;
        labelled.push([x, y]);
        ctx.fillStyle = COLOR["ink-2"];
        ctx.fillText(`${g.name} x${g.count}`, x + 8, y);
      }
      ctx.globalAlpha = 1;
    }

    ctx.strokeStyle = COLOR.llm;
    ctx.fillStyle = COLOR.llm;
    ctx.font = "11px system-ui";
    ctx.textAlign = "center";
    for (const post of view.posts) {
      if (post.f > view.frame || post.until <= view.frame) continue;
      ctx.setLineDash([4, 3]);
      ctx.beginPath(); ctx.arc(px(post.x), pz(post.z), post.radius * scale, 0, 7); ctx.stroke();
      ctx.setLineDash([]);
      ctx.fillText(post.name, px(post.x), pz(post.z) - post.radius * scale - 6);
    }

    const intent = view.layers.intent ? match.intents[WR.indexAt(match.intents, view.frame)] : null;
    if (intent) {
      const mark = (at, label, color) => {
        if (!at) return;
        const [x, y] = [px(at[0]), pz(at[1])];
        ctx.strokeStyle = color;
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        ctx.moveTo(x - 8, y); ctx.lineTo(x + 8, y); ctx.moveTo(x, y - 8); ctx.lineTo(x, y + 8);
        ctx.stroke();
        ctx.fillStyle = COLOR["ink-2"];
        ctx.textAlign = "left";
        ctx.fillText(label, x + 9, y - 8);
      };
      mark(intent.home, "home", COLOR.ours);
      mark(intent.enemy_start, "enemy (presumed)", COLOR.theirs);
      mark(intent.station, "station", COLOR.ours);
      mark(intent.staging, "staging", COLOR.warning);
      mark(intent.target, "target", COLOR.critical);
    }

    const state = WR.stateAt(match, view.frame);
    view.state = state;
    const byId = new Map(state.own.map((u) => [u.id, u]));

    // The pianist's names at the playhead: places, parties, groups and where each group is going.
    const call = view.layers.pianist && match.jev ? match.jev.calls[WR.indexAt(match.jev.calls, view.frame)] : null;
    if (call && view.frame - call.f < 60 * WR.FPS) {
      ctx.font = "10px system-ui";
      ctx.textAlign = "left";
      ctx.textBaseline = "middle";
      ctx.fillStyle = COLOR.muted;
      for (const p of call.places) {
        if (p.name === "home" || p.name === "enemy_base") continue;
        ctx.fillText(p.name.replace("spot_", "#").replace("passage_", "pass "), px(p.x) + 7, pz(p.z) - 7);
      }
      ctx.strokeStyle = COLOR.theirs;
      ctx.fillStyle = COLOR.theirs;
      for (const p of call.parties) {
        ctx.setLineDash([3, 3]);
        ctx.beginPath(); ctx.arc(px(p.x), pz(p.z), 12, 0, 7); ctx.stroke();
        ctx.setLineDash([]);
        ctx.fillText(`${p.name} ${p.composition || ""}`, px(p.x) + 14, pz(p.z) - 12);
      }
      ctx.font = "bold 11px system-ui";
      for (const g of call.groups) {
        const members = g.members.map((id) => byId.get(id)).filter(Boolean);
        const at = members.length ? [members.reduce((a, u) => a + u.x, 0) / members.length, members.reduce((a, u) => a + u.z, 0) / members.length] : g.at;
        if (!at) continue;
        const [x, y] = [px(at[0]), pz(at[1])];
        const kind = g.task?.kind || "hold";
        if (g.task?.to && kind !== "hold") {
          ctx.strokeStyle = kind === "engage" ? COLOR.critical : kind === "fight_to" ? COLOR.warning : COLOR.jev;
          ctx.lineWidth = 1.5;
          ctx.setLineDash(kind === "move_to" ? [4, 3] : []);
          ctx.beginPath(); ctx.moveTo(x, y); ctx.lineTo(px(g.task.to[0]), pz(g.task.to[1])); ctx.stroke();
          ctx.setLineDash([]);
        }
        ctx.fillStyle = COLOR.jev;
        ctx.fillText(`${g.name}${members.length > 1 ? ` (${members.length})` : ""} ${kind === "hold" ? "" : kind}`, x + 8, y + 9);
      }
      ctx.font = "11px system-ui";
    }

    if (view.layers.orders && view.zoom < DETAIL_ZOOM) {
      ctx.lineWidth = 1;
      ctx.globalAlpha = 0.6;
      for (const record of WR.range(match.commands, view.frame - ORDER_FRAMES, view.frame)) {
        for (const c of record.c) {
          const unit = byId.get(c[1]);
          const to = c[0] === "build" ? c.slice(3, 5) : c.slice(2, 4);
          if (!unit || to.length < 2 || !ORDER_COLOR[c[0]]) continue;
          ctx.strokeStyle = ORDER_COLOR[c[0]];
          ctx.beginPath(); ctx.moveTo(px(unit.x), pz(unit.z)); ctx.lineTo(px(to[0]), pz(to[1])); ctx.stroke();
        }
      }
      ctx.globalAlpha = 1;
    }

    // Buildings under mobile units, enemies over ours so a raid in the base stays visible.
    const mobileLast = (a, b) => (view.match.defs[a.def]?.speed > 0) - (view.match.defs[b.def]?.speed > 0);
    // Allies first, under our own: the same icons in the allied colour. A team game recorded by one seat would
    // otherwise show half our side's map as empty.
    const detail = view.zoom >= DETAIL_ZOOM;
    if (detail) drawFootprints(ctx, state, px, pz, box);
    for (const u of [...state.allies].sort(mobileLast)) {
      ctx.globalAlpha = u.flags & WR.FLAG.beingBuilt ? 0.4 : 1;
      drawUnit(ctx, u, px(u.x), pz(u.z), ALLY, box);
    }
    for (const u of [...state.own].sort(mobileLast)) {
      ctx.globalAlpha = u.flags & WR.FLAG.beingBuilt ? 0.4 : 1;
      const color = u.flags & WR.FLAG.attacker ? OWN_ATTACKER : u.flags & WR.FLAG.squad ? COLOR.llm : COLOR.ours;
      drawUnit(ctx, u, px(u.x), pz(u.z), color, box);
      if (u.damage > 0) {
        ctx.globalAlpha = 1;
        ctx.strokeStyle = COLOR.critical;
        ctx.beginPath(); ctx.arc(px(u.x), pz(u.z), 8, 0, 7); ctx.stroke();
      }
    }
    ctx.globalAlpha = 1;
    for (const e of state.enemies) drawUnit(ctx, e, px(e.x), pz(e.z), COLOR.theirs, box);
    if (detail) drawLabels(ctx, state, px, pz, box);
    drawStandingOrders(ctx, state, byId, px, pz, box, detail);
    drawSelection(ctx, state, px, pz, box);

    if (view.layers.deaths) {
      ctx.lineWidth = 2;
      for (const e of WR.range(match.events, view.frame - DEATH_FRAMES, view.frame)) {
        if ((e.k !== "destroyed" && e.k !== "enemy_destroyed") || (e.x === 0 && e.z === 0)) continue;
        ctx.globalAlpha = 1 - (view.frame - e.f) / DEATH_FRAMES;
        ctx.strokeStyle = e.k === "destroyed" ? COLOR.ours : COLOR.theirs;
        const [x, y] = [px(e.x), pz(e.z)];
        ctx.beginPath();
        ctx.moveTo(x - 5, y - 5); ctx.lineTo(x + 5, y + 5); ctx.moveTo(x + 5, y - 5); ctx.lineTo(x - 5, y + 5);
        ctx.stroke();
      }
      ctx.globalAlpha = 1;
    }
  }

  /// The footprint of every building at high zoom: `footprint` in engine squares of 8 elmos, turned by the
  /// `finished` facing (an odd facing swaps the sides; enemies, whose facing is unknown, as built south); a
  /// factory's front edge, where its units leave, drawn heavier.
  function drawFootprints(ctx, state, px, pz, box) {
    const facing = WR.facings(view.match);
    const draw = (u, color) => {
      const d = u.def >= 0 ? view.match.defs[u.def] : null;
      if (!d || !d.footprint || d.speed > 0) return;
      const f = facing.get(u.id) ?? 0;
      const [fx, fz] = f % 2 ? [d.footprint[1], d.footprint[0]] : [d.footprint[0], d.footprint[1]];
      const [wx, wz] = [fx * 8 * box.scale, fz * 8 * box.scale];
      const [x, y] = [px(u.x) - wx / 2, pz(u.z) - wz / 2];
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.globalAlpha = 0.7;
      ctx.strokeRect(x, y, wx, wz);
      if (d.class === "factory") {
        ctx.lineWidth = 3;
        ctx.beginPath();
        if (f === 0) { ctx.moveTo(x, y + wz); ctx.lineTo(x + wx, y + wz); }
        else if (f === 1) { ctx.moveTo(x + wx, y); ctx.lineTo(x + wx, y + wz); }
        else if (f === 2) { ctx.moveTo(x, y); ctx.lineTo(x + wx, y); }
        else { ctx.moveTo(x, y); ctx.lineTo(x, y + wz); }
        ctx.stroke();
      }
      ctx.globalAlpha = 1;
      ctx.lineWidth = 1;
    };
    for (const u of state.own) draw(u, COLOR.ours);
    for (const u of state.allies) draw(u, ALLY);
    for (const u of state.enemies) draw(u, COLOR.theirs);
  }

  /// Names under the icons at high zoom, ids too from twice that.
  function drawLabels(ctx, state, px, pz, box) {
    ctx.font = "10px system-ui";
    ctx.textAlign = "center";
    ctx.textBaseline = "top";
    ctx.globalAlpha = 0.9;
    const label = (u, color) => {
      const d = u.def >= 0 ? view.match.defs[u.def] : null;
      const icon = d ? view.icons && view.icons[d.name] : null;
      const half = d && icon ? iconPixels(d, icon, box) / 2 : 6;
      ctx.fillStyle = color;
      ctx.fillText(view.zoom >= 2 * DETAIL_ZOOM ? `${d ? d.name : "?"} #${u.id}` : d ? d.name : "?", px(u.x), pz(u.z) + half + 1);
    };
    for (const u of state.own) label(u, COLOR["ink-2"]);
    for (const u of state.enemies) label(u, COLOR.theirs);
    ctx.globalAlpha = 1;
    ctx.textBaseline = "middle";
  }

  /// Every unit's standing order at high zoom, and the selected unit's at any zoom: a line to the point or the
  /// target unit, and at a build site the footprint of what is to stand there. An idle unit has none.
  function drawStandingOrders(ctx, state, byId, px, pz, box, all) {
    if (!view.layers.orders && view.selected == null) return;
    const orders = WR.ordersAt(view.match, view.frame);
    ctx.lineWidth = 1.2;
    for (const u of state.own) {
      if (!(all && view.layers.orders) && u.id !== view.selected) continue;
      if (u.flags & WR.FLAG.idle) continue;
      const o = orders.get(u.id);
      if (!o) continue;
      let to = null;
      if (o.x != null) to = [o.x, o.z];
      else if (o.target != null) { const t = byId.get(o.target); if (t) to = [t.x, t.z]; }
      if (!to) continue;
      ctx.strokeStyle = ORDER_COLOR[o.kind] || COLOR.muted;
      ctx.globalAlpha = u.id === view.selected ? 1 : 0.6;
      ctx.setLineDash(o.kind === "move" ? [4, 3] : []);
      ctx.beginPath(); ctx.moveTo(px(u.x), pz(u.z)); ctx.lineTo(px(to[0]), pz(to[1])); ctx.stroke();
      ctx.setLineDash([]);
      if (o.kind === "build" && o.def != null) {
        const d = view.match.defs[o.def];
        const side = d && d.footprint ? Math.max(d.footprint[0], d.footprint[1]) * 8 * box.scale : 6;
        ctx.strokeRect(px(to[0]) - side / 2, pz(to[1]) - side / 2, side, side);
      }
    }
    ctx.globalAlpha = 1;
  }

  function drawSelection(ctx, state, px, pz, box) {
    if (view.selected == null) return;
    const u = state.own.find((v) => v.id === view.selected) || state.enemies.find((v) => v.id === view.selected);
    if (!u) return;
    ctx.strokeStyle = COLOR.ink;
    ctx.lineWidth = 1.5;
    ctx.beginPath(); ctx.arc(px(u.x), pz(u.z), Math.max(10, 8 * Math.sqrt(box.zoom)), 0, 7); ctx.stroke();
  }

  // The wheel zooms about the cursor, a drag pans, a double-click (or 0) resets; a click selects the nearest unit.
  function pointOf(event) {
    const box = view.mapBox;
    const rect = $("map").getBoundingClientRect();
    const [mx, my] = [event.clientX - rect.left, event.clientY - rect.top];
    return { mx, my, x: (mx - box.x) / box.scale, z: (my - box.y) / box.scale };
  }
  function zoomAt(factor, mx, my) {
    const box = view.mapBox;
    if (!box) return;
    const zoom = Math.min(MAX_ZOOM, Math.max(1, view.zoom * factor));
    if (zoom === view.zoom) return;
    const [x, z] = [(mx - box.x) / box.scale, (my - box.y) / box.scale];
    const scale = box.fit * zoom;
    // The elmo under the cursor stays under the cursor.
    view.centre = [x - (mx - box.paneW / 2) / scale, z - (my - box.paneH / 2) / scale];
    view.zoom = zoom;
    drawMap();
  }
  function resetZoom() {
    view.zoom = 1;
    view.centre = null;
    drawMap();
  }
  let drag = null;
  function selectAt(event) {
    if (!view.state || !view.mapBox) return;
    const { x, z } = pointOf(event);
    const box = view.mapBox;
    let best = null;
    // Within 12 px, or anywhere on the unit's drawn icon.
    const consider = (u) => {
      const def = u.def >= 0 ? view.match.defs[u.def] : null;
      const icon = def ? iconOf(def.name) : null;
      const reach = Math.max(12, def && icon ? iconPixels(def, icon, box) / 2 : 0) / box.scale;
      const d = Math.hypot(u.x - x, u.z - z);
      if (d < reach && (!best || d < best.d)) best = { d, u };
    };
    view.state.own.forEach(consider);
    view.state.enemies.forEach(consider);
    select(best ? best.u.id : null);
  }
  /// Selecting a unit opens the Unit tab on it; clearing the selection leaves the tab.
  function select(id) {
    view.selected = id;
    $("tab-unit").hidden = id == null;
    if (id != null) setTab("unit");
    else if (view.tab === "unit") setTab(view.match && view.match.jev ? "pianist" : "decisions");
    else renderAll();
  }

  function mapHover(event) {
    const tip = $("tooltip");
    const box = view.mapBox;
    if (!box || !view.state) return;
    const rect = $("map").getBoundingClientRect();
    const [mx, my] = [event.clientX - rect.left, event.clientY - rect.top];
    const [x, z] = [(mx - box.x) / box.scale, (my - box.y) / box.scale];
    const map = view.match.header.map;
    if (x < 0 || z < 0 || x > map.width || z > map.height) return void (tip.hidden = true);
    const reach = 10 / box.scale;
    let best = null;
    const consider = (u, ours) => {
      const d = Math.hypot(u.x - x, u.z - z);
      if (d < reach && (!best || d < best.d)) best = { d, u, ours };
    };
    view.state.own.forEach((u) => consider(u, true));
    view.state.enemies.forEach((u) => consider(u, false));
    const lines = [`${WR.gridName(view.match, x, z)}  (${Math.round(x)}, ${Math.round(z)})`];
    if (view.terrain) {
      const t = view.terrain;
      const cx = Math.min(t.width - 1, Math.max(0, Math.floor(x / t.cell))), cz = Math.min(t.height - 1, Math.max(0, Math.floor(z / t.cell)));
      const h = t.heights[cz * t.width + cx];
      const metal = t.metalValues ? t.metalValues[cz * t.width + cx] : 0;
      lines.push(`${h < 0 ? `water, ${-h} deep` : `height ${h}`}${metal ? `, metal ${metal}` : ""}`);
    }
    if (best) {
      const { u, ours } = best;
      const flags = [];
      if (u.flags & WR.FLAG.beingBuilt) flags.push("being built");
      if (u.flags & WR.FLAG.idle) flags.push("idle");
      if (u.flags & WR.FLAG.attacker) flags.push("attacker");
      if (u.flags & WR.FLAG.squad) flags.push("squad");
      lines.push(`${ours ? "ours" : "enemy"}: ${defName(u.def)} #${u.id}`, ours ? `health ${u.health}%` : `health ${u.health}`);
      if (flags.length) lines.push(flags.join(", "));
    }
    const truthNow = view.layers.truth ? view.match.truth[WR.indexAt(view.match.truth, view.frame)] : null;
    if (truthNow) {
      const near = truthNow.units.filter((u) => Math.hypot(u.x - x, u.z - z) < reach);
      const names = new Map();
      for (const u of near) names.set(u.name, (names.get(u.name) || 0) + 1);
      if (near.length) lines.push(`theirs (truth): ${[...names].map(([n, k]) => `${n} x${k}`).join(", ")}${near.length === 1 ? `, health ${near[0].health}%` : ""}`);
    }
    const census = view.layers.census && !truthNow ? view.match.census[WR.indexAt(view.match.census, view.frame)] : null;
    const groups = census ? census.enemy.filter((g) => Math.hypot(g.x - x, g.z - z) < reach * 1.5) : [];
    if (groups.length) lines.push(`census ${WR.clock(census.f)} (mean positions): ${groups.map((g) => `${g.name} x${g.count}`).join(", ")}`);
    tip.textContent = lines.join("\n");
    tip.hidden = false;
    tip.style.left = `${Math.min(mx + 14, rect.width - 180)}px`;
    tip.style.top = `${my + 14}px`;
  }

  // ---------------------------------------------------------------- timeline

  const GUTTER = 230; // room for the pianist lanes' counts ("pianist: builders (42 picked, 0 by code)")
  const LANE_H = 14;
  const CHART_H = 38;
  const LANES = [
    ["losses", "unit losses", "ours"], ["buildingLosses", "building losses", "ours"], ["extractorLosses", "extractor losses", "ours"],
    ["kills", "kills", "theirs"], ["waves", "waves / recalls", "ink"], ["turns", "LLM turns", "llm"],
    ["jevBuilders", "pianist: builders", "jev"], ["jevLabs", "pianist: labs", "jev"], ["jevGroups", "pianist: army", "jev"],
  ];
  // Every chart has its own zero-based axis; "theirs" comes from the census and exists once a minute at most.
  const CHARTS = [
    { label: "metal income", ours: "metalIncome" },
    { label: "energy income", ours: "energyIncome" },
    { label: "extractors", ours: "extractors", theirs: "enemyExtractors" },
    { label: "army", ours: "army", theirs: "enemyArmy" },
  ];

  function timelineGeometry(w) {
    const last = Math.max(view.match.lastFrame, 1);
    return { x0: GUTTER, x1: w - 12, last, fx: (f) => GUTTER + ((w - 12 - GUTTER) * f) / last };
  }

  function drawTimeline() {
    const { ctx, w } = fitCanvas($("timeline"));
    const match = view.match;
    if (!match) return;
    const g = timelineGeometry(w);
    ctx.font = "11px system-ui";
    ctx.textBaseline = "middle";
    let y = 6;

    for (const [key, label, color] of LANES) {
      const items = view.lanes[key];
      if ((key === "turns" || key.startsWith("jev")) && !items.length) continue;
      ctx.fillStyle = COLOR["ink-2"];
      ctx.textAlign = "right";
      // The pianist's lanes count the pick's plays apart from code's (rule and list): who moved the actor.
      const jevLane = key.startsWith("jev");
      const picked = jevLane ? items.filter((d) => d.source === "plan" || d.source === "jev").length : 0;
      ctx.fillText(jevLane ? `${label} (${picked} picked, ${items.length - picked} by code)` : `${label} (${items.length})`, GUTTER - 8, y + LANE_H / 2);
      ctx.fillStyle = COLOR["surface-2"];
      ctx.fillRect(g.x0, y + 1, g.x1 - g.x0, LANE_H - 2);
      for (const item of items) {
        ctx.fillStyle = jevLane && !(item.source === "plan" || item.source === "jev") ? COLOR["ink-2"] : COLOR[color];
        ctx.globalAlpha = item.faint ? 0.25 : 1;
        ctx.fillRect(Math.round(g.fx(item.f)) - 1, y + 2, 2, LANE_H - 4);
      }
      ctx.globalAlpha = 1;
      y += LANE_H;
    }
    y += 4;

    // Folded: the lanes and the axis only.
    const compact = document.body.dataset.compact === "1";
    if (compact) view.chartRows = [];

    // Legend, once, for the two-series charts below.
    if (!compact) {
    ctx.textAlign = "left";
    ctx.fillStyle = COLOR["ink-2"];
    ctx.strokeStyle = COLOR.ours;
    ctx.lineWidth = 2;
    ctx.beginPath(); ctx.moveTo(g.x0, y + 6); ctx.lineTo(g.x0 + 18, y + 6); ctx.stroke();
    ctx.fillText("ours (bot's view)", g.x0 + 24, y + 6);
    ctx.fillStyle = COLOR.theirs;
    ctx.beginPath(); ctx.arc(g.x0 + 140, y + 6, 4, 0, 7); ctx.fill();
    ctx.fillStyle = COLOR["ink-2"];
    ctx.fillText(match.truth.length ? "theirs (ground truth)" : match.census.length ? "theirs (census, once a minute)" : "theirs: no census in this match", g.x0 + 150, y + 6);
    y += 14;

    view.chartRows = [];
    for (const chart of CHARTS) {
      const top = y + 4;
      const bottom = y + CHART_H - 2;
      const theirs = chart.theirs ? match.theirs.map((c) => c[chart.theirs]) : [];
      const max = Math.max(1, ...match.series.map((s) => s[chart.ours]), ...theirs);
      const vy = (v) => bottom - ((bottom - top) * v) / max;
      ctx.strokeStyle = COLOR.line;
      ctx.lineWidth = 1;
      ctx.beginPath(); ctx.moveTo(g.x0, bottom + 0.5); ctx.lineTo(g.x1, bottom + 0.5); ctx.stroke();
      ctx.fillStyle = COLOR["ink-2"];
      ctx.textAlign = "right";
      ctx.fillText(chart.label, GUTTER - 8, (top + bottom) / 2);
      ctx.fillStyle = COLOR.muted;
      ctx.fillText(String(Math.round(max)), GUTTER - 8, top + 2);

      ctx.strokeStyle = COLOR.ours;
      ctx.lineWidth = 2;
      ctx.lineJoin = "round";
      ctx.beginPath();
      // One point per horizontal pixel is all the canvas can show.
      const step = Math.max(1, Math.floor(match.series.length / (g.x1 - g.x0)));
      for (let i = 0; i < match.series.length; i += step) {
        const s = match.series[i];
        if (i === 0) ctx.moveTo(g.fx(s.f), vy(s[chart.ours]));
        else ctx.lineTo(g.fx(s.f), vy(s[chart.ours]));
      }
      ctx.stroke();
      if (chart.theirs && match.truth.length) {
        ctx.strokeStyle = COLOR.theirs;
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        match.truth.forEach((c, i) => (i === 0 ? ctx.moveTo(g.fx(c.f), vy(c[chart.theirs])) : ctx.lineTo(g.fx(c.f), vy(c[chart.theirs]))));
        ctx.stroke();
      } else if (chart.theirs) {
        for (const c of match.census) {
          ctx.fillStyle = COLOR.surface;
          ctx.beginPath(); ctx.arc(g.fx(c.f), vy(c[chart.theirs]), 5, 0, 7); ctx.fill();
          ctx.fillStyle = COLOR.theirs;
          ctx.beginPath(); ctx.arc(g.fx(c.f), vy(c[chart.theirs]), 3.5, 0, 7); ctx.fill();
        }
      }
      view.chartRows.push({ chart, top, bottom });
      y += CHART_H;
    }
    }

    // Time axis: a label every few minutes, however long the game.
    const minutes = g.last / (60 * WR.FPS);
    const every = [1, 2, 5, 10, 20].find((n) => minutes / n <= 12) || 30;
    ctx.fillStyle = COLOR.muted;
    ctx.textAlign = "center";
    for (let m = 0; m <= minutes; m += every) {
      const x = Math.round(g.fx(m * 60 * WR.FPS)) + 0.5;
      ctx.strokeStyle = COLOR.line;
      ctx.lineWidth = 1;
      ctx.beginPath(); ctx.moveTo(x, y); ctx.lineTo(x, y + 4); ctx.stroke();
      ctx.fillText(`${m}:00`, x, y + 12);
    }
    view.timelineBottom = y;

    const line = (frame, color) => {
      const x = Math.round(g.fx(frame)) + 0.5;
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.beginPath(); ctx.moveTo(x, 4); ctx.lineTo(x, y); ctx.stroke();
    };
    if (view.hoverFrame != null) line(view.hoverFrame, COLOR.muted);
    line(view.frame, COLOR.ink);
  }

  function timelineFrame(event) {
    const rect = $("timeline").getBoundingClientRect();
    const g = timelineGeometry(rect.width);
    const t = (event.clientX - rect.left - g.x0) / (g.x1 - g.x0);
    return Math.round(Math.min(1, Math.max(0, t)) * g.last);
  }

  function timelineHover(event) {
    if (!view.match) return;
    const frame = timelineFrame(event);
    view.hoverFrame = frame;
    const match = view.match;
    const s = match.series[WR.indexAt(match.series, frame)];
    const c = match.theirs[WR.indexAt(match.theirs, frame)];
    const lines = [WR.clock(frame)];
    if (s) {
      lines.push(`metal +${s.metalIncome.toFixed(1)}   energy +${s.energyIncome.toFixed(0)}`);
      lines.push(`extractors ${s.extractors}${c ? ` / theirs ${c.enemyExtractors}` : ""}`);
      lines.push(`army ${s.army}${c ? ` / theirs ${c.enemyArmy}` : ""}`);
    }
    if (c && !match.truth.length) lines.push(`(census of ${WR.clock(c.f)})`);
    const near = (items) => WR.range(items, frame - 5 * WR.FPS, frame + 5 * WR.FPS);
    const lost = near(view.lanes.losses).concat(near(view.lanes.buildingLosses), near(view.lanes.extractorLosses));
    if (lost.length) lines.push(`lost: ${summarise(lost.map((e) => defName(e.d)))}`);
    const killed = near(view.lanes.kills);
    if (killed.length) lines.push(`killed: ${summarise(killed.map((e) => defName(e.d)))}`);
    for (const d of near(view.lanes.waves)) lines.push(decisionTitle(d));
    const plays = near(view.lanes.jevBuilders).concat(near(view.lanes.jevLabs), near(view.lanes.jevGroups)).sort((a, b) => a.f - b.f);
    for (const d of plays.slice(0, 8)) lines.push(`${WR.clock(d.f)} ${d.source}: ${decisionTitle(d)}`);
    if (plays.length > 8) lines.push(`... ${plays.length - 8} more plays`);
    const tip = $("chart-tooltip");
    tip.textContent = lines.join("\n");
    tip.hidden = false;
    const rect = $("timeline").getBoundingClientRect();
    const x = event.clientX - rect.left;
    tip.style.left = `${x > rect.width - 260 ? x - 250 : x + 14}px`;
    tip.style.bottom = "24px";
    if (event.buttons & 1) seek(frame);
    else drawTimeline();
  }

  function summarise(names) {
    const counts = new Map();
    for (const n of names) counts.set(n, (counts.get(n) || 0) + 1);
    return [...counts].map(([n, k]) => (k > 1 ? `${n} x${k}` : n)).join(", ");
  }

  // ---------------------------------------------------------------- panels

  function renderNow() {
    const match = view.match;
    const s = match.samples[WR.indexAt(match.samples, view.frame)];
    const series = match.series[WR.indexAt(match.series, view.frame)];
    const census = match.theirs[WR.indexAt(match.theirs, view.frame)];
    const now = $("now");
    now.textContent = "";
    if (!s) return;
    const stat = (label, value, theirs) => {
      const div = document.createElement("div");
      div.className = "stat";
      const b = document.createElement("b");
      b.textContent = value;
      if (theirs != null) {
        const small = document.createElement("small");
        small.textContent = ` / ${theirs}`;
        small.title = "theirs, from the last census";
        b.append(small);
      }
      const span = document.createElement("span");
      span.textContent = label;
      div.append(b, span);
      now.append(div);
    };
    stat(`metal (+${s.m[1].toFixed(1)} / -${s.m[2].toFixed(1)})`, Math.round(s.m[0]));
    stat(`energy of ${s.e[3]} (+${s.e[1].toFixed(0)} / -${s.e[2].toFixed(0)})`, Math.round(s.e[0]));
    stat("extractors", series.extractors, census?.enemyExtractors);
    stat("army", series.army, census?.enemyArmy);
    stat("builders", series.builders);
    stat("buildings", series.buildings);
    stat("enemies in view", series.enemiesVisible);
    // With a commander the game stands still during its turns, and a turn's wall time lands in this number too:
    // it measures the bot's own speed only in heuristic games.
    const turns = view.match.decisions.filter((d) => d.kind === "turn" && d.latency_ms != null && d.f <= view.frame);
    if (turns.length) {
      const seconds = turns.reduce((sum, d) => sum + d.latency_ms / 1000, 0);
      const sorted = turns.map((d) => d.latency_ms / 1000).sort((a, b) => a - b);
      stat("commander turns so far", turns.length);
      stat("thinking, median s a turn", sorted[Math.floor(sorted.length / 2)].toFixed(1));
      // 1.0 would mean it thinks for as long as the game runs: in a live game it would never catch up.
      stat("thinking per game second, s", (seconds / Math.max(1, view.frame / WR.FPS)).toFixed(2));
    } else {
      stat("slowest tick, ms", s.ms.toFixed(1));
    }
  }

  function renderRules() {
    if (view.tab !== "rules") return;
    const rules = WR.rulesInMinute(view.match, view.frame);
    const minute = Math.floor(view.frame / (60 * WR.FPS));
    $("rules-minute").textContent = `(minute ${minute})`;
    const box = $("rules");
    box.textContent = "";
    const max = Math.max(1, ...rules.map(([, n]) => n));
    for (const [rule, n] of rules.slice(0, 10)) {
      const row = document.createElement("div");
      row.className = `rule${rule.startsWith("D-") ? " directive" : ""}`;
      const name = document.createElement("span");
      name.textContent = rule;
      const bar = document.createElement("i");
      bar.style.width = `${(100 * n) / max}%`;
      const count = document.createElement("span");
      count.textContent = n;
      row.append(name, bar, count);
      box.append(row);
    }
    if (!rules.length) box.textContent = "none";
  }

  function decisionTitle(d) {
    const o = d.outputs || {};
    switch (d.kind) {
      case "wave": return `wave ${o.wave}: ${d.inputs.home_group} units to ${o.target.grid}${o.first_stop.grid !== o.target.grid ? `, staging at ${o.first_stop.grid}` : ""}`;
      case "recall": return `recall: ${d.inputs.intruders} enemies at the base, ${o.attackers_called_home} attackers called home`;
      case "assault": return `assault: ${d.inputs.gathered} of ${d.inputs.attackers} gathered, going in`;
      case "event": return String(o);
      case "turn": return d.inputs.wake ? `woken: ${d.inputs.wake}` : "turn";
      case "builder": case "lab": case "group": {
        // The one pass's plays (sources rule, plan, list): the state or step and what it did.
        if (o.played == null && (d.inputs?.state || d.inputs?.step)) return `${d.inputs.actor}: ${o.did || d.inputs.state || d.inputs.step}`;
        const kept = o.played !== o.choice;
        const params = [o.where_extractor && o.played === "extractor" ? o.where_extractor : null, o.where && /_to|_at|walk|split/.test(o.played) ? o.where : null, o.whom && o.played === "engage" ? o.whom : null, o.where_scout && o.played === "scout" ? o.where_scout : null].filter(Boolean);
        return `${d.inputs.actor}: ${o.played}${params.length ? ` ${params.join(" ")}` : ""}${kept ? ` (kept; it chose ${o.choice})` : ""}`;
      }
      case "global": return `global: ${Object.entries(o).map(([k, v]) => `${k.replace("global.", "")} ${Number(v).toFixed(2)}`).join(", ")}`;
      case "worlds": return `plan: ${o.split ? `split over ${o.split.length}` : `w${o.pick} at ${Number(o.confidence).toFixed(2)}`}${Array.isArray(o.changed) && o.changed.length ? `: ${o.changed.join("; ")}` : ", nothing changes"}`;
      default: return `${d.kind}: ${JSON.stringify(o)}`;
    }
  }

  const short = (value, limit) => {
    const text = typeof value === "string" ? value : JSON.stringify(value);
    return text.length > limit ? `${text.slice(0, limit)} ...` : text;
  };

  function el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text != null) node.textContent = text;
    return node;
  }

  function details(summary, text) {
    const d = el("details");
    d.append(el("summary", null, summary), el("pre", null, text));
    d.addEventListener("click", (e) => e.stopPropagation());
    return d;
  }

  function buildDecisionList() {
    const list = $("decisions");
    list.textContent = "";
    view.decisionItems = [];
    view.currentDecision = -2;
    const filters = $("decision-filters");
    filters.textContent = "";
    const hasJev = !!view.match.jev || view.match.decisions.some((d) => WR.PIANIST_SOURCES.has(d.source));
    const keys = hasJev ? ["heuristic", "llm", "event", "jev", "jevChangesOnly"] : ["heuristic", "llm", "event"];
    const words = { event: "brain events", jev: "pianist", jevChangesOnly: "changes only" };
    for (const key of keys) {
      const label = el("label");
      const box = el("input");
      box.type = "checkbox";
      box.checked = view.show[key];
      box.addEventListener("change", () => {
        view.show[key] = box.checked;
        buildDecisionList();
        renderDecisions();
      });
      label.append(box, ` ${words[key] || key}`);
      filters.append(label);
    }
    if (hasJev) {
      const select = el("select");
      const actors = [...new Set(view.match.decisions.filter((d) => WR.PIANIST_SOURCES.has(d.source) && d.inputs?.actor).map((d) => d.inputs.actor))].sort();
      select.append(new Option("every actor", ""));
      for (const a of actors) select.append(new Option(a, a));
      select.value = actors.includes(view.jevActor) ? view.jevActor : "";
      select.addEventListener("change", () => {
        view.jevActor = select.value;
        buildDecisionList();
        renderDecisions();
        renderPianist();
      });
      filters.append(select);
    }
    let shown = 0;
    const LIMIT = 4000;
    for (const d of view.match.decisions) {
      if (d.kind === "rules") continue;
      const jev = WR.PIANIST_SOURCES.has(d.source);
      const group = jev ? "jev" : d.kind === "event" ? "event" : d.source.startsWith("llm") ? "llm" : "heuristic";
      if (!view.show[group]) continue;
      if (jev) {
        const o = d.outputs || {};
        if (d.kind === "global") continue;
        if (view.jevActor && d.inputs?.actor !== view.jevActor) continue;
        if (view.show.jevChangesOnly && o.played != null && !WR.isChange(o)) continue;
      }
      if (++shown > LIMIT) break;
      const li = el("li", jev ? `jev${d.outputs?.played !== d.outputs?.choice ? " kept" : ""}` : d.source.startsWith("llm") ? "llm" : "heuristic");
      const head = el("div");
      head.append(el("span", "when", WR.clock(d.f)), el("span", d.kind === "turn" ? "wake" : null, decisionTitle(d)), el("span", "source", d.source));
      if (jev && d.outputs) {
        if (d.outputs.probability != null) head.append(el("span", "p", `p ${Number(d.outputs.probability).toFixed(2)} c ${Number(d.outputs.confidence).toFixed(2)}`));
        if (d.outputs.did && d.outputs.played != null) head.append(el("div", "did", d.outputs.did));
      }
      li.append(head);
      if (d.kind === "turn") {
        for (const text of d.outputs.said) li.append(el("div", "said", text));
        for (const call of d.outputs.calls) {
          const row = el("div", "call");
          row.append(el("b", null, call.tool), `(${short(call.arguments, 400)})`);
          const result = short(call.result, 160);
          row.append(el("span", "result", `  -> ${result}`));
          li.append(row);
          if (result.endsWith(" ...")) li.append(details(`full result of ${call.tool}`, JSON.stringify(call.result, null, 1)));
        }
        if (d.outputs.thinking.length) li.append(details("thinking", d.outputs.thinking.join("\n\n")));
        li.append(details("prompt", d.inputs.prompt));
        const meta = [];
        if (d.latency_ms != null) meta.push(`${(d.latency_ms / 1000).toFixed(1)} s wall`);
        if (d.cost != null) meta.push(`$${d.cost.toFixed(3)} list price`);
        if (d.outputs.stopped) meta.push(`stopped: ${d.outputs.stopped}`);
        if (meta.length) li.append(el("div", "source", meta.join(" · ")));
      }
      li.addEventListener("click", () => seek(d.f));
      list.append(li);
      view.decisionItems.push({ f: d.f, li });
    }
    if (shown > LIMIT) list.append(el("li", null, `... ${view.match.decisions.length - LIMIT} more; narrow the filters`));
    if (!view.decisionItems.length) list.append(el("li", null, "no decision records of the selected kinds"));
  }

  // ---------------------------------------------------------------- the pianist

  function setTab(name) {
    view.tab = name;
    for (const button of document.querySelectorAll("#tabs [data-tab]")) button.classList.toggle("active", button.dataset.tab === name);
    for (const tab of document.querySelectorAll(".tab")) tab.hidden = tab.dataset.tab !== name;
    try { localStorage.setItem("wr-tab", name); } catch (_) { /* no storage */ }
    view.callShown = null;
    view.currentDecision = -2;
    renderAll();
  }

  /// Each actor as the call at the playhead saw it, with its last decision; opened, its history and its entry.
  function renderPianist() {
    const jev = view.match.jev;
    if (!jev || view.tab !== "pianist") return;
    renderBuildOrder();
    const i = WR.indexAt(jev.calls, view.frame);
    const call = i >= 0 ? jev.calls[i] : null;
    $("pianist-summary").textContent = call ? `call ${i + 1} of ${jev.calls.length} at ${WR.clock(call.f)}` : "before the first call";
    const box = $("pianist-actors");
    box.textContent = "";
    if (!call) return;
    const actors = call.state.actors || {};
    const decisionLine = (d, withClock) => {
      const line = el("span", "hist");
      if (withClock) line.append(`${WR.clock(d.f)}  `);
      line.append(el("b", null, d.played));
      if (d.kept) line.append(el("span", "kept", ` kept (it chose ${d.choice})`));
      if (d.probability != null) line.append(` p ${Number(d.probability).toFixed(2)} c ${Number(d.confidence).toFixed(2)}`);
      if (d.source && d.source !== "jev") line.append(el("span", "source", ` ${d.source}`));
      if (d.did) line.append(` · ${d.did}`);
      return line;
    };
    for (const [name, raw] of Object.entries(actors)) {
      // An actor not asked in the call is one line in the state (H-HANDS-DIET), not an object.
      const entry = typeof raw === "string" ? { doing: raw } : raw;
      const selected = view.jevActor === name;
      const row = el("div", `actor${selected ? " selected" : ""}`);
      row.append(el("b", null, name), el("span", "doing", `${entry.doing || ""}${entry.enemies_near ? ` · ${entry.enemies_near}` : ""}${entry.under_fire ? " · UNDER FIRE" : ""}`));
      const history = jev.actors.get(name);
      const at = history ? WR.indexAt(history.decisions, view.frame) : -1;
      const last = at >= 0 ? history.decisions[at] : null;
      const line = el("div", "last");
      if (last) {
        line.append(decisionLine(last, false), el("span", "ago", `  ${WR.clock(last.f)}`));
      } else {
        line.append("not asked yet");
      }
      row.append(line);
      if (selected) {
        const more = el("div", "more");
        more.append(el("div", "meta", "its entry in the picture:"), el("pre", null, JSON.stringify(entry, null, 1)));
        if (history && at >= 0) {
          more.append(el("div", "meta", "its decisions up to now, newest first:"));
          for (const d of history.decisions.slice(Math.max(0, at - 11), at + 1).reverse()) {
            const item = el("div");
            item.append(decisionLine(d, true));
            more.append(item);
          }
        }
        more.addEventListener("click", (e) => e.stopPropagation());
        row.append(more);
      }
      row.addEventListener("click", () => {
        view.jevActor = selected ? "" : name;
        buildDecisionList();
        renderDecisions();
        renderPianist();
      });
      box.append(row);
    }
    renderPass();
    renderCall(call, i);
  }

  /// The pass at the playhead (logs of version 2): what was open, what the base world started, whether it asked
  /// (the gate's nouls per state as bars) or stood quiet, the worlds composed with the pick's probabilities, and the
  /// plan put in force.
  function renderPass() {
    const jev = view.match.jev;
    const box = $("pass");
    const title = $("pass-title");
    if (!jev || jev.version < 2 || !jev.passes.length) {
      box.hidden = title.hidden = true;
      return;
    }
    box.hidden = title.hidden = false;
    const at = WR.passAt(jev, view.frame);
    if (!at) {
      $("pass-summary").textContent = "before the first pass";
      box.textContent = "";
      return;
    }
    const { pass, index, gate, plan, pickCall } = at;
    const key = `${index}:${view.jevActor}`;
    const before = pass.played || [];
    const after = plan ? plan.played || [] : [];
    const summary = [WR.clock(pass.f)];
    if (before.length) summary.push(`${before.length} by ${[...new Set(before.map((d) => d.source))].join("/")} before the ask`);
    summary.push(pass.gate ? `asked ${pass.gate.length} nouls` : pass.quiet ? "quiet" : (pass.open || []).length ? "nothing open to ask" : "nothing open");
    if (plan) summary.push(`${plan.split ? `split pick over ${plan.split.length}` : `picked w${plan.pick} at ${Number(plan.confidence).toFixed(2)}`}${after.length ? `, ${after.length} started` : ", nothing changed"}`);
    $("pass-summary").textContent = summary.join(" · ");
    if (box.dataset.key === key) return;
    box.dataset.key = key;
    box.textContent = "";
    // The second in the order it ran: what happened, what code started before asking, what the gate asked, the
    // worlds and the pick, what the pick started. Numbered so the order reads at a glance (onepass-hard-4, 5:04:
    // a hunt started by rule before the ask read as the pick's "nothing changes").
    let step = 0;
    const section = (label, cls) => {
      const div = el("div", `step${cls ? ` ${cls}` : ""}`);
      div.append(el("div", "step-title", `${++step} · ${label}`));
      box.append(div);
      return div;
    };
    const plays = (parent, rows, none) => {
      if (!rows.length) {
        parent.append(el("div", "pass-line muted", none));
        return;
      }
      for (const d of rows) {
        const row = el("div", "pass-line play");
        row.append(el("span", "source", d.source), ` ${d.actor}: ${d.did || d.played}`);
        parent.append(row);
      }
    };
    const happened = section("this second");
    const facts = [];
    if (pass.events && pass.events.length) facts.push(`events: ${pass.events.join("; ")}`);
    if (pass.hunts && pass.hunts.length) facts.push(`hunts: ${pass.hunts.join("; ")}`);
    facts.push(`open: ${(pass.open || []).length ? pass.open.join(", ") : "nothing"}`);
    for (const f of facts) happened.append(el("div", "pass-line", f));
    const base = section("code started, before any ask (the rules' defaults, the lists' steps)", "base");
    plays(base, before, "nothing new: every actor keeps its course");
    const asked = section(pass.gate ? `the gate asked ${pass.gate.length} nouls` : pass.quiet ? "no ask" : "no ask: nothing open", "gate");
    if (pass.quiet) asked.append(el("div", "pass-line muted", pass.quiet));
    const flags = (gate && gate.flags) || {};
    if (pass.slots) {
      const slots = [...pass.slots].sort((a, b) => (b.name === view.jevActor) - (a.name === view.jevActor));
      for (const slot of slots) {
        const open = slot.states.some((st, i) => i !== slot.base && i !== 0 && !st.pair_only);
        if (!open && !(slot.kind || "").startsWith("threat")) continue;
        const block = el("div", `slot${slot.name === view.jevActor ? " selected" : ""}`);
        block.append(el("div", "id", `${slot.name} · ${slot.kind}${slot.idle ? " · idle" : ""}`));
        const answer = flags[`${slot.name}.answer`];
        const change = flags[`${slot.name}.change`];
        if (answer != null) block.append(bar("needs answering", answer, false, 1));
        if (change != null) block.append(bar("should change course", change, false, 1));
        slot.states.forEach((st, i) => {
          const rated = flags[st.id];
          const marks = [i === slot.base ? "base" : null, st.current ? "current" : null, st.default ? "default" : null, st.pair_only ? "pair only" : null].filter(Boolean);
          const row = el("div", `state${i === slot.base ? " base" : ""}`);
          const label = `${st.id.split(".").slice(1).join(".")}${marks.length ? ` (${marks.join(", ")})` : ""}`;
          if (rated != null) row.append(bar(label, rated, false, 1));
          else row.append(el("div", "ask", label));
          const told = flags[`${st.id}.told`];
          if (told != null) row.append(bar("told by the instructions", told, false, 1));
          row.append(el("div", "ask words", st.words));
          block.append(row);
        });
        asked.append(block);
      }
    }
    // The rebuilt hands (log version 3): every party is a question, and every actor has one menu of moves.
    if (pass.menus) {
      const parties = Object.keys(flags).filter((k) => k.endsWith(".answer")).sort();
      if (parties.length) {
        const block = el("div", "slot");
        block.append(el("div", "id", "enemy parties · does it need answering?"));
        for (const k of parties) block.append(bar(k.slice(0, -".answer".length), flags[k], false, 1));
        asked.append(block);
      }
      const menus = [...pass.menus].sort((a, b) => (b.name === view.jevActor) - (a.name === view.jevActor));
      for (const menu of menus) {
        if (menu.quiet && !menu.audit) continue;
        const block = el("div", `slot${menu.name === view.jevActor ? " selected" : ""}`);
        block.append(el("div", "id", `${menu.name} · ${menu.kind} · ${menu.course}${menu.idle ? " · idle" : ""}${menu.audit ? " · closed by the news layer, asked for its audit (not played)" : ""}`));
        const change = flags[`${menu.name}.change`];
        if (change != null) block.append(bar("should change course", change, false, 1));
        const moves = menu.moves.map((mv, i) => ({ mv, i, rated: flags[mv.id] })).sort((a, b) => (b.rated ?? -1) - (a.rated ?? -1) || a.i - b.i);
        for (const { mv, i, rated } of moves) {
          const row = el("div", `state${i === 0 ? " base" : ""}`);
          const label = `${mv.id.split(".").slice(1).join(".")}${i === 0 ? " (its course)" : ""}${mv.party ? ` (an answer to ${mv.party})` : ""}`;
          if (rated != null) row.append(bar(label, rated, false, 1));
          else row.append(el("div", "ask", label));
          const forbidden = flags[`${mv.id}.forbidden`];
          if (forbidden != null) row.append(bar("forbidden by the instructions", forbidden, false, 1));
          row.append(el("div", "ask words", mv.words));
          block.append(row);
        }
        asked.append(block);
      }
      if (pass.closed && pass.closed.length) asked.append(el("div", "pass-line muted", `closed by the news layer (a course and no news): ${pass.closed.join(", ")}`));
    }
    if (pass.gate) {
      const worlds = section(gate && gate.worlds ? `${gate.worlds.length} worlds${pickCall ? ", the pick's probabilities" : ", no pick"}` : "the gate opened nothing: no second call", "worlds");
      if (gate && gate.worlds) {
        const probs = pickCall ? (pickCall.answers["worlds.pick"] || {}).probabilities || {} : {};
        const top = Math.max(1e-6, ...Object.values(probs));
        gate.lines.forEach((text, i) => {
          const id = `w${i + 1}`;
          const picked = plan && plan.pick === i + 1;
          const row = el("div", `world${picked ? " picked" : ""}`);
          if (id in probs) row.append(bar(id, probs[id], picked, top));
          else row.append(el("div", "id", id));
          row.append(el("div", "ask words", text));
          // World 1 keeps what code started this second: said, so "nothing changes" reads right.
          if (i === 0 && before.length) row.append(el("div", "pass-line muted", `w1 keeps what code started this second: ${before.map((d) => `${d.actor} ${d.did || d.played}`).join("; ")}`));
          worlds.append(row);
        });
      }
      // The split pick (the `split` layer): per component its lines, the candidate and whether it was taken.
      if (gate && gate.components && plan && plan.split) {
        gate.components.forEach((c, j) => {
          const part = plan.split[j] || {};
          const block = section(`component ${j + 1}: ${c.actors.join(", ")}: ${part.taken ? `took w${part.candidate}` : part.candidate ? `kept w1 over w${part.candidate}` : "no answer"}${part.confidence != null ? ` at ${Number(part.confidence).toFixed(2)}` : ""}`, "worlds");
          c.lines.forEach((text, i) => {
            const row = el("div", `world${part.taken && part.candidate === i + 1 ? " picked" : ""}`);
            row.append(el("div", "id", `w${i + 1}`));
            row.append(el("div", "ask words", text));
            block.append(row);
          });
        });
      }
      const picked = section(plan ? (plan.split ? `the split pick over ${plan.split.length} components` : `the pick took w${plan.pick} at ${Number(plan.confidence).toFixed(2)}`) : "no pick", "plan");
      if (plan) plays(picked, after, "nothing changed: the plan is world 1's courses");
    }
  }

  /// The call at the playhead: what Jev was shown and what it answered; rebuilt when the call changes. With an actor
  /// selected, its questions come first.
  function renderCall(call, index) {
    $("call-summary").textContent = `${WR.clock(call.f)} · ${call.pick ? "the pick" : "the pre-pass"} · ${call.ms} ms · ${call.tokens.toLocaleString()} tokens · ${Object.keys(call.questions).length} questions${call.retries ? ` · ${call.retries} retries` : ""}`;
    const key = `${index}:${view.jevActor}`;
    if (view.callShown === key) return;
    view.callShown = key;
    const box = $("call");
    box.textContent = "";
    const questions = el("div", "questions");
    const picture = el("div", "picture");
    box.append(questions, picture);
    const played = new Map(call.played.map((d) => [d.actor, d]));
    const ordered = Object.entries(call.questions).sort(([a], [b]) => (b.startsWith(`${view.jevActor}.`) - a.startsWith(`${view.jevActor}.`)));
    for (const [id, q] of ordered) {
      const [actor, what] = id.split(".");
      const a = call.answers[id];
      const block = el("div", "q");
      block.append(el("div", "id", id));
      const ask = typeof q.instructions === "string" ? q.instructions : JSON.stringify(q.instructions);
      block.append(el("div", "ask", short(ask, 220)));
      if (!a) {
        block.append(el("div", "ask", "no answer"));
      } else if (a.type === "noul") {
        block.append(bar("yes", a.noul, false, 1));
      } else {
        const chosen = (what === "do" || what === "next") ? played.get(actor) : null;
        const ranked = Object.entries(a.probabilities || {}).sort((x, y) => y[1] - x[1]).slice(0, 6);
        for (const [option, p] of ranked) block.append(bar(option, p, chosen ? option === chosen.played : option === a.choice, ranked[0][1]));
        if (chosen?.kept) block.append(el("div", "ask", `kept its course: ${chosen.choice} did not beat continue by the margin`));
        const rest = Object.keys(a.probabilities || {}).length - ranked.length;
        if (rest > 0) block.append(el("div", "ask", `and ${rest} more under ${(ranked[ranked.length - 1][1]).toFixed(2)}`));
        const crit = q.criteria && typeof q.criteria === "object" && !Array.isArray(q.criteria) ? q.criteria : null;
        if (crit) block.append(details("the options as worded", Object.entries(crit).map(([k, v]) => `${k}: ${typeof v === "string" ? v : JSON.stringify(v)}`).join("\n")));
      }
      questions.append(block);
    }
    picture.append(el("div", "meta", `the picture Jev was shown (${call.model || "Jev"}):`));
    picture.append(details("instructions", call.instructions || "(none)"));
    for (const section of ["economy", "ours", "enemy", "actors", "places", "recent"]) {
      if (call.state[section] !== undefined) picture.append(details(section, JSON.stringify(call.state[section], null, 1)));
    }
    picture.append(details("rules", call.rules || "(none)"));
  }

  function bar(label, p, played, top) {
    const row = el("div", `bar${played ? " played" : ""}`);
    const fill = el("i");
    fill.style.width = `${Math.max(1, (100 * p) / Math.max(top, 1e-6))}%`;
    row.append(el("span", null, label), fill, el("span", null, Number(p).toFixed(2)));
    return row;
  }

  function buildPianistMinutes(jev) {
    const table = $("pianist-minutes");
    table.textContent = "";
    const head = el("tr");
    const v2 = jev.version >= 2;
    const columns = v2 ? ["minute", "calls", "median ms", "max ms", "tokens", "questions", "asked", "quiet", "picks", "w1", "rule", "plan", "list", "failed"] : ["minute", "calls", "median ms", "max ms", "tokens", "questions", "changes", "kept", "failed"];
    for (const h of columns) head.append(el("th", null, h));
    table.append(head);
    for (const r of WR.jevMinutes(jev)) {
      const tr = el("tr");
      const cells = v2 ? [r.minute, r.calls, r.medianMs, r.maxMs, r.tokens.toLocaleString(), r.questions, r.asks, r.quiet, r.picks, r.w1, r.rule, r.plan, r.list, r.errors] : [r.minute, r.calls, r.medianMs, r.maxMs, r.tokens.toLocaleString(), r.questions, r.changes, r.kept, r.errors];
      for (const v of cells) tr.append(el("td", null, String(v)));
      table.append(tr);
    }
  }

  /// Rebuilds the list for a match that has grown, keeping the opened details open and the scroll position.
  function rebuildDecisionList() {
    const list = $("decisions");
    const key = (d) => `${d.closest("li").querySelector(".when").textContent} ${d.querySelector("summary").textContent}`;
    const opened = new Set([...list.querySelectorAll("details[open]")].map(key));
    const scroll = list.scrollTop;
    const current = view.currentDecision;
    buildDecisionList();
    for (const d of list.querySelectorAll("details")) if (opened.has(key(d))) d.open = true;
    // Following the newest decision scrolls by itself (renderDecisions); otherwise stay where the reader was.
    if (!$("follow").checked) {
      view.currentDecision = current;
      list.scrollTop = scroll;
      view.decisionItems.forEach((item, i) => {
        item.li.classList.toggle("future", i > current);
        item.li.classList.toggle("current", i === current);
      });
    }
  }

  function renderDecisions() {
    if (view.tab !== "decisions") return;
    const items = view.decisionItems;
    const current = WR.indexAt(items, view.frame);
    if (current === view.currentDecision) return;
    view.currentDecision = current;
    items.forEach((item, i) => {
      item.li.classList.toggle("future", i > current);
      item.li.classList.toggle("current", i === current);
    });
    if (current >= 0) {
      // Scroll inside the list only; scrollIntoView would move the page too.
      const list = $("decisions");
      const li = items[current].li;
      list.scrollTop = li.offsetTop - list.offsetTop - 24;
    }
  }

  /// The selected unit: what it is, its standing order, its part in the pianist's picture, and its history.
  function renderUnit() {
    if (view.tab !== "unit" || view.selected == null || !view.match) return;
    const box = $("unit");
    box.textContent = "";
    const id = view.selected;
    const state = view.state || WR.stateAt(view.match, view.frame);
    const ours = state.own.find((u) => u.id === id);
    const u = ours || state.enemies.find((v) => v.id === id);
    const d = u && u.def >= 0 ? view.match.defs[u.def] : null;
    const history = WR.unitHistory(view.match, id);
    const born = history.events.find((e) => e.k === "created" && e.u === id);
    const dead = history.events.find((e) => (e.k === "destroyed" || e.k === "enemy_destroyed") && e.u === id);
    const title = el("h2", null, `${d ? d.name : "unit"} #${id} `);
    title.append(el("span", "muted", ours ? "ours" : u ? "enemy" : dead && dead.f <= view.frame ? `gone at ${WR.clock(dead.f)}` : "not on the map at the playhead"));
    box.append(title);
    const facts = el("div", "facts");
    const fact = (k, v) => facts.append(el("span", "k", k), el("span", "v", v));
    if (d) fact("type", `${d.class}${d.metal ? `, ${d.metal} metal` : ""}${d.reach ? `, reach ${d.reach}` : ""}${d.footprint ? `, footprint ${d.footprint[0] * 8} x ${d.footprint[1] * 8}` : ""}`);
    if (u) {
      fact("at", `${WR.gridName(view.match, u.x, u.z)} (${Math.round(u.x)}, ${Math.round(u.z)})`);
      fact("health", ours ? `${u.health}%` : String(u.health));
      const flags = [];
      if (u.flags & WR.FLAG.beingBuilt) flags.push("being built");
      if (u.flags & WR.FLAG.idle) flags.push("idle");
      if (u.flags & WR.FLAG.attacker) flags.push("attacker");
      if (u.flags & WR.FLAG.squad) flags.push("commander's squad");
      if (flags.length) fact("flags", flags.join(", "));
      if (u.damage) fact("damage", `${u.damage} this second`);
    }
    if (born) fact("created", `${WR.clock(born.f)}${born.by != null ? ` by #${born.by}` : ""}`);
    if (dead) fact("destroyed", `${WR.clock(dead.f)}${dead.by_d != null && dead.by_d >= 0 ? ` by a ${defName(dead.by_d)}` : ""}`);
    if (ours) {
      const order = WR.ordersAt(view.match, view.frame).get(id);
      fact("standing order", u.flags & WR.FLAG.idle ? "none (idle)" : order ? orderWords(order) : "none recorded");
    }
    box.append(facts);
    // Its part in the pianist's picture at the playhead: a builder or a plant by its actor name, a soldier by its group.
    const jev = view.match.jev;
    const call = jev && ours ? jev.calls[WR.indexAt(jev.calls, view.frame)] : null;
    if (call) {
      const actors = call.state.actors || {};
      const own = d && d.class === "commander" ? "commander" : [`constructor_${id}`, `plant_${id}`, `lab_${id}`].find((n) => actors[n]);
      const group = call.groups.find((g) => (g.members || []).includes(id));
      const actor = own || (group && group.name);
      if (actor) {
        box.append(el("h2", "call-title", `in the picture at ${WR.clock(call.f)}: ${actor}`));
        const entry = actors[actor] || {};
        const picture = el("div", "picture");
        for (const key of ["doing", "list", "at", "enemies_near", "under_fire", "losses"]) if (entry[key] != null && entry[key] !== "") picture.append(el("div", null, `${key}: ${typeof entry[key] === "string" ? entry[key] : JSON.stringify(entry[key])}`));
        if (group) picture.append(el("div", null, `group of ${group.members.length}: ${group.task ? `${group.task.kind}${group.task.place ? ` ${group.task.place}` : ""}` : "hold"}`));
        box.append(picture);
        const hist = jev.actors.get(actor);
        const at = hist ? WR.indexAt(hist.decisions, view.frame) : -1;
        if (hist && at >= 0) {
          box.append(el("div", "meta", "its decisions up to now, newest first:"));
          for (const dec of hist.decisions.slice(Math.max(0, at - 7), at + 1).reverse()) {
            const line = el("div", "hist");
            line.append(`${WR.clock(dec.f)} `, el("b", null, dec.played), dec.did ? ` · ${dec.did}` : "");
            box.append(line);
          }
        }
        const open = el("button", null, "open in the Pianist tab");
        open.addEventListener("click", () => { view.jevActor = actor; buildDecisionList(); setTab("pianist"); });
        box.append(open);
      }
    }
    // Its history: orders and events, newest first, click to seek there.
    box.append(el("h2", "call-title", `history: ${history.commands.length} orders, ${history.events.length} events`));
    const list = el("ol", "history");
    const rows = [...history.commands.map((o) => ({ f: o.f, text: orderWords(o) })), ...history.events.map((e) => ({ f: e.f, text: eventWords(e, id) }))].sort((a, b) => b.f - a.f);
    for (const r of rows.slice(0, 80)) {
      const li = el("li", r.f > view.frame ? "future" : null);
      li.append(el("span", "when", WR.clock(r.f)), r.text);
      li.addEventListener("click", () => seek(r.f));
      list.append(li);
    }
    box.append(list);
  }
  function orderWords(o) {
    const at = o.x != null ? ` at ${WR.gridName(view.match, o.x, o.z)} (${Math.round(o.x)}, ${Math.round(o.z)})` : o.target != null ? ` #${o.target}` : o.feature != null ? ` feature ${o.feature}` : "";
    return `${o.kind}${o.def != null ? ` ${defName(o.def)}` : ""}${at}${o.radius ? ` within ${o.radius}` : ""}`;
  }
  function eventWords(e, id) {
    if (e.k === "created") return e.u === id ? `created${e.by != null ? ` by #${e.by}` : ""}` : `began a ${defName(e.d)} #${e.u} at ${WR.gridName(view.match, e.x, e.z)}`;
    if (e.k === "finished") return e.u === id ? "finished" : `finished #${e.u}`;
    if (e.k === "destroyed") return `destroyed${e.by_d != null && e.by_d >= 0 ? ` by a ${defName(e.by_d)}` : ""}`;
    return e.k;
  }

  /// The opening as a list: every unit begun in the first eight minutes, by whom and where; click to seek and select.
  function renderBuildOrder() {
    const box = $("build-order");
    const key = `${view.match.header.wall_start}:${view.match.events.length}`;
    if (box.dataset.key === key) return;
    box.dataset.key = key;
    box.textContent = "";
    for (const e of view.match.events) {
      if (e.k !== "created") continue;
      if (e.f > 8 * 60 * WR.FPS) break;
      const by = e.by != null ? WR.stateAt(view.match, e.f).own.find((v) => v.id === e.by) : null;
      const cls = by ? classOf(by.def) : null;
      const who = !by ? "?" : cls === "commander" ? "commander" : cls === "factory" ? `plant_${by.id}` : `${cls}_${by.id}`;
      const li = el("li");
      li.append(el("span", "when", WR.clock(e.f)), `${who}: ${defName(e.d)} #${e.u} at ${WR.gridName(view.match, e.x, e.z)}`);
      li.addEventListener("click", () => { seek(e.f + 1); select(e.u); });
      box.append(li);
    }
  }

  function renderBotLog() {
    if (view.tab !== "rules") return;
    const lines = WR.range(view.match.botLog, view.frame - 60 * WR.FPS, view.frame).slice(-30);
    $("botlog").textContent = lines.length ? lines.map((l) => `${WR.clock(l.f)}  ${l.text}`).join("\n") : view.match.botLog.length ? "" : "bot.log not loaded";
  }

  // ---------------------------------------------------------------- playhead

  function renderAll() {
    if (!view.match) return;
    $("clock").textContent = WR.clock(view.frame);
    drawMap();
    drawTimeline();
    renderNow();
    renderRules();
    renderDecisions();
    renderBotLog();
    renderPianist();
    renderUnit();
  }

  function seek(frame) {
    view.frame = Math.min(Math.max(0, frame), view.match ? view.match.lastFrame : 0);
    renderAll();
  }

  function setPlaying(playing) {
    view.playing = playing && !!view.match;
    $("play").textContent = view.playing ? "Pause" : "Play";
    if (view.playing) {
      if (view.frame >= view.match.lastFrame) view.frame = 0;
      view.lastTime = performance.now();
      requestAnimationFrame(step);
    }
  }

  function step(now) {
    if (!view.playing) return;
    const elapsed = Math.min(0.1, (now - view.lastTime) / 1000);
    view.lastTime = now;
    seek(view.frame + elapsed * WR.FPS * view.speed);
    if (view.frame >= view.match.lastFrame) return setPlaying(false);
    requestAnimationFrame(step);
  }

  for (const button of document.querySelectorAll("#tabs [data-tab]")) button.addEventListener("click", () => setTab(button.dataset.tab));
  const setWide = (wide) => {
    document.body.dataset.wide = wide ? "1" : "";
    try { localStorage.setItem("wr-wide", wide ? "1" : ""); } catch (_) { /* no storage */ }
    renderAll();
  };
  $("wide").addEventListener("click", () => setWide(document.body.dataset.wide !== "1"));
  const setCompact = (compact) => {
    document.body.dataset.compact = compact ? "1" : "";
    try { localStorage.setItem("wr-compact", compact ? "1" : ""); } catch (_) { /* no storage */ }
    renderAll();
  };
  $("compact").addEventListener("click", () => setCompact(document.body.dataset.compact !== "1"));
  try { if (localStorage.getItem("wr-compact") === "1") document.body.dataset.compact = "1"; } catch (_) { /* no storage */ }
  try { if (localStorage.getItem("wr-wide") === "1") document.body.dataset.wide = "1"; } catch (_) { /* no storage */ }
  $("play").addEventListener("click", () => setPlaying(!view.playing));
  // Going anywhere by hand stops following the live match; ticking the box again jumps back to the newest sample.
  const leaveLive = () => ($("follow").checked = false);
  $("follow").addEventListener("change", (e) => e.target.checked && view.match && seek(view.match.lastFrame));
  $("play").addEventListener("click", leaveLive);
  $("back").addEventListener("click", leaveLive);
  $("timeline").addEventListener("mousedown", leaveLive);
  $("back").addEventListener("click", () => seek(view.frame - 10 * WR.FPS));
  $("forward").addEventListener("click", () => seek(view.frame + 10 * WR.FPS));
  $("forward").addEventListener("click", leaveLive);
  $("speed").addEventListener("change", (e) => (view.speed = Number(e.target.value)));
  $("files").addEventListener("change", (e) => openFiles(e.target.files));
  $("timeline").addEventListener("mousedown", (e) => view.match && seek(timelineFrame(e)));
  $("timeline").addEventListener("mousemove", timelineHover);
  $("timeline").addEventListener("mouseleave", () => {
    view.hoverFrame = null;
    $("chart-tooltip").hidden = true;
    drawTimeline();
  });
  $("map").addEventListener("mousemove", mapHover);
  $("map").addEventListener("mouseleave", () => ($("tooltip").hidden = true));
  $("map").addEventListener("wheel", (e) => {
    e.preventDefault();
    const { mx, my } = pointOf(e);
    zoomAt(Math.exp(-e.deltaY * 0.0015), mx, my);
  }, { passive: false });
  $("map").addEventListener("mousedown", (e) => {
    if (e.button !== 0 || !view.mapBox) return;
    drag = { mx: e.clientX, my: e.clientY, centre: [...view.centre], moved: false };
  });
  window.addEventListener("mousemove", (e) => {
    if (!drag || !view.mapBox) return;
    const [dx, dy] = [e.clientX - drag.mx, e.clientY - drag.my];
    if (!drag.moved && Math.hypot(dx, dy) < 3) return;
    drag.moved = true;
    view.centre = [drag.centre[0] - dx / view.mapBox.scale, drag.centre[1] - dy / view.mapBox.scale];
    drawMap();
  });
  // A drag ends on the window (the cursor may leave the map); a click that did not drag selects.
  let dragged = false;
  window.addEventListener("mouseup", () => {
    if (!drag) return;
    dragged = drag.moved;
    drag = null;
  });
  $("map").addEventListener("click", (e) => {
    if (dragged) return void (dragged = false);
    selectAt(e);
  });
  $("map").addEventListener("dblclick", (e) => {
    e.preventDefault();
    resetZoom();
  });
  for (const box of document.querySelectorAll("#layers input")) {
    box.addEventListener("change", () => {
      view.layers[box.dataset.layer] = box.checked;
      drawMap();
    });
  }
  document.addEventListener("keydown", (e) => {
    if (e.target.tagName === "INPUT" || e.target.tagName === "SELECT") return;
    if (e.key === " ") setPlaying(!view.playing);
    else if (e.key === "ArrowLeft") seek(view.frame - (e.shiftKey ? 60 : 10) * WR.FPS);
    else if (e.key === "ArrowRight") seek(view.frame + (e.shiftKey ? 60 : 10) * WR.FPS);
    else if (e.key === "w") return setWide(document.body.dataset.wide !== "1");
    else if (e.key === "c") return setCompact(document.body.dataset.compact !== "1");
    else if (e.key === "+" || e.key === "=") return view.mapBox && zoomAt(1.5, view.mapBox.paneW / 2, view.mapBox.paneH / 2);
    else if (e.key === "-") return view.mapBox && zoomAt(1 / 1.5, view.mapBox.paneW / 2, view.mapBox.paneH / 2);
    else if (e.key === "0") return resetZoom();
    else if (e.key === "Escape") return select(null);
    else return;
    leaveLive();
    e.preventDefault();
  });
  new ResizeObserver(() => renderAll()).observe(document.body);

  boot();
})();
