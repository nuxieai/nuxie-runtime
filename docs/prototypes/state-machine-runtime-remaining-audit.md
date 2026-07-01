# State Machine Runtime Remaining Audit

## Purpose

This audit is the current scope filter for roadmap item `#11` after the
component-comparand, scalar keyframe, ID keyframe, and callback-keyframe slices.
Older `Current #11 update` paragraphs remain useful history, but this document
is the current remaining-work checklist for deciding the next implementation
slice.

## Closed In The Current Runtime Lane

- Direct linear-animation application for transform doubles, solid colors,
  paint visibility bools, generic uint/ID values, strings, and the first
  callback event reports.
- Public `LinearAnimationInstance` `Event.trigger` callback reports through
  the Rust event-vector seam for plain animation-instance advancement.
- `LinearAnimationInstance` playback timing for simple state-machine usage,
  including loop, ping-pong, work-area, spill, and keep-going behavior.
- Simple animation states, blend states, blend-state transitions, transition
  timing, exit timing, interruption, random transition selection, and transition
  reset behavior for the currently supported animation families.
- State-machine inputs, scheduled fire/listener actions, fire-trigger actions
  through the default view-model context, and explicit data-context trigger
  reset.
- View-model transition conditions for number, integer, boolean, color, string,
  enum, asset, trigger/self, artboard, and root view-model pointer cases covered
  by current probes.
- Component, component-pair, artboard-component, component-view-model scalar,
  trigger/artboard, and intentionally unsupported pointer/artboard directions
  covered by current probes.
- Default-context source-to-target `propertyValue` binds for
  `ViewModelInstanceNumber`, `ViewModelInstanceBoolean`,
  `ViewModelInstanceString`, `ViewModelInstanceColor`,
  `ViewModelInstanceEnum`, `ViewModelInstanceAssetImage`,
  `ViewModelInstanceArtboard`, and `ViewModelInstanceTrigger` sources routed
  through `RuntimeDataBindGraph` source/target edges and covered by current
  probes.
- Graph-owned default `ViewModelInstanceNumber` source-node mutation by
  state-machine data-bind index, covered by a C++ probe through an existing
  `BlendState1DViewModel` consumer.
- Default root number property-name mutation, covered by a C++ probe through
  `ViewModelInstanceRuntime::propertyNumber("amount")->value(...)` with raw
  `propertyValue("amount")` fallback for the file-backed default instance, and
  the same `BlendState1DViewModel` consumer. The contract is
  `docs/prototypes/data-binding-graph-default-number-name-runtime-contract.md`.
- Default root boolean property-name mutation, covered by a C++ probe through
  `ViewModelInstanceRuntime::propertyBoolean("enabled")->value(...)` with raw
  `propertyValue("enabled")` fallback for the file-backed default instance,
  and the existing boolean transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-boolean-name-runtime-contract.md`.
- Default root string property-name mutation, covered by a C++ probe through
  `ViewModelInstanceRuntime::propertyString("label")->value(...)` with raw
  `propertyValue("label")` fallback for the file-backed default instance, and
  the existing string transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-string-name-runtime-contract.md`.
- Default root color property-name mutation, covered by a C++ probe through
  `ViewModelInstanceRuntime::propertyColor("tint")->value(...)` with raw
  `propertyValue("tint")` fallback for the file-backed default instance, and
  the existing color transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-color-name-runtime-contract.md`.
- Default root enum property-name mutation, covered by a C++ probe through
  `ViewModelInstanceRuntime::propertyEnum("choice")->valueIndex(...)` with raw
  `propertyValue("choice")` fallback for the file-backed default instance, and
  the existing enum transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-enum-name-runtime-contract.md`.
