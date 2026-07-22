# #RB-1 Compensation-Family Call-Site Inventory

Scout inventory (2026-07-21) for map Phase RB. The migration slice (e) works
through this list; the deletion gate (f) empties it. All paths repo-relative.

The family was inventoried because it emulated C++
`DataBindContainer`/`DataContext` change-propagation *by polling*. The five
public seams that originally fanned into it:

- `StateMachineInstance::bind_owned_view_model_contexts` / `::advance_data_context` (`crates/nuxie-runtime/src/state_machine/instance.rs:5083`, `:5277`)
- `ArtboardInstance::bind_owned_view_model_artboard_contexts` / `::advance_artboard_data_binds[_with_elapsed]` (`crates/nuxie-runtime/src/artboard_data_bind.rs:4963`, `:6223`, `:6391`)
- Authored-instance overlay constructor (retired): the duplicate mutable API
  and its facade methods were removed. The one canonical constructor now
  matches C++ clone + `completeViewModelInstance` behavior.
- `Scene::{advance, try_advance_with_factory, pointer_*}` (the former
  `flush_view_model` compensation is deleted; surviving seams are
  `crates/nuxie/src/scene.rs:15217-15455`)
- `File::advance_with_state_machines_and_view_model` family (`crates/nuxie/src/lib.rs:2756`, `:2859`, `:3453`, `:3552`)

## Item 1 - Mutation clocks / generations

- [x] f11 deleted the shared `mutation_generation` clock, every reroot/replace/
  merge/union/track helper and caller, `RuntimeArtboardOwnedContextKey`, and
  the component-list structural-generation poll. Exact cells now notify the
  one retained authored bind, which appends its exact occurrence to a reusable
  container queue; ViewModel replacement and mounted list rows use pushed
  structural-rebind sinks. Steady frames do not traverse candidates or compare
  a generation.
- [x] f9 deleted `owned_view_model_candidate_generations` and every
  state-machine read/write/clone/reset site.

## Item 2 - Candidate vectors

- `RuntimeOwnedViewModelBindingCandidate` remains as the ordered DataContext
  lookup/listener-addressing carrier. f9 deleted its `mutation_generation`
  accessor; constructors and path/context helpers remain.
- Ctor call sites: `artboard.rs:2778,2837,3402,10518,10770-10771,
  11122,11157-11158` (last four groups are tests); `instance.rs:4821,
  5099,5102`; `artboard_data_bind.rs:4942,4970,4979,5067,5073,5134,
  5138,5167,8511,8515,9690` (last is a test).
- SM retained vector `owned_view_model_candidates` remains for deterministic
  own/parent candidate order, key-frame graphs, listener writes, and public
  compatibility seams. It no longer has a parallel generation vector or
  participates in a steady-frame generation comparison.
- Artboard retained vector `artboard_owned_view_model_candidates`
  `artboard.rs:269`; init `:1078,6775`; artboard reads/writes
  `:2405-2407,2755,2836-2838,3394`; data-bind reads/writes
  `artboard_data_bind.rs:4649,5000-5019,5119-5124,5219-5227,5335,
  5361,6087,6273-6296,6396-6411,6463,8439`.
- `bind_owned_view_model_context_candidates` (SM)
  `instance.rs:5133-5176`; graph-level `data_bind_graph.rs:5056-5129`;
  callers `instance.rs:4820,5104`; `artboard.rs:2406,2798,2869,3418,
  5683`; `artboard_data_bind.rs:5183`; tests
  `artboard.rs:10520,10774,11126,11161`.
- [x] f13 deleted the two listener-write callers that rebound every graph
  source after mutating one retained cell. C++ `ListenerViewModelChange`
  updates the exact source bind and dirties its paired target bind; it never
  binds the whole DataContext again (`listener_viewmodel_change.cpp:42-80`;
  `viewmodel_instance_number.cpp:10-19`; `data_bind.cpp:502-546`). The
  surviving explicit-bind and pushed structural-relink operation is now named
  `bind_owned_data_binds_from_candidates`, matching
  `DataBindContainer::bindDataBindsFromContext` rather than the deleted
  compensation lifecycle (`state_machine_instance.cpp:2901-2913`;
  `data_bind_container.cpp:25-35`; `data_bind_context.cpp:56-89`). Its unused
  trigger-sync parameter and branch were also deleted.
