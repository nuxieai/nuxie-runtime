# Graph Runtime Contract

Date: 2026-06-28

This document starts the next milestone after binary import parity. It defines
the `rive-graph` seam so post-import graph work can move forward without pulling
new runtime behavior back into `rive-binary`.

## Formal Goal

Define and implement the next runtime seam around `rive-graph`: consume the
verified `rive-binary` imported model and build the post-import artboard graph
lifecycle through a finite, testable graph projection.

The goal is complete when `rive-graph` can project the C++ post-import artboard
graph facts needed by the next runtime slices, while `rive-binary` remains frozen
as the file-loading module except for import-parity bugs or upstream C++ import
drift.

## Scope Lock

`rive-graph` owns graph topology. It does not own frame execution.

The external seam for `rive-graph` is:

- Input: a verified `rive_binary::RuntimeFile`.
- Output: immutable or explicitly snapshot-style graph projections for file-owned
  collections, artboard-local object slots, component hierarchy, dependency edges,
  dependency order/cycles, and immediate post-import graph relationships.
- Errors: structural graph/projection diagnostics that can be derived from the
  imported model without mutating live runtime state.

Everything past that seam belongs to later runtime crates or later `rive-graph`
milestones with their own contracts.

## Owned By `rive-graph`

`rive-graph` owns:

- Artboard-local object slot projection from `rive-binary`.
- Component indexes and stable local/global ID maps.
- Parent/child hierarchy and missing-parent diagnostics.
- Capability classification needed by graph algorithms.
- Dependency edges and dependency order over imported graph relationships.
- Dependency cycle diagnostics.
- Immediate graph relationships established by C++ import, `onAddedDirty`,
  `onAddedClean`, or `buildDependencies`, when those relationships can be
  represented as static graph facts.
- File-level projection convenience over `rive-binary` collections when graph
  consumers need those collections alongside artboards.

Graph relationships should stay ID-based. Do not introduce `Rc<RefCell<dyn Core>>`
or pointer-style ownership to mimic the C++ runtime object graph.

## Not Owned By `rive-graph` Yet

These are future runtime slices, not blockers for the current graph seam:

- Binary file decoding or import-stack ownership rules.
- Mutable artboard instances and cloning.
- Dirt propagation and frame scheduling.
- Local/world transform mutation.
- Constraint solving.
- Layout solving.
- Animation advancement.
- State-machine execution.
- Live data-binding source/target mutation and scheduling.
- Rendering or draw-command emission.
- Text shaping/layout.
- Audio playback.
- Scripting execution.

Some of these may later live in `rive-graph` if the module grows deliberately, but
they need explicit admission through a new contract or a contract amendment.

## Admission Rule For New Graph Work

Before adding a new `rive-graph` relationship, edge family, lifecycle projection,
or C++ probe field, answer:

1. Is this derivable from the already imported `RuntimeFile` without executing a
   frame?
2. Does C++ establish the relationship during import, `onAddedDirty`,
   `onAddedClean`, validation, or `buildDependencies`?
3. Is the result a static graph fact, dependency edge, graph ordering fact, or
   structural diagnostic?
4. Is it needed by the next runtime slice as graph input rather than as live
   behavior?

If the answer is no to all four, do not add it to `rive-graph`.

If the answer requires new file-loading data, first check
[`binary-import-completion-contract.md`](binary-import-completion-contract.md).
Only add to `rive-binary` when the binary contract admits it.

## First Finite Slice

The first slice after this contract is ticket `#7`: complete the parent/child and
dependency graph surface.

The slice should extend the existing edge list toward the audited C++
`buildDependencies()` surface, especially:

- Path composer projections and dependencies.
- Follow-path dependencies.
- Text graph dependencies.
- Layout dependencies.
- Data-binding graph dependencies that are static graph facts.
- Remaining scroll/layout dependencies beyond the covered
  `ScrollConstraint -> ScrollBarConstraint` and
  `ScrollConstraint -> layout-provider content child` edges.
- Paint/effect graph dependencies.

Each edge family should have:

- A short C++ source audit or probe comparison.
- Focused synthetic coverage when practical.
- Corpus comparison coverage when the C++ probe can expose it.
- A classification against the admission rule above.

## Current Starting Point

The existing `rive-graph` prototype already covers:

- `GraphFile::from_runtime_file`.
- C++-style artboard-local object slots.
- File-level asset, view-model, data-enum, animation, and state-machine projection.
- Component parent resolution and child indexing.
- Capability flags for artboard/container/world-transform/transform/drawable.
- Draw target, draw rules, and clipping source relationships.
- Clipping source/clipped drawable projections.
- Synthetic path composer projections for each imported `Shape`, with path inputs
  sourced from `rive-binary`'s C++-equivalent shape registration facts.
- Dependency nodes for real imported components plus synthetic path composers,
  with a topological node order and a filtered real-component local-ID order.
- Dependency edges for parent-child, targeted constraints, IK constraints,
  draw-target drawable references, draw-rule target references, clipping sources,
  skinning for exact C++ skinnables (`Mesh` and `PointsPath`), Joystick
  custom-handle dependencies, path-composer shape/path prerequisites,
  clipping-shape-to-source-path-composer prerequisites, and the static
  `ScrollConstraint -> ScrollBarConstraint` and
  `ScrollConstraint -> layout-provider content child` dependencies.
- Topological dependency order and dependency-cycle diagnostics.
- C++ probe comparison through `make cpp-compare`.

## Completion Checklist

This graph-seam milestone can be marked complete when:

- The public `rive-graph` surface is documented as graph projection, not live
  runtime execution.
- The dependency edge families included in the milestone are listed and each one
  has evidence against C++ source/probe behavior.
- `rive-binary` has no new post-import runtime helper expansion for this work.
- Focused Rust tests pass for graph projection and dependency ordering.
- The C++ graph comparison passes for the supported fixture set.
- Any runtime behavior discovered during graph work is recorded as out of scope or
  moved to a later runtime plan.

Suggested verification:

```sh
make test
make cpp-compare
```

If `make cpp-compare` fails in `rive-binary`, triage it through the binary import
contract before changing `rive-binary`. If it fails in `rive-graph`, classify the
failure through this contract before implementing more graph surface.
