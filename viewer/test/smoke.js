// Smoke test for the viewer's model: node viewer/test/smoke.js <match dir>
// Loads every seat of a real match as the page does (records, transcripts, truth) and checks the invariants the UI
// relies on; reads the Jev logs through run/viewer_index.py (the server's index) and checks that the gate view
// accounts for every move of every menu. Prints a report.
"use strict";
const fs = require("fs");
const path = require("path");
const assert = require("assert");
const { execFileSync } = require("child_process");
const WR = require("../record.js");

const dir = process.argv[2];
if (!dir) throw new Error("usage: node viewer/test/smoke.js <match dir>");
const files = fs.readdirSync(dir);
const read = (name) => (fs.existsSync(path.join(dir, name)) ? fs.readFileSync(path.join(dir, name), "utf8") : null);
const batch = fs.existsSync(path.join(dir, "..", "batch.json")) ? JSON.parse(fs.readFileSync(path.join(dir, "..", "batch.json"), "utf8")) : null;

// Every seat: the headers first (the teams decide which ally rows are dropped), then the rows.
const records = files.filter((f) => /^record-.*\.jsonl$/.test(f)).sort();
assert(records.length, "no record-*.jsonl");
const texts = records.map(read);
const teams = new Set(texts.map((t) => JSON.parse(t.slice(0, t.indexOf("\n"))).team));
const match = WR.newMatch();
match.batch = batch;
for (const text of texts) WR.addSeat(match, WR.parseRecord(text, teams));
WR.settle(match);
assert.strictEqual(match.seats.length, records.length, "every record is a seat");
assert(match.seats.every((s, i) => s.index === i && s.name === `t${s.team}`), "seats are in team order and named by team");

for (const seat of match.seats) {
  assert.strictEqual(seat.header.version, 1);
  assert(seat.samples.length > 10, "samples");
  assert.strictEqual(seat.samples.length, seat.series.length, "a series entry per sample");
  assert(seat.samples.every((s, i, all) => i === 0 || s.f > all[i - 1].f), "samples are in frame order");
  for (const list of [seat.events, seat.decisions, seat.commands, seat.intents]) {
    assert(list.every((r, i, all) => i === 0 || r.f >= all[i - 1].f), "records are in frame order");
  }
  const defs = seat.defs.length;
  for (const s of seat.samples) for (const u of s.own) assert(u.length === 6 && u[1] >= 0 && u[1] < defs, "own unit row");
  for (const s of seat.samples) for (const e of s.en) assert(e.length === 6 && e[1] >= -1 && e[1] < defs, "enemy row");
  // No ally row of a seat we loaded survives: those units are that seat's own.
  for (const s of seat.samples) for (const u of s.al || []) assert(!teams.has(u[4]), "an ally row of a loaded seat was kept");
}

// The match's state: every seat's units carry their seat, no unit twice, enemies once each.
const mid = Math.floor(match.lastFrame / 2);
const state = WR.stateAt(match, mid);
assert.strictEqual(new Set(state.own.map((u) => u.id)).size, state.own.length, "a unit of ours is in one seat");
assert(state.own.every((u) => match.seats[u.seat]), "own units carry their seat");
assert.strictEqual(new Set(state.enemies.map((u) => u.id)).size, state.enemies.length, "an enemy is drawn once");
assert.strictEqual(WR.stateAt(match, -5).own.length, 0);
const total = WR.totalsAt(match, null, mid);
const perSeat = match.seats.map((s) => WR.totalsAt(match, s.index, mid));
assert.strictEqual(total.extractors, perSeat.reduce((n, t) => n + t.extractors, 0), "the side's extractors are the seats' summed");
assert(Math.abs(total.metalIncome - perSeat.reduce((n, t) => n + t.metalIncome, 0)) < 1e-6, "the side's income is the seats' summed");

// Interpolation stays between two samples of a seat.
const seat0 = match.seats[0];
const [a, b] = [seat0.samples[20], seat0.samples[21]];
const between = WR.seatStateAt(seat0, (a.f + b.f) / 2);
assert.strictEqual(between.own.length, a.own.length);
for (const u of between.own) {
  const next = b.own.find((r) => r[0] === u.id);
  const prev = a.own.find((r) => r[0] === u.id);
  if (next) assert(Math.abs(u.x - (prev[2] + next[2]) / 2) < 1, "midpoint");
}
assert.strictEqual(WR.gridName(match, 0, 0), "A1");
assert.strictEqual(WR.gridName(match, match.header.map.width, match.header.map.height), String.fromCharCode(64 + match.header.grid.columns) + match.header.grid.rows);