- `owned_view_model_context_candidates_for_nested_host`
  `artboard_data_bind.rs:5999-6043`; callers `:5055,5122,8444`.

## Item 3 - Listener observed-copy rescans

- [x] f4 deleted `changed_view_model_listener_actions_for_candidates` and
  its bind-time `MAX_VIEW_MODEL_LISTENER_ITERATIONS` loop. Candidate listeners
  now retain scalar-cell dependents and enqueue every mutation for next-frame
  `applyEvents`; the former compensation cap test was rewritten as the C++
  100-batch oracle, including batch 101 remaining pending.
- [x] f5 deleted the context-chain/single-context scanners, the `observed`
  value enum and readers, and the polling-era bind-time settlement API.
  Compatibility contexts resolve the retained scalar cell directly without
  concatenating temporary paths. Every owned-context listener path now uses
  the same mutation queue; binding itself is silent.

## Item 4 - Alias mirrors / detached trees

- [x] f7A closed the linked-`AssetFont` cell/delegation omission:
  `mirror_linked_instance` shares the exact retained Font cell and refreshes
  its payload snapshot at handle borrow, and all nested host/sync/full-data-
  bind writes delegate to the retained child. The oracle is C++
  `ViewModelInstanceViewModel` retaining one
  `rcp<ViewModelInstance>` (`viewmodel_instance_viewmodel.hpp:19-39`) while
  preserving `ViewModelInstanceAssetFont`'s file-index/live-Font two-part
  setter contract (`viewmodel_instance_asset_font.cpp:29-75`). The payload
  snapshot itself is not yet shared: a parent borrow held across a separate
  child mutation stays stale until the next handle borrow. The typed Font
  endpoint must remove that boundary before the direct-property clock union
  deletes. Superseded by f7B/f8 below.
- [x] f7B removed that owned String/Font payload snapshot boundary. String
  cells now own `Arc<[u8]>`, Font cells own the complete file-index/live-Font
  value, and every retained alias/source candidate points at that one typed
  endpoint. Owned reads return an `Arc`/value copy because a borrowed payload
  guard cannot remain live across an aliased mutation without also blocking
  that mutation. String keeps content equality and deep-copy clone semantics
  (`include/rive/generated/viewmodel/viewmodel_instance_string_base.hpp:33-55`;
  `src/viewmodel/viewmodel_instance_string.cpp:10-25`). Font preserves the
  pinned index/live setter ordering, including the possible two dependent
  cascades for a changed live pointer, and deep clones retain the file index
  but clear the private live Font
  (`src/viewmodel/viewmodel_instance_asset_font.cpp:13-86`). Candidate graphs
  retain the exact String/Font source cell, as C++ retains the source value
  and registers the bind on it (`src/data_bind/data_bind.cpp:210-216`). Scene's
  nested String cache is gone; public cursors reread the retained endpoint.
  This slice covers owned retained contexts. Imported override storage,
  artboard target/cache values, converter operands, and the remaining direct-
  property mutation-clock union are separate cuts and remain inventoried.