- Default root symbol-list-index property-name mutation, covered by a C++ probe
  through raw `ViewModelInstance::propertyValue("symbol")` / property-index
  lookup for the file-backed default instance and the existing
  symbol-list-index-to-string transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-symbol-list-index-name-runtime-contract.md`.
- Default root asset property-name mutation, covered by a C++ probe through
  raw `ViewModelInstance::propertyValue("image")` / property-index lookup for
  the file-backed default instance and the existing asset transition-condition
  consumer. The contract is
  `docs/prototypes/data-binding-graph-default-asset-name-runtime-contract.md`.
- Graph-owned default `ViewModelInstanceBoolean` source-node mutation by
  state-machine data-bind index, covered by a C++ probe through an existing
  transition-condition consumer.
- Graph-owned default `ViewModelInstanceString` source-node mutation by
  state-machine data-bind index, covered by a C++ probe through an existing
  transition-condition consumer.
- Graph-owned default `ViewModelInstanceColor` source-node mutation by
  state-machine data-bind index, covered by a C++ probe through an existing
  transition-condition consumer.
- Graph-owned default `ViewModelInstanceEnum` source-node mutation by
  state-machine data-bind index, covered by a C++ probe through an existing
  transition-condition consumer.
- Graph-owned default `ViewModelInstanceAssetImage` source-node mutation by
  state-machine data-bind index, covered by a C++ probe through an existing
  transition-condition consumer.
- Graph-owned default `ViewModelInstanceArtboard` source-node mutation by
  state-machine data-bind index, covered by a C++ probe through an existing
  transition-condition consumer.
- Graph-owned default `ViewModelInstanceTrigger` source-node mutation by
  state-machine data-bind index, covered by a C++ probe through an existing
  transition-condition consumer.
- Graph-owned default `ViewModelInstanceSymbolListIndex` source-node mutation
  by state-machine data-bind index, covered by a C++ probe through an existing
  symbol-list-index-to-string converter transition-condition consumer.
- File-backed imported `ViewModelInstance` context binding through
  `RuntimeDataBindGraph` source path re-resolution, covered by a C++ probe
  through existing `BlendState1DViewModel`, boolean transition-condition, and
  string/color/enum/asset/artboard/trigger/symbol-list-index
  transition-condition consumers.
- Owned runtime `ViewModelInstance` number context binding through
  `RuntimeDataBindGraph` source path re-resolution, covered by a C++ probe
  through an existing `BlendState1DViewModel` consumer.
- Owned runtime `ViewModelInstance` boolean context binding through
  `RuntimeDataBindGraph` source path re-resolution, covered by a C++ probe
  through an existing transition-condition consumer.
- Owned runtime `ViewModelInstance` string context binding through
  `RuntimeDataBindGraph` source path re-resolution, covered by a C++ probe
  through an existing transition-condition consumer.
- Owned runtime `ViewModelInstance` color context binding through
  `RuntimeDataBindGraph` source path re-resolution, covered by a C++ probe
  through an existing transition-condition consumer.
- Owned runtime `ViewModelInstance` enum context binding through
  `RuntimeDataBindGraph` source path re-resolution, covered by a C++ probe
  through an existing transition-condition consumer.
- Owned runtime `ViewModelInstanceSymbolListIndex` context binding through
  `RuntimeDataBindGraph` source path re-resolution, covered by a C++ probe
  through an existing symbol-list-index-to-string converter
  transition-condition consumer.
- Owned runtime `ViewModelInstanceAssetImage` context binding through
  `RuntimeDataBindGraph` source path re-resolution, covered by a C++ probe
  through an existing transition-condition consumer.
- Owned runtime `ViewModelInstanceArtboard` context binding through
  `RuntimeDataBindGraph` source path re-resolution, covered by a C++ probe
  through an existing transition-condition consumer.
- Owned runtime `ViewModelInstanceTrigger` context binding through
  `RuntimeDataBindGraph` source path re-resolution, covered by a C++ probe
  through an existing transition-condition consumer.
- External and owned view-model trigger identity for `StateMachineFireTrigger`,
  trigger/self conditions, default trigger reports, and explicit data-context
  trigger reset, covered by C++ probes for non-default imported and owned
  trigger contexts.
- First graph-owned converter execution slice:
  `DataConverterBooleanNegate` forward conversion on default-context boolean
  source-to-target bindings, covered by a C++ probe through an existing
  transition-condition consumer.
- Second graph-owned converter execution slice:
  `DataConverterTrigger` forward conversion on default-context trigger
  source-to-target bindings, covered by a C++ probe through an existing
  transition-condition consumer.
- First cross-type graph-owned converter execution slice:
  `DataConverterToNumber` forward conversion for default-context boolean
  sources feeding number targets, covered by a C++ probe through an existing
  blend-state consumer.
- Second cross-type graph-owned converter execution slice:
  `DataConverterToNumber` forward conversion for default-context enum sources
  feeding number targets, covered by a C++ probe through an existing
  blend-state consumer.
- Third cross-type graph-owned converter execution slice:
  `DataConverterToNumber` forward conversion for default-context color sources
  feeding number targets, covered by a C++ probe through an existing
  blend-state consumer.
- Fourth cross-type graph-owned converter execution slice:
  `DataConverterToNumber` forward conversion for default-context string
  sources feeding number targets, covered by a C++ probe through an existing
  blend-state consumer and the binary crate's C++ `std::atof`-style parser
  model.
- String-to-number main-to-target dirty slice: direct `DataConverterToNumber`
  now follows C++ state-machine target-dirty behavior for main-`ToTarget |
  TwoWay` string-to-number binds. Explicit data-context advancement preserves a
  manual bindable number target edit, then normal state-machine advancement
  overwrites it from the unchanged string source through forward
  `std::atof`-style conversion.
- Remaining scalar ToNumber main-to-target dirty slice: direct
  `DataConverterToNumber` now follows the same C++ state-machine target-dirty
  behavior for main-`ToTarget | TwoWay` boolean, enum, color, and
  symbol-list-index sources feeding number targets.
- Boolean ToNumber public-update target-to-source slice: direct
  `DataConverterToNumber` now covers public `updateDataBinds(true)` for a
  main-`ToTarget | TwoWay` number target backed by a boolean source. C++ base
  `reverseConvert` returns the edited numeric target value, but the boolean
  source is not written because the reverse value type does not match; the
  public update still drains the bind and reapplies the unchanged boolean
  source to the number target.
- Remaining ToNumber public-update target-to-source slice: direct
  `DataConverterToNumber` now covers the same public `updateDataBinds(true)`
  base-reverse/type-mismatch behavior for enum, color, string, and
  symbol-list-index sources feeding main-`ToTarget | TwoWay` number targets.
  The source remains unchanged and the public update reapplies
  `DataConverterToNumber::convert` from that unchanged source.
- Fifth cross-type graph-owned converter execution slice:
  `DataConverterToNumber` forward conversion for default-context
  symbol-list-index sources feeding number targets, covered by a C++ probe
  through an existing blend-state consumer.
- `DataConverterListToLength` graph-owned converter execution slice:
  read-only imported list length for default-context list sources feeding
  number targets, covered by a C++ probe through an existing blend-state
  consumer.
- List-to-length main-to-target dirty slice: direct
  `DataConverterListToLength` now follows C++ state-machine target-dirty
  behavior for main-`ToTarget | TwoWay` list-to-number binds. Explicit
  data-context advancement preserves a manual bindable number target edit,
  then normal state-machine advancement overwrites it from the unchanged
  imported list length.
- List-to-length public-update target-to-source slice: direct
  `DataConverterListToLength` now follows C++ public `updateDataBinds(true)`
  base-reverse/type-mismatch behavior for main-`ToTarget | TwoWay`
  list-to-number binds. The imported list-length source remains unchanged and
  the public update reapplies that unchanged length to the number target.
- List-to-length main-to-source target-to-source slice: direct
  `DataConverterListToLength` now follows C++ explicit data-context behavior
  for main-`ToSource | TwoWay` list-to-number binds. The target edit does not
  write the list source, and the same dirty pass refreshes the target through
  base reverse conversion to the default number value `0`.
- First artboard list-consumer slice: default root view-model list sources and
  direct `DataConverterNumberToList` number sources now bind to exact
  artboard-owned `ArtboardComponentList` targets at the immediate
  `Artboard::bindViewModelInstance(...)` boundary. Rust reports the same C++
  source list/number facts, target local, empty target list size, and reset
  flag behavior without admitting list item instancing, map-rule selection,
  layout, or virtualization. The contract is
  `docs/prototypes/data-binding-graph-artboard-list-consumer-runtime-contract.md`.
- `DataConverterRounder` graph-owned converter execution slice: forward
  conversion for default-context number sources feeding number targets,
  including imported `decimals`, covered by a C++ probe through an existing
  blend-state consumer.
- Rounder target-to-source slice: direct `DataConverterRounder` now covers
  main-`ToSource | TwoWay` number target-to-source writes. The edited target
  value flows through C++ main-direction rounder `convert` before writing the
  default-context number source.
- Rounder main-to-target dirty slice: direct `DataConverterRounder` now follows
  C++ state-machine target-dirty behavior for main-`ToTarget | TwoWay` number
  binds. Explicit data-context advancement preserves a manual bindable target
  edit, then normal state-machine advancement overwrites it from the unchanged
  source through forward rounder conversion.
- Rounder public-update target-to-source slice: direct `DataConverterRounder`
  now covers public `updateDataBinds(true)` for main-`ToTarget | TwoWay`
  number binds. The public update writes the edited target through C++ base
  `reverseConvert` pass-through, then reapplies source-to-target through
  rounder conversion in the same update.
- First `DataConverterRangeMapper` graph-owned converter execution slice:
  forward conversion for default-context number sources feeding number targets
  without resolved custom interpolators, including imported range bounds,
  flags, and `interpolationType`, covered by a C++ probe through an existing
  blend-state consumer.
- `DataConverterRangeMapper` resolved-interpolator graph-owned converter
  execution slice: forward conversion for default-context number sources
  feeding number targets with a resolved key-frame interpolator descriptor,
  covered by a C++ probe through an existing blend-state consumer.
- First `DataConverterOperationValue` graph-owned converter execution slice:
  forward conversion for default-context number sources feeding number targets,
  including imported `operationType` and `operationValue`, covered by a C++
  probe through an existing blend-state consumer.
- Second `DataConverterOperationValue` graph-owned converter execution slice:
  forward conversion for default-context symbol-list-index sources feeding
  number targets, covered by a C++ probe through an existing blend-state
  consumer.
- First `DataConverterOperationViewModel` graph-owned converter execution
  slice: forward conversion for default-context number sources feeding number
  targets, using a second imported default view-model number as the operation
  operand and covered by a C++ probe through an existing blend-state consumer.
- Operation-view-model target-to-source slice: direct
  `DataConverterOperationViewModel` now covers main-`ToSource | TwoWay` number
  target-to-source writes. The edited target value flows through `convert`
  with the imported secondary view-model number operand before writing the
  primary source.
- Operation-view-model main-to-target dirty slice: direct
  `DataConverterOperationViewModel` now follows C++ state-machine target-dirty
  behavior for main-`ToTarget | TwoWay` number binds. Explicit data-context
  advancement preserves a manual bindable target edit, then normal
  state-machine advancement overwrites it from the unchanged primary source
  through forward conversion with the imported secondary operand.
- Operation-view-model public-update target-to-source slice: direct
  `DataConverterOperationViewModel` now covers public `updateDataBinds(true)`
  for main-`ToTarget | TwoWay` number binds. The public update reverse-converts
  the edited target with the imported secondary operand, writes the primary
  source, then reapplies source-to-target in the same update.
- First system operation-value converter slice: direct
  `DataConverterSystemNormalizer` and `DataConverterSystemDegsToRads`
  conversion for default-context number sources feeding number targets when
  C++ `DataBind::toTarget()` is true, including default `ToTarget` forward
  arithmetic and `TwoWay | ToSource` reverse arithmetic, covered by a C++
  probe through an existing blend-state consumer.
- System operation-value target-to-source slice: direct
  `DataConverterSystemNormalizer` and `DataConverterSystemDegsToRads` now
  cover main-`ToSource | TwoWay` number target-to-source writes. C++ calls
  `convert` for the main-direction dispatch, and each system converter's
  `convert` method delegates to operation-value reverse arithmetic before the
  source write, after which the same bindable target refreshes from the changed
  source during explicit data-context advancement.
- System operation-value main-to-target dirty slice: direct
  `DataConverterSystemNormalizer` and `DataConverterSystemDegsToRads` now
  follow C++ state-machine target-dirty behavior for main-`ToTarget | TwoWay`
  number binds. Explicit data-context advancement preserves a manual bindable
  target edit, then normal state-machine advancement overwrites it from the
  unchanged source through system-converter forward operation-value arithmetic.
- System operation-value public-update target-to-source slice: direct
  `DataConverterSystemNormalizer` and `DataConverterSystemDegsToRads` now
  cover public `updateDataBinds(true)` for main-`ToTarget | TwoWay` number
  binds. The public update dispatches C++ `reverseConvert`, which selects
  operation-value forward arithmetic from the authored `ToTarget` direction,
  writes the source, then reapplies source-to-target in the same update.
- Direct interpolator main-to-target dirty slice: warmed direct
  `DataConverterInterpolator` state now follows C++ state-machine target-dirty
  behavior for main-`ToTarget | TwoWay` number binds. Explicit data-context
  advancement preserves a manual bindable target edit, then normal
  state-machine advancement reapplies the warmed direct interpolator
  source-to-target converter state even when elapsed time is zero.
- Direct interpolator public-update target-to-source slice: warmed direct
  `DataConverterInterpolator` state now covers public `updateDataBinds(true)`
  for main-`ToTarget | TwoWay` number binds. The public update dispatches C++
  `reverseConvert`, which delegates to the same stateful `convert` path,
  writes the source, then reapplies source-to-target in the same update.
- Direct interpolator main-to-source target-to-source slice: warmed direct
  `DataConverterInterpolator` state now follows C++ explicit data-context
  behavior for main-`ToSource | TwoWay` number binds. Main-direction
  interpolator conversion runs before the source write, then the same pass
  refreshes the target through the interpolator reverse/convert path even when
  the source value is unchanged.
- String binding report seam: the C++ probe now emits exact source/target
  `stringBindings` snapshots beside existing number and view-model binding
  reports, and Rust probe tests can compare default-context string bind values
  directly. This is the enabling slice for string-target dirty parity without
  adding new runtime semantics.
- Number-to-string main-to-target dirty slice: direct `DataConverterToString`
  number-to-string binds now follow C++ state-machine target-dirty behavior for
  main-`ToTarget | TwoWay` string targets. Explicit data-context advancement
  preserves a manual string target edit, then normal state-machine advancement
  overwrites it from the unchanged number source through C++ number formatting.
- Boolean-to-string main-to-target dirty slice: direct `DataConverterToString`
  boolean-to-string binds now follow the same C++ state-machine target-dirty
  behavior for main-`ToTarget | TwoWay` string targets, overwriting a preserved
  manual string edit from the unchanged boolean source on the next normal
  advance.
- String-to-string main-to-target dirty slice: direct `DataConverterToString`
  string-to-string binds now follow the same C++ state-machine target-dirty
  behavior for main-`ToTarget | TwoWay` string targets, preserving the manual
  edit through explicit data-context advancement and then overwriting it from
  the unchanged string source on the next normal advance.
- Trigger-to-string main-to-target dirty slice: direct `DataConverterToString`
  trigger-to-string binds now follow the same C++ state-machine target-dirty
  behavior for main-`ToTarget | TwoWay` string targets, preserving the manual
  edit through explicit data-context advancement and then overwriting it from
  the unchanged trigger count on the next normal advance.
- Symbol-list-index-to-string main-to-target dirty slice: direct
  `DataConverterToString` symbol-list-index-to-string binds now follow the
  same C++ state-machine target-dirty behavior for main-`ToTarget | TwoWay`
  string targets, preserving the manual edit through explicit data-context
  advancement and then overwriting it from the unchanged raw symbol-list-index
  source on the next normal advance.
- Color-to-string main-to-target dirty slice: direct `DataConverterToString`
  color-to-string binds now follow the same C++ state-machine target-dirty
  behavior for main-`ToTarget | TwoWay` string targets, preserving the manual
  edit through explicit data-context advancement and then overwriting it from
  the unchanged color source through imported `colorFormat` conversion on the
  next normal advance.
- Enum-to-string display-label-unsupported main-to-target dirty slice: direct
  `DataConverterToString` enum-to-string binds use C++'s empty-string fallback
  for the default-context string-target runtime graph path even with
  main-`ToTarget | TwoWay` flags, preserving the manual target edit through
  explicit data-context advancement and then overwriting it with an empty
  string on the next normal advance.
- Direct ToString public-update target-to-source slice: direct
  `DataConverterToString` now covers public `updateDataBinds(true)` for
  number, boolean, string, trigger, symbol-list-index, color, and enum sources
  feeding main-`ToTarget | TwoWay` string targets. Non-string sources remain
  unchanged after base reverse conversion, string sources receive the edited
  value, and the public update reapplies source-to-target in the same call.
- Direct ToString non-string main-to-source target-to-source slice: direct
  `DataConverterToString` now covers main-`ToSource | TwoWay` string targets
  backed by number, boolean, trigger, symbol-list-index, color, and enum
  sources. The edited string target does not write the non-string source,
  explicit data-context advancement preserves the edit, and the next normal
  state-machine advance refreshes the target to C++'s empty-string fallback
  through base reverse conversion.
- String-trim main-to-target dirty slice: direct `DataConverterStringTrim`
  binds now follow C++ state-machine target-dirty behavior for
  main-`ToTarget | TwoWay` string targets, preserving the manual target edit
  through explicit data-context advancement and then overwriting it from the
  unchanged string source through imported trim conversion on the next normal
  advance.
- String-remove-zeros main-to-target dirty slice: direct
  `DataConverterStringRemoveZeros` binds now follow C++ state-machine
  target-dirty behavior for main-`ToTarget | TwoWay` string targets,
  preserving the manual target edit through explicit data-context advancement
  and then overwriting it from the unchanged string source through remove-zero
  conversion on the next normal advance.
- String-pad main-to-target dirty slice: direct `DataConverterStringPad` binds
  now follow C++ state-machine target-dirty behavior for main-`ToTarget |
  TwoWay` string targets, preserving the manual target edit through explicit
  data-context advancement and then overwriting it from the unchanged string
  source through imported pad conversion on the next normal advance.
- String converter-family public-update target-to-source slice: direct
  `DataConverterStringTrim`, `DataConverterStringRemoveZeros`, and
  `DataConverterStringPad` now cover public `updateDataBinds(true)` for
  main-`ToTarget | TwoWay` string targets. The edited string writes to the
  string source through base reverse conversion, then the public update
  reapplies the direct string converter in the same call.
- String converter-family main-to-source target-to-source slice: direct
  `DataConverterStringTrim`, `DataConverterStringRemoveZeros`, and
  `DataConverterStringPad` now cover main-`ToSource | TwoWay` string targets.
  C++ keeps the string source unchanged during explicit data-context
  advancement even when the edited target would trim, remove zeros, or pad
  under forward conversion; the next normal state-machine advance refreshes
  the target from the unchanged source through base reverse conversion.
- First `DataConverterToString` graph-owned converter execution slice:
  forward conversion for default-context number sources feeding string targets,
  covered by a C++ probe through an existing string transition-condition
  consumer.
- Second `DataConverterToString` graph-owned converter execution slice:
  forward conversion for default-context boolean sources feeding string
  targets, covered by a C++ probe through an existing string
  transition-condition consumer.
- Third `DataConverterToString` graph-owned converter execution slice:
  forward conversion for default-context string sources feeding string targets,
  covered by a C++ probe through an existing string transition-condition
  consumer.
- Fourth `DataConverterToString` graph-owned converter execution slice:
  forward conversion for default-context trigger sources feeding string
  targets, covered by a C++ probe through an existing string
  transition-condition consumer.
- Fifth `DataConverterToString` graph-owned converter execution slice:
  forward conversion for default-context symbol-list-index sources feeding
  string targets, covered by a C++ probe through an existing string
  transition-condition consumer.
- Sixth `DataConverterToString` graph-owned converter execution slice:
  forward conversion for default-context color sources feeding string targets,
  including imported `colorFormat` descriptor bytes, covered by a C++ probe
  through an existing string transition-condition consumer.
- `DataConverterToString` enum-source runtime graph probe: default-context
  enum sources feeding string targets use C++'s empty-string fallback instead
  of enum display-label conversion even when imported `DataEnum` metadata is
  available. A C++ probe covers this non-transition behavior through an
  existing string transition-condition consumer.
- `DataConverterStringTrim` graph-owned converter execution slice: forward
  conversion for default-context string sources feeding string targets,
  including imported `trimType`, covered by a C++ probe through an existing
  string transition-condition consumer.
- `DataConverterStringRemoveZeros` graph-owned converter execution slice:
  forward conversion for default-context string sources feeding string targets,
  covered by a C++ probe through an existing string transition-condition
  consumer.
- `DataConverterStringPad` graph-owned converter execution slice: forward
  conversion for default-context string sources feeding string targets,
  including imported `length`, `text`, and `padType`, covered by a C++ probe
  through an existing string transition-condition consumer.
- First `DataConverterGroup` graph-owned converter execution slice: forward
  composition for default-context string sources feeding string targets through
  already-supported direct string converters, covered by a C++ probe through an
  existing string transition-condition consumer.
- String converter-group main-to-target dirty slice: the admitted
  `DataConverterStringTrim -> DataConverterStringPad` group now follows C++
  state-machine target-dirty behavior for main-`ToTarget | TwoWay` string
  targets, preserving the manual target edit through explicit data-context
  advancement and then overwriting it from the unchanged string source through
  ordered group conversion on the next normal advance.
- String converter-group public-update target-to-source slice: the admitted
  `DataConverterStringTrim -> DataConverterStringPad` group now follows C++
  public `updateDataBinds(true)` behavior for main-`ToTarget | TwoWay` string
  targets. Reverse group order writes the edited target to the string source
  through base reverse identity, then the same update reapplies forward
  trim-then-pad conversion to the target.
- String converter-group main-to-source target-to-source slice: the admitted
  `DataConverterStringTrim -> DataConverterStringPad` group now follows C++
  main-`ToSource | TwoWay` behavior for string targets. C++ keeps the string
  source unchanged during explicit data-context advancement even when the
  edited target would trim and pad under forward group conversion; the next
  normal state-machine advance refreshes the target from the unchanged source
  through reverse group order and base reverse identity.
- First cross-type `DataConverterGroup` graph-owned converter execution slice:
  forward composition for default-context number sources feeding string targets
  through a group whose first effective child is `DataConverterToString`,
  covered by a C++ probe through an existing string transition-condition
  consumer.
- Number-to-string converter-group main-to-target dirty slice: the admitted
  `DataConverterToString -> DataConverterStringPad` group now follows C++
  state-machine target-dirty behavior for main-`ToTarget | TwoWay` string
  targets, preserving the manual target edit through explicit data-context
  advancement and then overwriting it from the unchanged number source through
  ordered group conversion on the next normal advance.
- Number-to-string converter-group public-update target-to-source slice: the
  admitted `DataConverterToString -> DataConverterStringPad` group now follows
  C++ public `updateDataBinds(true)` behavior for main-`ToTarget | TwoWay`
  string targets. Reverse group conversion leaves the edited string target as
  a string, so the number source is not mutated, then the same public update
  reapplies forward group conversion from the unchanged number source.
- Number-to-string converter-group main-to-source target-to-source slice: the
  admitted `DataConverterToString -> DataConverterStringPad` group now follows
  C++ main-`ToSource | TwoWay` behavior for string targets. C++ keeps the
  number source unchanged during explicit data-context advancement, then the
  next normal state-machine advance refreshes the target through reverse
  `DataConverterStringPad -> DataConverterToString` group order to the
  empty-string fallback.
- First number-to-number `DataConverterGroup` graph-owned converter execution
  slice: forward composition for default-context number sources feeding number
  targets through an `OperationValue -> Rounder` group, covered by a C++ probe
  through an existing blend-state consumer.
- Number-to-number converter-group public-update target-to-source slice: the
  admitted `DataConverterOperationValue -> DataConverterRounder` group now
  follows C++ public `updateDataBinds(true)` behavior for main-`ToTarget |
  TwoWay` number targets. Reverse group order writes the edited target through
  `Rounder -> OperationValue`, then the same public update reapplies forward
  `OperationValue -> Rounder` conversion from the updated number source.
- Number-to-number converter-group main-to-source target-to-source slice: the
  admitted `DataConverterOperationValue -> DataConverterRounder` group now
  follows C++ explicit data-context behavior for main-`ToSource | TwoWay`
  number targets. Forward group order writes the edited target through
  `OperationValue -> Rounder`, then the same dirty pass refreshes the target
  through reverse `Rounder -> OperationValue` group order.
- First stateful `DataConverterInterpolator` graph-owned converter execution
  slice: direct default-context number sources feeding number targets now own
  per-source interpolator state, warm C++'s two-advance startup gate, defer
  initialized zero-second retargets until a positive elapsed pass, and keep the
  state machine advancing while interpolation remains active. A C++ probe
  covers retargeting through an existing blend-state consumer.
- First stateful `DataConverterGroup` graph-owned converter execution slice:
  default-context number sources feeding number targets can now run an
  admitted stateless number converter followed by a stateful
  `DataConverterInterpolator` child. Group child state is stored in imported
  group-item order, group advance aggregates child activity, and a C++ probe
  covers retargeting through an existing blend-state consumer.
- Stateful interpolator converter-group public-update target-to-source slice:
  the admitted `DataConverterOperationValue -> DataConverterInterpolator`
  group now follows C++ public `updateDataBinds(true)` behavior for warmed
  main-`ToTarget | TwoWay` number targets. Reverse group order runs the
  interpolator child's stateful reverse/convert path before operation-value
  reverse conversion, then the same public update reapplies forward group
  conversion with the same child state tree.
- Stateful interpolator converter-group main-to-target dirty slice: the
  admitted `DataConverterOperationValue -> DataConverterInterpolator` group
  now follows C++ state-machine target-dirty behavior for warmed main-`ToTarget
  | TwoWay` number targets. Explicit data-context advancement preserves a
  manual bindable target edit, then normal state-machine advancement reapplies
  the unchanged source through forward group conversion with the same child
  state tree.
- Stateful interpolator converter-group main-to-source target-to-source slice:
  the admitted `DataConverterOperationValue -> DataConverterInterpolator`
  group now follows C++ explicit data-context behavior for warmed
  main-`ToSource | TwoWay` number targets. Forward group conversion runs before
  the source write, then the same pass refreshes the target through reverse
  group conversion with the same child state tree even when the source value is
  unchanged.
- First deterministic `DataConverterFormula` graph-owned converter execution
  slice: default-context number sources feeding number targets can now evaluate
  imported formula output queues made from input, value, and operation tokens,
  including C++ stack collapse and arithmetic operation behavior. A C++ probe
  covers deterministic formula operations through an existing blend-state
  consumer.
- Second deterministic `DataConverterFormula` graph-owned converter execution
  slice: default-context symbol-list-index sources feeding number targets are
  now cast to `f32` before entering the deterministic formula evaluator. A C++
  probe covers the symbol-list-index formula path through an existing
  blend-state consumer.
- First `DataConverterFormula` non-number fallback slice: default-context
  boolean sources feeding number targets now enter the formula converter and
  write C++'s early fallback value `0.0`. A C++ probe uses a non-zero imported
  bindable target default to prove the fallback write is observable through an
  existing blend-state consumer.
- Remaining graph-represented `DataConverterFormula` non-number fallback
  slice: default-context enum, color, string, and trigger sources feeding
  number targets now enter the formula converter and write C++'s early fallback
  value `0.0`. A C++ probe matrix uses non-zero imported bindable target
  defaults to prove each fallback write is observable through an existing
  blend-state consumer.
- First graph-owned target-to-source slice: direct default-context number
  sources feeding number targets now honor `ToSource | TwoWay` target mutation
  on explicit data-context advance. A C++ probe uses two binds to the same
  source so a mutated first bind writes the source before a second bind drives
  an existing blend-state consumer.
- Boolean graph-owned target-to-source slice: direct default-context boolean
  sources feeding boolean targets now honor `ToSource | TwoWay` target mutation
  on explicit data-context advance. A C++ probe uses two binds to the same
  source so a mutated first bind writes the source before a second bind drives
  an existing transition-condition consumer.
- String graph-owned target-to-source slice: direct default-context string
  sources feeding string targets now honor `ToSource | TwoWay` target mutation
  on explicit data-context advance. A C++ probe uses two binds to the same
  source so a mutated first bind writes the source before a second bind drives
  an existing transition-condition consumer.
- Color graph-owned target-to-source slice: direct default-context color
  sources feeding color targets now honor `ToSource | TwoWay` target mutation
  on explicit data-context advance. A C++ probe uses two binds to the same
  source so a mutated first bind writes the source before a second bind drives
  an existing transition-condition consumer.
- Enum graph-owned target-to-source slice: direct default-context enum sources
  feeding enum targets now honor `ToSource | TwoWay` target mutation on
  explicit data-context advance. A C++ probe uses two binds to the same source
  so a mutated first bind writes the source before a second bind drives an
  existing transition-condition consumer.
- Asset graph-owned target-to-source slice: direct default-context asset
  sources feeding asset targets now honor `ToSource | TwoWay` target mutation
  on explicit data-context advance. A C++ probe uses two binds to the same
  source so a mutated first bind writes the source before a second bind drives
  an existing transition-condition consumer.
- Artboard graph-owned target-to-source slice: direct default-context artboard
  sources feeding artboard targets now honor `ToSource | TwoWay` target mutation
  on explicit data-context advance. A C++ probe uses two binds to the same
  source so a mutated first bind writes the source before a second bind drives
  an existing transition-condition consumer.
- Symbol-list-index graph-owned target-to-source slice: direct default-context
  symbol-list-index sources feeding `BindablePropertyInteger.propertyValue`
  targets now honor `ToSource | TwoWay` target mutation on explicit
  data-context advance. A C++ probe uses two binds to the same source so a
  mutated integer target writes the symbol-list-index source before a second
  bind drives an existing symbol-list-index-to-string transition-condition
  consumer. This matches C++'s uint-like target path; there is no separate
  `BindablePropertySymbolListIndex` schema type.
- Trigger graph-owned target-to-source slice: direct default-context trigger
  sources feeding trigger targets now honor `ToSource | TwoWay` target mutation
  on explicit data-context advance. A C++ probe uses two binds to the same
  source so a mutated first bind writes the trigger source before a second bind
  drives an existing trigger-to-string transition-condition consumer, while the
  existing trigger reset still runs after source-to-target application.
- View-model graph-owned target-to-source slice: direct default-context
  `ViewModelInstanceViewModel.propertyValue` sources feeding
  `BindablePropertyViewModel.propertyValue` targets now honor
  `ToSource | TwoWay` target mutation on explicit data-context advance. A C++
  probe reports source and target pointer instance indexes directly because
  post-relink transition comparator evaluation can dereference a missing
  to-target data bind for a to-source view-model fixture.
- Pure `ToSource` flag slice: the admitted direct target-to-source runtime path
  now has representative probe coverage without the `TwoWay` flag. A
  default-context number fixture proves pure `ToSource` target mutation writes
  back to the source while a second `ToTarget` bind observes the updated value.
- First reverse-converter target-to-source slice: default-context boolean
  sources with `DataConverterBooleanNegate` now apply C++'s symmetric
  `reverseConvert` before writing the source, covered by a C++ probe through a
  second direct boolean bind and transition-condition consumer.
- Range-mapper target-to-source slice: default-context number sources with
  `DataConverterRangeMapper` now cover the reachable main-`ToSource | TwoWay`
  state-machine target-to-source path, plus Rust reverse primitive parity for
  C++ `calculateReverseRange()` swapping input/output ranges. Public
  `updateDataBinds(true)` scheduling outside the state-machine
  bindable-property action path remains a follow-up slice.
- Range-mapper main-to-target dirty slice: direct `DataConverterRangeMapper`
  now joins the main-`ToTarget | TwoWay` number dirty contract. A state-machine
  bindable target edit is preserved through explicit `advancedDataContext()`
  and then overwritten from the unchanged source through forward
  `convert` on the next normal state-machine advance.
- Range-mapper public-update reverse target-to-source slice: direct
  `DataConverterRangeMapper` now joins the public `updateDataBinds(true)`
  target-to-source contract for main-`ToTarget | TwoWay` number binds. The
  public update reverse-converts the edited target into the source, then
  reapplies source-to-target during the same update.
- Range-mapper group public-update reverse target-to-source slice:
  `DataConverterGroup<RangeMapper, OperationValue>` now joins the public
  `updateDataBinds(true)` target-to-source contract for main-`ToTarget |
  TwoWay` number binds. The public update applies C++ reverse group order for
  the mutating bind, writes the source, then reapplies grouped
  source-to-target during the same update. Broader neighboring ordinary
  `ToTarget` dirty-list scheduling remains a separate slice.
- Public-update same-path observer preservation slice: the first neighboring
  ordinary `ToTarget` number observer now matches C++ ordering for public
  `updateDataBinds(true)`. The mutating grouped bind writes the source and
  reapplies itself during the public update, while the observer reports the
  new source value, preserves its previous target value for that action, and
  applies the new source on the next normal state-machine advance.
- Range-mapper group target-to-source slice: the first direct
  `DataConverterGroup` containing a `DataConverterRangeMapper` now covers a
  main-`ToSource | TwoWay` number bind. The group runs
  `RangeMapper -> OperationValue` in C++ forward group order before writing
  the source, with a second direct bind observing the grouped source value
  after normal state-machine advancement.
- Formula target-to-source slice: the first deterministic direct
  `DataConverterFormula` target-to-source path now covers a
  main-`ToSource | TwoWay` number bind. The formula's imported output queue
  runs through C++ main-direction `convert` before writing the source, then the
  same formula-bound target refreshes from that changed source during explicit
  data-context advancement, with a second direct bind observing the
  formula-written source value after normal state-machine advancement.
- Formula main-to-target dirty slice: direct deterministic
  `DataConverterFormula` now follows C++ state-machine target-dirty behavior
  for main-`ToTarget | TwoWay` number binds. Explicit data-context advancement
  preserves a manual bindable target edit, then normal state-machine
  advancement overwrites it from the unchanged source through formula
  `convert`.
- Formula public-update target-to-source slice: direct deterministic
  `DataConverterFormula` now covers public `updateDataBinds(true)` for
  main-`ToTarget | TwoWay` number binds. The public update dispatches C++
  `reverseConvert`, which delegates to formula `convert`, writes the source,
  then reapplies source-to-target in the same update.
- First exact number target-to-source direction slice: default-context number
  sources now follow C++ main-direction converter dispatch for the mutating
  bind, with exact C++ probe reports for direct, `OperationValue`, and
  `OperationValue` group source/target values. Main `ToSource` cases use
  `convert` and forward group order; main `ToTarget | TwoWay`
  `OperationValue` and `OperationValue` group target edits are now pinned as
  delayed source-to-target dirty updates rather than immediate reverse
  conversion. Broader dirty-list target scheduling remains a follow-up slice.
- First direct public-update target-to-source slice:
  `StateMachineInstance::updateDataBinds(true)` now covers a converter-free
  number main-`ToTarget | TwoWay` bind. The public update writes the edited
  target into the default source, then reapplies source-to-target during the
  same update while preserving the narrower state-machine bindable-property
  dirty behavior.
- First public-update reverse target-to-source slice:
  `StateMachineInstance::updateDataBinds(true)` now covers a direct
  `DataConverterOperationValue` main-`ToTarget | TwoWay` number bind. The
  public update reverse-converts the edited target into the source, then
  reapplies source-to-target in the same update while preserving the narrower
  state-machine bindable-property dirty behavior.
- First grouped public-update reverse target-to-source slice:
  `StateMachineInstance::updateDataBinds(true)` now covers a
  `DataConverterGroup<OperationValue>` main-`ToTarget | TwoWay` number bind.
  The grouped public update reverse-converts children in C++ reverse group
  order, writes the source, then reapplies source-to-target in forward group
  order during the same public update.
- First graph-owned view-model bindable slice: forward propagation for
  default-context `ViewModelInstanceViewModel.propertyValue` sources feeding
  `BindablePropertyViewModel.propertyValue` targets, covered by a C++ probe
  through an existing view-model pointer transition-condition consumer and the
  explicit data-context advance timing C++ requires for this target kind.
- First raw view-model pointer source mutation slice: default-context
  `ViewModelInstanceViewModel.propertyValue` index writes by referenced
  instance index, covered by a C++ probe proving the generated setter does not
  relink the cached imported pointer observed by an existing view-model pointer
  transition-condition consumer.
- First default view-model pointer relink slice: default-context
  `ViewModelInstanceViewModel` sources can be relinked by referenced imported
  instance index through the live cached-reference replacement path. The
  C++ probe covers the matching
  cached-reference replacement path through an existing view-model pointer
  transition-condition consumer, including source and target pointer updates
  on explicit data-context advance. The contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-relink-runtime-contract.md`.
