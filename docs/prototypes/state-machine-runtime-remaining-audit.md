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
  through the default view-model context, claimed relative `DataBindPath`
  fire-trigger actions pinned as unresolved for this scheduling path, and
  explicit data-context trigger reset.
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
- Default-context nested number source binding: an absolute
  `DataBindContext.sourcePathIds` path such as `[Root, child, amount]` walks a
  root `ViewModelInstanceViewModel.propertyValue` reference to an imported
  child instance and reads the child's
  `ViewModelInstanceNumber.propertyValue`, covered by a C++ probe through the
  existing `BlendState1DViewModel` consumer. The contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-nested-number-runtime-contract.md`.
- Default-context nested boolean source binding: an absolute
  `DataBindContext.sourcePathIds` path such as `[Root, child, enabled]` walks
  the same imported child reference and reads the child's
  `ViewModelInstanceBoolean.propertyValue`, covered by C++ boolean binding
  reports and the existing transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-nested-boolean-runtime-contract.md`.
- Default-context nested string source binding: an absolute
  `DataBindContext.sourcePathIds` path such as `[Root, child, label]` walks
  the same imported child reference and reads the child's
  `ViewModelInstanceString.propertyValue` bytes, covered by C++ string binding
  reports and the existing transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-nested-string-runtime-contract.md`.
- Default-context nested color source binding: an absolute
  `DataBindContext.sourcePathIds` path such as `[Root, child, tint]` walks the
  same imported child reference and reads the child's
  `ViewModelInstanceColor.propertyValue`, covered by C++ color binding reports
  and the existing transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-nested-color-runtime-contract.md`.
- Default-context nested enum source binding: an absolute
  `DataBindContext.sourcePathIds` path such as `[Root, child, choice]` walks
  the same imported child reference and reads the child's
  `ViewModelInstanceEnum.propertyValue`, covered by C++ enum binding reports
  and the existing transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-nested-enum-runtime-contract.md`.
- Default-context nested symbol-list-index source binding: an absolute
  `DataBindContext.sourcePathIds` path such as `[Root, child, symbol]` walks
  the same imported child reference and reads the child's
  `ViewModelInstanceSymbolListIndex.propertyValue` before the existing
  `DataConverterToString` path writes the string bindable, covered by the
  existing transition-condition and component-update report surfaces. The
  contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-nested-symbol-list-index-runtime-contract.md`.
- Default-context nested asset source binding: an absolute
  `DataBindContext.sourcePathIds` path such as `[Root, child, image]` walks
  the same imported child reference and reads the child's
  `ViewModelInstanceAssetImage.propertyValue`, covered by C++ asset binding
  reports and the existing transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-nested-asset-runtime-contract.md`.
- Graph-owned default `ViewModelInstanceNumber` source-node mutation by
  state-machine data-bind index, covered by a C++ probe through an existing
  `BlendState1DViewModel` consumer.
- Default number source mutation same-path observer propagation: the
  data-bind-index mutation path resolves the selected source path and updates
  all same-path default-context number source nodes, covered by a C++ probe
  with a neighboring ordinary direct `ToTarget` number bind.
- Default boolean source mutation same-path observer propagation: the
  data-bind-index mutation path resolves the selected source path and updates
  all same-path default-context boolean source nodes, covered by a C++ probe
  with a neighboring ordinary direct `ToTarget` boolean bind.
- Default string source mutation same-path observer propagation: the
  data-bind-index mutation path resolves the selected source path and updates
  all same-path default-context string source nodes, covered by a C++ probe
  with a neighboring ordinary direct `ToTarget` string bind.
- Default color source mutation same-path observer propagation: the
  data-bind-index mutation path resolves the selected source path and updates
  all same-path default-context color source nodes, covered by a C++ probe
  with a neighboring ordinary direct `ToTarget` color bind.
- Default enum source mutation same-path observer propagation: the
  data-bind-index mutation path resolves the selected source path and updates
  all same-path default-context enum source nodes, covered by a C++ probe with
  a neighboring ordinary direct `ToTarget` enum bind.
- Default symbol-list-index source mutation same-path observer propagation:
  the data-bind-index mutation path resolves the selected source path and
  updates all same-path default-context symbol-list-index source nodes,
  covered by a C++ probe with a neighboring ordinary direct `ToTarget`
  integer bind.
- Default asset source mutation same-path observer propagation: the
  data-bind-index mutation path resolves the selected source path and updates
  all same-path default-context asset source nodes, covered by a C++ probe
  with a neighboring ordinary direct `ToTarget` asset bind.
- Default artboard source mutation same-path observer propagation: the
  data-bind-index mutation path resolves the selected source path and updates
  all same-path default-context artboard source nodes, covered by a C++ probe
  with a neighboring ordinary direct `ToTarget` artboard bind.
- Default trigger source mutation same-path observer propagation: the
  data-bind-index mutation path resolves the selected source path and updates
  all same-path default-context trigger source nodes, covered by a C++ probe
  with a neighboring ordinary direct `ToTarget` trigger bind.
- Default list source mutation same-path observer propagation: the
  data-bind-index mutation path resolves the selected source path and updates
  all same-path default-context list source nodes, covered by a C++ probe with
  a neighboring ordinary direct `ToTarget` bindable-list bind.
- Default view-model pointer relink same-path observer propagation: the
  data-bind-index relink path resolves the selected source path and updates
  all same-path default-context view-model pointer source nodes, covered by a
  C++ probe with a neighboring ordinary direct `ToTarget` view-model bind.
- Default root number property-name mutation, covered by a C++ probe through
  `ViewModelInstanceRuntime::propertyNumber("amount")->value(...)` with raw
  `propertyValue("amount")` fallback for the file-backed default instance, and
  the same `BlendState1DViewModel` consumer. The contract is
  `docs/prototypes/data-binding-graph-default-number-name-runtime-contract.md`.
- Default number source handle slice:
  `StateMachineInstance` can now resolve root `ViewModelPropertyNumber.name`
  values on file view model `0` into
  `RuntimeDefaultViewModelNumberSourceHandle` and mutate graph-owned default
  number source nodes through that handle. Root-name handle lookup remains
  separate from slash-path lookup. The C++ probe compares the handle write
  against the default number by-name mutation command. The contract is
  `docs/prototypes/data-binding-graph-default-number-source-handle-runtime-contract.md`.
- Default nested number source handle slice:
  `StateMachineInstance` can now resolve a generated child path such as
  `child/amount` into `RuntimeDefaultViewModelNumberSourceHandle` through
  `default_view_model_number_source_handle_by_property_name_path` and mutate
  graph-owned default number source nodes through that handle. The C++ probe
  compares the handle write against the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind. The
  contract is
  `docs/prototypes/data-binding-graph-default-nested-number-source-handle-runtime-contract.md`.
- Default root boolean property-name mutation, covered by a C++ probe through
  `ViewModelInstanceRuntime::propertyBoolean("enabled")->value(...)` with raw
  `propertyValue("enabled")` fallback for the file-backed default instance,
  and the existing boolean transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-boolean-name-runtime-contract.md`.
- Default boolean source handle slice:
  `StateMachineInstance` can now resolve root `ViewModelPropertyBoolean.name`
  values on file view model `0` into
  `RuntimeDefaultViewModelBooleanSourceHandle` and mutate graph-owned default
  boolean source nodes through that handle. Root-name handle lookup remains
  separate from slash-path lookup. The C++ probe compares the handle write
  against the default boolean by-name mutation command. The contract is
  `docs/prototypes/data-binding-graph-default-boolean-source-handle-runtime-contract.md`.
- Default nested boolean source handle slice:
  `StateMachineInstance` can now resolve a generated child path such as
  `child/enabled` into `RuntimeDefaultViewModelBooleanSourceHandle` through
  `default_view_model_boolean_source_handle_by_property_name_path` and mutate
  graph-owned default boolean source nodes through that handle. The C++ probe
  compares the handle write against the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind. The contract is
  `docs/prototypes/data-binding-graph-default-nested-boolean-source-handle-runtime-contract.md`.
- Default root string property-name mutation, covered by a C++ probe through
  `ViewModelInstanceRuntime::propertyString("label")->value(...)` with raw
  `propertyValue("label")` fallback for the file-backed default instance, and
  the existing string transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-string-name-runtime-contract.md`.
- Default string source handle slice:
  `StateMachineInstance` can now resolve root `ViewModelPropertyString.name`
  values on file view model `0` into
  `RuntimeDefaultViewModelStringSourceHandle` and mutate graph-owned default
  string source nodes through that handle. Root-name handle lookup remains
  separate from slash-path lookup. The C++ probe compares the handle write
  against the default string by-name mutation command. The contract is
  `docs/prototypes/data-binding-graph-default-string-source-handle-runtime-contract.md`.
- Default nested string source handle slice:
  `StateMachineInstance` can now resolve a generated child path such as
  `child/label` into `RuntimeDefaultViewModelStringSourceHandle` through
  `default_view_model_string_source_handle_by_property_name_path` and mutate
  graph-owned default string source nodes through that handle. The C++ probe
  compares the handle write against the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind. The contract is
  `docs/prototypes/data-binding-graph-default-nested-string-source-handle-runtime-contract.md`.
- Default root color property-name mutation, covered by a C++ probe through
  `ViewModelInstanceRuntime::propertyColor("tint")->value(...)` with raw
  `propertyValue("tint")` fallback for the file-backed default instance, and
  the existing color transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-color-name-runtime-contract.md`.
- Default color source handle slice:
  `StateMachineInstance` can now resolve root `ViewModelPropertyColor.name`
  values on file view model `0` into
  `RuntimeDefaultViewModelColorSourceHandle` and mutate graph-owned default
  color source nodes through that handle. Root-name handle lookup remains
  separate from slash-path lookup. The C++ probe compares the handle write
  against the default color by-name mutation command. The contract is
  `docs/prototypes/data-binding-graph-default-color-source-handle-runtime-contract.md`.
- Default nested color source handle slice:
  `StateMachineInstance` can now resolve a generated child path such as
  `child/tint` into `RuntimeDefaultViewModelColorSourceHandle` through
  `default_view_model_color_source_handle_by_property_name_path` and mutate
  graph-owned default color source nodes through that handle. The C++ probe
  compares the handle write against the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind. The contract is
  `docs/prototypes/data-binding-graph-default-nested-color-source-handle-runtime-contract.md`.
- Default root enum property-name mutation, covered by a C++ probe through
  `ViewModelInstanceRuntime::propertyEnum("choice")->valueIndex(...)` with raw
  `propertyValue("choice")` fallback for the file-backed default instance, and
  the existing enum transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-enum-name-runtime-contract.md`.
- Default enum source handle slice:
  `StateMachineInstance` can now resolve root enum view-model property names
  on file view model `0` into `RuntimeDefaultViewModelEnumSourceHandle` and
  mutate graph-owned default enum source nodes through that handle. Root-name
  handle lookup remains separate from slash-path lookup. The C++ probe
  compares the handle write against the default enum by-name mutation command.
  The contract is
  `docs/prototypes/data-binding-graph-default-enum-source-handle-runtime-contract.md`.
- Default nested enum source handle slice:
  `StateMachineInstance` can now resolve a generated child path such as
  `child/choice` into `RuntimeDefaultViewModelEnumSourceHandle` through
  `default_view_model_enum_source_handle_by_property_name_path` and mutate
  graph-owned default enum source nodes through that handle. The C++ probe
  compares the handle write against the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind. The contract is
  `docs/prototypes/data-binding-graph-default-nested-enum-source-handle-runtime-contract.md`.
- Default root symbol-list-index property-name mutation, covered by a C++ probe
  through raw `ViewModelInstance::propertyValue("symbol")` / property-index
  lookup for the file-backed default instance and the existing
  symbol-list-index-to-string transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-symbol-list-index-name-runtime-contract.md`.
