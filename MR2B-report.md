# MR-2b C05 split-wave report

## Scope

C05 reconciled the 21 actionable rows still attributed to `crates/nuxie-runtime/src/draw.rs` after both MR-2 pure-move trains. The five pre-existing planned exceptions (B6-0094, B6-0282, B6-0322, B6-0324, and B6-0383) were not reprocessed.

## Moved rows

- B6-0299 (`src/math/raw_path.cpp`) — RawPath conversion, reversal, and pruning moved to `math/raw_path.rs`. Internal text importers now address that module directly; the non-orphan `draw` re-export remains only to preserve the public export in foreign-owned `runtime/src/lib.rs`.
- B6-0331 (`src/shapes/clipping_shape.cpp`) — the complete ClippingShape update owner moved to `shapes/clipping_shape.rs`.
- B6-0352 (`src/shapes/paint/shape_paint.cpp`) — the complete clone-owned ShapePaint renderer moved to `shapes/paint/shape_paint.rs`.
- B6-0357 (`src/shapes/paint/stroke_effect.cpp`) — the complete effect dispatcher moved to `shapes/paint/stroke_effect.rs`; concrete effect implementations remain in their natural modules.
- B6-0359 (`src/shapes/paint/trim_path.cpp`) — property reads, dispatch, C++ modulo semantics, and sequential/synchronized trim bodies moved to `shapes/paint/trim_path.rs`; the contour-measure core remains in `draw.rs` because it is shared with PathMeasure.
- B6-0361 (`src/shapes/path.cpp`) — Path update, deferral, and layout-controlled geometry ownership moved to `shapes/path.rs`.
- B6-0362 (`src/shapes/path_composer.cpp`) — the complete PathComposer update owner moved to `shapes/path_composer.rs`.
- B6-0368 (`src/shapes/shape.cpp`) — the complete clone-owned Shape renderer moved to `shapes/shape.rs`, delegating paint rendering to B6-0352's owner.

All moves preserve existing function signatures and public API. Required cross-module visibility changes are crate-private only. Per-row ownership arrays already named the planned target modules; the live-draw source-set list now names every new runtime owner, while B6-0299 is narrowed to its canonical target in both ledgers. Attribution comments moved with the per-row bodies.

## Justified exceptions

The following rows remain split because their `draw.rs` implementation is part of a function or retained owner that genuinely serves the named sibling row. Each manifest row now has a one-line MR-2b/C05 justification naming that sibling; no forwarding API or signature change was introduced.

- B6-0095 — shared component-list mount/draw and B6-0258 layout settlement.
- B6-0203 — shared `RuntimeDrawableList` relink with B6-0204 and B6-0205.
- B6-0204 — shared `RuntimeDrawableList` relink with B6-0203 and B6-0205.
- B6-0205 — shared retained ordering/hit traversal with B6-0203 and B6-0204.
- B6-0210 — shared live layout draw with B6-0258.
- B6-0248 — shared Taffy tree build with B6-0095 and B6-0304.
- B6-0258 — shared Taffy build/solve/draw with B6-0210 and B6-0304.
- B6-0303 — shared nested recursion with B6-0304 and B6-0305.
- B6-0304 — shared nested recursion with B6-0303 and B6-0305.
- B6-0305 — shared nested recursion with B6-0303 and B6-0304.
- B6-0385 — shared retained text replay with B6-0390 and B6-0402.
- B6-0390 — shared retained text replay with B6-0385 and B6-0402.
- B6-0402 — shared retained text replay with B6-0385 and B6-0390.

## Queued rows

None. C05 did not modify the foreign-owned roots `crates/nuxie-binary/src/lib.rs`, `crates/nuxie-scripting/src/vm.rs`, `crates/nuxie-runtime/src/lib.rs`, or `crates/nuxie/src/lib.rs`.

## Verification

- `cargo check --workspace --exclude nux-capi`: passed for the eight-row move batch.
- `make runtime-frame-loop-port-check`: passed (108 unit tests plus correspondence and ledger checks).
- `make rust-attribution-check`: passed (10 unit tests and complete Rust-source attribution coverage).
- The FL-E7 `invalidate_stroke_effects` structural ratchet now scans `shapes/path_composer.rs` alongside `draw.rs`, preserving its exact 4/4 proof after the owner move.
- Batch commit created successfully with the required `[MR-2b/C05] Split … from draw.rs` subject.
