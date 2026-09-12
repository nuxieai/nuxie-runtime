# Visible alignment without runtime changes

Source audit, 2026-09-12. This is a proposed ordinary-file composition, not native/Chrome qualification. No runtime files or compiler source were changed by this audit. The current private slot shape is definite, identity transformed, zero inset/margin/padding/border, with one untransformed visible layout child, fixed scales and positional alignment.

## Recommendation: land the existing native component anchor

Use an ordinary `ComponentOrigin` child of each visible LayoutComponent. Set the active cross-axis origin to the desired **physical** fraction a, and the other origin to explicit zero. Keep all sizing fields and the slot unchanged. Produce a root scalar target

    target = slot_anchor(f) + line_maximum * (a - f)

where f is the physical parent line fraction, slot_anchor(f) is the already available measured slot landmark, and line_maximum is the existing carry result. The final TranslationConstraint copies only the cross axis, with strength one, world spaces, offset false and no clamps. Its native anchor landing subtracts a times the visible component's **current native layout extent**, directly; no visible measurement node is needed.

The exact-real identity is:

    slot_top = line_origin + f * (L - h)
    slot_anchor(f) = slot_top + f * h = line_origin + f * L
    final_top = slot_anchor(f) + L * (a - f) - a * v
              = line_origin + a * (L - v)

Here h is slot extent and v is visible extent. This corrects the previous missing a*(h-v) term, including visible overflow. Under wrap reversal, convert both fractions to physical coordinates before emission. Algebra is not a claim of exact binary32 identity: the existing anchor and carry errors plus the new multiply, addition, origin multiply and final subtraction still need bounds.

### Native facts supporting this route

* `crates/nuxie-runtime/src/mechanical_port/source/component_origin.rs:15-29` marks the parent LayoutComponent as having an origin on import. Its setter callback marks world transform dirtiness, not layout dimensions. It is a Component, not a layout participant, and introduces no extra flex child. The origin is discovered from the owner's child list (`layout_component.rs:858-874`); require exactly one and reject preexisting origins in the closed base profile.
* `layout_component.rs:888-900` computes local_anchor as originX*layout.width and originY*layout.height for a non-artboard LayoutComponent. These are the actual published native sizes. It does not recompute authored percentages, infer bounds or depend on a diagnostic map.
* `layout_component.rs:944-980` only enters pivot composition when rotation is nonzero or scales differ from one. With the checked identity transform, adding an origin does not change the initial box translation or linear transform. `layout_translation` explicitly excludes the component's own origin (902 onward).
* `constraints/translation_constraint.rs:134-143` applies the translation and then calls `Constraint::land_anchor`. `constraints/constraint.rs:201-213` subtracts the world linear transform of local_anchor times strength. Under the checked identity linear transform, zero inactive origin and strength one, this is the desired active subtraction. The inactive coordinate is preserved; this claim requires all operands finite because even a zero coefficient times infinity produces NaN.
* The compiler already emits this same component-origin/final-translation combination for baseline alignment (`tools/html-to-riv/src/baseline.rs:169-183`). This is evidence of an existing file construction, not transferable wrapping qualification.

The constraint graph reads only unaffected slots and scalar helpers, then writes the visible transform. It never measures a visible transform that it also changes. The anchor is read during the final constraint from the native layout extent, so there is no independent visible measurement dependency to schedule. Native import, resize and clone tests must nevertheless establish correct layout-to-world update order; the source-level formula alone does not prove lifecycle behavior.

### Required implementation checks

Extend the generated-record allowlist with a narrowly validated ComponentOrigin attached only to the final visible role. Its exact fields must be parentId, active origin a, inactive origin zero. Update scalar validation to recognize the new slot-anchor target formula and final nonzero anchor; previous zero-anchor final-copy assumptions are obsolete. Do not change all helper origins: scalar helpers must remain zero anchored.

Bound the final pre-landing target and the final landed position separately. Use the visible native extent interval already in Domains for origin multiplication and final subtraction, including subnormal halving. Keep initial visible coordinates finite and ensure later constraint re-evaluations cannot accumulate drift: each execution overwrites the active coordinate with its fresh target before landing. Do not copy the inactive coordinate from a scalar zero target; its existing visible placement must survive.