- First imported view-model pointer relink slice: imported-context
  `ViewModelInstanceViewModel` sources can be relinked by referenced imported
  instance index through the live cached-reference replacement path while that
  context is bound. The C++ probe covers the matching replacement path through
  an existing view-model pointer transition-condition consumer, including
  source and target pointer updates on explicit data-context advance. The
  contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-relink-runtime-contract.md`.
- First owned view-model pointer relink slice: root
  `ViewModelPropertyViewModel` properties in `RuntimeOwnedViewModelInstance`
  can be set by referenced imported instance index, then resolved into
  `BindablePropertyViewModel.propertyValue` targets on explicit data-context
  advance. The C++ probe covers the matching
  `ViewModelInstanceRuntime::replaceViewModelByName` path through an existing
  view-model pointer transition-condition consumer.
- First owned generated child identity slice: root
  `ViewModelPropertyViewModel` properties in `RuntimeOwnedViewModelInstance`
  default to a non-null owned pointer identity when their referenced view model
  exists, matching the nested child created by C++
  `File::createViewModelInstance(viewModel)` before
  `replaceViewModelByName`. The C++ probe covers binding that owned root
  without replacement through an existing view-model pointer
  transition-condition consumer.
- First nested owned view-model pointer relink slice: owned root contexts can
  resolve one generated intermediate `ViewModelInstanceViewModel` child and
  relink a nested `ViewModelInstanceViewModel` source by referenced imported
  instance index. The C++ probe covers the matching
  `ViewModelInstanceRuntime::replaceViewModel("child/grandchild", value)`
  path through existing view-model binding reports. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-relink-runtime-contract.md`.
