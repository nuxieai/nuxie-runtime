# Data Binding Graph Owned ViewModel Imported-Intermediate Enum Name-Path Unsupported Runtime Contract

## Purpose

Pin the public enum name-path mutation boundary for owned view-model contexts
after a generated child has been replaced with an imported child instance.

C++ can bind through the imported child's existing
`ViewModelInstanceEnum.propertyValue` index, but an attempted public
`ViewModelInstanceRuntime::propertyEnum("child/choice")->valueIndex(...)`
write after replacing `child` with an imported instance leaves that imported
enum unchanged. Rust keeps the same boundary:
`RuntimeOwnedViewModelInstance::set_enum_by_property_name_path` returns
`false` once the path crosses an imported intermediate.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested enum source paths such as `child/choice`.
- The attempted public C++ mutation shape
  `ViewModelInstanceRuntime::propertyEnum("child/choice")->valueIndex(value)`.
- Verifying that both C++ and Rust preserve the imported child's existing enum
  source value after binding and state-machine advancement.

## Out Of Scope

- Supporting mutation through imported intermediates.
- Symbol-list-index, asset, artboard, trigger, list, and view-model pointer
  name-path mutation boundaries.
- Stable public object handles, reverse propagation, broader update queues,
  relative/parent/name-based lookup, listener-owned data binding, and nested
  artboard propagation.

## Completion Checks

- The C++ probe replaces `child` with an imported child, attempts to write
  `child/choice`, binds the owned context, and still behaves as if the imported
  child's original enum index is selected.
- Rust rejects the same mutation through
  `set_enum_by_property_name_path("child/choice", value)` after `child` is
  imported.
- The state-machine advance reports stay equal between C++ and Rust.