- [x] f8 deleted the dynamic structural-mirror refresh lifecycle. One retained
  `RuntimeOwnedViewModelEndpoint` now carries a ViewModel-valued property's
  active imported selection plus retained linked child; ordinary `Clone`
  detaches the endpoint before the graph remap.
  `RuntimeOwnedViewModelHandle::{borrow,borrow_mut}` no longer performs a
  refresh pass, and linked writes no longer recopy child/list topology after
  mutation. Active value/cell/list reads, sync and ViewModel writes, script
  advance, graph-cycle walks, and clock rerooting traverse the retained link
  directly and exclude the inactive compatibility storage. A parent borrow
  held across a direct retained-child replacement observes the new leaf
  immediately, matching the retained setter and
  synchronous relink ordering in
  `include/rive/viewmodel/viewmodel_instance_viewmodel.hpp:23-35` and
  `src/viewmodel/viewmodel_instance.cpp:118-188`. No dynamic or one-time
  scalar/list mirror copy remains: active paths traverse the retained child
  directly, while inactive authored/generated compatibility storage remains
  untouched and becomes visible again if authored selection replaces the
  explicit link. The direct-property clock union remained only for the
  then-unmigrated artboard target/cache consumers and was deleted in f11.
  Direct reassignment of the same retained child also runs the lifecycle and returns
  success; the prior `Ok(false)` equality shortcut was absent from C++.
  Selecting an authored instance after an explicit link detaches that link and
  its parent relay, because C++ has one active child pointer rather than two
  layered selections. Final evidence is runtime lib 361/361, nuxie lib
  132/132, C++ probe 708/708, ordinary and scripted goldens 317/317 entries
  plus 647/647 exact segments with zero failures, C API smoke green, and the
  full workspace green. Both Standards and Spec re-reviews are clean.
  Renderer goldens are not applicable because this slice changes no
  renderer/draw code.
- [x] f1 deleted `linked_aliases`, `add_linked_alias`,
  `remove_linked_alias`, and `synchronize_linked_aliases`. Linked scalar
  properties share retained cells by identity; deep-copy `Clone` remains the
  observable detached-tree contract.
- [x] The duplicate mutable authored-instance API and
  `overlay_active_instance` compensation were removed. Existing callers use
  the canonical `from_instance`, whose one source map preserves shared
  identity across ViewModel properties and list items like C++
  `File::completeViewModelInstance`.

- [x] f9 retained structural source dirt and removed the state-machine
  candidate-generation poll. `List`, `ListLength`, and `ViewModel` source
  paths now retain the exact property cell plus the actual list/child endpoint;
  structural cells carry dirt identity rather than copied payloads, and graph
  read models are derived from the retained object. List mutations notify
  their own list cell even for same-index swap and empty-to-empty update,
  ViewModel assignment notifies its endpoint, and nested replacement pushes
  one dedicated rebind sink through the weak-parent relay. Explicit same-source
  binds preserve C++'s reconcile dirt in favor order.
  The state machine deleted `owned_view_model_candidate_generations`, the
  candidate mutation accessor, steady-frame full-graph rebind, and the
  trigger-source refresh scan. C++ retains dependents on each
  `ViewModelInstanceValue`, notifies the exact list property, retains that
  source in `DataBind`, and synchronously relinks parent containers
  (`include/rive/viewmodel/viewmodel_instance_value.hpp:68-97`;
  `src/viewmodel/viewmodel_instance_list.cpp:26-60,76-143,183-225`;
  `src/data_bind/context/context_value.cpp:133-165`;
  `src/data_bind/context/context_value_list.cpp:17-29`;
  `src/data_bind/context/context_value_viewmodel.cpp:21-41`;
  `src/data_bind/data_bind_context.cpp:80-85`;
  `src/data_bind/data_bind.cpp:210-240,502-546`;
  `src/viewmodel/viewmodel_instance.cpp:346-415`). Candidate vectors remain
  for deterministic context resolution. Linked-child read models use a
  clone-fresh allocation identity, not the semantic instance ID, so detached
  graphs retain C++ pointer-key inequality. The artboard structural key, root
  clock, and target/cache values were subsequently removed or migrated in
  f11; f10 supersedes the converter-operand entry below.
  Evidence: runtime lib 363/363, nuxie lib 133/133, C++ probe 708/708,
  ordinary and scripted goldens 317/317 entries plus 647/647 exact segments
  with zero failures, C API smoke and workspace green; Standards and Spec
  re-reviews clean. Renderer goldens are not applicable because no
  renderer/draw code changed.

## Item 5 - Facade-wide rebind bits

- [x] e5(C) deleted Scene's `dirty` bit, `flush_view_model`, advance-time
  flush, and pointer-time rebinds.
- [x] f9 changed state-machine candidate refresh from a generation poll/full
  rebind into retained per-source dirt collection plus one pushed structural-
  rebind sink. f13 deleted the stale polling-era symbol by renaming the
  surviving operation `collect_retained_owned_view_model_dirt`; its callers
  still fold exact source dirt before the frame and consume the pushed
  structural-relink request.
