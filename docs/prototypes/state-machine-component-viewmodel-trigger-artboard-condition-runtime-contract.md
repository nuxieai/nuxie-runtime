# State Machine Component/ViewModel Trigger And Artboard Condition Runtime Contract

## Scope

This slice covers `TransitionViewModelCondition` comparators where a
`TransitionPropertyComponentComparator` is paired with a
`TransitionPropertyViewModelComparator`, in either order, and both sides resolve
to C++ `ComponentComparandKind::Trigger` or `ComponentComparandKind::Artboard`.

The Rust runtime must:

- Admit trigger/trigger and artboard/artboard component/ViewModel comparator
  pairs when C++ `TransitionViewModelCondition::initialize()` admits them.
- Read imported `BindablePropertyTrigger.propertyValue` and
  `BindablePropertyArtboard.propertyValue` from state-machine-owned data-bind
  targets.
- Compare both families as C++ `ComparisonShape::Uint32`: `Equal` and
  `NotEqual` are meaningful, while ordered operations evaluate false.
- Preserve the existing C++ data-context presence gate for ViewModel
  comparators.
- Preserve component missing/unsupported-property defaults through the existing
  component-comparand value helpers.
- Preserve trigger/self `TransitionViewModelCondition` behavior as a separate
  path from `BindablePropertyTrigger.propertyValue` comparison.

## Defaults

`BindablePropertyTrigger` inherits `BindablePropertyInteger`, so an omitted
`propertyValue` defaults to `0`.

`BindablePropertyArtboard` inherits `BindablePropertyId`, so an omitted
`propertyValue` defaults to `uint32_t(-1)`.

If a bindable instance is absent at evaluation time, C++ comparands return `0`.
Rust should keep that behavior by using `0` when a runtime state-machine
bindable instance cannot be found.

## Out Of Scope

This slice does not add live data-binding propagation, trigger firing/reset
semantics, listener-owned dispatch, nested or relative ViewModel paths,
ViewModel pointer comparisons, runtime-layout-driven artboard dimensions,
component-left/artboard-right support, layout solving, animation remapping, hit
testing, rendering, or public live ViewModel APIs.

## Completion Checks

- C++ probe coverage includes trigger and artboard component/ViewModel
  comparisons in both supported comparator orders.
- Coverage includes no-context gating and ordered-operation false behavior.
- `docs/porting-map.md` records that component/ViewModel trigger and artboard
  semantics are closed for static imported bindables.
