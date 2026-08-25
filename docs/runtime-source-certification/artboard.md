# Artboard-family literal source certification

Pinned upstream: `rive-runtime@4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

This receipt is a fresh, complete read of these translation units and their
headers; it does not inherit the verdicts in the B6 or phase-3 audit records:

- `src/artboard.cpp`
- `src/nested_artboard.cpp`
- `src/artboard_component_list.cpp`
- `src/nested_artboard_layout.cpp`
- `src/nested_artboard_leaf.cpp`

## Independent adversarial review: REJECTED

Commit `eb4b0c23d` is **not accepted as a complete literal certification**.
The mounted-size arithmetic added by that commit matches the pinned
`NestedArtboard::measureLayout` body, and the nested/list destruction order and
reset traversal survived review. The rejected receipt nevertheless had an
incomplete source denominator, its focused measurement test bypassed the
production registration path, and a fresh-clone default differed observably
from pinned C++. The denominator defect has since been corrected campaign-wide;
the two behavioral findings were corrected by `d3df628c7` and accepted by the
independent review recorded below.

## Independent re-review of `d3df628c7`: corrections accepted

The first independent adversarial review accepts both corrections in
`d3df628c7`. The review re-read pinned commit
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`, traced the complete public and
transient clone paths, inspected every production `clone_for_transient_layout`
call site, followed ordinary nested measurement through style registration and
the Taffy callback, and reran the focused evidence against the exact reviewed
commit. No production change was needed. This acceptance is deliberately
narrow: the Artboard family remains uncertified because the static frame
counter and cross-owner decoder findings below are still open.

### Exact denominator

The corrected campaign denominator (`4144a92c5`) contains 1,105 authority
owners and 7,818 authority units. This Artboard-family slice contains 395
units: 301 translation-unit units and 94 handwritten-header units. Those are
389 function definitions, the `Artboard::sm_frameId` static source statement,
and five header-guard macro definitions. Counts include overloads,
constructors/destructors, anonymous-namespace helpers, and both mutually
exclusive `incFrameId` bodies; compiler-generated implicit special members and
lambda call operators are excluded.

| Owner | `.cpp` | handwritten header | total |
| --- | ---: | ---: | ---: |
| `Artboard` / `ArtboardInstance` | 134 | 59 | 193 |
| `NestedArtboard` | 51 | 15 | 66 |
| `ArtboardComponentList` | 93 | 14 | 107 |
| `NestedArtboardLayout` | 21 | 4 | 25 |
| `NestedArtboardLeaf` | 2 | 2 | 4 |
| **Total** | **301** | **94** | **395** |

The earlier `NestedArtboard` count of 50 was off by one: there are 49 class
definitions plus both `buildVMIList` and `makeTranslate`. The previous count
also omitted the static `sm_frameId` statement, counted only one of the two
conditional inline `incFrameId` definitions, and excluded the five header
guards. The corrected denominator deliberately retains all of those authority
units.

The 58 `artboard.hpp` function definitions are `frameId`, both conditional
`incFrameId` bodies, `addedToHost`,
`setActiveFocusManager`, `focusManager`, `setActiveSemanticManager`,
`semanticManager`, `semanticBoundaryNode`, `shapeWorldTransform`,
`virtualizableComponent`, `updatesOwnLayout`, the testing constructor,
`didChange`, both `artboardId` accessors, both `artboardSource` accessors,
`factory`, `updateWorldTransform`, `ownedInheritedInterpolator`,
`canHaveOverrides`, `drawOrderChangeCounter`, `firstDrawable`, `clipPath`,
`backgroundPath`, both `objects` forms, `nestedArtboards`,
`artboardComponentLists`, `dataContext`, `scriptingVM`, `originalWidth`,
`originalHeight`, `resetSize`, both `find` templates, `count`, `objectAt`,
`objectIndex`, `animationCount`, `stateMachineCount`, `firstAnimation`,
`firstStateMachine`, `instance<T>`, `isInstance`, the `frameOrigin` getter,
`deserialize`, `hostOpacity`, `childOpacity`, `hasSelfTransform`,
`selfTransform`, and the six tools callbacks. Its 59th authority unit is the
header guard.

The remaining header definitions are:

- `nested_artboard.hpp` (14 functions plus header guard):
  `isArtboardDataBound`, `artboardCount`, `type`,
  `artboardInstance`, `sourceArtboard`, `parentArtboard`,
  `markHostTransformDirty`, `hostComponent`, `keyInput`, `textInput`,
  `gamepadDispatch`, `focused`, `blurred`, and `focusableArtboard`.
