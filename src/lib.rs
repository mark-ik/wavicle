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
//! **Status: M4 (lossless decoder complete; integer encoder landed).** The
//! decoder handles 16/24/32-bit integer and 32-bit float, mono and stereo,
//! bit-exact against the reference `wvunpack -r` (float also passes a BLAKE3
//! identity check). The encoder ([`encode_int`], behind the `encode` feature)
//! writes 8/16/24/32-bit integer single-block streams that the reference
//! `wvunpack` decodes losslessly, with a fixed single-term decorrelation.
//! Float encode (M5) and multi-block/decorrelation-tuning are the remaining
//! work. The founding plan and conformance-oracle method live in the
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

#[cfg(feature = "decode")]
pub mod decode;
#[cfg(feature = "decode")]
pub mod float;

#[cfg(feature = "encode")]
pub mod encode;

pub use block::{Block, BlockHeader, Blocks, StreamInfo};
pub use error::{Error, Scope};
#[cfg(feature = "decode")]
pub use decode::{DecodedStream, decode_stream};
#[cfg(feature = "encode")]
pub use encode::{EncodeParams, encode_int};
