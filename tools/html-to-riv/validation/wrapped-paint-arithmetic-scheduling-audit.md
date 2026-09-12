# Paint arithmetic, dependency and replica audit

This is a source audit of the private paint composition at the source hashes below. It makes no public admission claim, no renderer modification, and no new native or Chrome capture. The existing coordinate/normalizer analysis is sufficient for the paint signal's arithmetic **conditional on binding the emitted paint fields and the unchanged layout premises**. Mask coverage, final-record paint validation, and native visual/lifecycle evidence are separate obligations.

## Source-local subtraction

The paint helper uses a different construction from sizing `diff(a,b)`. Its anchor TransformConstraint first produces a full two-coordinate world transform. For the active cross coordinate let its actual finite machine values be A and B. `Graph::signal` then constructs:

1. A root child `source`, world-copying only the active coordinate of anchor a. Its active world coordinate is exactly A and its inactive world coordinate remains zero.
2. A child of source, world-copying only the active coordinate of anchor b. Before its constraint it has world coordinate A; after it, its world coordinate is exactly B. Its inactive coordinate stays zero because its parent is scalar and its authored translation is zero.
3. A root child, copying the preceding child's **source-local** coordinate. Its target parent is source, so this is `fl(B-A)`; it is not B, nor the child's authored zero translation.

`translation_constraint.rs:63–68` applies `inverse(target_parent_world) * target_world`. `Mat2D::invert:181–195` has determinant exactly one for identity linear matrices and finite translations: its translation is exactly -A. The matrix product at `mat2d.rs:255–265` reduces to one rounded addition B+(-A). Multiplication by one and zero terms are exact. The subsequent factor-one, strength-one TranslationConstraint weighted sum is exact provided both the old and new coordinates are finite. Signed zero does not invalidate these geometric equalities.

Thus the paint path has at most one potentially rounding pair subtraction after the anchor values. The existing allowance of three pair operations in `wrapped-anchor-error-audit.md` is sufficient; it does not need an increased coefficient just because it uses source-local space. The existing 31 eta same-line and (31+4N) eta different-line budgets remain conservative for this path. The sign is opposite the sizing difference, but the next absolute-value stage removes that difference.

This argument requires the actual record defaults: strength 1, source/destination space world except the explicitly source-local copy, copy factor 1, offset false, no clamps, zero authored helper origins/rotation/skew/translation and unit scale. The active `doesCopy` flag is true and the inactive flag false. A type/parent-only graph validator does not bind these facts. In particular, accidentally setting destination-local instead of source-local on the third copy would produce B instead of B-A, and allowing a nonzero inactive coordinate would invalidate scalar DistanceConstraint normalization.

## Full anchors and intermediate finiteness

Paint TransformConstraint anchors retain both coordinates before the scalar source copy. The source-coordinate result being finite is insufficient: a nonfinite inactive landmark can enter an otherwise zero matrix term, and `old * 0` cannot repair infinity or NaN. `wrapping_coordinates.rs:148–161` explicitly checks full landmark coordinate ranges against both slot dimensions before its scalar proof. Preserve this check and the closed identity/zero-parent shape; do not replace it with a cross-coordinate-only envelope.

The scalar child has unconstrained position A, and B-A is bounded by the independent anchor span plus the pair error budget already used by `wrapping_coordinates`. The source cache introduces no sum of A+B: it is a world copy to a child whose previous world position is A. Both values are finite. Inversion involves only exact sign changes and determinant one. The radius must continue to bound the native layout/anchor expressions; viewport size alone is not an adequate replacement.

## Absolute value, normalization and leader inversion

For d = fl(B-A), the negative copy is exactly -d. `snapped` parents the absolute node to d, targets -d in world space and applies local minimum zero. For d >= 0 it returns d exactly; for d < 0 it returns -d exactly, provided 2*d is finite. In the latter case the intermediate local difference is -2*d, exactly representable as a power-of-two scale, and reconstruction cancels to -d. This exact identity is special to max(d,-d); general carry maxima do not share it.

