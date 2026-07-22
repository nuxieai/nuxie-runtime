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

- `mutation_generation`: field `view_model.rs:1514`; accessor `:6355`; bump `mark_mutated()` `:6359-6362` (+ `synchronize_linked_aliases`); init `:3464`, `:6645`. Readers: `view_model.rs:2681,2988,2992-2994`; `artboard_data_bind.rs:530,548,567,586,604,696-697`; `artboard.rs:2648-2649,2684-2685,3324,3337`; `instance.rs:5182,5206`. Tests `view_model.rs:11321-11335,11355-11364`; `artboard_data_bind.rs:8274,8289` (rewrite).
- `RuntimeArtboardOwnedContextKey` (VM structure epoch, NOT render-cache): struct `artboard_data_bind.rs:507`, instance key `:511-518`; ctors/comparators `:521,:539,:557,:575,:594`; field set/cleared `:4576,4583,4638-4639,5398`; users `retain_owned_view_model_context_candidates` `:4627-4640`, `refresh_owned_view_model_artboard_context_if_mutated` `:5694-5697`, `:5890`.
- `owned_view_model_candidate_generations`: field `instance.rs:80`; carry `:573-574`; init `:893`; write `:5220`; clear `:5400`; gate read `:5184`.
- `reroot_mutation_clock` `view_model.rs:6423-6428`; callers `:1625,:3013,:3493,:6495,:6662`; related `replace_mutation_clock` `:6430`, `merge_mutation_clock` `:6436` (alias registry `:1787,:2013`), `track_mutation` `:6440`, reset `:6522`.

## Item 2 - Candidate vectors

- `RuntimeOwnedViewModelBindingCandidate` def `artboard_data_bind.rs:610`; ctors `root` `:617`, `root_handle` `:625`, `context_handle` `:633`, `declared_global_slot` `:641`; path helpers `source_path_for_context_path` `:652` (rewrite seam), `context_path_for_source_path` `:677`, `context_chain` `:692`, `mutation_generation` `:696`.
- Ctor call sites: `artboard.rs:2692,2744,5577,10097,10391-10392,10693,10728-10729` (last four tests); `instance.rs:4822,5127,5130`; `artboard_data_bind.rs:4204,4232,4241,4311,4317,4378,4382,4411,4461,4556,5358,5596,7663,7667`.
- SM retained vector `owned_view_model_candidates` `instance.rs:79` (+ key-frame `:129`); rw `:244-245,290,572,892,2248-2257,5174-5194,5391-5395,5399,5678,5828`; listener write `perform_owned_view_model_candidates_change` `:277-299`.
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
- [ ] Context-chain variant `instance.rs:4896-4922` (caller `:5151`) and
  single-context wrapper `:4886-4894` still poll `observed` copies until those
  contexts migrate to the retained candidate path.
- [ ] Remaining `observed` copies: field `instance.rs:649`; init `:860`;
  writes on the compatibility context-chain paths.

## Item 4 - Alias mirrors / detached trees

- `linked_aliases` field `view_model.rs:1529`; struct `:1533`; init `:3009,3490,6660`; `add_linked_alias` `:6365`, `remove_linked_alias` `:6384`, `synchronize_linked_aliases` `:6398`; registration `:1999-2016`, link/unlink `:1785-1801`. Test `view_model.rs:11299-11308` (alias sharing = observable clone contract; rewrite/keep).
- `from_instance_mutable` `view_model.rs:6486-6497` (PUBLIC; callers `crates/nuxie/src/lib.rs:2625,3334`; test `:11341` — API survives, internals replaced); `overlay_active_instance` `:6499+` (only `:6493`); `detach_list_storage` `:6418-6421` (callers `:3012,:3492,:6494`).

## Item 5 - Facade-wide rebind bits

- Scene `dirty` `scene.rs:4715`; init `:5287`; `flush_view_model` `:5389-5413`; consumers `:15534,:15599`; setters `:15327,:15373,:15430,:15462`; inline mount binds `:5260-5298,:5301,:5312,:5358,:5404,:5415,:5454`.
- `refresh_owned_view_model_candidates` `instance.rs:5390-5396`; callers `:5321,:5633,:5747`. Artboard analog `refresh_owned_view_model_artboard_context_if_mutated` `artboard_data_bind.rs:5694` (caller `:5755`).

## Item 6 - Per-source copied state (data_bind_graph.rs)

- `default_value` fields `:74` (target), `:219` (source); copies `:987-992,:4270,:4318-4322,:4427,:4637,:4666,:4747,:4781`; whole `:4744-6262` block is per-type default_value rw.
- `target_origin` field `:206`; set/cleared `:4356,7846,9387,9547,9560,9565,9582,9917`.
- `mark_reconcile_dirty` `:9569-9588`; callers `:4294,4331,4433,4580,4614,4650,4677`.
- `reset_converter_state` `:9906-9909`; batch `:4685`; callers `:4290,4325,4432,4579,4613,4649,4676,4687`. `reset_formula_random_state_for_source_change` `:9915-9924`.
- Reverse-write sibling scans: `bind_owned_view_model_target_to_source_bindings` `artboard_data_bind.rs:4273,5605,5717`; ordering comment `:5706-5719`.