- `artboard_component_list.hpp` (13 functions plus header guard):
  `artboardCount`, `transformComponent`,
  `parentArtboard`, `markHostTransformDirty`, `hostComponent`,
  `isLayoutProvider`, `numLayoutNodes`, `setVisibleIndices`,
  `shouldResetInstances`, `itemCount`, `item`, `type`, and
  `listScopeFocusNode`.
- `nested_artboard_layout.hpp` (3 functions plus header guard): `numLayoutNodes`,
  `isLayoutProvider`, and
  `transformComponent`.
- `nested_artboard_leaf.hpp` (1 function plus header guard): `fitChanged`.

### Falsifying findings

1. **Fresh clone change-state default differed; correction accepted.** Pinned
   `Artboard::instance<T>` constructs `new T`; `m_didChange` has an in-class
   default of `true`, and `instance<T>` never copies that runtime bit. This is
   observable after the source has drawn because `drawInternal` first clears
   the source bit. Rust `impl Clone for ArtboardInstance` now initializes the
   fresh occurrence to `true`, while
   `restore_transient_layout_transfer_state_from` explicitly restores the
   source bit for same-occurrence transient layout clones. The focused
   regression starts with a clean source and proves source `false`, public
   clone `true`, and transient clone `false`. The initial nested-layout paint
   evaluation now also calls that explicit transient path instead of silently
   relying on the former public-clone behavior. Adversarial call-site review
   found the other production same-occurrence clone in nested geometry
   traversal already uses `clone_for_transient_layout`; normal nested-host
   cloning continues through `RuntimeNestedArtboardInstance::clone`, where a
   fresh child occurrence is required and the new `true` default is literal.

2. **Production measurement registration lacked evidence; correction
   accepted.** `ordinary_nested_artboard_contributes_its_mounted_intrinsic_size`
   now imports a synthetic host `LayoutComponent` with an authored
   `LayoutComponentStyle` that is intrinsically sized and hugs both axes. A
   real `NestedArtboard` mounts an `80 x 60` child, and the test enters through
   `TaffyRuntimeLayoutEngine::compute_bounds`/`build_node`. It proves the
   unconstrained host is `80 x 60`; a second authored fixture with `50 x 40`
   maximums proves the production solve clamps the measured host to `50 x 40`.
   The independent review accepts this evidence: without the authored
   `intrinsicallySizedValue` and Hug-axis style, `build_node` would not install
   `LayoutComponentMeasure`; the `80 x 60` assertion therefore cannot be
   satisfied by the parent artboard's authored `200 x 100` bounds alone. The
   callback's ordinary/leaf branch reads the mounted child's dimensions and
   applies the same per-axis finite-mode clamp as pinned
   `NestedArtboard::measureLayout`.

3. **The inline frame counter remains unadjudicated.** Pinned `frameId()` is a
   static process-wide counter incremented by root `Artboard::draw`. Rust has
   the process-wide `artboard_draw_frame_id()` but also exposes
   `ArtboardInstance::frame_id()` from a per-occurrence counter and copies that
   counter in `Clone`. The receipt maps neither inline definition nor explains
   which Rust API is the literal owner. This must be resolved explicitly; an
   instance-local accessor cannot silently certify a static upstream symbol.

### Areas accepted by the adversarial pass

- `RuntimeNestedArtboardInstance` declares animation owners before the mounted
  child, preserving the pinned dependency-release-before-child-destruction
  boundary.
- `RuntimeComponentListItemInstance` declares row state machines before the
  row Artboard, preserving the pinned listener teardown order.
- `reset_retained_components_for_state_machine_settlement` retains the pinned
  early return and authored resetting-component order; component-list reset
  walks logical rows and acknowledges stateful contexts before resetting a
  mounted child.
- The ordinary/leaf nested measurement branch returns the mounted child's
  current dimensions and clamps axes independently, matching
  `NestedArtboard::measureLayout`. Its missing evidence is integration, not the
  branch arithmetic.

The unit of review was every out-of-line function, override, constructor,
destructor, anonymous-namespace helper, and conditional definition. Overloads
are listed separately below. “Owner-safe equivalent” means the C++ operation
is split across Rust value ownership or a retained subsystem, but its ordering
and observable effect were checked against the complete C++ body. “Taffy
adaptation” means the approved layout-engine substitution, not a parity claim
about Yoga internals.

