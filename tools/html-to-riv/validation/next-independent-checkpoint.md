# Next independent checkpoint: finish the wrapped sizing discriminator

Choose **L04 (`align-content`)**, sharing groundwork with L06 (`wrap-reverse`), after the current image checkpoint. The concrete deliverable is a compiler-private snapped sizing discriminator, paired with the existing snapped paint lowering and a precise admission-certificate boundary. This returns to an earlier pending layout item rather than adding more image combinations. The earlier pending typography rows still depend on unresolved A07/A08 text paint; their historical modified-runtime qualifications cannot supply a public route.

## Current immutable evidence and missing connection

- `validation/wrapping-module-review.md`: `src/wrapping.rs` reproduces all48 frozen three-item sizing outputs exactly. Its variable-count construction checks are broader than its native qualification; current public CSS does not call it.
- `validation/wrapped-snapped-gate-review.md`: ordinary abs/dead-zone/normalize/double/clamp repairs the65535.99609375 normalization residual to exact65536. Ten primitive scenes produce correct binary signals in80frames. The epsilon+0.0005 transition retains8failures because native DistanceConstraint returns early below0.001. This is not a general threshold proof.
- `validation/wrapped-snapped-layout-review.md`: snapped **paint** selectors pass384geometry/pixel frames and768 alternate-clear checks on48 actual dynamic layouts. All1,152 pair signals are exactly0/65536 and change across actual wrap boundaries. Sized intermediates deliberately remain byte-identical to the old unsnapped sizing stage.
- `validation/wrapping-paint-snapped-module-review.md`: actual `src/wrapping_paint.rs` reproduces those48 final files twice. No new visual qualification is inferred from that reproduction.
- Current `src/wrapping.rs::Graph::separated` still normalizes a signed difference directly, then takes its absolute value. In contrast, current `src/wrapping_paint.rs::Graph::snapped` applies abs, epsilon subtraction, zero floor, normalization, doubling and final clamp. The two parts of the same wrapped composition therefore still use different line-boundary arithmetic.

These are immutable-baseline ordinary-file experiments, not public wrapping admission. The current backlog summary understates the newer snapped-paint evidence; preserve that distinction when updating it.

## Proposed ordinary encoding

Use the existing snapped paint arithmetic for **adjacent sizing-slot anchors**:

`d = anchor[i+1] - anchor[i]`

`dead = max(0, abs(d) - epsilon)`

`separated = min(65536, 2 × normalize_distance(dead, 65536))`.

Use ordinary Nodes, TranslationConstraints and DistanceConstraint only. Feed this exact unsigned separation into the existing forward/backward line-maximum carry reset; keep the source slots independent of moved visible children. Keep existing positional alignment and reverse-wrap offset logic, and retain the existing snapped paint membership/leader implementation. The change belongs to compiler file construction, never the runtime or renderer. Recompute exact added-record preflight costs from the emitted graph rather than retaining the old52N−24 assumption.

For the first experiment use the already recorded experimental epsilon1/64, explicitly **not** a public semantic certificate. Do not repeat the primitive88-frame experiment or claim its failing transition repaired: this deliverable tests the previously unmodified sizing carry on real changing layouts.

## Finite acceptance

Reuse the exact48 authored inputs from `output/wrapped-snapped-layout-r1` and their frozen source recipes. They cover three slots, all four main directions, both wrap directions, positional line placement, percent/bounded-percent cross sizing, source order and nested alpha paint. Only the sizing graph changes; no authored sizes or descendants are dropped.

1. Emit each candidate once, then deterministically reproduce all48 RIV/map outputs. Bind current compiler-source and primitive-source identities separately from the preserved prior artifacts.
2. Import only ordinary bytes on the fixed baseline, with diagnostic metadata discarded. Observe each original and clone through the existing four-step viewport sequence:384 actual layout/pixel frames and768 alternate-clear checks. Reuse the pinned Chrome153.0.8010.12 and fixed native tools.
3. Observe all **768 adjacent sizing gates** (two per three-slot scene per frame), forward/backward carried maxima, and final visible child sizes/origins. Require exact0/65536 gates in this admitted experimental corpus, correct per-line maxima, and actual gate changes through resizing. Geometry-only success is insufficient.
4. Compare image bytes with the prior snapped-paint result. If complete PNGs and measurements match, bind their existing completed visual review by exact transfer. Directly inspect every changed full-resolution pair, retaining all pixel failures and original tolerances. A failed changed candidate is evidence, not a reason to modify the reference.
5. Keep the earlier threshold transition, zero-line/tiny-separation failures and finite-mask caveats linked explicitly. Construction tests for larger item counts do not qualify their native layout or quadratic paint cost.

## Proposed public scope, after the experiment

The first public route would admit explicit positional `align-content` and ordinary wrap/reverse-wrap only for scopes whose compiler can prove independent definite slots, a positive possible-line cross-size lower bound, bounded cross maxima/carry reset, anchor arithmetic error and finite viewport-mask coverage. Preserve source order and supported item alignment; scope initially excludes intrinsic/default-normal/stretch line distribution, arbitrary transforms, baseline/gap/helper combinations and nested independent wrap scopes.

A certificate must show `E_same <= epsilon`, distinct-line post-subtraction magnitude safely above native0.001, all carried maxima below65536 with arithmetic margin, and active/inactive masks valid throughout the stated resize/observation domain. It must derive epsilon from actual emitted arithmetic rather than treating1/64 or a depth limit as proof. Failure to derive the certificate remains a contextual unsupported-semantics diagnostic, not a runtime impossibility claim. Scene-wide record limits must charge actual sizing and paint graph costs.

**Next action:** freeze the current sizing and paint sources, implement the snapped adjacent-boundary candidate behind the private harness, and run this48-input changed-file comparison once. Its result determines the exact sizing arithmetic that the public certificate must bound. Do not reopen unchanged image campaigns or promote public wrapping solely because the existing paint experiment passes.
