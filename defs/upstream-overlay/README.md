# Upstream definition overlay

`make schema` copies these files over the vendored last-public definitions
(`defs/upstream-runtime`) and current runtime reconciliation
(`defs/upstream-reconciliation`) before generating
`crates/nuxie-schema/src/generated/schema.rs`.
Each file replaces the upstream file at the same relative path. They carry
definitions that were ported ahead of the incremental sync, as recorded in
`docs/upstream-sync-map.md`. Retire definitions only when the sequential sync's
schema source covers those properties. A shared file can represent multiple
upstream commits: preserve later properties when an earlier commit is reached,
and delete the file only when all its definitions are covered.

Upstream removed `dev/defs` from the runtime repository in d4fe1022, so these
files are reconstructed from the generated C++ headers of the introducing
commit rather than copied. Crossing that removal does not make these overlays
redundant; the replacement schema source must reproduce their definitions
before they can be removed.

| File | Upstream commit | Adds |
|------|-----------------|------|
| `bitmap_cache.json` | a4dbc3ff (cache as bitmap) | BitmapCache (type 136): `resolution` 417, `cacheFlags` 418, and its passthrough bits `cacheEnabled` 419 and `dither` 420 |
| `text/text_input.json` | 7098a7c8 (input alignment), bec99be4 (obscured input) | `alignValue` 222, `verticalAlignValue` 1094, `obscured` 1095 |
