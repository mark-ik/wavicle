#![no_main]

//! Fuzz the decoder against arbitrary bytes. The contract is that
//! `decode_stream`/`StreamInfo::scan` never panic on untrusted input; they
//! return `Err` instead. The crate is `#![forbid(unsafe_code)]`, so this is
//! about logic panics (out-of-bounds slicing, arithmetic), not memory safety.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = wavicle::decode_stream(data);
    let _ = wavicle::StreamInfo::scan(data);
});
