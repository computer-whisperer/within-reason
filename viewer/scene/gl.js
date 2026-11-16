// What the scene's parts share: shader programs and the little matrix arithmetic a camera needs. Matrices are
// Float32Array(16), column-major as WebGL takes them.
"use strict";

const WRGL = (() => {
  function shader(gl, type, source) {
    const s = gl.createShader(type);
    gl.shaderSource(s, source);
    gl.compileShader(s);
    if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
      const log = gl.getShaderInfoLog(s);
      const numbered = source.split("\n").map((line, i) => `${i + 1}: ${line}`).join("\n");
      throw new Error(`shader: ${log}\n${numbered}`);
    }
    return s;
  }

  /// A linked program with its uniforms' locations by name (`program.u.name`).
  function program(gl, vertex, fragment) {
    const p = gl.createProgram();
    gl.attachShader(p, shader(gl, gl.VERTEX_SHADER, vertex));
    gl.attachShader(p, shader(gl, gl.FRAGMENT_SHADER, fragment));
    gl.linkProgram(p);
    if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error(`program: ${gl.getProgramInfoLog(p)}`);
    const u = {};
    for (let i = 0; i < gl.getProgramParameter(p, gl.ACTIVE_UNIFORMS); i++) {
      const name = gl.getActiveUniform(p, i).name.replace(/\[0\]$/, "");
      u[name] = gl.getUniformLocation(p, name);
    }
    return { program: p, u };
  }

  function perspective(fovY, aspect, near, far) {
    const f = 1 / Math.tan(fovY / 2), m = new Float32Array(16);
    m[0] = f / aspect; m[5] = f; m[10] = (far + near) / (near - far); m[11] = -1; m[14] = (2 * far * near) / (near - far);
    return m;
  }

  const sub = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
  const add = (a, b) => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
  const scale = (a, k) => [a[0] * k, a[1] * k, a[2] * k];
  const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
  const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
  const normalize = (a) => { const n = Math.hypot(a[0], a[1], a[2]) || 1; return [a[0] / n, a[1] / n, a[2] / n]; };

  function lookAt(eye, target, up) {
    const f = normalize(sub(target, eye)), s = normalize(cross(f, up)), u = cross(s, f);
    const m = new Float32Array(16);
    m[0] = s[0]; m[4] = s[1]; m[8] = s[2]; m[12] = -dot(s, eye);
    m[1] = u[0]; m[5] = u[1]; m[9] = u[2]; m[13] = -dot(u, eye);
    m[2] = -f[0]; m[6] = -f[1]; m[10] = -f[2]; m[14] = dot(f, eye);
    m[15] = 1;
    return m;
  }

  function multiply(a, b) {
    const m = new Float32Array(16);
    for (let c = 0; c < 4; c++) for (let r = 0; r < 4; r++) m[c * 4 + r] = a[r] * b[c * 4] + a[4 + r] * b[c * 4 + 1] + a[8 + r] * b[c * 4 + 2] + a[12 + r] * b[c * 4 + 3];
    return m;
  }

  function invert(a) {
    const m = new Float32Array(16);
    const b00 = a[0] * a[5] - a[1] * a[4], b01 = a[0] * a[6] - a[2] * a[4], b02 = a[0] * a[7] - a[3] * a[4], b03 = a[1] * a[6] - a[2] * a[5];
    const b04 = a[1] * a[7] - a[3] * a[5], b05 = a[2] * a[7] - a[3] * a[6], b06 = a[8] * a[13] - a[9] * a[12], b07 = a[8] * a[14] - a[10] * a[12];
    const b08 = a[8] * a[15] - a[11] * a[12], b09 = a[9] * a[14] - a[10] * a[13], b10 = a[9] * a[15] - a[11] * a[13], b11 = a[10] * a[15] - a[11] * a[14];
    const det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
    if (!det) return null;
    const d = 1 / det;
    m[0] = (a[5] * b11 - a[6] * b10 + a[7] * b09) * d; m[1] = (a[2] * b10 - a[1] * b11 - a[3] * b09) * d;
    m[2] = (a[13] * b05 - a[14] * b04 + a[15] * b03) * d; m[3] = (a[10] * b04 - a[9] * b05 - a[11] * b03) * d;
    m[4] = (a[6] * b08 - a[4] * b11 - a[7] * b07) * d; m[5] = (a[0] * b11 - a[2] * b08 + a[3] * b07) * d;
    m[6] = (a[14] * b02 - a[12] * b05 - a[15] * b01) * d; m[7] = (a[8] * b05 - a[10] * b02 + a[11] * b01) * d;
    m[8] = (a[4] * b10 - a[5] * b08 + a[7] * b06) * d; m[9] = (a[1] * b08 - a[0] * b10 - a[3] * b06) * d;
    m[10] = (a[12] * b04 - a[13] * b02 + a[15] * b00) * d; m[11] = (a[9] * b02 - a[8] * b04 - a[11] * b00) * d;
    m[12] = (a[5] * b07 - a[4] * b09 - a[6] * b06) * d; m[13] = (a[0] * b09 - a[1] * b07 + a[2] * b06) * d;
    m[14] = (a[13] * b01 - a[12] * b03 - a[14] * b00) * d; m[15] = (a[8] * b03 - a[9] * b01 + a[10] * b00) * d;
    return m;
  }

  /// A point through a matrix: [x, y, z, w].
  function transform(m, p) {
    return [m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12], m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13], m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14], m[3] * p[0] + m[7] * p[1] + m[11] * p[2] + m[15]];
  }

  return { program, perspective, lookAt, multiply, invert, transform, sub, add, scale, dot, cross, normalize };
})();
