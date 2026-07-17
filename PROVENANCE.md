# Provenance

Which wavicle module derives from which reference source. Updated whenever a
port lands; see `ATTRIBUTION.md` for the license notices.

| Module | Derived from | Notes |
|---|---|---|
| (none yet) | — | Name-claim stub; no code ported. |

Planned lineage, from the founding plan:

| Module | Reference lineage |
|---|---|
| `format` | `include/wavpack.h`, `src/wavpack_local.h` (constants) |
| `bitstream` | `getbit`/`putbit`/`getbits` macro family |
| `block`, `metadata` | header struct + `read_next_header` resync |
| `decorr` | `decorr_utils.c` (inverse); fixed forward config (ours) |
| `entropy` | `read_words.c`, `write_words.c`, `entropy_utils.c` |
| `float` | `unpack_floats.c`, `pack_floats.c` |
| `decode`/`encode` drivers | `unpack.c` / `pack.c` (shape only) |

Deliberately NOT ported: `decorr_tables.h` (the encoder ships one fixed
decorrelation configuration instead of the reference's table-driven search),
`unpack3.c` (pre-4.0 legacy), DSD and hybrid sources, and all hand-written
assembly (scalar Rust only).