## Finding corrected by this audit

`NestedArtboard::measureLayout` at `nested_artboard.cpp:774-787` was absent.
The pinned implementation returns the mounted Artboard's width and height,
clamping each axis only when Yoga supplies a finite constraint. Rust's
`TaffyRuntimeLayoutEngine::measure_layout_component` recognized an ordinary
`NestedArtboard`/`NestedArtboardLeaf` as intrinsically sizeable but then fell
through to `Size::ZERO`.

The Rust dispatcher now maps both host types to the mounted occurrence's
`artboard_dimensions()` with the same per-axis clamp. A
`NestedArtboardLayout` remains separate because it transfers the mounted
Artboard's layout node into the parent tree. The focused regression
`ordinary_nested_artboard_contributes_its_mounted_intrinsic_size` now proves
both the unconstrained `80 x 60` result and the constrained `50 x 40` result
through the production Taffy registration and solve path.

## `src/artboard.cpp`

The 133 out-of-line function definitions (counting overloads and conditional
audio/tools definitions) plus the `sm_frameId` static source statement map as
follows.

| C++ symbols, exhaustively enumerated | Exact Rust owner symbols | Disposition |
| --- | --- | --- |
| `Artboard::sm_frameId` | process-wide `artboard_draw_frame_id`; instance-local `ArtboardInstance::frame_id` remains separately exposed | **Pending:** static/instance ownership is the unresolved finding below |
| `Artboard::Artboard`, `Artboard::~Artboard`, `ArtboardInstance::ArtboardInstance`, `ArtboardInstance::~ArtboardInstance`, `canContinue`, `Artboard::validateObjects`, `Artboard::initialize` | `ArtboardInstance::from_graph_inner`, `ArtboardInstance::build_component_occurrence_relations`, `Drop for ArtboardInstance`, Rust field drop order | Owner-safe equivalent; rejected objects become import errors rather than dangling nullable slots |
| `Artboard::addObject`, `addAnimation`, `addStateMachine`, `addScriptedObject`, `sortDependencies`, `cloneObjectDataBinds` | `from_graph_inner`, `build_component_interface_schedules`, retained `objects`, `linear_animations`, `state_machines`, script-owner tables, `RuntimeRetainedDataBind::clone` | Owner-safe equivalent |
| `Artboard::sortDrawOrder`, `clearRedundantOperations` | `RuntimeDrawableList::from_graph`, `sort_draw_order`, `clear_redundant_operations`, retained clipping/draw-rule ordering in `draw.rs` | Equivalent retained draw list |
| `Artboard::initScriptedObjects`, `pollAsyncWork`, `drawCanvases`, `advanceScriptedViewModels`, `internalDrawCanvases`, `findDrawCanvasLuauState` | `update_script_instances_with`, `poll_script_async_work_tree`, `draw_script_canvases`, `advance_script_instances_with`, recursive mounted-host dispatch | Approved scripting-backend adaptation; traversal and lifecycle order retained |
| `Artboard::resolve`, `idOf` | `component`, `component_handle`, `component_local_for_global`, retained local/global index tables | Equivalent checked lookup |
| `Artboard::onComponentDirty`, `onDirty`, `propagateSize`, `sharesLayoutWithHost` | `add_component_dirt`, `dispatch_component_on_dirty`, `set_artboard_dimensions`, `retain_runtime_layout_component_bounds`, `layout_node_owned_by_host` | Equivalent; preserves the Artboard-specific no-child-size-propagation override |
| `Artboard::host` (setter), `host` (getter), `parentArtboard`, `markHostTransformDirty`, `changed`, `isAncestor` | `added_to_host`, mounted occurrence ownership, `ancestor_artboard_sources`, `root_transform`, `mark_changed`, `take_parent_change_request` | Owner-safe equivalent without a raw host pointer |
| `Artboard::onAddedClean`, `layoutWidth`, `layoutHeight`, `layoutX`, `layoutY`, `updateRenderPath`, `origin`, `bounds`, `worldBounds`, `frameOrigin` | `from_graph_inner`, `artboard_dimensions`, `layout_bounds`, `set_frame_origin`, `artboard_bounds`, retained Artboard background/clip slots in `RuntimeArtboardPathState`, layout-state `current_bounds` | Equivalent under Taffy adaptation |
| `Artboard::update`, `addDirtyDataBind`, `updateDataBinds`, `updateComponents`, `updatePass`, `advanceInternal`, `advance`, `reset` | `update_component_handle_with_mode`, `advance_artboard_data_binds_with_elapsed`, `update_pass_with_script_mode`, `advance_retained_components_collect_events_with_scripts`, `advance`, `reset_retained_components_for_state_machine_settlement` | Equivalent pass ordering and 100-step settlement ceiling |
| `Artboard::takeLayoutData`, `cleanLayout`, `markLayoutDirty`, `syncStyleChangesWithUpdate`, `syncStyleChanges`, `calculateLayout` | `layout_node_owned_by_host`, `dirty_layout`, `mark_layout_node_changed`, `refresh_layout_constraint_bounds`, `RuntimeLayoutEngine::compute_bounds`, nested-layout transfer keys | Approved Taffy adaptation; ownership and invalidation edges retained |
| `Artboard::hitTest`, `rootTransform`, `hitTestPoint` | retained hit-path dispatch, `root_transform`, `mounted_root_transform`, `component_world_transform_with_scroll`, state-machine pointer routing | Equivalent; includes the live mounted self-transform/host recursion corrected in phase 3 |
| `Artboard::draw`, `drawInternal`, `addToRenderPath`, `addToRawPath` | `begin_draw_frame`, `draw_artboard_internal_with_path_cache`, `runtime_object_path_commands`, `geometry_world_bounds_with_context` | Equivalent renderer-neutral command traversal; backend calls are the approved renderer boundary |
| `Artboard::xChanged`, `yChanged`, `originXChanged`, `originYChanged` | property callbacks in `after_double_property_set`, `mark_world_transform_changed`, `mark_path_changed`, host transform publication | Equivalent dirt edges |
| `Artboard::isTranslucent()`; `isTranslucent(const LinearAnimation*)`; `isTranslucent(const LinearAnimationInstance*)`; `hasAudio` | `runtime_is_translucent`, `StaticScene::is_translucent`, `ArtboardInstance::has_audio`, recursive mounted-host scan | Equivalent; audio engine implementation remains the approved native backend |
| `Artboard::animationNameAt`, `animation(const std::string&)`, `animation(size_t)`, `ArtboardInstance::animationAt`, `animationNamed` | `linear_animation`, `linear_animations`, `linear_animation_instance`, public `ArtboardDefinition::animation_name`/name lookup | Equivalent |
| `hasParentFocusData`, `Artboard::rootFocusDataCount`, `rootFocusDataAt`, `buildFocusTreeVisit`, `buildFocusTree(FocusManager*, FocusNode)`, `buildFocusTree(FocusNode)`, `cleanupFocusTree`, `setExternalParentFocusNode`, `externalParentFocusNode`, `collapseSingle` | `build_artboard_focus_tree`, `build_component_focus_tree`, `RuntimeFocusTree::build_focus_tree`, `sync_mounted_focus_tree`, `cleanup_focus_tree`, `collapse_component` | Owner-safe equivalent retained focus tree |
| `Artboard::buildSemanticTree`, `cleanupSemanticTree`, `collapseBoundarySubtree`, `collapseSemanticBoundary`, `markSemanticBoundaryTransformDirty` | `RuntimeSemanticTree::synchronize`, `visit_artboard`, `refresh_bounds`, `rebuild_routes`, `collapse_component` semantic publication | Owner-safe equivalent retained semantic tree |
| `Artboard::stateMachineNameAt`, `stateMachine(const std::string&)`, `stateMachine(size_t)`, `defaultStateMachineIndex`, `ArtboardInstance::stateMachineAt`, `stateMachineNamed`, `defaultStateMachine`, `defaultScene` | `state_machine`, `state_machines`, `default_state_machine_index`, `state_machine_instance`, public state-machine name lookup, `StaticScene` selection | Equivalent default fallback order |
| `Artboard::nestedArtboard`, `nestedArtboardAtPath`, `ArtboardInstance::input`, `getNamedInput`, `getBool`, `getNumber`, `getTrigger`, `getTextRun` | `RuntimeNestedArtboards` sparse/ordered index, occurrence path routing (`occurrence_state_machine_input`, mounted-child traversal), retained text-run lookup | Owner-safe equivalent |
| `Artboard::import` | `GraphFile::from_runtime_file`, `ArtboardInstance::from_graph_inner` | Equivalent checked import registration |
| `Artboard::buildDataContext`, `internalDataContext`, `rebind`, `relinkDataContext`, `rebuildDataBind`, `unbind`, `clearDataContext`, `dataContext` | `bind_owned_view_model_artboard_data_context`, `relink_data_context_for_state_machine`, `clear_data_context_for_state_machine_bind`, `unbind_for_state_machine_view_model_clear`, recursive host binding | Owner-safe equivalent |
| `Artboard::bindViewModelInstance` (one argument), `bindViewModelInstance` (with parent), `setViewModelInstance`, `bindViewModelInstances`, `bind`, `globalViewModelInstance`, `setGlobalViewModelInstance` | owned `RuntimeOwnedDataContext` constructors and rebinding, `set_global_view_model_instance`, `global_view_model_instance` | Equivalent; Rust handles replace unsafe ref-count mutation |
| `Artboard::volume` (getter), `volume` (setter), `hostOpacity`, `audioEngine` (getter), `audioEngine` (setter) | `audio_event_playback`, `inherit_audio_configuration_from`, `child_opacity`, `set_host_opacity`, recursive mounted-owner propagation | Approved native-audio adaptation; opacity behavior exact |
| `Artboard::artboardFile`, `ArtboardInstance::file` (setter), `file` (getter), `artboardFile` | `runtime_file`, `runtime_file_arc`, `build_context` | Owner-safe equivalent lifetime retention |

