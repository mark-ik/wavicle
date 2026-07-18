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
//! **Status: M5 (round-trip codec complete for the tiny profile).** Both
//! directions, bit-exact and lossless: decode and encode of 16/24/32-bit
//! integer and 32-bit float, mono and stereo. Verified against the reference
//! `wvunpack`/`wavpack` over a corpus; float in both directions passes a
//! BLAKE3 round-trip identity check over the decoded f32 bytes. Decode is the
//! default build; [`encode_int`] and [`encode_float`] are behind the `encode`
//! feature and use a single fixed decorrelation term (a valid lossless choice;
//! smaller files via better decorrelation are a later, format-compatible
//! improvement). Remaining: multi-block encode for long files, and the Hocket
//! integration. The founding plan and conformance-oracle method live in the
//! repository's `design_docs/`.

#![forbid(unsafe_code)]

pub mod block;
pub mod error;
pub mod format;
pub mod metadata;

// Shared between the two directions (bit I/O, the median model, sample tables).
#[cfg(any(feature = "decode", feature = "encode"))]
pub mod bitstream;
#[cfg(any(feature = "decode", feature = "encode"))]
pub mod entropy;

#[cfg(any(feature = "decode", feature = "encode"))]
pub mod decorr;

#[cfg(any(feature = "decode", feature = "encode"))]
pub mod float;

#[cfg(feature = "decode")]
pub mod decode;

#[cfg(feature = "encode")]
pub mod encode;

pub use block::{Block, BlockHeader, Blocks, StreamInfo};
pub use error::{Error, Scope};
#[cfg(feature = "decode")]
pub use decode::{DecodedStream, decode_stream};
#[cfg(feature = "encode")]
pub use encode::{EncodeParams, encode_float, encode_int};
