# Integrating emitted flex descriptors with numerical admission

Status: implementation plan based on the registered `compiler.rs`, `flex_descriptor.rs`, `computed_provenance.rs`, and `flex_numeric.rs`. No source or admission changes in this task. Numerical proof remains separate from Chrome/native geometry and pixel qualification.

## Current seam and the missing link

`Emitter::children` carries only `definite_chain` and `numeric::Bounds`; neither is an ideal/native error envelope or a world-position proof. `Pending::finish` runs after all records exist and catches constraints added by an ancestor's baseline pass. Keep this finalization ordering. A child descriptor captured during emission cannot certify absence of future constraints.

Descriptors now retain actual scalar carriers, including equality identity, for dimensions, factors, basis, and bounds. They also retain logical order, native participant order, wrappers/helpers and selected native fields. `parent_main_error` and `parent_world_error` are still `None`; these must not become zero merely because parent sizing is definite. The analyzer requires full ideal ranges and errors, not just these two scalar error fields. Add a private typed proof context; diagnostic JSON is not a proof input.

`Group::structural_issues.is_empty()` is not the analyzer's sixteen-premise certificate. In particular, the current record scan does not check every transform/default, aspect ratio, animation, descendant, or ancestor. `start_aligned_cross` is an exclusion heuristic, not a final native cross-axis proof. `NumericFacts::available` permits `Auto`, which establishes descriptor completeness but not resolved numerical sizing.

Root is adding `Item::numeric_input()` as the narrow scalar-to-analyzer conversion. Treat that conversion as a prerequisite, not a parent/world or premise certificate. It must retain scalar identity, verify final participant bindings and native basis/factor bits, and keep authored shrink independent while checking the native linked fraction against grow.

For omitted bounds, `layout_box` deliberately omits zero minima and absent maxima. A missing `minWidth`/`minHeight` value and units pair must resolve through the pinned native default to an explicit zero minimum; an omitted maximum must resolve to no maximum. Check the exact LayoutComponentStyle defaults and its `apply_base_style` translation rather than treating every missing value as zero. In particular explicit automatic minima emit units 3 and must not pass the zero-minimum shortcut; partial/mismatched property pairs in mutated records must reject. Verify present point-bound values/units against the corresponding typed native bits, retain ideal carriers, and check both main and cross bounds. These checks belong in the conversion/default reader and structural proof respectively.

## First restricted implementation

Start with a private proof attempt on actual candidate output, before changing public routing:

1. Build an indexed final-record topology once. Retain parent IDs, each group's complete direct native participants, authored handles, style handles, constraints/origins, and transform-affecting ownership. Use actual IDs, never source names or fixture shapes. Parent-first analysis is necessary: pending groups are currently appended after their descendants.
2. Start at the artboard with the documented full resize domain `(0, 16384]` on each axis. Close this conservatively to `[0, 16384]` for interval work. Specify the host-input contract: if browser widths are real numbers converted to native f32, include conversion error; do not assume all callers pass exactly representable integers. Artboard local origin can be exact zero only after checking immutable defaults and emitted fields.
3. Initially admit proof attempts for zero-inset, fixed-point, unwrapped leaf items, or a fixed-point authored parent whose own placement has first been proved as an ordinary fixed participant in its parent group. Retain all four CSS directions and stable order. Parent point dimensions require computed scalar error and min/max-clamp semantics, even though they do not depend on viewport size. Exclude parent intrinsic sizing, flexible parent size, automatic minima, responsive bounds, and percentage sizing from this first implementation if their envelope operations are not yet implemented.
4. Analyze ancestor groups too. A top-level authored container is still positioned by the host's reversed-flow/end-alignment encoding. Its apparent CSS origin of zero is not an independent assertion that native cancellation and layout-transform arithmetic have zero error. Adapt fixed legacy participants to analyzer items using effective point main size as basis and exact zero factors, after checking the emitted Fixed/Auto mapping and bounds. Carry the resulting parent size and position envelopes down; also prove the other axis. The analyzer's one-dimensional position output alone does not establish two-dimensional world geometry.
5. For a target group, convert explicit point-basis flex items using their retained authored carriers and actual emitted values. Require computed/native basis, factors, units, bounds, cross sizes and scale modes to match the intended mapping. Positive-basis linked factors require ideal and native equality. Exact-zero basis allows unequal authored shrink only under the analyzer's whole-group rule. Do not rewrite authored shrink provenance to native grow.
6. Invoke `analyze` only with every established premise and an explicitly allocated geometry budget. Retain failures (`InitialFreeze`, `PossibleFreeze`, `AggregateUncertainty`, etc.) as unresolved reasons. An analyzer success is an arithmetic result for this restricted structure; keep public nonlegacy flex diagnostic until the concrete accepted family has passed the pinned Chrome/native workflow.

