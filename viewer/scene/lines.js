// Lines over the scene: orders, the commands just given, ranges, rings on the ground. Each a straight run between two
// places of the world, a set number of CSS pixels across however far away, solid or dashed, drawn over everything with
// no depth test: what they say must read through a hill, as the 2D map's lines did.
//
// Filled anew every frame (`clear`, `add` or `ring`, `draw`): twelve floats a line.
"use strict";

const WRLines = (() => {
  const VERTEX = `#version 300 es
  layout(location = 0) in vec2 aCorner;     // along the line 0..1, across it -1..1
  layout(location = 1) in vec3 iFrom;
  layout(location = 2) in vec3 iTo;
  layout(location = 3) in vec4 iColor;
  layout(location = 4) in vec2 iStyle;      // pixels across, a dash's length in pixels (0: solid)
  uniform mat4 uViewProj;
  uniform vec2 uViewport;
  uniform float uNear;
  uniform float uDpr;                       // device pixels a CSS pixel
  out vec4 vColor;
  out vec3 vAt;                             // pixels along from the start, pixels across from the middle, half the width
  out float vDash;
  void main() {
    vec4 a = uViewProj * vec4(iFrom, 1.0), b = uViewProj * vec4(iTo, 1.0);
    // An end behind the eye has no place on the screen: the line is cut where it passes the near plane.
    if (a.w < uNear && b.w < uNear) {
      gl_Position = vec4(0.0, 0.0, 2.0, 1.0);
      return;
    }
    if (a.w < uNear) a = mix(a, b, (uNear - a.w) / (b.w - a.w));
    if (b.w < uNear) b = mix(b, a, (uNear - b.w) / (a.w - b.w));
    vec2 sa = a.xy / a.w * uViewport * 0.5, sb = b.xy / b.w * uViewport * 0.5;
    float len = length(sb - sa);
    vec2 along = len > 1e-3 ? (sb - sa) / len : vec2(1.0, 0.0);
    float half_px = (iStyle.x * 0.5 + 0.75) * uDpr;  // and a soft edge
    vec2 at = mix(sa, sb, aCorner.x) + vec2(-along.y, along.x) * aCorner.y * half_px;
    gl_Position = vec4(at / (uViewport * 0.5), 0.0, 1.0);
    vColor = iColor;
    vAt = vec3(aCorner.x * len, aCorner.y * half_px, iStyle.x * 0.5 * uDpr);
    vDash = iStyle.y * uDpr;
  }`;

  const FRAGMENT = `#version 300 es
  precision highp float;
  in vec4 vColor;
  in vec3 vAt;
  in float vDash;
  out vec4 color;
  void main() {
    if (vDash > 0.0 && mod(vAt.x, vDash * 2.0) > vDash) discard;
    float edge = 1.0 - smoothstep(vAt.z - 0.75, vAt.z + 0.75, abs(vAt.y));
    color = vec4(vColor.rgb, vColor.a * edge);
  }`;

  const FLOATS = 12;

  function create(gl) {
    const program = WRGL.program(gl, VERTEX, FRAGMENT);
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([0, -1, 1, -1, 0, 1, 1, 1]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    [[1, 3, 0], [2, 3, 12], [3, 4, 24], [4, 2, 40]].forEach(([location, size, offset]) => {
      gl.enableVertexAttribArray(location);
      gl.vertexAttribPointer(location, size, gl.FLOAT, false, FLOATS * 4, offset);
      gl.vertexAttribDivisor(location, 1);
    });
    gl.bindVertexArray(null);
    let data = new Float32Array(FLOATS * 1024);
    let count = 0;

    const lines = {
      get count() { return count; },

      clear() {
        count = 0;
      },

      /// A line from `a` to `b` (places of the world), `rgba`, `width` pixels across; dashed when `dash` pixels is given.
      add(a, b, rgba, width = 1.5, dash = 0) {
        if (data.length < (count + 1) * FLOATS) {
          const bigger = new Float32Array(data.length * 2);
          bigger.set(data);
          data = bigger;
        }
        const o = count++ * FLOATS;
        data[o] = a[0]; data[o + 1] = a[1]; data[o + 2] = a[2];
        data[o + 3] = b[0]; data[o + 4] = b[1]; data[o + 5] = b[2];
        data[o + 6] = rgba[0]; data[o + 7] = rgba[1]; data[o + 8] = rgba[2]; data[o + 9] = rgba[3];
        data[o + 10] = width; data[o + 11] = dash;
      },

      /// A ring lying on the ground (and on the sea, over water): `heightAt(x, z)` says how high.
      ring(x, z, radius, heightAt, rgba, width = 1.5, dash = 0) {
        const segments = Math.round(Math.min(96, Math.max(16, radius / 10)));
        let before = null;
        for (let i = 0; i <= segments; i++) {
          const angle = (i / segments) * Math.PI * 2;
          const px = x + Math.sin(angle) * radius, pz = z + Math.cos(angle) * radius;
          const now = [px, Math.max(0, heightAt(px, pz)) + 3, pz];
          if (before) lines.add(before, now, rgba, width, dash);
          before = now;
        }
      },

      /// A level rectangle on the ground about a place, half as wide as `hx` and `hz` each way.
      box(x, z, hx, hz, heightAt, rgba, width = 1.5) {
        const y = Math.max(0, heightAt(x, z)) + 3;
        const corners = [[x - hx, y, z - hz], [x + hx, y, z - hz], [x + hx, y, z + hz], [x - hx, y, z + hz]];
        corners.forEach((c, i) => lines.add(c, corners[(i + 1) % 4], rgba, width));
      },

      draw(camera) {
        if (!count) return;
        gl.useProgram(program.program);
        gl.uniformMatrix4fv(program.u.uViewProj, false, camera.viewProj);
        gl.uniform2f(program.u.uViewport, camera.width, camera.height);
        gl.uniform1f(program.u.uNear, camera.near);
        gl.uniform1f(program.u.uDpr, camera.dpr || 1);
        gl.bindVertexArray(vao);
        gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
        gl.bufferData(gl.ARRAY_BUFFER, data.subarray(0, count * FLOATS), gl.DYNAMIC_DRAW);
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
        gl.disable(gl.DEPTH_TEST);
        gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, count);
        gl.enable(gl.DEPTH_TEST);
        gl.disable(gl.BLEND);
        gl.bindVertexArray(null);
      },
    };
    return lines;
  }

  return { create };
})();