- [x] f11 replaced the artboard analog with
  `refresh_retained_owned_view_model_artboard_sources`: scalar/source dirt
  drains exact retained occurrences from the container queue, while only
  pushed structural rebind dirt re-resolves paths. A second clean advance
  performs no bind scan.

## Item 6 - Per-source copied state (data_bind_graph.rs)

- [x] f9 migrated state-machine `List`, `ListLength`, and `ViewModel` sources
  to exact retained property cells and retained structural source objects.
  Their graph values remain derived read models rather than cell payloads;
  wakeup identity is the property cell and structural relinking is pushed, so
  no candidate-generation rebind is required. Artboard target/cache storage
  remains a separate copied-value consumer.
- [x] f10 migrated owned `OperationViewModel` and Project converter operands
  from copied snapshots/rescans to exact retained Number cells. State-machine
  operands register on their source node's existing
  `RuntimeRetainedDataBind` sink, matching C++ registering the outer
  `DataBind*` (`data_converter_operation_viewmodel.cpp:8-27,48-59`). Project
  conversion resolves the live cell value on demand; its serialized
  `resolved_values` vector is now default/imported compatibility state only.
  f11 supersedes the transitional artboard entry below.
- [x] f11 creates one `RuntimeArtboardAuthoredDataBindState` per authored
  `data_bind_index`. It owns the exact source, `RuntimeRetainedDataBind`
  direction/origin dirt, shared two-way converter state, outer converter
  operand subscriptions, and target-notification suppression. Source dirt is
  occurrence-indexed and updates the typed cache from that exact source without
  a candidate rescan; target adapters are preindexed by `data_bind_index`, so
  converter operands never fan out to same-path sibling binds. Direct list
  dirt executes the full retained list adapter and component-list occurrence
  reconciliation even when the item count is unchanged;
  reverse writes call the same retained bind's `update_source_binding`, so its
  self-echo is swallowed. Default/imported rebinding clears the old source.
  Formula-token and converter-property binds retain separate sinks because
  they are subordinate authored DataBinds in C++ rather than split records of
  the outer bind. Formula-token binds additionally own an exact primary-source
  queue independent of the outer `data_bind_index`; clone-local primary and
  converter-operand sinks preserve pending dirt. `bindsOnce` skips the
  DataBind source edge unless an attached Formula owns its independent C++
  dependency, and mixed Formula groups reset only their sourceChange children.
  The old shared-converter map, split occurrence sinks, reverse-write sibling
  resolver, structural key, and mutation clock are deleted.
- Per-target `default_value`/pending fields remain execution adapters for the
  serialized target and converter kinds; they no longer decide source
  identity, direction origin, or steady-frame wakeup.

## Item 7 - Trigger observed/rescan state

- [x] f12A removed the owned-context trigger copy. The adapter now stores
  metadata plus the selected `RuntimeViewModelCell`; owned bindings select
  the exact retained cell, whose state owns both
  `ValueFlags::valueChanged` and the per-layer use set. Layer keys are unique
  per state-machine layer occurrence (and refreshed on clone), matching C++'s
  `StateMachineLayerInstance*` keys rather than colliding on a shared numeric
  layer index (`viewmodel_instance_value.cpp:59-62,131-135,176-179`;
  `state_machine_input_instance.hpp:78-102`;
  `transition_viewmodel_condition.cpp:49-60`;
  `transition_property_viewmodel_comparator.cpp:50-67`). f12B subsequently
  removed the temporary copied compatibility variant for default/imported
  contexts.
