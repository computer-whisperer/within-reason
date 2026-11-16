// The 3D view following a game while it is played: the arena's follower appends a slice at a time to the match's
// game.tape (crates/arena/src/capture.rs) and the page reads the footer again as it polls. Start a game slow enough to
// watch, then this, which waits for the tape to appear:
//   target/release/arena --our-ai WRDistilled --matches 1 --speed 2 --max-minutes 6 --label live-test &
//   run/view_match.py run/matches --no-browser --port 8163 &
//   node viewer/test/live.js "http://127.0.0.1:8163/?match=matches/<batch>/00/" <out dir>
// It checks that the scene opens on the growing tape, that the tape grows while the page watches with the playhead on
// its newest frame ("follow live"), and that at the game's end the page takes the tape packed whole (no longer
// growing, as long or longer) and draws from it. It fails on any page exception or console error. SCENE_BROWSER=firefox
// drives Firefox; run both.
"use strict";
const fs = require("fs");

const url = process.argv[2];
const out = process.argv[3];
if (!url || !out) {
  console.error("usage: [SCENE_BROWSER=firefox] node viewer/test/live.js <match url> <out dir>");
  process.exit(2);
}
const firefox = process.env.SCENE_BROWSER === "firefox";
fs.mkdirSync(out, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let page = null;
const fail = (message) => {
  console.error(`FAIL: ${message}`);
  if (page) page.kill();
  process.exit(1);
};
setTimeout(() => fail("timed out"), 20 * 60 * 1000);

(async () => {
  // The match's files as the server lists them, until its tape is there.
  const match = new URL(url).searchParams.get("match");
  const listing = new URL(`/${match.replace(/\/?$/, "/")}index.json`, url);
  let waited = 0;
  for (;;) {
    const files = await fetch(listing).then((r) => (r.ok ? r.json() : null)).then((j) => (j ? j.files || [] : []), () => []);
    if (files.some((f) => (f.name || f) === "game.tape")) break;
    if (waited++ > 300) fail(`no game.tape in ${listing} after five minutes`);
    await sleep(1000);
  }
  page = await require("./driver").open({ firefox, out, fail });
  const { evaluate, shot, problems } = page;
  const check = (what) => {
    if (problems.length) fail(`${what}: ${problems.join("\n")}`);
  };
  await page.navigate(url);
  const now = () => evaluate(`(() => { const v = window.viewer, s = v && v.scene, t = s && s.tape;
    return t ? { growing: !!t.growing, last: t.lastFrame, slices: t.slices.length, frame: v.frame, drawn: s.stats.ms != null, units: s.stats.units || 0, loading: !!s.stats.loading, follow: document.getElementById("follow").checked } : null; })()`);
  let first = null;
  for (let i = 0; i < 300 && !(first && first.drawn); i++) {
    await sleep(200);
    first = await now();
  }
  check("opening");
  if (!first || !first.drawn) fail(`the scene did not open on the growing tape: ${JSON.stringify(first)}`);
  if (!first.growing) fail(`the tape was already packed whole when the page opened it (the game ended first): ${JSON.stringify(first)}`);
  console.log(`${page.name}; opened on a growing tape: ${first.slices} slices to ${(first.last / 1800).toFixed(2)} min, the playhead at ${(first.frame / 1800).toFixed(2)}, following ${first.follow}`);
  // While it grows: the tape's end moves on, the playhead with it.
  const seen = [first];
  let shotTaken = false;
  for (let i = 0; i < 600; i++) {
    await sleep(2000);
    const state = await now();
    check("following");
    if (!state) fail("the scene went away");
    seen.push(state);
    if (!shotTaken && seen.filter((s) => s.last > first.last).length >= 3) {
      await shot("growing.png");
      shotTaken = true;
    }
    if (!state.growing) break;
  }
  const end = seen[seen.length - 1];
  const growths = seen.filter((s, i) => i && s.last > seen[i - 1].last).length;
  if (end.growing) fail(`the tape was still growing after twenty minutes: ${JSON.stringify(end)}`);
  if (growths < 2) fail(`the tape grew ${growths} times while the page watched: ${JSON.stringify(seen.map((s) => s.last))}`);
  const lagging = seen.filter((s) => s.growing && s.follow && s.last - s.frame > 30);
  if (lagging.length > 2) fail(`the playhead stayed behind the tape's end while following: ${JSON.stringify(lagging.slice(0, 4))}`);
  if (end.last < seen.reduce((m, s) => Math.max(m, s.last), 0)) fail(`the tape packed whole is shorter than the growing one was: ${JSON.stringify(end)}`);
  // The tape packed whole: a seek into the middle draws.
  await evaluate(`window.viewer.seek(${Math.round(end.last / 2)})`);
  let settled = null;
  for (let i = 0; i < 100; i++) {
    await sleep(200);
    settled = await evaluate(`(() => { const s = window.viewer.scene; return { loading: !!s.stats.loading, pending: !!s.pending, units: s.stats.units || 0, frame: s.frame }; })()`);
    if (settled && !settled.loading && !settled.pending && settled.units) break;
  }
  check("after the game");
  if (!settled || settled.loading || !settled.units) fail(`the tape packed whole did not draw its middle: ${JSON.stringify(settled)}`);
  await shot("packed.png");
  const lastGrowing = (seen.filter((s) => s.growing).pop() || first).last;
  console.log(`grew ${growths} times while watched (${seen.length} looks, ${(first.last / 1800).toFixed(2)} -> ${(lastGrowing / 1800).toFixed(2)} min while growing); packed whole at ${(end.last / 1800).toFixed(2)} min, ${end.slices} slices; its middle drawn with ${settled.units} units`);
  console.log(`ok: screenshots in ${out}`);
  page.kill();
  process.exit(0);
})().catch((e) => fail(e.stack || String(e)));
