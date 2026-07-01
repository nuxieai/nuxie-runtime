# Data Binding Graph Owned ViewModel Imported-Intermediate Symbol-List-Index Runtime Contract

## Purpose

Admit the symbol-list-index sibling of the imported-intermediate scalar source
path for owned view-model contexts.

C++ can create an owned root `ViewModelInstance`, replace a generated
`ViewModelPropertyViewModel` child with an imported child instance, bind that
owned root to a state machine, and then resolve a source path such as
`child/symbol` from the imported child's existing
`ViewModelInstanceSymbolListIndex.propertyValue`. For this Rust slice,
`RuntimeOwnedViewModelInstance` records imported symbol-list-index snapshots
for referenced view-model instances so graph binding can read the same source
value.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested symbol-list-index source paths whose final segment is a
  `ViewModelPropertySymbolListIndex` on the imported child.
- Source-to-target graph binding for
  `RuntimeDataBindGraphValue::SymbolListIndex`.
- A C++ probe comparison that observes the bound symbol-list-index through an
  existing `DataConverterToString` and `TransitionViewModelCondition`.

## Out Of Scope

- Mutating through imported intermediates.
- Number, boolean, string, color, and enum reads already admitted; asset,
  artboard, trigger, list, and view-model pointer source reads beyond the
  slices already admitted.
- New converter semantics beyond using the existing symbol-list-index-to-string
  converter path as the observation point.
- Imported-instance mutation sharing, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Replacing an owned generated child with an imported child lets a nested
  symbol-list-index data-bind source read the imported child's existing value.
- A matching C++ probe and Rust report take the same state-machine transition
  when that imported symbol-list-index source is converted to the expected
  string.
- Existing owned generated nested symbol-list-index and imported-intermediate
  number, boolean, string, color, and enum probes continue to pass.