- [x] f12B established one serialized-instance cell catalog per loaded
  `nuxie::File`, passed into every root/nested `RuntimeArtboardBuildContext`.
  Standalone raw constructors create a fresh file occurrence, with a hidden
  bridge for callers that already own the shared catalog. Bare default and
  imported bindings attach
  both transition-trigger metadata and graph trigger sources to the exact
  file-owned instance cell. An imported context created through the artboard
  occurrence factory retains that same occurrence immediately, so writes made
  before machine binding preserve canonical C++ call order. A detached
  compatibility context adopts on bind and rejects pre-bind trigger writes.
  Machines built in one runtime occurrence
  therefore observe one counter/change/use state. Context clones copy the
  serialized payload without dynamic changed/use/dirt state, while public
  state-machine clones deep-copy the catalog and rebind their internal sources
  to preserve snapshot isolation. This matches the probe retaining
  `ViewModel::instance(index)` directly (`tools/cpp-probe/main.cpp:1267-1300,
  4683-4721`) and C++ transition evaluation using the DataBind's retained
  source (`transition_viewmodel_condition.cpp:49-60`;
  `transition_property_viewmodel_comparator.cpp:50-67`). The
  `StateMachineViewModelTriggerSource::{Retained,Copied}` split,
  `trigger_overrides`, default-trigger mirrors,
  `sync_default_view_model_triggers_from_active`,
  `reset_bound_trigger_sources`, and `reset_active_view_model_triggers` are
  deleted. File reset uses a build-time, cycle-safe unique trigger-cell set for
  the complete nested/list owner; owned reset walks live list storage so row
  insertion/removal is observed. Missing paths stay explicitly unbound.
  Evidence: runtime lib 399/399, nuxie lib 140/140, C++ probe 721/721,
  ordinary and scripted goldens 317/317 entries plus 647/647 exact segments;
  both have zero failures after restoring converter-owned subordinate DataBind
  discovery. C API smoke and the full workspace are green, and the workspace
  target now builds/exports the C++ probe so the 721 tests cannot silently
  skip.
  Renderer goldens are not applicable because no renderer/draw code changed.
- Instance fields `default_view_model_triggers` and `view_model_triggers`
  remain as metadata/source-handle vectors; transition evaluation and fire
  actions still thread them through the state-machine call tree. Input-trigger
  `used_layers` is a separate C++ `SMITrigger` contract and survives.
- `bind_active_owned_view_model_triggers_for_candidates` remains for identity
  and structural rebind. [x] f9 deleted
  `refresh_active_owned_view_model_triggers_for_candidates`; a retained trigger
  cell supplies steady-frame change state. [x] f12B deleted the remaining
  default/imported sync, mirror, copied-source, and reset helpers after binding
  those modes to their canonical file-owned cells.
- PUBLIC trigger reads (survive): `view_model_trigger_count` `instance.rs:5528`, `view_model_trigger_value_count` `:5567`, `view_model_trigger_property_id` `:5571`.

## Public API surface that must keep working

- StateMachineInstance: `bind_owned_view_model_contexts` `instance.rs:5083`; `bind_owned_view_model_context` `:4794` / `_handle` `:4806` / `_context_handle` `:4811` / `_mut` `:4840`; `bind_default_view_model_context` `:4676`; `bind_view_model_instance_context` `:4710`; `bind_imported_view_model_context` `:4751`; `advance_data_context` `:5277`; `update_data_binds_apply_target_to_source` `:5415`; `set_bindable_*_for_data_bind` `:2684-2882`; `set_default/owned/imported_view_model_*_source_*` + `relink_*` `:2808-4642`; trigger reads `:5528,5567,5571`; `pointer_*_with_owned_view_model_context*` `:1326-1820`.
- ArtboardInstance: `bind_owned_view_model_artboard_contexts` `artboard_data_bind.rs:4963`; `_context` `:4910` / `_handle` `:4926` / `_context_handle` `:4937`; `bind_default_view_model_artboard_list_context` `:4895`; `bind_owned_view_model_nested_artboard_contexts` `:5193`; `advance_artboard_data_binds` `:6223` / `_with_elapsed` `:6391`; `owned_view_model_context` `artboard.rs:2417`; composite advances `artboard.rs:2999,3100-3108`; script ViewModel writes `artboard.rs:1559,1582`; `artboard_list_binding_*_for_data_bind` `artboard_data_bind.rs:8596-8644`.
- File/facade: `instantiate_view_model`; canonical
  `instantiate_view_model_instance`; `bind_view_model`;
  `owned_view_model_context`; `view_model_index`;
  `advance_with_state_machines_and_view_model`; `try_advance_...factory`;
  `ViewModelInstance::{raw,raw_mut,handle}`;
  `RuntimeOwnedViewModelInstance::{new,from_instance,from_instance_name,detached_graph,list_items_by_property_name_path}`.
