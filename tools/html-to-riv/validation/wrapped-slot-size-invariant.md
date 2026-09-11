# Definite wrapping-slot size invariant and numeric bounds

For the definite, zero-inset flex slots used by `wrapped-combined-slots-r1`, the cross size used by native line alignment equals the final slot size read by TransformConstraint. The sufficient condition below follows the pinned Taffy code path; it is narrower than arbitrary definite CSS. It closes the target/final-size discrepancy term in the conditional anchor audit for this profile. It does not provide a positive separation bound for start/end anchors over every viewport in `(0,16384]`.

## Sufficient slot condition

Require a visible native flex-container slot, definite finite nonnegative width and height after resolution, no aspect ratio, zero margins/padding/borders/gap/scrollbar gutters, no relative inset, no intrinsic-size dependency, no native measurement callback, no baseline or stretch alignment of the slot, and no constraint changing its layout size. Resolve its preferred size and min/max against the same definite parent inner-size values passed to its final child-layout call. Child content may overflow but must not determine the slot's dimensions. Its descendants remain ordinary layout nodes. Require the parent's cross inner-size basis to remain unchanged from anonymous item generation through final placement; the definite parent with zero insets in the combined corpus satisfies this.

Nonnegative pixel/percentage preferred sizes and pixel/percentage bounds are permitted when finite and evaluated against that fixed basis. Auto preferred cross size, aspect ratio, nonzero padding/border, scrollbars, content measurement, or a changing percentage basis are outside this proof. Zero preferred size is allowed by the size invariant itself but may be rejected separately by the line-separation guard.

## Source proof

All references below are in `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs` unless specified.

1. Anonymous item generation resolves size, min and max against `constants.node_inner_size`, with the same aspect-ratio and box-sizing transformations (lines 530–561). Under this condition there is no aspect-ratio adjustment and the box adjustment is zero.
2. Hypothetical cross-size calculation resolves `child_cross = child.size.cross(...).maybe_clamp(child.min_size, child.max_size).maybe_max(0)` (1395–1402). Because the preferred size is definite, `unwrap_or_else` does not call the content-measurement branch (1410–1434). Denote the resulting binary32 value h.
3. Non-stretch used cross sizing copies this hypothetical value into `target_size`; `outer_target_size` adds zero margin and is also h (1615–1660). Cross alignment subsequently computes free space from this outer target (1752).
4. Final placement calls `perform_child_layout` with `item.target_size.map(Some)`, the same parent `node_inner_size`, and `SizingMode::ContentSize` (1936–1946). The tree trait forwards these dimensions unchanged in PerformLayout mode (`tree/traits.rs:380–403`).
5. The slot's flexbox `compute` keeps known dimensions ahead of style/minmax-derived dimensions via `known_dimensions.or(...)` (210–211), so both target dimensions remain known. In ContentSize mode preferred style sizes are not reselected (192–201). The child's native main size takes the known inner size and adds zero inset (288–291). The child's native cross size uses the known dimension and applies min/max again, with the same parent basis, then a zero floor (1885–1890).
6. That second clamp is idempotent: h was already clamped against those same machine min/max values. The helper implements max-with-min after min-with-max, so a conflicting min/max also stabilizes after one clamp (`util/math.rs:113+`). There is no extra percentage multiplication of h; the bounds are independently recomputed identically from the unchanged basis. With zero padding/borders/gutters, both possible relationships between the parent's and slot's axes preserve h.
7. The returned container size is published as the slot's `Layout.size` (1998 onward and final LayoutOutput). Runtime publication copies unrounded Taffy location/size into native Layout (`layout_component.rs:2289–2300`). Native rounding is disabled (2072–2073). TransformConstraint reads `LayoutComponent::local_bounds()`, whose extent is that exact published layout size (1135–1140).

Consequently target h and anchor-bound h are the same binary32 value, not merely close, for this condition. The `Esize` discrepancy term in `wrapped-anchor-error-audit.md` is zero. This claim concerns the size selected by native layout; it does not assert exact agreement with Chrome or real-number CSS sizing.

## Concrete combined-profile upper bounds

The combined corpus has parent width/height 50% of viewport dimensions, three item main sizes 60/60/40, and cross sizes:

- a: 50% of the parent cross size, or 35px clamped by min 50% and max 75% of the parent;
- b: 50px;
- c: 30px.

Let V be the viewport cross size, `C = fl(0.5*V)`, and assume finite representable viewport values in `(0,16384]`. Then C≤8192. Both a profiles are bounded by 4096: for the bounds profile, if C≤70 then the clamped result is at most35; if C>70, the 50% minimum dominates35 and is at most4096. All slots and all unstretched line maxima therefore satisfy `H≤4096`. There are at most three lines, so the sum of their extents is at most `3H≤12288`.

