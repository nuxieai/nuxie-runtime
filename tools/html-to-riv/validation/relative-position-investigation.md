# L17 relative positioning investigation

Status: implemented compiler candidate; geometry passes the initial matrix,
but painting, composition, parity and full regression qualification remain.
Chronological investigations below include superseded pre-implementation states.

## Proposed language and semantics

Qualify `position: static | relative`, physical `top/right/bottom/left`, and
one-to-four-value `inset`. Offsets accept auto, signed px/em/rem/percent and
unitless zero, with the existing finite-value resource bounds. Include supported
CSS-wide keywords, custom-property substitution and shorthand cascade resets.
Static offsets must have no layout or paint effect. Relative offsets move the
whole subtree while preserving its original flow footprint and sibling sizing.

Reference: [CSS Positioned Layout 3](https://www.w3.org/TR/css-position-3/#relative-positioning),
consulted 2026-09-10. Chrome 153.0.8010.12 remains the executable reference.
Opposing offsets, percentage bases and auto-height containers require explicit
Chrome discriminators; do not infer them from existing engine behavior.

Absolute positioning belongs to L18 and z-index to L19. Fixed/sticky positioning,
logical insets, writing-mode expansion and general math expressions are not
accepted by this increment. Existing global exclusions remain intact.

## Observed implementation seams

- `layout_component_style.rs::apply_item_style` already transports four offsets
  with units into YGStyle. Its horizontal mapping depends on `context.is_ltr`.
- `layout_style_applier.rs::set_position_type` maps both legacy Static and
  Relative to Taffy Relative. Its comment explicitly preserves Yoga static inset
  behavior. CSS static must therefore suppress effective insets in the compiler
  or an opt-in policy; changing legacy runtime defaults is inappropriate.
- Generated LayoutComponentStyleBase already has offset values, offset unit
  properties and position type. The schema-backed compiler writer can address
  these by name; a new wire property is not assumed necessary.
- Vendored Taffy flexbox resolves insets against `constants.node_inner_size`
  during item construction, then applies relative offsets during final layout.
  This is a responsive path, but percentage resolution for indefinite axes,
  reversed directions and conflicting edges is not yet qualified.
- Positioned-element paint ordering must also be tested against ordinary
  overlapping siblings. Passing rectangle geometry alone cannot qualify L17.

## Admission evidence

`output/playwright/html-to-riv/relative-position-admission/receipt.json` records
the frozen L16 publisher hash and five exact input hashes. The normal-flow
control compiles; relative, relative with offsets, static with an offset, and
relative with inset shorthand all reject (exit 1). Per-case logs retain the
diagnostics. These are admission reproducers, not passing feature tests.

## Qualification matrix and next actions

1. Capture independent Chrome scenes for each edge, auto pairs, opposing pairs,
   positive/negative offsets, percentages and zero; all four flex directions,
   both box-sizing modes, definite and intrinsic parent dimensions.
2. Include nested offsets, wrapping, sibling overlap/paint order, clipping,
   negative/auto margins, aspect ratios, text and images. Measure an unchanged
   sibling and parent extent as controls for flow-footprint preservation.
3. Add failing public compiler tests; implement typed syntax and emission only
   after choosing static suppression and verifying wire unit semantics.
4. Compile once; compare original and cloned scene geometry at
   240 → 390 → 768 → 240. Test policy installation/clearing if a policy is needed.
5. Qualify native/WASM parity, native and vector pixels with existing tolerances,
   inspect images and preserve every failure. Run the immutable full regression
   and update SUPPORT.md only when the supported increment is evidenced.

The running L16 full regression uses a frozen toolchain and shared corpus. This
investigation and its separate admission inputs do not modify those inputs.

## Initial Chrome matrix (2026-09-10)

`validation/relative-position-cases.json` now defines 192 scenes: four flex
directions × two child box-sizing modes × definite/intrinsic parent height ×
twelve modes. Modes include baseline, relative-auto, each physical edge,
opposing pairs, start/end percentages, static with inset, em/rem, and shorthand.
`output/playwright/html-to-riv/relative-position-oracle/capture.mjs` explicitly
requires Chrome 153.0.8010.12 and captured all 576 views with screenshots.
`check-invariants.py` and `invariants-receipt.json` record 3,456 passing reference
checks: unchanged parent/sibling rectangles, unchanged subtree sizes, coherent
subtree displacement and expected physical offsets across reversed directions.
These are browser-only checks; no native comparison or visual review is claimed.

Notable discriminator: in the 240px row/border-box intrinsic-height fixture,
the parent measures 240×110 with 20px padding. `left:10%;top:10%` shifts the card
by (20,7), and right/bottom percentages shift by (-20,-7). Thus vertical
percentages use the final 70px content height in this flex case. The definite
260px-height control shifts vertically by 22px. Taffy's early inset resolution
needs a direct runtime test against this evidence before compiler emission is
considered sufficient. Opposing left/right and top/bottom produce (15,11) in
every direction; the static-offset control remains at (0,0).

Next: establish a runtime offset discriminator and public compiler regression,
then implement syntax/emission and any evidenced opt-in runtime correction.
Nested positioning, wrap/clip compositions and positioned paint order still need
additional fixtures beyond this initial reference matrix.

## Engine discriminator and isolated correction experiment

`output/playwright/html-to-riv/relative-position-engine` contains a standalone
Cargo diagnostic, baseline results, a copied experimental Taffy source tree,
source hashes, and `compare.py` / `comparison-receipt.json`. It models the row,
border-box, 240px reference with definite/intrinsic height and zero/percentage
offsets. No production runtime source is changed by this experiment.

The existing engine matches 19 of 20 checked coordinates. For intrinsic height
with `top:10%`, its card y is 20; Chrome's is 27. Parent height and sibling
coordinates agree. Re-resolving insets in `calculate_layout_line` against the
final content size fixes that discrepancy: the isolated experiment matches all
20 coordinates, and the three controls remain unchanged.

This establishes a promising correction location, not production qualification.
Before shipping, introduce an explicit CSS opt-in rather than changing legacy
Yoga behavior, expand the engine test across the full reference matrix, and
exercise actual Rive import, cloned instances and responsive resize. The
experimental source deliberately lacks that opt-in and must not be used as a
production toolchain. Paint ordering and text pixels remain separate gates.

## Gated engine implementation and full initial matrix

The vendored engine now exposes `Style::css_relative_position`, default false,
and forwards it through both style trait implementations. Only opted-in flex
items re-resolve their insets against the final containing content size during
line placement. Legacy early-resolution behavior remains the default.

The new `final_height_percentage_offset_is_opt_in_and_clearable` regression first
failed at y=20 versus Chrome's y=27, then passed with the correction. It exercises
false → true → false and checks unchanged parent size and sibling placement.
All 114 engine library tests pass; `cargo check -p nuxie-runtime` also passes.

`relative-position-engine/src/bin/matrix.rs` maps the 192 reference scenes into
engine styles and resizes each same tree through 240→390→768→240. The legacy
engine has 128 coordinate failures across 16 scenes. The isolated experiment and
gated engine both match all 15,360 coordinates across 768 tree viewport updates.
With the flag off, the complete result is identical to the saved legacy result.
Hashes and counts are in `relative-position-engine/matrix-receipt.json`.

This is engine geometry evidence only. The diagnostic supplies already-computed
style values and suppresses static offsets itself. Public parsing/emission,
runtime occurrence transport, Rive import/clone, WASM parity and pixel checks
still need implementation and qualification. No accepted-language expansion is
claimed yet. The existing experimental copy remains diagnostic-only.

## Runtime occurrence transport and imported-scene evidence

`LayoutComponent::set_css_relative_position_occurrence` now installs/clears the
default-off policy, rejects non-layout targets, synchronizes style and marks
layout dirty. Instance cloning copies the policy, and `apply_base_style` forwards
it to the engine. This is candidate runtime plumbing; no compiler capability
manifest or public CSS emission is added yet.

`tests/relative_position_runtime.rs` imports compiler-produced normal-flow Rive
scenes, injects the existing offset wire fields, installs the policy and checks
all 192 Chrome reference scenes. Original and cloned instances pass 1,536
viewport updates / 30,720 coordinate checks. The effective engine flag is checked
after each update. Clearing the original leaves a previously cloned instance's
flag enabled; non-layout installation rejects. All checks pass.

Important correction to the earlier engine inference: the actual runtime's
layout passes already produce correct geometry for this matrix without policy
propagation. The initial runtime test had no matrix coordinate failures; its
only failure was expecting y=20 after clearing, whereas runtime y remained the
correct 27. The revised test explicitly checks effective flag ownership and
expects y=27 in both original and retained clone. Thus the engine reproducer
must not be represented as an observed imported-scene bug or as proof that this
matrix requires a new runtime capability. Further compiler integration should
consider whether this policy is necessary for broader cases before imposing a
new host requirement. All initial and corrected logs are preserved.

Evidence: `relative-position-engine/runtime-transport-receipt.json` pins the
test, oracle, runtime source and logs. Public CSS parsing, static suppression,
paint ordering, pixel inspection and native/WASM qualification remain open.

## Public compiler candidate

`position.rs` now parses typed physical offsets and emits existing Rive offset
properties. `style.rs` implements `position: static | relative`, physical
longhands and 1–4-value inset shorthand. Signed px/em/rem/percent, auto, bounded
zero/length values, CSS-wide resets, explicit inheritance and custom-property
substitution are covered by public tests. Position is not inherited by default.
Static elements retain computed insets for explicit inheritance but emit none.
Malformed substituted position/inset values compute to their initial values;
valid but unsupported positioning modes keep their rejection diagnostic.

Public tests first failed 2/3, then pass3/3. The actual compiler output also
passes all192 Chrome reference scenes through original and cloned imported
instances at240→390→768→240:1,536 instance updates. No injected offsets or new
runtime policy installation is used in this public oracle test. Existing wire
semantics suffice for this matrix, so the compiler does not emit a new required
host capability for relative positioning at this stage.

Evidence: `relative-position-engine/public-red.log`, `public-green.log`, and
`public-oracle.log`; permanent tests are `tests/relative_position.rs` and
`relative_position_public_compiler_matches_chrome_after_clone_and_resize` in
`tests/aspect_ratio_oracle.rs`. The full module run is in `full-public.log`.
This does not qualify positioned paint order, broader nested/wrapped/clipped
compositions, native/vector pixels, or WASM parity. Absolute/fixed/sticky and
logical insets remain rejected. SUPPORT.md labels this candidate accordingly.

Full module run completed successfully:312 tests across57 groups, no failures
or ignored tests. See `relative-position-engine/public-receipt.json`.

## Frozen pixel replay and positioned paint-order failure

`relative-position-toolchain` contains the frozen publisher, probe, Rust Metal
renderer and WASM module with hashes. Initial expanded native/WASM parity passes
all11 JavaScript tests, including the192 initial relative-position scenes.

`relative-position-composition-cases.json` adds64 scenes /192 Chrome captures:
four directions × two child box models × overlap, both-positioned siblings,
nested positioned descendants, clipping, order, text, image and wrapping/ratio
cases. Public original/clone resize geometry passes all64 scenes (512 instance
updates). Frozen native replay passes geometry192/192, but pixels159/192.
All33 pixel failures remain unwaived: overlap6, nested-positioned6, order6,
clip8, wrap-ratio6 and image1. These categories do not imply identical causes.

Direct inspection of row/border-box overlap at390px confirms a paint-order bug:
Chrome places the positioned orange card above its ordinary green sibling;
native draws the green sibling above the card and some teal descendant pixels.
`relative-position-composition-native/initial-overlap-inspection.json` hashes
the two directly inspected full PNGs. No full-cohort visual review is claimed.

The existing `Artboard::css_ordered_drawables` recursively reorders whole layout
subtrees for flex painting; it has no CSS positioned-element classification.
Compiler `positionTypeValue` alone cannot distinguish authored CSS static from
relative because the underlying legacy layout default is also Relative. A
paint-specific authored-position marker and explicit host contract may be needed.
Nested positioned descendants and clip scopes require particular care; sorting
only immediate sibling groups is not assumed sufficient. This provides a real
paint discriminator independently of the engine-only percentage-offset issue.

Evidence: `relative-position-parity.log`,
`relative-position-composition-public.log`, and
`relative-position-composition-native/replay.json`. Initial576-view native replay
completed with 529 pixel passes, 47 pixel failures and 0 geometry failures (session68424 exit1). Composition parity expansion and all
native/vector visual review remain pending.

Positioned paint-order follow-up:24-view discriminator isolates6 unclipped
ordinary-sibling failures;18 controls pass, all geometry matches. Nine earlier
composition views now have direct failure inspection evidence. See
`validation/positioned-paint-order-investigation.md` for the clip-preserving
paint-plan requirements and exact runtime seam.

L17 initial visual audit complete:576/576 native views reviewed,0 remaining. Final12 reverse-column bottom-offset views match Chrome in translation, narrow gap below green, bottom space, nested inset and host sizing. All576 initial vector views transfer with identical canonical compiler inputs and full Chrome/native PNG bytes; independent transfer audit passes. Composition native192/192 and full native5983/5983 already pass with complete visual evidence. Nine composition vector text pixel failures remain unwaived; remaining interaction qualification audit stays open. Evidence: output/playwright/html-to-riv/positioned-v17-receipt.json.

L17 interaction corpus adds32 scenes across four directions/two box models and auto margins, stretch, negative margins and percentage padding. Pinned Chrome153.0.8010.12 captures96 views. Frozen v17 native replay passes96/96 geometry and pixels (exit0); public original/clone resize regression passes (exit0). Permanent oracle and expanded native/WASM parity corpus added; parity50657 remains running. Visual review96 views and vector replay remain pending. No tolerance changes or qualification claim.

L17 interaction visual audit complete96/96. Final12 reverse-column/content-box views match Chrome, including growing percentage padding, bottom overflow and purple top-viewport clipping at768. All96 vector views transfer with exact canonical compiler inputs and full Chrome/native PNG identity; transfer audit passes. Public original/clone resize test and expanded native/WASM parity11/11 pass. Nine composition vector text failures remain unwaived; final feature qualification audit remains.

L17 final focused evidence audit verifies19 aggregate file hashes,888 passing native comparisons with complete image audits,672 initial/interaction vector comparisons with complete audits, and192 composition vector geometry comparisons with183 pixel passes/nine unwaived failures. Existing full native5983-check receipt remains separate from focused coverage. L17 stays partial for vector text; no external blocker is claimed. Next independent implementation item is L18 absolute positioning.
