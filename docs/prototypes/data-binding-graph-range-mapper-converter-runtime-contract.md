# Data Binding Graph RangeMapper Converter Runtime Contract

## Purpose

Admit the first `DataConverterRangeMapper` forward-conversion path in the
runtime data-binding graph.

This slice keeps the converter lane narrow: default-context number sources
feeding number targets can use imported range bounds, flags, and hold
interpolation without resolved custom interpolators.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Imported `minInput`, `maxInput`, `minOutput`, `maxOutput`, `flags`, and
  `interpolationType`.
- C++ range-mapper forward behavior for clamp lower, clamp upper, modulo, and
  reverse flags when no custom interpolator is resolved.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer,
  including an upper-clamp fixture.

## Out Of Scope

- Reverse conversion.
- Resolved keyframe interpolators on the range mapper.
- Non-number inputs beyond C++ default-output fallback.
- External and owned contexts for this converter.
- Converter group compositions involving range mapper.
- Operation, formula, system, interpolator, and list converters.
- Update-queue, relative-path, parent-path, nested-path, and listener-owned
  behavior.

## Completion Checks

- A default number source above `maxInput` with `ClampUpper` maps to the
  configured `maxOutput` before state-machine evaluation.
- The mapped number drives the same blend-state report as C++.
- Existing number source, mutation, and converter probes continue to pass.
