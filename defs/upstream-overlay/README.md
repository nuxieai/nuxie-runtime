# Upstream definition overlay

`make schema` copies these files over the pinned upstream `dev/defs`
(`RIVE_RUNTIME_REF`) before generating `crates/nuxie-schema/src/generated/schema.rs`.
Each file replaces the upstream file at the same relative path. They carry
definitions that were ported ahead of the incremental sync, as recorded in
`docs/upstream-sync-map.md`. Delete a file here once the sync reaches the
upstream commit that introduced it.

Upstream removed `dev/defs` from the runtime repository in d4fe1022, so these
files are reconstructed from the generated C++ headers of the introducing
commit rather than copied.

| File | Upstream commit | Adds |
|------|-----------------|------|
| `bitmap_cache.json` | a4dbc3ff (cache as bitmap) | BitmapCache (type 136): `resolution` 417, `cacheFlags` 418, and its passthrough bits `cacheEnabled` 419 and `dither` 420 |
| `text/text_input.json` | 7098a7c8 (input alignment), bec99be4 (obscured input) | `alignValue` 222, `verticalAlignValue` 1094, `obscured` 1095 |
