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
//! **Status: M0 (block parse).** The block header, metadata sub-block framing,
//! scope gates, and a whole-stream [`StreamInfo`] scan are implemented and
//! conformance-tested against reference-encoded fixtures. Sample decode is
//! not implemented yet. The founding plan, milestone ladder, and
//! conformance-oracle method live in the repository's `design_docs/`.

#![forbid(unsafe_code)]

pub mod block;
pub mod error;
pub mod format;
pub mod metadata;

pub use block::{Block, BlockHeader, Blocks, StreamInfo};
pub use error::{Error, Scope};
