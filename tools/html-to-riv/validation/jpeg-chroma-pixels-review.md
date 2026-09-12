# JPEG chroma boundary visual supplement

These two private corpora retain **152/152 geometry and image-presence passes, 80/152 pixel passes, and 72 pixel failures**. All failures are image-local RGB errors. Direct inspection confirms color differences inside correctly placed image bounds; no missing extent, displacement, clipping, or stray canvas paint was identified.

This review uses the saved ordinary-file generator outputs with the same authored asset bytes in Chrome and the immutable native renderer. It does **not** qualify public compiler admission, transports, arbitrary JPEG artwork, or a universal dimension-based support rule. No rendering, fixture regeneration, production changes, or gate changes were performed for this supplement.

| Saved corpus | Cases | Frames | Distinct complete pairs | Exact repeat transfers | Geometry / presence | Pixel pass / fail | Inspected sheets |
| --- | ---: | ---: | ---: | ---: | --- | --- | ---: |
| [Chroma width sweep](../output/jpeg-chroma-boundary-r1/visual/gallery.html) | 8 | 64 | 24 | 40 | 64 / 64 | 32 / 32 | 15 |
| [Horizontal color controls](../output/jpeg-horizontal-boundary-r1/visual/gallery.html) | 11 | 88 | 33 | 55 | 88 / 88 | 48 / 40 | 21 |
| Total | 19 | 152 | 57 | 95 | 152 / 152 | 80 / 72 | 36 |

Each source has original and clone observations at 240×240, 390×320, 768×560, and a return to 240×240. The recorded browser is Chrome 153.0.8010.12; the renderer is RustMetal RasterOrdering. Repetition transfers only within one case with the same source HTML, asset, Rive bytes and viewport, plus exact Chrome/native/diff PNG bytes **and full decoded RGBA equality**. There are no transfers between assets, viewports, or corpora.

## What inspection found

The chroma sweep uses horizontally uniform stripes in 17-row images. For these exact sources, 4:2:0 at widths 1–4 has visibly different row transitions: native red begins muting earlier, and the blue/purple and lower orange bands transition in different source rows. Widths 5 and 6 remain close. The width-four PNG and 4:2:2 controls remain close. Every pixel failure in this corpus is preserved below.

| Chroma source | Existing pixel result across 8 frames | First-frame image mean RGB error |
| --- | --- | ---: |
| 1×17, 4:2:0 | 0 pass / 8 fail | 9.686275 |
| 2×17, 4:2:0 | 0 pass / 8 fail | 9.686275 |
| 3×17, 4:2:0 | 0 pass / 8 fail | 9.699346 |
| 4×17, PNG | 8 pass / 0 fail | 0.044118 |
| 4×17, 4:2:0 | 0 pass / 8 fail | 9.691176 |
| 4×17, 4:2:2 | 8 pass / 0 fail | 0.240196 |
| 5×17, 4:2:0 | 8 pass / 0 fail | 0.521569 |
| 6×17, 4:2:0 | 8 pass / 0 fail | 0.526144 |

The horizontal corpus changes colors across columns. Both 4:2:2 and 4:2:0 fail for the 3×1 and 4×17 controls; 4×1 4:2:0 also fails. The magnified 4×17 pairs show consistent column-color differences across all 17 rows. Some numeric passes also have visible differences: 2×1 4:2:0 has a more purple native right pixel, and 4×1 4:2:2 changes the transition across its four source pixels. A numeric pass is not a claim of pixel identity.

| Horizontal source | Existing pixel result across 8 frames | First-frame image mean RGB error |
| --- | --- | ---: |
| 2×1, 4:2:2 | 8 pass / 0 fail | 0 |
| 2×1, 4:2:0 | 8 pass / 0 fail | 6 |
| 3×1, 4:2:2 | 0 pass / 8 fail | 8.444444 |
| 3×1, 4:2:0 | 0 pass / 8 fail | 8.888889 |
| 4×1, PNG | 8 pass / 0 fail | 0 |
| 4×1, 4:2:2 | 8 pass / 0 fail | 4.25 |
| 4×1, 4:2:0 | 0 pass / 8 fail | 7.25 |
| 5×1, 4:2:2 | 8 pass / 0 fail | 0.333333 |
| 5×1, 4:2:0 | 8 pass / 0 fail | 0.4 |
| 4×17, 4:2:2 | 0 pass / 8 fail | 4.269608 |
| 4×17, 4:2:0 | 0 pass / 8 fail | 7.274510 |

