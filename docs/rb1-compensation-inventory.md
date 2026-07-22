# #RB-1 Compensation-Family Call-Site Inventory

Scout inventory (2026-07-21) for map Phase RB. The migration slice (e) works
through this list; the deletion gate (f) empties it. All paths repo-relative.

The whole family exists to emulate C++ `DataBindContainer`/`DataContext`
change-propagation *by polling*. The five public seams that fan into it:

- `StateMachineInstance::bind_owned_view_model_contexts` / `::advance_data_context` (`crates/nuxie-runtime/src/state_machine/instance.rs:5121`, `:5316`)
- `ArtboardInstance::bind_owned_view_model_artboard_contexts` / `::advance_artboard_data_binds[_with_elapsed]` (`crates/nuxie-runtime/src/artboard_data_bind.rs:4225`, `:5534`, `:5690`)
- `File::from_instance_mutable` (`crates/nuxie-runtime/src/view_model.rs:6486`, re-exposed `crates/nuxie/src/lib.rs:2620`, `:3329`)
- `Scene::{advance, try_advance_with_factory, pointer_*}` -> `flush_view_model` (`crates/nuxie/src/scene.rs:5389`)
- `File::advance_with_state_machines_and_view_model` family (`crates/nuxie/src/lib.rs:2714`, `:2813`, `:3402`, `:3497`)

## Item 1 - Mutation clocks / generations

- `mutation_generation`: the root clock and its artboard readers remain in
  `view_model.rs`, `artboard.rs`, and `artboard_data_bind.rs`. f9 deleted the
  state-machine candidate projection and the candidate accessor; state-machine
  steady frames no longer sample this clock. Remaining consumers are artboard
  target/cache refresh and converter operands, plus their clock-lifecycle
  tests.
- `RuntimeArtboardOwnedContextKey` (VM structure epoch, NOT render-cache): struct `artboard_data_bind.rs:507`, instance key `:511-518`; ctors/comparators `:521,:539,:557,:575,:594`; field set/cleared `:4576,4583,4638-4639,5398`; users `retain_owned_view_model_context_candidates` `:4627-4640`, `refresh_owned_view_model_artboard_context_if_mutated` `:5694-5697`, `:5890`.
- [x] f9 deleted `owned_view_model_candidate_generations` and every
  state-machine read/write/clone/reset site.
- `reroot_mutation_clock` `view_model.rs:6423-6428`; callers `:1625,:3013,:3493,:6495,:6662`; related `replace_mutation_clock` `:6430`, `merge_mutation_clock` `:6436` (alias registry `:1787,:2013`), `track_mutation` `:6440`, reset `:6522`.

## Item 2 - Candidate vectors

- `RuntimeOwnedViewModelBindingCandidate` remains as the ordered DataContext
  lookup/listener-addressing carrier. f9 deleted its `mutation_generation`
  accessor; constructors and path/context helpers remain.
- Ctor call sites: `artboard.rs:2692,2744,5577,10097,10391-10392,10693,10728-10729` (last four tests); `instance.rs:4822,5127,5130`; `artboard_data_bind.rs:4204,4232,4241,4311,4317,4378,4382,4411,4461,4556,5358,5596,7663,7667`.
- SM retained vector `owned_view_model_candidates` remains for deterministic
  own/parent candidate order, key-frame graphs, listener writes, and public
  compatibility seams. It no longer has a parallel generation vector or
  participates in a steady-frame generation comparison.
