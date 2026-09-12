# Actual-record flex bridge validation

The private leaf-group numerical bridge passes the new native geometry controls. It does **not** establish public grow/shrink support: 26 of 96 Chrome/native pixel frames still fail the existing gates. Failures and tolerances are preserved.

## Reproduce and inspect

From the repository root:

```sh
python3 tools/html-to-riv/validation/check-flex-proof-bridge.py NEW_OUTPUT --render
```

The driver requires a fresh output directory. It copies the actual compiler source, builds `compile_profile_with_descriptors(..., Candidate)` and `flex_proof::analyze`, and retains source hashes, Cargo manifest/lock hashes, all 54 resolved Rust library/proc-macro artifact hashes, direct dependency bindings, compiler version, harness source/binary hashes and the exact invoked driver. It never reconstructs proofs from serialized diagnostic metadata. The baseline observer receives only the emitted ordinary `.riv` and viewport sizes; descriptors and source maps are used afterward for inspection/joining.

The current driver exits **1**, deliberately, after recording the preserved pixel failures. The original run used a driver that returned 0 when numerical checks passed despite the optional render command returning 1. That invocation is preserved as `output/flex-proof-bridge-r2/invoked-driver.py` with its original hash. The corrected driver was rerun completely under `output/flex-proof-bridge-r2/aggregate-driver-check`; it emitted the same observations and now reports the combined failure correctly. Neither run changed compiler sources or runtime sources. Their 27 source file hashes agree.

- [Compact tracked receipt](flex-proof-bridge-receipt.json)
- [New fixtures](flex-proof-bridge-cases.json)
- [Native classifications and comparisons](../output/flex-proof-bridge-r2/native-results.json)
- [Historical 48-case classification](../output/flex-proof-bridge-r2/historical-classifications.json)
- [Chrome/native gallery, including failed frames](../output/flex-proof-bridge-r2/render/gallery.html)
- [Full-resolution visual sheets and bindings](../output/flex-proof-bridge-r2/visual-sheets/reviewed-manifest.json)
- [Corrected driver run receipt](../output/flex-proof-bridge-r2/aggregate-driver-check/receipt.json)

The shared rendering driver has historical `public-baseline` labels in its output. Here its compiler executable is the private source-backed Candidate harness. Those labels do not imply public admission.

## Native proof observations

All 22 new fixtures compile and import on the immutable baseline. The positive family comprises all four CSS directions with CSS order, nested rigid percentage parent sizing, paint-only leaves, fractional equal grow/shrink factors and point bases, exact-zero bases with independent authored factors, decimal literals, and cross overflow. Each positive case has three children including a fixed sibling.

The whole viewport domain is `[0, 16384]` with input error `0.001` on each axis, root world origins zero and geometry budget `0.125`. Exactly 12 target groups are bounded. Four min/max controls remain `PossibleFreeze`, four nested-content controls remain `KnownTargetPreserved`, and two large ancestor-offset controls remain `GeometryBudget`. Separate large-root-origin uncertainty controls are also retained in each classification. No descendants were removed to make a group qualify.

Each file is compiled once and observed through original and clone at `400×200 → 100×80 → 1×1 → 16384×16384 → 400×200`. The harness also analyzes exact per-viewport domains. Native JSON numbers are restored to their actual f32 values before comparison; Python exact fractions compare each value against its f64 interval endpoints plus/minus error.

- 2,160 whole-domain and 2,232 exact-domain scalar comparisons pass: **4,392 total**, covering native sizes, native world origins, and derived far edges.
- The 72 additional exact-domain comparisons come from row/row-reverse freeze controls at a viewport where the analyzer proves no freeze. These point observations do not qualify the whole viewport range.
- All 22 independent repeat compiles produce identical RIV bytes and source maps.
- All 132 native geometry repetitions across clone and return-to-viewport observations are identical.

Far edges are calculated as `f32(native origin + native size)` using the audited translation-only model. They are not independently observed native corners or rendered pixel boundaries. Successful member checks also do not qualify an entire scene: ancestor domains establish inputs, while non-leaf groups remain unresolved.

The existing 48 private flex fixtures all compile without changing their HTML or dropping descendants. All 192 whole-domain groups remain unresolved: 96 `ParentSize` and 96 `KnownTargetPreserved`. This is honest coverage of the present leaf-only proof boundary, not proof that the historical compositions are unsupported by the runtime.

## Chrome/native pixels and complete visual review

Pinned Chrome `153.0.8010.12` was compared with the immutable native Rust Metal RasterOrdering renderer (`clockwise-atomic` CLI token). The twelve positive fixtures were observed at `400×200 → 100×80 → 600×320 → 400×200`, original and clone, for 96 frames. The 16384 and 1-pixel native geometry controls were not presented as pixel comparisons.

All 96 geometry frames pass the existing 0.1-pixel gate. The largest observed DOM/native geometry difference is approximately **0.009385 px**, on `b.x` in the row-reverse decimal fixture at 400×200. All 192 alternate-clear comparisons pass. Pixel gates pass **70/96 frames**, representing **24/36 distinct viewport pairs**.

All 36 distinct Chrome/native pairs were directly viewed in twelve full-resolution sheets. The 72 source images were copied without scaling, and every displayed source rectangle was verified byte-for-byte in RGB. All sources are opaque. Another 60 repeated frame pairs were bound to reviewed images using **120 exact full-RGBA image comparisons**, covering original/clone and return sizes. The complete corrected-driver rerun matched the original render in **192 additional full-RGBA image comparisons**. These image repetitions add lifecycle evidence, not independent fixture coverage.

| Fixture | 400×200 pixels | 100×80 pixels | 600×320 pixels |
| --- | --- | --- | --- |
| row / subunit point | pass | fail | pass |
| row / zero independent | pass | fail | pass |
| row / decimal point | pass | fail | pass |
| row-reverse / subunit point | fail | fail | pass |
| row-reverse / zero independent | pass | fail | pass |
| row-reverse / decimal point | pass | fail | pass |
| column / subunit point | pass | fail | pass |
| column / zero independent | pass | fail | pass |
| column / decimal point | pass | fail | pass |
| column-reverse / subunit point | pass | pass | pass |
| column-reverse / zero independent | pass | fail | pass |
| column-reverse / decimal point | pass | fail | pass |

Across the viewed sheets, child ordering, remaining main-axis space, painted parent backgrounds and the deliberate cross overflow follow the expected composition. The small viewport makes fractional edges prominent. The decimal column cases produce a very thin salmon-colored child at 100×80; its local RGB gate fails even while the geometry gate passes. Other failing pairs show narrow horizontal or vertical edge differences. Full diff images were additionally viewed for three failure examples and one passing frame. These observations do not prove that every failed pixel has the same antialiasing cause, and they do not override a failed gate.

Next qualification work should isolate the fractional-boundary and very-thin-paint failures using fixed native box controls, separating CSS used-size rounding from native coverage. The current evidence permits continued compiler-side investigation and keeps public flex admission unchanged. No runtime or renderer change, widened tolerance, raster substitute, browser-baked layout or resize recompilation was used.
