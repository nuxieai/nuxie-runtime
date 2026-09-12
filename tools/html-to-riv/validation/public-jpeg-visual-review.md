# Final public JPEG visual review

The final `public-jpeg-layout-r2` corpus has 56 authored cases: 34 compile and 22 diagnose. All **272 admitted original/clone frames pass geometry, original pixel, and image-presence gates** against Chrome 153.0.8010.12. This is finite, tolerance-based visual qualification; Chrome and native PNG bytes are not generally identical.

All 34 complete Chrome/native pairs were directly inspected on nine unscaled sheets (504×1168 maximum). Another 238 frames transfer only after exact full RGBA comparison with opaque-white canvas extension, plus identical request, scene, map, browser reference, browser measurements, native boxes and image-presence measurements. Each destination retains its own viewport and numeric gates. The sequence is 240×240 → 390×320 → 768×560 → 240×240 for both original and clone.

The final 34 request/scene/map tuples exactly match their earlier actual native evidence. This review generated no new native render. The public transfer receipt preserves that source join; the visual coverage independently verifies emitted asset bytes, bound asset snapshots, recorded native `decodeImage` payloads, source IDs, browser references, geometry/stream files and complete image hashes. Custom `--cases` authoring assets are bound independently of the reused compiler build.

Placement, image extent and the following sibling agree throughout the reviewed set. Sixteen tiny sources were additionally inspected in four 8× nearest-neighbor detail sheets. The diagonal scan patterns have visible small color differences, while their spatial structure remains aligned. The largest admitted image-region mean RGB error is 2.929742; the largest interior mean is 3.016243. The original thresholds and antialiasing treatment are unchanged.

The earlier candidate failure remains explicit. `single-chroma-2x1-420-baseline` passed geometry and presence but failed the local image-pixel gate in all eight R1 frames, with mean RGB error **7.5**. At frame zero its second pixel is Chrome `(135,119,94)` versus native `(107,133,96)`. Its complete pair and 20× detail were directly inspected and retained. The final compiler diagnoses the exact same authored request. The final profile also diagnoses other subsampled JPEGs at widths ≤4; this is an unqualified boundary, not proof that ordinary-runtime support is universally impossible. Existing image/aspect fractional failures remain separate preserved evidence.

The gallery has separate geometry, pixel and presence filters, full Chrome/native/diff links, per-frame metrics and transfer proofs, plus supplementary detail links. A pinned-Chrome check loaded all 68 preview images, verified all 213 local image/link targets, exercised the four filters and search, and reported no browser errors.

Evidence:

- [Gallery](../output/public-jpeg-layout-r2/visual/gallery.html)
- [Completed visual review receipt](../output/public-jpeg-layout-r2/visual/review-receipt.json), SHA256 `0cf793696ab77ea2c1b93538ee720851395632ef933f47f8498d719cdfff6a63`
- [Complete source/frame/image coverage](../output/public-jpeg-layout-r2/visual/coverage.json), SHA256 `75acc41b8f2569d88ed4d49a0258da0f76034dcd56fffe6897a727d2ecfc5bd2`
- [Tiny-pixel and retained-failure evidence](../output/public-jpeg-layout-r2/visual/details-receipt.json)
- [Gallery behavior check](../output/public-jpeg-layout-r2/visual/gallery-check-receipt.json)
- [Final native evidence join](../output/public-jpeg-layout-r2/combined-receipt.json)

`public-image-visual.py` now derives frame totals from compiled cases, accepts an optional explicit `--before-native` receipt, binds either full or reused build evidence and separate authoring assets, and snapshots preparation/completion helper versions. Preparation and completion snapshots for this run are separately bound. No prior receipts or images were rewritten. Optional before/after comparison was not used because all admitted scene bytes are unchanged. Complete white-extension equality proves visible-image transfer, not offscreen content or universal image-layout behavior.
