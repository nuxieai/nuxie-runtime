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
- Fifth cross-type graph-owned converter execution slice:
  `DataConverterToNumber` forward conversion for default-context
  symbol-list-index sources feeding number targets, covered by a C++ probe
  through an existing blend-state consumer.
- `DataConverterListToLength` graph-owned converter execution slice:
  read-only imported list length for default-context list sources feeding
  number targets, covered by a C++ probe through an existing blend-state
  consumer.
- `DataConverterRounder` graph-owned converter execution slice: forward
  conversion for default-context number sources feeding number targets,
  including imported `decimals`, covered by a C++ probe through an existing
  blend-state consumer.
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
- First system operation-value converter slice: direct
  `DataConverterSystemNormalizer` and `DataConverterSystemDegsToRads`
  conversion for default-context number sources feeding number targets when
  C++ `DataBind::toTarget()` is true, including default `ToTarget` forward
  arithmetic and `TwoWay | ToSource` reverse arithmetic, covered by a C++
  probe through an existing blend-state consumer.
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
  enum sources feeding string targets stay unsupported for C++ parity even
  when imported `DataEnum` metadata is available. A C++ probe covers this
  non-transition behavior through an existing string transition-condition
  consumer.
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
- First cross-type `DataConverterGroup` graph-owned converter execution slice:
  forward composition for default-context number sources feeding string targets
  through a group whose first effective child is `DataConverterToString`,
  covered by a C++ probe through an existing string transition-condition
  consumer.
- First number-to-number `DataConverterGroup` graph-owned converter execution
  slice: forward composition for default-context number sources feeding number
  targets through an `OperationValue -> Rounder` group, covered by a C++ probe
  through an existing blend-state consumer.
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

## Remaining Runtime Slices

- Scene callback dispatch side effects: listener notification, audio playback,
  open-url side effects, nested-artboard event propagation, and callback
  targets other than `Event.trigger`.
- Live view-model pointer relink APIs beyond the first owned root-property
  path: default and imported replacement paths, and nested owned paths that
  update or expose cached `referenceViewModelInstance` pointers rather than
  only raw generated `propertyValue` indexes.
- Listener-owned dispatch: hit testing, listener groups, pointer, keyboard,
  gamepad, semantic/focus inputs, and `ListenerViewModelChange`.
- Live view-model APIs and data-binding propagation governed by
  `docs/prototypes/data-binding-graph-runtime-contract.md`: beyond the finite
  graph-routed default and imported-file-backed external source-to-target
  `propertyValue` bind sets and owned
  number/boolean/string/color/enum/symbol-list-index/asset/artboard/trigger/view-model
  contexts listed above, add source mutation APIs beyond the default
  number/boolean/string/color/enum/asset/artboard/trigger/symbol-list-index/view-model source nodes,
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
  number/symbol-list-index-to-number conversion plus boolean fallback,
  remaining formula fallback source kinds,
  data-binding update queues, relative paths, parent paths, and nested paths.
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
