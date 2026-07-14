# Renderer R3 Exit Audit

Date: 2026-07-14

## Verdict

R3 corpus convergence is complete at `exact=1,353`, `diverges=0`,
`gated=115`, `total=1,468`. Every non-gated entry passes its committed
contract on the macOS Metal CI backend, and every retained gate has a specific
feature, backend/compiler boundary, or harness diagnostic. No
`algorithm-core` placeholder remains.

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
  `1,353/0/115`, normal V2 `584` exact segments, scripted V2 `35` exact
  segments, and `cargo test --workspace`.

## Retained Gate Taxonomy

| Count | Diagnostic |
| ---: | --- |
| 49 | `metal-webgpu-subpixel-edge-coverage` |
| 43 | `strict-replay-gradient-paint` |
| 7 | `native-clockwise-atomic-advanced-feather-parity` |
| 3 | `strict-replay-render-buffer` |
| 3 | `platform-image-decode-color-profile` |
| 2 | `metal-webgpu-atomic-intermediate-precision` |
| 1 | `dawn-wgpu-msaa-advanced-blend-intermediate-precision` |
| 1 | `dawn-wgpu-msaa-interleaved-feather-color-precision` |
| 1 | `dawn-wgpu-msaa-stroke-edge-coverage` |
| 1 | `incompatible-clip-rectangles` |
| 1 | `metal-webgpu-fixed-function-color-output` |
| 1 | `msaa-clip-intersection-edge-coverage` |
| 1 | `msaa-cubic-stroke-raster-parity` |
| 1 | `reference-harness: C++ Metal does not implement MSAA flush` |
| **115** | **Total** |

The final 43 generic placeholders were not runnable renderer failures: the
checked-in strict Dawn inventory proves that 41 require gradient-paint replay
reconstruction and two require render-buffer reconstruction. Together with the
two already named gradient rows and one already named render-buffer row, the
manifest now agrees exactly with
`tools/cpp-atlas-mask-oracle/msaa-reference-inventory.json`.

## Reproduction

```sh
make renderer-golden RENDERER_JOBS=4
make golden-compare scripted-golden-compare
cargo test --workspace
rg 'gated = "algorithm-core"' corpus-r.toml
```

The final command must produce no output. Gate counts can be regenerated from
the `[[entry]]` blocks in `corpus-r.toml`; they must total 115 and every gated
block must contain a nonempty `gated` field.