- Recursive generated owned view-model pointer relink slice:
  `RuntimeOwnedViewModelInstance` now records generated
  `ViewModelPropertyViewModel` descendants recursively, so a generated-only
  path such as `[child, middle, leaf]` can be relinked to a referenced imported
  instance before binding. The C++ probe covers the matching
  `ViewModelInstanceRuntime::replaceViewModel("child/middle/leaf", value)`
  path through existing view-model binding reports. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-recursive-relink-runtime-contract.md`.
- Owned view-model imported-intermediate source slice: replacing a generated
  root child with an imported child by instance index lets
  `RuntimeOwnedViewModelInstance` resolve `[child, grandchild]` through the
  imported child's existing nested `ViewModelInstanceViewModel` reference. The
  C++ probe also pins that attempting
  `ViewModelInstanceRuntime::replaceViewModel("child/grandchild", value)` from
  the owned root does not relink this admitted path, so Rust rejects nested
  mutation after the intermediate is imported. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-runtime-contract.md`.
- Owned view-model imported-intermediate number source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, amount]` through the
  imported child's existing `ViewModelInstanceNumber.propertyValue`. Rust
  records imported number snapshots per referenced view-model instance for
  read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-number-runtime-contract.md`.
- Owned view-model imported-intermediate boolean source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, enabled]` through the
  imported child's existing `ViewModelInstanceBoolean.propertyValue`. Rust
  records imported boolean snapshots per referenced view-model instance for
  read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-boolean-runtime-contract.md`.