The coordinate proof checks finite twice the conservative absolute upper bound. Subtracting the nonnegative emitted epsilon is the same native rounded addition as subtracting it; relu produces exact zero on the same-line branch when |d| <= epsilon. For the different-line branch, the actual dead interval is enclosed by the coordinate proof. The conditional lemma in `wrapping_normalizer.rs` then proves exact D = 65536 after scalar DistanceConstraint, doubling and upper clamp. It requires zero target, zero local anchor, zero inactive coordinate, mode 2 and strength 1, and the analyzed finite/normal square, square-root, quotient, projection and lerp intervals. Merely observing nonzero d is insufficient because of the native 0.001 early-return region and the strength-one DistanceConstraint lerp cancellation.

The leader node authors D on the active axis, targets the gate with factor -1 and `offset=true`. The offset uses the component's authored/composed translation, not its previous constrained world translation (`translation_constraint.rs:56–60,80–91`; `transform_component.rs:219`). With a root identity parent it therefore computes exactly D-gate on every update. For gate 0 or D the result is exactly D or 0. Repeated updates do not accumulate the offset. The inactive axis stays zero. Bind these fields explicitly; an offset-false copy gives -gate and does not implement the complement.

## Transform dependency order

The numeric graph does not rely on record order alone. `TargetedConstraint::build_dependencies` registers the owner as a dependent of its target. `TransformComponent::build_dependencies:102–113` registers a transform child as a dependent of its parent. `Constraint::on_added_dirty` appends constraints to their owner's list in import order; `TransformComponent::apply_constraints:188–194` applies that list in order. In particular, the normalized node's scalar copy must run before its DistanceConstraint.

For this closed paint graph the dependency chain is slot -> anchor -> cached source -> difference -> source-local scalar -> abs/dead -> normalized -> gate -> mask shape/path -> clip. Each target and parent is an earlier helper except the already-audited base slot targets; source caching only shares an upstream source. No paint target points to a visible child or back from a mask into slot layout. Parent/target edges therefore form a DAG in the closed construction. `artboard.rs:1397–1415` and `DependencySorter` put dependencies before dependents, and the normal update walks that order. Shape path composers register clip dependents (`clipping_shape.rs:289–310`), so changed gate geometry dirties the clip path.

This supports the absence of a newly introduced paint feedback cycle. It is **not** a blanket proof that arbitrary imported scenes always settle: the runtime update loop caps at 100 passes (`artboard.rs:1845+`), and the dependency sorter reports a cycle instead of repairing it. Qualification must bind the closed graph, preserve no-layout-feedback checks, and still verify first import, same-instance resize, clone resize, repeats and render-after-layout using native bytes-only observations. An arbitrary host that draws without running the runtime's normal update is outside this argument. No extra host CSS update is required or proposed.

## Exactly one active paint replica

Assume exact 0/D line signals, contiguous logical line partitions, positive distinguishable line extents, and masks whose zero position covers the complete observation domain and D position excludes it. For logical line [l,r], candidate group i is enabled iff i=0 or i starts a new line (leader mask). Replica (i,j) is emitted only for j>=i and is enabled iff j=i or j shares i's line (membership mask). For any item j in [l,r], exactly group i=l satisfies both conditions. Other earlier line leaders fail membership; earlier items of the same line fail leader; later groups either do not emit j or fail leader. Unpainted items still participate in line/leader determination, as required by `paint_records` and `paint_items`.

The proof applies to **mask predicates**, not yet raster pixels. Current mask bounds [0,32768] on each local axis have no negative margin. Original paint outside that rectangle can be cut, and antialias/transform rounding at an observation boundary needs a separately established coverage argument. The rectangle-intersection route in `wrapped-mask-coverage-audit.md` can establish equivalence without an arbitrary margin under its clipped-artboard, AABB and equal-matrix premises. Without an observation-domain restriction, translated inactive masks can intersect visible content (for example content at the D offset), so exact gates alone cannot prove suppression everywhere. Do not describe existing fixed masks as globally transparent/empty. This remains a mask-coverage obligation, not evidence that the composition is impossible.

The emitter visits groups in physical cross order (reversed when wrap reverses) and members in physical main order, skipping j<i. Each item's authored paint list order is preserved. Inactive replicas may be interspersed but cannot change the relative order of the active subsequence. This yields the intended physical line/item paint sequence; relating that sequence to browser painting for each admitted CSS profile still needs the Chrome comparison and semantics restrictions (no unmodeled z-index/stacking context).

## Ordinary draw-rule chain

