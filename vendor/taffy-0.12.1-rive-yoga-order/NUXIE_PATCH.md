# Nuxie Rive Yoga parity patches

This directory starts from the crates.io `taffy` 0.12.1 package. Nuxie keeps
Taffy as its Rust-native layout backend, but changes arithmetic and cache seams
needed to preserve the pinned Yoga owner's output and practical solve behavior.

## Source authority

- Rive runtime commit:
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- Rive layout call site: `src/layout_component.cpp`,
  `LayoutComponent::calculateLayoutInternal`.
- Yoga flex owner: `renderer/dependencies/rive-app_yoga_rive_changes_v2_0_1_2/yoga/Yoga.cpp`,
  where grow distribution evaluates
  `remainingFreeSpace / totalFlexGrowFactors * flexGrowFactor`.
- Yoga percentage owner: `renderer/dependencies/rive-app_yoga_rive_changes_v2_0_1_2/yoga/Utils.h`,
  where percentages resolve as `value.value * ownerSize * 0.01f`.
- Yoga measurement cache owner:
  `dependencies/rive-app_yoga_rive_changes_v2_0_1_2_grid/yoga/Yoga.cpp`,
  where `YGLayoutNodeInternal` retains and searches multiple measurements and
  accepts compatible constraints through `YGNodeCanUseCachedMeasurement`.

## Behavioral delta

Taffy 0.12.1 originally evaluates the equivalent expression as
`free_space * (child.flex_grow / sum_flex_grow)`. This fork evaluates it as
`free_space / sum_flex_grow * child.flex_grow`, matching Yoga's exact order.

Rive-authored dimension percentages use a dedicated compact-length tag that
retains their [0, 100] value and resolves it as `value * owner_size * 0.01`.
This leaves Taffy's native [0, 1] percentage representation and behavior
unchanged for every non-Rive caller.

Taffy's nine measurement entries are one direct-mapped entry per constraint
category. Deep Rive list/artboard trees can alternate many exact constraints
within a category, continually evicting the immediately reusable result. The
fork makes each category 32-way set-associative. Cache hits return the same
previously computed value and misses still execute the unchanged Taffy solve;
this changes retention only, not layout arithmetic or authored behavior.

The distinction is observable because each operation rounds to `f32`. In the
pinned `list_focus_order.riv` fixture, three growing column children produce
`137.20052` with Yoga but `137.20053` with upstream Taffy; the one-ULP
difference propagates into layout paths and transforms.

For `computed_values_test.riv`, Yoga resolves the animated `38.571426%` width
against 490 as `188.99997`; normalizing it to Taffy's [0, 1] representation
first resolves as `188.99998`. The resulting remaining flex width differs by
one ULP (`301.00003` versus `301`).

## Differential evidence

`wave_b_focus_test_078_direct_port_expected_red` compares the complete native
render stream with the pinned C++ silver. It fails at frame 0, operation 78
with upstream Taffy and passes with this single expression-order change.

`data_binding_computed_root_values` fails at frame 2, operation 191 with the
normalized Taffy percentage and passes with the dedicated Rive Yoga percentage
resolution path.

The pinned `data_bind_blob_test.riv` silver builds a finite, acyclic tree of
326 native layout owners at maximum depth 16. Upstream Taffy's direct-mapped
cache exceeded 1.4 million child-layout calls without finishing the first
solve in the bounded debug test. The set-associative cache completes the exact
silver comparison in about two seconds on the same debug build.

## Cross-size limits during flex-basis measurement

Child known cross dimensions are clamped by their resolved min/max limits before
measuring the flex basis. Upstream Taffy passed a definite authored width through
unchanged here: a 500px-wide text container capped at 168px measured two lines,
then drew four lines at its capped width while retaining the two-line height.
The main-axis basis remains unconstrained at this stage.

The pinned Yoga owner `YGNodeComputeFlexBasisForChild` calls
`YGConstrainMaxSizeForMode` before `YGLayoutNodeInternal`, including for exact
measure modes (Yoga.cpp lines 1568 and 1754 in the grid dependency listed above).
The public native regression `capped_text_measures_at_its_clamped_width_after_resize`
fails before this change and passes for pixel/percentage maxima at three widths
with it. Full browser and upstream-layout qualification is recorded in
`tools/html-to-riv/validation/percentage-limits-review.md`.

## Optional host text baseline

`compute_layout_with_measure_and_baseline` accepts an optional host callback
returning the measured first vertical baseline. It populates LayoutOutput before
caching, allowing ordinary flex baseline propagation through nested containers.
Existing `compute_layout_with_measure` delegates with no callback. Hosts must
dirty nodes when baseline data or callback policy changes. Nuxie enables this
only for the explicit CSS align-self baseline policy; raw Rive defaults remain
unchanged. Bridge regression: runtime tests/css_text_baseline.rs. Qualification
evidence is tracked in tools/html-to-riv/validation/align-self-investigation.md.

## Distributed overflow in reversed flex axes

Chromium keeps main-axis space-between overflow at flex-start, and cross-axis
line stretch/space-between overflow at flex cross-start. The generic alignment
fallback previously converted these to logical start, producing different
positions in reversed axes. Flex-only call sites now preserve these distinctions;
around/evenly retain their safe-center logical-start fallback and shared grid
alignment code is unchanged.

