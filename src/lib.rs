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
//! **Status: M2 (integer decode complete).** Lossless decode of 16/24/32-bit
//! mono and stereo streams (joint stereo, false stereo, zero-run silence,
//! multiblock, and the wvx extension for >24-bit) is verified sample-for-sample
//! against the reference wvunpack -r, with both CRCs enforced as hard errors.
//! Block parsing and scope gates landed at M0. Not yet implemented:
//! float (M3) and the encoder (M4+). The founding
//! plan, milestone ladder, and conformance-oracle method live in the
//! repository's `design_docs/`.

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

pub use block::{Block, BlockHeader, Blocks, StreamInfo};
#[cfg(feature = "decode")]
pub use decode::{DecodedStream, decode_stream};
pub use error::{Error, Scope};
