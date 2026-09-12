# Exact fixed stretch: pinned source review

Read-only review of the integrated `wrapping_stretch.rs` and public dispatch against Chrome commit `971a7443b0c9b0a9b2860529b33331b76077ec62`. No concrete missing admission predicate was found for the exact-divisible line-size construction. This is source review, not native/browser qualification; no build or capture was run.

## Checked assumptions

- Chrome `flex_layout_algorithm.cc:1750–1778` subtracts line cross sizes and gaps from the container cross extent, treats normal/default as stretch, and distributes nonnegative remaining space. `wrapping_stretch::calculate` uses the same zero-gap formula and checks divisibility in raw LayoutUnits, not whole pixels. Its raw upper bound 4,194,304 leaves every admitted sum exactly representable in f32; the native partition comparison is an additional explicit check.
- Chrome sets `is_multi_line_` from `!ResolvedIsFlexNowrap()` at line 178. A wrapping container that happens to have one line still follows the stretch branch. The special unconditional container-cross-size replacement at 1767 belongs to nowrap, not K=1. The plan's `max(C-H,0)` for a single wrapping line is correct, including an oversized cross-axis child.
- Empty items are rejected by the plan; nonempty zero-size items remain members. Chrome's pinned `GreedyBreakFlexItemsIntoLines` breaks only when `count != 0 && next_sum > line_break_size`, matching the plan and frozen native collector. A zero-main item after an oversized item starts a new line if the existing sum still exceeds the available size. Zero-height lines are legitimate; later arithmetic separation proof may conservatively reject them. Do not silently drop them from K.
- `ApplyReversals` at 1621 reverses the line vector for wrap-reverse and reverses item indices within each line for reverse direction. It runs before final positioning at 1299, therefore before the diffuser. Exact-divisible additions are identical for every line, so the current plan does not need a reversal parameter: line heights remain attached to their members and the ordinary parent supplies reversal. This will no longer suffice for remainder allocation.
- Negative free space falls back to flex-start for stretch and normal (`InitialContentPositionOffset`, 1660–1684). On reverse cross flow that includes the negative free-space offset. Public dispatch's positional flex-start base preserves the reverse-cross flag, rather than changing it to physical start, so the expanded-slot construction retains this behavior.
- Both root and child used sizes apply min/max before partition/maxima, with minimum winning. Generated slots remove authored cross bounds so expansion is not clamped away; original visible children retain all six authored fields. With fixed visible cross dimensions, align-self normal/stretch does not enlarge the child. Zero padding, border-free admitted boxes, no intrinsic children and zero flex factors are essential existing public predicates.
- Public dispatch binds the plan against the sorted source children and constructs each role from `plan.slots()[index]`; visible metadata remains the original `dims`. `resolve_layout` then binds the actual records, so the generated constants do not substitute for visible-source provenance.

One precision limit remains worth naming in evidence: exact line sizes do not imply exact subpixel item coordinates for center alignment. An odd raw LayoutUnit difference divided by two is truncated by Chrome's LayoutUnit arithmetic, while native f32 can represent the half-unit position. This is the existing fixed positional alignment distinction, bounded by 1/128px for this individual operation, not a newly discovered line-stretch failure. Test odd-unit centered children explicitly and retain actual geometry deltas; do not call the whole scene geometrically identical solely because stretch division is exact.

## Authoritative remainder implementation

Pinned header: [layout_unit_diffuser.h](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/layout/geometry/layout_unit_diffuser.h). It was fetched and inspected directly during this review. For nonnegative free space raw F and K buckets, its behavior is:

```
base = F / K; remainder = F % K
x = 0; y = K; dx = 2*remainder; dy = 2*K
repeat K times:
    x += dx
    if x >= y: y += dy; emit base + 1 raw unit
    else: emit base
```

This spreads the remainder, rather than giving all extra units to the first or last lines. For K=3 and remainder 1, additions above base are [0,1,0]; remainder 2 gives [1,0,1]. The sequence is assigned in the already-reversed line vector. An eventual certificate must retain reverse_cross and map those allocations back to source line membership before generating slots. Reverse main affects member order, not bucket order. Preserve the existing exact-sum bounds and source-to-slot binding when extending the plan; do not approximate with one rounded float quotient.

Additional pinned source checked: [flex_line_breaker.cc](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/layout/flex/flex_line_breaker.cc), including `BreakIntoLines` and `GreedyBreakFlexItemsIntoLines`. Local full flex algorithm is retained under `output/wrapped-boundary-quantization-source-r1/third_party/blink/renderer/core/layout/flex/flex_layout_algorithm.cc`.

Before public qualification, retain explicit cases for K=1 overflow, zero-main members after oversized items, min-over-max with a visible cross maximum, unequal line heights under both reversals, zero free space, and centered odd LayoutUnits. For eventual remainder support add K=2/3/4/7 allocation tests and wrap-reverse cases with unequal line heights; symmetric equal-height examples can conceal a reversed bucket assignment.
