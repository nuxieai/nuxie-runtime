# P06 linear gradient implementation and qualification plan

Status: public v26 candidate, undergoing final qualification before publication.
Single linear-gradient backgrounds are implemented with retained resize semantics,
checked metadata, exact premultiplied stop interpolation and tiled border paint.
The frozen full run passes 8,062 checks with 8,052 audited image pairs; current
renderer migration, final numeric boundaries and retained performance remain.
See the P06 row in [BACKLOG.md](../BACKLOG.md) for current status. Radial gradients
remain unsupported. The dated entries below preserve the implementation history;
earlier rejection and pending statements describe their recorded stage.

## Semantics to preserve

CSS Images 3 defines gradients using a direction and an ordered stop list.
Omitted positions are distributed; decreasing positions are moved forward;
coincident stops retain a sharp transition. Pixel/percentage mixtures must be
resolved again when layout changes. Transparency interpolation needs an explicit
check, since a transparent black stop must not introduce a dark fringe.
Source: [CSS Images 3, linear gradients and color stop fixup](https://www.w3.org/TR/css-images-3/#linear-gradients).

Start by qualifying single nonrepeating backgrounds with the existing color
language, default/cardinal/corner directions, angle units, omitted/percentage/
pixel stops, hard transitions, out-of-range positions and transparent colors.
The initial 16-scene Chrome corpus covers these distinctions at three widths.
This is an implementation order, not completion of the entire P06 item.
Additional acceptance work includes multiple-position stops, transition hints,
explicit interpolation spaces, shorthand reset/cascade/custom properties,
background-color underneath translucent gradients, border-box versus positioning
area, rounded clipping, nested opacity, text/image compositions and degenerate
sizes. Each must be implemented or explicitly documented with retained evidence;
no silent acceptance of an approximation. Repeating gradients and multiple
background layers need separate documented qualification. Radial gradients remain
P07; CSS transforms remain P11. No new syntax is admitted by this document.

## Existing implementation seam

Compiler `src/lib.rs` currently emits a Fill and SolidColor beneath the layout
owner. Runtime `shapes/paint/linear_gradient.rs` retains GradientStop children
and uses numeric start/end coordinates; `LinearGradientBase` serializes those
coordinates. Its current application path derives geometry from the stored
endpoints and transforms, not the layout owner's current width/height.

Therefore writing initial viewport endpoints alone cannot qualify responsive CSS.
Investigate a checked, versioned layout-owner policy that resolves gradient
geometry/stops after each layout update, retains source stop order, dirties the
paint correctly, survives cloning and clears atomically. Preserve ordinary Rive
gradient behavior when that policy is absent. A normalized shape transform is an
alternative only if it preserves CSS angle and corner semantics under arbitrary
aspect-ratio changes; diagonal tests must discriminate this.

## Validation gates

- Record public rejection before implementation; keep that frozen compiler.
- Compare resolved endpoints/stop fixup with independent analytic controls and
  pinned Chrome pixels. Inspect alpha and hard edges, not only average error.
- Compile once, import and clone, resize both through 240/390/768/240, and compare
  geometry/pixels without browser-baked rectangles or widened thresholds.
- Check native/WASM Rive bytes, source maps and requirements; malformed syntax,
  missing host capabilities, malformed targets and resource limits must reject.
- Verify source/renderer/artifact hashes, visual review coverage and full native
  regression before marking P06 qualified.

Initial evidence: `output/playwright/html-to-riv/linear-gradient-initial-admission/receipt.json`
and `linear-gradient-initial-oracle/oracle.json`. Chrome captures are references,
not native qualification or a visual inspection claim.

### P06 responsive gradient geometry and stop fixup implemented — 2026-09-10

Added runtime css_linear_gradient module resolving cardinal/angle/corner endpoints from current box dimensions, plus source-preserving omitted/decreasing/pixel/percentage stop fixup. Four standalone Rust tests pass, covering diagonal endpoints, corner midpoint constraints across aspect ratios, resize-dependent hard stops, out-of-range positions and invalid/degenerate inputs. Runtime cargo check passes. A separate harness compiles the runtime helper and compares its predicted red/blue ramp against120 pinned Chrome samples across eight directions and three widths; all pass the declared two-channel-unit quantization bound. This is geometry evidence only, not native rendering or public gradient admission. Parser, checked runtime paint update/clone seam, public transport/parity and pixel qualification remain. Receipt: output/playwright/html-to-riv/linear-gradient-geometry-progress.json.

### P06 specified gradient component parser tested — 2026-09-10

Added an unconnected gradient parser retaining direction, currentColor, px/em/rem/percentage/omitted positions and expanded double-position color stops. Supports existing named/hex/RGB/HSL colors, angles/cardinal/corner directions, comments and256 expanded stops. Three tests pass including all16 prospective Chrome inputs and malformed/unsupported/limit controls. Initial test caught nonfinite numeric token serialization producing a finite angle; validation now rejects before serialization and the original failure is retained. Independent pinned Chrome controls confirm single-color double-position gradients are valid, as are hints and explicit color spaces; the latter remain explicit parser exclusions pending implementation, not claimed invalid CSS. Public admission remains gated by missing runtime paint/host transport and qualification. Receipt: output/playwright/html-to-riv/linear-gradient-parser-progress.json.

### P06 computed gradient cascade integrated behind emission gate — 2026-09-10

Style retains a gradient separately from background color. Background shorthand resets both; background-color preserves the image; background-image none/initial/unset clears it. Relative stops resolve against the final font size (rem uses the profile16px root), so explicit inheritance copies computed lengths. CurrentColor stays symbolic for final color resolution. Gradient function syntax is preserved past the ordinary color-only serializer; variable fallbacks retain valid gradients and invalidate plain red/length/duplicate-none image values to unset. Public compilation explicitly rejects remaining gradients until responsive paint transport exists. Initial failures (test diagnostic-vector access, nested-function serialization) are preserved; corrected focused7/7 and complete library55/55 pass. P05 frozen full run remains independent and live. Receipt: output/playwright/html-to-riv/linear-gradient-cascade-progress.json.

### P06 checked gradient paint description and extended stop domain — 2026-09-10

Runtime inspection confirms ordinary Rive LinearGradient clamps stop positions to0..1. Added checked CssLinearGradient retaining authored direction/colors/positions with matching count and2..256 stop limits. Resolution first performs CSS stop fixup, then expands endpoints to contain out-of-range stops and normalizes them for the shader, preserving their colors inside the box. Seven standalone tests pass, including the prior four geometry controls, out-of-range color coordinates, clone/repeated resize and malformed/empty inputs. This does not yet install or draw CSS gradients. Layout drawing, positioning-area/border semantics, atomic host transport and native interpolation qualification remain. Receipt: output/playwright/html-to-riv/linear-gradient-paint-policy-progress.json.

### P06 experimental layout gradient drawing and atomic installation — 2026-09-10

LayoutComponent now retains optional checked gradient paint, clones authored values, and paints it after solid fills/before borders using current dimensions and rounded background geometry. The Artboard occurrence installer validates all non-root layout targets and duplicates before replacing the policy map; clearing one occurrence leaves its clone intact. Integration test passes eight original/clone resize draws with expected recorded endpoints, invalid-target atomicity, clear and clone independence. Early adapter type build failures and test-held factory borrow failure are preserved; corrected run passes. This is recorded-runtime evidence only: native interpolation, positioning-area/border/repeat behavior and public transport remain unqualified, and public gradient emission remains gated. Receipt: output/playwright/html-to-riv/linear-gradient-runtime-installation-progress.json.

### P06 exterior stops fixed; alpha interpolation mismatch isolated — 2026-09-10

Direct browser/native inspection isolates decreasing-stop failure: native lost the first red color before coincident leading stops. CSS resolution now adds equivalent constant-color endpoint stops before native normalization, retaining hard transitions without approximation. Eight pure tests and two runtime recording tests pass. Native128-frame rerun improves to112 passing; only16 transparent/mixed-alpha frames fail. Eight preserved interior samples confirm Chrome premultiplied interpolation versus native straight-color interpolation (each matches its respective analytic formula within2 channel units). Shader source also explicitly premultiplies after interpolating unmultiplied colors. Next requires a checked CSS interpolation path while preserving ordinary Rive gradient behavior; public admission remains gated. Receipt: output/playwright/html-to-riv/linear-gradient-interpolation-progress.json.

### P06 explicit premultiplied gradient transport tested — 2026-09-10

Added opt-in Factory::make_premultiplied_linear_gradient returningNone for unsupported/invalid input, forwarding through persistent factory wrappers. Recording validates finite coordinates, matching stop counts and normalized monotonic positions, and emits makePremultipliedLinearGradient. Stream parser retains a distinct resource and replay refuses unsupported factories; ordinary gradient commands remain unchanged. API regression passes; stream19/19 includes round-trip through persistent recording, unsupported rejection, no output for malformed inputs and ordinary-mode controls. Initial new tests omitted the empty frame marker; preserved failure and corrected fixtures. Native mode implementation and CSS draw selection remain pending, so transparency residuals are not claimed fixed. Receipt: output/playwright/html-to-riv/linear-gradient-premultiplied-transport-progress.json.

## Native interpolation identity

Added an immutable-before-publication interpolation mode to native Gradient.
Ordinary Rive constructors retain straight-alpha interpolation; the checked CSS
constructor selects premultiplied interpolation. Opacity modulation preserves
the mode, and complex-ramp cache equality/hashing includes it so identically
authored straight and premultiplied ramps cannot share incompatible pixels.
Two renderer-metal unit tests pass, covering separate cache entries through
1/.5/0/.5 modulation and invalid checked inputs. Evidence:
`output/playwright/html-to-riv/linear-gradient-native-mode-progress.json`.

This is internal preparation only: NativeMetalFactory does not yet expose the
new constructor, and neither shaders nor public CSS emission are enabled.
The two-texel fast cache also needs mode separation or routing CSS ramps through
the complex path. Ramp generation must premultiply before interpolation; both
atomic and ordinary path sampling must handle that mode before coverage and
advanced blending. Metal includes the generated GLSL fragments through its
wrapper, and build.rs recompiles those embedded sources.

## Native shader wiring under validation

CSS ramps now route through the complex cache with a separate location marker.
The ramp vertex shader premultiplies colors before interpolation. The existing
normalized row coordinate carries a +1 tag for CSS ramps; both ordinary-path
and atomic-path samplers remove the tag before texture access and unpremultiply
the sampled color before existing coverage/advanced-blend processing. Ordinary
Rive ramps retain the original two-texel optimization and sampling semantics.
The NativeMetalFactory checked method and experimental layout draw are wired.
Shader compilation and subsequent pixel comparisons are still required; see
`output/playwright/html-to-riv/linear-gradient-native-shader-progress.json`.
No public CSS acceptance or new pixel qualification is claimed.

The shader and replay builds completed successfully. Fresh r4 recordings pass
both runtime tests, and all128 lifecycle frames pass against Chrome153.0.8010.12
without tolerance changes. The two390px alpha sheets were directly inspected:
the gray transparent fringe is gone and mixed-alpha progression matches.
Full visual coverage remains incomplete. Receipt:
`output/playwright/html-to-riv/linear-gradient-native-shader-progress.json`.
The old112/128 r3 evidence remains preserved. This is still experimental
injected runtime paint, not public compiler qualification.

## Gradient coexistence and alpha visual controls

The frozen experimental P06 renderer passes16 synthetic images/160 analytic
samples exercising ordinary straight-alpha Rive ramps alongside premultiplied
CSS ramps. Both native paths, reversed insertion/draw orders, two-stop/simple
and three-stop/complex ramps, and host modulation1/.5 are covered. This tests
cache separation and preservation of existing straight-alpha behavior; it does
not qualify arbitrary host transforms or every renderer backend. Evidence:
`output/playwright/html-to-riv/linear-gradient-native-coexistence/receipt.json`.

All six distinct transparent/mixed-alpha browser/native/difference views at
240/390/768px have now been directly inspected and recorded. Remaining initial
gradient views still need inspection/artifact audits before public promotion.
The full128-frame automatic pass remains experimental runtime injection.

## Gradient diagnostic artifact audit

The new audit freshly recompiles all16 stripped diagnostic inputs with the
frozen baseline compiler and requires identical Rive bytes and runtime
requirements. It checks the independent authored gradient metadata, injected
pixel-bound owners, all128 original/clone frames at240/390/768/240px, source
geometry, stream identity and premultiplied commands, full image hashes, and
frozen renderer identity. Qualification remains runtime-experiment-only.
All16 scenes/128 frames pass. Twelve mutation controls reject missing or
duplicate frames, changed renderer/image/stream hashes, a false public claim,
changed paint or compiler CSS, missing pixel policy, wrong resize/geometry,
and missing clone identity. Source evidence is never mutated by the tests.

Evidence: `output/playwright/html-to-riv/linear-gradient-initial-lifecycle-r4/gradient-artifact-audit.json`
and `output/playwright/html-to-riv/linear-gradient-artifact-audit-controls.json`.
This closes the initial artifact audit, not the remaining42-view visual review
or public compiler promotion.

## Completed initial inspection exposes hard-stop residual

All48 distinct browser/native/difference views have been directly inspected.
The review receipt covers all128 lifecycle frames with exact full-image
transfers for repeated/original-clone views. Coverage is not qualification:
the768px hard-stop and decreasing-stop fixtures expose texture filtering
across a discontinuity. Chrome paints solid colors adjacent to the boundary,
but native produces mixed pixels. The hard-stop samples atx383/384 differ by
38 channel units; decreasing-stop x529 differs by161. Narrower initial widths
do not reproduce these sampled failures.

A new local boundary gate checks both fixtures across all16 lifecycle frames.
It fails4/16 (both768px scenes on original and clone), even though the existing
aggregate128-frame gate passes. No thresholds were widened and these failures
are not waived. See `linear-gradient-hard-stop-residual.json` and
`linear-gradient-hard-stop-gate-r4.json` under output/playwright/html-to-riv.
Fix discontinuity sampling before treating initial gradients as qualified.
Public CSS remains gated.

## Exact stop table preparation

Fixed512-texel ramp sampling cannot preserve arbitrary CSS discontinuities at
all responsive widths. Increasing its resolution only moves the failure. Added
a CPU table representation carrying original ARGB bytes and fullf32 stop bits
in separate RGBA8 rows. Coincident and narrowly separated stops stay distinct.
Two native renderer tests pass for byte/bit retention, upper-bound duplicate
selection, invalid input, and capacity including256 authored stops plus two
CSS endpoint-padding stops.

This table is not yet allocated or sampled by the renderer. The next step must
account for additional CSS rows without reducing ordinary Rive ramp capacity,
use texel-center lookups, recover stop positions, and interpolate the selected
segment in premultiplied space. The second-row coordinate must use actual
allocated texture height, not assume that the ramp draw viewport height matches
the allocation. The existing four boundary failures remain unresolved.
Receipt: `output/playwright/html-to-riv/linear-gradient-stop-table-progress.json`.

## Exact table native wiring

CSS gradients now allocate two rows while ordinary complex ramps retain one
row and ordinary simple ramps retain their two-texel fast path. Separate row
accounting updates capacity checks and flush height. Flat color spans upload
raw words without premultiplication. Two header columns hold stop count and
the position-row coordinate computed from actual allocated texture height.
This permits510 stops, exceeding the compiler's256 plus two padding stops.

Both native fragment paths decode positions from exact texel-center reads and
perform a bounded upper-bound search. The selected adjacent colors interpolate
in premultiplied space; duplicate positions select the final coincident stop.
The native shader build and two table tests pass. Replay build and actual
boundary/visual qualification remain pending; no fix claim yet.
Receipt: `output/playwright/html-to-riv/linear-gradient-exact-table-progress.json`.

## Runtime pipeline failure discovered after offline compilation

The first exact-table replay aborted before drawing: Metal reported a nil
vertex function. A minimal solid rectangle, ordinary gradient, and CSS gradient
all fail in the exact-table frozen binary and pass in the previous frozen
binary. This isolates pipeline construction, not stop sampling. Instrumentation
identified a renamed draw vertex export, IC instead of HC, rejected by the
hand-written native NSString lookup. The existing static-function-literal test
reproduces the defect (red receipt: linear-gradient-function-name-red.log).

The build now generates function NSString literals from actual shader exports,
preserving immortal borrowing while tracking minifier names. Temporary debug
logging was removed. The regression-test rebuild is in progress; rerun the
minimal replay and all gradient controls after it passes. Failed r5 artifacts
and pipeline repros remain preserved. No hard-stop fix qualification yet.

## Exact stop-table validation results

Generated function-name regression passes; all three minimal pipeline repros
(solid, ordinary gradient, CSS gradient) now render successfully. Temporary
debug logging is removed. With the frozen exact-table-r2 renderer, all128
initial Chrome/native frames pass and the targeted hard-stop gate passes16/16,
resolving the four r4 failures without threshold changes. Coexistence controls
pass16 images/160 samples on both native paths. The fresh artifact audit passes
16 scenes/128 frames. Both corrected768px boundary sheets were directly viewed:
the prior blended columns and highlighted difference stripes are gone.

Other changed images still need review. Table capacity/performance, wider
regressions, and public compilation/parity remain unqualified. Receipt:
`output/playwright/html-to-riv/linear-gradient-exact-table-progress.json`.
The failed r5 pipeline run and r4 boundary reproducers remain preserved.

## Capacity and broader native regression

The frozen exact-table-r2 renderer passes14,336 analytic samples across both
native paths with adjacent tables containing2/3/258/510 stops and an ordinary
complex ramp in one frame.511 stops are rejected before rendering. Evidence:
`output/playwright/html-to-riv/linear-gradient-table-capacity/receipt.json`.

The first full renderer library run returned457 passed/17 failed/6 ignored.
Nine failures require missing external fixtures. Two ownership-test failures
pass when rerun individually with one test thread. The remaining failures
exposed additional stale names in standalone resource/atlas/preload helpers.
Those now use generated exports for generated libraries; the standalone
specialization compiler still uses its separate fixed fixture-source names.
After corrections, all170 native_metal module tests pass serially. This is
not a passing full suite; fixtures and broader regression remain. Earlier
failed runs are preserved. Latest source helper changes are not covered by
the already frozen exact-table-r2 binary receipt.
Receipt: `output/playwright/html-to-riv/linear-gradient-capacity-and-renderer-review.json`.

## Full renderer library regression completed

Restored missing fixtures using tools/fetch-test-assets.sh with checksum
verification and supplied the existing upstream checkout for parity tests.
With one test thread, the complete renderer-metal library suite passes474 tests
with6 intentionally ignored. The earlier17-failure run is preserved; its
fixture/ownership/name issues are now accounted for by the passing serial run.
The upstream commit and six parity fixture hashes are recorded in
`output/playwright/html-to-riv/linear-gradient-full-renderer-receipt.json`.

A fresh replay build is byte-identical to exact-table-toolchain-r2. Inspection
of module configuration confirms the most recent atlas/resource/preload helper
fixes are test-only; canonical replay pixel evidence remains applicable. This
does not finish the r6 changed-image review, performance qualification, broader
gradient combinations, or public compiler transport/parity.

### Exact-stop gradient visual audit complete — 2026-09-10

Corrected runtime experiment r6 passes128/128 Chrome/native original-clone frames and16/16 targeted hard-stop frames. Full-resolution review now covers every frame:39 direct views,79 within-run exact-image transfers and10 cross-run exact-input/full-image transfers; combined audit reports zero remaining. Five review-identity tests include eight semantic/RIV mutations and stale/wrong-path/failed audit rejection. Stop-table capacity controls pass14336 samples; full renderer suite passes474 tests with6 ignored. Public gradient CSS remains gated: transport/emission, native/WASM parity, performance and broader composition qualification remain. Evidence: `output/playwright/html-to-riv/linear-gradient-exact-table-progress.json`, `linear-gradient-initial-lifecycle-r6/visual-coverage.json`, and `linear-gradient-review-identity-controls.json`.

### Gradient portable contract foundation — 2026-09-10

Added Rust version26 capability `layout-css-linear-gradient-v1` and occurrence payloads retaining corner/degree directions, unpremultiplied ARGB colors, omitted stops and signed pixel/percentage positions. Validation enforces2–256 stops, finite numeric values, unique non-root LayoutComponent targets and capability/version agreement. Four gradient contract tests and six opacity regression tests pass; the full library suite also passes. Version26 can coexist with opacity without weakening prior version checks. Public CSS emission remains gated while host installation, JS types and native/WASM parity are connected. Receipt: `output/playwright/html-to-riv/linear-gradient-contract-progress.json`.

### Gradient checked recording host and JS types — 2026-09-10

The probe validates version26 before drawing, converts retained gradient values into checked runtime paints, and installs occurrence targets atomically. Host tests pass four responsive widths,13 malformed-manifest controls and explicit missing-capability rejection; runtime tests pass original/clone resize and replacement/clear controls. JavaScript version26 declarations expose exclusive direction/position variants and pass typechecking. This qualifies the recording-host adapter only; public CSS emission, native/WASM parity, public native lifecycle pixels and unsupported-backend preflight remain. Receipt: `output/playwright/html-to-riv/linear-gradient-host-progress.json`.

Gradient host regression completes24/24 after rebuilding the probe with required `native-glyph-controls`. Initial21/23 attempt is retained: both failures explicitly rejected the missing native-glyph build feature. Updated receipt: `output/playwright/html-to-riv/linear-gradient-host-progress.json`.

### Public gradient candidate and stack correction — 2026-09-11

Public version26 emission is now connected. The previous gate is removed in the working tree, without claiming qualification. Six gradient transport/public tests and16-scene original-clone geometry test pass. Independent expectations exposed percentage fraction round-trip error30→30.000002; preserving authored percent token text fixes it. Negative angle expectations use equivalent canonical degrees. Large per-element emission frames caused default-stack failure on plain nested divs; finishing per-element emission before recursion fixes all46 contract tests and preserves65 accepted/66 rejected nested elements. Native/WASM/public pixel and broader composition checks remain. Receipts/logs: linear-gradient-stack-fix-receipt.json, linear-gradient-public-tests-r4.log in output/playwright/html-to-riv.

### Public gradient initial validation passes — 2026-09-11

Fresh native/WASM parity passes6 tests; public128 original-clone resize frames pass pinned Chrome geometry/native pixels. Five depth/element/hidden/source-target boundary tests pass. Fresh public artifact audit passes16scenes/128frames and23 negative controls reject mutations. All128 authored-source/full-image pairs match reviewed runtime diagnostics; specialized visual transfer audit remains pending. New composition corpus has54 Chrome reference images across18 accepted cases, including default image repetition beneath transparent borders. Broader semantics/performance/full regression remain; P06 is active, not qualified. Evidence: output/playwright/html-to-riv/linear-gradient-public-integration-progress.json.

### Gradient composition defect confirmed — 2026-09-11

Initial public visual transfer audit completes128/128 with zero new inspections, based on exact authored HTML/CSS and full browser/native PNG identity plus independent public provenance. Broader composition native r1 compares54 views:30 pass,24 fail, all geometry passes. Eight cases fail at each width, including transparent/translucent/corner/wide/rounded borders, content clipping and opacity overlap. Inspected horizontal transparent-border pair confirms Chrome repeats endpoint colors beneath borders while current native paint clamps. Correct fix needs2D tile coordinates wrapped before scalar gradient projection; simply repeating scalar t cannot represent diagonal/corner tiles. Evidence: linear-gradient-public-lifecycle-r1/visual-public-gradient-transfer.json and linear-gradient-composition-native-r1/receipt.json under output/playwright/html-to-riv. P06 remains active.

### Repetition implementation candidate — 2026-09-11

Public compiler regression before tile changes completes453 tests across79 suites. New checked tiled-premultiplied paint transport preserves existing untiled commands; API36 and stream22 tests pass. Candidate native gradient retains immutable tile metadata through modulation; runtime requests tiling for borders, shader auxiliary data preserves2D UV and wraps before projection. Native build/pixel verification pending; original24/54 composition failures remain evidence. Initial performance capture completes80 measured runs plus16warmups, recording end-to-end replay only (not GPU/frame timing). Evidence: linear-gradient-public-module-receipt.json, linear-gradient-tile-transport-tests.log and linear-gradient-performance-initial/receipt.json under output/playwright/html-to-riv.

### Tiled composition pixels corrected — 2026-09-11

Frozen tiled toolchain builds and solid smoke pass. Composition r3 now54/54 automatic comparisons pass, versus30/54 before; targeted border gate36/36 samples passes and detects the earlier3 fractional-border visual failures. Transport58 and coordinate3 tests pass. Corrected full-image review/artifact audit, tiled original-clone composition lifecycle, backend/modulation/performance and full regression remain. Initial128-frame replay is running. Evidence: output/playwright/html-to-riv/linear-gradient-tile-progress.json.

### Tiled lifecycle and renderer regression pass — 2026-09-11

All18 public composition scenes compile once and pass144 original/clone Chrome/native frames plus fresh lifecycle artifact audit. Every authored-source/full-image pair equals a reviewed static composition; final transfer audit underway. Corrected static audit passes54/54,27direct/27exact transfers. Full native renderer suite479pass0fail6ignored; host synthetic16images/1328analytic samples pass across bothMetalpaths/affine/modulation/sharedstops. Current compiler regression and broad integrated native/performance qualification remain. Receipt: output/playwright/html-to-riv/linear-gradient-tile-progress.json.

### Public gradient focused validation complete; broad integration running — 2026-09-11

Current compiler regression passes454 tests across80 suites, with no failures or ignored tests. All144 composition lifecycle frames now have audited exact-source/full-image review transfer from corrected static evidence; all4 stale/mutated evidence controls reject. Together with the initial128 public frames, focused public lifecycle coverage is complete. The normal browser regression now registers16 initial and18 accepted composition fixtures (102 additional comparisons); ten deliberately unsupported composition inputs remain rejection-only cases. Frozen tiled-toolchain full native regression is running against Chrome153.0.8010.12. P06 remains active pending broad regression, backend and performance qualification. Native Metal replay explicitly rejects MSAA mode, so shader compilation alone is not runtime MSAA evidence.

### Backend fill semantics and tile arithmetic boundary — 2026-09-11

The alternate frozen Metal comparison passed45/54 views. A minimized gradient-free evenodd ring fails identically before and after tiling; reversing its inner contour with nonzero filling restores the hole. Source inspection confirms forced ClockwiseAtomic mode deliberately overrides fill rules. Ordinary Atomics is now an implementation candidate with explicit selection and automatic non-raster fallback preserving fill semantics; build and native replay are pending. MSAA remains rejected by the Metal host.

A separate raw stream with finite positive tile width1e-40 panics at inverse-transform setup, while the otherwise-identical width10 control renders. Shared checked tile admission now rejects unrepresentable reciprocal/offset arithmetic in recording, stream parsing and the native constructor, while retaining representable small values. Regression tests and rebuilt repro verification are pending. This is raw renderer API evidence, not a claim that authored CSS reaches subnormal dimensions after pixel snapping. Reproducers: output/playwright/html-to-riv/linear-gradient-atomic-border-repro and linear-gradient-tiny-tile-repro-r1.

### Ordinary atomics and numeric boundary replay — 2026-09-11

Ordinary Atomics passes54/54 Chrome composition comparisons and18 deterministic repeats; changed-image review remains underway. The original tiny-tile repro rejects cleanly after shared inverse-transform validation. Further checked-constructor work preserves ordinary coefficients when valid, recovers short/long finite line coefficients using f64 intermediates when the f32 squared-length calculation fails, and rejects unrepresentable tile projections before drawing. One focused coefficient unit test and five actual raw-stream replays pass: ordinary control unchanged, short-line blue interior correct, tiny/coincident/overflowing-projection inputs return errors rather than panic. Frozen corrected binary: linear-gradient-coefficient-toolchain-r1. Full renderer regression is running. This does not qualify arbitrary host transforms or broad extreme-value precision.

### Atomic image-edge correction audited — 2026-09-11

Correcting swapped image-AA inset axes fixes the empty opacity-group and direct fullcanvas-image reproducers without masking alpha with an opaque clear. Square/transposed/rotated controls isolate the cause; shear/rotation/affine comparisons retain the existing pixel gates. Corrected ordinary Atomics has54/54 composition comparisons,18 deterministic repeats and complete audited visual coverage (3 corrected images inspected again,51 exact transfers). Full renderer480 tests pass,6 ignored. Evidence: `output/playwright/html-to-riv/linear-gradient-atomics-image-edges-validation-r1.json` and `linear-gradient-image-edges-full-renderer-tests.log`. The new frozen renderer is `linear-gradient-image-edges-toolchain-r1`; the ongoing8062-check Chrome regression intentionally retains its original tiled toolchain. A separate exact-image migration will verify newer renderer changes against its retained streams after completion and visual audit.
