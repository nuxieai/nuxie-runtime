# Content-derived image outer bounds

The compiler now checks the final content-plus-padding addition for image owners whose emitted axis is Auto/Hug. Previously it checked each inset, their sum and the inner image ratio separately, leaving their final addition unchecked. This is a prerequisite for percentage-padding support; public percentage-padding admission remains closed.

The check uses the actual sizes, parent direction and stretch decision passed to the ordinary layout emitter. It resolves every percentage inset against the original containing width, rounds bounds outward and rejects a nonfinite sum. Fixed and cross-axis Fill dimensions keep their existing bounds path. Inner content bounds remain separate.

Four new numeric tests cover the documented seven-level percentage chain with a 1:8192 image, a finite counterpart, actual Hug versus fixed/Fill axes, and missing content/containing-width bounds. In the overflow witness each preceding native binary32 component is finite while the final outer sum is infinite. These are compile-side numerical tests; no extreme scene is rendered, and they do not establish browser equivalence for percentage padding.

Validation: all 315 Rust tests pass. The new native CLI reproduces all 100 successful image files and source maps from the six final point-padding corpora byte-for-byte. See `output/image-outer-bound-regression-r1/receipt.json` and `results/manifest.json`. The immutable source guard and diff whitespace check pass. No runtime, renderer, schema or dependency changes. No new WASM or native-pixel qualification is claimed; the earlier frozen checkpoint belongs to its earlier source identity.

Remaining percentage-padding work includes shared emitted sizing/stretch for content-box lowering, missing acceptance contexts and unresolved row intrinsic measurement. The private generator and its completed visual review do not constitute public implementation.
