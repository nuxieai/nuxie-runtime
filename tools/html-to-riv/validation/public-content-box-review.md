# L12 public content-box checkpoint — partial

The compiler now lowers a bounded content-box profile into ordinary immutable
Rive layout fields. **L12 is partial, with admitted inputs that remain
unqualified.** The strengthened large-coordinate descendant control fails the
unchanged geometry gate; the retained fractional control fails six pixel frames.
Neither failure is converted into an accepted qualification result. This is a
compiler checkpoint, not a claim that all admitted content-box inputs match
Chrome, and not a reason to change the runtime.

The immutable target is `6c7ac16617835b5f581784ff08a9e779bb52faf3`.
The earlier [property/source audit](content-box-review.md) establishes that the
ordinary field named `boxSizing` is a fill-versus-fixed mode, not CSS box-sizing.
This implementation therefore adjusts existing width, height, min/max and
padding fields. It adds no schema, renderer, runtime, dependency, editor or root
build changes, and no runtime sidecar, measurement callback or viewport baking.

## Implemented boundary

`src/box_sizing.rs` separates authored computed styles from a lowered emission
view. Descendants inherit the original computed dimensions and padding; they
never inherit an already enlarged parent outer width. Fixed point dimensions
and fixed point minima/maxima gain their axis's fixed point padding sum. The
native operation is `content + (start + end)` with binary32 values. Automatic
preferred dimensions, automatic minima and absent maxima retain their native
encoding. Zero minima are canonically lowered to the padding sum. The earlier
finite candidate used the native padding floor for some zero minima, so that
candidate's file hashes are not represented as this public implementation's
hashes. Fresh public native evidence qualifies the canonical encoding only to
the extent recorded below.

Percentages on an axis with exact zero inset need no translation. Fixed padding
on the other axis does not prevent that case. CSS-wide `initial` and `unset`
compute content-box; an omitted declaration still uses the authoring reset's
border-box. Explicit inheritance, variables, failed-substitution unset recovery,
fallbacks, importance and inline priority use the existing computed-style path.
Font-relative lengths are converted by the existing unit path before lowering.
The source coefficient limit remains 1,000,000; a derived outer size may reach
3,000,000. No smaller input limit was introduced to avoid numeric evidence.

Intentionally diagnostic contexts remain explicit:

- Nonzero point padding combined with a percentage preferred size, minimum or
  maximum on the same axis needs a responsive percentage-plus-points composition.
  A single ordinary dimension cannot encode the sum.
- Nonzero percentage padding with content-box needs a mixed-size composition.
  Every percentage padding side depends on containing width, including vertical
  sides; it cannot generally be folded into a percentage-height field.
- Existing padding restrictions for alignment wrappers, baseline groups, gaps
  and nonlegacy flex declarations remain in place. This checkpoint does not
  qualify those interactions or borders. CSS grid remains outside scope.
- Recursive percentage amplification that exceeds finite binary32 geometry
  remains diagnostic before Rive bytes are returned.

The first two are limits of this bounded translation. Responsive wrappers that
separate the semantic content box from its outer contribution are possible
research directions, provided they preserve the containing block, sibling
positions and same-file resizing. A paint-only expansion would not satisfy
those requirements. No general immutable-runtime impossibility has been proven.

## Numeric and inheritance integration

The emitted sizes and all lower/upper/witness inputs to `numeric::Bounds::child`
use the lowered view. Bounds subtract native padding from native outer sizes,
including cancellation residuals. Feeding authored content width into this
existing border-box calculation would understate the descendant content range;
the public overflowing percentage-chain rejection specifically catches that.

Per-side padding metadata retains original scalar provenance through shorthand,
font conversion, inheritance and variables. Outward ideal intervals and native
binary32 results remain separate. A native-zero underflowed point inset still
contributes nonzero ideal size metadata. A native-zero percentage inset with
nonzero or missing ideal metadata remains unresolved rather than becoming point
provenance. Derived descriptor scalars bind the actual emitted values, while
existing padded-parent/item flex certificates remain unresolved.

The independent [numeric and inheritance audit](content-box-numeric-audit.md)
reviewed these paths and the final emitter refactor. Its arithmetic examples are
source/numeric evidence; they are not substituted for measured native/Chrome
geometry. The large-coordinate failure below shows why finite aggregation alone
does not establish browser-equivalent layout.

## Bound compiler and native workflow

Final r2 source and binary identities are in
`output/public-content-box-r2/source-bindings.json`: 30 source/build inputs,
CLI `e1b16fd2a068b96afe5f878ca0423877538760178ac78041f5c88b5918c2e9fc`,
WASM `559f70d7fabab80bb1b5b296d50030b44e8a1f52a23b7e74b793adf0618293d0`.
The durable [receipt](public-content-box-receipt.json) verifies those current
identities and binds the artifacts, tests and visual inspection.

