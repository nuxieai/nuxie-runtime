# Underline transport and runtime control review

Historical runtime-stage receipt. For current CSS compilation support and its
remaining qualification limits, see underline-css-review.md.

Status: A20 active; underline CSS remains rejected. Chromium 153.0.8010.12,
DPR 1, pinned Inter fixture. No Firefox acceptance target.

Version 3 requirements carry `text_underlines`: unique Text object IDs, each with
an ordered nonempty `lines` list of ARGB color, resolved thickness and offset in
text-local CSS pixels, and `none`/`auto`/`all` skip-ink. The corresponding required
capability is `text-solid-underlines-v1`. Thickness must be positive and at most
1,000,000; offset magnitude at most 1,000,000; both finite. Unknown fields/modes,
invalid values, missing capability, duplicate targets and wrong target types are
rejected. Existing wrapping policies can share a Text target with decorations.
Versions 1/2 keep their old interpretation and reject nonempty underline records.

The checked probe validates every target before installing records, then installs
resolved runtime lines before layout/resizing. The compiler currently emits no
version 3 records: the control supplies them explicitly. This does not qualify
CSS cascade, metric resolution, propagation, or published CSS compilation.

## Reproduction

Build `cargo build -p nuxie-html-to-riv --features native-glyph-controls --bin html-to-riv --example probe`;
use the existing native `renderer-replay` executable. Run
`npm --prefix tools/html-to-riv run test:underline-runtime`.
The command exits nonzero while the documented vector failures remain.

Artifacts: `output/playwright/html-to-riv/underline-runtime-controls/` contains
source, single compiled `.riv`, original source mapping, per-control manifests,
bounds, draw streams, browser/native/diff PNGs, report JSON and six review sheets.
The same scene bytes are imported and resized from 390 to 240/390/768; no layout
coordinates are baked from browser measurements. The control uses actual
pre-wrap wrapping. Normal compiler syntax and browser reset match on both sides.

## Results

24 comparisons: baseline, solid 2px/offset 1px, auto skip-ink 2px/offset 1px,
and auto skip-ink 3px/offset 4px, each at three widths in two renderer profiles.
All geometry has zero measured error. All 12 native-glyph cases pass; 8/12 vector
cases pass. All browser/native/diff pairs were visually inspected in the six
`review-{glyph,vector}-{240,390,768}.png` sheets. Wrapping, underline baselines,
line endpoints and descender gaps align visually; vector glyph rasterization
shows visibly stronger residual differences than the native-glyph profile.

At 240 the vector mean channel errors are:

| Control | Mean error | Existing limit |
| --- | ---: | ---: |
| No underline | 1.384909 | 1 |
| Solid | 1.313876 | 1 |
| Skip auto | 1.425204 | 1 |
| Offset | 1.378824 | 1 |

Thus the failure also exists without underline drawing. This is evidence of a
baseline glyph-rendering contribution, not permission to waive decorated cases.
No limits changed. Red-decoration coverage is checked separately within the
existing host-state control's 15% coverage bound; ordinary image and regional
limits still apply. For the offset control at 240, Chrome and native both paint
1,638 red pixels. No absence is accepted as a successful sparse underline.

## Other verification

- 81 Rust module tests pass, including three new transport contracts.
- Native host test passes: two differently colored lines actually draw;
  unavailable capabilities and malformed/wrong targets reject before drawing.
- TypeScript declarations and consumer test pass with version 3.
- WASM release builds; all five JS tests including native/WASM corpus parity pass.
  This verifies existing accepted syntax, not future underline CSS output.
- Module Clippy passes with `--all-targets --no-deps -- -D warnings`.
- Pure-runtime boundary check passes.

Logs: `/tmp/html-underline-transport-{tests,host,types,wasm,parity,clippy,boundary}.log`
and `/tmp/html-underline-runtime-control-final.log`.

Next: CSS declaration parsing, independent propagated decoration origins, font
metric resolution, emission of version 3, full native/WASM feature parity and
broader visual controls (fonts, explicit breaks, whitespace, nesting, inherited
color, visibility, clipping and opacity). Runtime control results are groundwork,
not completion of A20.
