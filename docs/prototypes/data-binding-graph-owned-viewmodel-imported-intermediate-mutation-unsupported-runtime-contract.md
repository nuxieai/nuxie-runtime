# Data Binding Graph Owned ViewModel Imported-Intermediate Mutation Unsupported Runtime Contract

## Purpose

Pin the unsupported C++ behavior for mutating through imported
`ViewModelInstanceViewModel` intermediates from an owned root view-model
context.

The read side can traverse imported intermediates for source binding parity,
but `ViewModelInstanceRuntime::replaceViewModel("child/...")` on the owned
root does not relink descendants once `child` has been replaced by an imported
runtime instance.

## In Scope

- Owned root view-model contexts bound with `bind_owned_view_model_context`.
- Root `ViewModelPropertyViewModel` replacement by imported instance index.
- Attempted nested and deeper `replaceViewModel` paths below that imported
  root property.
- C++ probe coverage showing the imported descendant already present in the
  file remains the source and target binding value.
- Rust returning `false` for `set_view_model_by_property_path` when the path
  would mutate through an imported intermediate.

## Out Of Scope

- Mutating the imported `ViewModelInstance` itself through a separate runtime
  handle.
- Persistent mutation of imported `RuntimeFile` instances across contexts.
- Public property-name handles or stable object handles.
- Scalar, list, symbol, asset, artboard, or trigger mutation through imported
  intermediates.
- Reverse target-to-source propagation, listener-owned data binding, and
  nested artboard propagation.

## Completion Checks

- The one-imported-intermediate unsupported relink probe continues to pass.
- A deeper `[child, middle, leaf]` relink attempt after replacing `child` with
  an imported instance leaves the original imported leaf selected with C++
  parity.
- The read-only imported-intermediate source probes continue to pass.
