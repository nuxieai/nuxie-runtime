# Data Binding Graph Owned ViewModel Nested Enum Name Path Runtime Contract

## Purpose

Admit generated owned nested enum name-path mutation as the next sibling of
the nested number, boolean, string, and color slices.

C++ can create an owned `ViewModelInstance`, look up a nested generated enum
with `ViewModelInstanceRuntime::propertyEnum("child/choice")`, mutate its
value index, and bind that owned instance to a state machine. For this Rust
slice, `RuntimeOwnedViewModelInstance` can store direct enum value indexes on
generated owned view-model children and resolve a slash-separated enum
property path through generated view-model pointers.

## In Scope

- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Nested enum paths whose last segment is a direct enum property `name` on the
  generated child.
- `RuntimeOwnedViewModelInstance::set_enum_by_property_name_path`.
- Source-to-target graph binding for enum data-bind paths longer than the root
  scalar shape, such as `[rootViewModel, childProperty, choiceProperty]`.
- A C++ probe flag that calls
  `ViewModelInstanceRuntime::propertyEnum("child/choice")`.

## Out Of Scope

- Nested symbol-list-index, asset, artboard, trigger, list, or view-model
  scalar value mutation.
- Imported-intermediate nested scalar reads or writes.
- Shared imported-instance mutation, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- A generated owned nested enum can be mutated by name path before binding.
- The matching C++ public `propertyEnum(namePath)` probe reports the same
  state-machine transition behavior after state-machine advance.
- Existing owned root scalar and nested number/boolean/string/color probes
  continue to pass.
