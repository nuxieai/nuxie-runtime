# A09 fractional background paint bounds

Status: cause isolated; opt-in runtime experiment passes. Public compiler/host
capability integration and full qualification are still required.

The original em-layout-cascade240 failure reproduces with exact matching logical
bounds. Root height137.5px and childY97.5px produce antialiased edges in native;
Chrome snaps them. Example: root pixel(10,137) Chrome(238,238,255), native(247,247,255).
At child pixel(30,97), Chrome keeps the parent background; native blends coral.

Reduction: one coral rectangle with margin-top:.5px, height80px and width100%.
All three widths fail. The otherwise identical1px control passes all three.
Both fixtures are retained in validation/cases.json. No em or rounded geometry is
needed to reproduce, ruling out em resolution or rounded corners as sole causes.

Hypotheses: missing paint-bound snapping, renderer antialias policy, rounded path
construction. Snapping both rectangle edges in scene CSS pixels after translation
fixes original and minimal cases without changing logical layout or renderer-wide
antialiasing. LayoutComponent now has an opt-in set_css_pixel_bounds method,
default false and preserved on clone. Unit-scale axis-aligned transforms use it;
other transforms preserve existing geometry. It affects shared layout paint/clip
paths, so clipping is part of the eventual contract and must be qualified.

The probe currently enables this ONLY with NUXIE_EXPERIMENTAL_CSS_PIXEL_BOUNDS=1.
This diagnostic switch is not a portable publish contract. Ordinary Rive import
and normal compiler output do not opt in. Do not mark A09 qualified yet.

Evidence:
- em-edge-repro: original failure reproduced.
- em-edge-minimal:3/6 (fractional failures, integer controls pass).
- em-edge-experiment:9/9 original/reduction/control, all three sheets inspected.
- em-edge-nearby:117/117 with diagnostic switch;94 image pairs match reviewed
  baselines,23 changed pairs across nine fixture families visually inspected.
  Includes rounded rows, fractional clipping, em flex sizing and themed cards.
- Compiler184/184 passes after runtime experiment. Publish parity is not a
  qualification gate for the new behavior yet, because it is not serialized.

All artifact directories are under output/playwright/html-to-riv/. No tolerances
were changed, and no authored fractional geometry was rounded by the compiler.

Next implementation:
1. Add a versioned layout paint capability and per-LayoutComponent targets in
   RuntimeRequirements, with strict version/capability/duplicate/target checks.
2. Emit targets for painted/clipped layouts, retain existing text version semantics,
   update Rust exports and JS/TypeScript contract, and check backward compatibility.
3. Validate every target before mutation; install the runtime policy through the
   normal probe/host path. Remove the diagnostic switch once replaced.
4. Public manifest/rejection/clone tests, native/WASM parity, source-map/requirements
   parity, same-scene resize, DPR and full native/vector qualification. Preserve
   raw-Rive controls proving default behavior unchanged. Continue all residual
   limitations honestly; interactions/editor/Grid remain excluded.

## Portable integration update

The diagnostic enable switch has been removed. Compiler output now carries
version5, layout-css-pixel-bounds-v1 and layout_pixel_bounds IDs for painted/clipped
layouts. Strict Rust validation rejects missing/mismatched capability, wrong version,
empty/duplicate targets; host validation checks actual LayoutComponent objects
before mutation. Probe installs after both text/layout validation. Versions1–4 keep
old contracts; v5 can include existing text records. TypeScript exposes the new
version and existing ellipsis types.

Rust188/188, JS9/9, TypeScript and native/WASM builds pass. JS covers full corpus
parity plus probe capability/type/duplicate rejection and raw-Rive logical-bounds
control. Native117/117 through normal loader is image-identical to inspected
experiment. DPR27/27 pass. Full native regression running; do not mark A09 complete.
Next: finish full run, inspect changed scenes, validate clone/default behavior and
fractional rectangle DPR explicitly, then reassess remaining affine/vector limits.

