# R3.1 Subpixel-Edge Cohort B Audit

Date: 2026-07-15

This audit owns only the following thirteen gated clockwise-atomic rows:

- `riv-databind_viewmodel-frame-{0,1}-clockwise-atomic`
- `riv-double_line-frame-0-clockwise-atomic`
- `riv-ellipsis-frame-0-clockwise-atomic`
- `riv-fit_font_size_test-frame-0-clockwise-atomic`
- `riv-focus_traversal-frame-0-clockwise-atomic`
- `riv-follow_path_path-frame-0-clockwise-atomic`
- `riv-format_number_with_commas-frame-0-clockwise-atomic`
- `riv-hello_world-frame-0-clockwise-atomic`
- `riv-hunter_x_demo-frame-0-clockwise-atomic`
- `riv-interpolate_to_end-frame-0-clockwise-atomic`
- `riv-keyboard_listener-frame-0-clockwise-atomic`
- `riv-library-frame-0-clockwise-atomic`

The corpus contract, pinned native references, statuses, and tolerances were
not changed. All rows retain `metal-webgpu-subpixel-edge-coverage`.

## Provenance and Method

- Rust worktree: detached `2d63a5e7de07f3157169e149ad3e8498f4f42186`.
- Native producer: `/Users/levi/dev/oss/rive-runtime` at
  `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2`; its tracked source diff was
  clean. The FFI decoder provenance gate bound the native archive to
  `/Users/levi/dev/oss/rive-runtime/renderer/out/debug/librive_decoders.a`.
- Control executable: `target/debug/renderer-replay` with SHA-256
  `4a33e017e12fa950d164395cbad759b6a1be723d564fad7e8d4308ddfcfc8fbd` at
  capture time. It is linked to `Metal.framework` and replayed the same pinned
  streams through `ffi-metal`.
- Three fresh Rust `rust-wgpu`/Metal rounds replayed every owned row through
  fail-closed `corpus-r --probe-gated`; the external runner's 30-second cap
  required four completed batches per round. Every round reproduced the same
  raw-RGBA threshold mask.
- Immutable native C++/Metal controls replayed all thirteen frozen streams:
  `13/13` passed at `0` different pixels and `0` maximum channel delta. Parent
  review independently byte-compared every native output with its committed
  reference and confirmed `13/13` exact files.

Connected components below use 4-neighbor masks over the repository's exact
raw-RGBA PNG decoder: a pixel is in the residual mask when any channel differs
by more than `2`.

| Row | Pixels/max | Components/largest | Alpha nonzero / >2 / max |
| --- | --- | --- | --- |
| `databind_viewmodel` frame 0 | 96 / 35 | 21 / 8 | 0 / 0 / 0 |
| `databind_viewmodel` frame 1 | 96 / 35 | 21 / 8 | 0 / 0 / 0 |
| `double_line` | 145 / 57 | 45 / 8 | 0 / 0 / 0 |
| `ellipsis` | 35 / 33 | 15 / 6 | 0 / 0 / 0 |
| `fit_font_size_test` | 106 / 42 | 35 / 9 | 0 / 0 / 0 |
| `focus_traversal` | 101 / 100 | 49 / 7 | 0 / 0 / 0 |
| `follow_path_path` | 223 / 85 | 88 / 9 | 0 / 0 / 0 |
| `format_number_with_commas` | 496 / 27 | 97 / 9 | 0 / 0 / 0 |
| `hello_world` | 59 / 47 | 24 / 5 | 0 / 0 / 0 |
| `hunter_x_demo` | 222 / 18 | 90 / 59 | 318 / 1 / 18 |
| `interpolate_to_end` | 97 / 33 | 19 / 9 | 0 / 0 / 0 |
| `keyboard_listener` | 178 / 58 | 71 / 7 | 0 / 0 / 0 |
| `library` | 119 / 59 | 46 / 8 | 0 / 0 / 0 |

The twelve exact-alpha rows total 1,751 residual pixels in 531 components;
their largest component is nine pixels. This is the shared sparse,
subpixel-contour signature. Hunter is distinct: it is the only alpha outlier,
has the only component larger than nine pixels, and is the only row in this
cohort with blend modes `14`, `15`, `21`, or `24` (the other twelve use only
mode `3`).

## Discriminating Controls

- **Schedule and local edge draw:** `databind_viewmodel` frames 0 and 1 have
  identical pinned-reference hashes and identical residual masks. A command
  prefix of 12 commands is native/Rust exact; prefixes 18 and 20 both retain
  exactly 96 over-threshold pixels. The residual therefore appears with the
  first fractional text path and is unchanged by later commands.
- **Clip:** the prefix control above contains no `clipPath`, yet it reproduces
  the residual. Five cohort streams contain clips and seven do not; the shared
  twelve-row signature spans both groups, so a clip-stack defect is not a
  shared explanation.
- **Blend:** the same unclipped prefix uses only blend mode `3` and reproduces
  the residual. Advanced blend state is not required for the shared twelve-row
  signature; Hunter's distinct alpha result must not be folded into it.
- **Decoder:** `make renderer-decoder-oracle` passed against the provenance
  bound archive: the reachable profiled JPEG is byte-exact and the ICC PNG is
  within channel delta `2` with exact alpha. Library is the only cohort stream
  with `decodeImage`; its exact-alpha, 46-component residual matches the edge
  cluster rather than a decoded-image plane.
- **Alpha:** raw alpha is exact for twelve rows. Hunter has one pixel over the
  channel allowance (maximum alpha delta 18), which separates it from the
  shared edge-coverage family instead of providing evidence for a common Rust
  alpha defect.

## Decision

No objective, mutation-sensitive evidence identifies a shared Rust renderer
defect. In particular, a renderer mutation would need to improve the stable
exact-alpha masks across the twelve-row cluster while preserving the clean
native controls; this audit supplies no candidate change that predicts that
outcome. Retain all thirteen rows as gated under their existing diagnostic.

The cohort evidence is stored outside the worktree at
`/tmp/rive-rust-subpixel-edge-b-artifacts/`: three Rust round directories,
four native-control directories, command-prefix controls,
`component-summary.tsv`, and the temporary raw-RGBA audit tool.
