// Drives the real page in headless Chromium over the DevTools protocol (node >= 22 for the global WebSocket):
//   run/view_match.py <matches dir | match dir> --no-browser &   then   node viewer/test/browser.js http://127.0.0.1:8137/ [screenshot.png]
// Given the server's root it checks the match browser and opens its first match; given a match (`...?match=...`)
// it loads it, follows it if it is live, scrubs, plays, hovers, zooms, selects a unit, narrows to a seat and back,
// and reads every panel the match has (commander, player, hands with a gate and an opened actor, cost); it fails
// on any page exception or console error. With a second argument it saves a screenshot of the page.
"use strict";
const { spawn } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const url = process.argv[2] || "http://127.0.0.1:8137/";
const screenshot = process.argv[3] || null;
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "wr-viewer-"));
const chrome = spawn(process.env.CHROMIUM || "chromium", [
  "--headless", "--disable-gpu", "--no-sandbox", "--remote-debugging-port=0", `--user-data-dir=${profile}`, "--window-size=1600,1000", "about:blank",
], { stdio: ["ignore", "ignore", "pipe"] });

const fail = (message) => {
  console.error(`FAIL: ${message}`);
  chrome.kill();
  process.exit(1);
};
setTimeout(() => fail("timed out"), 180000);

let stderr = "";
chrome.stderr.on("data", async (chunk) => {
  stderr += chunk;
  const m = /DevTools listening on (ws:\/\/[^\s]+)/.exec(stderr);
  if (!m || chrome.started) return;
  chrome.started = true;
  const port = new URL(m[1]).port;
  const targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
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
    if (message.method === "Runtime.consoleAPICalled" && message.params.type === "error") problems.push(JSON.stringify(message.params.args));
  };
  const send = (method, params = {}) => new Promise((resolve) => {
    waiting.set(++id, resolve);
    socket.send(JSON.stringify({ id, method, params }));
  });
  const evaluate = async (expression) => (await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true })).result.value;
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const mouse = (type, x, y, buttons = 0) => send("Input.dispatchMouseEvent", { type, x, y, button: buttons ? "left" : "none", buttons, clickCount: type === "mouseMoved" ? 0 : 1 });
  const centre = (selector, fx = 0.5, fy = 0.5) => evaluate(`(() => { const r = document.querySelector(${JSON.stringify(selector)}).getBoundingClientRect(); return [r.left + r.width * ${fx}, r.top + r.height * ${fy}]; })()`);

  socket.onopen = async () => {
    await send("Runtime.enable");
    await send("Page.enable");
    await send("Page.navigate", { url });
    for (let i = 0; i < 600 && !(await evaluate("document.body.dataset.loaded || document.body.dataset.browser")); i++) await sleep(100);
    const report = {};
    // The match browser (the server given the matches directory): rows for every batch, a link per match; the first
    // match with a record opens the viewer.
    if (await evaluate("document.body.dataset.browser === '1'")) {
      report.browser = await evaluate("({ rows: document.querySelectorAll('#browser table.batches tr').length - 1, links: document.querySelectorAll('#browser a.match').length })");
      if (!report.browser.rows || !report.browser.links) fail(`the match browser is empty: ${JSON.stringify(report.browser)}`);
      await evaluate("document.getElementById('browser').querySelector('input').value = 'zzz-no-such-batch'; document.getElementById('browser').querySelector('input').dispatchEvent(new Event('input'))");
      report.browser.filtered = await evaluate("[...document.querySelectorAll('#browser table.batches tr')].filter((tr) => !tr.hidden).length - 1");
      if (report.browser.filtered !== 0) fail("the browser's filter did not narrow the list");
      const href = await evaluate("(document.querySelector('#browser a.match.win, #browser a.match.loss') || document.querySelector('#browser a.match:not(.norecord)')).getAttribute('href')");
      await send("Page.navigate", { url: new URL(href, url).href });
      for (let i = 0; i < 600 && !(await evaluate("document.body.dataset.loaded")); i++) await sleep(100);
    }
    const loaded = await evaluate("document.body.dataset.loaded");
    if (!loaded) fail(`the match did not load: ${await evaluate("document.getElementById('status').textContent")}; page errors: ${problems.join(" | ")}`);
    report.loaded = loaded;
    report.subtitle = await evaluate("document.getElementById('subtitle').textContent");

    // Scrub: a click at 60 % of the timeline lands at about 60 % of the game.
    // A match still being played: the page follows it, and stops following once the reader scrubs.
    if (await evaluate("!viewer.match.result && !viewer.match.seats.every((s) => s.end)")) {
      const first = await evaluate("viewer.match.lastFrame");
      if (!(await evaluate("document.getElementById('follow').checked && viewer.frame === viewer.match.lastFrame"))) fail("a live match does not open on its newest sample");
      for (let i = 0; i < 450 && (await evaluate("viewer.match.lastFrame")) === first; i++) await sleep(100);
      report.live = { from: first, to: await evaluate("viewer.match.lastFrame"), following: await evaluate("viewer.frame === viewer.match.lastFrame") };
      if (!(report.live.to > first)) fail("the live match did not grow in 45 s (is it paused for a long turn, or over?)");
      if (!report.live.following) fail("the playhead did not follow the live match");
    }
    const [tx, ty] = await centre("#timeline", 0.6, 0.5);
    await mouse("mousePressed", tx, ty, 1);
    await mouse("mouseReleased", tx, ty);
    const share = await evaluate("viewer.frame / viewer.match.lastFrame");
    if (!(share > 0.5 && share < 0.7)) fail(`scrub landed at ${share}`);
    report.scrubbedTo = await evaluate("document.getElementById('clock').textContent");
    await mouse("mouseMoved", tx + 40, ty);
    report.chartTooltip = await evaluate("document.getElementById('chart-tooltip').hidden ? null : document.getElementById('chart-tooltip').textContent");
    if (!report.chartTooltip) fail("no timeline tooltip");

    // Play advances the playhead; pause stops it.
    const before = await evaluate("viewer.frame");
    await evaluate("document.getElementById('play').click()");
    await sleep(700);
    await evaluate("document.getElementById('play').click()");
    const after = await evaluate("viewer.frame");
    if (!(after > before)) fail(`play did not advance (${before} -> ${after})`);
    report.playedFrames = Math.round(after - before);

    // Hover one of our units on the map.
    let [ux, uy] = await evaluate("(() => { const u = viewer.state.own[0], b = viewer.mapBox, r = document.getElementById('map').getBoundingClientRect(); return [r.left + b.x + u.x * b.scale, r.top + b.y + u.z * b.scale]; })()");
    await mouse("mouseMoved", ux, uy);
    report.mapTooltip = await evaluate("document.getElementById('tooltip').textContent");
    if (!/ours( \(t\d+\))?: /.test(report.mapTooltip)) fail(`map tooltip: ${report.mapTooltip}`);

    // Zoom about that unit with the wheel: the unit stays under the cursor, footprints and names appear from 4x;
    // a click selects it and opens the Unit tab; Escape clears; a double-click resets the zoom. The protocol
    // truncates event coordinates to whole pixels, so the cursor is put on whole pixels first (a real cursor is
    // its own anchor; a fractional one here drifted 10 px over four steps).
    await mouse("mouseMoved", Math.round(ux), Math.round(uy));
    [ux, uy] = [Math.round(ux), Math.round(uy)];
    for (let i = 0; i < 4; i++) await send("Input.dispatchMouseEvent", { type: "mouseWheel", x: ux, y: uy, deltaX: 0, deltaY: -400 });
    await sleep(100);
    const zoom = await evaluate("viewer.zoom");
    if (!(zoom >= 4)) fail(`the wheel did not zoom (${zoom})`);
    // The unit is still (nearly) under the cursor: on a tall map in a low pane a fitted pixel is 30 elmos, so the
    // whole-pixel cursor may sit a few zoomed pixels off the unit; the cursor then goes onto it.
    const [nx, ny] = await evaluate("(() => { const u = viewer.state.own[0], b = viewer.mapBox, r = document.getElementById('map').getBoundingClientRect(); return [r.left + b.x + u.x * b.scale, r.top + b.y + u.z * b.scale]; })()");
    if (Math.hypot(nx - ux, ny - uy) > 16) fail(`the zoom was not about the cursor: the unit moved ${Math.hypot(nx - ux, ny - uy).toFixed(1)} px`);
    [ux, uy] = [Math.round(nx), Math.round(ny)];
    await mouse("mouseMoved", ux, uy);
    const tipZoomed = await evaluate("document.getElementById('tooltip').textContent");
    if (!/ours( \(t\d+\))?: /.test(tipZoomed)) fail(`the unit did not stay under the cursor after zooming: ${tipZoomed}`);
    await mouse("mousePressed", ux, uy, 1);
    await send("Input.dispatchMouseEvent", { type: "mouseReleased", x: ux, y: uy, button: "left", clickCount: 1 });
    await sleep(100);
    const unit = await evaluate("({ selected: viewer.selected, tab: viewer.tab, facts: document.querySelectorAll('#unit .facts .k').length, history: document.querySelectorAll('#unit .history li').length })");
    if (unit.selected == null || unit.tab !== "unit" || !unit.facts) fail(`clicking a unit did not open the Unit tab: ${JSON.stringify(unit)}`);
    report.zoom = { zoom, unit, icons: await evaluate("viewer.icons ? Object.keys(viewer.icons).length : 0") };
    await send("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape" });
    if ((await evaluate("viewer.selected")) != null) fail("Escape did not clear the selection");
    await send("Input.dispatchMouseEvent", { type: "mousePressed", x: ux, y: uy, button: "left", clickCount: 2 });
    await send("Input.dispatchMouseEvent", { type: "mouseReleased", x: ux, y: uy, button: "left", clickCount: 2 });
    await sleep(100);
    if ((await evaluate("viewer.zoom")) !== 1) fail("a double-click did not reset the zoom");
    await send("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape" });

    await evaluate("document.querySelectorAll('#layers input').forEach((box) => box.click())");
    // The tabs: decisions with its filters, the log.
    await evaluate("document.querySelector('#tabs [data-tab=decisions]').click()");
    await evaluate("document.querySelectorAll('#decision-filters input').forEach((box) => box.click())");
    await evaluate("document.querySelectorAll('#decision-filters input').forEach((box) => box.click())");
    await send("Input.dispatchKeyEvent", { type: "keyDown", key: "ArrowLeft", code: "ArrowLeft" });
    report.panels = await evaluate("({ decisions: document.querySelectorAll('#decisions li').length, llmTurns: document.querySelectorAll('#decisions li.llm').length, stats: document.querySelectorAll('#now .stat').length, mapPixels: (() => { const c = document.getElementById('map'); const d = c.getContext('2d').getImageData(0, 0, c.width, c.height).data; let n = 0; for (let i = 0; i < d.length; i += 4) if (d[i + 3] > 0 && (d[i] > 60 || d[i + 1] > 60 || d[i + 2] > 60)) n++; return n; })() })");
    await evaluate("document.querySelector('#tabs [data-tab=log]').click()");
    report.panels.rules = await evaluate("document.querySelectorAll('#rules .rule').length");
    report.panels.buildOrder = await evaluate("document.querySelectorAll('#build-order li').length");
    if (!report.panels.stats || !report.panels.mapPixels || !report.panels.decisions || !report.panels.buildOrder) fail(`empty panels: ${JSON.stringify(report.panels)}`);

    // Several seats of ours: every seat is loaded, the selector narrows the page to one and widens it again.
    const seats = await evaluate("viewer.match.seats.length");
    report.seats = { loaded: seats, buttons: await evaluate("document.querySelectorAll('#seats button').length"), units: await evaluate("viewer.match.seats.map((s) => viewer.state.own.filter((u) => u.seat === s.index).length)") };
    if (seats > 1) {
      if (report.seats.buttons !== seats + 1) fail(`the seat selector has ${report.seats.buttons} buttons for ${seats} seats`);
      const all = await evaluate("document.querySelector('#now .stat b').textContent");
      await evaluate("document.querySelectorAll('#seats button')[1].click()");
      report.seats.scoped = await evaluate("({ scope: viewer.scope, lanes: viewer.laneList.map((l) => l.label), metal: document.querySelector('#now .stat b').textContent })");
      if (report.seats.scoped.scope !== 0) fail("clicking a seat did not narrow the page to it");
      await evaluate("document.querySelectorAll('#seats button')[0].click()");
      if ((await evaluate("viewer.scope")) !== null || (await evaluate("document.querySelector('#now .stat b').textContent")) !== all) fail("the page did not widen to all seats again");
    }

    // The commander: a direction in force, its turns in the list.
    if (await evaluate("!!viewer.match.commander")) {
      await evaluate("document.querySelector('#tabs [data-tab=commander]').click()");
      report.commander = await evaluate("({ plan: document.querySelectorAll('#commander .words.plan').length, seats: document.querySelectorAll('#commander .words.seat').length, signals: document.querySelectorAll('#commander .signal').length, text: document.getElementById('commander').innerText.length })");
      if (!report.commander.text) fail("the commander panel is empty");
      if ((await evaluate("WR.callsOf(viewer.match.commander, 'direct', viewer.frame).length")) && !report.commander.plan && !report.commander.seats) fail("a direction was written and the panel shows none");
    }
    // The player: with several players a line a seat, then one seat's packet in full.
    if (await evaluate("viewer.match.players.size > 0")) {
      await evaluate("document.querySelector('#tabs [data-tab=player]').click()");
      report.player = { overview: await evaluate("document.querySelectorAll('#player .words.seat').length") };
      if (seats > 1) await evaluate("document.querySelectorAll('#seats button')[1].click()");
      Object.assign(report.player, await evaluate("({ packet: (document.querySelector('#player .words.plan') || { textContent: '' }).textContent.length, lists: document.querySelectorAll('#player .list').length, text: document.getElementById('player').innerText.length })"));
      if (!report.player.text) fail("the player panel is empty");
      if ((await evaluate("!!WR.packetAt(viewer.match, WV.playerOf(viewer.match.seats[0]), viewer.frame)")) && !report.player.packet) fail("a packet is in force and the panel shows none");
    }
    // The hands: the index of every seat's log, the gate at the playhead with its layers and menus, an actor opened.
    if (await evaluate("!document.querySelector('#tabs [data-tab=hands]').hidden")) {
      for (let i = 0; i < 600 && !(await evaluate("viewer.match.seats.every((s) => s.jev)")); i++) await sleep(100);
      if (!(await evaluate("viewer.match.seats.every((s) => s.jev)"))) fail(`the hands' indexes did not arrive: ${await evaluate("document.getElementById('status').textContent")}`);
      if (seats > 1) await evaluate("document.querySelectorAll('#seats button')[1].click()");
      await evaluate("document.querySelector('#tabs [data-tab=hands]').click()");
      const hands = { version: await evaluate("viewer.match.seats[0].jev.index.version"), gates: await evaluate("viewer.match.seats[0].jev.index.gates.length") };
      if (hands.version >= 3 && hands.gates) {
        // The page may be playing: pause it, and go to a gate in the middle of those the log has.
        const gateAt = await evaluate("(() => { viewer.playing = false; const g = viewer.match.seats[0].jev.index.gates; const f = g[Math.floor(g.length / 2)]; viewer.seek(f); return f; })()");
        const started = Date.now();
        for (let i = 0; i < 100 && !(await evaluate("document.querySelectorAll('#hands .slot').length")); i++) await sleep(50);
        hands.gateMs = Date.now() - started;
        Object.assign(hands, await evaluate("({ head: document.getElementById('hands-head').textContent.slice(0, 160), layers: document.querySelectorAll('#hands table.grid tr').length, slots: document.querySelectorAll('#hands .slot').length, bars: document.querySelectorAll('#hands .bar').length })"));
        const clock = (f) => `${Math.floor(f / 1800)}:${String(Math.floor(f / 30) % 60).padStart(2, "0")}`;
        if (!hands.head.includes(`the gate of ${clock(gateAt)}`)) fail(`the hands panel shows another gate than the one sought (${clock(gateAt)}): ${hands.head}`);
        if (!hands.slots || hands.layers < 5) fail(`the hands panel is empty: ${JSON.stringify(hands)}`);
        // Open the actor with the most moves: its moves by state, its units ringed, the decision list narrowed to it.
        await evaluate("[...document.querySelectorAll('#hands .slot .id')].sort((a, b) => parseInt(b.parentNode.querySelector('.ask').textContent) - parseInt(a.parentNode.querySelector('.ask').textContent))[0].click()");
        await sleep(200);
        hands.actor = await evaluate("viewer.actor && viewer.actor.name");
        hands.moves = await evaluate("document.querySelectorAll('#hands .slot.selected .state').length + document.querySelectorAll('#hands .slot.selected details.group').length");
        hands.ringed = await evaluate("WV.actorUnits(viewer.actor).length");
        if (!hands.actor || !hands.moves) fail(`clicking an actor did not open its menu: ${JSON.stringify(hands)}`);
        await evaluate("(() => { const d = document.querySelector('#hands .slot.selected details.group'); if (d) d.open = true; })()");
        await sleep(100);
        hands.reasons = await evaluate("[...new Set([...document.querySelectorAll('#hands .slot.selected details.group summary')].map((s) => s.textContent.replace(/ \\(\\d+\\)$/, '')))]");
        // The gate before and the next one.
        await evaluate("document.querySelectorAll('#hands-head .gate-nav button')[1].click()");
        await sleep(400);
        hands.next = await evaluate("document.getElementById('hands-head').textContent.slice(0, 40)");
        await evaluate("document.querySelector('#tabs [data-tab=decisions]').click()");
        hands.narrowed = await evaluate("({ chip: !!document.querySelector('#decision-filters .chip'), rows: document.querySelectorAll('#decisions li.jev').length })");
        if (!hands.narrowed.chip) fail("the decision list was not narrowed to the actor in focus");
        await send("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape" });
        if ((await evaluate("viewer.actor")) != null) fail("Escape did not clear the actor in focus");
        // A unit of ours ties to its actor: the Unit tab names it and lists its plays.
        const tied = await evaluate("(() => { const u = viewer.state.own.find((u) => u.seat === 0 && ['builder', 'army', 'commander'].includes(viewer.match.defs[u.def].class)); if (!u) return null; viewer.select(u.id); return u.id; })()");
        if (tied != null) {
          for (let i = 0; i < 40 && /loading/.test(await evaluate("document.getElementById('unit').innerText")); i++) await sleep(50);
          hands.unit = await evaluate("({ title: (document.querySelector('#unit h2.call-title') || { textContent: '' }).textContent.slice(0, 80), plays: document.querySelectorAll('#unit .hist').length, button: !!document.querySelector('#unit button') })");
          await evaluate("viewer.select(null)");
        }
      } else {
        hands.notice = await evaluate("document.getElementById('hands').innerText.slice(0, 120)");
        if (!hands.notice) fail("an old or empty log shows no notice");
      }
      await evaluate("document.querySelector('#tabs [data-tab=cost]').click()");
      hands.cost = await evaluate("({ title: document.querySelector('#cost h2').innerText, tables: document.querySelectorAll('#cost table.grid').length, minutes: document.querySelectorAll('#cost table.grid:last-of-type tr').length - 1, families: document.querySelectorAll('#cost .bar.family').length })");
      if (!hands.cost.minutes || !hands.cost.families) fail(`the cost panel is empty: ${JSON.stringify(hands.cost)}`);
      report.hands = hands;
    }
    if (seats > 1) await evaluate("document.querySelectorAll('#seats button')[0].click()");
    await evaluate("document.getElementById('wide').click()");
    await evaluate("document.getElementById('compact').click()");
    report.modes = await evaluate("({ wide: document.body.dataset.wide, compact: document.body.dataset.compact })");
    await evaluate("document.getElementById('wide').click()");
    await evaluate("document.getElementById('compact').click()");
    if (screenshot) {
      // With the layers back on and the map hovered nowhere: the page as a reader sees it.
      await evaluate("document.querySelectorAll('#layers input').forEach((box) => box.click())");
      await mouse("mouseMoved", 2, 2);
      await sleep(100);
      const shot = await send("Page.captureScreenshot", { format: "png" });
      fs.writeFileSync(screenshot, Buffer.from(shot.data, "base64"));
      report.screenshot = screenshot;
    }
    if (problems.length) fail(`page errors: ${problems.join(" | ")}`);
    console.log(JSON.stringify(report, null, 1));
    console.log("PASS");
    chrome.kill();
    fs.rmSync(profile, { recursive: true, force: true });
    process.exit(0);
  };
}
