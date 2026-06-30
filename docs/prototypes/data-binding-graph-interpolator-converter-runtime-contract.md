# Data Binding Graph Interpolator Converter Runtime Contract

## Scope

This slice admits the first stateful converter in `RuntimeDataBindGraph`:
direct `DataConverterInterpolator` bindings from a default root view-model
number source into a `BindablePropertyNumber.propertyValue` target observed by
`BlendState1DViewModel`.

The graph owns the per-source interpolator state. Imported converter metadata is
reduced to the finite runtime descriptor needed after import: duration and the
optional resolved key-frame interpolator descriptor. Live conversion no longer
needs to ask `rive-binary` for object data on every frame.

## C++ Behavior To Match

- Binding a data context resets converter state.
- The first dirty conversion initializes the interpolator from the current
  source value.
- Positive elapsed advances increment the C++ two-advance startup gate.
- After the startup gate, source changes retarget the current interpolator
  instead of snapping immediately to the new source value.
- While interpolation is active, state-machine advance reports that more work
  remains and reapplies the current converted value before layer evaluation.
- Duration and resolved cubic/elastic interpolator descriptors use the same
  factor math as existing transition/range-mapper interpolation.

## Out Of Scope

- `DataConverterInterpolator` inside `DataConverterGroup`.
- Reverse conversion and target-to-source bindings.
- Formula, number-to-list, generated-list, scripted, or context-aware converter
  scheduling.
- General `DataBindContainer` dirty queues, persisted polling, relative paths,
  parent paths, nested paths, listener-owned data binding, and nested artboard
  propagation.
- Public stable data-bind source handles or external live view-model mutation
  APIs beyond the existing default source mutation helpers.

## Verification

- A synthetic `.riv` fixture binds the default view-model number source through
  a direct `DataConverterInterpolator` into a blend-state number target.
- The probe warms the interpolator past C++'s startup gate, mutates the default
  source, advances over partial and full duration windows, and compares the
  Rust runtime update against the C++ probe.
