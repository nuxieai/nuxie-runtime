# Preserved Unicode-space break opportunities

A13's retained fixed-width-space failures exposed two distinctions: a break
opportunity is not the same as collapsible whitespace, and a preserved separator
contributes advance to both word fitting and line alignment.

The initial focused corpus passed 19/24. Simply extending the native whitespace
classification fixed left-aligned wrapping but left centered and right-aligned
failures (22/24). A subsequent alignment-only experiment passed 26/36 and failed
mixed spaces plus narrow overflow controls. In those controls Chromium moves a
word together with its trailing fixed-width space to the next line; merely
adding width after line fitting cannot reproduce that decision.

The final algorithm keeps preserved spaces in the preceding word. It emits
coincident end/start markers after the preserved separator sequence, producing
an optional break with no collapsible-space interval. Ordinary ASCII spaces keep
the existing collapse/break behavior. Existing GlyphLine fitting and alignment
then consume the correct word widths without new offsets or a separate alignment
correction. The 36 focused comparisons passed before contract integration.
The abandoned post-layout width adjustment is not in the final implementation.

The explicit capability is text-preserved-space-breaks-v1. The compiler requests
it when emitted text contains U+1680, U+2000–U+2006, U+2008–U+200A, U+205F or
U+3000. U+00A0, U+2007 and U+202F stay nonbreaking; ordinary ASCII scenes do not
need this capability. The host validates requirements before import and retains
the font-local break policy through replacement, decode, clear/reload and host
restoration. Space breaks and letter spacing are independent, composable policies.
Default decoded fonts and hosts that import raw Rive bytes keep legacy behavior.
The temporary NUXIE_EXPERIMENTAL_SPACE_BREAKS probe flag has been removed.

Regression tests check all declared code points, including glyph runs split by
script itemization; preservation of nonbreaking spaces; ordinary legacy markers;
font option cloning; combined spacing/break modes; replacement and independent
files; requirement selection and refusal by a spacing-only host; and checked-host
refusal before drawing when capabilities are unavailable. Native/WASM corpus
parity includes the requirement and unchanged source text.

A direct shaper regression also exposed an incorrect new break before U+2060
WORD JOINER. That break is suppressed. The corresponding browser input remains
in deferred-cases.json because the supplied fonts do not contain U+2060 and the
current compiler's font admission rejects it. The deferred corpus now supplies
its declared font before asserting missing-glyph, so it tests the actual admission
gap rather than failing earlier with missing-font. Default-ignorable character
admission remains future work; no invisible control or fallback syntax was silently
admitted to make the space corpus pass.

The final browser corpus adds center/right alignment, leading/trailing/repeated
spaces, nonbreaking spaces, mixed ASCII/preserved spaces, overflow fitting and all
members of the declared space set. Each scene is compiled once and resized in the
native runtime at 240/390/768px. Geometry and pixel tolerances remain unchanged.

Earlier experiment artifacts under output/playwright/html-to-riv are
space-break-before, space-break-experiment, space-break-preserve, and
space-break-after. They record each failed hypothesis and the focused correction.
The compiler/host contract is validated by space-break-full and the final vector
subset, not by the temporary environment override.

## Font admission and final qualification

The wider corpus exposed a separate false admission: Inter lacks U+1680 OGHAM
SPACE MARK, but the compiler skipped its glyph check because Unicode classifies
it as whitespace. Chromium fell back to a visible stem glyph while native drew
a missing-glyph box. That run passed 394/398, with three Ogham failures plus A09.
Its portable gallery is retained in space-break-ogham-before. The same Inter
input is now an explicit missing-glyph regression in deferred-cases.json.

U+1680 now requires a real glyph in the selected font. The positive corpus uses
unmodified static Noto Sans Ogham from Google Fonts revision
baa2e5561af8a4873b058859dcfe158bdd033942, under the supplied OFL 1.1 license.
The asset README records its pinned source and SHA-256. Exactly the same font
bytes go to Chromium and Rive, with no fallback. A browser measurement of
“one U+1680 two” using this font was 95.625px at both word-spacing 0 and 6px,
confirming that the reference browser does not add word spacing to this mark.

The final checked-host glyph gate passes **397/398**, retaining only the existing
A09 fractional-edge failure. All 48 focused preserved-space comparisons pass in
both glyph and vector renderer profiles. The original 33 word-spacing cases all
pass in the full glyph lane. Among the 348 previous scene PNGs, only the two
240px fixed-space failures change; the other 346 PNGs are byte-identical.

All 42 new browser/native pairs and the corrected older cases were visually
inspected at 240/390/768px. Alignment, line fitting, leading/trailing spaces,
nonbreaking controls and the visible Ogham stem agree. Contact sheets are retained
as space-break-full/spaces-{width}-{group}.png alongside the portable gallery.
All 59 Rust tests, five JS/WASM tests, one checked-host test, two gallery tests,
TypeScript, module Clippy and pure-runtime boundary checks pass. The 27 renderer
state controls also pass. Chromium 153.0.8010.12, DPR 1; actual Rust Metal replay.

Reproduce:

```sh
bash tools/html-to-riv/validation/run.sh native-glyphs
# After building the probe, from tools/html-to-riv:
NUXIE_NATIVE_GLYPHS=0 npm test -- --grep 'space-break-|word-spacing-fixed-space'
```

The full runner exits nonzero for A09 and therefore does not reach its final state
control stage; those controls were run separately. No thresholds were widened.
Final artifacts: space-break-full/gallery.html and space-break-vector/gallery.html.
A13 retains broader font/script qualification under T06/T08 and the deferred
format-control admission case. Next independent authoring feature: A14 explicit br.
