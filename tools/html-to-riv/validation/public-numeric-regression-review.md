# Three changed historical numeric outputs

The 694-output regression has **691 exact files/maps and three intentionally
changed files**. The changed requests are the historical `direct`, `fallback`
and `variable` provenance controls. All compile successfully; all three source
maps remain byte-identical. This audit uses frozen numeric r3 compiler
`a77a45eac1d958d728a041a4a82f3d7e67e1685afee4ed1c8c06768cdf629e5b`.

Each request authors the same width, `100.71428680419922px`, either literally,
as a missing-variable fallback or through `--n`. Before/after decoding finds
exactly one changed property: object4 / file record5, `LayoutComponent` type409,
width property7. Its binary32 value changes from **100.71399688720703**
(`0x42c96d91`) to **100.71428680419922** (`0x42c96db7`). Only byte92 differs in
each original400×100 file; every other byte and both source-map contents remain
identical. `field-audit.json` binds both files, their requests and decoded fields.

The old normalizer serialized the source as100.714px. The repaired serializer
retains the authored numeric prefix, so the new field equals the direct
decimal-to-binary32 conversion. Relative to the exact authored decimal, old
absolute error is0.00028991699218875; new error is0.00000000000000125. The latter
is the remaining binary32 representation error, not exact decimal equality.
No source was rewritten to recover the old bytes.

## Fresh native and Chrome evidence

The three historical controls had native geometry evidence only: their review
explicitly excludes browser/pixel qualification and their directory contains no
PNG files. Therefore no historical visual review is transferred. Their old
files, measured geometry, scalar-error receipt and review remain unchanged.

`public-numeric-regression-cases.json` preserves each HTML and CSS string exactly.
The unchanged public native driver compiles once, then imports/clones and resizes
both instances400×100 →200×80 →100×40 →400×100. It uses the unchanged baseline
probe/renderer, Chrome153.0.8010.12, RustMetal effective RasterOrdering, the same
reset, and the existing0.1px geometry / pixel gates. Results: **24/24 geometry,
24/24 pixels,48/48 clear-color checks**. Return-size and clone image checks pass.

The driver's fixed compile viewport is390×160, while the original historical
requests compile at400×100. This is recorded rather than hidden: the driver
file differs from each exact regression file only in the two Artboard initial
dimension fields. Each exact400×100 regression file was separately imported and
cloned with the same resize sequence. All24 geometry files and all24 recorded
drawing streams are byte-identical to the driver's corresponding observations.
`exact-file-transfer.json` binds those replay inputs, establishing the rendered
evidence for the exact changed historical files without altering the driver or
adding any runtime hooks. Source maps also remain exact between these profiles.

Native width is100.71428680419922 after restoring the probe's serialized float to
binary32; Chrome's rectangle is100.703125. Their difference,
0.01116180419921875px, passes the existing0.1px geometry gate. This is bounded
agreement, not exact browser geometry. The historical scalar comparison against
the authored decimal was never a Chrome-width measurement.

## Visual inspection and limits

All three distinct viewport pairs were directly viewed in
`output/numeric-token-changed-outputs-r1/visual-pairs.png`. The six placements
retain original pixel dimensions and complete RGBA equality with source images.
The other21 pairs transfer inspection through complete native-and-Chrome image
identity; `visual-evidence.json` records each source and target. All24 pairs are
covered, with no historical image-transfer claim.

The exact sources have transparent backgrounds and no text or painted child, so
both renderers show white canvases. Those images match, but **they cannot verify
the box width**. Width correctness here rests on the independent emitted-field
and read-only geometry checks. Painted numeric cases are qualified separately
by the numeric native corpus; this audit adds no painted coverage or broad
numeric-domain proof.

No production source, existing tests, runtime, renderer, dependencies, gates or
shared support documents were changed by this audit. The
[receipt](public-numeric-regression-receipt.json) records the narrow qualification
and its evidence paths.
