// The 3D view of a game's tape: the ground, the units, what flies, the orders, a camera as the game's own. The page's
// playhead says the frame (`draw(frame)`); this draws the state the tape holds for it, on the next animation frame.
//
// Over the picture, lines (lines.js): the commands as they are given, fading, as the game's spectator view shows
// them; the selected unit's order, its weapons' reach, its build reach and its sight; with their layers on, every
// unit's standing order and every static defence's reach; the metal spots. And words: a 2D canvas over the WebGL
// one (`hooks.overlay`), which the page paints through `hooks.onOverlay(painter)` with the functions that paint its
// 2D map: `painter.at(x, z, y)` is a place of the world on that canvas, `painter.line(a, b, colour, width, dash)` a
// line between two places (drawn here, in the world, so that it is cut properly where it passes behind the eye).
//
// The camera looks at a point on the ground from a distance, turned (yaw) and tilted (pitch): a drag moves the
// ground under the cursor, the wheel zooms about the cursor, a right or middle drag (or a drag with shift) turns and
// tilts, a double-click or `0` shows the whole map again.
"use strict";

const WRScene = (() => {
  const FOV = (40 * Math.PI) / 180;
  const SUN = WRGL.normalize([0.42, 0.8, 0.43]);

  async function open(canvas, url, hooks = {}) {
    const gl = canvas.getContext("webgl2", { antialias: true, alpha: false });
    if (!gl) throw new Error("this browser gives no WebGL2 context");
    const tape = await WRTape.open(url);
    const terrain = WRTerrain.create(gl, tape);
    const units = WRUnits.create(gl, tape);
    const effects = WREffects.create(gl, tape);
    const lines = WRLines.create(gl);
    const [mapX, mapZ] = terrain.mapSize;
    const heightAt = (x, z) => terrain.heightAt(x, z);
    const teamColor = new Map((tape.head.teams || []).map((t) => [t.team, t.color]));
    const allyOf = new Map((tape.head.teams || []).map((t) => [t.team, t.ally]));
    /// Row -> the unit's place in the state last drawn (-1: not alive then).
    const slots = new Int32Array(tape.unitCount);
    const scene = {
      tape, canvas, terrain,
      /// Where the camera looks (a point on the ground), how far from it, its turn and tilt in radians.
      camera: { target: [mapX / 2, 0, mapZ / 2], distance: 1, yaw: 0, pitch: 1.0 },
      /// grid: the named cells; fire: projectiles and flashes; commands: orders as they are given; orders: every
      /// unit's standing order; ranges: every static defence's reach; spots: the metal spots; icons: a unit too small
      /// on screen to read is its icon (off: its model at any distance).
      layers: { grid: true, fire: true, commands: true, orders: false, ranges: false, spots: true, icons: true },
      frame: 0, selected: -1, pending: 0, stats: {},
      /// The ally team whose sight the view shows (-1 every unit, as a spectator): what it did not see is a ghost, and
      /// of the orders only its own are drawn. Needs a tape that says what each side saw (`tape.hasSight`).
      seenBy: -1,
      /// Whether the camera has been fitted to the map yet: it cannot be while the canvas has no size (the page
      /// opens the scene with its canvas hidden), so the first frame drawn with a size does it.
      fitted: false,
    };

    function resetCamera() {
      if (!canvas.clientWidth || !canvas.clientHeight) {
        scene.fitted = false;
        return;
      }
      scene.fitted = true;
      const c = scene.camera;
      c.target = [mapX / 2, terrain.heightAt(mapX / 2, mapZ / 2), mapZ / 2];
      c.yaw = 0;
      c.pitch = (58 * Math.PI) / 180;
      // Far enough for the whole map: its height fills the view's height, or its width the view's width.
      const aspect = canvas.clientWidth / Math.max(1, canvas.clientHeight);
      c.distance = Math.max(mapZ / (2 * Math.tan(FOV / 2)), mapX / (2 * Math.tan(FOV / 2) * aspect)) * 1.05;
    }

    /// The camera's eye, matrices and what the shaders need of the view, for this frame.
    function view() {
      const c = scene.camera;
      const dpr = window.devicePixelRatio || 1;
      const width = Math.max(1, Math.round(canvas.clientWidth * dpr)), height = Math.max(1, Math.round(canvas.clientHeight * dpr));
      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
      }
      // From the target back along the look direction: yaw 0 looks north (-z), the pitch is the angle down.
      const back = [Math.sin(c.yaw) * Math.cos(c.pitch), Math.sin(c.pitch), Math.cos(c.yaw) * Math.cos(c.pitch)];
      const eye = WRGL.add(c.target, WRGL.scale(back, c.distance));
      const near = Math.max(4, c.distance * 0.02), far = c.distance * 6 + Math.hypot(mapX, mapZ);
      const viewProj = WRGL.multiply(WRGL.perspective(FOV, width / height, near, far), WRGL.lookAt(eye, c.target, [0, 1, 0]));
      return { eye, viewProj, inverse: WRGL.invert(viewProj), width, height, dpr, near, sun: SUN, pixels: height / (2 * Math.tan(FOV / 2)) };
    }

    /// What a unit type reaches: its weapons' ranges (each once, the longest first), its shield, its build reach, its
    /// sight and radar. Worked out once a type.
    const reaches = new Map();
    function reachOf(defId) {
      let reach = reaches.get(defId);
      if (reach) return reach;
      const def = tape.unitDefs.get(defId) || {};
      const weapons = new Set(), shields = new Set();
      for (const id of def.weapons || []) {
        const w = tape.weaponDefs.get(id);
        if (!w) continue;
        if (w.shield) shields.add(Math.round(w.shield_radius || 0));
        else if (w.range > 0 && (w.damage > 0 || w.paralyzer)) weapons.add(Math.round(w.range));
      }
      reach = {
        weapons: [...weapons].sort((a, b) => b - a).slice(0, 4), shields: [...shields].filter((r) => r > 0),
        build: def.builder && !def.factory ? def.build_distance || 0 : 0, sight: def.sight || 0, radar: def.radar || 0,
      };
      reaches.set(defId, reach);
      return reach;
    }

    /// Marks in each unit's flags what the side the view shows did not see: 8 not in its sight, 16 with it on its
    /// radar (a unit of that side is always seen). Cleared again when every unit is shown.
    function markSight(state) {
      const units = state.units, seen = state.seen;
      const sight = scene.seenBy >= 0 && tape.hasSight ? 1 << scene.seenBy : 0;
      for (let i = 0, j = 0; i < units.length; i += 16, j++) {
        let flags = units[i + 12] & ~24;
        if (sight && allyOf.get(units[i + 11]) !== scene.seenBy && !(seen[j] & sight)) flags |= seen[j] & (sight << 16) ? 24 : 8;
        units[i + 12] = flags;
      }
    }

    /// The frame's lines: spots, orders, commands, ranges.
    function drawLines(state) {
      const units = state.units, L = scene.layers;
      if (L.spots) for (const [x, z] of tape.head.spots || []) lines.ring(x, z, 26, heightAt, [1, 0.85, 0.35, 0.3], 1);
      const D = WRTape.DOING_STRIDE;
      // As one side saw it: its own orders alone (a player never sees the other side's).
      const theirs = (u) => scene.seenBy >= 0 && allyOf.get(units[u + 11]) !== scene.seenBy;
      for (let i = 0; i < state.doing.length; i += D) {
        const d = state.doing;
        const mine = d[i + 1] === scene.selected;
        if ((!mine && !L.orders) || d[i + 3] === 0) continue;
        const u = d[i] * 16, cmd = d[i + 2];
        if (!mine && theirs(u)) continue;
        const c = WRTape.commandColor(cmd);
        const rgba = [c[0], c[1], c[2], mine ? 0.95 : 0.4], width = mine ? 2 : 1.2;
        lines.add([units[u], units[u + 1] + 4, units[u + 2]], [d[i + 4], d[i + 5] + 4, d[i + 6]], rgba, width);
        // A build not yet begun: the footprint of what is to stand there.
        const def = cmd < 0 && d[i + 8] < 0 ? tape.unitDefs.get(-cmd) : null;
        if (def) lines.box(d[i + 4], d[i + 6], (def.xsize || 2) * 4, (def.zsize || 2) * 4, heightAt, rgba, width);
      }
      if (L.commands) {
        // As the game's spectator view draws them (BAR's gui_commands_fx): a unit's latest command, with those given
        // with it in the same frame, as one chain from the unit through each place in turn, fading; a later command
        // takes the earlier's place. A command that is not queued starts the chain again at the unit. (The game's
        // chain runs through the whole queue: one given queued behind older orders starts here at the unit.)
        const C = WRTape.COMMAND_STRIDE, given = state.commands;
        const latest = new Map();
        for (let i = 0; i < given.length; i += C) {
          if (!latest.has(given[i]) || given[i + 6] < latest.get(given[i])) latest.set(given[i], given[i + 6]);
        }
        const chainEnd = new Map();
        for (let i = 0; i < given.length; i += C) {
          const slot = given[i], u = slot * 16;
          if (given[i + 6] !== latest.get(slot) || theirs(u)) continue;
          const c = WRTape.commandColor(given[i + 1]), fade = 1 - given[i + 6] / WRTape.COMMAND_FRAMES;
          const from = (given[i + 7] > 0 && chainEnd.get(slot)) || [units[u], units[u + 1] + 4, units[u + 2]];
          const to = [given[i + 3], given[i + 4] + 4, given[i + 5]];
          lines.add(from, to, [c[0], c[1], c[2], 0.8 * fade * fade], 1.6);
          chainEnd.set(slot, to);
        }
      }
      if (L.ranges) {
        // Every finished unit that cannot move and can shoot: its longest reach, in its team's colour.
        for (let i = 0; i < units.length; i += 16) {
          if ((units[i + 12] & 6) !== 4) continue;
          const reach = reachOf(units[i + 14]).weapons[0];
          const c = teamColor.get(units[i + 11]) || [1, 1, 1];
          if (reach) lines.ring(units[i], units[i + 2], reach, heightAt, [c[0], c[1], c[2], 0.55], 1.2);
        }
      }
      const at = scene.selected >= 0 ? slots[scene.selected] : -1;
      if (at >= 0) {
        const [x, z] = [units[at * 16], units[at * 16 + 2]];
        const reach = reachOf(units[at * 16 + 14]);
        if (reach.sight) lines.ring(x, z, reach.sight, heightAt, [1, 1, 1, 0.3], 1, 3);
        if (reach.radar) lines.ring(x, z, reach.radar, heightAt, [0.4, 1, 0.6, 0.3], 1, 8);
        if (reach.build) lines.ring(x, z, reach.build, heightAt, [0.3, 1, 0.3, 0.7], 1.4, 5);
        for (const r of reach.shields) lines.ring(x, z, r, heightAt, [0.4, 0.7, 1, 0.8], 1.6);
        for (const r of reach.weapons) lines.ring(x, z, r, heightAt, [1, 0.3, 0.25, 0.85], 1.6);
      }
    }

    /// A CSS colour as the lines' four numbers.
    const cssColors = new Map();
    const probe = document.createElement("canvas").getContext("2d");
    function rgbaOf(css, alpha = 1) {
      let c = cssColors.get(css);
      if (!c) {
        probe.fillStyle = "#000";
        probe.fillStyle = css;
        const v = probe.fillStyle; // "#rrggbb" or "rgba(r, g, b, a)"
        c = v[0] === "#" ? [1, 3, 5].map((i) => parseInt(v.slice(i, i + 2), 16) / 255).concat(1) : v.match(/[\d.]+/g).map((n, i) => (i < 3 ? n / 255 : Number(n)));
        if (c.length < 4) c.push(1);
        cssColors.set(css, c);
      }
      return [c[0], c[1], c[2], c[3] * alpha];
    }

    /// The page's words over the picture: its canvas sized to this one, and the painter handed over.
    function paintOverlay(v, state) {
      const over = hooks.overlay;
      if (!over || !hooks.onOverlay) return;
      if (over.width !== v.width || over.height !== v.height) {
        over.width = v.width;
        over.height = v.height;
      }
      const ctx = over.getContext("2d");
      ctx.setTransform(v.dpr, 0, 0, v.dpr, 0, 0);
      ctx.clearRect(0, 0, canvas.clientWidth, canvas.clientHeight);
      const high = (p) => (p[2] != null ? p[2] : Math.max(0, heightAt(p[0], p[1])) + 3);
      hooks.onOverlay({
        ctx, state, width: canvas.clientWidth, height: canvas.clientHeight,
        /// A place of the world (on the ground, or `y` high) on the canvas, in its CSS pixels, with how many
        /// pixels an elmo covers there; null when it is behind the eye.
        at(x, z, y) {
          const p = WRGL.transform(v.viewProj, [x, y != null ? y : Math.max(0, heightAt(x, z)), z]);
          if (p[3] <= v.near) return null;
          return [(p[0] / p[3] * 0.5 + 0.5) * canvas.clientWidth, (0.5 - p[1] / p[3] * 0.5) * canvas.clientHeight, v.pixels / p[3] / v.dpr];
        },
        /// A line between two places ([x, z] on the ground, or [x, z, y]).
        line(a, b, css, width = 1.5, dash = null, alpha = 1) {
          lines.add([a[0], high(a), a[1]], [b[0], high(b), b[1]], rgbaOf(css, alpha), width, dash ? dash[0] : 0);
        },
        /// A ring on the ground, `radius` elmos.
        ring(x, z, radius, css, width = 1.2, dash = null, alpha = 1) {
          lines.ring(x, z, radius, heightAt, rgbaOf(css, alpha), width, dash ? dash[0] : 0);
        },
        /// Where a unit (its row) is this frame: [x, y, z], or null.
        unit(row) {
          const at = state && row >= 0 && row < slots.length ? slots[row] : -1;
          return at < 0 ? null : [state.units[at * 16], state.units[at * 16 + 1], state.units[at * 16 + 2]];
        },
      });
    }

    /// The ray under a point of the canvas (client pixels from its corner): an origin and a direction. Null when the
    /// view has no inverse (a canvas with no size).
    function ray(v, x, y) {
      if (!v.inverse) return null;
      const nx = (x / canvas.clientWidth) * 2 - 1, ny = 1 - (y / canvas.clientHeight) * 2;
      const a = WRGL.transform(v.inverse, [nx, ny, -1]), b = WRGL.transform(v.inverse, [nx, ny, 1]);
      const from = [a[0] / a[3], a[1] / a[3], a[2] / a[3]], to = [b[0] / b[3], b[1] / b[3], b[2] / b[3]];
      return { from, direction: WRGL.normalize(WRGL.sub(to, from)) };
    }

    /// Where a ray meets the ground: marched in steps, then halved in. Null when it leaves the map's air.
    function groundUnder(v, x, y) {
      const r = ray(v, x, y);
      if (!r) return null;
      const { from, direction } = r;
      const reach = scene.camera.distance * 8 + Math.hypot(mapX, mapZ);
      let before = 0;
      const above = (t) => from[1] + direction[1] * t - terrain.heightAt(from[0] + direction[0] * t, from[2] + direction[2] * t);
      for (let t = 0; t <= reach; t += Math.max(8, scene.camera.distance * 0.01)) {
        if (above(t) <= 0) {
          let low = before, high = t;
          for (let i = 0; i < 16; i++) {
            const mid = (low + high) / 2;
            if (above(mid) > 0) low = mid; else high = mid;
          }
          return WRGL.add(from, WRGL.scale(direction, high));
        }
        before = t;
      }
      return null;
    }

    /// Where a ray meets a level plane.
    function planeUnder(v, x, y, level) {
      const r = ray(v, x, y);
      if (!r) return null;
      const { from, direction } = r;
      if (Math.abs(direction[1]) < 1e-5) return null;
      const t = (level - from[1]) / direction[1];
      return t > 0 ? WRGL.add(from, WRGL.scale(direction, t)) : null;
    }

    function clampTarget() {
      const c = scene.camera;
      c.target[0] = Math.min(mapX, Math.max(0, c.target[0]));
      c.target[2] = Math.min(mapZ, Math.max(0, c.target[2]));
      c.target[1] = terrain.heightAt(c.target[0], c.target[2]);
      c.pitch = Math.min(1.553, Math.max(0.2, c.pitch));
      c.distance = Math.min(Math.hypot(mapX, mapZ) * 2.5, Math.max(60, c.distance));
    }

    function render() {
      scene.pending = 0;
      if (!canvas.clientWidth || !canvas.clientHeight) return; // hidden: drawn when it is shown (the resize observer)
      if (!scene.fitted) resetCamera();
      const started = performance.now();
      const v = view();
      const change = tape.groundAt(scene.frame);
      if (change) terrain.update(change);
      const state = tape.frame(scene.frame);
      gl.viewport(0, 0, v.width, v.height);
      gl.clearColor(0.05, 0.07, 0.1, 1);
      gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
      gl.enable(gl.DEPTH_TEST);
      gl.disable(gl.CULL_FACE);
      scene.stats.triangles = terrain.draw(v, scene.layers);
      scene.stats.features = units.drawFeatures(scene.frame, v);
      let selected = null;
      if (state) markSight(state);
      if (state) {
        slots.fill(-1);
        for (let i = 0; i < state.units.length; i += 16) slots[state.units[i + 13]] = i / 16;
        scene.stats.units = state.units.length / 16;
        scene.stats.projectiles = state.projectiles.length / 12;
        // The selected unit's place this frame, for its ring.
        const at = scene.selected >= 0 ? slots[scene.selected] * 16 : -1;
        if (at >= 0) selected = { x: state.units[at], y: state.units[at + 1], z: state.units[at + 2], radius: units.radiusOf(state.units[at + 14]) };
        scene.stats.draws = units.draw(state, v, selected, scene.layers.icons);
      }
      terrain.drawWater(v);
      if (state && scene.layers.fire) effects.draw(state, v);
      lines.clear();
      if (state) drawLines(state);
      paintOverlay(v, state);
      scene.stats.lines = lines.count;
      lines.draw(v);
      scene.stats.loading = !state;
      scene.stats.ms = performance.now() - started;
      scene.last = v;
      if (hover && !drag && hooks.onHover) hooks.onHover(pick(hover.x, hover.y), groundUnder(v, hover.x, hover.y), hover.e);
      if (hooks.onDraw) hooks.onDraw(scene);
    }

    function redraw() {
      if (!scene.pending) scene.pending = requestAnimationFrame(render);
    }

    /// The unit nearest a point of the canvas, as its row, or -1: within its drawn size or a dozen pixels.
    function pick(x, y) {
      const state = tape.frame(scene.frame);
      const v = scene.last || view();
      if (!state) return -1;
      let best = -1, bestDistance = Infinity;
      for (let i = 0; i < state.units.length; i += 16) {
        const radius = units.radiusOf(state.units[i + 14]);
        const p = WRGL.transform(v.viewProj, [state.units[i], state.units[i + 1] + radius * 0.5, state.units[i + 2]]);
        if (p[3] <= 0) continue;
        const sx = (p[0] / p[3] * 0.5 + 0.5) * canvas.clientWidth, sy = (0.5 - p[1] / p[3] * 0.5) * canvas.clientHeight;
        const size = Math.max(12, (radius * v.pixels) / p[3] / v.dpr);
        const d = Math.hypot(sx - x, sy - y);
        if (d <= size && d < bestDistance) {
          bestDistance = d;
          best = state.units[i + 13];
        }
      }
      return best;
    }

    // ------------------------------------------------------------------------------------------------ the mouse
    let drag = null;
    /// Where the mouse rests over the canvas: what is under it is told again after each picture, since the game and
    /// the camera move under a still mouse.
    let hover = null;
    const at = (e) => {
      const box = canvas.getBoundingClientRect();
      return [e.clientX - box.left, e.clientY - box.top];
    };
    canvas.addEventListener("contextmenu", (e) => e.preventDefault());
    canvas.addEventListener("mousedown", (e) => {
      const [x, y] = at(e);
      const turning = e.button !== 0 || e.shiftKey;
      const v = view();
      // A pan holds the ground point under the cursor: the plane through it, level.
      const hold = turning ? null : groundUnder(v, x, y) || planeUnder(v, x, y, scene.camera.target[1]);
      drag = { x, y, turning, hold, moved: false, yaw: scene.camera.yaw, pitch: scene.camera.pitch };
      e.preventDefault();
    });
    window.addEventListener("mousemove", (e) => {
      if (!drag) return;
      const [x, y] = at(e);
      if (!drag.moved && Math.hypot(x - drag.x, y - drag.y) < 3) return;
      drag.moved = true;
      const c = scene.camera;
      if (drag.turning) {
        c.yaw = drag.yaw - (x - drag.x) * 0.006;
        c.pitch = drag.pitch + (y - drag.y) * 0.005;
      } else if (drag.hold) {
        const now = planeUnder(view(), x, y, drag.hold[1]);
        if (now) {
          c.target[0] += drag.hold[0] - now[0];
          c.target[2] += drag.hold[2] - now[2];
        }
      }
      clampTarget();
      redraw();
    });
    window.addEventListener("mouseup", (e) => {
      if (!drag) return;
      const clicked = !drag.moved && !drag.turning;
      drag = null;
      if (clicked && hooks.onPick) {
        const [x, y] = at(e);
        hooks.onPick(pick(x, y));
      }
    });
    canvas.addEventListener("wheel", (e) => {
      e.preventDefault();
      const [x, y] = at(e);
      const c = scene.camera;
      const before = groundUnder(view(), x, y);
      c.distance *= Math.exp(e.deltaY * 0.0012);
      clampTarget();
      // The ground under the cursor stays under it.
      if (before) {
        const after = planeUnder(view(), x, y, before[1]);
        if (after) {
          c.target[0] += before[0] - after[0];
          c.target[2] += before[2] - after[2];
          clampTarget();
        }
      }
      redraw();
    }, { passive: false });
    canvas.addEventListener("dblclick", (e) => {
      e.preventDefault();
      resetCamera();
      redraw();
    });
    canvas.addEventListener("mousemove", (e) => {
      if (drag || !hooks.onHover) return;
      const [x, y] = at(e);
      hover = { x, y, e };
      hooks.onHover(pick(x, y), groundUnder(scene.last || view(), x, y), e);
    });
    canvas.addEventListener("mouseleave", () => {
      hover = null;
      if (hooks.onHover) hooks.onHover(-1, null, null);
    });

    // A slice arrived: the picture, and whatever the page shows of the tape (`hooks.onSlice`), in either view.
    tape.onSlice = () => {
      redraw();
      if (hooks.onSlice) hooks.onSlice();
    };
    terrain.onChange = redraw;
    units.onChange = redraw;
    new ResizeObserver(redraw).observe(canvas);
    resetCamera();

    return Object.assign(scene, {
      /// Shows the state at a frame.
      draw(frame) {
        scene.frame = Math.min(tape.lastFrame, Math.max(0, frame));
        redraw();
      },
      redraw, resetCamera, pick,
      /// The tape grew (one still being written, `tape.refresh`): what is drawn from its tables, again.
      grew() {
        units.tapeGrew();
        redraw();
      },
      /// Zooms by a factor about the view's centre.
      zoom(factor) {
        scene.camera.distance /= factor;
        clampTarget();
        redraw();
      },
      /// Marks a unit (its row) as selected, or none (-1).
      select(row) {
        scene.selected = row;
        redraw();
      },
      /// Brings the camera over a place, as near as `distance` when given.
      lookAt(x, z, distance) {
        scene.camera.target = [x, 0, z];
        if (distance) scene.camera.distance = distance;
        clampTarget();
        redraw();
      },
      /// The unit of a row at a frame (the one shown when not given): its floats, or null when it is not alive then
      /// or the tape is still being read there.
      unitNow(row, frame = scene.frame) {
        const state = tape.frame(frame);
        if (!state) return null;
        for (let i = 0; i < state.units.length; i += 16) if (state.units[i + 13] === row) return Array.from(state.units.subarray(i, i + 16));
        return null;
      },
      /// What the unit of a row is doing at a frame (the one shown when not given): {cmd, how, at, target, builds,
      /// queue} (a state's `doing`, tape.js), or null when it has no order and builds nothing.
      doingOf(row, frame = scene.frame) {
        const state = tape.frame(frame);
        const d = state ? state.doing : [];
        for (let i = 0; i < d.length; i += WRTape.DOING_STRIDE) {
          if (d[i + 1] === row) return { cmd: d[i + 2], how: d[i + 3], at: [d[i + 4], d[i + 5], d[i + 6]], target: d[i + 7], builds: d[i + 8], queue: d[i + 9] };
        }
        return null;
      },
      reachOf,
    });
  }

  return { open };
})();
