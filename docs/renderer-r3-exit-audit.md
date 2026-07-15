# Renderer R3 Exit Audit

Date: 2026-07-14

## Verdict

R3 corpus convergence is complete at `exact=1,358`, `diverges=0`,
`gated=110`, `total=1,468`. Every non-gated entry passes its committed
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
  `1,358/0/110`, normal V2 `584` exact segments, scripted V2 `35` exact
  segments, and `cargo test --workspace`.

## Retained Gate Taxonomy

| Count | Diagnostic |
| ---: | --- |
| 50 | `metal-webgpu-subpixel-edge-coverage` |
| 37 | `rust-wgpu-msaa-gradient-path` |
| 5 | `native-clockwise-atomic-advanced-feather-parity` |
| 5 | `rust-wgpu-msaa-feather-gradient-advanced-blend` |
| 3 | `rust-wgpu-msaa-image-mesh` |
| 3 | `platform-image-decode-color-profile` |
| 2 | `metal-webgpu-atomic-intermediate-precision` |
| 1 | `dawn-wgpu-msaa-advanced-blend-intermediate-precision` |
| 1 | `dawn-wgpu-msaa-interleaved-feather-color-precision` |
| 1 | `dawn-wgpu-msaa-stroke-edge-coverage` |
| 1 | `metal-webgpu-fixed-function-color-output` |
| 1 | `reference-harness: C++ Metal does not implement MSAA flush` |
| **110** | **Total** |

Operationally, these rows collapse into three groups:

| Rows | Disposition |
| ---: | --- |
| 0 | Reference/oracle harness gap |
| 60 | Reviewed backend, decoder, or precision boundary |
| 50 | Unsupported feature or remaining algorithm-parity boundary |

The actionable set is 50 rows: five prior clockwise-atomic feather boundaries
plus 45 concrete renderer findings exposed by strict replay. The other 60 rows
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
rows into the three executable renderer diagnostics above. The manifest and
`tools/cpp-atlas-mask-oracle/msaa-reference-inventory.json` now contain no
actionable strict-replay placeholder.

## Reproduction

```sh
make renderer-golden RENDERER_JOBS=4
make golden-compare scripted-golden-compare
cargo test --workspace
rg 'gated = "algorithm-core"' corpus-r.toml
```

The final command must produce no output. Gate counts can be regenerated from
the `[[entry]]` blocks in `corpus-r.toml`; they must total 110 and every gated
block must contain a nonempty `gated` field.
