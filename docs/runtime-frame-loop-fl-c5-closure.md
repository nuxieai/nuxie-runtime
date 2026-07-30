# StateMachine definition, occurrence, hit, bind, and advance owner-family closure

This is the binding production checklist for the complete pinned-C++ FL-C5
family at `d788e8ec6e8b598526607d6a1e8818e8b637b60c`:

- `include/rive/animation/state_machine.hpp`
- `src/animation/state_machine.cpp`
- `include/rive/animation/state_machine_instance.hpp`
- `src/animation/state_machine_instance.cpp`

It is a draft until every unchecked row has its named proof. FL-C5 is eligible
for whole-family review only when every member row below has:

1. its directive-selected filename-corresponding Rust owner;
2. a live C++ differential, focused unit test, live probe, or an explicit
   source-cited proof where execution is impractical;
3. a permanent checker rule with an injected negative control where the
   requirement is mechanically recognizable;
4. its construction, ownership, ordering, callback, queue, floating-point,
   clone, and teardown behavior covered; and
5. all W4 section C public APIs still reachable through the thin entry points.

The complete family is one publication unit. A subset does not close either
whole C++ source row. No performance measurement belongs in this family.

The already separately handled FL-C1 claimed-`DataBindPath` correction (W6) is
not part of FL-C5 and is not claimed here. The branch history interleaves the
following separately audited FL-C1 family commits:

- `82e229f3` resolves relative claimed paths by name;
- `69fee252` preserves unmapped name IDs as empty path names;
- `20cd8c02` makes the claimed-path probes live differentials;
- `efd87746` pins the per-step firing boundary and is the independently
  accepted FL-C1 family candidate after six audit rounds; and
- `e67ba822` promotes only that already accepted input/listener family.

Their acceptance is recorded in `docs/runtime-frame-loop-status.md` and
`docs/parity-closeout-status.md`. They are ancestry, not FL-C5 package work:
the FL-C5 implementation range itself did not take ownership of or promote an
FL-C1 owner. This clarification resolves the W33 S6 history-range ambiguity
without reclassifying the separately accepted work.

## Binding Rust ownership

