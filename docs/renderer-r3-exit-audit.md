# Renderer R3 Exit Audit

Date: 2026-07-14

## Verdict

R3 corpus convergence is complete at `exact=1,377`, `diverges=0`,
`gated=91`, `total=1,468`. Every non-gated entry passes its committed
contract on the macOS Metal CI backend, and every retained gate has a specific
feature, backend/compiler boundary, or harness diagnostic. No
`algorithm-core` placeholder remains.

This verdict establishes an honest corpus classification; it does not retire
the actionable gates or authorize R4. R3.1 owns their burn-down before
performance work.

The corpus manifest has no renderer-selection field separate from its status:
`status = "exact"` is the production Rust-renderer ratchet, while
`status = "gated"` retains the documented boundary.

## Entry Gates

- The GPU semantic-trap audit is closed. Shader provenance, clip-plane routing,
  decoded-image ingress, compiler semantics, and accepted findings are recorded
  in `docs/renderer-status.md`.
- The dual-renderer fuzz-replay entry gate is closed. The deterministic hostile
  stream families, deadlines, findings, and CI smoke gate are recorded in
  `docs/renderer-fuzz-replay.md`.
- The full renderer corpus and V2 regression floor pass: renderer
  `1,377/0/91`, normal V2 `584` exact segments, scripted V2 `35` exact
  segments, and `cargo test --workspace`.

## Retained Gate Taxonomy

| Count | Diagnostic |
| ---: | --- |
| 50 | `metal-webgpu-subpixel-edge-coverage` |
| 10 | `rust-wgpu-msaa-gradient-stroke-composite` |
| 5 | `native-clockwise-atomic-advanced-feather-parity` |
| 5 | `rust-wgpu-msaa-feather-gradient-advanced-blend` |
| 5 | `rust-wgpu-msaa-gradient-edge-residual` |
| 3 | `platform-image-decode-color-profile` |
| 2 | `metal-webgpu-atomic-intermediate-precision` |
| 1 | `dawn-wgpu-msaa-advanced-blend-intermediate-precision` |
| 1 | `dawn-wgpu-msaa-interleaved-feather-color-precision` |
| 1 | `dawn-wgpu-msaa-image-rect-dither-accumulation` |
| 1 | `dawn-wgpu-msaa-stroke-edge-coverage` |
| 1 | `metal-webgpu-fixed-function-color-output` |
| 1 | `reference-harness: C++ Metal does not implement MSAA flush` |
| 1 | `rust-wgpu-msaa-feather-gradient-stroke` |
| 1 | `rust-wgpu-msaa-gradient-advanced-blend` |
| 1 | `rust-wgpu-msaa-gradient-path-clip` |
| 1 | `rust-wgpu-msaa-incompatible-clip-rectangles` |
| 1 | `rust-wgpu-msaa-repeated-path-clipped-strokes` |
| **91** | **Total** |

Operationally, these rows collapse into three groups:

| Rows | Disposition |
| ---: | --- |
| 0 | Reference/oracle harness gap |
| 61 | Reviewed backend, decoder, or precision boundary |
| 30 | Unsupported feature or remaining algorithm-parity boundary |

The actionable set is 30 rows: five prior clockwise-atomic feather boundaries
plus 25 concrete renderer findings exposed by strict replay. The other 61 rows
remain parked unless same-backend evidence exposes a Rust defect.

R3.1 promoted `riv-bullet_man-frame-0-clockwise-atomic` after porting C++'s
incompatible transformed-rectangle fallback to the ordinary clip stack. Its
native Metal comparison is byte-exact under the unchanged `2/32` contract.
R3.1 also promoted `gm-beziers-msaa`: the unchanged row moves from
5,385 pixels/max delta 152 immediately before the dedicated C++ MSAA stroke
depth state (`90c8fd52`) to 8 pixels/max delta 3 immediately after it. The
existing focused duplicate-contour GPU regression pins that self-overdraw
behavior, correcting the stale cubic-raster classification.
The same historical replay closes `gm-cliprectintersections-msaa`: it moves
from 240 pixels/max delta 55 before `90c8fd52` to byte-exact/max delta 1 after
the stroke depth state. The retained edge components were stroke self-overdraw,
not a clip-intersection raster boundary.
R3.1 reclassified `riv-coin-frame-0-clockwise-atomic` after eight draw-prefix
comparisons showed the first excess on a clipped zero-feather ring, not an
advanced-feather draw. Its final 48 outliers form 13 one-pixel-wide path/clip
edge components, largest 12, matching the retained Metal/WebGPU subpixel-edge
boundary without a tolerance change.
R3.1 promoted `riv-bankcard-frame-0-clockwise-atomic` after proving that Rust
hoisted atlas blits ahead of ordinary paths in mixed atomic flushes. Preserving
the authored draw order reduces the native-Metal comparison from 1,485,510
pixels/max delta 20 to 22 pixels/max delta 18, passing the unchanged `2/32`
contract. A focused mixed path-to-atlas regression pins the ordering behavior.

