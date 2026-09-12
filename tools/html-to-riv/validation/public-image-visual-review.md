# Public image candidate visual review

The frozen public image candidate matches Chrome's measured geometry and image presence in **216/216 frames**. **198/216 frames pass the existing pixel gates**; all 18 failures remain visible in the gallery and receipts. The four previously broken automatic stretch cases now pass geometry in all 32 affected original/clone frames. This is visual evidence for specific emitted files, not completed public API or general codec qualification.

[Open the gallery](../output/public-image-layout-r2/visual/gallery.html). It includes measurements, exact artifact links, every failed frame, and the four Chrome / previous native / repaired native comparisons. [Coverage](../output/public-image-layout-r2/visual/coverage.json) records the full joins and transfer proofs. The [review receipt](../output/public-image-layout-r2/visual/review-receipt.json) has `visualReviewCompleted: true`; SHA-256 `8ffc18a27ca92d18f3591beb3ca41f826f7b1ef0ea3e3414b4f914b7c9c73212`.

## What was inspected

All **45 representative Chrome/native pairs** were directly inspected on 12 unscaled PNG sheets, plus a sheet containing four before/after triples. Each source image is copied to its sheet at native pixel dimensions. The other **171 frames** have checked, complete RGBA transfers from an inspected representative: 99 compare equal-sized canvases; 72 extend the smaller canvas with opaque white before exact comparison. These transfers require the same authored request, scene bytes, map, browser source, observed geometry and image metadata. They include original-instance restores and cloned-instance resize steps.

The opaque-white proof concerns the entire visible canvas, including alpha. It does not infer offscreen content. Each transferred frame keeps its own viewport, pixel metrics and failure result. The review does not use a crop, perceptual hash or tolerant comparison to establish transfer equality.

Visual coverage includes all seven authored format fixtures (opaque/alpha PNG, baseline/progressive JPEG, lossless/lossy/alpha WebP); intrinsic and responsive image sizes; both flex axes and reverse ordering; fixed-main stretch and nonstretch controls; contain, cover, none and scale-down; alignment; transparency, layering, exact keys and duplicate-byte aliases. The inspected pairs show matching source content, corners, clipping, layer order and major placement. Fractional paint differences described below remain.

## Retained failures

| Case | Failed frames | Preserved finding |
| --- | --- | --- |
| `column-auto-stretch` | 0–7 | Tail local RGB error, approximately 14.7284. Geometry is within the 0.1px gate, but the tail edges paint differently. |
| `fractional-linear` | 0, 3, 4, 7 | Global mismatch ratio; 470 mismatched pixels at each viewport. |
| `fractional-nearest` | 0, 3, 4, 7 | Global mismatch ratio; 589 mismatched pixels at each viewport. |
| `contain-quarter-fractional-alignment` | 1, 5 | Global mismatch ratio; 814 mismatched pixels at the middle viewport. |

For the column case, Chrome places the image bottom and tail at `133.328125`, while native reports `133.33333`. The major stretch repair is clear in the before/after sheet: the old native image remained 64px high. The remaining failure is observable in actual PNG pixels, not merely geometry serialization. At `(4,133)`, Chrome is `(23,33,43,255)` while native is `(96,90,88,255)`; at `(4,141)`, Chrome is white while native is `(178,182,185,255)`. Interior tail rows 134–140 match. [The bound edge check](../output/public-image-layout-r2/visual/asset-and-edge-check.json) retains these samples.

The linear and nearest fractional fixtures retain their 470 and 589 mismatched pixels at all three sizes. Their wider-viewport gates pass because the whole-canvas mismatch denominator grows; the underlying paint difference has not disappeared. The quarter-alignment case fails at a position that produces fractional image placement. No tolerance was changed and no failed frame was removed.

## Source and tool bindings

The [combined receipt](../output/public-image-layout-r2/combined-receipt.json) combines **32 fresh rendered frames** with **184 transferred frames from unchanged emitted scenes**. Those scene transfers are separate from this review's 171 repeated-frame visual transfers. The review independently checks every transferred result and receipt hash, complete parsed request equality, exact Rive/map bytes, reference HTML/reset construction, geometry/stream/PNG hashes, baseline frame instance/step identities, and authored source-ID observations.

Embedded `FileAssetContents` bytes and recorded `decodeImage` payloads match the exact unique supplied asset bytes. A supplemental check verifies all 27 browser asset tables and actual request logs against the authored byte hashes, including the fixture-specific URL namespace. The initial r1 browser-reference mistakes remain historical evidence; the review uses corrected `render-reference-r2` rows when transferring r1 scene output.

The frozen compiler SHA-256 is `96c66b6d0a78cb4b751fdca5cbcb011a291a93980074401bd5d1efe6d87d8cc5`. Chrome is `153.0.8010.12`. The immutable probe is `7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a`; renderer replay is `276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f`. Build snapshots are used when live compiler work has advanced. This review changed no production, runtime, renderer, native driver, comparison gate or fixture, and launched no native rerender.

The reusable [review script](public-image-visual.py) prepares the gallery and coverage, then seals the review only after all listed sheets have explicit inspection observations. Its preparation and final-sealing versions are frozen beside the gallery. The final seal rechecks all recorded artifact hashes. The full image Rust/CLI/WASM/JS contract, resource limits, diagnostic corpus and general codec-profile coverage remain separate delivery gates.
