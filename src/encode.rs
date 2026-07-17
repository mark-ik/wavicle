//! The integer encode driver (M4).
//!
//! Shaped after `pack_samples` in the reference `pack.c` (dbry/WavPack,
//! BSD-3-Clause; see ATTRIBUTION.md), narrowed to the one configuration this
//! milestone ships: a single block, no decorrelation, no joint stereo, no
//! extended-integer packing. Residuals equal the input samples, so the entropy
//! coder is exercised directly. Byte-identity with the reference encoder is a
//! non-goal (see the founding plan); the gate is that the reference `wvunpack`
//! decodes our output losslessly. Decorrelation (smaller files) is a later
//! addition that does not change the format the decoder reads.

use crate::bitstream::BitWriter;
use crate::decorr::{DecorrPass, forward_decorr_mono_pass, forward_decorr_stereo_pass};
use crate::entropy::WordsEncoder;
use crate::error::Error;
use crate::format::{self, Flags, meta};

/// The fixed decorrelation this milestone stamps: a single term-2 pass. A
/// conforming decoder reads whatever config we write, so this is a valid
/// lossless choice; more/better terms are a later compression improvement that
/// does not change the stream format. Delta 2 is a common adaptation rate.
const FIXED_TERM: i32 = 2;
const FIXED_DELTA: i32 = 2;

/// Parameters describing the integer PCM to encode.
#[derive(Clone, Copy, Debug)]
pub struct EncodeParams {
    /// 1 or 2. Two channels are stored as independent left/right.
    pub channels: u32,
    pub sample_rate: u32,
    /// Stored bits per sample: 8, 16, 24, or 32.
    pub bits_per_sample: u32,
}

/// Encode interleaved integer `samples` to a single-block `.wv` stream.
///
/// `samples` holds sign-extended `i32` values (`channels * frames` of them).
/// Each must fit in `bits_per_sample`.
pub fn encode_int(params: EncodeParams, samples: &[i32]) -> Result<Vec<u8>, Error> {
    let EncodeParams {
        channels,
        sample_rate,
        bits_per_sample,
    } = params;

    if channels != 1 && channels != 2 {
        return Err(Error::OutOfScope(crate::error::Scope::MoreThanTwoChannels));
    }
    if !matches!(bits_per_sample, 8 | 16 | 24 | 32) {
        return Err(Error::NotYetImplemented("bit depth must be 8, 16, 24, or 32"));
    }
    let srate_index = format::SAMPLE_RATES
        .iter()
        .position(|&r| r == sample_rate)
        .ok_or(Error::NotYetImplemented("non-standard sample rate"))?
        as u32;

    let mono = channels == 1;
    if samples.len() % channels as usize != 0 {
        return Err(Error::Truncated {
            need: channels as usize,
            have: samples.len(),
        });
    }
    let frames = (samples.len() / channels as usize) as u32;
    if frames > format::MAX_BLOCK_SAMPLES {
        return Err(Error::NotYetImplemented(
            "multi-block encode (over 131072 frames)",
        ));
    }

    let bytes_per_sample = bits_per_sample / 8;
    // The magnitude field is the actual max, computed like the reference (the
    // OR of every sample folded through its own sign), not the nominal bit
    // depth. This keeps the decoder's mute limit correct and avoids a 1<<31
    // overflow at 32-bit.
    let mag_acc = samples
        .iter()
        .fold(0u32, |acc, &s| acc | if s < 0 { !s as u32 } else { s as u32 });
    let magnitude = if mag_acc == 0 { 0 } else { 32 - mag_acc.leading_zeros() };
    if magnitude > 31 {
        return Err(Error::OverMagnitude);
    }

    // CRC over the original samples, exactly as the reference and our decoder
    // compute it (seed 0xffffffff; mono crc*3+s, stereo crc*9 + 3*L + R).
    let mut crc: u32 = 0xffffffff;
    if mono {
        for &s in samples {
            crc = crc.wrapping_add(crc << 1).wrapping_add(s as u32);
        }
    } else {
        for f in samples.chunks_exact(2) {
            crc = crc
                .wrapping_add(crc << 3)
                .wrapping_add((f[0] as u32) << 1)
                .wrapping_add(f[0] as u32)
                .wrapping_add(f[1] as u32);
        }
    }

    // Forward-decorrelate a working copy with the fixed term (weights and
    // history start at zero, which is what the decorr metadata records), then
    // entropy-encode the residuals.
    let mut residuals = samples.to_vec();
    let mut pass = DecorrPass {
        term: FIXED_TERM,
        delta: FIXED_DELTA,
        ..DecorrPass::default()
    };
    if mono {
        forward_decorr_mono_pass(&mut pass, &mut residuals);
    } else {
        forward_decorr_stereo_pass(&mut pass, &mut residuals);
    }

    let mut words = WordsEncoder::new();
    let mut bw = BitWriter::new();
    words.send_words_lossless(&mut bw, &residuals, frames, mono);
    words.finish(&mut bw);
    let wv = bw.close();

    // Header flags.
    let mut flags: u32 = bytes_per_sample - 1;
    if mono {
        flags |= 1 << 2; // MONO_FLAG
    }
    flags |= Flags::INITIAL_BLOCK | Flags::FINAL_BLOCK;
    flags |= magnitude << 18;
    flags |= srate_index << 23;

    // Decorrelation metadata for the single fixed term, all-zero starting
    // state (one term byte; zero weights; `term` zero history entries).
    let term_byte = (((FIXED_TERM + 5) as u8) & 0x1f) | ((FIXED_DELTA as u8) << 5);
    let weights_len = if mono { 1 } else { 2 };
    let samples_len = FIXED_TERM as usize * if mono { 2 } else { 4 };

    // Assemble metadata, then the header (whose ckSize we fill in last).
    let mut meta_bytes = Vec::new();
    push_sub_block(&mut meta_bytes, meta::DECORR_TERMS, &[term_byte]);
    push_sub_block(&mut meta_bytes, meta::DECORR_WEIGHTS, &vec![0u8; weights_len]);
    push_sub_block(&mut meta_bytes, meta::DECORR_SAMPLES, &vec![0u8; samples_len]);
    push_sub_block(&mut meta_bytes, meta::ENTROPY_VARS, &WordsEncoder::entropy_vars(mono));
    push_wv_sub_block(&mut meta_bytes, &wv);

    let ck_size = (block::HEADER_LEN - 8 + meta_bytes.len()) as u32;
    let mut out = Vec::with_capacity(block::HEADER_LEN + meta_bytes.len());
    write_header(&mut out, ck_size, frames, flags, crc);
    out.extend_from_slice(&meta_bytes);
    Ok(out)
}

