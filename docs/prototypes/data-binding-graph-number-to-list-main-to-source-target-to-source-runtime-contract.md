# Data Binding Graph NumberToList Main-To-Source Target-To-Source Runtime Contract

## Purpose

Pin C++ explicit data-context behavior for a main-`ToSource | TwoWay`
`DataConverterNumberToList` bind targeting
`BindablePropertyList.propertyValue`.

`DataConverterNumberToList` creates generated list values during forward
conversion, but it does not provide a generated-list reverse path for this
state-machine bindable target. C++ consumes the edited bindable-list target
during explicit `advancedDataContext()` without writing that scalar value back
to the numeric source.

## In Scope

- State-machine-owned `DataBindContext` targeting
  `BindablePropertyList.propertyValue`.
- Default root view-model context bound with `bind_default_view_model_context`.
- A default root `ViewModelInstanceNumber.propertyValue` source.
- Direct `DataConverterNumberToList` with a file-backed `viewModelId`.
- Main-`ToSource | TwoWay` flags.
- Mutating the bindable list target scalar by data-bind index.
- Explicit `StateMachineInstance::advance_data_context`, matching C++
  `advancedDataContext()`.
- C++ probe coverage for unchanged source number value and preserved target
  scalar value after the explicit data-context pass.

## Out Of Scope

- Public `updateDataBinds(true)` for this converter, already covered by
  `docs/prototypes/data-binding-graph-bindable-list-target-to-source-runtime-contract.md`.
- Writing edited target values into `ViewModelInstanceNumber` sources through
  generated-list reverse conversion.
- Generated list item identities, cloned item instances, and list item
  view-model traversal.
- `ArtboardComponentList` item instancing, map-rule selection, layout,
  virtualization, scrolling, or rendering.
- Owned view-model list storage and owned-context list binding.
- Relative, parent, nested, listener-owned, and nested-artboard data binding.

## Completion Checks

- A main-`ToSource | TwoWay` `DataConverterNumberToList` bind preserves the
  edited bindable-list target scalar after explicit data-context advancement.
- The default number source report remains unchanged after the explicit
  target-to-source pass.
- Existing direct list target-to-source and public
  `DataConverterNumberToList` target-to-source probes continue to pass.
