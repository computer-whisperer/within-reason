use std::io::{self, ErrorKind, Read, Write};

use serde::{Serialize, de::DeserializeOwned};

const MAX_FRAME: usize = 64 << 20;

/// The protocol's fingerprint (`build.rs`), the first eight bytes each side writes on a connection.
pub const FINGERPRINT: &str = env!("WITHIN_REASON_PROTOCOL");

/// The first exchange on a connection, before any frame: each side writes the fingerprint and reads the other's.
/// `InvalidData` when they differ: the two were built from different protocol sources and every frame would be misread.
/// A peer from before the handshake writes a frame instead (the shim its `Hello`), and that reads as a mismatch too.
pub fn handshake(stream: &mut (impl Read + Write)) -> io::Result<()> {
    let ours = fingerprint_bytes();
    stream.write_all(&ours)?;
    stream.flush()?;
    let mut theirs = [0u8; 8];
    stream.read_exact(&mut theirs)?;
    if theirs != ours {
        return Err(io::Error::new(ErrorKind::InvalidData, format!("protocol {:016x}, ours {FINGERPRINT}", u64::from_be_bytes(theirs))));
    }
    Ok(())
}

fn fingerprint_bytes() -> [u8; 8] {
    u64::from_str_radix(FINGERPRINT, 16).expect("build.rs writes sixteen hex digits").to_be_bytes()
}

/// Writes one message as a little-endian `u32` length followed by its postcard encoding.
pub fn write_frame<T: Serialize>(writer: &mut impl Write, message: &T) -> io::Result<()> {
    let body = postcard::to_stdvec(message).map_err(io::Error::other)?;
    let len = u32::try_from(body.len()).map_err(io::Error::other)?;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(&body)?;
    writer.flush()
}

/// Reassembles frames from a stream that may deliver them in pieces, so it works on both
/// blocking and non-blocking sockets.
#[derive(Default)]
pub struct FrameReader {
    buf: Vec<u8>,
}

impl FrameReader {
    /// Returns `Ok(None)` when no complete frame is available yet (non-blocking source), and an
    /// `UnexpectedEof` error when the peer closed the connection.
    pub fn read<T: DeserializeOwned>(&mut self, source: &mut impl Read) -> io::Result<Option<T>> {
        loop {
            if let Some(message) = self.take_frame()? {
                return Ok(Some(message));
            }
            let mut chunk = [0u8; 64 * 1024];
            match source.read(&mut chunk) {
                Ok(0) => return Err(ErrorKind::UnexpectedEof.into()),
                Ok(n) => self.buf.extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(None),
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
    }

    fn take_frame<T: DeserializeOwned>(&mut self) -> io::Result<Option<T>> {
        let Some(header) = self.buf.first_chunk::<4>() else {
            return Ok(None);
        };
        let len = u32::from_le_bytes(*header) as usize;
        if len > MAX_FRAME {
            return Err(io::Error::new(ErrorKind::InvalidData, format!("frame of {len} bytes")));
        }
        if self.buf.len() < 4 + len {
            return Ok(None);
        }
        let message = postcard::from_bytes(&self.buf[4..4 + len]).map_err(io::Error::other);
        self.buf.drain(..4 + len);
        message.map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Commands, UnitId};

    /// Yields one byte per read, then `WouldBlock` once drained.
    struct Trickle(Vec<u8>);

    impl Read for Trickle {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if self.0.is_empty() {
                return Err(ErrorKind::WouldBlock.into());
            }
            out[0] = self.0.remove(0);
            Ok(1)
        }
    }

    /// Two sides of one build agree; a peer that writes a frame first (a shim from before the handshake) does not.
    #[test]
    fn the_handshake_tells_a_stale_peer() {
        let (mut a, mut b) = std::os::unix::net::UnixStream::pair().unwrap();
        let other = std::thread::spawn(move || handshake(&mut b));
        handshake(&mut a).unwrap();
        other.join().unwrap().unwrap();
        let (mut a, mut b) = std::os::unix::net::UnixStream::pair().unwrap();
        // Longer than the fingerprint, as the old shim's `Hello` is; `b` is kept so the read sees no end of file.
        write_frame(&mut b, &Commands((0..4).map(|i| Command::Stop { unit: UnitId(i) }).collect())).unwrap();
        let error = handshake(&mut a).unwrap_err();
        drop(b);
        assert_eq!(error.kind(), ErrorKind::InvalidData, "{error}");
    }

    #[test]
    fn reassembles_split_frames_and_reports_would_block() {
        let mut wire = Vec::new();
        write_frame(&mut wire, &Commands(vec![Command::Stop { unit: UnitId(7) }])).unwrap();
        write_frame(&mut wire, &Commands(vec![])).unwrap();
        let partial = wire.split_off(wire.len() - 1);

        let mut reader = FrameReader::default();
        let mut source = Trickle(wire);
        let first: Commands = reader.read(&mut source).unwrap().unwrap();
        assert!(matches!(first.0[..], [Command::Stop { unit: UnitId(7) }]));
        assert!(reader.read::<Commands>(&mut source).unwrap().is_none());

        source.0 = partial;
        let second: Commands = reader.read(&mut source).unwrap().unwrap();
        assert!(second.0.is_empty());
    }

    /// The blast radius goes over the wire (H-MICRO-FORM-SPACING reads it): the Fatboy's, as the engine gives it (the
    /// duel prints the engine's figure for it at every start: 150, half the unit file's area of 300).
    #[test]
    fn a_definition_carries_its_blast_radius() {
        let fatboy = crate::UnitDefInfo {
            id: crate::UnitDefId(1), name: "armfboy".into(), metal_cost: 1400.0, energy_cost: 0.0, speed: 43.0,
            build_speed: 0.0, build_time: 1.0, build_distance: 0.0, extracts_metal: 0.0, metal_make: 0.0,
            energy_make: 0.0, energy_upkeep: 0.0, wind_cap: 0.0, tidal_make: 0.0, metal_storage: 0.0,
            energy_storage: 0.0, sonar_range: 0.0, hits_submerged: false, water_only: false, radar_range: 0.0,
            converter: None, weapon_count: 1, build_options: Vec::new(), move_class: None, footprint: (3, 3),
            death_blast: None, self_destruct_blast: None, self_destruct_seconds: 0.0, reach: 700.0, reload: 7.0,
            blast_radius: 150.0, stockpile_seconds: 0.0, stockpile_energy: 0.0, coverage: 0.0, sight: 0.0, jam_range: 0.0, stealth: false, cloak: None, shield: None, intercepted_by: 0,
        };
        let mut wire = Vec::new();
        write_frame(&mut wire, &fatboy).unwrap();
        let mut reader = FrameReader::default();
        let read: crate::UnitDefInfo = reader.read(&mut Trickle(wire)).unwrap().unwrap();
        assert_eq!((read.name.as_str(), read.reach, read.blast_radius), ("armfboy", 700.0, 150.0));
    }
}
