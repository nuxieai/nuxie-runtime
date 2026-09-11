# Ordinary embedded text: provisioning works, fidelity unresolved

The first three experiments import only ordinary Rive bytes through the immutable baseline. FontAsset/FileAssetContents carry the same licensed Roboto subset used by pinned Chrome 153.0.8010.12. No loader, font setter, policy or rendering adapter is installed. Original and clone both retain the decoded font across four viewport updates. The public compiler still rejects text and asset input.

Read-only observation reports four glyphs with advances 17.40625 each, matching Chrome's total advance 69.625. Native line height is37.5 and first baseline29.6875; Chrome's normal line box is38 with font ascent30. Metrics remain identical across all eight original/clone observations. Corrupt embedded font bytes produce an importable file with no shaped runs in either instance, demonstrating why import success alone cannot certify font provisioning.

| Candidate | Native Y | Ordinary stroke | Local interior RGB error | Result |
|---|---:|---:|---:|---|
| Default native text |20|0|8.834|8/8 pixel failures|
| Baseline offset |20.3125|0|9.397|8/8 pixel failures|
| Offset and stroke composition |20.3125|0.2|8.165|8/8 pixel failures|

The existing interior limit remains6. The baseline offset comes from the observed ascent difference; it aligns ink bounds but does not improve overall fidelity. The stroke is an experimental composition of existing Rive paints, not a runtime change or a general font-weight mapping. It does not pass the gate. All nine unique full-frame image pairs were inspected and all24 frames retain exact repeat/clone image evidence. These experiments demonstrate a live ordinary font path, not CSS text support or impossibility.

`ordinary-text-receipt.json` binds the snapshots, build/library hashes, images, metrics and preserved failures. The experiment generator is `examples/ordinary-text.rs`; the independent Chrome/native driver is `check-ordinary-text.mjs`; the separate read-only observer is `text-probe.rs`. The initial comparison reference deliberately remained font32 at CSS left20/top20 and normal line-height; generator overrides test candidates against that reference rather than changing the browser expectation.

Next investigations: ordinary line-box containment for CSS line metrics; alternative ordinary paint compositions for edge coverage; multiple sizes and fractional positions; then a broader licensed font/glyph corpus. For example, a second translucent Fill over the same glyph paths may change edge coverage without altering glyph advances, but this is only a candidate and must not be admitted without evidence. Do not compensate by weakening pixel tolerances or changing the renderer. The current fixture contains only U+0061 and cannot qualify general Latin text, font shorthand, wrapping or complex scripts.

## Expanded paint and font matrix (2026-09-11)

Two further subset-font cases test an extra ordinary Fill on the same TextStylePaint, at alpha 0.5 and 1.0. The translucent candidate fails all eight frames; the opaque candidate passes all eight. Both retain the original opaque Fill and the experimental baseline offset. This changes edge coverage without a renderer mutation; it is not an admitted CSS paint mapping.

The full licensed Roboto font then exercises nine cases, each across the same four viewport updates on the original and clone:

| Sample | Size | Extra fill alpha | Passing frames |
|---|---:|---:|---:|
| Agjp | 12 | 1 | 0/8 |
| Agjp | 16 | 1 | 0/8 |
| Agjp | 24 | 1 | 0/8 |
| Agjp | 32 | 1 | 8/8 |
| Agjp | 48 | 1 | 8/8 |
| Agjp | 64 | 1 | 8/8 |
| AV fi | 32 | 1 | 8/8 |
| Agjp control | 32 | 0 | 0/8 |
| Agjp at fractional x=20.5 | 32 | 1 | 8/8 |

Together these additional experiments have 48 passing and 40 failing frames. They do not replace the initial 24 failures. The full-font matrix alone passes 40/72. All use the original local pixel gates and pinned Chrome, with no public compiler text admission.

The 11 full-frame review sheets (three viewport pairs each) were directly inspected. At 12 and 16px the extra fill visibly overweights stems and crowds counters; 24px is closer but still fails the gate. Larger samples and the kerning/ligature sample are visually closer, with residual edge differences. The single-fill 32px control remains visibly lighter. Inspection of these 33 pairs does not certify uninspected lifecycle frames; those are separately recorded by automated comparisons. Review sheets live in each run directory: ordinary-text-native-r4, ordinary-text-native-r5 and ordinary-text-matrix-r1/<case>/render.

The full-font native Y candidate uses 20 + round(size * 1900/2048) - size * 1900/2048, while Chrome stays at top20 with normal line-height. This is an experimental metric adjustment, not a general CSS normal-line-height implementation. Wrapping, mixed runs, arbitrary fonts, device scales and general line-box layout remain unqualified.

The immutable outline path ignores embedded hint instructions. Whether Chrome hinting explains these failures is unresolved. A compiler-owned, size-specific derivative font is a plausible next experiment, with identity round-trip, unchanged shaping metrics, fractional placement and wrap-boundary controls required. See immutable-font-outline-candidate.md. No derivative asset or successful hinting correction is claimed here.

`ordinary-text-matrix-receipt.json` binds the expanded checkpoint: all 11 scenes regenerate byte-identically from the rebuilt generator and frozen fonts. Original/clone and repeated-size PNG hashes match in every case. The checkpoint preserves all 88 frames, including 40 failures.

## Derivative-font control (16px)

The compiler-owned vector-outline experiment compares original Roboto, byte-identical fontTools round-trip, unhinted flattened outlines and FreeType native-normal hinted outlines. All use the unchanged original font in Chrome, at 16px for “Agjp AV fi”, with single and double Fill separately. All 64 frames fail the existing local RGB gate. Original/identity/unhinted interior errors are 9.004 (single) and 9.626 (double); hinted errors worsen to 11.418 and 13.830. Eight first-frame full-image pairs were directly inspected: hinted text visibly changes stem/curve placement and does not improve the match. Other viewport pairs are retained but not claimed directly reviewed.

Glyph IDs, advances, positions and line bounds remain equal in the native observer. Cap-height and x-height do change: normalized -0.7109375/-0.5283203 become -0.75/-0.5625. Preserving font tables therefore does not prove identical runtime font metrics. The derivative generator also records composite quantization and a chi origin shift outside this sample. This specific candidate is unsuccessful; neither hinting as the cause nor general impossibility is established. See `derivative-font-receipt.json`, `derive-font-outlines.md`, and the preserved matrix. No further sizes are warranted for this setting before a distinct hypothesis.
