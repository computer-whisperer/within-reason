// Within Reason match viewer: the side panels, by who decides. The commander (its direction, marks, signals, chat,
// notes), the player of a seat (its packet, lists and allowances in force, its turns), the hands (the gate at the
// playhead: every actor's menu with each move's state, the pick, what was played, the audits), what the hands
// cost, the selected unit, the decision list, the log; and the match browser. Loaded after app.js, which it starts.
"use strict";

(() => {
  const { $, el, view, COLOR, seatColor, classOf, defName, playerOf, gateAt, pictureAt, seek, select, setTab, setScope, focusActor } = WV;
  const clock = WR.clock;
  const money = (dollars) => `$${dollars.toFixed(dollars < 10 ? 2 : 1)}`;
  const millions = (tokens) => `${(tokens / 1e6).toFixed(tokens < 1e7 ? 2 : 1)}M`;
  const multi = () => view.match.seats.length > 1;
  const scopedSeat = () => (view.scope != null ? view.match.seats[view.scope] : view.match.seats.length === 1 ? view.match.seats[0] : null);
  const short = (value, limit) => {
    const text = typeof value === "string" ? value : JSON.stringify(value);
    return text.length > limit ? `${text.slice(0, limit)} ...` : text;
  };

  function details(summary, text, open = false) {
    const d = el("details");
    d.open = open;
    d.append(el("summary", null, summary), el("pre", null, text));
    d.addEventListener("click", (e) => e.stopPropagation());
    return d;
  }

  function bar(label, p, strong, note) {
    const row = el("div", `bar${strong ? " played" : ""}`);
    const fill = el("i");
    fill.style.width = `${Math.max(1, 100 * Math.min(1, p))}%`;
    const name = el("span", null, label);
    name.title = label;
    row.append(name, fill, el("span", null, `${Number(p).toFixed(2)}${note || ""}`));
    return row;
  }

  /// A panel that is rebuilt only when its key changes (the playhead moves many times a second).
  function keyed(box, key) {
    if (box.dataset.key === key) return false;
    box.dataset.key = key;
    box.textContent = "";
    return true;
  }

  const section = (box, title, note) => {
    const h = el("h2", "call-title", `${title} `);
    if (note) h.append(el("span", "muted", note));
    box.append(h);
    return h;
  };

  const seatTag = (index) => {
    const tag = el("span", "seat-tag", view.match.seats[index].name);
    tag.style.borderColor = seatColor(index);
    tag.style.color = seatColor(index);
    return tag;
  };

  // ---------------------------------------------------------------- the seats and the stats

  /// The seat selector: all seats or one. Shown when the match has more than one seat of ours.
  function renderSeats() {
    const box = $("seats");
    box.hidden = !multi();
    if (box.hidden) return;
    box.textContent = "";
    const button = (label, scope, color) => {
      const b = el("button", view.scope === scope ? "active" : null, label);
      if (color) b.style.borderColor = color;
      if (color && view.scope === scope) b.style.background = color;
      b.addEventListener("click", () => setScope(scope));
      box.append(b);
    };
    box.append(el("span", "muted", "seats"));
    button("all", null, null);
    for (const seat of view.match.seats) button(seat.name, seat.index, seatColor(seat.index));
  }

  function renderNow() {
    const match = view.match;
    const total = WR.totalsAt(match, view.scope, view.frame);
    const now = $("now");
    now.textContent = "";
    if (!total.seats) return;
    const whole = view.scope == null || !multi();
    const theirs = whole ? match.truth[WR.indexAt(match.truth, view.frame)] : null;
    const stat = (label, value, other) => {
      const div = el("div", "stat");
      const b = el("b", null, value);
      if (other != null) {
        const small = el("small", null, ` / ${other}`);
        small.title = "theirs, from the ground truth";
        b.append(small);
      }
      div.append(b, el("span", null, label));
      now.append(div);
    };
    stat(`metal (+${total.metalIncome.toFixed(1)} / -${total.metalUse.toFixed(1)})`, Math.round(total.metal));
    stat(`energy of ${Math.round(total.energyStore)} (+${total.energyIncome.toFixed(0)} / -${total.energyUse.toFixed(0)})`, Math.round(total.energy));
    stat("extractors", total.extractors, theirs?.extractors);
    stat("army", total.army, theirs?.army);
    stat("army metal", Math.round(total.armyMetal), theirs ? Math.round(theirs.armyMetal) : null);
    stat("builders", total.builders);
    stat("enemies in view", total.enemiesVisible);
    const hands = WR.scoped(match, view.scope).filter((s) => s.jev);
    if (hands.length) {
      const minute = Math.floor(view.frame / (60 * WR.FPS));
      let tokens = 0;
      for (const s of hands) for (const m of s.jev.index.minutes) if (m.minute <= minute) tokens += Object.values(m.tokens).reduce((a, b) => a + b, 0);
      stat("Jev so far (to this minute's end)", money((tokens * WR.JEV_PRICE) / 1e6));
    }
  }

  // ---------------------------------------------------------------- the commander

  function turnLine(turn, match) {
    const parts = [`began ${clock(turn.f)}`];
    if (turn.wallSeconds != null) parts.push(`${turn.wallSeconds.toFixed(1)} s of thinking${match.realtime ? `, its orders in force from ${clock(WR.turnEnd(match, turn))}` : ""}`);
    else parts.push("still thinking, or cut off");
    if (turn.endedBy) parts.push(`ended by ${turn.endedBy}`);
    if (turn.cost != null) parts.push(`$${turn.cost.toFixed(3)} list price`);
    if (turn.stopped) parts.push(`stopped: ${turn.stopped}`);
    return parts.join(" · ");
  }

  /// The last turn of a transcript at the playhead, with what it said and called.
  function lastTurn(box, transcript) {
    const match = view.match;
    const i = WR.indexAt(transcript.turns, view.frame);
    if (i < 0) return void box.append(el("div", "muted", "no turn yet"));
    const turn = transcript.turns[i];
    section(box, `Turn ${i + 1} of ${transcript.turns.length}`, turnLine(turn, match));
    if (turn.wake) box.append(el("div", "wake", `woken: ${turn.wake}`));
    for (const text of turn.said) box.append(el("div", "said", text));
    for (const call of turn.calls) {
      const row = el("div", "call");
      row.append(el("b", null, call.tool), `(${short(call.arguments, 240)})`);
      if (call.result != null) row.append(el("span", "result", `  -> ${short(call.result, 160)}`));
      box.append(row);
    }
    if (turn.ordersResult) box.append(details("the orders call's answer", String(turn.ordersResult)));
    if (turn.thinking.length) box.append(details("thinking", turn.thinking.join("\n\n")));
    box.append(details("the report it was sent (prompt)", turn.prompt));
  }

  function noteList(box, transcript, title) {
    const notes = WR.callsOf(transcript, "note", view.frame);
    if (!notes.length) return;
    section(box, title, `${notes.length} so far, newest first`);
    const list = el("ol", "history");
    for (const n of notes.slice(-12).reverse()) {
      const li = el("li");
      li.append(el("span", "when", clock(n.f)), String(n.arguments.text || ""));
      li.addEventListener("click", () => seek(n.f));
      list.append(li);
    }
    box.append(list);
  }

  function renderCommander() {
    const match = view.match;
    const t = match.commander;
    const box = $("commander");
    if (!t) return;
    if (!keyed(box, `${view.version}:${view.scope}:${Math.floor(view.frame / WR.FPS)}`)) return;
    const direction = WR.directionAt(t, view.frame);
    const age = (f) => `written ${clock(f)}, ${WR.ago(view.frame - f)} ago`;
    section(box, "The direction in force", direction.plan ? `the plan ${age(direction.plan.f)}` : "none yet: the commander has not written one");
    if (direction.plan) box.append(el("div", "words plan", direction.plan.text));
    const seats = Object.entries(direction.seats).sort(([a], [b]) => a.localeCompare(b, undefined, { numeric: true }));
    for (const [name, part] of seats) {
      const seat = match.seats.find((s) => s.name === name);
      const block = el("div", `words seat${seat && view.scope === seat.index ? " selected" : ""}`);
      if (seat) block.style.borderLeftColor = seatColor(seat.index);
      const head = el("div", "meta");
      head.append(el("b", null, name), ` ${age(part.f)}`);
      block.append(head, part.text);
      if (seat) block.addEventListener("click", () => setScope(view.scope === seat.index ? null : seat.index));
      box.append(block);
    }
    const signals = WR.signalsAt(t, view.frame);
    if (signals.length) {
      section(box, "Signals", "the moment a waiting group goes");
      for (const s of signals) {
        const row = el("div", `signal${s.given ? " given" : ""}`);
        row.append(el("b", null, s.name), s.given ? ` given at ${clock(s.at)} (${WR.ago(view.frame - s.at)} ago)` : ` not yet given: set for ${clock(s.at)} (in ${WR.ago(s.at - view.frame)})`);
        row.addEventListener("click", () => seek(s.at));
        box.append(row);
      }
    }
    const marks = WR.marksAt(match, t, view.frame);
    if (marks.length) {
      section(box, "Marks", "places named for the whole side; drawn on the map");
      for (const m of marks) box.append(el("div", "signal", `${m.name}: ${WR.gridName(match, m.x, m.z)} (${Math.round(m.x)}, ${Math.round(m.z)}), marked ${clock(m.f)}`));
    }
    const transfers = WR.callsOf(t, "transfer", view.frame);
    if (transfers.length) {
      section(box, "Transfers", `${transfers.length} so far, newest first`);
      for (const c of transfers.slice(-6).reverse()) {
        const a = c.arguments;
        const what = a.units ? a.units.join(", ") : [a.metal ? `${a.metal} metal` : null, a.energy ? `${a.energy} energy` : null].filter(Boolean).join(", ");
        box.append(el("div", "signal", `${clock(c.f)}  ${what}${a.from ? ` from ${a.from}` : ""} to ${a.to}`));
      }
    }
    const said = WR.callsOf(t, "say", view.frame).map((c) => ({ f: c.f, who: "commander", text: c.arguments.text }));
    const chat = WR.chatUpTo(match, view.frame).map((e) => ({ f: e.f, who: `player ${e.player}`, text: e.text }));
    if (said.length || chat.length) {
      section(box, "Chat", `${chat.length} lines in the game, ${said.length} say calls; newest first`);
      const list = el("ol", "history");
      for (const line of [...said, ...chat].sort((a, b) => b.f - a.f).slice(0, 14)) {
        const li = el("li", line.who === "commander" ? "llm" : null);
        li.append(el("span", "when", clock(line.f)), `${line.who}: ${line.text}`);
        li.addEventListener("click", () => seek(line.f));
        list.append(li);
      }
      box.append(list);
    }
    noteList(box, t, "Notes");
    lastTurn(box, t);
  }

  // ---------------------------------------------------------------- the player

  function renderPlayer() {
    const match = view.match;
    const box = $("player");
    if (!match.players.size) return;
    if (!keyed(box, `${view.version}:${view.scope}:${Math.floor(view.frame / WR.FPS)}`)) return;
    const seat = scopedSeat();
    // All seats, a player each: one line a seat; a click narrows to it.
    if (!seat && match.players.size > 1) {
      section(box, "The players", "one a seat; click a seat for its packet, lists and turns");
      for (const s of match.seats) {
        const player = playerOf(s);
        if (!player) continue;
        const packet = WR.packetAt(match, player, view.frame);
        const i = WR.indexAt(player.turns, view.frame);
        const row = el("div", "words seat");
        row.style.borderLeftColor = seatColor(s.index);
        const head = el("div", "meta");
        head.append(el("b", null, s.name), ` turn ${i + 1} of ${player.turns.length}${i >= 0 ? `, the last began ${clock(player.turns[i].f)}` : ""}${packet ? ` · packet in force since ${clock(packet.from)} (${WR.ago(view.frame - packet.from)}), ${packet.text.length} characters` : " · no packet yet"}`);
        row.append(head, packet ? short(packet.text, 260) : "");
        row.addEventListener("click", () => setScope(s.index));
        box.append(row);
      }
      return;
    }
    const of = seat || match.seats[0];
    const player = playerOf(of);
    if (!player) return void box.append(el("div", "muted", `no player transcript for ${of.name}`));
    const shared = match.players.size === 1 && multi();
    const packet = WR.packetAt(match, player, view.frame);
    section(box, shared ? "The packet in force (one player for every seat)" : `The packet in force${multi() ? ` for ${of.name}` : ""}`, packet ? `since ${clock(packet.from)} (${WR.ago(view.frame - packet.from)} ago), written in the turn of ${clock(packet.f)}, ${packet.text.length} characters` : "none yet");
    if (packet) box.append(el("div", "words plan", packet.text));
    // A packet written in a turn that has not ended is not the hands' yet.
    const pending = player.turns.find((turn) => turn.f <= view.frame && WR.turnEnd(match, turn) > view.frame && turn.calls.some((c) => c.tool === "instruct"));
    if (pending) box.append(el("div", "wake", `a new packet is being written: its turn began ${clock(pending.f)} and ends ${clock(WR.turnEnd(match, pending))}`));
    if (match.commander) {
      const direction = WR.directionAt(match.commander, view.frame);
      const mine = direction.seats[of.name];
      if (direction.plan || mine) box.append(details("the commander's direction it reads beside its report", `${direction.plan ? `plan (${clock(direction.plan.f)}): ${direction.plan.text}\n\n` : ""}${mine ? `${of.name} (${clock(mine.f)}): ${mine.text}` : ""}`));
    }
    const suffix = multi() ? `_${of.name}` : "";
    const mineOnly = (key) => !shared || !/_t\d+$/.test(key) || key.endsWith(suffix);
    const lists = [...WR.latestByKey(player, "queue", view.frame)].filter(([key]) => mineOnly(key));
    if (lists.length) {
      section(box, "Build lists", "as last given, done by code in order; a list is not asked of the hands");
      for (const [key, { value, f }] of lists) {
        const actor = /_t\d+$/.test(key) || !multi() ? key : `${key}${suffix}`;
        const done = of.decisions.filter((d) => d.source === "list" && d.f >= f && d.f <= view.frame && d.inputs && d.inputs.actor === actor).length;
        const steps = Array.isArray(value) ? value : [String(value)];
        const row = el("div", "list");
        row.append(el("b", null, key), el("span", "muted", ` given ${clock(f)}; ${done} of ${steps.length} steps begun since`));
        const ol = el("ol");
        steps.forEach((step, i) => ol.append(el("li", i < done ? "done" : null, step)));
        row.append(ol);
        box.append(row);
      }
    }
    const produce = [...WR.latestByKey(player, "produce", view.frame)].filter(([key]) => mineOnly(key));
    if (produce.length) {
      section(box, "Produce allowances", "what each plant or builder may make");
      for (const [key, { value, f }] of produce) box.append(el("div", "signal", `${key}: ${Array.isArray(value) ? value.join(", ") : JSON.stringify(value)}  (given ${clock(f)})`));
    }
    const footwork = [...WR.latestByKey(player, "lane", view.frame)];
    if (footwork.length) {
      section(box, "Footwork", "the lane tool's settings");
      for (const [key, { value, f }] of footwork) box.append(el("div", "signal", `${key}: ${Array.isArray(value) ? value.join(", ") : value}  (given ${clock(f)})`));
    }
    const signals = WR.signalsAt(player, view.frame);
    const marks = WR.marksAt(match, player, view.frame);
    if (signals.length || marks.length) {
      section(box, "Its own signals and marks");
      for (const s of signals) box.append(el("div", `signal${s.given ? " given" : ""}`, `${s.name}: ${s.given ? `given at ${clock(s.at)}` : `set for ${clock(s.at)}`}`));
      for (const m of marks) box.append(el("div", "signal", `${m.name}: ${WR.gridName(match, m.x, m.z)}, marked ${clock(m.f)}`));
    }
    noteList(box, player, "Notes");
    lastTurn(box, player);
  }

  // ---------------------------------------------------------------- the hands

  const STATE_WORDS = {
    asked: "asked", audit: "asked for an audit (not played)", fuse: "not asked: fused by the decode", near: "not asked: shut by near", held: "not asked: held by its opener",
    blanked: "not asked: blanked by its place", standing: "not asked: an earlier answer stands (same / again)", news: "not asked: actor closed by news", unasked: "not asked",
  };
  const STATE_ORDER = ["asked", "audit", "standing", "held", "blanked", "near", "fuse", "news", "unasked"];
  const openActors = new Set();
  const openGroups = new Set();

  function countsWords(counts) {
    return STATE_ORDER.filter((k) => counts[k]).map((k) => `${counts[k]} ${k === "fuse" ? "fused" : k === "news" ? "closed" : k === "audit" ? "audited" : k === "near" ? "shut by near" : k}`).join(", ");
  }

  function renderHands() {
    const match = view.match;
    const head = $("hands-head");
    const box = $("hands");
    const seat = scopedSeat();
    if (!seat) {
      // Every seat's hands on one line each; a click narrows to the seat.
      head.textContent = "";
      if (!keyed(box, `all:${view.version}:${Math.floor(view.frame / WR.FPS)}`)) return;
      section(box, "The hands of every seat", "click a seat for its gate at the playhead");
      for (const s of match.seats) {
        const row = el("div", "words seat");
        row.style.borderLeftColor = seatColor(s.index);
        if (!s.jev) {
          row.append(el("b", null, s.name), " no log read yet");
        } else {
          const gates = s.jev.index.gates;
          const i = WR.lastAt(gates, view.frame);
          const minute = s.jev.index.minutes.find((m) => m.minute === Math.floor(view.frame / (60 * WR.FPS)));
          row.append(el("b", null, s.name), ` gate ${i + 1} of ${gates.length}${i >= 0 ? `, the last at ${clock(gates[i])}` : ""} · this minute: ${minute ? `${minute.gates} gates, ${minute.quiet} quiet seconds, ${minute.questions} questions, ${millions(Object.values(minute.tokens).reduce((a, b) => a + b, 0))} tokens` : "nothing"} · the game: ${money(s.jev.index.dollars)}`);
        }
        row.addEventListener("click", () => setScope(s.index));
        box.append(row);
      }
      return;
    }
    if (!seat.jev) {
      head.textContent = "";
      if (keyed(box, `none:${seat.index}`)) box.append(el("div", "muted", `${seat.name}: its log (jev-${seat.header.ai_id}.jsonl) is not read yet, or the match has none; the hands' logs are read through run/view_match.py`));
      return;
    }
    if (seat.jev.index.version < WR.JEV_VERSION) {
      head.textContent = "";
      if (keyed(box, `old:${seat.index}`)) box.append(el("div", "muted", `This log is of version ${seat.jev.index.version}: the hands before their rebuild (2026-10-02). The viewer reads version ${WR.JEV_VERSION} and newer; its cost is on the Cost tab.`));
      return;
    }
    const gate = gateAt(seat, view.frame);
    if (gate === undefined) {
      head.textContent = "loading the gate at the playhead ...";
      return;
    }
    if (gate === null) {
      head.textContent = "";
      if (keyed(box, `before:${seat.index}`)) box.append(el("div", "muted", "before the hands' first gate"));
      return;
    }
    // The head line follows the playhead; the body is rebuilt when the gate or what is opened changes.
    if (keyed(head, `${seat.index}:${gate.f}`)) {
      const title = el("h2", null, `${multi() ? `${seat.name}: ` : ""}the gate of ${clock(gate.f)} `);
      const answered = gate.answeredF == null ? "not answered yet" : gate.answeredF === gate.f ? "answered in the same second" : `answered ${((gate.answeredF - gate.f) / WR.FPS).toFixed(1)} s of game later`;
      title.append(el("span", "muted", `${answered} · ${gate.asked} questions${gate.call ? ` · ${gate.call.tokens.toLocaleString()} tokens in ${gate.call.batches} request${gate.call.batches > 1 ? "s" : ""}, ${gate.call.ms} ms` : ""}${gate.pick ? ` · the pick ${gate.pick.tokens.toLocaleString()} tokens, ${gate.pick.ms} ms` : ""}`));
      const nav = el("div", "gate-nav");
      const step = (label, frame) => {
        const b = el("button", null, label);
        b.disabled = frame == null;
        b.addEventListener("click", () => seek(frame));
        nav.append(b);
      };
      step("◀ the gate before", gate.prev);
      step("the next gate ▶", gate.next);
      head.append(title, nav);
    }
    if (!keyed(box, `${seat.index}:${gate.f}:${view.actor ? view.actor.name : ""}:${[...openActors].join(",")}:${[...openGroups].join(",")}`)) return;

    section(box, "What opened it", gate.events.length ? null : "no event: the picture changed, or an actor's own re-ask came due");
    for (const e of gate.events) box.append(el("div", "signal", e));
    for (const e of [...gate.hunts, ...gate.rove]) box.append(el("div", "signal muted", `in code: ${e}`));
    for (const e of gate.errors) box.append(el("div", "wake", `failed: ${e.error}`));
    for (const s of gate.shed) box.append(el("div", "wake", `the picture was over its size and shed: ${JSON.stringify(s.shed)}`));

    section(box, "The layers", "what each kept from being asked at this gate");
    const table = el("table", "grid");
    const row = (cells, cls) => {
      const tr = el("tr", cls);
      cells.forEach((c, i) => tr.append(el(cls === "head" ? "th" : "td", i === 0 ? "name" : null, String(c))));
      table.append(tr);
    };
    row(["layer", "on", "not asked", "asked for audit", "audits answered", "faults"], "head");
    const LAYER_WORDS = { news: "news: an actor with a course and no news is closed", same: "same: a question in the words of its last ask", again: "again: an actor reopened by an opener or a place alone", fuse: "fuse: the decode read the packet as not giving the move", openers: "openers: its opener said no at the last gate", places: "places: its place said no at the last gate", near: "near: two free builders of its type are nearer (code)" };
    for (const [name, counts] of Object.entries(gate.layers)) {
      const audits = gate.audits.filter((a) => a.layer === name);
      row([LAYER_WORDS[name] || name, counts.on ? "on" : "off", counts.skipped, counts.audited, audits.length, audits.filter((a) => a.fault).length]);
    }
    box.append(table);
    const t = gate.totals;
    box.append(el("div", "meta", `${t.moves} moves on ${gate.actors.length} menus: ${t.course} courses, ${countsWords(t)}. Open: ${gate.open.join(", ") || "nobody"}.${gate.closed.length ? ` Closed by news: ${gate.closed.join(", ")}.` : ""}`));

    if (gate.parties.length) {
      section(box, "Enemy parties", "does it need answering? (asked at every gate)");
      for (const p of gate.parties) box.append(bar(p.name, p.noul ?? 0, p.noul >= WR.FLAG_BAR, p.asked ? "" : " stands"));
    }
    if (gate.places.length) {
      const openPlaces = gate.places.filter((p) => p.noul >= WR.FUSE_BAR);
      section(box, "Places", `does any of ours have a reason to go there now? ${openPlaces.length} of ${gate.places.length} at ${WR.FUSE_BAR} or over`);
      for (const p of openPlaces) box.append(bar(p.name, p.noul, true, p.asked ? "" : " stands"));
      const rest = gate.places.filter((p) => !(p.noul >= WR.FUSE_BAR));
      if (rest.length) {
        const d = el("details");
        d.open = openGroups.has("places");
        d.addEventListener("toggle", () => (d.open ? openGroups.add("places") : openGroups.delete("places")));
        d.append(el("summary", null, `${rest.length} places under the bar`));
        for (const p of rest) d.append(bar(p.name, p.noul ?? 0, false, p.asked ? "" : " stands"));
        box.append(d);
      }
    }

    section(box, "Every actor's menu", "click an actor for its moves; its units are ringed on the map");
    const focus = view.actor && view.actor.seat === seat.index ? view.actor.name : null;
    const actors = [...gate.actors].sort((a, b) => (b.name === focus) - (a.name === focus) || (b.counts.played - a.counts.played) || (a.quiet - b.quiet));
    for (const actor of actors) {
      const opened = actor.name === focus || openActors.has(actor.name);
      const block = el("div", `slot${actor.name === focus ? " selected" : ""}${actor.quiet ? " closed" : ""}`);
      const line = el("div", "id");
      line.append(el("b", null, actor.name), ` · ${actor.kind} · ${actor.course}${actor.idle ? " · idle" : ""} · ${actor.state}`);
      if (actor.counts.played) line.append(el("span", "playedmark", " · played"));
      line.addEventListener("click", () => {
        if (opened) { openActors.delete(actor.name); focusActor(seat.index, actor.name === focus ? null : focus); }
        else { openActors.add(actor.name); focusActor(seat.index, actor.name); }
      });
      block.append(line);
      if (actor.change) block.append(bar("should it change course?", actor.change.noul, actor.change.noul >= WR.FLAG_BAR, actor.change.asked ? "" : " stands"));
      if (actor.newsAudit) block.append(el("div", `ask${actor.newsAudit.fault ? " fault" : ""}`, `audit of news: change ${Number(actor.newsAudit.change).toFixed(2)}, its best move ${actor.newsAudit.best || "none"} ${actor.newsAudit.best_p != null ? Number(actor.newsAudit.best_p).toFixed(2) : ""}${actor.newsAudit.fault ? ": a fault (it would have gone to the pick)" : ": no fault"}`));
      block.append(el("div", "ask", `${actor.moves.length} moves: ${countsWords(actor.counts) || "its course alone"}`));
      if (opened) renderMoves(block, actor);
      box.append(block);
    }

    section(box, "The pick", gate.components.length ? (gate.planned ? null : "worlds were composed; no plan row followed (the pick failed, or is not logged yet)") : "the gate flagged nothing: no worlds, no pick");
    gate.components.forEach((c, j) => {
      const verdict = c.taken == null ? "no answer" : c.taken ? `took w${c.candidate}` : c.candidate ? `kept w1 over w${c.candidate}` : "kept w1";
      const block = el("div", "step plan");
      block.append(el("div", "step-title", `${gate.components.length > 1 || c.actors ? `component ${j + 1}${c.actors ? `: ${c.actors.join(", ")}` : ""} · ` : ""}${verdict}${c.confidence != null ? ` at ${Number(c.confidence).toFixed(2)}` : ""}${c.sampledP != null ? ` (the candidate's noul ${Number(c.sampledP).toFixed(2)})` : ""}`));
      c.lines.forEach((text, i) => {
        const world = el("div", `world${c.taken && c.candidate === i + 1 ? " picked" : ""}${c.candidate === i + 1 ? " candidate" : ""}`);
        world.append(el("span", "id", `w${i + 1} `), el("span", "words", text));
        block.append(world);
      });
      box.append(block);
    });

    section(box, "Played", gate.played.length ? "by the lists and rules before the ask, and by the pick after it" : "nothing: every actor keeps its course");
    for (const d of gate.played) {
      const line = el("div", "pass-line play");
      line.append(el("span", "source", d.source || "?"), ` ${d.actor}: ${d.did || d.played}`);
      line.addEventListener("click", () => focusActor(seat.index, d.actor));
      box.append(line);
    }

    if (gate.audits.length) {
      section(box, "Audits of this gate", "a share of what a layer kept back is asked anyway: what it assumed beside the fresh answer");
      const audits = el("table", "grid");
      const head2 = el("tr", "head");
      for (const h of ["layer", "question", "assumed", "fresh", "fault"]) head2.append(el("th", null, h));
      audits.append(head2);
      for (const a of gate.audits) {
        const tr = el("tr", a.fault ? "fault" : null);
        const fresh = a.fresh != null ? Number(a.fresh).toFixed(2) : a.change != null ? `change ${Number(a.change).toFixed(2)}${a.best ? `, ${a.best} ${Number(a.best_p).toFixed(2)}` : ""}` : a.split != null ? `split took ${JSON.stringify(a.split)}, joint ${JSON.stringify(a.joint)}` : "";
        const assumed = typeof a.assumed === "number" ? a.assumed.toFixed(2) : String(a.assumed ?? "");
        for (const c of [a.layer, a.id || a.actor || "", `${assumed}${a.opener != null ? ` (opener ${Number(a.opener).toFixed(2)})` : ""}${a.place != null ? ` (place ${Number(a.place).toFixed(2)})` : ""}`, fresh, a.fault ? "FAULT" : "no"]) tr.append(el("td", null, c));
        audits.append(tr);
      }
      box.append(audits);
    }

    section(box, "What Jev was sent", gate.call ? `${gate.call.model || "Jev"}; the questions are the moves' words above` : "the call is not in the log");
    if (gate.instructions) box.append(details(`the packet as the hands had it (logged ${clock(gate.instructionsF)})`, gate.instructions));
    if (gate.call) for (const [name, value] of Object.entries(gate.call.state)) box.append(details(`picture: ${name}`, typeof value === "string" ? value : JSON.stringify(value, null, 1)));
  }

  /// An opened actor's moves, by state: the asked ones as bars, the rest folded by reason.
  function renderMoves(block, actor) {
    const moveRow = (mv, withWhy) => {
      const row = el("div", `state${mv.played ? " played" : ""}`);
      if (mv.noul != null) row.append(bar(mv.key, mv.noul, mv.noul >= WR.FLAG_BAR || !!mv.played, mv.played ? " played" : ""));
      else row.append(el("div", "ask key", mv.key));
      if (mv.forbidden) row.append(bar("forbidden by the packet?", mv.forbidden.noul, false, mv.forbidden.asked ? "" : " stands"));
      if (withWhy) row.append(el("div", `ask why${mv.audit && mv.audit.fault ? " fault" : ""}`, `${mv.why}${mv.audit ? `; the audit: fresh ${mv.audit.fresh != null ? Number(mv.audit.fresh).toFixed(2) : "?"}, ${mv.audit.fault ? "a FAULT" : "no fault"}` : ""}`));
      row.append(el("div", "ask words", mv.words));
      return row;
    };
    const course = actor.moves[0];
    if (course) block.append(el("div", "ask words", `its course: ${course.words}`));
    for (const state of STATE_ORDER) {
      const moves = actor.moves.filter((mv) => mv.state === state).sort((a, b) => (b.noul ?? -1) - (a.noul ?? -1));
      if (!moves.length) continue;
      if (state === "asked" || state === "audit") {
        block.append(el("div", "meta", `${STATE_WORDS[state]} (${moves.length}):`));
        for (const mv of moves) block.append(moveRow(mv, state === "audit" || !!mv.audit));
        continue;
      }
      const key = `${actor.name}:${state}`;
      const d = el("details", "group");
      d.open = openGroups.has(key);
      d.append(el("summary", null, `${STATE_WORDS[state]} (${moves.length})`));
      const fill = () => {
        if (d.dataset.filled) return;
        d.dataset.filled = "1";
        for (const mv of moves.slice(0, 400)) d.append(moveRow(mv, true));
        if (moves.length > 400) d.append(el("div", "meta", `... and ${moves.length - 400} more`));
      };
      if (d.open) fill();
      d.addEventListener("toggle", () => {
        if (d.open) { openGroups.add(key); fill(); } else openGroups.delete(key);
      });
      d.addEventListener("click", (e) => e.stopPropagation());
      block.append(d);
    }
  }

  // ---------------------------------------------------------------- the cost

  function renderCost() {
    const match = view.match;
    const box = $("cost");
    const minuteNow = Math.floor(view.frame / (60 * WR.FPS));
    if (!keyed(box, `${view.version}:${view.scope}:${minuteNow}`)) return;
    const hands = WR.scoped(match, view.scope).filter((s) => s.jev);
    if (!hands.length) return void box.append(el("div", "muted", "no hands' log read yet (the logs are read through run/view_match.py; a big one takes a few seconds the first time)"));
    const cost = WR.costView(hands.map((s) => ({ seat: s.index, index: s.jev.index })));
    const share = (tokens) => `${cost.tokens ? Math.round((100 * tokens) / cost.tokens) : 0}%`;
    section(box, `Jev: ${money(cost.dollars)}`, `${millions(cost.tokens)} input tokens at $${WR.JEV_PRICE} a million${view.scope == null && multi() ? `, ${hands.length} seats` : multi() ? `, seat ${hands[0].name}` : ""}; the output is free`);
    const table = (heads) => {
      const t = el("table", "grid");
      const tr = el("tr", "head");
      for (const h of heads) tr.append(el("th", null, h));
      t.append(tr);
      return t;
    };
    const cells = (t, values, cls) => {
      const tr = el("tr", cls);
      values.forEach((v, i) => tr.append(el("td", i === 0 ? "name" : null, String(v))));
      t.append(tr);
      return tr;
    };
    if (cost.seats.length > 1) {
      const t = table(["seat", "tokens", "dollars", "share"]);
      for (const s of cost.seats) {
        const tr = cells(t, [match.seats[s.seat].name, millions(s.tokens), money(s.dollars), share(s.tokens)]);
        tr.firstChild.style.color = seatColor(s.seat);
        tr.addEventListener("click", () => setScope(s.seat));
      }
      box.append(t);
    }
    section(box, "By request kind");
    const kinds = table(["request", "tokens", "dollars", "share"]);
    const KIND_WORDS = { gate: "the gate: every open actor's questions, the places, the parties", pick: "the pick: which of the flagged changes to play", decode: "the decode: what the packet gives each actor" };
    for (const [kind, tokens] of Object.entries(cost.kinds)) cells(kinds, [KIND_WORDS[kind] || kind, millions(tokens), money((tokens * WR.JEV_PRICE) / 1e6), share(tokens)]);
    box.append(kinds);
    section(box, "By question family", "as run/jev_products.py counts them: a gate's tokens split by the characters of its picture and of each question");
    const top = cost.families.length ? cost.families[0][1] : 1;
    for (const [family, tokens] of cost.families) {
      const row = el("div", "bar family");
      const fill = el("i");
      fill.style.width = `${Math.max(1, (100 * tokens) / top)}%`;
      row.append(el("span", null, family), fill, el("span", null, `${millions(tokens)} ${share(tokens)}`));
      box.append(row);
    }
    section(box, "By game minute", "click a minute to go there");
    const bySeat = cost.seats.length > 1 && cost.seats.length <= 8;
    const minutes = table(["min", "$", ...(bySeat ? cost.seats.map((s) => match.seats[s.seat].name) : []), "gate", "pick", "decode", "gates", "quiet s", "picks", "taken", "questions", "med ms", "max ms", "audits", "faults", "failed", "most of it"]);
    for (const m of cost.minutes) {
      const dollars = (tokens) => ((tokens * WR.JEV_PRICE) / 1e6).toFixed(3);
      const first = [...m.families].sort((a, b) => b[1] - a[1])[0];
      const tr = cells(minutes, [m.minute, dollars(m.tokens), ...(bySeat ? cost.seats.map((s) => dollars(m.seats.get(s.seat) || 0)) : []), millions(m.kinds.gate || 0), millions(m.kinds.pick || 0), millions(m.kinds.decode || 0), m.gates, m.quiet, m.picks, m.taken, m.questions, m.medianMs, m.maxMs, m.audits, m.faults, m.errors, first ? `${first[0]} ${Math.round((100 * first[1]) / Math.max(1, m.tokens))}%` : ""], m.minute === minuteNow ? "current" : null);
      tr.addEventListener("click", () => seek(m.minute * 60 * WR.FPS));
    }
    box.append(minutes);
  }

  // ---------------------------------------------------------------- the unit

  /// The selected unit: what it is, whose, its standing order, its actor in the hands' picture with that actor's
  /// last plays, and its history.
  function renderUnit() {
    if (view.selected == null) return;
    const match = view.match;
    const box = $("unit");
    const id = view.selected;
    const state = view.state || WR.stateAt(match, view.frame);
    const ours = state.own.find((u) => u.id === id);
    const u = ours || state.enemies.find((v) => v.id === id);
    const seat = ours ? match.seats[ours.seat] : null;
    const picture = seat ? pictureAt(seat, view.frame) : null;
    if (!keyed(box, `${id}:${view.version}:${Math.floor(view.frame / WR.FPS)}:${picture ? picture.f : "-"}`)) return;
    const d = u && u.def >= 0 ? match.defs[u.def] : null;
    const history = WR.unitHistory(match, id);
    const born = history.events.find((e) => e.k === "created" && e.u === id);
    const dead = history.events.find((e) => (e.k === "destroyed" || e.k === "enemy_destroyed") && e.u === id);
    const title = el("h2", null, `${d ? d.name : "unit"} #${id} `);
    title.append(el("span", "muted", ours ? (multi() ? `ours, seat ${seat.name}` : "ours") : u ? "enemy" : dead && dead.f <= view.frame ? `gone at ${clock(dead.f)}` : "not on the map at the playhead"));
    box.append(title);
    const facts = el("div", "facts");
    const fact = (k, v) => facts.append(el("span", "k", k), el("span", "v", v));
    if (d) fact("type", `${d.class}${d.metal ? `, ${d.metal} metal` : ""}${d.reach ? `, reach ${d.reach}` : ""}${d.footprint ? `, footprint ${d.footprint[0] * 8} x ${d.footprint[1] * 8}` : ""}`);
    if (u) {
      fact("at", `${WR.gridName(match, u.x, u.z)} (${Math.round(u.x)}, ${Math.round(u.z)})`);
      fact("health", ours ? `${u.health}%` : String(u.health));
      const flags = [];
      if (u.flags & WR.FLAG.beingBuilt) flags.push("being built");
      if (ours && u.flags & WR.FLAG.idle) flags.push("idle");
      if (flags.length) fact("flags", flags.join(", "));
      if (u.damage) fact("damage", `${u.damage} this second`);
    }
    if (born) fact("created", `${clock(born.f)}${born.by != null ? ` by #${born.by}` : ""}`);
    for (const e of history.events.filter((e) => e.k === "given" && e.u === id)) fact("given", `${clock(e.f)} from t${e.from_team} to t${e.to_team}`);
    if (dead) fact("destroyed", `${clock(dead.f)}${dead.by_d != null && dead.by_d >= 0 ? ` by a ${defName(dead.by_d)}` : ""}`);
    if (ours) {
      const order = WR.ordersAt(seat, view.frame).get(id);
      fact("standing order", u.flags & WR.FLAG.idle ? "none (idle)" : order ? orderWords(order) : "none recorded");
    }
    box.append(facts);
    // Its part in the hands' picture: a builder or a plant by its own name, a soldier by its group.
    if (ours && seat.jev) {
      const actor = picture ? WR.actorOfUnit(picture, id, d ? d.class : null) : null;
      if (picture === undefined) box.append(el("div", "meta", "loading the hands' picture ..."));
      else if (actor) {
        box.append(el("h2", "call-title", `in the hands' picture of ${clock(picture.f)}: ${actor}`));
        const raw = picture.actors[actor];
        const entry = typeof raw === "string" ? { doing: raw } : raw || {};
        const lines = el("div", "picture");
        for (const [key, value] of Object.entries(entry)) lines.append(el("div", null, `${key}: ${typeof value === "string" ? value : JSON.stringify(value)}`));
        const group = (picture.groups || []).find((g) => (g.members || []).includes(id));
        if (group) lines.append(el("div", null, `group of ${group.members.length}: ${group.task ? `${group.task.kind}${group.task.ward ? ` ${group.task.ward}` : ""}` : "hold"}`));
        box.append(lines);
        const plays = WR.actorPlays(seat, actor, view.frame);
        box.append(el("div", "meta", plays.length ? `its plays up to now (${plays.length}), newest first:` : "no play of its own in the record yet"));
        for (const p of plays.slice(-8).reverse()) {
          const line = el("div", "hist");
          line.append(`${clock(p.f)} `, el("span", "source", WR.playSource(p.source)), " ", el("b", null, (p.inputs.move || p.inputs.step || "").replace(`${actor}.`, "")), p.outputs && p.outputs.did ? ` · ${p.outputs.did}` : "");
          line.addEventListener("click", () => seek(p.f));
          box.append(line);
        }
        const open = el("button", null, "its menu at the gate, in the Hands tab");
        open.addEventListener("click", () => focusActor(seat.index, actor, "hands"));
        box.append(open);
      } else if (picture) box.append(el("div", "meta", "not an actor of the hands' picture at the playhead (a building, or a unit not in a group yet)"));
    }
    // Its history: orders and events, newest first, click to seek there.
    box.append(el("h2", "call-title", `history: ${history.commands.length} orders, ${history.events.length} events`));
    const list = el("ol", "history");
    const rows = [...history.commands.map((o) => ({ f: o.f, text: orderWords(o) })), ...history.events.map((e) => ({ f: e.f, text: eventWords(e, id) }))].sort((a, b) => b.f - a.f);
    for (const r of rows.slice(0, 80)) {
      const li = el("li", r.f > view.frame ? "future" : null);
      li.append(el("span", "when", clock(r.f)), r.text);
      li.addEventListener("click", () => seek(r.f));
      list.append(li);
    }
    box.append(list);
  }
  function orderWords(o) {
    const at = o.x != null ? ` at ${WR.gridName(view.match, o.x, o.z)} (${Math.round(o.x)}, ${Math.round(o.z)})` : o.target != null ? ` #${o.target}` : o.feature != null ? ` feature ${o.feature}` : "";
    return `${o.kind}${o.def != null ? ` ${defName(o.def)}` : ""}${at}${o.radius ? ` within ${o.radius}` : ""}`;
  }
  function eventWords(e, id) {
    if (e.k === "created") return e.u === id ? `created${e.by != null ? ` by #${e.by}` : ""}` : `began a ${defName(e.d)} #${e.u} at ${WR.gridName(view.match, e.x, e.z)}`;
    if (e.k === "finished") return e.u === id ? "finished" : `finished #${e.u}`;
    if (e.k === "destroyed") return `destroyed${e.by_d != null && e.by_d >= 0 ? ` by a ${defName(e.by_d)}` : ""}`;
    if (e.k === "given") return `given from t${e.from_team} to t${e.to_team}`;
    return e.k.replace(/_/g, " ");
  }

  // ---------------------------------------------------------------- the decision list

  const show = { commander: true, players: true, hands: true, events: false };
  let decisionItems = [];
  let currentDecision = -2;

  /// Everything decided, in order, for the scope: the commander's turns, the players' turns, the hands' plays (the
  /// pick's and the lists'), and the brain's events.
  function decisionRows() {
    const match = view.match;
    const rows = [];
    if (match.commander && show.commander) for (const turn of match.commander.turns) rows.push({ f: turn.f, kind: "turn", who: "commander", turn });
    const done = new Set();
    for (const seat of WR.scoped(match, view.scope)) {
      const player = playerOf(seat);
      if (player && show.players && !done.has(player)) {
        done.add(player);
        for (const turn of player.turns) rows.push({ f: turn.f, kind: "turn", who: match.players.size > 1 ? `player ${seat.name}` : "player", seat: seat.index, turn });
      }
      for (const d of seat.decisions) {
        if (d.kind === "event") {
          if (show.events) rows.push({ f: d.f, kind: "event", seat: seat.index, d });
        } else if (show.hands && WR.PLAY_SOURCES.has(d.source) && d.inputs && d.inputs.actor) {
          if (view.actor && (view.actor.seat !== seat.index || view.actor.name !== d.inputs.actor)) continue;
          rows.push({ f: d.f, kind: "play", seat: seat.index, d });
        }
      }
    }
    return rows.sort((a, b) => a.f - b.f);
  }

  function buildDecisionList() {
    const list = $("decisions");
    list.textContent = "";
    decisionItems = [];
    currentDecision = -2;
    const filters = $("decision-filters");
    filters.textContent = "";
    const words = { commander: "commander's turns", players: "players' turns", hands: "hands' plays", events: "brain events" };
    for (const key of Object.keys(show)) {
      if ((key === "commander" && !view.match.commander) || (key === "players" && !view.match.players.size)) continue;
      const label = el("label");
      const box = el("input");
      box.type = "checkbox";
      box.checked = show[key];
      box.addEventListener("change", () => {
        show[key] = box.checked;
        buildDecisionList();
        renderDecisions();
      });
      label.append(box, ` ${words[key]}`);
      filters.append(label);
    }
    if (view.actor) {
      const chip = el("button", "chip", `only ${view.actor.name} ✕`);
      chip.addEventListener("click", () => focusActor(null, null));
      filters.append(chip);
    }
    const rows = decisionRows();
    const LIMIT = 4000;
    for (const r of rows.slice(0, LIMIT)) {
      const li = el("li", r.kind === "turn" ? "llm" : r.kind === "play" ? "jev" : "heuristic");
      if (r.seat != null && multi()) li.style.borderLeftColor = seatColor(r.seat);
      const head = el("div");
      head.append(el("span", "when", clock(r.f)));
      if (r.kind === "turn") {
        const turn = r.turn;
        head.append(el("span", "wake", turn.wake ? `woken: ${turn.wake}` : "turn"), el("span", "source", r.who));
        li.append(head);
        for (const text of turn.said) li.append(el("div", "said", text));
        for (const call of turn.calls) {
          const row = el("div", "call");
          row.append(el("b", null, call.tool), `(${short(call.arguments, 400)})`);
          if (call.result != null) {
            const result = short(call.result, 160);
            row.append(el("span", "result", `  -> ${result}`));
            if (result.endsWith(" ...")) li.append(details(`full result of ${call.tool}`, typeof call.result === "string" ? call.result : JSON.stringify(call.result, null, 1)));
          }
          li.append(row);
        }
        if (turn.ordersResult) li.append(details("the orders call's answer", String(turn.ordersResult)));
        if (turn.thinking.length) li.append(details("thinking", turn.thinking.join("\n\n")));
        li.append(details("prompt", turn.prompt));
        li.append(el("div", "source", turnLine(turn, view.match)));
      } else if (r.kind === "play") {
        const d = r.d;
        head.append(el("span", null, `${d.inputs.actor}: ${(d.outputs && d.outputs.did) || d.inputs.move || d.inputs.step || ""}`), el("span", "source", `${d.source === "plan" ? "the pick" : d.source === "lane" ? "the lane" : "its list"}${multi() ? ` · ${view.match.seats[r.seat].name}` : ""}`));
        li.append(head);
      } else {
        head.append(el("span", null, String(r.d.outputs)), el("span", "source", `event${multi() ? ` · ${view.match.seats[r.seat].name}` : ""}`));
        li.append(head);
      }
      li.addEventListener("click", () => seek(r.f));
      list.append(li);
      decisionItems.push({ f: r.f, li });
    }
    if (rows.length > LIMIT) list.append(el("li", null, `... ${rows.length - LIMIT} more; narrow the filters or pick a seat`));
    if (!decisionItems.length) list.append(el("li", null, "nothing of the selected kinds"));
  }

  /// Rebuilds the list for a match that has grown, keeping the opened details open and the scroll position.
  function rebuildDecisionList() {
    const list = $("decisions");
    const key = (d) => `${d.closest("li").querySelector(".when").textContent} ${d.querySelector("summary").textContent}`;
    const opened = new Set([...list.querySelectorAll("details[open]")].map(key));
    const scroll = list.scrollTop;
    const current = currentDecision;
    buildDecisionList();
    for (const d of list.querySelectorAll("details")) if (opened.has(key(d))) d.open = true;
    // Following the newest decision scrolls by itself (renderDecisions); otherwise stay where the reader was.
    if (!$("follow").checked) {
      currentDecision = current;
      list.scrollTop = scroll;
      decisionItems.forEach((item, i) => {
        item.li.classList.toggle("future", i > current);
        item.li.classList.toggle("current", i === current);
      });
    }
  }

  function renderDecisions() {
    const current = WR.indexAt(decisionItems, view.frame);
    if (current === currentDecision) return;
    currentDecision = current;
    decisionItems.forEach((item, i) => {
      item.li.classList.toggle("future", i > current);
      item.li.classList.toggle("current", i === current);
    });
    if (current >= 0) {
      // Scroll inside the list only; scrollIntoView would move the page too.
      const list = $("decisions");
      const li = decisionItems[current].li;
      list.scrollTop = li.offsetTop - list.offsetTop - 24;
    }
  }

  // ---------------------------------------------------------------- the log

  function renderLog() {
    const match = view.match;
    const rules = WR.rulesInMinute(match, view.scope, view.frame);
    $("rules-minute").textContent = `(minute ${Math.floor(view.frame / (60 * WR.FPS))})`;
    const box = $("rules");
    box.textContent = "";
    const max = Math.max(1, ...rules.map(([, n]) => n));
    for (const [rule, n] of rules.slice(0, 10)) {
      const row = el("div", "rule");
      const fill = el("i");
      fill.style.width = `${(100 * n) / max}%`;
      row.append(el("span", null, rule), fill, el("span", null, n));
      box.append(row);
    }
    if (!rules.length) box.textContent = "none";
    renderBuildOrder();
    const ais = new Set(WR.scoped(match, view.scope).map((s) => s.header.ai_id));
    const lines = WR.range(match.botLog, view.frame - 60 * WR.FPS, view.frame).filter((l) => view.scope == null || ais.has(l.ai)).slice(-40);
    $("botlog").textContent = lines.length ? lines.map((l) => `${clock(l.f)}  ${multi() ? `[ai ${l.ai}] ` : ""}${l.text}`).join("\n") : match.botLog.length ? "" : "bot.log not loaded";
  }

  /// The opening as a list: every unit begun in the first eight minutes by the seats in scope, by whom and where.
  function renderBuildOrder() {
    const match = view.match;
    const box = $("build-order");
    if (!keyed(box, `${view.version}:${view.scope}`)) return;
    const rows = [];
    for (const seat of WR.scoped(match, view.scope)) {
      for (const e of seat.events) {
        if (e.f > 8 * 60 * WR.FPS) break;
        if (e.k === "created") rows.push({ e, seat });
      }
    }
    rows.sort((a, b) => a.e.f - b.e.f);
    for (const { e, seat } of rows.slice(0, 600)) {
      const by = e.by != null ? WR.seatStateAt(seat, e.f).own.find((v) => v.id === e.by) : null;
      const cls = by ? classOf(by.def) : null;
      const who = !by ? "?" : cls === "commander" ? "commander" : cls === "factory" ? `plant_${by.id}` : `${cls}_${by.id}`;
      const li = el("li");
      li.append(el("span", "when", clock(e.f)), `${multi() ? `${seat.name} ` : ""}${who}: ${defName(e.d)} #${e.u} at ${WR.gridName(match, e.x, e.z)}`);
      li.addEventListener("click", () => { seek(e.f + 1); select(e.u); });
      box.append(li);
    }
  }

  // ---------------------------------------------------------------- the match browser

  /// Every batch the server lists, newest first, each match a link into the viewer.
  function renderBrowser(index) {
    document.body.dataset.browser = "1";
    const box = $("browser");
    box.hidden = false;
    box.textContent = "";
    $("subtitle").textContent = `${index.root}: ${index.batches.length} ${index.kind === "batch" ? "batch" : "batches"}`;
    const filter = el("input");
    filter.type = "search";
    filter.placeholder = "filter by label, opponent, map, model, commit";
    const table = el("table", "batches");
    const head = el("tr");
    for (const h of ["started", "label", "teams", "opponent", "map", "player", "commander", "commit", "matches"]) head.append(el("th", null, h));
    table.append(head);
    const rows = [];
    for (const b of index.batches) {
      const tr = el("tr");
      const when = b.started ? new Date(b.started * 1000) : null;
      tr.append(el("td", "when", when ? `${when.toISOString().slice(0, 10)} ${when.toTimeString().slice(0, 5)}` : ""));
      tr.append(el("td", "label", b.label || b.batch));
      tr.append(el("td", null, b.ours != null ? `${b.ours + (b.allies || 0)} v ${b.enemies}${b.allies ? ` (${b.ours} ours)` : ""}` : ""));
      tr.append(el("td", null, [b.opponent, b.max_minutes ? `${b.max_minutes} min cap` : null].filter(Boolean).join(", ")));
      tr.append(el("td", null, b.map || ""));
      const players = b.player ? [b.player_model || "player", b.ours > 1 ? (b.players === "seat" ? "one a seat" : "one for all seats") : null, b.realtime ? "realtime" : null].filter(Boolean).join(", ") : b.packet ? `packet ${b.packet}` : "";
      tr.append(el("td", null, players));
      tr.append(el("td", null, b.commander || ""));
      tr.append(el("td", "mono", b.commit || ""));
      const cell = el("td", "matches");
      for (const m of b.matches || []) {
        const path = b.batch === "." ? `matches/${m.index ? `${m.index}/` : ""}` : `matches/${encodeURIComponent(b.batch)}/${m.index}/`;
        const a = el("a", `match ${m.record ? "" : "norecord"} ${(m.outcome || "").toLowerCase()}`);
        a.href = `?match=${path}`;
        const words = m.outcome ? `${m.outcome}${m.called ? " (called)" : ""}${m.minutes != null ? ` ${m.minutes.toFixed(1)} min` : ""}` : b.finished ? "ended, no result recorded" : "playing";
        a.textContent = `${m.index || "match"}: ${words}${m.arm ? ` (${m.arm})` : ""}${m.jev_dollars != null ? ` · Jev ~${money(m.jev_dollars)}` : ""}${m.record ? "" : " · no record"}`;
        a.title = `${b.batch}/${m.index}`;
        cell.append(a);
      }
      tr.append(cell);
      table.append(tr);
      rows.push({ tr, text: `${b.batch} ${b.label || ""} ${b.opponent || ""} ${b.map || ""} ${b.commit || ""} ${b.packet || ""} ${b.player_model || ""} ${b.commander || ""}`.toLowerCase() });
    }
    filter.addEventListener("input", () => {
      const q = filter.value.trim().toLowerCase();
      for (const r of rows) r.tr.hidden = !!q && !r.text.includes(q);
    });
    box.append(el("h2", null, "Matches"), filter, table);
    WV.status("pick a match; a batch without a result is still being played");
  }

  // ---------------------------------------------------------------- wiring

  Object.assign(WV.panels, {
    renderBrowser,
    matchOpened() { renderSeats(); buildDecisionList(); },
    matchGrew() { rebuildDecisionList(); },
    scopeChanged() { renderSeats(); buildDecisionList(); },
    render() {
      renderNow();
      // The decision list follows the hands' actor in focus.
      const actorKey = view.actor ? `${view.actor.seat}:${view.actor.name}` : "";
      if (actorKey !== (renderDecisions.actor || "")) {
        renderDecisions.actor = actorKey;
        buildDecisionList();
      }
      switch (view.tab) {
        case "commander": return renderCommander();
        case "player": return renderPlayer();
        case "hands": return renderHands();
        case "cost": return renderCost();
        case "unit": return renderUnit();
        case "decisions": return renderDecisions();
        case "log": return renderLog();
        default: return undefined;
      }
    },
  });
  WV.start();
})();
