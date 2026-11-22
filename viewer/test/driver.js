// The browsers the viewer's tests drive (node >= 22 for the global WebSocket): headless Chromium over the DevTools
// protocol, or headless Firefox over WebDriver BiDi. `open({ firefox, out, width, height, fail })` answers a page:
// `evaluate(expression)` answers the expression's value (through JSON), `shot(name)` saves the page's picture into
// `out`, the mouse verbs act at client pixels, `problems` gathers the page's errors, `kill()` ends the browser.
// CHROMIUM and FIREFOX name the binaries; SCENE_GPU=1 has Chromium use the machine's GPU instead of its software
// renderer.
"use strict";
const { spawn } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

async function open({ firefox, out, width = 1600, height = 1000, fail }) {
  const WIDTH = width, HEIGHT = height;
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), "wr-scene-"));
  let browser = null;
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

  const page = await (firefox ? gecko() : chromium());
  page.kill = () => browser && browser.kill();
  // The profile is the browser's scratch, some hundreds of megabytes in the temporary directory (memory, where that is
  // a tmpfs): it goes with the test's process, however the test ends. (Until 2026-11-22 every run left its own.)
  process.on("exit", () => {
    if (browser) browser.kill("SIGKILL");
    try {
      fs.rmSync(profile, { recursive: true, force: true, maxRetries: 10, retryDelay: 100 });
    } catch (_) {}
  });
  return page;
}

module.exports = { open };
