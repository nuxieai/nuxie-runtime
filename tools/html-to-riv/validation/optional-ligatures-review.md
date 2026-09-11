# Optional ligatures with CSS letter spacing

A12 now has a real optional-ligature fixture: the static Open Sans regular font
in tests/assets. The Rust regression proves it shapes fewer glyphs with default
features than with liga/clig disabled. Font bytes and the declared test family
are identical in Chromium and native output; the asset README records provenance,
license and SHA-256. The earlier Inter specimen alone did not prove ligature
suppression because that font did not form the tested fi/ffi ligatures.

Before this change the twelve new browser comparisons passed only 3/12: normal
spacing passed at each width; positive, negative and centered positive spacing
failed at all three widths. The shaping test also failed before implementation.
The retained baseline gallery is ligatures-before/gallery.html.

The new CssLetterSpacingExperimental font mode disables optional liga, clig,
dlig and hlig features when a shaping run's letter spacing is nonzero. It also
uses the existing spacing-at-cluster-end behavior. Normal/zero spacing retains
optional ligatures; required rlig features remain untouched. Explicit feature
options cannot re-enable these optional ligatures on a nonzero CSS spacing run.
Decoded fonts still default to ordinary per-glyph Rive behavior.

This is a new versioned requirement, text-css-letter-spacing-v1. The existing
text-cluster-spacing-v1 retains its narrower semantics. New compiler output
requests the CSS capability for every nonzero text style. A cluster-only host
fails ensure_supported for the new requirement. The checked host installs the
corresponding retained FontAsset policy before layout; the existing replacement
seam applies it on later font replacement too. Native/WASM output parity includes
the new manifest. This capability contract does not imply arbitrary CSS or
Unicode support, and direct raw Rive import still cannot enforce a sidecar.

Regression coverage proves the fixture actually forms ligatures, normal spacing
retains them, positive and negative CSS spacing suppress them, font options
preserve the selected policy, and both legacy and cluster-only modes retain
their old ligatures. Browser cases exercise normal, positive, negative and center
alignment at 240/390/768px after resizing the same compiled scene.

A12 remains partially qualified. Font fallback is not accepted by the current
single-font profile. Cursive scripts, RTL and broader Unicode shaping still need
separate qualification. No new font-feature-settings syntax, italic/variable
font admission, fallback syntax or CSS Grid support is introduced here.

Reproduce the full checked-host glyph lane:

```sh
bash tools/html-to-riv/validation/run.sh native-glyphs
```

For the vector ligature subset, use the already built probe with
NUXIE_NATIVE_GLYPHS=0 and npm test -- --grep optional-ligature- from the module.
All existing geometry and pixel tolerances are unchanged.

## Qualification results

Full glyph lane: **322/323**. Only the existing A09 em-layout-cascade fractional
shape-edge specimen at 240px fails. All twelve new ligature comparisons pass;
the combined vector spacing/ligature subset passes **39/39**. All 303 preexisting
native PNGs are byte-identical to requirements-full/gallery.html. The new twelve
browser/native pairs were visually inspected at all three widths: wrapping,
spacing, ligatures and centered final lines agree. Contact sheets are retained
as ligatures-full/ligatures-{240,390,768}.png alongside the full gallery.

All 53 Rust tests, five JS/WASM tests, one checked-host rejection test, two
gallery tests, TypeScript, module Clippy and pure-runtime boundary checks pass.
Renderer-state comparisons pass 27/27. The added compatibility assertions prove
that a cluster-only host rejects the new requirement and its shaping semantics
remain unchanged. Chromium 153.0.8010.12, DPR 1; actual Rust Metal replay.

Artifacts under output/playwright/html-to-riv: ligatures-before (red baseline),
ligatures-full (checked glyph lane), and ligatures-vector (39-case subset).
The full runner exits nonzero for the retained A09 failure; state controls were
run separately because the runner stops before that final stage on failure.
One preliminary run was discarded after a parallel cargo test rebuilt the shared
probe without glyph features. The reported full run rebuilt all required
features and completed without that infrastructure interference.

Next: keep A12's Unicode/fallback limitation explicit and investigate A13 word
spacing. Neither native text style nor the pinned C++ text headers currently
expose a word-spacing field; determine a responsive run mapping or explicit
runtime requirement before admitting the syntax.
