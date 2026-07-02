# Rive Rust Porting Map

Working directory: `/Users/levi/dev/rive-rust`

Reference runtime: `/Users/levi/dev/oss/rive-runtime`

Goal: build a Rust runtime that can load `.riv` files, instantiate artboards, maintain the runtime graph, advance animations and state machines, and eventually render through a Rust renderer API or compatibility bridge.

Initial constraint: do not begin with GPU rendering. Prove the headless schema, binary import, and runtime graph first.

## #1: Compatibility Target

Blocked by: none
Type: Discuss

### Question

Are we targeting full C++ runtime parity from the start, or a graph-first subset that can grow toward parity?

### Answer

Resolved for now: use a graph-first subset. The first milestone is a headless runtime that can load a simple `.riv`, build the artboard graph, instantiate it, and advance transforms deterministically. Rendering, text, scripting, audio, and platform integrations come later.

## #2: Repository Shape

Blocked by: #1
Type: Discuss

### Question

What crate layout should the Rust port use?

### Answer

Tentative layout:

```text
crates/
  rive-schema/        # generated schema metadata from dev/defs
  rive-binary/        # .riv reader, runtime header, property decoding
  rive-core/          # object arena, generated object enum, typed IDs
  rive-graph/         # artboard runtime graph, dirt, dependencies
  rive-animation/     # linear animations and state machines
  rive-render-api/    # renderer-agnostic draw commands
  rive-ffi/           # C ABI / Swift / Kotlin / wasm later
tools/
  rive-codegen/
  rive-compare/
fixtures/
  minimal/
  graph/
  animation/
docs/
  porting-map.md
```

This should stay flexible until the first prototype proves which boundaries are real.

## #3: Fixture And Comparison Harness

Blocked by: #1
Type: Research

### Question

What minimal `.riv` fixture corpus and C++ comparison harness do we need before implementing Rust behavior?

### Answer

Resolved in `docs/research/fixture-harness.md`. A starter fixture corpus was copied into `fixtures/minimal`, `fixtures/graph`, and `fixtures/animation`. `tools/cpp-probe` now imports fixtures through the C++ runtime and emits JSON for artboard names/counts, compact artboard-local object IDs, type keys, parent IDs, resolved parent IDs, graph order, and world transforms. `make cpp-compare` compares the current Rust graph projection against this C++ output.

## #4: Schema Generation

Blocked by: #2
Type: Prototype

### Question

How should Rust types be generated from `dev/defs`?

### Answer

Resolved in `docs/prototypes/schema-generation.md`. Added a Rust workspace with `crates/rive-schema` and `tools/rive-codegen`. `make schema` reads the C++ `dev/defs` JSON, writes a formatted generated schema, validates duplicate/reserved keys and bitmask passthrough invariants like the C++ generator, and generates `ObjectKind`, type-key metadata, runtime parent/ancestor metadata for `is_a`, definition mixin/generic/export-context metadata, abstract/cloneable flags, callback-key metadata matching `CoreRegistry::isCallback`, `object_supports_property` metadata matching `CoreRegistry::objectSupportsProperty`, CoreRegistry setter/getter family metadata, typed effective stored-field initializers, generated C++ value-accessor/change-hook/declaration/bitmask-constant helper metadata, and property-key metadata including alternates, runtime field kinds, descriptions, explicit and runtime initial values, C++ generator flags, bindable/animates/passthrough flags, raw annotations such as journal/records/parentable/conditional export, bitmask passthroughs, and deserialization/storage flags. Current output: 336 runtime definitions and 588 runtime properties. Tests now verify generated schema invariants, byte-compare formatted `rive-codegen` output with the checked-in generated schema, reject synthetic invalid key/bitmask metadata, compare Rust type/property keys against the generated C++ `*_base.hpp` headers, compare stored-field member presence, effective stored-field initializers, generated value-setter body presence and stored-field/passthrough body shape, pure-virtual value setter declarations, generated stored-field getter bodies and passthrough getter declarations, encoded `decode*`/`copy*` declarations, generated bitmask passthrough `Bitmask`/`BitOffset`/`FieldMask` constants, generated `Changed()` hook presence, generated `copy(...)` member assignments, and encoded-property copy hooks against C++ generated members, compare schema ancestors against C++ generated `isTypeOf` switches, compare abstract/cloneable flags against `CoreRegistry::makeCoreInstance` constructibility, generated `clone()` declarations, and generated clone implementation bodies, compare schema `deserializes` flags against C++ generated `deserialize` switches, compare every schema property against C++ `CoreRegistry::propertyFieldId` fallback families, including the fact that bitmask passthrough properties are not globally skippable, compare callback keys against C++ `CoreRegistry::isCallback`, compare object/property support against C++ `CoreRegistry::objectSupportsProperty`, including the exclusion of encoded bytes payload fields, and compare setter/getter families against C++ `CoreRegistry::set*`/`get*`, including the fact that semantic boolean bitmask passthroughs are settable but not getter-backed. Concrete object structs remain deferred until `rive-core`; ticket `#5` can now consume this metadata for binary import.

## #5: Binary Import

Blocked by: #4
Type: Prototype
Completion contract: `docs/prototypes/binary-import-completion-contract.md`

### Question

Can Rust decode `.riv` files into a stable object arena while preserving C++ compatibility behavior?

### Answer

Resolved in `docs/prototypes/binary-import.md`. Added `crates/rive-binary` and `riv-inspect`. Rust now parses runtime headers, LEB128 varuints, primitive field values, property ToC data, inherited properties, abstract/null object slots, and unknown object/property skipping using schema-modeled C++ `CoreRegistry::propertyFieldId` fallback families first and the file header ToC second, so conflicting header ToC entries cannot override generated registry field kinds, including after known-object generated-deserialize misses. Header integer and malformed-header semantics now match C++ `RuntimeHeader`: same-major newer minor versions are accepted, unsupported major versions are rejected as `unsupportedVersion`, bad/truncated fingerprints and incomplete required header segments are malformed, noncanonical and overwide LEB128 encodings are accepted, signed-`int` range overflows are malformed, signed-`int` boundary values such as `i32::MAX` ToC keys are accepted, duplicate ToC property keys overwrite earlier field ids with later field ids, truncated varuints are malformed, truncated packed ToC field-id words and missing property-ToC terminators are malformed, and ToC keys are preserved as 32-bit values. Length-prefixed string/bytes payloads also accept noncanonical LEB128 lengths while rejecting truncated length varuints. Object type-key reads also follow C++ signed-`int` behavior, allowing unknown keys above `u16` through `i32::MAX` to be skipped as null object slots when their properties are skippable; abstract known type keys also import as null slots according to `CoreRegistry::makeCoreInstance`, while object property keys follow C++ `uint16_t` range semantics, accepting `u16::MAX` and rejecting one-past max. Known-object property dispatch uses primary keys only like the generated C++ `deserialize` switches; alternate keys and schema-known properties absent from the generated C++ switch fall through to global/header fallback skipping instead of direct schema decoding, while duplicate stored properties expose the same final member state as C++ by letting the last serialized value win. Output is a stable `Vec<Option<RuntimeObject>>` preserving serialized object indexes, plus `import_statuses` metadata that tracks whether C++ would keep, drop, or null each slot after `object->import(importStack)`. C++ import-time mutations now include duplicate `FileAsset.assetId` normalization, matching `BackboardImporter::addFileAsset`, and only successfully imported assets participate in that mutation. Omitted stored scalar/string properties now expose generated C++ member defaults through typed helpers, including raw bytes for default strings and the C++ `Artboard` constructor override that makes inherited `clip` default to `true`, while keeping the serialized property list sparse. Runtime `uint` field values now match C++ `unsigned int` range semantics. String values now preserve raw bytes with an optional UTF-8 view instead of forcing lossy decoding, bool values match C++'s `byte == 1` decoding, and bytes fields preserve the full raw payload for future typed decoders while treating truncated declared payloads as malformed. Unknown string and bytes properties share the same global fallback field id like C++, so bytes payloads are skippable through the string/bytes path. Fixed-width primitive truncation for float32/double, color, and bool is also malformed. Missing-ToC fallback failures now match C++'s `nullptr` behavior when the reader has not errored, and synthetic tests cover the bool-field, callback, and bitmask-passthrough fallback quirks, including known-object callback/bitmask-passthrough EOF cases. Known non-stored properties now preserve skip metadata and decoded values for passthrough fields and header-ToC-backed bitmask passthrough schema fields, while bitmask passthrough keys are not treated as globally skippable for unknown-object fallback. Tests import the full starter fixture corpus, lock in C++-anchored expectations for `dependency_test.riv`, `two_artboards.riv`, and `smi_test.riv`, and include synthetic binary cases for exact C++ import result categories, header integer and malformed-header behavior, duplicate header-ToC overwrite behavior, noncanonical/overwide/truncated varuints, noncanonical string/bytes length prefixes, truncated packed ToC field ids, unknown object type keys, abstract known-object null slots, object property-key range behavior, alternate-key fallback behavior, duplicate stored-property overwrite behavior, duplicate file-asset id normalization, omitted stored-property defaults, `uint` value ranges, raw string/bytes payloads, noncanonical bool byte decoding, CoreRegistry-over-header fallback priority, string/bytes fallback id sharing, truncated bytes and fixed-width primitive payloads, missing-ToC behavior, bool/callback/bitmask-passthrough fallback behavior, known-object callback/bitmask-passthrough fallback behavior, import-stack keep/drop status, dropped-asset mutation exclusion, and passthrough/header-ToC-backed bitmask passthrough decoding. `make cpp-compare` now also runs `crates/rive-binary/tests/cpp_import.rs`, which compares exact Rust and C++ import result categories for synthetic forward-compatibility cases, compares C++ failed-import diagnostics against Rust dropped statuses for synthetic streams and every C++-accepted corpus fixture, compares decoded artboard names and dimensions against C++ probe JSON for the starter fixtures, compares `File::assets()` order/type/name/asset-id/CDN-base-url metadata for asset-heavy reference fixtures, compares `File::viewModel(i)` and `File::enums()` order/type/name plus view-model property/instance and data-enum value ownership, compares artboard-local object slots and component metadata against C++ `Artboard::objects()`/`Component` state, compares `Artboard::animation(i)` and `Artboard::stateMachine(i)` metadata including linear animation scalar fields, validated keyed-object/keyed-property ownership, first-keyframe metadata, ordered state-machine layer/input/listener/data-bind child lists, layer state counts, listener target/action/input-type counts, and data-bind property/converter fields, compares file-level and non-root artboard-local getter-backed stored property values against C++ `CoreRegistry` getters when those getters directly expose stored members, including file assets, view models, enums, and artboard-owned `KeyFrameInterpolator` descendants, and excludes virtual/override/pure-virtual runtime-state getters from that stored-value comparison. It also runs a corpus-wide structural summary comparison for every C++-accepted unit fixture, covering artboard names/bounds/object slots, file assets, view-model properties/instances, and data-enum values while matching C++ import ownership details such as manifest assets, user inputs, and scripted inputs, script/shader asset-content behavior in the non-scripting probe build, formula-token importer context, and scroll-physics backboard ownership. The full starter fixture corpus imports successfully; `dependency_test.riv` imports as 19 slots, 18 known objects, with artboard `Blue`.

The C++ constructor stored-field default override audit currently pins `Artboard.clip` as the only known object-specific default that differs from generated member initializers.

The file-asset helper source audit pins C++ `uniqueName()`, `uniqueFilename()`, `cdnUuidStr()`, and concrete asset extension overrides against the Rust binary metadata helpers.

The failed-import comparison is stricter than count parity: it compares the ordered C++ failed type-key diagnostics against the Rust dropped import-status type keys for synthetic streams and every C++-accepted corpus fixture.

The corpus comparison now also checks `CoreRegistry` getter-backed stored property values before artboard advance for every C++-accepted fixture, covering file assets, view models, view-model properties/instances, data enums, enum values, and non-root artboard-local stored fields whose getters directly expose stored members. Virtual, override, and pure-virtual getter families are excluded because they can report resolved runtime state instead of imported member values.

The import-status model now tracks explicit C++ `ImportStack` latest-importer keys and `lastAdded` order instead of context booleans, so tests can pin keyed ownership and `readNullObject()` routing. Synthetic Rust and C++ probe coverage now verifies that a null object after a state-machine layer is consumed by the latest `StateMachineLayerImporter` as a null state slot rather than leaking back to the older `StateMachineImporter` as an input placeholder, and that a latest importer whose `readNullObject()` returns false is skipped while searching backward for an older null-object consumer.

The corpus structural summary now checks every artboard-local object slot's null/type-key shape, compares component core types, names, parent IDs, and resolved parent slots for every accepted fixture, compares file asset indexes/core types/names/asset IDs/CDN base URLs/helper-derived strings, view-model and child indexes/core types/names/instance IDs/value children/list items/property links, data-enum/value indexes/core types/names/keys/values, linear-animation indexes/core types/names/scalar fields/keyed-object identities/keyed-property lists/first-keyframe metadata, artboard skin/skinnable/tendon relationships, mesh/path vertex weight relationships, shape-owned path/paint relationships including parametric paths and forward-referenced paint mutators, artboard `ShapePaintContainer` registration lists plus paint/mutator, feather, gradient-stop, stroke-effect, and target-effect identities, artboard-owned data-bind indexes/core types/property keys/flags/converter IDs/resolved converter core types/converter interpolator core types/converter-group item core types/interpolator core types/target core types/target locals, and state-machine/layer/input/listener/data-bind indexes/core types/names/counts plus listener targets, listener action/input-type child records, listener view-model path buffers, and state-machine data-bind property/converter fields including resolved converter-group item and interpolator core types. Selected animation/state-machine probe fixtures now also compare `CoreRegistry` getter-backed stored values for animation, artboard data-bind, and state-machine child objects. The summary models C++ import-stack ownership details such as listener actions staying attached across intervening data-bind records, listener input type importers gating keyboard/semantic/gamepad user inputs, scripted inputs requiring the latest `ScriptedObjectImporter` instead of generic component context, listener input-change invalid-object drops when a resolved state-machine input has the wrong kind while null/out-of-range inputs are accepted, listener input-change nested-input validation against artboard-local `NestedBool`/`NestedNumber`/`NestedTrigger` slots before state-machine fallback, transition condition/comparator importers requiring the matching state-transition, view-model-condition, and bindable-property context, transition input-condition invalid-object drops when input IDs are out of range or point at the wrong state-machine input kind, blend-state/direct-blend invalid-object drops when input IDs do not resolve to number inputs or blend animations attach to non-blend states, `StateMachineImporter::readNullObject` adding null input placeholders, `StateMachineLayerImporter::readNullObject` adding placeholder states that participate in `StateTransition.stateToId` resolution, required Any/Entry/Exit state-machine layer scaffold validation, fatal constraint initialization checks for `TransformComponent`/`Bone` parentage, fatal object-import failures preserving null artboard-local slots, recoverable NSlicer/Axis/TileMode wrong-parent `MissingObject` lifecycle failures that keep slots while skipping invalid registrations, fatal paint/effect initialization checks for `DashPath`/effects-container parentage, duplicate shape-paint mutators, and TrimPath mode values, fatal text initialization checks for `TextStyle` and `TextInput` parentage, `DataBind` targets determining whether a bind is artboard-owned or state-machine-owned, data-converter IDs resolving through the imported backboard converter list, converter interpolator IDs resolving through the pre-artboard backboard `KeyFrameInterpolator` list, `DataConverterGroupItem` objects attaching to the latest `DataConverterGroupImporter`, failed non-`DataBind` imports clearing C++ `lastBindableObject`, `KeyedObject::onAddedDirty` removing keyed objects whose target cannot resolve and removing keyed properties whose resolved target object does not support the property key, C++ auto-generated empty state machines, and `Artboard::validateObjects` nulling invalid targeted constraints, text styles, nested animations, scroll-bar constraints, and feather effects while preserving slot positions.

`Joystick.handleSourceId`, `xId`, and `yId` are now pinned at the binary import boundary: Rust decodes and preserves default, valid, missing, and wrong-type explicit ids without rejecting the file, matching C++ `File::import`. Rust also exposes `RuntimeFile::resolved_handle_source_for_joystick(_object)`, mirroring C++ `Joystick::onAddedDirty` by resolving against validated artboard-local slots and accepting only `TransformComponent` handle sources, plus `resolved_x_animation_for_joystick(_object)` and `resolved_y_animation_for_joystick(_object)`, mirroring C++ `Joystick::onAddedClean` axis animation lookup through `Artboard::animation(...)`.

View-model asset-value snapshot comparisons now include the same file-asset helper-derived fields as the `File::assets()` comparison, covering copied CDN URL/UUID and unique-name metadata.

The corpus comparison now also checks `File::dataResolver()` manifest name/path tables against `RuntimeFile::manifest()`, so manifest-backed data-bind path expansion has a C++-verified decoded source. Synthetic C++ probe coverage also pins signed `ManifestAsset` map keys, including wrapped high-bit IDs such as `u32::MAX -> -1`, plus the partial manifest map state C++ keeps when malformed manifest bytes make `ManifestAsset::decode()` return false but `FileAssetImporter::resolve()` still returns success.

Manifest-backed `DataBindPath::resolvedPath()` behavior is pinned for the C++ single-id expansion case, missing manifest ids resolving to an empty path, wrapped `uint32_t` IDs resolving through signed manifest keys, and multi-id paths remaining unexpanded.

Name-based `DataBindContext::resolvePath()` behavior is also pinned through the C++ probe: Rust exposes both the raw decoded `sourcePathIds` buffer and the lazily resolved buffer that expands the first manifest id only when the manifest lookup is non-empty, including the C++ behavior where multi-id name-based context buffers are replaced by the first id's manifest path.

`DataConverterNumberToList` file-backed `viewModelId` resolution is pinned through the C++ probe as well. Rust exposes `RuntimeFile::resolved_view_model_for_number_to_list_converter(_object)`, matching the `file(this)` assignment in C++ `File::read` and `File::viewModel(index)` null behavior for missing ids.

`DataConverter::outputType()` parity is pinned through the C++ probe. Rust exposes `RuntimeFile::data_converter_output_type(_for_object)` with C++ enum discriminants, including inherited operation converters returning `number`, interpolators returning `input`, `ScriptedDataConverter` returning `any`, and `DataConverterGroup` resolving from the last group item backward until a non-`input` child output type is found, otherwise returning `none`.

`DataConverter::bindFromContext()` parity has started at the binary layer. Rust exposes `RuntimeFile::data_converter_bind_context_effect(_for_object)`, modeling parent-data-bind storage, owned `DataBindContext` rebinding, `DataConverterGroup` child dispatch, `DataConverterOperationViewModel` number-source lookup/dependency registration, and `DataConverterFormula` parent-source binding. The helper takes live context, parent-source, and lookup-result state explicitly because those pointers are runtime state, not serialized fields. A C++ source audit pins the base method plus the group, operation-view-model, and formula overrides.

Data-converter lifecycle side effects are now partially pinned as well. Rust exposes `RuntimeFile::data_converter_unbind_effect(_for_object)`, `data_converter_update_effect(_for_object)`, `data_converter_reset_effect(_for_object)`, `data_converter_mark_dirty_effect(_for_object)`, `data_converter_property_change_effect(_for_object)`, `data_converter_add_dirty_data_bind_effect(_for_object)`, `data_converter_formula_add_dirt_effect(_for_object)`, and `data_bind_container_add_dirty_effect()`, matching C++ base converter owned-bind unbind/update delegation, `DataConverterGroup` child forwarding without base delegation, `DataConverterFormula` source-dependent cleanup, base reset no-op, group reset forwarding, `DataConverterInterpolator` advance-count/advancer reset, parent `DataBind` dirt propagation, dirty-queue selection for converter-owned binds, property-change dirty hooks for the audited converter fields, `DataConverterNumberToList.viewModelId` cache clearing, and formula random-cache clearing for `RandomMode::sourceChange`.

Imported `DataBind::sourceOutputType()`/`outputType()` parity is pinned through the C++ probe as well. Rust exposes `RuntimeFile::data_bind_source_output_type(_for_object)` and `data_bind_output_type(_for_object)` for the unbound binary graph: source output is `none`, and bind output returns a concrete converter output unless that converter reports `input` or `none`.

Data-bind lifecycle classification has started at the binary layer. Rust exposes `RuntimeFile::data_bind_to_source(_for_object)`, `data_bind_to_target(_for_object)`, `data_bind_binds_once(_for_object)`, `data_bind_is_main_to_source(_for_object)`, `data_bind_source_to_target_runs_first(_for_object)`, `data_bind_is_name_based(_for_object)`, `data_bind_target_supports_push(_for_object)`, `data_bind_uses_persisting_list(_for_object)`, `data_bind_can_skip(_for_object)`, `data_bind_collapse_effect(_for_object)`, `data_bind_add_dirt_effect(_for_object)`, `data_bind_add_effect(_for_object)`, `data_bind_source_effect(_for_object)`, `data_bind_clear_source_effect(_for_object)`, `data_bind_bind_effect(_for_object)`, `data_bind_target_effect(_for_object)`, `data_bind_unbind_effect(_for_object)`, `data_bind_initialize_effect(_for_object)`, `data_bind_relink_effect(_for_object)`, `data_bind_context_bind_effect(_for_object)`, `data_bind_update_effect(_for_object)`, `data_bind_remove_effect(_for_object)`, `data_bind_container_bind_context_effect()`, `data_bind_container_unbind_effect()`, `data_bind_container_advance_effect()`, `data_bind_container_update_effect()`, `data_bind_stateful_advance(_for_object)`, `data_bind_update_queue(_for_object)`, and `sorted_data_bind_ids()`, matching C++ `DataBind` flag helpers, `DataBind::targetSupportsPush()`, `DataBind::canSkip()`, `DataBind::collapse()`, `DataBind::addDirt()`, `DataBind::source()`, `DataBind::clearSource()`, `DataBind::bind()`, `DataBind::target()`, `DataBind::unbind()`, `DataBind::initialize()`, `DataBind::relinkDataBind()`, `DataBindContext::bindFromContext()`, `DataBind::update()`, `DataBind::updateSourceBinding()`, `DataBind::updateDependents()`, `DataBind::advance()`, the `DataBindContainer::addDataBind()` deferred-add gate, persisting-list enrollment, and existing-context bind/update branch, `DataBindContainer::addDirtyDataBind()` queue selection, `DataBindContainer::updateDataBind()` ordering, `DataBindContainer::removeDataBind()` deferred/immediate cleanup, `DataBindContainer::bindDataBindsFromContext()` context assignment, `DataBindContainer::unbindDataBinds()` unbind/clear behavior, `DataBindContainer::advanceDataBinds()` aggregation, `DataBindContainer::updateDataBinds()` scheduler behavior, and `DataBindContainer::sortDataBinds()` swap partitioning. `data_bind_can_skip` takes target collapsed state explicitly because C++ reads mutable `ComponentDirt::Collapsed` state rather than a serialized field, `data_bind_collapse_effect` takes current/requested collapsed state plus dirt/container presence explicitly because C++ mutates packed live flags and may call `addDirtyDataBind`, `data_bind_add_dirt_effect` takes current/added `RuntimeComponentDirt` plus live suppress/collapsed/context/container state explicitly because C++ ORs packed dirt bits, invalidates context values when `Dependents` is present after the OR, and queues only when contained and not collapsed, `data_bind_add_effect` takes live processing/data-context presence explicitly because C++ either queues pending additions or immediately appends, sets persisting/container state, and for `DataBindContext` binds an existing data context before running `updateDataBind(dataBind, true)`, `data_bind_source_effect` and `data_bind_clear_source_effect` take live source data explicitly because C++ mutates `m_Source`, adds/removes view-model dependents unless `Once` is set, and toggles `ArtboardComponentList::shouldResetInstances` only for number sources, `data_bind_bind_effect`, `data_bind_target_effect`, and `data_bind_unbind_effect` take live source/context/target/observer/dirt/container state explicitly because C++ creates/deletes context values from `outputType()`, resets/unbinds converters, removes/re-adds target property observers, and calls `addDirt(ComponentDirt::Bindings, true)` during bind, `data_bind_initialize_effect` and `data_bind_relink_effect` take live collapsable/container/data-context state explicitly because C++ registers Component targets with `pushUnique` semantics, immediately calls `collapse(isCollapsed())` on first registration, and container rebuilds only bind `DataBindContext` children, `data_bind_context_bind_effect` takes live data-context/source-lookup/current-source/context-value/observer/dirt state explicitly because C++ resolves name-based paths once, chooses relative versus absolute view-model property lookup, binds new sources, unbinds missing sources, dirties unchanged sources, and always calls converter `bindFromContext` when a converter exists and the incoming data context is non-null, `data_bind_update_effect` takes current dirt plus live source/context/target/apply-target-to-source state explicitly because C++ clears dirt, conditionally updates converter dependents, applies target-to-source before or after the update based on `SourceToTargetRunsFirst`, and suppresses dirt around source-to-target writes, `data_bind_remove_effect` takes live processing/persisting/dirty membership explicitly because C++ either queues pending removals during `updateDataBinds` processing or immediately erases from the primary, persisting, and dirty lists before clearing membership/container state, the container bind/unbind/advance/update helpers take ordered queue ids plus explicit live skip/advance results because C++ iterates mutable vectors directly while the binary layer does not own live bind state, and `data_bind_stateful_advance` takes source-bound/collapsed state explicitly because C++ checks live `m_Source` plus `DataBind::Flag::Collapsed` before delegating to the resolved converter's `advance()`. C++ probe comparisons now cover the push/persisting flags for imported artboard and state-machine data binds, while synthetic Rust coverage pins the flag combinations, computed-property, `Solo.activeComponentId`, `Shape.length`, scroll-derived property, bindable asset/view-model target exceptions, collapsed-target can-skip/display exceptions, collapse dirty-update requests, add-dirt suppression/duplicate/context-invalidation/collapsed-queue branches, addDataBind deferred/persisting/context branches, source/clearSource dependent and `ArtboardComponentList` reset branches, bind/target/unbind context creation, converter reset/unbind, observer removal/subscription, null target assignment, dirt scheduling branches, initialize/relink collapsable and container-rebuild branches, DataBindContext bind-from-context no-context/source-change/source-match/missing-source/name-based/converter branches, container bind/unbind/advance ordering, updateDataBinds early-return/drain/pending-flush behavior, updateDataBind ordering/source-context-target guards, removeDataBind deferred/immediate cleanup, sort-order swaps, to-target queue branch, and two-way-to-source branch. A hand-built Rust test pins converter-dependent update, non-stable sort tail behavior, and `DataBind::advance()` guard/delegation behavior. C++ source audits now pin the `DataBindFlags` bit layout, `ComponentDirt` bit layout, internal data-bind flag layout, helper bodies, `canSkip()`/`collapse()`/`addDirt()`/`source()`/`clearSource()`/`bind()`/`target()`/`unbind()`/`initialize()`/`relinkDataBind()`/`update()`/`updateSourceBinding()`/`updateDependents()`/`advance()` bodies, `DataBindContext::resolvePath()`/`bindFromContext()`, `Component::addCollapsable()`, artboard/state-machine `rebuildDataBind()`, container queue members, add behavior, remove cleanup, context bind/unbind, container advance, dirty queue selection, update order, sort order, and drain/flush order for future scheduler work. Full artboard/data-bind lifecycle scheduling remains a future runtime-state slice.

Artboard-backed view-model resolution is pinned through the C++ probe. Rust exposes `RuntimeFile::resolved_view_model_for_artboard(_object)`, matching the `File::viewModel(Artboard.viewModelId)` lookup used by C++ `File::createViewModelInstance(Artboard*)` and `createDefaultViewModelInstance(Artboard*)`.

View-model name lookup semantics are pinned through the C++ probe. Rust exposes `RuntimeFile::view_model_property_named(_bytes)`, `view_model_instance_named(_bytes)`, and `view_model_instance_value_named(_for_object)`, matching C++ linear first-match behavior for `ViewModel::property(name)`, `ViewModel::instance(name)`, and completed `ViewModelInstance::propertyValue(name)`, including duplicate names.

View-model symbol lookup semantics are pinned through the C++ probe. Rust exposes `RuntimeFile::view_model_property_for_symbol` and `view_model_instance_value_for_symbol(_object)`, matching C++ `SymbolType` lookup behavior, including first-match property scans by `symbolTypeValue`, last-wins instance-value symbol maps, and `ViewModelPropertySymbolListIndex` mapping to `itemIndex` for values.

View-model default-instance semantics are pinned through the C++ probe. Rust exposes `RuntimeFile::view_model_default_instance`, matching C++ `ViewModel::defaultInstance()` by resolving the first imported instance for a view model or no instance when the C++ vector is empty.

View-model instance property-id lookup semantics are pinned through the C++ probe. Rust exposes `RuntimeFile::view_model_instance_value_for_property_id(_object)`, matching C++ `ViewModelInstance::propertyValue(uint32_t)` first-match behavior over imported values, including duplicate property IDs and missing IDs.

`ViewModelInstanceListItem` target instance resolution is pinned through the C++ probe. Rust exposes `RuntimeFile::referenced_view_model_instance_for_list_item(_object)`, matching the `viewModelId`/`viewModelInstanceId` lookup C++ performs while completing view-model properties.

`ArtboardComponentList` list-item artboard selection is pinned through the C++ probe. Rust exposes `RuntimeFile::artboard_component_list_map_rules(_for_object)` and `resolved_artboard_for_artboard_component_list_item(_objects)`, matching `ArtboardListMapRule::onAddedDirty` registration plus `ArtboardComponentList::findArtboard` explicit-rule/fallback/null behavior.

`ViewModelInstance::propertyFromPath` semantics are pinned through the C++ probe. Rust exposes `RuntimeFile::view_model_instance_property_from_path(_for_object)`, matching direct property-id path traversal and nested `ViewModelInstanceViewModel` reference traversal.

`DataContext` view-model lookup semantics are now pinned through the C++ probe. Rust exposes absolute and relative property/instance lookup helpers over current-plus-parent view-model instance chains, including manifest-name relative paths, `ViewModelInstanceViewModel` traversal, and C++ parent fallback behavior.

`DataEnum` lookup semantics are pinned through the C++ probe. Rust exposes key and index lookup helpers matching C++ `DataEnum::value(...)` and `DataEnum::valueIndex(...)`, including empty-value fallback to the key, duplicate-key first-match behavior, and missing/out-of-range results.

`ViewModelPropertyEnumCustom` data-enum resolution is pinned through the C++ probe. Rust exposes property-level enum lookup helpers that match C++ `dataEnum(m_Enums[enumId])` timing, including invalid enum IDs and the case where a property appears before the enum it references and therefore is not backfilled later. `ViewModelPropertyEnumSystem` is modeled as C++'s static empty enum.

`ViewModelInstanceEnumRuntime` imported-data views are pinned through the C++ probe. Rust exposes helpers for current enum key, runtime value index, available keys, and enum type, including C++'s out-of-range fallback to an empty key and index `0` and the system enum static-empty behavior.

`ViewModelInstance*Runtime` imported value views are now pinned through the C++ probe. Rust exposes `RuntimeFile::view_model_instance_value_data_type(_for_object)` plus typed helpers for number, string, boolean, color, list size, trigger count, view-model index, symbol-list-index value, asset index, and artboard index, matching C++ runtime wrapper data types where wrappers exist and the data-bind graph's data-type mapping for view-model and symbol-list-index values.

Source-side data-binding values are pinned through the C++ probe as well. Rust exposes `RuntimeFile::view_model_instance_source_data_value(_for_object)`, matching the `DataValue*` payload shape produced by C++ `DataBindContextValue::syncSourceValue()` before converter execution.

Imported converter execution has started at the binary layer. Rust exposes `RuntimeFile::data_converter_convert(_for_object)` and a `RuntimeConvertedDataValue` result shape for forward `DataConverter::convert()` overrides, with direct C++ sample coverage for boolean negate, trigger increment, `ToNumber` raw-byte `std::atof`-style decimal and hex string numeric-prefix parsing, `ToString` including custom color-format markers, rounder, string trim/pad/remove-zeros, operation-value arithmetic, operation-view-model arithmetic against imported view-model instance chains, the abstract `DataConverter` base method's pass-through dispatch, concrete `DataConverterOperation` inherited pass-through, non-scripting `ScriptedDataConverter` inherited pass-through, list-to-length, number-to-list list-size outputs plus file-backed generated item shapes, range mapper flags plus imported cubic/elastic interpolator transforms, fresh first-run `DataConverterInterpolator` pass-through, direct elapsed `DataConverterInterpolator` state for number/color smoothing, deterministic formula functions plus supplied-random formula ranges over imported output queues, and forward group composition. `RuntimeFile::data_converter_bind_context_effect(_for_object)`, `data_converter_unbind_effect(_for_object)`, `data_converter_update_effect(_for_object)`, `data_converter_reset_effect(_for_object)`, `data_converter_mark_dirty_effect(_for_object)`, `data_converter_property_change_effect(_for_object)`, and `data_converter_add_dirty_data_bind_effect(_for_object)` cover adjacent C++ converter lifecycle entry points before and around conversion, including group child dispatch, operation-view-model/formula source binding effects, owned data-bind update/unbind behavior, reset propagation, dirty propagation, and the audited property-change dirty hooks. `RuntimeConvertedDataValue::GeneratedList` carries the generated list item view-model id and value core types for valid `DataConverterNumberToList` converters, while still returning an empty generated list for missing file/view-model cases like C++. `RuntimeFile::data_converter_convert_with_context(_for_object)` and `data_bind_convert_with_context(_for_object)` pass a C++-style `DataContext` view-model instance chain for context-aware converters, including bound, missing-source, and unbound `DataConverterOperationViewModel` behavior. `RuntimeFile::data_bind_convert(_for_object)` covers data-bind-aware system converters by passing imported `DataBind.flags` through the C++ direction-sensitive operation-value branch. Formulas needing random values return explicit `None` through the ordinary stateless API and evaluate through `data_converter_convert_with_formula_randoms`/`data_converter_reverse_convert_with_formula_randoms` when the caller supplies random values. Direct interpolator smoothing uses `RuntimeDataConverterInterpolatorState` with explicit `convert`/`reverse_convert`/`advance` calls; grouped stateful converter execution now uses `RuntimeDataConverterState` with explicit stateful forward/reverse/advance calls for `DataConverterGroup` pipelines containing interpolators, and data-bind advance delegates to the same state via `data_bind_stateful_advance(_for_object)` once the caller supplies live source/collapsed state. Full artboard/data-bind lifecycle scheduling around converter state remains a future runtime-state slice. Unknown future converter subclasses return `None` so future parity gaps remain explicit.

Reverse converter execution is pinned for the deterministic, supplied-random, and stateful interpolator/group subset as well. `RuntimeFile::data_converter_reverse_convert(_for_object)` matches C++ reverse group order, boolean negate, operation-value inverse arithmetic, context-aware operation-view-model inverse arithmetic, range-mapper reverse mapping including imported cubic/elastic interpolator transforms, fresh first-run `DataConverterInterpolator` pass-through, formula reverse conversion through C++'s forward formula evaluator, and base pass-through reverse behavior for converters that do not override `reverseConvert()`. `RuntimeFile::data_converter_reverse_convert_with_formula_randoms` mirrors random formula reverse evaluation when supplied random values are available, `RuntimeFile::data_converter_interpolator_reverse_convert` mirrors C++ `DataConverterInterpolator::reverseConvert()` delegating to the same stateful smoothing path as forward conversion, and `RuntimeFile::data_converter_stateful_reverse_convert` shares interpolator state across reverse `DataConverterGroup` pipelines. `RuntimeFile::data_bind_reverse_convert(_for_object)` covers the same direction-sensitive system converter branch, while `data_converter_reverse_convert_with_context(_for_object)` and `data_bind_reverse_convert_with_context(_for_object)` pass the imported view-model instance chain for context-aware reverse conversion.

The corpus sweep also compares exact Rust/C++ import-result categories for fixtures that C++ rejects with a classified `ImportResult`. This pinned `solar-system.riv`: C++ decodes all objects, then rejects the file as `malformed` during final import-stack resolution because a surviving `Drawable` has an invalid `blendModeValue`. Rust now mirrors that C++ artboard-local validation path, including `Mesh::onAddedClean` rejection of missing triangle index buffers and triangle indices outside the attached `MeshVertex`/subclass list.

The binary arena now keeps raw bytes for encoded fields while also exposing typed C++ byte-decoder views: encoded `List<Id>` fields such as `DataBindPath.path`, `DataBindContext.sourcePathIds`, and `DataConverterOperationViewModel.sourcePathIds` decode as `uint32_t` ID buffers, `RuntimeFile::data_bind_path_for_referencer(_object)` models C++ `DataBindPathImporter` latest-unclaimed claim semantics plus inline path fallback, `Mesh.triangleIndexBytes` decodes as the `uint16_t` index buffer C++ uses for mesh validation, `FileAsset.cdnUuid` formats through the same byte-reordered `cdnUuidStr()` shape as C++, FileAsset extension/unique-name/unique-filename helpers mirror C++ including raw-byte asset names, and `RuntimeFile::manifest()` decodes `ManifestAsset` name/path tables from attached in-band `FileAssetContents`, including signed-key and partial malformed decode state. Later graph/runtime layers can consume these without reimplementing embedded byte parsing at each call site.

File-level runtime graph support has started with C++ artboard, authored animation/state-machine, enum, view-model, and asset ownership. `RuntimeFile::artboards()`/`artboard(index)`/`default_artboard()`/`artboard_named(_bytes)` mirror `File::artboard(...)` accessors by returning imported artboards in file order, `resolved_artboard_for_referencer(_object)` mirrors `BackboardImporter::resolve()` for `NestedArtboard` and `ScriptInputArtboard` `artboardId` references, `resolved_handle_source_for_joystick(_object)` mirrors `Joystick::onAddedDirty` handle-source resolution, `resolved_x_animation_for_joystick(_object)` and `resolved_y_animation_for_joystick(_object)` mirror `Joystick::onAddedClean` axis animation resolution, `data_bind_path_for_referencer(_object)` mirrors `DataBindPathImporter` claim order for path referencers while preserving inline path-byte fallback, `scroll_physics()`/`scroll_physics_object(index)` and `resolved_scroll_physics_for_constraint(_object)` mirror `BackboardImporter::addPhysics` plus `ScrollConstraint.physicsId` resolution, `artboard_animations()`/`artboard_animation(index)`/`artboard_animation_named(_bytes)` and `artboard_state_machines()`/`artboard_state_machine(index)`/`artboard_state_machine_named(_bytes)` mirror authored `Artboard::animation(...)` and explicit `Artboard::stateMachine(...)` binary collections, `artboard_linear_animations()` exposes C++ keyed-object/keyed-property pruning plus first-keyframe grouping for validated linear animations, `artboard_state_machine_graphs()` exposes C++ state-machine layer/input/listener/action/listener-input-type/data-bind child ownership for authored state machines, `artboard_data_binds()` exposes C++ artboard-owned data binds with resolved converter, target object, and target-local IDs when the target is an artboard-local component, `data_converters()`/`data_converter(index)` expose the imported backboard converter list, `data_converter_interpolators()` exposes the pre-artboard backboard `KeyFrameInterpolator` list, `resolved_data_converter_for_data_bind(_object)`, `resolved_data_converter_for_group_item(_object)`, `resolved_interpolator_for_data_converter(_object)`, and `resolved_view_model_for_number_to_list_converter(_object)` model C++ converter, interpolator, and number-to-list view-model resolution, `data_converter_group_items()` models latest-`DataConverterGroupImporter` item ownership, `data_enums()`/`data_enum(index)` mirror `File::enums()` exact membership and latest-`DataEnumCustom` value ownership, data-enum lookup helpers mirror C++ `DataEnum::value(...)`/`valueIndex(...)`, enum-property lookup helpers mirror C++ import-time `ViewModelPropertyEnumCustom` enum pointer resolution, enum instance helpers mirror `ViewModelInstanceEnumRuntime` imported-data views, `view_models()`/`view_model(index)`/`view_model_named(_bytes)` mirror `File::viewModel(...)` order, raw-byte name lookup, latest-`ViewModelImporter` property ownership, import-time `ViewModelInstance` attachment by existing view-model id, and latest-importer ownership for `ViewModelInstanceValue` plus `ViewModelInstanceListItem` children, while `file_assets()`/`file_asset(index)` mirror `File::assets()` by returning imported backboard file assets while excluding `ManifestAsset`; `resolved_file_asset_for_object()` models `BackboardImporter::resolve()` index lookup plus the concrete `setAsset()` type filters for `Image`, `AudioEvent`, `TextStyle`, and scripted object referencers.

Scroll-physics import behavior is now source-audited at the C++ method-body level: `ScrollPhysics::import` must add to the latest `BackboardImporter`, and `ScrollConstraint::import` must clone the indexed physics object before delegating to `Component::import`.

Artboard skinning, vertex deformation, NSlicer, and shape paint relationships are now exposed from `rive-binary` as well: `artboard_skins()`/`artboard_skin()` report imported `Skin` objects, their resolved skinnable `Mesh`/`PointsPath`, owned `Tendon` records, and each tendon's resolved `Bone`; `artboard_meshes()`/`artboard_mesh()` report `Mesh` objects, their `MeshVertex` children, and each vertex's resolved `Weight`, including duplicate-weight overwrite order; `artboard_paths()`/`artboard_path()` report `Path` objects, their `PathVertex` children, and resolved `Weight`/`CubicWeight` attachments; `artboard_n_slicer_details()`/`artboard_n_slicer_detail()` report `NSlicerDetails` implementers (`NSlicer` and `NSlicedNode`), ordered `AxisX`/`AxisY` registrations, and patch-indexed `NSlicerTileMode` registrations with C++ duplicate overwrite behavior; `artboard_shapes()`/`artboard_shape()` report `Shape` objects, their C++-registered paths including parametric path subclasses, registered shape paints with resolved mutators including forward-referenced mutators, linear/radial gradient stops before update-time sorting, attached `Feather` records, registered stroke effects, and `TargetEffect` links to `GroupEffect` targets; `artboard_shape_paint_containers()`/`artboard_shape_paint_container()` report non-empty exact `ShapePaintContainer` registrations beyond `Shape`, such as root `Artboard` and `TextStylePaint`. These surfaces are backed by synthetic C++ probe comparisons plus corpus structural comparisons for every C++-accepted fixture, and the probe now emits `NSlicerDetails` registration state directly rather than relying only on Rust-side lifecycle tests.

## #6: Minimal Artboard Graph Lifecycle

Blocked by: #5
Type: Prototype

### Question

What is the smallest useful Rust implementation of the artboard runtime graph?

### Answer

Resolved in `docs/prototypes/artboard-graph.md`. Added `crates/rive-graph`, `graph-inspect`, and the C++ probe comparison. The graph projection now builds compact artboard-local object slots matching C++ `Artboard::objects()`: component objects, artboard-owned user-input/interpolator objects, null/abstract slots, and validation-null slots for invalid targeted constraints, text styles, nested animations, scroll-bar constraints, and feather effects, excluding concrete non-components routed to other runtime lists. It also projects file-level assets/view-models/data-enums through `rive-binary`'s public `RuntimeFile` collection helpers and projects artboard animation/state-machine groupings, including C++'s auto-generated empty state machine for artboards with no authored animations or state machines. It resolves local `parentId` links, builds C++ child indexes including import-time `LayoutComponent`/`Artboard` style adoption, projects transform constraint registration lists, emits topological dependency-node order plus a filtered real-component order with cycle diagnostics, records C++ `Component::graphOrder` parity for root-reachable `buildDependencies` graph nodes, records the artboard-local subset of C++ `Component::dependents()` for later dirt scheduling, aggregates structural diagnostics for unresolved graph references and cycles, records capability flags for artboard/container/world-transform/transform/drawable categories, records static component-list map-rule tables, and exposes C++-style nullable draw relationships for `DrawTarget`, `DrawRules`, and `ClippingShape`, including source shape locals and clipped drawable locals. This maps the first lifecycle phases as `import`, `on_added_dirty`, `on_added_clean`, and a minimal `build_dependencies`, now including audited Joystick custom-handle dependencies, the static `ScrollConstraint -> ScrollBarConstraint` dependency, `ScrollConstraint -> layout-provider content child` dependencies, synthetic path-composer projections and dependency-node edges for imported shapes, clipping-shape dependencies on source shape path composers, follow-path target/parent dependencies, text-follow-path target/text dependencies, stroke/fill/feather path-builder dependencies, and explicit effect-parent dependencies. `make cpp-compare` validates object counts, local type keys, names, parent IDs, resolved parent local IDs, child lists, transform constraint lists, component dependent lists, component-list map rules, graph-order values, draw target/rule/clipping relationships, file-owned collections, and animation/state-machine grouping against C++.

## #7: Parent/Child And Dependency Graph

Blocked by: #6
Type: Prototype
Contract: `docs/prototypes/graph-runtime-contract.md`

### Question

How should Rust represent hierarchy and dependency ordering without pointer graphs?

### Answer

In progress. The graph projection uses stable local/global IDs, derived component indexes, C++ child indexes, C++ transform constraint registration lists, explicit dependency edges for audited C++ parent dependency hooks, targeted constraints, IK target dependencies, IK chain off-branch child dependencies, draw-target drawable references, draw-rule target references, clipping sources, skinning dependencies for exact C++ skinnables (`Mesh` and `PointsPath`), Joystick custom-handle dependencies, `ScrollConstraint -> ScrollBarConstraint` dependencies, and `ScrollConstraint -> layout-provider content child` dependencies, plus a topological dependency-node order with cycle diagnostics and a filtered real-component local-ID order for existing callers. `children` mirrors C++ `ContainerComponent::children()` insertion order, including duplicate additions and inherited `LayoutComponent::onAddedDirty` adoption of resolved `LayoutComponentStyle` objects for `LayoutComponent` descendants such as `Artboard`; it is not merely a sorted inverse of serialized `parentId`. `ComponentNode::dependent_locals` mirrors the artboard-local subset of C++ `Component::dependents()` after dependency construction and draw-target initialization, preserving `DependencyHelper::addDependent` duplicate suppression while leaving synthetic `PathComposer`/`TextVariationHelper` nodes in the dependency-node graph. `ArtboardGraph::diagnostics` aggregates missing parent, unresolved graph reference, dependency-cycle, dependency-node-cycle, and draw-target-cycle facts without rejecting or mutating the graph. `ComponentNode::graph_order` now separately mirrors C++ `Artboard::sortDependencies` for root-reachable `buildDependencies` graph nodes, including synthetic `PathComposer` positions while excluding draw-target/draw-rule/clipping-source projection edges. Structural parent/child hierarchy remains available through `children`, while C++ `onAddedClean` artboard host registries are projected as static `nested_artboards`, `component_lists`, and combined `artboard_hosts` lists for exact nested-artboard host variants and exact `ArtboardComponentList` objects; `ComponentListNode::map_rules` now mirrors C++ `ArtboardListMapRule::onAddedDirty -> ArtboardComponentList::addMapRule` table registration without admitting list instancing, virtualization, data-context binding, or map-rule selection. C++ `onAddedClean` joystick registration is projected as static `joysticks` and `joysticks_apply_before_update` facts, with resolved custom handle sources, x/y animation globals, and y-then-x nested remap animation dependents collected from keyed animation targets. C++ reset and advance registries are projected as static `resetting_components` and `advancing_components` lists from the exact `ResettingComponent::from` and `AdvancingComponent::from` switches. Static drawable ordering is projected as `drawable_order`, including real drawables, layout proxies, foreground reordering, and flattened draw-rule locals. Static draw-target ordering is projected as `draw_target_order`, including parent-ordered rule groups, synthetic root target dependencies for resolved target drawables, flattened-rule target dependencies, and draw-target cycle diagnostics; unresolved target IDs remain nullable `draw_targets` facts. Static shape render-path deformer caches are projected as `shape_deformers`, mapping each imported `Shape` to the first ancestor accepted by `RenderPathDeformer::from`, currently exact `NSlicedNode`, without admitting `NSlicer` deformation math, `Path::buildPath` deformer application, gradient deformer updates, or point deformation. Static skeletal caches are projected as `skeletal_bones` and `skeletal_skins`, mapping exact `Bone` child registration, IK ancestor peer constraints, exact `Mesh`/`PointsPath` skin ownership, and valid tendon-to-bone registrations without admitting bone transforms, IK solving, skin matrices, vertex deformation, or skin/transform dirt propagation. `ParentChild` dependency edges are now emitted only for C++ `buildDependencies()` hooks that call `parent()->addDependent(this)`, such as `TransformComponent`, exact `Mesh`, `Constraint`, `TextStyle`, `FocusData`, `SemanticData`, and `NSlicer`; import-only records such as `DrawTarget`, `DrawRules`, `AxisX`, and `TextValueRun` do not receive dependency-order parent edges. It also projects C++'s synthetic per-`Shape` `PathComposer` nodes from `rive-binary`'s imported shape/path registration facts and inserts them into the dependency-node graph with shape/path prerequisite edges. Targeted-constraint edges now follow C++ `TargetedConstraint::buildDependencies()` by ordering the constrained component after the target and by skipping the generic parent-child dependency because that override does not call `Super::buildDependencies`; `IKConstraint::buildDependencies()` adds the target-to-constraint edge, and its `onAddedClean` chain walk makes off-chain transform children of ancestor bones depend on the constrained tip bone without admitting IK solving or transform dirt propagation; `Skin::buildDependencies()` is modeled by skipping its generic parent-child edge, making skinned `Mesh` and `PointsPath` objects depend on their skin, and making skins depend on tendon bones plus IK peer constraint parents; `Joystick::buildDependencies()` is modeled by skipping its generic parent-child edge and adding parent/handle-source edges only when a custom handle source resolves to a `TransformComponent`; `ScrollBarConstraint::buildDependencies()` is modeled by making the scroll bar depend on its resolved `ScrollConstraint` while preserving the inherited parent dependency; `ScrollConstraint::buildDependencies()` is modeled by making exact C++ `LayoutNodeProvider` content children depend on the scroll constraint; `PathComposer::buildDependencies()` is modeled by making the synthetic path composer depend on its owning shape and each registered shape path; `ClippingShape::buildDependencies()` is modeled by making each source shape path composer a prerequisite of the clipping shape and skipping the generic parent-child dependency because the override does not call `Super::buildDependencies`; `FollowPathConstraint::buildDependencies()` is modeled by making target shape/path composers prerequisites of the constraint, falling back to direct path dependencies for shape-less paths, making the constrained parent depend on the constraint, and skipping the inherited targeted edge; exact `ListFollowPathConstraint` children under exact `ArtboardComponentList` parents are also recorded as static `ConstrainableList::addListConstraint` registrations while `constrainList()`, list layout, and virtualization stay out of scope; `TextFollowPathModifier::buildDependencies()` is modeled by making target shape path composers or direct target paths prerequisites of the modifier and making the owning `Text` depend on the modifier; `TextVariationHelper::buildDependencies()` is modeled with a synthetic helper node for imported text styles that have axis/feature children, adding `artboard -> helper -> text` without admitting variable-font mutation; `Stroke::buildDependencies()` and effect-bearing `Fill::buildDependencies()` are modeled by resolving `ShapePaintContainer::pathBuilder()` to a synthetic path composer for shapes or a real path-builder component for other supported containers; `Feather::buildDependencies()` is modeled by making feathers depend on the owning paint container, routing shapes through their synthetic path composers; `GroupEffect::buildDependencies()` and `ScriptedPathEffect::buildDependencies()` are modeled as explicit effect-parent dependencies; `LinearGradient::buildDependencies()`, inherited by `RadialGradient`, is modeled by making gradients depend on the first owning `Node` above their shape paint, or the immediate paint container when no node exists, while leaving `updateDeformer()` to a later runtime/deformer slice. Generic parent-child edges are skipped for audited no-super shape/paint/effect families and for imported records that do not add C++ dependency edges. Nested-artboard host updates, component-list layout, list artboard instancing, virtualization, map-rule selection, cloning, joystick application, animation advancement, reset execution, `advanceComponent()` execution, component update scheduling, live data-binding behavior, deformer math, `sortDrawOrder()`, render linked-list mutation, active draw-target linked lists, clipping-stack operations, and draw commands remain out of scope for this graph projection. Next, use the same admission rule to add the remaining graph edge families one at a time. Avoid `Rc<RefCell<dyn Core>>`; all cross-object relationships should remain IDs resolved through graph context.

Current #7 scope also includes `layout_constraint_registrations`, which records the reciprocal `LayoutNodeProvider::addLayoutConstraint` and `ScrollConstraint::addLayoutChild` facts for exact `ScrollConstraint` layout-provider content children. This is still a static graph projection: layout solving, virtualization, Yoga updates, `constrainChild`, and scroll execution remain future runtime work.

Current #7 scope also includes artboard-owned and state-machine-owned `data_binds`: artboard-owned binds record C++ `DataBindContainer` membership and initialized `sortDataBinds()` ordering, while state-machine-owned binds now consume the verified `RuntimeFile::artboard_state_machine_graphs` ownership so bindable-property targets stay out of artboard registrations and component-target binds stay out of state-machine registrations. Data-context binding, dirty queues, property observers, converter execution, source/target mutation, state-machine execution, and data-bind advancement remain out of `rive-graph`.

Current #7 scope also includes state-machine `scripted_objects`, recording C++ `StateMachineImporter::addScriptedObject`, `StateMachine::addScriptedObject`, and `ScriptedObjectImporter::addInput` registrations while leaving script asset initialization, VM registration, cloning, script input hydration, script execution, and state-machine execution to later runtime crates.

Current #7 scope also includes `shape_paint_containers`, recording C++ shape-paint container membership, paint mutators, feathers, gradient stops, stroke effects, and target-effect group links while leaving paint mutation, effect execution, gradient stop sorting, path-effect application, renderer paint allocation, draw commands, and GPU work to later runtime/render crates.

Current #7 scope also includes `n_slicer_details`, recording exact C++ `NSlicerDetails` owner recognition plus ordered X/Y axes and patch-indexed tile-mode registrations while leaving NSlicer deformation math, patch solving, layout updates, path deformation, and render-path mutation to later deformer/runtime crates.

Current #7 scope also includes `meshes` and `paths`, recording ordered `MeshVertex`/`PathVertex` registration plus resolved `Weight`/`CubicWeight` attachment facts from C++ `onAddedDirty`. Vertex deformation, skinning math, `Path::buildPath`, contour/path tessellation, weight blending, and dirty propagation remain later runtime/deformer work.

## #8: Dirt Propagation And Transform Update

Blocked by: #7
Type: Prototype
Contract: `docs/prototypes/dirt-transform-runtime-contract.md`

### Question

Can Rust reproduce the C++ dirt scheduler and transform update semantics?

### Answer

Resolved. Added `crates/rive-runtime` with C++ `ComponentDirt` bit parity, mutable per-component dirt state, graph-dependent recursive dirtying through `ComponentNode::dependent_locals`, C++ `graphOrder` traversal, dirt-depth restart behavior, max-pass guard reporting, collapsed-component skip behavior, and basic transform/render-opacity updates. The C++ probe now has an opt-in `--runtime-update` mode, and `make cpp-compare` runs a runtime C++ comparison for initial update state. Animation, state machines, data binding, constraints, layout, cloning, draw commands, rendering, text, scripting, and audio remain later runtime slices.

## #9: Artboard Instancing And Cloning

Blocked by: #8
Type: Prototype
Contract: `docs/prototypes/artboard-instancing-runtime-contract.md`

### Question

How should source artboards and mutable artboard instances be separated?

### Answer

In progress. The next runtime seam separates imported source artboard data from mutable instance state without attempting full C++ clone parity yet. The first slice preserves artboard-local instance slots, keeps source global IDs/type names distinct from mutable component state, exposes mutable transform properties needed by animation, and compares initial plus mutated instance transform state against C++ cloned `ArtboardInstance` probe output. Full object cloning, data-bind retargeting, nested artboards, state-machine execution, layout, rendering, text, scripting, and audio remain out of scope for this slice.

## #10: Draw Graph Without Rendering

Blocked by: #9
Type: Prototype
Contracts: `docs/prototypes/draw-target-placement-order-runtime-contract.md`, `docs/prototypes/clipping-proxy-draw-order-runtime-contract.md`, `docs/prototypes/save-operation-elision-draw-order-runtime-contract.md`, `docs/prototypes/sorted-drawable-hidden-facts-runtime-contract.md`, `docs/prototypes/runtime-draw-command-stream-contract.md`, `docs/prototypes/runtime-empty-clip-command-stream-contract.md`, `docs/prototypes/runtime-path-empty-clip-command-stream-contract.md`, `docs/prototypes/runtime-shape-paint-command-payload-contract.md`, `docs/prototypes/runtime-solid-color-paint-payload-contract.md`, `docs/prototypes/runtime-straight-points-path-payload-contract.md`, `docs/prototypes/runtime-cubic-points-path-payload-contract.md`, `docs/prototypes/runtime-rounded-points-path-payload-contract.md`, `docs/prototypes/runtime-shape-paint-blend-mode-payload-contract.md`, `docs/prototypes/runtime-gradient-paint-payload-contract.md`, `docs/prototypes/runtime-feather-paint-payload-contract.md`, `docs/prototypes/runtime-rectangle-parametric-path-payload-contract.md`, `docs/prototypes/runtime-ellipse-parametric-path-payload-contract.md`, `docs/prototypes/runtime-polygon-parametric-path-payload-contract.md`, `docs/prototypes/runtime-star-parametric-path-payload-contract.md`, `docs/prototypes/runtime-triangle-parametric-path-payload-contract.md`, `docs/prototypes/runtime-weighted-points-path-input-contract.md`, `docs/prototypes/runtime-weighted-points-path-command-contract.md`, `docs/prototypes/runtime-inner-feather-path-payload-contract.md`, `docs/prototypes/runtime-drawable-will-draw-prereq-contract.md`, `docs/prototypes/runtime-trim-path-line-effect-contract.md`

### Question

Can Rust produce the same derived draw order without implementing a renderer?

### Answer

Partially open. `crates/rive-graph` now exposes draw target/rule/clipping relationships; `drawable_order` mirrors C++ `m_Drawables` initialization through foreground reordering, layout proxy insertion, and flattened draw-rule assignment; `draw_target_order` mirrors C++ `m_DrawTargets` initialization through parent-ordered rule groups, synthetic root target dependencies, and flattened-rule target dependencies; `sorted_drawable_order` mirrors `Artboard::sortDrawOrder()` through active-target grouping, before/after target placement splicing, clipping proxy start/end interleaving, `clearRedundantOperations()` save-operation elision, and imported hidden drawable facts; `PathComposerNode` records imported path hidden facts; `PathVertexNode` records imported `Weight`/`CubicWeight` payload words, while `SkeletalSkinNode`/`SkeletalTendonNode` record the skin matrix and tendon inverse-bind facts needed for weighted path deformation; `DrawableOrderNode`/`SortedDrawableNode` record resolved `ImageAsset` and referenced-artboard draw prerequisites; and `rive-runtime` exposes a renderer-independent logical draw command stream for simple runtime `willDraw()` filtering, image/nested-artboard reference-gated `willDraw()` filtering, pathless/hidden/collapsed source-path empty-clip suppression, structural `Shape::draw()` paint payloads for visible fills/strokes with path-kind selection, SolidColor authored/render-color paint state, gradient paint state for local-space shader inputs, shape-paint authored/resolved blend mode state, scalar feather paint payloads plus no-effect inner-feather derived path payloads, line-only `TrimPath` effect path payloads, straight/cubic/rounded and skinned weighted `PointsPath` raw command payloads, rounded `Rectangle` parametric raw command payloads, `Ellipse` parametric raw command payloads, rounded `Polygon` parametric raw command payloads, rounded `Star` parametric raw command payloads, and `Triangle` parametric raw command payloads for supported `move`/`line`/`cubic`/`close` geometry. Remaining work: broader effect path mutation (`DashPath`, `TargetEffect`/`GroupEffect`, scripted effects, cubic/curve and multi-contour trim/dash measuring), feather renderer save/translate/clip/draw behavior, effect-path-aware feather rebuilding, text/list/scripted drawability, and renderer integration. This should remain headless and comparable in tests before any GPU work begins.

## #11: Animation And State Machine Integration

Blocked by: #8, #9
Type: Prototype
Contracts: `docs/prototypes/linear-animation-runtime-contract.md`, `docs/prototypes/linear-animation-instance-runtime-contract.md`, `docs/prototypes/state-machine-animation-state-runtime-contract.md`, `docs/prototypes/state-machine-input-runtime-contract.md`, `docs/prototypes/state-machine-timed-transition-runtime-contract.md`, `docs/prototypes/state-machine-exit-time-runtime-contract.md`, `docs/prototypes/state-machine-exit-handoff-runtime-contract.md`, `docs/prototypes/state-machine-percentage-timing-runtime-contract.md`, `docs/prototypes/state-machine-cubic-transition-interpolator-runtime-contract.md`, `docs/prototypes/state-machine-elastic-transition-interpolator-runtime-contract.md`, `docs/prototypes/state-machine-early-exit-runtime-contract.md`, `docs/prototypes/state-machine-random-transition-runtime-contract.md`, `docs/prototypes/state-machine-fire-event-runtime-contract.md`, `docs/prototypes/state-machine-scheduled-listener-action-runtime-contract.md`, `docs/prototypes/state-machine-scheduled-listener-input-runtime-contract.md`, `docs/prototypes/state-machine-blend-state-1d-runtime-contract.md`, `docs/prototypes/state-machine-blend-state-direct-runtime-contract.md`, `docs/prototypes/state-machine-blend-transition-runtime-contract.md`, `docs/prototypes/state-machine-transition-animation-reset-runtime-contract.md`, `docs/prototypes/state-machine-direct-blend-transition-runtime-contract.md`, `docs/prototypes/state-machine-blend-percentage-duration-runtime-contract.md`, `docs/prototypes/state-machine-blend-percentage-exit-time-runtime-contract.md`, `docs/prototypes/state-machine-blend-pause-on-exit-runtime-contract.md`, `docs/prototypes/state-machine-blend-early-exit-runtime-contract.md`, `docs/prototypes/state-machine-blend-random-transition-runtime-contract.md`, `docs/prototypes/state-machine-bindable-blend-source-audit.md`, `docs/prototypes/state-machine-bindable-number-blend-runtime-contract.md`, `docs/prototypes/state-machine-bindable-number-mutation-audit.md`, `docs/prototypes/state-machine-bindable-number-mutation-runtime-contract.md`, `docs/prototypes/state-machine-viewmodel-number-condition-audit.md`, `docs/prototypes/state-machine-viewmodel-number-condition-runtime-contract.md`, `docs/prototypes/state-machine-fire-trigger-runtime-contract.md`, `docs/prototypes/state-machine-artboard-comparand-runtime-contract.md`, `docs/prototypes/state-machine-viewmodel-trigger-reset-runtime-contract.md`, `docs/prototypes/state-machine-viewmodel-boolean-condition-runtime-contract.md`, `docs/prototypes/state-machine-viewmodel-integer-number-condition-runtime-contract.md`, `docs/prototypes/state-machine-viewmodel-color-condition-runtime-contract.md`, `docs/prototypes/state-machine-viewmodel-string-condition-runtime-contract.md`, `docs/prototypes/state-machine-viewmodel-enum-condition-runtime-contract.md`, `docs/prototypes/state-machine-viewmodel-asset-condition-runtime-contract.md`, `docs/prototypes/state-machine-viewmodel-pointer-condition-runtime-contract.md`, `docs/prototypes/state-machine-viewmodel-trigger-condition-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-component-literal-condition-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-component-pair-condition-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-artboard-component-condition-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-component-viewmodel-condition-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-component-artboard-unsupported-condition-audit.md`
Additional current contract: `docs/prototypes/state-machine-component-viewmodel-pointer-unsupported-condition-audit.md`
Additional current contract: `docs/prototypes/state-machine-component-viewmodel-trigger-artboard-condition-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-runtime-artboard-dimension-comparand-contract.md`
Additional current contract: `docs/prototypes/linear-animation-color-keyframe-runtime-contract.md`
Additional current contract: `docs/prototypes/linear-animation-bool-keyframe-runtime-contract.md`
Additional current contract: `docs/prototypes/linear-animation-uint-keyframe-runtime-contract.md`
Additional current contract: `docs/prototypes/linear-animation-string-keyframe-runtime-contract.md`
Additional current contract: `docs/prototypes/linear-animation-id-keyframe-runtime-contract.md`
Additional current contract: `docs/prototypes/linear-animation-callback-keyframe-runtime-contract.md`
Additional current contract: `docs/prototypes/linear-animation-callback-keyframe-loop-edge-runtime-contract.md`
Additional current contract: `docs/prototypes/linear-animation-callback-keyframe-remaining-edge-runtime-contract.md`
Additional current contract: `docs/prototypes/linear-animation-instance-callback-report-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-default-viewmodel-number-binding-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-default-viewmodel-boolean-binding-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-default-viewmodel-string-binding-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-default-viewmodel-color-binding-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-default-viewmodel-enum-binding-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-default-viewmodel-asset-binding-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-default-viewmodel-artboard-binding-runtime-contract.md`
Additional current contract: `docs/prototypes/state-machine-default-viewmodel-trigger-binding-runtime-contract.md`
Current remaining-work audit: `docs/prototypes/state-machine-runtime-remaining-audit.md`

### Question

How should animations and state machines drive the graph scheduler?

### Answer

In progress. Direct `LinearAnimation::apply` parity is in place for transform `KeyFrameDouble` properties, `SolidColor.colorValue` `KeyFrameColor` properties, `Fill.isVisible`/`ShapePaint.isVisible` `KeyFrameBool` properties, `CustomPropertyEnum.propertyValue` `KeyFrameUint` and `KeyFrameId` properties, and `SemanticData.label` `KeyFrameString` properties, the narrow `LinearAnimationInstance` playback seam now has C++ time, speed, loop, work-area, spill, and keep-going parity while still routing application through the existing direct apply path, the first `StateMachineInstance` runtime seam can enter an `AnimationState` backed by an existing linear animation and advance/apply it through the animation-instance path, state machines are now controllable with runtime bool/number/trigger inputs plus simple input-condition transitions, timed `AnimationState -> AnimationState` transition mixing for millisecond durations is in place, absolute millisecond exit-time gating for simple animation-state transitions is in place, exit-transition handoff now covers pause-on-exit/source hold plus spilled-time handoff into the target state, percentage timing for simple transition duration and exit time is in place, `CubicEaseInterpolator` plus `ElasticInterpolator` transition mix easing is in place, early exit/transition interruption for simple animation-state transitions is in place, random transition selection for random source states is in place, import-authored `StateMachineFireEvent` collection for simple state/transition changes is in place, scheduled `ListenerFireEvent` actions attached to states/transitions are in place, scheduled listener input-change actions (`ListenerBoolChange`, `ListenerNumberChange`, and `ListenerTriggerChange`) are in place, the first `BlendState1DInput` runtime seam is in place for root-number-input blends over authored linear animations, `BlendStateDirect`/`BlendAnimationDirect` runtime blending is in place for input and authored mix-value sources, blend-state transition exit timing is in place for simple supported blend-state sources using `BlendStateTransition.exitBlendAnimationId`, transition animation reset parity is in place for transform `KeyFrameDouble` transition mixing, including a C++ probe-backed multi-animation `BlendState1DInput -> AnimationState` fixture, direct-blend transition coverage is in place for a `BlendStateDirect -> AnimationState` fixture using both authored mix-value and input-driven direct blend animations, percentage-duration blend transitions are pinned to C++'s current zero-duration behavior because `StateTransition::mixTime` only derives percentage mix duration from `AnimationState` sources, percentage exit-time blend transitions are covered with differing source animation durations to prove `BlendStateTransition.exitBlendAnimationId` drives exit gating, pause-on-exit source-hold behavior is covered for `BlendState1DInput -> AnimationState` transitions, early-exit/interruption coverage is in place for active `BlendState1DInput -> AnimationState` transitions, random transition selection is covered for both supported blend-state sources (`BlendState1DInput` and `BlendStateDirect`), the first view-model/data-bind blend-source audit is in place, static per-instance `BindablePropertyNumber.propertyValue` reads are implemented for `BlendState1DViewModel` and `BlendAnimationDirect.blendSource=dataBindId` without live data-context binding, the smallest C++ data-bind update path has been audited, explicit per-instance `BindablePropertyNumber` mutation is implemented with C++ probe coverage for both supported bindable blend-source consumers, the number-only `TransitionViewModelCondition` shape has been audited, the narrow runtime slice now projects comparator children, applies the C++ data-context presence gate, and evaluates `BindablePropertyNumber` and `BindablePropertyInteger` versus `TransitionValueNumberComparator` with C++ probe coverage for no-context, static-value, and mutated-value cases, scheduled `StateMachineFireTrigger` actions now increment imported `ViewModelInstanceTrigger` counts through a minimal authored default view-model data-context binding with C++ probe coverage, static artboard width/height/ratio numeric transition comparands now match the C++ probe for matching and non-matching thresholds, explicit data-context advancement now resets bound view-model trigger counts with C++ probe coverage, boolean `TransitionViewModelCondition` comparisons now support imported and explicitly mutated `BindablePropertyBoolean` values with C++ probe coverage, color `TransitionViewModelCondition` comparisons now support imported and explicitly mutated `BindablePropertyColor` values with C++ probe coverage for no-context, static-equal, mutated-equal, and static-not-equal cases, string `TransitionViewModelCondition` comparisons now support imported and explicitly mutated `BindablePropertyString` values with C++ probe coverage for no-context, static-equal, mutated-equal, and static-not-equal cases, enum `TransitionViewModelCondition` comparisons now support imported and explicitly mutated `BindablePropertyEnum` values with C++ probe coverage for no-context, static-equal, mutated-equal, static-not-equal, and ordered-operation cases, asset `TransitionViewModelCondition` comparisons now support imported and explicitly mutated `BindablePropertyAsset` ids with C++ probe coverage for no-context, static-equal, mutated-equal, static-not-equal, and ordered-operation cases, and view-model pointer `TransitionViewModelCondition` comparisons now support null and root data-context `BindablePropertyViewModel` pointer identity with C++ probe coverage for no-context, root-equal, root-vs-null not-equal, null-equal, and ordered-operation cases. Remaining runtime work is now tracked in `docs/prototypes/state-machine-runtime-remaining-audit.md`: live view-model APIs and data-binding propagation, listener-owned dispatch and input routing, nested artboard/animation remapping, remaining callback dispatch and callback-target side effects, custom/scripted runtime behavior, and later layout/text/render integration.

Current #11 update: trigger/self `TransitionViewModelCondition` comparisons now support default-context `BindablePropertyTrigger` sources with C++ probe coverage for no-context, value-trigger comparator, self-comparator, and same-layer used-source suppression cases. This closes the earlier remaining-work item named `trigger/self view-model transition conditions`; Live view-model APIs, nested animation remapping, data binding update queues, bindable-property override propagation from real data contexts, listener-owned dispatch, hit testing, pointer/keyboard/gamepad inputs, draw/render behavior, custom/scripted interpolators beyond transition timing, animation keyframe callback events, `ListenerViewModelChange`, component transition comparands, full layout solving, relative/parent/nested view-model paths for fire triggers, and callback keyframe coverage remain later slices.

Current #11 update: scheduled `StateMachineFireTrigger` actions now have a
C++-pinned relative `DataBindPath` boundary for the default state-machine
scheduling path. The relative-path fixture imports a claimed
`DataBindPath(path=[manifestPathId], isRelative=true)` immediately before the
fire trigger; C++ leaves the default-context trigger count unchanged, so Rust
keeps the target unresolved for this shape while preserving the existing
absolute-path trigger behavior. Broader relative/name-resolved trigger paths
owned by listener or nested-artboard data contexts remain future slices.

Current #11 update: the first component-comparand runtime slice supports `TransitionPropertyComponentComparator` on the left with literal value comparators on the right for CoreRegistry double, bool, string/bytes, color, generic uint-as-number, and special enum/trigger/asset/artboard uint fields. C++ probe coverage includes static component values, mutable transform reads, and missing/unsupported target default-value comparisons. Component-vs-component, component-vs-view-model, artboard-vs-component, component-vs-artboard, and component view-model pointer comparisons remain later slices.

Current #11 update: component-vs-component `TransitionPropertyComponentComparator` pairs now support C++ compatible number, bool, string/bytes, color, generic uint, enum, trigger, asset, and artboard comparisons, including mutable transform reads and missing/unsupported target default-value behavior. Component-vs-view-model, artboard-vs-component, component-vs-artboard, and component view-model pointer comparisons remain later slices.

Current #11 update: left-artboard/right-component `TransitionViewModelCondition` comparisons now support C++ numeric-compatible `TransitionPropertyArtboardComparator` versus `TransitionPropertyComponentComparator` pairs. Coverage includes static artboard width/height/ratio values compared against static component numbers, mutable transform reads, missing/unsupported component target defaults, and generic uint-as-number component properties. Component-vs-view-model, component-left/artboard-right if a later C++ audit finds a supported shape, component view-model pointer comparisons, and full layout solving remain later slices.

Current #11 update: component/ViewModel scalar `TransitionViewModelCondition` comparisons now support `TransitionPropertyComponentComparator` paired with `TransitionPropertyViewModelComparator` in either order for number/integer, bool, string/bytes, color, enum, and asset kinds. Coverage includes data-context gating, static component values, mutable component transforms, mutated ViewModel bindable values, missing/unsupported component defaults, incompatible kind rejection, and both component-left/ViewModel-right and ViewModel-left/component-right direction. Component/ViewModel trigger, artboard, and pointer semantics, component-left/artboard-right if a later C++ audit finds a supported shape, and full layout solving remain later slices.

Current #11 update: component-left/artboard-right `TransitionViewModelCondition` comparisons have been audited and are intentionally unsupported because C++ only appends `TransitionPropertyArtboardComparator` comparable kinds on the left side. A C++ probe rejection test now guards Rust against adding a mirrored direction that C++ does not support. Full layout solving remains a later slice.

Current #11 update: component/ViewModel pointer `TransitionViewModelCondition` comparisons have been audited and are intentionally unsupported because C++'s `ComparisonShape::ViewModel` construction only accepts `TransitionPropertyViewModelComparator`, even though a component ViewModel comparand class is declared. C++ probe rejection coverage now guards both component-left/ViewModel-right and ViewModel-left/component-right directions. Full layout solving remains a later slice.

Current #11 update: component/ViewModel trigger and artboard `TransitionViewModelCondition` comparisons now support `TransitionPropertyComponentComparator` paired with `TransitionPropertyViewModelComparator` in either supported order for C++ `ComparisonShape::Uint32` semantics. Runtime trigger bindables now preserve imported `BindablePropertyTrigger.propertyValue` separately from trigger source/reset behavior, runtime artboard bindables preserve imported `BindablePropertyArtboard.propertyValue` with the C++ `BindablePropertyId` missing-id default, and C++ probe coverage guards data-context gating, trigger/artboard equality, not-equality, ordered-operation false behavior, and both comparator orders. Full layout solving remains a later slice.

Current #11 update: `TransitionPropertyArtboardComparator` width, height, and ratio conditions now read current `ArtboardInstance` dimensions at evaluation time instead of freezing imported dimensions while building state-machine transitions. This applies to artboard-vs-literal and left-artboard/right-component comparisons. C++ probe coverage mutates the artboard instance layout rectangle before state-machine evaluation to prove parity with `ConditionComparandArtboardProperty::value` reading `layoutWidth()`/`layoutHeight()`. Yoga/layout solving, layout dirt propagation, nested layout, and renderer viewport behavior remain later slices.

Current #11 update: `KeyFrameColor` linear-animation application now covers `SolidColor.colorValue` with C++ `colorLerp` channel rounding for both interpolation and sub-1.0 animation mix. Animated color values live on `ArtboardInstance`, renderer-independent draw commands read the current solid color/render color, and component color transition comparands read the same runtime color value. C++ probe coverage includes solid-color draw payload interpolation/mix and a component color condition evaluated after an external color animation. `KeyFrameCallback`, gradient-stop color mutation in gradient payloads, deformer color updates, renderer paint allocation, and animation callback events remain later slices.

Current #11 update: `KeyFrameBool` linear-animation application now covers inherited `ShapePaint.isVisible` for `Fill` draw filtering with C++ direct-assignment semantics: interpolation holds the current keyframe value and sub-1.0 animation mix does not blend bools. Animated bool values live on `ArtboardInstance`, renderer-independent draw commands read the current fill visibility, and component bool transition comparands read the same runtime bool value. C++ probe coverage includes hidden fill payload filtering after a bool animation and a component bool condition evaluated after an external bool animation. `KeyFrameCallback`, stroke visibility reveal from a statically invisible stroke, callback events, and generalized bool-driven runtime behavior remain later slices.

Current #11 update: `KeyFrameUint` linear-animation application now covers runtime uint overlays and component uint comparands using `CustomPropertyEnum.propertyValue` as the C++ probe-backed surface. Uint keyframes follow C++ direct-assignment semantics: interpolation holds the current keyframe value and sub-1.0 animation mix does not blend uints. Animated uint values live on `ArtboardInstance`; component enum/trigger/asset/artboard/integer comparands and uint-backed number comparands read the current runtime uint value. `KeyFrameCallback`, relationship relinking for animated ID-like uint fields, layout solving for animated layout-style uint fields, text/style rebuilds, callback events, and generalized uint-driven runtime behavior remain later slices.

Current #11 update: `KeyFrameString` linear-animation application now covers runtime string overlays and component string comparands using `SemanticData.label` as the C++ probe-backed surface. String keyframes follow C++ direct-assignment semantics: interpolation holds the current keyframe value and sub-1.0 animation mix does not blend strings. Animated string values live on `ArtboardInstance`; component string literal, pair, and ViewModel comparands read the current runtime string bytes. `KeyFrameCallback`, text shaping/layout rebuilds, accessibility/semantics propagation, data-binding propagation, renderer updates, callback events, and generalized string-driven runtime behavior remain later slices.

Current #11 update: `KeyFrameId` linear-animation application now routes through the runtime uint overlay with C++ `setUint` direct-assignment semantics using `CustomPropertyEnum.propertyValue` as the C++ probe-backed ID-typed surface. Interpolation holds the current keyframe value and sub-1.0 animation mix does not blend IDs. Component enum/trigger/asset/artboard/integer comparands and uint-backed number comparands read the current runtime ID value. `KeyFrameCallback`, relationship relinking for animated parent/asset/text-style ID fields, text/style rebuilds, data-binding propagation, renderer updates, callback events, and generalized ID-driven runtime behavior remain later slices.

Current #11 update: the first `KeyFrameCallback` runtime slice now imports callback keyframes and reports crossed `Event.trigger` frames into the existing state-machine reported-event vector with C++ `secondsTo - frameSeconds` delay semantics. C++ probe coverage exercises an `AnimationState` crossing an event callback frame and compares event local ID, core type, name, and delay. Plain `LinearAnimationInstance` listener dispatch, listener-owned routing, hit testing, pointer/keyboard/gamepad input dispatch, audio playback, open-url side effects, nested-artboard event propagation, custom-property trigger callbacks, nested trigger callbacks, view-model trigger callbacks, reverse/work-area/multi-bounce callback edge cases, and callback-driven data-binding behavior remain later slices.

Current #11 update: `KeyFrameCallback` event reporting now has dedicated loop-edge coverage for state-machine animations. C++ probes cover a forward `Loop` wrap and a `PingPong` end-frame bounce, including event local ID, core type, name, `secondsDelay`, current animation time, and `didLoop`. Reverse-playback loop edges, work-area loop edges, multi-bounce ping-pong advances, public listener dispatch, audio/open-url side effects, trigger callback targets, nested-artboard event propagation, and callback-driven data-binding behavior remain later slices.

Current #11 update: remaining `KeyFrameCallback` state-machine edge coverage now includes reverse playback from the animation end time, enabled work-area loop wrapping, and multi-bounce ping-pong advances, all pinned against C++ reported-event payloads and current-animation timing. Public scene/listener dispatch, audio/open-url side effects, trigger callback targets, nested-artboard event propagation, and callback-driven data-binding behavior remain later slices.

Current #11 update: plain public `LinearAnimationInstance` callback reporting is now probe-backed for `Event.trigger` keyframes through Rust's `advance_linear_animation_instance_with_events` seam. The C++ probe records callback payloads from `LinearAnimationInstance::advance(seconds, KeyedCallbackReporter*)`, and Rust verifies event local ID, core type, name, delay, and animation timing against it. `Scene::advanceAndApply` listener notification, audio/open-url side effects, callback targets other than `Event.trigger`, nested-artboard event propagation, and callback-driven data binding remain later slices.

Current #11 update: the first live source-to-target data-bind path is in place for state-machine-owned `DataBindContext` objects targeting cloned `BindablePropertyNumber.propertyValue`. When the default root view-model context is bound, Rust resolves `sourcePathIds` against the imported default `ViewModelInstance`, applies a `ViewModelInstanceNumber` source to the cloned bindable before state-machine layer evaluation, and verifies that an existing `BlendState1DViewModel` consumer observes the C++ value. External context binding, source mutation APIs, non-number bindables, converters, generalized update queues, relative/parent/nested paths, listener-owned data binding, and nested artboard propagation remain later slices.

Current #11 update: default-context source-to-target binding now covers cloned `BindablePropertyBoolean.propertyValue` as the first non-number bindable path. Rust resolves a default `ViewModelInstanceBoolean` source through `DataBindContext.sourcePathIds`, applies it before transition evaluation, and verifies a boolean `TransitionViewModelCondition` against C++. External context binding, source mutation APIs, string/color/enum/asset/artboard source binds, converters, generalized update queues, relative/parent/nested paths, listener-owned data binding, and nested artboard propagation remain later slices.

Current #11 update: default-context source-to-target binding now covers cloned `BindablePropertyString.propertyValue`. Rust preserves `ViewModelInstanceString` bytes resolved through `DataBindContext.sourcePathIds`, applies them before transition evaluation, and verifies a string `TransitionViewModelCondition` against C++. External context binding, source mutation APIs, color/enum/asset/artboard source binds, converters, generalized update queues, relative/parent/nested paths, listener-owned data binding, and nested artboard propagation remain later slices.

Current #11 update: default-context source-to-target binding now covers cloned `BindablePropertyColor.propertyValue`. Rust applies a default `ViewModelInstanceColor` resolved through `DataBindContext.sourcePathIds` before transition evaluation and verifies a color `TransitionViewModelCondition` against C++. External context binding, source mutation APIs, enum/asset/artboard source binds, converters, generalized update queues, relative/parent/nested paths, listener-owned data binding, and nested artboard propagation remain later slices.

Current #11 update: default-context source-to-target binding now covers cloned `BindablePropertyEnum.propertyValue`. Rust copies the raw default `ViewModelInstanceEnum.propertyValue` resolved through `DataBindContext.sourcePathIds` before transition evaluation, matching C++ `DataBindContextValueEnum` rather than clamping through runtime enum helpers, and verifies an enum `TransitionViewModelCondition` against C++. External context binding, source mutation APIs, asset/artboard source binds, converters, generalized update queues, relative/parent/nested paths, listener-owned data binding, `Solo` name mapping, and nested artboard propagation remain later slices.

Current #11 update: default-context source-to-target binding now covers cloned `BindablePropertyAsset.propertyValue` as observed by asset transition comparands. Rust copies the raw default `ViewModelInstanceAssetImage.propertyValue` resolved through `DataBindContext.sourcePathIds` before transition evaluation and verifies an asset `TransitionViewModelCondition` against C++. External context binding, source mutation APIs, artboard source binds, render-image/imageValue side effects, `Image.setAsset` target binding, converters, generalized update queues, relative/parent/nested paths, listener-owned data binding, and nested artboard propagation remain later slices.

Current #11 update: default-context source-to-target binding now covers cloned `BindablePropertyArtboard.propertyValue` as observed by component/view-model artboard transition comparands. Rust copies the raw default `ViewModelInstanceArtboard.propertyValue` resolved through `DataBindContext.sourcePathIds` before transition evaluation and verifies the supported artboard comparand shape against C++. External context binding, source mutation APIs, literal artboard comparators, `ArtboardReferencer` target remapping, nested artboard propagation, bound view-model propagation, converters, generalized update queues, relative/parent/nested paths, and listener-owned data binding remain later slices.

Current #11 update: default-context source-to-target binding now covers cloned `BindablePropertyTrigger.propertyValue` as observed by component/view-model trigger transition comparands. Rust copies the raw default `ViewModelInstanceTrigger.propertyValue` resolved through `DataBindContext.sourcePathIds` before transition evaluation while preserving the existing source identity used by trigger/self conditions and used-layer suppression. External context binding, source mutation APIs, trigger callback targets, listener-owned trigger dispatch, converters, generalized update queues, relative/parent/nested paths, callback-driven data binding, and nested artboard propagation remain later slices.

## #12: Data Binding Graph

Blocked by: #9, #11
Type: Prototype

### Question

How should Rust model data binding as a graph over the object graph?

### Answer

In progress. The scope boundary is defined in
`docs/prototypes/data-binding-graph-runtime-contract.md`: live data-binding
behavior belongs behind a runtime data-binding graph, not in additional
per-bindable shims on `StateMachineInstance` and not in `rive-binary`. The graph
must own concrete context binding, source lookup, target writes,
source-to-target and target-to-source propagation, dirty queues, converter
execution, observer/polling behavior, pending add/remove handling, re-entry
protection, external view-model contexts, and later relative/parent/nested path
resolution. The first implementation slice should introduce
`RuntimeDataBindGraph` and migrate the already-proven finite default-context
`propertyValue` binds behind it while preserving the current C++ probe results;
external contexts, public source mutation APIs, converters, reverse propagation,
relative paths, parent paths, nested paths, listener-owned data binding, and
nested artboard propagation remain follow-up slices.

Current #12 update: `RuntimeDataBindGraph` now owns state-machine data-context
presence, default-view-model context binding state, one default-context dirty
bit, and a sorted default source-to-target binding queue for the finite
`propertyValue` set already covered by C++ probes. `StateMachineInstance` no
longer carries eight per-type default dirty flags or eight per-type default
apply methods. The migration contract is
`docs/prototypes/data-binding-graph-default-context-migration-runtime-contract.md`.
External contexts, public source mutation APIs, converters, reverse
propagation, relative/parent/nested path lookup, listener-owned data binding,
and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: default source-to-target bindings are now graph edges over
explicit source and target node tables. `RuntimeDataBindGraphDefaultBinding`
stores source and target handles, source nodes carry the resolved values, and
target nodes carry cloned bindable target identities. The node-table contract is
`docs/prototypes/data-binding-graph-node-table-runtime-contract.md`. This is a
behavior-preserving foundation slice; the next live data-binding slice should
mutate graph source nodes or bind external context source nodes rather than
adding another direct state-machine target write path.

Current #12 update: the first graph-owned source mutation path is in place for
default `ViewModelInstanceNumber` sources. Rust exposes
`StateMachineInstance::set_default_view_model_number_source_for_data_bind`,
which mutates the selected `RuntimeDataBindGraph` source node and dirties the
default edge when the default context is bound. The C++ probe mirrors this with
`--runtime-set-default-view-model-source-number`, resolving
`DataBindContext.sourcePathIds` against the default view-model instance and
mutating the resolved `ViewModelInstanceNumber.propertyValue`. The contract is
`docs/prototypes/data-binding-graph-default-number-source-mutation-runtime-contract.md`.
Non-number sources, external contexts, public source handles, converters,
reverse propagation, update-queue parity, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: default root number sources now have the first
property-name mutation API. Rust exposes
`StateMachineInstance::set_default_view_model_number_source_by_property_name`,
which resolves a root `ViewModelPropertyNumber.name` on file view model `0`
and mutates every matching graph source node. The C++ probe adds
`--runtime-set-default-view-model-source-number-by-name`, drives
`ViewModelInstanceRuntime::propertyNumber("amount")->value(...)` with a raw
`propertyValue("amount")` fallback for the file-backed default instance, and
compares the existing `BlendState1DViewModel` report surface. The contract is
`docs/prototypes/data-binding-graph-default-number-name-runtime-contract.md`.
Default string/color/enum/symbol-list-index/asset/artboard/trigger/list/view-model
name APIs, nested/relative/parent lookup, default public source handles beyond
the first number source handle, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: default root number sources now have the first stable
public source handle. `StateMachineInstance` can resolve a root
`ViewModelPropertyNumber.name` into `RuntimeDefaultViewModelNumberSourceHandle`,
and `set_default_view_model_number_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. Root-name handle lookup
remains separate from slash-path lookup. The C++ probe compares the handle
mutation against the default number by-name command and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-number-source-handle-runtime-contract.md`.
Default source handles for view-model sources, nested/relative/parent lookup,
reverse propagation, broader update queues, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: default nested number sources now have the first nested
stable public source handle. `StateMachineInstance` can resolve a generated
child path such as `child/amount` into
`RuntimeDefaultViewModelNumberSourceHandle` through
`default_view_model_number_source_handle_by_property_name_path`, and
`set_default_view_model_number_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. The C++ probe compares the
handle mutation against the authored `DataBindContext.sourcePathIds` mutation
path for the matching default-context data bind and verifies the existing
state-machine advance and component update reports. The
contract is
`docs/prototypes/data-binding-graph-default-nested-number-source-handle-runtime-contract.md`.
Nested default source handles for boolean/string/color/enum/symbol-list-index/
asset/artboard/trigger/list/view-model sources, relative/parent lookup,
reverse propagation, broader update queues, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: default root boolean sources now match the root number
property-name mutation shape. Rust exposes
`StateMachineInstance::set_default_view_model_boolean_source_by_property_name`,
which resolves a root `ViewModelPropertyBoolean.name` on file view model `0`
and mutates every matching graph source node. The C++ probe adds
`--runtime-set-default-view-model-source-bool-by-name`, drives
`ViewModelInstanceRuntime::propertyBoolean("enabled")->value(...)` with a raw
`propertyValue("enabled")` fallback for the file-backed default instance, and
compares the existing boolean transition-condition report surface. The contract
is
`docs/prototypes/data-binding-graph-default-boolean-name-runtime-contract.md`.
Default color/enum/symbol-list-index/asset/artboard/trigger/list/view-model
name APIs, nested/relative/parent lookup, public source handles, reverse
propagation, broader update queues, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: default root boolean sources now have the second stable
public source handle. `StateMachineInstance` can resolve a root
`ViewModelPropertyBoolean.name` into
`RuntimeDefaultViewModelBooleanSourceHandle`, and
`set_default_view_model_boolean_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. Root-name handle lookup
remains separate from slash-path lookup. The C++ probe compares the handle
mutation against the default boolean by-name command and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-boolean-source-handle-runtime-contract.md`.
Default source handles for string/color/enum/symbol-list-index/asset/artboard/
trigger/list/view-model sources, nested/relative/parent lookup, reverse
propagation, broader update queues, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: default nested boolean sources now have a stable public
source handle. `StateMachineInstance` can resolve a generated child path such
as `child/enabled` into `RuntimeDefaultViewModelBooleanSourceHandle` through
`default_view_model_boolean_source_handle_by_property_name_path`, and
`set_default_view_model_boolean_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. The C++ probe compares the
handle mutation against the authored `DataBindContext.sourcePathIds` mutation
path for the matching default-context data bind and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-nested-boolean-source-handle-runtime-contract.md`.
Nested default source handles for string/color/enum/symbol-list-index/asset/
artboard/trigger/list/view-model sources, relative/parent lookup, reverse
propagation, broader update queues, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: default root string sources now match the root number and
boolean property-name mutation shape. Rust exposes
`StateMachineInstance::set_default_view_model_string_source_by_property_name`,
which resolves a root `ViewModelPropertyString.name` on file view model `0`
and mutates every matching graph source node. The C++ probe adds
`--runtime-set-default-view-model-source-string-by-name`, drives
`ViewModelInstanceRuntime::propertyString("label")->value(...)` with a raw
`propertyValue("label")` fallback for the file-backed default instance, and
compares the existing string transition-condition report surface. The contract
is
`docs/prototypes/data-binding-graph-default-string-name-runtime-contract.md`.
Default enum/symbol-list-index/asset/artboard/trigger/list/view-model
name APIs, nested/relative/parent lookup, public source handles, reverse
propagation, broader update queues, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: default root string sources now have a stable public source
handle. `StateMachineInstance` can resolve a root
`ViewModelPropertyString.name` into `RuntimeDefaultViewModelStringSourceHandle`,
and `set_default_view_model_string_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. Root-name handle lookup
remains separate from slash-path lookup. The C++ probe compares the handle
mutation against the default string by-name command and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-string-source-handle-runtime-contract.md`.

Current #12 update: default nested string sources now have a stable public
source handle. `StateMachineInstance` can resolve a generated child path such
as `child/label` into `RuntimeDefaultViewModelStringSourceHandle` through
`default_view_model_string_source_handle_by_property_name_path`, and
`set_default_view_model_string_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. The C++ probe compares the
handle mutation against the authored `DataBindContext.sourcePathIds` mutation
path for the matching default-context data bind and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-nested-string-source-handle-runtime-contract.md`.
Default source handles for artboard/trigger/list/view-model sources,
nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: default root color sources now match the root
number/boolean/string property-name mutation shape. Rust exposes
`StateMachineInstance::set_default_view_model_color_source_by_property_name`,
which resolves a root `ViewModelPropertyColor.name` on file view model `0` and
mutates every matching graph source node. The C++ probe adds
`--runtime-set-default-view-model-source-color-by-name`, drives
`ViewModelInstanceRuntime::propertyColor("tint")->value(...)` with a raw
`propertyValue("tint")` fallback for the file-backed default instance, and
compares the existing color transition-condition report surface. The contract
is `docs/prototypes/data-binding-graph-default-color-name-runtime-contract.md`.
Default symbol-list-index/asset/artboard/trigger/list/view-model
name APIs, nested/relative/parent lookup, public source handles, reverse
propagation, broader update queues, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: default root color sources now have a stable public source
handle. `StateMachineInstance` can resolve a root
`ViewModelPropertyColor.name` into `RuntimeDefaultViewModelColorSourceHandle`,
and `set_default_view_model_color_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. Root-name handle lookup
remains separate from slash-path lookup. The C++ probe compares the handle
mutation against the default color by-name command and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-color-source-handle-runtime-contract.md`.

Current #12 update: default nested color sources now have a stable public
source handle. `StateMachineInstance` can resolve a generated child path such
as `child/tint` into `RuntimeDefaultViewModelColorSourceHandle` through
`default_view_model_color_source_handle_by_property_name_path`, and
`set_default_view_model_color_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. The C++ probe compares the
handle mutation against the authored `DataBindContext.sourcePathIds` mutation
path for the matching default-context data bind and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-nested-color-source-handle-runtime-contract.md`.
Default source handles for enum/symbol-list-index/asset/artboard/trigger/list/
view-model sources, nested/relative/parent lookup, reverse propagation, broader
update queues, listener-owned data binding, and nested artboard propagation
remain follow-up `#12` slices.

Current #12 update: default root enum sources now match the root scalar
property-name mutation shape. Rust exposes
`StateMachineInstance::set_default_view_model_enum_source_by_property_name`,
which resolves a root enum view-model property name on file view model `0` and
mutates every matching graph source node. The C++ probe adds
`--runtime-set-default-view-model-source-enum-by-name`, drives
`ViewModelInstanceRuntime::propertyEnum("choice")->valueIndex(...)` with a raw
`propertyValue("choice")` fallback for the file-backed default instance, and
compares the existing enum transition-condition report surface. The contract is
`docs/prototypes/data-binding-graph-default-enum-name-runtime-contract.md`.
Default asset/artboard/trigger/list/view-model
name APIs, nested/relative/parent lookup, public source handles, reverse
propagation, broader update queues, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: default root enum sources now have a stable public source
handle. `StateMachineInstance` can resolve a root enum view-model property
name into `RuntimeDefaultViewModelEnumSourceHandle`, and
`set_default_view_model_enum_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. Root-name handle lookup
remains separate from slash-path lookup. The C++ probe compares the handle
mutation against the default enum by-name command and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-enum-source-handle-runtime-contract.md`.

Current #12 update: default nested enum sources now have a stable public source
handle. `StateMachineInstance` can resolve a generated child path such as
`child/choice` into `RuntimeDefaultViewModelEnumSourceHandle` through
`default_view_model_enum_source_handle_by_property_name_path`, and
`set_default_view_model_enum_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. The C++ probe compares the
handle mutation against the authored `DataBindContext.sourcePathIds` mutation
path for the matching default-context data bind and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-nested-enum-source-handle-runtime-contract.md`.
Default source handles for symbol-list-index/asset/artboard/trigger/list/
view-model sources, nested/relative/parent lookup, reverse propagation, broader
update queues, listener-owned data binding, and nested artboard propagation
remain follow-up `#12` slices.

Current #12 update: default root symbol-list-index sources now match the root
scalar property-name mutation shape. Rust exposes
`StateMachineInstance::set_default_view_model_symbol_list_index_source_by_property_name`,
which resolves a root `ViewModelPropertySymbolListIndex.name` on file view
model `0` and mutates every matching graph source node. The C++ probe adds
`--runtime-set-default-view-model-source-symbol-list-index-by-name`, resolves
the default file-backed source through `ViewModelInstance::propertyValue` name
or property-index lookup, and compares the existing symbol-list-index-to-string
transition-condition report surface. The contract is
`docs/prototypes/data-binding-graph-default-symbol-list-index-name-runtime-contract.md`.
Default artboard/trigger/list/view-model
name APIs, nested/relative/parent lookup, public source handles, reverse
propagation, broader update queues, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: default root symbol-list-index sources now have a stable
public source handle. `StateMachineInstance` can resolve a root
`ViewModelPropertySymbolListIndex.name` into
`RuntimeDefaultViewModelSymbolListIndexSourceHandle`, and
`set_default_view_model_symbol_list_index_source_by_source_handle` writes
through the existing graph-owned source-path mutation path. Root-name handle
lookup remains separate from slash-path lookup. The C++ probe compares the
handle mutation against the default symbol-list-index by-name command and
verifies the existing state-machine advance and component update reports. The
contract is
`docs/prototypes/data-binding-graph-default-symbol-list-index-source-handle-runtime-contract.md`.

Current #12 update: default nested symbol-list-index sources now have a stable
public source handle. `StateMachineInstance` can resolve a generated child path
such as `child/symbol` into
`RuntimeDefaultViewModelSymbolListIndexSourceHandle` through
`default_view_model_symbol_list_index_source_handle_by_property_name_path`, and
`set_default_view_model_symbol_list_index_source_by_source_handle` writes
through the existing graph-owned source-path mutation path. The C++ probe
compares the handle mutation against the authored
`DataBindContext.sourcePathIds` mutation path for the matching default-context
data bind and verifies the existing state-machine advance and component update
reports. The contract is
`docs/prototypes/data-binding-graph-default-nested-symbol-list-index-source-handle-runtime-contract.md`.
Default source handles for asset/artboard/trigger/list/view-model sources,
nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: default root asset sources now match the root
symbol-list-index property-name mutation shape. Rust exposes
`StateMachineInstance::set_default_view_model_asset_source_by_property_name`,
which resolves a root `ViewModelPropertyAsset` or
`ViewModelPropertyAssetImage.name` on file view model `0` and mutates every
matching graph source node as a raw asset index. The C++ probe adds
`--runtime-set-default-view-model-source-asset-by-name`, resolves the default
file-backed source through `ViewModelInstance::propertyValue` name or
property-index lookup, and compares the existing asset transition-condition
report surface. The contract is
`docs/prototypes/data-binding-graph-default-asset-name-runtime-contract.md`.
Default trigger/list/view-model name APIs, nested/relative/parent
lookup, public source handles, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: default root asset sources now have a stable public source
handle. `StateMachineInstance` can resolve a root asset view-model property
name into `RuntimeDefaultViewModelAssetSourceHandle`, and
`set_default_view_model_asset_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. Root-name handle lookup
remains separate from slash-path lookup. The C++ probe compares the handle
mutation against the default asset by-name command and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-asset-source-handle-runtime-contract.md`.

Current #12 update: default nested asset sources now have a stable public
source handle. `StateMachineInstance` can resolve a generated child path such
as `child/image` into `RuntimeDefaultViewModelAssetSourceHandle` through
`default_view_model_asset_source_handle_by_property_name_path`, and
`set_default_view_model_asset_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. The C++ probe compares the
handle mutation against the authored `DataBindContext.sourcePathIds` mutation
path for the matching default-context data bind and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-nested-asset-source-handle-runtime-contract.md`.
Default source handles for artboard/trigger/list/view-model sources,
nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: default root artboard sources now match the root asset
property-name mutation shape. Rust exposes
`StateMachineInstance::set_default_view_model_artboard_source_by_property_name`,
which resolves a root `ViewModelPropertyArtboard.name` on file view model `0`
and mutates every matching graph source node as a raw artboard index. The C++
probe adds `--runtime-set-default-view-model-source-artboard-by-name`, resolves
the default file-backed source through `ViewModelInstance::propertyValue` name
or property-index lookup, and compares the existing artboard
transition-condition report surface. The contract is
`docs/prototypes/data-binding-graph-default-artboard-name-runtime-contract.md`.
Default list/view-model name APIs, nested/relative/parent lookup,
public source handles, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: default root artboard sources now have a stable public
source handle. `StateMachineInstance` can resolve a root artboard view-model
property name into `RuntimeDefaultViewModelArtboardSourceHandle`, and
`set_default_view_model_artboard_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. Root-name handle lookup
remains separate from slash-path lookup. The C++ probe compares the handle
mutation against the default artboard by-name command and verifies the
existing state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-artboard-source-handle-runtime-contract.md`.

Current #12 update: default nested artboard sources now have a stable public
source handle. `StateMachineInstance` can resolve a generated child path such
as `child/scene` into `RuntimeDefaultViewModelArtboardSourceHandle` through
`default_view_model_artboard_source_handle_by_property_name_path`, and
`set_default_view_model_artboard_source_by_source_handle` writes through the
existing graph-owned source-path mutation path. The C++ probe compares the
handle mutation against the authored `DataBindContext.sourcePathIds` mutation
path for the matching default-context data bind and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-nested-artboard-source-handle-runtime-contract.md`.
Default source handles for trigger/list/view-model sources,
nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: default root trigger sources now match the root artboard
property-name mutation shape, with the existing trigger target mirror behavior
preserved. Rust exposes
`StateMachineInstance::set_default_view_model_trigger_source_by_property_name`,
which resolves a root `ViewModelPropertyTrigger.name` on file view model `0`,
mutates every matching graph source node as a raw trigger count, and updates
matching default trigger mirrors when the source feeds a trigger bindable
target. The C++ probe adds
`--runtime-set-default-view-model-source-trigger-by-name`, resolves the default
file-backed source through `ViewModelInstance::propertyValue` name or
property-index lookup, and compares the existing trigger transition-condition
report surface. The contract is
`docs/prototypes/data-binding-graph-default-trigger-name-runtime-contract.md`.
At that point, default list and view-model name APIs,
nested/relative/parent lookup, public source handles, reverse propagation,
broader update queues, listener-owned data binding, and nested artboard
propagation remained follow-up `#12` slices.

Current #12 update: default root trigger sources now have a stable public
source handle. `StateMachineInstance` can resolve a root trigger view-model
property name into `RuntimeDefaultViewModelTriggerSourceHandle`, and
`set_default_view_model_trigger_source_by_source_handle` writes through the
existing graph-owned source-path mutation path while preserving the default
trigger target mirror update. Root-name handle lookup remains separate from
slash-path lookup. The C++ probe compares the handle mutation against the
default trigger by-name command and verifies the existing state-machine advance
and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-trigger-source-handle-runtime-contract.md`.

Current #12 update: default nested trigger sources now have a stable public
source handle. `StateMachineInstance` can resolve a generated child path such
as `child/fire` into `RuntimeDefaultViewModelTriggerSourceHandle` through
`default_view_model_trigger_source_handle_by_property_name_path`, and
`set_default_view_model_trigger_source_by_source_handle` writes through the
existing graph-owned source-path mutation path while preserving the matching
default trigger target mirror update. The C++ probe compares the handle
mutation against the authored `DataBindContext.sourcePathIds` mutation path
for the matching default-context data bind and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-nested-trigger-source-handle-runtime-contract.md`.
Default source handles for list/view-model sources, nested/relative/parent
lookup, reverse propagation, broader update queues, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: default root list sources now match the root trigger
property-name mutation shape for item-count parity. Rust exposes
`StateMachineInstance::set_default_view_model_list_source_item_count_by_property_name`,
which resolves a root `ViewModelPropertyList.name` on file view model `0` and
mutates every matching graph source node by item count. The C++ probe adds
`--runtime-set-default-view-model-source-list-by-name`, resolves the default
file-backed list through `ViewModelInstanceRuntime::propertyList` or raw
`ViewModelInstance::propertyValue` name/property-index lookup, and compares the
existing bindable-list source-size report surface. The contract is
`docs/prototypes/data-binding-graph-default-list-name-runtime-contract.md`.
At that point, default view-model name APIs, nested/relative/parent lookup,
public source handles, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remained follow-up
`#12` slices.

Current #12 update: default root list sources now have a stable public source
handle. `StateMachineInstance` can resolve a root list view-model property
name into `RuntimeDefaultViewModelListSourceHandle`, and
`set_default_view_model_list_source_item_count_by_source_handle` writes through
the existing graph-owned source-path mutation path by item count. Root-name
handle lookup remains separate from slash-path lookup. The C++ probe compares
the handle mutation against the default list by-name command and verifies the
existing data-context advance, state-machine advance, and list binding
reports. The contract is
`docs/prototypes/data-binding-graph-default-list-source-handle-runtime-contract.md`.

Current #12 update: default nested list sources now have a stable public
source handle. `StateMachineInstance` can resolve a generated child path such
as `child/items` into `RuntimeDefaultViewModelListSourceHandle` through
`default_view_model_list_source_handle_by_property_name_path`, and
`set_default_view_model_list_source_item_count_by_source_handle` writes through
the existing graph-owned source-path mutation path by item count. The C++ probe
compares the handle mutation against the authored `DataBindContext.sourcePathIds`
mutation path for the matching default-context data bind and verifies the
existing data-context advance, state-machine advance, and list binding reports.
The contract is
`docs/prototypes/data-binding-graph-default-nested-list-source-handle-runtime-contract.md`.
Default source handles for view-model sources, nested/relative/parent lookup,
reverse propagation, broader update queues, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: default root view-model sources now complete the root
property-name mutation/relink family. Rust exposes
`StateMachineInstance::relink_default_view_model_view_model_source_by_property_name`,
which resolves a root `ViewModelPropertyViewModel.name` on file view model `0`
and relinks every matching graph source node to the requested referenced
instance index. The C++ probe adds
`--runtime-relink-default-view-model-source-viewmodel-by-name`, resolves the
default file-backed pointer through
`ViewModelInstanceRuntime::replaceViewModel(name, referencedRuntime)`, and
compares the existing view-model pointer source/target report surface. The
contract is
`docs/prototypes/data-binding-graph-default-viewmodel-name-relink-runtime-contract.md`.
The default root property-name source mutation family is closed; nested,
relative, or parent lookup, public source handles, reverse propagation, broader
update queues, listener-owned data binding, and nested artboard propagation
remain follow-up `#12` slices.

Current #12 update: default root view-model sources now complete the default
root source-handle family. `StateMachineInstance` can resolve a root
view-model pointer property name into
`RuntimeDefaultViewModelViewModelSourceHandle`, and
`relink_default_view_model_view_model_source_by_source_handle` writes through
the existing graph-owned source-path relink path by imported referenced
instance index. Root-name handle lookup remains separate from slash-path
lookup. The C++ probe compares the handle relink against the default
view-model by-name relink command and verifies the existing data-context
advance, state-machine advance, source pointer, target pointer, and component
update reports. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-source-handle-runtime-contract.md`.

Current #12 update: default nested view-model pointer sources now complete the
default nested source-handle value-kind family. `StateMachineInstance` can
resolve a generated child path such as `child/grandchild` into
`RuntimeDefaultViewModelViewModelSourceHandle` through
`default_view_model_view_model_source_handle_by_property_name_path`, and
`relink_default_view_model_view_model_source_by_source_handle` relinks the
existing graph-owned source path by imported referenced instance index. The C++
probe compares the handle relink against the default view-model by-name path
relink command for the matching nested path and verifies the existing
data-context advance, state-machine advance, source pointer, target pointer,
and component update reports. The contract is
`docs/prototypes/data-binding-graph-default-nested-viewmodel-source-handle-runtime-contract.md`.
The default root source-handle family and default nested source-handle
value-kind family are closed for
number/boolean/string/color/enum/symbol-list-index/asset/artboard/trigger/list/
view-model sources; relative/parent lookup, reverse propagation, broader
update queues, listener-owned data binding, and nested artboard propagation
remain follow-up `#12` slices.

Current #12 update: default-context nested number source binding now has its
first absolute-path runtime slice. A `DataBindContext.sourcePathIds` path of
shape `[Root, child, amount]` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to the imported child
`ViewModelInstance`, then reads the child's
`ViewModelInstanceNumber.propertyValue` before writing the bindable number
target. C++ probe coverage verifies the non-zero child default through the
existing `BlendState1DViewModel` report surface. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-nested-number-runtime-contract.md`.
Nested mutation APIs, nested boolean and other value kinds, name-based relative
paths, parent paths, public source handles, reverse propagation, broader update
queues, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: default-context nested boolean source binding now follows
the nested number shape. A `DataBindContext.sourcePathIds` path of shape
`[Root, child, enabled]` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to the imported child
`ViewModelInstance`, then reads the child's
`ViewModelInstanceBoolean.propertyValue` before writing the bindable boolean
target. C++ probe coverage verifies the true child default through boolean
binding reports and the existing transition-condition consumer. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-nested-boolean-runtime-contract.md`.
Nested mutation APIs, nested string and other value kinds, name-based relative
paths, parent paths, public source handles, reverse propagation, broader update
queues, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: default-context nested string source binding now follows
the same absolute child traversal. A `DataBindContext.sourcePathIds` path of
shape `[Root, child, label]` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to the imported child
`ViewModelInstance`, then reads the child's
`ViewModelInstanceString.propertyValue` bytes before writing the bindable
string target. C++ probe coverage verifies the child default through string
binding reports and the existing transition-condition consumer. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-nested-string-runtime-contract.md`.
Nested mutation APIs, nested color and other value kinds, name-based relative
paths, parent paths, public source handles, reverse propagation, broader update
queues, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: default-context nested color source binding now follows
the same absolute child traversal. A `DataBindContext.sourcePathIds` path of
shape `[Root, child, tint]` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to the imported child
`ViewModelInstance`, then reads the child's
`ViewModelInstanceColor.propertyValue` before writing the bindable color
target. C++ probe coverage verifies the child default through color binding
reports and the existing transition-condition consumer. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-nested-color-runtime-contract.md`.
Nested mutation APIs, nested enum and other value kinds, name-based relative
paths, parent paths, public source handles, reverse propagation, broader update
queues, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: default-context nested enum source binding now follows the
same absolute child traversal. A `DataBindContext.sourcePathIds` path of shape
`[Root, child, choice]` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to the imported child
`ViewModelInstance`, then reads the child's
`ViewModelInstanceEnum.propertyValue` before writing the bindable enum target.
C++ probe coverage verifies the child default through enum binding reports and
the existing transition-condition consumer. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-nested-enum-runtime-contract.md`.
Nested mutation APIs, nested symbol-list-index and other value kinds,
name-based relative paths, parent paths, public source handles, reverse
propagation, broader update queues, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: default-context nested symbol-list-index source binding
now follows the same absolute child traversal. A
`DataBindContext.sourcePathIds` path of shape `[Root, child, symbol]` walks the
root `ViewModelInstanceViewModel.propertyValue` reference to the imported
child `ViewModelInstance`, then reads the child's
`ViewModelInstanceSymbolListIndex.propertyValue` before feeding the existing
`DataConverterToString` string bindable path. C++ probe coverage verifies the
child default through the existing transition-condition and component-update
report surfaces. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-nested-symbol-list-index-runtime-contract.md`.
Nested mutation APIs, nested asset and other value kinds, name-based relative
paths, parent paths, public source handles, reverse propagation, broader update
queues, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: default-context nested asset source binding now follows
the same absolute child traversal. A `DataBindContext.sourcePathIds` path of
shape `[Root, child, image]` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to the imported child
`ViewModelInstance`, then reads the child's
`ViewModelInstanceAssetImage.propertyValue` before writing the bindable asset
target. C++ probe coverage verifies the child default through asset binding
reports and the existing transition-condition consumer. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-nested-asset-runtime-contract.md`.
Nested mutation APIs, nested artboard and other value kinds, name-based
relative paths, parent paths, public source handles, reverse propagation,
broader update queues, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: default-context nested artboard source binding now follows
the same absolute child traversal. A `DataBindContext.sourcePathIds` path of
shape `[Root, child, scene]` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to the imported child
`ViewModelInstance`, then reads the child's
`ViewModelInstanceArtboard.propertyValue` before writing the bindable artboard
target. C++ probe coverage verifies the child default through artboard binding
reports and the existing transition-condition consumer. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-nested-artboard-runtime-contract.md`.
Nested mutation APIs, nested trigger and other value kinds, name-based
relative paths, parent paths, public source handles, reverse propagation,
broader update queues, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: default-context nested trigger source binding now follows
the same absolute child traversal. A `DataBindContext.sourcePathIds` path of
shape `[Root, child, fire]` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to the imported child
`ViewModelInstance`, then reads the child's
`ViewModelInstanceTrigger.propertyValue` before writing the bindable trigger
target. C++ probe coverage verifies the child default through trigger binding
reports and the existing transition-condition consumer. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-nested-trigger-runtime-contract.md`.
Nested mutation APIs, nested list and other value kinds, name-based relative
paths, parent paths, public source handles, reverse propagation, broader
update queues, listener-owned data binding, and nested artboard propagation
remain follow-up `#12` slices.

Current #12 update: default-context nested list source binding now follows the
same absolute child traversal. A `DataBindContext.sourcePathIds` path of shape
`[Root, child, items]` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to the imported child
`ViewModelInstance`, then reads the child's `ViewModelInstanceList` item
children before updating the bindable-list source-size report. C++ probe
coverage verifies the child default after explicit data-context advancement
and state-machine advancement. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-nested-list-runtime-contract.md`.
Nested mutation APIs, nested view-model value kind, name-based relative paths,
parent paths, public source handles, reverse propagation, broader update
queues, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: default-context nested view-model pointer source binding
now closes the absolute child traversal value-kind family. A
`DataBindContext.sourcePathIds` path of shape
`[Root, child, grandchild]` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to the imported child
`ViewModelInstance`, then reads the child's
`ViewModelInstanceViewModel.propertyValue` before writing the bindable
view-model target. C++ probe coverage verifies the imported grandchild
instance index after explicit data-context advancement and state-machine
advancement. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-nested-viewmodel-runtime-contract.md`.
Nested mutation APIs, name-based relative paths, parent paths, public source
handles, reverse propagation, broader update queues, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: graph-owned source mutation now also covers default
`ViewModelInstanceBoolean` sources. Rust exposes
`StateMachineInstance::set_default_view_model_boolean_source_for_data_bind`,
which mutates the selected `RuntimeDataBindGraph` boolean source node and
dirties the default edge when the default context is bound. The C++ probe
mirrors this with `--runtime-set-default-view-model-source-bool`, resolving
`DataBindContext.sourcePathIds` against the default view-model instance and
mutating the resolved `ViewModelInstanceBoolean.propertyValue`. The contract is
`docs/prototypes/data-binding-graph-default-boolean-source-mutation-runtime-contract.md`.
String/color/enum/asset/artboard/trigger sources, external contexts, public
source handles, converters, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: graph-owned source mutation now also covers default
`ViewModelInstanceString` sources. Rust exposes
`StateMachineInstance::set_default_view_model_string_source_for_data_bind`,
which mutates the selected `RuntimeDataBindGraph` string source node as raw
bytes and dirties the default edge when the default context is bound. The C++
probe mirrors this with `--runtime-set-default-view-model-source-string`,
resolving `DataBindContext.sourcePathIds` against the default view-model
instance and mutating the resolved `ViewModelInstanceString.propertyValue`. The
contract is
`docs/prototypes/data-binding-graph-default-string-source-mutation-runtime-contract.md`.
Color/enum/asset/artboard/trigger sources, external contexts, public source
handles, converters, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: graph-owned source mutation now also covers default
`ViewModelInstanceColor` sources. Rust exposes
`StateMachineInstance::set_default_view_model_color_source_for_data_bind`,
which mutates the selected `RuntimeDataBindGraph` color source node and dirties
the default edge when the default context is bound. The C++ probe mirrors this
with `--runtime-set-default-view-model-source-color`, resolving
`DataBindContext.sourcePathIds` against the default view-model instance and
mutating the resolved `ViewModelInstanceColor.propertyValue`. The contract is
`docs/prototypes/data-binding-graph-default-color-source-mutation-runtime-contract.md`.
Enum/asset/artboard/trigger sources, external contexts, public source handles,
converters, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: graph-owned source mutation now also covers default
`ViewModelInstanceEnum` sources. Rust exposes
`StateMachineInstance::set_default_view_model_enum_source_for_data_bind`, which
mutates the selected `RuntimeDataBindGraph` enum source node as a raw uint
property value and dirties the default edge when the default context is bound.
The C++ probe mirrors this with `--runtime-set-default-view-model-source-enum`,
resolving `DataBindContext.sourcePathIds` against the default view-model
instance and mutating the resolved `ViewModelInstanceEnum.propertyValue`. The
contract is
`docs/prototypes/data-binding-graph-default-enum-source-mutation-runtime-contract.md`.
Asset/artboard/trigger sources, external contexts, public source handles,
converters, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: graph-owned source mutation now also covers default
`ViewModelInstanceAssetImage` sources. Rust exposes
`StateMachineInstance::set_default_view_model_asset_source_for_data_bind`,
which resolves the selected `RuntimeDataBindGraph` asset source path, mutates
matching same-path asset source nodes as raw uint property values, and dirties
the default edges when the default context is bound. The C++ probe mirrors
this with
`--runtime-set-default-view-model-source-asset`, resolving
`DataBindContext.sourcePathIds` against the default view-model instance and
mutating the resolved `ViewModelInstanceAssetImage.propertyValue`. The contract
is
`docs/prototypes/data-binding-graph-default-asset-source-mutation-runtime-contract.md`.
Artboard/trigger sources, external contexts, public source handles, converters,
reverse propagation, update-queue parity, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: graph-owned source mutation now also covers default
`ViewModelInstanceArtboard` sources. Rust exposes
`StateMachineInstance::set_default_view_model_artboard_source_for_data_bind`,
which resolves the selected `RuntimeDataBindGraph` artboard source path,
mutates matching same-path artboard source nodes as raw uint property values,
and dirties the default edges when the default context is bound. The C++ probe
mirrors this with
`--runtime-set-default-view-model-source-artboard`, resolving
`DataBindContext.sourcePathIds` against the default view-model instance and
mutating the resolved `ViewModelInstanceArtboard.propertyValue`. The contract
is
`docs/prototypes/data-binding-graph-default-artboard-source-mutation-runtime-contract.md`.
Trigger sources, external contexts, public source handles, converters, reverse
propagation, update-queue parity, relative/parent/nested lookup, listener-owned
data binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: graph-owned source mutation now also covers default
`ViewModelInstanceTrigger` sources. Rust exposes
`StateMachineInstance::set_default_view_model_trigger_source_for_data_bind`,
which resolves the selected `RuntimeDataBindGraph` trigger source path,
mutates matching same-path trigger source nodes as raw trigger-count property
values, and dirties the default edges when the default context is bound. The
C++ probe mirrors this with
`--runtime-set-default-view-model-source-trigger`, resolving
`DataBindContext.sourcePathIds` against the default view-model instance and
mutating the resolved `ViewModelInstanceTrigger.propertyValue`. The contract is
`docs/prototypes/data-binding-graph-default-trigger-source-mutation-runtime-contract.md`.
The finite graph-owned default source-node mutation set now covers
number/boolean/string/color/enum/asset/artboard/trigger. External contexts,
public source handles, converters, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: graph-owned source mutation now also covers default
`ViewModelInstanceSymbolListIndex` sources. Rust exposes
`StateMachineInstance::set_default_view_model_symbol_list_index_source_for_data_bind`,
which mutates the selected `RuntimeDataBindGraph` symbol-list-index source node
as a raw uint property value and dirties the default edge when the default
context is bound. The C++ probe mirrors this with
`--runtime-set-default-view-model-source-symbol-list-index`, resolving
`DataBindContext.sourcePathIds` against the default view-model instance and
mutating the resolved `ViewModelInstanceSymbolListIndex.propertyValue`. The
contract is
`docs/prototypes/data-binding-graph-default-symbol-list-index-source-mutation-runtime-contract.md`.
The finite graph-owned default source-node mutation set now covers
number/boolean/string/color/enum/asset/artboard/trigger/symbol-list-index/view-model.
External contexts, public source handles, reverse propagation, update-queue
parity, relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: the data-binding graph now stores source path metadata and
can bind an imported file-backed `ViewModelInstance` as the active
state-machine data context. Rust exposes
`StateMachineInstance::bind_view_model_instance_context`, which resolves the
existing finite `propertyValue` source-node set against the selected
`RuntimeFile` view-model instance, marks missing/type-incompatible sources
unbound, and dirties the graph for the next state-machine advance. The C++
probe mirrors this with
`--runtime-bind-view-model-instance-state-machine-context`, calling
`StateMachineInstance::bindViewModelInstance(...)` on the selected imported
instance. C++ probe-backed consumers cover non-default
`ViewModelInstanceNumber`, `ViewModelInstanceBoolean`,
`ViewModelInstanceString`, `ViewModelInstanceColor`, `ViewModelInstanceEnum`,
`ViewModelInstanceAssetImage`, `ViewModelInstanceArtboard`,
`ViewModelInstanceTrigger`, and `ViewModelInstanceSymbolListIndex` sources
through `BlendState1DViewModel` and transition-condition paths. The contract is
`docs/prototypes/data-binding-graph-external-view-model-context-runtime-contract.md`.
Arbitrary user-created runtime view-model instances, public source handles,
converters, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: imported file-backed context binding now explicitly covers
`ViewModelInstanceSymbolListIndex` sources. The existing
`StateMachineInstance::bind_view_model_instance_context` graph path resolves
the selected imported instance's raw symbol-list-index value and feeds it into
the admitted `DataConverterToString` target path. The C++ probe mirrors this
with `StateMachineInstance::bindViewModelInstance(...)` and a fixture whose
default root value differs from the imported alternate value, proving the graph
refreshes the external source instead of falling back to default or target
initial state. The contract is
`docs/prototypes/data-binding-graph-external-symbol-list-index-context-runtime-contract.md`.
Owned runtime symbol-list-index contexts, stable public source handles, list
bindables, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: the first owned runtime view-model context path now covers
number sources. Rust exposes `RuntimeOwnedViewModelInstance::new` plus
`set_number_by_property_index`, and
`StateMachineInstance::bind_owned_view_model_context` refreshes matching graph
number source nodes from the owned context. The C++ probe mirrors this with
`--runtime-bind-owned-view-model-number-state-machine-context`, creating
`File::createViewModelInstance(...)`, mutating a
`ViewModelInstanceNumberRuntime` property, and binding the owned instance to
the state machine. The contract is
`docs/prototypes/data-binding-graph-owned-view-model-number-context-runtime-contract.md`.
Owned boolean/string/color/enum/asset/artboard/trigger values, stable public
source handles, converters, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: owned runtime view-model contexts now also cover boolean
sources. Rust exposes `RuntimeOwnedViewModelInstance::set_boolean_by_property_index`,
and the graph resolves owned `ViewModelInstanceBoolean` values into existing
boolean source nodes before state-machine transition evaluation. The C++ probe
mirrors this with `--runtime-bind-owned-view-model-bool-state-machine-context`.
The contract is
`docs/prototypes/data-binding-graph-owned-view-model-boolean-context-runtime-contract.md`.
Owned string/color/enum/asset/artboard/trigger values, stable public source
handles, converters, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: owned runtime view-model contexts now also cover string
sources as raw bytes. Rust exposes
`RuntimeOwnedViewModelInstance::set_string_by_property_index`, and the graph
resolves owned `ViewModelInstanceString` values into existing string source
nodes before state-machine transition evaluation. The C++ probe mirrors this
with `--runtime-bind-owned-view-model-string-state-machine-context`. The
contract is
`docs/prototypes/data-binding-graph-owned-view-model-string-context-runtime-contract.md`.
Owned color/enum/asset/artboard/trigger values, stable public source handles,
converters, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: owned runtime view-model contexts now also cover color
sources as packed ARGB values. Rust exposes
`RuntimeOwnedViewModelInstance::set_color_by_property_index`, and the graph
resolves owned `ViewModelInstanceColor` values into existing color source nodes
before state-machine transition evaluation. The C++ probe mirrors this with
`--runtime-bind-owned-view-model-color-state-machine-context`. The contract is
`docs/prototypes/data-binding-graph-owned-view-model-color-context-runtime-contract.md`.
Owned enum/asset/artboard/trigger values, stable public source handles,
converters, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: owned runtime view-model contexts now also cover enum
sources as raw `propertyValue` integers. Rust exposes
`RuntimeOwnedViewModelInstance::set_enum_by_property_index`, and the graph
resolves owned `ViewModelInstanceEnum` values into existing enum source nodes
before state-machine transition evaluation. The C++ probe mirrors this with
`--runtime-bind-owned-view-model-enum-state-machine-context`, mutating the
owned C++ runtime enum by value index. The contract is
`docs/prototypes/data-binding-graph-owned-view-model-enum-context-runtime-contract.md`.
Owned asset/artboard/trigger values, stable public source handles, converters,
reverse propagation, update-queue parity, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime view-model contexts now also cover
symbol-list-index sources as raw `propertyValue` integers. Rust exposes
`RuntimeOwnedViewModelInstance::set_symbol_list_index_by_property_index`, and
the graph resolves owned `ViewModelInstanceSymbolListIndex` values into
existing symbol-list-index source nodes before state-machine transition
evaluation. The C++ probe mirrors this with
`--runtime-bind-owned-view-model-symbol-list-index-state-machine-context`,
mutating the fresh C++ symbol-list-index instance value before binding. The
contract is
`docs/prototypes/data-binding-graph-owned-view-model-symbol-list-index-context-runtime-contract.md`.
Owned asset/artboard/trigger values, stable public source handles, converters,
reverse propagation, update-queue parity, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime view-model contexts now also cover asset
image sources as raw `propertyValue` integers. Rust exposes
`RuntimeOwnedViewModelInstance::set_asset_by_property_index`, and the graph
resolves owned `ViewModelInstanceAssetImage` values into existing asset source
nodes before state-machine transition evaluation. The C++ probe mirrors this
with `--runtime-bind-owned-view-model-asset-state-machine-context`, mutating
the fresh C++ asset instance value before binding. The contract is
`docs/prototypes/data-binding-graph-owned-view-model-asset-context-runtime-contract.md`.
Owned artboard/trigger values, stable public source handles, converters,
reverse propagation, update-queue parity, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime view-model contexts now also cover artboard
sources as raw `propertyValue` integers. Rust exposes
`RuntimeOwnedViewModelInstance::set_artboard_by_property_index`, and the graph
resolves owned `ViewModelInstanceArtboard` values into existing artboard source
nodes before state-machine transition evaluation. The C++ probe mirrors this
with `--runtime-bind-owned-view-model-artboard-state-machine-context`, mutating
the fresh C++ artboard instance value before binding. The contract is
`docs/prototypes/data-binding-graph-owned-view-model-artboard-context-runtime-contract.md`.
Owned trigger values, stable public source handles, converters, reverse
propagation, update-queue parity, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime view-model contexts now also cover trigger
sources as raw trigger-count `propertyValue` integers. Rust exposes
`RuntimeOwnedViewModelInstance::set_trigger_by_property_index`, and the graph
resolves owned `ViewModelInstanceTrigger` values into existing trigger source
nodes before state-machine transition evaluation. The C++ probe mirrors this
with `--runtime-bind-owned-view-model-trigger-state-machine-context`, mutating
the fresh C++ trigger instance value before binding. The contract is
`docs/prototypes/data-binding-graph-owned-view-model-trigger-context-runtime-contract.md`.
The finite owned source-node context set now covers
number/boolean/string/color/enum/symbol-list-index/asset/artboard/trigger.
Stable public source handles, converters, reverse propagation,
update-queue parity, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: external and owned trigger identity for
`StateMachineFireTrigger`, trigger/self conditions, and explicit
`advancedDataContext()` reset is now separated from the default imported
trigger report view. `StateMachineInstance` keeps the C++ probe-visible default
`ViewModelInstanceTrigger` report values distinct from the active bound
non-default imported or owned trigger context; firing and reset mutate the
active context, and graph-owned trigger source nodes reset on data-context
advance so raw `BindablePropertyTrigger.propertyValue` bindings do not retain
stale active counts. C++ probe coverage binds both non-default imported and
owned trigger contexts, fires state-machine trigger actions, advances the data
context, and verifies the default imported trigger report identity remains
unchanged. The contract is
`docs/prototypes/data-binding-graph-external-trigger-identity-runtime-contract.md`.
Stable public source handles, list/symbol/view-model bindables, converters,
reverse propagation, update-queue parity, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: the first graph-owned runtime converter slice now supports
stateless forward `DataConverterBooleanNegate` execution on default-context
`DataBindContext -> BindablePropertyBoolean.propertyValue` edges. Graph source
nodes carry a small converter descriptor: no-converter bindings keep direct
source-to-target behavior, boolean-negate bindings invert the source before
target writes, and converter-bearing bindings whose converter has not been
admitted remain unapplied instead of pretending to be pass-through. C++ probe
coverage verifies a default boolean source converted through
`DataConverterBooleanNegate` before `TransitionViewModelCondition` evaluation.
The contract is
`docs/prototypes/data-binding-graph-boolean-negate-converter-runtime-contract.md`.
Stable public source handles, list/symbol/view-model bindables, converters
beyond boolean negation, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: graph-owned runtime converter execution now also supports
stateless forward `DataConverterTrigger` execution on default-context
`DataBindContext -> BindablePropertyTrigger.propertyValue` edges. Trigger
source nodes carry the same converter descriptor as boolean sources, and the
graph applies the C++ trigger increment rule with `uint32_t` wrapping semantics
before target writes. C++ probe coverage verifies a default trigger source
converted through `DataConverterTrigger` before `TransitionViewModelCondition`
evaluation. The contract is
`docs/prototypes/data-binding-graph-trigger-converter-runtime-contract.md`.
Stable public source handles, list/symbol/view-model bindables, converters
beyond boolean negation and trigger increment, reverse propagation,
update-queue parity, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: graph-owned runtime converter execution now supports the
first cross-type source-to-target converter case:
`DataConverterToNumber` for default-context boolean sources feeding
`BindablePropertyNumber.propertyValue` targets. Number source nodes now carry a
graph value instead of assuming every number target is sourced by a number; the
admitted converter path converts `true -> 1.0` and `false -> 0.0` before target
writes while preserving the existing no-converter number path. C++ probe
coverage verifies the converted value through a `BlendState1DViewModel`
consumer. The contract is
`docs/prototypes/data-binding-graph-to-number-boolean-converter-runtime-contract.md`.
Stable public source handles, list/symbol/view-model bindables, remaining
`DataConverterToNumber` input kinds, converters beyond boolean negation,
trigger increment, and boolean-to-number, reverse propagation,
update-queue parity, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: `DataConverterToNumber` runtime execution now also admits
default-context enum sources feeding `BindablePropertyNumber.propertyValue`
targets. The graph keeps the raw enum `propertyValue` as the source node value
and converts it to `f32` before number target writes. C++ probe coverage
verifies the converted value through a `BlendState1DViewModel` consumer. The
contract is
`docs/prototypes/data-binding-graph-to-number-enum-converter-runtime-contract.md`.
Stable public source handles, list/symbol/view-model bindables, remaining
`DataConverterToNumber` input kinds, converters beyond boolean negation,
trigger increment, boolean-to-number, and enum-to-number, reverse propagation,
update-queue parity, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: `DataConverterToNumber` runtime execution now also admits
default-context color sources feeding `BindablePropertyNumber.propertyValue`
targets. The graph keeps the raw color source node value and converts it
through the C++ signed `int32_t` cast before writing the number target. C++
probe coverage verifies the converted value through a `BlendState1DViewModel`
consumer. The contract is
`docs/prototypes/data-binding-graph-to-number-color-converter-runtime-contract.md`.
Stable public source handles, list/symbol/view-model bindables, string and
symbol-list `DataConverterToNumber` input kinds, converters beyond boolean
negation, trigger increment, boolean-to-number, enum-to-number, and
color-to-number, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: `DataConverterToNumber` runtime execution now also admits
default-context string sources feeding `BindablePropertyNumber.propertyValue`
targets. The graph keeps raw string bytes from `ViewModelInstanceString` source
nodes and converts them through the binary crate's C++ `std::atof`-style
numeric-prefix parser before writing the number target. C++ probe coverage
verifies the converted value through a `BlendState1DViewModel` consumer using a
string with a numeric prefix and trailing bytes. The contract is
`docs/prototypes/data-binding-graph-to-number-string-converter-runtime-contract.md`.
Stable public source handles, list/symbol/view-model bindables, symbol-list
`DataConverterToNumber` input kinds, converters beyond boolean negation,
trigger increment, boolean-to-number, enum-to-number, color-to-number, and
string-to-number, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: the string-source `DataConverterToNumber` path now also
covers the main-`ToTarget | TwoWay` state-machine target-dirty behavior for
default-context number targets. A manual edit to the
`BindablePropertyNumber.propertyValue` target is preserved through explicit
data-context advancement, then the next normal state-machine advance overwrites
the target from the unchanged string source through C++ `std::atof`-style
forward conversion. The contract is
`docs/prototypes/data-binding-graph-to-number-string-main-to-target-two-way-target-dirty-runtime-contract.md`.
Stable public source handles, list/symbol/view-model bindables, other
`DataConverterToNumber` dirty paths, public-queue reverse conversion,
converter groups, formula functions/randoms, interpolator, number-to-list,
scripted converters, broader dirty/update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: the remaining direct scalar `DataConverterToNumber` paths
now also cover main-`ToTarget | TwoWay` state-machine target-dirty behavior for
default-context number targets. Boolean, enum, color, and symbol-list-index
sources preserve a manual `BindablePropertyNumber.propertyValue` edit through
explicit data-context advancement, then normal state-machine advancement
overwrites the target from the unchanged source through forward
`DataConverterToNumber::convert`. The contract is
`docs/prototypes/data-binding-graph-to-number-scalar-main-to-target-two-way-target-dirty-runtime-contract.md`.
Stable public source handles, list/view-model bindables, public-queue reverse
conversion beyond the linked `DataConverterToNumber` public-update slices,
converter groups, formula functions/randoms, interpolator, number-to-list,
scripted converters, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: `DataConverterToNumber` runtime execution now also admits
default-context symbol-list-index sources feeding
`BindablePropertyNumber.propertyValue` targets. The graph keeps the raw
`ViewModelInstanceSymbolListIndex.propertyValue` source node value and converts
it to `f32` before writing the number target. C++ probe coverage verifies the
converted value through a `BlendState1DViewModel` consumer. The contract is
`docs/prototypes/data-binding-graph-to-number-symbol-list-index-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, converter families
beyond the admitted boolean negation, trigger increment, and currently covered
`DataConverterToNumber` input set, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: the first list-source converter path now admits
`DataConverterListToLength` for default-context `ViewModelInstanceList`
sources feeding `BindablePropertyNumber.propertyValue` targets. The graph
carries the imported list item count as a finite source value and converts it
to a number before state-machine evaluation. C++ probe coverage verifies the
length through a `BlendState1DViewModel` consumer. The contract is
`docs/prototypes/data-binding-graph-list-to-length-converter-runtime-contract.md`.
Stable public source handles, list targets, list mutation APIs,
`DataConverterNumberToList`, generated runtime list items, reverse conversion
beyond the linked public-update and main-`ToSource` base-reverse paths,
broader update-queue parity, relative/parent/nested lookup, listener-owned
data binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterListToLength` now also covers
main-`ToTarget | TwoWay` state-machine target-dirty behavior for
default-context number targets. A manual edit to the
`BindablePropertyNumber.propertyValue` target is preserved through explicit
data-context advancement, then the next normal state-machine advance overwrites
the target from the unchanged imported list source length. The contract is
`docs/prototypes/data-binding-graph-list-to-length-main-to-target-two-way-target-dirty-runtime-contract.md`.
Stable public source handles, list targets, list mutation APIs,
`DataConverterNumberToList`, generated runtime list items, broader
update-queue parity beyond the linked public-update path,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterListToLength` now also covers public
`updateDataBinds(true)` target-to-source behavior for default-context
main-`ToTarget | TwoWay` number targets. C++ uses the base converter reverse
identity for the edited numeric target, does not write that numeric value into
the list source, then immediately reapplies the unchanged imported list length
to the number target. The contract is
`docs/prototypes/data-binding-graph-list-to-length-public-update-target-to-source-runtime-contract.md`.
Stable public source handles, list targets, list mutation APIs,
`DataConverterNumberToList`, generated runtime list items, converter groups,
broader dirty/update queues, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterListToLength` now also covers
main-`ToSource | TwoWay` target-to-source behavior for default-context number
targets. C++ does not write the edited number target into the list source, then
the same explicit data-context pass refreshes the target through base
`reverseConvert`, which yields the default number value `0` for this
list-to-number target. The contract is
`docs/prototypes/data-binding-graph-list-to-length-main-to-source-target-to-source-runtime-contract.md`.
Stable public source handles, list targets, list mutation APIs,
`DataConverterNumberToList`, generated runtime list items, converter groups,
broader dirty/update queues, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: `DataConverterRounder` runtime execution now admits
default-context number sources feeding `BindablePropertyNumber.propertyValue`
targets. The graph stores imported `decimals` on the converter descriptor and
applies C++'s `round(value * pow(10, decimals)) / pow(10, decimals)` behavior
before writing the number target. C++ probe coverage verifies the rounded value
through a `BlendState1DViewModel` consumer. The contract is
`docs/prototypes/data-binding-graph-rounder-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, remaining converter
families, converter groups, target-to-source propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterRounder` now also covers the
main-`ToSource | TwoWay` target-to-source path for default-context number
binds. A manual edit to a `BindablePropertyNumber.propertyValue` target is
passed through C++ main-direction rounder `convert` before writing the
`ViewModelInstanceNumber.propertyValue` source; a second direct number bind
observes the rounded source value after normal source-to-target application.
The contract is
`docs/prototypes/data-binding-graph-rounder-target-to-source-runtime-contract.md`.
Stable public source handles, list/view-model bindables, public-queue reverse
conversion, converter groups, main-`ToTarget | TwoWay` dirty behavior for
remaining converter families, formula functions/randoms, interpolator,
number-to-list, scripted converters, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterRounder` now also covers the
main-`ToTarget | TwoWay` state-machine target-dirty path for default-context
number binds. A manual edit to the `BindablePropertyNumber.propertyValue`
target is preserved through explicit data-context advancement, then the next
normal state-machine advance overwrites the target from the unchanged source
through C++ rounder forward conversion. The contract is
`docs/prototypes/data-binding-graph-rounder-main-to-target-two-way-target-dirty-runtime-contract.md`.
Stable public source handles, list/view-model bindables, public-queue reverse
conversion, converter groups, main-`ToTarget | TwoWay` dirty behavior for
remaining converter families, formula functions/randoms, interpolator,
number-to-list, scripted converters, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: the first `DataConverterRangeMapper` runtime execution
slice now admits default-context number sources feeding
`BindablePropertyNumber.propertyValue` targets when the converter has no
resolved custom interpolator. The graph stores imported range bounds, flags,
and `interpolationType`, then applies the C++ forward range-mapping behavior
before writing the number target. C++ probe coverage verifies an upper-clamped
input through a `BlendState1DViewModel` consumer. The contract is
`docs/prototypes/data-binding-graph-range-mapper-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, reverse conversion,
resolved converter interpolators, remaining converter families, converter
groups, update-queue parity, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: `DataConverterRangeMapper` runtime execution now also
admits resolved converter interpolators for default-context number sources
feeding `BindablePropertyNumber.propertyValue` targets. The graph stores the
resolved `CubicEaseInterpolator`/`ElasticInterpolator` descriptor from
`DataConverterRangeMapper.interpolatorId` and applies the C++ percent transform
inside the existing range-map path. C++ probe coverage verifies a resolved
`CubicEaseInterpolator` through a `BlendState1DViewModel` consumer. The
contract is
`docs/prototypes/data-binding-graph-range-mapper-interpolator-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, reverse conversion,
stateful converter interpolation, formula, number-to-list, list-to-length, and
scripted converters, converter groups requiring stateful scheduling,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: the first `DataConverterOperationValue` runtime execution
slice now admits default-context number sources feeding
`BindablePropertyNumber.propertyValue` targets. The graph stores imported
`operationType` and `operationValue`, then applies C++ forward arithmetic
behavior before writing the number target. C++ probe coverage verifies every
C++ `ArithmeticOperation` discriminant through a `BlendState1DViewModel`
consumer. The contract is
`docs/prototypes/data-binding-graph-operation-value-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, reverse conversion,
operation-view-model, system, formula, interpolator, number-to-list, and
scripted converters, converter groups involving operation converters,
update-queue parity, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: `DataConverterOperationValue` runtime execution now also
admits default-context symbol-list-index sources feeding
`BindablePropertyNumber.propertyValue` targets. The graph casts the imported
symbol-list-index value to `f32`, then applies the same C++ forward arithmetic
path used by number sources before writing the number target. C++ probe
coverage verifies this through a `BlendState1DViewModel` consumer. The
contract is
`docs/prototypes/data-binding-graph-operation-value-symbol-list-index-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, reverse conversion,
operation-view-model, system, formula, interpolator, number-to-list, and
scripted converters, converter groups involving operation converters,
update-queue parity, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: the first `DataConverterOperationViewModel` runtime
execution slice now admits default-context number sources feeding
`BindablePropertyNumber.propertyValue` targets when the converter's
`sourcePathIds` resolve to a second imported default view-model number. The
graph stores the resolved secondary operand on the converter descriptor and
uses the same forward arithmetic path as `DataConverterOperationValue`. C++
probe coverage verifies this through a `BlendState1DViewModel` consumer. The
contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, reverse conversion,
other grouped secondary-source dependency compositions, imported/owned context
recomputation for the secondary operand, dedicated grouped
operation-view-model probe coverage, formula, interpolator, number-to-list,
and scripted converters, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterOperationViewModel` converter source
paths now have an explicit name-path unsupported boundary. C++
`DataConverterOperationViewModel::bindFromContext()` calls
`DataContext::getViewModelProperty(sourcePathIds)` directly, so a manifest
path id for `factor` is not resolved through the file data resolver and leaves
the secondary operand missing. Rust mirrors the C++ operand `0.0` fallback
through the existing blend-state probe. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-name-path-unsupported-runtime-contract.md`.
Stable public source handles, list/view-model bindables, reverse conversion,
other grouped secondary-source dependency compositions, imported/owned context
recomputation for the secondary operand, dedicated grouped
operation-view-model probe coverage, formula, interpolator, number-to-list,
scripted converters, supported relative/parent/nested lookup surfaces,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: direct `DataConverterOperationViewModel` now refreshes
its cached secondary operand and dirties the dependent source when that
secondary default view-model number is mutated through the state-machine
default source API. This mirrors the C++ `bindFromContext()` relationship
where the resolved `ViewModelInstanceNumber` registers the owning `DataBind`
as a dependent. C++ probe coverage verifies a converter-bound primary number
bind and an ordinary secondary number bind through the same
`BlendState1DViewModel` consumer. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-secondary-source-mutation-runtime-contract.md`.
Stable public source handles, list/view-model bindables, other grouped
secondary-source dependency compositions, imported/owned context recomputation
for the secondary operand, formula, interpolator, number-to-list, scripted
converters, relative/parent/nested lookup, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterOperationViewModel` now also covers
the main-`ToSource | TwoWay` target-to-source path for default-context number
binds. A manual edit to a
`BindablePropertyNumber.propertyValue` target is passed through
`DataConverterOperationViewModel::convert` with the imported secondary
view-model number operand before writing the primary
`ViewModelInstanceNumber.propertyValue` source. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-target-to-source-runtime-contract.md`.
Stable public source handles, list/view-model bindables, public-queue reverse
conversion, other grouped secondary-source dependency compositions,
imported/owned context recomputation for the secondary operand, dedicated
grouped operation-view-model probe coverage, formula functions/randoms,
interpolator, number-to-list, scripted converters, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterOperationViewModel` now also covers
the main-`ToTarget | TwoWay` state-machine target-dirty path for
default-context number binds. A manual edit to the
`BindablePropertyNumber.propertyValue` target is preserved through explicit
data-context advancement, then the next normal state-machine advance overwrites
the target from the unchanged primary source through forward
`DataConverterOperationViewModel::convert` with the imported secondary
view-model number operand. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-main-to-target-two-way-target-dirty-runtime-contract.md`.
Stable public source handles, list/view-model bindables, public-queue reverse
conversion, other grouped secondary-source dependency compositions,
imported/owned context recomputation for the secondary operand, dedicated
grouped operation-view-model probe coverage, main-`ToTarget | TwoWay` dirty
behavior for other converter families, formula functions/randoms,
interpolator, number-to-list, scripted converters, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterSystemNormalizer` and
`DataConverterSystemDegsToRads` runtime execution now admits default-context
number sources feeding `BindablePropertyNumber.propertyValue` targets when C++
`DataBind::toTarget()` is true. The graph stores imported `operationType`,
`operationValue`, and the source-to-target direction choice from
`DataBind.flags`, applying forward operation-value arithmetic for default
`ToTarget` flags and reverse operation-value arithmetic for
`TwoWay | ToSource` flags. C++ probe coverage verifies both concrete converter
types through a `BlendState1DViewModel` consumer. The contract is
`docs/prototypes/data-binding-graph-system-operation-value-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, `ToSource`-only
update-queue behavior, target-to-source propagation, operation-view-model,
formula, interpolator, number-to-list, and scripted converters, converter
groups involving system converters, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: direct `DataConverterSystemNormalizer` and
`DataConverterSystemDegsToRads` now also cover the main-`ToSource | TwoWay`
target-to-source path for default-context number binds. C++ calls
`convert` for the main-direction target-to-source dispatch, and these system
converters' `convert` methods select operation-value reverse arithmetic when
the authored direction is `ToSource`, before writing the
`ViewModelInstanceNumber.propertyValue` source; the same bindable target is
then refreshed from the changed source during explicit data-context
advancement. The contract is
`docs/prototypes/data-binding-graph-system-operation-value-target-to-source-runtime-contract.md`.
Stable public source handles, list/view-model bindables, `ToSource`-only
update-queue behavior, operation-view-model target-to-source, formula
functions/randoms, interpolator, number-to-list, scripted converters, converter
groups involving system converters, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterSystemNormalizer` and
`DataConverterSystemDegsToRads` now also cover the main-`ToTarget | TwoWay`
state-machine target-dirty path for default-context number binds. A manual
edit to a system-converter `BindablePropertyNumber.propertyValue` target is
preserved through explicit data-context advancement, then the next normal
state-machine advance overwrites the target from the unchanged source through
C++ system-converter forward operation-value arithmetic. The contract is
`docs/prototypes/data-binding-graph-system-operation-value-main-to-target-two-way-target-dirty-runtime-contract.md`.
Stable public source handles, list/view-model bindables, `ToSource`-only
update-queue behavior, operation-view-model target-to-source,
main-`ToTarget | TwoWay` dirty behavior for other converter families, formula
functions/randoms, interpolator, number-to-list, scripted converters, converter
groups involving system converters, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterInterpolator` now also covers the
main-`ToTarget | TwoWay` state-machine target-dirty path for default-context
number binds after warming C++'s direct interpolator startup gate. A manual
edit to the `BindablePropertyNumber.propertyValue` target is preserved through
explicit data-context advancement, then the next normal state-machine advance
reapplies the warmed direct interpolator source-to-target converter state even
when elapsed time is zero. The contract is
`docs/prototypes/data-binding-graph-interpolator-main-to-target-two-way-target-dirty-runtime-contract.md`.
Stable public source handles, list/view-model bindables, grouped
interpolators, public-queue reverse conversion, number-to-list, scripted
converters, broader dirty/update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: direct `DataConverterInterpolator` now also covers
main-`ToSource | TwoWay` target-to-source behavior for warmed default-context
number binds. Explicit data-context advancement runs the edited number target
through C++ main-direction interpolator `convert`, then refreshes the visible
target during the same pass through `reverseConvert`, which delegates to the
same stateful convert path, even when the converted source value is unchanged.
The contract is
`docs/prototypes/data-binding-graph-interpolator-main-to-source-target-to-source-runtime-contract.md`.
Fresh/unwarmed interpolator target edits, grouped interpolators, color
interpolation targets, broader dirty/update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: the C++ runtime probe now emits exact `stringBindings`
source/target snapshots for state-machine `BindablePropertyString` data binds,
and the Rust probe harness can compare those values directly for
default-context string sources and string bindable targets. This is the narrow
reporting seam needed before adding main-`ToTarget | TwoWay` string target-dirty
parity tests, because earlier string converter coverage inferred behavior
through transition-condition consumers. The contract is
`docs/prototypes/data-binding-graph-string-binding-report-runtime-contract.md`.
Main-`ToTarget | TwoWay` string target-dirty behavior, enum, asset, artboard,
trigger, view-model, and list binding reports, public API design, broader
dirty/update queues, relative/parent/nested lookup, listener-owned data binding,
and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterToString` number-to-string binds now
cover the main-`ToTarget | TwoWay` state-machine target-dirty path for
default-context string targets. A manual edit to the
`BindablePropertyString.propertyValue` target is preserved through explicit
data-context advancement, then the next normal state-machine advance reapplies
the unchanged number source through C++ number-to-string formatting, including
when elapsed time is zero. The contract is
`docs/prototypes/data-binding-graph-to-string-number-main-to-target-two-way-target-dirty-runtime-contract.md`.
Other `DataConverterToString` input kinds, string converter families and
groups, public-queue reverse conversion, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterToString` boolean-to-string binds now
also cover the main-`ToTarget | TwoWay` state-machine target-dirty path for
default-context string targets. A manual `BindablePropertyString.propertyValue`
edit survives explicit data-context advancement, then the next normal
state-machine advance reapplies the unchanged boolean source through C++
boolean-to-string conversion, including when elapsed time is zero. The contract
is
`docs/prototypes/data-binding-graph-to-string-boolean-main-to-target-two-way-target-dirty-runtime-contract.md`.
Remaining `DataConverterToString` input kinds, string converter families and
groups, public-queue reverse conversion, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterToString` string-to-string binds now
also cover the main-`ToTarget | TwoWay` state-machine target-dirty path for
default-context string targets. A manual `BindablePropertyString.propertyValue`
edit survives explicit data-context advancement, then the next normal
state-machine advance reapplies the unchanged string source through C++ string
pass-through conversion, including when elapsed time is zero. The contract is
`docs/prototypes/data-binding-graph-to-string-string-main-to-target-two-way-target-dirty-runtime-contract.md`.
Remaining `DataConverterToString` input kinds, string converter families and
groups, public-queue reverse conversion, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterToString` trigger-to-string binds now
also cover the main-`ToTarget | TwoWay` state-machine target-dirty path for
default-context string targets. A manual `BindablePropertyString.propertyValue`
edit survives explicit data-context advancement, then the next normal
state-machine advance reapplies the unchanged trigger count through C++
trigger-to-string conversion, including when elapsed time is zero. The contract
is
`docs/prototypes/data-binding-graph-to-string-trigger-main-to-target-two-way-target-dirty-runtime-contract.md`.
Remaining `DataConverterToString` input kinds, string converter families and
groups, public-queue reverse conversion, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterToString`
symbol-list-index-to-string binds now also cover the main-`ToTarget | TwoWay`
state-machine target-dirty path for default-context string targets. A manual
`BindablePropertyString.propertyValue` edit survives explicit data-context
advancement, then the next normal state-machine advance reapplies the unchanged
raw symbol-list-index source through C++ decimal text conversion, including
when elapsed time is zero. The contract is
`docs/prototypes/data-binding-graph-to-string-symbol-list-index-main-to-target-two-way-target-dirty-runtime-contract.md`.
Remaining `DataConverterToString` input kinds, string converter families and
groups, public-queue reverse conversion, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterToString` color-to-string binds now
also cover the main-`ToTarget | TwoWay` state-machine target-dirty path for
default-context string targets. A manual `BindablePropertyString.propertyValue`
edit survives explicit data-context advancement, then the next normal
state-machine advance reapplies the unchanged color source through the imported
C++ `colorFormat` conversion, including when elapsed time is zero. The
contract is
`docs/prototypes/data-binding-graph-to-string-color-main-to-target-two-way-target-dirty-runtime-contract.md`.
Remaining `DataConverterToString` input kinds, string converter families and
groups, public-queue reverse conversion, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterToString` enum-to-string binds are now
explicitly pinned to C++'s empty-string fallback for the main-`ToTarget |
TwoWay` state-machine target-dirty path. A manual
`BindablePropertyString.propertyValue` edit survives explicit data-context
advancement, then the next normal state-machine advance overwrites it with an
empty string instead of enum metadata, matching C++'s display-label-unsupported
default-context string-target graph behavior. The contract is
`docs/prototypes/data-binding-graph-to-string-enum-main-to-target-two-way-target-dirty-runtime-contract.md`.
String converter families and groups, public-queue reverse conversion for
string converter families/groups, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterToString` now also covers public
`updateDataBinds(true)` target-to-source behavior for default-context
main-`ToTarget | TwoWay` string targets. C++ uses the base converter reverse
identity for the edited string target; non-string sources remain unchanged,
string sources receive the edited string, and the same public update reapplies
source-to-target through direct `DataConverterToString`. The contract is
`docs/prototypes/data-binding-graph-to-string-public-update-target-to-source-runtime-contract.md`.
String-source main-`ToSource | TwoWay` behavior for `DataConverterToString`,
string converter families and groups, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterToString` now also covers
main-`ToSource | TwoWay` target-to-source behavior for default-context string
targets backed by number, boolean, trigger, symbol-list-index, color, and enum
sources. C++ preserves the edited string target through explicit data-context
advancement because the reverse value does not match the non-string source,
then the next normal state-machine advance refreshes the target through base
`reverseConvert`, yielding the default empty string. The contract is
`docs/prototypes/data-binding-graph-to-string-non-string-main-to-source-target-to-source-runtime-contract.md`.
String-source main-`ToSource | TwoWay` behavior for `DataConverterToString`,
string converter families and groups, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterStringTrim` binds now cover the
main-`ToTarget | TwoWay` state-machine target-dirty path for default-context
string targets. A manual `BindablePropertyString.propertyValue` edit survives
explicit data-context advancement, then the next normal state-machine advance
reapplies the unchanged string source through imported C++ trim conversion,
including when elapsed time is zero. The contract is
`docs/prototypes/data-binding-graph-string-trim-main-to-target-two-way-target-dirty-runtime-contract.md`.
String converter groups, public-queue reverse conversion beyond the linked
direct string-family slice, broader dirty/update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: direct `DataConverterStringRemoveZeros` binds now cover the
main-`ToTarget | TwoWay` state-machine target-dirty path for default-context
string targets. A manual `BindablePropertyString.propertyValue` edit survives
explicit data-context advancement, then the next normal state-machine advance
reapplies the unchanged string source through C++ remove-zero conversion,
including when elapsed time is zero. The contract is
`docs/prototypes/data-binding-graph-string-remove-zeros-main-to-target-two-way-target-dirty-runtime-contract.md`.
String converter groups, public-queue reverse conversion beyond the linked
direct string-family slice, broader dirty/update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: direct `DataConverterStringPad` binds now cover the
main-`ToTarget | TwoWay` state-machine target-dirty path for default-context
string targets. A manual `BindablePropertyString.propertyValue` edit survives
explicit data-context advancement, then the next normal state-machine advance
reapplies the unchanged string source through imported C++ pad conversion,
including when elapsed time is zero. The contract is
`docs/prototypes/data-binding-graph-string-pad-main-to-target-two-way-target-dirty-runtime-contract.md`.
Converter groups, public-queue reverse conversion beyond the linked direct
string-family slice, broader dirty/update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: direct `DataConverterStringTrim`,
`DataConverterStringRemoveZeros`, and `DataConverterStringPad` now also cover
public `updateDataBinds(true)` target-to-source behavior for default-context
main-`ToTarget | TwoWay` string targets. C++ uses the base converter reverse
identity for the edited string target, writes that string into the default
view-model string source, then immediately reapplies source-to-target through
the direct string converter. The contract is
`docs/prototypes/data-binding-graph-string-converter-family-public-update-target-to-source-runtime-contract.md`.
String converter groups, main-`ToSource | TwoWay` behavior for string
converters, broader dirty/update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: direct `DataConverterStringTrim`,
`DataConverterStringRemoveZeros`, and `DataConverterStringPad` now also cover
main-`ToSource | TwoWay` target-to-source behavior for default-context string
targets. C++ preserves the edited string target through explicit data-context
advancement without mutating the string source, then the next normal
state-machine advance refreshes the target from the unchanged source through
base `reverseConvert` identity rather than forward trim/remove-zero/pad
conversion. The contract is
`docs/prototypes/data-binding-graph-string-converter-family-main-to-source-target-to-source-runtime-contract.md`.
String converter groups, string-source main-`ToSource | TwoWay` behavior for
`DataConverterToString`, broader dirty/update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: the first `DataConverterToString` runtime slice now
supports default-context number sources feeding
`BindablePropertyString.propertyValue` targets. String source nodes can carry a
number graph value when the data bind resolves to `DataConverterToString`, and
the converter descriptor stores imported `flags`/`decimals` so the graph can
apply the same C++ number formatting model already pinned in `rive-binary`
before string target writes. C++ probe coverage verifies the converted string
through a `TransitionViewModelCondition`. The contract is
`docs/prototypes/data-binding-graph-to-string-number-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, remaining
`DataConverterToString` input kinds and string converter families, converter
groups, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: `DataConverterToString` runtime execution now also admits
default-context boolean sources feeding `BindablePropertyString.propertyValue`
targets. The graph carries boolean source values on string-target bindings when
the data bind resolves to `DataConverterToString`, then applies the C++
`true -> "1"` and `false -> "0"` conversion before target writes. C++ probe
coverage verifies the converted string through a `TransitionViewModelCondition`.
The contract is
`docs/prototypes/data-binding-graph-to-string-boolean-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, remaining
`DataConverterToString` input kinds and string converter families, converter
groups, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: `DataConverterToString` runtime execution now also admits
default-context string sources feeding `BindablePropertyString.propertyValue`
targets. Converter-bearing string target bindings now carry raw source bytes
through the graph and preserve them unchanged before target writes, matching
C++ pass-through string conversion. C++ probe coverage verifies the converted
string through a `TransitionViewModelCondition`. The contract is
`docs/prototypes/data-binding-graph-to-string-string-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, remaining
`DataConverterToString` input kinds and string converter families, converter
groups, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: `DataConverterToString` runtime execution now also admits
default-context trigger sources feeding `BindablePropertyString.propertyValue`
targets. The graph carries raw trigger counts on string-target bindings when
the data bind resolves to `DataConverterToString`, then converts the count to
decimal text before target writes. C++ probe coverage verifies the converted
string through a `TransitionViewModelCondition`. The contract is
`docs/prototypes/data-binding-graph-to-string-trigger-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, remaining
`DataConverterToString` input kinds and string converter families, converter
groups, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: `DataConverterToString` runtime execution now also admits
default-context symbol-list-index sources feeding
`BindablePropertyString.propertyValue` targets. The graph carries raw
`ViewModelInstanceSymbolListIndex.propertyValue` source values on string-target
bindings when the data bind resolves to `DataConverterToString`, then converts
the index to decimal text before target writes. C++ probe coverage verifies the
converted string through a `TransitionViewModelCondition`. The contract is
`docs/prototypes/data-binding-graph-to-string-symbol-list-index-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, remaining
`DataConverterToString` input kinds and string converter families, converter
groups, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: `DataConverterToString` runtime execution now also admits
default-context color sources feeding `BindablePropertyString.propertyValue`
targets. The graph carries raw `ViewModelInstanceColor.propertyValue` source
values on string-target bindings when the data bind resolves to
`DataConverterToString`, stores imported `colorFormat` bytes on the converter
descriptor, and applies the C++ color-to-string formatter before target writes.
C++ probe coverage verifies the converted string through a
`TransitionViewModelCondition`. The contract is
`docs/prototypes/data-binding-graph-to-string-color-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, remaining
`DataConverterToString` input kinds and string converter families, converter
groups, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: `DataConverterToString` enum-source runtime graph behavior
is now pinned to C++'s empty-string fallback for default-context enum sources
feeding `BindablePropertyString.propertyValue` targets. Even with resolvable
imported `DataEnum` metadata, C++ does not take a string transition condition
for the enum display label, so Rust admits enum sources into this string-target
graph path only to write the same empty fallback. The contract is
`docs/prototypes/data-binding-graph-to-string-enum-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, string converter
families, converter groups, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: `DataConverterStringTrim` runtime execution now admits
default-context string sources feeding `BindablePropertyString.propertyValue`
targets. The graph stores imported `trimType` on the converter descriptor and
uses the shared C++-modeled trim helper from `rive-binary` before writing the
string target. C++ probe coverage verifies the trimmed string through a
`TransitionViewModelCondition`. The contract is
`docs/prototypes/data-binding-graph-string-trim-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, remaining string
converter families, converter groups, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: `DataConverterStringRemoveZeros` runtime execution now
admits default-context string sources feeding
`BindablePropertyString.propertyValue` targets. The graph recognizes the direct
converter and uses the shared C++-modeled trailing-zero remover from
`rive-binary` before writing the string target. C++ probe coverage verifies the
converted string through a `TransitionViewModelCondition`. The contract is
`docs/prototypes/data-binding-graph-string-remove-zeros-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, remaining string
converter families, converter groups, reverse propagation, update-queue parity,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: `DataConverterStringPad` runtime execution now admits
default-context string sources feeding `BindablePropertyString.propertyValue`
targets. The graph stores imported `length`, `text`, and `padType` on the
converter descriptor and uses the shared C++-modeled pad helper from
`rive-binary` before writing the string target. C++ probe coverage verifies the
padded string through a `TransitionViewModelCondition`. This closes the direct
string-converter family for the default-context string-source-to-string-target
runtime graph lane. The contract is
`docs/prototypes/data-binding-graph-string-pad-converter-runtime-contract.md`.
Stable public source handles, list/view-model bindables, converter groups,
reverse propagation, update-queue parity, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: the first `DataConverterGroup` runtime execution slice now
admits default-context string sources feeding
`BindablePropertyString.propertyValue` targets when the resolved group is made
from already-supported direct string converters. The graph stores ordered child
converter descriptors from imported `DataConverterGroupItem.converterId`
metadata, applies each child output into the next child, and treats cyclic or
unresolved children as unsupported. C++ probe coverage verifies a trim-then-pad
group through a `TransitionViewModelCondition`. The contract is
`docs/prototypes/data-binding-graph-string-converter-group-runtime-contract.md`.
Stable public source handles, list/view-model bindables, remaining converter
group shapes, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: the admitted string `DataConverterGroup` path now also
covers the main-`ToTarget | TwoWay` state-machine target-dirty behavior for
default-context string targets. A manual `BindablePropertyString.propertyValue`
edit survives explicit data-context advancement, then the next normal
state-machine advance reapplies the unchanged string source through imported
trim-then-pad group conversion in C++ child order, including when elapsed time
is zero. The contract is
`docs/prototypes/data-binding-graph-string-converter-group-main-to-target-two-way-target-dirty-runtime-contract.md`.
Cross-type and number converter groups, reverse propagation, broader
dirty/update queues, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: the admitted string `DataConverterGroup` path now also
covers public `updateDataBinds(true)` target-to-source behavior for
default-context main-`ToTarget | TwoWay` string targets. C++ writes the edited
target to the default string source through reverse group order, which is base
identity for the direct string children, then reapplies source-to-target
through forward trim-then-pad group conversion in the same update. The
contract is
`docs/prototypes/data-binding-graph-string-converter-group-public-update-target-to-source-runtime-contract.md`.
Cross-type and number converter groups, main-`ToSource | TwoWay` behavior for
converter groups, broader dirty/update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: the admitted string `DataConverterGroup` path now also
covers main-`ToSource | TwoWay` target-to-source behavior for default-context
string targets. C++ preserves the edited target through explicit data-context
advancement without mutating the string source, then the next normal
state-machine advance refreshes the target from the unchanged source through
reverse group order and base reverse identity for the direct string children.
The contract is
`docs/prototypes/data-binding-graph-string-converter-group-main-to-source-target-to-source-runtime-contract.md`.
Cross-type and number converter groups, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: the first cross-type `DataConverterGroup` runtime execution
slice now admits default-context number sources feeding
`BindablePropertyString.propertyValue` targets when the resolved group starts
with `DataConverterToString` and then flows through already-supported direct
string converters. The source-admission rule checks the first effective group
child before reading non-string sources, while conversion still executes the
ordered group pipeline and keeps cyclic or unresolved children unsupported. C++
probe coverage verifies a number-to-string-then-pad group through a
`TransitionViewModelCondition`. The contract is
`docs/prototypes/data-binding-graph-to-string-converter-group-runtime-contract.md`.
Stable public source handles, list/view-model bindables, remaining converter
group shapes, reverse propagation, update-queue parity, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: the admitted cross-type `DataConverterGroup` path now also
covers the main-`ToTarget | TwoWay` state-machine target-dirty behavior for
default-context number sources feeding string targets. A manual
`BindablePropertyString.propertyValue` edit survives explicit data-context
advancement, then the next normal state-machine advance reapplies the unchanged
number source through imported `DataConverterToString -> DataConverterStringPad`
group conversion in C++ child order, including when elapsed time is zero. The
contract is
`docs/prototypes/data-binding-graph-to-string-converter-group-main-to-target-two-way-target-dirty-runtime-contract.md`.
Number converter groups, reverse propagation, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: the admitted cross-type `DataConverterGroup` path now also
covers public `updateDataBinds(true)` target-to-source behavior for
default-context main-`ToTarget | TwoWay` string targets. C++ runs reverse group
conversion over the edited string target, does not mutate the number source
when the reverse value remains a string, then reapplies source-to-target from
the unchanged number through forward `DataConverterToString ->
DataConverterStringPad` group conversion in the same update. The contract is
`docs/prototypes/data-binding-graph-to-string-converter-group-public-update-target-to-source-runtime-contract.md`.
Number converter groups, cross-type main-`ToSource | TwoWay` behavior, broader
dirty/update queues, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: the admitted cross-type `DataConverterGroup` path now also
covers main-`ToSource | TwoWay` target-to-source behavior for default-context
string targets. C++ preserves the edited target through explicit data-context
advancement without mutating the number source, then the next normal
state-machine advance refreshes the target through reverse
`DataConverterStringPad -> DataConverterToString` group order to C++'s default
empty string fallback. The contract is
`docs/prototypes/data-binding-graph-to-string-converter-group-main-to-source-target-to-source-runtime-contract.md`.
Number converter groups, broader dirty/update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: the first number-to-number `DataConverterGroup` runtime
execution slice now admits default-context number sources feeding
`BindablePropertyNumber.propertyValue` targets when the resolved group is made
from already-supported number-output converters. C++ probe coverage verifies an
`OperationValue -> Rounder` group through a `BlendState1DViewModel` consumer.
The contract is
`docs/prototypes/data-binding-graph-number-converter-group-runtime-contract.md`.
Stable public source handles, list/view-model bindables, reverse conversion,
group children requiring live context binding beyond the first admitted
interpolator child path, formula randoms, generated lists, or target-to-source
queues, relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: the admitted number-to-number `DataConverterGroup` path
now also covers public `updateDataBinds(true)` target-to-source behavior for
default-context main-`ToTarget | TwoWay` number targets. C++ writes the edited
number target to the default number source through reverse
`DataConverterRounder -> DataConverterOperationValue` group order, then
reapplies source-to-target through forward `OperationValue -> Rounder`
conversion in the same update. The contract is
`docs/prototypes/data-binding-graph-number-converter-group-public-update-target-to-source-runtime-contract.md`.
Main-`ToSource | TwoWay` behavior for this number group, broader number
converter groups, broader dirty/update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: the admitted number-to-number `DataConverterGroup` path
now also covers main-`ToSource | TwoWay` target-to-source behavior for
default-context number targets. Explicit data-context advancement writes the
edited number target to the default number source through forward
`DataConverterOperationValue -> DataConverterRounder` group order, then
refreshes the target in the same dirty pass through reverse
`DataConverterRounder -> DataConverterOperationValue` group order. The
contract is
`docs/prototypes/data-binding-graph-number-converter-group-main-to-source-target-to-source-runtime-contract.md`.
Broader number converter groups, broader dirty/update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: the first stateful runtime data-converter slice now admits
direct `DataConverterInterpolator` bindings for default-context number sources
feeding `BindablePropertyNumber.propertyValue` targets. `RuntimeDataBindGraph`
owns per-source interpolator state, imports duration plus optional resolved
cubic/elastic interpolator descriptors, follows C++'s two-advance startup gate,
defers initialized zero-second retargets until a positive elapsed pass, and
keeps the state machine advancing while smoothing is active. C++ probe coverage
warms the converter, mutates the default source, and verifies partial/final
smoothing through an existing `BlendState1DViewModel` consumer. The contract is
`docs/prototypes/data-binding-graph-interpolator-converter-runtime-contract.md`.
Broader converter-group stateful scheduling, reverse conversion,
target-to-source queues, formula/number-to-list/generated-list/scripted
stateful scheduling, broader `DataBindContainer` dirty queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: stateful converter execution now extends through the first
`DataConverterGroup` path containing an interpolator child. Runtime group
converter state is stored as a tree matching imported group item order, so
already-admitted stateless number converters can feed a stateful
`DataConverterInterpolator` child and group advance aggregates child activity.
C++ probe coverage verifies an `OperationValue -> DataConverterInterpolator`
group after source mutation through the existing `BlendState1DViewModel`
consumer. The contract is
`docs/prototypes/data-binding-graph-interpolator-converter-group-runtime-contract.md`.
Reverse group conversion, target-to-source queues, formula/number-to-list/
generated-list/scripted stateful scheduling, context-aware group children,
broader `DataBindContainer` dirty queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: the admitted stateful `DataConverterGroup` path now also
covers public `updateDataBinds(true)` target-to-source behavior for warmed
main-`ToTarget | TwoWay` number targets. C++ applies reverse group order over
`DataConverterInterpolator -> DataConverterOperationValue`; the interpolator
child's `reverseConvert` delegates to its stateful `convert` using the existing
group child state tree, then the same public update reapplies source-to-target
through forward group order. The contract is
`docs/prototypes/data-binding-graph-interpolator-converter-group-public-update-target-to-source-runtime-contract.md`.
Fresh/unwarmed stateful-group public updates, main-`ToSource | TwoWay` and
state-machine target-dirty behavior for stateful groups, broader dirty queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: the admitted stateful `DataConverterGroup` path now also
covers main-`ToTarget | TwoWay` state-machine target-dirty behavior for warmed
number targets. A manual bindable number edit is preserved through explicit
data-context advancement, then the next normal state-machine advance reapplies
the unchanged source through forward
`DataConverterOperationValue -> DataConverterInterpolator` group order using
the existing group child state tree. The contract is
`docs/prototypes/data-binding-graph-interpolator-converter-group-main-to-target-two-way-target-dirty-runtime-contract.md`.
Fresh/unwarmed stateful-group target edits, main-`ToSource | TwoWay` behavior
for stateful groups, broader dirty queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: the admitted stateful `DataConverterGroup` path now also
covers main-`ToSource | TwoWay` target-to-source behavior for warmed number
targets. Explicit data-context advancement runs the edited target through
forward `DataConverterOperationValue -> DataConverterInterpolator` group order,
then refreshes the visible target during the same pass through reverse group
order with the same child state tree, even when the grouped source value is
unchanged. The contract is
`docs/prototypes/data-binding-graph-interpolator-converter-group-main-to-source-target-to-source-runtime-contract.md`.
Fresh/unwarmed stateful-group target edits, broader stateful group shapes,
color interpolation targets, broader dirty queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: the first deterministic `DataConverterFormula` runtime
execution slice now admits default-context number sources feeding
`BindablePropertyNumber.propertyValue` targets. Runtime formula descriptors are
built from the binary-layer C++ output queue, support `FormulaTokenInput`,
`FormulaTokenValue`, and `FormulaTokenOperation`, and use the same stack
collapse plus arithmetic operation behavior as C++ for deterministic formulas.
C++ probe coverage verifies the formula output through the existing
`BlendState1DViewModel` consumer. The contract is
`docs/prototypes/data-binding-graph-formula-converter-runtime-contract.md`.
Formula functions/randoms, formula parent-source binding and dirt propagation,
reverse conversion, target-to-source queues, number-to-list/generated-list/
scripted scheduling, broader `DataBindContainer` dirty queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: deterministic `DataConverterFormula` runtime execution now
also admits default-context symbol-list-index sources feeding
`BindablePropertyNumber.propertyValue` targets. The graph carries imported
`ViewModelInstanceSymbolListIndex.propertyValue` source values into the formula
converter as `f32`, matching C++ `DataValueSymbolListIndex` conversion before
formula stack evaluation. C++ probe coverage verifies the converted formula
output through the existing `BlendState1DViewModel` consumer. The contract is
`docs/prototypes/data-binding-graph-formula-symbol-list-index-converter-runtime-contract.md`.
Formula functions/randoms, formula parent-source binding and dirt propagation,
reverse conversion, target-to-source queues, number-to-list/generated-list/
scripted scheduling, broader `DataBindContainer` dirty queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: deterministic `DataConverterFormula` runtime execution now
also admits the first non-number fallback source:
`ViewModelInstanceBoolean.propertyValue` feeding
`BindablePropertyNumber.propertyValue`. The graph carries the boolean source
into the formula converter so C++'s early non-number branch writes `0.0`
instead of skipping the bind. C++ probe coverage uses a non-zero imported
bindable target default through the existing `BlendState1DViewModel` consumer
to prove the fallback write is observable. The contract is
`docs/prototypes/data-binding-graph-formula-boolean-fallback-runtime-contract.md`.
Formula fallback for enum/color/string/trigger and other non-number sources,
formula functions/randoms, formula parent-source binding and dirt propagation,
reverse conversion, target-to-source queues, number-to-list/generated-list/
scripted scheduling, broader `DataBindContainer` dirty queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: deterministic `DataConverterFormula` runtime execution now
also admits the remaining graph-represented non-number fallback sources for the
current number-target path: enum, color, string, and trigger. The graph carries
each source kind into the formula converter so C++'s early non-number branch
writes `0.0` instead of skipping the bind. C++ probe coverage uses non-zero
imported bindable target defaults through the existing `BlendState1DViewModel`
consumer to prove each fallback write is observable. The contract is
`docs/prototypes/data-binding-graph-formula-remaining-fallbacks-runtime-contract.md`.
Formula functions/randoms, formula parent-source binding and dirt propagation,
asset/artboard/view-model/list formula sources, reverse conversion,
target-to-source queues, number-to-list/generated-list/scripted scheduling,
broader `DataBindContainer` dirty queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: deterministic `DataConverterFormula` runtime execution now
also admits `FormulaTokenFunction` output-queue tokens for number and
symbol-list-index sources feeding number targets, including direct explicit
target-to-source, public `updateDataBinds(true)` target-to-source, and
main-`ToTarget | TwoWay` target-dirty scheduling for number-source
function-token formulas. The graph consumes the binary-layer formula output
descriptors so function argument counts match C++ shunting-yard resolution,
supports deterministic function types from `min` through `atangent2`, keeps
C++'s `0.0` fallback for unknown non-random function discriminants, casts
symbol-list-index inputs to `f32`, runs the same function-token formula
conversion before source writes and immediate source-to-target reapplication,
and preserves then overwrites manual target edits like C++. The contract is
`docs/prototypes/data-binding-graph-formula-functions-runtime-contract.md`.
Formula randoms, formula parent-source binding and dirt propagation,
asset/artboard/view-model/list formula sources, formula converter groups
beyond the admitted operation-value-to-formula public-update groups,
number-to-list/generated-list/scripted scheduling, broader
`DataBindContainer` dirty queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: deterministic `DataConverterFormula` now has its first
target-to-source runtime path. A main-`ToSource | TwoWay` number bind mutates a
`BindablePropertyNumber.propertyValue` target, applies the formula's imported
output queue through C++ main-direction `convert`, writes the
`ViewModelInstanceNumber.propertyValue` source, refreshes the formula-bound
target from that changed source during explicit data-context advancement, and a
second direct number bind observes the source value after normal
source-to-target application. The contract is
`docs/prototypes/data-binding-graph-formula-target-to-source-runtime-contract.md`.
Formula functions/randoms, formula parent-source binding and dirt propagation,
asset/artboard/view-model/list formula sources, formula groups,
main-`ToTarget | TwoWay` formula reverse scheduling, public
`DataBindContainer::updateDataBinds(true)` scheduling,
number-to-list/generated-list/scripted scheduling, broader dirty queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: deterministic `DataConverterFormula` now also covers the
main-`ToTarget | TwoWay` state-machine target-dirty path for direct number
binds. A manual edit to a formula-bound `BindablePropertyNumber.propertyValue`
target is preserved through explicit data-context advancement, then the next
normal state-machine advance overwrites the target from the unchanged source
through deterministic formula `convert`. The contract is
`docs/prototypes/data-binding-graph-formula-main-to-target-two-way-target-dirty-runtime-contract.md`.
Formula functions/randoms, formula parent-source binding and dirt propagation,
asset/artboard/view-model/list formula sources, formula groups, public
`DataBindContainer::updateDataBinds(true)` formula reverse scheduling,
number-to-list/generated-list/scripted scheduling, broader dirty queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: the first graph-owned target-to-source runtime path now
covers direct default-context number binds. Mutating a
`BindablePropertyNumber.propertyValue` target for a `ToSource | TwoWay`
`DataBindContext` marks that graph binding dirty; explicit
`advance_data_context` writes the target value back into the bound
`ViewModelInstanceNumber.propertyValue` source before normal source-to-target
application. C++ probe coverage uses two binds to the same source so the second
bind's existing `BlendState1DViewModel` consumer observes the target-to-source
write without adding new probe report fields. The contract is
`docs/prototypes/data-binding-graph-number-target-to-source-runtime-contract.md`.
Target-to-source for other value kinds, pure `ToSource` without `TwoWay`,
reverse converter execution, imported/owned contexts, pending dirty queues,
pending add/remove behavior, re-entry protection, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: graph-owned target-to-source runtime behavior now also
covers direct default-context boolean binds. Mutating a
`BindablePropertyBoolean.propertyValue` target for a `ToSource | TwoWay`
`DataBindContext` marks that graph binding dirty; explicit
`advance_data_context` writes the target value back into the bound
`ViewModelInstanceBoolean.propertyValue` source before normal source-to-target
application. C++ probe coverage uses two binds to the same source so the second
bind's existing `TransitionViewModelCondition` consumer observes the
target-to-source write without adding new probe report fields. The contract is
`docs/prototypes/data-binding-graph-boolean-target-to-source-runtime-contract.md`.
Target-to-source for string, color, enum, asset, artboard, trigger,
symbol-list-index, view-model, and list value kinds, pure `ToSource` without
`TwoWay`, reverse converter execution, imported/owned contexts, pending dirty
queues, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: graph-owned target-to-source runtime behavior now also
covers direct default-context string binds. Mutating a
`BindablePropertyString.propertyValue` target for a `ToSource | TwoWay`
`DataBindContext` marks that graph binding dirty; explicit
`advance_data_context` writes the target byte payload back into the bound
`ViewModelInstanceString.propertyValue` source before normal source-to-target
application. C++ probe coverage uses two binds to the same source so the second
bind's existing `TransitionViewModelCondition` consumer observes the
target-to-source write without adding new probe report fields. The contract is
`docs/prototypes/data-binding-graph-string-target-to-source-runtime-contract.md`.
Target-to-source for color, enum, asset, artboard, trigger, symbol-list-index,
view-model, and list value kinds, pure `ToSource` without `TwoWay`, reverse
converter execution, imported/owned contexts, pending dirty queues, pending
add/remove behavior, re-entry protection, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: graph-owned target-to-source runtime behavior now also
covers direct default-context color binds. Mutating a
`BindablePropertyColor.propertyValue` target for a `ToSource | TwoWay`
`DataBindContext` marks that graph binding dirty; explicit
`advance_data_context` writes the target color value back into the bound
`ViewModelInstanceColor.propertyValue` source before normal source-to-target
application. C++ probe coverage uses two binds to the same source so the second
bind's existing `TransitionViewModelCondition` consumer observes the
target-to-source write without adding new probe report fields. The contract is
`docs/prototypes/data-binding-graph-color-target-to-source-runtime-contract.md`.
Target-to-source for enum, asset, artboard, trigger, symbol-list-index,
view-model, and list value kinds, pure `ToSource` without `TwoWay`, reverse
converter execution, imported/owned contexts, pending dirty queues, pending
add/remove behavior, re-entry protection, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: graph-owned target-to-source runtime behavior now also
covers direct default-context enum binds. Mutating a
`BindablePropertyEnum.propertyValue` target for a `ToSource | TwoWay`
`DataBindContext` marks that graph binding dirty; explicit
`advance_data_context` writes the target enum value back into the bound
`ViewModelInstanceEnum.propertyValue` source before normal source-to-target
application. C++ probe coverage uses two binds to the same source so the second
bind's existing `TransitionViewModelCondition` consumer observes the
target-to-source write without adding new probe report fields. The contract is
`docs/prototypes/data-binding-graph-enum-target-to-source-runtime-contract.md`.
Target-to-source for asset, artboard, trigger, symbol-list-index, view-model,
and list value kinds, pure `ToSource` without `TwoWay`, reverse converter
execution, imported/owned contexts, pending dirty queues, pending add/remove
behavior, re-entry protection, relative/parent/nested lookup, listener-owned
data binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: graph-owned target-to-source runtime behavior now also
covers direct default-context asset binds. Mutating a
`BindablePropertyAsset.propertyValue` target for a `ToSource | TwoWay`
`DataBindContext` marks that graph binding dirty; explicit
`advance_data_context` writes the target asset ID back into the bound
`ViewModelInstanceAssetImage.propertyValue` source before normal
source-to-target application. C++ probe coverage uses two binds to the same
source so the second bind's existing `TransitionViewModelCondition` consumer
observes the target-to-source write without adding new probe report fields. The
contract is
`docs/prototypes/data-binding-graph-asset-target-to-source-runtime-contract.md`.
Target-to-source for artboard, trigger, symbol-list-index, view-model, and list
value kinds, pure `ToSource` without `TwoWay`, reverse converter execution,
imported/owned contexts, pending dirty queues, pending add/remove behavior,
re-entry protection, relative/parent/nested lookup, listener-owned data binding,
and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: graph-owned target-to-source runtime behavior now also
covers direct default-context artboard binds. Mutating a
`BindablePropertyArtboard.propertyValue` target for a `ToSource | TwoWay`
`DataBindContext` marks that graph binding dirty; explicit
`advance_data_context` writes the target artboard ID back into the bound
`ViewModelInstanceArtboard.propertyValue` source before normal source-to-target
application. C++ probe coverage uses two binds to the same source so the second
bind's existing `TransitionViewModelCondition` consumer observes the
target-to-source write without adding new probe report fields. The contract is
`docs/prototypes/data-binding-graph-artboard-target-to-source-runtime-contract.md`.
Target-to-source for trigger, symbol-list-index, view-model, and list value
kinds, pure `ToSource` without `TwoWay`, reverse converter execution,
imported/owned contexts, pending dirty queues, pending add/remove behavior,
re-entry protection, relative/parent/nested lookup, listener-owned data binding,
and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: graph-owned target-to-source runtime behavior now also
covers direct default-context symbol-list-index sources through
`BindablePropertyInteger.propertyValue` targets. C++ has no separate
`BindablePropertySymbolListIndex` schema type; the direct reverse path reads a
uint-like target and writes it back into
`ViewModelInstanceSymbolListIndex.propertyValue`. Mutating the integer target
for a `ToSource | TwoWay` `DataBindContext` marks that graph binding dirty;
explicit `advance_data_context` writes the target integer into the bound
symbol-list-index source before normal source-to-target application. C++ probe
coverage uses two binds to the same source so the second bind's existing
symbol-list-index-to-string `TransitionViewModelCondition` consumer observes
the target-to-source write without adding new probe report fields. The contract
is
`docs/prototypes/data-binding-graph-symbol-list-index-target-to-source-runtime-contract.md`.
Target-to-source for trigger, view-model, and list value kinds, pure `ToSource`
without `TwoWay`, reverse converter execution, imported/owned contexts, pending
dirty queues, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: graph-owned target-to-source runtime behavior now also
covers direct default-context trigger binds. Mutating a
`BindablePropertyTrigger.propertyValue` target for a `ToSource | TwoWay`
`DataBindContext` marks that graph binding dirty; explicit
`advance_data_context` writes the target trigger count back into the bound
`ViewModelInstanceTrigger.propertyValue` source before normal source-to-target
application, and the existing trigger reset still runs afterward. C++ probe
coverage uses two binds to the same source so the second bind's existing
trigger-to-string `TransitionViewModelCondition` consumer observes the
target-to-source write without adding new probe report fields. The contract is
`docs/prototypes/data-binding-graph-trigger-target-to-source-runtime-contract.md`.
Target-to-source for list value kinds, pure `ToSource` without
`TwoWay`, reverse converter execution, imported/owned contexts, pending dirty
queues, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: `BindablePropertyViewModel.propertyValue` now has its first
graph-owned source binding slice. Default-context
`ViewModelInstanceViewModel.propertyValue` sources resolve through the
binary-layer C++ view-model reference model, carry imported view-model instance
identity as a runtime graph value, and write that identity into
state-machine bindable view-model targets on explicit data-context advance
before the next transition evaluation. C++ probe coverage verifies the bound
pointer through a `TransitionViewModelCondition` pointer comparison against a
null bindable. The contract is
`docs/prototypes/data-binding-graph-viewmodel-bind-source-runtime-contract.md`.
Stable public source handles, list bindables, reverse propagation, broader
update-queue parity, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: default-context view-model pointer sources now have a
probe-backed raw mutation path. `RuntimeDataBindGraph` stores the imported
instance IDs for the source's referenced view model, so
`StateMachineInstance::set_default_view_model_view_model_source_for_data_bind`
can accept a referenced instance index. Like C++, this generated-setter-style
raw index write does not relink the cached imported
`referenceViewModelInstance`, so explicit `advance_data_context` does not make a
pointer equality transition observe a new instance. The C++ probe gained
`--runtime-set-default-view-model-source-viewmodel`, and coverage verifies this
non-relinking behavior. The contract is
`docs/prototypes/data-binding-graph-viewmodel-source-mutation-runtime-contract.md`.
Stable public source handles, list bindables, non-default view-model pointer
mutation through live relink APIs, reverse propagation, broader update-queue
parity, relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: graph-owned target-to-source runtime behavior now also
covers direct default-context view-model pointer binds. Mutating a
`BindablePropertyViewModel.propertyValue` target for a `ToSource | TwoWay`
`DataBindContext` marks that graph binding dirty; explicit
`advance_data_context` resolves the requested referenced imported instance,
relinks the bound `ViewModelInstanceViewModel` graph source, and propagates the
same pointer identity to other graph source nodes sharing the same source path
before normal source-to-target application. The C++ probe gained
`--runtime-set-state-machine-bindable-viewmodel` plus direct
`viewModelBindings` reports because post-relink transition evaluation can
dereference a missing to-target data bind for a to-source view-model fixture.
The contract is
`docs/prototypes/data-binding-graph-viewmodel-target-to-source-runtime-contract.md`.
Target-to-source for list value kinds, pure `ToSource` without `TwoWay`,
reverse converter execution, imported/owned contexts, broader dirty/update
queues, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: pure `ToSource` without `TwoWay` is now probe-backed for
the already admitted direct target-to-source paths. The graph's shared flag
helpers already matched C++ by treating pure `ToSource` as
target-to-source-capable but not source-to-target-capable; a representative
default-context number fixture now proves that a pure `ToSource`
`BindablePropertyNumber.propertyValue` target mutation writes back to the
source and is then observed by a second `ToTarget` bind to the same source
path. The contract is
`docs/prototypes/data-binding-graph-pure-to-source-target-runtime-contract.md`.
Target-to-source for list value kinds, reverse converter execution,
imported/owned contexts, broader dirty/update queues, pending add/remove
behavior, re-entry protection, relative/parent/nested lookup, listener-owned
data binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: reverse-converter target-to-source runtime behavior has
started with `DataConverterBooleanNegate`. A default-context boolean fixture
now mutates a `BindablePropertyBoolean.propertyValue` target on a
`ToSource | TwoWay` bind with `DataConverterBooleanNegate`; explicit
`advance_data_context` applies C++'s symmetric `reverseConvert` negation before
writing the `ViewModelInstanceBoolean.propertyValue` source, and a second
ordinary `ToTarget` boolean bind observes the reversed source value through an
existing transition-condition consumer. The contract is
`docs/prototypes/data-binding-graph-boolean-negate-target-to-source-runtime-contract.md`.
At that point, reverse conversion for other converters and converter groups,
list source/target propagation, imported/owned contexts, broader dirty/update
queues, pending add/remove behavior, re-entry protection, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remained
follow-up `#12` slices.

Current #12 update: `DataConverterRangeMapper` now has graph-owned
target-to-source coverage for the reachable main-`ToSource | TwoWay` path and
the Rust reverse primitive. The state-machine probe mutates a range-mapped
`BindablePropertyNumber.propertyValue` target, verifies the exact mutating bind
reports, and verifies a second direct number bind after normal source-to-target
application. The reverse primitive mirrors C++ `calculateReverseRange()` by
swapping input/output ranges while preserving range-mapper flags. The contract
is
`docs/prototypes/data-binding-graph-range-mapper-target-to-source-runtime-contract.md`.
Main-`ToTarget | TwoWay` range-mapper edits through the state-machine
bindable-property action path are now covered by the shared number dirty
contract below. At that point, public
`DataBindContainer::updateDataBinds(true)` scheduling outside that path,
range-mapper groups in target-to-source scheduling, remaining converter
families, list source/target propagation, imported/owned contexts, broader
dirty/update queues, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remained follow-up `#12` slices.

Current #12 update: range-mapper target-to-source execution now includes the
first direct `DataConverterGroup` shape containing a `DataConverterRangeMapper`.
A main-`ToSource | TwoWay` grouped number bind applies child converters in C++
forward group order (`RangeMapper -> OperationValue`) before writing the
source, and a second direct bind observes that grouped source value after
normal source-to-target application. The contract is
`docs/prototypes/data-binding-graph-range-mapper-group-target-to-source-runtime-contract.md`.
Main-`ToTarget | TwoWay` reverse group scheduling, public
`DataBindContainer::updateDataBinds(true)` scheduling, resolved-interpolator
range-mapper group children, stateful converter children, remaining converter
families, list source/target propagation, imported/owned contexts, broader
dirty/update queues, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: graph-owned number target-to-source binding now pins C++
main-direction converter dispatch with exact source/target probe reporting. A
`ToSource | TwoWay` numeric bind no longer eagerly writes its own bindable
target on initial data-context advance, and mutated direct,
`DataConverterOperationValue`, and `DataConverterGroup<OperationValue>`
targets write the exact C++ source value for the mutating bind. For main
`ToSource` bindings this means `convert` and forward group order, not
`reverseConvert` solely because the data flow is target-to-source. The
contracts are
`docs/prototypes/data-binding-graph-number-target-to-source-direction-runtime-contract.md`,
`docs/prototypes/data-binding-graph-operation-value-target-to-source-runtime-contract.md`,
and
`docs/prototypes/data-binding-graph-operation-value-group-target-to-source-runtime-contract.md`.

Current #12 update: main-`ToTarget | TwoWay` number bindings with
`DataConverterOperationValue`, `DataConverterGroup<OperationValue>`, and direct
`DataConverterRangeMapper` now pin C++'s target-dirty behavior. A manual
bindable target edit is preserved through explicit `advancedDataContext()`,
does not run `reverseConvert`, and is overwritten from the unchanged source
through forward `convert` on the next normal state-machine advance. The
contract is
`docs/prototypes/data-binding-graph-number-main-to-target-two-way-target-to-source-runtime-contract.md`.
The first same-path ordinary direct `ToTarget` observer for a dirty direct
`DataConverterOperationValue` bind is also probe-backed by
`docs/prototypes/data-binding-graph-operation-value-main-to-target-observer-runtime-contract.md`.
Exact public `updateDataBinds(true)` dirty-list scheduler parity and broader
dirty-list scheduler parity for arbitrary neighboring ordinary `ToTarget`
bindable targets, symbol-list-index sources, other converter families, list
source/target propagation, imported/owned contexts, pending add/remove
behavior, re-entry protection, relative/parent/nested lookup, listener-owned
data binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: the first public `updateDataBinds(true)`
target-to-source paths are now probe-backed for main-`ToTarget | TwoWay`
number binds with no converter, direct `DataConverterOperationValue`, grouped
`DataConverterGroup<OperationValue>`, direct `DataConverterRangeMapper`, and
direct `DataConverterGroup<RangeMapper, OperationValue>`, direct
`DataConverterRounder`, direct system operation-value converters, direct
`DataConverterOperationViewModel`, direct cross-type `DataConverterToNumber`,
direct deterministic `DataConverterFormula`, direct
`DataConverterGroup<OperationValue, Formula>`, warmed direct
`DataConverterInterpolator`, concrete `DataConverterOperation` pass-through,
non-scripting `ScriptedDataConverter` pass-through, direct boolean and
`DataConverterBooleanNegate` boolean binds, direct `DataConverterTrigger`, and
direct `DataConverterToString` string binds.
The C++ probe exposes `--runtime-update-state-machine-data-binds`, Rust
mirrors it through
`StateMachineInstance::update_data_binds_apply_target_to_source`, and the
tests verify that edited bindable targets are written or reverse-converted
into the default view-model source before source-to-target reapplication
leaves the target at the edited value. The first same-path ordinary
`ToTarget` observer scheduling slice now also pins C++'s public-update
preserve-then-next-advance ordering. The contracts are
`docs/prototypes/data-binding-graph-number-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-operation-value-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-operation-value-group-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-boolean-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-range-mapper-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-range-mapper-group-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-rounder-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-system-operation-value-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-operation-viewmodel-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-operation-viewmodel-secondary-source-mutation-runtime-contract.md`,
`docs/prototypes/data-binding-graph-operation-viewmodel-group-secondary-source-mutation-runtime-contract.md`,
`docs/prototypes/data-binding-graph-to-number-boolean-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-to-number-remaining-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-to-string-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-string-converter-family-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-formula-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-formula-group-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-operation-pass-through-runtime-contract.md`,
`docs/prototypes/data-binding-graph-scripted-pass-through-runtime-contract.md`,
`docs/prototypes/data-binding-graph-trigger-public-update-target-to-source-runtime-contract.md`,
`docs/prototypes/data-binding-graph-public-update-observer-preservation-runtime-contract.md`,
and
`docs/prototypes/data-binding-graph-interpolator-public-update-target-to-source-runtime-contract.md`.
Public-update coverage for remaining converter families, broader
mixed/stateful groups, full dirty-list scheduler parity beyond the admitted
same-path ordinary `ToTarget` observer ordering cases, imported/owned contexts,
pending add/remove behavior, re-entry protection, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: grouped `DataConverterOperationViewModel` public
`updateDataBinds(true)` target-to-source behavior now has a narrow parity
slice. A main-`ToTarget | TwoWay` number bind uses a `DataConverterGroup`
containing `DataConverterOperationValue` followed by
`DataConverterOperationViewModel`; public update reverse-converts the edited
target through the group in C++ child order, resolves the operation-view-model
secondary operand from the imported default root view-model instance, writes
the primary number source, and reapplies source-to-target in the same update.
The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-group-public-update-target-to-source-runtime-contract.md`.
Other grouped operation-view-model compositions, remaining mixed/stateful
groups, full dirty-list scheduler parity, imported/owned contexts, pending
add/remove behavior, re-entry protection, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: grouped `DataConverterOperationViewModel` secondary-source
mutation now has a narrow parity slice. A
`DataConverterGroup<OperationValue, OperationViewModel>` bind refreshes the
nested operation-view-model operand and dirties the owning source when the
secondary default view-model number changes through the state-machine default
source API, matching C++ dependency registration from the group's child
converter. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-group-secondary-source-mutation-runtime-contract.md`.
Other grouped operation-view-model compositions, remaining mixed/stateful
groups, full dirty-list scheduler parity, imported/owned contexts, pending
add/remove behavior, re-entry protection, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: grouped `DataConverterOperationViewModel` converter source
paths now have the same explicit name-path unsupported boundary as the direct
converter slice. A `DataConverterGroup<OperationValue, OperationViewModel>`
child converter receives a manifest path id for `factor`; C++ forwards
`bindFromContext()` to the child, the child still calls
`DataContext::getViewModelProperty(sourcePathIds)` directly, and the
secondary operand remains missing with the operand `0.0` fallback. Rust
mirrors that behavior through the grouped blend-state probe. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-group-name-path-unsupported-runtime-contract.md`.
Additional grouped operation-view-model compositions, remaining
mixed/stateful groups, full dirty-list scheduler parity, imported/owned
contexts, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: boolean public `updateDataBinds(true)` target-to-source
behavior now covers direct boolean binds and direct `DataConverterBooleanNegate`.
The C++ probe reports exact boolean binding source/target values, and Rust
compares main-`ToTarget | TwoWay` default-context fixtures where public update
writes the edited target into the source directly or through symmetric
BooleanNegate reverse conversion before source-to-target reapplication. The
contract is
`docs/prototypes/data-binding-graph-boolean-public-update-target-to-source-runtime-contract.md`.
Boolean converter groups, imported/owned contexts, broader dirty-list
scheduler parity beyond the admitted same-path direct boolean observer slice,
pending add/remove behavior, re-entry protection, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: boolean public-update same-path observer scheduling now
has its first non-number parity slice. A dirty main-`ToTarget | TwoWay`
boolean bind writes the shared boolean source during public
`updateDataBinds(true)`, while a neighboring ordinary direct `ToTarget`
boolean bind to the same source path reports the new source but preserves its
previous target until the next normal state-machine advance. The contract is
`docs/prototypes/data-binding-graph-boolean-public-update-observer-preservation-runtime-contract.md`.
Cross-type observers, BooleanNegate observers, multiple observers,
imported/owned contexts, full dirty-list scheduler parity, pending add/remove
behavior, re-entry protection, relative/parent/nested lookup, listener-owned
data binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: string public-update same-path observer scheduling now
covers direct byte-backed string values. A dirty main-`ToTarget | TwoWay`
string bind writes the shared string source during public
`updateDataBinds(true)`, while a neighboring ordinary direct `ToTarget` string
bind to the same source path reports the new source but preserves its previous
target until the next normal state-machine advance. The contract is
`docs/prototypes/data-binding-graph-string-public-update-observer-preservation-runtime-contract.md`.
Cross-type observers, `DataConverterToString` and string-family converter
observers, multiple observers, imported/owned contexts, full dirty-list
scheduler parity, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: color public `updateDataBinds(true)` target-to-source and
same-path observer scheduling now have a direct-value parity slice. A dirty
main-`ToTarget | TwoWay` color bind writes the shared color source during
public update and reapplies its own target immediately, while a neighboring
ordinary direct `ToTarget` color bind to the same source path reports the new
source but preserves its previous target until the next normal state-machine
advance. The contract is
`docs/prototypes/data-binding-graph-color-public-update-observer-preservation-runtime-contract.md`.
Color converters, cross-type observers, multiple observers,
imported/owned contexts, full dirty-list scheduler parity, pending add/remove
behavior, re-entry protection, relative/parent/nested lookup, listener-owned
data binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: enum public `updateDataBinds(true)` target-to-source and
same-path observer scheduling now have a direct-value parity slice. A dirty
main-`ToTarget | TwoWay` enum bind writes the shared enum source during public
update and reapplies its own target immediately, while a neighboring ordinary
direct `ToTarget` enum bind to the same source path reports the new source but
preserves its previous target until the next normal state-machine advance. The
contract is
`docs/prototypes/data-binding-graph-enum-public-update-observer-preservation-runtime-contract.md`.
Enum converters, cross-type observers, multiple observers,
imported/owned contexts, full dirty-list scheduler parity, pending add/remove
behavior, re-entry protection, relative/parent/nested lookup, listener-owned
data binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: asset public `updateDataBinds(true)` target-to-source and
same-path observer scheduling now have a direct-value parity slice. A dirty
main-`ToTarget | TwoWay` asset bind writes the shared asset source during
public update and reapplies its own target immediately, while a neighboring
ordinary direct `ToTarget` asset bind to the same source path reports the new
source but preserves its previous target until the next normal state-machine
advance. The contract is
`docs/prototypes/data-binding-graph-asset-public-update-observer-preservation-runtime-contract.md`.
Asset loading/replacement behavior, cross-type observers, multiple observers,
imported/owned contexts, full dirty-list scheduler parity, pending add/remove
behavior, re-entry protection, relative/parent/nested lookup, listener-owned
data binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: artboard public `updateDataBinds(true)` target-to-source
and same-path observer scheduling now have a direct-value parity slice. A
dirty main-`ToTarget | TwoWay` artboard bind writes the shared artboard source
during public update and reapplies its own target immediately, while a
neighboring ordinary direct `ToTarget` artboard bind to the same source path
reports the new source but preserves its previous target until the next normal
state-machine advance. The contract is
`docs/prototypes/data-binding-graph-artboard-public-update-observer-preservation-runtime-contract.md`.
Nested artboard instancing/advancement, cross-type observers, multiple
observers, imported/owned contexts, full dirty-list scheduler parity, pending
add/remove behavior, re-entry protection, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: direct symbol-list-index public `updateDataBinds(true)`
target-to-source and same-path observer scheduling now have an integer-target
parity slice. A dirty main-`ToTarget | TwoWay`
`BindablePropertyInteger.propertyValue` bind writes the shared
`ViewModelInstanceSymbolListIndex.propertyValue` source during public update
and reapplies its own target immediately, while a neighboring ordinary direct
`ToTarget` integer bind to the same source path reports the new source but
preserves its previous target until the next normal state-machine advance. The
contract is
`docs/prototypes/data-binding-graph-symbol-list-index-public-update-observer-preservation-runtime-contract.md`.
Converter-backed symbol-list-index paths, cross-type observers, multiple
observers, imported/owned contexts, full dirty-list scheduler parity, pending
add/remove behavior, re-entry protection, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: direct trigger public `updateDataBinds(true)` same-path
observer scheduling now has a parity slice. The existing dirty
main-`ToTarget | TwoWay` trigger public-update path still writes the shared
trigger source and reapplies its own target immediately, while a neighboring
ordinary direct `ToTarget` trigger bind to the same source path reports the
new source but preserves its previous target until the next normal
state-machine advance. The contract is
`docs/prototypes/data-binding-graph-trigger-public-update-observer-preservation-runtime-contract.md`.
Trigger converters/groups, listener dispatch, cross-type observers, multiple
observers, imported/owned contexts, full dirty-list scheduler parity, pending
add/remove behavior, re-entry protection, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: direct view-model pointer public
`updateDataBinds(true)` target-to-source and same-path observer scheduling now
have a parity slice. A dirty main-`ToTarget | TwoWay`
`BindablePropertyViewModel.propertyValue` bind writes the shared
`ViewModelInstanceViewModel.propertyValue` source during public update and
reapplies its own target immediately. Unlike the scalar direct value lanes, a
neighboring ordinary direct `ToTarget` view-model bind to the same source path
also updates its target during the same public update. The contract is
`docs/prototypes/data-binding-graph-viewmodel-public-update-observer-application-runtime-contract.md`.
The state-machine bindable-property action path now has the matching
main-`ToTarget | TwoWay` observer boundary: explicit `advanceDataContext()`
preserves the mutating target pointer, leaves the shared source and same-path
ordinary `ToTarget` observer target unapplied, then normal state-machine
advancement reapplies source-to-target for both pointer binds. The contract is
`docs/prototypes/data-binding-graph-viewmodel-main-to-target-observer-runtime-contract.md`.
Imported/owned/nested view-model contexts, relink APIs, pointer comparator
transition behavior, multiple observers, cross-type observers, full dirty-list
scheduler parity, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: default-context number source mutation by state-machine
data-bind index now updates same-path graph source nodes instead of only the
selected cloned edge. A C++ probe mutates a shared
`ViewModelInstanceNumber.propertyValue` through the first data bind and proves
a neighboring ordinary direct `ToTarget` number bind reports the updated
source and applies the updated target on the next state-machine advance. The
contract is
`docs/prototypes/data-binding-graph-default-number-source-mutation-runtime-contract.md`.
Same-path data-bind-index source mutation for non-number/non-boolean
families, imported and owned contexts, full dirty-list scheduler parity,
pending add/remove behavior, re-entry protection, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: default-context boolean source mutation by state-machine
data-bind index now updates same-path graph source nodes instead of only the
selected cloned edge. A C++ probe mutates a shared
`ViewModelInstanceBoolean.propertyValue` through the first data bind and
proves a neighboring ordinary direct `ToTarget` boolean bind reports the
updated source and applies the updated target on the next state-machine
advance. The contract is
`docs/prototypes/data-binding-graph-default-boolean-source-mutation-runtime-contract.md`.
Same-path data-bind-index source mutation for remaining
non-number/non-boolean/non-string families, imported and owned contexts, full
dirty-list scheduler parity, pending add/remove behavior, re-entry
protection, relative/parent/nested lookup, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: default-context string source mutation by state-machine
data-bind index now updates same-path graph source nodes instead of only the
selected cloned edge. A C++ probe mutates a shared
`ViewModelInstanceString.propertyValue` through the first data bind and proves
a neighboring ordinary direct `ToTarget` string bind reports the updated
source bytes and applies the updated target on the next state-machine
advance. The contract is
`docs/prototypes/data-binding-graph-default-string-source-mutation-runtime-contract.md`.
Same-path data-bind-index source mutation for remaining
non-number/non-boolean/non-string families, imported and owned contexts, full
dirty-list scheduler parity, pending add/remove behavior, re-entry
protection, relative/parent/nested lookup, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: default-context color source mutation by state-machine
data-bind index now updates same-path graph source nodes instead of only the
selected cloned edge. A C++ probe mutates a shared
`ViewModelInstanceColor.propertyValue` through the first data bind and proves
a neighboring ordinary direct `ToTarget` color bind reports the updated source
and applies the updated target on the next state-machine advance. The contract
is
`docs/prototypes/data-binding-graph-default-color-source-mutation-runtime-contract.md`.
Same-path data-bind-index source mutation for remaining
non-number/non-boolean/non-string/non-color/non-enum families, imported and
owned contexts, full dirty-list scheduler parity, pending add/remove
behavior, re-entry protection, relative/parent/nested lookup, listener-owned
data binding, color-space/render side effects, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: default-context enum source mutation by state-machine
data-bind index now updates same-path graph source nodes instead of only the
selected cloned edge. A C++ probe mutates a shared
`ViewModelInstanceEnum.propertyValue` raw index through the first data bind
and proves a neighboring ordinary direct `ToTarget` enum bind reports the
updated source and applies the updated target on the next state-machine
advance. The contract is
`docs/prototypes/data-binding-graph-default-enum-source-mutation-runtime-contract.md`.
Same-path data-bind-index source mutation for remaining
non-number/non-boolean/non-string/non-color/non-enum/non-symbol-list-index/non-asset/non-artboard/non-trigger/non-list
families, imported and owned contexts, enum key/name APIs, full dirty-list
scheduler parity, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, `Solo` name
mapping, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: default-context symbol-list-index source mutation by
state-machine data-bind index now updates same-path graph source nodes instead
of only the selected cloned edge. A C++ probe mutates a shared
`ViewModelInstanceSymbolListIndex.propertyValue` raw index through the first
data bind and proves a neighboring ordinary direct `ToTarget` integer bind
reports the updated symbol-list-index source and applies the updated target on
the next state-machine advance. The contract is
`docs/prototypes/data-binding-graph-default-symbol-list-index-source-mutation-runtime-contract.md`.
Same-path data-bind-index source mutation for remaining
non-number/non-boolean/non-string/non-color/non-enum/non-symbol-list-index/non-asset/non-artboard/non-trigger/non-list
families, imported and owned contexts, property-name APIs for remaining families,
full dirty-list scheduler parity, pending add/remove behavior, re-entry
protection, relative/parent/nested lookup, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: default-context asset source mutation by state-machine
data-bind index now updates same-path graph source nodes instead of only the
selected cloned edge. A C++ probe mutates a shared
`ViewModelInstanceAssetImage.propertyValue` raw asset id through the first
data bind and proves a neighboring ordinary direct `ToTarget` asset bind
reports the updated source and applies the updated target on the next
state-machine advance. The contract is
`docs/prototypes/data-binding-graph-default-asset-source-mutation-runtime-contract.md`.
Same-path data-bind-index source mutation for remaining
non-number/non-boolean/non-string/non-color/non-enum/non-symbol-list-index/non-asset/non-artboard/non-trigger/non-list
families, imported and owned contexts, file-asset/render-image side effects,
property-name APIs for remaining families, full dirty-list scheduler parity,
pending add/remove behavior, re-entry protection, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: default-context artboard source mutation by state-machine
data-bind index now updates same-path graph source nodes instead of only the
selected cloned edge. A C++ probe mutates a shared
`ViewModelInstanceArtboard.propertyValue` raw artboard id through the first
data bind and proves a neighboring ordinary direct `ToTarget` artboard bind
reports the updated source and applies the updated target on the next
state-machine advance. The contract is
`docs/prototypes/data-binding-graph-default-artboard-source-mutation-runtime-contract.md`.
Same-path data-bind-index source mutation for remaining
non-number/non-boolean/non-string/non-color/non-enum/non-symbol-list-index/non-asset/non-artboard/non-trigger/non-list
families, imported and owned contexts, artboard referencer/remapping side
effects, property-name APIs for remaining families, full dirty-list scheduler
parity, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: default-context trigger source mutation by state-machine
data-bind index now updates same-path graph source nodes instead of only the
selected cloned edge. A C++ probe mutates a shared
`ViewModelInstanceTrigger.propertyValue` raw trigger count through the first
data bind and proves a neighboring ordinary direct `ToTarget` trigger bind
reports the updated source and applies the updated target on the next
state-machine advance. The contract is
`docs/prototypes/data-binding-graph-default-trigger-source-mutation-runtime-contract.md`.
Same-path data-bind-index source mutation for remaining
non-number/non-boolean/non-string/non-color/non-enum/non-symbol-list-index/non-asset/non-artboard/non-trigger/non-list
families, imported and owned contexts, trigger callback/listener side effects,
property-name APIs for remaining families, full dirty-list scheduler parity,
pending add/remove behavior, re-entry protection, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: default-context list source mutation by state-machine
data-bind index now updates same-path graph source nodes instead of only the
selected cloned edge. A C++ probe mutates a shared
`ViewModelInstanceList` item count through the first data bind and proves a
neighboring ordinary direct `ToTarget` bindable-list bind reports the updated
source size and applies the updated target after data-context advancement. The
contract is
`docs/prototypes/data-binding-graph-default-viewmodel-list-source-mutation-runtime-contract.md`.
Same-path data-bind-index source mutation for remaining
view-model pointer/relink semantics, imported and owned contexts, list-item
identity/layout side effects, property-name APIs for remaining families, full
dirty-list scheduler parity, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: default-context view-model pointer relink by
state-machine data-bind index now updates same-path graph source nodes instead
of only the selected cloned edge. A C++ probe relinks a shared
`ViewModelInstanceViewModel` cached reference through the first data bind and
proves a neighboring ordinary direct `ToTarget` view-model bind reports the
updated source and target instance indexes after normal state-machine
advancement. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-relink-runtime-contract.md`.
The default data-bind-index same-path source mutation/relink observer family
now covers number, boolean, string, color, enum, symbol-list-index, asset,
artboard, trigger, list, and view-model relink. Imported and owned contexts,
list-item identity/layout side effects, property-name APIs for remaining
families, full dirty-list scheduler parity, pending add/remove behavior,
re-entry protection, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: grouped system operation-value public
`updateDataBinds(true)` target-to-source behavior now preserves the owning
data-bind direction inside `DataConverterGroup` children. Rust threads the
data-bind flags into grouped `DataConverterSystemNormalizer` and
`DataConverterSystemDegsToRads` construction, then compares a
`DataConverterGroup<OperationValue, System*>` public update against C++ for
both concrete system converters. The contract is
`docs/prototypes/data-binding-graph-system-operation-value-group-public-update-target-to-source-runtime-contract.md`.
Other system converter group compositions, symbol-list-index inputs, remaining
mixed/stateful groups, full dirty-list scheduler parity, imported/owned
contexts, pending add/remove behavior, re-entry protection,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: grouped deterministic formula public
`updateDataBinds(true)` target-to-source behavior now has a narrow parity
slice. A main-`ToTarget | TwoWay` number bind uses a `DataConverterGroup`
containing `DataConverterOperationValue` followed by
`DataConverterFormula`; public update reverse-converts the edited target
through C++ reverse group order, runs formula `reverseConvert` through
deterministic formula `convert`, writes the default number source, and
reapplies source-to-target in the same update. This now covers both the
input/value/operation formula group and the deterministic `FormulaTokenFunction`
formula group. The contract is
`docs/prototypes/data-binding-graph-formula-group-public-update-target-to-source-runtime-contract.md`.
Other formula group compositions, random formula behavior beyond the direct
host-supplied slice, remaining mixed/stateful groups, full dirty-list scheduler
parity, imported/owned contexts, pending add/remove behavior, re-entry
protection, relative/parent/nested lookup, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: deterministic formula number contexts
Direct graph-owned `DataConverterFormula` converters now cover number sources
that are rebound through imported and owned runtime view-model contexts before
normal state-machine advancement. The imported-context probe binds a
file-backed `ViewModelInstance`, mutates the bound number source, and proves
the formula target uses that rebound source. The owned-context probe creates a
runtime-owned view-model instance, mutates its number property before binding,
and proves the same formula conversion path matches C++. The contract is
`docs/prototypes/data-binding-graph-formula-number-context-runtime-contract.md`.
Formula random functions, symbol-list-index and non-number formula contexts,
reverse propagation, public update, target-dirty scheduling, imported-context
sharing beyond existing scalar sharing probes, generated list item identity,
relative/parent/nested lookup, listener-owned data binding, nested artboard
propagation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: deterministic formula number context fanout
Direct graph-owned `DataConverterFormula` number contexts now include a
same-path direct number observer. Imported-context number mutation fans out to
every bound same-path number source node in the active graph before dirty
reapplication, and owned-context binding refreshes both the formula source and
observer source. The contract is
`docs/prototypes/data-binding-graph-formula-number-context-fanout-runtime-contract.md`.

Current #12 update: deterministic formula owned number source mutation
Direct graph-owned `DataConverterFormula` number contexts now cover post-bind
owned root-number source mutation by state-machine data-bind index. The
same-path direct number observer and formula-converted number target both
refresh after `StateMachineInstance::set_owned_view_model_context_number_source_for_data_bind`.
The contract is
`docs/prototypes/data-binding-graph-formula-owned-number-mutation-runtime-contract.md`.
Formula random functions, reverse propagation, public update,
target-dirty scheduling, relative/parent/nested lookup, listener-owned data
binding, nested artboard propagation, and full dirty-list scheduler parity
remain follow-up `#12` slices.

Current #12 update: deterministic formula symbol-list-index contexts
Direct graph-owned `DataConverterFormula` converters now cover
symbol-list-index sources that are rebound through imported and owned runtime
view-model contexts before normal state-machine advancement. Rust casts the
rebound symbol-list-index value to `f32` before formula evaluation, matching
the default-context formula path. The contract is
`docs/prototypes/data-binding-graph-formula-symbol-list-index-context-runtime-contract.md`.
Formula random functions, boolean/enum/color/string/trigger/list formula
contexts, reverse propagation, public update, target-dirty scheduling,
imported-context sharing beyond existing scalar sharing probes, generated list
item identity, relative/parent/nested lookup, listener-owned data binding,
nested artboard propagation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: deterministic formula symbol-list-index context fanout
Direct graph-owned `DataConverterFormula` symbol-list-index contexts now
include a same-path direct integer observer. Imported-context
symbol-list-index mutation fans out to every bound same-path
symbol-list-index source node in the active graph before dirty reapplication,
and owned-context binding refreshes both the formula source and observer
source. The contract is
`docs/prototypes/data-binding-graph-formula-symbol-list-index-context-fanout-runtime-contract.md`.

Current #12 update: deterministic formula owned symbol-list-index source mutation
Direct graph-owned `DataConverterFormula` symbol-list-index contexts now cover
post-bind owned root symbol-list-index source mutation by state-machine
data-bind index. The same-path direct symbol-list-index observer and
formula-converted number target both refresh after
`StateMachineInstance::set_owned_view_model_context_symbol_list_index_source_for_data_bind`.
The contract is
`docs/prototypes/data-binding-graph-formula-owned-symbol-mutation-runtime-contract.md`.
Formula random functions, reverse propagation, public update,
target-dirty scheduling, relative/parent/nested lookup, listener-owned data
binding, nested artboard propagation, and full dirty-list scheduler parity
remain follow-up `#12` slices.

Current #12 update: deterministic formula boolean contexts
Direct graph-owned `DataConverterFormula` fallback now covers boolean sources
that are rebound through imported and owned runtime view-model contexts before
normal state-machine advancement. The fixture also includes a same-path direct
boolean observer, so imported-context boolean mutation now fans out to every
bound same-path boolean source node in the active graph before dirty
reapplication. The contract is
`docs/prototypes/data-binding-graph-formula-boolean-context-runtime-contract.md`.

Current #12 update: deterministic formula owned boolean source mutation
Direct graph-owned `DataConverterFormula` boolean fallback now covers
post-bind owned root boolean source mutation by state-machine data-bind index.
The same-path direct boolean observer and formula fallback number target both
refresh after
`StateMachineInstance::set_owned_view_model_context_boolean_source_for_data_bind`.
The C++ probe now retains active owned boolean contexts for this comparison.
The contract is
`docs/prototypes/data-binding-graph-formula-owned-boolean-mutation-runtime-contract.md`.
Enum/color/string/trigger/list formula contexts, imported same-path fanout for
non-boolean source kinds, formula random functions, reverse propagation,
public update, target-dirty scheduling, generated list item identity,
relative/parent/nested lookup, listener-owned data binding, nested artboard
propagation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: deterministic formula enum contexts
Direct graph-owned `DataConverterFormula` fallback now covers enum sources
that are rebound through imported and owned runtime view-model contexts before
normal state-machine advancement. The fixture also includes a same-path direct
enum observer, so imported-context enum mutation now fans out to every bound
same-path enum source node in the active graph before dirty reapplication. The
contract is
`docs/prototypes/data-binding-graph-formula-enum-context-runtime-contract.md`.

Current #12 update: deterministic formula owned enum source mutation
Direct graph-owned `DataConverterFormula` enum fallback now covers post-bind
owned root enum source mutation by state-machine data-bind index. The
same-path direct enum observer and formula fallback number target both refresh
after `StateMachineInstance::set_owned_view_model_context_enum_source_for_data_bind`.
The C++ probe now retains active owned enum contexts for this comparison. The
contract is
`docs/prototypes/data-binding-graph-formula-owned-enum-mutation-runtime-contract.md`.
Color/string/trigger/list formula contexts, imported same-path fanout for
non-boolean/non-enum source kinds, formula random functions, reverse
propagation, public update, target-dirty scheduling, generated list item
identity, relative/parent/nested lookup, listener-owned data binding, nested
artboard propagation, and full dirty-list scheduler parity remain follow-up
`#12` slices.

Current #12 update: deterministic formula color contexts
Direct graph-owned `DataConverterFormula` fallback now covers color sources
that are rebound through imported and owned runtime view-model contexts before
normal state-machine advancement. The fixture also includes a same-path direct
color observer, so imported-context color mutation now fans out to every bound
same-path color source node in the active graph before dirty reapplication. The
contract is
`docs/prototypes/data-binding-graph-formula-color-context-runtime-contract.md`.
String/trigger/list formula contexts, imported same-path fanout for
non-boolean/non-enum/non-color source kinds, formula random functions, reverse
propagation, public update, target-dirty scheduling, generated list item
identity, relative/parent/nested lookup, listener-owned data binding, nested
artboard propagation, and full dirty-list scheduler parity remain follow-up
`#12` slices.

Current #12 update: deterministic formula string contexts
Direct graph-owned `DataConverterFormula` fallback now covers string sources
that are rebound through imported and owned runtime view-model contexts before
normal state-machine advancement. The fixture also includes a same-path direct
string observer, so imported-context string mutation now fans out to every
bound same-path string source node in the active graph before dirty
reapplication. The contract is
`docs/prototypes/data-binding-graph-formula-string-context-runtime-contract.md`.
Trigger/list formula contexts, imported same-path fanout for
non-boolean/non-enum/non-color/non-string source kinds, formula random
functions, reverse propagation, public update, target-dirty scheduling,
generated list item identity, relative/parent/nested lookup, listener-owned
data binding, nested artboard propagation, and full dirty-list scheduler parity
remain follow-up `#12` slices.

Current #12 update: deterministic formula trigger contexts
Direct graph-owned `DataConverterFormula` fallback now covers trigger sources
that are rebound through imported and owned runtime view-model contexts before
normal state-machine advancement. The fixture also includes a same-path direct
trigger observer, so imported-context trigger mutation now fans out to every
bound same-path trigger source node in the active graph before dirty
reapplication. The contract is
`docs/prototypes/data-binding-graph-formula-trigger-context-runtime-contract.md`.
List formula contexts, imported same-path fanout for
non-boolean/non-enum/non-color/non-string/non-trigger source kinds, formula
random functions, reverse propagation, public update, target-dirty scheduling,
generated list item identity, relative/parent/nested lookup, listener-owned
data binding, nested artboard propagation, and full dirty-list scheduler parity
remain follow-up `#12` slices.

Current #12 update: deterministic formula list contexts
Direct graph-owned `DataConverterFormula` fallback now covers list sources
that are rebound through imported and owned runtime view-model contexts before
normal state-machine advancement. The fixture also includes a same-path direct
list observer, so imported-context list item-count mutation now fans out to
every bound same-path list source node in the active graph before dirty
reapplication. The contract is
`docs/prototypes/data-binding-graph-formula-list-context-runtime-contract.md`.
Generated list item identity beyond item-count parity, formula random
functions, reverse propagation, public update, target-dirty scheduling,
relative/parent/nested lookup, listener-owned data binding, nested artboard
propagation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: first graph formula random function
Direct graph-owned `DataConverterFormula` converters now admit the first
`FunctionType::random` slice for default-context number sources feeding number
targets. This is intentionally limited to `randomModeValue == 0` and a
host-supplied graph formula random stream; Rust caches the supplied draw per
formula converter like C++ default random mode, and the C++ probe derives the
first draw from the C++ state-machine number-binding report before supplying it
to Rust. The contract is
`docs/prototypes/data-binding-graph-formula-random-function-runtime-contract.md`.
Real Rust random generation, C++ probe random seeding/queueing,
`RandomMode::sourceChange`, broader `RandomMode::always` scheduling, cache
invalidation, random call counts, list random formulas, imported/owned
contexts, and full dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index source-to-target
Direct graph-owned `DataConverterFormula` random functions now cover the first
non-number source path for default-context symbol-list-index sources feeding
number targets. Rust casts the symbol-list-index source to `f32`, consumes the
host-supplied default-mode random value, and reuses the cached value on later
state-machine advancement like C++. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-runtime-contract.md`.
Real Rust random generation, C++ probe random seeding/queueing,
`RandomMode::always`, `RandomMode::sourceChange`, list random formulas, other
non-number random formulas, imported/owned contexts, and full dirty-list
scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index always mode source-to-target
Direct graph-owned `DataConverterFormula` random functions now cover
`RandomMode::always` for default-context symbol-list-index sources feeding
number targets. Rust casts the symbol-list-index source to `f32` and consumes
a fresh host-supplied random value on each source-to-target state-machine
evaluation like C++. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-always-runtime-contract.md`.
Real Rust random generation, C++ probe random seeding/queueing,
list random formulas, other non-number random formulas, imported/owned
contexts, and full dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index source-change mode source-to-target
Direct graph-owned `DataConverterFormula` random functions now cover
`RandomMode::sourceChange` for default-context symbol-list-index sources
feeding number targets. Rust casts the symbol-list-index source to `f32`,
caches the host-supplied random like default mode, and clears that formula
random cache when
`set_default_view_model_symbol_list_index_source_for_data_bind` mutates the
bound default source before the next source-to-target advance. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-source-change-runtime-contract.md`.
Real Rust random generation, C++ probe random seeding/queueing, list random
formulas, other non-number random formulas, imported/owned contexts, and full
dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index source-to-target call counts
Direct graph-owned `DataConverterFormula` random functions now expose Rust's
host-supplied random-stream call count for default-context symbol-list-index
sources feeding number targets. Default, always, and source-change random
modes cover source-to-target cache reuse, a fresh always-mode pull on a
source-mutation-scheduled second formula evaluation, and source-change cache
clearing after
`set_default_view_model_symbol_list_index_source_for_data_bind`. The contract
is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-call-count-runtime-contract.md`.
Direct explicit target-to-source, public-update, target-dirty, grouped
symbol-list-index, list/non-number, imported/owned, real random generation,
secondary dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices. The C++ probe now seeds deterministic random values
and reports per-action `randomTotalCalls`, so this slice compares the Rust
host-stream count directly against the C++ total after every source-to-target
report.

Current #12 update: graph formula random always mode source-to-target
Direct graph-owned `DataConverterFormula` random functions now cover the first
`RandomMode::always` slice for default-context number sources feeding number
targets. Rust accepts `randomModeValue == 1` for direct formula random tokens,
draws from the host-supplied graph formula random stream on each formula
evaluation, and avoids the default-mode cached random reuse for this mode. The
C++ probe derives two random draws from two state-machine number-binding
reports and supplies both to Rust before advancing. The contract is
`docs/prototypes/data-binding-graph-formula-random-always-runtime-contract.md`.
Real Rust random generation, C++ probe random seeding/queueing,
public update, target-dirty, grouped target-to-source, grouped public-update,
grouped target-dirty, list, and remaining non-number `RandomMode::always`
scheduling, broader cache invalidation, random call counts, imported/owned
contexts, and full dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random always mode explicit target-to-source
Direct graph-owned `DataConverterFormula` random functions now cover explicit
`advance_data_context` target-to-source scheduling for main-`ToSource |
TwoWay` default-context number binds when `randomModeValue == 1`. Rust
consumes one host-supplied random value for the target-to-source source write,
another for same-bind source-to-target reapplication, and fresh values on later
state-machine advances, matching the C++ probe reports. The contract is
`docs/prototypes/data-binding-graph-formula-random-always-target-to-source-runtime-contract.md`.
Target-dirty, grouped target-to-source, grouped public-update, grouped
target-dirty, list, and remaining non-number `RandomMode::always` scheduling,
broader cache invalidation, random call counts, imported/owned contexts, and
full dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random always mode target-dirty
Direct graph-owned `DataConverterFormula` random functions now cover
main-`ToTarget | TwoWay` target-dirty scheduling for default-context number
binds when `randomModeValue == 1`. Rust consumes a host-supplied random value
for the initial source-to-target pass, preserves a manual target edit through
explicit data-context advancement, and consumes fresh values on later normal
state-machine advances, matching C++. The contract is
`docs/prototypes/data-binding-graph-formula-random-always-target-dirty-runtime-contract.md`.
Grouped target-to-source, grouped public-update, grouped target-dirty, list,
and remaining non-number `RandomMode::always` scheduling, broader cache
invalidation, random call counts, imported/owned contexts, and full dirty-list
scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random always mode public update target-to-source
Direct graph-owned `DataConverterFormula` random functions now cover public
`update_data_binds_apply_target_to_source` scheduling for main-`ToTarget |
TwoWay` default-context number binds when `randomModeValue == 1`. Rust
consumes fresh host-supplied random values for the initial source-to-target
pass, the public target-to-source source write, same-update source-to-target
reapplication, and later state-machine advances, matching the C++ probe
reports. The contract is
`docs/prototypes/data-binding-graph-formula-random-always-public-update-target-to-source-runtime-contract.md`.
Grouped target-to-source, grouped public-update, grouped target-dirty, list,
and remaining non-number `RandomMode::always` scheduling, broader cache
invalidation, random call counts, imported/owned contexts, and full dirty-list
scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random source-change mode source-to-target
Direct graph-owned `DataConverterFormula` random functions now cover the first
`RandomMode::sourceChange` slice for default-context number sources feeding
number targets. Rust accepts `randomModeValue == 2` for direct formula random
tokens, caches the host-supplied random like the default mode, and clears that
formula random cache when `set_default_view_model_number_source_for_data_bind`
mutates the bound default source before the next source-to-target advance. The
C++ probe derives the pre-change and post-change random draws from
state-machine number-binding reports and supplies both to Rust before
advancing. The contract is
`docs/prototypes/data-binding-graph-formula-random-source-change-runtime-contract.md`.
Real Rust random generation, C++ probe random seeding/queueing,
grouped/list/non-number `RandomMode::sourceChange` scheduling, secondary
converter dependency invalidation, broader random call-count coverage,
imported/owned contexts, and full dirty-list scheduler parity remain follow-up
`#12` slices.

Current #12 update: graph formula random call-count introspection
Direct graph-owned `DataConverterFormula` random functions now expose Rust's
host-supplied random-stream call count through
`StateMachineInstance::data_bind_formula_random_call_count`. The C++ probe now
has an opt-in counted deterministic `RandomProvider` shim, exposed as
`runtimeStateMachineAdvances[].randomTotalCalls`, so the default-context number
source-to-target probes compare Rust counts directly against C++ call totals
for default, always, and source-change random modes. The existing list formula
fallback probe also proves that random tokens on list sources do not consume
random values because C++ returns the numeric fallback before evaluating
formula tokens. The contract is
`docs/prototypes/data-binding-graph-formula-random-call-count-runtime-contract.md`.
Real RNG generation/seeding, grouped converters, target-dirty scheduling,
imported/owned contexts, secondary dependency invalidation, and full dirty-list
scheduler parity remain follow-up `#12` slices. Direct explicit
target-to-source and public-update call-count coverage is tracked separately
below and can now use the probe-visible total-call field.

Current #12 update: graph formula random source-change mode explicit target-to-source
Direct graph-owned `DataConverterFormula` random functions now cover explicit
`advance_data_context` target-to-source scheduling for main-`ToSource |
TwoWay` default-context number binds when `randomModeValue == 2`. Rust
consumes a host-supplied random value for the target-to-source source write,
clears the formula random cache when that write changes the graph source, and
consumes a fresh value for same-pass source-to-target reapplication, matching
C++. The contract is
`docs/prototypes/data-binding-graph-formula-random-source-change-target-to-source-runtime-contract.md`.
Public update, target-dirty, grouped/list/non-number
`RandomMode::sourceChange` scheduling, secondary converter dependency
invalidation, broader random call-count coverage, imported/owned contexts, and
full dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random explicit target-to-source call counts
Direct graph-owned `DataConverterFormula` random functions now pin Rust's
host-supplied random-stream call counts for explicit `advance_data_context`
target-to-source scheduling on default-context number binds. Default random
mode consumes once and reuses the cached value across same-pass reapplication
and later advances; always mode consumes two values during the explicit
target-to-source pass and does not pull again on later normal advances;
source-change mode consumes for the source write, clears the cache for the
changed source, consumes again for same-pass reapplication, and likewise does
not pull again on later normal advances without another scheduled formula
evaluation. The C++ probe now seeds deterministic random values and reports
per-action `randomTotalCalls`, so this slice compares the Rust host-stream
count directly against the C++ total after every explicit target-to-source
report. The contract is
`docs/prototypes/data-binding-graph-formula-random-target-to-source-call-count-runtime-contract.md`.
Grouped/list/non-number paths, imported/owned contexts, secondary dependency
invalidation, real RNG generation/seeding, and full dirty-list scheduler parity
remain follow-up `#12` slices. Public-update and direct target-dirty call
counts are covered separately below.

Current #12 update: graph formula random public-update call counts
Direct graph-owned `DataConverterFormula` random functions now pin Rust's
host-supplied random-stream call counts for public
`update_data_binds_apply_target_to_source` scheduling on default-context
number binds. Default random mode consumes once during initial
source-to-target evaluation and reuses that cached value through the public
update and later advances; always mode consumes once initially, two more
values during the public update, and no additional values on later normal
advances; source-change mode consumes once initially, reuses that warmed value
for the public target-to-source write, consumes one refreshed value for
same-update reapplication, and does not pull again on later normal advances
without another scheduled formula evaluation. The C++ probe now seeds
deterministic random values and reports per-action `randomTotalCalls`, so this
slice compares the Rust host-stream count directly against the C++ total after
every public-update report. The contract is
`docs/prototypes/data-binding-graph-formula-random-public-update-call-count-runtime-contract.md`.
Grouped/list/non-number paths, imported/owned contexts, secondary dependency
invalidation, real RNG generation/seeding, and full dirty-list scheduler parity
remain follow-up `#12` slices. Direct target-dirty call counts are covered
separately below.

Current #12 update: graph formula random target-dirty call counts
Direct graph-owned `DataConverterFormula` random functions now pin Rust's
host-supplied random-stream call counts for main-`ToTarget | TwoWay`
target-dirty scheduling on default-context number binds. Default random mode
consumes once during initial source-to-target evaluation and reuses that
cached value through target-dirty preservation and later advances; always mode
consumes once initially, one more value on the first later normal reapply, and
no additional value on the second later normal advance in this direct fixture;
source-change mode consumes once initially, preserves the cache because the
target edit does not change the source, and reuses that cached value on later
normal advances. The contract is
`docs/prototypes/data-binding-graph-formula-random-target-dirty-call-count-runtime-contract.md`.
The C++ probe now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares the Rust host-stream count directly
against the C++ total after every target-dirty report. Grouped/list/non-number
paths, imported/owned contexts, secondary dependency invalidation, real RNG
generation/seeding, and full dirty-list scheduler parity remain follow-up
`#12` slices.

Current #12 update: graph formula random source-change mode public update target-to-source
Direct graph-owned `DataConverterFormula` random functions now cover public
`update_data_binds_apply_target_to_source` scheduling for main-`ToTarget |
TwoWay` default-context number binds when `randomModeValue == 2`. Rust warms
the source-change random cache during initial source-to-target application,
reuses that cached value for the public target-to-source source write, clears
the formula random cache when that write changes the graph source, and
consumes a fresh value for same-update source-to-target reapplication,
matching C++. The contract is
`docs/prototypes/data-binding-graph-formula-random-source-change-public-update-target-to-source-runtime-contract.md`.
Target-dirty, grouped/list/non-number `RandomMode::sourceChange` scheduling,
secondary converter dependency invalidation, broader random call counts,
imported/owned contexts, and full dirty-list scheduler parity remain follow-up
`#12` slices.

Current #12 update: graph formula random source-change mode target-dirty
Direct graph-owned `DataConverterFormula` random functions now cover
main-`ToTarget | TwoWay` target-dirty scheduling for default-context number
binds when `randomModeValue == 2`. Rust consumes a host-supplied random value
for the initial source-to-target pass, preserves a manual target edit through
explicit data-context advancement without treating the target edit as a source
change, and reuses the cached value on later normal state-machine advances,
matching C++. The contract is
`docs/prototypes/data-binding-graph-formula-random-source-change-target-dirty-runtime-contract.md`.
Grouped/list/non-number `RandomMode::sourceChange` scheduling, secondary
converter dependency invalidation, broader random call counts, imported/owned
contexts, and full dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random target-to-source
Direct graph-owned `DataConverterFormula` random functions now reuse the
host-supplied default-mode formula random cache through direct number
target-to-source scheduling. Rust threads the graph formula random stream into
the number target-to-source path, so both explicit `advancedDataContext()` for
a main-`ToSource | TwoWay` random formula bind and public
`updateDataBinds(true)` for a main-`ToTarget | TwoWay` random formula bind
match C++: the explicit path after a target mutation, and the public path after
the initial source-to-target draw. The contract is
`docs/prototypes/data-binding-graph-formula-random-target-to-source-runtime-contract.md`.
List random formulas, non-number random formulas, non-default random modes,
cache invalidation, imported/owned contexts, real random generation, and full
dirty-list scheduler parity remain follow-up `#12` slices. Grouped number
call-count paths are covered separately below.

Current #12 update: graph formula random target-dirty
Direct graph-owned `DataConverterFormula` random functions now cover
main-`ToTarget | TwoWay` state-machine target-dirty behavior for default-context
number binds. A manual edit to a random-formula-bound number target is
preserved through explicit data-context advancement, then the next normal
state-machine advance reapplies the unchanged source through the cached
host-supplied formula random value. The contract is
`docs/prototypes/data-binding-graph-formula-random-target-dirty-runtime-contract.md`.
List random formulas, cache invalidation, imported/owned contexts, real random
generation, and full dirty-list scheduler parity remain follow-up `#12` slices.
Grouped non-number target-to-source, public-update, and target-dirty scheduling
are covered separately below. Grouped number and non-number source-to-target
call-count paths are covered separately below.

Current #12 update: graph formula random group source-to-target
Grouped graph-owned `DataConverterFormula` random functions now cover the
first source-to-target group path for default-context number binds. A
`DataConverterGroup<OperationValue, Formula(random)>` bind threads the
host-supplied graph formula random stream through nested converter state,
draws the same default-mode random value as C++, and reuses the cached grouped
formula value on later state-machine advancement. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-runtime-contract.md`.
List random formulas, non-number random formulas, non-default random modes,
cache invalidation, call counts, imported/owned contexts, real random
generation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random group always mode source-to-target
Grouped graph-owned `DataConverterFormula` random functions now cover the
first grouped `RandomMode::always` source-to-target path for default-context
number binds. A `DataConverterGroup<OperationValue, Formula(random)>` bind
threads the host-supplied graph formula random stream through nested converter
state and, when the formula child has `randomModeValue == 1`, consumes a fresh
draw on each state-machine source-to-target evaluation. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-always-runtime-contract.md`.
List random formulas, cache invalidation, imported/owned contexts, real random
generation, and full dirty-list scheduler parity remain follow-up `#12` slices.
Grouped non-number target-to-source, public-update, and target-dirty scheduling
are covered separately below. Grouped number and non-number source-to-target
call-count paths are covered separately below.

Current #12 update: graph formula random group always mode explicit target-to-source
Grouped graph-owned `DataConverterFormula` random functions now cover explicit
`advance_data_context` target-to-source scheduling for
`DataConverterGroup<OperationValue, Formula(random)>` default-context number
binds when `randomModeValue == 1`. Rust consumes fresh host-supplied random
values for the target-to-source source write and same-pass source-to-target
reapplication. Later normal advances in this fixture do not reschedule the
grouped formula without another source or target mutation. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-always-target-to-source-runtime-contract.md`.
Grouped public-update, grouped target-dirty, source-change grouped
public-update/target-dirty, list, remaining non-number random scheduling,
cache invalidation, call counts, imported/owned contexts, real random
generation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random group always mode public update target-to-source
Grouped graph-owned `DataConverterFormula` random functions now cover public
`update_data_binds_apply_target_to_source` scheduling for
`DataConverterGroup<OperationValue, Formula(random)>` default-context number
binds when `randomModeValue == 1`. Rust consumes fresh host-supplied random
values for initial source-to-target application, the public target-to-source
source write, and same-update source-to-target reapplication. Later normal
advances in this fixture do not reschedule the grouped formula without another
source or target mutation. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-always-public-update-target-to-source-runtime-contract.md`.
List, remaining non-number random scheduling, cache invalidation, call counts,
imported/owned contexts, real random generation, and full dirty-list scheduler
parity remain follow-up `#12` slices. Grouped target-dirty and source-change
grouped target-dirty are covered separately below.

Current #12 update: graph formula random group always mode target-dirty
Grouped graph-owned `DataConverterFormula` random functions now cover
main-`ToTarget | TwoWay` target-dirty scheduling for
`DataConverterGroup<OperationValue, Formula(random)>` default-context number
binds when `randomModeValue == 1`. Rust consumes a host-supplied random value
for the initial source-to-target pass, preserves a manual target edit through
explicit data-context advancement, and consumes one fresh value on the first
later normal state-machine reapply. A second later normal advance in this
fixture does not reschedule the grouped formula. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-always-target-dirty-runtime-contract.md`.
Source-change grouped target-dirty, list, remaining non-number random
scheduling, cache invalidation, target-dirty call counts, imported/owned
contexts, real random generation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula random group source-change mode source-to-target
Grouped graph-owned `DataConverterFormula` random functions now cover the
first grouped `RandomMode::sourceChange` source-to-target path for
default-context number binds. A `DataConverterGroup<OperationValue,
Formula(random)>` bind caches its host-supplied random value like default
mode, then clears the nested formula cache when
`set_default_view_model_number_source_for_data_bind` mutates the bound default
source before the next source-to-target advance. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-source-change-runtime-contract.md`.
List random formulas, secondary converter dependency invalidation, cache
invalidation, grouped public-update/target-dirty call counts, imported/owned
contexts, real random generation, and full dirty-list scheduler parity remain
follow-up `#12` slices. Grouped non-number source-to-target,
target-to-source, public-update, and target-dirty scheduling is covered
separately below.

Current #12 update: graph formula random group source-to-target call counts
Grouped graph-owned `DataConverterFormula` random functions now pin Rust's
host-supplied random-stream call counts for source-to-target scheduling on
default-context number binds. For
`DataConverterGroup<OperationValue, Formula(random)>`, default random mode
consumes one value and reuses the nested formula cache, always mode consumes a
fresh value when a source mutation schedules a later source-to-target
evaluation, and source-change mode consumes once initially, consumes again
after the bound source changes, and then reuses that refreshed cache. The
contract is
`docs/prototypes/data-binding-graph-formula-random-group-call-count-runtime-contract.md`.
The C++ probe now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares the Rust host-stream count directly
against the C++ total after every grouped source-to-target report. Grouped
public-update/target-dirty call counts, list paths, imported/owned contexts,
secondary dependency invalidation, real RNG generation/seeding, and full
dirty-list scheduler parity remain follow-up `#12` slices. Grouped non-number
source-to-target, explicit target-to-source, public-update, and target-dirty
call counts are covered separately below.

Current #12 update: graph formula random group non-number source-to-target
Grouped graph-owned `DataConverterFormula` random functions now cover
source-to-target scheduling for default-context boolean, enum, color, string,
and trigger sources feeding number targets through
`DataConverterGroup<OperationValue, Formula(random)>`. Rust admits those
non-number sources into the grouped number-target path, lets the leading
`OperationValue` converter fall back to a number, then evaluates the grouped
random formula for `randomModeValue` values `0`, `1`, and `2`. Default mode
consumes one random value and reuses it; always mode consumes a fresh value
after source mutation; source-change mode clears the nested formula random
cache after source mutation and consumes one fresh value on the next
source-to-target pass. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-non-number-fallback-runtime-contract.md`.
The C++ probe now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares Rust call counts directly against
C++ for all five represented non-number source kinds. Grouped non-number
explicit target-to-source, public-update, and target-dirty behavior are covered
separately below. List paths, imported/owned contexts, secondary dependency
invalidation, real RNG generation/seeding, and full dirty-list scheduler parity
remain follow-up `#12` slices.

Current #12 update: graph formula random group source-change mode explicit target-to-source
Grouped graph-owned `DataConverterFormula` random functions now cover explicit
`advance_data_context` target-to-source scheduling for
`DataConverterGroup<OperationValue, Formula(random)>` default-context number
binds when `randomModeValue == 2`. Rust consumes a host-supplied random value
for the target-to-source source write, clears the nested formula random cache
when that write changes the graph source, and consumes a fresh value for
same-pass source-to-target reapplication, matching C++. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-source-change-target-to-source-runtime-contract.md`.
Grouped source-change public-update/target-dirty, list, secondary converter
dependency invalidation, cache invalidation, imported/owned contexts, real
random generation, and full dirty-list scheduler parity remain follow-up `#12`
slices. Grouped non-number explicit target-to-source, public-update, and
target-dirty call counts are covered separately below.

Current #12 update: graph formula random group explicit target-to-source call counts
Grouped graph-owned `DataConverterFormula` random functions now pin Rust's
host-supplied random-stream call counts for explicit
`advance_data_context` target-to-source scheduling on default-context number
binds. For `DataConverterGroup<OperationValue, Formula(random)>`, default
random mode consumes one value and reuses it through same-pass reapplication
and later advances; always mode consumes two values during the explicit pass
and does not pull again on later normal advances in this fixture; source-change
mode consumes for the source write, clears the nested cache for the changed
source, consumes again for same-pass reapplication, and likewise does not pull
again on later normal advances. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-target-to-source-call-count-runtime-contract.md`.
The C++ probe now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares the Rust host-stream count directly
against the C++ total after every grouped explicit target-to-source report.
Grouped target-dirty call counts, list paths, imported/owned contexts,
secondary dependency invalidation, real RNG generation/seeding, and full
dirty-list scheduler parity remain follow-up `#12` slices. Grouped non-number
explicit target-to-source, public-update, and target-dirty call counts are
covered separately below.

Current #12 update: graph formula random group non-number explicit target-to-source
Grouped graph-owned `DataConverterFormula` random functions now cover explicit
`advanceDataContext()` target-to-source scheduling for default-context
boolean, enum, color, string, and trigger sources feeding number targets
through `DataConverterGroup<OperationValue, Formula(random)>`. Rust preserves
the non-number source when grouped number conversion produces a number, then
performs the same immediate source-to-target reapply that C++ reports for
`randomModeValue` values `0`, `1`, and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-non-number-target-to-source-runtime-contract.md`.
The C++ probe seeds deterministic random values and reports per-action
`randomTotalCalls`; default, always, and source-change modes each consume one
visible random value for this explicit non-number target-to-source schedule
and reuse it on later advances. Grouped non-number public-update and
target-dirty behavior are covered separately below. List paths,
imported/owned contexts, secondary dependency invalidation, real RNG
generation/seeding, and full dirty-list scheduler parity remain follow-up
`#12` slices.

Current #12 update: graph formula random group non-number public update
Grouped graph-owned `DataConverterFormula` random functions now cover public
`updateDataBinds(true)` target-to-source scheduling for default-context
boolean, enum, color, string, and trigger sources feeding number targets
through `DataConverterGroup<OperationValue, Formula(random)>`. After an
initial source-to-target advance warms the grouped formula random path, Rust
preserves the non-number source during the public update and reapplies
source-to-target in the same update, matching C++ for `randomModeValue` values
`0`, `1`, and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-non-number-public-update-runtime-contract.md`.
The C++ probe seeds deterministic random values and reports per-action
`randomTotalCalls`; default and source-change modes keep the initial
`[1, 1, 1, 1]` count sequence, while always mode consumes two additional
public-update values for `[1, 3, 3, 3]`. Grouped non-number target-dirty
behavior is covered separately below. List paths, imported/owned contexts,
secondary dependency invalidation, real RNG generation/seeding, and full
dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random group non-number target-dirty
Grouped graph-owned `DataConverterFormula` random functions now cover
main-`ToTarget | TwoWay` target-dirty scheduling for default-context boolean,
enum, color, string, and trigger sources feeding number targets through
`DataConverterGroup<OperationValue, Formula(random)>`. After an initial
source-to-target advance warms the grouped formula random path, Rust preserves a
manual number target edit during explicit data-context advancement and then
matches C++'s later source-to-target reapply for `randomModeValue` values `0`,
`1`, and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-non-number-target-dirty-runtime-contract.md`.
The C++ probe seeds deterministic random values and reports per-action
`randomTotalCalls`; default mode keeps `[1, 1, 1, 1]`, always mode uses
`[1, 1, 2, 2]`, source-change mode keeps `[1, 1, 1, 1]` for boolean, enum,
color, and string sources, and trigger source-change uses `[1, 1, 2, 2]`
because the bound trigger reset clears the nested grouped formula random cache.
List paths, imported/owned contexts, secondary dependency invalidation, real RNG
generation/seeding, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random group source-change mode public update target-to-source
Grouped graph-owned `DataConverterFormula` random functions now cover public
`update_data_binds_apply_target_to_source` scheduling for
`DataConverterGroup<OperationValue, Formula(random)>` default-context number
binds when `randomModeValue == 2`. Rust warms the nested source-change random
cache during initial source-to-target application, reuses that cached value
for the public target-to-source source write, clears the nested formula random
cache when that write changes the graph source, and consumes a fresh value for
same-update source-to-target reapplication, matching C++. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-source-change-public-update-target-to-source-runtime-contract.md`.
Grouped source-change target-dirty, list, secondary converter dependency
invalidation, cache invalidation, target-dirty call counts, imported/owned
contexts, real random generation, and full dirty-list scheduler parity remain
follow-up `#12` slices. Grouped non-number target-dirty behavior is covered
above.

Current #12 update: graph formula random group public-update call counts
Grouped graph-owned `DataConverterFormula` random functions now pin Rust's
host-supplied random-stream call counts for public
`update_data_binds_apply_target_to_source` scheduling on default-context
number binds. For `DataConverterGroup<OperationValue, Formula(random)>`,
default random mode consumes one value during initial source-to-target
application and reuses it through the public update and later advances; always
mode consumes once initially, two more values during the public update, and no
additional values on later normal advances in this fixture; source-change mode
consumes once initially, reuses that warmed value for the public
target-to-source source write, consumes one refreshed value for same-update
reapplication, and likewise does not pull again on later normal advances. The
contract is
`docs/prototypes/data-binding-graph-formula-random-group-public-update-call-count-runtime-contract.md`.
The C++ probe now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares the Rust host-stream count directly
against the C++ total after every grouped public-update report. Grouped
target-dirty call counts, list paths, imported/owned contexts, secondary
dependency invalidation, real RNG generation/seeding, and full dirty-list
scheduler parity remain follow-up `#12` slices. Grouped target-dirty call
counts and grouped non-number target-dirty call counts are covered separately
below.

Current #12 update: graph formula random group source-change mode target-dirty
Grouped graph-owned `DataConverterFormula` random functions now cover
main-`ToTarget | TwoWay` target-dirty scheduling for
`DataConverterGroup<OperationValue, Formula(random)>` default-context number
binds when `randomModeValue == 2`. Rust consumes a host-supplied random value
for the initial source-to-target pass, preserves a manual target edit through
explicit data-context advancement without treating the target edit as a source
change, and reuses the cached value on later normal state-machine advances,
matching C++. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-source-change-target-dirty-runtime-contract.md`.
List, remaining non-number random scheduling, secondary converter dependency
invalidation, cache invalidation, call counts, imported/owned contexts, real
random generation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random group target-dirty call counts
Grouped graph-owned `DataConverterFormula` random functions now pin Rust's
host-supplied random-stream call counts for main-`ToTarget | TwoWay`
target-dirty scheduling on default-context number binds. For
`DataConverterGroup<OperationValue, Formula(random)>`, default random mode
consumes one value during initial source-to-target application and reuses it
through target-dirty preservation and later advances; always mode consumes
once initially, one more value on the first later normal reapply, and no
additional value on the second later normal advance in this fixture;
source-change mode consumes once initially, preserves the cache because the
target edit does not change the source, and reuses that cached value on later
normal advances. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-target-dirty-call-count-runtime-contract.md`.
The C++ probe now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares the Rust host-stream count directly
against the C++ total after every grouped target-dirty report. List/non-number
paths, imported/owned contexts, secondary dependency invalidation, real RNG
generation/seeding, and full dirty-list scheduler parity remain follow-up
`#12` slices.

Current #12 update: graph formula random non-number fallback
Direct graph-owned `DataConverterFormula` random functions now cover the
source-to-target fallback path for default-context boolean, enum, color,
string, and trigger sources feeding number targets. C++ returns `0.0` for
those non-number, non-symbol-list-index sources before evaluating formula
tokens, so `FunctionType::random` is ignored for `randomModeValue` values `0`,
`1`, and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-random-non-number-fallback-runtime-contract.md`.
List formulas, imported/owned contexts, real random generation, secondary
dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices. Boolean target-to-source and target-dirty behavior
are covered separately below. Enum/color/string/trigger target-to-source and
target-dirty behavior are covered separately below. Source-to-target and
target-to-source call counts are covered separately below.

Current #12 update: graph formula random non-number source-to-target call counts
Direct graph-owned `DataConverterFormula` random functions now cover Rust's
host-supplied random-stream call count for source-to-target fallback on
default-context boolean, enum, color, string, and trigger sources feeding
number targets. Rust keeps the count at zero across repeated advances for
`randomModeValue` values `0`, `1`, and `2`, matching the C++ probe observable
fallback values. The contract is
`docs/prototypes/data-binding-graph-formula-random-non-number-fallback-call-count-runtime-contract.md`.
Target-to-source non-number behavior is covered separately below.
The C++ probe now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares zero Rust host-stream pulls directly
against the zero C++ total after every source-to-target fallback report.
Imported/owned contexts, real random generation, secondary dependency
invalidation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random boolean fallback target-to-source
Direct graph-owned `DataConverterFormula` random functions now cover explicit
`advanceDataContext()` and public `updateDataBinds(true)` target-to-source
behavior for default-context boolean sources feeding number targets. Rust
preserves the unchanged boolean source when random-function formula conversion
produces a number, then reapplies that unchanged source through the formula
fallback so the number target returns to C++'s `0.0` fallback for
`randomModeValue` values `0`, `1`, and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-random-boolean-fallback-target-to-source-runtime-contract.md`.
Enum, color, string, trigger, list, and symbol-list-index target-to-source
behavior plus enum/color/string/trigger target-dirty behavior are covered
separately below. Imported/owned contexts, real random generation, secondary
dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices. Boolean target-to-source call counts are covered
separately below.

Current #12 update: graph formula random boolean target-to-source call counts
Direct graph-owned `DataConverterFormula` random functions now cover Rust's
host-supplied random-stream call count for boolean target-to-source fallback.
Explicit `advanceDataContext()` and public `updateDataBinds(true)` each
consume one hidden pull during reverse reapplication, then reuse that count
through later normal advances for `randomModeValue` values `0`, `1`, and `2`,
matching the C++ probe observable fallback values. The contract is
`docs/prototypes/data-binding-graph-formula-random-boolean-fallback-target-to-source-call-count-runtime-contract.md`.
The C++ probe now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares Rust's hidden reverse-path pull
count directly against the C++ total after every boolean target-to-source
report. Imported/owned contexts, real random generation, secondary dependency
invalidation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random boolean fallback target-dirty
Direct graph-owned `DataConverterFormula` random functions now cover
main-`ToTarget | TwoWay` target-dirty behavior for default-context boolean
sources feeding number targets. Rust preserves the manually edited number
target during explicit data-context advancement, keeps the boolean source
unchanged, reapplies C++'s numeric formula fallback on later normal
state-machine advancement, and keeps the host-supplied random-stream call
count at zero for `randomModeValue` values `0`, `1`, and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-random-boolean-fallback-main-to-target-two-way-target-dirty-runtime-contract.md`.
Enum/color/string/trigger target-dirty behavior is covered separately below.
Imported/owned contexts, real random generation, secondary dependency
invalidation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random remaining fallbacks target-dirty
Direct graph-owned `DataConverterFormula` random functions now cover
main-`ToTarget | TwoWay` target-dirty behavior for default-context enum,
color, string, and trigger sources feeding number targets. Rust preserves the
manually edited number target during explicit data-context advancement, keeps
enum/color/string sources unchanged, reapplies C++'s numeric formula fallback
on later normal state-machine advancement, and keeps the host-supplied
random-stream call count at zero for `randomModeValue` values `0`, `1`, and
`2`. Trigger source count/reset behavior is outside this mixed-source
number-target contract because the C++ probe does not expose trigger bindings
for this bind shape. The contract is
`docs/prototypes/data-binding-graph-formula-random-remaining-fallbacks-main-to-target-two-way-target-dirty-runtime-contract.md`.
Imported/owned contexts, real random generation, secondary dependency
invalidation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random remaining fallbacks target-to-source
Direct graph-owned `DataConverterFormula` random functions now cover explicit
`advanceDataContext()` and public `updateDataBinds(true)` target-to-source
behavior for default-context enum, color, string, and trigger sources feeding
number targets. Rust preserves the unchanged source when random-function
formula conversion produces a number, then reapplies that unchanged source
through the formula fallback so the number target returns to C++'s `0.0`
fallback for `randomModeValue` values `0`, `1`, and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-random-remaining-fallbacks-target-to-source-runtime-contract.md`.
List and symbol-list-index random target-to-source behavior is covered
separately below. Enum/color/string/trigger target-to-source call counts are
covered separately below. Imported/owned contexts, real random generation,
secondary dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula random remaining target-to-source call counts
Direct graph-owned `DataConverterFormula` random functions now cover Rust's
host-supplied random-stream call count for enum, color, string, and trigger
target-to-source fallback. Explicit `advanceDataContext()` and public
`updateDataBinds(true)` each consume one hidden pull during reverse
reapplication, then reuse that count through later normal advances for
`randomModeValue` values `0`, `1`, and `2`, matching the C++ probe observable
fallback values. The contract is
`docs/prototypes/data-binding-graph-formula-random-remaining-fallbacks-target-to-source-call-count-runtime-contract.md`.
The C++ probe now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares Rust's hidden reverse-path pull
count directly against the C++ total after every enum/color/string/trigger
target-to-source report. Imported/owned contexts, real random generation,
secondary dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula random list fallback target-to-source
Direct graph-owned `DataConverterFormula` random functions now cover explicit
`advanceDataContext()` and public `updateDataBinds(true)` target-to-source
behavior for default-context list sources feeding number targets. Rust
preserves the unchanged imported list source when random-function formula
conversion produces a number, then reapplies that unchanged source through the
formula fallback so the number target returns to C++'s `0.0` fallback for
`randomModeValue` values `0`, `1`, and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-random-list-fallback-target-to-source-runtime-contract.md`.
Symbol-list-index random target-to-source behavior is covered separately
below. Imported/owned contexts, real random generation, random call counts,
secondary dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index target-to-source
Direct graph-owned `DataConverterFormula` random functions now cover explicit
`advanceDataContext()` and public `updateDataBinds(true)` target-to-source
behavior for default-context symbol-list-index sources feeding number targets.
Rust preserves the unchanged symbol-list-index source when random-function
formula conversion produces a number, then reapplies that unchanged source
through the random formula scheduling path for `randomModeValue` values `0`,
`1`, and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-target-to-source-runtime-contract.md`.
Target-dirty scheduling is covered separately below. Grouped symbol-list-index
random formulas, imported/owned contexts, real random generation, random call
counts, secondary dependency invalidation, and full dirty-list scheduler
parity remain follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index explicit target-to-source call counts
Direct graph-owned `DataConverterFormula` random functions now expose Rust's
host-supplied random-stream call count for explicit
`advanceDataContext()` target-to-source scheduling on default-context
symbol-list-index sources feeding number targets. Default and source-change
modes consume one visible reapply draw because the symbol-list-index source is
preserved rather than changed; always mode consumes two values during the
explicit pass, including the hidden reverse-conversion draw, and later normal
advances do not pull more values. The C++ probe now seeds deterministic random
values and reports per-action `randomTotalCalls`, so this slice compares the
Rust host-stream count directly against the C++ total after every explicit
target-to-source report. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-target-to-source-call-count-runtime-contract.md`.
Direct public-update, target-dirty, grouped symbol-list-index, list/non-number,
imported/owned, real random generation, secondary dependency invalidation, and
full dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index public-update call counts
Direct graph-owned `DataConverterFormula` random functions now expose Rust's
host-supplied random-stream call count for public `updateDataBinds(true)`
target-to-source scheduling on default-context symbol-list-index sources
feeding number targets. Default and source-change modes keep the warmed
source-to-target draw because the symbol-list-index source is preserved rather
than changed; always mode consumes one initial draw plus two more values during
the public update, and later normal advances do not pull more values. The C++
probe now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares the Rust host-stream count directly
against the C++ total after every public-update report. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-public-update-call-count-runtime-contract.md`.
Direct target-dirty, grouped symbol-list-index, list/non-number,
imported/owned, real random generation, secondary dependency invalidation, and
full dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index target-dirty call counts
Direct graph-owned `DataConverterFormula` random functions now expose Rust's
host-supplied random-stream call count for main-`ToTarget | TwoWay`
target-dirty scheduling on default-context symbol-list-index sources feeding
number targets. Default and source-change modes consume one initial draw and
reuse it through target-dirty preservation and later normal advances; always
mode consumes one initial draw, one later reapply draw, and no additional
second-later draw. The C++ probe now seeds deterministic random values and
reports per-action `randomTotalCalls`, so this slice compares the Rust
host-stream count directly against the C++ total after every target-dirty
report. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-target-dirty-call-count-runtime-contract.md`.
Grouped symbol-list-index, list/non-number, imported/owned, real random
generation, secondary dependency invalidation, and full dirty-list scheduler
parity remain follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index target-dirty
Direct graph-owned `DataConverterFormula` random functions now cover
main-`ToTarget | TwoWay` target-dirty scheduling for default-context
symbol-list-index sources feeding number targets. Rust consumes or caches the
host-supplied random value during initial source-to-target application,
preserves a manual number target edit through explicit data-context
advancement, then reapplies the unchanged symbol-list-index source through the
random formula path on later normal advances for `randomModeValue` values `0`,
`1`, and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-target-dirty-runtime-contract.md`.
Grouped symbol-list-index default-mode source-to-target behavior is covered
separately below. Grouped non-default source-to-target and grouped explicit
target-to-source behavior are covered separately below. Grouped public-update
target-to-source and target-dirty behavior are covered separately below.
Imported/owned contexts, real random generation, grouped/list/non-number
random call counts, secondary dependency invalidation, and full dirty-list
scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index group source-to-target
Grouped graph-owned `DataConverterFormula` random functions now cover the
first grouped symbol-list-index source path:
`DataConverterGroup<OperationValue, Formula(random)>` over a default-context
symbol-list-index source feeding a number target. Rust admits the
symbol-list-index source for grouped number-producing converters, threads it
through the first group child, and evaluates the nested default-mode random
formula with the same host-supplied random stream and cache behavior as direct
formula binds. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-group-runtime-contract.md`.
Grouped non-default source-to-target behavior is covered separately below.
Grouped explicit target-to-source behavior is covered separately below.
Grouped public-update target-to-source and target-dirty behavior are covered
separately below. List formulas, remaining non-number random scheduling,
imported/owned contexts, real random generation, grouped target-to-source,
public-update, and target-dirty call counts, secondary dependency
invalidation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random symbol-list-index group non-default source-to-target
Grouped graph-owned `DataConverterFormula` random functions now cover
`RandomMode::always` and `RandomMode::sourceChange` source-to-target behavior
for `DataConverterGroup<OperationValue, Formula(random)>` over
default-context symbol-list-index sources feeding number targets. Rust
consumes fresh host-supplied random values for always-mode grouped formula
advancement, and clears the nested formula cache when
`set_default_view_model_symbol_list_index_source_for_data_bind` mutates the
bound default source for source-change mode. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-group-non-default-runtime-contract.md`.
Grouped explicit target-to-source behavior is covered separately below.
Grouped public-update target-to-source and target-dirty behavior are covered
separately below. List formulas, remaining non-number random scheduling,
imported/owned contexts, real random generation, grouped target-to-source,
public-update, and target-dirty call counts, secondary dependency
invalidation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random symbol-list-index group source-to-target call counts
Grouped graph-owned `DataConverterFormula` random functions now expose Rust's
host-supplied random-stream call count for source-to-target scheduling through
`DataConverterGroup<OperationValue, Formula(random)>` default-context
symbol-list-index sources feeding number targets. Default mode consumes one
initial draw and reuses it across later advancement, always mode consumes one
draw per grouped formula evaluation with the second evaluation scheduled by a
symbol-list-index source mutation, and source-change mode consumes one initial
draw plus one refresh after a symbol-list-index source mutation. The C++ probe
now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares the Rust host-stream count directly
against the C++ total after every grouped source-to-target report. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-group-call-count-runtime-contract.md`.
List/non-number paths, imported/owned contexts, real random generation,
secondary dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index group explicit target-to-source
Grouped graph-owned `DataConverterFormula` random functions now cover explicit
`advance_data_context` target-to-source scheduling for
`DataConverterGroup<OperationValue, Formula(random)>` default-context
symbol-list-index sources feeding number targets. Rust preserves the unchanged
symbol-list-index source when grouped reverse conversion produces a number,
marks the bind dirty for same-pass source-to-target reapplication, and matches
C++ target reports for `randomModeValue` values `0`, `1`, and `2`, including
the grouped reverse operation-value scale visible in main-`ToSource` target
values. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-group-target-to-source-runtime-contract.md`.
Grouped public-update target-to-source behavior is covered separately below.
Grouped target-dirty behavior is covered separately below. List formulas,
remaining non-number random scheduling, imported/owned contexts, real random
generation, grouped public-update and target-dirty call counts, secondary
dependency invalidation, and full dirty-list scheduler parity remain follow-up
`#12` slices.

Current #12 update: graph formula random symbol-list-index group explicit target-to-source call counts
Grouped graph-owned `DataConverterFormula` random functions now expose Rust's
host-supplied random-stream call count for explicit `advance_data_context`
target-to-source scheduling through
`DataConverterGroup<OperationValue, Formula(random)>` default-context
symbol-list-index sources feeding number targets. Default and source-change
modes consume one visible reapply draw because the symbol-list-index source is
preserved rather than changed; always mode consumes two values during the
explicit pass, including the hidden grouped reverse-conversion draw. The C++
probe now seeds deterministic random values and reports per-action
`randomTotalCalls`, so this slice compares the Rust host-stream count directly
against the C++ total after every grouped explicit target-to-source report.
The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-group-target-to-source-call-count-runtime-contract.md`.
List/non-number paths, imported/owned contexts, real random generation,
secondary dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index group public-update target-to-source
Grouped graph-owned `DataConverterFormula` random functions now cover public
`update_data_binds_apply_target_to_source` scheduling for
`DataConverterGroup<OperationValue, Formula(random)>` default-context
symbol-list-index sources feeding number targets. Rust preserves the unchanged
symbol-list-index source when grouped reverse conversion produces a number,
then reapplies that source in the same public update and matches C++ target
reports for `randomModeValue` values `0`, `1`, and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-group-public-update-target-to-source-runtime-contract.md`.
Grouped target-dirty behavior is covered separately below. List formulas,
remaining non-number random scheduling, imported/owned contexts, real random
generation, secondary dependency invalidation, and full dirty-list scheduler
parity remain follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index group public-update call counts
Grouped graph-owned `DataConverterFormula` random functions now expose Rust's
host-supplied random-stream call count for public `updateDataBinds(true)`
target-to-source scheduling through
`DataConverterGroup<OperationValue, Formula(random)>` default-context
symbol-list-index sources feeding number targets. Default and source-change
modes keep the warmed source-to-target draw because the symbol-list-index
source is preserved rather than changed; always mode consumes one initial draw
plus two more values during the public update. The C++ probe now seeds
deterministic random values and reports per-action `randomTotalCalls`, so this
slice compares the Rust host-stream count directly against the C++ total after
every grouped public-update report. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-group-public-update-call-count-runtime-contract.md`.
Imported/owned contexts, real random generation, secondary dependency
invalidation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random symbol-list-index group target-dirty
Grouped graph-owned `DataConverterFormula` random functions now cover
main-`ToTarget | TwoWay` target-dirty scheduling for
`DataConverterGroup<OperationValue, Formula(random)>` default-context
symbol-list-index sources feeding number targets. Rust preserves a manual
number target edit through explicit data-context advancement, then reapplies
the unchanged symbol-list-index source through the grouped formula path on
later normal advances for `randomModeValue` values `0`, `1`, and `2`,
matching C++. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-group-target-dirty-runtime-contract.md`.
List formulas, remaining non-number random scheduling, imported/owned
contexts, real random generation, secondary dependency invalidation, and full
dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random symbol-list-index group target-dirty call counts
Grouped graph-owned `DataConverterFormula` random functions now expose Rust's
host-supplied random-stream call count for main-`ToTarget | TwoWay`
target-dirty scheduling through
`DataConverterGroup<OperationValue, Formula(random)>` default-context
symbol-list-index sources feeding number targets. Default and source-change
modes consume one initial draw and reuse it through target-dirty preservation
and later normal advances; always mode consumes one initial draw, one later
reapply draw, and no additional second-later draw. The C++ probe now seeds
deterministic random values and reports per-action `randomTotalCalls`, so this
slice compares the Rust host-stream count directly against the C++ total after
every grouped target-dirty report. The contract is
`docs/prototypes/data-binding-graph-formula-random-symbol-list-index-group-target-dirty-call-count-runtime-contract.md`.
Remaining non-list non-number paths, imported/owned contexts, real random
generation, secondary dependency invalidation, and full dirty-list scheduler
parity remain follow-up `#12` slices.

Current #12 update: graph formula list fallback
Direct graph-owned `DataConverterFormula` now admits default-context list
sources feeding number targets. Rust keeps the list item count as a graph
source value, enters the formula converter, and matches C++ by writing the
early fallback value `0.0` for both `FormulaTokenInput` and
`FunctionType::random` output tokens with `randomModeValue` values `0`, `1`,
and `2`. The contract is
`docs/prototypes/data-binding-graph-formula-list-fallback-runtime-contract.md`.
Public-update, explicit target-to-source, and deterministic/random
target-dirty behavior for formula list sources is covered separately below.
The first list-target behavior is covered separately below. Generated list
items, imported/owned contexts, real random generation,
secondary dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula list fallback bindable-list target
Direct graph-owned `DataConverterFormula` now admits default-context list
sources feeding state-machine `BindablePropertyList.propertyValue` targets for
the deterministic `FormulaTokenInput` fallback path. Rust reports the imported
source list size, preserves the list target scalar during explicit
data-context advancement, and applies C++'s numeric fallback target value on
later normal state-machine advancement. The contract is
`docs/prototypes/data-binding-graph-formula-list-fallback-bindable-list-target-runtime-contract.md`.
Formula random-function list targets, deterministic target-to-source, and
deterministic/random target-dirty behavior are covered separately below.
Generated list items, imported/owned contexts, real random generation,
secondary dependency invalidation, and full dirty-list scheduler
parity remain follow-up `#12` slices.

Current #12 update: graph formula random list fallback bindable-list target
Direct graph-owned `DataConverterFormula` now admits default-context list
sources feeding state-machine `BindablePropertyList.propertyValue` targets
through `FunctionType::random` output tokens with `randomModeValue` values
`0`, `1`, and `2`. Rust reports the imported source list size, preserves the
list target scalar during explicit data-context advancement, ignores supplied
random values for this observable fallback, and applies C++'s numeric fallback
target value on later normal state-machine advancement. The contract is
`docs/prototypes/data-binding-graph-formula-random-list-fallback-bindable-list-target-runtime-contract.md`.
Random explicit and public-update target-to-source behavior is covered
separately below. Random target-dirty behavior is covered separately below.
Generated list items, imported/owned contexts, real random generation, random
call counts are covered separately below. Secondary dependency invalidation
and full dirty-list scheduler
parity remain follow-up `#12` slices.

Current #12 update: graph formula list fallback bindable-list explicit target-to-source
Direct graph-owned `DataConverterFormula` now covers explicit
`advanceDataContext()` target-to-source behavior for main-`ToSource | TwoWay`
default-context list sources feeding state-machine
`BindablePropertyList.propertyValue` targets through a deterministic
`FormulaTokenInput` converter. Rust preserves the edited list-target scalar,
keeps reporting the imported source list size, and avoids reapplying C++'s
numeric formula fallback during the same explicit target-to-source pass. The
contract is
`docs/prototypes/data-binding-graph-formula-list-fallback-bindable-list-explicit-target-to-source-runtime-contract.md`.
The deterministic public-update list-target reverse path is covered
separately below. Random explicit list-target target-to-source behavior is
covered separately below, and random public-update list-target
target-to-source behavior is covered separately below. Generated list items,
the deterministic and random target-dirty paths are covered separately below.
Imported/owned contexts, real random generation, secondary dependency
invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula list fallback bindable-list public update target-to-source
Direct graph-owned `DataConverterFormula` now covers public
`updateDataBinds(true)` target-to-source behavior for main-`ToTarget | TwoWay`
default-context list sources feeding state-machine
`BindablePropertyList.propertyValue` targets through a deterministic
`FormulaTokenInput` converter. Rust keeps reporting the imported source list
size and, matching C++, reapplies the numeric formula fallback to the list
target during the same public update. The contract is
`docs/prototypes/data-binding-graph-formula-list-fallback-bindable-list-public-update-target-to-source-runtime-contract.md`.
Random explicit list-target target-to-source behavior is covered separately
below, and random public-update list-target target-to-source behavior is
covered separately below. Generated list items, target-dirty scheduling for
formula list targets is covered separately below for deterministic and random
tokens. Imported/owned contexts, real random generation, secondary dependency
invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula list fallback bindable-list target-dirty
Direct graph-owned `DataConverterFormula` now covers main-`ToTarget | TwoWay`
target-dirty behavior for default-context list sources feeding state-machine
`BindablePropertyList.propertyValue` targets through a deterministic
`FormulaTokenInput` converter. Rust preserves the manually edited list-target
scalar during explicit data-context advancement, keeps reporting the imported
source list size, and reapplies C++'s numeric formula fallback on later normal
state-machine advancement. The contract is
`docs/prototypes/data-binding-graph-formula-list-fallback-bindable-list-main-to-target-two-way-target-dirty-runtime-contract.md`.
Random target-dirty behavior is covered separately below. Generated list
items, imported/owned contexts, real random generation, secondary dependency
invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula random list fallback bindable-list target-dirty
Direct graph-owned `DataConverterFormula` now covers main-`ToTarget | TwoWay`
target-dirty behavior for default-context list sources feeding state-machine
`BindablePropertyList.propertyValue` targets through `FunctionType::random`
output tokens with `randomModeValue` values `0`, `1`, and `2`. Rust preserves
the manually edited list-target scalar during explicit data-context
advancement, keeps reporting the imported source list size, ignores supplied
random values for this observable fallback, and reapplies C++'s numeric
formula fallback on later normal state-machine advancement. The contract is
`docs/prototypes/data-binding-graph-formula-random-list-fallback-bindable-list-main-to-target-two-way-target-dirty-runtime-contract.md`.
Generated list items, imported/owned contexts, real random generation, random
call counts are covered separately below. Secondary dependency invalidation
and full dirty-list scheduler
parity remain follow-up `#12` slices.

Current #12 update: graph formula random list fallback bindable-list explicit target-to-source
Direct graph-owned `DataConverterFormula` now covers explicit
`advanceDataContext()` target-to-source behavior for main-`ToSource | TwoWay`
default-context list sources feeding state-machine
`BindablePropertyList.propertyValue` targets through `FunctionType::random`
output tokens with `randomModeValue` values `0`, `1`, and `2`. Rust preserves
the edited list-target scalar, keeps reporting the imported source list size,
ignores supplied random values for this observable reverse fallback, and
avoids reapplying C++'s numeric formula fallback during the same explicit
target-to-source pass. The contract is
`docs/prototypes/data-binding-graph-formula-random-list-fallback-bindable-list-explicit-target-to-source-runtime-contract.md`.
Random public-update list-target target-to-source behavior is covered
separately below. Random target-dirty behavior is covered separately above.
Generated list items, imported/owned contexts, real random generation, random
call counts are covered separately below. Secondary dependency invalidation
and full dirty-list scheduler
parity remain follow-up `#12` slices.

Current #12 update: graph formula random list fallback bindable-list public update target-to-source
Direct graph-owned `DataConverterFormula` now covers public
`updateDataBinds(true)` target-to-source behavior for main-`ToTarget | TwoWay`
default-context list sources feeding state-machine
`BindablePropertyList.propertyValue` targets through `FunctionType::random`
output tokens with `randomModeValue` values `0`, `1`, and `2`. Rust keeps
reporting the imported source list size, ignores supplied random values for
this observable reverse fallback, and reapplies the numeric formula fallback
to the list target during the same public update. The contract is
`docs/prototypes/data-binding-graph-formula-random-list-fallback-bindable-list-public-update-target-to-source-runtime-contract.md`.
Random target-dirty behavior is covered separately above. Generated list
items, imported/owned contexts, real random generation, secondary dependency
invalidation, and full dirty-list scheduler parity remain follow-up `#12`
slices. Random call counts are covered by the list call-count slice below.

Current #12 update: graph formula random list fallback call counts
Direct graph-owned `DataConverterFormula` now covers Rust's host-supplied
random-stream call count for default-context list sources flowing through
`FunctionType::random` fallback tokens. Source-to-target and target-dirty
list fallback paths keep the count at zero, bindable-list targets keep the
count at zero, and number-target explicit/public reverse reapplication
also keeps the count at zero before landing on the same C++ probe observable
fallback values. Rust short-circuits direct formula conversion for preserved
list sources during number target-to-source application so it does not pull a
host random value that C++ never requests. The C++ probe now installs the
counted random provider with `--runtime-random-reset` and repeated
`--runtime-random-value` arguments, then compares each
`RandomProvider::totalCalls()` report against the matching Rust state-machine
clone. The contract is
`docs/prototypes/data-binding-graph-formula-random-list-fallback-call-count-runtime-contract.md`.
Imported/owned contexts, real random generation, secondary dependency
invalidation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula boolean fallback public update target-to-source
Direct graph-owned `DataConverterFormula` now covers public
`updateDataBinds(true)` target-to-source behavior for main-`ToTarget | TwoWay`
default-context boolean sources feeding number targets. Rust preserves the
unchanged boolean source when reverse formula conversion produces a number,
then reapplies that unchanged source through the formula fallback so the
number target returns to C++'s `0.0` fallback. The contract is
`docs/prototypes/data-binding-graph-formula-boolean-fallback-public-update-target-to-source-runtime-contract.md`.
Enum, color, string, trigger, list, and symbol-list-index public-update
behavior is covered separately below. Explicit main-`ToSource` behavior for
boolean and the other graph-represented fallback source kinds is also covered
separately below. Imported/owned contexts, random formula reverse behavior,
secondary dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula boolean fallback explicit target-to-source
Direct graph-owned `DataConverterFormula` now covers explicit
`advanceDataContext()` target-to-source behavior for main-`ToSource | TwoWay`
default-context boolean sources feeding number targets. Rust preserves the
unchanged boolean source when main-direction formula conversion produces a
number, then reapplies that unchanged source through the formula fallback so
the number target returns to C++'s `0.0` fallback. The contract is
`docs/prototypes/data-binding-graph-formula-boolean-fallback-explicit-target-to-source-runtime-contract.md`.
Enum, color, string, and trigger explicit behavior is covered separately
below. List, symbol-list-index, imported/owned contexts, random formula
reverse behavior, secondary dependency invalidation, and full dirty-list
scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula remaining fallbacks explicit target-to-source
Direct graph-owned `DataConverterFormula` now covers explicit
`advanceDataContext()` target-to-source behavior for main-`ToSource | TwoWay`
default-context enum, color, string, and trigger sources feeding number
targets. Rust preserves the unchanged non-number source when main-direction
formula conversion produces a number, then reapplies that unchanged source
through the formula fallback so the number target returns to C++'s `0.0`
fallback. The contract is
`docs/prototypes/data-binding-graph-formula-remaining-fallbacks-explicit-target-to-source-runtime-contract.md`.
List explicit behavior is covered separately below. Symbol-list-index,
imported/owned contexts, random formula reverse behavior, secondary dependency
invalidation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula remaining fallbacks public update target-to-source
Direct graph-owned `DataConverterFormula` now covers public
`updateDataBinds(true)` target-to-source behavior for main-`ToTarget | TwoWay`
default-context enum, color, string, and trigger sources feeding number
targets. Rust preserves the unchanged non-number source when reverse formula
conversion produces a number, then reapplies that unchanged source through the
formula fallback so the number target returns to C++'s `0.0` fallback. The
contract is
`docs/prototypes/data-binding-graph-formula-remaining-fallbacks-public-update-target-to-source-runtime-contract.md`.
Explicit main-`ToSource` behavior for these source kinds is covered separately
above. List and symbol-list-index public-update behavior is covered separately
below. Imported/owned contexts, random formula reverse behavior, secondary
dependency invalidation, and full dirty-list scheduler parity remain follow-up
`#12` slices.

Current #12 update: graph formula list fallback public update target-to-source
Direct graph-owned `DataConverterFormula` now covers public
`updateDataBinds(true)` target-to-source behavior for main-`ToTarget | TwoWay`
default-context list sources feeding number targets. Rust preserves the
unchanged imported list source when reverse formula conversion produces a
number, then reapplies that unchanged list source through the formula fallback
so the number target returns to C++'s `0.0` fallback. The contract is
`docs/prototypes/data-binding-graph-formula-list-fallback-public-update-target-to-source-runtime-contract.md`.
The first deterministic formula list target and deterministic explicit
formula list-target target-to-source behavior are covered separately above.
Deterministic and random target-dirty behavior is covered separately below.
Generated list items, list-target target-to-source scheduling beyond the
deterministic explicit/public-update slices, imported/owned contexts, random
formula reverse behavior, secondary dependency invalidation, and full
dirty-list scheduler parity remain follow-up `#12` slices. Explicit
main-`ToSource` behavior for number targets and symbol-list-index
public-update reverse behavior are covered separately below.

Current #12 update: graph formula list fallback explicit target-to-source
Direct graph-owned `DataConverterFormula` now covers explicit
`advanceDataContext()` target-to-source behavior for main-`ToSource | TwoWay`
default-context list sources feeding number targets. Rust preserves the
unchanged imported list source when main-direction formula conversion produces
a number, then reapplies that unchanged list source through the formula
fallback so the number target returns to C++'s `0.0` fallback. The contract is
`docs/prototypes/data-binding-graph-formula-list-fallback-explicit-target-to-source-runtime-contract.md`.
The deterministic formula list target explicit behavior is covered separately
above. Deterministic and random target-dirty behavior is covered separately
below. Symbol-list-index explicit behavior is covered separately below.
Imported/owned contexts, random formula reverse behavior, secondary
dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula list fallback target-dirty
Direct graph-owned `DataConverterFormula` now covers main-`ToTarget | TwoWay`
target-dirty behavior for default-context list sources feeding number targets
through a deterministic `FormulaTokenInput` converter. Rust preserves the
manually edited number target during explicit data-context advancement, keeps
reporting the imported source list size, and reapplies C++'s numeric formula
fallback on later normal state-machine advancement. The contract is
`docs/prototypes/data-binding-graph-formula-list-fallback-main-to-target-two-way-target-dirty-runtime-contract.md`.
Random target-dirty behavior is covered separately below. Imported/owned
contexts, secondary dependency invalidation, and full dirty-list scheduler
parity remain follow-up `#12` slices.

Current #12 update: graph formula random list fallback target-dirty
Direct graph-owned `DataConverterFormula` now covers main-`ToTarget | TwoWay`
target-dirty behavior for default-context list sources feeding number targets
through `FunctionType::random` output tokens with `randomModeValue` values
`0`, `1`, and `2`. Rust preserves the manually edited number target during
explicit data-context advancement, keeps reporting the imported source list
size, ignores supplied random values for this observable fallback, and
reapplies C++'s numeric formula fallback on later normal state-machine
advancement. The contract is
`docs/prototypes/data-binding-graph-formula-random-list-fallback-main-to-target-two-way-target-dirty-runtime-contract.md`.
Imported/owned contexts, real random generation, random call counts,
secondary dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula symbol-list-index public update target-to-source
Direct graph-owned `DataConverterFormula` now covers public
`updateDataBinds(true)` target-to-source behavior for main-`ToTarget | TwoWay`
default-context symbol-list-index sources feeding number targets. Rust
preserves the unchanged symbol-list-index source when reverse formula
conversion produces a number, then reapplies that unchanged source through the
formula converter so the number target returns to the C++ formula value. The
contract is
`docs/prototypes/data-binding-graph-formula-symbol-list-index-public-update-target-to-source-runtime-contract.md`.
Explicit main-`ToSource` symbol-list-index behavior and target-dirty behavior
are covered separately below, and random formula reverse behavior is covered
separately above. Imported/owned contexts, secondary dependency invalidation,
and full dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula symbol-list-index explicit target-to-source
Direct graph-owned `DataConverterFormula` now covers explicit
`advanceDataContext()` target-to-source behavior for main-`ToSource | TwoWay`
default-context symbol-list-index sources feeding number targets. Rust
preserves the unchanged symbol-list-index source when main-direction formula
conversion produces a number, then reapplies that unchanged source through the
formula converter so the number target returns to the C++ formula value. The
contract is
`docs/prototypes/data-binding-graph-formula-symbol-list-index-explicit-target-to-source-runtime-contract.md`.
Target-dirty behavior is covered separately below, and random formula reverse
behavior is covered separately above. Imported/owned contexts, secondary
dependency invalidation, and full dirty-list scheduler parity remain
follow-up `#12` slices.

Current #12 update: graph formula symbol-list-index target-dirty
Direct graph-owned `DataConverterFormula` now covers main-`ToTarget | TwoWay`
target-dirty behavior for default-context symbol-list-index sources feeding
number targets through deterministic input/value/operation formula tokens.
Rust preserves the manually edited number target during explicit data-context
advancement, keeps the symbol-list-index source unchanged, and reapplies
C++'s formula value on later normal state-machine advancement. The contract
is
`docs/prototypes/data-binding-graph-formula-symbol-list-index-main-to-target-two-way-target-dirty-runtime-contract.md`.
Deterministic function-token target-dirty behavior is covered separately
below. Imported/owned contexts, secondary dependency invalidation, and full
dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula symbol-list-index function target-dirty
Direct graph-owned `DataConverterFormula` now covers main-`ToTarget | TwoWay`
target-dirty behavior for default-context symbol-list-index sources feeding
number targets through deterministic `FormulaTokenFunction` output tokens.
Rust preserves the manually edited number target during explicit data-context
advancement, keeps the symbol-list-index source unchanged, and reapplies
C++'s function-token formula value on later normal state-machine advancement.
The contract is
`docs/prototypes/data-binding-graph-formula-symbol-list-index-function-main-to-target-two-way-target-dirty-runtime-contract.md`.
Imported/owned contexts, secondary dependency invalidation, and full
dirty-list scheduler parity remain follow-up `#12` slices.

Current #12 update: graph formula random group public update target-to-source
Grouped graph-owned `DataConverterFormula` random functions now cover public
`updateDataBinds(true)` target-to-source scheduling for a
main-`ToTarget | TwoWay` default-context number bind. A
`DataConverterGroup<OperationValue, Formula(random)>` bind applies the target
mutation through C++ reverse group order, reuses the cached default-mode
formula random value, and reapplies source-to-target in the same public update.
The contract is
`docs/prototypes/data-binding-graph-formula-random-group-public-update-target-to-source-runtime-contract.md`.
List random formulas, non-number random formulas, non-default random modes,
cache invalidation, call counts, imported/owned contexts, real random
generation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random group explicit target-to-source
Grouped graph-owned `DataConverterFormula` random functions now cover explicit
main-`ToSource | TwoWay` target-to-source scheduling for default-context number
binds. A manual edit to a `DataConverterGroup<OperationValue,
Formula(random)>` target is applied through forward group order during
explicit data-context advancement, and a neighboring direct number bind
observes the grouped source write on subsequent state-machine advancement. The
contract is
`docs/prototypes/data-binding-graph-formula-random-group-target-to-source-runtime-contract.md`.
List random formulas, non-number random formulas, non-default random modes,
cache invalidation, call counts, imported/owned contexts, real random
generation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: graph formula random group target-dirty
Grouped graph-owned `DataConverterFormula` random functions now cover
main-`ToTarget | TwoWay` state-machine target-dirty behavior for
default-context number binds. A manual edit to a
`DataConverterGroup<OperationValue, Formula(random)>` target is preserved
through explicit data-context advancement, then the next normal state-machine
advance reapplies the unchanged source through the cached grouped formula
random value. The contract is
`docs/prototypes/data-binding-graph-formula-random-group-target-dirty-runtime-contract.md`.
List random formulas, non-number random formulas, non-default random modes,
cache invalidation, call counts, imported/owned contexts, real random
generation, and full dirty-list scheduler parity remain follow-up `#12`
slices.

Current #12 update: non-scripting `ScriptedDataConverter` now participates in
the runtime data-bind graph as inherited C++ base converter pass-through. A
main-`ToTarget | TwoWay` number bind with a direct scripted converter applies
source-to-target unchanged, then public `updateDataBinds(true)` applies base
`reverseConvert` unchanged before reapplying source-to-target in the same
update. The contract is
`docs/prototypes/data-binding-graph-scripted-pass-through-runtime-contract.md`.
Actual script execution, script assets, scripted listener actions, scripted
converter groups, full dirty-list scheduler parity, imported/owned contexts,
pending add/remove behavior, re-entry protection, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: concrete `DataConverterOperation` now participates in the
runtime data-bind graph as inherited C++ base converter pass-through. A
main-`ToTarget | TwoWay` number bind with a direct operation base converter
applies source-to-target unchanged, then public `updateDataBinds(true)` applies
base `reverseConvert` unchanged before reapplying source-to-target in the same
update. The contract is
`docs/prototypes/data-binding-graph-operation-pass-through-runtime-contract.md`.
Arithmetic operation-value/view-model behavior, operation converter groups
beyond already admitted concrete group slices, full dirty-list scheduler
parity, imported/owned contexts, pending add/remove behavior, re-entry
protection, relative/parent/nested lookup, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: trigger public `updateDataBinds(true)` target-to-source
behavior now has a narrow C++ parity slice. The C++ probe reports exact
trigger binding source/target values, and Rust compares a main-`ToTarget |
TwoWay` trigger bind with direct `DataConverterTrigger`: public update applies
inherited base `reverseConvert` as target-to-source pass-through, mirrors the
default trigger source, then reapplies `DataConverterTrigger::convert` so the
bindable target receives the incremented trigger count. The contract is
`docs/prototypes/data-binding-graph-trigger-public-update-target-to-source-runtime-contract.md`.
Listener dispatch, trigger side effects, trigger converter groups,
imported/owned contexts, broader update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: explicit `advancedDataContext()` trigger converter
target-to-source behavior now has a narrow default-context parity slice. A
main-`ToSource | TwoWay` trigger bind with direct `DataConverterTrigger`
applies C++ main-to-source conversion to the edited bindable target before
writing the default trigger source, and the C++ probe compares exact
source/target trigger binding rows after the explicit data-context actions.
The contract is
`docs/prototypes/data-binding-graph-trigger-converter-target-to-source-runtime-contract.md`.
Trigger source reset reapplication after later state-machine advancement,
listener dispatch, trigger side effects, trigger converter groups,
imported/owned contexts, broader update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: trigger source reset now reapplies to default-context
trigger bindable targets on the following state-machine advance. Explicit
`advancedDataContext()` resets bound `ViewModelInstanceTrigger.propertyValue`
sources to `0`; Rust now marks changed trigger sources for source-to-target
reapplication, clears stale reset reapply state when an explicit target edit is
consumed first, and uses C++'s non-main converter direction for
`DataConverterTrigger` main-`ToSource | TwoWay` source-to-target application so
reset source `0` writes target `0`. The contract is
`docs/prototypes/data-binding-graph-trigger-source-reset-reapply-runtime-contract.md`.
Listener dispatch, trigger side effects, trigger converter groups,
imported/owned contexts, broader update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: trigger converter-group public `updateDataBinds(true)`
target-to-source behavior now has its first parity slice. A main-`ToTarget |
TwoWay` trigger bind with `DataConverterGroup<DataConverterTrigger>` uses C++
group reverse order to pass the edited target value through inherited
`DataConverterTrigger::reverseConvert`, writes the default trigger source, then
reapplies source-to-target through group forward order so
`DataConverterTrigger::convert` increments the bindable target. The contract is
`docs/prototypes/data-binding-graph-trigger-converter-group-public-update-target-to-source-runtime-contract.md`.
Explicit trigger group behavior outside the admitted one-child and first
two-child all-trigger slices, mixed or larger trigger groups, listener
dispatch, trigger side effects, imported/owned contexts, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: trigger converter-group explicit `advancedDataContext()`
target-to-source behavior now has its matching default-context parity slice. A
main-`ToSource | TwoWay` trigger bind with
`DataConverterGroup<DataConverterTrigger>` uses C++ group forward order in the
main-to-source direction, so `DataConverterTrigger::convert` increments the
edited bindable target before writing the default trigger source. The contract
is
`docs/prototypes/data-binding-graph-trigger-converter-group-target-to-source-runtime-contract.md`.
Mixed or larger trigger groups, listener dispatch, trigger side effects,
imported/owned contexts, broader update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: trigger converter groups now cover the first multi-child
all-trigger shape. A
`DataConverterGroup<DataConverterTrigger, DataConverterTrigger>` is probe-backed
for both public `updateDataBinds(true)` main-`ToTarget | TwoWay` and explicit
`advancedDataContext()` main-`ToSource | TwoWay` paths. Public update reverses
the group for target-to-source pass-through and reapplies forward conversion so
the two trigger converters increment the target twice; explicit
main-to-source advancement applies group forward order before writing the
source. The contract is
`docs/prototypes/data-binding-graph-trigger-converter-multi-group-runtime-contract.md`.
Mixed trigger groups, larger trigger groups, listener dispatch, trigger side
effects, imported/owned contexts, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: owned runtime view-model contexts now cover the first
live view-model pointer replacement path. `RuntimeOwnedViewModelInstance`
records root `ViewModelPropertyViewModel` properties plus the imported instance
IDs for their referenced view model, and
`set_view_model_by_property_index` stores an imported child pointer that
`RuntimeDataBindGraph` resolves into `BindablePropertyViewModel.propertyValue`
targets on explicit data-context advance. The C++ probe mirrors this with
`--runtime-bind-owned-view-model-viewmodel-state-machine-context`, creating an
owned root instance, wrapping a referenced imported child instance in
`ViewModelInstanceRuntime`, calling `replaceViewModelByName`, and binding the
owned root to the state machine. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-context-runtime-contract.md`.
Stable public source handles, list bindables, default/imported-context
view-model pointer relink APIs, unmutated generated-owned child identity,
remaining property-name APIs, reverse propagation, broader update-queue
parity, relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned runtime view-model contexts now also model C++'s
generated nested child before any replacement. Valid root
`ViewModelPropertyViewModel` sources in `RuntimeOwnedViewModelInstance` default
to a non-null owned pointer identity, so binding an owned root context without
calling `set_view_model_by_property_index` matches
`File::createViewModelInstance(viewModel)` and can drive
`BindablePropertyViewModel.propertyValue` pointer comparisons against `null`.
The C++ probe mirrors this with
`--runtime-bind-owned-view-model-viewmodel-default-state-machine-context`, which
creates and binds the owned root without `replaceViewModelByName`. The contract
is
`docs/prototypes/data-binding-graph-owned-viewmodel-generated-child-runtime-contract.md`.
Stable public source handles, list bindables, default/imported-context
view-model pointer relink APIs, reverse propagation, broader update-queue
parity, relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: artboard-owned list-consumer binding now has its first C++
probe-backed runtime boundary. Exact `ArtboardComponentList` targets bound to
the default root view-model context report direct `ViewModelInstanceList`
source sizes and direct `DataConverterNumberToList` source numbers, plus the
C++ immediate target local, empty target-list size, and reset flag behavior.
The contract is
`docs/prototypes/data-binding-graph-artboard-list-consumer-runtime-contract.md`.
Artboard component-list item instancing, generated child identity propagation,
map-rule selection, list layout/virtualization, reverse conversion,
target-to-source list behavior, broader update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: artboard-owned list-consumer binding now has an explicit
direct-update boundary. After binding the default artboard view-model context,
direct `Artboard::updateDataBinds(true)` preserves the immediate empty
`ArtboardComponentList` target-list report for direct list sources and direct
`DataConverterNumberToList` sources; it does not perform the post-bind
target-count mutation. The contract is
`docs/prototypes/data-binding-graph-artboard-list-direct-update-boundary-runtime-contract.md`.

Current #12 update: artboard-owned list-consumer binding now also covers the
first post-bind advance target-count report. After binding the default artboard
view-model context, a zero-second public `Artboard::advance(0.0f)` updates the
exact `ArtboardComponentList` target list count for direct list sources and
direct `DataConverterNumberToList` sources. Rust mirrors that through
`ArtboardInstance::advance_artboard_data_binds()`, while direct
`Artboard::updateDataBinds(true)` is covered separately as a no-target-count
boundary. The contract is
`docs/prototypes/data-binding-graph-artboard-list-advance-target-count-runtime-contract.md`.
Child artboard clone surfaces, item identity reuse/disposal, map-rule-driven
child creation, list layout/virtualization, target-to-source list behavior,
generated-list reverse conversion, broader update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: default-context view-model pointer sources now have a live
relink API separate from the raw generated setter path.
`StateMachineInstance::relink_default_view_model_view_model_source_for_data_bind`
updates same-path graph sources to a referenced imported child instance,
matching the C++ probe's cached-reference replacement path. Normal
state-machine advancement then pushes that relinked pointer into same-path
`BindablePropertyViewModel.propertyValue` targets. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-relink-runtime-contract.md`.
Imported-context view-model relinking, nested owned view-model pointer paths,
remaining property-name APIs, public object-handle APIs, reverse propagation,
broader update queues, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: imported view-model contexts now have the first live
view-model pointer relink API.
`StateMachineInstance::relink_view_model_instance_view_model_source_for_data_bind`
updates the currently bound imported-context graph source to a referenced
imported child instance; explicit data-context advance pushes it into
`BindablePropertyViewModel.propertyValue`, matching the C++ probe's
cached-reference replacement path. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-relink-runtime-contract.md`.
Persistent `RuntimeFile` mutation across rebinds, nested owned view-model
pointer paths, remaining property-name APIs, public object-handle APIs, reverse
propagation, broader update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned view-model contexts now support the first nested
view-model pointer relink path. `RuntimeOwnedViewModelInstance` stores
one-intermediate generated owned `ViewModelPropertyViewModel` children and
`set_view_model_by_property_path(&[rootProperty, nestedProperty], index)`
can relink the nested source to a referenced imported instance before binding
the owned context. The C++ probe covers the equivalent
`ViewModelInstanceRuntime::replaceViewModel("child/grandchild", value)` path.
The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-relink-runtime-contract.md`.
Persistent imported instance mutation, remaining property-name APIs, public
object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now recursively record generated
`ViewModelPropertyViewModel` children for generated-only pointer paths. A deep
owned path such as `[root, child, middle, leaf]` can be relinked with
`RuntimeOwnedViewModelInstance::set_view_model_by_property_path`, then bound to
the state machine and advanced into a `BindablePropertyViewModel.propertyValue`
target. The C++ probe covers the matching
`ViewModelInstanceRuntime::replaceViewModel("child/middle/leaf", value)` path.
The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-recursive-relink-runtime-contract.md`.
Remaining property-name APIs, public object-handle APIs, reverse propagation,
broader update queues, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse one imported
replacement intermediate for view-model pointer sources. Replacing a generated
root child with an imported child by instance index makes a source path such as
`[root, child, grandchild]` read the imported child's existing nested
`ViewModelInstanceViewModel` reference, matching C++ state-machine data-context
binding. The companion probe pins that attempting to relink through the
imported intermediate with
`ViewModelInstanceRuntime::replaceViewModel("child/grandchild", value)` does
not update this admitted binding path, so Rust returns `false` for
`set_view_model_by_property_path(&[child, grandchild], value)` once `child` is
imported. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-runtime-contract.md`.
Persistent imported instance mutation, remaining property-name APIs, public
object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse one imported
replacement intermediate for direct number sources. Replacing a generated root
child with an imported child by instance index makes a source path such as
`[root, child, amount]` read the imported child's existing
`ViewModelInstanceNumber.propertyValue`, matching C++ state-machine
data-context binding. Rust records imported number snapshots per referenced
view-model instance and uses them only for read-only graph source resolution.
The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-number-runtime-contract.md`.
At that slice boundary, boolean and other imported-intermediate scalar kinds,
persistent imported instance mutation, remaining property-name APIs, public
object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remained follow-up `#12` slices.

Current #12 update: owned view-model contexts now explicitly pin direct number
name-path mutation through an imported replacement intermediate as
unsupported. The C++ probe replaces generated `child` with an imported child,
attempts `ViewModelInstanceRuntime::propertyNumber("child/amount")->value(...)`,
then binds the owned context; C++ leaves the imported child's existing number
source selected. Rust matches by returning `false` for
`set_number_by_property_name_path("child/amount", value)` once `child` is
imported and by preserving the read-only imported number snapshot. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-number-name-path-unsupported-runtime-contract.md`.
Other imported-intermediate value mutation APIs, remaining property-name APIs,
public object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse one imported
replacement intermediate for direct boolean sources. Replacing a generated
root child with an imported child by instance index makes a source path such
as `[root, child, enabled]` read the imported child's existing
`ViewModelInstanceBoolean.propertyValue`, matching C++ state-machine
data-context binding as observed through a boolean transition condition. Rust
records imported boolean snapshots per referenced view-model instance and uses
them only for read-only graph source resolution. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-boolean-runtime-contract.md`.
At that slice boundary, string and other imported-intermediate scalar kinds
beyond number and boolean, persistent imported instance mutation, remaining
property-name APIs, public object-handle APIs, reverse propagation, broader
update queues, relative/parent/nested lookup, listener-owned data binding, and
nested artboard propagation remained follow-up `#12` slices.

Current #12 update: owned view-model contexts now explicitly pin direct boolean
name-path mutation through an imported replacement intermediate as
unsupported. The C++ probe replaces generated `child` with an imported child,
attempts
`ViewModelInstanceRuntime::propertyBoolean("child/enabled")->value(...)`, then
binds the owned context; C++ leaves the imported child's existing boolean
source selected. Rust matches by returning `false` for
`set_boolean_by_property_name_path("child/enabled", value)` once `child` is
imported and by preserving the read-only imported boolean snapshot. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-boolean-name-path-unsupported-runtime-contract.md`.
Other imported-intermediate value mutation APIs, remaining property-name APIs,
public object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse one imported
replacement intermediate for direct string sources. Replacing a generated
root child with an imported child by instance index makes a source path such
as `[root, child, label]` read the imported child's existing
`ViewModelInstanceString.propertyValue`, matching C++ state-machine
data-context binding as observed through a string transition condition. Rust
records imported string snapshots per referenced view-model instance and uses
them only for read-only graph source resolution. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-string-runtime-contract.md`.
At that slice boundary, color and other imported-intermediate scalar kinds
beyond number, boolean, and string, persistent imported instance mutation,
remaining property-name APIs, public object-handle APIs, reverse propagation,
broader update queues, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remained follow-up `#12` slices.

Current #12 update: owned view-model contexts now explicitly pin direct string
name-path mutation through an imported replacement intermediate as
unsupported. The C++ probe replaces generated `child` with an imported child,
attempts `ViewModelInstanceRuntime::propertyString("child/label")->value(...)`,
then binds the owned context; C++ leaves the imported child's existing string
source selected. Rust matches by returning `false` for
`set_string_by_property_name_path("child/label", value)` once `child` is
imported and by preserving the read-only imported string snapshot. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-string-name-path-unsupported-runtime-contract.md`.
Other imported-intermediate value mutation APIs, remaining property-name APIs,
public object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse one imported
replacement intermediate for direct color sources. Replacing a generated root
child with an imported child by instance index makes a source path such as
`[root, child, tint]` read the imported child's existing
`ViewModelInstanceColor.propertyValue`, matching C++ state-machine
data-context binding as observed through a color transition condition. Rust
records imported color snapshots per referenced view-model instance and uses
them only for read-only graph source resolution. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-color-runtime-contract.md`.
At that slice boundary, enum and other imported-intermediate scalar kinds
beyond number, boolean, string, and color, persistent imported instance
mutation, remaining property-name APIs, public object-handle APIs, reverse
propagation, broader update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remained
follow-up `#12` slices.

Current #12 update: owned view-model contexts now explicitly pin direct color
name-path mutation through an imported replacement intermediate as
unsupported. The C++ probe replaces generated `child` with an imported child,
attempts `ViewModelInstanceRuntime::propertyColor("child/tint")->value(...)`,
then binds the owned context; C++ leaves the imported child's existing color
source selected. Rust matches by returning `false` for
`set_color_by_property_name_path("child/tint", value)` once `child` is
imported and by preserving the read-only imported color snapshot. The contract
is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-color-name-path-unsupported-runtime-contract.md`.
Other imported-intermediate value mutation APIs, remaining property-name APIs,
public object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse one imported
replacement intermediate for direct enum sources. Replacing a generated root
child with an imported child by instance index makes a source path such as
`[root, child, choice]` read the imported child's existing
`ViewModelInstanceEnum.propertyValue` index, matching C++ state-machine
data-context binding as observed through an enum transition condition. Rust
records imported enum-index snapshots per referenced view-model instance and
uses them only for read-only graph source resolution. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-enum-runtime-contract.md`.
At that slice boundary, symbol-list-index and other imported-intermediate
scalar kinds beyond number, boolean, string, color, and enum, persistent
imported instance mutation, remaining property-name APIs, public object-handle
APIs, reverse propagation, broader update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remained
follow-up `#12` slices.

Current #12 update: owned view-model contexts now explicitly pin direct enum
name-path mutation through an imported replacement intermediate as
unsupported. The C++ probe replaces generated `child` with an imported child,
attempts
`ViewModelInstanceRuntime::propertyEnum("child/choice")->valueIndex(...)`, then
binds the owned context; C++ leaves the imported child's existing enum source
selected. Rust matches by returning `false` for
`set_enum_by_property_name_path("child/choice", value)` once `child` is
imported and by preserving the read-only imported enum snapshot. The contract
is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-enum-name-path-unsupported-runtime-contract.md`.
Other imported-intermediate value mutation APIs, remaining property-name APIs,
public object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse one imported
replacement intermediate for direct symbol-list-index sources. Replacing a
generated root child with an imported child by instance index makes a source
path such as `[root, child, symbol]` read the imported child's existing
`ViewModelInstanceSymbolListIndex.propertyValue`, matching C++ state-machine
data-context binding as observed through the existing `DataConverterToString`
path and a string transition condition. Rust records imported
symbol-list-index snapshots per referenced view-model instance and uses them
only for read-only graph source resolution. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-symbol-list-index-runtime-contract.md`.
At that slice boundary, asset and other imported-intermediate source kinds
beyond number, boolean, string, color, enum, and symbol-list-index, persistent
imported instance mutation, remaining property-name APIs, public object-handle
APIs, reverse propagation, broader update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remained
follow-up `#12` slices.

Current #12 update: owned view-model contexts now explicitly pin direct
symbol-list-index name-path mutation through an imported replacement
intermediate as unsupported. The C++ probe replaces generated `child` with an
imported child, resolves the owner with
`ViewModelInstanceRuntime::propertyViewModel("child")`, attempts to write the
child's `ViewModelInstanceSymbolListIndex.propertyValue`, then binds the owned
context; C++ leaves the imported child's existing symbol-list-index source
selected. Rust matches by returning `false` for
`set_symbol_list_index_by_property_name_path("child/symbol", value)` once
`child` is imported and by preserving the read-only imported
symbol-list-index snapshot. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-symbol-list-index-name-path-unsupported-runtime-contract.md`.
Other imported-intermediate value mutation APIs, remaining property-name APIs,
public object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse one imported
replacement intermediate for direct asset sources. Replacing a generated root
child with an imported child by instance index makes a source path such as
`[root, child, image]` read the imported child's existing
`ViewModelInstanceAssetImage.propertyValue`, matching C++ state-machine
data-context binding as observed through an asset transition condition. Rust
records imported asset-id snapshots per referenced view-model instance and
uses them only for read-only graph source resolution. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-asset-runtime-contract.md`.
At that slice boundary, artboard and other imported-intermediate source kinds
beyond number, boolean, string, color, enum, symbol-list-index, and asset,
persistent imported instance mutation, remaining property-name APIs, public
object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remained follow-up `#12` slices.

Current #12 update: owned view-model contexts now explicitly pin direct asset
name-path mutation through an imported replacement intermediate as
unsupported. The C++ probe replaces generated `child` with an imported child,
resolves the owner with `ViewModelInstanceRuntime::propertyViewModel("child")`,
attempts to write the child's `ViewModelInstanceAssetImage.propertyValue`,
then binds the owned context; C++ leaves the imported child's existing asset
source selected. Rust matches by returning `false` for
`set_asset_by_property_name_path("child/image", value)` once `child` is
imported and by preserving the read-only imported asset snapshot. The contract
is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-asset-name-path-unsupported-runtime-contract.md`.
Other imported-intermediate value mutation APIs, remaining property-name APIs,
public object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse one imported
replacement intermediate for direct artboard sources. Replacing a generated
root child with an imported child by instance index makes a source path such
as `[root, child, scene]` read the imported child's existing
`ViewModelInstanceArtboard.propertyValue`, matching C++ state-machine
data-context binding as observed through an artboard transition condition
that stays unsatisfied after the imported source updates the bindable away
from the forced target value. Rust records imported artboard-id snapshots per
referenced view-model instance and uses them only for read-only graph source
resolution. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-artboard-runtime-contract.md`.
At that slice boundary, trigger and list imported-intermediate source kinds
beyond number, boolean, string, color, enum, symbol-list-index, asset, and
artboard, persistent imported instance mutation, remaining property-name APIs,
public object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remained follow-up `#12` slices.

Current #12 update: owned view-model contexts now explicitly pin direct
artboard name-path mutation through an imported replacement intermediate as
unsupported. The C++ probe replaces generated `child` with an imported child,
resolves the owner with `ViewModelInstanceRuntime::propertyViewModel("child")`,
attempts to write the child's `ViewModelInstanceArtboard.propertyValue`, then
binds the owned context; C++ leaves the imported child's existing artboard
source selected. Rust matches by returning `false` for
`set_artboard_by_property_name_path("child/scene", value)` once `child` is
imported and by preserving the read-only imported artboard snapshot. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-artboard-name-path-unsupported-runtime-contract.md`.
Other imported-intermediate value mutation APIs, remaining property-name APIs,
public object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse one imported
replacement intermediate for direct trigger sources. Replacing a generated
root child with an imported child by instance index makes a source path such
as `[root, child, fire]` read the imported child's existing
`ViewModelInstanceTrigger.propertyValue`, matching C++ state-machine
data-context binding as observed through a trigger transition condition that
stays unsatisfied after the imported source updates the bindable away from the
authored target value. Rust records imported trigger-count snapshots per
referenced view-model instance and uses them only for read-only graph source
resolution. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-trigger-runtime-contract.md`.

Current #12 update: owned view-model contexts now explicitly pin direct trigger
name-path mutation through an imported replacement intermediate as
unsupported. The C++ probe replaces generated `child` with an imported child,
resolves the owner with `ViewModelInstanceRuntime::propertyViewModel("child")`,
attempts to write the child's `ViewModelInstanceTrigger.propertyValue`, then
binds the owned context; C++ leaves the imported child's existing trigger
count selected. Rust matches by returning `false` for
`set_trigger_by_property_name_path("child/fire", value)` once `child` is
imported and by preserving the read-only imported trigger snapshot. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-trigger-name-path-unsupported-runtime-contract.md`.
Other imported-intermediate value mutation APIs, remaining property-name APIs,
public object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse one imported
replacement intermediate for direct list sources. Replacing a generated root
child with an imported child by instance index makes a source path such as
`[root, child, items]` read the imported child's existing
`ViewModelInstanceList` item count, matching C++ state-machine data-context
binding as observed through bindable-list reports. Rust records imported list
item-count snapshots per referenced view-model instance and creates a
schema-valid list source placeholder when the default file instance cannot
resolve the path, so the later owned-context bind can hydrate the source. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-list-runtime-contract.md`.
Persistent imported instance mutation, remaining property-name APIs, public
object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now explicitly pin direct list
name-path mutation through an imported replacement intermediate as
unsupported. The C++ probe replaces generated `child` with an imported child,
then attempts to append blank item runtimes through
`ViewModelInstanceRuntime::propertyList("child/items")`; C++ leaves the
imported child's existing list size selected. Rust matches by returning
`false` for `set_list_item_count_by_property_name_path("child/items", count)`
once `child` is imported and by preserving the read-only imported list
snapshot. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-list-name-path-unsupported-runtime-contract.md`.
Other imported-intermediate value mutation APIs, remaining property-name APIs,
public object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now traverse deeper imported
replacement intermediates for read-only view-model pointer sources. Replacing
a generated root child with an imported child by instance index lets a source
path such as `[child, middle, leaf]` read through the imported child's existing
imported middle and the middle's existing imported leaf, matching C++
state-machine data-context binding. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-deep-imported-intermediate-runtime-contract.md`.
Persistent imported instance mutation, remaining property-name APIs, public
object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: owned view-model contexts now explicitly pin mutation
through imported replacement intermediates as unsupported. The C++ probe
replaces the generated root child with an imported child, then attempts
`ViewModelInstanceRuntime::replaceViewModel("child/middle/leaf", value)` from
the owned root; C++ leaves the existing imported leaf selected, and Rust
matches by returning `false` for
`set_view_model_by_property_name_path("child/middle/leaf", value)` once
`child` is imported. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-imported-intermediate-mutation-unsupported-runtime-contract.md`.
Persistent imported instance mutation, remaining property-name APIs, public
object-handle APIs, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: owned generated view-model pointer replacement now has a
property-name path API. `RuntimeOwnedViewModelInstance` records
`ViewModelPropertyViewModel.name` for generated children and exposes
`set_view_model_by_property_name_path("child/middle/leaf", index)`, matching
the C++ `ViewModelInstanceRuntime::replaceViewModel` path shape for this
owned/generated slice while preserving the unsupported imported-intermediate
mutation boundary. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-name-path-runtime-contract.md`.
Persistent imported instance mutation, public object-handle APIs, reverse
propagation, broader update queues, relative/parent lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: imported view-model contexts now preserve the first
persistent view-model pointer relink across rebinding the same imported
context. Rust records a state-machine-local overlay keyed by imported
view-model index, imported instance index, and source path when
`relink_view_model_instance_view_model_source_for_data_bind` updates a
`ViewModelInstanceViewModel` source; a later
`bind_view_model_instance_context` of the same imported instance replays that
overlay, matching C++ `ViewModelInstance::replaceViewModelByProperty`
behavior. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-persistent-relink-runtime-contract.md`.
Sharing imported-instance mutations across independent state-machine instances,
remaining property-name APIs, stable public object handles, reverse
propagation, broader update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: imported view-model contexts now have the first
property-name path relink API for view-model pointer sources. Rust resolves a
slash-separated `ViewModelPropertyViewModel.name` path against the currently
bound imported view-model context, records the relink through the existing
state-machine-local imported overlay, and replays it when the same imported
context is rebound. The C++ probe uses `completeViewModelProperties` before
calling `ViewModelInstanceRuntime::replaceViewModel`, matching the public
runtime name-path API's dependency on completed value-to-property links. The
contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-name-path-runtime-contract.md`.

Current #12 update: imported view-model pointer relinks now have a narrow
shared runtime context. Rust exposes `RuntimeImportedViewModelInstanceContext`
for a file-backed imported view-model instance; relinking a
`ViewModelInstanceViewModel` source through one state machine records the
source-path overlay in that context, and a second state machine bound through
the same context observes the relink. The C++ probe uses two authored state
machines bound to the same imported `ViewModelInstance`, matching C++'s
instance mutation behavior without making `RuntimeFile` mutable. The contract
is
`docs/prototypes/data-binding-graph-imported-viewmodel-shared-relink-runtime-contract.md`.

Current #12 update: imported view-model number sources now have the first
shared scalar mutation path. `RuntimeImportedViewModelInstanceContext` records
number source overrides by resolved data-bind source path; mutating a
`ViewModelInstanceNumber.propertyValue` source through one state machine is
observed when a second authored state machine binds through the same imported
context. The C++ probe adds
`--runtime-set-view-model-instance-source-number` and compares both state
machines through the established number binding report surface. The contract
is
`docs/prototypes/data-binding-graph-imported-viewmodel-number-shared-mutation-runtime-contract.md`.

Current #12 update: imported view-model number sources now have the first
root property-name mutation API. `RuntimeImportedViewModelInstanceContext::
set_number_by_property_name` resolves a root `ViewModelPropertyNumber.name`
against the file-backed imported view model, records the existing number
override by resolved source path, and lets two state machines bound through
the same context observe the mutation. The C++ probe adds
`--runtime-set-view-model-instance-source-number-by-name`, calls
`ViewModelInstanceRuntime::propertyNumber(name)`, and compares both state
machines through the existing number binding report surface. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-number-name-runtime-contract.md`.

Current #12 update: imported view-model number sources now have the first
nested property-name path mutation API. `RuntimeImportedViewModelInstanceContext::
set_number_by_property_name_path` resolves `child/amount` through one
`ViewModelPropertyViewModel` segment to a `ViewModelPropertyNumber` leaf,
records the override by the existing graph source path, and lets two state
machines bound through the same imported context observe the mutation. The C++
probe uses `--runtime-set-view-model-instance-source-number-by-name` with the
slash path after completing view-model properties, matching
`ViewModelInstanceRuntime::propertyNumber("child/amount")`. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-nested-number-name-path-runtime-contract.md`.

Current #12 update: imported view-model number sources now have the first
stable public source handle. `RuntimeImportedViewModelInstanceContext` can
resolve a root or nested number property name into
`RuntimeImportedViewModelNumberSourceHandle`, and
`set_number_by_source_handle` writes through the existing resolved source-path
override only when the handle belongs to the same imported view-model instance
context. The C++ probe compares the handle mutation against
`ViewModelInstanceRuntime::propertyNumber(name)` and verifies both state
machines bound through the same context observe the changed source value. The
contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-number-source-handle-runtime-contract.md`.
Nested property-name paths for boolean/string/color/enum/symbol-list-index/
asset/artboard/trigger/list/view-model sources, stable public handles beyond
the first imported number source handle, reverse propagation, broader update
queues, relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: imported view-model boolean sources now have the nested
property-name path mutation API. `RuntimeImportedViewModelInstanceContext::
set_boolean_by_property_name_path` resolves `child/enabled` through one
`ViewModelPropertyViewModel` segment to a `ViewModelPropertyBoolean` leaf,
records the override by the existing graph source path, and lets two state
machines bound through the same imported context observe the mutation. The C++
probe uses `--runtime-set-view-model-instance-source-bool-by-name` with the
slash path after completing view-model properties, matching
`ViewModelInstanceRuntime::propertyBoolean("child/enabled")`. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-nested-boolean-name-path-runtime-contract.md`.

Current #12 update: imported view-model boolean sources now have a stable
public source handle matching the number handle shape.
`RuntimeImportedViewModelInstanceContext` can resolve a root or nested boolean
property name into `RuntimeImportedViewModelBooleanSourceHandle`, and
`set_boolean_by_source_handle` writes through the existing resolved
source-path override only when the handle belongs to the same imported
view-model instance context. The C++ probe compares the handle mutation against
`ViewModelInstanceRuntime::propertyBoolean(name)` and verifies both state
machines bound through the same context observe the changed source value. The
contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-boolean-source-handle-runtime-contract.md`.
Nested property-name paths for string/color/enum/symbol-list-index/asset/
artboard/trigger/list/view-model sources, stable public handles beyond the
first imported number and boolean source handles, reverse propagation, broader
update queues, relative/parent/nested lookup, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: imported view-model string sources now have the nested
property-name path mutation API. `RuntimeImportedViewModelInstanceContext::
set_string_by_property_name_path` resolves `child/label` through one
`ViewModelPropertyViewModel` segment to a `ViewModelPropertyString` leaf,
records the byte override by the existing graph source path, and lets two
state machines bound through the same imported context observe the mutation.
The C++ probe uses `--runtime-set-view-model-instance-source-string-by-name`
with the slash path after completing view-model properties, matching
`ViewModelInstanceRuntime::propertyString("child/label")`. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-nested-string-name-path-runtime-contract.md`.

Current #12 update: imported view-model string sources now have a stable public
source handle matching the number/boolean handle shape.
`RuntimeImportedViewModelInstanceContext` can resolve a root or nested string
property name into `RuntimeImportedViewModelStringSourceHandle`, and
`set_string_by_source_handle` writes through the existing resolved source-path
override only when the handle belongs to the same imported view-model instance
context. The C++ probe compares the handle mutation against
`ViewModelInstanceRuntime::propertyString(name)` and verifies both state
machines bound through the same context observe the changed source bytes. The
contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-string-source-handle-runtime-contract.md`.
Nested property-name paths for color/enum/symbol-list-index/asset/artboard/
trigger/list/view-model sources, stable public handles beyond the first
imported number, boolean, and string source handles, reverse propagation,
broader update queues, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: imported view-model color sources now have the nested
property-name path mutation API. `RuntimeImportedViewModelInstanceContext::
set_color_by_property_name_path` resolves `child/tint` through one
`ViewModelPropertyViewModel` segment to a `ViewModelPropertyColor` leaf,
records the color override by the existing graph source path, and lets two
state machines bound through the same imported context observe the mutation.
The C++ probe uses `--runtime-set-view-model-instance-source-color-by-name`
with the slash path after completing view-model properties, matching
`ViewModelInstanceRuntime::propertyColor("child/tint")`. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-nested-color-name-path-runtime-contract.md`.

Current #12 update: imported view-model color sources now have a stable public
source handle matching the scalar handle shape.
`RuntimeImportedViewModelInstanceContext` can resolve a root or nested color
property name into `RuntimeImportedViewModelColorSourceHandle`, and
`set_color_by_source_handle` writes through the existing resolved source-path
override only when the handle belongs to the same imported view-model instance
context. The C++ probe compares the handle mutation against
`ViewModelInstanceRuntime::propertyColor(name)` and verifies both state
machines bound through the same context observe the changed color source
value. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-color-source-handle-runtime-contract.md`.
Nested property-name paths for enum/symbol-list-index/asset/artboard/trigger/
list/view-model sources, stable public handles beyond the first imported
number, boolean, string, and color source handles, reverse propagation, broader
update queues, relative/parent/nested lookup, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: imported view-model enum sources now have the nested
property-name path mutation API. `RuntimeImportedViewModelInstanceContext::
set_enum_by_property_name_path` resolves `child/choice` through one
`ViewModelPropertyViewModel` segment to a `ViewModelPropertyEnum*` leaf,
records the enum value-index override by the existing graph source path, and
lets two state machines bound through the same imported context observe the
mutation. The C++ probe uses
`--runtime-set-view-model-instance-source-enum-by-name` with the slash path
after completing view-model properties, matching
`ViewModelInstanceRuntime::propertyEnum("child/choice")`. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-nested-enum-name-path-runtime-contract.md`.

Current #12 update: imported view-model enum sources now have a stable public
source handle for enum value-index mutation. `RuntimeImportedViewModelInstanceContext`
can resolve a root or nested `ViewModelPropertyEnum*` name into
`RuntimeImportedViewModelEnumSourceHandle`, and `set_enum_by_source_handle`
writes through the existing resolved source-path override only when the handle
belongs to the same imported view-model instance context. The C++ probe
compares the handle mutation against
`ViewModelInstanceRuntime::propertyEnum(name)` and verifies both state machines
bound through the same context observe the changed enum source value index.
The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-enum-source-handle-runtime-contract.md`.
Nested property-name paths for symbol-list-index/asset/artboard/trigger/list/
view-model sources, stable public handles beyond the first imported number,
boolean, string, color, and enum source handles, reverse propagation, broader
update queues, relative/parent/nested lookup, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: imported view-model symbol-list-index sources now have the
nested property-name path mutation API. `RuntimeImportedViewModelInstanceContext::
set_symbol_list_index_by_property_name_path` resolves `child/symbol` through
one `ViewModelPropertyViewModel` segment to a
`ViewModelPropertySymbolListIndex` leaf, records the index override by the
existing graph source path, and lets two state machines bound through the same
imported context observe the mutation. The C++ probe uses
`--runtime-set-view-model-instance-source-symbol-list-index-by-name` with the
slash path after completing view-model properties, matching
`ViewModelInstanceRuntime::propertySymbolListIndex("child/symbol")`. The
contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-nested-symbol-list-index-name-path-runtime-contract.md`.

Current #12 update: imported view-model symbol-list-index sources now have a
stable public source handle for symbol value-index mutation.
`RuntimeImportedViewModelInstanceContext` can resolve a root or nested
`ViewModelPropertySymbolListIndex.name` into
`RuntimeImportedViewModelSymbolListIndexSourceHandle`, and
`set_symbol_list_index_by_source_handle` writes through the existing resolved
source-path override only when the handle belongs to the same imported
view-model instance context. The C++ probe compares the handle mutation against
`ViewModelInstanceRuntime::propertySymbolListIndex(name)` and verifies both
state machines bound through the same context observe the changed symbol index
value. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-symbol-list-index-source-handle-runtime-contract.md`.
Nested property-name paths for asset/artboard/trigger/list/view-model sources,
stable public handles beyond the first imported number, boolean, string,
color, enum, and symbol-list-index source handles, reverse propagation,
broader update queues, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: imported view-model asset sources now have the nested
property-name path boundary pinned. C++ probe
`--runtime-set-view-model-instance-source-asset-by-name` uses root
`ViewModelInstance::propertyValue(name)` asset-image lookup, which mutates
`image` but does not resolve slash paths such as `child/image`, even after
completing view-model properties. Rust mirrors that by making
`RuntimeImportedViewModelInstanceContext::set_asset_by_property_name_path`
return `false` for nested slash paths and by keeping the original source value.
The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-nested-asset-name-path-unsupported-runtime-contract.md`.

Current #12 update: imported view-model asset sources now have a root-only
stable public source handle. `RuntimeImportedViewModelInstanceContext` can
resolve a root `ViewModelPropertyAsset*` name into
`RuntimeImportedViewModelAssetSourceHandle`, and
`set_asset_by_source_handle` writes through the existing resolved source-path
override only when the handle belongs to the same imported view-model instance
context. Slash-path handle lookup remains unresolved, matching the nested asset
boundary above. The C++ probe compares the handle mutation against the root
asset by-name path and verifies both state machines bound through the same
context observe the changed asset index. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-asset-source-handle-runtime-contract.md`.
Nested property-name paths for artboard/trigger/list/view-model sources,
stable public handles beyond the first imported number, boolean, string,
color, enum, symbol-list-index, and asset source handles, reverse propagation,
broader update queues, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: imported view-model artboard sources now have the nested
property-name path boundary pinned. C++ probe
`--runtime-set-view-model-instance-source-artboard-by-name` uses root
`ViewModelInstance::propertyValue(name)` artboard lookup, which mutates
`scene` but does not resolve slash paths such as `child/scene`, even after
completing view-model properties. Rust mirrors that by making
`RuntimeImportedViewModelInstanceContext::set_artboard_by_property_name_path`
return `false` for nested slash paths and by keeping the original source
value. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-nested-artboard-name-path-unsupported-runtime-contract.md`.

Current #12 update: imported view-model artboard sources now have a root-only
stable public source handle. `RuntimeImportedViewModelInstanceContext` can
resolve a root `ViewModelPropertyArtboard.name` into
`RuntimeImportedViewModelArtboardSourceHandle`, and
`set_artboard_by_source_handle` writes through the existing resolved
source-path override only when the handle belongs to the same imported
view-model instance context. Slash-path handle lookup remains unresolved,
matching the nested artboard boundary above. The C++ probe compares the handle
mutation against the root artboard by-name path and verifies both state
machines bound through the same context observe the changed artboard index. The
contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-artboard-source-handle-runtime-contract.md`.
Nested property-name paths for trigger/list/view-model sources, stable public
handles beyond the first imported number, boolean, string, color, enum,
symbol-list-index, asset, and artboard source handles, reverse propagation,
broader update queues, relative/parent/nested lookup, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: imported view-model trigger sources now have the nested
property-name path boundary pinned. C++ probe
`--runtime-set-view-model-instance-source-trigger-by-name` uses root
`ViewModelInstance::propertyValue(name)` trigger lookup, which mutates `fire`
but does not resolve slash paths such as `child/fire`, even after completing
view-model properties. Rust mirrors that by making
`RuntimeImportedViewModelInstanceContext::set_trigger_by_property_name_path`
return `false` for nested slash paths and by keeping the original source
value. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-nested-trigger-name-path-unsupported-runtime-contract.md`.

Current #12 update: imported view-model trigger sources now have a root-only
stable public source handle. `RuntimeImportedViewModelInstanceContext` can
resolve a root `ViewModelPropertyTrigger.name` into
`RuntimeImportedViewModelTriggerSourceHandle`, and
`set_trigger_by_source_handle` writes through the existing resolved source-path
override only when the handle belongs to the same imported view-model instance
context. Slash-path handle lookup remains unresolved, matching the nested
trigger boundary above. The C++ probe compares the handle mutation against the
root trigger by-name path and verifies the later-bound state machine advances
with the same admitted post-bind behavior. Trigger binding/source-count report
parity, listener notification, and event dispatch remain out of scope. The
contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-trigger-source-handle-runtime-contract.md`.
Nested property-name paths for list sources, stable public object
handles beyond the first imported number, boolean, string, color, enum,
symbol-list-index, asset, artboard, trigger, list, and view-model source
handles, reverse propagation, broader update queues, relative/parent/nested
lookup, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: imported view-model list sources now have the nested
property-name path boundary pinned. The C++ probe path
`--runtime-set-view-model-instance-source-list-by-name` uses
`ViewModelInstanceRuntime::propertyList(path)`, but the current C++ probe
crashes when asked to mutate the nested slash path `child/items` in the
synthetic imported-context fixture. Rust mirrors the safe boundary by making
`RuntimeImportedViewModelInstanceContext::
set_list_item_count_by_property_name_path` return `false` for nested slash
paths and by keeping the original source item count. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-nested-list-name-path-unsupported-runtime-contract.md`.

Current #12 update: imported view-model list sources now have a root-only
stable public source handle. `RuntimeImportedViewModelInstanceContext` can
resolve a root `ViewModelPropertyList.name` into
`RuntimeImportedViewModelListSourceHandle`, and
`set_list_item_count_by_source_handle` writes through the existing resolved
source-path list item-count override only when the handle belongs to the same
imported view-model instance context. Slash-path handle lookup remains
unresolved, matching the nested list boundary above. The C++ probe compares the
handle mutation against the root list by-name path and verifies the existing
bindable-list source-size reports. Stable list item handles, list item
identity, and list item value mutation remain out of scope. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-list-source-handle-runtime-contract.md`.

Current #12 update: imported view-model pointer sources now have the nested
property-name path relink API. `RuntimeImportedViewModelInstanceContext::
set_view_model_by_property_name_path` resolves `child/grandchild` through
nested `ViewModelPropertyViewModel` names to a view-model pointer leaf,
records the selected referenced instance as the existing imported pointer
override, and lets two state machines bound through the same imported context
observe the relink. The C++ probe uses
`--runtime-relink-view-model-instance-source-viewmodel-by-name-path` with the
slash path after completing view-model properties, matching
`ViewModelInstanceRuntime::replaceViewModel("child/grandchild", value)`. The
contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-nested-viewmodel-name-path-runtime-contract.md`.

Current #12 update: imported view-model pointer sources now have stable public
source handles for both root and slash-separated view-model property paths.
`RuntimeImportedViewModelInstanceContext` can resolve a root
`ViewModelPropertyViewModel.name` or a nested path such as `child/grandchild`
into `RuntimeImportedViewModelViewModelSourceHandle`, and
`set_view_model_by_source_handle` writes through the existing resolved
source-path view-model pointer override only when the handle belongs to the
same imported view-model instance context. The C++ probe compares both root
`current` and nested `child/grandchild` handle relinks against
`--runtime-relink-view-model-instance-source-viewmodel-by-name-path` and
verifies the existing view-model binding reports. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-viewmodel-source-handle-runtime-contract.md`.
Stable public object handles beyond these imported source handles, reverse
propagation, broader update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: imported view-model boolean sources now match the shared
scalar mutation pattern. `RuntimeImportedViewModelInstanceContext` records
boolean source overrides by resolved data-bind source path; mutating a
`ViewModelInstanceBoolean.propertyValue` source through one state machine is
observed when a second authored state machine binds through the same imported
context. The C++ probe adds
`--runtime-set-view-model-instance-source-bool` and compares both state
machines through state-machine advance reports. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-boolean-shared-mutation-runtime-contract.md`.

Current #12 update: imported view-model boolean sources now have the root
property-name mutation API. `RuntimeImportedViewModelInstanceContext::
set_boolean_by_property_name` resolves a root
`ViewModelPropertyBoolean.name` against the file-backed imported view model,
records the existing boolean override by resolved source path, and lets two
state machines bound through the same context observe the mutation. The C++
probe adds `--runtime-set-view-model-instance-source-bool-by-name`, calls
`ViewModelInstanceRuntime::propertyBoolean(name)`, and compares both state
machines through the existing state-machine advance report surface. The
contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-boolean-name-runtime-contract.md`.

Current #12 update: imported view-model string sources now match the shared
scalar mutation pattern. `RuntimeImportedViewModelInstanceContext` records
string source overrides by resolved data-bind source path; mutating a
`ViewModelInstanceString.propertyValue` source through one state machine is
observed when a second authored state machine binds through the same imported
context. The C++ probe adds
`--runtime-set-view-model-instance-source-string` and compares both state
machines through the established string binding report surface. The contract
is
`docs/prototypes/data-binding-graph-imported-viewmodel-string-shared-mutation-runtime-contract.md`.

Current #12 update: imported view-model string sources now have the root
property-name mutation API. `RuntimeImportedViewModelInstanceContext::
set_string_by_property_name` resolves a root `ViewModelPropertyString.name`
against the file-backed imported view model, records the existing string
override by resolved source path, and lets two state machines bound through
the same context observe the mutation. The C++ probe adds
`--runtime-set-view-model-instance-source-string-by-name`, calls
`ViewModelInstanceRuntime::propertyString(name)`, and compares both state
machines through the existing string binding report surface. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-string-name-runtime-contract.md`.

Current #12 update: imported view-model color sources now match the shared
scalar mutation pattern. `RuntimeImportedViewModelInstanceContext` records
color source overrides by resolved data-bind source path; mutating a
`ViewModelInstanceColor.propertyValue` source through one state machine is
observed when a second authored state machine binds through the same imported
context. The C++ probe adds
`--runtime-set-view-model-instance-source-color` plus color binding reports,
and compares both state machines through that report surface. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-color-shared-mutation-runtime-contract.md`.

Current #12 update: imported view-model color sources now have the root
property-name mutation API. `RuntimeImportedViewModelInstanceContext::
set_color_by_property_name` resolves a root `ViewModelPropertyColor.name`
against the file-backed imported view model, records the existing color
override by resolved source path, and lets two state machines bound through
the same context observe the mutation. The C++ probe adds
`--runtime-set-view-model-instance-source-color-by-name`, calls
`ViewModelInstanceRuntime::propertyColor(name)`, and compares both state
machines through the existing color binding report surface. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-color-name-runtime-contract.md`.

Current #12 update: imported view-model enum sources now match the shared
scalar mutation pattern. `RuntimeImportedViewModelInstanceContext` records
enum source overrides by resolved data-bind source path; mutating a
`ViewModelInstanceEnum.propertyValue` source through one state machine is
observed when a second authored state machine binds through the same imported
context. The C++ probe adds
`--runtime-set-view-model-instance-source-enum` plus enum binding reports, and
compares both state machines through that report surface. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-enum-shared-mutation-runtime-contract.md`.

Current #12 update: imported view-model enum sources now have the root
property-name mutation API. `RuntimeImportedViewModelInstanceContext::
set_enum_by_property_name` resolves a root enum view-model property name
against `ViewModelPropertyEnum`, `ViewModelPropertyEnumCustom`, and
`ViewModelPropertyEnumSystem`, records the existing enum override by resolved
source path, and lets two state machines bound through the same context
observe the mutation. The C++ probe adds
`--runtime-set-view-model-instance-source-enum-by-name`, calls
`ViewModelInstanceRuntime::propertyEnum(name)`, and compares both state
machines through the existing enum binding report surface. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-enum-name-runtime-contract.md`.

Current #12 update: imported view-model symbol-list-index sources now match
the shared scalar mutation pattern. `RuntimeImportedViewModelInstanceContext`
records symbol-list-index source overrides by resolved data-bind source path;
mutating a `ViewModelInstanceSymbolListIndex.propertyValue` source through one
state machine is observed when a second authored state machine binds through
the same imported context. The C++ probe adds
`--runtime-set-view-model-instance-source-symbol-list-index` plus
symbol-list-index binding reports for `BindablePropertyInteger` targets, and
compares both state machines through that report surface. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-symbol-list-index-shared-mutation-runtime-contract.md`.

Current #12 update: imported view-model symbol-list-index sources now have the
root property-name mutation API. `RuntimeImportedViewModelInstanceContext::
set_symbol_list_index_by_property_name` resolves a root
`ViewModelPropertySymbolListIndex.name` against the file-backed imported view
model, records the existing symbol-list-index override by resolved source path,
and lets two state machines bound through the same context observe the
mutation. The C++ probe adds
`--runtime-set-view-model-instance-source-symbol-list-index-by-name`, resolves
`ViewModel::property(name)` to a root property index, reads
`ViewModelInstance::propertyValue(index)` as a `ViewModelInstanceSymbolListIndex`,
and compares both state machines through the existing symbol-list-index
binding report surface. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-symbol-list-index-name-runtime-contract.md`.

Current #12 update: imported view-model asset sources now match the shared
scalar mutation pattern. `RuntimeImportedViewModelInstanceContext` records
asset source overrides by resolved data-bind source path; mutating a
`ViewModelInstanceAssetImage.propertyValue` source through one state machine
is observed when a second authored state machine binds through the same
imported context. The C++ probe adds
`--runtime-set-view-model-instance-source-asset` plus asset binding reports for
`BindablePropertyAsset` targets, and compares both state machines through that
report surface. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-asset-shared-mutation-runtime-contract.md`.

Current #12 update: imported view-model asset sources now have the root
property-name mutation API. `RuntimeImportedViewModelInstanceContext::
set_asset_by_property_name` resolves a root `ViewModelPropertyAssetImage` or
`ViewModelPropertyAsset` name against the file-backed imported view model,
records the existing asset override by resolved source path, and lets two
state machines bound through the same context observe the mutation. The C++
probe adds `--runtime-set-view-model-instance-source-asset-by-name`, resolves
the root imported `ViewModelInstanceAssetImage` by name, and compares both
state machines through the existing asset binding report surface. The contract
is
`docs/prototypes/data-binding-graph-imported-viewmodel-asset-name-runtime-contract.md`.

Current #12 update: imported view-model artboard sources now match the shared
scalar mutation pattern. `RuntimeImportedViewModelInstanceContext` records
artboard source overrides by resolved data-bind source path; mutating a
`ViewModelInstanceArtboard.propertyValue` source through one state machine is
observed when a second authored state machine binds through the same imported
context. The C++ probe adds
`--runtime-set-view-model-instance-source-artboard` plus artboard binding
reports for `BindablePropertyArtboard` targets, and compares both state
machines through that report surface. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-artboard-shared-mutation-runtime-contract.md`.

Current #12 update: imported view-model artboard sources now have the root
property-name mutation API. `RuntimeImportedViewModelInstanceContext::
set_artboard_by_property_name` resolves a root `ViewModelPropertyArtboard`
name against the file-backed imported view model, records the existing
artboard override by resolved source path, and lets two state machines bound
through the same context observe the mutation. The C++ probe adds
`--runtime-set-view-model-instance-source-artboard-by-name`, resolves the root
imported `ViewModelInstanceArtboard` by name, and compares both state machines
through the existing artboard binding report surface. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-artboard-name-runtime-contract.md`.

Current #12 update: imported view-model trigger sources now match the shared
imported `propertyValue` mutation pattern before trigger reset.
`RuntimeImportedViewModelInstanceContext` records trigger source overrides by
resolved data-bind source path; mutating a `ViewModelInstanceTrigger` source
through one state machine is observed when a second authored state machine
binds through the same imported context before ordinary advancement can
consume/reset the trigger count. The C++ probe adds
`--runtime-set-view-model-instance-source-trigger` and compares the observing
state machine through its ordinary advance report. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-trigger-shared-mutation-runtime-contract.md`.

Current #12 update: imported view-model trigger sources now have the root
property-name mutation API. `RuntimeImportedViewModelInstanceContext::
set_trigger_by_property_name` resolves a root `ViewModelPropertyTrigger` name
against the file-backed imported view model, records the existing trigger
override by resolved source path, and lets an observing state machine bound
through the same context see the trigger count before ordinary advancement can
consume/reset it. The C++ probe adds
`--runtime-set-view-model-instance-source-trigger-by-name`, resolves the root
imported `ViewModelInstanceTrigger` by name, and compares the observing state
machine through its ordinary advance report. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-trigger-name-runtime-contract.md`.

Current #12 update: imported view-model list sources now match the shared
imported item-count mutation pattern. `RuntimeImportedViewModelInstanceContext`
records list source item-count overrides by resolved data-bind source path;
mutating a `ViewModelInstanceList` source through one state machine is
observed when a second authored state machine binds through the same imported
context. The C++ probe adds
`--runtime-set-view-model-instance-source-list` and compares the observing
state machine through existing `BindablePropertyList` source-size and target
reports. The contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-list-shared-mutation-runtime-contract.md`.

Current #12 update: imported view-model list sources now have the root
property-name mutation API. `RuntimeImportedViewModelInstanceContext::
set_list_item_count_by_property_name` resolves a root `ViewModelPropertyList`
name against the file-backed imported view model, records the existing list
item-count override by resolved source path, and lets an observing state
machine bound through the same context see the list size through existing
`BindablePropertyList` reports. The C++ probe adds
`--runtime-set-view-model-instance-source-list-by-name`, resolves the root
imported list by name, clears and repopulates it with blank items, and compares
the observing state machine through the existing list report surface. The
contract is
`docs/prototypes/data-binding-graph-imported-viewmodel-list-name-runtime-contract.md`.

Current #12 update: default view-model list sources now have the same direct
source mutation seam as the default scalar source family. Mutating a default
`ViewModelInstanceList` source by state-machine data-bind index updates
same-path graph source nodes with the observable list item count, and the C++
probe adds
`--runtime-set-default-view-model-source-list` to compare existing
`BindablePropertyList` source-size and target reports after data-context
advancement. The contract is
`docs/prototypes/data-binding-graph-default-viewmodel-list-source-mutation-runtime-contract.md`.
Imported-instance mutation beyond shared view-model pointer, number, boolean,
string, color, enum, symbol-list-index, asset, artboard, trigger, and list
contexts,
remaining property-name APIs beyond imported view-model pointer and root
number/boolean/string/color/enum/symbol-list-index/asset/artboard/trigger/list sources and owned generated pointer paths,
stable public object handles, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned runtime view-model contexts now have the first scalar
property-name mutation API. `RuntimeOwnedViewModelInstance` records root
`ViewModelProperty.name` values and exposes
`set_number_by_property_name("amount", value)`, matching the C++ public
`ViewModelInstanceRuntime::propertyNumber(name)->value(...)` path used before
binding an owned context to a state machine. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-number-name-runtime-contract.md`.
Remaining owned scalar name APIs, nested scalar name paths,
imported-instance mutation sharing, stable public object handles, reverse
propagation, broader update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime view-model contexts now have the first stable
public source handle. `RuntimeOwnedViewModelInstance` can resolve a root
number property name into `RuntimeOwnedViewModelNumberSourceHandle`, and
`set_number_by_source_handle` writes through the existing owned number
property-index storage before binding the owned context to a state machine.
Root-name handle lookup remains separate from slash-path lookup. The C++ probe
compares the handle mutation against the existing owned-number runtime context
command and verifies the existing state-machine advance and component update
reports. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-number-source-handle-runtime-contract.md`.
Nested number paths are covered separately below. Other nested/relative/parent
lookup, reverse propagation, broader update queues, listener-owned data
binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: owned runtime boolean sources now have a stable public
source handle. `RuntimeOwnedViewModelInstance` can resolve a root boolean
property name into `RuntimeOwnedViewModelBooleanSourceHandle`, and
`set_boolean_by_source_handle` writes through the existing owned boolean
property-index storage before binding the owned context to a state machine.
Slash-path handle lookup remains unresolved. The C++ probe compares the handle
mutation against the existing owned-boolean runtime context command and
verifies the existing state-machine advance and component update reports. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-boolean-source-handle-runtime-contract.md`.
Nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime string sources now have a stable public
source handle. `RuntimeOwnedViewModelInstance` can resolve a root string
property name into `RuntimeOwnedViewModelStringSourceHandle`, and
`set_string_by_source_handle` writes through the existing owned raw string
storage before binding the owned context to a state machine. Slash-path handle
lookup remains unresolved. The C++ probe compares the handle mutation against
the existing owned-string runtime context command and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-string-source-handle-runtime-contract.md`.
Nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime color sources now have a stable public source
handle. `RuntimeOwnedViewModelInstance` can resolve a root color property name
into `RuntimeOwnedViewModelColorSourceHandle`, and
`set_color_by_source_handle` writes through the existing owned color
property-index storage before binding the owned context to a state machine.
Slash-path handle lookup remains unresolved. The C++ probe compares the handle
mutation against the existing owned-color runtime context command and verifies
the existing state-machine advance and component update reports. The contract
is
`docs/prototypes/data-binding-graph-owned-viewmodel-color-source-handle-runtime-contract.md`.
Nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime enum sources now have a stable public source
handle. `RuntimeOwnedViewModelInstance` can resolve a root enum property name
into `RuntimeOwnedViewModelEnumSourceHandle`, and
`set_enum_by_source_handle` writes through the existing owned enum value-index
storage before binding the owned context to a state machine. Slash-path handle
lookup remains unresolved. The C++ probe compares the handle mutation against
the existing owned-enum runtime context command and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-enum-source-handle-runtime-contract.md`.
Nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime symbol-list-index sources now have a stable
public source handle. `RuntimeOwnedViewModelInstance` can resolve a root
symbol-list-index property name into
`RuntimeOwnedViewModelSymbolListIndexSourceHandle`, and
`set_symbol_list_index_by_source_handle` writes through the existing owned
symbol-list-index storage before binding the owned context to a state machine.
Slash-path handle lookup remains unresolved. The C++ probe compares the handle
mutation against the existing owned-symbol-list-index runtime context command
and verifies the existing state-machine advance and component update reports.
The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-symbol-list-index-source-handle-runtime-contract.md`.
Nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime asset sources now have a stable public source
handle. `RuntimeOwnedViewModelInstance` can resolve a root asset property name
into `RuntimeOwnedViewModelAssetSourceHandle`, and
`set_asset_by_source_handle` writes through the existing owned raw asset id
storage before binding the owned context to a state machine. Slash-path handle
lookup remains unresolved. The C++ probe compares the handle mutation against
the existing owned-asset runtime context command and verifies the existing
state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-asset-source-handle-runtime-contract.md`.
Nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime artboard sources now have a stable public
source handle. `RuntimeOwnedViewModelInstance` can resolve a root artboard
property name into `RuntimeOwnedViewModelArtboardSourceHandle`, and
`set_artboard_by_source_handle` writes through the existing owned raw artboard
id storage before binding the owned context to a state machine. Slash-path
handle lookup remains unresolved. The C++ probe compares the handle mutation
against the existing owned-artboard runtime context command and verifies the
existing state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-artboard-source-handle-runtime-contract.md`.
Nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime trigger sources now have a stable public
source handle. `RuntimeOwnedViewModelInstance` can resolve a root trigger
property name into `RuntimeOwnedViewModelTriggerSourceHandle`, and
`set_trigger_by_source_handle` writes through the existing owned raw trigger
count storage before binding the owned context to a state machine. Slash-path
handle lookup remains unresolved. The C++ probe compares the handle mutation
against the existing owned-trigger runtime context command and verifies the
existing state-machine advance and component update reports. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-trigger-source-handle-runtime-contract.md`.
Nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime list sources now have a stable public source
handle. `RuntimeOwnedViewModelInstance` can resolve a root list property name
into `RuntimeOwnedViewModelListSourceHandle`, and
`set_list_item_count_by_source_handle` writes through the existing owned list
item-count storage before binding the owned context to a state machine.
Slash-path handle lookup remains unresolved. The C++ probe compares the handle
mutation against the existing owned-list runtime context command and verifies
the existing state-machine advance and list binding reports. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-list-source-handle-runtime-contract.md`.
Nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime view-model pointer sources now have a stable
public source handle. `RuntimeOwnedViewModelInstance` can resolve a root
view-model pointer property name into
`RuntimeOwnedViewModelViewModelSourceHandle`, and
`set_view_model_by_source_handle` writes through the existing owned
view-model pointer storage before binding the owned context to a state
machine. Slash-path handle lookup remains unresolved. The C++ probe compares
the handle relink against the existing owned view-model pointer runtime
context command and verifies the existing state-machine advance and component
update reports. The owned runtime root source-handle family is now covered for
number, boolean, string, color, enum, symbol-list-index, asset, artboard,
trigger, list, and view-model pointer sources. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-viewmodel-source-handle-runtime-contract.md`.
Nested/relative/parent lookup, reverse propagation, broader update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned runtime view-model root scalar property-name mutation
now covers every scalar kind already backed by owned property-index storage:
number, boolean, string, color, enum, symbol-list-index, asset, artboard, and
trigger. The existing owned scalar C++ parity probes still drive C++ through
`ViewModelProperty.name` and now drive Rust through the matching
`RuntimeOwnedViewModelInstance::set_*_by_property_name` APIs. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-root-scalar-name-runtime-contract.md`.
Nested scalar name paths, imported-instance mutation sharing, stable public
object handles, reverse propagation, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: owned root view-model list sources now have explicit
root-name parity coverage. `RuntimeOwnedViewModelInstance::
set_list_item_count_by_property_name_path("items", count)` uses the existing
single-segment list name path to mutate a root `ViewModelPropertyList` item
count before binding; the C++ probe drives
`ViewModelInstanceRuntime::propertyList("items")` through
`--runtime-bind-owned-view-model-list-name-path-state-machine-context` and
compares the existing `BindablePropertyList` source-size reports. The contract
is
`docs/prototypes/data-binding-graph-owned-viewmodel-root-list-name-runtime-contract.md`.
List item identity, generated item instances, list item handles, reverse
propagation, broader update queues, relative/parent/nested lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned generated nested number paths now have the first
nested scalar name-path mutation API. Rust stores direct number values on
generated owned view-model children, exposes
`set_number_by_property_name_path("child/amount", value)`, and resolves number
data-bind paths longer than the root scalar shape when binding an owned
context. The C++ probe uses
`ViewModelInstanceRuntime::propertyNumber("child/amount")->value(...)`. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-number-name-path-runtime-contract.md`.

Current #12 update: owned generated nested number paths now have the first
stable public nested source handle. `RuntimeOwnedViewModelInstance` can
resolve `child/amount` into `RuntimeOwnedViewModelNumberSourceHandle` through
`number_source_handle_by_property_name_path`, and
`set_number_by_source_handle` writes through the same generated-child number
storage before binding. The C++ probe compares the handle mutation against
`ViewModelInstanceRuntime::propertyNumber("child/amount")->value(...)` and
verifies the existing state-machine advance, number binding, and component
update reports. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-number-source-handle-runtime-contract.md`.
Other nested scalar source handles beyond number, imported-intermediate nested
scalar paths, imported-instance mutation sharing, stable public object handles,
reverse propagation, broader update queues, relative/parent lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned generated nested boolean paths now match the nested
number pattern. Rust stores direct boolean values on generated owned
view-model children, exposes
`set_boolean_by_property_name_path("child/enabled", value)`, and resolves
boolean data-bind paths longer than the root scalar shape when binding an
owned context. The C++ probe uses
`ViewModelInstanceRuntime::propertyBoolean("child/enabled")->value(...)`. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-boolean-name-path-runtime-contract.md`.

Current #12 update: owned generated nested boolean paths now have a stable
public nested source handle. `RuntimeOwnedViewModelInstance` can resolve
`child/enabled` into `RuntimeOwnedViewModelBooleanSourceHandle` through
`boolean_source_handle_by_property_name_path`, and
`set_boolean_by_source_handle` writes through the same generated-child boolean
storage before binding. The C++ probe compares the handle mutation against
`ViewModelInstanceRuntime::propertyBoolean("child/enabled")->value(...)` and
verifies the existing state-machine advance and component update reports. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-boolean-source-handle-runtime-contract.md`.
Other nested scalar source handles beyond number/boolean, imported-intermediate
nested scalar paths, imported-instance mutation sharing, stable public object
handles, reverse propagation, broader update queues, relative/parent lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned generated nested string paths now match the
number/boolean nested pattern. Rust stores direct string values on generated
owned view-model children, exposes
`set_string_by_property_name_path("child/label", value)`, and resolves string
data-bind paths longer than the root scalar shape when binding an owned
context. The C++ probe uses
`ViewModelInstanceRuntime::propertyString("child/label")->value(...)`. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-string-name-path-runtime-contract.md`.

Current #12 update: owned generated nested string paths now have a stable
public nested source handle. `RuntimeOwnedViewModelInstance` can resolve
`child/label` into `RuntimeOwnedViewModelStringSourceHandle` through
`string_source_handle_by_property_name_path`, and
`set_string_by_source_handle` writes through the same generated-child raw
string storage before binding. The C++ probe compares the handle mutation
against `ViewModelInstanceRuntime::propertyString("child/label")->value(...)`
and verifies the existing state-machine advance and component update reports.
The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-string-source-handle-runtime-contract.md`.
Other nested scalar source handles beyond number/boolean/string,
imported-intermediate nested scalar paths, imported-instance mutation sharing,
stable public object handles, reverse propagation, broader update queues,
relative/parent lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: owned generated nested color paths now match the
number/boolean/string nested pattern. Rust stores direct color values on
generated owned view-model children, exposes
`set_color_by_property_name_path("child/tint", value)`, and resolves color
data-bind paths longer than the root scalar shape when binding an owned
context. The C++ probe uses
`ViewModelInstanceRuntime::propertyColor("child/tint")->value(...)`. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-color-name-path-runtime-contract.md`.

Current #12 update: owned generated nested color paths now have a stable
public nested source handle. `RuntimeOwnedViewModelInstance` can resolve
`child/tint` into `RuntimeOwnedViewModelColorSourceHandle` through
`color_source_handle_by_property_name_path`, and
`set_color_by_source_handle` writes through the same generated-child color
storage before binding. The C++ probe compares the handle mutation against
`ViewModelInstanceRuntime::propertyColor("child/tint")->value(...)` and
verifies the existing state-machine advance and component update reports. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-color-source-handle-runtime-contract.md`.
Other nested scalar source handles beyond number/boolean/string/color,
imported-intermediate nested scalar paths, imported-instance mutation sharing,
stable public object handles, reverse propagation, broader update queues,
relative/parent lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: owned generated nested enum paths now match the
number/boolean/string/color nested pattern. Rust stores direct enum value
indexes on generated owned view-model children, exposes
`set_enum_by_property_name_path("child/choice", value)`, and resolves enum
data-bind paths longer than the root scalar shape when binding an owned
context. The C++ probe uses
`ViewModelInstanceRuntime::propertyEnum("child/choice")->valueIndex(...)`. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-enum-name-path-runtime-contract.md`.

Current #12 update: owned generated nested enum paths now have a stable public
nested source handle. `RuntimeOwnedViewModelInstance` can resolve
`child/choice` into `RuntimeOwnedViewModelEnumSourceHandle` through
`enum_source_handle_by_property_name_path`, and
`set_enum_by_source_handle` writes through the same generated-child enum
value-index storage before binding. The C++ probe compares the handle mutation
against
`ViewModelInstanceRuntime::propertyEnum("child/choice")->valueIndex(...)` and
verifies the existing state-machine advance and component update reports. The
contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-enum-source-handle-runtime-contract.md`.
Other nested scalar source handles beyond number/boolean/string/color/enum,
imported-intermediate nested scalar paths, imported-instance mutation sharing,
stable public object handles, reverse propagation, broader update queues,
relative/parent lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: owned generated nested symbol-list-index paths now match
the other generated nested scalar paths. Rust stores direct
symbol-list-index values on generated owned view-model children, exposes
`set_symbol_list_index_by_property_name_path("child/symbol", value)`, and
resolves symbol-list-index data-bind paths longer than the root scalar shape
when binding an owned context. The C++ probe resolves the parent view-model
path with `ViewModelInstanceRuntime::propertyViewModel("child")` and mutates
the child's `ViewModelInstanceSymbolListIndex` through `propertyValue`.
The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-symbol-list-index-name-path-runtime-contract.md`.

Current #12 update: owned generated nested symbol-list-index paths now have a
stable public nested source handle. `RuntimeOwnedViewModelInstance` can resolve
`child/symbol` into `RuntimeOwnedViewModelSymbolListIndexSourceHandle` through
`symbol_list_index_source_handle_by_property_name_path`, and
`set_symbol_list_index_by_source_handle` writes through the same
generated-child symbol-list-index storage before binding. The C++ probe
compares the handle mutation against the existing owned symbol-list-index
name-path command. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-symbol-list-index-source-handle-runtime-contract.md`.
Other nested scalar source handles beyond
number/boolean/string/color/enum/symbol-list-index, imported-intermediate
nested scalar paths, imported-instance mutation sharing, stable public object
handles, reverse propagation, broader update queues, relative/parent lookup,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: owned generated nested asset paths now match the other
generated nested scalar paths. Rust stores direct asset IDs on generated owned
view-model children, exposes
`set_asset_by_property_name_path("child/image", value)`, and resolves asset
data-bind paths longer than the root scalar shape when binding an owned
context. The C++ probe resolves the parent view-model path with
`ViewModelInstanceRuntime::propertyViewModel("child")` and mutates the child's
`ViewModelInstanceAssetImage` through `propertyValue`. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-asset-name-path-runtime-contract.md`.

Current #12 update: owned generated nested asset paths now have a stable
public nested source handle. `RuntimeOwnedViewModelInstance` can resolve
`child/image` into `RuntimeOwnedViewModelAssetSourceHandle` through
`asset_source_handle_by_property_name_path`, and
`set_asset_by_source_handle` writes through the same generated-child asset
storage before binding. The C++ probe compares the handle mutation against the
existing owned asset name-path command. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-asset-source-handle-runtime-contract.md`.
Nested object source handles beyond asset, imported-intermediate nested scalar
paths, imported-instance mutation sharing, reverse propagation, broader update
queues, relative/parent lookup, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: owned generated nested artboard paths now match the other
generated nested scalar paths. Rust stores direct artboard IDs on generated
owned view-model children, exposes
`set_artboard_by_property_name_path("child/scene", value)`, and resolves
artboard data-bind paths longer than the root scalar shape when binding an
owned context. The C++ probe resolves the parent view-model path with
`ViewModelInstanceRuntime::propertyViewModel("child")` and mutates the child's
`ViewModelInstanceArtboard` through `propertyValue`. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-artboard-name-path-runtime-contract.md`.

Current #12 update: owned generated nested artboard paths now have a stable
public nested source handle. `RuntimeOwnedViewModelInstance` can resolve
`child/scene` into `RuntimeOwnedViewModelArtboardSourceHandle` through
`artboard_source_handle_by_property_name_path`, and
`set_artboard_by_source_handle` writes through the same generated-child
artboard storage before binding. The C++ probe compares the handle mutation
against the existing owned artboard name-path command. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-artboard-source-handle-runtime-contract.md`.
Nested object source handles beyond asset/artboard, imported-intermediate
nested scalar paths, imported-instance mutation sharing, reverse propagation,
broader update queues, relative/parent lookup, listener-owned data binding,
and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: owned generated nested trigger paths now match the other
generated nested scalar paths for raw `propertyValue` binding. Rust stores
direct trigger values on generated owned view-model children, exposes
`set_trigger_by_property_name_path("child/fire", value)`, and resolves trigger
data-bind paths longer than the root scalar shape when binding an owned
context. The C++ probe resolves the parent view-model path with
`ViewModelInstanceRuntime::propertyViewModel("child")` and mutates the child's
`ViewModelInstanceTrigger` through `propertyValue`. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-trigger-name-path-runtime-contract.md`.

Current #12 update: owned generated nested trigger paths now have a stable
public nested source handle. `RuntimeOwnedViewModelInstance` can resolve
`child/fire` into `RuntimeOwnedViewModelTriggerSourceHandle` through
`trigger_source_handle_by_property_name_path`, and
`set_trigger_by_source_handle` writes through the same generated-child trigger
storage before binding. The C++ probe compares the handle mutation against the
existing owned trigger name-path command. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-trigger-source-handle-runtime-contract.md`.
Trigger firing APIs, listener/callback dispatch, nested list item identity,
view-model value paths, imported-intermediate nested scalar paths,
imported-instance mutation sharing, stable public object handles beyond
asset/artboard/trigger, reverse propagation, broader update queues,
relative/parent lookup, and nested artboard propagation remain follow-up `#12`
slices.

Current #12 update: owned generated nested list paths now cover the direct
list source fact needed by bindable-list parity. Rust stores only item counts
on generated owned view-model children, exposes
`set_list_item_count_by_property_name_path("child/items", count)`, and
resolves `RuntimeDataBindGraphValue::List` source paths longer than the root
shape when binding an owned context. The C++ probe resolves
`ViewModelInstanceRuntime::propertyList("child/items")`, adds blank list item
instances to set the observed size without creating item identity cycles, then
compares the same bindable-list report surface. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-list-name-path-runtime-contract.md`.

Current #12 update: owned generated nested list paths now have a stable public
nested source handle for item-count mutation. `RuntimeOwnedViewModelInstance`
can resolve `child/items` into `RuntimeOwnedViewModelListSourceHandle` through
`list_source_handle_by_property_name_path`, and
`set_list_item_count_by_source_handle` writes through the same generated-child
list item-count storage before binding. The C++ probe compares the handle
mutation against the existing owned list name-path command. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-list-source-handle-runtime-contract.md`.

Current #12 update: owned generated nested view-model pointer paths now have a
stable public nested source handle. `RuntimeOwnedViewModelInstance` can resolve
`child/middle/leaf` into `RuntimeOwnedViewModelViewModelSourceHandle` through
`view_model_source_handle_by_property_name_path`, and
`set_view_model_by_source_handle` relinks the same generated-child pointer
storage before binding. The C++ probe compares the handle relink against the
existing owned generated view-model name-path command. The contract is
`docs/prototypes/data-binding-graph-owned-viewmodel-nested-viewmodel-source-handle-runtime-contract.md`.
List item identity, item-level view-model traversal, imported-intermediate
nested scalar paths, imported-instance mutation sharing, stable public object
handles beyond asset/artboard/trigger/list/view-model, reverse propagation,
broader update queues, relative/parent lookup, and nested artboard propagation
remain follow-up `#12` slices.

Current #12 update: state-machine `BindablePropertyList.propertyValue`
targets now have a probe-backed target-to-source boundary. Rust exposes
`StateMachineInstance::set_bindable_list_for_data_bind`, tracks list target
dirty state in `RuntimeDataBindGraph`, and consumes explicit
`advancedDataContext()` plus public `updateDataBinds(true)` as a C++-compatible
no-op for direct `ViewModelInstanceList` sources and
`DataConverterNumberToList` sources: the edited target scalar is preserved,
the source list size or source number report stays unchanged, and no generated
list item runtime instances are admitted. The contract is
`docs/prototypes/data-binding-graph-bindable-list-target-to-source-runtime-contract.md`.
Artboard component-list item instancing, generated child identity propagation,
map-rule selection, list layout/virtualization, generated-list reverse
converters that materialize item identity, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: direct `DataConverterNumberToList` now also has an
explicit main-`ToSource | TwoWay` target-to-source boundary for
state-machine `BindablePropertyList.propertyValue` targets. C++ consumes the
edited bindable-list target during `advancedDataContext()` without writing the
edited scalar into the numeric source and without admitting generated list item
runtime instances. The contract is
`docs/prototypes/data-binding-graph-number-to-list-main-to-source-target-to-source-runtime-contract.md`.
Artboard component-list item instancing, generated child identity propagation,
map-rule selection, list layout/virtualization, generated-list reverse
converters that materialize item identity, broader update queues,
relative/parent/nested lookup, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: state-machine `DataBindContext` name-based source paths now
have an explicit unsupported parity boundary. C++ clones state-machine data
binds with a null `DataBind::file()` pointer, so a `NameBased`
`DataBindContext.sourcePathIds` buffer does not expand through the file
manifest during runtime binding even when the manifest contains the matching
name path. Rust keeps the source unresolved for this shape while still
reporting the cloned bindable number target's initial value. The contract is
`docs/prototypes/data-binding-graph-name-based-state-machine-source-path-unsupported-runtime-contract.md`.

Current #12 update: artboard-owned `NameBased` source paths targeting
`ArtboardComponentList` now have an explicit runtime unsupported boundary.
The C++ probe binds the default artboard view-model context against a manifest
path for `items` and reports the component-list target row plus an empty target
list size, but no resolved source list. Direct post-bind
`Artboard::updateDataBinds(true)` and post-bind `Artboard::advance(0.0f)`
preserve the same unresolved source and empty target-list facts. Rust keeps
the same target row and unresolved source facts across all three entrypoints.
The contract is
`docs/prototypes/data-binding-graph-artboard-name-based-source-path-unsupported-runtime-contract.md`.

Current #12 update: file-backed `DataContext` lookup facts now have a complete
read-only runtime report. `runtime_data_context_lookup_reports`
enumerates imported view models, instances, and explicit instance values in
C++ order, resolves absolute `viewModelId`/`viewModelPropertyId` paths and
manifest-name relative paths through the existing `rive-binary` lookup helpers,
reports `ViewModelInstance::propertyFromPath`, and emits the C++ probe's first
absolute and manifest-relative parent fallback property lookups. The report has
full `--data-context-lookups` C++ probe parity for a nested view-model fixture.
The contract is
`docs/prototypes/data-context-file-backed-lookup-runtime-contract.md`.
Live data-bind wiring for those relative/name paths, converter name paths,
live parent paths, runtime mutation/relink through these reports, listener-owned
data binding, and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: file-backed `DataContext` lookup facts now also have a
graph-callable runtime resolver. `RuntimeDataContext` wraps a borrowed
`RuntimeFile`, a current imported `ViewModelInstance`, and optional parent
contexts, and delegates absolute, manifest-relative, parent-chain, and
`ViewModelInstance::propertyFromPath` lookups to the existing C++-audited
`rive-binary` helpers. The existing full `--data-context-lookups` comparison
now flows through this resolver, and a direct Rust test pins the positive
absolute root lookup facts plus the unresolved relative/nested/parent
boundaries exposed by the current fixture. The contract is
`docs/prototypes/data-context-runtime-lookup-support-contract.md`.
Live data-bind wiring for relative/name paths, converter name paths, runtime
mutation/relink through data contexts, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

Current #12 update: imported state-machine source binding now routes its
already-admitted absolute source lookup through `RuntimeDataContext`.
`bind_imported_view_model_context` uses the same context wrapper for source
values and operation-view-model operands, while preserving the existing
override-map application after lookup. The contract is
`docs/prototypes/data-binding-graph-imported-source-runtime-data-context-wiring-contract.md`.
Relative/name/parent path admission, listener-owned data contexts, nested
artboard propagation, broader dirty/update queues, and reverse propagation
remain follow-up `#12` slices.

Current #12 update: direct `DataConverterOperationViewModel` secondary operand
resolution is now the first graph-owned converter path to consume
`RuntimeDataContext`. The converter still reads its `sourcePathIds` as an
absolute `viewModelId`/`viewModelPropertyId` path against the default
view-model instance, and missing, non-number, and manifest-name path cases
preserve the existing C++ `0.0` operand fallback. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-runtime-data-context-operand-contract.md`.
Live relative/name converter paths, broader dirty/update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: `DataConverterOperationViewModel` non-number secondary
operands now have explicit direct and grouped default-context fallback
coverage. C++ `bindFromContext` only stores `ViewModelInstanceNumber`
secondary sources, so a `ViewModelInstanceSymbolListIndex` operand path uses
the `0.0` fallback both directly and inside
`DataConverterGroup<OperationValue, OperationViewModel>`. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-non-number-operand-runtime-contract.md`.
Imported/owned runtime-context fallback is covered by the later
operation-viewmodel non-number context recompute slice. Relative/name
converter paths, broader dirty/update queues, listener-owned data binding, and
nested artboard propagation remain follow-up `#12` slices.

Current #12 update: `DataConverterOperationViewModel` non-number secondary
operands now also have imported and owned runtime-context fallback coverage
for direct converters and for
`DataConverterGroup<OperationValue, OperationViewModel>`. The C++ probe uses
an additive operation-viewmodel converter so a mistaken symbol-list-index
numeric operand would be observable, while C++ keeps the `0.0` operand
fallback because the resolved secondary source is not a
`ViewModelInstanceNumber`. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-non-number-context-recompute-runtime-contract.md`.
Imported symbol-list-index source mutation is covered by the later
operation-viewmodel imported symbol mutation slice. Relative/name converter
paths, owned-context source mutation APIs, broader dirty/update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: direct `DataConverterOperationViewModel` operands now
recompute for imported and owned runtime view-model contexts. Imported context
binding resolves the secondary number operand through the bound
`RuntimeDataContext`; owned context binding resolves it from owned runtime
view-model storage; default-context rebinding restores the stored default
operand. Missing, non-number, and manifest-name converter paths keep the
existing C++ `0.0` fallback. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-context-recompute-runtime-contract.md`.
Relative/name converter paths, imported/owned recomputation for other
converter families, broader dirty/update queues, listener-owned data binding,
and nested artboard propagation remain follow-up `#12` slices.

Current #12 update: grouped `DataConverterOperationViewModel` operands now
have explicit imported and owned context recompute coverage for the
`DataConverterGroup<OperationValue, OperationViewModel>` path. Imported
context binding resolves the nested secondary number operand from the bound
view-model instance, and owned context binding resolves it from owned runtime
view-model storage. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-group-context-recompute-runtime-contract.md`.
Reverse two-item `OperationViewModel, OperationValue` order is covered by the
later group reverse-order coverage slice. Longer/exotic converter group
permutations, relative/name converter paths, broader dirty/update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: grouped `DataConverterOperationViewModel` operands now
also have explicit default-context rebind coverage after imported and owned
context recompute. The recursive reset restores the stored default operand for
`DataConverterGroup<OperationValue, OperationViewModel>` after a non-default
runtime context changed the nested secondary number operand. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-group-default-rebind-runtime-contract.md`.
Reverse two-item `OperationViewModel, OperationValue` order is covered by the
later group reverse-order coverage slice. Longer/exotic converter group
permutations, relative/name converter paths, broader dirty/update queues,
listener-owned data binding, and nested artboard propagation remain follow-up
`#12` slices.

Current #12 update: grouped `DataConverterOperationViewModel` operands now
have the first observable non-default group-order coverage. A C++ probe uses
`DataConverterGroup<OperationViewModel, OperationValue>` with mixed
multiply/subtract operations so group order affects the converted number, then
binds an imported runtime view-model context to refresh the
operation-viewmodel operand before the ordered group runs. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-group-order-runtime-contract.md`.
Default, owned, and imported-mutation coverage for the same reverse two-item
order is covered by the later group reverse-order coverage slice. Longer and
exotic converter group permutations, relative/name converter paths, broader
dirty/update queues, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: grouped `DataConverterOperationViewModel` reverse-order
coverage now includes default context binding, owned context binding, and
imported secondary-number source mutation for the two-item
`DataConverterGroup<OperationViewModel, OperationValue>` path. The additive
operation-viewmodel plus multiplying operation-value fixture makes the order
observable in all three contexts. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-group-reverse-order-coverage-runtime-contract.md`.
Longer and exotic converter group permutations, relative/name converter paths,
broader dirty/update queues, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: imported runtime number source mutation now refreshes
direct `DataConverterOperationViewModel` secondary operands when the mutated
source path matches the converter operand path. This keeps a bound imported
`factor` source and the dependent converted `amount` source in sync while
leaving the stored default operand intact for later default-context rebinding.
The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-imported-number-mutation-runtime-contract.md`.
Relative/name converter paths, owned-context source mutation APIs,
mutation-driven recompute for other converter families, broader dirty/update
queues, listener-owned data binding, and nested artboard propagation remain
follow-up `#12` slices.

Current #12 update: imported runtime number source mutation now also refreshes
`DataConverterOperationViewModel` secondary operands nested inside the grouped
`DataConverterGroup<OperationValue, OperationViewModel>` path. The grouped
converted `amount` source, the direct `amount` source, and the direct `factor`
source all match C++ after mutating the bound imported `factor` source. The
contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-group-imported-number-mutation-runtime-contract.md`.
Longer and exotic converter group permutations, relative/name converter paths,
broader dirty/update queues, listener-owned data binding, and nested artboard
propagation remain follow-up `#12` slices.

Current #12 update: imported runtime symbol-list-index source mutation now
preserves the `DataConverterOperationViewModel` non-number secondary operand
fallback for direct converters and for
`DataConverterGroup<OperationValue, OperationViewModel>`. The fixture includes
a separate symbol-list-index source bind so the mutation targets the converter
operand path; C++ updates that ordinary source bind while keeping the
operation-viewmodel operand at the `0.0` fallback. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-imported-symbol-mutation-runtime-contract.md`.

Current #12 update: owned runtime number source mutation now refreshes
`DataConverterOperationViewModel` secondary operands for direct converters and
for `DataConverterGroup<OperationValue, OperationViewModel>` after the owned
context is already bound. Rust exposes
`StateMachineInstance::set_owned_view_model_context_number_source_for_data_bind`,
and the C++ probe retains the active owned number context for
`--runtime-set-owned-view-model-source-number`. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-owned-number-mutation-runtime-contract.md`.

Current #12 update: owned runtime symbol-list-index source mutation now
preserves the `DataConverterOperationViewModel` non-number secondary operand
fallback for direct converters and for
`DataConverterGroup<OperationValue, OperationViewModel>`. Rust exposes
`StateMachineInstance::set_owned_view_model_context_symbol_list_index_source_for_data_bind`,
and the C++ probe retains active owned contexts for
`--runtime-set-owned-view-model-source-symbol-list-index`. The contract is
`docs/prototypes/data-binding-graph-operation-viewmodel-owned-symbol-mutation-runtime-contract.md`.
Longer and exotic converter group permutations, relative/name converter paths,
owned-context source mutation APIs beyond root number and symbol-list-index
OperationViewModel operands, mutation-driven recompute for other converter
families, broader dirty/update queues, listener-owned data binding, and nested
artboard propagation remain follow-up `#12` slices.

## #13: Nested Artboards And Hosts

Blocked by: #9, #12
Type: Prototype

### Question

What nested artboard and host behavior must exist before runtime parity is credible?

### Answer

Open. Support nested artboard instance ownership, host transforms, data contexts, view model binding, hit testing hooks, focusable behavior, and advancement of nested animations/state machines.

## #14: Layout, Text, Scripting, Audio Scope

Blocked by: #10, #12, #13
Type: Research

### Question

Which optional runtime systems should be native Rust, feature-gated wrappers, or deferred?

### Answer

Open. Research C++ feature flags and dependency boundaries for Yoga layout, text shaping/bidi, scripting, and audio. Decide which can be deferred, which need compatibility shims, and which should be reimplemented in Rust.

## #15: Public API And FFI Surface

Blocked by: #11, #12, #13
Type: Discuss

### Question

What public Rust and C-compatible API should the runtime expose?

### Answer

Open. Define the ergonomic Rust API first, then the FFI layer. The public surface should probably expose file loading, artboard instancing, animation/state-machine control, inputs/events, advancement, draw command extraction, and asset hooks.

## #16: Renderer Strategy

Blocked by: #10, #15
Type: Research

### Question

Should Rust eventually own rendering, emit draw commands to platform renderers, or bridge into the existing C++ renderer?

### Answer

Open. Defer until the headless graph and draw command stream are stable. The near-term renderer strategy is renderer-independent command output. A temporary C++ bridge may be useful for parity tests, but should not shape the core graph design.
