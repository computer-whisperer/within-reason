// The 3D view in headless Chromium over the DevTools protocol (node >= 22 for the global WebSocket):
//   run/view_match.py <matches dir> --no-browser --port 8137 &
//   node viewer/test/scene.js "http://127.0.0.1:8137/?match=matches/<batch>/<NN>/" <out dir> [seconds ...]
// Opens a match that has a tape (game.tape, run/tape.py), waits for the scene, and for each game time given (default
// a third and two thirds through) seeks there and saves three screenshots into the out dir: the whole map, a close
// look at the busiest ground, and the same turned and tilted. It prints what the scene drew (units, projectiles, the
// milliseconds a frame took, the slices fetched) and fails on any page exception or console error, on a scene that
// never opens, and on a frame with no units when the tape has them.
// CHROMIUM names the browser (default chromium); SCENE_GPU=1 uses the machine's GPU instead of the software renderer.
"use strict";
const { spawn } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const url = process.argv[2];
const out = process.argv[3];
if (!url || !out) {
  console.error("usage: node viewer/test/scene.js <match url> <out dir> [seconds ...]");
  process.exit(2);
}
const times = process.argv.slice(4).map(Number);
fs.mkdirSync(out, { recursive: true });
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "wr-scene-"));
const gpu = process.env.SCENE_GPU ? ["--use-angle=vulkan", "--enable-features=Vulkan", "--ignore-gpu-blocklist"] : ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"];
const chrome = spawn(process.env.CHROMIUM || "chromium", [
  "--headless=new", "--no-sandbox", ...gpu, "--remote-debugging-port=0", `--user-data-dir=${profile}`, "--window-size=1600,1000", "about:blank",
], { stdio: ["ignore", "ignore", "pipe"] });

const fail = (message) => {
  console.error(`FAIL: ${message}`);
  chrome.kill();
  process.exit(1);
};
setTimeout(() => fail("timed out"), 240000);

let stderr = "";
chrome.stderr.on("data", async (chunk) => {
  stderr += chunk;
  const m = /DevTools listening on (ws:\/\/[^\s]+)/.exec(stderr);
  if (!m || chrome.started) return;
  chrome.started = true;
  const targets = await (await fetch(`http://127.0.0.1:${new URL(m[1]).port}/json`)).json();
  run(new WebSocket(targets.find((t) => t.type === "page").webSocketDebuggerUrl));
});