The original r1 run rendered 34 scenes, including five parent-only numeric
controls. A later emitter stack repair changed the compiler binary. The frozen
final r2 CLI exactly reproduces all 34 original Rive files and source maps in
`output/public-content-box-r2/reproduce-r1/receipt.json`; therefore their native
evidence still describes those same files. It does not directly describe the
revised fixture HTML for the five strengthened cases.

The current fixture JSON contains 29 unchanged r1 sources and five strengthened
sources with a `width:100%;height:1px` child. These five were separately compiled
and rendered by r2 in `output/public-content-box-r2/boundary-render/receipt.json`.
Their files/maps were also exactly recompiled in `reproduce-boundaries/receipt.json`.
The original parent-only artifacts remain in the r1 receipt and source snapshot.

Both native runs use the unchanged read-only baseline probe and renderer replay,
pinned Chrome `153.0.8010.12`, Rust Metal effective `RasterOrdering` (CLI token
`clockwise-atomic`), the same browser reset and the same pixel gates. Each file
is compiled once at 390×160, then imported and cloned; each instance is resized
through 240×160 → 390×200 → 768×120 → 240×160. Probe input is the ordinary Rive
file and viewport sequence. The source map identifies objects for observation;
it does not drive layout. Replays with cyan and transparent clear colors retain
the same rendered pixels, and all return-size/clone image identity checks pass.

Geometry still uses 0.1 CSS px. Nontext pixel mismatch ratio still uses 0.005,
mean channel error 1, region mean RGB error 10 and interior RGB error 6, with
antialiasing included. No gate, renderer, browser reference or failure was changed
to obtain this checkpoint.

| Evidence set | Scenes | Frames | Geometry pass | Pixel pass | Clear checks pass |
| --- | ---: | ---: | ---: | ---: | ---: |
| Original r1, including parent-only controls | 34 | 272 | 272 | 266 | 544 |
| Strengthened child additions | 5 | 40 | 32 | 40 | 80 |
| **Current fixture corpus: 29 reused + 5 strengthened** | **34** | **272** | **264** | **266** | **544** |
| Superseded parent-only controls, retained separately | 5 | 40 | 40 | 40 | 80 |
| **All retained runs: original 34 + additions 5** | **39** | **312** | **304** | **306** | **624** |

`output/public-content-box-r2/native-aggregate.json` validates the current
requests against the revised JSON before selecting their measured rows. The
combined totals include retained controls without treating them as measurements
of the strengthened source.

## Preserved failures and numeric boundary observations

`content-large-rounded-outer` authors content width `999999.875px`, left/right
padding `1000000px`, height `20px`, and a child with width `100%` and height
`1px`. Native read-only geometry and Chrome's `getBoundingClientRect()` both
report parent width **3,000,000px**. Native child width is **1,000,000px**;
Chrome's child rectangle width is **999,999.75px**. The **0.25px** difference
fails all eight original/clone geometry frames. Chrome serializes computed
width as `1e+06px` for both elements, so that string cannot resolve the observed
rectangle discrepancy. These measurements do not isolate Chrome's internal
content-box arithmetic or rule out projection/rounding effects.

The old parent-only control passed because it compared only the equal parent
rectangles. The strengthened child starts at x=1,000,000px, outside each captured
viewport. All eight pixel frames pass, but that visible agreement cannot qualify
its offscreen width. The current compiler still admits this input; this is a
recorded qualification gap requiring a numeric admission guard or a demonstrated
ordinary-file composition in a follow-up. It is not declared universally
impossible on the immutable runtime.

Frame-zero measurements for all five strengthened numeric controls are retained
in the receipt (raw probe JSON uses binary32 decimal serialization):

| Control | Authored content width; left/right padding | Native parent / child width | Chrome parent / child rectangle width |
| --- | --- | --- | --- |
| add-side-first | 1,000,000; .03125 / .03125 | 1,000,000.0625 / 1,000,000 | 1,000,000.0625 / 1,000,000 |
| padding-cancellation | .03125; 1,000,000 / .03125 | 1,000,000 / 0 | 1,000,000.0625 / 0 |
| rounded-outer | 999,999.875; 1,000,000 / 1,000,000 | 3,000,000 / 1,000,000 | 3,000,000 / 999,999.75 |
| content-cancellation | 999,999.9375; 1,000,000 / 1,000,000 | 3,000,000 / 1,000,000 | 3,000,000 / 1,000,000 |
| full-source-limits | 1,000,000; 1,000,000 / 1,000,000 | 3,000,000 / 1,000,000 | 3,000,000 / 1,000,000 |

The four passing controls are finite observations, not proof of the entire
large-coordinate domain. The padding-cancellation parent's .0625px residual is
within the existing geometry tolerance, not exact agreement.

