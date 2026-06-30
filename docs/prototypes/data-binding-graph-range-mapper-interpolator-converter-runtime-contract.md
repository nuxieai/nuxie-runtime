# Data Binding Graph Range Mapper Interpolator Runtime Contract

## Purpose

Admit `DataConverterRangeMapper` runtime graph execution when the converter
resolves a supported imported key-frame interpolator.

This slice extends the existing default-context number-source range-mapper
path beyond raw `interpolationType` handling without entering stateful
converter scheduling.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterRangeMapper` converters.
- Backboard-owned `CubicEaseInterpolator` and `ElasticInterpolator`
  descriptors resolved through `DataConverterRangeMapper.interpolatorId`.
- C++ percent transform behavior for resolved interpolators, including the
  existing hold-interpolation boundary behavior.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer,
  using a resolved `CubicEaseInterpolator`.

## Out Of Scope

- Reverse conversion.
- `DataConverterInterpolator` stateful smoothing.
- Stateful `DataConverterGroup` execution and data-bind advance scheduling.
- Custom/scripted key-frame interpolators.
- Formula, number-to-list, list-to-length, and scripted converters.
- Relative-path, parent-path, nested-path, and listener-owned behavior.

## Completion Checks

- A default number source flows through a `DataConverterRangeMapper` whose
  `interpolatorId` resolves to an imported key-frame interpolator.
- The resolved interpolator transforms the range-map percent before the number
  target is written.
- The converted number drives the same blend-state report as C++.
