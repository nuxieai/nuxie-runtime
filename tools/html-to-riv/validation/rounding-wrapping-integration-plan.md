# Rounded paint integration into the actual wrapping composition

Historical design audit, 2026-09-12. The extraction, owned integration and 48-case native campaign described here are now implemented; see wrapped-rounded-review.md and GOAL-RESTART.md for current results and next work.

Original audit: This plan follows GOAL-RESTART.md and preserves the immutable runtime contract. It does not qualify new behavior. Product sources were not edited by this audit.

## Concrete construction path

Extract the live corner, thin-extent and four-mask composition from `validation/rounding-paint-bridge.rs` into a private compiler module, tentatively `paint_box.rs`. Its input is one already validated visible LayoutComponent ID; output is a read-only trace containing the two corner sources, four selected scalar inputs/rounding traces, two thin predicates/final ends and four mask IDs. Emit it once per distinct visible geometry, before line-group replicas. Cache by geometry ID, not color or replica index. All generated scalar Nodes remain in identity artboard space.

Keep `wrapping_paint.rs:100` line anchors, signal cache, leader/member selection, reverse-cross group traversal and reverse-main member traversal unchanged. In the replica block at `wrapping_paint.rs:130`, replace ForegroundLayoutDrawable with root Shape plus the prototype's 32768-square Rectangle. Clone the original Fill and SolidColor under that Shape. Attach the four cached box clips and existing leader/member clips to every replica. Preserve within-item paint order and the DrawRules/DrawTarget chain at line 147, now targeting these Shapes. Do not round slot or visible layout geometry. Keep original colors hidden and root ArtboardFill unchanged.

`paint_items` should return the paint trace (or an owned emission result) instead of discarding handles. Candidate owns this trace alongside records; no mutable record accessor is added. `compose_with_bounds` constructs all records once, validates them, then creates Derived. No append or repair after Derived construction. A failed private construction discards its owned candidate; if the lower emitter remains callable with an external mutable Vec, stage/rollback both appended records and original color changes on every failure.

## Budget before emission

With the exact prototype graph, one distinct geometry costs 2302 shared records: 4 corner records + 4×564 scalar rounding records + 2×17 thin helpers + 8 mask records. Each paint replica changes from 3 records to 8 (Shape, Rectangle, Fill, SolidColor and 4 clips), in addition to its existing line clips. Therefore the new cost is:

`old_paint_records(counts) + 2302 * distinct_geometry_count + 5 * replica_count`

The saturation path below adds 8 shared records per geometry, changing 2302 to 2310. For three one-paint owners and six replicas this is 7073 paint records, before the unchanged base and 141 sizing records. These are exact proposed graph counts, not a measured implementation count. If thin handling or source measurement changes, recompute them.

Calculate counts with checked arithmetic before cloning/emitting in `wrapping_composition.rs:187`. Keep explicit aggregate record budget and ordinary object-ID capacity checks. Change existing successful fixtures that pass a 1000-record budget to an explicit adequate budget, while retaining exact-boundary success and one-less failure tests; do not silently increase a public resource limit. Empty paint lists and shared geometry need explicit count semantics. The existing one-solid-paint closed base has one distinct geometry per slot.

## Domain proof and saturation

The existing `wrapping_masks::prove` only proves active/inactive line mask coverage and stored initial viewport membership. It does not bound corner inputs to paint_rounding. `wrapping_coordinates::visible_world()` describes initial transforms; the active cross axis must instead use final `wrapping_position::Item::landed`, and the orthogonal axis uses the initial bound. Add visible extents in the immutable corner matrix operation order and prove all intermediate entries remain finite. Include original import before resize and all declared original/clone viewport states. Strength-one constraints still evaluate old×0, so finite final values alone are insufficient.

Do not solve conservative edge bounds by narrowing the declared [0,16384]² viewport domain to captured sizes. A concrete full-domain route is to clamp each selected corner coordinate to [-16384,16384] in an ordinary root Node/TranslationConstraint before passing it to paint_rounding. This requires two records per scalar, four scalars per owner. Prove raw corner transform finiteness separately from the clamped rounding-domain guarantee. The unselected coordinate on every scalar root is zero.

For ordered edges and positive dimensions, saturation preserves the rounded rectangle's intersection with an origin-zero artboard whose extents are at most 16384: below the lower limit the adjusted end remains negative; above the upper limit even a thin one-pixel extension starts at the artboard boundary; an edge within the visible interval is unchanged. Retain the thin predicate from the raw extent, not from saturated edges. Clamp-to-zero would be incorrect: a wholly negative thin box could become a spurious visible one-pixel box. Write an explicit proof/test of the actual f32 operations and all boundary cases before claiming this saturation composition.

Keep current line masks at their existing 0/65536 sentinels. Extend mask proof to cover both the four edge masks and the large paint rectangle with final ends that can reach 16385. Bound viewport extents to 16384 for this certificate because that is the rounding saturation boundary; the existing line-mask certificate alone accepts 32768. Rejection of a larger domain is an unresolved composition boundary, not proof that larger scenes are impossible.

## Thin extent and Chrome fidelity are separate obligations