Every foreground replica except the last has its own DrawRules pointing to a DrawTarget whose drawable is the next replica and placement is 1/After. `DrawTarget::placement` confirms 1 means After in the linked list, which initially sounds reversed. However `artboard.rs:1235+` makes the last linked drawable the first drawing occurrence, and drawing follows `prev_drawable` (`:2380+`). Thus placing replica k after k+1 in the list causes k to paint before k+1.

Artboard import finds the nearest flattened draw rule for each drawable (`:939–954`). The closed graph puts the rule directly on each foreground, and forbids inherited base draw rules. Artboard builds draw-target dependencies from a target drawable's own flattened rule (`:1040–1102`), so the terminal chain is installed first; consecutive insertions then produce the full reversed linked list. The targets are acyclic because each points to the next replica. The explicit chain therefore fixes relative replica order despite the runtime's foreground relocation during import. Intervening mask shapes carry no fills; originals are made transparent; the closed base/paint preservation validator must continue excluding additional visible drawing objects.

## Remaining checks before qualification

- Bind the complete emitted paint records, defaults, per-owner constraint order, source-space difference, leader offset, masks, clip attachments and draw-rule chain to these semantics. Reemitting an identical graph by itself is not a source argument.
- Establish mask coverage and inactive exclusion, including transform rounding and the native raster observation domain, using a file-level construction if necessary.
- Independently inspect native imported ordering/clip activity and pixel results on first import, repeated resize and clones. Existing sizing-only position success does not certify paint.
- Preserve same-line rounding and small-separation failures as rejection evidence; the sufficient coordinate route does not prove arbitrary wrapping possible or impossible.

## Source identity

Every native file below was compared byte-for-byte with baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3` during this audit and matched. Compiler identities describe this audit's inspected source, not a claim about future edits.

| Source | SHA-256 |
| --- | --- |
| `tools/html-to-riv/src/wrapping_paint.rs` | `8c28ce23bdf732b34612e120e88f891e5a0df78e99ba312c3c4c0e0fe319bca2` |
| `tools/html-to-riv/src/wrapping_coordinates.rs` | `4ef3dc247187d7f75a6be8a5b807784fe3e0d74835c9173c10e3f1ad44b3b1db` |
| `tools/html-to-riv/src/wrapping_normalizer.rs` | `782dee7383b6df8b4ab5be43a512c0a61cee9aab20b802dca03589c5ea28d346` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/translation_constraint.rs` | `b7d9eb0c64ebb1da20fbf37a296e16c41e8ed42b437d59c9f849e39b8d5eac14` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/transform_constraint.rs` | `450f0443e9499db7c5b7a4261d77fe5ede678e40f975ec0b77278c6b5b240f8d` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/targeted_constraint.rs` | `3e2b462b6ecaf322ab7c333542c649a7c4ff3904f103b5c67001a885fd068b84` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/constraint.rs` | `d89f7c20f18c2f3ca437129d3098a6f98b11326bd0ad121aa932570bd64f364e` |
| `crates/nuxie-runtime/src/mechanical_port/source/math/mat2d.rs` | `f7a43f39caef3529a3ee1a16f8b0dd06d9370f09d8185ecbf01606c1ff475ed2` |
| `crates/nuxie-runtime/src/mechanical_port/source/transform_component.rs` | `861f8b7d460f7cbb07456901acf19e70720bfd7559d8a50964bbbc00ddff6cac` |
| `crates/nuxie-runtime/src/mechanical_port/source/artboard.rs` | `6f218337cbf8daa1567f20244cadd9f0676cb1e6546ae0bc23751d02b404da01` |
| `crates/nuxie-runtime/src/mechanical_port/source/dependency_sorter.rs` | `4f0398127d6c592fff4e8b089156a24fe00063f8433d78af2ffb0f55a55dd0a4` |
| `crates/nuxie-runtime/src/mechanical_port/source/shapes/clipping_shape.rs` | `466f8cb71255d1c082efdd41b27ee4f31b2f11cc3f4bdb5e512cf798539732d3` |
| `crates/nuxie-runtime/src/mechanical_port/source/draw_target.rs` | `504ae78a001fc4d3bf9eed007ae7e114e41374174b2deaa34a6dc9f2d503dd9b` |
| `crates/nuxie-runtime/src/mechanical_port/source/foreground_layout_drawable.rs` | `e5163120dee92a7d708bed0cf6b5cc1185a05a9c6e9c616c5ddb04568defb4c4` |
