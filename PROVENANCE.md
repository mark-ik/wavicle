# Provenance

Which wavicle module derives from which reference source. Updated whenever a
port lands; see `ATTRIBUTION.md` for the license notices.

All ports are from **dbry/WavPack at tag 5.9.0** (the same version pinned as
the conformance oracle), read from copies kept outside this repo.

| Module | Derived from | Notes |
|---|---|---|
| `format` | `include/wavpack.h`, `src/wavpack_local.h` | constants only |
| `block`, `metadata` | header struct, `read_next_header` bounds, sub-block framing | M0 |
| `bitstream` | `wavpack_local.h` getbit/getbits macros, `open_utils.c` bs_open_read, `read_words.c` read_code and the unary/escape reads | M1; portable variant, not the CTZ/NEXT8 optimizations |
| `entropy` | `read_words.c` get_words_lossless, `entropy_utils.c` wp_exp2s + exp2 table, median macros in `wavpack_local.h` | M1; lossless path only |
| `decorr` | `decorr_utils.c` read_decorr_* + restore_weight, `unpack.c` decorr_mono_pass/decorr_stereo_pass, weight macros in `wavpack_local.h` | M1; inverse only |
| `decode` driver | `unpack.c` unpack_samples (shape, joint stereo, CRC, fixup lossless path) | M1; hard-errors where the reference mutes |

Planned lineage for later milestones:

| Module | Reference lineage |
|---|---|
| `float` | `unpack_floats.c`, `pack_floats.c` |
| `encode` driver + halves | `pack.c`, `write_words.c` (fixed decorr config, ours) |

Deliberately NOT ported: `decorr_tables.h` (the encoder ships one fixed
decorrelation configuration instead of the reference's table-driven search),
`unpack3.c` (pre-4.0 legacy), DSD and hybrid sources, and all hand-written
assembly (scalar Rust only).
