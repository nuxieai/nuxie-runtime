# Percentage spacing (L15)

Status: experimental implementation; all 120 focused Chrome scenes pass geometry
through 960 original/clone viewport checks. Full compiler297 and Taffy111 tests
pass. Fresh candidate native/WASM build and rendered pixel qualification remain
pending; original rounding pixel failures are preserved and unwaived.
The frozen L14 compiler rejects the first fixture with `unsupported-value`.
The failing request, diagnostic and launch receipt are preserved under
`output/playwright/html-to-riv/percentage-spacing-investigation`.

Target syntax: nonnegative percentage values in physical padding and margin
longhands and one-to-four-value shorthands, mixed with existing lengths; margin
retains `auto`. Preserve percentages for runtime layout and same-scene resizing.
Negative margins remain L16. Grid, logical properties, new math syntax and
writing modes are outside this item. Percentage spacing remains excluded until
implementation and qualification are complete.

The initial 64 fixtures cover four flex directions, both box-sizing values,
definite and content-derived host width, percentage padding, percentage margins,
mixed lengths and percentage spacing combined with automatic margins. Child and
sibling bounds expose reference-axis mistakes and cyclic sizing effects.
Chrome references are captured at 240/390/768 by `content-auto-oracle.mjs` into
`percentage-spacing-initial-oracle`; capture session 62216 completed successfully: 64 scenes / 192 views on Chrome
153.0.8010.12; the receipt records the oracle SHA-256. These fixtures are separate from the running full L14 corpus.

Next: inspect captured cyclic/reference-axis results, map runtime percentage
units and container reference behavior, add public compiler tests, implement
spacing values without resolving them to compile-time pixels, then validate
native/WASM artifacts, original/clone resize sequences, and real native pixels.
Expand nested, wrapping, min/max, image/text composition and cascade cases before
qualification. Do not infer runtime compatibility from the existence of unit fields.

Runtime seam found during source inspection: `LayoutComponentStyle::apply`
preserves padding units, but its `margin_unit` closure converts every margin
unit to Point when `context.has_layout_parent` is false. Consequently unit-field
serialization alone cannot establish top-level percentage-margin fidelity.
Added eight root fixtures (padding, margin, mixed lengths, automatic margin;
row/column child flow) to capture 24 Chrome references independently. Investigate
whether compiler root wrappers provide a layout parent and test actual imported
root behavior before deciding whether this runtime branch needs changing.

Compiler seam: padding is currently `[f32; 4]`; margins are `Auto | Px(f32)`.
The publisher writes padding unit 1 unconditionally and margin unit 1/3.
L15 needs retained percentage values in both computed style and publication,
including inheritance, physical longhands and shorthands. Existing substitution
validation must be checked for consistency; a parser-only patch is insufficient.

Implementation evidence: `percentage-spacing-investigation/implementation-receipt.json`.
Padding now retains `Spacing::Px | Percent`; margins retain `Auto | Px | Percent`.
Physical shorthand/longhand values publish Rive percentage unit 2 directly,
with finite nonnegative percentages in [0, 10000], consistent with dimension limits.
Explicit inheritance, custom properties, mixed em lengths, important precedence,
reset keywords and profile bounds have public tests. Initial and root Chrome
oracles pass 72 scenes × original/clone × four viewport updates = 576 checks at
0.1px tolerance. The eight authored-root fixtures pass without changing runtime
margin handling; the earlier source concern did not reproduce for compiler roots.
New fixtures are included in the native/WASM parity corpus, not yet run on a
fresh percentage-spacing WASM build. Pixel qualification remains pending.

L15 composition test is red: 176 coordinate mismatches across the 32-scene original/clone resize corpus. Examples: row wrap tail.y=109.171844 versus Chrome225.20313 at240; intrinsic row card.width=60 versus Chrome72.34375. Full public regression stopped with exit101 at this new test (27 sibling oracle tests passed). Preserve `percentage-spacing-validation/public-full-v2.log`; investigate wrapped-line free-space allocation and cyclic percentage padding/intrinsic sizing separately from the eight pixel-rounding failures. Qualification remains incomplete.

L15 diagnosis correction: wrapped-line mismatches came from the public oracle helper omitting installation of the emitted align-content policy. Added the same installation as the native probe; wrapping errors disappear. The frozen composition native replay confirms 96 comparisons with 12 geometry failures, all intrinsic-container views, and 11 pixel failures. Corrected public test retains 144 intrinsic coordinate mismatches. A separate column padding reference-axis candidate is under test; no runtime fix is qualified yet. Evidence: `percentage-spacing-validation/composition-align-content.log` and `percentage-spacing-composition-native/replay.json`.

The L15 column padding-axis experiment completed with the same 144 intrinsic coordinate mismatches and was reverted exactly to its saved source. No runtime change retained from this experiment. The next target remains intrinsic percentage sizing; the public helper align-content correction is retained.

