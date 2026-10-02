// Within Reason match viewer: parsing and the match model. No DOM in here, so it also runs under node
// (viewer/test/smoke.js). The record format is documented in docs/harness/record-format.md.
"use strict";

const WR = (() => {
  const FPS = 30;
  const FLAG = { beingBuilt: 1, idle: 2, attacker: 4, squad: 8 };

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

  function parseRecord(text) {
    const { values, bad } = parseLines(text);
    const match = {
      header: null, samples: [], events: [], decisions: [], commands: [], intents: [], result: null,
      restarts: [], badLines: bad, census: [], truth: [], botLog: [],
    };
    for (const r of values) {
      switch (r.t) {
        case "header":
          if (match.header) match.restarts.push(r.first_tick_frame ?? 0);
          else match.header = r;
          break;
        case "s": match.samples.push(r); break;
        case "ev": match.events.push(r); break;
        case "d": match.decisions.push(r); break;
        case "cmd": match.commands.push(r); break;
        case "intent": match.intents.push(r); break;
        case "result": match.result = r; break;
        default: break; // records from a newer format version are skipped, not fatal
      }
    }
    if (!match.header) throw new Error("no header line: not a Within Reason match record");
    if (match.header.format !== "within-reason-record") throw new Error(`unknown format ${match.header.format}`);
    match.defs = match.header.unit_defs;
    match.classByName = new Map(match.defs.map((d) => [d.name, d.class]));
    match.lastFrame = match.samples.length ? match.samples[match.samples.length - 1].f : 0;
    match.series = ownSeries(match);
    return match;
  }

  // Per-sample numbers for the economy strip.
  function ownSeries(match) {
    const classOf = (def) => (def >= 0 && match.defs[def] ? match.defs[def].class : "other");
    return match.samples.map((s) => {
      let extractors = 0, army = 0, builders = 0, buildings = 0;
      for (const u of s.own) {
        if (u[5] & FLAG.beingBuilt) continue;
        const c = classOf(u[1]);
        if (c === "extractor") extractors++;
        else if (c === "army") army++;
        else if (c === "builder") builders++;
        if (c === "extractor" || c === "factory" || c === "turret" || c === "building") buildings++;
      }
      return { f: s.f, metalIncome: s.m[1], energyIncome: s.e[1], metal: s.m[0], energy: s.e[0], extractors, army, builders, buildings, enemiesVisible: s.en.length, decideMs: s.ms };
    });
  }

  // The strategist / commander transcript, folded into turns. A turn is also a decision record in the
  // source-agnostic shape of the match record: {f, source, kind, inputs, outputs, latency_ms}.
  function parseStrategist(text, source) {
    const { values } = parseLines(text);
    const turns = [];
    let turn = null;
    for (const r of values) {
      if (r.kind === "turn") {
        const woken = /Woken because: (.*)/.exec(r.prompt || "");
        turn = { f: r.frame, prompt: r.prompt || "", wake: woken ? woken[1] : null, calls: [], said: [], thinking: [], wallSeconds: null, cost: null, stopped: null };
        turns.push(turn);
      } else if (!turn) {
        continue;
      } else if (r.kind === "tool_call") {
        turn.calls.push({ tool: r.tool, arguments: r.arguments, result: r.result });
      } else if (r.kind === "assistant") {
        for (const part of r.message?.message?.content || []) {
          if (part.type === "text" && part.text.trim()) turn.said.push(part.text.trim());
          if (part.type === "thinking" && part.thinking?.trim()) turn.thinking.push(part.thinking.trim());
        }
      } else if (r.kind === "result") {
        turn.cost = r.message?.total_cost_usd ?? null;
      } else if (r.kind === "turn_end") {
        turn.wallSeconds = r.wall_seconds;
      } else if (r.kind === "stopped") {
        turn.stopped = r.reason || r.message || "stopped";
      }
    }
    return turns.map((t) => ({
      t: "d", f: t.f, source, kind: "turn",
      inputs: { wake: t.wake, prompt: t.prompt },
      outputs: { calls: t.calls, said: t.said, thinking: t.thinking, stopped: t.stopped },
      latency_ms: t.wallSeconds == null ? null : t.wallSeconds * 1000, cost: t.cost,
    }));
  }

  // Posts the commander gave its squads, from the `squad` tool calls: [{f, name, x, z, radius, until}].
  function squadPosts(decisions, lastFrame) {
    const posts = [];
    const open = new Map();
    const close = (name, f) => {
      const post = open.get(name);
      if (post) post.until = f;
      open.delete(name);
    };
    for (const d of decisions) {
      if (d.kind !== "turn") continue;
      for (const call of d.outputs.calls) {
        const a = call.arguments || {};
        if (call.tool !== "squad" || !a.name) continue;
        if (a.release || a.order || a.post) close(a.name, d.f);
        if (a.post && !a.release) {
          const post = { f: d.f, name: a.name, x: a.post.x, z: a.post.z, radius: a.post.radius, until: lastFrame + 1 };
          posts.push(post);
          open.set(a.name, post);
        }
      }
    }
    return posts;
  }

  // `census f=<frame> enemy|own namexCOUNT@x,z ...` lines of engine.log (WITHIN_REASON_OBSERVE=1).
  function parseCensus(text, classByName) {
    const byFrame = new Map();
    const line = /census f=(\d+) (enemy|own) ?(.*)/g;
    for (const m of text.matchAll(line)) {
      const f = Number(m[1]);
      const groups = [];
      for (const g of m[3].matchAll(/(\w+?)x(\d+)@(-?\d+),(-?\d+)/g)) {
        groups.push({ name: g[1], count: Number(g[2]), x: Number(g[3]), z: Number(g[4]), class: classByName.get(g[1]) || "other" });
      }
      if (!byFrame.has(f)) byFrame.set(f, { f, own: [], enemy: [] });
      byFrame.get(f)[m[2]] = groups;
    }
    const total = (groups, cls) => groups.reduce((n, g) => n + (g.class === cls ? g.count : 0), 0);
    return [...byFrame.values()].sort((a, b) => a.f - b.f).map((c) => ({
      ...c,
      enemyExtractors: total(c.enemy, "extractor"), enemyArmy: total(c.enemy, "army"),
      ownExtractors: total(c.own, "extractor"), ownArmy: total(c.own, "army"),
    }));
  }

  // truth-<ai>.jsonl (WITHIN_REASON_OBSERVE=1): every enemy unit every two seconds, `[id, name, x, z, health %, being built]`.
  // Entries carry the same totals as census entries, so the charts and the stats take either.
  function parseTruth(text, classByName) {
    const out = [];
    for (const line of text.split("\n")) {
      if (!line) continue;
      let row;
      try {
        row = JSON.parse(line);
      } catch (_) {
        continue; // a killed match may end mid-line
      }
      const units = row.enemy.map((u) => ({ id: u[0], name: u[1], class: classByName.get(u[1]) || "other", x: u[2], z: u[3], health: u[4], building: !!u[5] }));
      const count = (cls) => units.reduce((n, u) => n + (u.class === cls && !u.building ? 1 : 0), 0);
      out.push({ f: row.f, units, enemyExtractors: count("extractor"), enemyArmy: count("army") });
    }
    return out.sort((a, b) => a.f - b.f);
  }

  // The pianist's log, jev-<ai_id>.jsonl (docs/harness/record-format.md, "The pianist's log"): a header line, then
  // `call` lines (one per request: the state, the questions, the answers, and the groups, places and parties by
  // name), `error` lines, and from version 2 (2026-09-26, the one pass) a `pass` line per second the pass had
  // anything to say (`open`, `plan`, `gate`/`quiet`, `events`, `hunts`, `slots` on an asking second (version 2) or
  // `menus`, `closed` and `layers` (version 3, the rebuilt hands), `played` with
  // sources rule/list), `worlds_gate` lines (the pre-pass's `flags`, the composed `worlds`, their `lines`) and
  // `plan` lines (the pick, its confidence, `changed`, `played` with source plan). Version-1 logs carried `played` on
  // the calls and `standing` lines (the executor's plays); the first day's logs carried no `played` at all. Every
  // play, from whichever line, lands in the actor's history with its source.
  const PIANIST_SOURCES = new Set(["jev", "rule", "plan", "list", "standing", "policy", "gate"]);

  function parseJev(text) {
    const { values, bad } = parseLines(text);
    const jev = { header: null, calls: [], errors: [], passes: [], gates: [], plans: [], badLines: bad, actors: new Map() };
    let instructions = "";
    const played = (rows, f, source) => {
      for (const d of rows || []) {
        const entry = { ...d, f, source: d.source || source };
        if (!jev.actors.has(entry.actor)) jev.actors.set(entry.actor, { name: entry.actor, kind: entry.kind, decisions: [] });
        jev.actors.get(entry.actor).decisions.push(entry);
      }
    };
    for (const r of values) {
      switch (r.t) {
        case "header": jev.header = r; break;
        case "error": jev.errors.push(r); break;
        case "pass": jev.passes.push(r); played(r.played, r.f, "rule"); break;
        case "standing": jev.passes.push({ ...r, open: Object.keys(r.orders || {}), legacy: true }); played(r.played, r.f, "standing"); break;
        case "worlds_gate": jev.gates.push(r); break;
        case "plan": jev.plans.push(r); played(r.played, r.f, "plan"); break;
        case "decompress": break;
        default: {
          if (r.t && r.t !== "call") break;
          if (typeof r.instructions === "string") instructions = r.instructions;
          else if (r.state && typeof r.state.instructions === "string") instructions = r.state.instructions;
          const rules = (r.state && r.state.rules) || (jev.header && jev.header.rules) || "";
          const call = {
            f: r.f, ms: r.ms, model: r.model, tokens: r.usage ? r.usage.input_tokens || 0 : 0, retries: r.retries || 0,
            instructions, rules, state: r.state || {}, questions: r.questions || {}, answers: r.answers || {},
            played: r.played || (jev.header && jev.header.version >= 2 ? [] : playedFromAnswers(r)), groups: r.groups || [], places: r.places || [], parties: r.parties || [],
            pick: Object.keys(r.questions || {}).some((q) => q.startsWith("worlds.")),
          };
          jev.calls.push(call);
          played(call.played, call.f, "jev");
        }
      }
    }
    for (const list of [jev.calls, jev.passes, jev.gates, jev.plans]) list.sort((a, b) => a.f - b.f);
    for (const actor of jev.actors.values()) actor.decisions.sort((a, b) => a.f - b.f);
    jev.version = jev.header ? jev.header.version || 1 : 1;
    return jev;
  }

  // A log without `played` (the first day's): the decisions as far as the answers tell them.
  function playedFromAnswers(r) {
    const out = [];
    for (const [id, a] of Object.entries(r.answers || {})) {
      const [actor, what] = id.split(".");
      if (a.type !== "choice" || !(what === "do" || what === "next")) continue;
      const kind = actor === "commander" || actor.startsWith("constructor") ? "builder" : actor.startsWith("lab") ? "lab" : actor.startsWith("group") ? "group" : "global";
      out.push({ actor, kind, busy: "continue" in (a.probabilities || {}), choice: a.choice, played: a.choice, kept: false, probability: a.probabilities ? a.probabilities[a.choice] : null, confidence: a.confidence, did: null });
    }
    return out;
  }

  // A play that changed something (a "continue" or a "wait" of the menus' days did not).
  function isChange(d) {
    return d.played !== "continue" && d.played !== "nothing" && d.played !== "wait" && !d.kept;
  }

  // The pass at `frame` with what followed it: the gate's flags, the worlds and the pick. Null before the first pass.
  function passAt(jev, frame) {
    const i = indexAt(jev.passes, frame);
    if (i < 0) return null;
    const pass = jev.passes[i];
    const at = (list) => {
      const j = indexAt(list, pass.f);
      return j >= 0 && list[j].f === pass.f ? list[j] : null;
    };
    const gate = pass.gate ? at(jev.gates) : null;
    const plan = pass.gate ? at(jev.plans) : null;
    const pickCall = pass.gate ? jev.calls.find((c) => c.f === pass.f && c.pick) : null;
    return { pass, index: i, gate, plan, pickCall };
  }

  // Calls per game minute: [{minute, calls, medianMs, maxMs, tokens, questions, asks, quiet, picks, w1, rule, plan, list, changes, errors}].
  function jevMinutes(jev) {
    const rows = new Map();
    const row = (f) => {
      const minute = Math.floor(f / (60 * FPS));
      if (!rows.has(minute)) rows.set(minute, { minute, calls: 0, ms: [], tokens: 0, questions: 0, asks: 0, quiet: 0, picks: 0, w1: 0, rule: 0, plan: 0, list: 0, changes: 0, kept: 0, errors: 0 });
      return rows.get(minute);
    };
    for (const c of jev.calls) {
      const r = row(c.f);
      r.calls++;
      r.ms.push(c.ms);
      r.tokens += c.tokens;
      r.questions += Object.keys(c.questions).length;
      for (const d of c.played) {
        if (d.kept) r.kept++;
        else if (isChange(d)) r.changes++;
      }
    }
    for (const p of jev.passes) {
      const r = row(p.f);
      if (p.gate) r.asks++;
      if (p.quiet) r.quiet++;
      for (const d of p.played || []) {
        if (d.source === "list") r.list++;
        else r.rule++;
        r.changes++;
      }
    }
    for (const p of jev.plans) {
      const r = row(p.f);
      r.picks++;
      if (p.split ? !(p.changed || []).length : p.pick === 1) r.w1++;
      r.plan += (p.played || p.changed || []).length;
      r.changes += (p.played || p.changed || []).length;
    }
    for (const e of jev.errors) row(e.f).errors++;
    return [...rows.values()].sort((a, b) => a.minute - b.minute).map((r) => {
      const sorted = [...r.ms].sort((a, b) => a - b);
      return { ...r, medianMs: sorted.length ? sorted[sorted.length >> 1] : 0, maxMs: sorted.length ? sorted[sorted.length - 1] : 0 };
    });
  }

  // bot.log lines that carry a frame: [{f, text}].
  function parseBotLog(text) {
    const lines = [];
    for (const raw of text.split("\n")) {
      const m = /^\[ai -?\d+\] f=(\d+) (.*)/.exec(raw);
      if (m) lines.push({ f: Number(m[1]), text: m[2] });
    }
    return lines;
  }

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

  function range(items, from, to) {
    return items.slice(indexAt(items, from - 1) + 1, indexAt(items, to) + 1);
  }

  // Units at `frame`, positions interpolated towards the next sample. Rows: {id, def, x, z, health, flags, damage}.
  function stateAt(match, frame) {
    const i = indexAt(match.samples, frame);
    if (i < 0) return { sample: null, own: [], allies: [], enemies: [] };
    const a = match.samples[i];
    const b = match.samples[i + 1];
    // A long gap is a bot restart or a held game, not motion.
    const t = b && b.f - a.f <= 4 * (match.header.sample_frames || FPS) ? (frame - a.f) / (b.f - a.f) : 0;
    const blend = (rows, next, healthIndex) => {
      const later = new Map((next || []).map((r) => [r[0], r]));
      return rows.map((r) => {
        const n = t > 0 ? later.get(r[0]) : null;
        const [x, z] = n ? [r[2] + (n[2] - r[2]) * t, r[3] + (n[3] - r[3]) * t] : [r[2], r[3]];
        return { id: r[0], def: r[1], x, z, health: r[healthIndex], flags: r[5] || 0 };
      });
    };
    const damaged = new Map(((b || a).dmg || []).map(([id, amount]) => [id, amount]));
    const own = blend(a.own, b && b.own, 4);
    for (const u of own) u.damage = damaged.get(u.id) || 0;
    // Allies (`al`: [id, def, x, z, team, being built]): other seats of ours, or anybody else on our side.
    const allies = blend(a.al || [], b && b.al, 4).map((u) => ({ ...u, team: u.health, health: null, flags: u.flags ? FLAG.beingBuilt : 0 }));
    return { sample: a, own, allies, enemies: blend(a.en, b && b.en, 4) };
  }

  // Heuristic firings summed over the game minute that contains `frame`: [[rule, count]] by count.
  function rulesInMinute(match, frame) {
    const start = Math.floor(frame / (60 * FPS)) * 60 * FPS;
    const totals = new Map();
    for (const d of range(match.decisions, start, start + 60 * FPS - 1)) {
      if (d.kind !== "rules") continue;
      for (const [rule, n] of Object.entries(d.outputs)) totals.set(rule, (totals.get(rule) || 0) + n);
    }
    return [...totals.entries()].sort((a, b) => b[1] - a[1]);
  }

  // The timeline's lanes, from events and decisions: {losses, buildingLosses, extractorLosses, kills, waves, turns}.
  function lanes(match) {
    const cls = (def) => (def >= 0 && match.defs[def] ? match.defs[def].class : "other");
    const out = { losses: [], buildingLosses: [], extractorLosses: [], kills: [], waves: [], turns: [], jevBuilders: [], jevLabs: [], jevGroups: [] };
    for (const e of match.events) {
      if (e.k === "enemy_destroyed") out.kills.push(e);
      if (e.k !== "destroyed") continue;
      const c = cls(e.d);
      if (c === "extractor") out.extractorLosses.push(e);
      else if (c === "army" || c === "builder" || c === "commander" || c === "other") out.losses.push(e);
      else out.buildingLosses.push(e);
    }
    for (const d of match.decisions) {
      if (d.kind === "wave" || d.kind === "recall" || d.kind === "assault") out.waves.push(d);
      if (d.kind === "turn") out.turns.push(d);
      // The pianist's decisions: a change of course is a solid mark, a "continue" (or a kept course) a faint one.
      if (PIANIST_SOURCES.has(d.source)) {
        const o = d.outputs || {};
        const change = o.played == null || isChange(o);
        const lane = d.kind === "builder" ? out.jevBuilders : d.kind === "lab" ? out.jevLabs : d.kind === "group" ? out.jevGroups : null;
        if (lane) lane.push({ ...d, faint: !change });
      }
    }
    return out;
  }

  function clock(frame) {
    const seconds = Math.floor(frame / FPS);
    return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
  }

  // Same cells as `World::grid` in crates/bot/src/world.rs.
  function gridName(match, x, z) {
    const { columns, rows } = match.header.grid;
    const cell = (v, extent, n) => Math.min(n - 1, Math.max(0, Math.floor((v / extent) * n)));
    return String.fromCharCode(65 + cell(x, match.header.map.width, columns)) + (cell(z, match.header.map.height, rows) + 1);
  }

  // The standing order of every unit at `frame`: the last command the bot sent it, ended by a `stop` (the idle flag
  // and the unit's death are the reader's to check). A cache on the match advances with the playhead and starts
  // over when it goes back.
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
  function ordersAt(match, frame) {
    let cache = match.orderCache;
    if (!cache || cache.frame > frame) cache = match.orderCache = { frame: -1, index: 0, byUnit: new Map() };
    const commands = match.commands;
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

  // Everything the record holds about one unit: its events (and what it began, as a builder), the commands sent to
  // it (`stop` included), and the damage it took, as [frame, amount].
  function unitHistory(match, id) {
    const events = match.events.filter((e) => e.u === id || e.by === id);
    const commands = [];
    for (const r of match.commands) for (const c of r.c) if (c[1] === id && ORDER_KINDS.has(c[0])) commands.push(orderOf(c, r.f));
    const damage = [];
    for (const s of match.samples) for (const [u, amount] of s.dmg || []) if (u === id) damage.push([s.f, amount]);
    return { events, commands, damage };
  }

  // The engine's facing of every finished building of ours (0 south, 1 east, 2 north, 3 west), by unit id.
  function facings(match) {
    if (!match.facingCache || match.facingCache.count !== match.events.length) {
      const map = new Map();
      for (const e of match.events) if (e.k === "finished" && e.facing != null) map.set(e.u, e.facing);
      match.facingCache = { count: match.events.length, map };
    }
    return match.facingCache.map;
  }

  return { FPS, FLAG, PIANIST_SOURCES, isChange, passAt, parseRecord, parseStrategist, parseJev, jevMinutes, parseCensus, parseTruth, parseBotLog, squadPosts, indexAt, range, stateAt, rulesInMinute, lanes, clock, gridName, ordersAt, unitHistory, facings };
})();

if (typeof module !== "undefined") module.exports = WR;
