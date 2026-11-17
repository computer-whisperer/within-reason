// The 3D view in a real headless browser (node >= 22 for the global WebSocket):
//   run/view_match.py <matches dir> --no-browser --port 8137 &
//   node viewer/test/scene.js "http://127.0.0.1:8137/?match=matches/<batch>/<NN>/" <out dir> [seconds ...]
// Opens a match that has a tape (game.tape, run/tape.py) as a user gets it and checks that the scene it opens on is a
// view of the map (a camera with a place, a frame drawn with units); then for each game time given (default a third
// and two thirds through) seeks there and saves three screenshots into the out dir: the whole map, a close look at
// the busiest ground, and the same turned and tilted. It times seeks to slices not loaded, drives the mouse (wheel,
// drag, shift-drag, hover, click) and prints what the scene drew. It fails on any page exception or console error.
//
// SCENE_BROWSER=firefox drives Firefox (over WebDriver BiDi) where the default drives Chromium (over the DevTools
// protocol): **run both**, the page is used in Firefox. CHROMIUM and FIREFOX name the binaries. SCENE_GPU=1 has
// Chromium use the machine's GPU instead of its software renderer.
"use strict";
const fs = require("fs");
const path = require("path");

const url = process.argv[2];
const out = process.argv[3];
if (!url || !out) {
  console.error("usage: [SCENE_BROWSER=firefox] node viewer/test/scene.js <match url> <out dir> [seconds ...]");
  process.exit(2);
}
const times = process.argv.slice(4).map(Number);
const firefox = process.env.SCENE_BROWSER === "firefox";
fs.mkdirSync(out, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const [WIDTH, HEIGHT] = [1600, 1000];

let browser = null;
const fail = (message) => {
  console.error(`FAIL: ${message}`);
  if (browser) browser.kill();
  process.exit(1);
};
setTimeout(() => fail("timed out"), 300000);

(async () => {
  const page = await require("./driver").open({ firefox, out, width: WIDTH, height: HEIGHT, fail });
  browser = page;
  const { evaluate, shot, problems } = page;
  /// Waits until the scene has drawn the playhead's own frame with nothing still on its way.
  const settled = async (what) => {
    for (let i = 0; i < 300; i++) {
      const state = await evaluate(`(() => { const s = window.viewer && window.viewer.scene; return s ? { loading: !!s.stats.loading, pending: !!s.pending, drawn: s.stats.ms != null, frame: s.frame, at: window.viewer.frame } : null; })()`);
      if (state && state.drawn && !state.loading && !state.pending && Math.abs(state.frame - state.at) < 1) return;
      if (problems.length) fail(`${what}: ${problems.join("\n")}`);
      await sleep(100);
    }
    fail(`${what}: the scene did not settle`);
  };

  await page.navigate(url);
  for (let i = 0; i < 400; i++) {
    if (await evaluate("!!(window.viewer && window.viewer.scene && document.body.dataset.loaded)")) break;
    if (problems.length) fail(problems.join("\n"));
    await sleep(100);
  }
  if (!(await evaluate("!!(window.viewer && window.viewer.scene)"))) fail(`the scene did not open: ${await evaluate("document.getElementById('status').textContent")}`);
  await evaluate("(window.viewer.playing = false)");
  // The page as it opens, before this test touches the camera: the whole map in view, a frame drawn. (Until
  // 2026-11-15 night this test reset the camera before every screenshot and so never saw that the scene opened with
  // its camera at infinity: fitted while the canvas was still hidden. The user found it in Firefox.)
  await settled("the scene as it opens");
  const opened = await evaluate("(() => { const s = window.viewer.scene; return { distance: s.camera.distance, inverse: !!(s.last && s.last.inverse), units: s.stats.units, triangles: s.stats.triangles }; })()");
  if (!Number.isFinite(opened.distance) || !opened.inverse || !opened.units || opened.triangles < 20000) fail(`the scene as it opens is not a view of the map: ${JSON.stringify(opened)}`);
  await shot("opened.png");
  // The map's own texture is the largest thing fetched (a 20 by 20 map's tiles are 70 MB): given half a minute.
  let ground = "plain";
  for (let i = 0; i < 300 && ground === "plain"; i++) {
    ground = await evaluate("window.viewer.scene.terrain.note()");
    if (ground === "plain") await sleep(100);
  }
  const last = await evaluate("window.viewer.scene.tape.lastFrame");
  const info = await evaluate(`(() => { const t = window.viewer.scene.tape; const gl = window.viewer.scene.canvas.getContext("webgl2"); const dbg = gl.getExtension("WEBGL_debug_renderer_info");
    return { map: t.head.map.name, minutes: (t.lastFrame / 1800).toFixed(1), units: t.unitCount, slices: t.slices.length, bytes: t.size, s3tc: !!gl.getExtension("WEBGL_compressed_texture_s3tc"), renderer: dbg ? gl.getParameter(dbg.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER) }; })()`);
  console.log(`${page.name}; tape: ${info.map}, ${info.minutes} min, ${info.units} unit lives, ${info.slices} slices, ${(info.bytes / 1e6).toFixed(1)} MB; the ground's texture: ${ground}; S3TC ${info.s3tc}; ${info.renderer}`);
  for (const seconds of times.length ? times : [last / 90, last / 45]) {
    const frame = Math.min(last, Math.round(seconds * 30));
    const tag = `${Math.floor(seconds / 60)}m${String(Math.round(seconds % 60)).padStart(2, "0")}`;
    const t0 = Date.now();
    await evaluate(`window.viewer.seek(${frame})`);
    await settled(`seek to ${tag}`);
    const seekMs = Date.now() - t0;
    await sleep(1500); // models and textures still arriving redraw as they come
    await settled(`assets at ${tag}`);
    await evaluate("(window.viewer.scene.resetCamera(), window.viewer.scene.redraw())"); // back from the last close look
    await settled(`the whole map at ${tag}`);
    await shot(`${tag}-map.png`);
    const stats = await evaluate("(() => { const s = window.viewer.scene.stats; return { units: s.units, projectiles: s.projectiles, ms: s.ms, draws: s.draws, triangles: s.triangles }; })()");
    if (!stats.units) fail(`${tag}: no units drawn`);
    // The icons switch, as the user flips it: off, every unit its model at this distance; then on again.
    const iconsSwitch = (on) => evaluate(`(() => { const box = document.querySelector('input[data-scene-layer="icons"]'); box.checked = ${on}; box.dispatchEvent(new Event("change")); return window.viewer.scene.layers.icons; })()`);
    if ((await iconsSwitch(false)) !== false) fail("the icons switch did not reach the scene");
    await settled(`the whole map without icons at ${tag}`);
    await shot(`${tag}-map-models.png`);
    const modelsMs = await evaluate("window.viewer.scene.stats.ms");
    await iconsSwitch(true);
    await settled(`icons again at ${tag}`);
    // The busiest ground: the cell of 600 elmos with the most mobile units.
    const busy = await evaluate(`(() => { const s = window.viewer.scene, st = s.tape.frame(s.frame), cells = new Map(); let best = null;
      for (let i = 0; i < st.units.length; i += 16) { if (!(st.units[i + 12] & 2)) continue; const k = Math.floor(st.units[i] / 600) + "," + Math.floor(st.units[i + 2] / 600); const c = cells.get(k) || { n: 0, x: 0, z: 0 }; c.n++; c.x += st.units[i]; c.z += st.units[i + 2]; cells.set(k, c); if (!best || c.n > best.n) best = c; }
      return best ? [best.x / best.n, best.z / best.n, best.n] : null; })()`);
    if (busy) {
      await evaluate(`window.viewer.scene.lookAt(${busy[0]}, ${busy[1]}, 900)`);
      await settled(`close at ${tag}`);
      await sleep(1200);
      await settled(`close assets at ${tag}`);
      await shot(`${tag}-close.png`);
      await evaluate(`(() => { const c = window.viewer.scene.camera; c.yaw = 0.9; c.pitch = 0.55; c.distance = 500; window.viewer.scene.redraw(); })()`);
      await settled(`turned at ${tag}`);
      await shot(`${tag}-turned.png`);
    }
    console.log(`${tag}: ${stats.units} units in ${stats.draws} model draws, ${stats.projectiles} projectiles, ${Math.round(stats.triangles / 1000)}k ground triangles, a frame ${stats.ms.toFixed(1)} ms (${modelsMs.toFixed(1)} ms with no icons), the seek ${seekMs} ms${busy ? `; the close look at (${Math.round(busy[0])}, ${Math.round(busy[1])}) with ${busy[2]} mobile units` : ""}`);
  }
  // Seeks, now that the models are here: to five far-apart moments whose slices are not loaded, each timed from the
  // seek to the frame drawn.
  const seeks = [];
  for (const share of [0.07, 0.83, 0.31, 0.64, 0.47]) {
    const t0 = Date.now();
    await evaluate(`window.viewer.seek(${Math.round(last * share)})`);
    await settled(`seek to ${share}`);
    seeks.push(Date.now() - t0);
  }
  console.log(`seeks to slices not loaded: ${seeks.join(", ")} ms`);
  // The mouse: the wheel zooms about the cursor, a drag moves the ground, a shift-drag turns, a hover says what is
  // under it and a click picks it.
  await evaluate("(window.viewer.scene.resetCamera(), window.viewer.scene.redraw())");
  await settled("reset");
  const box = await evaluate(`(() => { const r = window.viewer.scene.canvas.getBoundingClientRect(); return [r.left, r.top, r.width, r.height]; })()`);
  const [cx, cy] = [box[0] + box[2] / 2, box[1] + box[3] / 2];
  const camera = () => evaluate("(() => { const c = window.viewer.scene.camera; return [c.target[0], c.target[2], c.distance, c.yaw, c.pitch]; })()");
  const before = await camera();
  await page.wheel(cx, cy, -600);
  await sleep(200);
  const zoomed = await camera();
  if (!(zoomed[2] < before[2] * 0.9)) fail(`the wheel did not zoom in: distance ${before[2]} -> ${zoomed[2]}`);
  await page.drag(cx, cy, cx + 120, cy + 60, false);
  await sleep(200);
  const panned = await camera();
  if (Math.hypot(panned[0] - zoomed[0], panned[1] - zoomed[1]) < 20) fail("a drag did not move the ground");
  await page.drag(cx, cy, cx + 150, cy - 80, true);
  await sleep(200);
  const turned = await camera();
  if (Math.abs(turned[3] - panned[3]) < 0.2) fail("a shift-drag did not turn the camera");
  // A unit under the cursor: the first one alive, its place put under the view's centre.
  const picked = await evaluate(`(() => { const s = window.viewer.scene, st = s.tape.frame(s.frame); s.resetCamera(); s.lookAt(st.units[0], st.units[2], 700); return st.units[13]; })()`);
  await settled("before the pick");
  await sleep(200);
  await page.move(cx + 3, cy + 3);
  await page.move(cx, cy);
  await sleep(200);
  const tip = await evaluate("(document.getElementById('tooltip').hidden ? '' : document.getElementById('tooltip').textContent)");
  if (!/#\d+/.test(tip)) fail(`the hover over a unit says: ${JSON.stringify(tip)}`);
  await page.click(cx, cy);
  await sleep(300);
  const selected = await evaluate("window.viewer.scene.selected");
  console.log(`mouse: the wheel zoomed ${before[2].toFixed(0)} -> ${zoomed[2].toFixed(0)}, a drag moved the ground ${Math.hypot(panned[0] - zoomed[0], panned[1] - zoomed[1]).toFixed(0)} elmos, a shift-drag turned ${(turned[3] - panned[3]).toFixed(2)} rad; the hover says ${JSON.stringify(tip.split("\n")[0])}; a click selected row ${selected} (the unit under it is row ${picked})`);
  if (selected < 0) fail("a click on a unit selected nothing");
  await shot("selected.png");
  // The selected unit's panel, from the tape whoever's it is: what it is, its health, who made it.
  await sleep(300);
  const panel = await evaluate(`(() => { const box = document.getElementById("unit"), tab = box.closest(".tab"); return { shown: !!tab && !tab.hidden, text: box.textContent }; })()`);
  if (!panel.shown || !/#\d+/.test(panel.text) || !/made/.test(panel.text) || !/health|gone|not made/.test(panel.text)) fail(`the unit panel: ${JSON.stringify(panel).slice(0, 500)}`);
  // The lines over the scene: the selected unit's alone at first, then every standing order and every defence's reach,
  // the whole map in view halfway through the game.
  const linesNow = async (what) => (await settled(what), evaluate("window.viewer.scene.stats.lines"));
  const selectedLines = await linesNow("the selected unit's lines");
  await evaluate(`window.viewer.seek(${Math.round(last * 0.5)})`);
  await evaluate("(window.viewer.scene.resetCamera(), window.viewer.scene.redraw())");
  const fewLines = await linesNow("the lines halfway");
  // The mouse has rested at the centre since the click: after the seek the hover names what is under it now.
  const rest = await evaluate(`(() => { const s = window.viewer.scene, r = s.canvas.getBoundingClientRect(), row = s.pick(r.width / 2, r.height / 2), t = document.getElementById("tooltip");
    return { id: row >= 0 ? s.tape.unit(row).id : null, first: t.hidden ? "" : t.textContent.split("\\n")[0] }; })()`);
  if (rest.id != null ? !rest.first.includes(`#${rest.id}`) : /#\d+/.test(rest.first)) fail(`the hover after a seek, the mouse still: ${JSON.stringify(rest)}`);
  // The selected unit's orders: each one listed inside the span its heading names, once that span is read.
  let orders = null;
  for (let i = 0; i < 50 && !(orders && orders.span && !orders.coming); i++) {
    await sleep(100);
    orders = await evaluate(`(() => { const box = document.getElementById("unit"), m = /from (\\d+):(\\d\\d) to (\\d+):(\\d\\d)/.exec(box.textContent);
      const seconds = (t) => { const [a, b] = t.split(":").map(Number); return a * 60 + b; };
      return { span: m && [Number(m[1]) * 60 + Number(m[2]), Number(m[3]) * 60 + Number(m[4])], coming: /still reading the tape there/.test(box.textContent),
        when: [...box.querySelectorAll("ol.orders .when")].map((w) => seconds(w.textContent)) }; })()`);
  }
  if (orders.span && orders.when.some((t) => t < orders.span[0] || t > orders.span[1])) fail(`orders outside the panel's span: ${JSON.stringify(orders)}`);
  await evaluate("(Object.assign(window.viewer.scene.layers, { orders: true, ranges: true }), window.viewer.scene.redraw())");
  const allLines = await linesNow("every order and reach");
  if (!(allLines > fewLines)) fail(`every unit's orders and every defence's reach drew no more lines (${fewLines} -> ${allLines})`);
  const overlay = await evaluate(`(() => { const o = document.getElementById("scene-overlay"), c = document.getElementById("scene"); return { hidden: o.hidden, w: o.width, h: o.height, cw: c.width, ch: c.height }; })()`);
  if (overlay.hidden || overlay.w !== overlay.cw || overlay.h !== overlay.ch) fail(`the overlay over the scene: ${JSON.stringify(overlay)}`);
  await shot("orders-and-ranges.png");
  await evaluate("(Object.assign(window.viewer.scene.layers, { orders: false, ranges: false }), window.viewer.scene.redraw())");
  // As one side saw it: some of the other side's units are ghosts halfway through, and none of the chosen side's.
  await evaluate(`(() => { const select = document.getElementById("seen-by"); select.value = "0"; select.dispatchEvent(new Event("change")); })()`);
  await settled("as side 1 saw it");
  const counts = await evaluate(`(() => { const s = window.viewer.scene, st = s.tape.frame(s.frame), ally = new Map(s.tape.head.teams.map((t) => [t.team, t.ally]));
    const c = { own: 0, ownGhost: 0, other: 0, ghost: 0, unseen: 0, radar: 0, marks: (s.tape.events || []).filter((e) => e.k === "mark").length };
    for (let i = 0, j = 0; i < st.units.length; i += 16, j++) { const f = st.units[i + 12]; if (ally.get(st.units[i + 11]) === 0) { c.own++; if (f & 8) c.ownGhost++; } else { c.other++; if (f & 8) c.ghost++; if ((f & 24) === 24) c.radar++; if (!(st.seen[j] & 1)) c.unseen++; } }
    return c; })()`);
  // (How many it saw is the game's: a side at home sees none of the other's. The ghosts are exactly the units the
  // tape says it did not have in sight.)
  if (counts.ownGhost || !counts.ghost || counts.ghost !== counts.unseen) fail(`as side 1 saw it: ${JSON.stringify(counts)}`);
  await shot("seen-by-side-1.png");
  await evaluate(`(() => { const select = document.getElementById("seen-by"); select.value = "-1"; select.dispatchEvent(new Event("change")); })()`);
  let sightWords = `as side 1 saw it: ${counts.ghost} of the other side's ${counts.other} units unseen (${counts.radar} of them on its radar), none of its own ${counts.own}; ${counts.marks} players' marks in the game`;
  // With a record: what the tape says our side saw against what our bot's record says it saw, at a few moments.
  if (!(await evaluate("window.viewer.match.tapeOnly"))) {
    const rows = [];
    for (const share of [0.2, 0.4, 0.6, 0.8]) {
      await evaluate(`window.viewer.seek(${Math.round(last * share)})`);
      await settled(`the sight at ${share}`);
      rows.push(await evaluate(`(() => { const v = window.viewer, s = v.scene, st = s.tape.frame(s.frame), ally = new Map(s.tape.head.teams.map((t) => [t.team, t.ally]));
        const ours = ally.get(v.match.seats[0].team); let sight = 0, radar = 0, all = 0;
        for (let i = 0, j = 0; i < st.units.length; i += 16, j++) { if (ally.get(st.units[i + 11]) === ours) continue; all++; if (st.seen[j] & (1 << ours)) sight++; else if (st.seen[j] & (1 << (ours + 16))) radar++; }
        return WR.clock(v.frame) + ": tape " + sight + " in sight + " + radar + " on radar of " + all + ", the record " + WR.stateAt(v.match, v.frame).enemies.length; })()`));
    }
    sightWords += `; ours against the record: ${rows.join("; ")}`;
  }
  // The strip across the top: a line a side with the tape's numbers, both sides', with or without a record of ours;
  // a click on a number puts its chart on the timeline and a second click takes it away; a side unfolds.
  const now = await evaluate(`(() => { const rows = () => document.querySelectorAll(".tl-row.chart").length, before = rows();
    const cell = [...document.querySelectorAll("#strip tr.side td.num")].find((td) => td.title.startsWith("energy income") && !td.classList.contains("charted"));
    if (cell) cell.click();
    const added = rows() - before;
    const again = [...document.querySelectorAll("#strip tr.side td.num")].find((td) => td.title.startsWith("energy income"));
    if (cell && again) again.click();
    const fold = document.querySelector("#strip td.who.folds");
    if (fold) fold.click();
    const teams = document.querySelectorAll("#strip tr.team").length;
    if (fold) document.querySelector("#strip td.who.folds").click();
    return { tapeOnly: document.body.dataset.tapeonly === "1", sides: document.querySelectorAll("#strip tr.side").length, absent: document.querySelectorAll("#strip tr.side td.absent").length, added, back: rows() - before,
      folds: !!fold, teams, lanes: document.querySelectorAll(".tl-row.lane").length, text: document.querySelector("#strip tr.side").textContent }; })()`);
  if (now.sides < 2 || now.absent) fail(`the strip has no line a side, or a side without the tape's numbers: ${JSON.stringify(now).slice(0, 300)}`);
  if (now.added !== 1 || now.back !== 0) fail(`a click on a number of the strip did not show its chart and a second take it away: ${JSON.stringify(now).slice(0, 300)}`);
  if (now.folds && !now.teams) fail("a side of several players did not unfold");
  // With a record, a unit of ours: the tape's panel, then the record's part (the hands' picture, its history there).
  let oursPanel = null;
  if (!now.tapeOnly) {
    const id = await evaluate(`(() => { const v = window.viewer, u = WR.stateAt(v.match, v.frame).own[0]; if (u) v.select(u.id); return u ? u.id : null; })()`);
    await settled("a unit of ours selected");
    oursPanel = await evaluate(`document.getElementById("unit").textContent`);
    if (id == null || !oursPanel.includes(`#${id}`) || !/Orders given to it/i.test(oursPanel) || !/history in the record/.test(oursPanel)) fail(`the panel of our unit #${id}: ${JSON.stringify(oursPanel).slice(0, 600)}`);
    await shot("ours-selected.png");
    // In the 2D map the panel reads the tape at the playhead (the 3D view draws nothing there, its last frame stale).
    await evaluate(`document.getElementById("mode").click()`);
    await evaluate(`window.viewer.seek(${Math.round(last * 0.8)})`);
    let at = null;
    for (let i = 0; i < 50 && !at; i++) {
      await sleep(100);
      at = await evaluate(`(() => { const v = window.viewer, u = v.scene.unitNow(v.selectedRow, v.frame); const text = document.getElementById("unit").textContent;
        return u ? { place: "(" + Math.round(u[0]) + ", " + Math.round(u[2]) + ")", text } : (v.frame >= v.scene.tape.unit(v.selectedRow).destroyed ? { place: "gone", text } : null); })()`);
    }
    if (!at || !at.text.includes(at.place)) fail(`the panel in the 2D map after a seek: ${JSON.stringify(at).slice(0, 400)}`);
    oursPanel += ` | in the 2D map at ${Math.round(last * 0.8 / 30)} s: ${at.place}`;
    await evaluate(`document.getElementById("mode").click()`);
  }
  console.log(`the unit panel: ${JSON.stringify(panel.text.slice(0, 160))}...;${oursPanel ? ` ours: ${JSON.stringify(oursPanel.slice(0, 120))}...${oursPanel.includes(" | ") ? `; ${oursPanel.split(" | ").pop()}` : ""};` : ""} after a seek the resting hover says ${JSON.stringify(rest.first)}, the panel lists ${orders.when.length} orders in ${orders.span ? orders.span.map((t) => `${Math.floor(t / 60)}:${String(t % 60).padStart(2, "0")}`).join("-") : "no span"}; lines: ${selectedLines} with the selection, ${fewLines} halfway, ${allLines} with every order and reach; overlay ${overlay.w}x${overlay.h}; ${sightWords}; the strip: ${now.sides} sides${now.folds ? `, ${now.teams} players unfolded` : ""}, ${now.lanes} lanes, ${JSON.stringify(now.text.slice(0, 80))}`);
  if (problems.length) fail(problems.join("\n"));
  console.log(`ok: screenshots in ${out}`);
  browser.kill();
  process.exit(0);
})();