- Default symbol-list-index source handle slice:
  `StateMachineInstance` can now resolve root
  `ViewModelPropertySymbolListIndex.name` values on file view model `0` into
  `RuntimeDefaultViewModelSymbolListIndexSourceHandle` and mutate graph-owned
  default symbol-list-index source nodes through that handle. Root-name handle
  lookup remains separate from slash-path lookup. The C++ probe compares the
  handle write against the default symbol-list-index by-name mutation command.
  The contract is
  `docs/prototypes/data-binding-graph-default-symbol-list-index-source-handle-runtime-contract.md`.
- Default nested symbol-list-index source handle slice:
  `StateMachineInstance` can now resolve a generated child path such as
  `child/symbol` into `RuntimeDefaultViewModelSymbolListIndexSourceHandle`
  through
  `default_view_model_symbol_list_index_source_handle_by_property_name_path`
  and mutate graph-owned default symbol-list-index source nodes through that
  handle. The C++ probe compares the handle write against the authored
  `DataBindContext.sourcePathIds` mutation path for the matching
  default-context data bind. The contract is
  `docs/prototypes/data-binding-graph-default-nested-symbol-list-index-source-handle-runtime-contract.md`.
- Default root asset property-name mutation, covered by a C++ probe through
  raw `ViewModelInstance::propertyValue("image")` / property-index lookup for
  the file-backed default instance and the existing asset transition-condition
  consumer. The contract is
  `docs/prototypes/data-binding-graph-default-asset-name-runtime-contract.md`.
- Default asset source handle slice:
  `StateMachineInstance` can now resolve root asset view-model property names
  on file view model `0` into `RuntimeDefaultViewModelAssetSourceHandle` and
  mutate graph-owned default asset source nodes through that handle. Root-name
  handle lookup remains separate from slash-path lookup. The C++ probe
  compares the handle write against the default asset by-name mutation command.
  The contract is
  `docs/prototypes/data-binding-graph-default-asset-source-handle-runtime-contract.md`.
- Default nested asset source handle slice:
  `StateMachineInstance` can now resolve a generated child path such as
  `child/image` into `RuntimeDefaultViewModelAssetSourceHandle` through
  `default_view_model_asset_source_handle_by_property_name_path` and mutate
  graph-owned default asset source nodes through that handle. The C++ probe
  compares the handle write against the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind. The contract is
  `docs/prototypes/data-binding-graph-default-nested-asset-source-handle-runtime-contract.md`.
- Default root artboard property-name mutation, covered by a C++ probe through
  raw `ViewModelInstance::propertyValue("scene")` / property-index lookup for
  the file-backed default instance and the existing artboard
  transition-condition consumer. The contract is
  `docs/prototypes/data-binding-graph-default-artboard-name-runtime-contract.md`.
- Default artboard source handle slice:
  `StateMachineInstance` can now resolve root artboard view-model property
  names on file view model `0` into
  `RuntimeDefaultViewModelArtboardSourceHandle` and mutate graph-owned default
  artboard source nodes through that handle. Root-name handle lookup remains
  separate from slash-path lookup. The C++ probe compares the handle write
  against the default artboard by-name mutation command. The contract is
  `docs/prototypes/data-binding-graph-default-artboard-source-handle-runtime-contract.md`.
- Default nested artboard source handle slice:
  `StateMachineInstance` can now resolve a generated child path such as
  `child/scene` into `RuntimeDefaultViewModelArtboardSourceHandle` through
  `default_view_model_artboard_source_handle_by_property_name_path` and mutate
  graph-owned default artboard source nodes through that handle. The C++ probe
  compares the handle write against the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind. The contract is
  `docs/prototypes/data-binding-graph-default-nested-artboard-source-handle-runtime-contract.md`.
- Default root trigger property-name mutation, covered by a C++ probe through
  raw `ViewModelInstance::propertyValue("fire")` / property-index lookup for
  the file-backed default instance and the existing trigger transition-condition
  consumer. The contract is
  `docs/prototypes/data-binding-graph-default-trigger-name-runtime-contract.md`.
- Default trigger source handle slice:
  `StateMachineInstance` can now resolve root trigger view-model property
  names on file view model `0` into
  `RuntimeDefaultViewModelTriggerSourceHandle` and mutate graph-owned default
  trigger source nodes through that handle while preserving trigger target
  mirror updates. Root-name handle lookup remains separate from slash-path
  lookup. The C++ probe compares the handle write against the default trigger
  by-name mutation command. The contract is
  `docs/prototypes/data-binding-graph-default-trigger-source-handle-runtime-contract.md`.
- Default nested trigger source handle slice:
  `StateMachineInstance` can now resolve a generated child path such as
  `child/fire` into `RuntimeDefaultViewModelTriggerSourceHandle` through
  `default_view_model_trigger_source_handle_by_property_name_path` and mutate
  graph-owned default trigger source nodes through that handle while preserving
  matching trigger target mirror updates. The C++ probe compares the handle
  write against the authored `DataBindContext.sourcePathIds` mutation path for
  the matching default-context data bind. The contract is
  `docs/prototypes/data-binding-graph-default-nested-trigger-source-handle-runtime-contract.md`.
- Default root list property-name item-count mutation, covered by a C++ probe
  through `ViewModelInstanceRuntime::propertyList("items")` with raw
  `ViewModelInstance::propertyValue("items")` / property-index fallback for the
  file-backed default instance and the existing bindable-list source-size
  consumer. The contract is
  `docs/prototypes/data-binding-graph-default-list-name-runtime-contract.md`.
- Default list source handle slice:
  `StateMachineInstance` can now resolve root list view-model property names
  on file view model `0` into `RuntimeDefaultViewModelListSourceHandle` and
  mutate graph-owned default list source nodes by item count through that
  handle. Root-name handle lookup remains separate from slash-path lookup. The
  C++ probe compares the handle write against the default list by-name mutation
  command. The contract is
  `docs/prototypes/data-binding-graph-default-list-source-handle-runtime-contract.md`.
- Default nested list source handle slice:
  `StateMachineInstance` can now resolve a generated child path such as
  `child/items` into `RuntimeDefaultViewModelListSourceHandle` through
  `default_view_model_list_source_handle_by_property_name_path` and mutate
  graph-owned default list source nodes by item count through that handle. The
  C++ probe compares the handle write against the authored
  `DataBindContext.sourcePathIds` mutation path for the matching
  default-context data bind. The contract is
  `docs/prototypes/data-binding-graph-default-nested-list-source-handle-runtime-contract.md`.
- Default root view-model property-name relink, covered by a C++ probe through
  `ViewModelInstanceRuntime::replaceViewModel("current", referencedRuntime)`
  for the file-backed default instance and the existing view-model pointer
  source/target consumer. The contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-name-relink-runtime-contract.md`.
- Default view-model pointer source handle slice:
  `StateMachineInstance` can now resolve root view-model pointer property names
  on file view model `0` into
  `RuntimeDefaultViewModelViewModelSourceHandle` and relink graph-owned
  default view-model source nodes by imported referenced instance index through
  that handle. Root-name handle lookup remains separate from slash-path lookup,
  and the default root source-handle family is now covered for
  number/boolean/string/color/enum/symbol-list-index/asset/artboard/trigger/list/view-model
  sources. The C++ probe compares the handle relink against the default
  view-model by-name relink command. The contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-source-handle-runtime-contract.md`.
- Default nested view-model pointer source handle slice:
  `StateMachineInstance` can now resolve a generated child path such as
  `child/grandchild` into `RuntimeDefaultViewModelViewModelSourceHandle`
  through
  `default_view_model_view_model_source_handle_by_property_name_path` and
  relink graph-owned default view-model pointer source nodes by imported
  referenced instance index through that handle. The C++ probe compares the
  handle relink against the default view-model by-name path relink command for
  the matching nested path. The contract is
  `docs/prototypes/data-binding-graph-default-nested-viewmodel-source-handle-runtime-contract.md`.
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
- Boolean public-update target-to-source slice: main-`ToTarget | TwoWay`
  boolean binds now participate in public `updateDataBinds(true)` for direct
  no-converter and direct `DataConverterBooleanNegate` paths. The C++ probe
  reports exact boolean binding source/target rows, and Rust mirrors direct
  writes plus symmetric BooleanNegate reverse conversion before same-update
  source-to-target reapplication.
- Second graph-owned converter execution slice:
  `DataConverterTrigger` forward conversion on default-context trigger
  source-to-target bindings, covered by a C++ probe through an existing
  transition-condition consumer.
- Trigger public-update target-to-source slice: main-`ToTarget | TwoWay`
  trigger binds now participate in public `updateDataBinds(true)` for direct
  `DataConverterTrigger`; C++ inherited `reverseConvert` passes the edited
  target value through to the source, then source-to-target reapplication
  increments the bindable trigger target through `DataConverterTrigger::convert`.
- Trigger converter explicit target-to-source slice: main-`ToSource | TwoWay`
  trigger binds now participate in explicit `advanceDataContext()` for direct
  `DataConverterTrigger`; C++ main-to-source conversion increments the edited
  bindable target before writing the default trigger source.
- Trigger source reset reapply slice: trigger source reset from
  `advanceDataContext()` now reapplies reset value `0` to direct and direct
  `DataConverterTrigger` bindable targets on the following state-machine
  advance.
- Trigger converter-group public-update slice: main-`ToTarget | TwoWay`
  trigger binds now cover public `updateDataBinds(true)` for the first trigger
  group shape, `DataConverterGroup<DataConverterTrigger>`, using C++ group
  reverse/forward order around the inherited trigger reverse pass-through and
  forward increment.
- Trigger converter-group explicit target-to-source slice:
  main-`ToSource | TwoWay` trigger binds now cover explicit
  `advanceDataContext()` for the first trigger group shape,
  `DataConverterGroup<DataConverterTrigger>`, using C++ group forward order in
  the main-to-source direction so the trigger converter increments the edited
  target before writing the default trigger source.
- Trigger converter multi-group slice:
  `DataConverterGroup<DataConverterTrigger, DataConverterTrigger>` now covers
  the first multi-child all-trigger group for both public
  `updateDataBinds(true)` main-`ToTarget | TwoWay` and explicit
  `advanceDataContext()` main-`ToSource | TwoWay` trigger binds.
- Boolean public-update observer preservation slice:
  same-path direct boolean observers now have the first non-number dirty-list
  scheduling probe. The dirty main-`ToTarget | TwoWay` boolean bind updates
  the source during public update, while a neighboring ordinary `ToTarget`
  bind reports the new source and preserves its target until the next normal
  state-machine advance.
- String public-update observer preservation slice:
  same-path direct string observers now extend the dirty-list scheduling probe
  to byte-backed string values. The dirty main-`ToTarget | TwoWay` string bind
  updates the source during public update, while a neighboring ordinary
  `ToTarget` bind reports the new source and preserves its target until the
  next normal state-machine advance.
- Color public-update observer preservation slice:
  direct color binds now cover public `updateDataBinds(true)`
  target-to-source and the first same-path observer case. The dirty
  main-`ToTarget | TwoWay` color bind updates the source and its own target
  during public update, while a neighboring ordinary `ToTarget` bind reports
  the new source and preserves its target until the next normal advance.