## Item 7 - Trigger observed/rescan state

- `StateMachineViewModelTriggerInstance` `state_machine.rs:2527-2591` (`used_layers` `:2532`; `reset` `:2567,2573,2581`; `is_changed_for_layer` `:2585-2590`); instance fields `default_view_model_triggers` `instance.rs:64`, `view_model_triggers` `:65`; ctor `:810`; threaded through `state_machine.rs:1139,1230,1301,1372,2006,2110,2880,2989,3070,3196`, `transition_conditions.rs:984,1468`; input-trigger used_layers `state_machine.rs:2503-2521`.
- `bind_active_owned_view_model_triggers_for_candidates` `instance.rs:5984-6009` (callers `:5082,:5190`); `refresh_active_...` `:6011-6034` (`:5192`); `sync_default_view_model_triggers_from_active` `:6072-6082` (`:5373,:5874`); `reset_bound_trigger_sources` `data_bind_graph.rs:7786-7826` (`instance.rs:5368`); context-chain binders `:6036,:5149`; `reset_active_view_model_triggers` `:6065` (`:5218`).
- PUBLIC trigger reads (survive): `view_model_trigger_count` `instance.rs:5486`, `view_model_trigger_value_count` `:5510`, `view_model_trigger_property_id` `:5514`.

## Public API surface that must keep working

- StateMachineInstance: `bind_owned_view_model_contexts` `:5121`; `bind_owned_view_model_context` `:4800` / `_handle` `:4812` / `_context_handle` `:4817` / `_mut` `:4851`; `bind_default_view_model_context` `:4711`; `bind_view_model_instance_context` `:4737`; `bind_imported_view_model_context` `:4764`; `advance_data_context` `:5316`; `update_data_binds_apply_target_to_source` `:5403`; `set_bindable_*_for_data_bind` `:2557-2782`; `set_default/owned/imported_view_model_*_source_*` + `relink_*` `:2806-4679`; trigger reads `:5486,5510,5514`; `pointer_*_with_owned_view_model_context*` `:1244-1862`.
- ArtboardInstance: `bind_owned_view_model_artboard_contexts` `:4225`; `_context` `:4172` / `_handle` `:4188` / `_context_handle` `:4199`; `bind_default_view_model_artboard_list_context` `:4157`; `bind_owned_view_model_nested_artboard_contexts` `:4434`; `advance_artboard_data_binds` `:5534` / `_with_elapsed` `:5690`; `owned_view_model_context` `artboard.rs:2328`; composite advances `artboard.rs:3008,:3068`; script VM writes `:1498,:1521`; `artboard_list_binding_*_for_data_bind` `:7748-7788`.
- File/facade: `instantiate_view_model` `:2591/3304`; `instantiate_view_model_instance` `:2602/3313`; `instantiate_mutable_view_model_instance` `:2620/3329`; `bind_view_model` `:2646/3345`; `owned_view_model_context` `:2658/3356`; `view_model_index` `:2578/3294`; `advance_with_state_machines_and_view_model` `:2714/3402`; `try_advance_...factory` `:2813/3497`; `ViewModelInstance::{raw,raw_mut,handle}` `:3637-3647`; `RuntimeOwnedViewModelInstance::{new,from_instance,from_instance_mutable,detached_graph,list_items_by_property_name_path}`.
- Scene: `advance` `:15516`; `try_advance_with_factory` `:15582`; `pointer_down/move/up/exit` `:15277,:15334,:15380,:15436`; `view_models` `:8472`; `create_view_model_listener` `:12872`; `add_listener_view_model_*_action` `:13058-13182`; `add_view_model_trigger_condition` `:13645`; `scrub` `:15474`; `instantiate` `:6859`.
- FlowSession: delegates to Scene; no direct family reach.

## Deletion-gate checklist (f)

Safe to remove once the public seams are re-implemented (zero non-test,
non-family callers): `changed_view_model_listener_actions_for_candidates`,
`rebind_owned_view_model_context_candidates`,
`refresh_owned_view_model_candidates`,
`refresh_owned_view_model_artboard_context_if_mutated`,
`owned_view_model_context_candidates_for_nested_host`,
`bind_active/refresh_active_owned_view_model_triggers_for_candidates`,
`sync_default_view_model_triggers_from_active`,
`reset_bound_trigger_sources`, `RuntimeArtboardOwnedContextKey` +
`matches_candidate*`, `owned_view_model_candidate_generations`,
`linked_aliases`/`synchronize_linked_aliases`, and the
`mutation_generation` clock.

## Test-rewrite counts

mutation_generation ~9 test sites (rewrite); candidates ~12 (rewrite to new
context type); listener cap 1 (rewritten in f4 to the C++ batch cap);
from_instance_mutable 1 (API
survives); alias sharing 1 (observable contract, keep); Scene dirty 0
(integration coverage remains); per-source copied state: rewrite at graph
API level; triggers: public read API survives, internal rescan tests delete.
