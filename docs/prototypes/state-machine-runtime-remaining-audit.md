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
- File-backed imported `ViewModelInstance` context binding through
  `RuntimeDataBindGraph` source path re-resolution, covered by a C++ probe
  through existing `BlendState1DViewModel`, boolean transition-condition, and
  string/color/enum/asset/artboard/trigger transition-condition consumers.
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

## Remaining Runtime Slices

- Scene callback dispatch side effects: listener notification, audio playback,
  open-url side effects, nested-artboard event propagation, and callback
  targets other than `Event.trigger`.
- Listener-owned dispatch: hit testing, listener groups, pointer, keyboard,
  gamepad, semantic/focus inputs, and `ListenerViewModelChange`.
- Live view-model APIs and data-binding propagation governed by
  `docs/prototypes/data-binding-graph-runtime-contract.md`: beyond the finite
  graph-routed default and imported-file-backed external source-to-target
  `propertyValue` bind sets and owned
  number/boolean/string/color/enum/asset/artboard/trigger contexts listed
  above, add source mutation APIs beyond the default
  number/boolean/string/color/enum/asset/artboard/trigger source nodes,
  list/symbol/view-model bindables, converters beyond the admitted boolean
  negate, trigger, boolean-to-number, enum-to-number, color-to-number, and
  string-to-number, symbol-list-index-to-number, number-to-string, and
  boolean-to-string, string-to-string, trigger-to-string,
  symbol-list-index-to-string, and color-to-string paths, data-binding update
  queues, relative paths, parent paths, and nested paths.
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
