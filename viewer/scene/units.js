// The units: each type's model drawn once for all its units (instanced), straight from the reader's array of the
// state at a frame (sixteen floats a unit, by type: place, front, up, health, build progress, team, flags, row,
// type, paralysis). A unit too small on screen to read is the game's own icon instead, as in the game (unless the
// view's `icons` switch is off). Health bars stand over the hurt and the unfinished.
"use strict";

const WRUnits = (() => {
  const STRIDE = 16 * 4;
  /// A unit whose radius is under this many pixels is drawn as its icon.
  const ICON_BELOW = 9;
  /// An icon's half-width in pixels at the icon table's size 1.
  const ICON_HALF = 9;
  const ICON_CELL = 64;

  const SHARED = `
  layout(location = 3) in vec3 iPos;
  layout(location = 4) in vec3 iFront;
  layout(location = 5) in vec3 iUp;
  layout(location = 6) in vec4 iState;   // health, build, team, flags
  layout(location = 7) in vec3 iIdent;   // row, type, paralysis
  uniform mat4 uViewProj;
  uniform float uPixels;                 // pixels an elmo covers at distance 1
  uniform vec3 uTeams[64];
  `;

  const MODEL_VERTEX = `#version 300 es
  layout(location = 0) in vec3 aPlace;
  layout(location = 1) in vec3 aNormal;
  layout(location = 2) in vec2 aUV;
  layout(location = 8) in float aPiece;  // the vertex's piece, in the engine's order from 0; -1 none
  layout(location = 9) in float iBlock;  // the unit's block of piece transforms in uPieces; -1 at rest
  ${SHARED}
  uniform highp sampler2D uPieces;       // a row a block: three texels a piece (rotation by columns, translation)
  uniform float uRadius;
  uniform float uIconBelow;
  out vec2 vUV;
  out vec3 vNormal;
  out vec3 vTeam;
  out float vLocalY;
  out vec4 vState;
  out float vParalysis;
  void main() {
    // Its piece where the unit's pieces stand this frame (turned, moved), else where the model has it at rest.
    vec3 place = aPlace, normal = aNormal;
    if (iBlock >= 0.0 && aPiece >= 0.0) {
      int x = int(aPiece) * 3, y = int(iBlock);
      vec4 a = texelFetch(uPieces, ivec2(x, y), 0), b = texelFetch(uPieces, ivec2(x + 1, y), 0), c = texelFetch(uPieces, ivec2(x + 2, y), 0);
      mat3 r = mat3(a.xyz, vec3(a.w, b.xy), vec3(b.zw, c.x));
      place = r * aPlace + c.yzw;
      normal = r * aNormal;
    }
    // The engine's unit frame: x along up x front, y along up, z along front.
    vec3 right = cross(iUp, iFront);
    vec3 world = iPos + right * place.x + iUp * place.y + iFront * place.z;
    vNormal = right * normal.x + iUp * normal.y + iFront * normal.z;
    vUV = aUV;
    vTeam = uTeams[int(iState.z)];
    vLocalY = aPlace.y;
    vState = iState;
    vParalysis = iIdent.z;
    vec4 centre = uViewProj * vec4(iPos, 1.0);
    gl_Position = uViewProj * vec4(world, 1.0);
    // Too small to read: its icon stands for it (the icon pass), and the model goes outside the view.
    if (uRadius * uPixels / max(centre.w, 1.0) < uIconBelow) gl_Position = vec4(0.0, 0.0, 2.0, 1.0);
  }`;

  const MODEL_FRAGMENT = `#version 300 es
  precision highp float;
  uniform sampler2D uTex1;   // colour, and in its alpha where the team's colour goes
  uniform sampler2D uTex2;   // red: glow; alpha: where the model is cut away
  uniform vec3 uSun;
  uniform vec2 uHeights;     // the model's lowest and highest y
  in vec2 vUV;
  in vec3 vNormal;
  in vec3 vTeam;
  in float vLocalY;
  in vec4 vState;
  in float vParalysis;
  out vec4 color;
  void main() {
    // The game's textures are stored top row first; the model's v runs from the bottom.
    vec2 uv = vec2(vUV.x, 1.0 - vUV.y);
    vec4 t1 = texture(uTex1, uv);
    vec4 t2 = texture(uTex2, uv);
    if (t2.a < 0.5) discard;
    vec3 base = mix(t1.rgb, vTeam, t1.a);
    vec3 n = normalize(vNormal);
    vec3 c = base * (0.5 + 0.75 * max(dot(n, uSun), 0.0)) + t1.rgb * t2.r * 0.8;
    bool sieve = fract((gl_FragCoord.x + gl_FragCoord.y) * 0.5) < 0.5;
    if (vState.y < 0.999) {
      // Being built: solid up to its progress, a green ghost above.
      float built = uHeights.x + (uHeights.y - uHeights.x) * vState.y;
      if (vLocalY > built) {
        if (sieve) discard;
        c = mix(c, vec3(0.25, 1.0, 0.45), 0.7);
      }
    }
    int flags = int(vState.w);
    if ((flags & 1) != 0) {
      // Cloaked: half there, bluish.
      if (sieve) discard;
      c = mix(c, vec3(0.5, 0.7, 1.0), 0.5);
    }
    if (vParalysis >= 1.0) c = mix(c, vec3(0.75, 0.85, 1.0), 0.45);   // stunned
    if ((flags & 8) != 0) {
      // Not seen by the side the view shows: a ghost, three pixels of four away, grey.
      if (fract(gl_FragCoord.x * 0.5) < 0.5 || fract(gl_FragCoord.y * 0.5) < 0.5) discard;
      c = mix(c, vec3(dot(c, vec3(0.3, 0.5, 0.2))), 0.7) * 0.8;
    }
    color = vec4(c, 1.0);
  }`;

  const ICON_VERTEX = `#version 300 es
  layout(location = 0) in vec2 aCorner;
  ${SHARED}
  uniform highp sampler2D uTypes;        // by type: the icon's cell (column, row), its size, the type's radius
  uniform vec2 uViewport;
  uniform float uIconBelow;
  uniform float uIconHalf;
  uniform float uCells;                  // cells along the atlas's side
  out vec2 vUV;
  out vec3 vTeam;
  out float vAlpha;
  void main() {
    vec4 info = texelFetch(uTypes, ivec2(int(iIdent.y), 0), 0);
    vec4 centre = uViewProj * vec4(iPos, 1.0);
    bool readable = info.w * uPixels / max(centre.w, 1.0) >= uIconBelow;
    if (readable || centre.w <= 0.0 || info.z <= 0.0) {
      gl_Position = vec4(0.0, 0.0, 2.0, 1.0);
      return;
    }
    vec2 extent = vec2(uIconHalf * info.z) * 2.0 / uViewport;
    gl_Position = vec4(centre.xy / centre.w + aCorner * extent, 0.0, 1.0);
    vUV = (info.xy + vec2(aCorner.x * 0.5 + 0.5, 0.5 - aCorner.y * 0.5)) / uCells;
    vTeam = uTeams[int(iState.z)];
    vAlpha = iState.y < 0.999 ? 0.45 : 1.0;
    // Not seen by the side the view shows: faint, less so when on its radar.
    int flags = int(iState.w);
    if ((flags & 8) != 0) vAlpha *= (flags & 16) != 0 ? 0.55 : 0.25;
  }`;

  const ICON_FRAGMENT = `#version 300 es
  precision highp float;
  uniform sampler2D uAtlas;
  in vec2 vUV;
  in vec3 vTeam;
  in float vAlpha;
  out vec4 color;
  void main() {
    vec4 t = texture(uAtlas, vUV);
    if (t.a < 0.08) discard;
    // The game's icons are white shapes with dark edges: tinted as the engine tints them, by the team's colour.
    color = vec4(t.rgb * vTeam, t.a * vAlpha);
  }`;

  const BAR_VERTEX = `#version 300 es
  layout(location = 0) in vec2 aCorner;
  ${SHARED}
  uniform highp sampler2D uTypes;
  uniform vec2 uViewport;
  uniform float uIconBelow;
  out vec2 vAt;
  out float vHealth;
  out float vBuild;
  void main() {
    vec4 info = texelFetch(uTypes, ivec2(int(iIdent.y), 0), 0);
    vec4 centre = uViewProj * vec4(iPos + vec3(0.0, info.w * 1.6 + 6.0, 0.0), 1.0);
    bool whole = iState.x >= 0.995 && iState.y >= 0.999;
    bool small = info.w * uPixels / max(centre.w, 1.0) < uIconBelow;
    if (whole || small || centre.w <= 0.0) {
      gl_Position = vec4(0.0, 0.0, 2.0, 1.0);
      return;
    }
    float extent = clamp(info.w * uPixels / centre.w, 12.0, 34.0);
    gl_Position = vec4(centre.xy / centre.w + aCorner * vec2(extent, 2.5) * 2.0 / uViewport, 0.0, 1.0);
    vAt = aCorner * 0.5 + 0.5;
    vHealth = iState.x;
    vBuild = iState.y;
  }`;

  const BAR_FRAGMENT = `#version 300 es
  precision highp float;
  in vec2 vAt;
  in float vHealth;
  in float vBuild;
  out vec4 color;
  void main() {
    // An unfinished unit's bar is its build progress, in the nanoframe's green-blue.
    bool building = vBuild < 0.999;
    float level = building ? vBuild : vHealth;
    vec3 full = building ? vec3(0.35, 0.75, 1.0) : mix(vec3(0.9, 0.15, 0.1), vec3(0.2, 0.9, 0.25), smoothstep(0.2, 0.75, vHealth));
    color = vAt.x <= level ? vec4(full, 0.95) : vec4(0.05, 0.05, 0.05, 0.7);
  }`;

  const RING_VERTEX = `#version 300 es
  layout(location = 0) in vec2 aCorner;
  uniform mat4 uViewProj;
  uniform vec4 uAt;      // the unit's place and the ring's radius
  out vec2 vAt;
  void main() {
    vAt = aCorner;
    gl_Position = uViewProj * vec4(uAt.x + aCorner.x * uAt.w, uAt.y + 2.0, uAt.z + aCorner.y * uAt.w, 1.0);
  }`;

  const RING_FRAGMENT = `#version 300 es
  precision highp float;
  in vec2 vAt;
  out vec4 color;
  void main() {
    float r = length(vAt);
    float edge = fwidth(r) * 1.5;
    float ring = smoothstep(0.86 - edge, 0.86, r) * (1.0 - smoothstep(1.0 - edge, 1.0, r));
    if (ring < 0.02) discard;
    color = vec4(1.0, 1.0, 1.0, ring * 0.9);
  }`;

  /// A box of a type's footprint and height, in the models' vertex layout: what stands in until a type's model is
  /// here, and for a type whose model the cache lacks.
  function box(def) {
    const hx = (def.xsize || 2) * 4, hz = (def.zsize || 2) * 4, h = Math.max(8, def.height || 16);
    const faces = [
      [[0, 1, 0], [-hx, h, -hz], [-hx, h, hz], [hx, h, hz], [hx, h, -hz]],
      [[1, 0, 0], [hx, 0, -hz], [hx, h, -hz], [hx, h, hz], [hx, 0, hz]],
      [[-1, 0, 0], [-hx, 0, hz], [-hx, h, hz], [-hx, h, -hz], [-hx, 0, -hz]],
      [[0, 0, 1], [hx, 0, hz], [hx, h, hz], [-hx, h, hz], [-hx, 0, hz]],
      [[0, 0, -1], [-hx, 0, -hz], [-hx, h, -hz], [hx, h, -hz], [hx, 0, -hz]],
    ];
    const vertices = [], indices = [];
    faces.forEach(([n, ...corners], f) => {
      for (const c of corners) vertices.push(c[0], c[1], c[2], n[0], n[1], n[2], 0.5, 0.5, -1);
      indices.push(f * 4, f * 4 + 1, f * 4 + 2, f * 4, f * 4 + 2, f * 4 + 3);
    });
    return { vertices: new Float32Array(vertices), indices: new Uint32Array(indices), low: 0, high: h, radius: Math.max(def.radius || 0, Math.hypot(hx, hz)), tex1: "", tex2: "" };
  }

  function create(gl, tape) {
    const model = WRGL.program(gl, MODEL_VERTEX, MODEL_FRAGMENT);
    const icon = WRGL.program(gl, ICON_VERTEX, ICON_FRAGMENT);
    const bar = WRGL.program(gl, BAR_VERTEX, BAR_FRAGMENT);
    const ring = WRGL.program(gl, RING_VERTEX, RING_FRAGMENT);
    const instances = gl.createBuffer();
    let capacity = 0;
    /// The features' own instances: the same sixteen floats a feature, by type, rebuilt when the set alive changes.
    const featureInstances = gl.createBuffer();
    let featureRanges = [], featureVersion = -1;

    /// Points the five per-unit attributes of the bound vertex array at an instance buffer, `first` units in.
    const pointInstances = (first, buffer = instances) => {
      gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
      const at = first * STRIDE;
      for (const [location, size, offset] of [[3, 3, 0], [4, 3, 12], [5, 3, 24], [6, 4, 36], [7, 3, 52]]) {
        gl.enableVertexAttribArray(location);
        gl.vertexAttribPointer(location, size, gl.FLOAT, false, STRIDE, at + offset);
        gl.vertexAttribDivisor(location, 1);
      }
      // Each unit's block of piece transforms, beside the units; a feature's pieces stand at rest.
      if (buffer === instances) {
        gl.bindBuffer(gl.ARRAY_BUFFER, blocks);
        gl.enableVertexAttribArray(9);
        gl.vertexAttribPointer(9, 1, gl.FLOAT, false, 4, first * 4);
        gl.vertexAttribDivisor(9, 1);
      } else {
        gl.disableVertexAttribArray(9);
        gl.vertexAttrib1f(9, -1);
      }
    };
    /// The units' blocks (`tape.frame`'s `pieceAt`), and the blocks themselves as a texture (`pieces`).
    const blocks = gl.createBuffer();
    let blockCapacity = 0, restBlocks = new Float32Array(0);
    const pieceTexture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, pieceTexture);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, 1, 1, 0, gl.RGBA, gl.FLOAT, new Float32Array(4));
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

    // Team colours, by team number.
    const teams = new Float32Array(64 * 3).fill(0.6);
    for (const team of tape.head.teams || []) if (team.team < 64) teams.set(team.color, team.team * 3);

    // Textures by the name a model gives (the game's unittextures/ files), each fetched once.
    // (While a model's own texture is on its way, and on a box, the whole surface takes the team's colour.)
    const white = WRAssets.plainTexture(gl, [255, 255, 255, 255]);
    const noGlow = WRAssets.plainTexture(gl, [0, 0, 0, 255]);
    const textures = new Map();
    /// Fetches a texture's file into its entry, when the tape names a copy of it (a growing tape may name it later:
    /// the entry stays a stand-in, `stand`, and is filled in place at `tapeGrew`, so the models holding it see it).
    const fill = (entry, key) => {
      const file = (tape.head.assets || {})[key];
      entry.stand = !file;
      if (file && /\.dds$/.test(key)) {
        WRAssets.fetchBytes(file).then((bytes) => {
          entry.texture = WRAssets.uploadDDS(gl, WRAssets.parseDDS(bytes));
          units.onChange && units.onChange();
        }).catch((e) => console.error(key, e));
      }
    };
    const textureOf = (name, fallback) => {
      if (!name) return { texture: fallback };
      const key = `unittextures/${name.toLowerCase()}`;
      let entry = textures.get(key);
      if (!entry) {
        entry = { texture: fallback };
        textures.set(key, entry);
        fill(entry, key);
      }
      return entry;
    };

    // Models by unit type, each built when its type first appears.
    const models = new Map();
    const build = (shape) => {
      const vao = gl.createVertexArray();
      gl.bindVertexArray(vao);
      gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
      gl.bufferData(gl.ARRAY_BUFFER, shape.vertices, gl.STATIC_DRAW);
      for (const [location, size, offset] of [[0, 3, 0], [1, 3, 12], [2, 2, 24], [8, 1, 32]]) {
        gl.enableVertexAttribArray(location);
        gl.vertexAttribPointer(location, size, gl.FLOAT, false, 36, offset);
      }
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, gl.createBuffer());
      gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, shape.indices, gl.STATIC_DRAW);
      pointInstances(0);
      gl.bindVertexArray(null);
      return { vao, count: shape.indices.length, low: shape.low, high: shape.high, radius: shape.radius, tex1: textureOf(shape.tex1, white), tex2: textureOf(shape.tex2, noGlow) };
    };
    /// A type's model: a unit type's (`feature` false) or a feature type's. A box of its footprint stands in until
    /// the model is here and where the cache has none; a feature with no model at all (an engine-made tree) is null.
    const modelOf = (defId, feature = false) => {
      const key = feature ? -1 - defId : defId;
      let entry = models.get(key);
      if (entry !== undefined) return entry;
      const def = (feature ? tape.featureDefs : tape.unitDefs).get(defId) || {};
      const file = def.model && (tape.head.assets || {})[(def.model.path || "").toLowerCase()];
      if (feature && !file) {
        models.set(key, null);
        return null;
      }
      entry = build(box(def));
      entry.radius = def.radius || entry.radius;
      entry.stand = !file; // (asked again when a growing tape brings the file)
      models.set(key, entry);
      if (file && /\.s3o$/.test(file)) {
        WRAssets.fetchBytes(file).then((bytes) => {
          const shape = WRAssets.parseS3O(bytes);
          // Each vertex's piece as the engine numbers it (the tape's pieces, by name, from 0); a feature's and a piece
          // the tape does not name stand at rest (-1).
          const names = !feature && def.pieces ? def.pieces.names.map((n) => String(n).toLowerCase()) : [];
          const engine = shape.pieces.map((p) => names.indexOf(String(p.name).toLowerCase()));
          for (let v = 8; v < shape.vertices.length; v += 9) shape.vertices[v] = engine[shape.vertices[v]] ?? -1;
          const made = build(shape);
          made.radius = def.radius || made.radius;
          models.set(key, made);
          units.onChange && units.onChange();
        }).catch((e) => console.error(def.name, e));
      }
      return entry;
    };

    // The features in type order, and the frames at which the set alive changes: worked out once, and again when a
    // growing tape brings more.
    let featureCount = 0, featureOrder = [], featureChanges = [];
    const indexFeatures = () => {
      featureCount = tape.features.length / 14;
      featureOrder = Array.from({ length: featureCount }, (_, i) => i).sort((a, b) => tape.features[a * 14 + 11] - tape.features[b * 14 + 11]);
      featureChanges = [];
      for (let i = 0; i < featureCount; i++) {
        featureChanges.push(tape.features[i * 14 + 12]);
        if (tape.features[i * 14 + 13] >= 0) featureChanges.push(tape.features[i * 14 + 13]);
      }
      featureChanges.sort((a, b) => a - b);
      featureVersion = -1;
    };
    indexFeatures();
    /// How many of those changes lie at or before a frame: the same number, the same features alive.
    const changesBy = (frame) => {
      let low = 0, high = featureChanges.length;
      while (low < high) {
        const mid = (low + high) >> 1;
        if (featureChanges[mid] <= frame) low = mid + 1; else high = mid;
      }
      return low;
    };
    const buildFeatures = (frame) => {
      const f = tape.features;
      const data = new Float32Array(featureCount * 16);
      featureRanges = [];
      let n = 0;
      for (const i of featureOrder) {
        const a = i * 14;
        if (f[a + 12] > frame || (f[a + 13] >= 0 && f[a + 13] <= frame)) continue;
        const def = f[a + 11];
        const last = featureRanges[featureRanges.length - 1];
        if (last && last[0] === def) last[2]++;
        else featureRanges.push([def, n, 1]);
        // place, front, up as they are; whole, finished, no team's colour (63), no row
        data.set(f.subarray(a, a + 9), n * 16);
        data.set([1, 1, 63, 0, -1, def, 0], n * 16 + 9);
        n++;
      }
      gl.bindBuffer(gl.ARRAY_BUFFER, featureInstances);
      gl.bufferData(gl.ARRAY_BUFFER, data.subarray(0, n * 16), gl.DYNAMIC_DRAW);
    };

    // The types' table for the icon and bar passes (a texel a type: the icon's cell, its size, the type's radius), and
    // the icons' atlas: every icon of the game (viewer/icons, its own bitmaps), so that a type a growing tape meets
    // later has its cell already (`types`).
    let typeData = new Float32Array(4);
    let typeWidth = 1;
    const typeTexture = gl.createTexture();
    let atlas = WRAssets.plainTexture(gl, [255, 255, 255, 255]);
    let cells = 1;
    let icons = null; // name -> [column, row, size], once the atlas is drawn
    const types = () => {
      typeWidth = Math.max(1, ...tape.defs.units.map((d) => d.id)) + 1;
      typeData = new Float32Array(typeWidth * 4);
      for (const d of tape.defs.units) {
        const cell = icons && icons.get(d.name);
        typeData.set(cell ? [cell[0], cell[1], cell[2], d.radius || 16] : [0, 0, 0, d.radius || 16], d.id * 4);
      }
      gl.bindTexture(gl.TEXTURE_2D, typeTexture);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, typeWidth, 1, 0, gl.RGBA, gl.FLOAT, typeData);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    };
    types();
    (async () => {
      const table = await (await fetch("icons.json")).json();
      const files = [...new Set(Object.values(table).map((t) => t.file))];
      cells = Math.max(1, Math.ceil(Math.sqrt(files.length)));
      const canvas = document.createElement("canvas");
      canvas.width = canvas.height = cells * ICON_CELL;
      const ctx = canvas.getContext("2d");
      await Promise.all(files.map((file, i) => new Promise((done) => {
        const image = new Image();
        image.onload = () => { ctx.drawImage(image, (i % cells) * ICON_CELL, Math.floor(i / cells) * ICON_CELL, ICON_CELL, ICON_CELL); done(); };
        image.onerror = done;
        image.src = `icons/${file}`;
      })));
      icons = new Map(Object.entries(table).map(([name, t]) => { const i = files.indexOf(t.file); return [name, [i % cells, Math.floor(i / cells), t.size || 1]]; }));
      types();
      atlas = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, atlas);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, canvas);
      gl.generateMipmap(gl.TEXTURE_2D);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      units.onChange && units.onChange();
    })().catch((e) => console.error("the icons:", e));

    // One quad for the icon, bar and ring passes.
    const quad = gl.createVertexArray();
    gl.bindVertexArray(quad);
    gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    pointInstances(0);
    gl.bindVertexArray(null);

    const common = (p, camera) => {
      gl.useProgram(p.program);
      gl.uniformMatrix4fv(p.u.uViewProj, false, camera.viewProj);
      gl.uniform1f(p.u.uPixels, camera.pixels);
      gl.uniform3fv(p.u.uTeams, teams);
    };

    const units = {
      onChange: null, ICON_BELOW,
      /// The tape grew (one still being written): the types' table, the features, and the models that had no file.
      tapeGrew() {
        types();
        indexFeatures();
        for (const [key, entry] of models) if (entry === null || entry.stand) models.delete(key);
        for (const [key, entry] of textures) if (entry.stand) fill(entry, key);
      },

      /// Draws the features standing at a frame (trees, rocks, wrecks): models only, never icons. Says how many.
      drawFeatures(frame, camera) {
        if (!featureCount) return 0;
        const version = changesBy(frame);
        if (version !== featureVersion) {
          featureVersion = version;
          buildFeatures(frame);
        }
        common(model, camera);
        gl.uniform3fv(model.u.uSun, camera.sun);
        gl.uniform1f(model.u.uIconBelow, 0);
        gl.uniform1i(model.u.uTex1, 0);
        gl.uniform1i(model.u.uTex2, 1);
        // (a feature's pieces stand at rest: its block attribute is -1, the texture never read)
        gl.activeTexture(gl.TEXTURE2);
        gl.bindTexture(gl.TEXTURE_2D, pieceTexture);
        gl.uniform1i(model.u.uPieces, 2);
        let drawn = 0;
        for (const [defId, first, n] of featureRanges) {
          const m = modelOf(defId, true);
          if (!m) continue;
          gl.bindVertexArray(m.vao);
          pointInstances(first, featureInstances);
          gl.uniform1f(model.u.uRadius, m.radius);
          gl.uniform2f(model.u.uHeights, m.low, m.high);
          gl.activeTexture(gl.TEXTURE0);
          gl.bindTexture(gl.TEXTURE_2D, m.tex1.texture);
          gl.activeTexture(gl.TEXTURE1);
          gl.bindTexture(gl.TEXTURE_2D, m.tex2.texture);
          gl.drawElementsInstanced(gl.TRIANGLES, m.count, gl.UNSIGNED_INT, 0, n);
          drawn += n;
        }
        gl.bindVertexArray(null);
        return drawn;
      },

      /// Draws the units of a state (`tape.frame`'s answer), a unit too small to read as its icon unless `icons` is
      /// false (then its model at any distance, and no bar). Says how many it drew as models' draw calls.
      draw(state, camera, selected, icons = true) {
        const count = state.units.length / 16;
        if (!count) return 0;
        gl.bindBuffer(gl.ARRAY_BUFFER, instances);
        if (state.units.length > capacity) {
          capacity = state.units.length * 2;
          gl.bufferData(gl.ARRAY_BUFFER, capacity * 4, gl.DYNAMIC_DRAW);
        }
        gl.bufferSubData(gl.ARRAY_BUFFER, 0, state.units);
        // Each unit's block of piece transforms (a tape with the moving pieces), and the blocks as a texture.
        const animated = state.pieceAt && state.pieceAt.length === count && state.pieces.length > 0;
        gl.bindBuffer(gl.ARRAY_BUFFER, blocks);
        if (count > blockCapacity) {
          blockCapacity = count * 2;
          gl.bufferData(gl.ARRAY_BUFFER, blockCapacity * 4, gl.DYNAMIC_DRAW);
          restBlocks = new Float32Array(blockCapacity).fill(-1);
        }
        gl.bufferSubData(gl.ARRAY_BUFFER, 0, animated ? state.pieceAt : restBlocks, 0, count);
        gl.bindTexture(gl.TEXTURE_2D, pieceTexture);
        if (animated) {
          const width = state.pieceStride * 3;
          gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, width, state.pieces.length / (width * 4), 0, gl.RGBA, gl.FLOAT, state.pieces);
        }

        common(model, camera);
        gl.uniform3fv(model.u.uSun, camera.sun);
        gl.uniform1f(model.u.uIconBelow, icons ? ICON_BELOW : 0);
        gl.activeTexture(gl.TEXTURE2);
        gl.bindTexture(gl.TEXTURE_2D, pieceTexture);
        gl.uniform1i(model.u.uPieces, 2);
        gl.uniform1i(model.u.uTex1, 0);
        gl.uniform1i(model.u.uTex2, 1);
        let draws = 0;
        for (let r = 0; r < state.ranges.length; r += 3) {
          const [defId, first, n] = [state.ranges[r], state.ranges[r + 1], state.ranges[r + 2]];
          const m = modelOf(defId);
          gl.bindVertexArray(m.vao);
          pointInstances(first);
          gl.uniform1f(model.u.uRadius, m.radius);
          gl.uniform2f(model.u.uHeights, m.low, m.high);
          gl.activeTexture(gl.TEXTURE0);
          gl.bindTexture(gl.TEXTURE_2D, m.tex1.texture);
          gl.activeTexture(gl.TEXTURE1);
          gl.bindTexture(gl.TEXTURE_2D, m.tex2.texture);
          gl.drawElementsInstanced(gl.TRIANGLES, m.count, gl.UNSIGNED_INT, 0, n);
          draws++;
        }

        // Over everything: the selection's ring on the ground, then icons and bars with no depth test, as the game
        // draws its own.
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
        gl.bindVertexArray(quad);
        if (selected) {
          gl.useProgram(ring.program);
          gl.uniformMatrix4fv(ring.u.uViewProj, false, camera.viewProj);
          gl.uniform4f(ring.u.uAt, selected.x, selected.y, selected.z, selected.radius * 1.35 + 4);
          gl.depthMask(false);
          gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
          gl.depthMask(true);
        }
        gl.disable(gl.DEPTH_TEST);
        pointInstances(0);
        if (icons) {
          common(icon, camera);
          gl.uniform2f(icon.u.uViewport, camera.width, camera.height);
          gl.uniform1f(icon.u.uIconBelow, ICON_BELOW);
          gl.uniform1f(icon.u.uIconHalf, ICON_HALF * camera.dpr);
          gl.uniform1f(icon.u.uCells, cells);
          gl.activeTexture(gl.TEXTURE0);
          gl.bindTexture(gl.TEXTURE_2D, typeTexture);
          gl.uniform1i(icon.u.uTypes, 0);
          gl.activeTexture(gl.TEXTURE1);
          gl.bindTexture(gl.TEXTURE_2D, atlas);
          gl.uniform1i(icon.u.uAtlas, 1);
          gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, count);
        }

        common(bar, camera);
        gl.uniform2f(bar.u.uViewport, camera.width, camera.height);
        gl.uniform1f(bar.u.uIconBelow, ICON_BELOW);
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, typeTexture);
        gl.uniform1i(bar.u.uTypes, 0);
        gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, count);
        gl.enable(gl.DEPTH_TEST);
        gl.disable(gl.BLEND);
        gl.bindVertexArray(null);
        return draws;
      },

      /// A type's radius as the game has it.
      radiusOf(defId) {
        return (tape.unitDefs.get(defId) || {}).radius || 16;
      },
    };
    return units;
  }

  return { create };
})();