For start/center/end align-content with zero gaps, only the first visual line receives the positional free-space offset; subsequent offsets are zero (`common/alignment.rs:59–114`). Its magnitude is bounded by `max(C,3H)`. Accumulated line positions are bounded by that offset plus3H. Item positioning adds at mostH and anchor reconstruction adds at mostH. Thus slot/anchor positions relative to the shared parent fit within `max(C,3H)+5H`. For a root-origin parent the simple conservative envelope

```
B = 2*Cmax + 8*Hmax = 49152
```

covers those locations, shared offsets and scalar inputs to the anchor reconstruction. This deliberately uses a loose source-derived sum, not a measured maximum. It is an upper bound for the sizing/anchor profile, not for DistanceConstraint sentinel helpers or arbitrary painted descendants.

For a nested scene whose every admitted level independently satisfies those same C/H bounds, summing local envelopes gives `B_world ≤ D*(2*Cmax+8*Hmax)` for depth D. At D≤128 this is at most6291456 before an outward rounding allowance. This assumption must be checked per ancestor: viewport bounds alone do not constrain arbitrary pixel-sized overflowing ancestors. A computed interval guard should use the actual parent translation bound and actual line/slot bounds, rather than paying this loose depth bound unnecessarily. Same-parent ancestry is shared in the same-line error proof, so only the magnitude bound, not an additional depth multiplier in the operation count, is required.

For rigorous floating bounds, evaluate each nonnegative envelope with outward rounding and ensure every derived intermediate remains finite. The concrete endpoint C/H/3H above is exactly representable and monotone rounding cannot cross it. If different authored coefficients or counts are admitted, recompute their intervals; do not reuse49152 as a universal cap.

## Lower bounds and a concrete obstruction

There is no uniform positive lower bound for a's cross size over the open viewport domain. `0.5*C` tends to zero; the bounds profile's 75% maximum also tends to zero. For sufficiently tiny binary32 V, percentage multiplication can underflow to zero. Positive CSS/viewport input is therefore insufficient to prove a positive native line extent.

A concrete in-domain configuration uses main viewport1 and cross viewport0.001953125. Parent main size0.5 forces three separate lines. In the percentage profile a's cross size is exactly0.00048828125; in the bounds profile it is exactly0.000732421875. The order b,a,c puts this tiny line between fixed50 and30 lines. With zero gap, either start or end anchoring has an adjacent-line separation equal to the tiny line extent, below the immutable DistanceConstraint cutoff0.001. This follows from exact dyadic arithmetic; it is an analytical counterexample, not a new native replay claim.

For a fixed common fraction f, ideal adjacent-line anchor separation is `(1-f)*L_previous + f*L_next`, with visual order adjusted under wrap reversal. Therefore a guard should compute a lower interval bound for this expression over every possible adjacent partition. Upper H is not a substitute for that calculation.

There is a useful narrower exception: for center anchors in this exact three-item profile, two distinct adjacent lines cannot both contain only the one variable-sized item. At least one contains b or c, so ideal separation is at least15. Machine rounding and the previously derived anchor budget must still be subtracted before admission. This does not generalize to two percentage-only items on neighboring lines.

A guard can either restrict viewport to an explicitly declared positive lower bound that keeps every possible line separation above `2*epsilon + 0.001` plus gate-rounding margin, or reject start/end profiles whose interval lower bound reaches zero. Center-only qualification could use the partition-aware15 bound for this exact profile. No empirical threshold adjustment can fix the absence of a positive lower bound over the requested full open viewport domain.

## Scope of this audit

This is a source proof and bound derivation for a specific native slot shape. It does not modify runtime/public sources, run new native experiments, qualify other item counts, or erase the retained fractional raster failures. The snapped-gate prototype has its own normalization/threshold evidence; its operation budget and transition-region exclusions remain separate obligations.

## Immutable source bindings

Each file matches baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3` byte-for-byte.

| Source | SHA-256 |
| --- | --- |
| `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs` | `75f7a79993712f3f115762291819bf3aab530e79002746c63ad9ad500cc728c2` |
| `vendor/taffy-0.12.1-rive-yoga-order/src/compute/common/alignment.rs` | `f082c4a5b0607c0083c5b774c66b4006bd7324dc80159674df1141c7a20f0547` |
| `vendor/taffy-0.12.1-rive-yoga-order/src/tree/traits.rs` | `456ae6981c69509a246ab369698bd17999681880ae2e150b6a114de1dc59e2b2` |
| `vendor/taffy-0.12.1-rive-yoga-order/src/util/math.rs` | `ee37a509ac3eef3581571a27b167c03fc65c37ee4110cbbd4c89f7cdc149f920` |
| `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs` | `5154d46f471e2497886b2dbd420d5a0532baf8521f6cfe655a432c60d05122c3` |