L15 reduced reproducer: 16 scenes / 48 Chrome views / 128 original-and-clone viewport updates isolate control, padding, margin, combined spacing, fixed parent, disabled shrink, content-box and nested cases in both directions. Baseline reports 184 coordinate mismatches; row padding loss reproduces even with a fixed parent and disabled shrink. `percentage-spacing-validation/minimal-investigation.json` records the evidence. A parent-width-preservation measurement experiment is running against all four percentage-spacing oracle tests; not yet qualified.

L15 parent-width experiment is terminal: initial/root oracles still pass; minimal coordinate mismatches fall from184 to64 and composition mismatches from144 to48. The candidate remains unqualified with failing tests preserved; investigate the remaining column cases and require full regression before qualification. Log: `percentage-spacing-validation/parent-width-experiment.log`.

L15 geometry candidate passes all four Chrome-oracle tests: 120 scenes / 960 original-and-clone viewport updates at unchanged 0.1px tolerance. Preserving parent width for auto-width row measurement and remeasuring content-sized columns after inline width resolves removes the focused geometry failures. This is a broad candidate, not a qualified runtime change: full compiler regression is running (97293), and dedicated runtime checks, fresh parity, native pixel reruns and visual review remain pending. Evidence: `percentage-spacing-validation/geometry-candidate-receipt.json`. Existing frozen pixel failures remain unwaived.

L15 fresh geometry-candidate validation is terminal: native/WASM11 tests pass; native360/360 geometry comparisons pass,347 pixel comparisons pass and13 pixel failures remain unwaived (initial6, root2, composition5, minimal0). Focused rounding references add27 scenes/81 Chrome views; all agree with truncating each percentage edge to1/64px before summation. A dedicated public rounding test uses stricter0.001px tolerance without changing standard gates. Evidence: `percentage-spacing-validation/geometry-native-receipt.json`.

Rounding implementation experiment: resolve each percentage edge against the
current containing inline size, then floor independently to 26.6 precision
before padding/margin summation. Preserve literal length edges and auto margins.
The initial experiment could not compile because Rect lacks zip_map; the revised
local helper operates on four explicit edges. Test log:
`percentage-spacing-validation/rounding-experiment-v2.log` (session91459).
This diagnostic implementation currently affects generic percentage edges and
must not be considered qualified. Before shipping, make CSS precision explicit
in the compiler/runtime capability contract, preserve non-opted-in runtime
behavior, test cloning/target validation and native/WASM contract parity, and
include percentage spacing in requirements-version compatibility tests. Also
scope the row/column measurement changes and assess recursive measurement cost
on nested columns. Pixel and full regression receipts for the previous frozen
candidate do not qualify this rounding experiment.

L15 rounding experiment passes all five oracle groups:147 scenes /1176 original-and-clone viewport updates, including27 rounding scenes at stricter0.001px tolerance. Independent edge truncation removes the numerical rounding reproducer. This is still a generic diagnostic implementation: explicit CSS opt-in, requirements contract, measurement scope/performance review, new parity, native pixel/visual checks and full regression remain required. Evidence: `percentage-spacing-validation/rounding-candidate-receipt.json`. The13 frozen-candidate pixel failures are not yet claimed fixed.

L15 explicit opt-in implemented: requirements version16 adds `layout-css-percentage-spacing-v1` and unique `layout_percentage_spacing` occurrence IDs. Rust validation checks capability/version/target consistency; compiler emission, native probe installation, cloned runtime state and JavaScript API types are updated. Taffy defaults remain off; the policy enables measurement and percentage edge precision across the opted-in solve tree. All five focused oracle groups still pass (147 scenes/1176 instance-viewports), and Taffy111 tests pass. Contract tests are running; fresh parity/pixels/full regression and opt-out/performance coverage remain pending. Evidence: `percentage-spacing-validation/opt-in-receipt.json`.

L15 opt-out coverage passes: the same Taffy tree toggles false/true/false, restoring float geometry when CSS precision is disabled. All112 Taffy tests pass. Added version16 host tests for unsupported-capability rejection before stream output, invalid/duplicate/missing targets and compile-once viewport replay. Updated TypeScript version/capability assertions; strict typecheck passes. Initial frozen build failed on a missing probe import and is preserved; corrected build runs in `percentage-spacing-v16-r2-toolchain` (57092). Host tests, fresh parity and441 native comparison reruns await the corrected toolchain.

L15 version16 focused gates are green:441/441 Chrome/native geometry and real Rust Metal pixel comparisons pass;27/27 native/WASM and host-contract tests pass. All13 earlier geometry-candidate pixel failures are fixed in this fresh run, with thresholds unchanged and red artifacts retained. Visual inspection has begun:6 directly inspected views plus3 exact within-run image transfers account for9/441 comparisons,432 remaining. Added reusable `validation/make-replay-sheets.py` to prepare unscaled comparison sheets separately from review recording. Full public suite19571 remains running; full native regression, visual completion and nested measurement cost/coverage audits remain outstanding. Receipt: `percentage-spacing-validation/v16-receipt.json`.