## `src/nested_artboard.cpp`

All 51 out-of-line definitions map as follows.

| C++ symbols, exhaustively enumerated | Exact Rust owner symbols | Disposition |
| --- | --- | --- |
| `buildVMIList`, `NestedArtboard::NestedArtboard`, `~NestedArtboard`, `clone`, `nest`, `applyOriginOverride`, `clearNestedAnimations` | `RuntimeNestedArtboardInstance`, its `Clone`, field drop order, `build_runtime_nested_artboard_instance`, `apply_nested_artboard_origin_override` | Owner-safe equivalent |
| `tryScheduleBindStateful`, `bindStateful`, `findStatefulChildVmi`, `setActiveViewModelInstance`, `updateArtboard` | `pending_stateful_binding`, `bind_owned_view_model_occurrence_data_context`, `reuse_owned_stateful_view_model_context`, `commit_nested_artboard_replacement`, `rebind_owned_view_model_context_after_nested_artboard_swap` | Equivalent stateful/swap ordering |
| `detectArtboardDataBinding`, `registerFocusScope`, `syncNestedFocusTree` | data-bind host catalog plus `RuntimeFocusTree::sync_mounted_focus_tree`, `install_external_focus_domain`, retained structural scopes | Owner-safe equivalent |
| `makeTranslate`, `draw`, `willDraw`, `hitTest`, `hitTestHost`, `hostTransformPoint`, `worldTransformForArtboard`, `worldToLocal` | mounted drawable recursion in `draw_artboard_internal_with_path_cache`, ordinary component-world dispatch, retained hit paths, `root_transform`, `mounted_root_transform`, inverse world-transform routing | Equivalent |
| `import`, `addNestedAnimation`, `onAddedClean` | `from_graph_inner`, `build_runtime_nested_artboard_instance`, retained animation-owner schedule | Equivalent checked construction |
| `update`, `collapse`, `hasNestedStateMachines`, `nestedAnimations`, `nestedArtboard`, `stateMachine`, `input(name)`, `input(name, stateMachineName)` | `update_nested_artboard_from_host_dirt`, `collapse_component`, `RuntimeNestedArtboards` accessors/index, occurrence state-machine input routing | Equivalent |
| `measureLayout` | `TaffyRuntimeLayoutEngine::measure_layout_component` ordinary/leaf branches | **Corrected here; exact per-axis clamp** |
| `controlSize` | no-op, as pinned | Exact |
| `decodeDataBindPathIds`, `copyDataBindPathIds`, `internalDataContext`, `relinkDataContext`, `clearDataContext`, `unbind`, `updateDataBinds`, `bindViewModelInstance` | retained `data_bind_path_ids`, `bind_owned_view_model_occurrence_data_context`, `bind_owned_view_model_animation_data_context`, child bind/unbind/update recursion | Owner-safe equivalent |
| `calculateLocalElapsedSeconds`, `advanceComponent`, `reset` | `calculate_local_elapsed_seconds`, `begin_advance`, `advance_after_animation_owners`, `advance_outer_update`, `reset_retained_components_for_state_machine_settlement` | Exact quantization and animation-before-child ordering |
| `file` (setter), `file` (getter), `referencedArtboardId`, `referencedArtboard` | mounted child `build_context`, source graph global id, `runtime_nested_artboard_instance_for_id` | Owner-safe equivalent |

