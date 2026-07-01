# Data Context File-Backed Lookup Runtime Contract

## Purpose

This slice closes the first file-backed `DataContext` lookup gap from the
runtime map without admitting live data-binding scheduling.

The supported contract is a read-only Rust report of the import-time lookup
facts C++ exposes through:

- `DataContext::getViewModelInstance`
- `DataContext::getViewModelProperty`
- `DataContext::getRelativeViewModelInstance`
- `DataContext::getRelativeViewModelProperty`
- `ViewModelInstance::propertyFromPath`
- `DataContext` parent fallback for absolute and manifest-relative property
  lookup

The report is built from imported `ViewModel`, `ViewModelInstance`, explicit
`ViewModelInstanceValue` children, and `ManifestAsset` name IDs.

## In Scope

- Enumerate imported view models, instances, and explicit instance values in
  C++ file order.
- Emit absolute lookup reports rooted at each imported instance, using
  `viewModelId` followed by `viewModelPropertyId` path segments.
- Emit relative lookup reports rooted at each imported instance, using
  `ManifestAsset` name IDs matched by view-model property name.
- Recurse through explicit `ViewModelInstanceViewModel` references up to the
  same depth guard as the C++ probe.
- Report the resolved value/instance by view-model index, instance index,
  value index, core type, property id, and imported name.
- Emit `propertyFromPath` reports for property-id paths relative to each root
  instance.
- Emit the same first available parent fallback absolute and manifest-relative
  property reports as the C++ probe.
- Compare the full report with the C++ probe `--data-context-lookups` output
  for a file-backed nested view-model fixture.

## Out Of Scope

- Creating missing default property values during import.
- Live `DataBindContext::resolvePath` scheduling, dirty queues, observer
  propagation, converter advancement, or target mutation.
- Listener-owned data contexts and nested-artboard propagated contexts.
- Runtime mutation or relinking through these lookup reports.

## Completion

This slice is complete when Rust exposes the read-only data-context lookup
report, focused C++ probe parity covers the complete `--data-context-lookups`
array for a nested view-model fixture, full workspace verification passes, and
the runtime audit marks this file-backed lookup report as closed while leaving
live data-binding behavior explicitly open.