- Enum public-update observer preservation slice:
  direct enum binds now cover public `updateDataBinds(true)` target-to-source
  and the first same-path observer case. The dirty main-`ToTarget | TwoWay`
  enum bind updates the source and its own target during public update, while
  a neighboring ordinary `ToTarget` bind reports the new source and preserves
  its target until the next normal advance.
- Asset public-update observer preservation slice:
  direct asset binds now cover public `updateDataBinds(true)` target-to-source
  and the first same-path observer case. The dirty main-`ToTarget | TwoWay`
  asset bind updates the source and its own target during public update, while
  a neighboring ordinary `ToTarget` bind reports the new source and preserves
  its target until the next normal advance.
- Artboard public-update observer preservation slice:
  direct artboard binds now cover public `updateDataBinds(true)`
  target-to-source and the first same-path observer case. The dirty
  main-`ToTarget | TwoWay` artboard bind updates the source and its own target
  during public update, while a neighboring ordinary `ToTarget` bind reports
  the new source and preserves its target until the next normal advance.
- Symbol-list-index public-update observer preservation slice:
  direct integer-backed symbol-list-index binds now cover public
  `updateDataBinds(true)` target-to-source and the first same-path observer
  case. The dirty main-`ToTarget | TwoWay` integer bind updates the
  symbol-list-index source and its own target during public update, while a
  neighboring ordinary `ToTarget` bind reports the new source and preserves
  its target until the next normal advance.
- Trigger public-update observer preservation slice:
  direct trigger binds now cover the first same-path observer case for public
  `updateDataBinds(true)`. The dirty main-`ToTarget | TwoWay` trigger bind
  updates the source and its own target during public update, while a
  neighboring ordinary `ToTarget` bind reports the new source and preserves
  its target until the next normal advance.
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
- Artboard list-consumer direct-update boundary: after binding the default
  artboard view-model context, direct `Artboard::updateDataBinds(true)` keeps
  exact `ArtboardComponentList` target list counts at the same empty value
  observed immediately after binding for direct list sources and direct
  `DataConverterNumberToList` sources. The target-count mutation is reserved
  for the public post-bind advance boundary. The contract is
  `docs/prototypes/data-binding-graph-artboard-list-direct-update-boundary-runtime-contract.md`.
- Artboard list-consumer post-bind advance target-count slice: after binding
  the default artboard view-model context, a zero-second public
  `Artboard::advance(0.0f)` updates exact `ArtboardComponentList` target list
  counts for direct list sources and direct `DataConverterNumberToList`
  sources. Rust mirrors that report through
  `ArtboardInstance::advance_artboard_data_binds()` without admitting child
  artboard clone surfaces, item identity reuse/disposal, map-rule-driven child
  creation, layout, or virtualization. The contract is
  `docs/prototypes/data-binding-graph-artboard-list-advance-target-count-runtime-contract.md`.
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
- Operation-view-model converter name-path unsupported boundary: converter
  `sourcePathIds` containing a manifest path id for a secondary `factor`
  property do not use `DataBindContext::resolvePath()` or relative manifest
  lookup in C++; the secondary operand remains missing and the converter falls
  back to operand `0.0`. The contract is
  `docs/prototypes/data-binding-graph-operation-viewmodel-name-path-unsupported-runtime-contract.md`.
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
- Operation-view-model secondary-source mutation slice: direct
  `DataConverterOperationViewModel` now refreshes its secondary operand and
  dirties the dependent primary source when the secondary default view-model
  number is mutated through the state-machine default source API. This mirrors
  C++ `bindFromContext()` registering the owning data bind as a dependent of
  the resolved `ViewModelInstanceNumber`. The contract is
  `docs/prototypes/data-binding-graph-operation-viewmodel-secondary-source-mutation-runtime-contract.md`.
- Operation-view-model group public-update target-to-source slice:
  `DataConverterGroup<OperationValue, OperationViewModel>` now covers public
  `updateDataBinds(true)` for main-`ToTarget | TwoWay` number binds. The
  public update reverse-converts in C++ child order, resolves the
  operation-view-model secondary operand from the imported default root
  view-model instance, writes the primary source, and reapplies
  source-to-target in the same update. The contract is
  `docs/prototypes/data-binding-graph-operation-viewmodel-group-public-update-target-to-source-runtime-contract.md`.
- Operation-view-model group secondary-source mutation slice:
  `DataConverterGroup<OperationValue, OperationViewModel>` now refreshes the
  nested operation-view-model operand and dirties the owning source when the
  secondary default view-model number is mutated through the state-machine
  default source API. The contract is
  `docs/prototypes/data-binding-graph-operation-viewmodel-group-secondary-source-mutation-runtime-contract.md`.
- Operation-view-model group converter name-path unsupported boundary:
  `DataConverterGroup<OperationValue, OperationViewModel>` forwards
  `bindFromContext()` to the operation-view-model child, which still reads
  converter `sourcePathIds` directly through `DataContext` and does not use
  `DataBindContext::resolvePath()` or relative manifest lookup. A manifest
  path id for the secondary `factor` property leaves the grouped secondary
  operand missing with the operand `0.0` fallback. The contract is
  `docs/prototypes/data-binding-graph-operation-viewmodel-group-name-path-unsupported-runtime-contract.md`.
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
- System operation-value group public-update target-to-source slice:
  `DataConverterGroup<OperationValue, System*>` now covers public
  `updateDataBinds(true)` for main-`ToTarget | TwoWay` number binds. The graph
  threads the owning data-bind flags into grouped
  `DataConverterSystemNormalizer` and `DataConverterSystemDegsToRads`
  construction, matching C++ direction-sensitive child conversion before the
  source write and immediate source-to-target reapplication. The contract is
  `docs/prototypes/data-binding-graph-system-operation-value-group-public-update-target-to-source-runtime-contract.md`.
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
- Deterministic `DataConverterFormula` function-token slice:
  default-context number and symbol-list-index sources feeding number targets
  now execute `FormulaTokenFunction` output-queue tokens for deterministic
  function types, using the C++ formula argument counts exposed by
  `rive-binary`. Direct number-source explicit target-to-source and public
  update target-to-source scheduling now run the same function-token formula
  conversion before source writes and same-bind source-to-target reapplication,
  and direct main-`ToTarget | TwoWay` target-dirty scheduling preserves then
  reapplies manual target edits like C++. Random function behavior beyond the
  host-supplied default-mode slice below remains outside the graph until fuller
  random-source state contracts exist.
- First `DataConverterFormula` random function-token slice:
  default-context number sources feeding number targets now execute a direct
  `FunctionType::random` output-queue token when `randomModeValue` is the
  default `0`. Rust exposes a narrow host-supplied graph formula random stream
  and caches the drawn value per formula converter like C++ default random
  mode. The C++ probe derives the first draw from the C++ number-binding report
  and supplies the same value to Rust before advancing.
- `DataConverterFormula` random symbol-list-index source-to-target slice:
  default-context symbol-list-index sources feeding number targets now execute
  a direct `FunctionType::random` output-queue token when `randomModeValue` is
  the default `0`. Rust casts the symbol-list-index source to `f32` before
  formula evaluation, consumes the host-supplied default-mode random value, and
  reuses the cached value on later state-machine advancement like C++. List
  formulas, other non-number random formulas, non-default random modes,
  imported/owned contexts, and real random generation remain follow-up slices.
- `DataConverterFormula` random symbol-list-index always-mode source-to-target
  slice: default-context symbol-list-index sources feeding number targets now
  execute direct `FunctionType::random` output-queue tokens when
  `randomModeValue == 1`. Rust casts the symbol-list-index source to `f32`
  before formula evaluation and consumes a fresh host-supplied random value on
  each source-to-target state-machine advancement like C++. List formulas,
  other non-number random formulas, imported/owned contexts, and real random
  generation remain follow-up slices.
- `DataConverterFormula` random symbol-list-index source-change
  source-to-target slice: default-context symbol-list-index sources feeding
  number targets now execute direct `FunctionType::random` output-queue tokens
  when `randomModeValue == 2`. Rust casts the symbol-list-index source to
  `f32`, caches the host-supplied random like default mode, then clears that
  formula cache when the bound default symbol-list-index source mutates. List
  formulas, other non-number random formulas, imported/owned contexts, and
  real random generation remain follow-up slices.
- `DataConverterFormula` random always-mode source-to-target slice:
  default-context number sources feeding number targets now execute direct
  `FunctionType::random` output-queue tokens when `randomModeValue == 1`.
  Rust draws from the host-supplied graph formula random stream on each
  formula evaluation instead of reusing the default-mode cached random value.
  The C++ probe derives two draws from two number-binding reports and supplies
  both values to Rust before advancing. Broader `RandomMode::always`
  scheduling, real Rust random generation, C++ random seeding/queueing, cache
  invalidation, call counts, list scheduling, and non-number random formulas
  remain outside the graph until fuller random-source state contracts exist.
- `DataConverterFormula` random always-mode explicit target-to-source slice:
  default-context number sources feeding number targets now execute
  main-`ToSource | TwoWay` explicit `advance_data_context` target-to-source
  scheduling when `randomModeValue == 1`. Rust consumes a fresh host-supplied
  random value for the source write, another for same-bind source-to-target
  reapplication, and fresh values on later state-machine advances, matching
  the C++ probe reports. Target-dirty, grouped target-to-source, grouped
  public-update, grouped target-dirty, list, remaining non-number
  `RandomMode::always`, real Rust random generation, C++ random
  seeding/queueing, cache invalidation, and call counts remain follow-up
  slices.
- `DataConverterFormula` random always-mode public update target-to-source
  slice: default-context number sources feeding number targets now execute
  main-`ToTarget | TwoWay` public `update_data_binds_apply_target_to_source`
  scheduling when `randomModeValue == 1`. Rust consumes fresh host-supplied
  random values for the initial source-to-target pass, the public
  target-to-source source write, same-update source-to-target reapplication,
  and later state-machine advances, matching the C++ probe reports.
  Target-dirty, grouped target-to-source, grouped public-update, grouped
  target-dirty, list, remaining non-number `RandomMode::always`, real Rust
  random generation, C++ random seeding/queueing, cache invalidation, and call
  counts remain follow-up slices.
- `DataConverterFormula` random always-mode target-dirty slice:
  default-context number sources feeding number targets now execute
  main-`ToTarget | TwoWay` target-dirty scheduling when `randomModeValue == 1`.
  Rust consumes a host-supplied random value for the initial source-to-target
  pass, preserves a manual target edit through explicit data-context
  advancement, and consumes fresh values on later normal state-machine
  advances, matching C++. Grouped target-to-source, grouped public-update,
  grouped target-dirty, list, remaining non-number `RandomMode::always`, real
  Rust random generation, C++ random seeding/queueing, cache invalidation, and
  call counts remain follow-up slices.
- `DataConverterFormula` random source-change source-to-target slice:
  default-context number sources feeding number targets now execute direct
  `FunctionType::random` output-queue tokens when `randomModeValue == 2`.
  Rust caches the host-supplied random value like the default mode, but clears
  that cache when `set_default_view_model_number_source_for_data_bind` mutates
  the bound default source. The C++ probe derives the pre-change and
  post-change draws from number-binding reports and supplies both values to
  Rust before advancing. Broader `RandomMode::sourceChange` scheduling,
  secondary converter dependency invalidation, real Rust random generation,
  C++ random seeding/queueing, call counts, list scheduling, and non-number
  random formulas remain outside the graph until fuller random-source state
  contracts exist.