## `src/artboard_component_list.cpp`

All 93 out-of-line definitions map as follows: 87
`ArtboardComponentList` definitions including overloads, the five
`ArtboardListDrawIndexDependent` definitions, and the anonymous-namespace
`artboardHasFocusContent` helper.

| C++ symbols, exhaustively enumerated | Exact Rust owner symbols | Disposition |
| --- | --- | --- |
| `ArtboardListDrawIndexDependent::{constructor, destructor, addDirt, relinkDataBind, clear}` | `component_list_draw_index_sink`, `RuntimeCellDirtSink`, `runtime_component_list_order` | Owner-safe equivalent dependent lifetime/cache invalidation |
| `ArtboardComponentList::ArtboardComponentList`, `~ArtboardComponentList`, `clear`, `collapse`, `clone` | `RuntimeConstrainableListState::default/clone`, Rust field drop order, `collapse_component`, transient component-list restore | Owner-safe equivalent; state machines precede row Artboards in field drop order |
| `listItem`, `artboardInstance`, `indexOfArtboardInstance`, `stateMachineInstance` | `component_list_state`, `component_list_items`, logical/occurrence identity lookup | Equivalent checked indexing |
| `layoutNode`, `markLayoutNodeDirty`, `updateLayoutBounds`, `cascadeLayoutStyle`, `syncStyleChanges`, `layoutBounds`, `layoutBoundsForNode`, `markHostingLayoutDirty`, `syncLayoutChildren`, `mainAxisIsRow`, `layoutParent` | `build_node`, `component_list_item_style`, `update_component_list_layout_bounds`, `mark_layout_node_dirty`, retained `layout_size`, `runtime_component_list_item_layout_size` | Approved Taffy adaptation; transferred-node ownership retained |
| `findArtboard`, `disposeListItem`, `createArtboard`, `createStateMachineInstance`, `listsAreEqual`, `updateList`, `createArtboardAt`, `addArtboardAt`, `bindArtboard`, `removeArtboardAt`, `removeArtboard` | `sync_component_list_items`, `create_component_list_item_instance`, `RuntimeComponentListResourcePools::{take,put}`, occurrence-identity reconciliation | Owner-safe equivalent list lifecycle |
| `ensureListScopeFocusNode`, `removeListScopeFocusNode`, `makeListRowFocusNode`, `reparentListRowsInScope`, `artboardHasFocusContent`, `listItemNeedsBuildUnderRow`, `syncListRowNodesWithList` (current-list overload), `syncListRowNodesWithList` (previous-list overload), `linkStateMachineToArtboard` | `RuntimeFocusTree::build_focus_tree`, `rebuild_after_structure_change`, `sync_mounted_focus_tree`, `install_external_focus_domain`, `FocusNode::structural_scope` | Owner-safe equivalent retained focus scopes/rows |
| `advanceComponent`, `reset` | `advance_component_list_entry`, `reset_component_list_instances` | Exact NewFrame/non-NewFrame and logical-row reset ordering |
| `willDraw`, `draw`, `hitTest`, `hitTestHost`, `hostTransformPoint`, `worldTransformForArtboard`, `update`, `updateWorldTransform`, `updateArtboardsWorldTransform`, `artboardPosition`, `worldToLocal`, `listTransform`, `listItemTransforms` | `runtime_component_list_order`, component-list draw recursion, retained item transforms, mounted hit routing, `runtime_component_list_item_base_transforms` | Equivalent |
| `invalidateOrderedListIndicesCache`, `recomputeListUsesDrawIndexSort`, `listItemDrawIndex`, `clearDrawIndexListeners`, `removeDrawIndexListenerForItem`, `syncDrawIndexListeners`, `ensureOrderedListIndices`, `orderedListIndices` | `runtime_component_list_order`, `component_list_draw_index_sink`, retained `RuntimeComponentListOrderCache` | Exact finite-value fallback and stable logical-index tiebreak |
| `updateConstraints`, `addVirtualizable`, `virtualizableChanged`, `removeVirtualizable`, `setVirtualizablePosition`, `virtualizationEnabled`, `scrollConstraint` | retained constraint dispatch, `add_component_list_virtualizable`, `remove_component_list_virtualizable`, `set_component_list_virtualizable_position`, `component_list_virtualization` | Equivalent under Taffy/constraint ownership |
| `internalDataContext`, `bindViewModelInstance`, `clearDataContext`, `unbind`, `updateDataBinds` | row `RuntimeOwnedDataContext`, `sync_component_list_items`, `advance_component_list_entry`, clear/drop | Owner-safe equivalent |
| `file` (setter), `file` (getter), `addMapRule` | `build_context`, `ArtboardListMapRule` graph catalog | Equivalent |
| `createArtboardRecorders`, `applyRecorders(Artboard*, source)`, `applyRecorders(StateMachineInstance*, source)` | `RuntimeComponentListItemInstance::restore_from_fresh`, cold child/state-machine reconstruction with backend-owner adoption | Owner-safe equivalent pooled reset semantics |
| `computeLayoutBounds`, `size`, `itemSize`, `setItemSize`, `gap` | `update_component_list_layout_bounds`, retained logical item sizes/layout size, layout-style gap lookup | Equivalent under Taffy adaptation |
| `attachArtboardOverride`, `clearArtboardOverride` | `component_list_item_override_local`, `component_list_item_style`, `mark_component_list_override_changed` | Equivalent, including default/specific selection and pinned height-hug quirk |