The tables report existing verdicts and a descriptive metric; they do not infer a new universal threshold from mean RGB error. In particular, the 4×17 4:2:2 source in the chroma sweep passes while the same dimensions/sampling in the horizontal corpus fails. These are different authored patterns and different bytes. This evidence supports keeping source content in the qualification discussion rather than assuming dimensions and sampling alone establish fidelity.

## Evidence verification

All 57 distinct complete Chrome/native pairs and their saved full diffs were inspected with `view_image(detail=original)`. The 29 full-frame sheets preserve unscaled pixels; seven supplemental sheets show every case's exact image box at nearest-neighbor 24×. Every sheet is at most 1600 pixels per axis, and the viewing tool did not resize these sheets. Linked PNGs preserve the original artifact dimensions even when the HTML gallery fits them to the panel. Supplemental RGB differences are explicitly multiplied by four and clamped to 255; they never replace the original gates.

The completion scripts rehash 80 source-binding records and 1,072 source/frame artifact records across the two corpora. They verify 19 source assets against frozen build bindings and exact authored PPM patterns. The preparation scripts verify that the exact same asset bytes occur in the browser data URL and ordinary ImageAsset/FileAssetContents records, and that render/replay Rive bytes agree. No surrogate assets are used.

The additional metadata checks verify baseline SOF0, 8-bit three-component JPEGs with precisely `(2,1),(1,1),(1,1)` for 4:2:2 or `(2,2),(1,1),(1,1)` for 4:2:0, matching dimensions and neutral JFIF headers before the first SOS. PNG controls have exact source RGB pixels, 8-bit RGB IHDR, only IHDR/IDAT/IEND chunks, and valid chunk CRCs. This is a finite source/header check, not an independent JPEG entropy validator or a claim about metadata after the first SOS.

All 114 distinct full Chrome/native image buffers are opaque and have exact white RGBA outside their integer image boxes. Exact repeated buffers inherit this check. The original 32 chroma and 40 horizontal failure rows remain in both coverage and completion receipts, including frame number, viewport, local error and source bindings.

The independent final composition verifier initially caught a contact-sheet title overlapping some preceding frames' bottom white margin. That first presentation, its receipts and the precise failing cells are preserved under each `visual/presentation-r1/` directory with an archive manifest. Only row spacing changed. All ten affected sheets were recomposed and re-inspected; the other 26 sheets are byte-identical to their directly inspected originals. The final verifier checks every displayed full-image rectangle against the original RGBA buffer and every magnified crop against its stated transform, so labels cannot silently cover captured pixels. It also verifies that source artifacts, frames, transfers and verdicts match the preserved first revision exactly. No original capture was changed.

## Receipts and reproduction

The preparation receipts retain `reviewCompleted: false` because they were written before direct inspection. The separate final receipts record `visualReviewCompleted: true`, bind every inspected sheet and observation, and preserve the original numeric outcomes:

- [Chroma completion receipt](../output/jpeg-chroma-boundary-r1/visual/review-receipt.json), SHA-256 `2537a268f7340aed9f28a77f312d3425d709771afbc55c31e176a0773257a758`.
- [Horizontal completion receipt](../output/jpeg-horizontal-boundary-r1/visual/review-receipt.json), SHA-256 `b0240c8eb94d7217600c65a62d2c6954885e983ce2eea002f4675039a93a5bd6`.
- [Chroma source metadata](../output/jpeg-chroma-boundary-r1/visual/source-metadata-verification.json) and [background verification](../output/jpeg-chroma-boundary-r1/visual/full-background-verification.json).
- [Horizontal source metadata](../output/jpeg-horizontal-boundary-r1/visual/source-metadata-verification.json) and [background verification](../output/jpeg-horizontal-boundary-r1/visual/full-background-verification.json).
- [Chroma package receipt](../output/jpeg-chroma-boundary-r1/visual/package-receipt.json) and [horizontal package receipt](../output/jpeg-horizontal-boundary-r1/visual/package-receipt.json) bind this review and the final visual package.

Commands used from `tools/html-to-riv`, after direct inspection of the generated sheets:

```sh
python3 output/jpeg-chroma-boundary-r1/visual/build.py
python3 output/jpeg-horizontal-boundary-r1/visual/build.py
python3 output/jpeg-chroma-boundary-r1/visual/complete.py
python3 output/jpeg-horizontal-boundary-r1/visual/complete.py
python3 output/jpeg-chroma-boundary-r1/visual/verify.py
python3 output/jpeg-horizontal-boundary-r1/visual/verify.py
```

Preparation/completion scripts refuse to overwrite their receipts. `verify.py` can recheck the existing package without changing it. No new native or browser run is needed to repeat the integrity check. This supplement does not repeat or expand the earlier [24-case JPEG visual review](jpeg-subsampling-pixels-review.md).