- `DataConverterFormula` random source-change target-to-source slice:
  default-context number sources feeding number targets now execute
  main-`ToSource | TwoWay` explicit `advance_data_context` target-to-source
  scheduling when `randomModeValue == 2`. Rust consumes a host-supplied random
  value for the target-to-source source write, clears that source's formula
  random cache when the graph source changes, and consumes a fresh value for
  same-pass source-to-target reapplication, matching C++. Public update,
  target-dirty, grouped/list/non-number `RandomMode::sourceChange`, secondary
  converter dependency invalidation, real Rust random generation, C++ random
  seeding/queueing, and call counts remain follow-up slices.
- `DataConverterFormula` random source-change public-update target-to-source
  slice: default-context number sources feeding number targets now execute
  main-`ToTarget | TwoWay` public `update_data_binds_apply_target_to_source`
  scheduling when `randomModeValue == 2`. Rust warms the source-change random
  cache during initial source-to-target application, reuses that cached value
  for the public target-to-source source write, clears the formula random
  cache when the graph source changes, and consumes a fresh value for
  same-update source-to-target reapplication, matching C++. Target-dirty,
  grouped/list/non-number `RandomMode::sourceChange`, secondary converter
  dependency invalidation, real Rust random generation, C++ random
  seeding/queueing, and call counts remain follow-up slices.
- `DataConverterFormula` random source-change target-dirty slice:
  default-context number sources feeding number targets now execute
  main-`ToTarget | TwoWay` target-dirty scheduling when `randomModeValue == 2`.
  Rust consumes a host-supplied random value for the initial source-to-target
  pass, preserves a manual target edit through explicit data-context
  advancement without treating the target edit as a source change, and reuses
  the cached value on later normal state-machine advances, matching C++.
  Grouped/list/non-number `RandomMode::sourceChange`, secondary converter
  dependency invalidation, real Rust random generation, C++ random
  seeding/queueing, and call counts remain follow-up slices.
- `DataConverterFormula` random target-to-source slice:
  default-context number sources feeding number targets now reuse the
  host-supplied default-mode formula random cache through direct
  target-to-source scheduling. The C++ probe covers explicit
  `advancedDataContext()` for a main-`ToSource | TwoWay` random formula bind
  with a direct observer, and public `updateDataBinds(true)` for a
  main-`ToTarget | TwoWay` random formula bind. List formulas, non-number,
  non-default random modes, cache invalidation, call counts, imported/owned
  contexts, and real random generation remain follow-up slices.
- `DataConverterFormula` random target-dirty slice:
  direct main-`ToTarget | TwoWay` random formula binds now preserve a manual
  target edit through explicit data-context advancement, then reapply the
  unchanged source through the cached formula random value on the next normal
  state-machine advance. List formulas, non-number, non-default random modes,
  cache invalidation, call counts, imported/owned contexts, and real random
  generation remain follow-up slices.
- `DataConverterFormula` random group source-to-target slice:
  default-context number sources feeding number targets through
  `DataConverterGroup<OperationValue, Formula(random)>` now thread the
  host-supplied default-mode random stream through nested group converter
  state. The grouped formula draws the C++-derived random value on first
  source-to-target advancement and reuses the cached value on later
  state-machine advancement. List formulas, non-number, non-default random
  modes, cache invalidation, call counts, imported/owned contexts, and real
  random generation remain follow-up slices.
- `DataConverterFormula` random group always-mode source-to-target slice:
  default-context number sources feeding number targets through
  `DataConverterGroup<OperationValue, Formula(random)>` now thread
  `randomModeValue == 1` through nested group converter state. The grouped
  formula consumes a fresh C++-derived random value on each source-to-target
  state-machine advancement. List formulas, non-number, non-default grouped
  target-to-source/public update/target-dirty scheduling, cache invalidation,
  call counts, imported/owned contexts, and real random generation remain
  follow-up slices.
- `DataConverterFormula` random group always-mode target-to-source slice:
  default-context number sources feeding number targets through a
  main-`ToSource | TwoWay` `DataConverterGroup<OperationValue,
  Formula(random)>` now apply explicit target mutations through forward group
  order when `randomModeValue == 1`. Rust consumes fresh host-supplied random
  values for the target-to-source source write, same-pass source-to-target
  reapplication, and later state-machine advances, matching C++. Grouped
  public-update, grouped target-dirty, source-change grouped
  target-to-source/public-update/target-dirty, list formulas, non-number,
  cache invalidation, call counts, imported/owned contexts, and real random
  generation remain follow-up slices.
- `DataConverterFormula` random group always-mode public-update
  target-to-source slice: default-context number sources feeding number targets
  through a main-`ToTarget | TwoWay` `DataConverterGroup<OperationValue,
  Formula(random)>` now apply public target mutations through reverse group
  order when `randomModeValue == 1`. Rust consumes fresh host-supplied random
  values for initial source-to-target application, the public target-to-source
  source write, same-update source-to-target reapplication, and later
  state-machine advances, matching C++. Grouped target-dirty, source-change
  grouped target-to-source/public-update/target-dirty, list formulas,
  non-number, cache invalidation, call counts, imported/owned contexts, and
  real random generation remain follow-up slices.
- `DataConverterFormula` random group always-mode target-dirty slice:
  default-context number sources feeding number targets through a
  main-`ToTarget | TwoWay` `DataConverterGroup<OperationValue,
  Formula(random)>` now preserve a manual target edit through explicit
  data-context advancement when `randomModeValue == 1`, then consume fresh
  host-supplied random values on later normal state-machine advances,
  matching C++. Source-change grouped target-to-source/public-update/
  target-dirty, list formulas, non-number, cache invalidation, call counts,
  imported/owned contexts, and real random generation remain follow-up slices.
- `DataConverterFormula` random group source-change source-to-target slice:
  default-context number sources feeding number targets through
  `DataConverterGroup<OperationValue, Formula(random)>` now thread
  `randomModeValue == 2` through nested group converter state. The grouped
  formula caches the C++-derived random value like default mode, then clears
  that nested formula cache when the bound default source mutates. List
  formulas, non-number, non-default grouped target-to-source/public
  update/target-dirty scheduling, secondary converter dependency invalidation,
  cache invalidation, call counts, imported/owned contexts, and real random
  generation remain follow-up slices.
- `DataConverterFormula` random group source-change target-to-source slice:
  default-context number sources feeding number targets through a
  main-`ToSource | TwoWay` `DataConverterGroup<OperationValue,
  Formula(random)>` now apply explicit target mutations through forward group
  order when `randomModeValue == 2`. Rust consumes a host-supplied random
  value for the target-to-source source write, clears the nested formula
  random cache when the graph source changes, and consumes a fresh value for
  same-pass source-to-target reapplication, matching C++. Grouped
  public-update, grouped target-dirty, list formulas, non-number, secondary
  converter dependency invalidation, cache invalidation, call counts,
  imported/owned contexts, and real random generation remain follow-up slices.
- `DataConverterFormula` random group source-change public-update
  target-to-source slice: default-context number sources feeding number targets
  through a main-`ToTarget | TwoWay` `DataConverterGroup<OperationValue,
  Formula(random)>` now apply public target mutations through reverse group
  order when `randomModeValue == 2`. Rust warms the nested source-change random
  cache during initial source-to-target application, reuses that cached value
  for the public target-to-source source write, clears the nested formula
  random cache when the graph source changes, and consumes a fresh value for
  same-update source-to-target reapplication, matching C++. Grouped
  target-dirty, list formulas, non-number, secondary converter dependency
  invalidation, cache invalidation, call counts, imported/owned contexts, and
  real random generation remain follow-up slices.
- `DataConverterFormula` random group public-update target-to-source slice:
  default-context number sources feeding number targets through a
  main-`ToTarget | TwoWay` `DataConverterGroup<OperationValue,
  Formula(random)>` now apply public `updateDataBinds(true)` target mutations
  through reverse group order, reuse the cached default-mode formula random
  value, and reapply source-to-target in the same update. List formulas,
  non-number, non-default random modes, cache invalidation, call counts,
  imported/owned contexts, and real random generation remain follow-up slices.
- `DataConverterFormula` random group target-dirty slice:
  direct main-`ToTarget | TwoWay` grouped random formula binds now preserve a
  manual target edit through explicit data-context advancement, then reapply
  the unchanged source through the cached grouped formula random value on the
  next normal state-machine advance. List formulas, non-number, non-default
  random modes, cache invalidation, call counts, imported/owned contexts, and
  real random generation remain follow-up slices.
- `DataConverterFormula` random group explicit target-to-source slice:
  default-context number sources feeding number targets through a
  main-`ToSource | TwoWay` `DataConverterGroup<OperationValue,
  Formula(random)>` now apply target mutations through forward group order
  during explicit data-context advancement. A neighboring direct number bind
  observes the grouped source write on subsequent state-machine advancement.
  List formulas, non-number, non-default random modes, cache invalidation,
  call counts, imported/owned contexts, and real random generation remain
  follow-up slices.
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
- View-model public-update observer application slice: direct default-context
  `ViewModelInstanceViewModel.propertyValue` sources feeding
  `BindablePropertyViewModel.propertyValue` targets now honor
  `ToTarget | TwoWay` target mutation on public `updateDataBinds(true)`.
  C++ applies same-path ordinary `ToTarget` view-model observer targets during
  the same public update, so Rust marks changed same-path pointer sources for
  immediate source-to-target application.
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
- Default source-mutation same-path observer slice:
  default-context number source mutation by state-machine data-bind index now
  mutates every same-path number source node rather than only the selected
  cloned edge, so a neighboring ordinary direct `ToTarget` number bind reports
  the updated source and applies the updated target on the next state-machine
  advance.
- Boolean source-mutation same-path observer slice:
  default-context boolean source mutation by state-machine data-bind index now
  mutates every same-path boolean source node rather than only the selected
  cloned edge, so a neighboring ordinary direct `ToTarget` boolean bind
  reports the updated source and applies the updated target on the next
  state-machine advance.
- String source-mutation same-path observer slice:
  default-context string source mutation by state-machine data-bind index now
  mutates every same-path string source node rather than only the selected
  cloned edge, so a neighboring ordinary direct `ToTarget` string bind reports
  the updated source bytes and applies the updated target on the next
  state-machine advance.
- Color source-mutation same-path observer slice:
  default-context color source mutation by state-machine data-bind index now
  mutates every same-path color source node rather than only the selected
  cloned edge, so a neighboring ordinary direct `ToTarget` color bind reports
  the updated source and applies the updated target on the next state-machine
  advance.
- Enum source-mutation same-path observer slice:
  default-context enum source mutation by state-machine data-bind index now
  mutates every same-path enum source node rather than only the selected
  cloned edge, so a neighboring ordinary direct `ToTarget` enum bind reports
  the updated raw enum index and applies the updated target on the next
  state-machine advance.
- Symbol-list-index source-mutation same-path observer slice:
  default-context symbol-list-index source mutation by state-machine data-bind
  index now mutates every same-path symbol-list-index source node rather than
  only the selected cloned edge, so a neighboring ordinary direct `ToTarget`
  integer bind reports the updated source and applies the updated target on
  the next state-machine advance.
- Asset source-mutation same-path observer slice:
  default-context asset source mutation by state-machine data-bind index now
  mutates every same-path asset source node rather than only the selected
  cloned edge, so a neighboring ordinary direct `ToTarget` asset bind reports
  the updated source and applies the updated target on the next state-machine
  advance.
- Artboard source-mutation same-path observer slice:
  default-context artboard source mutation by state-machine data-bind index
  now mutates every same-path artboard source node rather than only the
  selected cloned edge, so a neighboring ordinary direct `ToTarget` artboard
  bind reports the updated source and applies the updated target on the next
  state-machine advance.
