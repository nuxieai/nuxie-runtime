# Generated wrapping helpers preserve the native layout tree

This source audit supports the private `wrapping_composition.rs` preservation check. It is not a claim about line classification, constraint numerical accuracy, paint equality or clipping coverage.

The completed base scene comes from `wrapping_slots::inspect`, with authored dimensions joined by `wrapping_domains::resolve`. `Domains` exposes read-only facts. Composition clones those records and runs the existing sizing and paint emitters; the resulting `Candidate` owns records and checked inputs without a mutable accessor. The independent prefix check permits only the planned original SolidColor values to become transparent. Layout records, styles, units, sizes, topology and all other stored properties remain byte-exact.

The appended-kind allowlist excludes LayoutParticipant, LayoutComponent/style/appliers, LayoutConstraint, nested artboards/lists, Text and Image. Typed parent checks permit only scalar Nodes at the artboard, scalar/helper child hierarchies, and ForegroundLayoutDrawable/TranslationConstraint directly under existing visible owners. Nothing appends under measured slots or their ancestors except root scalar Nodes. TransformConstraint measurements target only the measured slots. Scalar constraints read earlier helper Nodes; their parent chains terminate at the artboard. Paint references identify the expected generated type. This blocks a feedback path through transformed visible owners and rejects dangling IDs. It does not prove the numerical effect of each constraint field.

## Immutable source chain

- `layout/layout_node_provider.rs:163–175` recognizes LayoutComponent, nested artboards/lists, and Text/Image/Shape only when they have an attached provider.
- `layout_component.rs:386–421` recursively visits transparent Node/Solo hierarchies when collecting providers. Other appended kinds are ignored. The collected providers exclusively determine native child installation at631–666.
- `shapes/shape.rs:648–670` requires a LayoutParticipant child to participate. Excluding that kind everywhere in the appended graph is essential; admitting Shape alone would not justify blanket nonparticipation if arbitrary descendants were allowed.
- `foreground_layout_drawable.rs:27–48` registers foreground paint; `layout_component.rs:1177–1179` only sets HasForegroundDrawable. That flag selects rendering paths, not layout sizes.
- `constraints/constraint.rs:68–88` registers constraints on their parent transform. `constraints/translation_constraint.rs:134–142` writes world translation, not the solved layout dimensions or style. Existing owners receive only those translation constraints; other constraint parents are generated Nodes.
- `layout_component.rs:2354–2415` publishes dimensions from solved layout. Paint opacity, clipping sources and draw rules do not supply dimensions.

Together with the definite, zero-inset base slot invariant, these conditions preserve the native size/topology inputs through the generated graph. Constraint evaluation order, numeric error, ordinary paint replication fidelity and mask coverage remain separate obligations. Root inspected the provider traversal and independently reviewed the typed parent/reference checks; a parallel source audit supplied the remaining source chain.

## Source bindings

| Source | SHA-256 |
| --- | --- |
| `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_node_provider.rs` | `1d3e595d60d91e5b110d89865b17db1410814897674f1d8498ee46cfad6f1170` |
| `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs` | `5154d46f471e2497886b2dbd420d5a0532baf8521f6cfe655a432c60d05122c3` |
| `crates/nuxie-runtime/src/mechanical_port/source/shapes/shape.rs` | `a3ae184d6a877dd9df4e60ce445d1067135cae7a84b35d73302133bcb8e772eb` |
| `crates/nuxie-runtime/src/mechanical_port/source/foreground_layout_drawable.rs` | `e5163120dee92a7d708bed0cf6b5cc1185a05a9c6e9c616c5ddb04568defb4c4` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/constraint.rs` | `d89f7c20f18c2f3ca437129d3098a6f98b11326bd0ad121aa932570bd64f364e` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/translation_constraint.rs` | `b7d9eb0c64ebb1da20fbf37a296e16c41e8ed42b437d59c9f849e39b8d5eac14` |
