# Ordinary content-owner composition

A private compiler composition repairs the retained large-padding child-size
failure without changing the runtime. It adds an unpainted ordinary layout
object inside the authored padding/paint object. All 272 geometry comparisons
pass in the revised experiment; all rendered pixels exactly preserve the public
baseline, including six existing fractional-edge failures. This is experimental
evidence for L12, not new public admission.

## Mechanism and corrections

The public compiler currently translates fixed content sizes into border-box
sizes. At large coordinates, adding and then subtracting padding can erase
content precision. The candidate keeps the outer translated preferred/min/max
sizes, padding, paint, source identity and sibling contribution. A separate
unpainted owner holds the authored content preferred/min/max sizes and original
child direction/distribution. Authored children attach to that owner; CSS
inheritance still uses the authored style. No browser geometry enters emission.

The initial candidate packed this single inner object in a normal column. It
repaired the original large-coordinate failure but broke automatic cross-axis
height stretching: a row parent's 120px-high padded child gave its inner owner
only 40px of intrinsic height. Two 20px children with space-evenly distribution
landed at y=10/30 instead of Chrome's y=30/70. Those 16 failed original/clone
frames remain in `output/content-owner-stretch-native-r1`.

Revision 2 chooses a normal row or column matching the authored participant's
parent axis. This preserves cross-axis stretch through the inner owner while
retaining the authored container's child flow. All six added stretch controls
then match the public baseline and Chrome.

Revision 3 updates the private exponent guard to follow both emitted owners.
The old guard can infer zero content from a rounded outer size even when the
new inner owner retains 0.03125px. Ten nested 1000000% dimensions then overflow
the runtime's multiply-before-scale calculation. Twenty compile-only controls
cover both axes, preferred/min/max cancellation, zero content and no padding.
The revised guard rejects six additional unsafe inputs before writing output.
These unsafe scenes were never imported or rendered. The guard change preserves
all 34 rendered Rive files and source maps exactly, so their revision-2 native
evidence transfers without another render.

## Evidence

| Implementation | Scenes / frames | Geometry passes | Pixel passes |
| --- | ---: | ---: | ---: |
| Frozen public baseline | 34 / 272 | 264 / 272 | 266 / 272 |
| Private revision 1, fixed-column packing | 34 / 272 | 256 / 272 | 250 / 272 |
| Private revision 2, parent-axis packing | 34 / 272 | 272 / 272 | 266 / 272 |
| Private revision 3, updated bounds | 34 exact files/maps | Transferred from revision 2 | Transferred from revision 2 |

The source corpus has 28 admitted original cases plus six separate stretch
controls. Five percentage-owner/padding diagnostic cases remain separately
preserved. Each native row uses pinned Chrome 153.0.8010.12 and the immutable
Rust Metal RasterOrdering renderer. Files compile once at 390×160; originals
and independent clones resize through 240×160 → 390×200 → 768×120 → 240×160.
Each complete implementation run passes 544 cyan/transparent clear checks and
170 repeated-frame identities. Thresholds were not changed.

The [independent browser audit](content-owner-browser-review.md) preserves
computed CSS, Typed OM, ResizeObserver sizes, rectangles and PNGs for all 33
original sources. The [public baseline review](content-owner-public-review.md)
records the before-state. The [visual review](content-owner-candidate-visual-review.md)
inspects all 15 unscaled comparison sheets: 50 complete Chrome/public/candidate
triples plus 222 verified complete-RGBA white-canvas transfers. Public and revised
candidate native pixels are identical in all 272 frames. The parent also
directly inspected the Chrome/failed/repaired stretch pair and the retained
fractional Chrome/native pair.

The large repaired child remains offscreen. Its binary32 width is
999999.8125px, matching Chrome's local ResizeObserver width; Chrome's projected
rectangle is 999999.75px. The residual 0.0625px is within the unchanged geometry
gate, not exact rectangle equivalence. Passing screenshots do not establish
offscreen geometry. At the retained fractional edge, Chrome paints a full dark
pixel while native paints partial coverage; the composition does not fix it.

## Public integration still required

The implementation exists only in frozen experimental source copies built by
`content-owner-candidate.py`. Public Rust/CLI/WASM/JavaScript behavior is
unchanged. The last public checkpoint remains 262 Rust/42 Node tests and 731
exact prior outputs; those suites were not needlessly repeated for this
validation-only change.

Before integration, replace the private full-Style clone used for numeric views
with a small explicit representation, bind guards to the final packing axis,
and update descriptor/provenance handling for the extra object. Revision 3's
copied provenance is not a synthetic-inner certificate; descriptor capture is
deliberately unavailable in the private harness. Preserve padding/alignment,
baseline, gap and nonlegacy-flex exclusions until separately justified. Add
public behavior, stack/depth, object/resource and transport regressions, and
classify every changed historical output. Expand meaningful auto/clamp and
composition boundaries as part of that integration.

The wrapper applies only when the owner's dimensions/bounds are point/auto and
padding is fixed. Existing percentage cases outside this composition retain
their current handling or diagnostics. Responsive percentage-plus-point owners,
nonzero percentage padding, fractional painting and broader composition support
remain open. No item is declared impossible. Backlog counts remain 13 qualified,
14 partial, 3 investigating and 69 pending across all 99 items.

## Reproduction and frozen artifacts

`content-owner-candidate.py NEW_OUTPUT` builds revision 1; add
`--packing parent-axis` for revision 2 and `--content-bounds` for revision 3.
It copies the public source, applies a checked emitter-only patch, and records
source, dependency, compiler, harness and toolchain identities. No runtime or
dependency overrides are used. `content-owner-preflight.py CASES BUILD OUTPUT`
checks exact inputs, deterministic bytes/maps and diagnostics. The existing
`check-public-baseline.mjs` accepts the resulting private compiler executable;
its historical public labels do not imply public support for this experiment.

Frozen revisions are under `output/content-owner-candidate-build-r1` through
`r3`; preflight, public, browser and native receipts retain every command and
source. `content-owner-bounds.py NEW_OUTPUT` reproduces the compile-only
overflow controls against those frozen compilers. Run
`python3 tools/html-to-riv/validation/content-owner-checkpoint.py` to verify
source patches, emitted files, native geometry, complete image transfers,
guard results, all 99 statuses and runtime immutability. The tracked aggregate
receipt is `content-owner-checkpoint-receipt.json`.
