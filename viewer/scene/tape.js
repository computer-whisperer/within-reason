// The game tape in the page: the reader (crates/tape, built to scene/tape.wasm by run/build_viewer.sh) and the byte
// ranges it is fed. A tape is read as its footer, its index, its tables, then a slice at a time around the playhead
// (docs/harness/tape-format.md). One tape a page.
"use strict";

const WRTape = (() => {
  let wasm = null;
  /// Slices kept decoded: the one under the playhead, its neighbours, and a few behind for scrubbing back.
  const KEEP = 8;
  const UNIT_STRIDE = 16, PROJECTILE_STRIDE = 12, BURST_STRIDE = 8;

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

  /// Bytes `first` to `last` of a file, or with `first` null its last `last` bytes; with them the file's size.
  async function range(url, first, last) {
    const response = await fetch(url, { headers: { Range: first == null ? `bytes=-${last}` : `bytes=${first}-${last}` } });
    if (response.status !== 206) throw new Error(`${url}: the server does not answer byte ranges (status ${response.status}); the 3D view needs run/view_match.py`);
    const size = Number((response.headers.get("Content-Range") || "").split("/")[1] || 0);
    return { bytes: new Uint8Array(await response.arrayBuffer()), size };
  }

  async function open(url) {
    await reader();
    const footer = await range(url, null, 16);
    if (new TextDecoder().decode(footer.bytes.subarray(8)) !== "WRTAPEND") throw new Error(`${url} is not a finished tape`);
    const indexAt = Number(new DataView(footer.bytes.buffer).getBigUint64(0, true));
    check(give((await range(url, indexAt, footer.size - 17)).bytes, wasm.wr_open), "the tape's index");
    const chunks = [];
    for (let i = 0; i < wasm.wr_chunk_count(); i++) {
      const kind = wasm.wr_chunk(i, 0);
      chunks.push({ kind: String.fromCharCode(kind & 255, (kind >> 8) & 255, (kind >> 16) & 255, (kind >> 24) & 255), offset: wasm.wr_chunk(i, 1), length: wasm.wr_chunk(i, 2), first: wasm.wr_chunk(i, 3) });
    }
    // Everything but the slices, fetched together and handed in in file order (the unit table before any slice).
    const tables = chunks.filter((c) => c.kind !== "SLIC");
    const bodies = await Promise.all(tables.map((c) => range(url, c.offset, c.offset + c.length - 1)));
    bodies.forEach((body, i) => check(give(body.bytes, wasm.wr_set_chunk), `the tape's ${tables[i].kind} chunk`));
    const head = JSON.parse(text(0));
    const defs = JSON.parse(text(1));
    const tape = {
      url, size: footer.size, head, defs,
      teams: JSON.parse(text(2) || "{}"), events: JSON.parse(text(3) || "[]"),
      lastFrame: head.last_frame, sliceFrames: head.slice_frames,
      slices: chunks.filter((c) => c.kind === "SLIC").map((c) => ({ ...c, state: "out", used: 0 })),
      /// The unit table, a row a unit life (the row number is the tape's id of the unit): eight integers (engine id,
      /// type, team, the frames created, finished and destroyed (4294967295 never), killer row + 1, builder row + 1)
      /// and eight floats (full health, where it first stood, where it died, 1 when its type moves).
      unitInts: array(4, Uint32Array).slice(), unitFloats: array(5, Float32Array).slice(),
      /// Fourteen floats a feature: place, front, up, metal, energy, type, the frames it appeared and went (-1 never).
      features: array(6, Float32Array).slice(),
      ground: { columns: wasm.wr_ground_info(0), rows: wasm.wr_ground_info(1), low: wasm.wr_ground_info(2), step: wasm.wr_ground_info(3) },
      onSlice: null, clock: 0,
    };
    tape.unitDefs = new Map(defs.units.map((d) => [d.id, d]));
    tape.weaponDefs = new Map(defs.weapons.map((d) => [d.id, d]));
    tape.featureDefs = new Map(defs.features.map((d) => [d.id, d]));
    tape.unitCount = tape.unitInts.length / 8;
    return Object.assign(tape, methods);
  }

  const methods = {
    sliceIndex(frame) {
      return Math.min(this.slices.length - 1, Math.max(0, Math.floor(frame / this.sliceFrames)));
    },

    /// Sees that the slice under a frame is loaded, and the next one (the playhead is going there). Answers whether
    /// the frame's own slice is ready now; `onSlice` is called when one arrives.
    want(frame) {
      const at = this.sliceIndex(frame);
      for (const index of [at, at + 1]) {
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
      if (!this.want(frame) || !wasm.wr_frame(frame)) return null;
      return { units: array(0, Float32Array), ranges: array(1, Uint32Array), projectiles: array(2, Float32Array), bursts: array(3, Float32Array) };
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

    /// The row of the unit that held an engine id at a frame, or -1: the tie to the match record, which names units
    /// by the engine's id.
    rowOf(engineId, frame) {
      const i = this.unitInts;
      for (let row = 0; row < this.unitCount; row++) {
        const a = row * 8;
        if (i[a + 3] > frame) break; // in the order created
        if (i[a] === engineId && frame < i[a + 5]) return row;
      }
      return -1;
    },
  };

  return { open, UNIT_STRIDE, PROJECTILE_STRIDE, BURST_STRIDE };
})();
