# Intrinsic first-baseline metrics

The compiler extends the existing first-baseline composition to empty automatic-height boxes and automatic-height normal columns whose descendants have provable fixed used heights. This applies to horizontal LTR single-line row and row-reverse baseline groups. L03 remains partial.

Empty intrinsic height is zero. A column sums descendant used heights, applies fixed min/max bounds (minimum wins), and propagates its first order-modified child's baseline. Nested intrinsic columns recurse. The baseline must remain inside the used box. Summation must be finite and no greater than 1,000,000px before clamping. Fixed-height columns continue to require only the first child's baseline metric. Ordinary measurement objects and constraints are unchanged; no runtime or renderer modifications are involved.

Explicit auto minima, percentage bounds, unresolved descendant metrics, nested row/reversed-column topology, out-of-box baselines and last baseline remain diagnostic. This does not qualify arbitrary automatic or responsive layouts. Existing fractional pixel failures remain preserved in the earlier first-baseline evidence.

## Evidence

The 16-scene public corpus covers both row directions, empty auto boxes, fixed minima/maxima, nested automatic columns, ordering, zero first children and a following footer. All 128 original/clone resize frames pass geometry and pixel gates; all 256 clear controls pass. Chrome is pinned to 153.0.8010.12 and the native backend is immutable rust-metal RasterOrdering. Tolerances are unchanged.

All 16 frame-1 Chrome/native pairs were directly inspected on four contact sheets; no divergence was observed. Exact decoded RGBA crop/white-extension checks independently cover Chrome and native for the other 112 frames. The frozen compiler regenerates all 16 files and source maps exactly. All 288 prior unique corpus requests retain identical bytes and parsed maps; this is output regression evidence, not a fresh rendering claim for those scenes.

The locked Rust suite passes 87 tests; Node passes 24 tests including intrinsic CLI/WASM parity and rejected-context parity. TypeScript and the immutable source guard pass. A separate read-only review found no actionable correctness defects. See public-baseline-intrinsic-receipt.json for evidence bindings.

Last-baseline admission was added in the subsequent checkpoint; see last-baseline-review.md. Restrictions above describe the intrinsic first-baseline checkpoint.