- Artboard retained vector `artboard_owned_view_model_candidates` `artboard.rs:267`; init `:1019,6667`; rw `artboard.rs:2316-2318,2668,2743-2745`; `artboard_data_bind.rs:4265,4363-4368,4460-4473,5400,5580-5603,5695-5705,5750,6000,6483,6936,7089,7591-7596`.
- `bind_owned_view_model_context_candidates` (SM) `instance.rs:5168-5224`; graph-level `data_bind_graph.rs:4658-4683`; callers `instance.rs:5076,5078,5132,5188,5395`; `artboard.rs:2317,2706,2776,5585`; `artboard_data_bind.rs:4424`; tests `artboard.rs:10099,10101,10697,10699,10732,10734`.
- `rebind_owned_view_model_context_candidates` `instance.rs:5069-5085`; callers `:5107,:5188,:2257`.
- `owned_view_model_context_candidates_for_nested_host` `artboard_data_bind.rs:5321-5365`; callers `:4299,4366,7596`.

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
  explicit link. The direct-property clock union remains only for unmigrated
  artboard target/cache and converter consumers. Direct
  reassignment of the same retained child also runs the lifecycle and returns
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
- `from_instance_mutable` `view_model.rs:6486-6497` (PUBLIC; callers `crates/nuxie/src/lib.rs:2625,3334`; test `:11341` — API survives, internals replaced); `overlay_active_instance` `:6499+` (only `:6493`); `detach_list_storage` `:6418-6421` (callers `:3012,:3492,:6494`).

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
  clock, target/cache values, and converter operands remain inventoried.
  Evidence: runtime lib 363/363, nuxie lib 133/133, C++ probe 708/708,
  ordinary and scripted goldens 317/317 entries plus 647/647 exact segments
  with zero failures, C API smoke and workspace green; Standards and Spec
  re-reviews clean. Renderer goldens are not applicable because no
  renderer/draw code changed.

## Item 5 - Facade-wide rebind bits

- [x] e5(C) deleted Scene's `dirty` bit, `flush_view_model`, advance-time
  flush, and pointer-time rebinds.
- [x] f9 changed state-machine `refresh_owned_view_model_candidates` from a
  candidate-generation poll/full rebind into retained per-source dirt
  collection plus one pushed structural-rebind sink. The helper name remains
  because its callers still need to collect source dirt before the frame.
- Artboard analog `refresh_owned_view_model_artboard_context_if_mutated`
  remains and still compares `RuntimeArtboardOwnedContextKey` generations.

## Item 6 - Per-source copied state (data_bind_graph.rs)

- [x] f9 migrated state-machine `List`, `ListLength`, and `ViewModel` sources
  to exact retained property cells and retained structural source objects.
  Their graph values remain derived read models rather than cell payloads;
  wakeup identity is the property cell and structural relinking is pushed, so
  no candidate-generation rebind is required. Artboard target/cache storage
  and project-converter operands remain separate consumers of copied values.
- `default_value` fields `:74` (target), `:219` (source); copies `:987-992,:4270,:4318-4322,:4427,:4637,:4666,:4747,:4781`; whole `:4744-6262` block is per-type default_value rw.
- `target_origin` field `:206`; set/cleared `:4356,7846,9387,9547,9560,9565,9582,9917`.
- `mark_reconcile_dirty` `:9569-9588`; callers `:4294,4331,4433,4580,4614,4650,4677`.
- `reset_converter_state` `:9906-9909`; batch `:4685`; callers `:4290,4325,4432,4579,4613,4649,4676,4687`. `reset_formula_random_state_for_source_change` `:9915-9924`.
- Reverse-write sibling scans: `bind_owned_view_model_target_to_source_bindings` `artboard_data_bind.rs:4273,5605,5717`; ordering comment `:5706-5719`.

## Item 7 - Trigger observed/rescan state

- `StateMachineViewModelTriggerInstance` `state_machine.rs:2527-2591` (`used_layers` `:2532`; `reset` `:2567,2573,2581`; `is_changed_for_layer` `:2585-2590`); instance fields `default_view_model_triggers` `instance.rs:64`, `view_model_triggers` `:65`; ctor `:810`; threaded through `state_machine.rs:1139,1230,1301,1372,2006,2110,2880,2989,3070,3196`, `transition_conditions.rs:984,1468`; input-trigger used_layers `state_machine.rs:2503-2521`.
- `bind_active_owned_view_model_triggers_for_candidates` remains for identity
  and structural rebind. [x] f9 deleted
  `refresh_active_owned_view_model_triggers_for_candidates`; a retained trigger
  cell supplies steady-frame change state. `sync_default_view_model_triggers_from_active`,
  `reset_bound_trigger_sources`, context-chain binders, and
  `reset_active_view_model_triggers` remain for later trigger-state cleanup.