- Scene: `advance` `scene.rs:15390`; `try_advance_with_factory` `:15455`; `pointer_down/move/up/exit` `:15217,:15254,:15284,:15320`; `view_models` `:8428`; `create_view_model_listener` `:12828`; `add_listener_view_model_*_action` `:13014-13138`; `add_view_model_trigger_condition` `:13601`; `scrub` `:15348`; `instantiate` `:6830`.
- FlowSession: delegates to Scene; no direct family reach.

## Deletion-gate checklist (f)

Remaining work is now explicitly split into f14 (one atomic code landing) and
f15 (closure evidence):

- [ ] f14A: add the production owned `DataContext` carrier: ordered local
  retained handles/scopes, optional parent, actual-instance-id resolution,
  exact property/structural source lookup, and context identity. Slot keys are
  placement metadata only, matching `data_context.hpp:37-61,104-124` and
  `data_context.cpp:166-261,397-506`; delete candidate path rewriting rather
  than transplanting it into the new type.
- [ ] f14B: migrate all candidate consumers in `artboard.rs`,
  `artboard_data_bind.rs`, `data_bind_graph.rs`, and
  `state_machine/instance.rs`. Nested artboards and component-list rows build
  local-plus-parent contexts instead of prefixing child scope paths into every
  inherited candidate. Listener cell binding/writes, converter operands,
  font sync, trigger/reset advance coverage, public bind seams, clone/reset,
  and pushed structural relink must all use the same retained context.
- [ ] f14C: transition trigger compare/use obtains the source cell from the
  bindable target's retained `DataBind`; fire-trigger actions retain the
  authored path and resolve it from the live context at perform time. Delete
  the active trigger-cell vector and the call-tree parameters that carry it.
- [ ] f14D: zero production occurrences of
  `RuntimeOwnedViewModelBindingCandidate`, `owned_view_model_candidates`,
  `artboard_owned_view_model_candidates`,
  `owned_view_model_context_candidates_for_nested_host`,
  `bind_active_owned_view_model_triggers_for_candidates`, and the active
  `StateMachineViewModelTriggerInstance` cell-shadow mechanism. Roughly twelve
  candidate tests migrate to direct parent/local-context contracts; public
  trigger inspection survives through immutable metadata plus the canonical
  retained source.
- [ ] f15: repeat this inventory, run every Phase-RB floor (including scripted
  immediately before push and the 1,468 renderer rows), and obtain clean
  independent Standards/Spec reviews. Only then close the parent (f) and
  #RB-1 boxes.

f13 already deleted the listener-triggered whole-context rebind compensation,
the dead trigger-sync branch, and the obsolete
`rebind_owned_view_model_context_candidates` /
`refresh_owned_view_model_candidates` symbols while retaining their legitimate
explicit-bind, pushed-structural-relink, and exact-dirt collection operations
under C++-shaped names. Already deleted:
`refresh_owned_view_model_artboard_context_if_mutated`,
`RuntimeArtboardOwnedContextKey` + `matches_candidate*`, the complete
`mutation_generation` clock family,
`owned_view_model_candidate_generations`,
`refresh_active_owned_view_model_triggers_for_candidates`, and
`sync_default_view_model_triggers_from_active`,
`reset_bound_trigger_sources`, and
`linked_aliases`/`synchronize_linked_aliases`.

## Test-rewrite counts

mutation-generation/clock tests were rewritten to exact-cell and pushed-sink
contracts; candidates ~12 (rewrite to new
context type); listener cap 1 (rewritten in f4 to the C++ batch cap);
duplicate mutable authored-instance API 1 (removed; test migrated to the
canonical constructor); alias sharing 1 (observable contract, keep); Scene dirty 0
(integration coverage remains); per-source copied state: rewrite at graph
API level; triggers: public read API survives, internal rescan tests delete.
