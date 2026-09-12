# Rounded wrapping paint domain review

Read-only source review, 2026-09-12. No concrete defect found in the new arithmetic certificate under the checked composition's identity transforms, source-size bounds and dependency ordering. This review adds no pixel, scheduling or public CSS qualification. Product sources were frozen and not edited by the reviewer.

## Native operation correspondence

`LayoutComponent::constraint_bounds` returns `local_bounds`, constructed at (0,0) with the actual native layout dimensions. `TransformConstraint::target_transform_for` constructs the local origin and multiplies it by the target's world matrix. With the separately checked identity linear entries and origins (0,0)/(1,1), the selected translations are numerically p and fl(p+s). Multiplication by 0/1 and the identity matrix does not introduce another nontrivial rounding step; signed zero may normalize.

The thin helper first world-copies the trailing corner onto a Node parented under the leading corner. Its root reader uses TranslationConstraint sourceSpace Local. The runtime inverts the target parent's world transform and multiplies by the target world matrix before selecting translation. For identity matrices that inverse contains exactly -p and the selected result is fl(fl(p+s)-p). The emitted source-space setting therefore agrees with the modeled q, rather than obtaining an independent exact native width.

The two gain/clamp predicates operate on fl(q-1/16). The sign test agrees with q>1/16: around the threshold the binary32 subtraction is exact; outside that neighborhood its rounding cannot reverse a nonzero sign. Four-stage positive saturation is separately native-evidenced by the existing scalar experiment. This review does not transfer that evidence to browser LayoutUnit equality.

## Bounds and rejection

MachineInterval admits only finite nonnegative dimensions. Monotonic binary32 endpoint additions enclose trailing corners. Independent endpoint subtraction encloses all local differences and rejects an overflow even where correlation could have proved a narrower result. This is conservative rejection, not an unsound acceptance.

The q-s error bound preserves the repeated p: first charge addition error at |p|+s, then charge the final subtraction at s plus that first error. Binary64 next-up operations and a full binary32 minimum subnormal allowance conservatively enclose the positive error calculations. Point domains directly evaluate native addition/subtraction; the zero-size case is exactly numerical zero. The interval branches only admit a uniformly true or uniformly false threshold predicate. A size interval straddling the threshold can remain unresolved even if its per-viewport graph works; this is an intentional sufficient certificate boundary.

The active coordinate bounds come from final landed positioning; the orthogonal coordinate comes from initial visible world bounds. Initial owner transforms are finite under wrapping_coordinates, and corner Nodes themselves initially have zero translation. Correct dependency evaluation is essential: corner constraints must read the final visible owner after positioning, then thin helpers must read both final corners. The domain function is conditional on this checked target DAG; it is not independent proof of runtime scheduling. It does not need to evaluate hypothetical corner reads before the source's positioning constraint when that dependency order holds.

## Saturation and paint extent

For nonnegative dimensions, raw trailing edges are ordered after raw leading edges. Rounding and clamping are monotone. If both edges are below -16384, the saturated interval ends no later than -16383 after the one-pixel minimum and stays outside the artboard. If both exceed 16384, the saturated leading edge is at the artboard boundary and contributes no interior coverage. For a spanning interval, any edge that can bound visible coverage is unchanged. This validates saturation for an origin-zero artboard whose extents are at most 16384; it would not validate clamp-to-zero.

Rounded endpoints are integers in [-16384,16384]. Adding the binary flag, subtracting the minimum from the rounded trailing edge, clamping locally and adding the minimum back remain exactly representable binary32 integer operations. Final trailing edges lie within [-16384,16385]. Each 32768-square edge mask and the origin-zero paint rectangle covers the necessary complementary extent of the admitted artboard. The required inherited artboard clip remains a separately bound premise.

## Limits of this review

No arbitrary transforms, different origin conventions, wider viewport domain, unbound traces, or unscheduled transient corner reads are admitted by this reasoning. The certificate proves the thin predicate against the native dimension, not Chrome's layout quantization. Actual same-file original/clone resize, mask geometry, overlap order and Chrome/native pixels still require their native campaign. No tolerances were changed and no previous failures were removed.

## Reviewed source identities

- `tools/html-to-riv/src/paint_box_domains.rs`: `effe43f2923052276ba78cb4e0393cd5d6f9b04c02f8fc5960dc00d5fd067feb`
- `tools/html-to-riv/src/paint_box.rs`: `b7d48547f7b3db19010e83b128a219ba6f7b1b4085ee005b4da7c20541574297`
- `tools/html-to-riv/src/paint_box_binding.rs`: `10057b168d8badf5c9ca432d7a68f24b26211878d32e2eb7b712e8e35c2beda8`
- `tools/html-to-riv/src/wrapping_coordinates.rs`: `4ef3dc247187d7f75a6be8a5b807784fe3e0d74835c9173c10e3f1ad44b3b1db`
- `tools/html-to-riv/src/wrapping_position.rs`: `dd204e22c7a71cabefb2108ceec09d42b062f215fb53ffa71c2cb2fe371030b0`
- `tools/html-to-riv/src/wrapping_composition.rs`: `3b2fb3272c2e13df0aeb3f606976e977b05f6a27d797ea94adba6370d8c55f1a`
- `crates/nuxie-runtime/src/mechanical_port/source/constraints/transform_constraint.rs`: `450f0443e9499db7c5b7a4261d77fe5ede678e40f975ec0b77278c6b5b240f8d`
- `crates/nuxie-runtime/src/mechanical_port/source/constraints/translation_constraint.rs`: `b7d9eb0c64ebb1da20fbf37a296e16c41e8ed42b437d59c9f849e39b8d5eac14`
- `crates/nuxie-runtime/src/mechanical_port/source/math/mat2d.rs`: `f7a43f39caef3529a3ee1a16f8b0dd06d9370f09d8185ecbf01606c1ff475ed2`
- `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs`: `5154d46f471e2497886b2dbd420d5a0532baf8521f6cfe655a432c60d05122c3`