use crate::block;

fn write_header(out: &mut Vec<u8>, ck_size: u32, frames: u32, flags: u32, crc: u32) {
    out.extend_from_slice(&format::MAGIC);
    out.extend_from_slice(&ck_size.to_le_bytes());
    out.extend_from_slice(&format::MAX_STREAM_VERS.to_le_bytes()); // 0x410
    out.push(0); // block_index high byte
    out.push(0); // total_samples high byte
    out.extend_from_slice(&frames.to_le_bytes()); // total_samples (block_index == 0)
    out.extend_from_slice(&0u32.to_le_bytes()); // block_index
    out.extend_from_slice(&frames.to_le_bytes()); // block_samples
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&crc.to_le_bytes());
}

/// Append a small metadata sub-block (up to 510 data bytes). Odd-length data
/// sets the ODD_SIZE flag and is padded with one byte, as the decoder expects.
fn push_sub_block(out: &mut Vec<u8>, id: u8, data: &[u8]) {
    let odd = data.len() % 2 == 1;
    let words = data.len().div_ceil(2);
    debug_assert!(words <= 255);
    out.push(if odd { id | meta::ODD_SIZE } else { id });
    out.push(words as u8);
    out.extend_from_slice(data);
    if odd {
        out.push(0);
    }
}

/// Append the `ID_WV_BITSTREAM` sub-block, always in the large (3-byte size)
/// form as the reference does. The payload is even by construction.
fn push_wv_sub_block(out: &mut Vec<u8>, wv: &[u8]) {
    debug_assert!(wv.len() % 2 == 0);
    let words = (wv.len() / 2) as u32;
    out.push(meta::WV_BITSTREAM | meta::LARGE);
    out.push(words as u8);
    out.push((words >> 8) as u8);
    out.push((words >> 16) as u8);
    out.extend_from_slice(wv);
}