## Dedicated layout paint DPR qualification

`validation/layout-pixel-bounds-dpr-control.mjs` passes 27/27 comparisons against
Chromium 153.0.8010.12: fractional-origin rectangles, fractional rounded backgrounds,
and rounded overflow clipping, each at DPR 1/2/3 and widths 240/390/768. Each scene
is compiled once at 390px and resized by the runtime. Native/WASM Rive bytes,
source maps and runtime requirements match. Geometry tolerance remains 0.1 CSS px;
shape pixel limits are unchanged and antialiasing exclusion is disabled.

All nine `review-{fractional,rounded,clipped}-{1,2,3}.png` sheets were visually
inspected. Straight rectangle edges align; rounded and clipped shapes retain
small corner antialiasing differences within the existing limits, without visible
child leakage or shifted straight edges. Artifacts and per-frame metrics:
`output/playwright/html-to-riv/layout-pixel-bounds-dpr/`.

These checks qualify the three translation-only paint cases at integer DPRs;
they do not establish affine transform, fractional DPR, vector text, or complete
regression qualification. Full native 1477-check run remains active; resume session
65444 and `/tmp/html-layout-contract-full.log` before any build or restart. Clone
policy regression coverage remains pending.

Clone regression added in runtime `packed_layout_tests`:
`css_pixel_bounds_survive_clone_and_can_be_disabled` compares world paint bounds
before opt-in, after opt-in, after clone preparation and after disabling the clone's
policy; logical layout and the source's policy must remain unchanged. Execution
is pending the active full visual run (session 65444). Run
`CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --lib css_pixel_bounds_survive_clone_and_can_be_disabled`
after the visual process terminates; this is not yet a passing-test claim.

Clone test execution started as a separate library-test build (does not replace
probe/renderer binaries): session 7361, log `/tmp/html-layout-clone.log`.
The test build is confirmed live; resume this handle before retrying. Full visual
session 65444 remains live, with completion still unproven. Previous instruction
to wait for the entire visual run before this library-only build was conservative;
renderer executables remain fixed throughout the concurrent library-test build.

Clone-policy validation now passes: `cargo test -p nuxie-runtime --lib
css_pixel_bounds_survive_clone_and_can_be_disabled` ran 1/1 test successfully.
The test checks raw world paint bounds, opt-in snapped bounds, preserved behavior
on a prepared clone, reversible disabling on the clone, unchanged source paint,
and unchanged logical layout. Log: output/playwright/html-to-riv/layout-contract/clone.log.
Full visual session 65444 is still active; full regression and changed-image review
remain pending.

## Full native regression after portable integration

Full run terminated successfully: 1477/1477 checks (1467 scene comparisons and
10 pixel-validator controls). The original em-layout-cascade 240px failure now
passes through normal requirements installation. Log and gallery:
`output/playwright/html-to-riv/layout-contract-full/`.

Image/source comparison against `underline-fractional-full` followed by the
reviewed `layout-contract` targeted run transfers review evidence for 1226/1467
pairs. Another 241 passing comparisons in 96 fixture families require fresh
visual inspection. `baseline-comparison.json` identifies each pair; sheets are
being generated under `layout-contract-full/inspection/` (session 98163, log
`/tmp/html-layout-full-sheets.log`). Do not treat green pixel checks as completed
visual review. Full run session 65444 is terminal; do not resume or restart it.
Native/WASM, manifest rejection and clone/DPR gates have already passed. Remaining
qualification work is this visual review plus honest accounting of vector/affine
limitations; no tolerance changes were made.

First full-run inspection batch: four sheets (10 changed comparisons) inspected:
text-hidden-fractional, nested-percentage-column, rgb-legacy-and-modern, and
intrinsic-text-row-sizing. Clip cutoffs, sibling visibility, nested rectangle edges
and intrinsic label sizing align. Small glyph/corner antialiasing differences
remain inside unchanged limits. Detailed observations and remaining queue are in
`layout-contract-full/visual-inspection.json`: 231 comparisons in 92 families remain.
All 96 sheets finished generating successfully; session 98163 is terminal.