`content-point-fractional-paint-control` preserves the earlier fractional edge
failure. All eight geometry frames pass. Pixel frames 0,2,3,4,6,7 fail mismatch
ratio; the 240×160 ratio is .00625 and 768×120 ratio is .008333333333333333. The
390×200 frames pass the whole-image ratio because that edge occupies a smaller
fraction of the canvas; this does not erase the visible boundary difference or
qualify fractional painting. Both native drivers intentionally finish with
`failed-public-baseline` and exit 1.

## Visual inspection coverage

All 60 distinct current image pairs were directly viewed in ten sheets under
`output/public-content-box-r2/visual/sheet-0.png` through `sheet-9.png`. Each
viewport is placed at its original pixel dimensions; the generator verifies
complete RGBA equality for all 120 placements and refuses to change an existing
reviewed sheet. No crop or rescale is used in those sheets.

The remaining current pairs have complete pair identity transfers: 72 to the
previously reviewed 36-pair finite-candidate audit, and 140 to current directly
viewed pairs. Another 40 transfers cover the superseded parent-only images.
`visual-evidence.json` records both images' dimensions, source hashes, placement
coordinates and all transfers. These transfers cover all 272 current pairs and
all 312 retained pairs. They transfer visual inspection only, never source
semantics, measured geometry or qualification.

The first three sheets show inherited sizes/padding, CSS-wide initial behavior,
priority selection and ordinary row order. The mixed border-box child retains
its authored width while the content-box child gains only its own padding;
purple descendant extents remain aligned. Sheets 3–5 show row/column reversal,
end-positioned automatic margins and responsive space-around/space-evenly
distribution. Sibling extents and gaps align in Chrome and native. Sheets 6–8
show zero-inset percentages responding to viewport changes, literal zero insets,
the zero-content padding floor and automatic minima/maxima. Padding bands and
visible overflow agree in those inspected pairs. The large add-side-first case
shows a thin green child row over the dark parent; sheet 9 shows the other large
controls' visible parent strips. Offscreen children are not visually qualified.
The earlier fractional pair inspection remains applicable by exact pixel
identity: native's thin coverage boundary differs from Chrome's terminal row.

## Tests, stack repair and regression

The full post-refactor suite passed **233 Rust tests** and **39 Node tests** before
the five fixture children and depth129 assertion were added. Logs are
`output/public-content-box-r1/tests-after-stack-refactor.log` and
`output/public-content-box-r1/transport-final.log`; strict TypeScript checking
also passed. The final current-fixture retests pass **3 Rust integration tests**
(`strengthened-tests-r2.log`) and **1 selected Node test**
(`strengthened-transport-r3.log`) under r2. That selected test covers 34×3
CLI/WASM file/map comparisons, 14 diagnostic/no-output cases, depth64/128 success
and depth129/130 diagnostics/no output. The 29 nonnumeric cases also compare exact
bytes/maps with independently authored border-box controls; all 34 repeat
deterministically. These are focused retests, not a second full-suite claim.

Depth validation uncovered a pre-existing default-WASM stack failure: the old
variable-recovery compiler traps on equivalent border-box depth128/130 requests.
The initial content-box compiler also traps there. The final compiler moves large
emission temporaries out of recursive frames, preserving source-map/emission
order and the depth limit. The unchanged default build stack now handles64/128
and returns depth-limit diagnostics with zero bytes at130 for both box-sizing
modes. The focused final transport test additionally covers first-invalid
depth129 for content-box. The direct/JS comparison and raw trap logs remain in
`output/public-content-box-r1/depth-comparison/receipt.json`; this historical
comparison was not rewritten to claim it tested129.

The parent separately verified **629/629 prior files and source maps** exactly
with r2 in `output/content-box-prior-regression-r2/manifest.json`. This regression
and the 34+5 content-box reproductions bind the stack repair to unchanged bytes.
No extra native rerender was used to imply a new visual result for identical
files.

Earlier failures remain in their logs: initial type inference, native stack
overflow before emitter repair, the incorrectly escaped test fixture, the obsolete
padding-rejection expectation, and the initial WASM trap. A final focused Node
attempt using the mutable target binary concurrently with a Rust build returned
a null subprocess status; its log is preserved as `strengthened-transport-r2.log`.
Its cause was not established. The subsequent frozen-r2-binary run passes in
`strengthened-transport-r3.log`. None of these failures are removed from evidence.

Next work must isolate the large-coordinate rectangle discrepancy and decide a
sound admission guard or ordinary composition, preserve the fractional paint
negative control, and independently investigate responsive mixed-unit sizing.
This checkpoint completes implementation/evidence bookkeeping; it does not
complete L12 or the compiler goal.
