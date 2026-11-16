// The game's own files as the page draws with them: S3O models, DDS textures, the map's SMF and SMT. Served by
// run/view_match.py at assets/<name> out of run/data/tape-assets, the copies the tape's capture made (named by
// content); a tape's header says which copy a path of the game's archives is (`head.assets`). Nothing here is
// converted and kept: the files are read as they are, each time.
"use strict";

const WRAssets = (() => {
  async function fetchBytes(name) {
    const response = await fetch(`assets/${name}`);
    if (!response.ok) throw new Error(`asset ${name}: ${response.status}`);
    return response.arrayBuffer();
  }

  function cString(bytes, at) {
    let end = at;
    while (end < bytes.length && bytes[end] !== 0) end++;
    return new TextDecoder().decode(bytes.subarray(at, end));
  }

  // ------------------------------------------------------------------------------------------------------- S3O
  // The engine's model format (rts/Rendering/Models/s3o.h): a header, then pieces in a tree, each with vertices of
  // eight floats (place, normal, texture u v) relative to the piece, an index table (triangles, strips or quads),
  // and its offset from its parent. Here the tree is flattened at its rest pose: one vertex and index list, each
  // vertex moved by its piece's offsets, with the piece's number kept beside it for the day pieces are posed.
  function parseS3O(buffer) {
    const bytes = new Uint8Array(buffer);
    const view = new DataView(buffer);
    if (cString(bytes, 0) !== "Spring unit") throw new Error("not an S3O model");
    const i32 = (at) => view.getInt32(at, true);
    const f32 = (at) => view.getFloat32(at, true);
    const model = {
      radius: f32(16), height: f32(20), mid: [f32(24), f32(28), f32(32)],
      tex1: i32(44) ? cString(bytes, i32(44)) : "", tex2: i32(48) ? cString(bytes, i32(48)) : "",
      pieces: [],
    };
    const vertices = [], indices = [];
    let lowest = Infinity, highest = -Infinity;
    const walk = (at, parent, origin) => {
      const piece = {
        name: cString(bytes, i32(at)), parent, offset: [f32(at + 40), f32(at + 44), f32(at + 48)],
        origin: [origin[0] + f32(at + 40), origin[1] + f32(at + 44), origin[2] + f32(at + 48)],
      };
      const number = model.pieces.length;
      model.pieces.push(piece);
      const base = vertices.length / 9;
      const count = i32(at + 12), vertexAt = i32(at + 16), kind = i32(at + 24), tableSize = i32(at + 28), tableAt = i32(at + 32);
      for (let v = 0; v < count; v++) {
        const o = vertexAt + v * 32;
        const y = f32(o + 4) + piece.origin[1];
        if (y < lowest) lowest = y;
        if (y > highest) highest = y;
        vertices.push(f32(o) + piece.origin[0], y, f32(o + 8) + piece.origin[2], f32(o + 12), f32(o + 16), f32(o + 20), f32(o + 24), f32(o + 28), number);
      }
      const index = (n) => view.getUint32(tableAt + n * 4, true);
      if (kind === 0) {
        for (let n = 0; n + 2 < tableSize; n += 3) indices.push(base + index(n), base + index(n + 1), base + index(n + 2));
      } else if (kind === 1) {
        // Strips: 0xffffffff ends one; the winding alternates.
        let run = 0;
        for (let n = 0; n < tableSize; n++) {
          if (index(n) === 0xffffffff) { run = 0; continue; }
          if (run >= 2) {
            if (run % 2 === 0) indices.push(base + index(n - 2), base + index(n - 1), base + index(n));
            else indices.push(base + index(n - 1), base + index(n - 2), base + index(n));
          }
          run++;
        }
      } else {
        for (let n = 0; n + 3 < tableSize; n += 4) indices.push(base + index(n), base + index(n + 1), base + index(n + 2), base + index(n), base + index(n + 2), base + index(n + 3));
      }
      const children = i32(at + 4), childrenAt = i32(at + 8);
      for (let c = 0; c < children; c++) walk(i32(childrenAt + c * 4), number, piece.origin);
    };
    walk(i32(36), -1, [0, 0, 0]);
    model.vertices = new Float32Array(vertices);
    model.indices = new Uint32Array(indices);
    model.low = lowest === Infinity ? 0 : lowest;
    model.high = highest === -Infinity ? model.height : highest;
    return model;
  }

  // ------------------------------------------------------------------------------------------------------- DDS
  const FOURCC = (s) => s.charCodeAt(0) | (s.charCodeAt(1) << 8) | (s.charCodeAt(2) << 16) | (s.charCodeAt(3) << 24);

  /// A DDS file's mip levels as they lie in it. `format`: dxt1, dxt3, dxt5, or rgba / bgra / bgr for the plain ones.
  function parseDDS(buffer) {
    const view = new DataView(buffer);
    if (view.getUint32(0, true) !== FOURCC("DDS ")) throw new Error("not a DDS texture");
    const height = view.getUint32(12, true), width = view.getUint32(16, true);
    const mipCount = Math.max(1, view.getUint32(28, true));
    const pfFlags = view.getUint32(80, true), fourCC = view.getUint32(84, true), bits = view.getUint32(88, true);
    let format, blockBytes = 0;
    if (pfFlags & 4) {
      format = { [FOURCC("DXT1")]: "dxt1", [FOURCC("DXT3")]: "dxt3", [FOURCC("DXT5")]: "dxt5" }[fourCC];
      if (!format) throw new Error(`a DDS texture in a format this page does not read (${String.fromCharCode(fourCC & 255, (fourCC >> 8) & 255, (fourCC >> 16) & 255, fourCC >> 24)})`);
      blockBytes = format === "dxt1" ? 8 : 16;
    } else {
      const red = view.getUint32(92, true);
      format = bits === 32 ? (red === 0xff ? "rgba" : "bgra") : "bgr";
    }
    const mips = [];
    let at = 128, w = width, h = height;
    for (let level = 0; level < mipCount && at < buffer.byteLength; level++) {
      const size = blockBytes ? Math.max(1, (w + 3) >> 2) * Math.max(1, (h + 3) >> 2) * blockBytes : w * h * (bits >> 3);
      if (at + size > buffer.byteLength) break;
      mips.push({ width: w, height: h, data: new Uint8Array(buffer, at, size) });
      at += size;
      w = Math.max(1, w >> 1);
      h = Math.max(1, h >> 1);
    }
    return { width, height, format, mips };
  }

  /// A 5:6:5 colour as bytes.
  const rgb565 = (c) => [((c >> 11) & 31) * 255 / 31, ((c >> 5) & 63) * 255 / 63, (c & 31) * 255 / 31];

  /// DXT1, DXT3 or DXT5 blocks as RGBA bytes: for a browser without the S3TC extension.
  function decodeDXT(data, width, height, format) {
    const out = new Uint8Array(width * height * 4);
    const blockBytes = format === "dxt1" ? 8 : 16;
    const blocksX = Math.max(1, (width + 3) >> 2), blocksY = Math.max(1, (height + 3) >> 2);
    const alpha = new Uint8Array(16), a8 = new Uint8Array(8);
    for (let by = 0; by < blocksY; by++) {
      for (let bx = 0; bx < blocksX; bx++) {
        let at = (by * blocksX + bx) * blockBytes;
        alpha.fill(255);
        if (format === "dxt3") {
          for (let i = 0; i < 16; i++) alpha[i] = ((data[at + (i >> 1)] >> ((i & 1) * 4)) & 15) * 17;
          at += 8;
        } else if (format === "dxt5") {
          a8[0] = data[at]; a8[1] = data[at + 1];
          for (let i = 2; i < 8; i++) a8[i] = a8[0] > a8[1] ? ((8 - i) * a8[0] + (i - 1) * a8[1]) / 7 : i < 6 ? ((6 - i) * a8[0] + (i - 1) * a8[1]) / 5 : i === 6 ? 0 : 255;
          // 48 bits of three-bit picks, low first
          let bitsLow = data[at + 2] | (data[at + 3] << 8) | (data[at + 4] << 16), bitsHigh = data[at + 5] | (data[at + 6] << 8) | (data[at + 7] << 16);
          for (let i = 0; i < 16; i++) {
            if (i < 8) { alpha[i] = a8[bitsLow & 7]; bitsLow >>= 3; } else { alpha[i] = a8[bitsHigh & 7]; bitsHigh >>= 3; }
          }
          at += 8;
        }
        const c0 = data[at] | (data[at + 1] << 8), c1 = data[at + 2] | (data[at + 3] << 8);
        const p0 = rgb565(c0), p1 = rgb565(c1);
        const four = format !== "dxt1" || c0 > c1;
        const palette = [p0, p1,
          four ? [(2 * p0[0] + p1[0]) / 3, (2 * p0[1] + p1[1]) / 3, (2 * p0[2] + p1[2]) / 3] : [(p0[0] + p1[0]) / 2, (p0[1] + p1[1]) / 2, (p0[2] + p1[2]) / 2],
          four ? [(p0[0] + 2 * p1[0]) / 3, (p0[1] + 2 * p1[1]) / 3, (p0[2] + 2 * p1[2]) / 3] : [0, 0, 0]];
        for (let i = 0; i < 16; i++) {
          const x = bx * 4 + (i & 3), y = by * 4 + (i >> 2);
          if (x >= width || y >= height) continue;
          const pick = (data[at + 4 + (i >> 2)] >> ((i & 3) * 2)) & 3;
          const o = (y * width + x) * 4;
          out[o] = palette[pick][0]; out[o + 1] = palette[pick][1]; out[o + 2] = palette[pick][2];
          out[o + 3] = !four && pick === 3 ? 0 : alpha[i];
        }
      }
    }
    return out;
  }

  /// A parsed DDS as a texture. With the S3TC extension its blocks go to the GPU as they are, every mip level; without
  /// it the level that fits `maxPlain` is decoded here and the GPU makes the smaller ones.
  function uploadDDS(gl, dds, { maxPlain = 2048 } = {}) {
    const texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);
    const s3tc = gl.getExtension("WEBGL_compressed_texture_s3tc");
    const compressed = dds.format.startsWith("dxt");
    if (compressed && s3tc) {
      const code = { dxt1: s3tc.COMPRESSED_RGBA_S3TC_DXT1_EXT, dxt3: s3tc.COMPRESSED_RGBA_S3TC_DXT3_EXT, dxt5: s3tc.COMPRESSED_RGBA_S3TC_DXT5_EXT }[dds.format];
      dds.mips.forEach((mip, level) => gl.compressedTexImage2D(gl.TEXTURE_2D, level, code, mip.width, mip.height, 0, mip.data));
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAX_LEVEL, dds.mips.length - 1);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, dds.mips.length > 1 ? gl.LINEAR_MIPMAP_LINEAR : gl.LINEAR);
    } else {
      const mip = dds.mips.find((m) => m.width <= maxPlain && m.height <= maxPlain) || dds.mips[dds.mips.length - 1];
      let pixels;
      if (compressed) pixels = decodeDXT(mip.data, mip.width, mip.height, dds.format);
      else {
        const step = dds.format === "bgr" ? 3 : 4;
        pixels = new Uint8Array(mip.width * mip.height * 4);
        for (let i = 0, o = 0; o < pixels.length; i += step, o += 4) {
          const swap = dds.format !== "rgba";
          pixels[o] = mip.data[i + (swap ? 2 : 0)]; pixels[o + 1] = mip.data[i + 1]; pixels[o + 2] = mip.data[i + (swap ? 0 : 2)];
          pixels[o + 3] = step === 4 ? mip.data[i + 3] : 255;
        }
      }
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, mip.width, mip.height, 0, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      gl.generateMipmap(gl.TEXTURE_2D);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
    }
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    return texture;
  }

  /// One opaque pixel of a colour: what stands in for a texture that is missing or not yet here.
  function plainTexture(gl, rgba) {
    const texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(rgba));
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    return texture;
  }

  // ------------------------------------------------------------------------------------------------ SMF and SMT
  // The map (rts/Map/SMF/SMFFormat.h). The SMF holds a table of tile numbers, one a 32-texel tile of the map's
  // texture, and names the SMT files that hold the tiles: each tile four mip levels of DXT1 (32, 16, 8 and 4 texels:
  // 512, 128, 32 and 8 bytes). The whole texture at a level is those tiles laid side by side, block rows copied
  // without unpacking anything.
  function parseSMF(buffer) {
    const bytes = new Uint8Array(buffer);
    const view = new DataView(buffer);
    if (cString(bytes, 0) !== "spring map file") throw new Error("not an SMF map");
    const i32 = (at) => view.getInt32(at, true);
    const smf = { mapx: i32(24), mapy: i32(28), minHeight: view.getFloat32(44, true), maxHeight: view.getFloat32(48, true), files: [] };
    let at = i32(60);
    const fileCount = i32(at);
    smf.tileCount = i32(at + 4);
    at += 8;
    for (let f = 0; f < fileCount; f++) {
      const count = i32(at);
      const name = cString(bytes, at + 4);
      smf.files.push({ name, count });
      at += 4 + name.length + 1;
    }
    smf.tilesX = smf.mapx / 4;
    smf.tilesY = smf.mapy / 4;
    smf.tiles = new Int32Array(buffer.slice(at, at + smf.tilesX * smf.tilesY * 4));
    return smf;
  }

  const TILE_BYTES = 680;
  const LEVEL_AT = [0, 512, 640, 672];

  /// The map's texture at tile mip `level` (0: a texel an elmo) as one DXT1 image: { width, height, data }.
  /// `tileFiles` are the SMT buffers in the SMF's order.
  function composeMap(smf, tileFiles, level) {
    const blocks = 8 >> level; // DXT blocks along a tile's side
    const rowBytes = blocks * 8;
    const width = smf.tilesX * blocks * 4, height = smf.tilesY * blocks * 4;
    const data = new Uint8Array(smf.tilesX * blocks * smf.tilesY * blocks * 8);
    // Where each tile number lives: (file bytes, offset of the tile).
    const sources = [];
    smf.files.forEach((file, f) => {
      const bytes = new Uint8Array(tileFiles[f]);
      for (let t = 0; t < file.count; t++) sources.push([bytes, 32 + t * TILE_BYTES + LEVEL_AT[level]]);
    });
    const stride = smf.tilesX * rowBytes;
    for (let ty = 0; ty < smf.tilesY; ty++) {
      for (let tx = 0; tx < smf.tilesX; tx++) {
        const source = sources[smf.tiles[ty * smf.tilesX + tx]];
        if (!source) continue;
        const [bytes, from] = source;
        for (let row = 0; row < blocks; row++) {
          data.set(bytes.subarray(from + row * rowBytes, from + (row + 1) * rowBytes), (ty * blocks + row) * stride + tx * rowBytes);
        }
      }
    }
    return { width, height, data };
  }

  /// The map's texture on the GPU: the finest tile level whose image fits `limit` texels a side, and the coarser
  /// levels under it as its mips. Without the S3TC extension, one coarse level decoded here.
  function uploadMap(gl, smf, tileFiles, limit) {
    const s3tc = gl.getExtension("WEBGL_compressed_texture_s3tc");
    const texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);
    const longest = Math.max(smf.tilesX, smf.tilesY) * 32;
    let level = 0;
    while (level < 3 && (longest >> level) > (s3tc ? limit : Math.min(limit, 2048))) level++;
    if (s3tc) {
      for (let l = level; l <= 3; l++) {
        const image = composeMap(smf, tileFiles, l);
        gl.compressedTexImage2D(gl.TEXTURE_2D, l - level, s3tc.COMPRESSED_RGB_S3TC_DXT1_EXT, image.width, image.height, 0, image.data);
      }
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAX_LEVEL, 3 - level);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, level < 3 ? gl.LINEAR_MIPMAP_LINEAR : gl.LINEAR);
    } else {
      const image = composeMap(smf, tileFiles, level);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, image.width, image.height, 0, gl.RGBA, gl.UNSIGNED_BYTE, decodeDXT(image.data, image.width, image.height, "dxt1"));
      gl.generateMipmap(gl.TEXTURE_2D);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
    }
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    return { texture, level, texelsPerElmo: 1 / (1 << level) };
  }

  return { fetchBytes, parseS3O, parseDDS, decodeDXT, uploadDDS, plainTexture, parseSMF, composeMap, uploadMap };
})();

if (typeof module !== "undefined") module.exports = WRAssets;
