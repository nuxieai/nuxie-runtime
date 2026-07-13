# Wgpu Resource-Seam Adversarial Review

Date: 2026-07-13

This review closes the mid-R2 audit required by `docs/renderer-port-map.md`.
It covers the Rust-owned plumbing between typed render-stream replay and the
ported renderer algorithms. Pixel parity remains governed by
`make renderer-golden`; this document does not promote corpus entries or
change tolerances.

## Scope And Evidence

Reviewed Rust surfaces:

- `crates/nuxie-renderer/src/lib.rs`
- `crates/nuxie-renderer/src/*_pipeline.rs`
- `crates/nuxie-renderer/src/generated/*.wgsl`
- `tools/renderer-replay/src/main.rs`

Compared against the generated WGSL declarations and the C++ resource model
in `renderer/src/render_context.cpp`, especially `LogicalFlush::pushDraws`,
`LogicalFlush::rewind`, resource layout/write, and `RenderContext::flush`.

## Findings

| Area | Result | Evidence and disposition |
| --- | --- | --- |
| Bind-group lifecycles | Accepted | Every Rust layout agrees with the generated WGSL group, binding, and resource type. Prepared bind groups retain their buffers, textures, views, and samplers through pass encoding. Common layouts are duplicated across pipeline modules, which is a maintenance risk but not a parity defect. |
| Buffer reuse and rewind | Fixed / deferred | `WgpuBuffer::unmap` snapshots immutable GPU contents, and the regression test proves later maps cannot mutate submitted draws. Atomic disjoint groups now split before exhausting 16-bit path IDs. C++-equivalent logical-flush rollover for clip-dependent runs remains an R3 resource-budget task. Frame-local GPU allocation and the lack of a reusable upload ring are R4 performance measurements. |
| Readback synchronization | Accepted | Final rendering submits before `map_async`, blocks with `device.poll(wait_indefinitely)`, validates map completion, strips 256-byte row padding, and unmaps. Intermediate atomic groups submit and wait before reusing targets. Mipmap submissions share the same ordered queue. No stale-resource or missing-wait path was found. |
| Pipeline caching | Deferred | Pipelines and layouts are created once per `WgpuFactory` and shared by its frames through `Arc<Context>`. All render-pipeline descriptors use `cache: None`; cross-factory cache persistence is an R4 startup/performance question, not a correctness gap. |
| Stream replay glue | Fixed / deferred | Render-target and decoded-image extents are checked against the selected adapter/device limit before texture creation, eliminating stream-triggered validation failures. Replay propagates factory/frame errors. Oversized buffers, non-finite numeric payloads, and device-loss behavior remain explicit R3 fuzz-replay inputs. |

## Correctness Changes

1. Zero-sized or over-limit frame textures now return
   `RendererError::InvalidTextureExtent` before wgpu texture creation.
2. Decoded images above the active device limit retain their dimensions but
   have no GPU texture, matching the existing failed-image behavior without a
   validation panic. The 2,080-pixel corpus regression still proves the real
   adapter limit is used rather than a fixed 2,048 cap.
3. Generic disjoint atomic groups preserve draw order while splitting at
   65,535 paths. Other oversized atomic runs return a named `Unsupported`
   error instead of panicking on `u16` conversion.

## Accepted Boundaries

- **R3:** Port C++ logical-flush rollover for clip-dependent or otherwise
  inseparable runs, including the complete path, contour, and tessellation
  resource budget from `LogicalFlush::pushDraws`.
- **R3 entry gate:** Fuzz replay must cover zero/oversized frames, oversized
  images and buffers, NaN/Inf transforms and paint values, deep clip stacks,
  path-count rollover, map errors, and device-loss reporting. Streams must not
  panic, hang, or silently lose the device.
- **R4:** Measure the per-group submit/wait policy, immutable buffer snapshots,
  frame-local dummy resources, upload-buffer reuse, and cross-factory pipeline
  cache value before optimizing them.

## Verification Contract

- `cargo test -p nuxie-renderer`
- `cargo test --workspace`
- `make renderer-golden`
- `make golden-compare`
- `make scripted-golden-compare`
- `cargo fmt --all -- --check`
- `git diff --check`

The renderer metric must remain at least `exact=153`, `diverges=0`,
`gated=1,314`, `total=1,467`. This review is complete only when all commands
pass with no `.riv` or V2 regression.
