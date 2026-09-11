# Ordinary embedded text: provisioning works, fidelity unresolved

The first three experiments import only ordinary Rive bytes through the immutable baseline. FontAsset/FileAssetContents carry the same licensed Roboto subset used by pinned Chrome 153.0.8010.12. No loader, font setter, policy or rendering adapter is installed. Original and clone both retain the decoded font across four viewport updates. The public compiler still rejects text and asset input.

Read-only observation reports four glyphs with advances 17.40625 each, matching Chrome's total advance 69.625. Native line height is37.5 and first baseline29.6875; Chrome's normal line box is38 with font ascent30. Metrics remain identical across all eight original/clone observations. Corrupt embedded font bytes produce an importable file with no shaped runs in either instance, demonstrating why import success alone cannot certify font provisioning.

| Candidate | Native Y | Ordinary stroke | Local interior RGB error | Result |
|---|---:|---:|---:|---|
| Default native text |20|0|8.834|8/8 pixel failures|
| Baseline offset |20.3125|0|9.397|8/8 pixel failures|
| Offset and stroke composition |20.3125|0.2|8.165|8/8 pixel failures|

The existing interior limit remains6. The baseline offset comes from the observed ascent difference; it aligns ink bounds but does not improve overall fidelity. The stroke is an experimental composition of existing Rive paints, not a runtime change or a general font-weight mapping. It does not pass the gate. All nine unique full-frame image pairs were inspected and all24 frames retain exact repeat/clone image evidence. These experiments demonstrate a live ordinary font path, not CSS text support or impossibility.

`ordinary-text-receipt.json` binds the snapshots, build/library hashes, images, metrics and preserved failures. The experiment generator is `examples/ordinary-text.rs`; the independent Chrome/native driver is `check-ordinary-text.mjs`; the separate read-only observer is `text-probe.rs`. The current comparison reference deliberately remains font32 at CSS left20/top20 and normal line-height; generator overrides test candidates against that reference rather than changing the browser expectation.

Next investigations: ordinary line-box containment for CSS line metrics; alternative ordinary paint compositions for edge coverage; multiple sizes and fractional positions; then a broader licensed font/glyph corpus. For example, a second translucent Fill over the same glyph paths may change edge coverage without altering glyph advances, but this is only a candidate and must not be admitted without evidence. Do not compensate by weakening pixel tolerances or changing the renderer. The current fixture contains only U+0061 and cannot qualify general Latin text, font shorthand, wrapping or complex scripts.
