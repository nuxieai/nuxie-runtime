# Data Binding Graph Owned ViewModel Nested Artboard Name Path Runtime Contract

## Purpose

Admit generated owned nested artboard name-path mutation as the next sibling
after the nested asset path slice.

C++ can create an owned `ViewModelInstance`, resolve a generated child with
`ViewModelInstanceRuntime::propertyViewModel("child")`, look up the child's
`scene` value with `ViewModelInstance::propertyValue("scene")`, mutate its raw
`propertyValue`, and bind that owned instance to a state machine. For this Rust
slice, `RuntimeOwnedViewModelInstance` can store direct artboard values on
generated owned view-model children and resolve a slash-separated artboard
property path through generated view-model pointers.

## In Scope

- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Nested artboard paths whose last segment is a direct artboard property
  `name` on the generated child.
- `RuntimeOwnedViewModelInstance::set_artboard_by_property_name_path`.
- Source-to-target graph binding for artboard data-bind paths longer than the
  root scalar shape.
- A C++ probe flag that resolves the parent path with `propertyViewModel`,
  then calls `propertyValue("scene")` and mutates the resulting
  `ViewModelInstanceArtboard`.

## Out Of Scope

- Nested trigger, list, or view-model value mutation.
- Imported-intermediate nested scalar reads or writes.
- Artboard referencer remapping, nested artboard propagation, render-side
  nested artboard behavior, shared imported-instance mutation, stable public
  object handles, reverse propagation, broader update queues,
  relative/parent/name-based lookup, and listener-owned data binding.

## Completion Checks

- A generated owned nested artboard can be mutated by name path before binding.
- The matching C++ probe reports the same state-machine transition behavior
  after resolving the parent path, mutating the child artboard value, and
  advancing the state machine.
- Existing owned root scalar and nested
  number/boolean/string/color/enum/symbol-list-index/asset probes continue to
  pass.
