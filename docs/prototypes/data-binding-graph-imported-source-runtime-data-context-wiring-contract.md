# Imported Source RuntimeDataContext Wiring Contract

## Scope

This slice routes already-supported imported state-machine data-bind source
resolution through `RuntimeDataContext`.

The covered behavior is limited to absolute `viewModelId` /
`viewModelPropertyId` source paths that were already admitted by
`RuntimeDataBindGraph::bind_imported_view_model_context`.

## C++ Parity Points

- Binding an imported view-model instance still resolves number, boolean,
  string, color, enum, symbol-list-index, asset, artboard, trigger, list, and
  view-model pointer sources with the same absolute-path behavior as before.
- Imported-context override maps are still applied after source lookup.
- `DataConverterOperationViewModel` operand refresh continues to use the same
  `RuntimeDataContext` for imported contexts.

## Out Of Scope

- New relative/name/parent path admission.
- Listener-owned data contexts and nested-artboard-propagated contexts.
- Runtime mutation or relinking through `RuntimeDataContext`.
- Dirty-list scheduling, update queues, and reverse propagation.

## Tests

- `cargo test --workspace --quiet`
- `RIVE_CPP_PROBE=... cargo test -p rive-runtime --test cpp_probe --quiet`
