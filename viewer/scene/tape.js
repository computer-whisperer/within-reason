// The game tape in the page: the reader (crates/tape, built to scene/tape.wasm by run/build_viewer.sh) and the byte
// ranges it is fed. A tape is read as its footer, its index, its tables, then a slice at a time around the playhead
// (docs/harness/tape-format.md). One tape a page.
"use strict";

const WRTape = (() => {
  let wasm = null;
  /// Slices kept decoded: the one under the playhead and its neighbours, the minute before it a unit's panel reads
  /// for its orders (`wantSpan`), and a few more for scrubbing back.
  const KEEP = 12;
  const UNIT_STRIDE = 16, PROJECTILE_STRIDE = 12, BURST_STRIDE = 8, DOING_STRIDE = 10, COMMAND_STRIDE = 8, HISTORY_STRIDE = 8;
  /// How long a command given stays in a state's `commands`, in frames (the reader's COMMAND_FRAMES).
  const COMMAND_FRAMES = 45;

  /// The engine's commands a unit stands under, by id: the word, and the game's own colour for its line (BAR's
  /// cmdcolors). A negative id is a build of that unit type.
  const COMMANDS = {
    0: ["stop"], 5: ["wait"], 10: ["move", [0.5, 1, 0.5]], 15: ["patrol", [0.2, 0.5, 1]], 16: ["fight", [1, 0.2, 1]],
    20: ["attack", [1, 0.2, 0.2]], 21: ["area attack", [1, 0.2, 0.2]], 25: ["guard", [0.6, 1, 1]], 40: ["repair", [1, 0.9, 0.2]],
    65: ["self-destruct"], 75: ["load", [0.4, 0.9, 0.9]], 76: ["load onto", [0.4, 0.9, 0.9]], 80: ["unload", [1, 0.8, 0]], 81: ["unload", [1, 0.8, 0]],
    90: ["reclaim", [0.5, 1, 0.4]], 105: ["manual fire", [1, 0.45, 0.1]], 110: ["restore", [0, 1, 0]], 125: ["resurrect", [0.9, 0.5, 1]],
    130: ["capture", [1, 1, 0.3]], 135: ["repair level"], 145: ["idle mode"], 34923: ["set target", [1, 0.5, 0.2]],
  };
  const BUILD = ["build", [0.3, 1, 0.3]];
  const OTHER_COLOR = [0.75, 0.75, 0.75];
  const commandName = (cmd) => (cmd < 0 ? BUILD : COMMANDS[cmd] || [`command ${cmd}`])[0];
  const commandColor = (cmd) => (cmd < 0 ? BUILD : COMMANDS[cmd] || [])[1] || OTHER_COLOR;

  async function reader() {
    if (!wasm) {
      const response = await fetch("scene/tape.wasm");
      if (!response.ok) throw new Error("scene/tape.wasm is not built (run/build_viewer.sh)");
      wasm = (await WebAssembly.instantiate(await response.arrayBuffer(), {})).instance.exports;
    }
    return wasm;
  }

  /// Hands bytes to a function of the reader that takes a pointer and a length.
  function give(bytes, call) {
    const ptr = wasm.wr_alloc(bytes.length);
    new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
    const answer = call(ptr, bytes.length);
    wasm.wr_free(ptr, bytes.length);
    return answer;
  }

  function text(which) {
    return new TextDecoder().decode(new Uint8Array(wasm.memory.buffer, wasm.wr_text(which), wasm.wr_text_len(which)));
  }

  function check(code, what) {
    if (code < 0) throw new Error(`${what}: ${text(9)}`);
    return code;
  }

  /// A view of one of the reader's arrays. Good until the reader's memory grows: taken fresh at every use.
  function array(which, Type) {
    return new Type(wasm.memory.buffer, wasm.wr_array(which), wasm.wr_array_len(which));
  }

  /// Bytes `first` to `last` of a file, or with `first` null its last `last` bytes; with them the file's size. Never
  /// from the browser's cache: a growing tape's footer moves, and the tape packed whole at its game's end puts other
  /// bytes at the offsets a cache would hold (the server is local).
  async function range(url, first, last) {
    const response = await fetch(url, { cache: "no-store", headers: { Range: first == null ? `bytes=-${last}` : `bytes=${first}-${last}` } });
    if (response.status !== 206) throw new Error(`${url}: the server does not answer byte ranges (status ${response.status}); the 3D view needs run/view_match.py`);
    const size = Number((response.headers.get("Content-Range") || "").split("/")[1] || 0);
    return { bytes: new Uint8Array(await response.arrayBuffer()), size };
  }

  /// The footer's word for a tape whose index stands before it: a finished tape, or one still growing between writes.
  const whole = (footer) => new TextDecoder().decode(footer.bytes.subarray(8)) === "WRTAPEND";

  /// The index the footer points at, handed to the reader by `call` (`wr_open` for a tape newly opened, `wr_reindex`
  /// for one that grew: the slices read stay), then everything but the slices, fetched together and handed in in the
  /// index's order (the unit table before any slice). Answers the tape's fields that come from them.
  async function readIndex(url, footer, call) {
    const indexAt = Number(new DataView(footer.bytes.buffer).getBigUint64(0, true));
    check(give((await range(url, indexAt, footer.size - 17)).bytes, call), "the tape's index");
    const chunks = [];
    for (let i = 0; i < wasm.wr_chunk_count(); i++) {
      const kind = wasm.wr_chunk(i, 0);
      chunks.push({ kind: String.fromCharCode(kind & 255, (kind >> 8) & 255, (kind >> 16) & 255, (kind >> 24) & 255), offset: wasm.wr_chunk(i, 1), length: wasm.wr_chunk(i, 2), first: wasm.wr_chunk(i, 3) });
    }
    const tables = chunks.filter((c) => c.kind !== "SLIC");
    const bodies = await Promise.all(tables.map((c) => range(url, c.offset, c.offset + c.length - 1)));
    bodies.forEach((body, i) => check(give(body.bytes, wasm.wr_set_chunk), `the tape's ${tables[i].kind} chunk`));
    const head = JSON.parse(text(0));
    const defs = JSON.parse(text(1));
    const fields = {
      size: footer.size, indexAt, head, defs,
      teams: JSON.parse(text(2) || "{}"), events: JSON.parse(text(3) || "[]"),
      lastFrame: head.last_frame, sliceFrames: head.slice_frames,
      /// Whether the tape is still being written (the arena's follower): read its footer again for more (`refresh`).
      growing: !!head.growing,
      sliceChunks: chunks.filter((c) => c.kind === "SLIC"),
      /// The unit table, a row a unit life (the row number is the tape's id of the unit): eight integers (engine id,
      /// type, team, the frames created, finished and destroyed (4294967295 never), killer row + 1, builder row + 1)
      /// and eight floats (full health, where it first stood, where it died, 1 when its type moves).
      unitInts: array(4, Uint32Array).slice(), unitFloats: array(5, Float32Array).slice(),
      /// Fourteen floats a feature: place, front, up, metal, energy, type, the frames it appeared and went (-1 never).
      features: array(6, Float32Array).slice(),
      ground: { columns: wasm.wr_ground_info(0), rows: wasm.wr_ground_info(1), low: wasm.wr_ground_info(2), step: wasm.wr_ground_info(3) },
      unitDefs: new Map(defs.units.map((d) => [d.id, d])),
      weaponDefs: new Map(defs.weapons.map((d) => [d.id, d])),
      featureDefs: new Map(defs.features.map((d) => [d.id, d])),
    };
    fields.unitCount = fields.unitInts.length / 8;
    // The rows that held each engine id, in the order created: the tie to the match record, which names a unit by
    // the engine's id.
    fields.rowsById = new Map();
    for (let row = 0; row < fields.unitCount; row++) {
      const id = fields.unitInts[row * 8];
      if (!fields.rowsById.has(id)) fields.rowsById.set(id, []);
      fields.rowsById.get(id).push(row);
    }
    return fields;
  }

  async function open(url) {
    await reader();
    const footer = await range(url, null, 16);
    if (!whole(footer)) throw new Error(`${url} is not a finished tape`);
    const fields = await readIndex(url, footer, wasm.wr_open);
    const tape = {
      url, ...fields,
      slices: fields.sliceChunks.map((c) => ({ ...c, state: "out", used: 0 })),
      onSlice: null, clock: 0, worked: null, workedLoads: 0,
      /// How many slices have been read: a count that changes when what `ordersOf` can see does.
      loads: 0,
    };
    return Object.assign(tape, methods);
  }

  const methods = {
    /// A tape still being written (`growing`): its footer read again, and when it has moved, the new index and tables;
    /// the slices read stay. A tape packed again whole at its game's end is a new, smaller file: read afresh. Answers
    /// whether the tape changed. Never throws: a tape caught between writes is asked again at the next call.
    async refresh() {
      try {
        const footer = await range(this.url, null, 16);
        const indexAt = whole(footer) ? Number(new DataView(footer.bytes.buffer).getBigUint64(0, true)) : null;
        if (indexAt == null || (indexAt === this.indexAt && footer.size === this.size)) return false;
        const afresh = footer.size < this.size;
        const before = new Map(this.slices.map((slice) => [slice.first, slice]));
        const fields = await readIndex(this.url, footer, afresh ? wasm.wr_open : wasm.wr_reindex);
        Object.assign(this, fields);
        this.slices = fields.sliceChunks.map((c) => {
          const old = before.get(c.first);
          return { ...c, state: !afresh && old && old.state === "in" ? "in" : "out", used: old ? old.used : 0 };
        });
        this.worked = null;
        this.loads++;
        return true;
      } catch (e) {
        console.error("the tape's refresh:", e);
        return false;
      }
    },

    sliceIndex(frame) {
      return Math.min(this.slices.length - 1, Math.max(0, Math.floor(frame / this.sliceFrames)));
    },

    /// Sees that every slice over frames `from` to `to` is loaded (what `ordersOf` reads). Answers how many are not in
    /// yet; `onSlice` is called as each arrives.
    wantSpan(from, to) {
      let missing = 0;
      for (let index = this.sliceIndex(from); index <= this.sliceIndex(to); index++) {
        const slice = this.slices[index];
        slice.used = ++this.clock;
        if (slice.state !== "in") missing++;
        if (slice.state === "out") this.load(index);
      }
      return missing;
    },

    /// Sees that the slice under a frame is loaded, and the next one (the playhead is going there), and near its start
    /// the one before. Answers whether the frame's own slice is ready now; `onSlice` is called when one arrives.
    want(frame) {
      const at = this.sliceIndex(frame);
      // (and the one before while its last orders still fade: a state's `commands`)
      const before = frame - at * this.sliceFrames < COMMAND_FRAMES ? [at - 1] : [];
      for (const index of [at, at + 1, ...before]) {
        const slice = this.slices[index];
        if (!slice) continue;
        slice.used = ++this.clock;
        if (slice.state === "out") this.load(index);
      }
      return this.slices[at].state === "in";
    },

    async load(index) {
      const slice = this.slices[index];
      slice.state = "coming";
      try {
        const body = await range(this.url, slice.offset, slice.offset + slice.length - 1);
        check(give(body.bytes, wasm.wr_load_slice), `slice ${index}`);
        slice.state = "in";
        this.loads++;
      } catch (e) {
        slice.state = "out";
        console.error(e);
        return;
      }
      // The least lately used go when more than KEEP are held.
      const held = this.slices.map((s, i) => [s, i]).filter(([s]) => s.state === "in");
      held.sort((a, b) => b[0].used - a[0].used);
      for (const [s, i] of held.slice(KEEP)) {
        wasm.wr_drop_slice(i);
        s.state = "out";
      }
      if (this.onSlice) this.onSlice(index);
    },

    /// The state at a frame: views of the reader's arrays, good until the next call into the reader. Null while the
    /// frame's slice is on its way.
    frame(frame) {
      if (!this.want(frame)) return null;
      // Asked again for the frame last worked out (the panels and the hover ask for the one just drawn), with no slice
      // read since, the reader's arrays already hold it.
      if (frame !== this.worked || this.workedLoads !== this.loads) {
        this.worked = null;
        if (!wasm.wr_frame(frame)) return null;
        this.worked = frame;
        this.workedLoads = this.loads;
      }
      return {
        units: array(0, Float32Array), ranges: array(1, Uint32Array), projectiles: array(2, Float32Array), bursts: array(3, Float32Array),
        /// Ten floats a unit with a head order or a build under way: its place in `units`, its row, the command, how
        /// it points (0 nowhere, 1 a place, 2 a unit, 3 a feature), the place, the target (a row or a feature's
        /// number, -1), the row it is building (-1), its queue's length. Eight a command given in the last second
        /// and a half: the unit's place in `units`, the command, how it points, the place, frames since, 1 for shift.
        doing: array(8, Float32Array), commands: array(9, Float32Array),
        /// Beside `units`, a unit each: the ally teams that have it in sight (bit `a` for ally team a) and, sixteen
        /// bits up, those that have it on radar.
        seen: array(11, Uint32Array),
        /// The moving pieces (none on a tape captured `--no-pieces`): beside `units`, each unit's block in `pieces` (-1: at rest), and the
        /// blocks, `pieceStride` transforms each of twelve floats (a rotation by columns, then a translation: a
        /// vertex v of the model at rest is drawn at R v + t).
        pieceAt: array(13, Float32Array), pieces: array(12, Float32Array), pieceStride: wasm.wr_piece_stride(),
      };
    },

    /// Brings the ground to a frame. Null when it has not changed since the last call; else the rectangle that did
    /// (in corners of the height map) and the whole grid's heights (u16: `low + step * value`).
    groundAt(frame) {
      if (!wasm.wr_ground(Math.max(0, Math.floor(frame)))) return null;
      return { x: wasm.wr_ground_dirty(0), z: wasm.wr_ground_dirty(1), w: wasm.wr_ground_dirty(2), h: wasm.wr_ground_dirty(3), heights: array(7, Uint16Array) };
    },

    groundNow() {
      return array(7, Uint16Array);
    },

    /// A unit's table row as an object.
    unit(row) {
      const i = this.unitInts, f = this.unitFloats, a = row * 8;
      if (row < 0 || row >= this.unitCount) return null;
      return {
        row, id: i[a], def: this.unitDefs.get(i[a + 1]), team: i[a + 2], created: i[a + 3], finished: i[a + 4], destroyed: i[a + 5],
        killer: i[a + 6] - 1, builder: i[a + 7] - 1, maxHealth: f[a], birth: [f[a + 1], f[a + 2], f[a + 3]], death: [f[a + 4], f[a + 5], f[a + 6]], mobile: f[a + 7] > 0,
      };
    },

    /// The row of the unit that held an engine id at a frame, or -1.
    rowOf(engineId, frame) {
      const i = this.unitInts;
      for (const row of this.rowsById.get(engineId) || []) if (i[row * 8 + 3] <= frame && frame < i[row * 8 + 5]) return row;
      return -1;
    },

    /// The row an engine id names nearest a frame: the unit alive then, else the last one of that id made before it.
    rowNear(engineId, frame) {
      const alive = this.rowOf(engineId, frame);
      if (alive >= 0) return alive;
      let best = -1;
      for (const row of this.rowsById.get(engineId) || []) if (this.unitInts[row * 8 + 3] <= frame) best = row;
      return best;
    },

    /// A feature's table row as an object.
    feature(index) {
      const f = this.features, a = index * 14;
      if (index < 0 || a >= f.length) return null;
      return { index, at: [f[a], f[a + 1], f[a + 2]], metal: f[a + 9], energy: f[a + 10], def: this.featureDefs.get(f[a + 11]), created: f[a + 12], gone: f[a + 13] };
    },

    /// Every order given to a unit in the slices now loaded (about a minute around the playhead), in frame order:
    /// [{f, cmd, shift, how, at, target}] with `how` and `target` as in a state's `doing`.
    ordersOf(row) {
      const n = wasm.wr_orders_of(row);
      const rows = array(10, Float32Array);
      const out = [];
      for (let i = 0; i < n; i++) {
        const a = i * HISTORY_STRIDE;
        out.push({ f: rows[a], cmd: rows[a + 1], shift: rows[a + 2] > 0, how: rows[a + 3], at: [rows[a + 4], rows[a + 5], rows[a + 6]], target: rows[a + 7] });
      }
      return out;
    },
  };

  return { open, UNIT_STRIDE, PROJECTILE_STRIDE, BURST_STRIDE, DOING_STRIDE, COMMAND_STRIDE, COMMAND_FRAMES, commandName, commandColor };
})();
