# Flex pixel failures isolated with fixed-box controls

The selected private flex failures persist without flex growth/shrink. Four public fixed-size controls reproduce the full native images from the original flex cases exactly; both the native and Chrome image identities are checked. A single untransformed, opaque box with height 25.5px reproduces a related fractional-edge failure while both renderers report exactly the same height. Changing its ordinary paint primitive does not repair that minimal failure.

These are demonstrated limitations of the tested paint encodings on the immutable baseline. They do not prove that all ordinary-file compositions are impossible, and they do not expand public flex support or change pixel tolerances. No runtime or renderer edits were made.

## Executable feedback loop

A bound one-frame replay reproduces the original error in about a tenth of a second on the validation host:

```sh
node tools/html-to-riv/validation/replay-pixel-case.mjs \
  tools/html-to-riv/output/flex-proof-bridge-r2/render/receipt.json \
  bridge-row-subunit-point 1 FRESH_OUTPUT
```

It verifies the saved Chrome/stream/renderer/pixel-gate hashes, replays the actual native stream and applies the existing pixel gates. The observed native image is identical; mismatch ratio is 0.020625, and the ratio/mean-channel gates fail with exit 1. This replay is a paint diagnostic, not a fresh import/resize test.

The complete controls are reproducible with:

```sh
node tools/html-to-riv/validation/check-flex-fixed-controls.mjs FRESH_OUTPUT
```

The command runs all three corpora and exits 1 for the retained pixel failures. It uses frozen public/Candidate binaries with verified hashes, imports ordinary bytes and observes originals/clones. Each diagnostic case deliberately repeats one selected viewport; these are fixed-size isolation controls, not a responsive output profile or a proposal to compile browser-measured rectangles. The new canonical run is `output/flex-fixed-controls-r2`.

## Reduction and hypotheses

Starting with the original row-subunit scene at 100×80, removing `a`, then `b`, then the fixed sibling and then the leading box leaves a failing painted parent. Removing the outer wrapper preserves the failure. Removing the explicit width reduces the case to:

```html
<div id="p"></div>
```

```css
#p { height: 25.5px; background: #14233f; }
```

Removing either the height or the background makes the comparison pass. The minimized case has no nonlegacy flex factors, nested content, alpha paint, constraints, transform, or unusual inherited context. The 1.25% whole-frame mismatch is concentrated at its fractional bottom boundary; measured geometry is equal.

Three predictions were tested separately: (1) fractional paint coverage should survive fixed sizing and disappear at integer edges; (2) a used-size difference should disappear when both sides receive the same fixed dimensions; (3) a LayoutComponent-specific paint difference should disappear when using an ordinary Shape/Rectangle instead.

The selected controls support (1). Equal point dimensions and exact native image reproduction falsify (2) as a sufficient explanation for these failures. The Shape/Rectangle control falsifies (3) for the minimal box: both ordinary paint paths produce identical full native images. See `solid-border-alternatives-review.md` for that independent primitive experiment and its integer-height positive control.

## Fixed-size controls

Each original case has two controls with flex removed: explicit sizes taken from native f32 observations and explicit sizes from Chrome's used geometry. The original HTML, ordering and colors remain. The native-size controls for all four selected cases reproduce both original native and original Chrome pixels exactly:

- row subunit factors at 100×80;
- row-reverse subunit factors at 400×200;
- column decimal factors at 100×80;
- column-reverse decimal factors at 100×80.

Both Chrome-size subunit controls are also exact on both sides. The two decimal Chrome-size controls keep Chrome pixels identical, alter 206 and 175 native channel values respectively, and still fail the same mismatch/local-color gates. CSS used-size rounding changes some paint samples in those decimal cases, but correcting that difference alone does not repair the failure.

These statements are full-image comparisons, not equality inferred from matching error metrics. They establish four particular failure controls, not every retained flex failure.

## Fractional height observations

The public single-box matrix includes 25, 25.25, 25.5, 25.75, 26, 0.03125, 0.25, 0.5, 0.75, 1 and 1.25px heights. All measured Chrome/native heights agree. Integer heights 25, 26 and 1 pass the pixel gates; all other cases fail at least a local or whole-image gate.

At sample x=20, height 25.5px gives Chrome row 25 RGBA `(20,35,63,255)` and native `(138,145,159,255)`. At height 25.25px, Chrome row 25 is white while native is `(196,200,207,255)`. Native paints partial coverage where the observed Chrome box uses a hard boundary. The independent visual review samples x=50; a native channel can differ by one across the edge, so the exact samples must retain their coordinates.

Do **not** derive a universal nearest-pixel rule from the larger boxes: Chrome's 0.25px case paints one opaque row, its 1.25px case paints one row, its 25.25px case paints 25 rows, and its 0.03125px case paints none. A proposed compiler paint-snapping composition must explain and test the small-size behavior as well as ordinary fractional edges, including both axes, origins and resizing. The current evidence does not establish such an encoding.

## Counts, identities and visual review

| Corpus | Cases / frames | Geometry passes | Pixel passes | Clear passes |
|---|---:|---:|---:|---:|
| Reductions | 9 / 72 | 72 | 16 | 144 |
| Single-box heights | 11 / 88 | 88 | 24 | 176 |
| Fixed flex controls | 8 / 64 | 64 | 0 | 128 |

All 28 distinct pairs were independently inspected in eight unscaled sheets. The 56 displayed source crops match their full-resolution input pixels. All 196 repeated pairs and 448 alternate-clear images match through full RGBA comparison. An additional 448 full-image comparisons bind the canonical r2 rerun to those reviewed r1 images, with identical requests, Rive bytes and maps. See `flex-fixed-visual-review.md` for the independent review, `output/flex-fixed-controls-r1/visual-evidence.json` for its exact bindings, and the compact `flex-fixed-controls-receipt.json` for the canonical run.

The initial edge-matrix setup had duplicate generated fixture names and stopped before rendering. That setup error and log are preserved under r1; the corrected matrix uses unique `0p...` names. It does not contribute to frame counts.

## Compiler consequence

The numerical leaf proof can continue to be developed independently of this paint discrepancy. Public grow/shrink admission still requires an explicit fidelity contract backed by evidence; failing paint cannot be hidden by a passing numerical bound. Border alternatives have separately demonstrated ordinary responsive representations, but the fractional-edge counterexample persists for all three tested alternatives.

Next work should evaluate any further plausible file-level paint strategy against this minimal red-capable command and the original controls. If no viable strategy remains, document the precise unsupported contract and the missing native capability separately. Do not compensate by silently rounding layout geometry, rasterizing, baking browser layouts or recompiling on resize.
