//! The decode driver: whole-stream lossless decode to interleaved samples.
//!
//! Shaped after `unpack_samples` in the reference `unpack.c` (dbry/WavPack,
//! BSD-3-Clause; see ATTRIBUTION.md), narrowed to the lossless integer path
//! this milestone implements. One deliberate divergence: where the reference
//! silently mutes a block on CRC mismatch or over-magnitude samples, wavicle
//! returns a hard error, because a silently-zeroed block would change decoded
//! content (and any content-addressed identity over it) without a signal.

use crate::bitstream::BitReader;
use crate::block::Blocks;
use crate::decorr::{
    DecorrPass, decorr_mono_pass, decorr_stereo_pass, read_decorr_samples, read_decorr_terms,
    read_decorr_weights,
};
use crate::entropy::WordsDecoder;
use crate::error::Error;
use crate::format::{Flags, meta};

/// A fully decoded stream: interleaved samples plus the facts needed to
/// interpret them.
#[derive(Clone, Debug)]
pub struct DecodedStream {
    /// Interleaved output samples (mono: one per frame; stereo: two).
    pub samples: Vec<i32>,
    pub channels: u32,
    pub sample_rate: u32,
    pub bits_per_sample: u32,
    pub is_float: bool,
}

/// Decode an entire `.wv` byte stream losslessly.
///
/// M1 scope: 16-bit integer PCM, mono or stereo (including false stereo and
/// joint stereo). Float and extended-integer streams parse but return
/// [`Error::NotYetImplemented`].
pub fn decode_stream(stream: &[u8]) -> Result<DecodedStream, Error> {
    let mut out: Option<DecodedStream> = None;

    for block in Blocks::new(stream) {
        let block = block?;
        let h = block.header;
        if h.block_samples == 0 {
            continue;
        }
        if h.flags.is_float() {
            return Err(Error::NotYetImplemented("32-bit float decode (M3)"));
        }
        if h.flags.0 & Flags::INT32_DATA != 0 {
            return Err(Error::NotYetImplemented("extended integer decode (M2)"));
        }
        if h.flags.bytes_per_sample() != 2 {
            return Err(Error::NotYetImplemented("non-16-bit integer decode (M2)"));
        }

        let samples = decode_block(&block)?;
        let rate = h
            .flags
            .sample_rate()
            .ok_or(Error::NotYetImplemented("non-standard sample rate"))?;

        match &mut out {
            None => {
                out = Some(DecodedStream {
                    samples,
                    channels: h.flags.output_channels(),
                    sample_rate: rate,
                    bits_per_sample: h.flags.bytes_per_sample() * 8,
                    is_float: false,
                })
            }
            Some(existing) => existing.samples.extend_from_slice(&samples),
        }
    }

    out.ok_or(Error::Truncated { need: 32, have: 0 })
}

/// Decode one audio block to its output samples (already un-joint-stereoed,
/// shifted, false-stereo-expanded, and CRC-checked).
fn decode_block(block: &crate::block::Block<'_>) -> Result<Vec<i32>, Error> {
    let h = &block.header;
    let flags = h.flags;
    // MONO_DATA in the reference: the stored stream is one channel either
    // because the file is mono or because identical channels collapsed.
    let mono_data = flags.mono_stored() || flags.0 & Flags::FALSE_STEREO != 0;

    let mut passes: Option<Vec<DecorrPass>> = None;
    let mut words: Option<WordsDecoder> = None;
    let mut wv_payload: Option<&[u8]> = None;

    for sub in block.sub_blocks() {
        let sub = sub?;
        crate::metadata::check_scope(sub)?;
        match sub.id {
            meta::DECORR_TERMS => passes = Some(read_decorr_terms(sub.data, mono_data)?),
            meta::DECORR_WEIGHTS => {
                let p = passes
                    .as_mut()
                    .ok_or(Error::MissingSubBlock("decorr terms before weights"))?;
                read_decorr_weights(sub.data, p, mono_data)?;
            }
            meta::DECORR_SAMPLES => {
                let p = passes
                    .as_mut()
                    .ok_or(Error::MissingSubBlock("decorr terms before samples"))?;
                read_decorr_samples(sub.data, p, mono_data)?;
            }
            meta::ENTROPY_VARS => {
                words = Some(WordsDecoder::from_entropy_vars(sub.data, mono_data)?)
            }
            meta::WV_BITSTREAM => wv_payload = Some(sub.data),
            _ => {}
        }
    }

    let mut passes = passes.ok_or(Error::MissingSubBlock("decorr terms"))?;
    let mut words = words.ok_or(Error::MissingSubBlock("entropy vars"))?;
    let wv = wv_payload.ok_or(Error::MissingSubBlock("wv bitstream"))?;

    let frames = h.block_samples as usize;
    let stored_len = if mono_data { frames } else { frames * 2 };
    let mut buffer = vec![0i32; stored_len];

    let mut bs = BitReader::new(wv);
    let produced = words.get_words_lossless(&mut bs, &mut buffer, h.block_samples, mono_data);
    if produced != h.block_samples || bs.errored() {
        return Err(Error::Truncated {
            need: h.block_samples as usize,
            have: produced as usize,
        });
    }

    if mono_data {
        for pass in passes.iter_mut() {
            decorr_mono_pass(pass, &mut buffer);
        }
    } else {
        for pass in passes.iter_mut() {
            decorr_stereo_pass(pass, &mut buffer);
        }
    }

    // Joint-stereo un-mix and the block CRC, in the reference's one pass.
    let mut crc: u32 = 0xffffffff;
    let mute_limit = (1i64 << flags.magnitude()) + 2;
    if mono_data {
        for s in buffer.iter() {
            if (*s as i64).abs() > mute_limit {
                return Err(Error::OverMagnitude);
            }
            crc = crc.wrapping_mul(3).wrapping_add(*s as u32);
        }
    } else {
        let joint = flags.0 & Flags::JOINT_STEREO != 0;
        for f in buffer.chunks_exact_mut(2) {
            if joint {
                f[1] = f[1].wrapping_sub(f[0] >> 1);
                f[0] = f[0].wrapping_add(f[1]);
            }
            crc = crc
                .wrapping_add(crc << 3)
                .wrapping_add((f[0] as u32) << 1)
                .wrapping_add(f[0] as u32)
                .wrapping_add(f[1] as u32);
            if (f[0] as i64).abs() > mute_limit || (f[1] as i64).abs() > mute_limit {
                return Err(Error::OverMagnitude);
            }
        }
    }

    if crc != h.crc {
        return Err(Error::CrcMismatch {
            stored: h.crc,
            computed: crc,
        });
    }

    // Lossless fixup: only the final left-shift applies on this path.
    let shift = flags.output_shift();
    if shift != 0 {
        for s in buffer.iter_mut() {
            *s = ((*s as u32) << shift) as i32;
        }
    }

    // FALSE_STEREO: stored mono, output stereo.
    if flags.0 & Flags::FALSE_STEREO != 0 {
        let mut expanded = Vec::with_capacity(frames * 2);
        for s in &buffer {
            expanded.push(*s);
            expanded.push(*s);
        }
        buffer = expanded;
    }

    Ok(buffer)
}
