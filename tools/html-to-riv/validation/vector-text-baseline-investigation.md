# Vector text baseline and coverage investigation

Status: active. No shipping runtime change has been retained from these experiments.
Chrome 153.0.8010.12 is the reference. Numeric and pixel gates are unchanged.

## Reproducer and controls

The percentage-spacing version16-r3 matrix passes669/669 native-glyph comparisons.
With native glyphs disabled, geometry passes669/669 and pixels pass651/669.
All18 failures are text compositions. All669 views have audited visual evidence;
the failing images are inspected, not waived.

A one-scene vector replay fails twice with identical PNGs. Native glyphs pass
against identical compiled Rive bytes and the same Chrome image. Removing
siblings, setting all spacing to zero, and removing wrappers preserves failure.
The reduced fragment is `<div id=inner>Quiet weekend</div>` with
`#inner{display:block;width:110px;font:16px/1.4 Inter}`.

Individual words and glyphs pass with nonzero error; that does not prove an
absence of rendering differences. The full phrase also fails with nowrap.
Integer line heights22/23/24 pass. The22.4px case fails, as does22.25px in a
five-value fractional sweep. Preserve the original failures as regression inputs.

## Findings

1. The native glyph rasterizer snaps the final axis-aligned world baseline to a
   device pixel. Vector outline construction retains fractional baselines.
2. A temporary construction-time snap improves24 text views from6 to16 passes.
   Both the reduced and original240px scene pass. Eight768px failures remain.
3. Auditing emitted glyph coordinates proves16 views reach the intended final
   baseline. Content-box cases can miss by0.5625px. All border-box cases snap
   exactly, so transform timing cannot explain every residual.
4. In the row-border-box768 text rectangle, summed ink is105740 for Chrome,
   92139 for snapped vectors and108466 for native glyphs. This is a diagnostic
   sum of255 minus mean RGB, not a new acceptance metric.
5. Disabling CoreGraphics font smoothing at the glyph-producing probe reduces
   native ink to93676 and increases text RGB error from2.0653 to4.1162. Both
   controls pass. Font smoothing accounts for much of the coverage difference,
   but does not establish pixel equivalence with the outline renderer.

The first smoothing attempt toggled the renderer after glyph images were already
embedded in the stream. It did not exercise rasterization and is invalid as a
smoothing experiment. The corrected experiment runs in the native-glyph probe.

## Current experiment and next decisions

A diagnostic rebuild at draw time tests whether using the final transform removes
the content-box baseline error. It remains isolated from shipping source and must
be compared with the same24 Chrome text views. Then evaluate coverage differences
independently. Do not apply arbitrary emboldening or change generic vector paint
behavior to make a text fixture pass.

Any shipping solution needs an explicit CSS opt-in, draw-time device transform
handling, clone/resize and host-transform coverage, native/WASM compatibility,
and preserved default Rive behavior. A draw-time rebuild on every frame is a
probe, not a production implementation.

## Evidence locations

All paths below are relative to `output/playwright/html-to-riv/`:

- `percentage-spacing-validation/vector-text-diagnosis.json`: reductions and typography controls.
- `percentage-spacing-validation/vector-v16-r3-receipt.json`: authoritative original669-vector result.
- `percentage-spacing-vector-snap-experiment/receipt.json`: construction-time experiment.
- `percentage-spacing-vector-snap-experiment/baseline-audit.json`: final baseline audit.
- `percentage-spacing-vector-snap-experiment/border-768-ink-profile.json`: scanline coverage.
- `percentage-spacing-vector-smoothing-experiment/receipt.json`: corrected smoothing control.
- `percentage-spacing-vector-draw-snap-experiment/`: final-transform diagnostic under evaluation.
