# Fixed wrapping: normal and stretch line distribution

Implement this as a compiler-owned slot composition. The immutable runtime already lays out fixed wrapping slots; increase each slot's cross size to the calculated stretched size of its line, retaining the visible child's original fixed size. This makes ordinary native line packing provide the spacing, while the existing visible-child alignment graph positions the fixed paint inside that space. No runtime or schema change is needed. This is a proposed construction, not qualified support.

## Formula and assumptions

Keep current public admission: one transparent fixed root, fixed empty leaf boxes, zero margins/padding/gaps, flex 0 0 auto, no baseline alignment, stable CSS order, and admitted normalized source dimensions. Clamp each preferred dimension by its bounds before computing packing, with minimum winning over maximum. Both root dimensions are fixed, so the line partition is independent of viewport resize; only artboard clipping changes.

In logical main order, pack used main sizes into root inner main size M, placing an oversized first item on its own line. Let K be the resulting nonempty line count, H[j] the maximum used cross size of members of line j, C the root used inner cross size, and S=sum(H[j]). The real-number target is:

```
F = max(C - S, 0)
D = F / K
T[j] = H[j] + D
B[j] = sum(T[q] for q < j)
y_logical(i) = B[line(i)] + a[i] * (T[line(i)] - visible_cross[i])
```

For reverse cross flow, convert to physical start with `C - y_logical(i) - visible_cross[i]`. Fractions a are relative to logical cross start; preserve the existing conversion for physical start/end. When F is zero, ordinary flex-start packing is the stretch overflow fallback. Fixed visible dimensions never grow merely because line boxes grow. `normal` uses stretch behavior in this admitted flex context. This follows the [CSS flex cross-size algorithm](https://www.w3.org/TR/css-flexbox-1/#algo-line-stretch); the specification does not establish Chrome's finite-precision remainder allocation.

Do not implement the formula by rounding one f32 D and assuming browser identity. Derive pinned Chrome's LayoutUnit division/remainder and positional accumulation order before admitting nonexact distribution. First implement the exact subset: integer LayoutUnit H/C, F divisible by K in LayoutUnits, and all emitted sums/coordinates exactly representable in f32. For example C=100px with three 20px lines leaves a nonexact 40/3px addition and must retain an explicit numeric-qualification diagnostic in that first subset. C=120px with three 20px lines has exact D=20px. Negative/zero free space is also exact without division. This is a bounded first implementation, not a claim that fractional distribution is impossible.

## Concrete ordinary-file construction

1. Derive a retained source certificate from the normalized parent/leaf six-field dimensions and stable order. Its line membership, used dimensions, T values, and arithmetic predicates must be reproducible without browser measurements. Verify that the frozen native packing arithmetic and integer LayoutUnit packing produce the same partition, including exact-fit boundaries and oversized items.
2. Preserve root dimensions and direction/wrap-reverse. Use native positional cross-start alignment. For every member of line j, emit a slot with its original used main size and cross size T[j]. Its visible child retains the original six authored dimensions. Slots must not retain authored cross max bounds that could undo the expansion: derived slot preferred/min/max metadata describes the generated slot, while a separate source certificate retains the child's actual authored bounds.
3. Give all slots on a line the same T[j]. Because only cross dimensions change, main packing remains unchanged; the native line maximum is now T[j]. Invoke the existing `wrapping::align_items` with cross-start line fraction and original per-item alignment. The existing anchored formula uses the expanded maximum and positions center/end children correctly. Slot measurement does not depend on moved visible children, preserving the acyclic construction.
4. Reuse paint selection and ordinary/rounded paint binders only after re-proving their domains for expanded slots. Original paint may remain available for integral contained rectangles; it is not automatically certified just because slot and visible IDs are unchanged.

This avoids runtime line-count division or data-dependent multiplication. A later responsive-parent extension can use certified boundary signals and finite line-count branches; it is unnecessary for the present fixed-root public scope.

## Source seams and immutable capability

- `src/public_wrapping.rs:135–137` currently rejects Content::Normal/Stretch through `fraction(None)`. Add a separate stretch plan branch, not a fabricated positional fraction.
- `src/public_wrapping.rs:169–174` currently clones each child's layout into both slot and visible owner. This is the main construction seam: produce derived expanded-slot metadata separately from authored visible metadata.
- `src/wrapping.rs:86–135` measures slots, detects boundaries, carries their cross maxima, and lands visible anchors. No new target primitive is required for the expanded-slot construction.
- `src/wrapping_domains.rs` and `src/wrapping_sizes.rs` bind generated preferred/min/max fields. Add a source-derived slot certificate instead of forging authored literals with `exact_constant` or weakening existing source binding.
- `src/wrapping_position.rs:40–43, 103–135` already models distinct slot/visible extents and carried line maxima. Extend the proof chain to bind the stretched-slot source formula and the partition preservation premise.
- `src/wrapping_slots.rs` permits distinct visible sizes, while composition independently binds original paint and scalar records. Preserve those checks.
- Immutable `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_component_style.rs:274–351` couples exposed positional item/content alignment to FlexStart/Center/FlexEnd; there is no exposed stretch-content choice among those mappings. Do not exploit an unknown enum/default to reach stretch.
- The underlying vendored engine does have stretch: `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:1578–1600` computes remaining space and divides by line count. Its existence does not create an exposed ordinary-file encoding. Expanded fixed slots use the exposed positional path instead.

## Required proof and validation

Unit tests must cover source-bound partition/cross calculations, all six bound fields, min-over-max, zero free space, cross overflow, one line, unequal heights, oversized main items, order ties, exact fit and one-LayoutUnit adjacent boundaries. Include nondivisible free-space diagnostics. Mutation controls must reject changed authored bounds, another same-ID source scene, wrong line membership, wrong expanded field bits, and altered slot cross bounds.

Use the frozen public constructor plus the unchanged Chrome/native visual driver. Exercise row/column and both reversals; align-self start/center/end/physical start/end; 1/2/3/many lines; transparent children; initial/clone/repeated artboard resizes; viewport clipping on every edge. Include differing line maxima so an implementation that merely inserts equal gaps fails. Check actual visible geometry and paint streams as well as pixels, and directly inspect every new representative pair. Keep the current normal public corpus byte/map regression campaign.

Next implementation: add a private source-bound `fixed_stretch_plan` module for exact LayoutUnit division and native partition equivalence, with generated slot metadata and no public admission. Construct the expanded-slot scene through the existing closed composition, then qualify its Chrome geometry, stream, and visual results before enabling the public Normal/Stretch branch. Fractional remainder allocation is the subsequent explicit extension.

## Private implementation checkpoint

`src/wrapping_stretch.rs` now implements retained parent/child source witnesses, derived slot dimensions, bounded exact LayoutUnit distribution, native-float partition agreement and independent field validation. Five focused tests pass, covering both axes, unequal/overflow lines, zero/oversized/exact-fit items, conflicting bounds, ±1 LayoutUnit partition boundaries, nondivisible remainders and source/slot/axis mutation rejection. `compiler.rs` registers the private module only; the public planner still rejects normal/stretch. Next connect the validated Plan to emitted slot metadata while preserving visible authored dimensions, then run public transport and native/Chrome qualification. No new public support or full product build is claimed by these unit tests.