- Trigger source-mutation same-path observer slice:
  default-context trigger source mutation by state-machine data-bind index now
  mutates every same-path trigger source node rather than only the selected
  cloned edge, so a neighboring ordinary direct `ToTarget` trigger bind
  reports the updated source and applies the updated target on the next
  state-machine advance.
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
- Formula group public-update target-to-source slice: the admitted
  `DataConverterOperationValue -> DataConverterFormula` group now covers
  public `updateDataBinds(true)` for main-`ToTarget | TwoWay` number binds.
  The public update follows C++ reverse group order through deterministic
  formula conversion before writing the source, then reapplies forward group
  conversion in the same update.
- First exact number target-to-source direction slice: default-context number
  sources now follow C++ main-direction converter dispatch for the mutating
  bind, with exact C++ probe reports for direct, `OperationValue`, and
  `OperationValue` group source/target values. Main `ToSource` cases use
  `convert` and forward group order; main `ToTarget | TwoWay`
  `OperationValue` and `OperationValue` group target edits are now pinned as
  delayed source-to-target dirty updates rather than immediate reverse
  conversion. Broader dirty-list target scheduling remains a follow-up slice.
- Direct OperationValue main-to-target same-path observer slice: the admitted
  direct `DataConverterOperationValue` main-`ToTarget | TwoWay` number bind now
  compares a neighboring ordinary direct `ToTarget` observer on the same source
  path. Explicit `advanceDataContext()` preserves the mutating target edit and
  leaves the observer on the unchanged source/target values; the following
  normal state-machine advances continue matching C++ for both binds.
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
- Non-scripting scripted converter pass-through slice:
  `ScriptedDataConverter` now participates in the runtime graph as inherited
  C++ base converter pass-through for a default-context number bind. Normal
  source-to-target application and public `updateDataBinds(true)`
  target-to-source application both leave the value unchanged, matching the
  non-scripting C++ probe build.
- Operation base converter pass-through slice:
  concrete `DataConverterOperation` now participates in the runtime graph as
  inherited C++ base converter pass-through for a default-context number bind.
  Normal source-to-target application and public `updateDataBinds(true)`
  target-to-source application both leave the value unchanged, matching C++.
- Trigger public-update target-to-source slice:
  `DataConverterTrigger` now covers the public `updateDataBinds(true)`
  target-to-source path for a default-context trigger bind. The C++ probe now
  reports trigger binding source/target rows, and Rust mirrors inherited
  `reverseConvert` pass-through plus same-update source-to-target increment.
- Trigger converter explicit target-to-source slice:
  `DataConverterTrigger` now also covers explicit `advanceDataContext()`
  target-to-source behavior for a main-`ToSource | TwoWay` default-context
  trigger bind. Rust mirrors C++ main-to-source conversion of the edited
  bindable target before writing the default trigger source, with trigger
  binding reports compared after the explicit data-context actions.
- Trigger source reset reapply slice:
  `advanceDataContext()` trigger source reset now reapplies to default-context
  trigger bindable targets on the following state-machine advance. Rust marks
  changed trigger sources for source-to-target reapplication, clears stale
  reset reapply state when a later explicit target edit is consumed first, and
  mirrors C++'s non-main converter direction for direct `DataConverterTrigger`
  main-`ToSource | TwoWay` source-to-target application.
- Trigger converter-group public-update slice:
  `DataConverterGroup<DataConverterTrigger>` now covers public
  `updateDataBinds(true)` target-to-source behavior for default-context
  trigger binds. The C++ probe reports trigger binding source/target rows, and
  Rust mirrors group reverse order for target-to-source pass-through plus group
  forward order for source-to-target increment.
- Trigger converter-group explicit target-to-source slice:
  `DataConverterGroup<DataConverterTrigger>` now also covers explicit
  `advanceDataContext()` target-to-source behavior for default-context trigger
  binds. The C++ probe reports trigger binding source/target rows, and Rust
  mirrors group forward order in the main-to-source direction before writing
  the default trigger source.
- Trigger converter multi-group slice:
  `DataConverterGroup<DataConverterTrigger, DataConverterTrigger>` now covers
  the first multi-child all-trigger group shape. Public update mirrors C++
  reverse-order target-to-source pass-through plus same-update forward
  reapplication through two trigger increments, and explicit
  `advanceDataContext()` mirrors forward-order main-to-source conversion
  before writing the default trigger source.
- Boolean public-update target-to-source slice:
  direct boolean and `DataConverterBooleanNegate` now cover public
  `updateDataBinds(true)` target-to-source behavior for default-context
  boolean binds. The C++ probe reports boolean binding source/target rows, and
  Rust mirrors direct writes plus symmetric BooleanNegate reverse conversion.
- Boolean public-update observer preservation slice:
  a neighboring ordinary direct `ToTarget` boolean bind now follows the C++
  same-path observer ordering already admitted for numbers: source row updates
  during public update, observer target stays at its previous value until the
  next normal state-machine advance.
- String public-update observer preservation slice:
  a neighboring ordinary direct `ToTarget` string bind now follows the same
  C++ public-update observer ordering, preserving its previous target bytes
  during public update while reporting the new source bytes.
- Color public-update observer preservation slice:
  direct color public update now writes the default color source from a
  main-`ToTarget | TwoWay` target edit and uses the same C++ observer ordering
  as number/boolean/string for a neighboring ordinary direct `ToTarget` bind.
- Enum public-update observer preservation slice:
  direct enum public update now writes the default enum source from a
  main-`ToTarget | TwoWay` target edit and uses the same C++ observer ordering
  as number/boolean/string/color for a neighboring ordinary direct `ToTarget`
  bind.
- Asset public-update observer preservation slice:
  direct asset public update now writes the default asset source from a
  main-`ToTarget | TwoWay` target edit and uses the same C++ observer ordering
  as number/boolean/string/color/enum for a neighboring ordinary direct
  `ToTarget` bind.
- Artboard public-update observer preservation slice:
  direct artboard public update now writes the default artboard source from a
  main-`ToTarget | TwoWay` target edit and uses the same C++ observer ordering
  as number/boolean/string/color/enum/asset for a neighboring ordinary direct
  `ToTarget` bind.
- Symbol-list-index public-update observer preservation slice:
  direct symbol-list-index public update now writes the default
  symbol-list-index source from a main-`ToTarget | TwoWay` integer target edit
  and uses the same C++ observer ordering as the other direct value
  public-update lanes for a neighboring ordinary direct `ToTarget` bind.
- Trigger public-update observer preservation slice:
  direct trigger public update now uses the same C++ observer ordering as the
  other direct value public-update lanes for a neighboring ordinary direct
  `ToTarget` trigger bind.
- View-model public-update observer application slice:
  direct view-model public update now writes the default view-model pointer
  source from a main-`ToTarget | TwoWay` target edit and follows C++'s
  pointer-specific behavior where a neighboring ordinary direct `ToTarget`
  view-model observer updates its target during the same public update.
- View-model main-to-target same-path observer slice:
  direct view-model pointer target edits through the state-machine
  bindable-property action path now follow C++'s delayed pointer target
  application. Explicit `advanceDataContext()` preserves the mutating target
  edit, keeps the shared source unchanged, and leaves the same-path ordinary
  `ToTarget` observer target unapplied; the next normal state-machine advance
  reapplies source-to-target for both pointer binds.
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
  instance index through the live cached-reference replacement path. The graph
  now updates same-path view-model pointer sources, and the C++ probe covers
  the matching cached-reference replacement path through an existing
  view-model pointer transition-condition consumer plus a neighboring ordinary
  direct `ToTarget` observer after normal state-machine advancement. The
  contract is
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
- Owned view-model imported-intermediate number name-path mutation unsupported
  boundary: replacing a generated root child with an imported child by
  instance index still lets the graph read `[child, amount]`, but attempting
  `ViewModelInstanceRuntime::propertyNumber("child/amount")->value(...)` from
  the owned root leaves the imported child's existing number selected in C++.
  Rust keeps `set_number_by_property_name_path("child/amount", value)`
  generated-only and returns `false` once `child` is imported. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-number-name-path-unsupported-runtime-contract.md`.
- Owned view-model imported-intermediate boolean source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, enabled]` through the
  imported child's existing `ViewModelInstanceBoolean.propertyValue`. Rust
  records imported boolean snapshots per referenced view-model instance for
  read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-boolean-runtime-contract.md`.
- Owned view-model imported-intermediate boolean name-path mutation unsupported
  boundary: replacing a generated root child with an imported child by
  instance index still lets the graph read `[child, enabled]`, but attempting
  `ViewModelInstanceRuntime::propertyBoolean("child/enabled")->value(...)`
  from the owned root leaves the imported child's existing boolean selected in
  C++. Rust keeps
  `set_boolean_by_property_name_path("child/enabled", value)` generated-only
  and returns `false` once `child` is imported. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-boolean-name-path-unsupported-runtime-contract.md`.
- Owned view-model imported-intermediate string source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, label]` through the
  imported child's existing `ViewModelInstanceString.propertyValue`. Rust
  records imported string snapshots per referenced view-model instance for
  read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-string-runtime-contract.md`.
- Owned view-model imported-intermediate string name-path mutation unsupported
  boundary: replacing a generated root child with an imported child by
  instance index still lets the graph read `[child, label]`, but attempting
  `ViewModelInstanceRuntime::propertyString("child/label")->value(...)` from
  the owned root leaves the imported child's existing string selected in C++.
  Rust keeps `set_string_by_property_name_path("child/label", value)`
  generated-only and returns `false` once `child` is imported. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-string-name-path-unsupported-runtime-contract.md`.
- Owned view-model imported-intermediate color source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, tint]` through the
  imported child's existing `ViewModelInstanceColor.propertyValue`. Rust
  records imported color snapshots per referenced view-model instance for
  read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-color-runtime-contract.md`.
- Owned view-model imported-intermediate color name-path mutation unsupported
  boundary: replacing a generated root child with an imported child by
  instance index still lets the graph read `[child, tint]`, but attempting
  `ViewModelInstanceRuntime::propertyColor("child/tint")->value(...)` from the
  owned root leaves the imported child's existing color selected in C++. Rust
  keeps `set_color_by_property_name_path("child/tint", value)` generated-only
  and returns `false` once `child` is imported. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-color-name-path-unsupported-runtime-contract.md`.
- Owned view-model imported-intermediate enum source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, choice]` through the
  imported child's existing `ViewModelInstanceEnum.propertyValue` index. Rust
  records imported enum-index snapshots per referenced view-model instance for
  read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-enum-runtime-contract.md`.
- Owned view-model imported-intermediate enum name-path mutation unsupported
  boundary: replacing a generated root child with an imported child by
  instance index still lets the graph read `[child, choice]`, but attempting
  `ViewModelInstanceRuntime::propertyEnum("child/choice")->valueIndex(...)`
  from the owned root leaves the imported child's existing enum selected in
  C++. Rust keeps
  `set_enum_by_property_name_path("child/choice", value)` generated-only and
  returns `false` once `child` is imported. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-enum-name-path-unsupported-runtime-contract.md`.
- Owned view-model imported-intermediate symbol-list-index source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, symbol]` through the
  imported child's existing `ViewModelInstanceSymbolListIndex.propertyValue`.
  Rust records imported symbol-list-index snapshots per referenced view-model
  instance for read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-symbol-list-index-runtime-contract.md`.
- Owned view-model imported-intermediate symbol-list-index name-path mutation
  unsupported boundary: replacing a generated root child with an imported
  child by instance index still lets the graph read `[child, symbol]`, but
  attempting to resolve the owner with
  `ViewModelInstanceRuntime::propertyViewModel("child")` and write
  `ViewModelInstanceSymbolListIndex.propertyValue` leaves the imported child's
  existing symbol-list-index selected in C++. Rust keeps
  `set_symbol_list_index_by_property_name_path("child/symbol", value)`
  generated-only and returns `false` once `child` is imported. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-symbol-list-index-name-path-unsupported-runtime-contract.md`.
