// The ground. Its heights live in a texture the vertex shader reads (the tape's own u16 a corner of an 8-elmo
// square), so a crater is a texture update and no mesh changes. The map is cut into patches of 64 squares; a patch is
// drawn with fewer triangles the further it is (five meshes, 64 squares a side down to 4), with a skirt hanging from
// its rim to hide the seam against a neighbour drawn finer or coarser. Its colour is the map's own texture (the SMT's
// tiles laid into one image, assets.js), its shading from a slope texture worked out here from the heights.
"use strict";

const WRTerrain = (() => {
  const PATCH = 64;
  const LEVELS = 5;

  const VERTEX = `#version 300 es
  layout(location = 0) in vec3 aGrid;   // column and row within the patch in this level's steps; 1 for a skirt's lower rim
  layout(location = 1) in vec2 aPatch;  // the patch's first corner
  uniform mat4 uViewProj;
  uniform float uStep;                  // corners a step of this level
  uniform highp usampler2D uHeight;
  uniform vec2 uCorners;                // the last corner: columns - 1, rows - 1
  uniform vec2 uScale;                  // a height is uScale.x + uScale.y * value
  out vec2 vUV;
  out vec3 vWorld;
  void main() {
    vec2 corner = min(aPatch + aGrid.xy * uStep, uCorners);
    float h = uScale.x + uScale.y * float(texelFetch(uHeight, ivec2(corner), 0).r);
    vWorld = vec3(corner.x * 8.0, h, corner.y * 8.0);
    vUV = corner / uCorners;
    gl_Position = uViewProj * vec4(vWorld.x, h - aGrid.z * uStep * 10.0, vWorld.z, 1.0);
  }`;

  const FRAGMENT = `#version 300 es
  precision highp float;
  uniform sampler2D uDiffuse;
  uniform sampler2D uSlope;
  uniform vec3 uSun;
  uniform float uGrid;       // elmos a named cell (1024), 0 for no grid
  in vec2 vUV;
  in vec3 vWorld;
  out vec4 color;
  void main() {
    vec2 s = texture(uSlope, vUV).rg * 2.0 - 1.0;
    vec3 n = vec3(s.x, sqrt(max(0.0, 1.0 - dot(s, s))), s.y);
    vec3 c = texture(uDiffuse, vUV).rgb * (0.42 + 0.85 * max(dot(n, uSun), 0.0));
    if (vWorld.y < 0.0) {
      // under water: bluer and darker with depth
      float depth = clamp(-vWorld.y / 100.0, 0.0, 1.0);
      c = mix(c, vec3(0.04, 0.15, 0.25), 0.3 + 0.55 * depth);
    }
    if (uGrid > 0.0) {
      // the named cells' lines, a constant width on screen
      vec2 cell = vWorld.xz / uGrid;
      vec2 line = abs(fract(cell - 0.5) - 0.5) / fwidth(cell);
      c = mix(c, vec3(1.0), 0.22 * (1.0 - min(min(line.x, line.y), 1.0)));
    }
    color = vec4(c, 1.0);
  }`;

  const WATER_VERTEX = `#version 300 es
  layout(location = 0) in vec2 aCorner;
  uniform mat4 uViewProj;
  uniform vec2 uSize;
  out vec3 vWorld;
  void main() {
    vWorld = vec3(aCorner.x * uSize.x, 0.0, aCorner.y * uSize.y);
    gl_Position = uViewProj * vec4(vWorld, 1.0);
  }`;

  const WATER_FRAGMENT = `#version 300 es
  precision highp float;
  uniform vec3 uEye;
  in vec3 vWorld;
  out vec4 color;
  void main() {
    // more of the sky's colour, and less seen through, the flatter the look
    float glance = 1.0 - abs(normalize(uEye - vWorld).y);
    color = vec4(mix(vec3(0.07, 0.24, 0.36), vec3(0.45, 0.62, 0.72), glance * glance), 0.38 + 0.4 * glance);
  }`;

  /// A patch's mesh at a level: a grid of (64 >> level) steps a side and a skirt round it.
  function mesh(level) {
    const n = PATCH >> level;
    const vertices = [], indices = [];
    for (let z = 0; z <= n; z++) for (let x = 0; x <= n; x++) vertices.push(x, z, 0);
    const at = (x, z) => z * (n + 1) + x;
    for (let z = 0; z < n; z++) {
      for (let x = 0; x < n; x++) indices.push(at(x, z), at(x, z + 1), at(x + 1, z), at(x + 1, z), at(x, z + 1), at(x + 1, z + 1));
    }
    // The skirt: each rim corner again, lowered in the shader, joined to the rim.
    const rim = [];
    for (let x = 0; x <= n; x++) rim.push([x, 0]);
    for (let z = 1; z <= n; z++) rim.push([n, z]);
    for (let x = n - 1; x >= 0; x--) rim.push([x, n]);
    for (let z = n - 1; z >= 1; z--) rim.push([0, z]);
    const first = vertices.length / 3;
    for (const [x, z] of rim) vertices.push(x, z, 1);
    for (let i = 0; i < rim.length; i++) {
      const j = (i + 1) % rim.length;
      const a = at(rim[i][0], rim[i][1]), b = at(rim[j][0], rim[j][1]);
      indices.push(a, b, first + i, b, first + j, first + i);
    }
    return { vertices: new Float32Array(vertices), indices: new Uint16Array(indices) };
  }

  /// The slope texture's bytes for a rectangle of corners: the ground normal's x and z at each, from its neighbours.
  function slopes(heights, columns, rows, step, x0, z0, w, h) {
    const out = new Uint8Array(w * h * 2);
    const at = (x, z) => heights[Math.min(rows - 1, Math.max(0, z)) * columns + Math.min(columns - 1, Math.max(0, x))] * step;
    for (let z = 0; z < h; z++) {
      for (let x = 0; x < w; x++) {
        const cx = x0 + x, cz = z0 + z;
        const dx = (at(cx + 1, cz) - at(cx - 1, cz)) / 16, dz = (at(cx, cz + 1) - at(cx, cz - 1)) / 16;
        const n = 1 / Math.hypot(dx, 1, dz);
        out[(z * w + x) * 2] = (-dx * n * 0.5 + 0.5) * 255;
        out[(z * w + x) * 2 + 1] = (-dz * n * 0.5 + 0.5) * 255;
      }
    }
    return out;
  }

  function create(gl, tape) {
    const { columns, rows, low, step } = tape.ground;
    const program = WRGL.program(gl, VERTEX, FRAGMENT);
    const water = WRGL.program(gl, WATER_VERTEX, WATER_FRAGMENT);
    const mapSize = [(columns - 1) * 8, (rows - 1) * 8];
    /// The heights as the page keeps them, for the camera and the cursor: brought up to date with the texture.
    const heights = new Uint16Array(tape.groundNow());

    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    const heightTexture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, heightTexture);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R16UI, columns, rows, 0, gl.RED_INTEGER, gl.UNSIGNED_SHORT, heights);
    for (const [name, value] of [[gl.TEXTURE_MIN_FILTER, gl.NEAREST], [gl.TEXTURE_MAG_FILTER, gl.NEAREST], [gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE], [gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE]]) gl.texParameteri(gl.TEXTURE_2D, name, value);

    const slopeTexture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, slopeTexture);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RG8, columns, rows, 0, gl.RG, gl.UNSIGNED_BYTE, slopes(heights, columns, rows, step, 0, 0, columns, rows));
    gl.generateMipmap(gl.TEXTURE_2D);
    for (const [name, value] of [[gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR], [gl.TEXTURE_MAG_FILTER, gl.LINEAR], [gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE], [gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE]]) gl.texParameteri(gl.TEXTURE_2D, name, value);

    // Until the map's own texture is here, and for a map whose files the cache lacks: a plain earth.
    let diffuse = WRAssets.plainTexture(gl, [118, 112, 96, 255]);
    let diffuseNote = "plain";

    const levels = [];
    const patchesX = Math.ceil((columns - 1) / PATCH), patchesZ = Math.ceil((rows - 1) / PATCH);
    for (let level = 0; level < LEVELS; level++) {
      const m = mesh(level);
      const vao = gl.createVertexArray();
      gl.bindVertexArray(vao);
      gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
      gl.bufferData(gl.ARRAY_BUFFER, m.vertices, gl.STATIC_DRAW);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
      const instances = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, instances);
      gl.bufferData(gl.ARRAY_BUFFER, patchesX * patchesZ * 8, gl.DYNAMIC_DRAW);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 0, 0);
      gl.vertexAttribDivisor(1, 1);
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, gl.createBuffer());
      gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, m.indices, gl.STATIC_DRAW);
      levels.push({ vao, instances, count: m.indices.length, patches: new Float32Array(patchesX * patchesZ * 2) });
    }
    gl.bindVertexArray(null);

    const waterVao = gl.createVertexArray();
    gl.bindVertexArray(waterVao);
    gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([0, 0, 0, 1, 1, 0, 1, 1]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);
    let lowest = 0;
    for (let i = 0; i < heights.length; i++) if (heights[i] < heights[lowest]) lowest = i;
    const hasWater = low + step * heights[lowest] < 0;

    // The map's own texture, when the cache holds its files.
    (async () => {
      const assets = tape.head.assets || {};
      const smfPath = (tape.head.map_files || []).find((p) => p.endsWith(".smf"));
      if (!smfPath || !assets[smfPath]) return;
      const smf = WRAssets.parseSMF(await WRAssets.fetchBytes(assets[smfPath]));
      const names = smf.files.map((f) => assets[`maps/${f.name.toLowerCase()}`]);
      if (names.some((n) => !n)) return;
      const tiles = await Promise.all(names.map(WRAssets.fetchBytes));
      // A texel an elmo where the GPU takes the image whole (a 32 by 32 map's is 16384 texels a side, 134 MB packed).
      const made = WRAssets.uploadMap(gl, smf, tiles, Math.min(16384, gl.getParameter(gl.MAX_TEXTURE_SIZE)));
      diffuse = made.texture;
      diffuseNote = `${1 << made.level} elmos a texel`;
      terrain.onChange && terrain.onChange();
    })().catch((e) => console.error("the map's texture:", e));

    const terrain = {
      mapSize, onChange: null,
      note: () => diffuseNote,

      /// The ground's height at a place, between the four corners round it.
      heightAt(x, z) {
        const gx = Math.min(columns - 1.001, Math.max(0, x / 8)), gz = Math.min(rows - 1.001, Math.max(0, z / 8));
        const ix = Math.floor(gx), iz = Math.floor(gz), fx = gx - ix, fz = gz - iz;
        const at = (cx, cz) => heights[cz * columns + cx];
        const top = at(ix, iz) * (1 - fx) + at(ix + 1, iz) * fx, bottom = at(ix, iz + 1) * (1 - fx) + at(ix + 1, iz + 1) * fx;
        return low + step * (top * (1 - fz) + bottom * fz);
      },

      /// Takes a change of the ground: `tape.groundAt`'s answer.
      update(change) {
        heights.set(change.heights);
        const { x, z, w, h } = change;
        const part = new Uint16Array(w * h);
        for (let row = 0; row < h; row++) part.set(heights.subarray((z + row) * columns + x, (z + row) * columns + x + w), row * w);
        gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
        gl.bindTexture(gl.TEXTURE_2D, heightTexture);
        gl.texSubImage2D(gl.TEXTURE_2D, 0, x, z, w, h, gl.RED_INTEGER, gl.UNSIGNED_SHORT, part);
        // The slopes of the rectangle and of the ring round it, whose neighbours changed.
        const x0 = Math.max(0, x - 1), z0 = Math.max(0, z - 1), x1 = Math.min(columns, x + w + 1), z1 = Math.min(rows, z + h + 1);
        gl.bindTexture(gl.TEXTURE_2D, slopeTexture);
        gl.texSubImage2D(gl.TEXTURE_2D, 0, x0, z0, x1 - x0, z1 - z0, gl.RG, gl.UNSIGNED_BYTE, slopes(heights, columns, rows, step, x0, z0, x1 - x0, z1 - z0));
        gl.generateMipmap(gl.TEXTURE_2D);
      },

      draw(camera, layers) {
        // Each patch to the level its distance from the eye asks for.
        const counts = new Array(LEVELS).fill(0);
        for (let pz = 0; pz < patchesZ; pz++) {
          for (let px = 0; px < patchesX; px++) {
            const cx = (px + 0.5) * PATCH * 8, cz = (pz + 0.5) * PATCH * 8;
            const distance = Math.hypot(cx - camera.eye[0], camera.eye[1], cz - camera.eye[2]);
            const level = Math.min(LEVELS - 1, Math.max(0, Math.floor(Math.log2(distance / 2600))));
            const list = levels[level].patches;
            list[counts[level] * 2] = px * PATCH;
            list[counts[level] * 2 + 1] = pz * PATCH;
            counts[level]++;
          }
        }
        gl.useProgram(program.program);
        gl.uniformMatrix4fv(program.u.uViewProj, false, camera.viewProj);
        gl.uniform2f(program.u.uCorners, columns - 1, rows - 1);
        gl.uniform2f(program.u.uScale, low, step);
        gl.uniform3fv(program.u.uSun, camera.sun);
        gl.uniform1f(program.u.uGrid, layers.grid ? 1024 : 0);
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, heightTexture);
        gl.uniform1i(program.u.uHeight, 0);
        gl.activeTexture(gl.TEXTURE1);
        gl.bindTexture(gl.TEXTURE_2D, diffuse);
        gl.uniform1i(program.u.uDiffuse, 1);
        gl.activeTexture(gl.TEXTURE2);
        gl.bindTexture(gl.TEXTURE_2D, slopeTexture);
        gl.uniform1i(program.u.uSlope, 2);
        let triangles = 0;
        for (let level = 0; level < LEVELS; level++) {
          if (!counts[level]) continue;
          const l = levels[level];
          gl.bindVertexArray(l.vao);
          gl.bindBuffer(gl.ARRAY_BUFFER, l.instances);
          gl.bufferSubData(gl.ARRAY_BUFFER, 0, l.patches, 0, counts[level] * 2);
          gl.uniform1f(program.u.uStep, 1 << level);
          gl.drawElementsInstanced(gl.TRIANGLES, l.count, gl.UNSIGNED_SHORT, 0, counts[level]);
          triangles += (l.count / 3) * counts[level];
        }
        gl.bindVertexArray(null);
        return triangles;
      },

      /// The water's surface, after everything solid.
      drawWater(camera) {
        if (!hasWater) return;
        gl.useProgram(water.program);
        gl.uniformMatrix4fv(water.u.uViewProj, false, camera.viewProj);
        gl.uniform2f(water.u.uSize, mapSize[0], mapSize[1]);
        gl.uniform3fv(water.u.uEye, camera.eye);
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
        gl.depthMask(false);
        gl.bindVertexArray(waterVao);
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
        gl.bindVertexArray(null);
        gl.depthMask(true);
        gl.disable(gl.BLEND);
      },
    };
    return terrain;
  }

  return { create };
})();
