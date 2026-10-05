// Within Reason match viewer: parsing and the match model. No DOM in here, so it also runs under node
// (viewer/test/smoke.js). The formats are documented in docs/harness/record-format.md.
//
// A match is every seat of ours that recorded it: one `seat` per record-<ai>.jsonl, a transcript per player
// (strategist-<ai>.jsonl; one for all seats when the game ran with `--players one`), the commander's transcript,
// and the opponent's ground truth. Rows are added as they arrive (a live match grows), never parsed twice.
"use strict";

const WR = (() => {
  const FPS = 30;
  const FLAG = { beingBuilt: 1, idle: 2 };
  /// Dollars a million input tokens of Jev (docs/studies/2026-10-04-jev-where-the-tokens-go.md).
  const JEV_PRICE = 0.042;
  /// The oldest pianist's log the Hands panels read: the rebuilt hands (one menu per actor, layers).
  const JEV_VERSION = 3;
  /// A gate's bars (`crates/bot/src/brain/pianist/compose.rs`, `decode.rs`): a noul at the flag goes to the pick;
  /// a decode reading under the fuse bar fuses its move off.
  const FLAG_BAR = 0.5;
  const FUSE_BAR = 0.3;

  // JSON Lines with a possibly truncated last line (a killed match). Returns the values and the bad line count.
  function parseLines(text) {
    const values = [];
    let bad = 0;
    for (const line of text.split("\n")) {
      if (!line.trim()) continue;
      try {
        values.push(JSON.parse(line));
      } catch {
        bad++;
      }
    }
    return { values, bad };
  }

  // ---------------------------------------------------------------- a seat's record

  const SAMPLE_UNUSED = ["dealt", "shots", "ff", "xf", "xo", "xi", "lane"];

  function newSeat() {
    return {
      header: null, samples: [], events: [], decisions: [], commands: [], intents: [], result: null, end: null,
      restarts: [], series: [], defs: [], classByName: new Map(), lastFrame: 0, team: null, name: "", index: 0,
    };
  }

  /// Adds parsed record rows to a seat. `ourTeams`: the teams of the seats being loaded; a sample's ally rows of
  /// those teams are dropped (each is in its own seat's record: three quarters of a four-seat record).
  function addRecordRows(seat, rows, ourTeams) {
    for (const r of rows) {
      switch (r.t) {
        case "header":
          if (seat.header) {
            seat.restarts.push(r.first_tick_frame ?? 0);
            break;
          }
          if (r.format !== "within-reason-record") throw new Error(`unknown format ${r.format}`);
          seat.header = r;
          seat.defs = r.unit_defs;
          seat.classByName = new Map(r.unit_defs.map((d) => [d.name, d.class]));
          seat.team = r.team;
          seat.name = `t${r.team}`;
          break;
        case "s":
          if (ourTeams && r.al && r.al.length) r.al = r.al.filter((u) => !ourTeams.has(u[4]));
          // The fire instrument's rows (run/fire.py reads them) are not drawn: not kept, eight seats being held.
          for (const key of SAMPLE_UNUSED) delete r[key];
          seat.samples.push(r);
          seat.series.push(seriesOf(seat, r));
          seat.lastFrame = r.f;
          break;
        case "ev": seat.events.push(r); break;
        case "d": seat.decisions.push(r); break;
        case "cmd": seat.commands.push(r); break;
        case "intent": seat.intents.push(r); break;
        case "result": seat.result = r; break;
        case "end": seat.end = r; break;
        default: break; // rows of a newer format are skipped, not fatal
      }
    }
  }

  /// One whole record as text (the tests, and files opened by hand).
  function parseRecord(text, ourTeams) {
    const { values, bad } = parseLines(text);
    const seat = newSeat();
    addRecordRows(seat, values, ourTeams);
    if (!seat.header) throw new Error("no header line: not a Within Reason match record");
    seat.badLines = bad;
    return seat;
  }

  // One sample's numbers for the stats and the charts.
  function seriesOf(seat, s) {
    let extractors = 0, army = 0, armyMetal = 0, builders = 0, buildings = 0;
    for (const u of s.own) {
      if (u[5] & FLAG.beingBuilt) continue;
      const d = u[1] >= 0 ? seat.defs[u[1]] : null;
      const c = d ? d.class : "other";
      if (c === "extractor") extractors++;
      else if (c === "army") { army++; armyMetal += d.metal || 0; }
      else if (c === "builder") builders++;
      if (c === "extractor" || c === "factory" || c === "turret" || c === "building") buildings++;
    }
    return { f: s.f, metalIncome: s.m[1], energyIncome: s.e[1], metal: s.m[0], energy: s.e[0], extractors, army, armyMetal, builders, buildings, enemiesVisible: s.en.length, decideMs: s.ms };
  }

  // ---------------------------------------------------------------- the match

  /// The seats together. `seats` in team order; the first seat's header stands for the map, the grid and the
  /// unit table (every seat of one game writes the same).
  function newMatch() {
    return { seats: [], header: null, defs: [], classByName: new Map(), truth: [], players: new Map(), commander: null, botLog: [], batch: null, result: null, lastFrame: 0, realtime: null };
  }

  function addSeat(match, seat) {
    match.seats.push(seat);
    match.seats.sort((a, b) => a.team - b.team);
    match.seats.forEach((s, i) => (s.index = i));
    const first = match.seats[0];
    match.header = first.header;
    match.defs = first.defs;
    match.classByName = first.classByName;
  }

  /// After rows arrived: the match's last frame, and its result (any seat's result line).
  function settle(match) {
    match.lastFrame = Math.max(0, ...match.seats.map((s) => s.lastFrame));
    match.result = match.seats.map((s) => s.result).find(Boolean) || null;
    if (match.realtime == null && match.batch && match.batch.realtime != null) match.realtime = !!match.batch.realtime;
  }

  /// The seats in scope: one (`scope` is its index) or all (`scope` null).
  function scoped(match, scope) {
    return scope == null ? match.seats : match.seats.filter((s) => s.index === scope);
  }

  /// The side's sizes from the header's seat table: ours (our ally team), theirs, and who they are.
  function sides(match) {
    const table = (match.header && match.header.seats) || [];
    const mine = match.header ? match.header.ally_team : 0;
    const live = table.filter((s) => s.controller !== "Gaia");
    const name = (s) => (s.controller && s.controller.Ai ? `${s.controller.Ai.short_name}${s.controller.Ai.profile ? ` ${s.controller.Ai.profile}` : ""}` : "a person");
    const count = (list) => {
      const names = new Map();
      for (const s of list) names.set(name(s), (names.get(name(s)) || 0) + 1);
      return [...names].map(([n, k]) => (k > 1 ? `${k} ${n}` : n)).join(", ");
    };
    const ours = live.filter((s) => s.ally_team === mine);
    const theirs = live.filter((s) => s.ally_team !== mine);
    return { ours: ours.length, theirs: theirs.length, oursWords: count(ours), theirsWords: count(theirs) };
  }

  // ---------------------------------------------------------------- transcripts

  /// A player's or the commander's transcript, folded into turns as its rows arrive.
  function newTranscript(role) {
    return { role, turns: [], open: null };
  }

  function addTranscriptRows(transcript, rows) {
    for (const r of rows) {
      if (r.kind === "turn") {
        const woken = /Woken because: (.*)/.exec(r.prompt || "");
        transcript.open = { f: r.frame, prompt: r.prompt || "", wake: woken ? woken[1] : null, calls: [], said: [], thinking: [], wallSeconds: null, endedBy: null, cost: null, stopped: null, role: transcript.role };
        transcript.turns.push(transcript.open);
        continue;
      }
      const turn = transcript.open;
      if (!turn) continue;
      if (r.kind === "tool_call") {
        // An `orders` call is the turn's other tools in one: each nested call stands as its own.
        if (r.tool === "orders" && r.arguments && Array.isArray(r.arguments.calls)) {
          for (const c of r.arguments.calls) turn.calls.push({ tool: c.tool, arguments: c.arguments || {}, result: null, within: "orders" });
          turn.ordersResult = r.result;
        } else {
          turn.calls.push({ tool: r.tool, arguments: r.arguments || {}, result: r.result });
        }
      } else if (r.kind === "assistant") {
        for (const part of r.message?.message?.content || []) {
          if (part.type === "text" && part.text.trim()) turn.said.push(part.text.trim());
          if (part.type === "thinking" && part.thinking?.trim()) turn.thinking.push(part.thinking.trim());
        }
      } else if (r.kind === "result") {
        turn.cost = r.message?.total_cost_usd ?? null;
      } else if (r.kind === "turn_end") {
        turn.wallSeconds = r.wall_seconds;
        turn.endedBy = r.ended_by || null;
      } else if (r.kind === "stopped") {
        turn.stopped = r.reason || r.message || "stopped";
      }
    }
  }

  function parseTranscript(text, role) {
    const transcript = newTranscript(role);
    addTranscriptRows(transcript, parseLines(text).values);
    return transcript;
  }

  /// Whether the game ran on while its players thought (a realtime game): the batch says so; without a batch (a
  /// game with people) it did unless a turn's wall time is longer than the game time to the next turn.
  function inferRealtime(match) {
    if (match.batch && match.batch.realtime != null) return !!match.batch.realtime;
    for (const t of [match.commander, ...match.players.values()].filter(Boolean)) {
      for (let i = 0; i + 1 < t.turns.length; i++) {
        const turn = t.turns[i];
        if (turn.wallSeconds != null && (t.turns[i + 1].f - turn.f) / FPS < turn.wallSeconds - 2) return false;
      }
    }
    return true;
  }

  /// The frame a turn's orders are in force from: its end in a realtime game (the hands read them "once your turn
  /// ends"), its own frame in a game that waits for the turn.
  function turnEnd(match, turn) {
    return match.realtime && turn.wallSeconds != null ? turn.f + Math.round(turn.wallSeconds * FPS) : turn.f;
  }

  /// Every call of a tool up to `frame`, oldest first: [{f, arguments, result, turn}].
  function callsOf(transcript, tool, frame) {
    const out = [];
    if (!transcript) return out;
    for (const turn of transcript.turns) {
      if (turn.f > frame) break;
      for (const c of turn.calls) if (c.tool === tool) out.push({ f: turn.f, arguments: c.arguments || {}, result: c.result, turn });
    }
    return out;
  }

  /// The commander's direction in force at `frame`: the plan and each seat's paragraph with the frame it was
  /// written ("what you do not name stands").
  function directionAt(transcript, frame) {
    const direction = { plan: null, seats: {} };
    for (const c of callsOf(transcript, "direct", frame)) {
      if (typeof c.arguments.plan === "string") direction.plan = { text: c.arguments.plan, f: c.f };
      for (const [seat, text] of Object.entries(c.arguments.seats || {})) direction.seats[seat] = { text, f: c.f };
    }
    return direction;
  }

  /// A tool whose arguments are an object of name to value, the last value of each name standing (`mark`,
  /// `queue`, `produce`, `lane`, `signal`): Map name -> {value, f, turn}. A null value forgets the name.
  function latestByKey(transcript, tool, frame, keepNull = false) {
    const out = new Map();
    for (const c of callsOf(transcript, tool, frame)) {
      for (const [key, value] of Object.entries(c.arguments)) {
        if (value == null && !keepNull) out.delete(key);
        else out.set(key, { value, f: c.f, turn: c.turn, result: c.result });
      }
    }
    return out;
  }

  /// The places marked by `mark` calls in force at `frame`: [{name, x, z, f}]. A place is [x, z], those two
  /// numbers written as text, or a grid cell (its centre).
  function marksAt(match, transcript, frame) {
    const out = [];
    const map = match.header.map;
    const { columns, rows } = match.header.grid;
    for (const [name, { value, f }] of latestByKey(transcript, "mark", frame)) {
      let at = null;
      if (Array.isArray(value) && value.length === 2) at = value.map(Number);
      else if (typeof value === "string") {
        const pair = /^\s*\[?\s*(-?[\d.]+)\s*,\s*(-?[\d.]+)\s*\]?\s*$/.exec(value);
        const cell = /^\s*([A-Za-z])(\d+)\s*$/.exec(value);
        if (pair) at = [Number(pair[1]), Number(pair[2])];
        else if (cell) at = [((cell[1].toUpperCase().charCodeAt(0) - 65 + 0.5) * map.width) / columns, ((Number(cell[2]) - 0.5) * map.height) / rows];
      }
      if (at && at.every(Number.isFinite)) out.push({ name, x: at[0], z: at[1], f });
    }
    return out;
  }

  /// The signals set up to `frame` and whether each is given by then: [{name, given, at, f}] (`at`: the frame it
  /// is or will be given; `f`: the turn that set it).
  function signalsAt(transcript, frame) {
    const out = [];
    for (const [name, { value, f, turn, result }] of latestByKey(transcript, "signal", frame)) {
      if (typeof value !== "string") continue;
      let at = f;
      const clock = /^(\d+):(\d\d)$/.exec(value.trim());
      if (clock) at = (Number(clock[1]) * 60 + Number(clock[2])) * FPS;
      else {
        // "now": the tool's answer says the game clock it was given at (the turn began earlier).
        const said = new RegExp(`${name} is given now \\((\\d+):(\\d\\d)\\)`).exec(result || turn.ordersResult || "");
        if (said) at = (Number(said[1]) * 60 + Number(said[2])) * FPS;
      }
      out.push({ name, given: at <= frame, at, f, set: value });
    }
    return out;
  }

  /// The player's packet in force at `frame`: the last `instruct` whose turn had ended. {text, f, from} or null.
  function packetAt(match, transcript, frame) {
    let packet = null;
    if (!transcript) return null;
    for (const turn of transcript.turns) {
      if (turn.f > frame) break;
      const from = turnEnd(match, turn);
      if (from > frame) continue;
      for (const c of turn.calls) if (c.tool === "instruct" && typeof c.arguments.text === "string") packet = { text: c.arguments.text, f: turn.f, from };
    }
    return packet;
  }

  // ---------------------------------------------------------------- the opponent's truth

  /// truth-<ai>.jsonl rows (WITHIN_REASON_OBSERVE=1): every enemy unit every two seconds,
  /// `[id, name, x, z, health %, being built]`. The seats of a match write the same file: one is read.
  function addTruthRows(match, rows) {
    const metalByName = match.metalByName || (match.metalByName = new Map(match.defs.map((d) => [d.name, d.metal || 0])));
    for (const row of rows) {
      if (!row || !Array.isArray(row.enemy)) continue;
      let extractors = 0, army = 0, armyMetal = 0;
      const units = row.enemy.map((u) => {
        const cls = match.classByName.get(u[1]) || "other";
        if (!u[5]) {
          if (cls === "extractor") extractors++;
          else if (cls === "army") { army++; armyMetal += metalByName.get(u[1]) || 0; }
        }
        return { id: u[0], name: u[1], class: cls, x: u[2], z: u[3], health: u[4], building: !!u[5] };
      });
      match.truth.push({ f: row.f, units, extractors, army, armyMetal });
    }
  }

  // bot.log lines that carry a frame: [{f, ai, text}].
  function parseBotLog(text) {
    const lines = [];
    for (const raw of text.split("\n")) {
      const m = /^\[ai (-?\d+)\] f=(\d+) (.*)/.exec(raw);
      if (m) lines.push({ ai: Number(m[1]), f: Number(m[2]), text: m[3] });
    }
    return lines;
  }

  // ---------------------------------------------------------------- lookups

  // Index of the last item with f <= frame, or -1.
  function indexAt(items, frame) {
    let lo = 0, hi = items.length - 1, found = -1;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (items[mid].f <= frame) {
        found = mid;
        lo = mid + 1;
      } else {
        hi = mid - 1;
      }
    }
    return found;
  }

  // Index of the last number <= value in a sorted list, or -1.
  function lastAt(numbers, value) {
    let lo = 0, hi = numbers.length - 1, found = -1;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (numbers[mid] <= value) {
        found = mid;
        lo = mid + 1;
      } else {
        hi = mid - 1;
      }
    }
    return found;
  }

  function range(items, from, to) {
    return items.slice(indexAt(items, from - 1) + 1, indexAt(items, to) + 1);
  }

  /// One seat's units at `frame`, positions interpolated towards its next sample.
  function seatStateAt(seat, frame) {
    const i = indexAt(seat.samples, frame);
    if (i < 0) return { sample: null, own: [], allies: [], enemies: [] };
    const a = seat.samples[i];
    const b = seat.samples[i + 1];
    // A long gap is a bot restart or a held game, not motion.
    const t = b && b.f - a.f <= 4 * (seat.header.sample_frames || FPS) ? (frame - a.f) / (b.f - a.f) : 0;
    const blend = (rows, next, healthIndex) => {
      const later = t > 0 ? new Map((next || []).map((r) => [r[0], r])) : null;
      return rows.map((r) => {
        const n = later ? later.get(r[0]) : null;
        const [x, z] = n ? [r[2] + (n[2] - r[2]) * t, r[3] + (n[3] - r[3]) * t] : [r[2], r[3]];
        return { id: r[0], def: r[1], x, z, health: r[healthIndex], flags: r[5] || 0, seat: seat.index };
      });
    };
    const damaged = new Map(((b || a).dmg || []).map(([id, amount]) => [id, amount]));
    const own = blend(a.own, b && b.own, 4);
    for (const u of own) u.damage = damaged.get(u.id) || 0;
    // Allies (`al`: [id, def, x, z, team, being built]): whoever else is on our side and not a seat we load.
    const allies = blend(a.al || [], b && b.al, 4).map((u) => ({ ...u, team: u.health, health: null, flags: u.flags ? FLAG.beingBuilt : 0, seat: null }));
    return { sample: a, own, allies, enemies: blend(a.en, b && b.en, 4) };
  }

  /// Every seat's units at `frame`: `own` carries each unit's seat; the enemies any seat sees, once each (an
  /// identified sighting over a radar contact).
  function stateAt(match, frame) {
    const out = { own: [], allies: [], enemies: [], samples: [] };
    const seen = new Map();
    for (const seat of match.seats) {
      const s = seatStateAt(seat, frame);
      out.samples.push(s.sample);
      for (const u of s.own) out.own.push(u);
      for (const u of s.allies) out.allies.push(u);
      for (const e of s.enemies) {
        const known = seen.get(e.id);
        if (!known || (known.def < 0 && e.def >= 0)) seen.set(e.id, e);
      }
    }
    out.enemies = [...seen.values()];
    // A seat whose record came later than the others' samples: its units are its own, not allies.
    if (out.allies.length) {
      const own = new Set(out.own.map((u) => u.id));
      const had = new Set();
      out.allies = out.allies.filter((u) => !own.has(u.id) && !had.has(u.id) && had.add(u.id));
    }
    return out;
  }

  /// The side's numbers at `frame` over the seats in scope: each seat's last sample summed.
  function totalsAt(match, scope, frame) {
    const total = { metal: 0, metalIncome: 0, metalUse: 0, energy: 0, energyIncome: 0, energyUse: 0, energyStore: 0, extractors: 0, army: 0, armyMetal: 0, builders: 0, buildings: 0, enemiesVisible: 0, seats: 0 };
    for (const seat of scoped(match, scope)) {
      const i = indexAt(seat.samples, frame);
      if (i < 0) continue;
      const s = seat.samples[i], n = seat.series[i];
      total.seats++;
      total.metal += s.m[0]; total.metalIncome += s.m[1]; total.metalUse += s.m[2] || 0;
      total.energy += s.e[0]; total.energyIncome += s.e[1]; total.energyUse += s.e[2] || 0; total.energyStore += s.e[3] || 0;
      for (const key of ["extractors", "army", "armyMetal", "builders", "buildings"]) total[key] += n[key];
      total.enemiesVisible = Math.max(total.enemiesVisible, n.enemiesVisible);
    }
    return total;
  }

  /// Heuristic firings (the footwork rules' counts) summed over the game minute that contains `frame`.
  function rulesInMinute(match, scope, frame) {
    const start = Math.floor(frame / (60 * FPS)) * 60 * FPS;
    const totals = new Map();
    for (const seat of scoped(match, scope)) {
      for (const d of range(seat.decisions, start, start + 60 * FPS - 1)) {
        if (d.kind !== "rules") continue;
        for (const [rule, n] of Object.entries(d.outputs)) totals.set(rule, (totals.get(rule) || 0) + n);
      }
    }
    return [...totals.entries()].sort((a, b) => b[1] - a[1]);
  }

  /// The timeline's lanes over the seats in scope. Every item carries its seat's index. A kill every seat saw is
  /// one kill.
  function lanes(match, scope) {
    const out = { losses: [], buildingLosses: [], extractorLosses: [], kills: [], builders: [], labs: [], groups: [] };
    const killed = new Set();
    for (const seat of scoped(match, scope)) {
      const cls = (def) => (def >= 0 && seat.defs[def] ? seat.defs[def].class : "other");
      for (const e of seat.events) {
        if (e.k === "enemy_destroyed" && !killed.has(e.u)) {
          killed.add(e.u);
          out.kills.push({ ...e, seat: seat.index });
        }
        if (e.k !== "destroyed") continue;
        const c = cls(e.d);
        const lane = c === "extractor" ? out.extractorLosses : c === "army" || c === "builder" || c === "commander" || c === "other" ? out.losses : out.buildingLosses;
        lane.push({ ...e, seat: seat.index });
      }
      // The hands' plays: the pick's (source plan) and the lists' (source list, code alone).
      for (const d of seat.decisions) {
        const lane = d.kind === "builder" ? out.builders : d.kind === "lab" ? out.labs : d.kind === "group" ? out.groups : null;
        if (lane && (d.source === "plan" || d.source === "list")) lane.push({ ...d, seat: seat.index });
      }
    }
    for (const lane of Object.values(out)) lane.sort((a, b) => a.f - b.f);
    return out;
  }

  /// An actor's plays up to `frame` out of its seat's record, oldest first.
  function actorPlays(seat, actor, frame) {
    return seat.decisions.filter((d) => d.f <= frame && d.inputs && d.inputs.actor === actor && (d.source === "plan" || d.source === "list"));
  }

  function clock(frame) {
    const seconds = Math.floor(frame / FPS);
    return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
  }

  /// "12 s", "3:20": how long ago, in game time.
  function ago(frames) {
    const seconds = Math.max(0, Math.round(frames / FPS));
    return seconds < 60 ? `${seconds} s` : `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
  }

  // Same cells as `World::grid` in crates/bot/src/world.rs.
  function gridName(match, x, z) {
    const { columns, rows } = match.header.grid;
    const cell = (v, extent, n) => Math.min(n - 1, Math.max(0, Math.floor((v / extent) * n)));
    return String.fromCharCode(65 + cell(x, match.header.map.width, columns)) + (cell(z, match.header.map.height, rows) + 1);
  }

  // The standing order of every unit of a seat at `frame`: the last command the bot sent it, ended by a `stop`
  // (the idle flag and the unit's death are the reader's to check). A cache on the seat advances with the
  // playhead and starts over when it goes back.
  const ORDER_KINDS = new Set(["build", "move", "fight", "guard", "repair", "reclaim", "reclaim_feature", "resurrect", "attack", "stop"]);
  function orderOf(c, f) {
    const [kind, unit] = c;
    const o = { kind, unit, f };
    if (kind === "build") { o.def = c[2]; if (c.length >= 5) { o.x = c[3]; o.z = c[4]; } }
    else if (kind === "move" || kind === "fight") { o.x = c[2]; o.z = c[3]; }
    else if (kind === "guard" || kind === "repair" || kind === "attack") o.target = c[2];
    else if (kind === "reclaim") { o.x = c[2]; o.z = c[3]; o.radius = c[4]; }
    else if (kind === "reclaim_feature" || kind === "resurrect") o.feature = c[2];
    return o;
  }
  function ordersAt(seat, frame) {
    let cache = seat.orderCache;
    if (!cache || cache.frame > frame) cache = seat.orderCache = { frame: -1, index: 0, byUnit: new Map() };
    const commands = seat.commands;
    while (cache.index < commands.length && commands[cache.index].f <= frame) {
      const record = commands[cache.index++];
      for (const c of record.c) {
        if (!ORDER_KINDS.has(c[0])) continue;
        if (c[0] === "stop") cache.byUnit.delete(c[1]);
        else cache.byUnit.set(c[1], orderOf(c, record.f));
      }
    }
    cache.frame = frame;
    return cache.byUnit;
  }

  // Everything the records hold about one unit: its events (and what it began, as a builder), the commands sent
  // to it (`stop` included) with the seat that sent them, and the damage it took, as [frame, amount]. An enemy
  // seen by several seats has each event once.
  function unitHistory(match, id) {
    const events = [], commands = [], damage = [];
    const had = new Set();
    for (const seat of match.seats) {
      for (const e of seat.events) {
        if (e.u !== id && e.by !== id) continue;
        const key = `${e.f}:${e.k}:${e.u}`;
        if (had.has(key)) continue;
        had.add(key);
        events.push(e);
      }
      for (const r of seat.commands) for (const c of r.c) if (c[1] === id && ORDER_KINDS.has(c[0])) commands.push({ ...orderOf(c, r.f), seat: seat.index });
      for (const s of seat.samples) for (const [u, amount] of s.dmg || []) if (u === id) damage.push([s.f, amount]);
    }
    events.sort((a, b) => a.f - b.f);
    commands.sort((a, b) => a.f - b.f);
    return { events, commands, damage };
  }

  // The engine's facing of every finished building of ours (0 south, 1 east, 2 north, 3 west), by unit id.
  function facings(match) {
    const count = match.seats.reduce((n, s) => n + s.events.length, 0);
    if (!match.facingCache || match.facingCache.count !== count) {
      const map = new Map();
      for (const seat of match.seats) for (const e of seat.events) if (e.k === "finished" && e.facing != null) map.set(e.u, e.facing);
      match.facingCache = { count, map };
    }
    return match.facingCache.map;
  }

  /// The chat lines of the game up to `frame`, each once (every seat's record hears them all): [{f, player, text}].
  function chatUpTo(match, frame) {
    const had = new Set();
    const out = [];
    for (const seat of match.seats) {
      for (const e of seat.events) {
        if (e.k !== "chat" || e.f > frame) continue;
        const key = `${e.f}:${e.player}:${e.text}`;
        if (!had.has(key)) { had.add(key); out.push(e); }
      }
    }
    return out.sort((a, b) => a.f - b.f);
  }

  // ---------------------------------------------------------------- the hands' gate

  /// The actor a unit is in the hands' picture: a builder or a plant by its own name, a soldier by its group.
  /// `picture`: what the server sends for `?picture=` (groups, actors). Names carry the seat (`_t1`) in a game of
  /// several seats.
  function actorOfUnit(picture, id, cls) {
    if (!picture) return null;
    const actors = picture.actors || {};
    const own = Object.keys(actors).find((name) => new RegExp(`^(constructor|plant|lab|factory)_${id}(_t\\d+)?$`).test(name));
    if (own) return own;
    if (cls === "commander") return Object.keys(actors).find((name) => /^commander(_t\d+)?$/.test(name)) || null;
    const group = (picture.groups || []).find((g) => (g.members || []).includes(id));
    return group ? Object.keys(actors).find((name) => name === `group_${group.name}` || name.startsWith(`group_${group.name}_t`)) || `group_${group.name}` : null;
  }

  /// The units of an actor in the picture: a group's members, a builder's or a plant's own id.
  function unitsOfActor(picture, actor, commanderIds) {
    const own = /^(?:constructor|plant|lab|factory)_(\d+)/.exec(actor);
    if (own) return [Number(own[1])];
    if (/^commander/.test(actor)) return commanderIds || [];
    const group = /^group_(.+?)(_t\d+)?$/.exec(actor);
    const found = group && picture ? (picture.groups || []).find((g) => g.name === group[1] || `${g.name}` === `${group[1]}${group[2] || ""}`) : null;
    return found ? found.members || [] : [];
  }

  /// The decode's readings by actor (`decode.rs`: `<actor>|place|<place>`, `|build|<type>`, `|join|<group>`,
  /// `|follow|<ward>`, `|detach|<n>`): Map actor -> [[kind, value, reading]].
  function readingsByActor(reads) {
    const out = new Map();
    for (const [k, p] of Object.entries(reads || {})) {
      const [actor, kind, value] = k.split("|");
      if (value == null) continue;
      if (!out.has(actor)) out.set(actor, []);
      out.get(actor).push([kind, value, p]);
    }
    return out;
  }

  /// An actor's readings that bear on one of its moves: [[words, reading]].
  function readingsOf(actorReads, key) {
    const out = [];
    for (const [kind, value, p] of actorReads || []) {
      const hit = kind === "place" ? key.endsWith(`_${value}`)
        : kind === "build" ? key === `build_${value}` || key.startsWith(`build_${value}_`)
        : kind === "join" ? key === `join_${value}` || key === `join_${value.replace(/^group_/, "")}`
        : kind === "follow" ? key === `follow_${value}`
        : kind === "detach" ? key.startsWith(`send_${value}_`)
        : false;
      if (hit) out.push([`${kind} ${value}`, p]);
    }
    return out;
  }

  /// A gate as the server sends it (`?gate=`), worked out for the panel: per actor its menu with every move's
  /// state in words. A move is: its course (the first); asked and answered; asked for an audit (logged, not
  /// played); or not asked, because its actor is closed by news, the decode fused it, `near` shut it, its opener
  /// holds it, its place blanked it, or an earlier answer stands (`same`, `again`).
  function gateView(gate) {
    const pass = gate.pass;
    const calls = gate.calls || [];
    const answers = {};
    for (const c of calls) Object.assign(answers, c.answers || {});
    const flags = (gate.worlds_gate && gate.worlds_gate.flags) || {};
    const sent = new Set(pass.gate || []);
    const noulOf = (id) => (answers[id] && answers[id].type === "noul" ? answers[id].noul : flags[id]);
    const auditById = new Map();
    const newsAudit = new Map();
    for (const a of gate.audits || []) {
      if (a.id) auditById.set(a.id, a);
      else if (a.actor) newsAudit.set(a.actor, a);
    }
    const layers = pass.layers || {};
    const nearOn = !!(layers.near && layers.near.on);
    const played = new Map();
    for (const d of (gate.plan && gate.plan.played) || []) played.set(d.played, d);
    const places = [], parties = [];
    for (const id of new Set([...sent, ...Object.keys(flags), ...Object.keys(answers)])) {
      const dot = id.lastIndexOf(".");
      const name = id.slice(0, dot), what = id.slice(dot + 1);
      if (name.includes(".")) continue;
      const entry = { name, noul: noulOf(id), asked: sent.has(id), audit: auditById.get(id) || null };
      if (what === "place") places.push(entry);
      else if (what === "answer") parties.push(entry);
    }
    const byNoul = (a, b) => (b.noul ?? -1) - (a.noul ?? -1) || a.name.localeCompare(b.name);
    places.sort(byNoul);
    parties.sort(byNoul);
    const placeNames = places.map((p) => p.name).sort((a, b) => b.length - a.length);
    const reads = readingsByActor(gate.reads);
    const totals = { moves: 0, course: 0, asked: 0, audit: 0, news: 0, fuse: 0, near: 0, held: 0, blanked: 0, standing: 0, unasked: 0, played: 0 };
    const actors = (pass.menus || []).map((menu) => {
      const counts = { asked: 0, audit: 0, news: 0, fuse: 0, near: 0, held: 0, blanked: 0, standing: 0, unasked: 0, played: 0 };
      const moves = menu.moves.map((mv, i) => {
        const key = mv.id.slice(menu.name.length + 1);
        const out = { id: mv.id, key, words: mv.words, party: mv.party || null, noul: noulOf(mv.id), state: "", why: "", audit: auditById.get(mv.id) || null, played: played.get(mv.id) || null };
        const forbidden = noulOf(`${mv.id}.forbidden`);
        if (forbidden != null) out.forbidden = { noul: forbidden, asked: sent.has(`${mv.id}.forbidden`), audit: auditById.get(`${mv.id}.forbidden`) || null };
        if (out.played) counts.played++;
        if (i === 0) {
          out.state = "course";
          out.why = "its course: what it does when nothing else is picked";
          return out;
        }
        const readings = readingsOf(reads.get(menu.name), key);
        const low = readings.filter(([, p]) => p < FUSE_BAR).sort((a, b) => a[1] - b[1]);
        const shut = mv.far ? "near" : mv.fused ? (low.length ? "fuse" : nearOn && menu.kind === "builder" && /_spot_\d+$/.test(key) ? "near" : "fuse") : mv.held ? "held" : mv.blanked ? "blanked" : null;
        const reason = () => {
          if (shut === "fuse") return low.length ? `fused by the decode: the packet read ${low.map(([k, p]) => `${k} ${p.toFixed(2)}`).join(", ")} (under ${FUSE_BAR})` : "fused by the decode (its reading is not among those in force)";
          if (shut === "near") return mv.far ? "shut by near: two free builders of its type are nearer" : "shut by near: two free builders of its type are nearer (inferred: fused with no decode reading under the bar; the log does not write the near mark)";
          if (shut === "held") {
            const openers = [mv.party ? `${mv.party}.answer` : null, `${menu.name}.change`].filter(Boolean).map((id) => `${id} ${noulOf(id) != null ? Number(noulOf(id)).toFixed(2) : "?"} now`);
            return `held by its opener, which said no at the last gate (${openers.join(", ")})`;
          }
          if (shut === "blanked") {
            const place = placeNames.find((p) => key.endsWith(`_${p}`));
            return `blanked by its place${place ? `: ${place}.place said no at the last gate (${noulOf(`${place}.place`) != null ? Number(noulOf(`${place}.place`)).toFixed(2) : "?"} now)` : ", whose question said no at the last gate"}`;
          }
          return "";
        };
        if (shut && mv.audit && sent.has(mv.id)) {
          out.state = "audit";
          out.shut = shut;
          out.why = `${reason()}; asked anyway for the layer's audit, not played`;
        } else if (shut) {
          // Drawn for the audit and still not sent: its actor is closed, or its question stands from its last ask.
          out.state = shut;
          out.why = `not asked: ${reason()}${mv.audit ? " (drawn for the audit, and not sent: its actor is closed or the question stands from its last ask)" : ""}`;
        } else if (sent.has(mv.id)) {
          out.state = "asked";
          out.why = out.noul != null ? "asked and answered" : calls.length ? "asked; no answer came back" : "asked; the answer is not in the log yet";
          if (out.audit) out.why += ` (an audit of ${out.audit.layer}: the answer that stood was ${typeof out.audit.assumed === "number" ? out.audit.assumed.toFixed(2) : out.audit.assumed})`;
        } else if (flags[mv.id] != null) {
          out.state = "standing";
          out.why = "not asked: an earlier answer stands (same: the question is in the words of its last ask; or again: its actor was reopened by an opener or a place alone)";
        } else if (menu.quiet && !mv.party) {
          out.state = "news";
          out.why = "not asked: its actor is closed by news (a course and no news)";
        } else {
          out.state = "unasked";
          out.why = "not asked, and no answer of it stands";
        }
        counts[out.state]++;
        return out;
      });
      for (const [k, v] of Object.entries(counts)) totals[k] += v;
      totals.moves += moves.length;
      totals.course++;
      const change = noulOf(`${menu.name}.change`);
      return {
        name: menu.name, kind: menu.kind, course: menu.course, idle: !!menu.idle, quiet: !!menu.quiet, audited: !!menu.audit,
        state: menu.quiet ? (menu.audit ? "closed by news, asked anyway for its audit (not played)" : "closed by news: a course and no news") : moves.length > 1 ? "open" : "nothing to choose",
        change: change == null ? null : { noul: change, asked: sent.has(`${menu.name}.change`) }, newsAudit: newsAudit.get(menu.name) || null, moves, counts,
      };
    });
    // The pick: per component (the split layer) its actors, its worlds in words, the candidate and whether taken.
    const plan = gate.plan;
    const components = [];
    const worldsGate = gate.worlds_gate;
    if (worldsGate && worldsGate.components) {
      worldsGate.components.forEach((c, j) => {
        const part = (plan && plan.split && plan.split[j]) || null;
        components.push({ actors: c.actors, lines: c.lines, candidate: part ? part.candidate : null, taken: part ? !!part.taken : null, confidence: part ? part.confidence : null, sampledP: part ? part.sampled_p : null });
      });
    } else if (worldsGate && worldsGate.lines && worldsGate.lines.length) {
      components.push({ actors: null, lines: worldsGate.lines, candidate: plan ? plan.candidate ?? plan.pick : null, taken: plan ? plan.pick > 1 : null, confidence: plan ? plan.confidence : null, sampledP: plan ? plan.sampled_p : null });
    }
    const tokens = (rows) => rows.reduce((n, c) => n + ((c.usage && c.usage.input_tokens) || 0), 0);
    return {
      f: pass.f, answeredF: calls.length ? calls[0].f : null, prev: gate.prev, next: gate.next,
      events: pass.events || [], hunts: pass.hunts || [], rove: pass.rove || [], open: pass.open || [], closed: pass.closed || [],
      layers, asked: sent.size, totals, places, parties, actors, components,
      played: [...(pass.played || []), ...((plan && plan.played) || [])], planned: !!plan, changed: (plan && plan.changed) || [],
      audits: gate.audits || [], errors: gate.errors || [], shed: gate.shed || [],
      call: calls.length ? { tokens: tokens(calls), ms: calls.reduce((n, c) => n + (c.ms || 0), 0), batches: calls.reduce((n, c) => n + (c.batches || 1), 0), questions: sent.size, model: calls[0].model, state: calls[0].state || {} } : null,
      pick: (gate.picks || []).length ? { tokens: tokens(gate.picks), ms: gate.picks.reduce((n, c) => n + (c.ms || 0), 0), calls: gate.picks.length } : null,
      instructions: gate.instructions || null, instructionsF: gate.instructions_f ?? null,
    };
  }

  /// The Jev bill of the seats in scope from their indexes (`?index`): per minute and in all, by request kind and
  /// question family, with each seat's share. `indexes`: [{seat, index}] of the seats that have one.
  function costView(indexes) {
    const minutes = new Map();
    const families = new Map();
    const kinds = { gate: 0, pick: 0, decode: 0 };
    const seats = [];
    for (const { seat, index } of indexes) {
      let total = 0;
      for (const m of index.minutes) {
        if (!minutes.has(m.minute)) minutes.set(m.minute, { minute: m.minute, tokens: 0, kinds: { gate: 0, pick: 0, decode: 0 }, seats: new Map(), families: new Map(), gates: 0, quiet: 0, picks: 0, taken: 0, errors: 0, questions: 0, calls: 0, maxMs: 0, medianMs: [], audits: 0, faults: 0 });
        const row = minutes.get(m.minute);
        let here = 0;
        for (const [kind, tokens] of Object.entries(m.tokens)) {
          row.kinds[kind] = (row.kinds[kind] || 0) + tokens;
          kinds[kind] = (kinds[kind] || 0) + tokens;
          here += tokens;
        }
        for (const [family, tokens] of Object.entries(m.families)) {
          row.families.set(family, (row.families.get(family) || 0) + tokens);
          families.set(family, (families.get(family) || 0) + tokens);
        }
        row.tokens += here;
        row.seats.set(seat, (row.seats.get(seat) || 0) + here);
        for (const key of ["gates", "quiet", "picks", "taken", "errors", "questions"]) row[key] += m[key] || 0;
        row.calls += Object.values(m.calls).reduce((a, b) => a + b, 0);
        row.maxMs = Math.max(row.maxMs, m.maxMs || 0);
        row.medianMs.push(m.medianMs || 0);
        for (const [n, faults] of Object.values(m.audits || {})) { row.audits += n; row.faults += faults; }
        total += here;
      }
      seats.push({ seat, tokens: total, dollars: (total * JEV_PRICE) / 1e6 });
    }
    const tokens = seats.reduce((n, s) => n + s.tokens, 0);
    const rows = [...minutes.values()].sort((a, b) => a.minute - b.minute);
    for (const r of rows) r.medianMs = r.medianMs.sort((a, b) => a - b)[r.medianMs.length >> 1] || 0;
    return { tokens, dollars: (tokens * JEV_PRICE) / 1e6, kinds, seats, minutes: rows, families: [...families].sort((a, b) => b[1] - a[1]) };
  }

  return {
    FPS, FLAG, JEV_PRICE, JEV_VERSION, FLAG_BAR, FUSE_BAR,
    parseLines, newSeat, addRecordRows, parseRecord, newMatch, addSeat, settle, scoped, sides,
    newTranscript, addTranscriptRows, parseTranscript, inferRealtime, turnEnd, callsOf, directionAt, latestByKey, marksAt, signalsAt, packetAt,
    addTruthRows, parseBotLog, indexAt, lastAt, range, seatStateAt, stateAt, totalsAt, rulesInMinute, lanes, actorPlays, clock, ago, gridName,
    ordersAt, unitHistory, facings, chatUpTo, actorOfUnit, unitsOfActor, readingsByActor, readingsOf, gateView, costView,
  };
})();

if (typeof module !== "undefined") module.exports = WR;
