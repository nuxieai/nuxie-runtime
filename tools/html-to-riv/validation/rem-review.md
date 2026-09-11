# Root-relative lengths (A10)

The fragment profile explicitly fixes the host html font to 16px in reset.css.
Authored rules are scoped below the host body and cannot change that root.
Every rem length, including font size, uses this root basis. Mixed em/rem/px
shorthands retain each unit's own basis. Font shorthand and inheritance use the
existing computed-value path; no layout rectangles come from the browser.

The new public compiler regression failed on unsupported rem font shorthand
before implementation. It now checks byte identity against independent px
sources for nested font sizes, shorthand/important ordering, mixed units and
inherited line height. Malformed tokens, negative values, zero font sizes,
unknown units and resource limits remain rejected. The fixed rem basis permits
resolved-bound checking even on unmatched declarations. Existing rejection tests
now exercise ex instead of newly supported rem.

Four new browser fixtures cover mixed layout units, typography and inheritance,
flex sizing, and host scoping with multiple fragment roots. Each compiles once
at 390px and is imported/resized to 240, 390 and 768px. The root CSS rule pins the
previous browser default explicitly; all existing fixtures stay in the full gate.

A10's documented subset is qualified. The full experimental glyph lane passes
**259/260** checks; its sole failure is the preserved A09 fractional-edge case.
All 12 new geometry/pixel checks pass in both the experimental glyph and default
vector profiles. The vector run is new-only, not a new full-vector qualification.
All 45 Rust tests, five JS/WASM tests, two gallery tests, TypeScript, module
Clippy and the pure-runtime boundary check pass. Host renderer controls pass
27/27 in a separate invocation after the full script's expected pixel failure.

All 12 new browser/native/diff triples were visually inspected. Wrapping,
leading, sizes, spacing, sibling-root behavior and colors agree within the
unchanged gates. The nine shape PNGs are byte-identical between render profiles;
the three vector text images were also inspected. Chromium 153.0.8010.12, DPR 1,
actual Rust Metal replay. No pixel thresholds were changed.

Commands:

```sh
CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native-glyphs
NUXIE_NATIVE_GLYPHS=1 node tools/html-to-riv/validation/glyph-state-control.mjs
cd tools/html-to-riv
env -u NUXIE_NATIVE_GLYPHS npm test -- --grep 'rem-' --output=test-results-rem-vector
```

Intentional exclusions: configurable host root fonts, full documents, ex/ch,
calc, negative margins and previously excluded property combinations. The rem
feature does not broaden these independent semantics. A09's fractional shape
failure remains in the main gate without relaxed tolerances.

Review artifacts: output/playwright/html-to-riv/rem-qualified/gallery.html and
review-{240,390,768}.png; rem-vector/gallery.html holds the default profile.
Support for rem does not resolve existing fractional shape or vector text
rasterization differences when composing other values. Next: A11 percentage
minimum/maximum dimensions, retaining native percentage units through resize.
