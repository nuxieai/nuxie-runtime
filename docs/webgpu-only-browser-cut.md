# WebGPU-only browser renderer cut

## Objective

Make WebGPU the sole supported browser renderer. Remove the WebGL2/FemtoVG
implementation, its public API, automatic fallback, browser qualification
surface, and maintenance documentation. A browser without a usable WebGPU
adapter must receive an explicit initialization error.

This is an intentional product support-matrix reduction. It is not evidence
that the removed WebGL2 implementation reached C++ renderer parity.

## Interface

- Keep `BrowserFactory` as the canvas-presentation adapter around `WgpuFactory`.
- Change `BrowserFactory::new` to accept only `(canvas, width, height)`.
- Probe WebGPU Core and then Compatibility before constructing `WgpuFactory`.
- Return a `RendererError::Adapter` with actionable WebGPU context when the API
  or both adapter levels are unavailable.
- Remove `BrowserBackendPreference`, `BrowserBackend`, `fallback_reason`,
  backend-selection enums, and all public `WebGl2*` exports.
- Make `webgpu_adapter_info` return `&WgpuAdapterInfo`, not `Option`.
- Keep active-frame resize protection.
- Make ordinary `BrowserFrame::present` submit directly to the WebGPU canvas
  surface without GPU-to-CPU readback or Canvas2D presentation.
- Reserve `BrowserFrame::finish_with_readback` for explicit pixel capture. It
  returns exactly `width * height * 4` RGBA bytes and does not present.

## Removal surface

- Delete `crates/nuxie-renderer/src/webgl2.rs` and
  `crates/nuxie-renderer/src/webgl2_limits.rs`.
- Remove the wasm-only `femtovg`, `glow`, `rgb`, and `imgref` dependency chain
  when no remaining WebGPU code requires those crates.
- Remove WebGL2-only GPU-canvas translation/rendering code and tests while
  preserving the WebGPU GPU-canvas implementation and its tests.
- Simplify `tools/browser-renderer-smoke` to exercise WebGPU only, including
  Core/Compatibility admission, resize, presentation, stream replay,
  GPU-canvas, path clip, and explicit failure when WebGPU is unavailable.
- Remove stale HTML controls/query parameters for selecting or forcing WebGL2.
- Update re-exports, documentation, status/defect ledgers, and parity checks so
  the supported browser contract is unambiguously WebGPU-only.
- Retire the WebGL2 qualification fixtures as removed product surface, not
  fixed parity. Keep the linked defect rows open until Editor consumes this
  runtime landing and records its immutable product checkpoint.

## Constraints

- Do not touch `crates/nuxie-runtime`, `crates/nuxie-graph`, animation/state
  machine ownership, or the active frame-loop manifests.
- Do not change the native renderer or its C++-refereed pixel output.
- Do not loosen any pixel, workspace, WebGPU browser, size, or parity gate.
- Do not add a hidden fallback, feature-gated WebGL2 path, FemtoVG fork, or
  editor-side workaround.
- Preserve the pinned C++ oracle and checked-in feature provenance.

## Acceptance

1. `rg -n -i 'webgl2|femtovg|BrowserBackendPreference|BrowserBackend' crates tools makefile Cargo.lock`
   returns no live implementation/API/test references. Historical documentation
   may mention the retired backend only when clearly marked as removed.
2. `cargo test -p nuxie-renderer`
3. `cargo test --workspace`
4. `make browser-renderer-smoke`
   proves normal presentation calls neither `GPUBuffer.mapAsync` nor
   `CanvasRenderingContext2D.putImageData`, while explicit readback performs
   one mapped readback and returns exact RGBA pixels.
5. `make browser-renderer-gpu-smoke`
6. `make renderer-golden-same-runner` remains at the unchanged 1,468-row pixel
   floor.
7. `make golden-compare` and `make scripted-golden-compare` preserve their
   existing zero-failure floors.
8. `make size-report` remains below the 9 MiB native SDK ceiling and records
   that the browser-only deletion does not affect that native limit.

## Review

Review the final diff for stale fallback semantics, accidental WebGPU test
deletion, public API remnants, lockfile residue, and any edit inside the
reserved runtime/frame-loop owner surface.
