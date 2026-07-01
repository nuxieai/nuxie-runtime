# Data Binding Graph Name-Based State-Machine Source-Path Unsupported Runtime Contract

## Purpose

Pin the C++ runtime boundary for state-machine-owned `DataBindContext` records
with the `NameBased` flag. Imported state-machine data binds are cloned with a
null `DataBind::file()` pointer, so `DataBindContext::bindFromContext()` cannot
use `file()->dataResolver()` to expand manifest-backed source-path ids. For
this shape, C++ leaves the data bind source unresolved even when the file has a
manifest path that would otherwise resolve.

This slice prevents `rive-runtime` from using the import-time manifest
resolution helper for cloned state-machine binds that C++ would not resolve at
runtime.

## In Scope

- State-machine-owned `DataBindContext` records targeting
  `BindablePropertyNumber.propertyValue`.
- Default root view-model contexts already supported by
  `RuntimeDataBindGraph`.
- C++ `DataBindFlags::NameBased` (`1 << 4`) where the raw
  `sourcePathIds` buffer is `[77]`.
- A manifest asset that maps path id `77` to name id `5`, and name id `5` to
  `amount`.
- A direct `ViewModelInstanceNumber.propertyValue` source named `amount` that
  would resolve if the cloned data bind had a resolver-backed file pointer.
- C++ probe coverage showing the source remains unresolved and the cloned
  bindable number target keeps its initial value.

## Out Of Scope

- File-backed name-based data binds whose live `DataBind::file()` is non-null.
- Artboard-owned, listener-owned, nested-artboard, and component-list data
  binding.
- Public API lookup by manifest name or string path.
- Parent-chain fallback, relative view-model traversal, and nested source
  paths for live runtime data contexts.
- Implementing name-based source resolution beyond the C++ state-machine
  runtime behavior pinned here.

## Completion Checks

- Runtime source construction for default state-machine data binds continues to
  use the raw decoded `DataBindContext.sourcePathIds` buffer.
- The admitted synthetic file reports no default source value for the
  name-based data bind, matching C++.
- The cloned bindable number target remains observable at its initial value,
  matching C++ probe output.