- Owned view-model imported-intermediate string source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, label]` through the
  imported child's existing `ViewModelInstanceString.propertyValue`. Rust
  records imported string snapshots per referenced view-model instance for
  read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-string-runtime-contract.md`.
- Owned view-model imported-intermediate color source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, tint]` through the
  imported child's existing `ViewModelInstanceColor.propertyValue`. Rust
  records imported color snapshots per referenced view-model instance for
  read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-color-runtime-contract.md`.
- Owned view-model imported-intermediate enum source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, choice]` through the
  imported child's existing `ViewModelInstanceEnum.propertyValue` index. Rust
  records imported enum-index snapshots per referenced view-model instance for
  read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-enum-runtime-contract.md`.
- Owned view-model imported-intermediate symbol-list-index source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, symbol]` through the
  imported child's existing `ViewModelInstanceSymbolListIndex.propertyValue`.
  Rust records imported symbol-list-index snapshots per referenced view-model
  instance for read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-symbol-list-index-runtime-contract.md`.
- Owned view-model imported-intermediate asset source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, image]` through the
  imported child's existing `ViewModelInstanceAssetImage.propertyValue`. Rust
  records imported asset-id snapshots per referenced view-model instance for
  read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-asset-runtime-contract.md`.
- Owned view-model imported-intermediate artboard source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, scene]` through the
  imported child's existing `ViewModelInstanceArtboard.propertyValue`. Rust
  records imported artboard-id snapshots per referenced view-model instance
  for read-only graph binding. The C++ probe observes this through a
  transition condition that remains unsatisfied after the imported source
  updates the bindable away from the forced target value. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-artboard-runtime-contract.md`.
