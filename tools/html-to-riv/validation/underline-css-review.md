# A20 CSS underline implementation review

Status: active, partial visual qualification. This supersedes earlier notes that
underline CSS is rejected. Chrome 153.0.8010.12 remains the reference.

## Implementation

`decoration.rs` separates CSS properties from applied ancestor origins. Shorthand
expands before cascade computation. Font-relative lengths resolve using the
final computed font size, independent of declaration order. Offset and skip-ink
inherit, while line/color/thickness do not. An origin is appended after the
local color resolves; inherited origins remain when a child declares none.
Each text leaf resolves origins against its used font where Chrome does so,
and emits ordered version 3 underline records. Subsequent wrap policies cannot
downgrade that version. The checked probe installs records before layout.

Accepted syntax and deliberate exclusions are listed in SUPPORT.md. This does
not introduce general block containers, inline mixed content or editor code.

## Independent origin reference

Twenty Chrome probes cover block/flex parent display, 40→20 and 20→40 font sizes,
and auto/from-font/10%/0.1em/3px thickness. The child text uses its own font size
for auto/from-font/percent; em computes at the ancestor. Reproduce with
`node tools/html-to-riv/validation/underline-origin-reference.mjs` against the
checked `underline-origin-reference.json`. All 20 pass. This reference also
covers block parents that are not yet accepted by the compiler; it informs CSS
semantics without claiming support for those layouts.

## Compiler and image evidence

Six public compiler tests cover ancestor none retention, separate local color,
computed-versus-used metric resolution, em declaration ordering, signed offsets,
skip-ink inheritance, CSS-wide values, version 3 plus wrap policy composition,
shorthand resetting only its own fields, and rejected modes/duplicate components.
The module suite has 87 passing tests. WASM builds and the five JS tests pass,
including byte/source-map/requirements equality across the accepted corpus with
all 17 underline fixtures.

Focused fixtures: auto, skip-none, from-font, percentage metrics, negative offset,
right alignment, pre-wrap, font declaration order, block/flex text children with
three ancestor metric kinds, multiple origins, Open Sans from-font, and shorthand.
Each compiles at 390 and resizes the same scene to 240/390/768 in the runtime.

- Native glyph renderer: 51/51 pass.
- Vector renderer: 49/51 pass.
- Layout: zero measured error for all bounds in both profiles.
- Visual review: all 102 browser/native/diff pairs inspected in 18 contact sheets.
  Baselines, wrapping, red/blue origin order and skip gaps align; thin from-font
  lines and vector glyphs retain raster differences.

Artifacts: `output/playwright/html-to-riv/underline-css-glyph-final/` and
`underline-css-vector/`, plus their test-results directories. Reproduce with
`NUXIE_NATIVE_GLYPHS=1` or `0`, `NUXIE_HTML_REVIEW_DIR=<absolute output dir>` and
`npm --prefix tools/html-to-riv test -- --grep underline- --output=<separate dir>`.

Retained vector failures at 390:

| Fixture | Interior RGB error | Limit |
| --- | ---: | ---: |
| underline-from-font | 6.741839 | 6 |
| underline-from-font-opensans | 6.880056 | 6 |

No tolerance changed. These remain failures even though global error and geometry
pass. Initial fixtures incorrectly used general block parents and were rejected
by the existing `unsupported-block-context` rule (nine cases). Their first-run
report is retained in `underline-css-glyph/`; corrected fixtures test supported
flex parents with block/flex text children. The compiler's block-layout scope did
not change to make the fixtures pass.

## Remaining work

A20 is not complete. Extend CJK/All skip-ink pixel coverage, sparse-decoration
negative controls in the ordinary corpus, absent/zero font-metric fallback,
explicit breaks and trailing/hanging whitespace, hidden text, opacity/clipping,
transparent/currentColor and richer composition tests. Investigate retained
from-font raster differences. No claim of general inline decoration propagation,
vertical writing, arbitrary fonts or raw-Rive-only support is made.

Logs: `/tmp/html-underline-css-{suite,parity,glyph-final,vector,clippy-final,boundary}.log`.
Full regression: 974/975 runner tests pass, with the existing
`em-layout-cascade` 240 failure (449 mismatch pixels). The gallery contains
965/966 passing visual cases; the remaining nine runner tests are not gallery
comparisons. All 915 native PNGs shared with the previous `locale-initial`
gallery are byte-identical; 51 new underline image cases were added, with no
missing prior cases. Baseline comparison: `/tmp/html-underline-baseline-compare.json`.
The full gate log is `/tmp/html-underline-full-gate.log`.

The standalone 48 underline references, 20 origin references and 27 native
renderer-state controls pass. Both reference scripts are now in run.sh so future
full runs include them automatically; they were run independently for this
receipt because the runner was already in progress when they were added.
Module Clippy and boundary checks pass. A20 remains active, and the unchanged
older A09 failure remains recorded separately.
