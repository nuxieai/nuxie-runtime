# Parity scorecard

## C++ → Rust file correspondence

Files: 448
Status counts: `divergent-by-decision`: 2; `faithful`: 408; `pending`: 38
Named pending files: 38

### assets-importers

- `src/assets/audio_asset.cpp`
- `src/assets/script_asset.cpp`

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
- `src/factory.cpp`
- `src/file.cpp`
- `src/focus_data.cpp`
- `src/renderer.cpp`
- `src/simple_array.cpp`

### scripted

- `src/scripted/scripted_data_converter.cpp`
- `src/scripted/scripted_interpolator.cpp`
- `src/scripted/scripted_layout.cpp`

### unavailable

- `src/audio/audio_engine.cpp`
- `src/audio/audio_reader.cpp`
- `src/audio/audio_sound.cpp`
- `src/audio/audio_source.cpp`
- `src/audio_event.cpp`
- `src/command_queue.cpp`
- `src/input/gamepad_batch.cpp`
- `src/lua/lua_audio.cpp`
- `src/lua/lua_image_decode.cpp`
- `src/lua/lua_scripted_context.cpp`
- `src/lua/renderer/lua_blob.cpp`
- `src/lua/renderer/lua_gpu.cpp`
- `src/lua/renderer/lua_image.cpp`
- `src/lua/renderer/lua_mesh.cpp`
- `src/semantic/semantic_data.cpp`
- `src/semantic/semantic_inference_registry.cpp`
- `src/semantic/semantic_manager.cpp`
- `src/semantic/semantic_provider.cpp`

## Rust → C++ attribution

Ledger coverage: 382 Rust files (353 attributed by manifest inversion; 29 classified additions)
Addition categories: `codegen`: 16; `flowsession-abi`: 7; `retained-render`: 3; `scene-api`: 2; `test-infra`: 1

## Test correspondence

Files: 148
Test cases: 1316
Status counts: `n-a`: 2; `partial`: 17; `pending`: 92; `ported-differential`: 4; `ported-direct`: 33

## Silver corpus

Entries: 238
Status counts: `divergent`: 93; `exact`: 76; `pending-scripted`: 41; `provenance-unknown`: 2; `unsupported`: 26
Exact ratchet: 76/76 (met)

## Golden corpus

Entries: 320

## Runtime frame-loop ledger

Files: 342 (`divergent-by-decision`: 1; `faithful`: 341)
Members: 34 (`adapted`: 4; `faithful`: 30)
Gaps: 10 (`closed`: 10; `open`: 0)

## D-row register — approved divergences and adaptations

Rows: 12

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
