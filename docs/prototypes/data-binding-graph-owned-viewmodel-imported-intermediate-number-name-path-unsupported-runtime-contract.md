# Data Binding Graph Owned ViewModel Imported-Intermediate Number Name-Path Unsupported Runtime Contract

## Purpose

Pin the public name-path mutation boundary for owned view-model contexts after
a generated child has been replaced with an imported child instance.

C++ can bind through the imported child's existing
`ViewModelInstanceNumber.propertyValue`, but an attempted public
`ViewModelInstanceRuntime::propertyNumber("child/amount")->value(...)` write
after replacing `child` with an imported instance leaves that imported number
unchanged. Rust keeps the same boundary: imported-intermediate number snapshots
are read-only for this context shape, and
`RuntimeOwnedViewModelInstance::set_number_by_property_name_path` returns
`false` once the path crosses an imported intermediate.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested number source paths such as `child/amount`.
- The attempted public C++ mutation shape
  `ViewModelInstanceRuntime::propertyNumber("child/amount")->value(value)`.
- Verifying that both C++ and Rust preserve the imported child's existing
  number source value after binding and advancing the data context.

## Out Of Scope

- Supporting mutation through imported intermediates.
- Boolean, string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, and view-model pointer name-path mutation boundaries.
- Stable public object handles, reverse propagation, broader update queues,
  relative/parent/name-based lookup, listener-owned data binding, and nested
  artboard propagation.

## Completion Checks

- The C++ probe replaces `child` with an imported child, attempts to write
  `child/amount`, binds the owned context, and still reports the imported
  child's original number value.
- Rust rejects the same mutation through
  `set_number_by_property_name_path("child/amount", value)` after `child` is
  imported.
- The state-machine number binding reports stay equal between C++ and Rust.