- Owned view-model imported-intermediate asset source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, image]` through the
  imported child's existing `ViewModelInstanceAssetImage.propertyValue`. Rust
  records imported asset-id snapshots per referenced view-model instance for
  read-only graph binding. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-asset-runtime-contract.md`.
- Owned view-model imported-intermediate asset name-path mutation unsupported
  boundary: replacing a generated root child with an imported child by
  instance index still lets the graph read `[child, image]`, but attempting to
  resolve the owner with `ViewModelInstanceRuntime::propertyViewModel("child")`
  and write `ViewModelInstanceAssetImage.propertyValue` leaves the imported
  child's existing asset selected in C++. Rust keeps
  `set_asset_by_property_name_path("child/image", value)` generated-only and
  returns `false` once `child` is imported. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-asset-name-path-unsupported-runtime-contract.md`.
- Owned view-model imported-intermediate artboard source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, scene]` through the
  imported child's existing `ViewModelInstanceArtboard.propertyValue`. Rust
  records imported artboard-id snapshots per referenced view-model instance
  for read-only graph binding. The C++ probe observes this through a
  transition condition that remains unsatisfied after the imported source
  updates the bindable away from the forced target value. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-artboard-runtime-contract.md`.
- Owned view-model imported-intermediate artboard name-path mutation
  unsupported boundary: replacing a generated root child with an imported
  child by instance index still lets the graph read `[child, scene]`, but
  attempting to resolve the owner with
  `ViewModelInstanceRuntime::propertyViewModel("child")` and write
  `ViewModelInstanceArtboard.propertyValue` leaves the imported child's
  existing artboard selected in C++. Rust keeps
  `set_artboard_by_property_name_path("child/scene", value)` generated-only
  and returns `false` once `child` is imported. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-artboard-name-path-unsupported-runtime-contract.md`.
- Owned view-model imported-intermediate trigger source slice:
  replacing a generated root child with an imported child by instance index
  lets `RuntimeOwnedViewModelInstance` resolve `[child, fire]` through the
  imported child's existing `ViewModelInstanceTrigger.propertyValue`. Rust
  records imported trigger-count snapshots per referenced view-model instance
  for read-only graph binding. The C++ probe observes this through a
  transition condition that remains unsatisfied after the imported source
  updates the bindable away from the authored target value. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-trigger-runtime-contract.md`.
- Owned view-model imported-intermediate trigger name-path mutation
  unsupported boundary: replacing a generated root child with an imported
  child by instance index still lets the graph read `[child, fire]`, but
  attempting to resolve the owner with
  `ViewModelInstanceRuntime::propertyViewModel("child")` and write
  `ViewModelInstanceTrigger.propertyValue` leaves the imported child's
  existing trigger count selected in C++. Rust keeps
  `set_trigger_by_property_name_path("child/fire", value)` generated-only and
  returns `false` once `child` is imported. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-trigger-name-path-unsupported-runtime-contract.md`.
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
- Owned view-model imported-intermediate list name-path mutation unsupported
  boundary: replacing a generated root child with an imported child by
  instance index still lets the graph read `[child, items]`, but attempting to
  append blank item runtimes through
  `ViewModelInstanceRuntime::propertyList("child/items")` leaves the imported
  child's existing list size selected in C++. Rust keeps
  `set_list_item_count_by_property_name_path("child/items", count)`
  generated-only and returns `false` once `child` is imported. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-list-name-path-unsupported-runtime-contract.md`.
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
- Owned generated nested view-model pointer source-handle slice:
  generated owned view-model children can now expose a stable public
  view-model pointer source handle. `RuntimeOwnedViewModelInstance` can resolve
  `child/middle/leaf` into `RuntimeOwnedViewModelViewModelSourceHandle`
  through `view_model_source_handle_by_property_name_path`, and
  `set_view_model_by_source_handle` relinks the same generated-child pointer
  storage before binding. The C++ probe compares against the existing owned
  generated view-model name-path command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-viewmodel-source-handle-runtime-contract.md`.
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
- Imported nested number property-name path slice:
  `RuntimeImportedViewModelInstanceContext::set_number_by_property_name_path`
  resolves a slash-separated path such as `child/amount` through one nested
  `ViewModelPropertyViewModel` segment to a `ViewModelPropertyNumber` leaf and
  records the override by the existing graph source path. The C++ probe calls
  `ViewModelInstanceRuntime::propertyNumber("child/amount")` after completing
  view-model properties and proves two authored state machines bound through
  the same imported context observe the mutation. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-number-name-path-runtime-contract.md`.
- First imported number source handle slice:
  `RuntimeImportedViewModelInstanceContext` can now resolve a root or nested
  number source into `RuntimeImportedViewModelNumberSourceHandle` and mutate
  through that handle only when it belongs to the same imported view-model
  instance context. The C++ probe compares the handle write against
  `ViewModelInstanceRuntime::propertyNumber(name)` and proves two authored
  state machines bound through the same imported context observe the mutation.
  The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-number-source-handle-runtime-contract.md`.
- Imported nested boolean property-name path slice:
  `RuntimeImportedViewModelInstanceContext::set_boolean_by_property_name_path`
  resolves a slash-separated path such as `child/enabled` through one nested
  `ViewModelPropertyViewModel` segment to a `ViewModelPropertyBoolean` leaf and
  records the override by the existing graph source path. The C++ probe calls
  `ViewModelInstanceRuntime::propertyBoolean("child/enabled")` after
  completing view-model properties and proves two authored state machines bound
  through the same imported context observe the mutation. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-boolean-name-path-runtime-contract.md`.
- Imported boolean source handle slice:
  `RuntimeImportedViewModelInstanceContext` can now resolve a root or nested
  boolean source into `RuntimeImportedViewModelBooleanSourceHandle` and mutate
  through that handle only when it belongs to the same imported view-model
  instance context. The C++ probe compares the handle write against
  `ViewModelInstanceRuntime::propertyBoolean(name)` and proves two authored
  state machines bound through the same imported context observe the mutation.
  The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-boolean-source-handle-runtime-contract.md`.
- Imported nested string property-name path slice:
  `RuntimeImportedViewModelInstanceContext::set_string_by_property_name_path`
  resolves a slash-separated path such as `child/label` through one nested
  `ViewModelPropertyViewModel` segment to a `ViewModelPropertyString` leaf and
  records the byte override by the existing graph source path. The C++ probe
  calls `ViewModelInstanceRuntime::propertyString("child/label")` after
  completing view-model properties and proves two authored state machines bound
  through the same imported context observe the mutation. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-string-name-path-runtime-contract.md`.
- Imported string source handle slice:
  `RuntimeImportedViewModelInstanceContext` can now resolve a root or nested
  string source into `RuntimeImportedViewModelStringSourceHandle` and mutate
  through that handle only when it belongs to the same imported view-model
  instance context. The C++ probe compares the handle write against
  `ViewModelInstanceRuntime::propertyString(name)` and proves two authored
  state machines bound through the same imported context observe the source
  bytes. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-string-source-handle-runtime-contract.md`.
- Imported nested color property-name path slice:
  `RuntimeImportedViewModelInstanceContext::set_color_by_property_name_path`
  resolves a slash-separated path such as `child/tint` through one nested
  `ViewModelPropertyViewModel` segment to a `ViewModelPropertyColor` leaf and
  records the color override by the existing graph source path. The C++ probe
  calls `ViewModelInstanceRuntime::propertyColor("child/tint")` after
  completing view-model properties and proves two authored state machines bound
  through the same imported context observe the mutation. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-color-name-path-runtime-contract.md`.
- Imported color source handle slice:
  `RuntimeImportedViewModelInstanceContext` can now resolve a root or nested
  color source into `RuntimeImportedViewModelColorSourceHandle` and mutate
  through that handle only when it belongs to the same imported view-model
  instance context. The C++ probe compares the handle write against
  `ViewModelInstanceRuntime::propertyColor(name)` and proves two authored
  state machines bound through the same imported context observe the color
  source value. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-color-source-handle-runtime-contract.md`.
- Imported nested enum property-name path slice:
  `RuntimeImportedViewModelInstanceContext::set_enum_by_property_name_path`
  resolves a slash-separated path such as `child/choice` through one nested
  `ViewModelPropertyViewModel` segment to an enum leaf and records the enum
  value-index override by the existing graph source path. The C++ probe calls
  `ViewModelInstanceRuntime::propertyEnum("child/choice")` after completing
  view-model properties and proves two authored state machines bound through
  the same imported context observe the mutation. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-enum-name-path-runtime-contract.md`.
- Imported enum source handle slice:
  `RuntimeImportedViewModelInstanceContext` can now resolve a root or nested
  enum source into `RuntimeImportedViewModelEnumSourceHandle` and mutate
  through that handle only when it belongs to the same imported view-model
  instance context. The C++ probe compares the handle write against
  `ViewModelInstanceRuntime::propertyEnum(name)` and proves two authored state
  machines bound through the same imported context observe the enum value
  index. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-enum-source-handle-runtime-contract.md`.
- Imported nested symbol-list-index property-name path slice:
  `RuntimeImportedViewModelInstanceContext::
  set_symbol_list_index_by_property_name_path` resolves a slash-separated path
  such as `child/symbol` through one nested `ViewModelPropertyViewModel`
  segment to a `ViewModelPropertySymbolListIndex` leaf and records the index
  override by the existing graph source path. The C++ probe calls
  `ViewModelInstanceRuntime::propertySymbolListIndex("child/symbol")` after
  completing view-model properties and proves two authored state machines bound
  through the same imported context observe the mutation. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-symbol-list-index-name-path-runtime-contract.md`.
- Imported symbol-list-index source handle slice:
  `RuntimeImportedViewModelInstanceContext` can now resolve a root or nested
  symbol-list-index source into
  `RuntimeImportedViewModelSymbolListIndexSourceHandle` and mutate through
  that handle only when it belongs to the same imported view-model instance
  context. The C++ probe compares the handle write against
  `ViewModelInstanceRuntime::propertySymbolListIndex(name)` and proves two
  authored state machines bound through the same imported context observe the
  symbol index value. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-symbol-list-index-source-handle-runtime-contract.md`.
- Imported nested asset property-name path boundary:
  `RuntimeImportedViewModelInstanceContext::set_asset_by_property_name_path`
  returns `false` for a slash-separated path such as `child/image`, matching
  the C++ probe's root `ViewModelInstance::propertyValue(name)` asset-image
  lookup, which does not resolve slash paths even after completing view-model
  properties. The C++ probe proves two authored state machines bound through
  the same imported context keep the original asset source. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-asset-name-path-unsupported-runtime-contract.md`.
- Imported asset source handle slice:
  `RuntimeImportedViewModelInstanceContext` can now resolve a root asset source
  into `RuntimeImportedViewModelAssetSourceHandle` and mutate through that
  handle only when it belongs to the same imported view-model instance context.
  Slash-path handle lookup remains unresolved, matching the nested asset
  boundary. The C++ probe compares the handle write against the root asset
  by-name mutation path and proves two authored state machines bound through
  the same imported context observe the asset index. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-asset-source-handle-runtime-contract.md`.