L05 public runtime matrix proves main-axis behavior; L06 isolated Taffy test
`reversed_overflow_lines_match_chromium_reference` checks84 independent Chromium
overflow references including all seven line alignments and all four directions.
All98 Taffy library tests pass. The L06 test reads the checked-in compiler oracle
from this repository, not an upstream golden generated by the implementation.
Run via `cargo test --manifest-path vendor/taffy-0.12.1-rive-yoga-order/Cargo.toml --lib`.
The vendor directory is explicitly excluded from the root workspace so its
dev-dependency tests can run independently. Full runtime/compiler/pixel
qualification remains tracked in tools/html-to-riv/validation/wrap-reverse-investigation.md.

## CSS auto-margin allocation

Main-axis auto margins consume positive remaining space before justification;
the justification remainder is now zero after allocation. Cross-axis overflow
with auto margins retains physical-start placement and places the deficit in
the opposite margin instead of assigning a negative start margin. Two isolated
regressions exercise distribution and cross overflow; all100 library tests pass.
The public compiler/runtime oracle checks132 Chromium scenes at396 widths on
original and cloned scenes. Full compiler/pixel qualification is tracked in
tools/html-to-riv/validation/auto-margin-investigation.md.

## CSS partial flex factors

Initial free space already excludes total gaps. The partial-grow and partial-shrink branches now scale that value without subtracting gaps again. A Chromium oracle covers144 scenes/432 viewports across four directions, six factor pairs, zero/nonzero gaps, growth, shrinkage and max-constrained freezing. The pre-fix test reports638 coordinate mismatches; after the fix all101 isolated library tests pass. Compiler admission and native/WASM/pixel qualification remain pending in tools/html-to-riv/validation/partial-flex-investigation.md.

## CSS intrinsic flex basis (L10, experimental qualification)

Content-size measurements of flex containers exclude their own min/max constraints on the requested axis, preserving cross constraints. This keeps the basis independent of the later hypothetical-size clamp. The violation loop floors border-box targets at padding plus border, so zero content size retains occupied space during freezing and redistribution. `css_content_auto_regression` compares36 Chromium viewports without text; all102 library tests pass after these changes. Full compiler/native/WASM/pixel qualification is still pending; see tools/html-to-riv/validation/content-auto-investigation.md.

Experimental L11 percentage-basis correction: only Auto retrieves an authored main dimension when basis resolution has no definite value. An unresolved percentage proceeds to content measurement. The independent Chromium column oracle detects18 incorrect fixed-height container sizes before this change; full standalone engine suite103 tests passes afterward. Public runtime and pixel qualification remain pending, so compiler admission stays guarded.

L11 experiment additionally separates ContentSize/InherentSize cache keys and uses line main-size sums for intrinsic columns. Expanded72-viewpoint oracle and cache-isolation regression bring standalone suite to104 passing tests. Public runtime and broader min/max/wrap/pixel evidence remain pending.

Further L11 experiments align intrinsic shrink fraction scaling in both directions and floor each item contribution by its own padding/border, not the parent inset. Independent Chromium small-content regression covers48 viewports; standalone suite105 passes. Runtime parity/pixels remain pending.


## CSS aspect-ratio input precision (L13, focused qualification)

Opted-in CSS ratios truncate their resolved source dimensions and each padding
edge to 1/64px before ratio transfer. Ordinary Rive ratios retain float inputs.
`FlexboxContainerStyle::quantize_gap` defaults to false; when enabled, resolved
gaps truncate before flex allocation, including later percentage re-resolution.
The runtime derives this policy for the whole solve tree when it contains an
explicit CSS ratio occurrence, because ancestor gaps affect ratio source sizes.
This does not enable CSS Grid. The policy is recomputed for cloned trees.

The 40-scene fractional stress corpus detects 60/120 failing comparisons in the
prior snapshot. All 17 public ratio oracle tests and all 108 isolated Taffy
library tests pass after correction; the new direct gap test also checks the
non-opted-in path. Full compiler/pixel qualification is tracked in
`tools/html-to-riv/VALIDATION.md` and the immutable `aspect-ratio-gap-toolchain`.


## CSS nested column intrinsic sizing (L13, focused qualification)

`FlexboxContainerStyle::css_intrinsic_sizing` is an independent false-by-default
policy. The runtime enables it throughout explicitly opted-in CSS ratio solve
trees. Column cross measurement omits the post-flex known block size. During
intrinsic inline measurement, ratio-derived child contributions retain their
min-content inline floor, without flooring the eventual post-flex layout.
This distinguishes intrinsic wrapper size from descendant ratio overflow.
The runtime also truncates ancestor pixel padding edges before allocation.
The independent policy increases Style storage by8 bytes on this build.

The public nested oracle covers16 scenes at three viewport widths on original
and cloned instances. All18 ratio oracle tests and108 engine library tests
pass. Native/WASM/pixel qualification is tracked in the compiler validation
receipts; this is not a claim of complete CSS intrinsic-sizing coverage.


CSS exact computed ratio pairs: optional `CoreStyle::aspect_ratio_pair` carries
positive signed-32-bit components through flex sizing. Missing pairs retain
ordinary scalar arithmetic. Initial ratio-box transfers, intrinsic column basis
and post-flex cross sizing use signed64-bit26.6 multiply/divide, truncate and
clamp derived raw units to signed32-bit. Runtime occurrence policy validates
components and propagates the pair into cloned styles. Style storage increases
by8 bytes (568String/536Arc);109 engine unit tests pass, including large extent,
fractional truncation, saturation and no-pair controls. Absolute-position/Grid
ratio paths remain outside compiler qualification.
