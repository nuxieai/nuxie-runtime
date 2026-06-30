# Data Binding Graph Interpolator Converter Group Runtime Contract

## Scope

This slice admits `DataConverterInterpolator` when it appears inside a
`DataConverterGroup` used by a default root view-model number source feeding a
`BindablePropertyNumber.propertyValue` target.

The runtime graph stores group converter state as a tree whose child slots match
the imported group item order. Stateless children continue to use the existing
graph converter functions. Interpolator children own the same per-source
stateful smoothing model as the direct interpolator slice.

## C++ Behavior To Match

- Group conversion runs child converters in imported order.
- Group advancement visits child converters in imported order and keeps the
  state machine alive if any stateful child remains active.
- Interpolator children retain C++ startup-gate, zero-second retarget deferral,
  duration, and optional resolved interpolator behavior from the direct slice.
- Unsupported or cyclic group children remain explicit `Unsupported` entries
  rather than being treated as pass-through.

## Out Of Scope

- Reverse group conversion and target-to-source bindings.
- Formula, number-to-list, generated-list, scripted, or context-aware group
  children.
- Nested group scheduling beyond the same child-state tree mechanism exercised
  by admitted child converters.
- General `DataBindContainer` dirty queues, persisted polling, relative paths,
  parent paths, nested paths, listener-owned data binding, and nested artboard
  propagation.

## Verification

- A synthetic `.riv` fixture binds a default view-model number source through
  an `OperationValue -> DataConverterInterpolator` group into a blend-state
  number target.
- The probe warms the group, mutates the source, advances over partial duration
  windows, and compares Rust state-machine/runtime update output against C++.