| Pinned C++ owner | Binding Rust owner | Closure rule |
| --- | --- | --- |
| `state_machine.hpp/.cpp` | `crates/nuxie-runtime/src/state_machine/state_machine.rs` (new) | Own the complete definition/import adaptation. Existing `crates/nuxie-runtime/src/state_machine.rs` becomes a thin entry point and re-export surface. |
| `state_machine_instance.hpp/.cpp` | `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (new) | Own the instance, hit hierarchy, bind/event/advance orchestration, and C++-corresponding lifecycle. Existing `crates/nuxie-runtime/src/state_machine/instance.rs` becomes a thin entry point and re-export surface. |
| Private `StateMachineLayerInstance` in `state_machine_instance.cpp:140-711` | `crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs` | Remains the accepted FL-C3 owner. FL-C5 changes it only for W4-confirmed per-layer state-change/current-state divergence. |
| Artboard borrow-model call sites | `crates/nuxie-runtime/src/artboard.rs` | Thin delegating wrappers only. The implementation of `advance`, both `advanceAndApply` shapes, and the settlement policy belongs to the new instance owner. |

## Proof-key catalog

Member rows use these compact proof keys. A key is not satisfied by a test
name alone: its stated cases and the pinned source citations must be present
in the test/probe receipt.

### Behavioral proof keys

| Key | Required proof |
| --- | --- |
| `B-DEF` | Focused definition/import tests plus a pinned-source comparison for authored collection order, duplicates/null slots, first-match lookup, count/index behavior, and dirty/clean phase status order. |
| `B-LAYER` | Existing FL-C3 differentials, extended only for retained per-layer changed flags, `currentState`, `layerState`, and the W2 transition/FP adversarial cases named by the row. |
| `B-CTOR` | Constructor-order differential proving inputs → layers/entry callbacks → ordinary binds → authored listener groups → provider/nested/list/text hits → scripted clones/facilities → hit sort → focus tree. |
| `B-HIT` | Pinned-C++ pointer differential returning tri-state `none/hit/hitOpaque`, covering reset → prepare → process, `canHit`, exit cleanup, draw-order re-sort, nested/list reverse routing, and drag enable/disable. |
| `B-LVM` | Listener-ViewModel focused tests for binding/relink/clear order, duplicate cells, deferred reporting, and trigger-zero suppression. |
| `B-ADV` | Pinned-C++ differential for raw `advance(seconds,newFrame)` and both facade overloads, including zero, signed zero, NaN/infinities, queue terms, five settlement passes, unconditional probing, and input `advanced()`. |
| `B-BIND` | Pinned-C++ differential for the complete bind family and every distinct null branch, including the inherited-context prior-registration hazard. |
| `B-EVENT` | Pinned-C++ differential for report queues, current-versus-pending visibility, exact 100-batch boundary, listener-major dispatch, bubbling, trigger reset suppression, and the recorded audio seam. |
| `B-KEY` | Focused unit tests and live source-value probes for first source bind, holder type, initialize/converter/enroll order, already-bound context, duplicate build, removal order, and teardown. |
| `B-FOCUS` | Focus/semantic orchestration differential limited to members defined in the pinned files; manager internals are source-cited recorded seams. |
| `B-LIFE` | Focused disposal/drop/snapshot/remount tests proving explicit nested detach, observable teardown order, and non-aliasing snapshot `Clone`. |
| `B-API` | Compile-time/downstream test proving every W4 §C public name remains reachable with unchanged signature/visibility through re-export. |
| `B-SOURCE` | Exact pinned-source citation used only for compile-time, profiler-only, deleted, undefined, or intentionally empty members that cannot produce a meaningful runtime differential. |

### Structural proof keys

| Key | Checker rule and injected negative |
| --- | --- |
| `S-OWNER` | Require both new owner files; require old entry files to contain only module declarations, imports needed for re-export, re-exports, and compatibility type aliases/wrappers. Inject a displaced implementation into each entry point and require failure. |
| `S-SLOTS` | Reject `filter_map`/flattening/deduplication of authored layer/input/listener/bind/script occurrence collections. Inject one dropped malformed listener and require failure. |
| `S-ORDER` | Reject map/set/sort/reconstructed candidates where pinned authored traversal is required and reject per-advance reconstruction. Inject each forbidden shape. |
| `S-LAYER` | Require retained per-layer changed flag and current-state access; reject cached-only count, missing `stateChangedByIndex`, and StateMachineInstance-level aliases for the three stale private transition declarations. |
| `S-HIT` | Require trait plus all six concrete hit types, tri-state `HitResult`, three distinct passes, `canHit` propagation, exit release, draw-order counter check, reverse list traversal, and enable/disable walks. Inject removal/wrong order/wrong boolean return controls. |
| `S-BIND` | Require the named C++ bind-family primary methods and their ordered calls; reject typed APIs that bypass rather than delegate to the primary family. Inject swapped artboard/machine bind order and collapsed null branches. |
| `S-EVENT` | Require two event and two listener-VM batch states (or documented cursor adaptation), events-before-VM order, the literal 100 bound, trigger-zero guard, bubbling-before-audio seam, and `playsAudio == true`. |
| `S-ADV` | Reject clean zero-delta early return, capability-gated settlement probes, finite rejection inside C++-corresponding paths, and return expressions omitting pending event/VM terms or exact-zero forcing. |
| `S-LIFE` | Require explicit dispose/detach path, observable Drop-order comments/tests, and non-aliasing `Clone`; reject shallow sharing of mutable queues, registrations, pointer state, layer trigger IDs, or script tables. |
| `S-KEY` | Require first-bind selection, supported typed holders, initialize-before-converter/enrollment order, state-owned tracking, and removal-before-state drop. |
| `S-SEAM` | Require every out-of-scope dependency to carry `RECORDED` plus its owning ledger row; reject an FL-C5 “faithful” claim for the deferred implementation. |
| `S-API` | Generate/check the complete W4 §C public-name inventory against the thin re-export surfaces. Inject removal or visibility reduction of every public group. |
| `S-FIELDS` | Require explicit Rust initialization/ownership for every C++ field without a header initializer and preserve queue/collection identities; reject semantically meaningful `Default` sentinels for queued events. |
| `S-TOOLS` | Gate testing/tools-only members under their corresponding Rust feature/test surface or record a source-cited non-public adaptation; do not silently promote them to ordinary production API. |

## Landed proof receipts

The 294 member rows below are checked only through these landed proof
receipts. Each row retains its exact pinned-C++ member citation in the first
column, its behavioral key in the third column, and its structural key(s) in
the fourth column. `[x] CLOSED` means those cited receipts cover the row; the
following `W4:` text preserves the pre-FL-C5 audit status rather than
rewriting history.

| Proof key | Landed behavioral receipt and source citation |
| --- | --- |
| `B-DEF` | `fl_c5_definition_*` (5 focused tests), `fl_c5_state_machine_definition_authored_collections_match_cpp` (a full same-byte definition comparison plus a separate safe authored-null definition seam), `fl_c5_typed_named_inputs_match_cpp_with_an_earlier_wrong_type`, and the exact malformed-listener and added-phase probes cover `state_machine.cpp:12-165`, `state_machine.hpp:20-55`, and typed instance lookup at `state_machine_instance.cpp:2689-2714`. |
| `B-LAYER` | `fl_c5_state_changed_*` (5 focused tests), `fl_c5_state_changed_layers_and_convergence_match_cpp_probe`, `fl_c5_current_state_and_animation_authored_compression_match_cpp_probe`, and `fl_c5_random_transition_edges_weighted_boundaries_and_wraparound_match_cpp_probe` cover `state_machine_instance.cpp:140-711`. The accepted FL-C3 transition differentials remain the base proof for unchanged private-layer behavior. |
| `B-CTOR` | `fl_c5_constructor_order_phase_trace_and_explicit_fields`, `fl_c5_constructor_order_retains_unresolved_pointer_group_occurrence`, and `fl_c5_constructor_order_source_and_runtime_boundaries_match_cpp` cover `state_machine_instance.cpp:1707-2128`. |
| `B-HIT` | `fl_c5_hit_*` and `fl_c5_pointer_*` focused tests, including `fl_c5_hit_sort_preserves_the_exact_adversarial_swap_order`, plus the shared-target, nested-routing, reverse-component-list, and nonfinite pointer probes cover `state_machine_instance.hpp:479-505` and `.cpp:712-1616,2255-2318,3173-3187`. |
| `B-LVM` | `fl_c5_event_trigger_zero_suppression_and_duplicate_listener_fifo`, `fl_c5_event_mid_callback_visibility_excludes_the_reporting_snapshot`, the bind/relink focused tests, and the trigger-zero/mid-callback pinned probes cover `.cpp:1324-1545,2320-2344,3021-3060`. |
| `B-ADV` | The `fl_c5_advance_*` focused tests and pinned probes (`fl_c5_raw_advance_order_matches_pinned_cpp`, `fl_c5_advance_return_terms`, `fl_c5_advance_fp_matrix`, `fl_c5_advance_view_models_false`, the live two-runtime `fl_c5_five_pass_unconditional_probe`, and `fl_c5_zero_delta_bookkeeping`) cover `.cpp:2546-2668`. The public scripted-mount differential additionally proves immediate `pointer_down` and `advance_and_apply` are not suppressed by facade preparation state. |
| `B-BIND` | The seven `fl_c5_bind_*` focused tests and four bind-family pinned probes cover `.cpp:2716-2976`, including the inherited A→B prior-registration hazard. |
| `B-EVENT` | The `fl_c5_event_*` focused tests, including `fl_c5_event_bubbling_precedes_the_recorded_audio_seam_through_two_ancestors` with one ordinary Event and one AudioEvent, and the chaining, 100-batch, trigger-zero, visibility, live-projection, and bubbling probes cover `.cpp:2320-2344,3016-3187`. The nested-relative claimed-path differential is strict per-step equality. |
| `B-KEY` | The `fl_c5_keyframe_data_bind_*` focused tests and the first-source, bound-context, supported-holder/removal, source-order, and live `fl_c5_keyframe_initialize_converter_and_enrollment_are_observed_end_to_end` probes cover `.cpp:3189-3390`. |
| `B-FOCUS` | The `fl_c5_focus_semantic_*` focused tests, including the distinct manager-node-ID → SemanticData-local-ID lookup, action 0/1/2/invalid switch, and observable semantic-listener callback in `semantic_callbacks_apply_constraints_preserve_duplicates_and_defer_actions`, plus the focus/semantic queue, manager-switch, and `FocusState` probes cover only `.cpp:2346-2544,3392-3418`; manager/data internals remain `RECORDED` below. |
| `B-LIFE` | `fl_c5_clone_teardown_rebuilds_mutable_state_without_aliasing` now exercises report/current/bubble queues, listener reports, pointer state, hit owners, listener groups, nested registrations, detached primary context state, callback dirt sinks, layers, and cold script tables; `fl_c5_clone_teardown_dispose_is_repeatable_and_drop_order_is_observable`, `fl_c5_dispose_nested_event_source_order_and_rust_idempotence`, and the constructor/disposal source probe cover `.cpp:1707-2243`. |
| `B-API` | The downstream `fl_c5_public_reexports_are_downstream_visible_after_file_split` contains 328 exact typed coercions covering every W4 §C item—including the generic hydration functions—so receiver, parameter, return, ownership, or visibility changes fail compilation. The structural checker requires the real coercion body, exact inventory count, and SHA-256 digest of the complete exhaustive signature block; its count-preserving still-compiling substitution negative proves a missing signature cannot hide behind a duplicate. `cargo test -p nuxie --test public_api` supplies the facade receipt. |
| `B-SOURCE` | The exact member citation in each such row is the proof for profiler-only, deleted, undefined, test/tools-only, intentionally empty, constant, or out-of-scope members. `S-TOOLS`, `S-SEAM`, and the public API inventory prevent those source-only decisions from becoming silent production claims. |

All structural keys are exercised by the runtime-frame-loop checker. Its
FL-C5 tests inject forbidden source for definition slots/owners, lifecycle and
field order, per-layer state, the complete hit hierarchy, bind branches,
event batches/audio seams, advance/FP/probe rules, keyframe lifecycle,
focus/semantic seams, both thin entry points, and the public export hubs. The
compile-time W4 inventory is the visibility/signature guard for every
individual public method group.

## Complete member closure

The status column records closure first and preserves the W4 audit status
after `W4:`. “Binding adaptation” in a proof cell means the directives approve
the Rust ownership shape but not omission of the observable behavior.

### `StateMachine` definition and `state_machine.cpp`

Rust destination for every row in this table:
`crates/nuxie-runtime/src/state_machine/state_machine.rs`.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W1–W4 |
| --- | --- | --- | --- | --- |
| `m_Layers` (`state_machine.hpp:20`) | [x] CLOSED; W4: faithful-looking | `B-DEF`: authored order and duplicate definitions | `S-OWNER`, `S-SLOTS`, `S-ORDER` | Empty vector; duplicate names/pointers retained. |
| `m_Inputs` (`state_machine.hpp:21`) | [x] CLOSED; W4: faithful-looking | `B-DEF`: indexed `None`/null compatibility slot | `S-SLOTS`, `S-FIELDS` | Unknown serialized input creates a hole and still counts. |
| `m_Listeners` (`state_machine.hpp:22`) | [x] CLOSED; W4: divergent | `B-DEF`: inert unbuildable listener retains authored index | `S-SLOTS` | Malformed listener followed by valid listener; no index compaction. |
| `m_dataBinds` (`state_machine.hpp:23`) | [x] CLOSED; W4: scattered | `B-DEF`: one authored occurrence order projected into graph/container queue | `S-ORDER`, `S-FIELDS` | Duplicate bind targets retained even when lookup maps select first/last as specified. |
| `m_scriptedObjects` (`state_machine.hpp:24`) | [x] CLOSED; W4: faithful-looking | `B-DEF`: borrowed occurrence order and returned-list independence | `S-ORDER`, `S-FIELDS` | Duplicate source pointer entries retained; clearing returned list does not mutate owner. |
| `addLayer` (`state_machine.hpp:26`; `state_machine.cpp:82-85`) | [x] CLOSED; W4: scattered | `B-DEF`: import-time append adaptation | `S-SLOTS`, `S-ORDER` | Null and duplicates append without validation. |
| `addInput` (`state_machine.hpp:27`; `state_machine.cpp:87-90`) | [x] CLOSED; W4: scattered | `B-DEF`: import-time append/null-object adaptation | `S-SLOTS` | Null compatibility input remains at exact index. |
| `addListener` (`state_machine.hpp:28`; `state_machine.cpp:92-95`) | [x] CLOSED; W4: divergent | `B-DEF`: every authored listener gets a slot | `S-SLOTS` | Unbuildable listener is inert, never filtered. |
| `addDataBind` (`state_machine.hpp:29`; `state_machine.cpp:97-100`) | [x] CLOSED; W4: scattered | `B-DEF`: authored append represented once | `S-ORDER` | Null/duplicate occurrence is not silently deduplicated. |
| constructor (`state_machine.hpp:32`; `state_machine.cpp:12`) | [x] CLOSED; W4: scattered | `B-DEF`: zero-member machine and complete immutable-build adaptation | `S-OWNER`, `S-FIELDS` | No synthesized layers/inputs/listeners/binds/scripts. |
| destructor (`state_machine.hpp:33`; `state_machine.cpp:14`) | [x] CLOSED; W4: faithful-looking | `B-SOURCE`: RAII owner/borrow split | `S-FIELDS` | Same definition cannot be independently owned by two collections. |
| `import` (`state_machine.hpp:35`; `state_machine.cpp:70-80`) | [x] CLOSED; W4: divergent | `B-DEF`: missing-artboard status and superclass/import adaptation | `S-SEAM`, `S-ORDER` | Missing importer returns failure and does not attach; parse order retained. |
| `layerCount` (`state_machine.hpp:37`) | [x] CLOSED; W4: faithful-looking | `B-DEF`: on-demand length | `S-SLOTS` | Zero and duplicate layers. |
| `inputCount` (`state_machine.hpp:38`) | [x] CLOSED; W4: faithful-looking | `B-DEF`: counts null slot | `S-SLOTS` | One compatibility hole yields one. |
| `listenerCount` (`state_machine.hpp:39`) | [x] CLOSED; W4: divergent | `B-DEF`: authored count including inert slot | `S-SLOTS` | Malformed then valid yields count two. |
| `dataBindCount` (`state_machine.hpp:40`) | [x] CLOSED; W4: scattered | `B-DEF`: definition-level authored occurrence count | `S-ORDER` | Typed decomposition must not change count. |
| `addScriptedObject` (`state_machine.hpp:41`; `state_machine.cpp:162-165`) | [x] CLOSED; W4: scattered | `B-DEF`: import collection adaptation | `S-ORDER` | Same borrowed pointer appended twice. |
| `scriptedObjects` (`state_machine.hpp:42-45`) | [x] CLOSED; W4: faithful-looking | `B-DEF`: complete ordered borrowed view | `S-API`, `S-ORDER` | Caller mutation cannot mutate definition. |
| `input(name)` (`state_machine.hpp:47`; `state_machine.cpp:102-112`) | [x] CLOSED; W4: scattered | `B-DEF`: exact, case-sensitive, first match | `S-API`, `S-ORDER` | Duplicate name, absent name, leading null-slot crash/source-cited malformed behavior. |
| `input(index)` (`state_machine.hpp:48`; `state_machine.cpp:114-121`) | [x] CLOSED; W4: faithful-looking | `B-DEF`: in-range null and out-of-range `None` | `S-API` | `index == count`, `SIZE_MAX`, null slot. |
| `layer(name)` (`state_machine.hpp:49`; `state_machine.cpp:123-133`) | [x] CLOSED; W4: scattered | `B-DEF`: exact first match | `S-API`, `S-ORDER` | Duplicate names and case-only mismatch. |
| `layer(index)` (`state_machine.hpp:50`; `state_machine.cpp:135-142`) | [x] CLOSED; W4: faithful-looking | `B-DEF`: indexed optional adaptation | `S-API` | Empty machine and `SIZE_MAX`. |
| `dataBind(index)` (`state_machine.hpp:51`; `state_machine.cpp:153-160`) | [x] CLOSED; W4: scattered | `B-DEF`: polymorphic-index adaptation | `S-API`, `S-ORDER` | `index == count`; duplicate targets remain separately addressable. |
| `listener(index)` (`state_machine.hpp:52`; `state_machine.cpp:144-151`) | [x] CLOSED; W4: divergent | `B-DEF`: authored-index lookup after inert-slot retention | `S-SLOTS`, `S-API` | Malformed slot before valid listener; `SIZE_MAX`. |
| `onAddedDirty` (`state_machine.hpp:54`; `state_machine.cpp:16-41`) | [x] CLOSED; W4: missing | `B-DEF`: inputs → layers → listeners and first-error stop | `S-ORDER`, `S-SEAM` | Input 2 failure blocks all layers/listeners; null input malformed crash; invalid layer triplet. |
| `onAddedClean` (`state_machine.hpp:55`; `state_machine.cpp:43-68`) | [x] CLOSED; W4: missing | `B-DEF`: same phase order and first-error stop | `S-ORDER`, `S-SEAM` | Layer clean failure blocks later layers and all listeners. |

### File statics and private `StateMachineLayerInstance`

The accepted destination is
`crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs`,
except profiler-only `getStateName`, which may be a source-cited omission.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W2/W4 |
| --- | --- | --- | --- | --- |
| `getStateName` (`state_machine_instance.cpp:95-120`) | [x] CLOSED; W4: missing | `B-SOURCE`: profiler-only labels | `S-SEAM` | Null instance, null animation, unknown subtype → `"Blend"`. |
| `kPointerHitListenerTypes` (`state_machine_instance.cpp:127-137`) | [x] CLOSED; W4: scattered | `B-HIT`: exact nine-member classification | `S-HIT` | Drag-start-only is pointer; component-provided is not; pointer+focus remains pointer-capable. |
| `maxIterations` (`state_machine_instance.cpp:686-687`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: 101-success transition guard | `S-LAYER` | Updates 0…100 execute; 101st success stops. |
| `m_stateMachineInstance` (`state_machine_instance.cpp:688`) | [x] CLOSED; W4: scattered | `B-LAYER`: index/argument identity adaptation | `S-FIELDS` | No observable null/default owner. |
| `m_layer` (`state_machine_instance.cpp:689`) | [x] CLOSED; W4: scattered | `B-LAYER`: stable layer definition handle | `S-FIELDS` | Release-mode re-init cannot invent a second owner. |
| `m_artboardInstance` (`state_machine_instance.cpp:690`) | [x] CLOSED; W4: scattered | `B-LAYER`: explicit operation borrow | `S-FIELDS` | Null artboard remains malformed, not a default scene. |
| `m_anyStateInstance` (`state_machine_instance.cpp:692`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: Any built before Entry | `S-FIELDS`, `S-ORDER` | Aliasing with current/source and reset teardown. |
| `m_currentState` (`state_machine_instance.cpp:693`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: retained optional occurrence | `S-FIELDS`, `S-LAYER` | Null destination/current; same-state no-op. |
| `m_stateFrom` (`state_machine_instance.cpp:694`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: interrupted-transition source lifetime | `S-FIELDS` | Alias with Any/current; older source retirement. |
| `m_transition` (`state_machine_instance.cpp:696`) | [x] CLOSED; W4: scattered | `B-LAYER`: active definition handle | `S-FIELDS` | Reset preserves the pinned stale transition state. |
| `m_transitionDurationProperty` (`state_machine_instance.cpp:697`) | [x] CLOSED; W4: scattered | `B-LAYER`: occurrence-local bound duration | `S-FIELDS` | Negative, fractional, NaN/infinity/out-of-range conversion source behavior. |
| `m_animationReset` (`state_machine_instance.cpp:698`) | [x] CLOSED; W4: scattered | `B-LAYER`: release/clear timing | `S-FIELDS` | Repeated clear and interruption before completion. |
| `m_transitionCompleted` (`state_machine_instance.cpp:699`) | [x] CLOSED; W4: scattered | `B-LAYER`: end callbacks once | `S-FIELDS` | Already-complete mix with false latch. |
| `m_holdAnimationFrom` (`state_machine_instance.cpp:701`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: pause-on-exit hold flag | `S-FIELDS` | Reset/interruption with pending held animation. |
| `m_mix` (`state_machine_instance.cpp:703`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: exact clamp/NaN differential | `S-FIELDS` | Negative seconds, infinities, NaN, signed zero. |
| `m_mixFrom` (`state_machine_instance.cpp:704`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: interrupted partial-mix carry | `S-FIELDS` | Early interruption at partial mix. |
| `m_stateMachineChangedOnAdvance` (`state_machine_instance.cpp:705`) | [x] CLOSED; W4: scattered | `B-LAYER`: persist on same-frame follow-up, reset on new frame | `S-LAYER` | Multiple transitions in one layer still one changed layer. |
| `m_waitingForExit` (`state_machine_instance.cpp:707`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: waiting propagation | `S-FIELDS` | Any waits while current transition succeeds. |
| `m_holdAnimation` (`state_machine_instance.cpp:708`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: one-shot apply then clear | `S-FIELDS` | Held animation plus reset. |
| `m_holdTime` (`state_machine_instance.cpp:709-710`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: unvalidated spilled/hold time | `S-FIELDS` | NaN/infinite/negative spilled time. |
| destructor (`state_machine_instance.cpp:143-148`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`, `B-SOURCE`: owner-drop order | `S-FIELDS` | Aliased Any/current/source source-cited double-delete hazard. |
| `init` (`state_machine_instance.cpp:150-175`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: per-layer RNG seed, Any bind, Entry | `S-ORDER` | Two-layer deterministic seed; null Any; release-mode re-init. |
| `resetState` (`state_machine_instance.cpp:177-192`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: exact retained stale mix/hold behavior | `S-LAYER` | `stateFrom == current`, current==Any, active transition reset. |
| `updateMix` (`state_machine_instance.cpp:194-223`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: completion/callback/FP differential | `S-ORDER` | Zero mix time; negative/NaN seconds; completion latch. |
| `advance` (`state_machine_instance.cpp:225-278`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: current → mix → source → apply → chained transitions | `S-ORDER`, `S-LAYER` | 101 transitions; null current; `newFrame=false` after change. |
| `resolvedDuration` (`state_machine_instance.cpp:283-291`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: rounded/clamped bound value | `S-FIELDS` | `-0.1`, `0.5`, `1.5`, NaN, infinity, `>UINT32_MAX`. |
| `resolvedMixTime` (`state_machine_instance.cpp:294-316`) | [x] CLOSED; W4: scattered | `B-LAYER`: percent/ms conversion | `S-FIELDS` | Blend source, null animation, infinite animation duration. |
| `isTransitioning` (`state_machine_instance.cpp:318-322`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: source+duration+mix predicate | `S-LAYER` | Null source, mix one, NaN mix. |
| `updateState` (`state_machine_instance.cpp:324-341`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: early-exit gate; Any before current | `S-ORDER` | Both sources allowed; Any waiting/current allowed. |
| `fireEvents` (`state_machine_instance.cpp:343-353`) | [x] CLOSED; W4: scattered | `B-LAYER`: occurrence filter/FIFO | `S-ORDER` | Duplicate pointer; null source-cited malformed action; mixed occurrences. |
| `performListenerActions` (`state_machine_instance.cpp:355-367`) | [x] CLOSED; W4: scattered | `B-LAYER`: matching FIFO and terminal Rust error adaptation | `S-ORDER` | Duplicate actions; interleaved start/end occurrences. |
| `canChangeState` (`state_machine_instance.cpp:369-374`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: definition-identity self guard | `S-LAYER` | Same pointer, equivalent distinct state, null/null. |
| `randomValue` (`state_machine_instance.cpp:376`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: one RNG draw iff positive total | `S-LAYER` | Inject negative, NaN, and exactly one. |
| `changeState` (`state_machine_instance.cpp:378-410`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: outgoing end → construct/binds → incoming start | `S-ORDER` | Repeated direct call, null destination, null make-instance. |
| `findRandomTransition` (`state_machine_instance.cpp:412-468`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: authored weighted scan and strict boundary | `S-ORDER` | Wrapping total; RNG 0/boundary/1/NaN; all waiting. |
| `findAllowedTransition` (`state_machine_instance.cpp:470-509`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: first allowed authored transition | `S-ORDER` | Stale self weight; denied then allowed; waiting then allowed. |
| `buildAnimationResetForTransition` (`state_machine_instance.cpp:511-517`) | [x] CLOSED; W4: scattered | `B-LAYER`: replacement factory call | `S-FIELDS` | Null source/current; null factory result. |
| `clearAnimationReset` (`state_machine_instance.cpp:519-526`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: release then null | `S-FIELDS` | Repeated clear; replacement before completion. |
| `tryChangeState` (`state_machine_instance.cpp:528-630`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: full transition/callback/retirement ordering | `S-ORDER` | Partial interruption; zero duration; invalid exit cast; Any source; null destination. |
| `apply` (`state_machine_instance.cpp:632-663`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: reset → held → outgoing → current | `S-ORDER` | Null interpolator, NaN mix, null current. |
| `stateChangedOnAdvance` (`state_machine_instance.cpp:665-668`) | [x] CLOSED; W4: scattered | `B-LAYER`: retained flag, not aggregate-only | `S-LAYER` | Query after same-frame convergence. |
| `currentState` (`state_machine_instance.cpp:670-673`) | [x] CLOSED — `state_machine_generic_layer_state_occurrence_matches_cpp_probe` now matches pinned C++ core type `60`; W4: missing | `B-LAYER`: borrowed optional definition | `S-LAYER`, `S-API` | Null current. |
| `currentAnimation` (`state_machine_instance.cpp:675-684`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: animation-only compressed view | `S-LAYER` | Blend/current null; null animation occurrence. |
| `evaluatedRandomWeight` shared scratch use (`state_machine_instance.cpp:428-456`) | [x] CLOSED; W4: divergent, approved adaptation | `B-LAYER`: equal results plus two-instance race isolation | `S-FIELDS` | Duplicate transitions, wraparound, concurrent instances. |

### Hit-component hierarchy

Rust destination for every row in this table:
`crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W2/W4 |
| --- | --- | --- | --- | --- |
| `HitComponent::m_component` (`state_machine_instance.hpp:504`) | [x] CLOSED; W4: scattered | `B-HIT`: stable optional component identity | `S-HIT`, `S-FIELDS` | Nullable component only where subclass permits it. |
| `HitComponent::m_stateMachineInstance` (`state_machine_instance.hpp:505`) | [x] CLOSED; W4: scattered | `B-HIT`: operation-borrow adaptation | `S-HIT`, `S-FIELDS` | No shallow owner back-pointer alias in snapshots. |
| `HitComponent::component` (`state_machine_instance.hpp:479`) | [x] CLOSED; W4: scattered | `B-HIT`: exact identity used by sorting | `S-HIT` | Null/custom component and duplicate target. |
| `HitComponent` constructor (`state_machine_instance.hpp:480-483`) | [x] CLOSED; W4: divergent | `B-HIT`: every concrete category constructible | `S-HIT` | Layout, shape, text, nested artboard, component list, provider target. |
| virtual destructor (`state_machine_instance.hpp:484`) | [x] CLOSED; W4: scattered | `B-SOURCE`: trait-object drop | `S-HIT`, `S-LIFE` | Wrapper never owns nested target. |
| `processEvent` (`state_machine_instance.hpp:485-489`) | [x] CLOSED; W4: divergent | `B-HIT`: tri-state dispatch | `S-HIT` | Opaque front target still sends cleanup with `canHit=false` behind it. |
| `processGamepadInvocation` (`state_machine_instance.hpp:490-492`) | [x] CLOSED; W4: scattered | `B-HIT`: nested/list broadcast aggregation | `S-HIT` | Nested opaque result; component-list early opacity. |
| `prepareEvent` (`state_machine_instance.hpp:493-495`) | [x] CLOSED; W4: divergent | `B-HIT`: separate complete prepare pass | `S-HIT` | Shared target hit-tested once; duplicate listeners all receive hover. |
| `hitTest` (`state_machine_instance.hpp:496`) | [x] CLOSED; W4: divergent | `B-HIT`: full component geometry | `S-HIT` | Singular transforms, hidden/collapsed ancestors, raw unpainted shape path. |
| base `enablePointerEvents` (`state_machine_instance.hpp:497`) | [x] CLOSED; W4: missing | `B-HIT`: base no-op and concrete walks | `S-HIT` | Nested wrapper no-op; shared group across hit owners. |
| base `disablePointerEvents` (`state_machine_instance.hpp:498`) | [x] CLOSED; W4: missing | `B-HIT`: base no-op and concrete walks | `S-HIT` | Disable pointer 1 while pointer 2 active. |
| testing `earlyOutCount` (`state_machine_instance.hpp:500`) | [x] CLOSED; W4: missing | `B-SOURCE` or test-only counter parity | `S-TOOLS` | Repeated early-out events. |
| `HitDrawable::hitRadius` (`state_machine_instance.cpp:732`) | [x] CLOSED; W4: missing | `B-SOURCE`: value 2, unused by base | `S-HIT` | Concrete shape/text hard-code radius 2. |
| `HitDrawable::isHovered` (`state_machine_instance.cpp:733`) | [x] CLOSED; W4: divergent | `B-HIT`: one transient per hit owner | `S-HIT` | Multiple pointer IDs and stale value after early-out. |
| `HitDrawable::canEarlyOut` (`state_machine_instance.cpp:734`) | [x] CLOSED; W4: divergent | `B-HIT`: aggregate monotonic flag | `S-HIT` | Enter disables; opacity disables; later mutation does not recompute. |
| `HitDrawable::needsDownListener` (`state_machine_instance.cpp:735`) | [x] CLOSED; W4: divergent | `B-HIT`: aggregate down need | `S-HIT` | Click-only requires down and up. |
| `HitDrawable::needsUpListener` (`state_machine_instance.cpp:736`) | [x] CLOSED; W4: divergent | `B-HIT`: aggregate up need | `S-HIT` | Up-only target after prior hover. |
| `HitDrawable::isOpaque` (`state_machine_instance.cpp:737`) | [x] CLOSED; W4: divergent | `B-HIT`: explicit/provider opacity | `S-HIT` | Reused layout upgrades; shape provider opacity remains discarded. |
| `HitDrawable::m_drawable` (`state_machine_instance.cpp:738`) | [x] CLOSED; W4: divergent | `B-HIT`: drawable identity/dynamic opacity | `S-HIT` | Opacity changes after construction. |
| `HitDrawable::listeners` (`state_machine_instance.cpp:739`) | [x] CLOSED; W4: divergent | `B-HIT`: ordered duplicates on shared owner | `S-HIT`, `S-ORDER` | Same group appended twice; consumed first occurrence skips later. |
| `HitDrawable` constructor (`state_machine_instance.cpp:719-731`) | [x] CLOSED; W4: divergent | `B-HIT`: target opacity disables early-out | `S-HIT` | Null malformed drawable; opacity mutation. |
| base `hitTest` (`state_machine_instance.cpp:741`) | [x] CLOSED; W4: missing | `B-SOURCE`: always false | `S-HIT` | Concrete subclass without override. |
| `prepareEvent` (`state_machine_instance.cpp:743-767`) | [x] CLOSED; W4: divergent | `B-HIT`: early-out/exit/hover broadcast | `S-HIT` | Exit avoids geometry; duplicate idempotent hover calls. |
| `processGamepadInvocation` (`state_machine_instance.cpp:769-774`) | [x] CLOSED; W4: scattered | `B-HIT`: ordinary drawable returns none | `S-HIT` | Script-aware behavior must use its owning wrapper. |
| `processEvent` (`state_machine_instance.cpp:776-818`) | [x] CLOSED; W4: divergent | `B-HIT`: unconsumed ordered groups and opacity | `S-HIT` | Hover without action still hit; scroll makes opaque; occluded target none. |
| `addListener` (`state_machine_instance.cpp:820-838`) | [x] CLOSED; W4: divergent | `B-HIT`: aggregate flags then append | `S-HIT`, `S-ORDER` | Duplicate group; click-only; enter listener. |
| `enablePointerEvents` (`state_machine_instance.cpp:840-846`) | [x] CLOSED; W4: missing | `B-HIT`: ordered duplicate-preserving group walk | `S-HIT` | One pointer enable after other pointer consumed. |
| `disablePointerEvents` (`state_machine_instance.cpp:848-854`) | [x] CLOSED; W4: missing | `B-HIT`: ordered duplicate-preserving group walk | `S-HIT` | Disable shared group through multiple hit owners. |
| `HitExpandable` constructor (`state_machine_instance.cpp:861-866`) | [x] CLOSED; W4: scattered | `B-HIT`: drawable/component may differ | `S-HIT` | Text run uses owner text drawable. |
| `HitExpandable::hitTest` (`state_machine_instance.cpp:868-871`) | [x] CLOSED; W4: divergent | `B-HIT`: component hit test flags | `S-HIT` | No-paint shape, clipped ancestor, singular transform. |
| `HitTextRun` constructor (`state_machine_instance.cpp:877-887`) | [x] CLOSED; W4: missing | `B-HIT`: owner drawable plus run hit-target flag | `S-HIT` | Null run; reused run; flag remains after listener removal. |
| `HitLayout` constructor (`state_machine_instance.cpp:893-897`) | [x] CLOSED; W4: missing | `B-HIT`: same drawable/component | `S-HIT` | Proxy/layout/null malformed target. |
| `HitLayout::hitTest` (`state_machine_instance.cpp:899-902`) | [x] CLOSED; W4: missing | `B-HIT`: layout bounds participate | `S-HIT` | Outside unclipped bounds, hidden, singular, frame origin. |
| `HitNestedArtboard` constructor (`state_machine_instance.cpp:908-911`) | [x] CLOSED; W4: missing | `B-HIT`: borrowed nested target wrapper | `S-HIT` | Wrong component subtype source-cited malformed case. |
| `HitNestedArtboard` destructor (`state_machine_instance.cpp:912`) | [x] CLOSED; W4: missing | `B-SOURCE`: empty wrapper drop | `S-HIT`, `S-LIFE` | Nested artboard remains externally owned. |
| `HitNestedArtboard::hitTest` (`state_machine_instance.cpp:914-941`) | [x] CLOSED; W4: missing | `B-HIT`: transform then authored nested-SM scan | `S-HIT` | Collapsed/paused/singular; first misses, second hits. |
| `HitNestedArtboard::processGamepadInvocation` (`state_machine_instance.cpp:942-960`) | [x] CLOSED; W4: scattered | `B-HIT`: all nested machines, return none | `S-HIT` | Child opaque ignored; multiple children; null malformed wrapper. |
| `HitNestedArtboard::processEvent` (`state_machine_instance.cpp:961-1067`) | [x] CLOSED; W4: missing | `B-HIT`: transform, supported routing, occlusion→exit | `S-HIT` | First child hit then later miss overwrite; occluded move exits; drag returns none. |
| `HitNestedArtboard::prepareEvent` (`state_machine_instance.cpp:1068-1071`) | [x] CLOSED; W4: missing | `B-SOURCE`: intentional no-op | `S-HIT` | Child hover changes only during process. |
| `HitComponentList` constructor (`state_machine_instance.cpp:1077-1080`) | [x] CLOSED; W4: missing | `B-HIT`: borrowed list wrapper | `S-HIT` | Collapsed and duplicate-index list. |
| `HitComponentList` destructor (`state_machine_instance.cpp:1081`) | [x] CLOSED; W4: missing | `B-SOURCE`: empty wrapper drop | `S-HIT`, `S-LIFE` | Items remain externally owned. |
| `HitComponentList::hitTest` (`state_machine_instance.cpp:1083-1107`) | [x] CLOSED; W4: missing | `B-HIT`: reverse ordered indices | `S-HIT`, `S-ORDER` | Duplicate indices; null top item; singular item transform. |
| `HitComponentList::processEvent` (`state_machine_instance.cpp:1108-1226`) | [x] CLOSED; W4: missing | `B-HIT`: strongest-result aggregation and cleanup | `S-HIT` | Opaque first then previously hovered second; drag; parent `canHit=false`. |
| `HitComponentList::processGamepadInvocation` (`state_machine_instance.cpp:1227-1269`) | [x] CLOSED; W4: missing | `B-HIT`: reverse broadcast, stop after opaque | `S-HIT`, `S-ORDER` | Duplicate index; collapsed list; opaque top item. |
| `HitComponentList::prepareEvent` (`state_machine_instance.cpp:1270-1273`) | [x] CLOSED; W4: missing | `B-SOURCE`: intentional no-op | `S-HIT` | No pre-process child hover mutation. |

### Listener-ViewModel helper members

Rust destination for every row:
`crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W2/W4 |
| --- | --- | --- | --- | --- |
| binding `m_parent` (`state_machine_instance.cpp:1289`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: stable listener occurrence/index | `S-FIELDS` | Null parent makes dirt inert. |
| binding `m_viewModelInstanceValue` (`state_machine_instance.cpp:1290-1292`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: retained cell plus weak sink adaptation | `S-FIELDS` | Two bindings same cell; property notifying during clear. |
| base binding constructor (`state_machine_instance.cpp:1401-1407`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: retain then register | `S-ORDER` | Null property malformed; duplicate registration. |
| base `relinkDataBind` (`state_machine_instance.cpp:1409`) | [x] CLOSED; W4: faithful-looking | `B-SOURCE`: no-op | `S-BIND` | Base use across context replacement retains old property. |
| base destructor (`state_machine_instance.cpp:1411-1414`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: clear before retained cell drop | `S-LIFE` | Already cleared; active notification. |
| base `clearDataContext` (`state_machine_instance.cpp:1416-1424`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: unregister before release; idempotent | `S-ORDER` | Repeated clear. |
| base `addDirt` (`state_machine_instance.cpp:1481-1488`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: every dirt enqueues, no dedup | `S-EVENT` | Repeated dirt; dirt after clear; ignored recurse/value. |
| listener binding `m_listener` (`state_machine_instance.cpp:1305`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: retained definition/index adaptation | `S-FIELDS` | Null listener; synchronous dirt during base registration. |
| listener binding constructor (`state_machine_instance.cpp:1426-1432`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: base registration precedes subtype field | `S-ORDER` | Null listener. |
| listener binding `relinkDataBind` (`state_machine_instance.cpp:1434-1452`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: null context retains old; unresolved clears; same cell no-op | `S-BIND` | New context/same cell; missing path; null context. |
| input binding `m_listenerInput` (`state_machine_instance.cpp:1318`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: authored input identity | `S-FIELDS` | Duplicate authored paths. |
| input binding constructor (`state_machine_instance.cpp:1454-1460`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: independent duplicate registrations | `S-ORDER` | Null input. |
| input binding `relinkDataBind` (`state_machine_instance.cpp:1462-1479`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: per-input equivalent lifecycle | `S-BIND` | Duplicate inputs same cell; unresolved replacement. |
| listener VM `m_stateMachineInstance` (`state_machine_instance.cpp:1393-1394`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: stable instance operation channel | `S-FIELDS` | No mutable queue alias after clone. |
| listener VM `m_listener` (`state_machine_instance.cpp:1395`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: retained immutable listener | `S-FIELDS` | Null source-cited malformed listener. |
| listener VM `m_dataContext` (`state_machine_instance.cpp:1396`) | [x] CLOSED; W4: scattered | `B-LVM`: old context retention after binding clear | `S-FIELDS` | Clear bindings then query context remains non-null. |
| listener VM `m_propertyBindings` (`state_machine_instance.cpp:1397-1398`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: authored-discovery order | `S-ORDER` | Interleaved non-VM inputs and duplicate VM inputs. |
| `ListenerViewModel` constructor (`state_machine_instance.cpp:1325-1328`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: owner/listener identity | `S-FIELDS` | Null listener; lifetime bounded by instance. |
| destructor (`state_machine_instance.cpp:1490`) | [x] CLOSED; W4: faithful-looking | `B-LVM`, `B-LIFE`: idempotent clear | `S-LIFE` | Direct destruction while bound. |
| `clearDataContext` (`state_machine_instance.cpp:1330`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: clear bindings but retain context | `S-BIND` | Query context after clear. |
| `bindFromContext` (`state_machine_instance.cpp:1331-1373`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: store context, clear, single-or-all authored paths | `S-ORDER`, `S-BIND` | Null malformed context; unresolved; duplicate VM inputs. |
| `reportToStateMachine` (`state_machine_instance.cpp:1374-1381`) | [x] CLOSED; W4: divergent | `B-LVM`: suppress trigger value zero only | `S-EVENT` | Trigger `0→1→0`; signed zero suppressed; repeated value one; duplicate bindings. |
| `listener` (`state_machine_instance.cpp:1382`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: retained accessor | `S-API` | Null malformed listener. |
| `dataContext` (`state_machine_instance.cpp:1383-1391`) | [x] CLOSED; W4: scattered | `B-LVM`: borrowed getter over retained context | `S-BIND` | Bindings cleared while context retained. |

### `StateMachineInstance` fields and inline/header surface

Rust destination for every row:
`crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W1/W4 |
| --- | --- | --- | --- | --- |
| `DataBindChanged` typedef (`state_machine_instance.hpp:55`) | [x] CLOSED; W4: missing | `B-SOURCE`: tools callback adaptation | `S-TOOLS` | Nullable callback. |
| tools `InputChanged` typedef (`state_machine_instance.hpp:59`) | [x] CLOSED; W4: missing | `B-SOURCE`: tools callback adaptation | `S-TOOLS` | Nullable callback. |
| `m_DataContext` (`state_machine_instance.hpp:93`) | [x] CLOSED; W4: scattered | `B-BIND`: explicit empty initial state | `S-FIELDS`, `S-BIND` | No invented default context. |
| `m_reportedEvents` (`state_machine_instance.hpp:384`) | [x] CLOSED; W4: divergent, approved cursor adaptation | `B-EVENT`: pending queue | `S-EVENT`, `S-FIELDS` | Host drain must not consume listener delivery. |
| `m_reportingEvents` (`state_machine_instance.hpp:385`) | [x] CLOSED; W4: divergent, approved cursor adaptation | `B-EVENT`: current snapshot visibility | `S-EVENT` | Callback sees only newly chained pending reports. |
| `m_machine` (`state_machine_instance.hpp:386`) | [x] CLOSED; W4: scattered, approved index/arena adaptation | `B-CTOR`: stable supplied definition | `S-FIELDS` | No observable null/default state. |
| `m_needsAdvance` (`state_machine_instance.hpp:387`) | [x] CLOSED; W4: faithful-looking | `B-ADV`: exact latch-only accessor | `S-FIELDS` | Pending ordinary event while latch false. |
| `m_inputInstances` (`state_machine_instance.hpp:388`) | [x] CLOSED; W4: faithful-looking | `B-CTOR`, `B-ADV`: authored null slots, every-slot advanced | `S-SLOTS`, `S-FIELDS` | Unsupported index 0 then valid input; null slot malformed advance. |
| `m_layerCount` (`state_machine_instance.hpp:389`) | [x] CLOSED; W4: faithful-looking | `B-CTOR`: derive before access | `S-FIELDS` | Definition count cannot diverge from owned vector length. |
| `m_layers` (`state_machine_instance.hpp:390`) | [x] CLOSED; W4: faithful-looking | `B-CTOR`: authored owned occurrences | `S-FIELDS`, `S-ORDER` | Empty and multi-layer machine. |
| `m_hitComponents` (`state_machine_instance.hpp:391`) | [x] CLOSED; W4: divergent | `B-HIT`: complete polymorphic ordered owners | `S-HIT`, `S-FIELDS` | Nested/provider-only machine makes `hasListeners` true. |
| `m_listenerGroups` (`state_machine_instance.hpp:392`) | [x] CLOSED; W4: divergent | `B-HIT`: retained groups, including unresolved targets | `S-HIT`, `S-SLOTS` | Unresolved target group still reset each event. |
| `m_parentStateMachineInstance` (`state_machine_instance.hpp:393`) | [x] CLOSED; W4: scattered | `B-EVENT`: ID/owner-safe parent adaptation | `S-FIELDS` | Null and nested parent. |
| `m_parentNestedArtboard` (`state_machine_instance.hpp:394`) | [x] CLOSED; W4: scattered | `B-EVENT`: parent nested identity | `S-FIELDS` | Null and replaced nested host. |
| shadow `m_dataBinds` (`state_machine_instance.hpp:395`) | [x] CLOSED; W4: scattered | `B-SOURCE`: tools-only empty-shadow behavior or documented Rust tools adaptation | `S-TOOLS` | Callback installed after later keyframe binds. |
| `m_listenerViewModels` (`state_machine_instance.hpp:396`) | [x] CLOSED; W4: faithful-looking | `B-LVM`: authored raw-owner equivalent | `S-FIELDS`, `S-ORDER` | Duplicate listeners independently owned. |
| `m_reportedListenerViewModels` (`state_machine_instance.hpp:397`) | [x] CLOSED; W4: faithful-looking | `B-EVENT`: pending FIFO | `S-EVENT` | Same listener twice. |
| `m_reportingListenerViewModels` (`state_machine_instance.hpp:398`) | [x] CLOSED; W4: faithful-looking | `B-EVENT`: current snapshot | `S-EVENT` | First callback reports second; second waits next batch. |
| `m_bindablePropertyInstances` (`state_machine_instance.hpp:399-400`) | [x] CLOSED; W4: scattered | `B-CTOR`: source identity → one clone | `S-FIELDS` | Structurally equal distinct sources; duplicate target reuse. |
| `m_scriptedObjectsMap` (`state_machine_instance.hpp:401-402`) | [x] CLOSED; W4: scattered, approved lifecycle adaptation | `B-CTOR`, `B-LIFE`: occurrence identity/order rules | `S-FIELDS`, `S-LIFE` | Duplicate source pointer and equivalent different pointer. |
| `m_bindableDataBindsToTarget` (`state_machine_instance.hpp:403-404`) | [x] CLOSED; W4: scattered | `B-KEY`: last map entry while occurrences retained | `S-FIELDS` | Duplicate ToTarget/TwoWay. |
| `m_bindableDataBindsToSource` (`state_machine_instance.hpp:405-406`) | [x] CLOSED; W4: scattered | `B-KEY`: last ToSource map entry | `S-FIELDS` | Duplicate ToSource. |
| `m_transitionPropertyInstances` (`state_machine_instance.hpp:410-412`) | [x] CLOSED; W4: scattered | `B-LAYER`: occurrence-local transition values | `S-FIELDS` | Duplicate key overwrites lookup without rewriting earlier bind target. |
| `m_stateKeyFrameDataBinds` (`state_machine_instance.hpp:417-418`) | [x] CLOSED; W4: scattered | `B-KEY`: state-keyed build/removal tracking | `S-KEY`, `S-FIELDS` | Build twice; remove unknown; teardown with active binds. |
| `m_drawOrderChangeCounter` (`state_machine_instance.hpp:419`) | [x] CLOSED; W4: missing | `B-HIT`: constructor sort and change-triggered resort | `S-HIT`, `S-FIELDS` | Nonzero initial counter; wrap/change; unmatched custom hit. |
| `m_focusManager` (`state_machine_instance.hpp:425`) | [x] CLOSED; W4: scattered | `B-FOCUS`: owned internal domain | `S-FIELDS`, `S-SEAM` | No nodes and external replacement. |
| `m_externalFocusManager` (`state_machine_instance.hpp:426`) | [x] CLOSED; W4: scattered | `B-FOCUS`: identity/fallback adaptation | `S-SEAM` | Same pointer with different desired parent is no-op. |
| `m_focusListenerGroups` (`state_machine_instance.hpp:427`) | [x] CLOSED; W4: scattered | `B-FOCUS`: authored registration order | `S-ORDER` | Duplicate focus callbacks. |
| `m_keyboardListenerGroups` (`state_machine_instance.hpp:428-429`) | [x] CLOSED; W4: scattered | `B-CTOR`: authored construction | `S-ORDER`, `S-SEAM` | Listener flags plus scripted wants flags. |
| `m_gamepadListenerGroups` (`state_machine_instance.hpp:430`) | [x] CLOSED; W4: scattered | `B-CTOR`: authored construction | `S-ORDER`, `S-SEAM` | Null/wrong target remains pinned malformed behavior. |
| `m_gamepadScriptedDrawables` (`state_machine_instance.hpp:437`) | [x] CLOSED; W4: scattered | `B-CTOR`: non-owning authored facility list | `S-FIELDS` | Focused drawable excluded from later broadcast. |
| `m_embedderGamepads` (`state_machine_instance.hpp:439`) | [x] CLOSED; W4: missing/out-of-scope definition | `B-SOURCE`: ownership boundary only | `S-SEAM` | Buffer parsing/mutation cases belong to `gamepad_batch.cpp`. |
| `m_semanticManager` (`state_machine_instance.hpp:442`) | [x] CLOSED; W4: missing dependency | `B-FOCUS`: queue/orchestration boundary | `S-SEAM` | Internal manager absent/present. |
| `m_externalSemanticManager` (`state_machine_instance.hpp:443`) | [x] CLOSED; W4: missing dependency | `B-FOCUS`: selected-manager boundary | `S-SEAM` | External→null with/without internal manager. |
| `QueuedFocusEvent::group` (`state_machine_instance.hpp:448`) | [x] CLOSED; W4: faithful-looking value adaptation | `B-FOCUS`: explicit non-default construction | `S-FIELDS` | Null malformed group. |
| `QueuedFocusEvent::isFocus` (`state_machine_instance.hpp:449`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: exact direction | `S-FIELDS` | Focus and blur duplicates. |
| `m_queuedFocusEvents` (`state_machine_instance.hpp:451`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: FIFO moved batch | `S-EVENT` | Callback queues another focus change. |
| `m_semanticListenerGroups` (`state_machine_instance.hpp:455-456`) | [x] CLOSED; W4: divergent dependency shape | `B-FOCUS`: authored queue producers | `S-SEAM`, `S-ORDER` | Null owner and duplicate action. |
| `QueuedSemanticEvent::group` (`state_machine_instance.hpp:459`) | [x] CLOSED; W4: faithful-looking value adaptation | `B-FOCUS`: explicit construction | `S-FIELDS` | Null group is skipped during processing. |
| `QueuedSemanticEvent::actionType` (`state_machine_instance.hpp:460`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: exact enum payload | `S-FIELDS` | Tap/increase/decrease and invalid cast no-op at manager seam. |
| `m_queuedSemanticEvents` (`state_machine_instance.hpp:462`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: FIFO moved batch | `S-EVENT` | Null, valid, null-listener, valid order. |
| tools `m_inputChangedCallback` (`state_machine_instance.hpp:472`) | [x] CLOSED; W4: missing | `B-SOURCE`: nullable tools callback | `S-TOOLS` | Replacement and clear. |
| `FocusState::hasFocus` (`state_machine_instance.hpp:335`) | [x] CLOSED; W4: missing | `B-FOCUS`: host snapshot | `S-API` | Focused scope without Focusable gives true. |
| `FocusState::expectsKeyboardInput` (`state_machine_instance.hpp:336`) | [x] CLOSED; W4: missing | `B-FOCUS`: Focusable capability | `S-API` | Focused non-keyboard node gives false. |

### `StateMachineInstance` methods and file-local keyframe helpers

Rust destination for every row:
`crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`, except
the three stale private transition declarations, which must not acquire an
instance-level Rust implementation.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W1–W4 |
| --- | --- | --- | --- | --- |
| constructor (`state_machine_instance.hpp:105-106`; `.cpp:1707-2128`) | [x] CLOSED; W4: divergent | `B-CTOR` | `S-OWNER`, `S-ORDER`, `S-FIELDS` | Null source/artboard; null input; duplicate transition bind; entry action before ordinary binds/listeners; partial failure after nested registration. |
| deleted copy constructor (`state_machine_instance.hpp:107`) | [x] CLOSED; W4: divergent, approved snapshot adaptation | `B-LIFE`: snapshot versus cold remount | `S-LIFE`, `S-API` | Queues/pointer state copied by value; script tables cold; trigger IDs regenerated; no alias. |
| destructor (`state_machine_instance.hpp:108`; `.cpp:2141-2199`) | [x] CLOSED; W4: divergent | `B-LIFE`: observable teardown order | `S-LIFE` | Internal focused cleanup queues but does not run blur actions; external trees untouched; destruction without dispose is prevented/adapted. |
| `updateListeners` (`state_machine_instance.hpp:75-78`; `.cpp:1494-1545`) | [x] CLOSED; W4: divergent | `B-HIT` | `S-HIT`, `S-ADV` | NaN frame origin; shared front/back group; opaque front sends back exit; unknown pointer exit allocates/releases state. |
| `getNamedInput` (`state_machine_instance.hpp:80-81`; `.cpp:2689-2701`) | [x] CLOSED; W4: scattered | `B-DEF`: typed first match | `S-SLOTS`, `S-API` | Null slot before match; same name different types. |
| `notifyEventListeners` (`state_machine_instance.hpp:82-83`; `.cpp:3062-3171`) | [x] CLOSED; W4: scattered | `B-EVENT`: local → bubble → audio seam | `S-EVENT`, `S-SEAM` | `[A,A]` for single/multi; target mismatch; null malformed event; nested audio through two ancestors. |
| `sortHitComponents` (`state_machine_instance.hpp:84`; `.cpp:2255-2304`) | [x] CLOSED; W4: missing | `B-HIT`: exact swap-derived order | `S-HIT` | Multiple Artboard targets, duplicate drawable, unmatched custom component; not stable-sort semantics. |
| stale private `randomValue` declaration (`state_machine_instance.hpp:85`) | [x] CLOSED; W4: scattered to layer | `B-SOURCE`: no definition exists | `S-LAYER` | Must not alias layer method on instance. |
| stale private `findRandomTransition` declaration (`state_machine_instance.hpp:86-88`) | [x] CLOSED; W4: scattered to layer | `B-SOURCE`: no definition exists | `S-LAYER` | No instance-level symbol unless explicit unreachable ABI stub is required. |
| stale private `findAllowedTransition` declaration (`state_machine_instance.hpp:89-91`) | [x] CLOSED; W4: scattered to layer | `B-SOURCE`: no definition exists | `S-LAYER` | Same. |
| `completeViewModelInstances` (`state_machine_instance.hpp:97`; `.cpp:2792-2829`) | [x] CLOSED; W4: missing | `B-BIND`: main then globals; preserve occupied cross-model slot | `S-BIND`, `S-ORDER` | Missing file no-op; null default skipped; cross-VM override remains. |
| `addToHitLookup` (`state_machine_instance.hpp:98-102`; `.cpp:1619-1705`) | [x] CLOSED; W4: divergent | `B-HIT`: type branches, dedup, recursion, opacity | `S-HIT`, `S-ORDER` | Reused layout opacity upgrade; shape opacity discard; mixed/deep container; duplicate target; unsupported target. |
| `markNeedsAdvance` (`state_machine_instance.hpp:110`; `.cpp:2667`) | [x] CLOSED; W4: scattered | `B-ADV`: set-only latch | `S-ADV` | Mark then `newFrame=false` remains true. |
| `advance(seconds,newFrame)` (`state_machine_instance.hpp:113`; `.cpp:2546-2585`) | [x] CLOSED — `state_machine_viewmodel_trigger_conditions_match_cpp_probe` now matches pinned C++ `advanced=true` for the bound value-trigger case; W4: divergent | `B-ADV` | `S-ADV`, `S-EVENT`, `S-HIT` | Draw-order change; chained focus lost-latch edge; `-0`; NaN; infinities; trigger clear; reports created during layer advance. |
| inline `advance(seconds)` (`state_machine_instance.hpp:115`) | [x] CLOSED; W4: scattered | `B-ADV`: delegates with `newFrame=true` | `S-ADV`, `S-API` | Equivalent to explicit true. |
| `needsAdvance` (`state_machine_instance.hpp:118`; `.cpp:2668`) | [x] CLOSED; W4: faithful-looking | `B-ADV`: latch only | `S-ADV` | Event pending while false; focus queue semantics. |
| `resetState` (`state_machine_instance.hpp:120`; `.cpp:2670-2676`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: authored layers only | `S-LAYER` | Active transition; queues/context/input unchanged. |
| `stateMachine` (`state_machine_instance.hpp:123`) | [x] CLOSED; W4: scattered, approved index/arena adaptation | `B-API`: stable `state_machine_index`/arena | `S-API`, `S-FIELDS` | Snapshot/remount retains resolvable definition. |
| `inputCount` (`state_machine_instance.hpp:125`) | [x] CLOSED; W4: faithful-looking | `B-DEF`: slot count | `S-SLOTS` | Null slot counts. |
| `input(index)` (`state_machine_instance.hpp:126`; `.cpp:2680-2687`) | [x] CLOSED; W4: faithful-looking | `B-DEF`: optional indexed occurrence | `S-API` | In-range null vs out-of-range. |
| `getBool` (`state_machine_instance.hpp:127`; `.cpp:2703-2706`) | [x] CLOSED; W4: scattered | `B-DEF`: typed first-name lookup | `S-API` | Number then bool same name. The pinned typed loop dereferences every occurrence, so null definition slots are proven separately rather than passed into this undefined malformed-instance path. |
| `getNumber` (`state_machine_instance.hpp:128`; `.cpp:2707-2710`) | [x] CLOSED; W4: scattered | `B-DEF`: typed first-name lookup | `S-API` | Duplicate number names. |
| `getTrigger` (`state_machine_instance.hpp:129`; `.cpp:2711-2714`) | [x] CLOSED; W4: scattered | `B-DEF`: typed first-name lookup | `S-API` | Exact type/name and authored first match on well-formed occurrences; null definition slots are a separate definition-seam proof. |
| `bindViewModelInstance` (`state_machine_instance.hpp:130-131`; `.cpp:2831-2842`) | [x] CLOSED; W4: divergent | `B-BIND`: distinct null/non-null branches | `S-BIND` | Null clears machine context/listeners and artboard unbind only; does not explicitly unbind machine binds. |
| `setViewModelInstance` (`state_machine_instance.hpp:135`; `.cpp:2716-2733`) | [x] CLOSED; W4: missing | `B-BIND`: null no-op; stage without bind | `S-BIND` | Replace main then inspect stale paths before explicit bind. |
| `setGlobalViewModelInstance` (`state_machine_instance.hpp:140-141`; `.cpp:2735-2774`) | [x] CLOSED; W4: missing | `B-BIND`: validation and named-slot replacement | `S-BIND`, `S-ORDER` | Null/file/name/non-global failures; put type B in slot A; preserve other slot order. |
| `bind` (`state_machine_instance.hpp:144`; `.cpp:2776-2790`) | [x] CLOSED; W4: scattered | `B-BIND`: complete → artboard → machine | `S-BIND` | No context no-op; shared context with missing defaults and sibling. |
| `globalViewModelInstance` (`state_machine_instance.hpp:147`; `.cpp:2844-2859`) | [x] CLOSED; W4: missing | `B-BIND`: pure slot read without global validation | `S-BIND`, `S-API` | Unknown/non-global name and unusual slot keys. |
| `bindDataContext` (`state_machine_instance.hpp:148`; `.cpp:2861-2868`) | [x] CLOSED; W4: divergent | `B-BIND`: clear/register/clear artboard/bind artboard/bind machine | `S-BIND` | Null must not become safe clear; incomplete context remains incomplete. |
| `inheritDataContext` (`state_machine_instance.hpp:149`; `.cpp:2870-2878`) | [x] CLOSED; W4: scattered | `B-BIND`: null no-op, no prior clear, machine only | `S-BIND` | Inherit A then B leaves stale registration on A. |
| `dataContext(setter)` (`state_machine_instance.hpp:150`; `.cpp:2880-2884`) | [x] CLOSED; W4: scattered | `B-BIND`: clear then internal apply, machine only | `S-BIND` | Null with/without VM listeners. |
| `dataContext(getter)` (`state_machine_instance.hpp:151`) | [x] CLOSED; W4: missing | `B-BIND`: retained optional getter | `S-BIND`, `S-API` | After clear. |
| `rebind` (`state_machine_instance.hpp:152`; `.cpp:2916-2921`) | [x] CLOSED; W4: scattered | `B-BIND`: artboard clear/apply then machine apply | `S-BIND` | Rebind after clear with VM listener. |
| `currentAnimationCount` (`state_machine_instance.hpp:154`; `.cpp:2985-2996`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: per-layer count | `S-LAYER`, `S-API` | Two layers same source animation count twice. |
| `currentAnimationByIndex` (`state_machine_instance.hpp:155`; `.cpp:2998-3014`) | [x] CLOSED; W4: faithful-looking | `B-LAYER`: compact authored order | `S-LAYER`, `S-API` | Interleaved non-animation layers. |
| `stateChangedCount` (`state_machine_instance.hpp:159`; `.cpp:2955-2966`) | [x] CLOSED; W4: faithful-looking result, divergent cache shape | `B-LAYER`: scan retained flags; derived cache allowed | `S-LAYER` | Several transitions in one layer count one. |
| `stateChangedByIndex` (`state_machine_instance.hpp:164`; `.cpp:2968-2983`) | [x] CLOSED — `state_machine_generic_layer_state_occurrence_matches_cpp_probe` now matches pinned C++ core type `60` and changed-layer occurrence order; W4: missing | `B-LAYER`: compact authored changed-layer order | `S-LAYER`, `S-API` | Layers 0 and 2 changed; index 1 returns layer 2 current state; out of range null. |
| `advanceAndApply(seconds)` (`state_machine_instance.hpp:166`; `.cpp:2601-2604`) | [x] CLOSED; W4: scattered | `B-ADV`: exact delegate with VM=true | `S-ADV`, `S-OWNER` | Byte-equivalent to bool overload true. |
| `advanceAndApply(seconds,advanceViewModels)` (`state_machine_instance.hpp:171`; `.cpp:2606-2665`) | [x] CLOSED; W4: faithful-looking behavior, scattered owner | `B-ADV`: consolidated owner | `S-ADV`, `S-OWNER` | Idle `-0`; hidden focus; sixth dirt pass; VM=false; infinite nested ping-pong. |
| `advancedDataContext` (`state_machine_instance.hpp:172`; `.cpp:2587-2593`) | [x] CLOSED; W4: scattered | `B-ADV`: each settlement iteration if bound | `S-ADV` | Five dirty passes produce five advances. |
| `reset` (`state_machine_instance.hpp:173`; `.cpp:2595-2599`) | [x] CLOSED; W4: scattered | `B-ADV`: VM advanced before artboard reset | `S-ADV`, `S-ORDER` | Resettable observes post-consumed trigger. |
| `name` (`state_machine_instance.hpp:174`; `.cpp:2678`) | [x] CLOSED; W4: scattered | `B-API`: source machine name | `S-API` | Null source remains malformed, not empty string. |
| `pointerMove` (`state_machine_instance.hpp:175-177`; `.cpp:1568-1573`) | [x] CLOSED; W4: divergent | `B-HIT`: C++ path forwards coordinates/timestamp | `S-HIT`, `S-ADV` | Multiple IDs; negative/NaN timestamp and nonfinite coordinates. |
| `pointerDown` (`state_machine_instance.hpp:178`; `.cpp:1574-1577`) | [x] CLOSED — `flow_pointer_callbacks_receive_event_time_and_the_prior_delivered_position` and `listener_missing_context_hydration_keeps_the_table_until_context_arrives` now retain and dispatch both ordinary and deferred scripted callbacks; W4: divergent | `B-HIT`: timestamp zero through hit owners | `S-HIT` | Down outside after hover; duplicate click listeners. |
| `pointerUp` (`state_machine_instance.hpp:179`; `.cpp:1578-1581`) | [x] CLOSED; W4: divergent | `B-HIT`: click/up phase | `S-HIT` | Up without down; different pointer IDs; click plus up. |
| `pointerExit` (`state_machine_instance.hpp:180`; `.cpp:1582-1585`) | [x] CLOSED; W4: divergent | `B-HIT`: process then release pointer state | `S-HIT` | Exit during drag; repeated exit. |
| `dragStart` (`state_machine_instance.hpp:181-184`; `.cpp:1586-1597`) | [x] CLOSED; W4: divergent | `B-HIT`: optional disable before timestamp-zero event | `S-HIT` | External default versus internal disable=false; nested base no-op. |
| `dragEnd` (`state_machine_instance.hpp:185`; `.cpp:1598-1606`) | [x] CLOSED; W4: faithful-looking tail, surrounding divergence | `B-HIT`: enable → dragEnd(0) → move(timestamp) | `S-HIT`, `S-ORDER` | Different drag/move target; move opaque but return drag result. |
| `tryChangeState` (`state_machine_instance.hpp:187`; `.cpp:2306-2318`) | [x] CLOSED; W4: faithful-looking | `B-ADV`: bind update then every layer | `S-ADV`, `S-ORDER` | Two layers become eligible and both transition. |
| `hitTest` (`state_machine_instance.hpp:188`; `.cpp:1547-1566`) | [x] CLOSED; W4: divergent | `B-HIT`: first geometric hit in sorted hit list | `S-HIT` | Occluded geometric target still true; hidden raw-path edge; nested/list target; NaN forwarded. |
| `durationSeconds` (`state_machine_instance.hpp:190`) | [x] CLOSED; W4: missing | `B-SOURCE`: constant `-1` | `S-API` | Exact constant. |
| `loop` (`state_machine_instance.hpp:191`) | [x] CLOSED; W4: missing | `B-SOURCE`: `Loop::oneShot` | `S-API` | Exact enum. |
| `isTranslucent` (`state_machine_instance.hpp:192`) | [x] CLOSED; W4: missing | `B-SOURCE`: constant true | `S-API` | Exact constant. |
| `artboard` (`state_machine_instance.hpp:196`) | [x] CLOSED; W4: scattered | `B-API`: explicit-borrow adaptation | `S-OWNER`, `S-API` | Correct backing instance on every call. |
| `setParentStateMachineInstance` (`state_machine_instance.hpp:198-201`) | [x] CLOSED; W4: missing | `B-EVENT`: owner-safe parent identity | `S-FIELDS` | Set, replace, clear. |
| `parentStateMachineInstance` (`state_machine_instance.hpp:202-205`) | [x] CLOSED; W4: missing | `B-EVENT`: optional getter/adaptation | `S-API` | Null and nested parent. |
| `setParentNestedArtboard` (`state_machine_instance.hpp:207-210`) | [x] CLOSED; W4: scattered | `B-EVENT`: local-ID/handle adaptation | `S-FIELDS` | Set, replace, clear. |
| `parentNestedArtboard` (`state_machine_instance.hpp:211`) | [x] CLOSED; W4: missing | `B-EVENT`: optional getter/adaptation | `S-API` | Null and nested parent. |
| `notify` (`state_machine_instance.hpp:212-213`; `.cpp:3041-3046`) | [x] CLOSED; W4: scattered | `B-EVENT`: immediate nested dispatch then bind update | `S-EVENT`, `S-ORDER` | Nested action dirties bind; bubbling precedes final local bind update. |
| `notifyListenerViewModels` (`state_machine_instance.hpp:214-215`; `.cpp:3048-3060`) | [x] CLOSED; W4: faithful-looking | `B-EVENT`: snapshot FIFO/duplicates | `S-EVENT` | First reports second; null malformed pointer; terminal Rust error documented. |
| `reportEvent` (`state_machine_instance.hpp:219`; `.cpp:3016-3019`) | [x] CLOSED; W4: scattered | `B-EVENT`: exact FIFO report append | `S-EVENT` | Duplicate report; null malformed event; negative/NaN/infinite/`-0` delay. |
| `applyEvents` (`state_machine_instance.hpp:221`; `.cpp:2320-2344`) | [x] CLOSED — `synchronous_pointer_events_survive_the_followup_advance_once_in_authored_order` and `synchronous_and_advance_events_share_one_cycle_in_cpp_order_without_replay` now retain the synchronous prefix and apply it once in C++ order; W4: faithful-looking | `B-EVENT` | `S-EVENT` | Event chains event+VM; exactly 100 finite batches; 101 chain; callback count query. |
| `reportListenerViewModel` (`state_machine_instance.hpp:223`; `.cpp:3021-3025`) | [x] CLOSED; W4: faithful-looking | `B-EVENT`: borrowed/indexed FIFO append | `S-EVENT` | Same listener twice; null malformed. |
| `reportedEventCount` (`state_machine_instance.hpp:226`; `.cpp:3027-3030`) | [x] CLOSED; W4: faithful-looking | `B-EVENT`: pending-only visibility | `S-EVENT`, `S-API` | Inside callback after chaining one event. |
| `reportedEventAt` (`state_machine_instance.hpp:229`; `.cpp:3032-3039`) | [x] CLOSED; W4: divergent API adaptation | `B-EVENT`: live projection and out-of-range adaptation | `S-EVENT`, `S-API` | Index==count; C++ null/+0 sentinel; Rust `None`; mutable payload refresh. |
| `playsAudio` (`state_machine_instance.hpp:230`) | [x] CLOSED; W4: missing | `B-SOURCE`: constant true | `S-EVENT`, `S-SEAM`, `S-API` | Audio play call stays recorded under `audio_event.cpp`. |
| `bindablePropertyInstance` (`state_machine_instance.hpp:231-232`; `.cpp:3189-3199`) | [x] CLOSED; W4: scattered | `B-KEY`: exact source identity → typed clone | `S-FIELDS` | Equivalent different address/global ID. |
| `bindableDataBindToSource` (`state_machine_instance.hpp:233-234`; `.cpp:3201-3210`) | [x] CLOSED; W4: scattered | `B-KEY`: last duplicate source bind | `S-FIELDS` | Two ToSource binds. |
| `bindableDataBindToTarget` (`state_machine_instance.hpp:235-236`; `.cpp:3212-3221`) | [x] CLOSED; W4: scattered | `B-KEY`: last target bind | `S-FIELDS` | ToTarget and TwoWay duplicates. |
| `findTransitionPropertyInstance` (`state_machine_instance.hpp:241-243`; `.cpp:3223-3237`) | [x] CLOSED; W4: scattered | `B-LAYER`: two-key occurrence lookup adaptation | `S-FIELDS` | Missing outer/inner key; duplicate duration bind. |
| file-local `keyFrameHolderPropertyKey` (`.cpp:3239-3256`) | [x] CLOSED; W4: scattered | `B-KEY`: number/color/bool/string only | `S-KEY` | ID/uint/custom returns zero/unbound. |
| file-local `makeKeyFrameValueHolder` (`.cpp:3258-3274`) | [x] CLOSED; W4: scattered | `B-KEY`: exact holder type | `S-KEY` | Four supported types; unsupported null. |
| `buildStateKeyFrameBinds` (`state_machine_instance.hpp:251`; `.cpp:3276-3374`) | [x] CLOSED; W4: scattered | `B-KEY` | `S-KEY`, `S-ORDER` | Duplicate source bind first wins; unsupported/null keyframe; bound context; build twice; observable initialize/converter order. |
| `removeStateKeyFrameBinds` (`state_machine_instance.hpp:255`; `.cpp:3376-3390`) | [x] CLOSED; W4: scattered | `B-KEY`: remove/delete in build order then erase | `S-KEY`, `S-LIFE` | Unknown state; removal during update callback; destructor with active binds. |
| `hasListeners` (`state_machine_instance.hpp:257`) | [x] CLOSED; W4: divergent | `B-HIT`: hit-owner nonempty meaning | `S-HIT`, `S-API` | Nested/component-list hit proxy with no authored pointer listener. |
| `hasFocusNodes` (`state_machine_instance.hpp:258`; `.cpp:3392-3397`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: selected manager result | `S-API` | Manager exists but no nodes. |
| `focusNext` (`state_machine_instance.hpp:259`; `.cpp:3399-3404`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: delegate and defer callbacks | `S-API`, `S-SEAM` | Hidden current target dropped first. |
| `focusPrevious` (`state_machine_instance.hpp:260`; `.cpp:3406-3411`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: delegate and defer callbacks | `S-API`, `S-SEAM` | No primary focus with several roots. |
| `clearFocus` (`state_machine_instance.hpp:261`; `.cpp:3413-3418`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: focus clears before callback | `S-API`, `S-SEAM` | Call twice; only first blur. |
| `clearDataContext` (`state_machine_instance.hpp:262`; `.cpp:2923-2934`) | [x] CLOSED; W4: scattered | `B-BIND`: unregister/null then clear listener cells only | `S-BIND`, `S-ORDER` | State-machine binds/artboard/scripts retain their pinned state. |
| `relinkDataContext` (`state_machine_instance.hpp:263`; `.cpp:2936-2939`) | [x] CLOSED; W4: scattered | `B-BIND`: artboard-only delegation | `S-BIND` | Nested VM reference used only by state-machine listener remains unaffected here. |
| `rebuildDataBind` (`state_machine_instance.hpp:264`; `.cpp:2941-2947`) | [x] CLOSED; W4: scattered | `B-BIND`: context-bind subtype only | `S-BIND` | Plain bind ignored; null malformed; cleared context forwarded. |
| private `unbind` (`state_machine_instance.cpp:2949-2953`) | [x] CLOSED; W3 lifecycle inventory | `B-BIND`, `B-LIFE`: `unbind` calls `clearDataContext` before the complete machine DataBind unbind; destructor receipt observes the same order | `S-BIND`, `S-LIFE` | Context/listener registrations disappear before source/observer/converter bindings; no artboard unbind or scripted-context clear is invented. |
| `internalDataContext` (`state_machine_instance.hpp:265`; `.cpp:2901-2914`) | [x] CLOSED — `first_factory_pointer_prepares_and_applies_fixed_bindings_before_callback` observes the fixed source and `listener_missing_context_hydration_keeps_the_table_until_context_arrives` retains the deferred listener through hydration; W4: scattered | `B-BIND`: assign → binds → listener cells → script contexts → init/hydrate | `S-BIND`, `S-ORDER` | Null with VM listeners; script mutates context; multiple script visits. |
| `scriptedObject` (`state_machine_instance.hpp:266`; `.cpp:2130-2139`) | [x] CLOSED; W4: scattered | `B-CTOR`: exact source/global identity adaptation | `S-FIELDS`, `S-API` | Equivalent different source returns none. |
| `queueFocusEvent` (`state_machine_instance.hpp:269`; `.cpp:2409-2414`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: FIFO append and mark | `S-EVENT` | Null malformed group; duplicates. |
| `queueSemanticEvent` (`state_machine_instance.hpp:272-273`; `.cpp:2475-2480`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: FIFO append and mark | `S-EVENT` | Duplicate same action. |
| `fireSemanticAction` (`state_machine_instance.hpp:276-277`; `.cpp:2509-2544`) | [x] CLOSED; W4: missing dependency | `B-FOCUS`: dispatch orchestration to recorded lookup seam | `S-SEAM` | Missing manager/id/data; invalid enum; nested owner. |
| mutable `focusManager` (`state_machine_instance.hpp:281-285`) | [x] CLOSED; W4: scattered | `B-FOCUS`: external else internal selection | `S-SEAM` | Null external falls back. |
| const `focusManager` (`state_machine_instance.hpp:289-293`) | [x] CLOSED; W4: scattered | `B-FOCUS`: same selection | `S-SEAM` | Same as mutable. |
| `hasExternalFocusManager` (`state_machine_instance.hpp:296-299`) | [x] CLOSED; W4: missing | `B-FOCUS`: identity query/adaptation | `S-API`, `S-SEAM` | Install, replace, clear. |
| `internalFocusManager` (`state_machine_instance.hpp:304`) | [x] CLOSED; W4: missing | `B-FOCUS`: owned-manager access/adaptation | `S-API`, `S-SEAM` | Ignores selected external manager. |
| `submitGamepadsFromBuffer` (`state_machine_instance.hpp:310`; `gamepad_batch.cpp:165-296`) | [x] CLOSED; W4: missing, out of FL-C5 source definition | `B-SOURCE`: declaration/seam only | `S-SEAM` | Null/version/truncation/rollback/NaN cases remain owning row’s proof. |
| `broadcastGamepadToScriptedDrawables` (`state_machine_instance.hpp:317-319`; `gamepad_batch.cpp:298-362`) | [x] CLOSED; W4: faithful-looking, out-of-scope definition | `B-SOURCE`: declaration and caller boundary | `S-SEAM` | Nested before local; skip focused; direct script hit nonopaque. |
| `setExternalFocusManager` (`state_machine_instance.hpp:325`; `.cpp:2346-2368`) | [x] CLOSED; W4: divergent dependency shape | `B-FOCUS`: clean old → assign → rebuild | `S-SEAM`, `S-ORDER` | Focused switch queues blur; identical pointer no-op despite parent change. |
| `setFocus` (`state_machine_instance.hpp:328`; `.cpp:2416-2428`) | [x] CLOSED; W4: missing | `B-FOCUS`: node or clear | `S-API`, `S-SEAM` | FocusData with null node behaves as clear. |
| `focusState` (`state_machine_instance.hpp:343`; `.cpp:2430-2447`) | [x] CLOSED; W4: missing | `B-FOCUS`: `{hasFocus, expectsKeyboardInput}` | `S-API`, `S-SEAM` | Focused node without Focusable; accepting/nonaccepting Focusable. |
| `semanticManager` (`state_machine_instance.hpp:348-352`) | [x] CLOSED; W4: missing dependency | `B-FOCUS`: selected-manager boundary only | `S-SEAM` | External, internal, neither. |
| `enableSemantics` (`state_machine_instance.hpp:357`; `.cpp:2370-2381`) | [x] CLOSED; W4: missing dependency | `B-FOCUS`: idempotent orchestration to seam | `S-SEAM` | External already set; null artboard. |
| `setExternalSemanticManager` (`state_machine_instance.hpp:364-365`; `.cpp:2383-2407`) | [x] CLOSED; W4: missing dependency | `B-FOCUS`: clean/assign/rebuild orchestration to seam | `S-SEAM`, `S-ORDER` | Same manager/different parent no-op; external→null with/without internal. |
| testing `hitComponentsCount` (`state_machine_instance.hpp:368`) | [x] CLOSED; W4: missing | `B-HIT`: list length | `S-TOOLS`, `S-HIT` | Provider/nested/list-only hits count. |
| testing `hitComponent` (`state_machine_instance.hpp:369-376`) | [x] CLOSED; W4: missing | `B-HIT`: indexed optional projection | `S-TOOLS`, `S-HIT` | Index==count. |
| testing `layerState` (`state_machine_instance.hpp:377`; `.cpp:1609-1616`) | [x] CLOSED; W4: missing | `B-LAYER`: machine-count bound then current state | `S-TOOLS`, `S-LAYER` | Definition count disagrees with occurrence length; out of range. |
| `enablePointerEvents` (`state_machine_instance.hpp:379`; `.cpp:3173-3179`) | [x] CLOSED; W4: missing | `B-HIT`: current sorted hit walk | `S-HIT` | Negative pointer ID; duplicates. |
| `disablePointerEvents` (`state_machine_instance.hpp:380`; `.cpp:3181-3187`) | [x] CLOSED; W4: missing | `B-HIT`: current sorted hit walk | `S-HIT` | Disable twice then enable once. |
| `dispose` (`state_machine_instance.hpp:381`; `.cpp:2201-2206`) | [x] CLOSED; W4: missing | `B-LIFE`: explicit nested detach, repeatable | `S-LIFE` | Call twice then child emits. |
| `removeEventListeners` (`state_machine_instance.hpp:421`; `.cpp:2208-2243`) | [x] CLOSED; W4: scattered ownership adaptation | `B-LIFE`: current nested traversal and all-duplicate removal | `S-LIFE` | Child removed/replaced before disposal; null elements skipped. |
| `initScriptedObjects` (`state_machine_instance.hpp:422`; `.cpp:2886-2899`) | [x] CLOSED — `listener_missing_context_hydration_keeps_the_table_until_context_arrives` proves the deferred listener retains its C++-equivalent callback table until initialization/hydration; W4: divergent, approved facade-timing adaptation | `B-CTOR`, `B-BIND`: initialization/hydration phase equivalence | `S-LIFE`, `S-BIND` | Two observable scripts; hydration failure does not abort later ordinary C++ work; terminal resource fence remains documented. |
| `processFocusEvents` (`state_machine_instance.hpp:452`; `.cpp:2449-2473`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: moved one-batch FIFO | `S-EVENT` | Callback changes focus; chained event waits next frame. |
| `processSemanticEvents` (`state_machine_instance.hpp:463`; `.cpp:2482-2507`) | [x] CLOSED; W4: faithful-looking | `B-FOCUS`: moved one-batch FIFO with null skips | `S-EVENT` | Null/valid/null-listener/valid. |
| tools `onInputChanged` (`state_machine_instance.hpp:467-470`) | [x] CLOSED; W4: missing | `B-SOURCE`: replace nullable callback | `S-TOOLS` | Set, replace, clear. |
| tools `onDataBindChanged` (`state_machine_instance.hpp:471`; `.cpp:2245-2253`) | [x] CLOSED; W4: missing | `B-SOURCE`: current shadow-vector behavior or documented tools adaptation | `S-TOOLS` | Later keyframe bind does not inherit callback; null clears. |

## Twelve required adversarial publication rows

- [x] Definition import and collection ownership: `B-DEF`, `S-OWNER`,
  `S-SLOTS`, `S-ORDER`, and `S-FIELDS` prove all five
  definition collections; missing artboard/state-machine importers; null-object
  input holes; input → layer → listener dirty/clean ordering; first-error stop
  without rollback; exact counts and index/name lookup; duplicate names,
  duplicate pointers, case mismatch, `index == count`, and `SIZE_MAX`.
- [x] Occurrence construction order: `B-CTOR`, `S-ORDER`, and `S-FIELDS`
  are landed through
  `fl_c5_constructor_order_phase_trace_and_explicit_fields`,
  `fl_c5_constructor_order_retains_unresolved_pointer_group_occurrence`,
  `fl_c5_constructor_order_source_and_runtime_boundaries_match_cpp`, and
  `listener_missing_context_hydration_keeps_the_table_until_context_arrives`.
  The `state_machine_instance_constructor_phase_reorder`,
  `state_machine_listener_slot_compaction`, and
  `scripted_object_unbound_constructor_enters_live_context` ratchets reject
  the displaced shapes. These proofs show inputs and tools indices
  precede layers; Any/Entry keyframe binds and Entry callbacks can run before
  ordinary machine binds/listeners; bindable reuse and duplicate transition
  property overwrite; event/VM exclusive listener paths; focus/keyboard/
  semantic/pointer/gamepad availability; provider groups, nested registrations,
  component lists, TextInput, scripted clones/facilities, hit sort, then focus
  tree. Include null machine/artboard/input/gamepad target and partial failure
  after a nested registration.
- [x] Ordered duplicates and nullable slots: `B-DEF`, `B-HIT`, `B-EVENT`,
  `S-SLOTS`, and `S-ORDER` prove inert malformed
  listeners retain indices; null/unsupported inputs retain slots; duplicate
  listener groups, actions, notifications, bind targets, scripted source
  pointers, provider targets, component-list indices, and nested notifier
  registrations remain observable in their pinned order. No `filter_map`,
  set, or map may replace an authored occurrence vector.
- [x] Transition search and state change: `B-LAYER`, `S-LAYER`, and
  `S-ORDER` are landed through
  `state_machine_generic_layer_state_occurrence_matches_cpp_probe`,
  `fl_c5_state_changed_layers_and_convergence_match_cpp_probe`,
  `fl_c5_current_state_and_animation_authored_compression_match_cpp_probe`,
  and `fl_c5_random_transition_edges_weighted_boundaries_and_wraparound_match_cpp_probe`.
  The `state_machine_layer_current_state_access_required`,
  `state_machine_changed_state_query_required`, and
  `state_machine_transition_candidate_reorder` ratchets enforce the retained
  occurrence and authored search. The proofs cover Any before current,
  first-match nonrandom selection, weighted authored order, wrapping weight
  sum, strict cumulative boundary, RNG 0/exact-boundary/1/NaN, waiting-for-exit,
  early interruption, spilled time, zero-duration callback pairing, 101-success
  guard, held animation/reset ordering, per-layer changed flags, compressed
  changed-state/current-animation access, and state reset during a transition.
- [x] Hit listener and focus ownership: `B-HIT`, `B-FOCUS`, `S-HIT`, and
  `S-SEAM` are landed through
  `flow_pointer_callbacks_receive_event_time_and_the_prior_delivered_position`,
  `listener_missing_context_hydration_keeps_the_table_until_context_arrives`,
  `fl_c5_hit_result_is_tristate_and_aggregates_strongest`,
  `fl_c5_hit_three_passes_continue_after_opaque_with_can_hit_false`,
  `fl_c5_hit_click_only_duplicate_groups_require_down_and_up`,
  `fl_c5_hit_component_identity_reuses_owner_but_retains_duplicate_groups`,
  `fl_c5_pointer_drag_discards_event_timestamps_then_follows_with_move`,
  `fl_c5_pointer_exit_releases_group_history_and_drag_state`,
  `fl_c5_pointer_cpp_paths_accept_nonfinite_coordinates_and_timestamps`,
  `fl_c5_hit_component_shared_target_down_up_matches_cpp_probe`,
  `fl_c5_nested_pointer_authored_child_routing_matches_cpp_probe`,
  `fl_c5_component_list_pointer_reverse_overlap_matches_cpp_probe`,
  `fl_c5_pointer_fp_nonfinite_coordinates_match_cpp_probe`,
  `fl_c5_focus_semantic_focus_state_and_owner_safe_focus_accessors`,
  `fl_c5_focus_semantic_manager_switch_is_identity_noop_and_restores_internal`,
  `fl_c5_focus_semantic_batches_snapshot_clear_and_keep_focus_then_semantic_fifo`,
  `fl_c5_focus_semantic_callback_generated_batches_obey_phase_snapshots`,
  `fl_c5_focus_semantic_recorded_semantic_manager_boundaries_keep_call_order`,
  `fl_c5_focus_queue_chained_callback_waits_source_contract`,
  `fl_c5_focus_queue_snapshot_and_duplicate_source_contract`,
  `fl_c5_focus_manager_switch_order`, `fl_c5_focus_state`,
  `fl_c5_semantic_queue_focus_then_semantic_phase_contract`, and
  `fl_c5_semantic_queue_snapshot_null_and_duplicate_source_contract`. The
  `state_machine_hit_trait_required`,
  `state_machine_hit_concrete_types_required`,
  `state_machine_hit_three_pass_order_required`, and
  `state_machine_focus_then_semantic_phase_required` ratchets enforce the
  owner structure. These proofs cover shared-target dedup and
  duplicate listener append; reset → prepare → process; opacity propagation
  without skipping cleanup; Artboard-first/draw-chain sorting and counter
  re-sort; shape/layout/text geometry; nested authored routing; component-list
  reverse routing and opaque→exit cleanup; drag disable/enable; provider opacity
  upgrade/discard rules; frame-origin transforms; focused-manager switch order;
  and provider/nested/list-only `hasListeners`.
- [x] DataContext bind rebind and clear: `B-BIND` and `S-BIND` are landed
  through `first_factory_pointer_prepares_and_applies_fixed_bindings_before_callback`,
  `listener_missing_context_hydration_keeps_the_table_until_context_arrives`,
  `fl_c5_bind_staged_main_and_globals_apply_only_through_primary_bind`,
  `fl_c5_bind_null_matrix_keeps_every_cpp_branch_distinct`,
  `fl_c5_bind_data_context_and_rebind_preserve_artboard_machine_order`,
  `fl_c5_bind_setters_preserve_an_existing_unregistered_context`,
  `fl_c5_bind_inherit_a_then_b_retains_the_prior_registration_hazard`,
  `fl_c5_bind_shared_context_repoints_all_registered_machine_sinks`,
  `fl_c5_bind_typed_context_apis_delegate_without_signature_changes`,
  `fl_c5_bind_family_typed_default_context_matches_cpp_probe`,
  `fl_c5_bind_null_matrix_matches_pinned_cpp_members`,
  `fl_c5_inherit_context_a_then_b_retains_pinned_cpp_registration_hazard`,
  and `fl_c5_complete_view_models_main_then_globals_matches_pinned_cpp`. The
  `state_machine_bind_primary_family_required`,
  `state_machine_bind_null_branches_distinct_required`,
  `state_machine_internal_context_listener_before_script_required`, and
  `state_machine_typed_context_primary_delegation_required` ratchets enforce
  the family. These proofs cover all distinct null
  branches; staged main/global setters; completion order and cross-model global
  occupancy; complete → artboard → machine bind; bind-null’s limited unbind;
  `bindDataContext(nullptr)` failure; inherited A→B prior-registration hazard;
  setter/getter, artboard-only relink, subtype-only rebuild, listener-cell
  clear/relink, scripted context pass order, and destructor unbind order.
- [x] Event application and chained reports: `B-EVENT`, `B-LVM`, and
  `S-EVENT` are landed through
  `synchronous_pointer_events_survive_the_followup_advance_once_in_authored_order`,
  `synchronous_and_advance_events_share_one_cycle_in_cpp_order_without_replay`,
  `fl_c5_event_host_drain_leaves_the_core_queue_for_apply_events`,
  `fl_c5_event_apply_batches_chaining_and_exact_100_cap`,
  `fl_c5_event_listener_fire_reports_live_payload_before_advance`,
  `fl_c5_event_mid_callback_visibility_excludes_the_reporting_snapshot`,
  `fl_c5_event_trigger_zero_suppression_and_duplicate_listener_fifo`,
  `fl_c5_event_listener_major_event_minor_single_and_multi_order`,
  `fl_c5_event_bubbling_precedes_the_recorded_audio_seam_through_two_ancestors`,
  `fl_c5_apply_events_chaining_and_listener_order`,
  `fl_c5_apply_events_100_batches`, `fl_c5_event_mid_callback_visibility`,
  `fl_c5_trigger_zero_suppression`, `fl_c5_event_bubbling_audio_seam_order`,
  and `fl_c5_live_event_projection`. The
  `state_machine_event_apply_order_required`,
  `state_machine_event_exact_100_batches_required`,
  `state_machine_event_pending_cursor_required`, and
  `state_machine_vm_listener_firing_boundary_required` ratchets enforce the
  queue semantics. These proofs show both pending queues
  are snapshotted/cleared before callbacks; events precede VM reports; callback
  inspection sees only newly pending reports; exactly 100 batches run and the
  boundary warning semantics are retained; batch 101 remains pending; single
  listener breaks after its first match while multi-input listeners continue
  across events; local dispatch precedes bubbling and the recorded audio seam;
  host draining is isolated; trigger zero and signed zero are suppressed.
  The former `flc5-vm-listener-firing-boundary` gap is closed: WP6 restored
  queue-on-one-advance/apply-at-next-new-frame behavior and all four
  claimed-path probes, including NESTED-relative, now require strict per-step
  equality.
- [x] Zero-second and floating-point edges: `B-ADV`, `B-HIT`, `B-LAYER`,
  and `S-ADV` prove that every
  C++-corresponding path forward `+0`, `-0`, NaN, positive/negative infinity,
  and negative ordinary values without Rust finite validation. Cover advance
  seconds, pointer positions/timestamps, event delays, transition mix/duration,
  animation duration, spilled time, frame origin, and singular transforms.
  Keep validation only on separately named Rust convenience entry points.
- [x] Advance return and pending work: `B-ADV`, `S-ADV`, and `S-ORDER` are
  landed through `state_machine_viewmodel_trigger_conditions_match_cpp_probe`,
  `fl_c5_advance_raw_order_and_clean_zero_bookkeeping`,
  `fl_c5_advance_new_frame_false_preserves_the_sticky_latch`,
  `fl_c5_advance_fp_values_forward_without_validation_and_zero_forces_facade`,
  `fl_c5_advance_bind_generated_report_is_a_raw_return_term`,
  `fl_c5_advance_five_passes_probe_transitions_unconditionally`,
  `fl_c5_advance_view_models_false_skips_only_data_context_advancement`,
  `fl_c5_advance_focus_chaining_and_hidden_target_boundaries`,
  `fl_c5_raw_advance_order_matches_pinned_cpp`, `fl_c5_advance_return_terms`,
  `fl_c5_advance_fp_matrix`, `fl_c5_advance_view_models_false`,
  `fl_c5_five_pass_unconditional_probe`, and `fl_c5_zero_delta_bookkeeping`.
  The `state_machine_advance_raw_order_required`,
  `state_machine_advance_return_terms_required`,
  `state_machine_advance_unconditional_settlement_required`, and
  `state_machine_advance_clean_zero_fast_path` ratchets enforce the result.
  These proofs show raw new-frame order is
  draw-sort check → focus batch → semantic batch → apply events → clear latch →
  pre-layer binds → authored layers → converter advance → every input
  `advanced`; same-frame calls retain state-change flags; reports created after
  application wait for the next new frame; raw/facade return terms differ as
  pinned; both signed zeros force facade keep-going; no clean-zero fast path;
  and every one of five settlement passes probes transitions unconditionally.
- [x] Keyframe DataBind lifecycle: `B-KEY`, `S-KEY`, and `S-LIFE` prove
  first source bind per keyframe
  target, supported number/color/bool/string holders, traversal order,
  holder-before-clone sequence, initialize before converter installation,
  enrollment and live resolution, already-bound-context immediate binding,
  converter advancement, duplicate build behavior, remove-in-build-order,
  removal during processing hazard, and destructor tracking cleanup.
- [x] Clone remount and teardown isolation: `B-LIFE`, `S-LIFE`, and
  `S-FIELDS` distinguish the approved
  Rust snapshot from a cold remount. Prove immutable definitions may share but
  mutable layers, random scratch, trigger IDs, hit/group pointer state, event
  and notification queues, registrations, script tables, bind occurrences,
  contexts, and callback sinks do not alias. Prove only snapshots retain
  pending owned values; cold remounts are empty; `dispose` detaches nested
  registrations; observable Drop order matches the C++ owner boundary.
- [x] Direct C++ file correspondence: `B-API`, `S-OWNER`, `S-API`, and
  `S-SEAM` require the two new focused owner
  files, keep `state_machine_layer_instance.rs` as the private-layer owner,
  reduce both old files to thin entry/re-export surfaces, keep `artboard.rs`
  wrappers delegating only, preserve every W4 §C public name, and reject any
  displaced implementation or a false fidelity claim across a recorded seam.

- [x] Permanent structural ratchets: the complete checker suite exercises
  every `S-*` key, including both thin entry points, public export hubs,
  rejected compensation mechanisms, and deferred-owner non-promotion.

## Out-of-scope recorded seams

FL-C5 must carry these dependencies as `RECORDED`; it must not make their owner
rows faithful or implement their internals under a StateMachine filename.

| Status | Deferred owner | FL-C5-visible seam | Owning row / required disposition |
| --- | --- | --- | --- |
| **RECORDED** | `src/listener_group.cpp` | Hover/click phase, consumed/dragged, disabled state, and per-pointer group internals behind the ListenerGroup-shaped seam | Pending FL-D row `B6-0259`; keep Rust pointer capture/history tables until that row lands. Delete only per-listener orchestration displaced by FL-C5 hit owners. |
| **RECORDED** | `src/animation/text_input_listener_group.cpp` | Text-input listener-group internals | Pending FL-E row `B6-0083`. FL-C5 only preserves construction/routing order to the seam. |
| **RECORDED** | `src/input/gamepad_batch.cpp` | `submitGamepadsFromBuffer` definition and byte parser; scripted broadcast definition | Pending absent row `B6-0241`. Header declarations remain mapped; no FL-C5 implementation claim. |
| **RECORDED** | `src/input/focus_manager.cpp` | Focus tree traversal, cleanup, and manager internals | Pending row `B6-0238`; existing `focus.rs` verdict remains `DIVERGENT`. FL-C5 ports its own selection/queue/process/call ordering only. |
| **RECORDED** | `src/semantic/semantic_manager.cpp` | Manager/tree/node-ID lookup for `enableSemantics`, `setExternalSemanticManager`, `semanticManager`, and `fireSemanticAction` | Pending absent row `B6-0329`. FL-C5 owns the action switch and dispatch ordering behind an instance-owned `SemanticNodeResolver`; the production default is absent, so node `77` returns `false` without fabricating an ordinal mapping. The deferred owner must install the production resolver. |
| **RECORDED** | `src/semantic/semantic_data.cpp` | Semantic node callback internals | Pending absent row `B6-0327`. Same recorded boundary. |
| **RECORDED** | `src/audio_event.cpp` | Actual audio playback at the tail of `notifyEventListeners` | Pending absent row `B6-0113`. FL-C5 ports listener → bubble → typed `AudioEventSeam` order. Its production default records count and last occurrence observably; the deferred owner later replaces that handoff with playback. |
| **RECORDED** | State-machine/artboard importers | Import-stack mechanics and importer ownership | Existing adapted rows `B6-0228` and `B6-0212`. FL-C5 represents `state_machine.cpp` import/onAdded semantics through the accepted Rust import architecture without changing their dispositions. |

## Compensation KEEP / DELETE decisions

Every W4 §B mechanism is accounted for. `KEEP` means a documented Rust
adaptation, not permission to bypass a C++-corresponding primary path.

| Rust-only mechanism (W4 §B) | Binding verdict | Closure citation and required proof |
| --- | --- | --- |
| Retained definition arena + numeric `state_machine_index` | [x] **KEEP** | `stateMachine`, `m_machine`, W4 public API list; `B-LIFE`, `B-API`, `S-FIELDS`. |
| Public snapshot `Clone` | [x] **KEEP** | Deleted copy-constructor and lifecycle rows; `B-LIFE`, `S-LIFE`. |
| `listener_definitions: Arc<Vec<_>>` | [x] **KEEP** | Listener slot/group rows; stable immutable identity, no compaction; `S-SLOTS`. |
| File/default/owned VM catalogs and selectors | [x] **KEEP** | Bind-family and public typed-context rows; they delegate to the primary bind family. |
| `requires_post_update_state_probe` and `post_update_probe_pending` | [x] **DELETE** | Advance rows; `B-ADV` proves unconditional probing each settlement pass; `S-ADV` rejects both flags/gates. |
| Cached `changed_state_count` as sole state | [x] **DELETE** | Restore per-layer flags and `stateChangedByIndex`; an aggregate may remain only as a derived cache; `B-LAYER`, `S-LAYER`. |
| `has_advanced_once` + clean zero-delta fast path | [x] **DELETE** | Raw/facade advance rows; `B-ADV` proves bind/layer/input bookkeeping still runs; `S-ADV`. |
| Public/core event dual cursors | [x] **KEEP** | Event queue/report access rows; `B-EVENT`, `S-EVENT`. |
| Rust notification queue object | [x] **KEEP** | Listener-VM fields/report rows; duplicate FIFO and weak sink adaptation. |
| `RuntimeDataBindContainerQueue`, occurrence vector/enum | [x] **KEEP** | Definition/data-bind/keyframe rows; authored cross-family order remains one logical queue. |
| Per-animation keyframe graph cache | [x] **KEEP** | Keyframe rows; prove equivalence to on-demand C++ clones and occurrence isolation. |
| `owned_view_model_rebind_sink` | [x] **KEEP** | Bind/relink rows; pushed structural replacement adaptation. |
| Pointer capture/history tables | [x] **KEEP until FL-D** | Hit rows and recorded `listener_group.cpp` seam; delete only displaced per-listener traversal. |
| Finite/nonnegative validation + `Result` host seams | [x] **KEEP only on distinct Rust convenience APIs** | C++ pointer/advance rows must forward all FP values; `B-HIT`, `B-ADV`, `S-ADV`. |
| Script lifecycle maps/flags | [x] **KEEP** | Constructor/bind/init/lifecycle rows; facade mount timing adaptation. |
| `scripted_input_group_generation` and synchronization API | [x] **KEEP** | Public API list and constructor seam; late-mount adaptation. |
| Terminal retained `script_error` | [x] **KEEP** | Listener/event/script lifecycle rows; ordinary protected-call failure is consumed, selected resource failure stays terminal. |
| `active_owned_view_model_advance_context` | [x] **KEEP** | Advanced-context/reset and public context API rows. |
| `scripted_facade_root_view_model` identity cache | [x] **KEEP** | Bind/lifecycle rows; repeated A→A versus A→B facade adaptation. |
| Per-layer monotonic trigger-layer ID | [x] **KEEP** | Layer fields/Clone rows; regenerated on snapshot to prevent aliasing. |
| Per-layer evaluated-random-weight scratch | [x] **KEEP** | Layer scratch row; equal output plus cross-instance isolation. |
| Typed bindable arrays and transition-duration occurrences | [x] **KEEP** | Definition/property/keyframe rows; preserve duplicate occurrences and converter ownership. |
| Action owner arena/handles | [x] **KEEP** | Definition/action ordering and public scripted APIs; stable-owner adaptation. |
| Definition-level `requires_post_update_state_probe` scan | [x] **DELETE** | Same unconditional-probe proof as the instance flags; `S-ADV`. |
| Host report snapshot/refresh projection | [x] **KEEP** | `reportedEventAt`, host drain, and public event-context rows; live payload refresh required. |
| Formula-random injection/count APIs | [x] **KEEP** | W4 public API list; oracle/test seam. |
| Transition-duration and VM-trigger probes | [x] **KEEP** | W4 public API list; differential introspection seam. |
| Directional focus convenience APIs | [x] **KEEP** | W4 public API list; remain distinct delegating extensions. |
| Alternate boolean return shapes | [x] **KEEP on distinct Rust APIs** | W4 public API list; C++-corresponding primary methods retain their own result/FP behavior. |

Verification receipt: all 25 `KEEP` decisions above are tied to a behavioral
proof key, structural rule, or the compile-time W4 public API inventory; the
focused runtime and `cpp_probe` batteries execute those cited receipts. All
four `DELETE` rows are absent as stored production mechanisms:
`requires_post_update_state_probe`, `post_update_probe_pending`,
`has_advanced_once`, and a stored/cached `changed_state_count` field do not
exist. The public `changed_state_count()` compatibility method remains only as
an on-demand count of retained per-layer flags. The masking differentials are
`fl_c5_five_pass_unconditional_probe`,
`fl_c5_zero_delta_bookkeeping`, and
`fl_c5_state_changed_layers_and_convergence_match_cpp_probe`; the
`S-ADV`/`S-LAYER` injected negatives reject reintroducing the deleted gates,
scan, fast path, or cached-only representation.

## Public API preservation list

Every row from W4 §C must remain reachable through the thin entry points.
`S-API` must check names and visibility; the cited downstream evidence must
compile after the split.

| Public Rust API to preserve | Current definition / evidence | Required adaptation |
| --- | --- | --- |
| `RuntimeStateMachine` public `global_id`, `name`, `inputs`, `layers` | `state_machine/state_machine.rs:9-15`; `artboard.rs:3687-3693,4444-4480` | Re-export unchanged from the new definition owner. |
| `RuntimeStateMachine::scripted_listener_actions` | `state_machine/state_machine.rs:124-126`; scripted lifecycle tests | Keep filtered Rust convenience view. |
| `StateMachineInstance: Clone` | `state_machine/state_machine_instance.rs:2040-2205`; `flow_session.rs:1250-1252` | Keep approved non-aliasing snapshot. |
| `state_machine_index` | `state_machine/state_machine_instance.rs:4888`; artboard and `cpp_probe` consumers | Keep stable numeric handle. |
| `input_index_named` | `state_machine/state_machine_instance.rs:5049`; flow/scene/C API/public tests | Re-export unchanged. |
| Indexed `set_bool`, `set_number`, `fire_trigger` | `state_machine/state_machine_instance.rs:5055-5087`; flow/scene/public tests | Keep mutating convenience APIs. |
| `focus_up`, `focus_down`, `focus_left`, `focus_right` | `state_machine/state_machine_instance.rs:5100-5114`; higher-level focus routing | Keep directional extensions. |
| `key_input`, `text_input`, `gamepad_dispatch` | `state_machine/state_machine_instance.rs:5325-5510`; facade/input tests | Keep typed host-input APIs; do not claim the gamepad buffer parser. |
| Pointer API families with owned/event context, timestamp, or script host | `state_machine/state_machine_instance.rs:6391-7035`; scene/fuzz/workspace consumers | Keep signatures; route C++-corresponding base paths through hit owners and keep validating convenience paths distinct. |
| `pointer_down_with_event_context`, `pointer_up_with_event_context` | `state_machine/state_machine_instance.rs:6410,6659`; pointer context tests | Keep rendered occurrence metadata. |
| `take_reported_events` | `state_machine/state_machine_instance.rs:11874-11891`; flow/scene | Keep host cursor isolation. |
| `reported_event_snapshot` | `state_machine/state_machine_instance.rs:11853-11859`; `cpp_probe` | Keep immutable projection. |
| `has_pending_listener_view_model_reports` | `state_machine/state_machine_instance.rs:11813-11815`; frame loop/tests | Keep private-queue visibility. |
| `script_error`, `retain_scripted_object_data_context_error` | `state_machine/state_machine_instance.rs:4842-4853`; facade/flow | Keep terminal error channel. |
| `scripted_objects`, instance `scripted_listener_actions` | `state_machine/state_machine_instance.rs:3558-3566`; facade/lifecycle tests | Keep occurrence exposure. |
| Script occurrence installation/hydration APIs (`set_script_instance_for_global`, `set_script_input_for_global`, `set_scripted_listener_action_instance`, `set_scripted_object_instance`, `hydrate_and_initialize_*`, `install_scripted_object_data_context`) | `state_machine/state_machine_instance.rs:3535-3549,3998-4035,4351-4694`; facade/golden runner | Keep late-mount bridge. |
| `synchronize_scripted_input_groups` | `state_machine/state_machine_instance.rs:3435`; facade | Keep generation-based cache rebuild. |
| Scripted binding/query family (`scripted_listener_action_input_snapshots`, `bind_scripted_listener_action_sources`, `bind_scripted_listener_input_source`, `bind_scripted_listener_converter_own_sources`, `finalize_scripted_listener_input_sources`, converter occurrence/snapshot APIs) | `state_machine/state_machine_instance.rs:3569-3680,4098-4466`; facade/golden/tests | Keep graph/converter/VM bridge. |
| Facade context transactions (`begin_scripted_object_data_context_bind`, `begin_retained_scripted_object_data_context_rebind`, `finish_scripted_object_data_context_bind`) | `state_machine/state_machine_instance.rs:10722-10807`; facade/golden | Keep fallible phased wrapper around primary bind family. |
| Transaction transfer (`adopt_scripted_listener_action_state_from`, `rehome_owned_data_context_for_transaction`) | `state_machine/state_machine_instance.rs:4746-4839`; flow | Keep candidate/commit adaptation. |
| Context-binding family (`bind_empty_data_context`, `bind_default_view_model_context`, `bind_view_model_instance_context`, `bind_imported_view_model_context`, `bind_owned_view_model_context`, `bind_owned_view_model_handle`, `bind_owned_view_model_context_handle`, `bind_owned_view_model_context_mut`, `bind_owned_view_model_contexts`, `bind_script_artboard_data_context`) | `state_machine/state_machine_instance.rs:10527-10852,11206`; facade/probes | Keep typed wrappers, delegating to C++-shaped primary operations. |
| `set_bindable_{number,boolean,integer,color,string,enum,asset,artboard,list,trigger,view_model}_for_data_bind` | `state_machine/state_machine_instance.rs:7926-8146`; artboard/probes | Keep direct typed mutation seams. |
| Default-VM source setter/query/handle families | `state_machine/state_machine_instance.rs:8204-9345`; `cpp_probe` | Keep typed path/source handle APIs. |
| Imported/owned VM source setter families | `state_machine/state_machine_instance.rs:9385-10070`; `cpp_probe` | Keep ownership-specific adaptations. |
| Converter binding APIs (`bind_state_machine_data_bind_source`, `bind_state_machine_data_converter_own_sources`, `finalize_state_machine_data_bind_source`, `rebind_state_machine_data_converter_final_input`) | `state_machine/state_machine_instance.rs:4165-4235`; workspace consumers | Keep graph build-phase bridge. |
| `update_data_binds_apply_target_to_source` | `state_machine/state_machine_instance.rs:11636-11778`; downstream runtime/facade | Keep explicit public container update. |
| Formula-random APIs | `state_machine/state_machine_instance.rs:10565-10575`; differential probes | Keep deterministic oracle seam. |
| Transition-duration probes | `state_machine/state_machine_instance.rs:10583-10594`; differential probes | Keep occurrence introspection. |
| VM-trigger probes | `state_machine/state_machine_instance.rs:11893-11936`; differential probes | Keep trigger-cell introspection. |
| `bindable_*_value_for_data_bind` and default-source query families | `state_machine/state_machine_instance.rs:8204-8438`; differential probes | Keep graph introspection. |
| `StateMachineEventContext`, `StateMachineReportedEvent` accessors | `state_machine/event_report.rs:45-67,175-210`; flow/scene | Re-export ownership-safe report/context projections unchanged. |

## W34 proof-gap disposition

The W32 non-blocking observations are not silently promoted into stronger
claims than the receipts establish:

- [x] Definition null-slot behavior is now live at the safe definition seam:
  `--state-machine-definition-null-hole-sample` drives the pinned C++
  `StateMachineImporter::readNullObject` and observes the exact indexed null
  slot, duplicate layer names, listener indices, and duplicate DataBind
  property keys without entering C++ `onAddedDirty`, whose authored null
  dereference is itself malformed behavior. The serialized Rust fixture
  preserves the corresponding indexed `None` hole and both duplicate bind
  occurrences, and its exact input/layer sequences are compared with the C++
  seam. The original well-formed same-byte definition differential also
  retains all name, count, order, duplicate-first-match, and case-sensitivity
  comparisons. Typed lookup uses a separate well-formed live fixture because
  pinned C++ `getNamedInput` dereferences every instantiated slot.
- [ ] A single live definition fixture spanning null and duplicate
  DataBind/ScriptedObject occurrences is still absent. Existing focused tests
  prove duplicate listener/bind occurrences and script occurrence ownership
  independently, but combining malformed nullable imports across all
  collections would enter pinned C++ null-dereference paths rather than add a
  safe observable oracle.
- [ ] The hardest cross-collection duplicate/null fixture remains absent for
  the same reason. Closure relies on per-collection authored-order receipts
  and does not claim a live all-collections malformed execution.
- [ ] The complete bind null matrix is not live in one table-driven
  differential. The distinct implemented null branches and inherited A→B
  registration hazard remain covered by focused and live probes, but
  unconstructed null combinations remain source-cited rather than checked.
- [x] Clone isolation now mutates or identity-checks reporting/current/bubble
  queues, listener-report queues, pointer state, hit owners, listener groups,
  nested registrations, detached primary context state, callback dirt sinks,
  layer identities, and cold script tables. Converter/bind-graph isolation is
  also exercised by the existing state-machine snapshot differentials; no
  shallow-all-fields claim is inferred from a single pointer comparison.
- [x] `fl_c5_hit_sort_preserves_the_exact_adversarial_swap_order` observes the
  actual in-place algorithm with the Artboard-first swap, duplicate drawable
  identities, and three reversed drawable identities. It would fail under a
  stable sort or a scan that stops after the first duplicate.
- [x] `fl_c5_keyframe_initialize_converter_and_enrollment_are_observed_end_to_end`
  first observes the initialized clone's converted source value, then dirties
  the live source and requires both runtimes to produce a second converted
  value through the enrolled state-machine container.

## Permanent enforcement required before publication

The structural checker and injected negatives must permanently reject:

1. either new owner file missing, or implementation remaining in either thin
   entry point;
2. `filter_map`, flattening, or deduplication of authored listener/input/bind/
   script occurrences;
3. per-advance reconstruction or replacement maps/sets/sorts for pinned owner
   traversal;
4. a boolean-only hit result, per-listener hit traversal, missing concrete hit
   type, missing prepare pass, or stopping iteration after opacity;
5. failure to re-sort on draw-order counter change, forward component-list
   traversal, or failure to convert occluded pointer work to exit cleanup;
6. a clean zero-delta early return or finite validation in a C++-corresponding
   advance/pointer path;
7. `requires_post_update_state_probe`, `post_update_probe_pending`, or the
   definition capability scan;
8. cached-only changed-state reporting or missing per-layer current/changed
   access;
9. bind-family null branches collapsed together, machine-before-artboard bind,
   or typed APIs bypassing the primary bind operations;
10. trigger-reset-to-zero reports, VM-before-event delivery, a changed
    100-iteration bound, or current-batch exposure through pending-event APIs;
11. audio execution claimed locally rather than recorded, or bubbling ordered
    after the audio seam;
12. shallow snapshot sharing of mutable occurrence state or missing explicit
    nested-event detach;
13. keyframe last-bind selection, wrong holder set, converter-before-initialize,
    or state drop before bind removal;
14. deletion/visibility reduction of any W4 §C public API; and
15. any deferred owner marked faithful without its own source row closing.

## Publication packet

Candidate identity:

- immutable combined-family production candidate:
  `afcb705806e0f8d0d21196064cc9456daeb237b9`. FL-C5 round-two production
  landed as `ea38d33b`, FL-B round-two corrections as `edddf491`,
  round-four corrective production as `2e2d3c6d`, and `95333c41` completed
  the tools-enabled probe target. E2 evidence was published as `eaf8a6f6`;
  the retained-arena hookup was authorized by `6bb6ba31`; and round-five
  corrective production landed as `691c5262`, adding per-source chain
  atomicity, retained-arena resolution for all consumers, repository-wide
  semantic guards, and recursive tracked-receipt validation. E3 evidence was
  published as `50bf85e8`; round-six corrective production landed as
  `9434b39c`, adding per-animation chain atomicity, nested-simple event
  delivery, instance-owned seam mechanics, restored reset order, and
  retained-definition blend proofs. E4 evidence was published as `76ab8d86`.
  Round-seven corrective `f4f013dd` added callback-major singleton delivery;
  pre-freeze scout corrective `192cbbbe` moved bubbling to report time,
  narrowed settlement, and made ownership ratchets alias-resistant. Merge
  commit `171b5703` then integrated origin/main boundary `afe71e30`, which
  removed `nux-apple-runtime` and Apple packaging for a pure engine boundary.
  E5 evidence was published as `3bef19da`. Independent round-seven verdicts
  W71/W72/W73 rejected the invented Blend1D compensation, incomplete
  failing-owner bubble/audio tail, and evadable ownership detector.
  Round-eight corrective `99ef7700` reverted the compensation with a
  symmetric differential, completed the failing-owner chain, and replaced
  the detector with a syn-AST ownership resolver plus a fail-closed
  tripwire. The orchestrator's novel module-re-export evasion was detected.
  E6 evidence was published as `499d86b8`. Independent round-eight verdicts
  W76/W77/W78 rejected incorrect negated/mixed cfg handling, remaining
  resolver and token-composition evasions, self-service allow suppression,
  non-reproducible detector packaging, terminal-error visibility before the
  complete chain, removal of the BlendDirect clone/remount proof, and E6
  fingerprint/provenance generated from uncommitted Cargo-lock churn.
  Round-nine corrective `afcb7058` implements exact cfg evaluation,
  module-re-export and associated-type resolution, fail-closed scanning of
  macro and attribute token streams, a checker-validated per-site registry
  allowlist, and reproducible standalone detector packaging with its committed
  `Cargo.lock`. It withholds a failing owner's terminal error until bubbling
  and audio unwind finish, restores an honest BlendDirect clone/remount proof,
  and records the FL-G03 disposition.
  This immutable candidate remains main-integrated; no post-acceptance rebase
  remains. The E7 trace names the full `afcb7058` candidate and is generated
  only after the intended evidence set is staged and a porcelain gate confirms
  no unstaged/untracked or Cargo-lock churn. W79 records the fast-suite run
  over the exact round-nine production later frozen at that commit. The
  floor7 receipts on `171b5703` remain the standing complete-floor reference
  under the dated coordinator policy below;
- pinned C++:
  `d788e8ec6e8b598526607d6a1e8818e8b637b60c`;
- trace source fingerprint and Rust runner provenance:
  `docs/runtime-frame-loop-trace.json`; `make
  runtime-frame-loop-port-check` confirms both are current for this final
  evidence tree.

Coordinator floor-policy directive (Levi, 2026-07-30): interim correction
rounds bind only the fast behavioral battery—runtime library, tools-enabled
C++ differentials, and both ordinary and scripted golden corpora. The complete
pixel same-runner, pixel static-reference, browser/WebGPU, and committed-tree
size floor runs once on the final independently accepted candidate immediately
before promotion, not once per correction round. Six consecutive identical
complete-floor cycles, P2 through P7, provided zero new information for the
delivery-semantics corrections. The floor7 receipts on `171b5703` therefore
remain the standing full-floor reference; they are not relabeled as
candidate-specific `afcb7058` receipts.

The four nested-event ownership ratchets now use a syn-based resolver across
every non-owner Rust source. It resolves use declarations and aliases, glob
imports, type aliases, raw identifiers, spaced paths, and angle-bracket UFCS.
Macro or otherwise unresolved guarded-member tokens are a fail-closed
regression tripwire rather than an enumerated evasion list. A deliberately
blessed site must have an explicit `[[owner_boundary_allow]]` registry row in
`docs/runtime-frame-loop-gaps.toml` naming its file, guarded kind, and exact
expected site count. Inline comments and string literals have no suppression
power, and the checker rejects registry drift in either direction. Permanent
negatives retain every W66/W68/W71/W72/W73 spelling and fully-qualified
controls; exotic future spellings containing a guarded type or member trip the
token rule.

Gate receipts:

| Gate | Green receipt |
| --- | --- |
| W34 family-review repairs | `docs/runtime-frame-loop-fl-c5-evidence/W34-fix-round-report.md` maps O1–O4 and S1–S5/S7 (plus the S6 clarification) to their production changes, pinned C++ citations, strengthened tests, honest proof-gap dispositions, and refreshed acceptance receipts. |
| W41 FL-C5 round-two repairs | `docs/runtime-frame-loop-fl-c5-evidence/W41-report.md` records the binding resolver/audio/delegation/O1/inventory/five-pass seams and the historical P3 receipts. It explicitly treats internal closeouts as non-acceptance context; the independent rejection verdicts remain tracked separately. |
| W43 FL-B round-two corrections | `docs/runtime-frame-loop-fl-c5-evidence/W43-report.md` records retained-handle `loopValue`, the doomed keyed-object sink, their live differentials, and the historical P3 rerun. |
| W50/W51/W52 round-four reviews | `docs/runtime-frame-loop-fl-c5-evidence/W50-oracle-round4.md`, `W51-standards-round4.md`, and `W52-flb-round4.md` preserve the three independent E2 rejection verdicts: production chain atomicity, retained-arena completeness, owner/semantic/stamp guards, reproducible provenance, and publication consistency. |
| W53 round-five corrective | `docs/runtime-frame-loop-fl-c5-evidence/W53-report.md` maps every W50/W51/W52 blocker to the round-five production, live differentials, structural negatives, and pre-E3 green floors. It records that trace fingerprint and runner provenance remained intentionally owned by this final evidence pass. |
| W55/W56/W57 round-five reviews | `docs/runtime-frame-loop-fl-c5-evidence/W55-oracle-round5.md`, `W56-standards-round5.md`, and `W57-flb-round5.md` preserve the three independent E3 rejection verdicts: per-animation chain granularity, nested-simple delivery, seam ownership/ratchet strength, reset order, and retained-definition blend proof coverage. |
| W58 round-six corrective | `docs/runtime-frame-loop-fl-c5-evidence/W58-report.md` maps every W55/W56/W57 blocker to round-six production, four named live differentials, restored teardown/reset order, strengthened structural negatives, and pre-E4 green floors. It records that trace fingerprint and runner provenance remained intentionally owned by this final evidence pass. |
| W66/W68 pre-freeze scout verdicts | `docs/runtime-frame-loop-fl-c5-evidence/W66-prereview.md` preserves the scout’s report-time bubbling, settlement, differential, singleton-batch, ownership-ratchet, and packet findings. `W68-reclear.md` preserves the follow-up UFCS and guarded-enum-alias findings after the first corrective. |
| W67/W69 corrective reports | `docs/runtime-frame-loop-fl-c5-evidence/W67-report.md` records the round-seven report-time/singleton corrective and its complete acceptance battery. `W69-report.md` records the final angle-bracket UFCS and enum-alias checker corrections and 67/67 structural receipt. |
| W71/W72/W73 round-seven verdicts | `docs/runtime-frame-loop-fl-c5-evidence/W71-oracle-round7.md`, `W72-standards-round7.md`, and `W73-flb-round7.md` preserve the three independent rejection verdicts: Blend1D's non-C++ compensation and asymmetric proof, incomplete failing-owner chain completion, plain/glob/type/macro ownership-detector evasions, and the stale publication pointer. |
| W74 round-eight corrective | `docs/runtime-frame-loop-fl-c5-evidence/W74-report.md` maps the W71/W72/W73 blockers to the symmetric Blend1D differential, failing-owner bubble/audio-tail completion, syn-AST resolver and fail-closed token tripwire, the orchestrator-probed module-re-export evasion, and the pre-E6 fast-suite receipts. It records runtime 725/725, tools differentials 823/823, supplemental `nuxie --lib` 147/147, and both golden corpora at 317/317 entries plus 647/647 exact segments with zero divergences. Trace fingerprint and artifact provenance remained intentionally owned by E6. |
| W76/W77/W78 round-eight verdicts | `docs/runtime-frame-loop-fl-c5-evidence/W76-oracle-round8.md`, `W77-standards-round8.md`, and `W78-flb-round8.md` preserve the three independent rejection verdicts: incorrect negated/mixed cfg handling, module/associated-type/macro/attribute evasions, self-service allow suppression, non-reproducible detector packaging, early failing-owner error observability, loss of the held BlendDirect clone/remount proof, and E6 fingerprint/provenance generated from an uncommitted Cargo-lock state. |
| W79 round-nine corrective | `docs/runtime-frame-loop-fl-c5-evidence/W79-report.md` maps every W76/W77/W78 blocker to exact cfg and resolution logic, fail-closed token scanning, the checker-validated registry, committed standalone detector lockfile, full-chain error withholding, restored BlendDirect proof, and the FL-G03 disposition. It records runtime 726/726, tools differentials 823/823, supplemental `nuxie --lib` 147/147, checker 71/71, and both golden corpora at 317/317 entries plus 647/647 exact segments with zero divergences. Trace fingerprint and artifact provenance remained intentionally owned by E7. |
| Seven reopened member behaviors | This W31 run executes one passing test for each named receipt: `state_machine_generic_layer_state_occurrence_matches_cpp_probe` (`currentState`, `stateChangedByIndex`), `state_machine_viewmodel_trigger_conditions_match_cpp_probe` (`advance(seconds,newFrame)`), `flow_pointer_callbacks_receive_event_time_and_the_prior_delivered_position` (`pointerDown`), both `synchronous_*` event-cycle tests (`applyEvents`), `first_factory_pointer_prepares_and_applies_fixed_bindings_before_callback` (`internalDataContext`), and `listener_missing_context_hydration_keeps_the_table_until_context_arrives` (`pointerDown`, `internalDataContext`, `initScriptedObjects`). |
| Focused and broad behavioral floor | `docs/runtime-frame-loop-fl-c5-evidence/W30-report.md`: C++ probes 804/804, `nuxie --lib` 146/146, runtime library 713/713, scripting library 205/205, and `sound` 1/1 exact. W31 reruns the complete runtime library and C++ probe gates below. |
| Structural checker and provenance | Final `make runtime-frame-loop-port-check`: 71/71 checker tests, including recursive tracked-receipt stamps, exact inventory/gating/repository-wide semantic and syn-AST seam-owner negatives, exact registry validation, the fail-closed tripwire, clean-cache locked detector packaging, and mutated-ref/artifact negatives, followed by the live checker accepting E7 `rust_ref`, candidate fingerprint, manifest, Rust runner provenance, and all eight artifact hashes. |
| Standing public/downstream API reference | The E5 sandbox run passed 14/14 code/API cases; only `public_api_exposes_the_default_rust_renderer` could not construct an adapter because that sandbox reported `metal found no adapters`. The historical external `floor-public-api.log` remains the 15/15 adapter-capable receipt. No test was skipped or weakened. |
| E7 ordinary and scripted golden corpora | W79 records both fast-suite comparisons at 317/317 exact entries and 647/647 exact segments, with zero divergences, unsupported features, or not-yet cases. |
| Standing static-reference renderer corpus | `docs/runtime-frame-loop-fl-c5-evidence/floor7-pixel-static.log`: `171b5703`-stamped standing full-floor reference, exact=1,468, byte-exact=837, diverges=0, gated=0. |
| Standing same-runner pixel corpus | `docs/runtime-frame-loop-fl-c5-evidence/floor7-pixel-same.log`: `171b5703`-stamped standing full-floor reference, exact=1,468, byte-exact=1,370, diverges=0, gated=0. |
| Standing browser/WebGPU corpus | `docs/runtime-frame-loop-fl-c5-evidence/floor7-browser.log`: `171b5703`-stamped standing full-floor reference; browser and GPU smoke pass; prohibited surface and CPU presentation are zero; typed readback is one; error-scope, uniform-limit, alpha, and bounded-recovery invariants hold. |
| Standing link-closure size | `docs/runtime-frame-loop-fl-c5-evidence/floor7-size.log`: `171b5703`-stamped standing full-floor reference; scripting off=8,218,120 bytes and scripting on=9,318,856 bytes, both below the 9 MiB (9,437,184-byte) budget. |
| Apple boundary | Upstream boundary commit `afe71e30` on 2026-07-30 removed `nux-apple-runtime` and Apple packaging. Apple/XCFramework/ABI/header packaging is no longer an acceptance leg. All historical Apple receipts remain under `docs/runtime-frame-loop-fl-c5-evidence/superseded/`; the size floor remains operative. |
| E7 fast-suite and publication commands | On `afcb7058`: runtime library 726/726; live tools-enabled C++ differential suite 823/823; supplemental `nuxie --lib` 147/147; both golden comparisons 317/317 and 647/647; `make runtime-frame-loop-port-check` green with 71/71 tests and live provenance; `cargo fmt --all -- --check` green; the pre-trace porcelain gate contains only the intended staged evidence set with no unstaged/untracked or Cargo-lock churn; and working-tree/staged evidence/docs `git diff --check` is green. The complete pixel/browser/size floor is deferred exactly as the coordinator policy requires. Earlier historical counts remain preserved in their reports rather than silently rewritten. |

Remaining out-of-scope work is unchanged and explicitly `RECORDED`: pending
rows `B6-0259` (`listener_group.cpp`), `B6-0083`
(`text_input_listener_group.cpp`), `B6-0241` (`gamepad_batch.cpp`),
`B6-0238` (`focus_manager.cpp`), `B6-0329`
(`semantic_manager.cpp`), `B6-0327` (`semantic_data.cpp`), and `B6-0113`
(`audio_event.cpp`), plus adapted importer rows `B6-0228` and `B6-0212`.
None is promoted by FL-C5.

Compensation verdict: all 25 `KEEP` adaptations remain tied to their
behavioral, structural, or public-API proofs. All four `DELETE` mechanisms
remain absent:
`requires_post_update_state_probe`, `post_update_probe_pending`,
`has_advanced_once`, and stored/cached-only `changed_state_count`. The
on-demand compatibility query is not stored state. No compensation verdict
changed during closure completion.

Closure checklist:

- [x] every member row and all twelve adversarial rows are checked;
- [x] the two new owner modules exist and both legacy files are thin entry
  points/re-exports;
- [x] every compensation `KEEP` is documented in the corresponding member
  proof and every `DELETE` has a C++ differential proving the behavior it
  masked;
- [x] every recorded seam names its owning row and remains unpromoted;
- [x] every structural rule has a passing injected negative control
  (`make runtime-frame-loop-port-test`: use the live count reported by the
  gate receipt);
- [x] focused Rust tests and pinned-C++ differentials are green;
- [x] the E6 interim fast behavioral floor and structural/provenance gates
  required by the 2026-07-30 coordinator policy are green; floor7 remains the
  standing complete-floor reference, and the next complete pixel/browser/size
  run is reserved for the final independently accepted candidate before
  promotion;
- [x] exact source citations, test names, checker counts, gate counts, trace
  receipt, and immutable candidate identity are recorded here and in the
  mechanical status layers; and
- [x] no performance measurement was run or used to select work.
