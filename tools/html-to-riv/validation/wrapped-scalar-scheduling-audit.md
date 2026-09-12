# Immutable sizing graph scheduling audit

This is a source audit of the private `wrapping.rs` sizing graph and its closed
`wrapping_scalar.rs` record grammar, against the pinned runtime baseline
`6c7ac16617835b5f581784ff08a9e779bb52faf3`. It adds no runtime changes, native
capture or public qualification. Line references below are repository-relative;
they refer to the inspected source, not an upstream implementation assumed to
behave identically. Paint gates and paint ordering require their own audit.

## Dependency order is explicit

All three concrete constraint implementations dispatch dependency construction
to `TargetedConstraint::build_dependencies`: Distance at
`crates/nuxie-runtime/src/mechanical_port/source/generated/core_registry.rs:53715`,
Translation at 53962, and Transform at 54214. The actual implementation in
`constraints/targeted_constraint.rs:139` adds the constrained **parent transform**
as a dependent of the target. A constraint record is therefore not an arithmetic
instruction that must itself appear in dependency order. Its owner's update
executes it. Do not infer an owner-to-constraint dependency from the unused base
`Constraint::build_dependencies` implementation for these concrete types.

`TransformComponent::build_dependencies` (`transform_component.rs:102`) adds
parent-to-child dependencies; Node dispatch is in `generated/core_registry.rs:54406`.
LayoutComponent does the same at `layout_component.rs:1187`. Initialization calls
concrete dependency construction for every object at `artboard.rs:913`, then sorts
at 1032. `dependency_sorter.rs:46` visits dependents recursively and prepends each
source, producing source-before-dependent order in an acyclic graph. The sorter
reports a cycle and can return a partial order; object IDs alone do not establish
correct scheduling or successful cycle handling.

For the checked sizing construction, each new helper consumes only older helper
owners or independent base slots. Parent operands and target operands both create
edges. All helpers descend from the artboard, so they are reachable from the root
sort. The final helper target creates an edge to the earlier-ID visible child
(`wrapping.rs:120`). No sizing helper targets that visible child. Combined with
the closed base's independent slots and the composition's layout-isolation
restriction, this establishes an acyclic sizing graph despite the final target
having a later ID than the visible object. It does not establish acyclicity for
arbitrary uninspected additions to the file.

## Two constraints on the normalized owner

`wrapping.rs:203` first creates a Node with a TranslationConstraint copying the
dead-zone scalar; line 204 then appends its DistanceConstraint. The scalar grammar
independently requires this sequence (`wrapping_scalar.rs`, `Reader::boundary`).
`Artboard::initialize_handle` invokes `on_added_dirty` in object-vector order
(`artboard.rs:778`). `Constraint::on_added_dirty` registers on the parent
(`constraints/constraint.rs:68`), and `TransformComponent::add_constraint` pushes
onto its vector (`transform_component.rs:242`). Target resolution occurs during
this lifecycle stage (`constraints/targeted_constraint.rs:121`). Thus successful
initialization gives the normalized owner the ordered vector [copy, distance].

The actual update dispatcher composes the owner's world transform before calling
constraints (`generated/core_registry.rs:2127–2160`).
`component_update_constraints_handle` snapshots the vector at 2345–2350 and
iterates it in order at 2386–2393. Each application is synchronous and ends before
the next. The distance constraint therefore reads the current copy result; the
gate's target dependency makes the gate run after both. This is a registration
and execution argument, not a tie-breaking assumption about the sorter.

## Visible LayoutComponent applies its constraint twice

A LayoutComponent first goes through the transform-super constraint pass. It then
runs `update_after_transform_super` (`layout_component.rs:1240–1256`), which on
WORLD_TRANSFORM recomposes from solved layout and requests another constraint pass
(`generated/core_registry.rs:2192–2208`). The final TranslationConstraint is applied
again after that recomposition. The arithmetic argument must establish finite
initial transforms and correct final-copy semantics for this second pass; it
cannot assume a single invocation or that the first constrained transform is the
second pass's input. Both passes read the same already-updated target, and neither
changes the independent slot's solved size. Closed base inspection excludes other
layout constraints, animations and transforms that could alter this conclusion.

## Resizing and cloning

Width/height setters dirty layout through callbacks
(`layout_component.rs:3155–3177`). The normal update pass synchronizes layout
before its component walk (`artboard.rs:2071–2100`); style synchronization computes
layout and publishes bounds when the artboard updates its own layout
(`artboard.rs:2024–2034`). Changed solved bounds update retained layout and mark
world transforms dirty (`layout_component.rs:2395–2405`). Recursive dirt propagation
follows dependents (`component.rs:145–210`); duplicate dirt is suppressed at
700–705. Thus a changed measured slot dirties the scalar consumers as well as its
ordinary descendants. Unchanged measured inputs may retain their previous scalar
results, which is semantically appropriate for this nonanimated graph.

`Artboard::update_components_handle` walks sorted owners, clears dirt immediately
before updating, and restarts if an earlier owner becomes dirty
(`artboard.rs:1833–1893`). There is a 100-pass cap. Acyclic scalar reads/writes do
not themselves require a fixed-point iteration, but this audit does not convert
the existence of that cap into a general convergence guarantee for all layouts.

Cloning preserves object-vector positions by cloning occurrences in source order
(`artboard.rs:3864–3869`) and calls the same `initialize_handle` at 3879. Target IDs
are resolved in the clone's context and constraint vectors/dependencies are rebuilt
through initialization. Consequently the source argument applies to successfully
initialized clones, not only the imported definition. Historical captures are
useful corroboration, but do not qualify a newly derived-epsilon candidate.

## Proven scope and remaining validation

The inspected closed sizing graph has explicit operand dependencies, ordered
copy-then-distance execution, and a final visible-output dependency. Ordinary
initialization and clone initialization install those relationships. Normal
resize/update has a source path that republishes measured bounds and propagates
dirt through the relationships. These facts discharge a scheduling premise for
the sizing arithmetic under the checked closed-file and successful-lifecycle
conditions.

Still required before public admission:

- Bind this audit to the exact frozen compiler records, immutable source hashes,
  compiled consumer and actual ordinary imported object types. The structural
  grammar does not by itself verify deserialization/lifecycle dispatch in a run.
- Exercise the actual derived-epsilon candidate across resize transitions on the
  same original and clone, checking first settled frame, repeated unchanged frames,
  scalar intermediates, visible geometry and Chrome/native pixels. Preserve cycle,
  import, convergence and stale-frame failures rather than hiding them with extra
  arbitrary update calls.
- Finish numeric final-position and mask coverage proofs and separately inspect
  paint dependencies. This audit does not prove CSS equivalence, exact gates,
  normalizer arithmetic, clip coverage or renderer equality.
- Keep admission restricted to the checked standalone artboard lifecycle; arbitrary
  nested hosting, animation, external transform mutation or uninspected added
  constraints are outside this argument.
