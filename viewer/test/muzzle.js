// The moving pieces checked against the engine's own shots (a tape with them, as captured by default): a shot leaves its gun at
// a muzzle piece, so where it began must lie on one of its owner's pieces as the reader composes them (turned and moved),
// and nearer than with the pieces at rest. A turn the wrong way round, a piece numbered wrong or a model space mirrored
// would put the muzzles elsewhere.
//   node viewer/test/muzzle.js "http://127.0.0.1:8163/?match=matches/<batch>/<NN>/"
// It looks at the shots just made at a frame every 37 from the third game minute, up to 400 of them, and fails when the
// median distance of all of them is over 3 elmos or not under half the one at rest, printing both by unit type.
// (Where a shot began is where the widget first saw it, a frame after it left: the nearer of its head and its tail.)
"use strict";
const url = process.argv[2];
if (!url) {
  console.error("usage: [SCENE_BROWSER=firefox] node viewer/test/muzzle.js <match url>");
  process.exit(2);
}
let page = null;
const fail = (message) => {
  console.error(`FAIL: ${message}`);
  if (page) page.kill();
  process.exit(1);
};
setTimeout(() => fail("timed out"), 10 * 60 * 1000);

/// Runs in the page.
async function measure() {
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const v = window.viewer, s = v.scene, t = s.tape;
  const origins = new Map();
  const originsOf = (def) => {
    if (origins.has(def.id)) return origins.get(def.id);
    const p = def.pieces;
    if (!p) return null;
    const o = [];
    for (let i = 0; i < p.parents.length; i++) {
      const q = p.parents[i];
      const b = q > 0 ? o[q - 1] : [0, 0, 0];
      o.push([0, 1, 2].map((k) => b[k] + p.offsets[i][k]));
    }
    origins.set(def.id, o);
    return o;
  };
  const composed = [], rest = [], byType = {};
  for (let f = 3600; f < t.lastFrame && composed.length < 400; f += 37) {
    v.seek(f);
    for (let i = 0; i < 60 && (s.pending || s.stats.loading || Math.abs(s.frame - f) > 1); i++) await sleep(30);
    const st = t.frame(f);
    if (!st) continue;
    const slotOf = new Map();
    for (let i = 0; i < st.units.length; i += 16) slotOf.set(st.units[i + 13], i);
    for (let i = 0; i < st.projectiles.length; i += 12) {
      if (st.projectiles[i + 8] > 0.5) continue; // just made
      const at = slotOf.get(st.projectiles[i + 10] - 1); // (the owner's row + 1; 0 none)
      if (at === undefined) continue;
      const block = st.pieceAt[at / 16];
      if (block < 0) continue;
      const def = t.unitDefs.get(st.units[at + 14]);
      const o = def && originsOf(def);
      if (!o) continue;
      const U = Array.from(st.units.slice(at, at + 9));
      const front = U.slice(3, 6), up = U.slice(6, 9);
      const right = [up[1] * front[2] - up[2] * front[1], up[2] * front[0] - up[0] * front[2], up[0] * front[1] - up[1] * front[0]];
      const world = (m) => [0, 1, 2].map((k) => U[k] + right[k] * m[0] + up[k] * m[1] + front[k] * m[2]);
      const head = Array.from(st.projectiles.slice(i, i + 3)), tail = Array.from(st.projectiles.slice(i + 3, i + 6));
      const near = (w) => Math.min(Math.hypot(...w.map((x, k) => x - head[k])), Math.hypot(...w.map((x, k) => x - tail[k])));
      let best = Infinity, bestRest = Infinity;
      const base = block * st.pieceStride * 12;
      for (let p = 0; p < o.length; p++) {
        const T = st.pieces.subarray(base + p * 12, base + p * 12 + 12);
        const ms = [0, 1, 2].map((k) => T[k] * o[p][0] + T[3 + k] * o[p][1] + T[6 + k] * o[p][2] + T[9 + k]);
        best = Math.min(best, near(world(ms)));
        bestRest = Math.min(bestRest, near(world(o[p])));
      }
      composed.push(best);
      rest.push(bestRest);
      (byType[def.name] = byType[def.name] || []).push([best, bestRest]);
    }
  }
  const quantile = (a, q) => {
    const b = [...a].sort((x, y) => x - y);
    return Math.round(b[Math.min(b.length - 1, Math.floor(b.length * q))] * 10) / 10;
  };
  return {
    shots: composed.length,
    composed: { median: quantile(composed, 0.5), p90: quantile(composed, 0.9) },
    rest: { median: quantile(rest, 0.5), p90: quantile(rest, 0.9) },
    byType: Object.fromEntries(Object.entries(byType).map(([k, l]) => [k, { n: l.length, composed: quantile(l.map((x) => x[0]), 0.5), rest: quantile(l.map((x) => x[1]), 0.5) }])),
  };
}

(async () => {
  page = await require("./driver").open({ firefox: process.env.SCENE_BROWSER === "firefox", out: require("os").tmpdir(), fail });
  await page.navigate(url);
  for (let i = 0; i < 300 && !(await page.evaluate("!!(window.viewer && window.viewer.scene && window.viewer.scene.tape)")); i++) await new Promise((r) => setTimeout(r, 100));
  if (!(await page.evaluate("!!window.viewer.scene.tape.head.pieces"))) fail("the tape was captured without the moving pieces (run/tape.py --no-pieces, or an arena game without --pieces)");
  const result = await page.evaluate(`(${measure.toString()})()`);
  if (page.problems.length) fail(page.problems.join("\n"));
  if (!result) fail("the measure gave nothing");
  console.log(`${page.name}: ${result.shots} shots; where each began against the nearest of its owner's pieces, composed: median ${result.composed.median} elmos (90 % under ${result.composed.p90}); at rest: ${result.rest.median} (${result.rest.p90})`);
  for (const [type, r] of Object.entries(result.byType)) console.log(`  ${type}: ${r.n} shots, composed ${r.composed}, at rest ${r.rest}`);
  if (!(result.shots >= 20)) fail("fewer than twenty shots found");
  if (!(result.composed.median <= 3 && result.composed.median < result.rest.median / 2)) fail("the shots do not leave from the composed pieces");
  console.log("ok");
  page.kill();
  process.exit(0);
})().catch((e) => fail(e.stack || String(e)));
