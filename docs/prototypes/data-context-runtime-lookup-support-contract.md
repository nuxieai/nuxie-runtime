# Data Context Runtime Lookup Support Contract

## Scope

This slice extracts the file-backed `DataContext` lookup behavior into a
graph-callable runtime API without admitting live data-bind scheduling.

`RuntimeDataContext` is a borrowed runtime view over:

- the imported `RuntimeFile`
- one current `ViewModelInstance`
- zero or more parent `ViewModelInstance` contexts

It delegates to the existing C++-audited `rive-binary` lookup helpers for
absolute property/instance lookup, manifest-relative property/instance lookup,
and `ViewModelInstance::propertyFromPath`.

## C++ Parity Points

- Absolute root instance and root property lookup preserve the same
  `viewModelId`/`viewModelPropertyId` path semantics as the
  `--data-context-lookups` probe.
- Manifest-relative and parent-fallback lookups preserve unresolved `None`
  results where C++ does not materialize a value in the current synthetic
  fixture.
- The existing `runtime_data_context_lookup_reports` path now uses
  `RuntimeDataContext`, and the full C++ probe lookup-report comparison still
  passes.

## Out Of Scope

- Live `DataBindContext` binding through relative/name paths.
- Converter source-path binding.
- Runtime mutation or relinking through `RuntimeDataContext`.
- Listener-owned data contexts, nested-artboard-propagated contexts, and dirty
  update queues.

## Tests

- `data_context_file_backed_lookup_reports_match_cpp_probe`
- `runtime_data_context_covers_absolute_and_unresolved_relative_parent_paths`