The component-list flag test
`upstream_flagged_component_list_joins_layout_through_a_group` remains an
expected-red cross-owner dependency: the component-list owner honors
`DrawableFlag::ParticipatesInLayout` when retained, but the pinned fixture's
flag is not decoded by the binary/property owner. It is not silently counted
as a certified component-list behavior.

## `src/nested_artboard_layout.cpp`

All 21 definitions map as follows.

| C++ symbols, exhaustively enumerated | Exact Rust owner symbols | Disposition |
| --- | --- | --- |
| `NestedArtboardLayout::clone` | `RuntimeNestedArtboardInstance::clone`, reset transfer key/cache | Owner-safe equivalent |
| `layoutBounds`, `layoutNode`, `markHostingLayoutDirty`, `markLayoutNodeDirty` | `runtime_nested_artboard_layout_bounds_frame`, `apply_nested_artboard_layout_bounds`, `mark_layout_node_dirty`, transfer-generation keys | Approved Taffy adaptation |
| `update`, `updateConstraints`, `onAddedClean` | `runtime_nested_artboard_layout_world_transform`, generic constraint dispatch, initial override/style transfer | Equivalent; includes exact mounted-origin compensation |
| `updateLayoutBounds`, `cascadeLayoutStyle`, `updateWidthOverride`, `updateHeightOverride`, `isRow`, `syncStyleChanges` | parent/child Taffy solve, `nested_artboard_layout_style`, `nested_artboard_layout_axis_*`, `apply_nested_artboard_layout_bounds_after_parent_solve` | Approved Taffy adaptation; source quirks retained |
| `instanceWidthChanged`, `instanceHeightChanged`, `instanceWidthUnitsValueChanged`, `instanceHeightUnitsValueChanged`, `instanceWidthScaleTypeChanged`, `instanceHeightScaleTypeChanged` | nested-layout property callbacks plus `mark_layout_node_changed`/`mark_layout_node_dirty` | Equivalent dirt publication |
| `updateArtboard` | `commit_nested_artboard_replacement`, `mark_nested_artboard_layout_changed`, refreshed parent-owned layout transfer | Owner-safe equivalent clear/swap/resync transaction |