Leaf-only applies to layout descendants: ordinary Fill/SolidColor records may remain. This gives a concrete initial `DescendantGeometryAccounted` proof without pretending that nested paints, percentage descendants, or intrinsic measurements preserve targets automatically. It is a first slice, not a removal of those features from the roadmap.

## Premise extraction

| Analyzer premise | Available evidence / required addition |
| --- | --- |
| ZeroInsetsAndGaps, NoAutoMargins | Computed padding/margin flags plus final style-property audit. Parent spacing and generated helpers already identified; check all sides/gaps/borders/insets and schema defaults. |
| NoWrap, LeftToRight, NoAspectRatio | Current CSS whitelist excludes these variants, but bind this to final ordinary style fields and immutable defaults. Do not infer from an absent descriptor field. |
| PhysicalFlexStart, NativeReverseFlow, ActualParticipantOrder | Existing direction wire/alignment and complete native order are useful. Check the expected permutation against logically sorted items, with no helpers/wrappers; retain each check as evidence. |
| CrossStartDefinite | Require actual native fixed point cross size, zero minimum/absent maximum, supported self-alignment and zero automatic margins; audit cross position math. Existing computed heuristic alone is insufficient. |
| NoIntrinsicSizing, KnownTargetPreserved | Point bases, explicit bounds, correct native scale/basis, no aspect/padding floor/auto minimum, plus leaf layout topology initially. Parent size must itself have a proved envelope. Auto basis remains outside the single-pass model. |
| IdentityOwnTransform, ZeroOrigins, PureTranslation | Final LayoutComponent/Artboard properties and immutable schema/runtime defaults. Check explicit transforms/origins and relevant ancestor chain; do not merely check local CSS. Rive layout-generated translation arithmetic still needs its error accounted. |
| NoConstraintsOrAnimation | Final ownership index already finds constraints and ComponentOrigin locally. Extend through the ancestor chain, inspect animation targets, and assert no relevant alternate transform owner. |
| DescendantGeometryAccounted | Initially prove absence of layout descendants; later recursively propagate size/world errors and target preservation. Existing field intentionally remains None. |

Some defaults are provable without adding emitted fields: reference the pinned schema/runtime behavior and check absence of overrides. Never serialize defaults solely to make the checker easier, since previous bytes must remain stable.

## Percentage and nested extension

After the point profile, add a checked domain operation for preferred percentages, following native `fl(fl(coefficient * owner) * 0.01f32)` while preserving ideal division by 100. A coefficient carrier is not a resolved parent envelope. `ScalarProvenance::preferred_percent` handles a concrete scalar owner; it is not sufficient for an entire viewport interval. Add interval arithmetic at the proof layer, propagating the owner's error, coefficient error, operation rounding and finite limits. Padding percentage uses a different native operation order and remains excluded initially.

Compute used parent content size with real min/max semantics and padding floors before giving it to flex. The current overflow guard's upper/lower/witness values must not be reinterpreted as ideal/native error bounds. For flexible parents, use prior group analysis sizes; for descendants, propagate both axes and native world-transform additions. Freeze-dependent groups, Auto basis/intrinsic content, wrappers, padding, auto margins, distributed spacing and wrapped composition require explicit extensions and regression evidence rather than a blanket premise list.

## Concrete regression controls

- Actual compiled point-parent groups for all four directions, negative/positive `order`, a fixed sibling, equal positive-point factors, and exact-zero unequal factors; compare analyzer permutation to final records.
- Parent/ancestor siblings that give a nonzero offset, reversed directions, deep fixed chains, and a large coordinate with a small local offset. Removing ancestor error must make a dedicated test fail.
- Non-dyadic direct/variable/em/rem factors and basis; equal native bits with distinct original ideals; underflowed nonzero ideal basis; unavailable original metadata. Preserve conservative rejection.
- Parent min greater than max, fractional point bounds, max clamping, auto minimum, intrinsic/percentage parent and later percentage chains. Unknown must remain unknown until that operation is proved.
- A baseline ancestor that appends an origin constraint after descendant capture; helper siblings, alignment wrappers and a nested layout child. Each must prevent the initial leaf/direct certificate.
- Mutated ordinary records in private tests: fraction, units, basis, flow, alignment, parent ID, scale, transform/origin, constraint ownership, and participant order. A computed-style match must not override the mismatch.
- Existing freeze/error controls and public strict unmatched/losing declaration diagnostics. No test-only path may bypass production proof conditions.
- Reproduce existing public RIV bytes/maps; then native original/import/clone/resize geometry over the full validation sequence, pinned Chrome comparison, retained fractional pixel failures, and direct visual coverage. Domain arithmetic does not certify raster pixels.

## Remaining scope

Independent grow/shrink/basis remains the overall feature: point and Auto basis, min/max freezes, partial sums below one, arbitrary supported ancestor contexts, descendants, and interaction with compiler shims. The current analyzer intentionally proves only a single-pass structural family. Keep unresolved profiles listed separately from impossible target capabilities; none of the missing premises in this plan establishes runtime impossibility.