// A killed match: cut the file mid-line; everything before the cut must still load.
const cut = WR.parseRecord(texts[0].slice(0, Math.floor(texts[0].length / 2)), teams);
assert(cut.badLines <= 1 && cut.samples.length > 5 && cut.result === null, "truncated record loads");

// Rows arriving in pieces (a live match) give the same seat as the whole text.
const lines = WR.parseLines(texts[0]).values;
const piecewise = WR.newSeat();
for (let i = 0; i < lines.length; i += 997) WR.addRecordRows(piecewise, lines.slice(i, i + 997), teams);
assert.strictEqual(piecewise.samples.length, seat0.samples.length, "piecewise samples");
assert.deepStrictEqual(piecewise.series[piecewise.series.length - 1], seat0.series[seat0.series.length - 1], "piecewise series");

// Standing orders and a unit's history.
const orders = WR.ordersAt(seat0, match.lastFrame);
assert(orders instanceof Map, "orders");
assert(WR.ordersAt(seat0, 60 * WR.FPS) instanceof Map, "orders at 1:00, the cache starting over");
const first = seat0.samples[20].own[0][0];
const history = WR.unitHistory(match, first);
assert(history.commands.every((o, i, all) => i === 0 || o.f >= all[i - 1].f), "history in order");
assert(history.events.every((e) => e.u === first || e.by === first), "history is the unit's");
assert(WR.facings(match) instanceof Map, "facings");

const lanes = WR.lanes(match, null);
const kills = lanes.kills.map((e) => e.u);
assert.strictEqual(new Set(kills).size, kills.length, "a kill every seat saw is one kill");
const report = {
  seats: match.seats.map((s) => ({ name: s.name, samples: s.samples.length, events: s.events.length, decisions: s.decisions.length, last: WR.clock(s.lastFrame) })),
  sides: WR.sides(match), lastClock: WR.clock(match.lastFrame), result: match.result && match.result.result.outcome,
  lanes: Object.fromEntries(Object.entries(lanes).map(([k, v]) => [k, v.length])), rulesInMinute5: WR.rulesInMinute(match, null, 5 * 60 * WR.FPS).slice(0, 3),
  totalsAtMid: { clock: WR.clock(mid), extractors: total.extractors, armyMetal: total.armyMetal, metalIncome: Number(total.metalIncome.toFixed(1)) },
  chat: WR.chatUpTo(match, match.lastFrame).length,
};

// The opponent's truth: one file (the seats write the same).
const truthName = files.filter((f) => /^truth-.*\.jsonl$/.test(f)).sort()[0];
if (truthName) {
  WR.addTruthRows(match, WR.parseLines(read(truthName)).values);
  assert(match.truth.length > 0 && match.truth.every((t, i, all) => i === 0 || t.f >= all[i - 1].f), "truth in frame order");
  const last = match.truth[match.truth.length - 1];
  report.truth = { file: truthName, rows: match.truth.length, lastArmy: last.army, lastArmyMetal: last.armyMetal, lastExtractors: last.extractors };
}