- Imported nested artboard property-name path boundary:
  `RuntimeImportedViewModelInstanceContext::set_artboard_by_property_name_path`
  returns `false` for a slash-separated path such as `child/scene`, matching
  the C++ probe's root `ViewModelInstance::propertyValue(name)` artboard
  lookup, which does not resolve slash paths even after completing view-model
  properties. The C++ probe proves two authored state machines bound through
  the same imported context keep the original artboard source. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-artboard-name-path-unsupported-runtime-contract.md`.
- Imported artboard source handle slice:
  `RuntimeImportedViewModelInstanceContext` can now resolve a root artboard
  source into `RuntimeImportedViewModelArtboardSourceHandle` and mutate through
  that handle only when it belongs to the same imported view-model instance
  context. Slash-path handle lookup remains unresolved, matching the nested
  artboard boundary. The C++ probe compares the handle write against the root
  artboard by-name mutation path and proves two authored state machines bound
  through the same imported context observe the artboard index. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-artboard-source-handle-runtime-contract.md`.
- Imported nested trigger property-name path boundary:
  `RuntimeImportedViewModelInstanceContext::set_trigger_by_property_name_path`
  returns `false` for a slash-separated path such as `child/fire`, matching
  the C++ probe's root `ViewModelInstance::propertyValue(name)` trigger lookup,
  which does not resolve slash paths even after completing view-model
  properties. The C++ probe proves the bound state machine keeps the original
  trigger source. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-trigger-name-path-unsupported-runtime-contract.md`.
- Imported trigger source handle slice:
  `RuntimeImportedViewModelInstanceContext` can now resolve a root trigger
  source into `RuntimeImportedViewModelTriggerSourceHandle` and mutate through
  that handle only when it belongs to the same imported view-model instance
  context. Slash-path handle lookup remains unresolved, matching the nested
  trigger boundary. The C++ probe compares the handle write against the root
  trigger by-name mutation path and proves a later-bound state machine follows
  the same admitted post-bind behavior. Trigger binding/source-count report
  parity, listener notification, and event dispatch remain out of scope. The
  contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-trigger-source-handle-runtime-contract.md`.
- Imported nested list property-name path boundary:
  `RuntimeImportedViewModelInstanceContext::
  set_list_item_count_by_property_name_path` returns `false` for a
  slash-separated path such as `child/items`, matching the current safe
  boundary after the C++ nested list-by-name mutation probe crashes for the
  synthetic imported-context fixture. The C++ baseline still proves the bound
  state machine observes the original nested list source. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-list-name-path-unsupported-runtime-contract.md`.
- Imported list source handle slice:
  `RuntimeImportedViewModelInstanceContext` can now resolve a root list source
  into `RuntimeImportedViewModelListSourceHandle` and mutate the source
  item-count override through that handle only when it belongs to the same
  imported view-model instance context. Slash-path handle lookup remains
  unresolved, matching the nested list boundary. The C++ probe compares the
  handle write against the root list by-name mutation path and proves an
  observing state machine sees the same bindable-list source size. Stable list
  item handles, list item identity, and list item value mutation remain out of
  scope. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-list-source-handle-runtime-contract.md`.
- Imported nested view-model pointer property-name path slice:
  `RuntimeImportedViewModelInstanceContext::set_view_model_by_property_name_path`
  resolves a slash-separated path such as `child/grandchild` through nested
  `ViewModelPropertyViewModel` names, records the selected referenced imported
  instance as the existing view-model pointer override, and lets two authored
  state machines bound through the same imported context observe the relink.
  The C++ probe calls
  `ViewModelInstanceRuntime::replaceViewModel("child/grandchild", value)` after
  completing view-model properties. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-viewmodel-name-path-runtime-contract.md`.
- Imported view-model pointer source handle slice:
  `RuntimeImportedViewModelInstanceContext` can now resolve root and
  slash-separated view-model pointer source paths into
  `RuntimeImportedViewModelViewModelSourceHandle` and relink through that
  handle only when it belongs to the same imported view-model instance
  context. The C++ probe compares root `current` and nested
  `child/grandchild` handle relinks against
  `ViewModelInstanceRuntime::replaceViewModel(path, value)` through the
  existing view-model binding report surface. This closes the imported
  source-handle family for number, boolean, string, color, enum,
  symbol-list-index, asset, artboard, trigger, list, and view-model pointer
  sources. The contract is
  `docs/prototypes/data-binding-graph-imported-viewmodel-viewmodel-source-handle-runtime-contract.md`.
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
  The graph updates same-path list source facts used by the existing
  `BindablePropertyList` report surface, and the C++ probe covers both the
  selected bind and a neighboring ordinary direct `ToTarget` observer after
  `--runtime-set-default-view-model-source-list`, data-context advancement,
  and state-machine advancement. The contract is
  `docs/prototypes/data-binding-graph-default-viewmodel-list-source-mutation-runtime-contract.md`.
- First owned scalar property-name slice:
  `RuntimeOwnedViewModelInstance` records root `ViewModelProperty.name` values
  and can mutate a root number property through
  `set_number_by_property_name`, matching the C++ public
  `ViewModelInstanceRuntime::propertyNumber(name)->value(...)` path before
  binding an owned context to a state machine. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-number-name-runtime-contract.md`.
- First owned number source handle slice:
  `RuntimeOwnedViewModelInstance` can now resolve a root number property name
  into `RuntimeOwnedViewModelNumberSourceHandle` and mutate owned number
  storage through that handle before binding. Root-name handle lookup remains
  separate from slash-path lookup. Nested number paths are covered separately;
  other nested/relative/parent lookup remains a follow-up slice. The C++ probe
  compares the handle write against the existing owned-number runtime context
  command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-number-source-handle-runtime-contract.md`.
- Owned boolean source handle slice:
  `RuntimeOwnedViewModelInstance` can now resolve a root boolean property name
  into `RuntimeOwnedViewModelBooleanSourceHandle` and mutate owned boolean
  storage through that handle before binding. Root-name handle lookup remains
  separate from slash-path lookup. Nested boolean paths are covered separately;
  other nested/relative/parent lookup remains a follow-up slice. The C++ probe
  compares the handle write against the existing owned-boolean runtime context
  command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-boolean-source-handle-runtime-contract.md`.
- Owned string source handle slice:
  `RuntimeOwnedViewModelInstance` can now resolve a root string property name
  into `RuntimeOwnedViewModelStringSourceHandle` and mutate owned raw string
  storage through that handle before binding. Root-name handle lookup remains
  separate from slash-path lookup. Nested string paths are covered separately;
  other nested/relative/parent lookup remains a follow-up slice. The C++ probe
  compares the handle write against the existing owned-string runtime context
  command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-string-source-handle-runtime-contract.md`.
- Owned color source handle slice:
  `RuntimeOwnedViewModelInstance` can now resolve a root color property name
  into `RuntimeOwnedViewModelColorSourceHandle` and mutate owned color storage
  through that handle before binding. Root-name handle lookup remains separate
  from slash-path lookup. Nested color paths are covered separately; other
  nested/relative/parent lookup remains a follow-up slice. The C++ probe
  compares the handle write against the existing owned-color runtime context
  command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-color-source-handle-runtime-contract.md`.
- Owned enum source handle slice:
  `RuntimeOwnedViewModelInstance` can now resolve a root enum property name
  into `RuntimeOwnedViewModelEnumSourceHandle` and mutate owned enum
  value-index storage through that handle before binding. Root-name handle
  lookup remains separate from slash-path lookup. Nested enum paths are covered
  separately; other nested/relative/parent lookup remains a follow-up slice.
  The C++ probe compares the handle write against the existing owned-enum
  runtime context command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-enum-source-handle-runtime-contract.md`.
- Owned symbol-list-index source handle slice:
  `RuntimeOwnedViewModelInstance` can now resolve a root symbol-list-index
  property name into `RuntimeOwnedViewModelSymbolListIndexSourceHandle` and
  mutate owned symbol-list-index storage through that handle before binding.
  Root-name handle lookup remains separate from slash-path lookup. Nested
  symbol-list-index paths are covered separately; other nested/relative/parent
  lookup remains a follow-up slice. The C++ probe compares the handle write
  against the existing owned-symbol-list-index runtime context command. The
  contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-symbol-list-index-source-handle-runtime-contract.md`.
- Owned asset source handle slice:
  `RuntimeOwnedViewModelInstance` can now resolve a root asset property name
  into `RuntimeOwnedViewModelAssetSourceHandle` and mutate owned raw asset id
  storage through that handle before binding. Root-name handle lookup remains
  separate from slash-path lookup. Nested asset paths are covered separately;
  other nested/relative/parent lookup remains a follow-up slice. The C++ probe
  compares the handle write against the existing owned-asset runtime context
  command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-asset-source-handle-runtime-contract.md`.
- Owned artboard source handle slice:
  `RuntimeOwnedViewModelInstance` can now resolve a root artboard property name
  into `RuntimeOwnedViewModelArtboardSourceHandle` and mutate owned raw
  artboard id storage through that handle before binding. Root-name handle
  lookup remains separate from slash-path lookup. Nested artboard paths are
  covered separately; other nested/relative/parent lookup remains a follow-up
  slice. The C++ probe compares the handle write against the existing
  owned-artboard runtime context command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-artboard-source-handle-runtime-contract.md`.
- Owned trigger source handle slice:
  `RuntimeOwnedViewModelInstance` can now resolve a root trigger property name
  into `RuntimeOwnedViewModelTriggerSourceHandle` and mutate owned raw trigger
  count storage through that handle before binding. Root-name handle lookup
  remains separate from slash-path lookup. Nested trigger paths are covered
  separately; other nested/relative/parent lookup remains a follow-up slice.
  The C++ probe compares the handle write against the existing owned-trigger
  runtime context command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-trigger-source-handle-runtime-contract.md`.
- Owned list source handle slice:
  `RuntimeOwnedViewModelInstance` can now resolve a root list property name
  into `RuntimeOwnedViewModelListSourceHandle` and mutate owned list item-count
  storage through that handle before binding. Root-name handle lookup remains
  separate from slash-path lookup. Nested list paths are covered separately;
  other nested/relative/parent lookup remains a follow-up slice. The C++ probe
  compares the handle write against the existing owned-list runtime context
  command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-list-source-handle-runtime-contract.md`.
- Owned view-model pointer source handle slice:
  `RuntimeOwnedViewModelInstance` can now resolve a root view-model pointer
  property name into `RuntimeOwnedViewModelViewModelSourceHandle` and relink
  owned view-model pointer storage through that handle before binding.
  Root-name handle lookup remains separate from slash-path lookup. Generated
  nested view-model pointer paths are covered separately, and the owned runtime
  root source-handle family is now covered for number, boolean, string, color,
  enum, symbol-list-index, asset, artboard, trigger, list, and view-model
  pointer sources. The C++ probe compares the handle relink against the
  existing owned view-model pointer runtime context command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-viewmodel-source-handle-runtime-contract.md`.
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
- First owned generated nested source-handle slice:
  generated owned view-model children can now expose a stable public number
  source handle. `RuntimeOwnedViewModelInstance` can resolve
  `child/amount` into `RuntimeOwnedViewModelNumberSourceHandle` through
  `number_source_handle_by_property_name_path`, and
  `set_number_by_source_handle` mutates the same generated-child number
  storage before binding. The C++ probe compares against the existing owned
  number name-path command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-number-source-handle-runtime-contract.md`.
