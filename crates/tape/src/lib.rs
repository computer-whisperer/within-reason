//! The game tape: one seekable file of everything a viewer draws of a game, both sides, made from the raw stream the
//! capture widget writes while the headless engine plays a demo. `docs/harness/tape-format.md` is the format;
//! `docs/design/2026-11-15-game-tape.md` why it is so.

pub mod format;
pub mod game;
#[cfg(feature = "pack")]
pub mod pack;
pub mod read;
pub mod slice;
pub mod tables;

/// Frames a slice covers: ten seconds.
pub const SLICE_FRAMES: u32 = 300;
/// Frames a projectile's end and a burst stay in the slices after them, for a reader to draw the flash.
pub const LINGER_FRAMES: u32 = 60;