function run(socket) {
  let id = 0;
  const waiting = new Map();
  const problems = [];
  socket.onmessage = ({ data }) => {
    const message = JSON.parse(data);
    if (message.id) waiting.get(message.id)?.(message.result);
    if (message.method === "Runtime.exceptionThrown") problems.push(message.params.exceptionDetails.exception?.description || message.params.exceptionDetails.text);
    if (message.method === "Runtime.consoleAPICalled" && message.params.type === "error") problems.push(message.params.args.map((a) => a.description || a.value).join(" "));
  };
  const send = (method, params = {}) => new Promise((resolve) => {
    waiting.set(++id, resolve);
    socket.send(JSON.stringify({ id, method, params }));
  });
  const evaluate = async (expression) => (await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true })).result.value;
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const shot = async (name) => {
    const { data } = await send("Page.captureScreenshot", { format: "png" });
    fs.writeFileSync(path.join(out, name), Buffer.from(data, "base64"));
  };
  /// Waits until the scene has drawn the playhead's own frame with nothing still on its way.
  const settled = async (what) => {
    for (let i = 0; i < 300; i++) {
      const state = await evaluate(`(() => { const s = window.viewer.scene; return s ? { loading: !!s.stats.loading, pending: !!s.pending, frame: s.frame, at: window.viewer.frame } : null; })()`);
      if (state && !state.loading && !state.pending && Math.abs(state.frame - state.at) < 1) return;
      if (problems.length) fail(`${what}: ${problems.join("\n")}`);
      await sleep(100);
    }
    fail(`${what}: the scene did not settle`);
  };

  socket.onopen = async () => {
    await send("Runtime.enable");
    await send("Page.enable");
    await send("Page.navigate", { url });
    for (let i = 0; i < 400; i++) {
      if (await evaluate("!!(window.viewer && window.viewer.scene && document.body.dataset.loaded)")) break;
      if (problems.length) fail(problems.join("\n"));
      await sleep(100);
    }
    if (!(await evaluate("!!(window.viewer && window.viewer.scene)"))) fail(`the scene did not open: ${await evaluate("document.getElementById('status').textContent")}`);
    await evaluate("window.viewer.playing = false");
    // The map's own texture is the largest thing fetched (a 20 by 20 map's tiles are 70 MB): given half a minute.
    let ground = "plain";
    for (let i = 0; i < 300 && ground === "plain"; i++) {
      ground = await evaluate("window.viewer.scene.terrain.note()");
      if (ground === "plain") await sleep(100);
    }
    const last = await evaluate("window.viewer.scene.tape.lastFrame");
    const info = await evaluate(`(() => { const t = window.viewer.scene.tape; const gl = window.viewer.scene.canvas.getContext("webgl2"); const dbg = gl.getExtension("WEBGL_debug_renderer_info");
      return { map: t.head.map.name, minutes: (t.lastFrame / 1800).toFixed(1), units: t.unitCount, slices: t.slices.length, bytes: t.size, s3tc: !!gl.getExtension("WEBGL_compressed_texture_s3tc"), renderer: dbg ? gl.getParameter(dbg.UNMASKED_RENDERER_WEBGL) : "?" }; })()`);
    console.log(`tape: ${info.map}, ${info.minutes} min, ${info.units} unit lives, ${info.slices} slices, ${(info.bytes / 1e6).toFixed(1)} MB; the ground's texture: ${ground}; S3TC ${info.s3tc}; ${info.renderer}`);
    for (const seconds of times.length ? times : [last / 90, last / 45]) {
      const frame = Math.min(last, Math.round(seconds * 30));
      const tag = `${Math.floor(seconds / 60)}m${String(Math.round(seconds % 60)).padStart(2, "0")}`;
      const t0 = Date.now();
      await evaluate(`window.viewer.scene.resetCamera(); window.viewer.seek(${frame})`);
      await settled(`seek to ${tag}`);
      const seekMs = Date.now() - t0;
      await sleep(1500); // models and textures still arriving redraw as they come
      await settled(`assets at ${tag}`);
      await shot(`${tag}-map.png`);
      const stats = await evaluate("(() => { const s = window.viewer.scene.stats; return { units: s.units, projectiles: s.projectiles, ms: s.ms, draws: s.draws, triangles: s.triangles }; })()");
      if (!stats.units) fail(`${tag}: no units drawn`);
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
      console.log(`${tag}: ${stats.units} units in ${stats.draws} model draws, ${stats.projectiles} projectiles, ${Math.round(stats.triangles / 1000)}k ground triangles, a frame ${stats.ms.toFixed(1)} ms, the seek ${seekMs} ms${busy ? `; the close look at (${Math.round(busy[0])}, ${Math.round(busy[1])}) with ${busy[2]} mobile units` : ""}`);
    }
    // Seeks, now that the models are here: to five far-apart moments whose slices are not loaded, each timed from
    // the seek to the frame drawn.
    const seeks = [];
    for (const share of [0.07, 0.83, 0.31, 0.64, 0.47]) {
      const t0 = Date.now();
      await evaluate(`window.viewer.seek(${Math.round(last * share)})`);
      await settled(`seek to ${share}`);
      seeks.push(Date.now() - t0);
    }
    console.log(`seeks to slices not loaded: ${seeks.join(", ")} ms`);
    // The mouse: the wheel zooms about the cursor, a drag moves the ground, a shift-drag turns, a click picks a unit
    // and a hover says what is under it.
    await evaluate("window.viewer.scene.resetCamera(); window.viewer.scene.redraw()");
    await settled("reset");
    const box = await evaluate(`(() => { const r = window.viewer.scene.canvas.getBoundingClientRect(); return [r.left, r.top, r.width, r.height]; })()`);
    const [cx, cy] = [box[0] + box[2] / 2, box[1] + box[3] / 2];
    const camera = () => evaluate("(() => { const c = window.viewer.scene.camera; return [c.target[0], c.target[2], c.distance, c.yaw, c.pitch]; })()");
    const mouse = (type, x, y, extra = {}) => send("Input.dispatchMouseEvent", { type, x, y, button: "none", buttons: 0, ...extra });
    const before = await camera();
    await mouse("mouseWheel", cx, cy, { deltaX: 0, deltaY: -600 });
    await sleep(150);
    const zoomed = await camera();
    if (!(zoomed[2] < before[2] * 0.8)) fail(`the wheel did not zoom in: distance ${before[2]} -> ${zoomed[2]}`);
    await mouse("mousePressed", cx, cy, { button: "left", buttons: 1, clickCount: 1 });
    await mouse("mouseMoved", cx + 120, cy + 60, { button: "left", buttons: 1 });
    await mouse("mouseReleased", cx + 120, cy + 60, { button: "left", clickCount: 1 });
    await sleep(150);
    const panned = await camera();
    if (Math.hypot(panned[0] - zoomed[0], panned[1] - zoomed[1]) < 20) fail("a drag did not move the ground");
    await mouse("mousePressed", cx, cy, { button: "left", buttons: 1, clickCount: 1, modifiers: 8 });
    await mouse("mouseMoved", cx + 150, cy - 80, { button: "left", buttons: 1, modifiers: 8 });
    await mouse("mouseReleased", cx + 150, cy - 80, { button: "left", clickCount: 1, modifiers: 8 });
    await sleep(150);
    const turned = await camera();
    if (Math.abs(turned[3] - panned[3]) < 0.2) fail("a shift-drag did not turn the camera");
    // A unit under the cursor: the first one alive, its place put under the view's centre.
    const picked = await evaluate(`(() => { const s = window.viewer.scene, st = s.tape.frame(s.frame); s.resetCamera(); s.lookAt(st.units[0], st.units[2], 700); return st.units[13]; })()`);
    await settled("before the pick");
    await sleep(200);
    await mouse("mouseMoved", cx, cy);
    await sleep(150);
    const tip = await evaluate("document.getElementById('tooltip').hidden ? '' : document.getElementById('tooltip').textContent");
    if (!/#\d+/.test(tip)) fail(`the hover over a unit says: ${JSON.stringify(tip)}`);
    await mouse("mousePressed", cx, cy, { button: "left", buttons: 1, clickCount: 1 });
    await mouse("mouseReleased", cx, cy, { button: "left", clickCount: 1 });
    await sleep(300);
    const selected = await evaluate("window.viewer.scene.selected");
    console.log(`mouse: the wheel zoomed ${before[2].toFixed(0)} -> ${zoomed[2].toFixed(0)}, a drag moved the ground ${Math.hypot(panned[0] - zoomed[0], panned[1] - zoomed[1]).toFixed(0)} elmos, a shift-drag turned ${(turned[3] - panned[3]).toFixed(2)} rad; the hover says ${JSON.stringify(tip.split("\n")[0])}; a click selected row ${selected} (the unit under it is row ${picked})`);
    if (selected < 0) fail("a click on a unit selected nothing");
    await shot("selected.png");
    if (problems.length) fail(problems.join("\n"));
    console.log(`ok: screenshots in ${out}`);
    chrome.kill();
    process.exit(0);
  };
}