- PUBLIC trigger reads (survive): `view_model_trigger_count` `instance.rs:5486`, `view_model_trigger_value_count` `:5510`, `view_model_trigger_property_id` `:5514`.

## Public API surface that must keep working

- StateMachineInstance: `bind_owned_view_model_contexts` `:5121`; `bind_owned_view_model_context` `:4800` / `_handle` `:4812` / `_context_handle` `:4817` / `_mut` `:4851`; `bind_default_view_model_context` `:4711`; `bind_view_model_instance_context` `:4737`; `bind_imported_view_model_context` `:4764`; `advance_data_context` `:5316`; `update_data_binds_apply_target_to_source` `:5403`; `set_bindable_*_for_data_bind` `:2557-2782`; `set_default/owned/imported_view_model_*_source_*` + `relink_*` `:2806-4679`; trigger reads `:5486,5510,5514`; `pointer_*_with_owned_view_model_context*` `:1244-1862`.
- ArtboardInstance: `bind_owned_view_model_artboard_contexts` `:4225`; `_context` `:4172` / `_handle` `:4188` / `_context_handle` `:4199`; `bind_default_view_model_artboard_list_context` `:4157`; `bind_owned_view_model_nested_artboard_contexts` `:4434`; `advance_artboard_data_binds` `:5534` / `_with_elapsed` `:5690`; `owned_view_model_context` `artboard.rs:2328`; composite advances `artboard.rs:3008,:3068`; script VM writes `:1498,:1521`; `artboard_list_binding_*_for_data_bind` `:7748-7788`.
- File/facade: `instantiate_view_model` `:2591/3304`; `instantiate_view_model_instance` `:2602/3313`; `instantiate_mutable_view_model_instance` `:2620/3329`; `bind_view_model` `:2646/3345`; `owned_view_model_context` `:2658/3356`; `view_model_index` `:2578/3294`; `advance_with_state_machines_and_view_model` `:2714/3402`; `try_advance_...factory` `:2813/3497`; `ViewModelInstance::{raw,raw_mut,handle}` `:3637-3647`; `RuntimeOwnedViewModelInstance::{new,from_instance,from_instance_mutable,detached_graph,list_items_by_property_name_path}`.
- Scene: `advance` `:15516`; `try_advance_with_factory` `:15582`; `pointer_down/move/up/exit` `:15277,:15334,:15380,:15436`; `view_models` `:8472`; `create_view_model_listener` `:12872`; `add_listener_view_model_*_action` `:13058-13182`; `add_view_model_trigger_condition` `:13645`; `scrub` `:15474`; `instantiate` `:6859`.
- FlowSession: delegates to Scene; no direct family reach.

## Deletion-gate checklist (f)

Safe to remove once the public seams are re-implemented (zero non-test,
non-family callers): `rebind_owned_view_model_context_candidates`,
`refresh_owned_view_model_candidates`,
`refresh_owned_view_model_artboard_context_if_mutated`,
`owned_view_model_context_candidates_for_nested_host`,
`bind_active_owned_view_model_triggers_for_candidates`,
`sync_default_view_model_triggers_from_active`,
`reset_bound_trigger_sources`, `RuntimeArtboardOwnedContextKey` +
`matches_candidate*`, and the `mutation_generation` clock. Already deleted:
`owned_view_model_candidate_generations`,
`refresh_active_owned_view_model_triggers_for_candidates`, and
`linked_aliases`/`synchronize_linked_aliases`.

## Test-rewrite counts

mutation_generation ~9 test sites (rewrite); candidates ~12 (rewrite to new
context type); listener cap 1 (rewritten in f4 to the C++ batch cap);
from_instance_mutable 1 (API
survives); alias sharing 1 (observable contract, keep); Scene dirty 0
(integration coverage remains); per-source copied state: rewrite at graph
API level; triggers: public read API survives, internal rescan tests delete.