- Owned generated nested boolean name-path slice:
  generated owned view-model children can store direct boolean values, and
  `RuntimeOwnedViewModelInstance::set_boolean_by_property_name_path` can
  mutate paths such as `child/enabled` before binding. The graph resolves
  matching nested boolean source paths for owned contexts, and the C++ probe
  calls
  `ViewModelInstanceRuntime::propertyBoolean("child/enabled")->value(...)`.
  The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-boolean-name-path-runtime-contract.md`.
- Owned generated nested boolean source-handle slice:
  generated owned view-model children can now expose a stable public boolean
  source handle. `RuntimeOwnedViewModelInstance` can resolve
  `child/enabled` into `RuntimeOwnedViewModelBooleanSourceHandle` through
  `boolean_source_handle_by_property_name_path`, and
  `set_boolean_by_source_handle` mutates the same generated-child boolean
  storage before binding. The C++ probe compares against the existing owned
  boolean name-path command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-boolean-source-handle-runtime-contract.md`.
- Owned generated nested string name-path slice:
  generated owned view-model children can store direct string values, and
  `RuntimeOwnedViewModelInstance::set_string_by_property_name_path` can mutate
  paths such as `child/label` before binding. The graph resolves matching
  nested string source paths for owned contexts, and the C++ probe calls
  `ViewModelInstanceRuntime::propertyString("child/label")->value(...)`. The
  contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-string-name-path-runtime-contract.md`.
- Owned generated nested string source-handle slice:
  generated owned view-model children can now expose a stable public string
  source handle. `RuntimeOwnedViewModelInstance` can resolve `child/label`
  into `RuntimeOwnedViewModelStringSourceHandle` through
  `string_source_handle_by_property_name_path`, and
  `set_string_by_source_handle` mutates the same generated-child raw string
  storage before binding. The C++ probe compares against the existing owned
  string name-path command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-string-source-handle-runtime-contract.md`.
- Owned generated nested color name-path slice:
  generated owned view-model children can store direct color values, and
  `RuntimeOwnedViewModelInstance::set_color_by_property_name_path` can mutate
  paths such as `child/tint` before binding. The graph resolves matching
  nested color source paths for owned contexts, and the C++ probe calls
  `ViewModelInstanceRuntime::propertyColor("child/tint")->value(...)`. The
  contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-color-name-path-runtime-contract.md`.
- Owned generated nested color source-handle slice:
  generated owned view-model children can now expose a stable public color
  source handle. `RuntimeOwnedViewModelInstance` can resolve `child/tint`
  into `RuntimeOwnedViewModelColorSourceHandle` through
  `color_source_handle_by_property_name_path`, and
  `set_color_by_source_handle` mutates the same generated-child color storage
  before binding. The C++ probe compares against the existing owned color
  name-path command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-color-source-handle-runtime-contract.md`.
- Owned generated nested enum name-path slice:
  generated owned view-model children can store direct enum value indexes, and
  `RuntimeOwnedViewModelInstance::set_enum_by_property_name_path` can mutate
  paths such as `child/choice` before binding. The graph resolves matching
  nested enum source paths for owned contexts, and the C++ probe calls
  `ViewModelInstanceRuntime::propertyEnum("child/choice")->valueIndex(...)`.
  The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-enum-name-path-runtime-contract.md`.
- Owned generated nested enum source-handle slice:
  generated owned view-model children can now expose a stable public enum
  source handle. `RuntimeOwnedViewModelInstance` can resolve `child/choice`
  into `RuntimeOwnedViewModelEnumSourceHandle` through
  `enum_source_handle_by_property_name_path`, and
  `set_enum_by_source_handle` mutates the same generated-child enum
  value-index storage before binding. The C++ probe compares against the
  existing owned enum name-path command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-enum-source-handle-runtime-contract.md`.
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
- Owned generated nested symbol-list-index source-handle slice:
  generated owned view-model children can now expose a stable public
  symbol-list-index source handle. `RuntimeOwnedViewModelInstance` can resolve
  `child/symbol` into `RuntimeOwnedViewModelSymbolListIndexSourceHandle`
  through `symbol_list_index_source_handle_by_property_name_path`, and
  `set_symbol_list_index_by_source_handle` mutates the same generated-child
  symbol-list-index storage before binding. The C++ probe compares against the
  existing owned symbol-list-index name-path command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-symbol-list-index-source-handle-runtime-contract.md`.
- Owned generated nested asset name-path slice:
  generated owned view-model children can store direct asset IDs, and
  `RuntimeOwnedViewModelInstance::set_asset_by_property_name_path` can mutate
  paths such as `child/image` before binding. The graph resolves matching
  nested asset source paths for owned contexts, and the C++ probe resolves
  `child` with `propertyViewModel` before mutating the child's
  `ViewModelInstanceAssetImage` through `propertyValue`. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-asset-name-path-runtime-contract.md`.
- Owned generated nested asset source-handle slice:
  generated owned view-model children can now expose a stable public asset
  source handle. `RuntimeOwnedViewModelInstance` can resolve `child/image`
  into `RuntimeOwnedViewModelAssetSourceHandle` through
  `asset_source_handle_by_property_name_path`, and
  `set_asset_by_source_handle` mutates the same generated-child asset storage
  before binding. The C++ probe compares against the existing owned asset
  name-path command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-asset-source-handle-runtime-contract.md`.
- Owned generated nested artboard name-path slice:
  generated owned view-model children can store direct artboard IDs, and
  `RuntimeOwnedViewModelInstance::set_artboard_by_property_name_path` can
  mutate paths such as `child/scene` before binding. The graph resolves
  matching nested artboard source paths for owned contexts, and the C++ probe
  resolves `child` with `propertyViewModel` before mutating the child's
  `ViewModelInstanceArtboard` through `propertyValue`. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-artboard-name-path-runtime-contract.md`.
- Owned generated nested artboard source-handle slice:
  generated owned view-model children can now expose a stable public artboard
  source handle. `RuntimeOwnedViewModelInstance` can resolve `child/scene`
  into `RuntimeOwnedViewModelArtboardSourceHandle` through
  `artboard_source_handle_by_property_name_path`, and
  `set_artboard_by_source_handle` mutates the same generated-child artboard
  storage before binding. The C++ probe compares against the existing owned
  artboard name-path command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-artboard-source-handle-runtime-contract.md`.
- Owned generated nested trigger name-path slice:
  generated owned view-model children can store direct trigger values, and
  `RuntimeOwnedViewModelInstance::set_trigger_by_property_name_path` can
  mutate paths such as `child/fire` before binding. The graph resolves
  matching nested trigger source paths for owned contexts, and the C++ probe
  resolves `child` with `propertyViewModel` before mutating the child's
  `ViewModelInstanceTrigger` through `propertyValue`. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-trigger-name-path-runtime-contract.md`.
- Owned generated nested trigger source-handle slice:
  generated owned view-model children can now expose a stable public trigger
  source handle. `RuntimeOwnedViewModelInstance` can resolve `child/fire` into
  `RuntimeOwnedViewModelTriggerSourceHandle` through
  `trigger_source_handle_by_property_name_path`, and
  `set_trigger_by_source_handle` mutates the same generated-child trigger
  storage before binding. The C++ probe compares against the existing owned
  trigger name-path command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-trigger-source-handle-runtime-contract.md`.
- Owned generated nested list name-path item-count slice:
  generated owned view-model children can store direct list item counts, and
  `RuntimeOwnedViewModelInstance::set_list_item_count_by_property_name_path`
  can mutate paths such as `child/items` before binding. The graph resolves
  matching nested `RuntimeDataBindGraphValue::List` source paths for owned
  contexts, and the C++ probe resolves `child/items` with `propertyList`,
  adding blank item instances to set the observed list size without admitting
  item identity or item-level traversal. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-list-name-path-runtime-contract.md`.
- Owned generated nested list source-handle slice:
  generated owned view-model children can now expose a stable public list
  source handle for item-count mutation. `RuntimeOwnedViewModelInstance` can
  resolve `child/items` into `RuntimeOwnedViewModelListSourceHandle` through
  `list_source_handle_by_property_name_path`, and
  `set_list_item_count_by_source_handle` mutates the same generated-child list
  item-count storage before binding. The C++ probe compares against the
  existing owned list name-path command. The contract is
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-list-source-handle-runtime-contract.md`.
- First `BindablePropertyList.propertyValue` target-to-source slice:
  state-machine list targets can be mutated by data-bind index, and explicit
  `advancedDataContext()` plus public `updateDataBinds(true)` consume the
  dirty target as a C++-compatible no-op for direct `ViewModelInstanceList`
  sources and `DataConverterNumberToList` sources. The contract is
  `docs/prototypes/data-binding-graph-bindable-list-target-to-source-runtime-contract.md`.
- Number-to-list main-to-source target-to-source slice:
  direct `DataConverterNumberToList` on a main-`ToSource | TwoWay`
  `BindablePropertyList.propertyValue` bind now has explicit
  `advancedDataContext()` coverage. C++ preserves the edited target scalar and
  leaves the numeric source unchanged, without admitting generated list item
  runtime instances. The contract is
  `docs/prototypes/data-binding-graph-number-to-list-main-to-source-target-to-source-runtime-contract.md`.
- State-machine `NameBased` data-bind source-path unsupported boundary:
  cloned state-machine `DataBindContext` records have a null `DataBind::file()`
  pointer in C++, so manifest-backed `sourcePathIds` do not resolve during
  runtime binding for this shape. Rust keeps the source unbound while reporting
  the cloned bindable number target's initial value. The contract is
  `docs/prototypes/data-binding-graph-name-based-state-machine-source-path-unsupported-runtime-contract.md`.
- Artboard component-list `NameBased` data-bind source-path unsupported
  boundary: binding the default artboard view-model context against a manifest
  path for `items` keeps the component-list target row and empty target-list
  fact, but C++ does not resolve the source list for this runtime shape. Direct
  post-bind `Artboard::updateDataBinds(true)` and post-bind
  `Artboard::advance(0.0f)` preserve the same unresolved source and empty
  target-list facts. Rust preserves the same unresolved source facts across all
  three entrypoints. The contract is
  `docs/prototypes/data-binding-graph-artboard-name-based-source-path-unsupported-runtime-contract.md`.
- File-backed `DataContext` lookup report slice:
  `runtime_data_context_lookup_reports` now records the full C++-order
  `--data-context-lookups` surface for imported view models, instances, and
  explicit values: absolute `DataContext` property/instance lookups,
  `ViewModelInstance::propertyFromPath`, manifest-relative property/instance
  lookups, and the first absolute plus manifest-relative parent fallback
  property lookup. The C++ probe compares the complete read-only report for a
  nested view-model fixture. The contract is
  `docs/prototypes/data-context-file-backed-lookup-runtime-contract.md`.

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
  paths, and stable public handles beyond the admitted default nested-number/
  boolean/string/color/enum/symbol-list-index/asset/artboard/trigger/list/view-model
  handles plus default/imported/owned root source handles and owned
  nested-number/boolean/string/color/enum/symbol-list-index/asset/artboard/trigger/list/view-model
  handles,
  especially remaining nested/relative/parent handles that update or expose
  cached source indexes.
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
  non-number fallbacks, deterministic direct formula functions, and the
  operation-value-to-formula public-update groups, concrete operation
  pass-through, non-scripting scripted converter pass-through, broader
  dirty-list target scheduling, data-binding update queues, full artboard
  component-list item instancing, map-rule-driven child
  creation, list layout/virtualization, remaining generated-list reverse
  converters, live relative/name data-bind wiring beyond the read-only
  file-backed lookup report, remaining converter name paths beyond the direct
  and grouped
  `DataConverterOperationViewModel` unsupported boundaries, relative paths,
  parent paths, and nested source kinds beyond the default-context number,
  boolean, string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, and view-model slices.
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
