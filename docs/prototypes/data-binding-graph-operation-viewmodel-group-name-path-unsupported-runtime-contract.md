# Data Binding Graph Operation ViewModel Group Name-Path Unsupported Runtime Contract

## Purpose

Pin the grouped converter-side name-path boundary for
`DataConverterOperationViewModel`.

C++ `DataConverterGroup::bindFromContext()` forwards binding to each child
converter. When the child is `DataConverterOperationViewModel`, that child
still reads `sourcePathIds` with
`DataContext::getViewModelProperty(...)`. It does not use
`DataBindContext::resolvePath()`, the data resolver, or relative manifest-name
lookup. A manifest path id that resolves to the `factor` property therefore
does not bind the grouped secondary operand; the converter falls back to
operand `0.0`.

## In Scope

- A state-machine `DataBindContext` targeting
  `BindablePropertyNumber.propertyValue`.
- Default root view-model context binding.
- Primary numeric `DataBindContext.sourcePathIds` of shape `[0, amount]`.
- A `DataConverterGroup` containing `DataConverterOperationValue` followed by
  `DataConverterOperationViewModel`.
- The grouped operation-view-model child's `sourcePathIds` containing a
  manifest path id for `factor`.
- C++ parity for the missing secondary operand fallback through the existing
  blend-state report.

## Out Of Scope

- Adding relative or name-based lookup to
  `DataConverterOperationViewModel::sourcePathIds`.
- Other grouped operation-view-model compositions.
- Parent paths, nested paths, listener-owned data binding, nested artboards,
  and external contexts.
- Changing binary/import manifest resolution helpers; this is a live converter
  binding boundary, not a binary decode boundary.

## Completion Checks

- The grouped manifest-backed converter source-path fixture matches C++.
- Rust keeps the grouped operation-view-model child operand at `0.0` for the
  unsupported converter name path.
- The existing grouped absolute-path operation-view-model tests continue to
  resolve the secondary operand normally.