Tests should cover row/column, reverse-main/wrap-reverse, every positional a/f pair, v<h, v=h, v>h, v=0, point/percentage/mixed min-max and conflicting minima, varied native slot placement, original/clone/repeated resize and preexisting-origin rejection. Compare actual native bounds as well as pixels to Chrome before public admission. The visible extent domains remain relevant to overflow and error bounds even though no scalar measurement is emitted.

## Alternative: absolute unpainted native layout proxy

If a future target lacks component-anchor landing, an absolute LayoutComponent under the same slot can expose native visible extent without constraining the visible owner. Clone the visible preferred dimensions and every min/max float/unit pair bit-for-bit into a new uniquely owned style, retain Fixed scale modes, add positionTypeValue=2, explicit left/top Point zero, no right/bottom inset, and zero margins/padding/border. Add no fill, stroke or descendant; do not use display:none, which suppresses layout. Measure proxy top and bottom through ordinary TransformConstraints; subtract to obtain extent with a bounded cancellation error.

In pinned Taffy `compute/flexbox.rs:520-529`, absolute children are excluded from flex item generation. The absolute pass runs after normal flow placement (388-390). Its `inset_relative_size` is the container size minus border/gutter (2167-2169), equal to the slot content basis only under this zero-inset profile. Preferred/min/max resolve independently against this same basis (2212-2233); known dimensions are clamped, and final size is reclamped (2262). With definite dimensions, no aspect ratio, fixed scales and no intrinsic sizing, the proxy and visible resolve the same native machine extent. Fixed scales give the visible zero flex grow/shrink (`layout_component.rs:3358-3362`), preventing a main-axis flex discrepancy. Reapplying the same clamp is idempotent, including minimum exceeding maximum.

This route needs a new explicit layout-isolation proof: the current blanket exclusion of generated LayoutComponent/Style records would correctly reject it. The absolute proxy changes topology and can change reported overflow content size (`flexbox.rs:425-428`), even though it does not alter fixed container dimensions or normal flex packing. Do not describe it as having no layout effects. Scroll/intrinsic/auto-size behavior falls outside the argument. Its absolute positioning fields must also be checked after import because the native position-type callback can initialize unset left/top from existing layout (`layout_component.rs:3023 onward`). Explicit serialized point-zero insets are essential; verify on native import.

This is feasible but more invasive than the anchor route: two additional layout records, extra landmarks, enlarged measurement coordinate budgets and lifecycle/topology checks are unnecessary when native land_anchor already reads the right extent.

## Alternative: scalar size calculation from a measured slot

Ordinary translation helpers can represent constant point values and sequential multiplication. The native percentage order is `(coefficient * parent_extent) * 0.01_f32` (`vendor/taffy-0.12.1-rive-yoga-order/src/util/resolve.rs:39,59,80`). A graph must retain that two-step order, not fold the coefficient into one multiply. Resolve preferred, minimum and maximum against the original slot basis, never the preferred or already clamped output. Native maybe_clamp performs maximum selection then minimum selection (`util/math.rs:113 onward`); a conflicting minimum wins.

The measured slot extent is already rounded differently from its native layout dimension when reconstructed from world landmarks. Its percentage result therefore need not equal the visible native result, especially at bound transitions. Moreover, the current graph maximum reconstructs `a + max(b-a,0)`, which can overshoot a native operand-selection max by an ulp. Reusing that construction for min/max adds error at every clamp and cannot be called exact native size resolution. Point-only static folding avoids some problems, but does not solve the requested mixed responsive case. Overflow before a finite clamp must still reject, and large scalar factors need new bounds.

This route introduces the most independent arithmetic assumptions and record cost. It remains an investigable composition, not an immutable-runtime impossibility, but should not be implemented for this correction while the component-anchor route uses the existing native extent directly.

## Status

Prefer ComponentOrigin plus slot-anchor/line-maximum target. Both alternatives remain documented, not qualified. This audit supplies source rationale and explicit outstanding obligations; it does not expose public wrapping, produce a new native campaign, or alter any retained visual failures.
