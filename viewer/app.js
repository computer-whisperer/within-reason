// Within Reason match viewer: loading, the map, the timeline and the playhead. Parsing and the model are in
// record.js; the side panels (commander, player, hands, cost, unit, decisions, log) in panels.js.
//
// URL parameters: match=<base URL of the match files, default "match/">, t=<seconds>, seat=<team number>,
// bg=<image URL drawn under the map>. Without a server, use "Open files" and pick the records and their siblings.
// Every seat of ours in the match directory is loaded. A match without a result line is still being played: its
// files are polled for new bytes (run/view_match.py serves `?from=<offset>`), the new rows are added to the model,
// and with "follow live" ticked the playhead stays on the newest sample. The hands' logs (jev-<n>.jsonl) are never
// fetched whole: the server sends an index of each, and the gate and the picture at the playhead on demand.
"use strict";

const WV = (() => {
  const $ = (id) => document.getElementById(id);
  const css = (name) => getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  const COLOR = {};
  for (const name of ["surface", "surface-2", "line", "ink", "ink-2", "muted", "ours", "theirs", "good", "critical", "warning", "llm", "jev"]) COLOR[name] = css(`--${name}`);
  /// A colour a seat of ours: apart from each other on the dark ground, and none near the enemy's orange-red.
  const SEAT_COLORS = ["#3987e5", "#e0b23a", "#2fb8a6", "#b07ae8", "#7cc644", "#e88ac0", "#5cc8f0", "#c9c4b0"];
  const ALLY = "#8f8f86";
  const ORDER_COLOR = { build: COLOR.good, fight: COLOR.critical, move: COLOR.muted, attack: COLOR.critical, guard: COLOR.jev, repair: COLOR.good, reclaim: COLOR.warning, reclaim_feature: COLOR.warning, resurrect: COLOR.warning };
  /// Zoom from which buildings show their footprints and units their names; the map's zoom limit over the fitted scale.
  const DETAIL_ZOOM = 4;
  const MAX_ZOOM = 32;
  /// How long an order line and a death mark stay on the map, in frames.
  const ORDER_FRAMES = 3 * WR.FPS;
  const DEATH_FRAMES = 20 * WR.FPS;

  const view = {
    match: null, frame: 0, playing: false, speed: 10, lastTime: 0, live: null,
    layers: { terrain: true, nobots: true, notanks: false, metal: true, grid: true, spots: true, truth: true, orders: true, marks: true, deaths: true, hands: true },
    terrain: null, background: null, mapBox: null, hoverFrame: null, tab: null,
    /// The seat the panels, the stats and the charts are narrowed to (its index), or null for all seats.
    scope: null,
    /// The map's view: zoom over the fitted scale (1 to MAX_ZOOM) and the elmo at the pane's centre; the selected
    /// unit; the hands' actor in focus ({seat, name}); the game's icon table (viewer/icons.json).
    zoom: 1, centre: null, selected: null, actor: null, icons: null,
    /// What changes when rows arrive: the lanes and the timeline's picture are rebuilt when it does.
    version: 0,
  };
  window.viewer = view;
  // For the tests (viewer/test/browser.js), which drive the page from outside.
  view.seek = (frame) => seek(frame);
  view.select = (id) => select(id);
  view.gateAt = (seat, frame) => gateAt(view.match.seats[seat], frame);

  const seatColor = (index) => (index == null ? ALLY : view.match && view.match.seats.length > 1 ? SEAT_COLORS[index % SEAT_COLORS.length] : COLOR.ours);
  const inScope = (index) => view.scope == null || view.scope === index;

  function el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text != null) node.textContent = text;
    return node;
  }

  function status(text) {
    $("status").textContent = text;
    $("status").title = text;
  }

  // ---------------------------------------------------------------- loading

  async function fetchText(url) {
    try {
      const response = await fetch(url);
      return response.ok ? await response.text() : null;
    } catch {
      return null;
    }
  }

  async function fetchJson(url) {
    const text = await fetchText(url);
    if (text == null) return null;
    try { return JSON.parse(text); } catch { return null; }
  }

  /// A file of the match: how many of its bytes have been read, and the line the writer was half way through.
  const feeds = new Map();
  const POLL_MS = 3000;

  /// The complete lines a file has gained since the last pull; null when it has not grown.
  async function pullLines(base, name) {
    const feed = feeds.get(name) || { bytes: 0, rest: "", decoder: new TextDecoder() };
    let chunk, tail;
    try {
      const response = await fetch(`${base}${name}?from=${feed.bytes}`);
      if (!response.ok) return null;
      chunk = new Uint8Array(await response.arrayBuffer());
      // A server that does not know `from` (anything but run/view_match.py) sends the whole file every time.
      tail = response.headers.get("X-From") === String(feed.bytes);
    } catch {
      return null;
    }
    const fresh = tail ? chunk : chunk.subarray(feed.bytes);
    if (!fresh.length) return null;
    feed.bytes += fresh.length;
    const text = feed.rest + feed.decoder.decode(fresh, { stream: true });
    const cut = text.lastIndexOf("\n");
    feed.rest = text.slice(cut + 1);
    feeds.set(name, feed);
    return cut < 0 ? [] : text.slice(0, cut).split("\n");
  }

  function rowsOf(lines) {
    const rows = [];
    for (const line of lines || []) {
      if (!line) continue;
      try { rows.push(JSON.parse(line)); } catch { /* a torn line of a killed match */ }
    }
    return rows;
  }

  const numbered = (files, pattern) => files.filter((f) => pattern.test(f)).sort((a, b) => Number(/\d+/.exec(a)[0]) - Number(/\d+/.exec(b)[0]));

  /// Rows into the model: `sources` gives the new rows of each file by name. Seats are made as their headers
  /// arrive; the teams of all of them are known before any sample is added (their ally rows are dropped).
  function addRows(live, sources) {
    const match = view.match || WR.newMatch();
    const records = [...sources].filter(([name]) => /^record-\d+\.jsonl$/.test(name));
    for (const [, rows] of records) for (const r of rows) if (r.t === "header" && r.team != null) live.teams.add(r.team);
    for (const [name, rows] of records) {
      let seat = live.seats.get(name);
      if (!seat) {
        if (!rows.some((r) => r.t === "header")) continue;
        seat = WR.newSeat();
        live.seats.set(name, seat);
      }
      const fresh = !seat.header;
      WR.addRecordRows(seat, rows, live.teams);
      if (fresh && seat.header) WR.addSeat(match, seat);
    }
    if (!match.seats.length) return false;
    for (const [name, rows] of sources) {
      const player = /^strategist-(\d+)\.jsonl$/.exec(name);
      if (player) {
        const ai = Number(player[1]);
        if (!match.players.has(ai)) match.players.set(ai, WR.newTranscript("player"));
        WR.addTranscriptRows(match.players.get(ai), rows);
      } else if (name === "commander.jsonl") {
        if (!match.commander) match.commander = WR.newTranscript("commander");
        WR.addTranscriptRows(match.commander, rows);
      } else if (/^truth-\d+\.jsonl$/.test(name)) {
        WR.addTruthRows(match, rows);
      }
    }
    WR.settle(match);
    match.realtime = WR.inferRealtime(match);
    view.match = match;
    view.version++;
    return true;
  }

  /// Pulls every file of the match being served; true when anything grew.
  async function pullMatch(live) {
    const listing = (await fetchJson(`${live.base}index.json`)) || { files: [] };
    const files = listing.files || [];
    live.files = files;
    showReplay(live.base, listing.replays || []);
    // The seats of one game write the same truth file (one ally team sees one enemy): one is read.
    const truth = numbered(files, /^truth-\d+\.jsonl$/)[0];
    const names = [...numbered(files, /^record-\d+\.jsonl$/), ...numbered(files, /^strategist-\d+\.jsonl$/), ...(files.includes("commander.jsonl") ? ["commander.jsonl"] : []), ...(truth ? [truth] : [])];
    const pulled = await Promise.all(names.map(async (name) => [name, await pullLines(live.base, name)]));
    const sources = new Map(pulled.filter(([, lines]) => lines).map(([name, lines]) => [name, rowsOf(lines)]));
    const log = files.includes("bot.log") ? await pullLines(live.base, "bot.log") : null;
    let grew = sources.size > 0 && addRows(live, sources);
    if (view.match) {
      if (listing.batch && !view.match.batch) {
        view.match.batch = listing.batch;
        WR.settle(view.match);
        view.match.realtime = WR.inferRealtime(view.match);
      }
      view.match.batchResult = listing.result || null;
      if (log && log.length) {
        view.match.botLog.push(...WR.parseBotLog(log.join("\n")));
        grew = true;
      }
    }
    return grew;
  }

  /// The hands' logs: the server's index of each seat's (run/viewer_index.py), asked again as a live match grows.
  /// A big log's first read takes a few seconds: the status line says how far it is.
  async function pullJev(live) {
    if (!view.match || live.jevBusy) return;
    live.jevBusy = true;
    let changed = false;
    for (const seat of view.match.seats) {
      const name = `jev-${seat.header.ai_id}.jsonl`;
      if (!live.files.includes(name)) continue;
      for (;;) {
        const index = await fetchJson(`${live.base}${name}?index`);
        if (!index) break;
        if (!index.ready) {
          status(`reading ${name}: ${Math.round(100 * index.progress)}% (once; the index is kept beside the log)`);
          await new Promise((r) => setTimeout(r, 500));
          continue;
        }
        if (!seat.jev || seat.jev.index.size !== index.size) {
          // The last gate of a log still being written may gain its answer: asked for again.
          if (seat.jev) for (const key of [...jevCache.gate.keys()]) if (key.startsWith(`${seat.index}:`) && !jevCache.gate.get(key).complete) jevCache.gate.delete(key);
          seat.jev = { name, index };
          changed = true;
        }
        break;
      }
    }
    live.jevBusy = false;
    if (changed) {
      view.version++;
      describeMatch();
      showTabs();
      renderAll();
    }
  }

  /// What the hands' panels and the map ask of a seat's log, kept by the frame of the gate or the call: a gate is
  /// about a megabyte of words, so one request is out at a time and the playhead's own is asked last.
  const jevCache = { gate: new Map(), picture: new Map(), busy: new Set(), wanted: new Map() };
  function jevAsk(kind, seat, frames, frame) {
    if (!seat.jev || !view.live) return null;
    const i = WR.lastAt(frames, frame);
    if (i < 0) return null;
    const key = `${seat.index}:${frames[i]}`;
    const cache = jevCache[kind];
    if (cache.has(key)) return cache.get(key).value;
    const lane = `${kind}:${seat.index}`;
    jevCache.wanted.set(lane, { key, frame: frames[i] });
    if (!jevCache.busy.has(lane)) {
      jevCache.busy.add(lane);
      (async () => {
        while (jevCache.wanted.has(lane)) {
          const want = jevCache.wanted.get(lane);
          jevCache.wanted.delete(lane);
          if (cache.has(want.key)) continue;
          const body = await fetchJson(`${view.live.base}${seat.jev.name}?${kind}=${want.frame}`);
          if (!body) continue;
          const value = kind === "gate" ? (body.pass ? WR.gateView(body) : null) : body;
          cache.set(want.key, { value, complete: kind === "gate" ? body.next != null || !!view.match.result : true });
          if (cache.size > (kind === "gate" ? 12 : 96)) cache.delete(cache.keys().next().value);
        }
        jevCache.busy.delete(lane);
        renderAll();
      })();
    }
    return undefined; // on its way
  }
  /// The gate at the playhead of a seat's hands: its view, null when there is none yet, undefined while it loads.
  const gateAt = (seat, frame) => (seat.jev ? jevAsk("gate", seat, seat.jev.index.gates, frame) : null);
  /// The picture's names (groups, places, parties, actors) of a seat's last call at the playhead.
  const pictureAt = (seat, frame) => (seat.jev ? jevAsk("picture", seat, seat.jev.index.calls, frame) : null);

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
      const index = await fetchJson("matches/index.json");
      if (index && index.kind !== "match") return WV.panels.renderBrowser(index);
      base = "match/";
    }
    if (!base.endsWith("/")) base += "/";
    $("browse").hidden = false;
    const live = { base, files: [], seats: new Map(), teams: new Set() };
    view.live = live;
    status("loading the match");
    await pullMatch(live);
    if (!view.match) {
      if (live.files.length) return status(`No record-*.jsonl in the match directory (was it played with WITHIN_REASON_RECORD=1?). Files: ${live.files.join(", ")}`);
      return status("No match loaded. Start with run/view_match.py <match dir>, or use Open files.");
    }
    const seat = params.get("seat");
    if (seat != null) view.scope = view.match.seats.findIndex((s) => String(s.team) === seat.replace(/^t/, ""));
    if (view.scope < 0) view.scope = null;
    opened(Number(params.get("t") || 0) * WR.FPS);
    loadIcons();
    loadBackground(params.get("bg") || `maps/${encodeURIComponent(view.match.header.map.name)}.png`);
    loadTerrain(live.base, view.match.header.terrain);
    pullJev(live);
    if (!finished()) follow(live, !params.get("t"));
  }

  /// A match is over when a seat wrote its result line, or every seat its end line (a game with people has no
  /// result: the engine drops the connection).
  const finished = () => !!view.match.result || view.match.seats.every((s) => s.end);

  /// Keeps a match that is still being played up to date, until it is over.
  function follow(live, stayOnNewest) {
    $("follow-label").hidden = false;
    $("follow").checked = stayOnNewest;
    if (stayOnNewest) seek(view.match.lastFrame);
    let polls = 0;
    const timer = setInterval(async () => {
      if (live.pulling) return;
      live.pulling = true;
      const grew = await pullMatch(live);
      live.pulling = false;
      if (grew) {
        describeMatch();
        WV.panels.matchGrew();
        seek($("follow").checked ? Infinity : view.frame);
      }
      if (grew || polls++ % 4 === 0) pullJev(live);
      if (finished()) {
        clearInterval(timer);
        $("follow-label").hidden = true;
        pullJev(live);
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

  /// Files picked by hand, without the server: the records and their siblings. The hands' logs are not read this
  /// way (they are gigabytes; the server indexes them).
  async function openFiles(fileList) {
    const sources = new Map();
    let log = null;
    for (const file of fileList) {
      if (/^jev.*\.jsonl$/.test(file.name)) continue;
      const text = await file.text();
      if (file.name === "bot.log") log = text;
      else if (/\.jsonl$/.test(file.name)) sources.set(file.name, WR.parseLines(text).values);
    }
    view.match = null;
    view.live = null;
    const ok = [...sources.keys()].some((n) => /^record-\d+\.jsonl$/.test(n)) && addRows({ seats: new Map(), teams: new Set() }, sources);
    if (!ok) return status("Pick the match's record-*.jsonl files (and optionally strategist-*.jsonl, commander.jsonl, a truth-*.jsonl, bot.log).");
    if (log) view.match.botLog = WR.parseBotLog(log);
    opened(0);
  }

  /// The match is in the model: the page is set up around it.
  function opened(frame) {
    const match = view.match;
    view.frame = Math.min(frame, match.lastFrame);
    describeMatch();
    buildLegend();
    showTabs();
    WV.panels.matchOpened();
    // The tab last used, when this match has it; setTab renders everything.
    let saved = null;
    try { saved = localStorage.getItem("wr-tab"); } catch (_) { /* no storage */ }
    const has = (name) => name && name !== "unit" && !document.querySelector(`#tabs [data-tab="${name}"]`)?.hidden;
    setTab(has(saved) ? saved : has("commander") ? "commander" : has("player") ? "player" : "decisions");
    document.body.dataset.loaded = `${match.seats.length} seats, ${match.seats.reduce((n, s) => n + s.samples.length, 0)} samples`;
  }

  /// The tabs this match has something for.
  function showTabs() {
    const match = view.match;
    const show = (name, on) => { document.querySelector(`#tabs [data-tab="${name}"]`).hidden = !on; };
    const hands = match.seats.some((s) => s.jev) || (view.live && view.live.files.some((f) => /^jev-\d+\.jsonl$/.test(f)));
    show("commander", !!match.commander);
    show("player", match.players.size > 0);
    show("hands", !!hands);
    show("cost", !!hands);
  }

  function describeMatch() {
    const match = view.match;
    const { header, result } = match;
    const sides = WR.sides(match);
    const parts = [header.map.name, `${sides.ours} v ${sides.theirs}: ${sides.oursWords} against ${sides.theirsWords}`];
    const batch = match.batch;
    if (batch) {
      if (batch.player) parts.push(`player ${batch.player_model || "?"}${match.seats.length > 1 ? (batch.players === "seat" ? ", one a seat" : ", one for all seats") : ""}`);
      if (batch.commander) parts.push(`commander ${batch.commander}`);
      parts.push(batch.realtime ? "realtime" : "the game waits for turns");
    }
    const r = (result && result.result) || match.batchResult;
    if (r) parts.push(`${r.outcome}${r.called ? " (called)" : ""} after ${r.game_minutes.toFixed(1)} min${r.reason ? `: ${r.reason}` : ""}`);
    else parts.push(finished() ? "ended without a result line" : "being played");
    $("subtitle").textContent = parts.join("  ·  ");
    const notes = [`${match.seats.length} seat${match.seats.length > 1 ? "s" : ""} of ours loaded (${match.seats.map((s) => s.name).join(", ")})`];
    const restarts = match.seats.flatMap((s) => s.restarts.map((f) => `${s.name} at ${WR.clock(f)}`));
    if (restarts.length) notes.push(`bot restarted: ${restarts.join(", ")}`);
    const hands = match.seats.filter((s) => s.jev);
    if (hands.length) {
      const old = hands.filter((s) => s.jev.index.version < WR.JEV_VERSION);
      const cost = WR.costView(hands.map((s) => ({ seat: s.index, index: s.jev.index })));
      notes.push(`hands: ${hands.reduce((n, s) => n + s.jev.index.gates.length, 0)} gates, ${(cost.tokens / 1e6).toFixed(1)}M tokens, $${cost.dollars.toFixed(2)}${old.length ? ` (${old.map((s) => s.name).join(", ")}: a log older than version ${WR.JEV_VERSION}, not read)` : ""}`);
    }
    notes.push(match.truth.length ? "opponent ground truth" : "no truth file (play with WITHIN_REASON_OBSERVE=1 for the opponent's)");
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
    for (const seat of view.match.seats) entry("army", seatColor(seat.index), view.match.seats.length > 1 ? `seat ${seat.name}` : "ours");
    for (const cls of ["commander", "builder", "extractor", "factory", "turret", "building"]) entry(cls, COLOR["ink-2"], cls);
    entry("army", ALLY, "ally (not a seat of ours)");
    entry("army", COLOR.theirs, "enemy seen");
    entry("unknown", COLOR.theirs, "radar contact");
  }

  // ---------------------------------------------------------------- icons

  // The game's minimap icons (viewer/icons.json and viewer/icons/, exported by run/icons.py from BAR's
  // gamedata/icontypes.lua): white shapes, tinted here by seat as the game tints them by team.
  const iconImages = new Map();
  const tinted = new Map();
  async function loadIcons() {
    view.icons = await fetchJson("icons.json");
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
      if (tinted.size > 1200) tinted.clear();
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

  /// The player whose packet a seat's hands read: its own, or the one player of all seats.
  function playerOf(seat) {
    const players = view.match.players;
    return players.get(seat.header.ai_id) || (players.size === 1 ? [...players.values()][0] : null);
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

    const state = WR.stateAt(match, view.frame);
    view.state = state;
    const byId = new Map(state.own.map((u) => [u.id, u]));
    if (view.layers.marks) drawMarks(ctx, px, pz);
    if (view.layers.hands) drawHands(ctx, byId, px, pz);

    if (view.layers.orders && view.zoom < DETAIL_ZOOM) {
      ctx.lineWidth = 1;
      ctx.globalAlpha = 0.6;
      for (const seat of WR.scoped(match, view.scope)) {
        for (const record of WR.range(seat.commands, view.frame - ORDER_FRAMES, view.frame)) {
          for (const c of record.c) {
            const unit = byId.get(c[1]);
            const to = c[0] === "build" ? c.slice(3, 5) : c.slice(2, 4);
            if (!unit || to.length < 2 || !ORDER_COLOR[c[0]]) continue;
            ctx.strokeStyle = ORDER_COLOR[c[0]];
            ctx.beginPath(); ctx.moveTo(px(unit.x), pz(unit.z)); ctx.lineTo(px(to[0]), pz(to[1])); ctx.stroke();
          }
        }
      }
      ctx.globalAlpha = 1;
    }

    // Buildings under mobile units, enemies over ours so a raid in the base stays visible. Allies that are not a
    // seat of ours first, in grey; then every seat's units in its colour, the seats out of scope faint.
    const mobileLast = (a, b) => (match.defs[a.def]?.speed > 0) - (match.defs[b.def]?.speed > 0);
    const detail = view.zoom >= DETAIL_ZOOM;
    if (detail) drawFootprints(ctx, state, px, pz, box);
    for (const u of [...state.allies].sort(mobileLast)) {
      ctx.globalAlpha = u.flags & WR.FLAG.beingBuilt ? 0.4 : 1;
      drawUnit(ctx, u, px(u.x), pz(u.z), ALLY, box);
    }
    for (const u of [...state.own].sort(mobileLast)) {
      ctx.globalAlpha = (u.flags & WR.FLAG.beingBuilt ? 0.4 : 1) * (inScope(u.seat) ? 1 : 0.3);
      drawUnit(ctx, u, px(u.x), pz(u.z), seatColor(u.seat), box);
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
    drawSelection(ctx, state, byId, px, pz, box);

    if (view.layers.deaths) {
      ctx.lineWidth = 2;
      const drawn = new Set();
      for (const seat of match.seats) {
        for (const e of WR.range(seat.events, view.frame - DEATH_FRAMES, view.frame)) {
          if ((e.k !== "destroyed" && e.k !== "enemy_destroyed") || (e.x === 0 && e.z === 0) || drawn.has(e.u)) continue;
          drawn.add(e.u);
          ctx.globalAlpha = 1 - (view.frame - e.f) / DEATH_FRAMES;
          ctx.strokeStyle = e.k === "destroyed" ? seatColor(seat.index) : COLOR.theirs;
          const [x, y] = [px(e.x), pz(e.z)];
          ctx.beginPath();
          ctx.moveTo(x - 5, y - 5); ctx.lineTo(x + 5, y + 5); ctx.moveTo(x + 5, y - 5); ctx.lineTo(x - 5, y + 5);
          ctx.stroke();
        }
      }
      ctx.globalAlpha = 1;
    }
  }

  /// The places somebody named: the commander's marks (the whole side's), each player's own, and every seat's
  /// start with the enemy's presumed one.
  function drawMarks(ctx, px, pz) {
    const match = view.match;
    ctx.font = "11px system-ui";
    ctx.textAlign = "left";
    ctx.textBaseline = "middle";
    const cross = (x, z, label, color, size = 7) => {
      const [cx, cy] = [px(x), pz(z)];
      ctx.strokeStyle = color;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(cx - size, cy); ctx.lineTo(cx + size, cy); ctx.moveTo(cx, cy - size); ctx.lineTo(cx, cy + size);
      ctx.stroke();
      ctx.fillStyle = color;
      ctx.fillText(label, cx + size + 2, cy - size);
    };
    for (const seat of match.seats) {
      const intent = seat.intents[WR.indexAt(seat.intents, view.frame)];
      if (!intent) continue;
      if (intent.home && inScope(seat.index)) cross(intent.home[0], intent.home[1], match.seats.length > 1 ? `home ${seat.name}` : "home", seatColor(seat.index));
      if (intent.enemy_start && seat.index === 0) cross(intent.enemy_start[0], intent.enemy_start[1], "enemy (presumed)", COLOR.theirs);
    }
    const pin = (m, color, by) => {
      const [x, y] = [px(m.x), pz(m.z)];
      ctx.strokeStyle = color;
      ctx.fillStyle = color;
      ctx.lineWidth = 1.5;
      ctx.beginPath(); ctx.moveTo(x, y); ctx.lineTo(x, y - 14); ctx.stroke();
      ctx.beginPath(); ctx.moveTo(x, y - 14); ctx.lineTo(x + 9, y - 11); ctx.lineTo(x, y - 8); ctx.closePath(); ctx.fill();
      ctx.beginPath(); ctx.arc(x, y, 2, 0, 7); ctx.fill();
      ctx.fillText(by ? `${m.name} (${by})` : m.name, x + 11, y - 11);
    };
    if (match.commander) for (const m of WR.marksAt(match, match.commander, view.frame)) pin(m, COLOR.llm, null);
    const done = new Set();
    for (const seat of WR.scoped(match, view.scope)) {
      const player = playerOf(seat);
      if (!player || done.has(player)) continue;
      done.add(player);
      for (const m of WR.marksAt(match, player, view.frame)) pin(m, seatColor(seat.index), match.players.size > 1 ? seat.name : null);
    }
  }

  /// The hands' names at the playhead, per seat in scope: places (once each), enemy parties, groups and where each
  /// group is going. From the picture of the seat's last call (asked of the server; drawn when it has come).
  function drawHands(ctx, byId, px, pz) {
    const match = view.match;
    const placed = new Set();
    for (const seat of WR.scoped(match, view.scope)) {
      const picture = pictureAt(seat, view.frame);
      if (!picture || picture.f == null || view.frame - picture.f > 60 * WR.FPS) continue;
      ctx.font = "10px system-ui";
      ctx.textAlign = "left";
      ctx.textBaseline = "middle";
      ctx.fillStyle = COLOR.muted;
      for (const p of picture.places) {
        if (p.name === "home" || p.name === "enemy_base" || placed.has(p.name)) continue;
        placed.add(p.name);
        ctx.fillText(p.name.replace("spot_", "#").replace("passage_", "pass "), px(p.x) + 7, pz(p.z) - 7);
      }
      ctx.strokeStyle = COLOR.theirs;
      ctx.fillStyle = COLOR.theirs;
      ctx.lineWidth = 1;
      for (const p of picture.parties) {
        if (placed.has(`party:${p.name}`)) continue;
        placed.add(`party:${p.name}`);
        ctx.setLineDash([3, 3]);
        ctx.beginPath(); ctx.arc(px(p.x), pz(p.z), 12, 0, 7); ctx.stroke();
        ctx.setLineDash([]);
        ctx.fillText(`${p.name} ${(p.composition || "").length > 44 ? `${p.composition.slice(0, 42)} ...` : p.composition || ""}`, px(p.x) + 14, pz(p.z) - 12);
      }
      ctx.font = "bold 11px system-ui";
      const color = match.seats.length > 1 ? seatColor(seat.index) : COLOR.jev;
      for (const g of picture.groups) {
        const members = g.members.map((id) => byId.get(id)).filter(Boolean);
        const at = members.length ? [members.reduce((a, u) => a + u.x, 0) / members.length, members.reduce((a, u) => a + u.z, 0) / members.length] : g.at;
        if (!at) continue;
        const [x, y] = [px(at[0]), pz(at[1])];
        const kind = g.task?.kind || "hold";
        if (g.task?.to && kind !== "hold") {
          ctx.strokeStyle = kind === "engage" ? COLOR.critical : kind === "fight_to" ? COLOR.warning : color;
          ctx.lineWidth = 1.5;
          ctx.setLineDash(kind === "move_to" ? [4, 3] : []);
          ctx.beginPath(); ctx.moveTo(x, y); ctx.lineTo(px(g.task.to[0]), pz(g.task.to[1])); ctx.stroke();
          ctx.setLineDash([]);
        }
        ctx.fillStyle = color;
        ctx.fillText(`${g.name}${members.length > 1 ? ` (${members.length})` : ""} ${kind === "hold" ? "" : kind}`, x + 8, y + 9);
      }
    }
    ctx.font = "11px system-ui";
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
    for (const u of state.own) draw(u, seatColor(u.seat));
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
    const match = view.match;
    ctx.lineWidth = 1.2;
    for (const u of state.own) {
      if (!(all && view.layers.orders && inScope(u.seat)) && u.id !== view.selected) continue;
      if (u.flags & WR.FLAG.idle) continue;
      const o = WR.ordersAt(match.seats[u.seat], view.frame).get(u.id);
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
        const d = match.defs[o.def];
        const side = d && d.footprint ? Math.max(d.footprint[0], d.footprint[1]) * 8 * box.scale : 6;
        ctx.strokeRect(px(to[0]) - side / 2, pz(to[1]) - side / 2, side, side);
      }
    }
    ctx.globalAlpha = 1;
  }

  /// A ring on the selected unit, and a thinner one on every unit of the hands' actor in focus.
  function drawSelection(ctx, state, byId, px, pz, box) {
    const ring = (u, width) => {
      ctx.strokeStyle = COLOR.ink;
      ctx.lineWidth = width;
      ctx.beginPath(); ctx.arc(px(u.x), pz(u.z), Math.max(10, 8 * Math.sqrt(box.zoom)), 0, 7); ctx.stroke();
    };
    if (view.actor) for (const id of actorUnits(view.actor)) if (byId.has(id) && id !== view.selected) ring(byId.get(id), 1);
    if (view.selected == null) return;
    const u = byId.get(view.selected) || state.enemies.find((v) => v.id === view.selected);
    if (u) ring(u, 1.5);
  }

  /// The units of a hands' actor at the playhead: a builder's or a plant's own id, the commander, a group's members.
  function actorUnits(actor) {
    const seat = view.match.seats[actor.seat];
    if (!seat) return [];
    const commanders = (view.state ? view.state.own : []).filter((u) => u.seat === seat.index && classOf(u.def) === "commander").map((u) => u.id);
    return WR.unitsOfActor(pictureAt(seat, view.frame), actor.name, commanders);
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
    else if (view.tab === "unit") setTab("decisions");
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
      if (ours && u.flags & WR.FLAG.idle) flags.push("idle");
      const whose = ours ? (view.match.seats.length > 1 ? `ours (${view.match.seats[u.seat].name})` : "ours") : "enemy";
      lines.push(`${whose}: ${defName(u.def)} #${u.id}`, ours ? `health ${u.health}%` : `health ${u.health}`);
      if (flags.length) lines.push(flags.join(", "));
    }
    const truthNow = view.layers.truth ? view.match.truth[WR.indexAt(view.match.truth, view.frame)] : null;
    if (truthNow) {
      const near = truthNow.units.filter((u) => Math.hypot(u.x - x, u.z - z) < reach);
      const names = new Map();
      for (const u of near) names.set(u.name, (names.get(u.name) || 0) + 1);
      if (near.length) lines.push(`theirs (truth): ${[...names].map(([n, k]) => `${n} x${k}`).join(", ")}${near.length === 1 ? `, health ${near[0].health}%` : ""}`);
    }
    tip.textContent = lines.join("\n");
    tip.hidden = false;
    tip.style.left = `${Math.min(mx + 14, rect.width - 180)}px`;
    tip.style.top = `${my + 14}px`;
  }

  // ---------------------------------------------------------------- timeline

  const GUTTER = 200;
  const LANE_H = 12;
  const CHART_H = 32;
  // Every chart has its own zero-based axis. With all seats a chart is the seats stacked (the top edge is the
  // side's sum); with one seat, its line against the other seats' faint ones. "theirs" is the ground truth.
  const CHARTS = [
    { label: "metal income", key: "metalIncome" },
    { label: "energy income", key: "energyIncome" },
    { label: "extractors", key: "extractors", theirs: "extractors" },
    { label: "army metal", key: "armyMetal", theirs: "armyMetal" },
  ];

  function timelineGeometry(w) {
    const last = Math.max(view.match.lastFrame, 1);
    return { x0: GUTTER, x1: w - 12, last, fx: (f) => GUTTER + ((w - 12 - GUTTER) * f) / last };
  }

  /// The lanes of the timeline for the scope: [{label, color, items: [{f, to?, seat?, faint?}]}].
  function timelineLanes() {
    const match = view.match;
    const key = `${view.version}:${view.scope}`;
    if (view.lanesKey === key) return view.laneList;
    const lanes = WR.lanes(match, view.scope);
    view.lanes = lanes;
    const multi = match.seats.length > 1;
    const list = [
      { label: "unit losses", color: "ours", bySeat: multi, items: lanes.losses },
      { label: "building losses", color: "ours", bySeat: multi, items: lanes.buildingLosses },
      { label: "extractor losses", color: "ours", bySeat: multi, items: lanes.extractorLosses },
      { label: "kills", color: "theirs", items: lanes.kills },
    ];
    const turns = (transcript, seat) => transcript.turns.map((t) => ({ f: t.f, to: WR.turnEnd(match, t), seat, turn: t }));
    if (match.commander) list.push({ label: "commander turns", color: "llm", items: turns(match.commander, null), bars: true });
    const players = new Map();
    for (const seat of WR.scoped(match, view.scope)) {
      const player = playerOf(seat);
      if (player && !players.has(player)) players.set(player, seat.index);
    }
    if (players.size) {
      const items = [...players].flatMap(([player, seat]) => turns(player, seat)).sort((a, b) => a.f - b.f);
      list.push({ label: players.size > 1 ? "player turns, by seat" : "player turns", color: "llm", bySeat: players.size > 1, items, bars: players.size === 1 });
    }
    // The lane stands from the start for a seat with a log, and fills when its index has come: the page does not
    // jump when it does.
    if (expectsHands()) {
      const hands = WR.scoped(match, view.scope).filter((s) => s.jev);
      const items = hands.flatMap((s) => s.jev.index.gates.map((f) => ({ f, seat: s.index }))).sort((a, b) => a.f - b.f);
      list.push({ label: "hands: gates asked", color: "jev", bySeat: multi, items });
    }
    for (const [name, label] of [["builders", "hands: builders"], ["labs", "hands: plants"], ["groups", "hands: army"]]) {
      const items = lanes[name].map((d) => ({ ...d, faint: d.source === "list" }));
      if (items.length) list.push({ label, color: "jev", bySeat: multi, items, plays: true });
    }
    view.lanesKey = key;
    view.laneList = list;
    return list;
  }

  /// Whether a seat in scope has a hands' log, read yet or not.
  function expectsHands() {
    return WR.scoped(view.match, view.scope).some((s) => s.jev || (view.live && view.live.files.includes(`jev-${s.header.ai_id}.jsonl`)));
  }

  /// The Jev bill a game minute of the seats in scope that have an index: [{minute, seats: Map seat -> dollars}].
  function costMinutes() {
    const hands = WR.scoped(view.match, view.scope).filter((s) => s.jev);
    return hands.length ? WR.costView(hands.map((s) => ({ seat: s.index, index: s.jev.index }))) : null;
  }

  /// Everything of the timeline that does not move with the playhead, drawn once into its own canvas and blitted.
  function timelinePicture(w, h, dpr) {
    const match = view.match;
    const compact = document.body.dataset.compact === "1";
    const key = `${view.version}:${view.scope}:${w}:${dpr}:${compact}`;
    if (view.timelineKey === key) return view.timelineCanvas;
    const canvas = view.timelineCanvas || (view.timelineCanvas = document.createElement("canvas"));
    const lanes = timelineLanes();
    const cost = costMinutes();
    const charts = compact ? [] : [...CHARTS, ...(expectsHands() ? [{ label: "Jev $ a minute", cost: true }] : [])];
    const height = 6 + lanes.length * LANE_H + 4 + (charts.length ? 14 + charts.length * CHART_H : 0) + 20;
    view.timelineHeight = height;
    canvas.width = Math.round(w * dpr);
    canvas.height = Math.round(height * dpr);
    const ctx = canvas.getContext("2d");
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const g = timelineGeometry(w);
    ctx.font = "10.5px system-ui";
    ctx.textBaseline = "middle";
    let y = 6;
    for (const lane of lanes) {
      ctx.fillStyle = COLOR["ink-2"];
      ctx.textAlign = "right";
      const picked = lane.plays ? lane.items.filter((d) => !d.faint).length : 0;
      ctx.fillText(lane.plays ? `${lane.label} (${picked} pick, ${lane.items.length - picked} list)` : `${lane.label} (${lane.items.length})`, GUTTER - 8, y + LANE_H / 2);
      ctx.fillStyle = COLOR["surface-2"];
      ctx.fillRect(g.x0, y + 1, g.x1 - g.x0, LANE_H - 2);
      for (const item of lane.items) {
        ctx.fillStyle = lane.bySeat && item.seat != null ? seatColor(item.seat) : COLOR[lane.color];
        ctx.globalAlpha = item.faint ? 0.35 : 1;
        const x = Math.round(g.fx(item.f));
        // A turn is as wide as it took, in a game that ran on meanwhile.
        const wide = lane.bars && item.to > item.f ? Math.max(2, Math.round(g.fx(item.to)) - x) : 2;
        ctx.fillRect(x - (wide === 2 ? 1 : 0), y + 2, wide, LANE_H - 4);
      }
      ctx.globalAlpha = 1;
      y += LANE_H;
    }
    y += 4;
    view.chartRows = [];
    const seats = WR.scoped(match, null);
    const one = view.scope != null ? match.seats[view.scope] : match.seats.length === 1 ? match.seats[0] : null;
    if (charts.length) {
      // Legend, once.
      ctx.textAlign = "left";
      let lx = g.x0;
      const swatch = (color, text) => {
        ctx.fillStyle = color;
        ctx.fillRect(lx, y + 2, 10, 8);
        ctx.fillStyle = COLOR["ink-2"];
        ctx.fillText(text, lx + 14, y + 6);
        lx += 22 + ctx.measureText(text).width;
      };
      if (match.seats.length > 1) for (const s of match.seats) swatch(seatColor(s.index), one && one !== s ? `${s.name} (faint)` : s.name);
      else swatch(COLOR.ours, "ours (bot's view)");
      swatch(COLOR.theirs, match.truth.length ? "theirs (ground truth)" : "theirs: no truth file in this match");
      if (!one) { ctx.fillStyle = COLOR.muted; ctx.fillText("stacked: the top edge is the side's sum", lx, y + 6); }
      y += 14;
      // One column every two pixels: each seat's value at that frame.
      const columns = [];
      for (let x = g.x0; x <= g.x1; x += 2) columns.push(((x - g.x0) / (g.x1 - g.x0)) * g.last);
      const sample = (seat, chartKey) => {
        let i = -1;
        return columns.map((f) => {
          while (i + 1 < seat.series.length && seat.series[i + 1].f <= f) i++;
          return i >= 0 && f <= seat.lastFrame + 2 * WR.FPS ? seat.series[i][chartKey] : null;
        });
      };
      for (const chart of charts) {
        const top = y + 4;
        const bottom = y + CHART_H - 2;
        ctx.strokeStyle = COLOR.line;
        ctx.lineWidth = 1;
        ctx.beginPath(); ctx.moveTo(g.x0, bottom + 0.5); ctx.lineTo(g.x1, bottom + 0.5); ctx.stroke();
        let max = 1;
        if (chart.cost && !cost) {
          max = 0;
        } else if (chart.cost) {
          // Bars a game minute, stacked by seat.
          const dollars = (tokens) => (tokens * WR.JEV_PRICE) / 1e6;
          max = Math.max(0.01, ...cost.minutes.map((m) => dollars(m.tokens)));
          for (const m of cost.minutes) {
            let base = 0;
            const [xa, xb] = [g.fx(m.minute * 60 * WR.FPS), Math.min(g.x1, g.fx((m.minute + 1) * 60 * WR.FPS))];
            for (const [seat, tokens] of [...m.seats].sort((a, b) => a[0] - b[0])) {
              const hgt = ((bottom - top) * dollars(tokens)) / max;
              ctx.fillStyle = seatColor(seat);
              ctx.fillRect(xa + 0.5, bottom - base - hgt, Math.max(1, xb - xa - 1), hgt);
              base += hgt;
            }
          }
        } else {
          const values = seats.map((s) => sample(s, chart.key));
          const sums = columns.map((_, c) => values.reduce((n, v) => n + (v[c] || 0), 0));
          const theirs = chart.theirs ? match.truth.map((t) => t[chart.theirs]) : [];
          // The axis ends a little over the 99th percentile: a second's spike (a wreck reclaimed, a transfer) is
          // cut off at the top rather than flattening the whole curve.
          const peak = (list) => {
            const sorted = list.filter((v) => v > 0).sort((a, b) => a - b);
            return sorted.length ? Math.max(1, Math.min(sorted[sorted.length - 1], sorted[Math.floor(0.99 * (sorted.length - 1))] * 1.1)) : 1;
          };
          max = one ? Math.max(peak(values[seats.indexOf(one)].map((v) => v || 0)), ...(match.seats.length === 1 ? [peak(theirs)] : [])) : Math.max(peak(sums), peak(theirs));
          const vy = (v) => bottom - ((bottom - top) * Math.min(v, max)) / max;
          const cx = (c) => g.x0 + 2 * c;
          if (one) {
            // The seat's line over the other seats' faint ones, on one axis.
            seats.forEach((s, si) => {
              ctx.strokeStyle = seatColor(s.index);
              ctx.globalAlpha = s === one ? 1 : 0.3;
              ctx.lineWidth = s === one ? 2 : 1;
              ctx.beginPath();
              let started = false;
              values[si].forEach((v, c) => {
                if (v == null) return;
                if (started) ctx.lineTo(cx(c), vy(v)); else ctx.moveTo(cx(c), vy(v));
                started = true;
              });
              ctx.stroke();
            });
            ctx.globalAlpha = 1;
          } else {
            const base = columns.map(() => 0);
            seats.forEach((s, si) => {
              ctx.fillStyle = seatColor(s.index);
              ctx.globalAlpha = 0.85;
              ctx.beginPath();
              ctx.moveTo(cx(0), vy(base[0]));
              const tops = base.map((b, c) => b + (values[si][c] || 0));
              tops.forEach((v, c) => ctx.lineTo(cx(c), vy(v)));
              for (let c = columns.length - 1; c >= 0; c--) ctx.lineTo(cx(c), vy(base[c]));
              ctx.closePath();
              ctx.fill();
              tops.forEach((v, c) => (base[c] = v));
            });
            ctx.globalAlpha = 1;
          }
          // Theirs: the side's whole, so drawn where ours is the side's whole too (all seats, or a game of one).
          if (chart.theirs && match.truth.length && (!one || match.seats.length === 1)) {
            ctx.strokeStyle = COLOR.theirs;
            ctx.lineWidth = 1.5;
            ctx.beginPath();
            const step = Math.max(1, Math.floor(match.truth.length / (g.x1 - g.x0)));
            for (let i = 0; i < match.truth.length; i += step) {
              const t = match.truth[i];
              if (i === 0) ctx.moveTo(g.fx(t.f), vy(t[chart.theirs])); else ctx.lineTo(g.fx(t.f), vy(t[chart.theirs]));
            }
            ctx.stroke();
          }
        }
        ctx.fillStyle = COLOR["ink-2"];
        ctx.textAlign = "right";
        ctx.fillText(chart.label, GUTTER - 8, (top + bottom) / 2 + 4);
        ctx.fillStyle = COLOR.muted;
        ctx.fillText(chart.cost ? `$${max.toFixed(2)}` : String(Math.round(max)), GUTTER - 8, top + 2);
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
    view.timelineKey = key;
    return canvas;
  }

  function drawTimeline() {
    const canvas = $("timeline");
    if (!view.match) return;
    const dpr = window.devicePixelRatio || 1;
    const picture = timelinePicture(canvas.clientWidth, canvas.clientHeight, dpr);
    // The lanes and charts of the match decide the timeline's height.
    if (canvas.style.height !== `${view.timelineHeight}px`) canvas.style.height = `${view.timelineHeight}px`;
    const { ctx, w } = fitCanvas(canvas);
    ctx.drawImage(picture, 0, 0, picture.width / dpr, picture.height / dpr);
    const g = timelineGeometry(w);
    const line = (frame, color) => {
      const x = Math.round(g.fx(frame)) + 0.5;
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.beginPath(); ctx.moveTo(x, 4); ctx.lineTo(x, view.timelineBottom); ctx.stroke();
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
    const total = WR.totalsAt(match, view.scope, frame);
    const theirs = view.scope == null || match.seats.length === 1 ? match.truth[WR.indexAt(match.truth, frame)] : null;
    const lines = [WR.clock(frame)];
    if (total.seats) {
      lines.push(`metal +${total.metalIncome.toFixed(1)}   energy +${total.energyIncome.toFixed(0)}`);
      lines.push(`extractors ${total.extractors}${theirs ? ` / theirs ${theirs.extractors}` : ""}`);
      lines.push(`army ${total.army} worth ${Math.round(total.armyMetal)}${theirs ? ` / theirs ${theirs.army} worth ${Math.round(theirs.armyMetal)}` : ""}`);
      if (view.scope == null && match.seats.length > 1) {
        for (const seat of match.seats) {
          const t = WR.totalsAt(match, seat.index, frame);
          if (t.seats) lines.push(`  ${seat.name}: +${t.metalIncome.toFixed(1)} m, ${t.extractors} mex, army ${Math.round(t.armyMetal)}`);
        }
      }
    }
    const near = (items) => WR.range(items, frame - 5 * WR.FPS, frame + 5 * WR.FPS);
    const lanes = view.lanes || WR.lanes(match, view.scope);
    const lost = near(lanes.losses).concat(near(lanes.buildingLosses), near(lanes.extractorLosses));
    if (lost.length) lines.push(`lost: ${summarise(lost.map((e) => defName(e.d)))}`);
    const killed = near(lanes.kills);
    if (killed.length) lines.push(`killed: ${summarise(killed.map((e) => defName(e.d)))}`);
    for (const lane of view.laneList || []) {
      if (!lane.bars && !/turns/.test(lane.label)) continue;
      for (const item of near(lane.items).slice(0, 3)) {
        const who = item.seat != null && match.seats.length > 1 ? `player ${match.seats[item.seat].name}` : lane.label.replace(/ turns.*/, "");
        lines.push(`${WR.clock(item.f)} ${who}: ${item.turn.wake || "turn"}${item.turn.wallSeconds != null ? ` (${item.turn.wallSeconds.toFixed(0)} s)` : ""}`.slice(0, 110));
      }
    }
    const plays = near(lanes.builders).concat(near(lanes.labs), near(lanes.groups)).sort((a, b) => a.f - b.f);
    for (const d of plays.slice(0, 6)) lines.push(`${WR.clock(d.f)} ${d.source}: ${d.inputs.actor}: ${d.outputs.did || ""}`.slice(0, 110));
    if (plays.length > 6) lines.push(`... ${plays.length - 6} more plays`);
    const tip = $("chart-tooltip");
    tip.textContent = lines.join("\n");
    tip.hidden = false;
    const rect = $("timeline").getBoundingClientRect();
    const x = event.clientX - rect.left;
    tip.style.left = `${x > rect.width - 420 ? x - 400 : x + 14}px`;
    tip.style.bottom = "24px";
    if (event.buttons & 1) seek(frame);
    else drawTimeline();
  }

  function summarise(names) {
    const counts = new Map();
    for (const n of names) counts.set(n, (counts.get(n) || 0) + 1);
    return [...counts].map(([n, k]) => (k > 1 ? `${n} x${k}` : n)).join(", ");
  }

  // ---------------------------------------------------------------- playhead

  function setTab(name) {
    view.tab = name;
    for (const button of document.querySelectorAll("#tabs [data-tab]")) button.classList.toggle("active", button.dataset.tab === name);
    for (const tab of document.querySelectorAll(".tab")) tab.hidden = tab.dataset.tab !== name;
    if (name !== "unit") try { localStorage.setItem("wr-tab", name); } catch (_) { /* no storage */ }
    renderAll();
  }

  /// Narrows the page to one seat (its index) or widens it to all (null).
  function setScope(scope) {
    view.scope = scope;
    if (view.actor && scope != null && view.actor.seat !== scope) view.actor = null;
    WV.panels.scopeChanged();
    renderAll();
  }

  /// Puts a hands' actor in focus: its seat in scope, its units ringed on the map, its menu opened in the Hands tab.
  function focusActor(seat, name, tab) {
    view.actor = name ? { seat, name } : null;
    if (name && view.scope !== seat) {
      view.scope = seat;
      WV.panels.scopeChanged();
    }
    if (tab) setTab(tab);
    else renderAll();
  }

  function renderAll() {
    if (!view.match) return;
    $("clock").textContent = WR.clock(view.frame);
    drawMap();
    drawTimeline();
    WV.panels.render();
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

  function wire() {
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
      else if (e.key === "Escape") return view.selected != null ? select(null) : focusActor(null, null);
      else return;
      leaveLive();
      e.preventDefault();
    });
    new ResizeObserver(() => renderAll()).observe(document.body);
  }

  return {
    $, el, view, COLOR, status, seatColor, inScope, classOf, defName, playerOf, summarise, gateAt, pictureAt, actorUnits,
    seek, select, setTab, setScope, focusActor, renderAll, drawMap, start: () => { wire(); boot(); }, panels: {},
  };
})();
