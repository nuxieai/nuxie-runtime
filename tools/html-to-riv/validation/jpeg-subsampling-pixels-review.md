# Private JPEG subsampling visual review

The saved private corpus contains **24 scenes / 192 original-and-clone frames**. Geometry and image presence pass **192/192**; the existing pixel gates pass **176/192**. Visual inspection confirms the **16 retained failures**: `1x17-420-baseline` and `1x17-420-progressive` each differ from Chrome in all eight frames. The difference is color reconstruction within the correctly positioned image, not missing image rows or incorrect geometry.

The [gallery](../output/jpeg-subsampling-native-r1/visual/gallery.html) contains complete Chrome/native/diff sheets, magnified image-box comparisons and a [24× skinny close-up](../output/jpeg-subsampling-native-r1/visual/skinny-24x.png). The [completed review receipt](../output/jpeg-subsampling-native-r1/visual/review-receipt.json), SHA256 `09fddefd2c718d0d0354674ed98d2f41ba212566374ff57c63f945f52593e0f1`, binds every inspected sheet and its observation. Original results remain in [native-receipt.json](../output/jpeg-subsampling-native-r1/native-receipt.json); no results or tolerances were modified.

## Coverage and source identity

Each source size—96×64, 97×65, 31×47 and 1×17—has six encodings: PNG control, 4:4:4 baseline JPEG, and baseline/progressive JPEG for both 4:2:2 and 4:2:0. Each ordinary `.riv` is replayed at 240×240, 390×320, 768×560 and restored 240×240, for both original and cloned instances. Chrome is **153.0.8010.12**.

Inspection covers **72 distinct complete pairs** in 18 full sheets, all 24 distinct image-box pairs in four crop sheets, and one additional skinny close-up. All 23 sheets were opened with `view_image`. Another **120 frames** transfer only after proving the same case, exact source HTML, encoded asset, Rive file and viewport, plus equality of all three PNG byte strings and their complete decoded RGBA pixels. There is no cross-viewport or cross-asset transfer. Every frame retains its own recorded numeric verdict.

The [coverage manifest](../output/jpeg-subsampling-native-r1/visual/coverage.json) binds 1,348 source/frame artifacts. The builder and finalizer rechecked frozen tool/source bindings, generation hashes, exact embedded asset bytes, copied replay scene bytes, reference HTML including its data URL, request/case identity, per-frame result records, geometry/stream hashes, image dimensions and observed image fields. Each copied source asset was also compared byte-for-byte to its frozen build input. The complete input images remain unchanged.

Full sheets retain unscaled source pixels. The viewing tool automatically reduced the six largest 2336×2404 sheets to 1554×1600 despite the original-detail request; this is disclosed in the receipt. All source image-box pixels were additionally inspected through explicitly labeled nearest-neighbor magnification. A separate [full-background check](../output/jpeg-subsampling-native-r1/visual/full-background-verification.json) verifies all 144 representative Chrome/native canvases are exactly opaque white outside their integer image boxes, without relying on the reduced previews.

## Visual findings

- **PNG and 4:4:4 controls:** complete colored patterns, correct edges and placement, and close Chrome/native color agreement. No missing or duplicated image content appeared.
- **4:2:2 baseline/progressive:** all tested dimensions, including 1×17, retain close agreement. Wider source images show small chroma-boundary differences in the magnified comparisons, consistent with the existing numeric passes.
- **Wider 4:2:0 baseline/progressive:** correct geometry and intact patterns, but visible color differences around tile transitions. They pass the recorded gates; they are **not pixel-identical**. For each encoding mode, image-local mean RGB error is 2.928331 at 96×64, 1.646312 at 97×65 and 1.888813 at 31×47. The saved threshold masks mark 217 pixels at 96×64 and 22 at 31×47; those observations remain visible alongside their numeric pass.
- **1×17 4:2:0 baseline/progressive:** the close-up shows different transition colors/rows around the red-to-muted, middle blue/purple and lower orange bands. Both modes have image-local mean RGB error **9.686275**, while all 17 pixels remain present at the correct location. The same-source PNG, 4:4:4 and 4:2:2 controls pass. The whole-frame threshold mismatch count is zero for these narrow failures, demonstrating why the separate image-local RGB gate must remain authoritative.
- Across the complete frames, no additional clipping, displacement, missing paint or stray canvas content was identified. Resize/restore and clone repeats preserve the exact observed pictures under the stated transfer conditions.

The supplementary black-background difference images are absolute per-channel RGB differences multiplied by four, explicitly labeled. They help locate differences and do not replace the original diff images or numeric gates. Raw narrow-row Chrome/native RGBA values are retained in [skinny-24x.json](../output/jpeg-subsampling-native-r1/visual/skinny-24x.json).

## Qualification boundary

This is **private ordinary-file generator / immutable-runtime codec evidence**, not public HTML/CSS compiler admission, output, transport or layout qualification. No production, runtime, renderer, dependency, build configuration or shared progress document was changed, and no new native/browser render was run for this review. Entropy validation is covered separately in [jpeg-subsampling-review.md](jpeg-subsampling-review.md).

The skinny failures remain unresolved in this corpus. The parent is separately investigating additional widths; this review neither predicts those outcomes nor converts the wider cases into a universal 4:2:0 guarantee. Other artwork, dimensions, metadata, sampling layouts and public compiler compositions require their own evidence.
