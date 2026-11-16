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
const { spawn } = require("child_process");
const fs = require("fs");
const os = require("os");
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
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "wr-scene-"));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const [WIDTH, HEIGHT] = [1600, 1000];

let browser = null;
const fail = (message) => {
  console.error(`FAIL: ${message}`);
  if (browser) browser.kill();
  process.exit(1);
};
setTimeout(() => fail("timed out"), 300000);

/// A browser driven over a WebSocket: `evaluate(expression)` answers the expression's value (through JSON), `shot`
/// saves the page's picture, the mouse verbs act at client pixels, and `problems` gathers the page's errors.
async function chromium() {
  const gpu = process.env.SCENE_GPU ? ["--use-angle=vulkan", "--enable-features=Vulkan", "--ignore-gpu-blocklist"] : ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"];
  browser = spawn(process.env.CHROMIUM || "chromium", ["--headless=new", "--no-sandbox", ...gpu, "--remote-debugging-port=0", `--user-data-dir=${profile}`, `--window-size=${WIDTH},${HEIGHT}`, "about:blank"], { stdio: ["ignore", "ignore", "pipe"] });
  const port = await new Promise((resolve) => {
    let text = "";
    browser.stderr.on("data", (chunk) => {
      text += chunk;
      const m = /DevTools listening on (ws:\/\/[^\s]+)/.exec(text);
      if (m) resolve(new URL(m[1]).port);
    });
  });
  const targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
  const socket = new WebSocket(targets.find((t) => t.type === "page").webSocketDebuggerUrl);
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
  await new Promise((resolve) => (socket.onopen = resolve));
  await send("Runtime.enable");
  await send("Page.enable");
  const mouse = (type, x, y, extra = {}) => send("Input.dispatchMouseEvent", { type, x, y, button: "none", buttons: 0, ...extra });
  return {
    name: "Chromium", problems,
    navigate: (to) => send("Page.navigate", { url: to }),
    evaluate: async (expression) => (await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true })).result.value,
    shot: async (name) => fs.writeFileSync(path.join(out, name), Buffer.from((await send("Page.captureScreenshot", { format: "png" })).data, "base64")),
    wheel: (x, y, deltaY) => mouse("mouseWheel", x, y, { deltaX: 0, deltaY }),
    move: (x, y) => mouse("mouseMoved", x, y),
    async drag(x1, y1, x2, y2, shift) {
      const modifiers = shift ? 8 : 0;
      await mouse("mousePressed", x1, y1, { button: "left", buttons: 1, clickCount: 1, modifiers });
      await mouse("mouseMoved", x2, y2, { button: "left", buttons: 1, modifiers });
      await mouse("mouseReleased", x2, y2, { button: "left", clickCount: 1, modifiers });
    },
    async click(x, y) {
      await mouse("mousePressed", x, y, { button: "left", buttons: 1, clickCount: 1 });
      await mouse("mouseReleased", x, y, { button: "left", clickCount: 1 });
    },
  };
}

async function gecko() {
  browser = spawn(process.env.FIREFOX || "firefox", ["--headless", "--remote-debugging-port=0", "--profile", profile, "--width", String(WIDTH), "--height", String(HEIGHT), "about:blank"], { stdio: ["ignore", "ignore", "pipe"] });
  const address = await new Promise((resolve) => {
    let text = "";
    browser.stderr.on("data", (chunk) => {
      text += chunk;
      const m = /WebDriver BiDi listening on (ws:\/\/[^\s]+)/.exec(text);
      if (m) resolve(m[1]);
    });
  });
  const socket = new WebSocket(`${address}/session`);
  let id = 0;
  const waiting = new Map();
  const problems = [];
  socket.onmessage = ({ data }) => {
    const message = JSON.parse(data);
    if (message.id) waiting.get(message.id)?.(message);
    if (message.method === "log.entryAdded" && message.params.level === "error") problems.push(message.params.text);
  };
  const send = (method, params = {}) => new Promise((resolve) => {
    waiting.set(++id, (message) => {
      if (message.type === "error") fail(`${method}: ${message.error}: ${message.message}`);
      resolve(message.result);
    });
    socket.send(JSON.stringify({ id, method, params }));
  });
  await new Promise((resolve) => (socket.onopen = resolve));
  await send("session.new", { capabilities: {} });
  const context = (await send("browsingContext.getTree", {})).contexts[0].context;
  await send("session.subscribe", { events: ["log.entryAdded"] });
  const pointer = (actions) => send("input.performActions", { context, actions: [{ type: "pointer", id: "mouse", parameters: { pointerType: "mouse" }, actions }] });
  const key = (type) => send("input.performActions", { context, actions: [{ type: "key", id: "keys", actions: [{ type, value: "" }] }] }); // shift
  const at = (x, y) => ({ type: "pointerMove", x: Math.round(x), y: Math.round(y) });
  return {
    name: "Firefox", problems,
    navigate: (to) => send("browsingContext.navigate", { context, url: to, wait: "none" }),
    async evaluate(expression) {
      // Through JSON, so that a value comes back as the page had it whatever its shape.
      const answer = await send("script.evaluate", { expression: `(async () => JSON.stringify(await (${expression})))()`, target: { context }, awaitPromise: true });
      if (answer.type !== "success") {
        problems.push(answer.exceptionDetails ? answer.exceptionDetails.text : "an evaluation threw");
        return undefined;
      }
      return answer.result.type === "string" ? JSON.parse(answer.result.value) : undefined;
    },
    shot: async (name) => fs.writeFileSync(path.join(out, name), Buffer.from((await send("browsingContext.captureScreenshot", { context })).data, "base64")),
    wheel: (x, y, deltaY) => send("input.performActions", { context, actions: [{ type: "wheel", id: "wheel", actions: [{ type: "scroll", x: Math.round(x), y: Math.round(y), deltaX: 0, deltaY }] }] }),
    move: (x, y) => pointer([at(x, y)]),
    async drag(x1, y1, x2, y2, shift) {
      if (shift) await key("keyDown");
      await pointer([at(x1, y1), { type: "pointerDown", button: 0 }, at((x1 + x2) / 2, (y1 + y2) / 2), at(x2, y2), { type: "pointerUp", button: 0 }]);
      if (shift) await key("keyUp");
    },
    click: (x, y) => pointer([at(x, y), { type: "pointerDown", button: 0 }, { type: "pointerUp", button: 0 }]),
  };
}

(async () => {
  const page = await (firefox ? gecko() : chromium());
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
  if (problems.length) fail(problems.join("\n"));
  console.log(`ok: screenshots in ${out}`);
  browser.kill();
  process.exit(0);
})();