- Owned view-model imported-intermediate trigger source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, fire]` through the
  imported child's existing `ViewModelInstanceTrigger.propertyValue`. Rust
  records imported trigger-count snapshots per referenced view-model instance
  for read-only graph binding. The C++ probe observes this through a
  transition condition that remains unsatisfied after the imported source
  updates the bindable away from the authored target value. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-trigger-runtime-contract.md`.
- Owned view-model imported-intermediate list source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, items]` through the
  imported child's existing `ViewModelInstanceList` item count. Rust records
  imported list snapshots per referenced view-model instance for read-only
  graph binding, and the state-machine graph keeps a schema-valid list source
  placeholder when the default file instance cannot resolve the authored path.
  The C++ probe observes this through bindable-list source-size reports. The
  contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-list-runtime-contract.md`.
- Owned view-model deep imported-intermediate source slice: replacing a
  generated root child with an imported child by instance index lets
  `RuntimeOwnedViewModelInstance` resolve `[child, middle, leaf]` through the
  imported child's existing imported middle and the middle's existing imported
  leaf. The C++ probe covers the same read-only imported chain through existing
  view-model binding reports. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-deep-imported-intermediate-runtime-contract.md`.
- Owned view-model imported-intermediate mutation unsupported boundary:
  attempting to relink a descendant path below an imported root replacement
  from the owned root leaves the existing imported descendant selected in C++.
  Rust rejects the same `set_view_model_by_property_name_path` mutation once
  the path would cross an imported intermediate. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-mutation-unsupported-runtime-contract.md`.
- Owned generated view-model pointer name-path slice:
  `RuntimeOwnedViewModelInstance` records generated
  `ViewModelPropertyViewModel.name` values and can relink paths such as
  `child/middle/leaf` through
  `set_view_model_by_property_name_path`, matching the C++ public path shape
  for generated owned view-model pointer replacement. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-name-path-runtime-contract.md`.
- First persistent imported view-model pointer relink slice:
  imported-context `ViewModelInstanceViewModel` relinks are remembered in a
  state-machine-local overlay keyed by imported view-model index, imported
  instance index, and source path. Rebinding the same imported context replays
  the relinked pointer with C++ parity. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-persistent-relink-runtime-contract.md`.
- First imported view-model pointer property-name path slice:
  imported-context `ViewModelInstanceViewModel` relinks can resolve a
  slash-separated `ViewModelPropertyViewModel.name` path against the currently
  bound imported view-model context, record the same state-machine-local
  overlay, and replay it after rebinding. The C++ probe runs
  `completeViewModelProperties` before calling
  `ViewModelInstanceRuntime::replaceViewModel`. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-name-path-runtime-contract.md`.
- Shared imported view-model pointer relink context slice:
  `RuntimeImportedViewModelInstanceContext` now owns view-model pointer relink
  overlays for one file-backed imported view-model instance. Relinking a
  `ViewModelInstanceViewModel` source through one state machine updates that
  context, and binding a second state machine through the same context sees
  the relinked source. The C++ probe covers two authored state machines bound
  to the same imported `ViewModelInstance`. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-shared-relink-runtime-contract.md`.
- Shared imported number source mutation slice:
  `RuntimeImportedViewModelInstanceContext` now also owns number source
  overrides for one file-backed imported view-model instance. Mutating a
  `ViewModelInstanceNumber.propertyValue` source through one state machine
  updates that context, and binding a second state machine through the same
  context sees the number source mutation. The C++ probe covers two authored
  state machines bound to the same imported `ViewModelInstance`. The contract
  is
  `docs/prototypes/data-binding-graph-imported-viewmodel-number-shared-mutation-runtime-contract.md`.
- First imported scalar property-name slice:
  `RuntimeImportedViewModelInstanceContext::set_number_by_property_name` now
  resolves a root `ViewModelPropertyNumber.name` against one file-backed
  imported view-model instance and records the existing number source override
  by resolved path. The C++ probe calls
  `ViewModelInstanceRuntime::propertyNumber(name)` and proves two authored
  state machines bound through the same imported context observe the mutation.
  The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-number-name-runtime-contract.md`.
- Shared imported boolean source mutation slice:
  `RuntimeImportedViewModelInstanceContext` now also owns boolean source
  overrides for one file-backed imported view-model instance. Mutating a
  `ViewModelInstanceBoolean.propertyValue` source through one state machine
  updates that context, and binding a second state machine through the same
  context sees the boolean source mutation. The C++ probe covers two authored
  state machines bound to the same imported `ViewModelInstance`. The contract
  is
  `docs/prototypes/data-binding-graph-imported-viewmodel-boolean-shared-mutation-runtime-contract.md`.
- Imported boolean property-name slice:
  `RuntimeImportedViewModelInstanceContext::set_boolean_by_property_name` now
  resolves a root `ViewModelPropertyBoolean.name` against one file-backed
  imported view-model instance and records the existing boolean source
  override by resolved path. The C++ probe calls
  `ViewModelInstanceRuntime::propertyBoolean(name)` and proves two authored
  state machines bound through the same imported context observe matching
  advancement. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-boolean-name-runtime-contract.md`.
- Shared imported string source mutation slice:
  `RuntimeImportedViewModelInstanceContext` now also owns string source
  overrides for one file-backed imported view-model instance. Mutating a
  `ViewModelInstanceString.propertyValue` source through one state machine
  updates that context, and binding a second state machine through the same
  context sees the string source mutation. The C++ probe covers two authored
  state machines bound to the same imported `ViewModelInstance`. The contract
  is
  `docs/prototypes/data-binding-graph-imported-viewmodel-string-shared-mutation-runtime-contract.md`.
- Imported string property-name slice:
  `RuntimeImportedViewModelInstanceContext::set_string_by_property_name` now
  resolves a root `ViewModelPropertyString.name` against one file-backed
  imported view-model instance and records the existing string source override
  by resolved path. The C++ probe calls
  `ViewModelInstanceRuntime::propertyString(name)` and proves two authored
  state machines bound through the same imported context observe the mutation.
  The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-string-name-runtime-contract.md`.
- Shared imported color source mutation slice:
  `RuntimeImportedViewModelInstanceContext` now also owns color source
  overrides for one file-backed imported view-model instance. Mutating a
  `ViewModelInstanceColor.propertyValue` source through one state machine
  updates that context, and binding a second state machine through the same
  context sees the color source mutation. The C++ probe covers two authored
  state machines bound to the same imported `ViewModelInstance` and now emits
  color binding reports. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-color-shared-mutation-runtime-contract.md`.
