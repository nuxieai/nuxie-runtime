# Independent flex line and item alignment experiment

The ordinary-file wrapper candidate preserves independent `align-self` positions for the tested wrapped lines using `align-content:flex-start`, `center`, and `flex-end`. It does not implement `normal` or `stretch`. These are fixture-specific composition results, not public compiler qualifications and not proof that the remaining values are impossible.

Evidence: `output/line-alignment-wrappers-r1/experiment-manifest.json`, `matrix.json`, `reproductions.json`, and the tracked `line-alignment-wrappers-receipt.json`. The raw driver's `failed-public-baseline` status and public scope wording are overridden by the experimental manifest. Its raw failures remain unchanged.

## Candidate and scope

Forty exact scenes cross row/column, wrap/wrap-reverse, five line-alignment values, and two strategies. Three colored items have different fixed cross sizes and individual start/center/end alignments. The containing main size is 50% of the viewport and its cross size is 120px. Viewports 400×400, 200×200, 120×120, and 400×400 change the line arrangement; the same ordinary bytes are tested on the original and cloned scene. A following green footer tests parent extent.

The exact-match Python adapter compiles transformed fixture HTML/CSS with the frozen public spacing compiler. It then decodes and byte-roundtrips the ordinary records and changes `flexDirectionValue`, `flexWrapValue`, and `layoutAlignmentType` using existing schema fields. No browser rectangles enter compilation. The adapter is experimental authoring machinery, not a runtime requirement.

The direct strategy retains existing public per-item alignment lowering. The wrapper strategy places every authored item inside a transparent cross-stretched perpendicular layout participant, carrying its authored fixed main dimension. The inner box stretches on the wrapper cross axis; the wrapper's main-axis alignment preserves the item's start/center/end position. There are three added wrapper layout/style pairs in these fixtures. Wrapper IDs are removed from the read-only source-map join; no host setter controls layout or painting.

The parent alignment field chooses the line alignment, while wrapper main-axis alignment independently chooses item alignment. Normal and stretch deliberately reuse the start candidate as a diagnostic attempt; they have no implemented line-stretch composition in this experiment.

## Results

Every table cell aggregates both axes and both wrap directions, 32 frames per candidate/value:

| align-content | Direct geometry / pixels | Wrapper geometry / pixels |
| --- | --- | --- |
| flex-start | 32/32 each | 32/32 each |
| center | 16/32 each | 32/32 each |
| flex-end | 16/32 each | 32/32 each |
| normal | 0/32 each | 0/32 each |
| stretch | 0/32 each | 0/32 each |

Overall: 320 frames, 160 geometry passes, 160 pixel passes, and 640 passing cyan/transparent clear checks. All 40 emitted files and source maps were regenerated through the exact adapter and matched the originals byte for byte. Source, executable, compiler checkpoint, fixture, driver, reset, matrix, and image hashes are bound in the manifest/receipt. The original augment build command was not preserved. A reconstructed build now binds the saved sources, compiler version, command, and dependencies in `rebuild-receipt.json`. Its executable differs from the original, but `rebuilt-reproduction.json` proves that the rebuilt augmenter reproduces all 40 original RIV files byte for byte and all 40 parsed source maps exactly. Executable binary identity is not claimed; this adds reproduction evidence without expanding rendering or visual coverage.

The direct center/end failures occur at frames 0, 3, 4, and 7, when all items share one line. For row/wrap center, the first item's native y is 55 instead of Chrome's 45; flex-end gives 110 instead of 90. Column analogues fail on x. Wrap-reverse changes the direction of the corresponding offset. The all-item wrapper candidate fixes these mismatches at every tested viewport.

Normal/stretch fail at all frames with both strategies. On the first row/wrap frame, native b.y=5 and c.y=0, versus Chrome b.y=50 and c.y=90. The native candidate packs the line's cross extent instead of providing the browser's stretched line area. The reverse cases and column axis show the analogous visible difference. This identifies a failed candidate, not an immutable-runtime impossibility.

## Direct visual review and next action

All 40 frame-0 Chrome/native pairs were inspected directly on `visual-{0,4,8,12,16,20,24,28,32,36}.png`. The visible results agree with the geometry failures above: wrapper positional modes match; direct center/end and both normal/stretch strategies diverge. The other 280 frames have automated evidence only; they were not directly visually reviewed. `visual-direct.json` lists exact image hashes.

Keep positional wrappers as a viable composition candidate. Before public admission, validate arbitrary authored ordering, reverse main directions, responsive item dimensions, bounds, intrinsic sizing, nested wrapping, and baseline/spacing interactions, with the normal public interface tests. Investigate an ordinary composition that supplies the stretched line cross extent for normal/stretch; do not silently map these values to start. No runtime change is required or proposed by this receipt.
