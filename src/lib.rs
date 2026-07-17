//! wavicle — a pure-Rust WavPack v5 codec.
//!
//! A wavicle is Eddington's word for the quantum entity that is both wave and
//! particle. A lossless codec makes audio exactly that: a continuous wave to
//! the ear, discrete bits on disk, one identity in both observations.
//!
//! Scope: lossless mono/stereo decode and encode of the WavPack v5 stream —
//! 16/24/32-bit integer and bit-exact 32-bit float PCM. Pure Rust, no C,
//! wasm-capable. Out of scope: DSD, hybrid/lossy modes, correction files,
//! more than two channels, pre-4.0 legacy streams.
//!
//! **Status: M3 (lossless decoder complete).** Bit-exact lossless decode of
//! 16/24/32-bit integer and 32-bit float, mono and stereo (joint stereo,
//! false stereo, zero-run silence, multiblock, the wvx extension for >24-bit
//! integers and float). Verified sample-for-sample against the reference
//! wvunpack -r, with all CRCs enforced as hard errors; float decode also
//! passes a BLAKE3 round-trip identity check over the decoded f32 bytes.
//! Block parsing and scope gates landed at M0. The encoder (M4+) is not yet
//! implemented. The founding plan, milestone ladder, and conformance-oracle
//! method live in the repository's `design_docs/`.

#![forbid(unsafe_code)]

pub mod block;
pub mod error;
pub mod format;
pub mod metadata;

#[cfg(feature = "decode")]
pub mod bitstream;
#[cfg(feature = "decode")]
pub mod decode;
#[cfg(feature = "decode")]
pub mod decorr;
#[cfg(feature = "decode")]
pub mod entropy;
#[cfg(feature = "decode")]
pub mod float;

pub use block::{Block, BlockHeader, Blocks, StreamInfo};
#[cfg(feature = "decode")]
pub use decode::{DecodedStream, decode_stream};
pub use error::{Error, Scope};