The prototype computes extent by subtracting two raw native-f32 world corner translations. At large origins, a small true extent can be lost or cross the 1/16 threshold. Using `TransformConstraint.sourceSpaceValue=Local` does not remove this issue: immutable `transform_constraint.rs:20` computes `target.world_transform() * local_anchor` first, then applies the parent's inverse at line 64. The low bits are already lost. TranslationConstraint likewise starts from the target's world transform before conversion. Neither audited constraint exposes a direct local width/height scalar.

For the existing wrapping campaign, derive a conservative subtraction-error bound and show each live size interval stays strictly above 1/16 by that margin; a constant true thin flag is then valid, although retaining the dynamic graph simplifies extraction. Do not infer this from sampled large boxes. For general thin dimensions, retain the unresolved flag until a local-extent composition or a bound proving predicate equivalence is implemented. A source-bound ordinary scalar calculation of native used size is a possible follow-up, but it must follow the existing min/max/percent f32 semantics and be independently validated; it is not present in the audited constraints.

Even exact native f32 rounding is not proof of Chrome LayoutUnit equivalence. Half-boundary tests must distinguish the source layout discrepancy from paint rounding. Preserve any remaining failure, without tolerance changes or a blanket “browser-equivalent” claim.

## Independent binding and ownership

Extend the closed reader in `wrapping_paint_binding.rs:80` to consume the complete new suffix, including cached corner/box blocks followed by line groups/replicas. A reusable independent reader for paint_rounding should read actual record kinds, every present field, omitted defaults, axis selection, clamp limits, strength, source/destination spaces, factors, thresholds and operands. Do not validate by re-emitting the expected Vec and comparing it. Trace IDs must agree with the reader's actual outputs; they cannot certify themselves.

Bind corner targets to the correct visible owner and exact (0,0)/(1,1) origins; bind saturation and thin predicate operands; bind mask parents, rectangle coordinates and every replica's four clip sources; bind cloned fill/color and physical draw order. Reject added properties, extra/missing records, graph sharing across wrong owners, wrong axis, changed clip defaults and mismatched spans. Retain the existing exact original-color-hide verification.

`wrapping_composition::preserved` currently rejects this valid new shape at lines 129–161. It allows only root Nodes, Fill/ClippingShape/DrawRules under ForegroundLayoutDrawable, TransformConstraint reads of slots, and DrawTarget references to ForegroundLayoutDrawable. Make it span-aware: sizing helpers keep the original slot-only reference policy; the bound paint suffix may read finalized visible transforms and own root Shape drawables. Accept Fill/ClippingShape/DrawRules under bound paint Shapes and DrawTargets to those Shapes. Do not permit all visible reads globally. Preserve absence of any new LayoutComponent/LayoutComponentStyle and disallow paint helper references from sizing or visible-position constraints. This establishes the dependency direction: independent slots → sizing → visible positions → corner/rounding/masks → paint.

Add a rounding/domain proof field and binding to Derived, with typed unresolved reasons for nonfinite corners, thin predicate uncertainty, viewport coverage and resource exhaustion. Existing private Candidate may remain explicitly unqualified, but `compose_with_bounds` must not return a Derived whose new paint prerequisites are unchecked.

## Implementation and validation sequence

1. The standalone evidence checkpoint is now consolidated in rounding-paint-review.md. Extract the reusable box emitter, add exact preflight and independent field reader, then integrate all ownership/cost/proof changes together.
2. Rust tests: both axes; all main/cross reversals and 0/.5/1 alignments; zero/one/multiple owners; exact aggregate cost and rollback; ordinary schema encoding; every new operand/default/mask/order mutation rejected; source feedback and cross-owner cache confusion rejected. Saturation reference tests include both edges beyond either limit, spanning the artboard, nextafter around limits and signed half ties. Thin threshold tests use nextafter(1/16), large origins and zero extent.
3. Rebuild the actual product and frozen Derived constructor. Construct each recipe twice and compare scene/map/trace artifacts. Existing old constructor artifacts and failures stay intact.
4. Rerun the actual 48-case Derived campaign against pinned Chrome and immutable native renderer using the same ordinary files through original/clone resize cycles. Observe all four masks plus line gates and draw chain. Retain existing 384 geometry frames and specifically account for the 48 formerly failing pixel frames. New shapes can change draw ordering despite identical geometry; validate alpha overlap regions and clear-color independence.
5. Add overlapping translucent owners on the same line and across lines, unequal visible extents, both reverse axes, one-to-multiple-line responsive transitions, zero/thin visible sizes and saturation boundaries. A single-owner success cannot transfer to these cases. Inspect all distinct image results and record exact repeat transfers.
6. Measure practical cost for 1, 3 and 8 visible owners, plus the accepted aggregate-budget boundary: encoded bytes, native import time, same-scene resize/update time, render time, clone time and release/lifecycle behavior. Use repeated runs and report hardware/build identity and distribution, not a single favorable timing. Keep malformed/over-budget rejection bounded before graph allocation. Roughly 2300 helpers per owner is material even when exact record arithmetic passes; record an explicit practical qualification or unresolved cost before public admission.
7. Keep public wrapping unadmitted until public parser/planner provenance, diagnostics, Rust/CLI/WASM/JavaScript parity and resource/lifecycle paths exercise the actual composition. No editor integration, runtime changes, browser-baked geometry, sidecars or resize recompilation.

This is an implementation plan. No product modification, native execution or new visual qualification was performed during this audit.
