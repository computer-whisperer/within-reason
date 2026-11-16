// What flies and what bursts: every projectile in flight at the frame as a streak (a beam as a line to what it was
// aimed at), each in its weapon's own colour and a width that never falls under a couple of pixels, so that who is
// firing at whom reads from any distance; and a flash where a projectile ended or a unit died, sized by the weapon's
// blast or the unit. Not the game's particle effects: their information.
//
// The reader's arrays (tape.js): a projectile is twelve floats (where it is, where its tail or a beam's far end is,
// weapon type, path, frames since it began, frames it lives, owner row, team), a burst eight (place, frames since,
// weapon or unit type, kind: 2 a projectile's end, 3 a unit's death; the owner's or the dead unit's row + 1).
"use strict";

const WREffects = (() => {
  const STREAK_VERTEX = `#version 300 es
  layout(location = 0) in vec2 aCorner;     // along the streak 0..1, across it -1..1
  layout(location = 1) in vec3 iHead;
  layout(location = 2) in vec3 iTail;
  layout(location = 3) in vec4 iColor;      // rgb, and how much the core whitens
  layout(location = 4) in vec2 iWidth;      // elmos across, and the least pixels across
  uniform mat4 uViewProj;
  uniform vec2 uViewport;
  uniform float uPixels;
  out vec2 vAt;
  out vec4 vColor;
  void main() {
    vec4 a = uViewProj * vec4(iTail, 1.0), b = uViewProj * vec4(iHead, 1.0);
    vec4 here = mix(a, b, aCorner.x);
    // Across the streak on the screen: square to its direction there.
    vec2 along = (b.xy / max(b.w, 1e-3) - a.xy / max(a.w, 1e-3)) * uViewport;
    float len = length(along);
    vec2 across = len > 1e-3 ? vec2(-along.y, along.x) / len : vec2(0.0, 1.0);
    float half_px = max(iWidth.x * uPixels / max(here.w, 1.0), iWidth.y) * 0.5;
    here.xy += across * aCorner.y * half_px * 2.0 / uViewport * here.w;
    // A streak seen end-on is a dot: widen it along itself by its own half width.
    here.xy += (len > 1e-3 ? along / len : vec2(1.0, 0.0)) * (aCorner.x * 2.0 - 1.0) * half_px * 2.0 / uViewport * here.w;
    gl_Position = here;
    vAt = aCorner;
    vColor = iColor;
  }`;

  const STREAK_FRAGMENT = `#version 300 es
  precision highp float;
  in vec2 vAt;
  in vec4 vColor;
  out vec4 color;
  void main() {
    float across = 1.0 - abs(vAt.y);
    float fade = 0.35 + 0.65 * vAt.x;                       // brighter at the head
    vec3 c = mix(vColor.rgb, vec3(1.0), vColor.a * smoothstep(0.45, 1.0, across));
    color = vec4(c * fade * smoothstep(0.0, 0.5, across), 1.0);
  }`;

  const BURST_VERTEX = `#version 300 es
  layout(location = 0) in vec2 aCorner;     // -1..1
  layout(location = 1) in vec3 iAt;
  layout(location = 2) in vec4 iColor;      // rgb and strength
  layout(location = 3) in vec2 iSize;       // elmos across now, and the least pixels across
  uniform mat4 uViewProj;
  uniform vec2 uViewport;
  uniform float uPixels;
  out vec2 vAt;
  out vec4 vColor;
  void main() {
    vec4 centre = uViewProj * vec4(iAt, 1.0);
    float half_px = max(iSize.x * uPixels / max(centre.w, 1.0), iSize.y) * 0.5;
    gl_Position = vec4(centre.xy + aCorner * half_px * 2.0 / uViewport * centre.w, centre.zw);
    vAt = aCorner;
    vColor = iColor;
  }`;

  const BURST_FRAGMENT = `#version 300 es
  precision highp float;
  in vec2 vAt;
  in vec4 vColor;
  out vec4 color;
  void main() {
    float r = length(vAt);
    if (r > 1.0) discard;
    float glow = (1.0 - r) * (1.0 - r);
    color = vec4(mix(vColor.rgb, vec3(1.0), glow * 0.7) * glow * vColor.a, 1.0);
  }`;

  /// How a weapon type's shots are drawn: colour, width in elmos, least width in pixels, how long a streak (frames
  /// of flight behind the head), how much the core whitens.
  function look(def) {
    const kind = def ? def.kind : "piece";
    const rgb = def && def.color ? def.color.map((v) => Math.min(1, Math.max(0.15, v))) : [1, 0.8, 0.4];
    const blast = def ? def.aoe || 0 : 0;
    switch (kind) {
      case "BeamLaser": case "LightningCannon":
        return { rgb, width: Math.max(2, (def.thickness || 2) * 1.2), least: 2, tail: 0, core: 0.8 };
      case "LaserCannon":
        return { rgb, width: Math.max(2, (def.thickness || 2) * 1.2), least: 2, tail: 2.5, core: 0.7 };
      case "Cannon":
        return { rgb: [1, 0.72, 0.3], width: Math.min(14, 3 + blast / 24), least: 2.5, tail: 1.2, core: 0.6 };
      case "MissileLauncher": case "StarburstLauncher": case "TorpedoLauncher":
        return { rgb: [1, 0.86, 0.62], width: Math.min(16, 4 + blast / 40), least: 2.5, tail: 5, core: 0.5 };
      case "AircraftBomb":
        return { rgb: [0.95, 0.6, 0.3], width: Math.min(14, 5 + blast / 40), least: 3, tail: 1.5, core: 0.3 };
      case "Flame":
        return { rgb: [1, 0.5, 0.12], width: 14, least: 3, tail: 1.5, core: 0.4 };
      case "DGun":
        return { rgb: [1, 0.55, 0.15], width: 26, least: 5, tail: 2, core: 0.8 };
      case "piece":
        return { rgb: [0.45, 0.4, 0.35], width: 3, least: 1.5, tail: 1, core: 0 };
      default:
        return { rgb, width: 4, least: 2, tail: 1.5, core: 0.5 };
    }
  }

  function create(gl, tape) {
    const streak = WRGL.program(gl, STREAK_VERTEX, STREAK_FRAGMENT);
    const burst = WRGL.program(gl, BURST_VERTEX, BURST_FRAGMENT);
    const looks = new Map();
    const lookOf = (wdef) => {
      let l = looks.get(wdef);
      if (!l) {
        l = look(wdef >= 0 ? tape.weaponDefs.get(wdef) : null);
        looks.set(wdef, l);
      }
      return l;
    };

    const strip = (corners, layout) => {
      const vao = gl.createVertexArray();
      gl.bindVertexArray(vao);
      gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(corners), gl.STATIC_DRAW);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
      const buffer = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
      const stride = layout.reduce((n, size) => n + size, 0) * 4;
      let offset = 0;
      layout.forEach((size, i) => {
        gl.enableVertexAttribArray(i + 1);
        gl.vertexAttribPointer(i + 1, size, gl.FLOAT, false, stride, offset);
        gl.vertexAttribDivisor(i + 1, 1);
        offset += size * 4;
      });
      gl.bindVertexArray(null);
      return { vao, buffer, floats: stride / 4, data: new Float32Array(1024) };
    };
    const streaks = strip([0, -1, 1, -1, 0, 1, 1, 1], [3, 3, 4, 2]);
    const bursts = strip([-1, -1, 1, -1, -1, 1, 1, 1], [3, 4, 2]);
    const room = (set, count) => {
      if (set.data.length < count * set.floats) set.data = new Float32Array(count * set.floats * 2);
      return set.data;
    };
    const upload = (set, count) => {
      gl.bindBuffer(gl.ARRAY_BUFFER, set.buffer);
      gl.bufferData(gl.ARRAY_BUFFER, set.data.subarray(0, count * set.floats), gl.DYNAMIC_DRAW);
    };

    return {
      /// Draws the projectiles and bursts of a state (`tape.frame`'s answer), over the solid world, adding light.
      draw(state, camera) {
        const projectiles = state.projectiles, count = projectiles.length / 12;
        const flashes = state.bursts, flashCount = flashes.length / 8;
        if (!count && !flashCount) return;
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.ONE, gl.ONE);
        gl.depthMask(false);

        if (count) {
          const out = room(streaks, count);
          for (let i = 0; i < count; i++) {
            const p = i * 12, o = i * 12;
            const l = lookOf(projectiles[p + 6]);
            let [tx, ty, tz] = [projectiles[p + 3], projectiles[p + 4], projectiles[p + 5]];
            if (projectiles[p + 7] !== 1) {
              // Not a beam: the tail is where it was a frame ago; drawn `tail` frames long.
              tx = projectiles[p] + (tx - projectiles[p]) * l.tail;
              ty = projectiles[p + 1] + (ty - projectiles[p + 1]) * l.tail;
              tz = projectiles[p + 2] + (tz - projectiles[p + 2]) * l.tail;
            }
            out[o] = projectiles[p]; out[o + 1] = projectiles[p + 1]; out[o + 2] = projectiles[p + 2];
            out[o + 3] = tx; out[o + 4] = ty; out[o + 5] = tz;
            out[o + 6] = l.rgb[0]; out[o + 7] = l.rgb[1]; out[o + 8] = l.rgb[2]; out[o + 9] = l.core;
            out[o + 10] = l.width; out[o + 11] = l.least * camera.dpr;
          }
          upload(streaks, count);
          gl.useProgram(streak.program);
          gl.uniformMatrix4fv(streak.u.uViewProj, false, camera.viewProj);
          gl.uniform2f(streak.u.uViewport, camera.width, camera.height);
          gl.uniform1f(streak.u.uPixels, camera.pixels);
          gl.bindVertexArray(streaks.vao);
          gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, count);
        }

        if (flashCount) {
          const out = room(bursts, flashCount);
          let n = 0;
          for (let i = 0; i < flashCount; i++) {
            const b = i * 8;
            const age = flashes[b + 3], kind = flashes[b + 5];
            let size, life, rgb;
            if (kind === 3) {
              // A unit's death: by its size, and its death blast when the type has one.
              const def = tape.unitDefs.get(flashes[b + 4]);
              const blast = def && def.death >= 0 ? (tape.weaponDefs.get(def.death) || {}).aoe || 0 : 0;
              size = Math.max(((def && def.radius) || 16) * 2.2, Math.min(blast, 700));
              life = Math.min(55, 18 + size / 12);
              rgb = [1, 0.62, 0.25];
            } else {
              const def = tape.weaponDefs.get(flashes[b + 4]);
              if (def && (def.kind === "BeamLaser" || def.kind === "LightningCannon")) continue; // a beam's end is its line
              const l = lookOf(flashes[b + 4]);
              size = Math.max(10, Math.min((def && def.aoe) || 0, 900) * 1.1);
              life = Math.min(50, 8 + size / 14);
              rgb = l.rgb;
            }
            if (age >= life) continue;
            const t = age / life;
            const o = n * 9;
            out[o] = flashes[b]; out[o + 1] = flashes[b + 1] + 4; out[o + 2] = flashes[b + 2];
            out[o + 3] = rgb[0]; out[o + 4] = rgb[1]; out[o + 5] = rgb[2]; out[o + 6] = (1 - t) * (1 - t) * 1.3;
            out[o + 7] = size * (0.45 + 0.55 * Math.sqrt(t)); out[o + 8] = (kind === 3 ? 7 : 4) * camera.dpr * (1 - t);
            n++;
          }
          if (n) {
            upload(bursts, n);
            gl.useProgram(burst.program);
            gl.uniformMatrix4fv(burst.u.uViewProj, false, camera.viewProj);
            gl.uniform2f(burst.u.uViewport, camera.width, camera.height);
            gl.uniform1f(burst.u.uPixels, camera.pixels);
            gl.bindVertexArray(bursts.vao);
            gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, n);
          }
        }
        gl.bindVertexArray(null);
        gl.depthMask(true);
        gl.disable(gl.BLEND);
      },
    };
  }

  return { create };
})();