- Imported color property-name slice:
  `RuntimeImportedViewModelInstanceContext::set_color_by_property_name` now
  resolves a root `ViewModelPropertyColor.name` against one file-backed
  imported view-model instance and records the existing color source override
  by resolved path. The C++ probe calls
  `ViewModelInstanceRuntime::propertyColor(name)` and proves two authored
  state machines bound through the same imported context observe the mutation.
  The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-color-name-runtime-contract.md`.
- Shared imported enum source mutation slice:
  `RuntimeImportedViewModelInstanceContext` now also owns enum source
  overrides for one file-backed imported view-model instance. Mutating a
  `ViewModelInstanceEnum.propertyValue` source through one state machine
  updates that context, and binding a second state machine through the same
  context sees the enum source mutation. The C++ probe covers two authored
  state machines bound to the same imported `ViewModelInstance` and now emits
  enum binding reports. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-enum-shared-mutation-runtime-contract.md`.
- Imported enum property-name slice:
  `RuntimeImportedViewModelInstanceContext::set_enum_by_property_name` now
  resolves root enum view-model property names across
  `ViewModelPropertyEnum`, `ViewModelPropertyEnumCustom`, and
  `ViewModelPropertyEnumSystem`, then records the existing enum source
  override by resolved path. The C++ probe calls
  `ViewModelInstanceRuntime::propertyEnum(name)` and proves two authored
  state machines bound through the same imported context observe the mutation.
  The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-enum-name-runtime-contract.md`.
- Shared imported symbol-list-index source mutation slice:
  `RuntimeImportedViewModelInstanceContext` now also owns symbol-list-index
  source overrides for one file-backed imported view-model instance. Mutating a
  `ViewModelInstanceSymbolListIndex.propertyValue` source through one state
  machine updates that context, and binding a second state machine through the
  same context sees the symbol-list-index source mutation. The C++ probe covers
  two authored state machines bound to the same imported `ViewModelInstance`
  and now emits symbol-list-index binding reports for
  `BindablePropertyInteger` targets. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-symbol-list-index-shared-mutation-runtime-contract.md`.
- Imported symbol-list-index property-name slice:
  `RuntimeImportedViewModelInstanceContext::set_symbol_list_index_by_property_name`
  now resolves a root `ViewModelPropertySymbolListIndex.name` against one
  file-backed imported view-model instance and records the existing
  symbol-list-index source override by resolved path. The C++ probe resolves
  `ViewModel::property(name)` to a root property index, reads
  `ViewModelInstance::propertyValue(index)` as a
  `ViewModelInstanceSymbolListIndex`, and proves two authored state machines
  bound through the same imported context observe the mutation. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-symbol-list-index-name-runtime-contract.md`.
- Shared imported asset source mutation slice:
  `RuntimeImportedViewModelInstanceContext` now also owns asset source
  overrides for one file-backed imported view-model instance. Mutating a
  `ViewModelInstanceAssetImage.propertyValue` source through one state machine
  updates that context, and binding a second state machine through the same
  context sees the asset source mutation. The C++ probe covers two authored
  state machines bound to the same imported `ViewModelInstance` and now emits
  asset binding reports for `BindablePropertyAsset` targets. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-asset-shared-mutation-runtime-contract.md`.
- Imported asset property-name slice:
  `RuntimeImportedViewModelInstanceContext::set_asset_by_property_name` now
  resolves a root `ViewModelPropertyAssetImage` or `ViewModelPropertyAsset`
  name against one file-backed imported view-model instance and records the
  existing asset source override by resolved path. The C++ probe resolves the
  root imported `ViewModelInstanceAssetImage` by name and proves two authored
  state machines bound through the same imported context observe the mutation.
  The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-asset-name-runtime-contract.md`.
- Shared imported artboard source mutation slice:
  `RuntimeImportedViewModelInstanceContext` now also owns artboard source
  overrides for one file-backed imported view-model instance. Mutating a
  `ViewModelInstanceArtboard.propertyValue` source through one state machine
  updates that context, and binding a second state machine through the same
  context sees the artboard source mutation. The C++ probe covers two authored
  state machines bound to the same imported `ViewModelInstance` and now emits
  artboard binding reports for `BindablePropertyArtboard` targets. The
  contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-artboard-shared-mutation-runtime-contract.md`.
- Imported artboard property-name slice:
  `RuntimeImportedViewModelInstanceContext::set_artboard_by_property_name` now
  resolves a root `ViewModelPropertyArtboard` name against one file-backed
  imported view-model instance and records the existing artboard source
  override by resolved path. The C++ probe resolves the root imported
  `ViewModelInstanceArtboard` by name and proves two authored state machines
  bound through the same imported context observe the mutation. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-artboard-name-runtime-contract.md`.
- Shared imported trigger source mutation slice:
  `RuntimeImportedViewModelInstanceContext` now also owns trigger source
  overrides for one file-backed imported view-model instance. Mutating a
  `ViewModelInstanceTrigger.propertyValue` source through one state machine
  updates that context, and binding a second state machine through the same
  context sees the trigger count before ordinary advancement can consume or
  reset it. The C++ probe covers two authored state machines bound to the same
  imported `ViewModelInstance` and compares the observing state machine through
  its ordinary advance report. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-trigger-shared-mutation-runtime-contract.md`.
- Imported trigger property-name slice:
  `RuntimeImportedViewModelInstanceContext::set_trigger_by_property_name` now
  resolves a root `ViewModelPropertyTrigger` name against one file-backed
  imported view-model instance and records the existing trigger source override
  by resolved path. The C++ probe resolves the root imported
  `ViewModelInstanceTrigger` by name and proves an observing authored state
  machine bound through the same imported context sees the trigger count before
  ordinary advancement can consume or reset it. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-trigger-name-runtime-contract.md`.
- Shared imported list source mutation slice:
  `RuntimeImportedViewModelInstanceContext` now also owns list source
  item-count overrides for one file-backed imported view-model instance.
  Mutating a `ViewModelInstanceList` source through one state machine updates
  that context, and binding a second state machine through the same context
  sees the list size. The C++ probe covers two authored state machines bound
  to the same imported `ViewModelInstance` and compares the observing state
  machine through existing `BindablePropertyList` source-size and target
  reports. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-list-shared-mutation-runtime-contract.md`.
- Imported list property-name slice:
  `RuntimeImportedViewModelInstanceContext::set_list_item_count_by_property_name`
  now resolves a root `ViewModelPropertyList` name against one file-backed
  imported view-model instance and records the existing list item-count
  override by resolved path. The C++ probe resolves the root imported list by
  name, clears and repopulates it with blank items, and proves an observing
  authored state machine bound through the same imported context sees the
  source list size through existing `BindablePropertyList` reports. The
  contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-list-name-runtime-contract.md`.
- Default list source mutation slice:
  default root view-model contexts can now mutate a direct
  `ViewModelInstanceList` source item count by state-machine data-bind index.
  The graph updates the same list source fact used by the existing
  `BindablePropertyList` report surface, and the C++ probe covers
  `--runtime-set-default-view-model-source-list` followed by data-context and
  state-machine advancement. The contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-list-source-mutation-runtime-contract.md`.
- First owned scalar property-name slice:
  `RuntimeOwnedViewModelInstance` records root `ViewModelProperty.name` values
  and can mutate a root number property through
  `set_number_by_property_name`, matching the C++ public
  `ViewModelInstanceRuntime::propertyNumber(name)->value(...)` path before
  binding an owned context to a state machine. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-number-name-runtime-contract.md`.
- Owned root scalar property-name completion slice:
  `RuntimeOwnedViewModelInstance` can now mutate all root scalar kinds already
  backed by property-index storage by property name: number, boolean, string,
  color, enum, symbol-list-index, asset, artboard, and trigger. The owned
  scalar C++ parity probes continue to drive C++ through
  `ViewModelProperty.name` and now drive Rust through the matching
  `set_*_by_property_name` APIs. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-root-scalar-name-runtime-contract.md`.
- Owned root list property-name slice:
  `RuntimeOwnedViewModelInstance::set_list_item_count_by_property_name_path`
  now has explicit root-list parity coverage through the single-segment
  `"items"` path. Rust mutates the stored root `ViewModelPropertyList` item
  count before binding an owned context, while the C++ probe uses
  `ViewModelInstanceRuntime::propertyList("items")` and the existing
  owned-list name-path flag. The state machine compares the same
  `BindablePropertyList` source-size reports as the nested list path. The
  contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-root-list-name-runtime-contract.md`.
- First owned generated nested scalar name-path slice:
  generated owned view-model children can store direct number values, and
  `RuntimeOwnedViewModelInstance::set_number_by_property_name_path` can mutate
  paths such as `child/amount` before binding. The graph resolves matching
  nested number source paths for owned contexts, and the C++ probe calls
  `ViewModelInstanceRuntime::propertyNumber("child/amount")->value(...)`. The
  contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-number-name-path-runtime-contract.md`.