// The transcripts: the players' (one per seat, or one for all) and the commander's.
for (const name of files.filter((f) => /^strategist-\d+\.jsonl$/.test(f)).sort()) {
  match.players.set(Number(/\d+/.exec(name)[0]), WR.parseTranscript(read(name), "player"));
}
if (files.includes("commander.jsonl")) match.commander = WR.parseTranscript(read("commander.jsonl"), "commander");
match.realtime = WR.inferRealtime(match);
report.realtime = match.realtime;
report.players = [...match.players].map(([ai, t]) => {
  assert(t.turns.every((turn, i, all) => Number.isFinite(turn.f) && Array.isArray(turn.calls) && (i === 0 || turn.f >= all[i - 1].f)), "turns in frame order");
  const packet = WR.packetAt(match, t, match.lastFrame);
  const early = WR.packetAt(match, t, 0);
  assert(!early || early.from <= 0, "no packet is in force before its turn ended");
  return {
    ai, turns: t.turns.length, calls: t.turns.reduce((n, turn) => n + turn.calls.length, 0), woken: t.turns.filter((turn) => turn.wake).length,
    packetChars: packet ? packet.text.length : 0, packetAt: packet ? WR.clock(packet.from) : null,
    lists: WR.latestByKey(t, "queue", match.lastFrame).size, produce: WR.latestByKey(t, "produce", match.lastFrame).size, notes: WR.callsOf(t, "note", match.lastFrame).length,
  };
});
if (match.commander) {
  const t = match.commander;
  const direction = WR.directionAt(t, match.lastFrame);
  const marks = WR.marksAt(match, t, match.lastFrame);
  const signals = WR.signalsAt(t, match.lastFrame);
  const map = match.header.map;
  assert(marks.every((m) => m.x >= 0 && m.x <= map.width && m.z >= 0 && m.z <= map.height), "marks are on the map");
  assert(signals.every((s) => Number.isFinite(s.at)), "signals have a frame");
  const directs = WR.callsOf(t, "direct", match.lastFrame);
  if (directs.length) assert(direction.plan || Object.keys(direction.seats).length, "a direction is in force after a direct call");
  assert(!WR.directionAt(t, -1).plan, "no direction before the first turn");
  report.commander = {
    turns: t.turns.length, directs: directs.length, planAt: direction.plan ? WR.clock(direction.plan.f) : null, seats: Object.keys(direction.seats),
    marks: marks.map((m) => `${m.name} ${WR.gridName(match, m.x, m.z)}`), signals: signals.map((s) => `${s.name} ${s.given ? "given" : "set"} ${WR.clock(s.at)}`),
    notes: WR.callsOf(t, "note", match.lastFrame).length, says: WR.callsOf(t, "say", match.lastFrame).length,
  };
}

// The Jev logs through the server's index: the summary, a gate in the middle of the game, the picture there.
const indexer = path.join(__dirname, "..", "..", "run", "viewer_index.py");
const python = (args) => execFileSync("python3", [indexer, ...args], { maxBuffer: 1 << 28, encoding: "utf8" });
const jevFiles = files.filter((f) => /^jev-\d+\.jsonl$/.test(f)).sort();
report.jev = [];
for (const name of jevFiles) {
  const file = path.join(dir, name);
  const summary = python([file]);
  const gate = JSON.parse(python([file, "--gate", String(mid)]));
  const entry = { file: name, summary: summary.split("\n")[0].replace(`${file}: `, "") };
  if (gate.pass) {
    const view = WR.gateView(gate);
    assert.strictEqual(view.actors.length, gate.pass.menus.length, "an actor per menu");
    const moves = gate.pass.menus.reduce((n, m) => n + m.moves.length, 0);
    const t = view.totals;
    assert.strictEqual(t.moves, moves, "every move is in the view");
    assert.strictEqual(t.course + t.asked + t.audit + t.news + t.fuse + t.near + t.held + t.blanked + t.standing + t.unasked, moves, "every move has one state");
    for (const actor of view.actors) for (const mv of actor.moves) assert(mv.state && mv.why, `a move without a state: ${mv.id}`);
    // What the pass logged as sent is what the view calls asked: the moves, the audits, and the questions that
    // are not moves (change, answer, place, forbidden).
    const sentMoves = gate.pass.gate.filter((id) => view.actors.some((actor) => actor.moves.some((mv) => mv.id === id))).length;
    assert.strictEqual(t.asked + t.audit, sentMoves, "asked + audited moves are the moves the pass sent");
    assert(view.f <= mid && (view.next == null || view.next > mid), "the gate is the one at or before the frame");
    const picture = JSON.parse(python([file, "--picture", String(mid)]));
    assert(picture.f == null || (Array.isArray(picture.groups) && typeof picture.actors === "object"), "the picture");
    // A unit of ours that is an actor of the picture is found by its id, and the actor's units lead back to it.
    const builder = Object.keys(picture.actors || {}).find((n) => /^constructor_\d+/.test(n));
    if (builder) {
      const id = Number(/\d+/.exec(builder)[0]);
      assert.strictEqual(WR.actorOfUnit(picture, id, "builder"), builder, "a builder is its own actor");
      assert.deepStrictEqual(WR.unitsOfActor(picture, builder, []), [id], "an actor's units");
    }
    entry.gate = { at: WR.clock(view.f), answered: view.answeredF == null ? null : WR.clock(view.answeredF), asked: view.asked, actors: view.actors.length, totals: t, places: view.places.length, parties: view.parties.length, components: view.components.length, played: view.played.length, audits: view.audits.length, reads: Object.keys(gate.reads).length };
  }
  report.jev.push(entry);
}
console.log(JSON.stringify(report, null, 1));
