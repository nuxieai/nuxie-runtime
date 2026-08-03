# Parity scorecard

## C++ → Rust file correspondence

Files: 448
Status counts: `divergent-by-decision`: 4; `faithful`: 425; `pending`: 19
Named pending files: 19

### focus-input

- `src/input/focus_manager.cpp`
- `src/input/focus_node.cpp`
- `src/input/focusable.cpp`

### lua-scripting

- `src/lua/lua_data_context.cpp`
- `src/lua/lua_data_value.cpp`
- `src/lua/lua_properties.cpp`
- `src/lua/lua_state.cpp`
- `src/lua/renderer/lua_gradient.cpp`
- `src/lua/rive_lua_libs.cpp`

### misc-core

- `src/core.cpp`
- `src/focus_data.cpp`

### scripted

- `src/scripted/scripted_interpolator.cpp`

### unavailable

- `src/command_queue.cpp`
- `src/lua/lua_scripted_context.cpp`
- `src/lua/renderer/lua_gpu.cpp`
- `src/semantic/semantic_data.cpp`
- `src/semantic/semantic_inference_registry.cpp`
- `src/semantic/semantic_manager.cpp`
- `src/semantic/semantic_provider.cpp`

## Rust → C++ attribution

Ledger coverage: 442 Rust files (411 attributed by manifest inversion; 31 classified additions)
Addition categories: `codegen`: 18; `flowsession-abi`: 7; `retained-render`: 2; `scene-api`: 3; `test-infra`: 1

## Test correspondence

Files: 148
Test cases: 1316
Status counts: `n-a`: 2; `partial`: 21; `pending`: 83; `ported-differential`: 5; `ported-direct`: 37

## Silver corpus

Entries: 238
Status counts: `divergent`: 94; `exact`: 76; `pending-scripted`: 41; `provenance-unknown`: 2; `unsupported`: 25
Exact ratchet: 76/76 (met)

## Golden corpus

Entries: 321

## Runtime frame-loop ledger

Files: 342 (`divergent-by-decision`: 1; `faithful`: 341)
Members: 34 (`adapted`: 4; `faithful`: 30)
Gaps: 10 (`closed`: 10; `open`: 0)

## D-row register — approved divergences and adaptations

Rows: 13

- D1 — `f32::total_cmp` sort order vs C++ `operator<` on NaN/±0 (reproducibility over degenerate-input parity).
- D2 — Saturating float→int casts vs C++ UB; PingPong `duration==0` is the one constructible divergence.
- D3 — **Taffy, not Yoga** — edge-case layouts verify `tolerant`; fence: never pin Taffy behavior-by-behavior.
- D4 — luaur-rt pinned =0.1.8 as the scripting engine (mlua fallback untriggered); Luau engine-version skew is a standing WATCH (`deferred-2026-07-19-luau-engine`).
- D5 — Rust image decoders vs platform decoders — JPEG color-profile rows resolvable only by CoreGraphics; dimension+tolerant-pixel verification, never payload hashes.
- D6 — Renderer fuzz-accepted findings R3-FZ-03/04/05 (area-capped, neither rasterization canonical).
- D7 — GPU integer semantics (unsigned-cast fixed-point limits; checked-sub vs deliberate wrap in row-wrap rebuild).
- D8 — Jellyfish dither-accumulation precision gate.
- D9 — `solar-system.riv` malformed-blendMode import rejection (`rejects-malformed`).
- D10 — 108 renderer rows contract-exact under the reviewed 2/32 Metal-vs-WebGPU subpixel budget (not byte-exact).
- D11 — **Bounded host decoded-image policy (2026-07-21).** The high-level `nuxie::File` import path caps the aggregate decoded RGBA bytes retained by one artboard-tree render cache at 64 MiB by default (`FileImportLimits::max_retained_decoded_image_bytes`); pinned C++ has no aggregate ceiling.
- D16 — **Pure-Rust profiler capture backend (user-approved P1-m decomposition question 4, 2026-08-01).** The pinned 16-line `src/profiler/profiler.cpp` MicroProfile wrapper is replaced by a pluggable Rust `ProfileCapture` trait, with no MicroProfile or C++ FFI dependency.
- D17 — **Symphonia audio decoder/resampler (Levi-approved P2-f decomposition question 5, 2026-08-01).** The pinned miniaudio memory decoder/channel converter/resampler is replaced by pure-Rust Symphonia decode plus the Rive-owned headless engine glue.