## `src/nested_artboard_leaf.cpp`

| C++ symbols, exhaustively enumerated | Exact Rust owner symbols | Disposition |
| --- | --- | --- |
| `NestedArtboardLeaf::clone` | `RuntimeNestedArtboardInstance::clone` | Owner-safe equivalent |
| `NestedArtboardLeaf::update` | `nested_artboard_leaf_uint_property_changed`, `update_nested_artboard_from_host_dirt`, `runtime_nested_artboard_leaf_alignment`, `runtime_nested_artboard_leaf_world_transform`; layout-fit same-frame resize/reflow in the host update pass | Exact, including Fit::layout resize and same-frame child update |

The inline `fitChanged` override in `nested_artboard_leaf.hpp` maps to
`nested_artboard_leaf_uint_property_changed` and adds recursive world-transform
dirt exactly when the `fit` property changes.

## Evidence

- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime ordinary_nested_artboard_contributes_its_mounted_intrinsic_size --lib`
- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime public_clone_starts_changed_while_transient_clone_preserves_source_state --lib`
- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime public_artboard_clone_is_cold_but_transient_layout_clone_keeps_scripts --lib`
- `make --no-print-directory runtime-source-symbol-check`
- `cargo test -p nuxie-runtime layout_fit_leaf_resizes_its_mounted_artboard_from_the_parent_layout_frame --lib`
- `cargo test -p nuxie-runtime artboard_size_change_uses_artboard_propagate_size_override --lib`
- `cargo test -p nuxie-runtime nested_layout_host_follows_its_interpolating_parent_frame --lib`
- `cargo test -p nuxie-runtime component_list --lib`

Result of the original audit: one hidden source omission was found and
corrected. Result of the first independent adversarial review of `d3df628c7`:
**ACCEPTED FOR BOTH CORRECTIONS**. Fresh-clone `didChange` now follows pinned C++
while same-occurrence transient clones preserve the source bit, and ordinary
nested measurement now has production registration/solve evidence. All three
focused tests above passed against the exact reviewed commit with
`CARGO_INCREMENTAL=0`. The live shared tree briefly failed the measurement
test's compile step because an unrelated concurrent `draw.rs` edit referenced
an unresolved `path_cache_image_dimensions`; the exact-commit worktree removes
that unrelated failure from this verdict.

The overall Artboard family verdict remains **REJECTED / UNCERTIFIED** while the
static process-wide frame-counter ownership and the cross-owner
`ParticipatesInLayout` decoder red remain unresolved. The corrected
1,105-owner/7,818-unit campaign denominator is accepted; it is no longer an
open Artboard finding. Neither remaining gap is silently credited to this
family.

## Second independent adversarial re-review of `d3df628c7`: corrections accepted

This second review was performed independently of `e8642fce4` and accepts the
two corrections in `d3df628c7`. It read the pinned `Artboard::instance<T>`,
`m_didChange`, `onComponentDirty`, `changed`, `draw`, and `drawInternal` paths;
the complete `NestedArtboard` source and header; C++ layout measure-function
registration and solve entry; and the corresponding Rust clone, nested-host,
layout-build, measure-callback, and draw/change paths. It found no counterexample
to either corrected behavior.

For clone/change semantics, pinned public instancing default-constructs the new
Artboard before copying authored Core fields, so `m_didChange` starts `true` and
is not inherited from a clean source. Pinned `drawInternal` clears the source
bit before its opacity early return, while subsequent component dirt or
`changed()` restores it. Rust now makes the same distinction: public `Clone`
starts a new occurrence at `true`; both production same-occurrence layout/draw
views enter through `clone_for_transient_layout`, which restores the source bit;
and a normal `RuntimeNestedArtboardInstance::clone` deliberately creates a new
mounted occurrence. The focused clone test isolates the bit transition, and
source inspection additionally confirms that the real Rust draw path clears
the bit before its matching opacity early return. Changing the public default
therefore does not leak into either transient call site or alter the source.

For ordinary nested measurement, pinned `LayoutComponent::syncStyle` registers
its measure callback only when the style is intrinsically sized and the node is
a layout leaf. That callback walks non-layout intrinsically-sizeable children
and reaches `NestedArtboard::measureLayout`, which returns the mounted
occurrence's dimensions with an independent finite-mode clamp on each axis.
The corrected fixture creates exactly that production topology: the ordinary
`NestedArtboard` is measured content rather than a transferred layout-provider
node, its parent has an authored intrinsically-sized Hug style, and
`compute_bounds` builds and solves the Taffy tree. Without registration the Hug
leaf cannot produce the asserted `80 x 60`; the second fixture's distinct
`50 x 40` maximums exercise both finite axes through the same solve. The Rust
callback reads the live mounted child dimensions and applies the same two
axis-local minima as the pinned body.

All three focused commands listed in Evidence passed from an isolated detached
worktree at exact commit `d3df628c7` with `CARGO_INCREMENTAL=0` (the repository's
ignored fixture files were mirrored into that worktree solely so the lib-test
target could compile). This second acceptance is limited to the two corrected
rows. The Artboard family remains **REJECTED / UNCERTIFIED** for the independent
static-frame-counter and cross-owner `ParticipatesInLayout` gaps already
recorded above.