- Owned generated nested boolean name-path slice:
  generated owned view-model children can store direct boolean values, and
  `RuntimeOwnedViewModelInstance::set_boolean_by_property_name_path` can
  mutate paths such as `child/enabled` before binding. The graph resolves
  matching nested boolean source paths for owned contexts, and the C++ probe
  calls
  `ViewModelInstanceRuntime::propertyBoolean("child/enabled")->value(...)`.
  The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-boolean-name-path-runtime-contract.md`.
- Owned generated nested string name-path slice:
  generated owned view-model children can store direct string values, and
  `RuntimeOwnedViewModelInstance::set_string_by_property_name_path` can mutate
  paths such as `child/label` before binding. The graph resolves matching
  nested string source paths for owned contexts, and the C++ probe calls
  `ViewModelInstanceRuntime::propertyString("child/label")->value(...)`. The
  contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-string-name-path-runtime-contract.md`.
- Owned generated nested color name-path slice:
  generated owned view-model children can store direct color values, and
  `RuntimeOwnedViewModelInstance::set_color_by_property_name_path` can mutate
  paths such as `child/tint` before binding. The graph resolves matching
  nested color source paths for owned contexts, and the C++ probe calls
  `ViewModelInstanceRuntime::propertyColor("child/tint")->value(...)`. The
  contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-color-name-path-runtime-contract.md`.
- Owned generated nested enum name-path slice:
  generated owned view-model children can store direct enum value indexes, and
  `RuntimeOwnedViewModelInstance::set_enum_by_property_name_path` can mutate
  paths such as `child/choice` before binding. The graph resolves matching
  nested enum source paths for owned contexts, and the C++ probe calls
  `ViewModelInstanceRuntime::propertyEnum("child/choice")->valueIndex(...)`.
  The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-enum-name-path-runtime-contract.md`.
- Owned generated nested symbol-list-index name-path slice:
  generated owned view-model children can store direct symbol-list-index
  values, and
  `RuntimeOwnedViewModelInstance::set_symbol_list_index_by_property_name_path`
  can mutate paths such as `child/symbol` before binding. The graph resolves
  matching nested symbol-list-index source paths for owned contexts, and the
  C++ probe resolves `child` with `propertyViewModel` before mutating the
  child's `ViewModelInstanceSymbolListIndex` through `propertyValue`. The
  contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-symbol-list-index-name-path-runtime-contract.md`.
- Owned generated nested asset name-path slice:
  generated owned view-model children can store direct asset IDs, and
  `RuntimeOwnedViewModelInstance::set_asset_by_property_name_path` can mutate
  paths such as `child/image` before binding. The graph resolves matching
  nested asset source paths for owned contexts, and the C++ probe resolves
  `child` with `propertyViewModel` before mutating the child's
  `ViewModelInstanceAssetImage` through `propertyValue`. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-asset-name-path-runtime-contract.md`.
- Owned generated nested artboard name-path slice:
  generated owned view-model children can store direct artboard IDs, and
  `RuntimeOwnedViewModelInstance::set_artboard_by_property_name_path` can
  mutate paths such as `child/scene` before binding. The graph resolves
  matching nested artboard source paths for owned contexts, and the C++ probe
  resolves `child` with `propertyViewModel` before mutating the child's
  `ViewModelInstanceArtboard` through `propertyValue`. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-artboard-name-path-runtime-contract.md`.
- Owned generated nested trigger name-path slice:
  generated owned view-model children can store direct trigger values, and
  `RuntimeOwnedViewModelInstance::set_trigger_by_property_name_path` can
  mutate paths such as `child/fire` before binding. The graph resolves
  matching nested trigger source paths for owned contexts, and the C++ probe
  resolves `child` with `propertyViewModel` before mutating the child's
  `ViewModelInstanceTrigger` through `propertyValue`. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-trigger-name-path-runtime-contract.md`.
- Owned generated nested list name-path item-count slice:
  generated owned view-model children can store direct list item counts, and
  `RuntimeOwnedViewModelInstance::set_list_item_count_by_property_name_path`
  can mutate paths such as `child/items` before binding. The graph resolves
  matching nested `RuntimeDataBindGraphValue::List` source paths for owned
  contexts, and the C++ probe resolves `child/items` with `propertyList`,
  adding blank item instances to set the observed list size without admitting
  item identity or item-level traversal. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-list-name-path-runtime-contract.md`.
- First `BindablePropertyList.propertyValue` target-to-source slice:
  state-machine list targets can be mutated by data-bind index, and explicit
  `advancedDataContext()` plus public `updateDataBinds(true)` consume the
  dirty target as a C++-compatible no-op for direct `ViewModelInstanceList`
  sources and `DataConverterNumberToList` sources. The contract is
  `docs/prototypes/data-binding-graph-bindable-list-target-to-source-runtime-contract.md`.
- State-machine `NameBased` data-bind source-path unsupported boundary:
  cloned state-machine `DataBindContext` records have a null `DataBind::file()`
  pointer in C++, so manifest-backed `sourcePathIds` do not resolve during
  runtime binding for this shape. Rust keeps the source unbound while reporting
  the cloned bindable number target's initial value. The contract is
  `docs/prototypes/data-binding-graph-name-based-state-machine-source-path-unsupported-runtime-contract.md`.
- Artboard component-list `NameBased` data-bind source-path unsupported
  boundary: binding the default artboard view-model context against a manifest
  path for `items` keeps the component-list target row and empty target-list
  fact, but C++ does not resolve the source list for this runtime shape. Rust
  preserves the same unresolved source facts. The contract is
  `docs/prototypes/data-binding-graph-artboard-name-based-source-path-unsupported-runtime-contract.md`.

## Remaining Runtime Slices

- Scene callback dispatch side effects: listener notification, audio playback,
  open-url side effects, nested-artboard event propagation, and callback
  targets other than `Event.trigger`.
- Live view-model pointer relink APIs beyond the first default, imported,
  owned root-property, generated-only owned, and imported-intermediate owned
  read paths: imported-instance mutation beyond shared view-model pointer
  relink, number source, boolean source, string source, color source, enum
  source, symbol-list-index source, asset source, artboard source, trigger
  source, and list source contexts,
  property-name APIs beyond imported view-model pointer and root
  number/boolean/string/color/enum/symbol-list-index/asset/artboard/trigger/list sources, owned generated view-model pointer
  paths, and stable public handles that update or expose cached
  `propertyValue` indexes.
- Listener-owned dispatch: hit testing, listener groups, pointer, keyboard,
  gamepad, semantic/focus inputs, and `ListenerViewModelChange`.
- Live view-model APIs and data-binding propagation governed by
  `docs/prototypes/data-binding-graph-runtime-contract.md`: beyond the finite
  graph-routed default and imported-file-backed external source-to-target
  `propertyValue` bind sets and owned
  number/boolean/string/color/enum/symbol-list-index/asset/artboard/trigger/list/view-model
  contexts listed above, add source mutation APIs beyond the default
  number/boolean/string/color/enum/asset/artboard/trigger/list/symbol-list-index/view-model source nodes,
  list/symbol bindables and non-default view-model pointer mutation, converters
  beyond the admitted boolean
  negate, trigger, boolean-to-number, enum-to-number, color-to-number, and
  string-to-number, symbol-list-index-to-number, number-to-string, and
  boolean-to-string, string-to-string, trigger-to-string,
  symbol-list-index-to-string, color-to-string, string-trim, and
  string-remove-zeros, string-pad, rounder, range-mapper with/without resolved
  interpolator, list-to-length default list, operation-value
  number/symbol-list-index, system-operation-value direct number,
  operation-view-model default-number,
  string converter group, number-to-string converter group,
  number-to-number converter group paths, and direct stateful
  data-converter-interpolator number smoothing plus grouped
  operation-value-to-interpolator number smoothing, deterministic formula
  number/symbol-list-index-to-number conversion plus graph-represented
  non-number fallbacks,
  first direct number/boolean/string/color/enum/asset/artboard/symbol-list-index/trigger/view-model
  target-to-source propagation,
  first artboard list-consumer immediate bind report,
  data-binding update queues, artboard component-list item instancing,
  map-rule selection, list layout/virtualization, remaining generated-list
  reverse converters, admitted live relative/name lookup with file pointers,
  converter name paths, relative paths, parent paths, and nested paths.
- Nested artboard and nested animation/state-machine remapping.
- Custom/scripted interpolators beyond transition timing and scripted listener
  actions.
- Full layout solving, text shaping/layout rebuilds, renderer integration, and
  draw/render side effects that require the later layout/render lanes.

## Next-Slice Rule

Prefer the next slice that removes a blocker for `#12` or narrows an already
admitted runtime path. Live data-binding work should now enter through
`docs/prototypes/data-binding-graph-runtime-contract.md`; avoid starting hit
testing, nested artboards, or rendering unless the slice contract names the
smallest observable C++ behavior and keeps unrelated runtime systems out of
scope.
