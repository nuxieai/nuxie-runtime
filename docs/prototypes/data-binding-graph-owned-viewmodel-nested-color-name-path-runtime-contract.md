# Data Binding Graph Owned ViewModel Nested Color Name Path Runtime Contract

## Purpose

Admit generated owned nested color name-path mutation as the next sibling of
the nested number, boolean, and string slices.

C++ can create an owned `ViewModelInstance`, look up a nested generated color
with `ViewModelInstanceRuntime::propertyColor("child/tint")`, mutate it, and
bind that owned instance to a state machine. For this Rust slice,
`RuntimeOwnedViewModelInstance` can store direct color values on generated
owned view-model children and resolve a slash-separated color property path
through generated view-model pointers.

## In Scope

- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Nested color paths whose last segment is a direct
  `ViewModelPropertyColor.name` on the generated child.
- `RuntimeOwnedViewModelInstance::set_color_by_property_name_path`.
- Source-to-target graph binding for color data-bind paths longer than the
  root scalar shape, such as `[rootViewModel, childProperty, tintProperty]`.
- A C++ probe flag that calls
  `ViewModelInstanceRuntime::propertyColor("child/tint")`.

## Out Of Scope

- Nested enum, symbol-list-index, asset, artboard, trigger, list, or
  view-model scalar value mutation.
- Imported-intermediate nested scalar reads or writes.
- Shared imported-instance mutation, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- A generated owned nested color can be mutated by name path before binding.
- The matching C++ public `propertyColor(namePath)` probe reports the same
  state-machine transition behavior after state-machine advance.
- Existing owned root scalar and nested number/boolean/string probes continue
  to pass.
