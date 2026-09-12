# Private sizing scalar binding

The new `src/wrapping_scalar.rs` reader checks the actual sizing record segment against a closed scalar instruction grammar. It never calls the emitter or encodes an expected record vector. Each instruction checks its object type, exact parent/target operands, explicit values, and absence of every other property. It reconstructs the measurement/boundary/carry/final-position instruction relationships and checks all observation handles in `wrapping::Trace`. The reader consumes the full sizing segment, including the ordering of the copy and DistanceConstraint on each normalized node.

`compose_with_bounds` passes the same bound slot/visible roles, physical fractions, derived epsilon and owned immutable candidate to the reader and position analysis. The `Derived` result retains both analyses. This is private validation, not public admission.

## Native meaning of the checked instructions

The fixed consumer's `source/generated/node_base.rs` initializes x/y to0, and `source/generated/transform_component_base.rs` initializes rotation0 and scales1. Their absence from scalar nodes is intentional. The reader permits only parentId and the explicitly expected active-coordinate constant on a Node. Orthogonal constants, transforms and all other fields reject.

`source/generated/constraints/constraint_base.rs` initializes strength1. `transform_space_constraint_base.rs` initializes source/destination spaces0 (world), `transform_component_constraint_base.rs` initializes copy factor1, offsetfalse, min/maxfalse, min/max space0, and `translation_constraint_base.rs` supplies the corresponding Y defaults. `transform_constraint_base.rs` initializes both origins0. The reader rejects unmodelled fields rather than trusting a missing-field fallback invented by the compiler.

For the unchanged `source/constraints/translation_constraint.rs`, destination-local copy composes the copied translation with the parent's world transform; this is the sum instruction. World copy with local min0 first subtracts the parent translation, clamps the difference, then re-adds the parent; this is the rounded max instruction modelled by the carry analysis. It must not be equated with exact f32::max. Ordinary negative scale followed by destination-local copy implements subtraction. Root min0 implements the dead zone. The final copy uses world coordinates and strength1.

For `source/constraints/distance_constraint.rs`, exact mode2/distance65536/zero-origin target together with the preceding scalar copy match the conditional normalizer's shape. The following root copy has factor2 and active maximum65536. Mode, distance, target and both constraint positions are explicitly bound.

The transform measurement instruction always has a direct checked slot target and only the active origin fraction. Its following scalar projection removes the other translation coordinate. Source/destination spaces and strength remain their audited defaults.

## Scope and outstanding work

This is a structural association with the existing operation ledger, not independent evidence that the ledger's arithmetic theorem is correct. It closes sizing field/operand/trace drift, including extra-field drift. It does not bind paint graph semantics, establish runtime dependency evaluation on resize/cloning, fix differing visible/slot extents, or establish clipping/rendering behavior. Scheduling is audited separately. No public route, mask geometry, output bytes or runtime changes are implied by this checker.

Mutation controls change identity transforms, orthogonal coordinate, strength, source space, offset, operands, orthogonal origin, distance/mode/origin target, constraint order and trace outputs. Valid compositions cover both axes, direction reversals, positional fractions and variable item counts through the existing composition matrix. These checks supplement the native/Chrome campaign required before admission; they do not replace it.
