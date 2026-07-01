# Data Binding Graph Operation ViewModel Name-Path Unsupported Runtime Contract

## Purpose

Pin the converter-side name-path boundary for
`DataConverterOperationViewModel`.

C++ `DataConverterOperationViewModel::bindFromContext()` reads the converter's
`sourcePathIds` with `DataContext::getViewModelProperty(...)`. It does not use
`DataBindContext::resolvePath()`, the data resolver, or relative manifest-name
lookup. A manifest path id that resolves to the `factor` property therefore
does not bind the secondary operand; the converter falls back to operand
`0.0`.

## In Scope

- Direct `DataConverterOperationViewModel` attached to a
  `BindablePropertyNumber.propertyValue` state-machine data bind.
- Default root view-model context binding.
- Primary numeric `DataBindContext.sourcePathIds` of shape `[0, amount]`.
- Converter `sourcePathIds` containing a manifest path id for `factor`.
- C++ parity for the missing secondary operand fallback through the existing
  blend-state report.

## Out Of Scope

- Adding relative or name-based lookup to
  `DataConverterOperationViewModel::sourcePathIds`.
- Additional grouped `DataConverterOperationViewModel` compositions beyond the
  direct `DataConverterGroup<OperationValue, OperationViewModel>` unsupported
  boundary covered by
  `docs/prototypes/data-binding-graph-operation-viewmodel-group-name-path-unsupported-runtime-contract.md`.
- Parent paths, nested paths, listener-owned data binding, nested artboards,
  and external contexts.
- Changing binary/import manifest resolution helpers; this is a live converter
  binding boundary, not a binary decode boundary.

## Completion Checks

- The manifest-backed converter source-path fixture matches C++.
- Rust keeps the converter secondary operand at `0.0` for the unsupported
  converter name path.
- The existing absolute-path `DataConverterOperationViewModel` converter test
  continues to resolve the secondary operand normally.
