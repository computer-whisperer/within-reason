//! The container and the number codings both ends share: little-endian, LEB128 varints, zigzag for the signed.

use std::fmt;

pub const MAGIC: &[u8; 8] = b"WRTAPE\x01\x00";
pub const END: &[u8; 8] = b"WRTAPEND";
/// A chunk's header: kind, flags, raw length, stored length.
pub const CHUNK_HEADER: usize = 16;
pub const FOOTER: usize = 16;
pub const FLAG_ZSTD: u32 = 1;
/// A frame that never comes: a unit not yet finished, or alive at the tape's end.
pub const NEVER: u32 = u32::MAX;

#[derive(Debug, Clone, PartialEq)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub fn err<T>(what: impl Into<String>) -> Result<T> {
    Err(Error(what.into()))
}

/// A byte writer.
#[derive(Default, Clone)]
pub struct W(pub Vec<u8>);

impl W {
    pub fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    pub fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn var(&mut self, mut v: u64) {
        while v >= 0x80 {
            self.0.push((v as u8) | 0x80);
            v >>= 7;
        }
        self.0.push(v as u8);
    }
    pub fn zig(&mut self, v: i64) {
        self.var(((v << 1) ^ (v >> 63)) as u64);
    }
    pub fn bytes(&mut self, b: &[u8]) {
        self.0.extend_from_slice(b);
    }
    /// A run of bytes with its length before it: a column of a slice or a table.
    pub fn column(&mut self, c: &W) {
        self.u32(c.0.len() as u32);
        self.0.extend_from_slice(&c.0);
    }
}

/// A byte reader over a slice.
#[derive(Clone)]
pub struct R<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> R<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        R { b, i: 0 }
    }
    pub fn left(&self) -> usize {
        self.b.len() - self.i
    }
    pub fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.left() < n {
            return err(format!("the tape ends {} bytes short", n - self.left()));
        }
        let s = &self.b[self.i..self.i + n];
        self.i += n;
        Ok(s)
    }
    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    pub fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub fn var(&mut self) -> Result<u64> {
        let (mut v, mut shift) = (0u64, 0);
        loop {
            let b = self.u8()?;
            v |= ((b & 0x7f) as u64) << shift;
            if b < 0x80 {
                return Ok(v);
            }
            shift += 7;
            if shift > 63 {
                return err("a varint runs past 64 bits");
            }
        }
    }
    pub fn zig(&mut self) -> Result<i64> {
        let v = self.var()?;
        Ok(((v >> 1) as i64) ^ -((v & 1) as i64))
    }
    pub fn column(&mut self) -> Result<R<'a>> {
        let n = self.u32()? as usize;
        Ok(R::new(self.take(n)?))
    }
}

/// One chunk as it lies in the file.
pub fn write_chunk(file: &mut Vec<u8>, kind: &[u8; 4], raw: &[u8], compressed: Option<Vec<u8>>) -> (u64, u32) {
    match &compressed {
        Some(c) => write_stored(file, kind, raw.len(), c, true),
        None => write_stored(file, kind, raw.len(), raw, false),
    }
}

/// A chunk whose payload is already as it is stored (`zstd`: compressed from `raw_len` bytes, else those bytes).
pub fn write_stored(file: &mut Vec<u8>, kind: &[u8; 4], raw_len: usize, stored: &[u8], zstd: bool) -> (u64, u32) {
    let offset = file.len() as u64;
    let flags = if zstd { FLAG_ZSTD } else { 0 };
    file.extend_from_slice(kind);
    file.extend_from_slice(&flags.to_le_bytes());
    file.extend_from_slice(&(raw_len as u32).to_le_bytes());
    file.extend_from_slice(&(stored.len() as u32).to_le_bytes());
    file.extend_from_slice(stored);
    (offset, (CHUNK_HEADER + stored.len()) as u32)
}

/// A chunk's kind and its payload, decompressed. `bytes` is the chunk whole, header first.
pub fn read_chunk(bytes: &[u8]) -> Result<([u8; 4], Vec<u8>)> {
    let mut r = R::new(bytes);
    let kind: [u8; 4] = r.take(4)?.try_into().unwrap();
    let flags = r.u32()?;
    let raw_len = r.u32()? as usize;
    let stored_len = r.u32()? as usize;
    let stored = r.take(stored_len)?;
    if flags & FLAG_ZSTD == 0 {
        return Ok((kind, stored.to_vec()));
    }
    use std::io::Read;
    let mut decoder = ruzstd::decoding::StreamingDecoder::new(stored).map_err(|e| Error(format!("zstd: {e}")))?;
    let mut out = Vec::with_capacity(raw_len);
    decoder.read_to_end(&mut out).map_err(|e| Error(format!("zstd: {e}")))?;
    if out.len() != raw_len {
        return err(format!("a {} chunk unpacks to {} bytes where its header says {}", String::from_utf8_lossy(&kind), out.len(), raw_len));
    }
    Ok((kind, out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varints_round_trip() {
        let mut w = W::default();
        let values = [0i64, 1, -1, 63, -64, 64, 300, -300, i32::MAX as i64, i32::MIN as i64, i64::MAX, i64::MIN];
        for v in values {
            w.zig(v);
        }
        w.var(u64::MAX);
        let mut r = R::new(&w.0);
        for v in values {
            assert_eq!(r.zig().unwrap(), v);
        }
        assert_eq!(r.var().unwrap(), u64::MAX);
        assert_eq!(r.left(), 0);
        assert!(r.u8().is_err());
    }
}