Strict gradient and render-buffer replay now cover the complete 732-case Dawn
registry. One continuous capture preserved all 686 prior PNGs byte-for-byte
and added the 46 previously blocked references. Isolated Rust probes promoted
`riv-interactive_scrolling-frame-0-msaa` byte-exact and converted the other 45
rows into three executable renderer queues. The manifest and
`tools/cpp-atlas-mask-oracle/msaa-reference-inventory.json` now contain no
actionable strict-replay placeholder.

The first ordinary MSAA gradient slice now renders and binds the shared C++
color-ramp texture for direct path fills and strokes, including destination-read
accounting and gradient auxiliary transforms. Seventeen of the 37 gradient
rows promote under unchanged `2/32`. The remaining 20 are no longer grouped
behind a generic gradient-path gate: they are queryable as repeated path-clipped
strokes, gradient advanced blending, feathered gradient strokes, incompatible
clip rectangles, a clipped gradient path, ten gradient stroke-composite rows,
and five identical 45-pixel/max-delta-3 edge residuals.

The C++ MSAA image-mesh path is now ported with the generated WGSL, typed
position/UV/index streams, authored sampler and blend state, path and
rectangle clipping, and draw-order depth. `gm-mesh-msaa` and
`riv-tape-frame-0-msaa` promote under unchanged contracts. Disposable C++
Dawn command-prefix captures isolate Jellyfish: the background and all 19
meshes have no pixels beyond delta 2, while its three subsequent translucent
image rectangles accumulate 3,691/max 3, 8,548/max 4, and 11,988/max 5.
Jellyfish therefore retains a measured image-rectangle dither-accumulation
precision gate rather than an image-mesh feature gate.

### Post-R3.1 Decoder Update (2026-07-15)

The production macOS CoreGraphics JPEG decoder closes the final three
`platform-image-decode-color-profile` rows under their unchanged `2/32`
contracts. Same-backend Metal comparisons against the committed references are
zero/max 0 for `riv-clipping_and_draw_order-frame-0-msaa`, zero pixels over
threshold/max 2 for `riv-data_binding_images_test-frame-0-clockwise-atomic`,
and zero/max 0 for `riv-data_binding_images_test-frame-0-msaa`.
`make renderer-decoder-oracle` independently reports zero decode delta for the
reachable JPEG. The active ratchet is now exact=1,405/diverges=0/gated=63;
the retained reviewed-boundary set falls from 61 to 58. The 91-row taxonomy
above remains the historical R3 exit snapshot.

### R3.1 Repeated Singleton Promotions (2026-07-15)

An independent Sol review rendered four fresh Rust wgpu/Metal rounds for each
of `gm-dstreadshuffle-msaa`, `riv-jellyfish_test-frame-0-msaa`, and
`gm-strokes_poly-msaa`. Every row was byte-stable across those fresh rounds.
Against the unchanged Dawn references and `2/32` contracts, the comparisons
are 0 pixels/max delta 1 at 530x690, 0/max 1 at 2080x2080, and 12/max 46 at
400x400 respectively. Stream hashes, reference hashes, provenance identities,
dimensions, and unique reference ownership all remain valid. Exactly these
three rows advance from gated to exact; the active ratchet is
exact=1,408/diverges=0/gated=60.

## Reproduction

```sh
make renderer-golden RENDERER_JOBS=4
make golden-compare scripted-golden-compare
cargo test --workspace
rg 'gated = "algorithm-core"' corpus-r.toml
```

The final command must produce no output. Gate counts can be regenerated from
the `[[entry]]` blocks in `corpus-r.toml`; every currently gated block must
contain a nonempty `gated` field. The 91-row table above is the historical R3
exit snapshot, not a current-manifest invariant.