Second full-run inspection batch: 19 sheets / 54 changed comparisons
inspected: flex-inflexible-explicit-basis, flex-shorthand-and-cascade,
inherit-percent-dimensions, letter-spacing-intrinsic-end, percent-limits-flex,
percent-limits-inherit, percent-limits-min-wins, and matrix-0 through matrix-11.
Widths, gaps, responsive placement and parent extents align. Small corner/glyph
antialiasing differences remain within existing limits; no shifted straight edges
observed. Inspection queue now records 64/241 changed comparisons reviewed,
177 remaining across 73 families. Per-sheet observations:
`layout-contract-full/visual-inspection.json`.

Third full-run inspection batch: all 38 remaining selector-family sheets,
76 changed comparisons, inspected. Includes adjacent/general siblings,
attributes, structural/nth selectors, negation, :is/:where and plan-card compositions.
Colors, visible-item ordering, dimensions and gaps agree with Chrome. Rounded
cards, buttons and stacks retain small corner antialiasing differences within
unchanged limits; no shifted straight edges observed. Queue now records
140/241 changed comparisons reviewed; 101 comparisons in 35 custom-property
families remain. Per-sheet observations and exact widths are retained in
`output/playwright/html-to-riv/layout-contract-full/visual-inspection.json`.

## Full native visual qualification complete

Final batch: all 35 remaining custom-property sheets / 101 changed comparisons
inspected. Colors, reset/empty regions, inheritance/fallback results, flex placement,
and themed-card geometry agree with Chrome; small rounded-corner antialiasing
differences remain inside unchanged limits. The inspection queue is empty:
241/241 changed comparisons inspected, plus 1226 image/source-identical pairs
whose review transferred from explicit reviewed baselines. All 1467 scene pairs
now have review evidence. Full numeric result remains 1477/1477 checks.

This closes native qualification of the version-5 layout paint contract and
resolves the original A09 fractional shape-edge failure. Rust188, JS9 (corpus
native/WASM parity and host rejection), TypeScript, native/WASM builds, clone1,
layout DPR27 and separate underline DPR27 already pass. Ordinary Rive policy
remains opt-in; no logical dimension rounding or tolerance widening.

A09 remains partial for vector text: earlier em-nested-typography and
em-font-shorthand-final-size 240px failures require fresh accounting. Focused
vector run started in session60933, `/tmp/html-layout-em-vector.log`, output
`layout-contract-em-vector/`. Resume that handle before restarting. Affine and
fractional-DPR qualification is not established. S09/S10 broader substituted-value
grammar audit remains independent work; the overall backlog is not complete.

## Focused vector result

The vector rerun terminated: 31/33 comparisons pass. It covers five em families (em-flex-sizing,
em-font-shorthand-final-size, em-global-values, em-layout-cascade,
em-nested-typography), four rem families, and two background controls.
All 11 browser/vector/diff sheets were visually inspected at all three widths.
Shape edges, spacing and responsive geometry align; text shows the existing
vector coverage differences. Original em-layout-cascade240 now passes.

The two failures are em-nested-typography240 (mean channel error1.2762630208)
and em-font-shorthand-final-size240 (1.0319140625), both above the unchanged1
limit and matching the historical failure metrics. No claim of image identity
with an old vector run is made. All 12 rem and six background-control comparisons
pass. Artifacts: output/playwright/html-to-riv/layout-contract-em-vector/,
including pixels.log, review.json and inspection/*.png. Sessions60933 and63329
are terminal; no work is still running.

A09 native qualification is complete; vector text fidelity remains partial.
Next independent compiler work resumes S09/S10 substituted-value grammar audit.
