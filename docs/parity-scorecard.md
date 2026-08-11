# Parity scorecard

## C++ → Rust file correspondence

Files: 456
Status counts: `divergent-by-decision`: 4; `faithful`: 450; `partial`: 2
Named pending files: 0

## Rust → C++ attribution

Ledger coverage: 487 Rust files (457 attributed by manifest inversion; 30 classified additions)
Addition categories: `baseline-adaptation`: 8; `codegen`: 17; `product-data`: 1; `retained-render`: 1; `test-infra`: 3

## Test correspondence

Files: 157
Test cases: 1404
Status counts: `n-a`: 2; `partial`: 29; `pending`: 78; `ported-differential`: 5; `ported-direct`: 43

## Silver corpus

Entries: 252
Status counts: `divergent`: 92; `exact`: 87; `pending-scripted`: 41; `provenance-unknown`: 3; `unsupported`: 29
Exact ratchet: 87/87 (met)

## Golden corpus

Entries: 364

## Runtime frame-loop ledger

Files: 354 (`divergent-by-decision`: 1; `faithful`: 353)
Members: 35 (`adapted`: 4; `faithful`: 31)
Gaps: 10 (`closed`: 10; `open`: 0)

## D-row register — approved divergences and adaptations

Rows: 14

- D1 — `f32::total_cmp` sort order vs C++ `operator<` on NaN/±0 (reproducibility over degenerate-input parity).
- D2 — Saturating float→int casts vs C++ UB; PingPong `duration==0` is the one constructible divergence.
- D3 — **Taffy, not Yoga** — edge-case layouts verify `tolerant`; fence: never pin Taffy behavior-by-behavior.
- D4 — luaur-rt pinned =0.1.8 as the scripting engine (mlua fallback untriggered).
- D5 — Rust image decoders vs platform decoders — JPEG color-profile rows resolvable only by CoreGraphics; dimension+tolerant-pixel verification, never payload hashes.
- D6 — Renderer fuzz-accepted findings R3-FZ-03/04/05 (area-capped, neither rasterization canonical).
- D7 — GPU integer semantics (unsigned-cast fixed-point limits; checked-sub vs deliberate wrap in row-wrap rebuild).
- D8 — Jellyfish dither-accumulation precision gate.
- D9 — `solar-system.riv` malformed-blendMode import rejection (`rejects-malformed`).
- D10 — 108 renderer rows contract-exact under the reviewed 2/32 Metal-vs-WebGPU subpixel budget (not byte-exact).
- D11 — **Bounded host decoded-image policy (2026-07-21).** The high-level `nuxie::File` import path caps the aggregate decoded RGBA bytes retained by one artboard-tree render cache at 64 MiB by default (`FileImportLimits::max_retained_decoded_image_bytes`); pinned C++ has no aggregate ceiling.
- D16 — **Pure-Rust profiler capture backend (user-approved P1-m decomposition question 4, 2026-08-01).** The pinned 16-line `src/profiler/profiler.cpp` MicroProfile wrapper is replaced by a pluggable Rust `ProfileCapture` trait, with no MicroProfile or C++ FFI dependency.
- D17 — **Symphonia audio decoder/resampler (Levi-approved P2-f decomposition question 5, 2026-08-01).** The pinned miniaudio memory decoder/channel converter/resampler is replaced by pure-Rust Symphonia decode plus the Rive-owned headless engine glue.
- D18 — **wgpu Lua GPU execution contract (Levi-approved GPUCEIL D-row, 2026-08-03).** The pinned ORE-backed objects in `src/lua/renderer/lua_gpu.cpp` are represented by Rust userdata, immutable backend-neutral submission snapshots, and retained wgpu resources.

## Additive host-extension register

Rows: 3

- X1 — **semantic-geometry-cache-authority.** An opaque, fail-closed equality token may invalidate editor semantic-geometry caches.
- X2 — **scripted-global-occurrence-broadcast.** A host facade may broadcast an input to all currently retained occurrences of an authored global id.
- X3 — **direct-gpu-bytecode-input-projection.** Editor-driven direct GPU bytecode programs may expose scalar input setters that reuse the exact C++ `ScriptedObject` table-write conversions.
