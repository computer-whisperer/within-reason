// The 3D view of a game's tape: the ground, the units, a camera as the game's own. The page's playhead says the
// frame (`draw(frame)`); this draws the state the tape holds for it, on the next animation frame.
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
    const [mapX, mapZ] = terrain.mapSize;
    const scene = {
      tape, canvas, terrain,
      /// Where the camera looks (a point on the ground), how far from it, its turn and tilt in radians.
      camera: { target: [mapX / 2, 0, mapZ / 2], distance: 1, yaw: 0, pitch: 1.0 },
      layers: { grid: true, fire: true },
      frame: 0, selected: -1, pending: 0, stats: {}, state: null,
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
      return { eye, viewProj, inverse: WRGL.invert(viewProj), width, height, dpr, sun: SUN, pixels: height / (2 * Math.tan(FOV / 2)) };
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
      if (state) {
        // The selected unit's place this frame, for its ring (and for whoever asks where it is).
        scene.stats.units = state.units.length / 16;
        scene.stats.projectiles = state.projectiles.length / 12;
        if (scene.selected >= 0) {
          for (let i = 0; i < state.units.length; i += 16) {
            if (state.units[i + 13] === scene.selected) {
              selected = { x: state.units[i], y: state.units[i + 1], z: state.units[i + 2], radius: units.radiusOf(state.units[i + 14]) };
              break;
            }
          }
        }
        scene.stats.draws = units.draw(state, v, selected);
      }
      terrain.drawWater(v);
      if (state && scene.layers.fire) effects.draw(state, v);
      scene.stats.loading = !state;
      scene.stats.ms = performance.now() - started;
      scene.last = v;
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
      hooks.onHover(pick(x, y), groundUnder(scene.last || view(), x, y), e);
    });
    canvas.addEventListener("mouseleave", () => hooks.onHover && hooks.onHover(-1, null, null));

    tape.onSlice = redraw;
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
      /// The unit of a row in the state last drawn: its floats, or null when it is not alive then.
      unitNow(row) {
        const state = tape.frame(scene.frame);
        if (!state) return null;
        for (let i = 0; i < state.units.length; i += 16) if (state.units[i + 13] === row) return Array.from(state.units.subarray(i, i + 16));
        return null;
      },
    });
  }

  return { open };
})();
