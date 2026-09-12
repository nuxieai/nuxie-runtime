# Public wrapping integration: qualification in progress

The public parser now computes flex-wrap and positional align-content, retains original numeric provenance, and routes admitted wrapping through the existing independently bound ordinary-file composition. The runtime is unchanged. This document records work in progress, not a qualification receipt.

## Admission under test

Exactly one transparent authored root, with direct empty leaf boxes. Root and children have definite fixed preferred dimensions and fixed minima, with optional fixed maxima. Source values must prove equal to their computed float before independent 1/64 normalization; original provenance is retained. All four flex directions and both wrapping modes route through the composition. Positional line alignment, positional child self alignment, CSS order, currentColor and transparent paints are covered by the proposed corpus.

Zero margins/padding/gaps and ordinary flex sizing are required. Percentages, auto dimensions/minima, normal/stretch line distribution, nested content, assets, root background painting and unproved numerical contexts diagnose. Valid unsupported CSS values must not silently recover as invalid substitutions. No browser geometry enters compilation.

## Evidence sequence

1. Focused library tests exercise retained provenance and alignment. Public Rust tests exercise compilation, deterministic bytes, stable DOM identities/order, explicit nowrap equivalence and unsupported contexts. Separate receiving-value tests cover variables and inheritance/reset semantics.
2. Freeze the complete current compiler and run all Rust, native build, WASM build, TypeScript and Node tests using public-value-build.py. The new Node corpus checks exact CLI/JavaScript-WASM bytes/maps and rejection without output artifacts.
3. Run public-wrapping-cases.json directly through the frozen CLI and unchanged check-wrapped-rounded-baseline.mjs, immutable native probe and renderer. Sixteen fixtures produce 128 original/clone frames. Compare geometry and native pixels with pinned Chrome under the compiler reset, plus alternate-clear controls. Root dimensions are fixed: viewport changes test clipping and persistence, not responsive reflow.
4. Inspect distinct browser/native/diff image pairs, preserving failures and unchanged tolerances. Transfer repeat reviews only through exact image identity. Check native build identity separately from the source guard.
5. Replay 1,076 previously accepted public inputs through the new frozen CLI and raw WASM, comparing prior scene bytes and source maps. Use public-accepted-regression.py with explicit fresh build and output directories.
6. Record actual results and remaining limitations before changing backlog evidence states or claiming public visual qualification. Preserve all failed build/capture runs.

The earlier private campaigns validate composition premises but cannot substitute for steps 2–5 through this public parser/planner path. Broader wrapping, responsive expressions and the remaining 99-item backlog stay in scope.

## Recorded results

Frozen build `output/public-wrapping-product-build-r1` passes 463 Rust tests, 57 Node tests, native/WASM builds, TypeScript and the source guard. CLI SHA256 `2cae98f4fbfb31cb9efe98feef9403255bfdb9155d36af509fc03bcdec8e673a`; WASM `5de17bb328d185fd93dd6fd26ec6cde57fadc4b1edf823006ed97958e8b5cacd`.

`output/public-wrapping-accepted-regression-r1/receipt.json` records all 1,076 prior inputs passing on both newly executed CLI and raw WASM: scene bytes remain exact, as do CLI map bytes and WASM map structures.

`output/playwright/public-wrapping-r2/receipt.json` records 128 geometry passes, 128 pixel passes and 256 alternate-clear passes. The unchanged driver retains historical “private-derived” labels in its receipt/gallery; its actual compiler input is the frozen public CLI, with no recipe adapter. The r1 capture stopped before its first pixel frame because the independent expected CSSOM alpha serialization was 0.502 rather than Chrome's 0.5. Only expectedPaint metadata changed for r2; all 16 authored HTML/CSS/viewports and other fixture fields were verified identical to the frozen build. The original expectation and failure remain preserved.

The historical private clip review helper rejected an unexpected mask rectangle in this graph. Its failed setup is preserved under r2/visual; it provides no stream qualification here. `public-wrapping-visual.py` separately prepares full-size sheets and verifies repeat image/geometry identity without invoking that observer. There are 48 representative pairs on 12 sheets and 80 exact repeat transfers. Direct-review notes are separate. No native stream-observer qualification is claimed from the failed helper.

Next: finish review coverage and evidence binding, verify effective baseline build identity, and address public resource/lifecycle boundaries and remaining admission combinations before updating backlog qualification states. Do not repeat the completed unchanged build, 128 captures or 1,076 regression inputs.
