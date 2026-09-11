# Em lengths: partial validation receipt

A09 accepts em where the existing language accepts scalar lengths: dimensions,
min/max sizes, flex basis, physical margins/padding, gaps, radius, font size and
line height. Font size uses the parent's computed font size. Other em lengths
use the element's final cascaded font size, even when declared before it. Font
shorthand expands before computation. An em line height inherits its computed
pixel length; a unitless line height retains its multiplier. Existing property
restrictions remain, including nonnegative margins and coupled flex factors.
Rem, other relative units and calc remain rejected in this milestone.

Three public compiler tests cover byte equivalence to independent px values,
shorthand/important ordering, global inheritance, malformed and unmatched
syntax, and coefficient/resolved-length resource bounds. The first regressions
failed before implementation. The fractional visual fixture also has an exact
px equivalence assertion, establishing that its rendering difference persists
without em syntax. No runtime or renderer source changed for A09.

## Reproduce

From the repository root:

```sh
CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native-glyphs
# The full script stops at the preserved pixel failure; run host controls:
NUXIE_NATIVE_GLYPHS=1 node tools/html-to-riv/validation/glyph-state-control.mjs
# Inspect the default vector profile without changing the compiled probe:
cd tools/html-to-riv
env -u NUXIE_NATIVE_GLYPHS npm test -- --grep 'em-' --output=test-results-em-vector
```

Full experimental macOS glyph gate: **247 pass, 1 fail out of 248**.
All 44 Rust tests, five JavaScript/WASM tests, two gallery checks, TypeScript,
module Clippy and pure-runtime boundary check pass. Native/WASM bytes and maps
agree for the expanded accepted corpus. Host-state controls separately pass
27/27. The final added px-control assertion passes in the three-test em rerun.

Five new fixtures compile once at 390px and resize to 240/390/768px. All 15
geometry comparisons pass. All 15 browser/native/diff triples were visually
inspected: font-size inheritance, wrapping, leading, spacing, flex layout and
colors agree within the documented checks, with residual edge/text coverage
differences. Chromium 153.0.8010.12 at DPR 1; actual Rust Metal replay.

## Remaining failures

- Both profiles: em-layout-cascade at 240px has 449 mismatching pixels,
  fraction 0.00584635, above the unchanged shape limit 0.005. Geometry is exact:
  a starts at (23.75,23.75), b at (20,97.5), root height 137.5. Difference images
  isolate fractional shape boundaries. This is consistent with differing edge
  snapping/coverage; no renderer fix is claimed. Whole-frame mean RGBA error
  is 0.203 and all local/interior bounds pass. The same edge differences appear
  at larger widths, whose whole-frame mismatch fractions pass.
- Default vector new-only gate: **12 pass, 3 fail out of 15**. In addition to
  the shape case, em-nested-typography at 240px has whole-frame mean RGBA
  error 1.2763, and em-font-shorthand-final-size at 240px has 1.0319, both above
  the unchanged limit 1. These text cases pass with experimental native glyphs.
  This filtered run is not a fresh full-vector qualification.

No fixtures were removed, no tolerances widened, and no browser-computed
rectangles were substituted. A09 remains partial pending renderer qualification.
Continue independent A10 rem work while preserving these reproducers.

Local review artifacts: output/playwright/html-to-riv/em-qualified/gallery.html,
review-{240,390,768}.png, and the em-vector/gallery.html profile. The main corpus
and compiler regression are the durable reproductions; generated artifacts can
be rebuilt with the commands above.
