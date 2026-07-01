# Data Binding Graph Owned ViewModel Imported-Intermediate String Name-Path Unsupported Runtime Contract

## Purpose

Pin the public string name-path mutation boundary for owned view-model
contexts after a generated child has been replaced with an imported child
instance.

C++ can bind through the imported child's existing
`ViewModelInstanceString.propertyValue`, but an attempted public
`ViewModelInstanceRuntime::propertyString("child/label")->value(...)` write
after replacing `child` with an imported instance leaves that imported string
unchanged. Rust keeps the same boundary:
`RuntimeOwnedViewModelInstance::set_string_by_property_name_path` returns
`false` once the path crosses an imported intermediate.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested string source paths such as `child/label`.
- The attempted public C++ mutation shape
  `ViewModelInstanceRuntime::propertyString("child/label")->value(value)`.
- Verifying that both C++ and Rust preserve the imported child's existing
  string source value after binding and state-machine advancement.

## Out Of Scope

- Supporting mutation through imported intermediates.
- Color, enum, symbol-list-index, asset, artboard, trigger, list, and
  view-model pointer name-path mutation boundaries.
- Stable public object handles, reverse propagation, broader update queues,
  relative/parent/name-based lookup, listener-owned data binding, and nested
  artboard propagation.

## Completion Checks

- The C++ probe replaces `child` with an imported child, attempts to write
  `child/label`, binds the owned context, and still behaves as if the imported
  child's original string value is selected.
- Rust rejects the same mutation through
  `set_string_by_property_name_path("child/label", value)` after `child` is
  imported.
- The state-machine advance reports stay equal between C++ and Rust.
