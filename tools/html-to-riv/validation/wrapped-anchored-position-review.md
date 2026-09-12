# Anchored visible-box positioning: conditional arithmetic review

This checkpoint replaces the private slot-height correction with an anchor on
actual visible geometry. It does not admit public wrapping or qualify native
pixels. Runtime files remain unchanged.

## The repaired expression

Let `p` be the actual native slot cross position, `h` its actual extent, `L`
the native maximum slot extent on its line, `v` the actual visible extent,
`f` the physical line fraction and `a` the physical desired fraction. Fractions
are 0, 1/2 or 1; wrap reversal maps each authored fraction to `1 - fraction`.
The emitted target is the measured slot landmark `p + h*f`, plus the propagated
line maximum times `a-f`. A ComponentOrigin on the visible layout component
has active fraction `a` and inactive fraction zero. The final world translation
constraint lands that origin, producing

```
p + h*f + L*(a-f) - v*a
= p + (L-h)*(a-f) + a*(h-v).
```

The last term was missing in the old graph. The retained algebraic regression
uses line origin 7, line extent 80, slot extent 50 and visible extent 20. It
checks all physical fractions. A slot-aligned end target differs from the
required visible-box end target by 30 units; this is a semantic counterexample,
not a new observed Chrome/native capture.

## Immutable source operation binding

`crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs`:

* `origin_offset_with` (888–892) multiplies the fractions by actual layout
  width/height; `local_anchor` (894–899) exposes that origin.
* `build_own_transform_with` (944–980) enters the pivot composition branch only
  for nonidentity rotation/scale. The closed base has rotation zero, scales
  one, and zero authored transforms. Adding ComponentOrigin therefore does
  not alter its initial own transform or invoke large-pivot cancellation.
* `compose_world_transform` (989–1013) preserves the previously audited finite
  initial world coordinates under that identity premise.

`constraints/translation_constraint.rs` (134–142) copies with strength one,
then calls `Constraint::land_anchor`. Both old coordinates must remain finite
because `old * 0` does not erase infinity safely.
`constraints/constraint.rs` (201–212) transforms the local anchor by the world
linear matrix, scales it by strength, and subtracts it from world translation.
Identity linear entries and strength one make the active transformed anchor
an exact copy of the rounded `v*a`; the other coordinate subtracts exact zero.
Finite visible width/height also make inactive `extent * 0` safe.

## What the bound establishes

`wrapping_position::prove` additionally receives the closed composition's
`wrapping_domains::Slot` slice. The root must bind that slice and coordinates
to the same base, item ordering, row axis and fractions. For each item it:

1. Encloses the ideal `p+h*f` using slot size and native-location domains,
   then expands by the existing conditional 31-eta landmark budget.
2. Bounds the actual maximum scaling and its inherited carry error.
3. Bounds the target sum and rounding error.
4. Bounds `v*a` against actual visible native size, including subnormal
   halving (not presumed exact).
5. Bounds the final subtraction and its absolute error against the expression
   above. Every machine interval rejects nonfinite endpoints.

`target_error` describes the prelanding target. `landed_error` describes final
visible translation. Neither includes Chrome-versus-native placement error.
The coordinate proof supplies finite initial translations on both axes.
After landing, the active coordinate is finite by this proof and the inactive
coordinate remains finite. This gives an induction over repeated evaluation,
conditional on graph binding and dependency execution.

## Verification and remaining gates

Unit tests simulate all partitions of five distinct slot extents, signed
positions, all authored fractions and both cross reversals. Visible extents
include zero, half, equal and twice slot extent. They check actual machine
intermediates and final error against ideal real arithmetic. Additional tests
cover input error cancellation, subnormal offset/origin products, overflowing
target sums and landing subtraction, invalid fractions and invalid errors.
These are arithmetic tests, not native rendering tests.

Generated record fields, ComponentOrigin ownership, graph evaluation,
layout isolation with the added origin, clipping coverage, actual native
original/clone lifecycle, and Chrome pixel comparison require their own
checks. In particular this source audit does not claim that arbitrary visible
contents or arbitrary host transforms are supported.
