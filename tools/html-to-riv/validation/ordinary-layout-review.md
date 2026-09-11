# First ordinary-file baseline experiments

The unchanged runtime was rebuilt from its original locked workspace, outside the compiler's isolated Cargo workspace. `build-baseline.py` records the effective runtime dependency/features tree, Cargo/rustc commands and source identity, then freezes the renderer and a read-only import/clone/resize probe. No runtime, renderer, schema, shared vendor or root build file changed.

The compiler-local ordinary-layout example emits baseline Rive records using the schema-aware wire writer. It is a capability experiment, not the rebuilt public HTML/CSS interface. No runtime requirements, source map, font modes or post-import CSS setters are supplied. The probe imports once, clones once and resizes both occurrences through 240×160 → 390×200 → 768×120 → 240×160.

| Ordinary file experiment | Geometry | Chrome/native pixels | Decision |
| --- | --- | --- | --- |
| Solid named-color layout fill | 8/8 | 8/8 | Existing direct fill works in this corpus |
| Fractional-width child on HSL background | 8/8 | 8/8 | Existing layout/fill passes current tolerances in this corpus; not an exact fractional-edge guarantee |
| Nested percentage-size fills | 8/8 | 8/8 | Existing parent/child painting works here; arbitrary sibling/stacking order remains open |
| Independent modest circular corners | 8/8 | 8/8 | Existing corner properties work for tested values |
| Large top corners, direct properties | 8/8 | 2/8 | Six preserved visual failures; direct native clamp differs from CSS |
| HSLA translucent fill on white | 8/8 | 8/8 | Ordinary alpha paint works here; not group opacity |
| Large top corners, clipped paint-child composition | 8/8 | 8/8 | Bounded file-level composition succeeds; full feature remains pending |

Chrome is pinned at 153.0.8010.12. Native output uses the baseline rust-metal path on Apple M5 Max; the baseline CLI requires a non-MSAA mode token, but rust-metal uses its ordinary native factory, not the separate forced-atomic replay function. No alternate-backend claim is made. Existing 0.1px geometry and unchanged whole-image/region/interior pixel thresholds apply; no tolerances were increased.

All 21 distinct image pairs were visually inspected at full resolution in paired sheets, including the visible failing corner profiles. The 56 final rows have exact source/RIV/geometry/full-PNG coverage across direct reviews, earlier-run and clone/repeat transfers. The frozen final probe reproduces all 56 prior geometry/recording frames exactly. The final example binary regenerates the exact seven tested RIV files.

## The corner limitation and composition

For `border-radius:100px 100px 0 0`, CSS can retain 100px top corners in a 240×160 box: neither horizontal nor vertical radius sums exceed the corresponding side. Baseline Rive independently clamps to half the smaller box dimension, producing 80px. At 768×120 it produces 60px. The native layout geometry still matches, so geometry-only testing misses this difference.

The composition experiment declares `min-height:100px` on the outer box. An ordinary LayoutComponent with a rectangular clip contains an ordinary paint LayoutComponent with percentage dimensions and `minHeight:200px`, the two 100px top corner properties and a fill. The outer clip removes the excess lower portion; existing layout computes dimensions on each resize. The file contains the whole arrangement. The intermediate paint object is not a DOM element, so read-only geometry compares the visible outer object to Chrome rather than treating that internal object as authored layout.

This proves the tested equal-top-corner arrangement only. Narrower widths, heights below the authored minimum, unequal corners, all four corners, percentage/elliptical radii, content inside the wrapper, borders and arbitrary nested constraints remain to test. It does not establish general CSS radius normalization or authorize accepting those cases. No runtime fix, custom shader, script, host callback, rasterization or recompilation is involved.

## Durable evidence

`ordinary-layout-receipt.json` pins the frozen toolchain, original/clone reproduction, all final native comparisons, visual coverage and source/RIV hashes. Local bundles are under `output/immutable-baseline-toolchain-r2`, `output/immutable-probe-r2` and `output/immutable-layout-native-r3`. The earlier r1/r2 images and receipts remain available, including the failing direct encoding.

Four compiler-only color tests pass, covering the named-color vocabulary, hex/RGB/HSL syntax and diagnostics. Public HTML/CSS admission, Rust/CLI/WASM/JS artifact parity and broader realistic compositions are not restored yet; no backlog item is marked qualified from these experiments.
