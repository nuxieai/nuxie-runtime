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
not part of FL-C5 and is not claimed here.

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

## Complete member closure

The status column is the W4 audit status before FL-C5 work. “Binding
adaptation” in a proof cell means the directives approve the Rust ownership
shape but not omission of the observable behavior.

### `StateMachine` definition and `state_machine.cpp`

Rust destination for every row in this table:
`crates/nuxie-runtime/src/state_machine/state_machine.rs`.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W1–W4 |
| --- | --- | --- | --- | --- |
| `m_Layers` (`state_machine.hpp:20`) | faithful-looking | `B-DEF`: authored order and duplicate definitions | `S-OWNER`, `S-SLOTS`, `S-ORDER` | Empty vector; duplicate names/pointers retained. |
| `m_Inputs` (`state_machine.hpp:21`) | faithful-looking | `B-DEF`: indexed `None`/null compatibility slot | `S-SLOTS`, `S-FIELDS` | Unknown serialized input creates a hole and still counts. |
| `m_Listeners` (`state_machine.hpp:22`) | divergent | `B-DEF`: inert unbuildable listener retains authored index | `S-SLOTS` | Malformed listener followed by valid listener; no index compaction. |
| `m_dataBinds` (`state_machine.hpp:23`) | scattered | `B-DEF`: one authored occurrence order projected into graph/container queue | `S-ORDER`, `S-FIELDS` | Duplicate bind targets retained even when lookup maps select first/last as specified. |
| `m_scriptedObjects` (`state_machine.hpp:24`) | faithful-looking | `B-DEF`: borrowed occurrence order and returned-list independence | `S-ORDER`, `S-FIELDS` | Duplicate source pointer entries retained; clearing returned list does not mutate owner. |
| `addLayer` (`state_machine.hpp:26`; `state_machine.cpp:82-85`) | scattered | `B-DEF`: import-time append adaptation | `S-SLOTS`, `S-ORDER` | Null and duplicates append without validation. |
| `addInput` (`state_machine.hpp:27`; `state_machine.cpp:87-90`) | scattered | `B-DEF`: import-time append/null-object adaptation | `S-SLOTS` | Null compatibility input remains at exact index. |
| `addListener` (`state_machine.hpp:28`; `state_machine.cpp:92-95`) | divergent | `B-DEF`: every authored listener gets a slot | `S-SLOTS` | Unbuildable listener is inert, never filtered. |
| `addDataBind` (`state_machine.hpp:29`; `state_machine.cpp:97-100`) | scattered | `B-DEF`: authored append represented once | `S-ORDER` | Null/duplicate occurrence is not silently deduplicated. |
| constructor (`state_machine.hpp:32`; `state_machine.cpp:12`) | scattered | `B-DEF`: zero-member machine and complete immutable-build adaptation | `S-OWNER`, `S-FIELDS` | No synthesized layers/inputs/listeners/binds/scripts. |
| destructor (`state_machine.hpp:33`; `state_machine.cpp:14`) | faithful-looking | `B-SOURCE`: RAII owner/borrow split | `S-FIELDS` | Same definition cannot be independently owned by two collections. |
| `import` (`state_machine.hpp:35`; `state_machine.cpp:70-80`) | divergent | `B-DEF`: missing-artboard status and superclass/import adaptation | `S-SEAM`, `S-ORDER` | Missing importer returns failure and does not attach; parse order retained. |
| `layerCount` (`state_machine.hpp:37`) | faithful-looking | `B-DEF`: on-demand length | `S-SLOTS` | Zero and duplicate layers. |
| `inputCount` (`state_machine.hpp:38`) | faithful-looking | `B-DEF`: counts null slot | `S-SLOTS` | One compatibility hole yields one. |
| `listenerCount` (`state_machine.hpp:39`) | divergent | `B-DEF`: authored count including inert slot | `S-SLOTS` | Malformed then valid yields count two. |
| `dataBindCount` (`state_machine.hpp:40`) | scattered | `B-DEF`: definition-level authored occurrence count | `S-ORDER` | Typed decomposition must not change count. |
| `addScriptedObject` (`state_machine.hpp:41`; `state_machine.cpp:162-165`) | scattered | `B-DEF`: import collection adaptation | `S-ORDER` | Same borrowed pointer appended twice. |
| `scriptedObjects` (`state_machine.hpp:42-45`) | faithful-looking | `B-DEF`: complete ordered borrowed view | `S-API`, `S-ORDER` | Caller mutation cannot mutate definition. |
| `input(name)` (`state_machine.hpp:47`; `state_machine.cpp:102-112`) | scattered | `B-DEF`: exact, case-sensitive, first match | `S-API`, `S-ORDER` | Duplicate name, absent name, leading null-slot crash/source-cited malformed behavior. |
| `input(index)` (`state_machine.hpp:48`; `state_machine.cpp:114-121`) | faithful-looking | `B-DEF`: in-range null and out-of-range `None` | `S-API` | `index == count`, `SIZE_MAX`, null slot. |
| `layer(name)` (`state_machine.hpp:49`; `state_machine.cpp:123-133`) | scattered | `B-DEF`: exact first match | `S-API`, `S-ORDER` | Duplicate names and case-only mismatch. |
| `layer(index)` (`state_machine.hpp:50`; `state_machine.cpp:135-142`) | faithful-looking | `B-DEF`: indexed optional adaptation | `S-API` | Empty machine and `SIZE_MAX`. |
| `dataBind(index)` (`state_machine.hpp:51`; `state_machine.cpp:153-160`) | scattered | `B-DEF`: polymorphic-index adaptation | `S-API`, `S-ORDER` | `index == count`; duplicate targets remain separately addressable. |
| `listener(index)` (`state_machine.hpp:52`; `state_machine.cpp:144-151`) | divergent | `B-DEF`: authored-index lookup after inert-slot retention | `S-SLOTS`, `S-API` | Malformed slot before valid listener; `SIZE_MAX`. |
| `onAddedDirty` (`state_machine.hpp:54`; `state_machine.cpp:16-41`) | missing | `B-DEF`: inputs → layers → listeners and first-error stop | `S-ORDER`, `S-SEAM` | Input 2 failure blocks all layers/listeners; null input malformed crash; invalid layer triplet. |
| `onAddedClean` (`state_machine.hpp:55`; `state_machine.cpp:43-68`) | missing | `B-DEF`: same phase order and first-error stop | `S-ORDER`, `S-SEAM` | Layer clean failure blocks later layers and all listeners. |

### File statics and private `StateMachineLayerInstance`

The accepted destination is
`crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs`,
except profiler-only `getStateName`, which may be a source-cited omission.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W2/W4 |
| --- | --- | --- | --- | --- |
| `getStateName` (`state_machine_instance.cpp:95-120`) | missing | `B-SOURCE`: profiler-only labels | `S-SEAM` | Null instance, null animation, unknown subtype → `"Blend"`. |
| `kPointerHitListenerTypes` (`state_machine_instance.cpp:127-137`) | scattered | `B-HIT`: exact nine-member classification | `S-HIT` | Drag-start-only is pointer; component-provided is not; pointer+focus remains pointer-capable. |
| `maxIterations` (`state_machine_instance.cpp:686-687`) | faithful-looking | `B-LAYER`: 101-success transition guard | `S-LAYER` | Updates 0…100 execute; 101st success stops. |
| `m_stateMachineInstance` (`state_machine_instance.cpp:688`) | scattered | `B-LAYER`: index/argument identity adaptation | `S-FIELDS` | No observable null/default owner. |
| `m_layer` (`state_machine_instance.cpp:689`) | scattered | `B-LAYER`: stable layer definition handle | `S-FIELDS` | Release-mode re-init cannot invent a second owner. |
| `m_artboardInstance` (`state_machine_instance.cpp:690`) | scattered | `B-LAYER`: explicit operation borrow | `S-FIELDS` | Null artboard remains malformed, not a default scene. |
| `m_anyStateInstance` (`state_machine_instance.cpp:692`) | faithful-looking | `B-LAYER`: Any built before Entry | `S-FIELDS`, `S-ORDER` | Aliasing with current/source and reset teardown. |
| `m_currentState` (`state_machine_instance.cpp:693`) | faithful-looking | `B-LAYER`: retained optional occurrence | `S-FIELDS`, `S-LAYER` | Null destination/current; same-state no-op. |
| `m_stateFrom` (`state_machine_instance.cpp:694`) | faithful-looking | `B-LAYER`: interrupted-transition source lifetime | `S-FIELDS` | Alias with Any/current; older source retirement. |
| `m_transition` (`state_machine_instance.cpp:696`) | scattered | `B-LAYER`: active definition handle | `S-FIELDS` | Reset preserves the pinned stale transition state. |
| `m_transitionDurationProperty` (`state_machine_instance.cpp:697`) | scattered | `B-LAYER`: occurrence-local bound duration | `S-FIELDS` | Negative, fractional, NaN/infinity/out-of-range conversion source behavior. |
| `m_animationReset` (`state_machine_instance.cpp:698`) | scattered | `B-LAYER`: release/clear timing | `S-FIELDS` | Repeated clear and interruption before completion. |
| `m_transitionCompleted` (`state_machine_instance.cpp:699`) | scattered | `B-LAYER`: end callbacks once | `S-FIELDS` | Already-complete mix with false latch. |
| `m_holdAnimationFrom` (`state_machine_instance.cpp:701`) | faithful-looking | `B-LAYER`: pause-on-exit hold flag | `S-FIELDS` | Reset/interruption with pending held animation. |
| `m_mix` (`state_machine_instance.cpp:703`) | faithful-looking | `B-LAYER`: exact clamp/NaN differential | `S-FIELDS` | Negative seconds, infinities, NaN, signed zero. |
| `m_mixFrom` (`state_machine_instance.cpp:704`) | faithful-looking | `B-LAYER`: interrupted partial-mix carry | `S-FIELDS` | Early interruption at partial mix. |
| `m_stateMachineChangedOnAdvance` (`state_machine_instance.cpp:705`) | scattered | `B-LAYER`: persist on same-frame follow-up, reset on new frame | `S-LAYER` | Multiple transitions in one layer still one changed layer. |
| `m_waitingForExit` (`state_machine_instance.cpp:707`) | faithful-looking | `B-LAYER`: waiting propagation | `S-FIELDS` | Any waits while current transition succeeds. |
| `m_holdAnimation` (`state_machine_instance.cpp:708`) | faithful-looking | `B-LAYER`: one-shot apply then clear | `S-FIELDS` | Held animation plus reset. |
| `m_holdTime` (`state_machine_instance.cpp:709-710`) | faithful-looking | `B-LAYER`: unvalidated spilled/hold time | `S-FIELDS` | NaN/infinite/negative spilled time. |
| destructor (`state_machine_instance.cpp:143-148`) | faithful-looking | `B-LAYER`, `B-SOURCE`: owner-drop order | `S-FIELDS` | Aliased Any/current/source source-cited double-delete hazard. |
| `init` (`state_machine_instance.cpp:150-175`) | faithful-looking | `B-LAYER`: per-layer RNG seed, Any bind, Entry | `S-ORDER` | Two-layer deterministic seed; null Any; release-mode re-init. |
| `resetState` (`state_machine_instance.cpp:177-192`) | faithful-looking | `B-LAYER`: exact retained stale mix/hold behavior | `S-LAYER` | `stateFrom == current`, current==Any, active transition reset. |
| `updateMix` (`state_machine_instance.cpp:194-223`) | faithful-looking | `B-LAYER`: completion/callback/FP differential | `S-ORDER` | Zero mix time; negative/NaN seconds; completion latch. |
| `advance` (`state_machine_instance.cpp:225-278`) | faithful-looking | `B-LAYER`: current → mix → source → apply → chained transitions | `S-ORDER`, `S-LAYER` | 101 transitions; null current; `newFrame=false` after change. |
| `resolvedDuration` (`state_machine_instance.cpp:283-291`) | faithful-looking | `B-LAYER`: rounded/clamped bound value | `S-FIELDS` | `-0.1`, `0.5`, `1.5`, NaN, infinity, `>UINT32_MAX`. |
| `resolvedMixTime` (`state_machine_instance.cpp:294-316`) | scattered | `B-LAYER`: percent/ms conversion | `S-FIELDS` | Blend source, null animation, infinite animation duration. |
| `isTransitioning` (`state_machine_instance.cpp:318-322`) | faithful-looking | `B-LAYER`: source+duration+mix predicate | `S-LAYER` | Null source, mix one, NaN mix. |
| `updateState` (`state_machine_instance.cpp:324-341`) | faithful-looking | `B-LAYER`: early-exit gate; Any before current | `S-ORDER` | Both sources allowed; Any waiting/current allowed. |
| `fireEvents` (`state_machine_instance.cpp:343-353`) | scattered | `B-LAYER`: occurrence filter/FIFO | `S-ORDER` | Duplicate pointer; null source-cited malformed action; mixed occurrences. |
| `performListenerActions` (`state_machine_instance.cpp:355-367`) | scattered | `B-LAYER`: matching FIFO and terminal Rust error adaptation | `S-ORDER` | Duplicate actions; interleaved start/end occurrences. |
| `canChangeState` (`state_machine_instance.cpp:369-374`) | faithful-looking | `B-LAYER`: definition-identity self guard | `S-LAYER` | Same pointer, equivalent distinct state, null/null. |
| `randomValue` (`state_machine_instance.cpp:376`) | faithful-looking | `B-LAYER`: one RNG draw iff positive total | `S-LAYER` | Inject negative, NaN, and exactly one. |
| `changeState` (`state_machine_instance.cpp:378-410`) | faithful-looking | `B-LAYER`: outgoing end → construct/binds → incoming start | `S-ORDER` | Repeated direct call, null destination, null make-instance. |
| `findRandomTransition` (`state_machine_instance.cpp:412-468`) | faithful-looking | `B-LAYER`: authored weighted scan and strict boundary | `S-ORDER` | Wrapping total; RNG 0/boundary/1/NaN; all waiting. |
| `findAllowedTransition` (`state_machine_instance.cpp:470-509`) | faithful-looking | `B-LAYER`: first allowed authored transition | `S-ORDER` | Stale self weight; denied then allowed; waiting then allowed. |
| `buildAnimationResetForTransition` (`state_machine_instance.cpp:511-517`) | scattered | `B-LAYER`: replacement factory call | `S-FIELDS` | Null source/current; null factory result. |
| `clearAnimationReset` (`state_machine_instance.cpp:519-526`) | faithful-looking | `B-LAYER`: release then null | `S-FIELDS` | Repeated clear; replacement before completion. |
| `tryChangeState` (`state_machine_instance.cpp:528-630`) | faithful-looking | `B-LAYER`: full transition/callback/retirement ordering | `S-ORDER` | Partial interruption; zero duration; invalid exit cast; Any source; null destination. |
| `apply` (`state_machine_instance.cpp:632-663`) | faithful-looking | `B-LAYER`: reset → held → outgoing → current | `S-ORDER` | Null interpolator, NaN mix, null current. |
| `stateChangedOnAdvance` (`state_machine_instance.cpp:665-668`) | scattered | `B-LAYER`: retained flag, not aggregate-only | `S-LAYER` | Query after same-frame convergence. |
| `currentState` (`state_machine_instance.cpp:670-673`) | missing | `B-LAYER`: borrowed optional definition | `S-LAYER`, `S-API` | Null current. |
| `currentAnimation` (`state_machine_instance.cpp:675-684`) | faithful-looking | `B-LAYER`: animation-only compressed view | `S-LAYER` | Blend/current null; null animation occurrence. |
| `evaluatedRandomWeight` shared scratch use (`state_machine_instance.cpp:428-456`) | divergent, approved adaptation | `B-LAYER`: equal results plus two-instance race isolation | `S-FIELDS` | Duplicate transitions, wraparound, concurrent instances. |

### Hit-component hierarchy

Rust destination for every row in this table:
`crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W2/W4 |
| --- | --- | --- | --- | --- |
| `HitComponent::m_component` (`state_machine_instance.hpp:504`) | scattered | `B-HIT`: stable optional component identity | `S-HIT`, `S-FIELDS` | Nullable component only where subclass permits it. |
| `HitComponent::m_stateMachineInstance` (`state_machine_instance.hpp:505`) | scattered | `B-HIT`: operation-borrow adaptation | `S-HIT`, `S-FIELDS` | No shallow owner back-pointer alias in snapshots. |
| `HitComponent::component` (`state_machine_instance.hpp:479`) | scattered | `B-HIT`: exact identity used by sorting | `S-HIT` | Null/custom component and duplicate target. |
| `HitComponent` constructor (`state_machine_instance.hpp:480-483`) | divergent | `B-HIT`: every concrete category constructible | `S-HIT` | Layout, shape, text, nested artboard, component list, provider target. |
| virtual destructor (`state_machine_instance.hpp:484`) | scattered | `B-SOURCE`: trait-object drop | `S-HIT`, `S-LIFE` | Wrapper never owns nested target. |
| `processEvent` (`state_machine_instance.hpp:485-489`) | divergent | `B-HIT`: tri-state dispatch | `S-HIT` | Opaque front target still sends cleanup with `canHit=false` behind it. |
| `processGamepadInvocation` (`state_machine_instance.hpp:490-492`) | scattered | `B-HIT`: nested/list broadcast aggregation | `S-HIT` | Nested opaque result; component-list early opacity. |
| `prepareEvent` (`state_machine_instance.hpp:493-495`) | divergent | `B-HIT`: separate complete prepare pass | `S-HIT` | Shared target hit-tested once; duplicate listeners all receive hover. |
| `hitTest` (`state_machine_instance.hpp:496`) | divergent | `B-HIT`: full component geometry | `S-HIT` | Singular transforms, hidden/collapsed ancestors, raw unpainted shape path. |
| base `enablePointerEvents` (`state_machine_instance.hpp:497`) | missing | `B-HIT`: base no-op and concrete walks | `S-HIT` | Nested wrapper no-op; shared group across hit owners. |
| base `disablePointerEvents` (`state_machine_instance.hpp:498`) | missing | `B-HIT`: base no-op and concrete walks | `S-HIT` | Disable pointer 1 while pointer 2 active. |
| testing `earlyOutCount` (`state_machine_instance.hpp:500`) | missing | `B-SOURCE` or test-only counter parity | `S-TOOLS` | Repeated early-out events. |
| `HitDrawable::hitRadius` (`state_machine_instance.cpp:732`) | missing | `B-SOURCE`: value 2, unused by base | `S-HIT` | Concrete shape/text hard-code radius 2. |
| `HitDrawable::isHovered` (`state_machine_instance.cpp:733`) | divergent | `B-HIT`: one transient per hit owner | `S-HIT` | Multiple pointer IDs and stale value after early-out. |
| `HitDrawable::canEarlyOut` (`state_machine_instance.cpp:734`) | divergent | `B-HIT`: aggregate monotonic flag | `S-HIT` | Enter disables; opacity disables; later mutation does not recompute. |
| `HitDrawable::needsDownListener` (`state_machine_instance.cpp:735`) | divergent | `B-HIT`: aggregate down need | `S-HIT` | Click-only requires down and up. |
| `HitDrawable::needsUpListener` (`state_machine_instance.cpp:736`) | divergent | `B-HIT`: aggregate up need | `S-HIT` | Up-only target after prior hover. |
| `HitDrawable::isOpaque` (`state_machine_instance.cpp:737`) | divergent | `B-HIT`: explicit/provider opacity | `S-HIT` | Reused layout upgrades; shape provider opacity remains discarded. |
| `HitDrawable::m_drawable` (`state_machine_instance.cpp:738`) | divergent | `B-HIT`: drawable identity/dynamic opacity | `S-HIT` | Opacity changes after construction. |
| `HitDrawable::listeners` (`state_machine_instance.cpp:739`) | divergent | `B-HIT`: ordered duplicates on shared owner | `S-HIT`, `S-ORDER` | Same group appended twice; consumed first occurrence skips later. |
| `HitDrawable` constructor (`state_machine_instance.cpp:719-731`) | divergent | `B-HIT`: target opacity disables early-out | `S-HIT` | Null malformed drawable; opacity mutation. |
| base `hitTest` (`state_machine_instance.cpp:741`) | missing | `B-SOURCE`: always false | `S-HIT` | Concrete subclass without override. |
| `prepareEvent` (`state_machine_instance.cpp:743-767`) | divergent | `B-HIT`: early-out/exit/hover broadcast | `S-HIT` | Exit avoids geometry; duplicate idempotent hover calls. |
| `processGamepadInvocation` (`state_machine_instance.cpp:769-774`) | scattered | `B-HIT`: ordinary drawable returns none | `S-HIT` | Script-aware behavior must use its owning wrapper. |
| `processEvent` (`state_machine_instance.cpp:776-818`) | divergent | `B-HIT`: unconsumed ordered groups and opacity | `S-HIT` | Hover without action still hit; scroll makes opaque; occluded target none. |
| `addListener` (`state_machine_instance.cpp:820-838`) | divergent | `B-HIT`: aggregate flags then append | `S-HIT`, `S-ORDER` | Duplicate group; click-only; enter listener. |
| `enablePointerEvents` (`state_machine_instance.cpp:840-846`) | missing | `B-HIT`: ordered duplicate-preserving group walk | `S-HIT` | One pointer enable after other pointer consumed. |
| `disablePointerEvents` (`state_machine_instance.cpp:848-854`) | missing | `B-HIT`: ordered duplicate-preserving group walk | `S-HIT` | Disable shared group through multiple hit owners. |
| `HitExpandable` constructor (`state_machine_instance.cpp:861-866`) | scattered | `B-HIT`: drawable/component may differ | `S-HIT` | Text run uses owner text drawable. |
| `HitExpandable::hitTest` (`state_machine_instance.cpp:868-871`) | divergent | `B-HIT`: component hit test flags | `S-HIT` | No-paint shape, clipped ancestor, singular transform. |
| `HitTextRun` constructor (`state_machine_instance.cpp:877-887`) | missing | `B-HIT`: owner drawable plus run hit-target flag | `S-HIT` | Null run; reused run; flag remains after listener removal. |
| `HitLayout` constructor (`state_machine_instance.cpp:893-897`) | missing | `B-HIT`: same drawable/component | `S-HIT` | Proxy/layout/null malformed target. |
| `HitLayout::hitTest` (`state_machine_instance.cpp:899-902`) | missing | `B-HIT`: layout bounds participate | `S-HIT` | Outside unclipped bounds, hidden, singular, frame origin. |
| `HitNestedArtboard` constructor (`state_machine_instance.cpp:908-911`) | missing | `B-HIT`: borrowed nested target wrapper | `S-HIT` | Wrong component subtype source-cited malformed case. |
| `HitNestedArtboard` destructor (`state_machine_instance.cpp:912`) | missing | `B-SOURCE`: empty wrapper drop | `S-HIT`, `S-LIFE` | Nested artboard remains externally owned. |
| `HitNestedArtboard::hitTest` (`state_machine_instance.cpp:914-941`) | missing | `B-HIT`: transform then authored nested-SM scan | `S-HIT` | Collapsed/paused/singular; first misses, second hits. |
| `HitNestedArtboard::processGamepadInvocation` (`state_machine_instance.cpp:942-960`) | scattered | `B-HIT`: all nested machines, return none | `S-HIT` | Child opaque ignored; multiple children; null malformed wrapper. |
| `HitNestedArtboard::processEvent` (`state_machine_instance.cpp:961-1067`) | missing | `B-HIT`: transform, supported routing, occlusion→exit | `S-HIT` | First child hit then later miss overwrite; occluded move exits; drag returns none. |
| `HitNestedArtboard::prepareEvent` (`state_machine_instance.cpp:1068-1071`) | missing | `B-SOURCE`: intentional no-op | `S-HIT` | Child hover changes only during process. |
| `HitComponentList` constructor (`state_machine_instance.cpp:1077-1080`) | missing | `B-HIT`: borrowed list wrapper | `S-HIT` | Collapsed and duplicate-index list. |
| `HitComponentList` destructor (`state_machine_instance.cpp:1081`) | missing | `B-SOURCE`: empty wrapper drop | `S-HIT`, `S-LIFE` | Items remain externally owned. |
| `HitComponentList::hitTest` (`state_machine_instance.cpp:1083-1107`) | missing | `B-HIT`: reverse ordered indices | `S-HIT`, `S-ORDER` | Duplicate indices; null top item; singular item transform. |
| `HitComponentList::processEvent` (`state_machine_instance.cpp:1108-1226`) | missing | `B-HIT`: strongest-result aggregation and cleanup | `S-HIT` | Opaque first then previously hovered second; drag; parent `canHit=false`. |
| `HitComponentList::processGamepadInvocation` (`state_machine_instance.cpp:1227-1269`) | missing | `B-HIT`: reverse broadcast, stop after opaque | `S-HIT`, `S-ORDER` | Duplicate index; collapsed list; opaque top item. |
| `HitComponentList::prepareEvent` (`state_machine_instance.cpp:1270-1273`) | missing | `B-SOURCE`: intentional no-op | `S-HIT` | No pre-process child hover mutation. |

### Listener-ViewModel helper members

Rust destination for every row:
`crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W2/W4 |
| --- | --- | --- | --- | --- |
| binding `m_parent` (`state_machine_instance.cpp:1289`) | faithful-looking | `B-LVM`: stable listener occurrence/index | `S-FIELDS` | Null parent makes dirt inert. |
| binding `m_viewModelInstanceValue` (`state_machine_instance.cpp:1290-1292`) | faithful-looking | `B-LVM`: retained cell plus weak sink adaptation | `S-FIELDS` | Two bindings same cell; property notifying during clear. |
| base binding constructor (`state_machine_instance.cpp:1401-1407`) | faithful-looking | `B-LVM`: retain then register | `S-ORDER` | Null property malformed; duplicate registration. |
| base `relinkDataBind` (`state_machine_instance.cpp:1409`) | faithful-looking | `B-SOURCE`: no-op | `S-BIND` | Base use across context replacement retains old property. |
| base destructor (`state_machine_instance.cpp:1411-1414`) | faithful-looking | `B-LVM`: clear before retained cell drop | `S-LIFE` | Already cleared; active notification. |
| base `clearDataContext` (`state_machine_instance.cpp:1416-1424`) | faithful-looking | `B-LVM`: unregister before release; idempotent | `S-ORDER` | Repeated clear. |
| base `addDirt` (`state_machine_instance.cpp:1481-1488`) | faithful-looking | `B-LVM`: every dirt enqueues, no dedup | `S-EVENT` | Repeated dirt; dirt after clear; ignored recurse/value. |
| listener binding `m_listener` (`state_machine_instance.cpp:1305`) | faithful-looking | `B-LVM`: retained definition/index adaptation | `S-FIELDS` | Null listener; synchronous dirt during base registration. |
| listener binding constructor (`state_machine_instance.cpp:1426-1432`) | faithful-looking | `B-LVM`: base registration precedes subtype field | `S-ORDER` | Null listener. |
| listener binding `relinkDataBind` (`state_machine_instance.cpp:1434-1452`) | faithful-looking | `B-LVM`: null context retains old; unresolved clears; same cell no-op | `S-BIND` | New context/same cell; missing path; null context. |
| input binding `m_listenerInput` (`state_machine_instance.cpp:1318`) | faithful-looking | `B-LVM`: authored input identity | `S-FIELDS` | Duplicate authored paths. |
| input binding constructor (`state_machine_instance.cpp:1454-1460`) | faithful-looking | `B-LVM`: independent duplicate registrations | `S-ORDER` | Null input. |
| input binding `relinkDataBind` (`state_machine_instance.cpp:1462-1479`) | faithful-looking | `B-LVM`: per-input equivalent lifecycle | `S-BIND` | Duplicate inputs same cell; unresolved replacement. |
| listener VM `m_stateMachineInstance` (`state_machine_instance.cpp:1393-1394`) | faithful-looking | `B-LVM`: stable instance operation channel | `S-FIELDS` | No mutable queue alias after clone. |
| listener VM `m_listener` (`state_machine_instance.cpp:1395`) | faithful-looking | `B-LVM`: retained immutable listener | `S-FIELDS` | Null source-cited malformed listener. |
| listener VM `m_dataContext` (`state_machine_instance.cpp:1396`) | scattered | `B-LVM`: old context retention after binding clear | `S-FIELDS` | Clear bindings then query context remains non-null. |
| listener VM `m_propertyBindings` (`state_machine_instance.cpp:1397-1398`) | faithful-looking | `B-LVM`: authored-discovery order | `S-ORDER` | Interleaved non-VM inputs and duplicate VM inputs. |
| `ListenerViewModel` constructor (`state_machine_instance.cpp:1325-1328`) | faithful-looking | `B-LVM`: owner/listener identity | `S-FIELDS` | Null listener; lifetime bounded by instance. |
| destructor (`state_machine_instance.cpp:1490`) | faithful-looking | `B-LVM`, `B-LIFE`: idempotent clear | `S-LIFE` | Direct destruction while bound. |
| `clearDataContext` (`state_machine_instance.cpp:1330`) | faithful-looking | `B-LVM`: clear bindings but retain context | `S-BIND` | Query context after clear. |
| `bindFromContext` (`state_machine_instance.cpp:1331-1373`) | faithful-looking | `B-LVM`: store context, clear, single-or-all authored paths | `S-ORDER`, `S-BIND` | Null malformed context; unresolved; duplicate VM inputs. |
| `reportToStateMachine` (`state_machine_instance.cpp:1374-1381`) | divergent | `B-LVM`: suppress trigger value zero only | `S-EVENT` | Trigger `0→1→0`; signed zero suppressed; repeated value one; duplicate bindings. |
| `listener` (`state_machine_instance.cpp:1382`) | faithful-looking | `B-LVM`: retained accessor | `S-API` | Null malformed listener. |
| `dataContext` (`state_machine_instance.cpp:1383-1391`) | scattered | `B-LVM`: borrowed getter over retained context | `S-BIND` | Bindings cleared while context retained. |

### `StateMachineInstance` fields and inline/header surface

Rust destination for every row:
`crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W1/W4 |
| --- | --- | --- | --- | --- |
| `DataBindChanged` typedef (`state_machine_instance.hpp:55`) | missing | `B-SOURCE`: tools callback adaptation | `S-TOOLS` | Nullable callback. |
| tools `InputChanged` typedef (`state_machine_instance.hpp:59`) | missing | `B-SOURCE`: tools callback adaptation | `S-TOOLS` | Nullable callback. |
| `m_DataContext` (`state_machine_instance.hpp:93`) | scattered | `B-BIND`: explicit empty initial state | `S-FIELDS`, `S-BIND` | No invented default context. |
| `m_reportedEvents` (`state_machine_instance.hpp:384`) | divergent, approved cursor adaptation | `B-EVENT`: pending queue | `S-EVENT`, `S-FIELDS` | Host drain must not consume listener delivery. |
| `m_reportingEvents` (`state_machine_instance.hpp:385`) | divergent, approved cursor adaptation | `B-EVENT`: current snapshot visibility | `S-EVENT` | Callback sees only newly chained pending reports. |
| `m_machine` (`state_machine_instance.hpp:386`) | scattered, approved index/arena adaptation | `B-CTOR`: stable supplied definition | `S-FIELDS` | No observable null/default state. |
| `m_needsAdvance` (`state_machine_instance.hpp:387`) | faithful-looking | `B-ADV`: exact latch-only accessor | `S-FIELDS` | Pending ordinary event while latch false. |
| `m_inputInstances` (`state_machine_instance.hpp:388`) | faithful-looking | `B-CTOR`, `B-ADV`: authored null slots, every-slot advanced | `S-SLOTS`, `S-FIELDS` | Unsupported index 0 then valid input; null slot malformed advance. |
| `m_layerCount` (`state_machine_instance.hpp:389`) | faithful-looking | `B-CTOR`: derive before access | `S-FIELDS` | Definition count cannot diverge from owned vector length. |
| `m_layers` (`state_machine_instance.hpp:390`) | faithful-looking | `B-CTOR`: authored owned occurrences | `S-FIELDS`, `S-ORDER` | Empty and multi-layer machine. |
| `m_hitComponents` (`state_machine_instance.hpp:391`) | divergent | `B-HIT`: complete polymorphic ordered owners | `S-HIT`, `S-FIELDS` | Nested/provider-only machine makes `hasListeners` true. |
| `m_listenerGroups` (`state_machine_instance.hpp:392`) | divergent | `B-HIT`: retained groups, including unresolved targets | `S-HIT`, `S-SLOTS` | Unresolved target group still reset each event. |
| `m_parentStateMachineInstance` (`state_machine_instance.hpp:393`) | scattered | `B-EVENT`: ID/owner-safe parent adaptation | `S-FIELDS` | Null and nested parent. |
| `m_parentNestedArtboard` (`state_machine_instance.hpp:394`) | scattered | `B-EVENT`: parent nested identity | `S-FIELDS` | Null and replaced nested host. |
| shadow `m_dataBinds` (`state_machine_instance.hpp:395`) | scattered | `B-SOURCE`: tools-only empty-shadow behavior or documented Rust tools adaptation | `S-TOOLS` | Callback installed after later keyframe binds. |
| `m_listenerViewModels` (`state_machine_instance.hpp:396`) | faithful-looking | `B-LVM`: authored raw-owner equivalent | `S-FIELDS`, `S-ORDER` | Duplicate listeners independently owned. |
| `m_reportedListenerViewModels` (`state_machine_instance.hpp:397`) | faithful-looking | `B-EVENT`: pending FIFO | `S-EVENT` | Same listener twice. |
| `m_reportingListenerViewModels` (`state_machine_instance.hpp:398`) | faithful-looking | `B-EVENT`: current snapshot | `S-EVENT` | First callback reports second; second waits next batch. |
| `m_bindablePropertyInstances` (`state_machine_instance.hpp:399-400`) | scattered | `B-CTOR`: source identity → one clone | `S-FIELDS` | Structurally equal distinct sources; duplicate target reuse. |
| `m_scriptedObjectsMap` (`state_machine_instance.hpp:401-402`) | scattered, approved lifecycle adaptation | `B-CTOR`, `B-LIFE`: occurrence identity/order rules | `S-FIELDS`, `S-LIFE` | Duplicate source pointer and equivalent different pointer. |
| `m_bindableDataBindsToTarget` (`state_machine_instance.hpp:403-404`) | scattered | `B-KEY`: last map entry while occurrences retained | `S-FIELDS` | Duplicate ToTarget/TwoWay. |
| `m_bindableDataBindsToSource` (`state_machine_instance.hpp:405-406`) | scattered | `B-KEY`: last ToSource map entry | `S-FIELDS` | Duplicate ToSource. |
| `m_transitionPropertyInstances` (`state_machine_instance.hpp:410-412`) | scattered | `B-LAYER`: occurrence-local transition values | `S-FIELDS` | Duplicate key overwrites lookup without rewriting earlier bind target. |
| `m_stateKeyFrameDataBinds` (`state_machine_instance.hpp:417-418`) | scattered | `B-KEY`: state-keyed build/removal tracking | `S-KEY`, `S-FIELDS` | Build twice; remove unknown; teardown with active binds. |
| `m_drawOrderChangeCounter` (`state_machine_instance.hpp:419`) | missing | `B-HIT`: constructor sort and change-triggered resort | `S-HIT`, `S-FIELDS` | Nonzero initial counter; wrap/change; unmatched custom hit. |
| `m_focusManager` (`state_machine_instance.hpp:425`) | scattered | `B-FOCUS`: owned internal domain | `S-FIELDS`, `S-SEAM` | No nodes and external replacement. |
| `m_externalFocusManager` (`state_machine_instance.hpp:426`) | scattered | `B-FOCUS`: identity/fallback adaptation | `S-SEAM` | Same pointer with different desired parent is no-op. |
| `m_focusListenerGroups` (`state_machine_instance.hpp:427`) | scattered | `B-FOCUS`: authored registration order | `S-ORDER` | Duplicate focus callbacks. |
| `m_keyboardListenerGroups` (`state_machine_instance.hpp:428-429`) | scattered | `B-CTOR`: authored construction | `S-ORDER`, `S-SEAM` | Listener flags plus scripted wants flags. |
| `m_gamepadListenerGroups` (`state_machine_instance.hpp:430`) | scattered | `B-CTOR`: authored construction | `S-ORDER`, `S-SEAM` | Null/wrong target remains pinned malformed behavior. |
| `m_gamepadScriptedDrawables` (`state_machine_instance.hpp:437`) | scattered | `B-CTOR`: non-owning authored facility list | `S-FIELDS` | Focused drawable excluded from later broadcast. |
| `m_embedderGamepads` (`state_machine_instance.hpp:439`) | missing/out-of-scope definition | `B-SOURCE`: ownership boundary only | `S-SEAM` | Buffer parsing/mutation cases belong to `gamepad_batch.cpp`. |
| `m_semanticManager` (`state_machine_instance.hpp:442`) | missing dependency | `B-FOCUS`: queue/orchestration boundary | `S-SEAM` | Internal manager absent/present. |
| `m_externalSemanticManager` (`state_machine_instance.hpp:443`) | missing dependency | `B-FOCUS`: selected-manager boundary | `S-SEAM` | External→null with/without internal manager. |
| `QueuedFocusEvent::group` (`state_machine_instance.hpp:448`) | faithful-looking value adaptation | `B-FOCUS`: explicit non-default construction | `S-FIELDS` | Null malformed group. |
| `QueuedFocusEvent::isFocus` (`state_machine_instance.hpp:449`) | faithful-looking | `B-FOCUS`: exact direction | `S-FIELDS` | Focus and blur duplicates. |
| `m_queuedFocusEvents` (`state_machine_instance.hpp:451`) | faithful-looking | `B-FOCUS`: FIFO moved batch | `S-EVENT` | Callback queues another focus change. |
| `m_semanticListenerGroups` (`state_machine_instance.hpp:455-456`) | divergent dependency shape | `B-FOCUS`: authored queue producers | `S-SEAM`, `S-ORDER` | Null owner and duplicate action. |
| `QueuedSemanticEvent::group` (`state_machine_instance.hpp:459`) | faithful-looking value adaptation | `B-FOCUS`: explicit construction | `S-FIELDS` | Null group is skipped during processing. |
| `QueuedSemanticEvent::actionType` (`state_machine_instance.hpp:460`) | faithful-looking | `B-FOCUS`: exact enum payload | `S-FIELDS` | Tap/increase/decrease and invalid cast no-op at manager seam. |
| `m_queuedSemanticEvents` (`state_machine_instance.hpp:462`) | faithful-looking | `B-FOCUS`: FIFO moved batch | `S-EVENT` | Null, valid, null-listener, valid order. |
| tools `m_inputChangedCallback` (`state_machine_instance.hpp:472`) | missing | `B-SOURCE`: nullable tools callback | `S-TOOLS` | Replacement and clear. |
| `FocusState::hasFocus` (`state_machine_instance.hpp:335`) | missing | `B-FOCUS`: host snapshot | `S-API` | Focused scope without Focusable gives true. |
| `FocusState::expectsKeyboardInput` (`state_machine_instance.hpp:336`) | missing | `B-FOCUS`: Focusable capability | `S-API` | Focused non-keyboard node gives false. |

### `StateMachineInstance` methods and file-local keyframe helpers

Rust destination for every row:
`crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`, except
the three stale private transition declarations, which must not acquire an
instance-level Rust implementation.

| C++ member and cite | Current status | Required behavioral proof | Required structural proof | Adversarial cases from W1–W4 |
| --- | --- | --- | --- | --- |
| constructor (`state_machine_instance.hpp:105-106`; `.cpp:1707-2128`) | divergent | `B-CTOR` | `S-OWNER`, `S-ORDER`, `S-FIELDS` | Null source/artboard; null input; duplicate transition bind; entry action before ordinary binds/listeners; partial failure after nested registration. |
| deleted copy constructor (`state_machine_instance.hpp:107`) | divergent, approved snapshot adaptation | `B-LIFE`: snapshot versus cold remount | `S-LIFE`, `S-API` | Queues/pointer state copied by value; script tables cold; trigger IDs regenerated; no alias. |
| destructor (`state_machine_instance.hpp:108`; `.cpp:2141-2199`) | divergent | `B-LIFE`: observable teardown order | `S-LIFE` | Internal focused cleanup queues but does not run blur actions; external trees untouched; destruction without dispose is prevented/adapted. |
| `updateListeners` (`state_machine_instance.hpp:75-78`; `.cpp:1494-1545`) | divergent | `B-HIT` | `S-HIT`, `S-ADV` | NaN frame origin; shared front/back group; opaque front sends back exit; unknown pointer exit allocates/releases state. |
| `getNamedInput` (`state_machine_instance.hpp:80-81`; `.cpp:2689-2701`) | scattered | `B-DEF`: typed first match | `S-SLOTS`, `S-API` | Null slot before match; same name different types. |
| `notifyEventListeners` (`state_machine_instance.hpp:82-83`; `.cpp:3062-3171`) | scattered | `B-EVENT`: local → bubble → audio seam | `S-EVENT`, `S-SEAM` | `[A,A]` for single/multi; target mismatch; null malformed event; nested audio through two ancestors. |
| `sortHitComponents` (`state_machine_instance.hpp:84`; `.cpp:2255-2304`) | missing | `B-HIT`: exact swap-derived order | `S-HIT` | Multiple Artboard targets, duplicate drawable, unmatched custom component; not stable-sort semantics. |
| stale private `randomValue` declaration (`state_machine_instance.hpp:85`) | scattered to layer | `B-SOURCE`: no definition exists | `S-LAYER` | Must not alias layer method on instance. |
| stale private `findRandomTransition` declaration (`state_machine_instance.hpp:86-88`) | scattered to layer | `B-SOURCE`: no definition exists | `S-LAYER` | No instance-level symbol unless explicit unreachable ABI stub is required. |
| stale private `findAllowedTransition` declaration (`state_machine_instance.hpp:89-91`) | scattered to layer | `B-SOURCE`: no definition exists | `S-LAYER` | Same. |
| `completeViewModelInstances` (`state_machine_instance.hpp:97`; `.cpp:2792-2829`) | missing | `B-BIND`: main then globals; preserve occupied cross-model slot | `S-BIND`, `S-ORDER` | Missing file no-op; null default skipped; cross-VM override remains. |
| `addToHitLookup` (`state_machine_instance.hpp:98-102`; `.cpp:1619-1705`) | divergent | `B-HIT`: type branches, dedup, recursion, opacity | `S-HIT`, `S-ORDER` | Reused layout opacity upgrade; shape opacity discard; mixed/deep container; duplicate target; unsupported target. |
| `markNeedsAdvance` (`state_machine_instance.hpp:110`; `.cpp:2667`) | scattered | `B-ADV`: set-only latch | `S-ADV` | Mark then `newFrame=false` remains true. |
| `advance(seconds,newFrame)` (`state_machine_instance.hpp:113`; `.cpp:2546-2585`) | divergent | `B-ADV` | `S-ADV`, `S-EVENT`, `S-HIT` | Draw-order change; chained focus lost-latch edge; `-0`; NaN; infinities; trigger clear; reports created during layer advance. |
| inline `advance(seconds)` (`state_machine_instance.hpp:115`) | scattered | `B-ADV`: delegates with `newFrame=true` | `S-ADV`, `S-API` | Equivalent to explicit true. |
| `needsAdvance` (`state_machine_instance.hpp:118`; `.cpp:2668`) | faithful-looking | `B-ADV`: latch only | `S-ADV` | Event pending while false; focus queue semantics. |
| `resetState` (`state_machine_instance.hpp:120`; `.cpp:2670-2676`) | faithful-looking | `B-LAYER`: authored layers only | `S-LAYER` | Active transition; queues/context/input unchanged. |
| `stateMachine` (`state_machine_instance.hpp:123`) | scattered, approved index/arena adaptation | `B-API`: stable `state_machine_index`/arena | `S-API`, `S-FIELDS` | Snapshot/remount retains resolvable definition. |
| `inputCount` (`state_machine_instance.hpp:125`) | faithful-looking | `B-DEF`: slot count | `S-SLOTS` | Null slot counts. |
| `input(index)` (`state_machine_instance.hpp:126`; `.cpp:2680-2687`) | faithful-looking | `B-DEF`: optional indexed occurrence | `S-API` | In-range null vs out-of-range. |
| `getBool` (`state_machine_instance.hpp:127`; `.cpp:2703-2706`) | scattered | `B-DEF`: typed first-name lookup | `S-API` | Number then bool same name; null hole. |
| `getNumber` (`state_machine_instance.hpp:128`; `.cpp:2707-2710`) | scattered | `B-DEF`: typed first-name lookup | `S-API` | Duplicate number names. |
| `getTrigger` (`state_machine_instance.hpp:129`; `.cpp:2711-2714`) | scattered | `B-DEF`: typed first-name lookup | `S-API` | Null hole before trigger. |
| `bindViewModelInstance` (`state_machine_instance.hpp:130-131`; `.cpp:2831-2842`) | divergent | `B-BIND`: distinct null/non-null branches | `S-BIND` | Null clears machine context/listeners and artboard unbind only; does not explicitly unbind machine binds. |
| `setViewModelInstance` (`state_machine_instance.hpp:135`; `.cpp:2716-2733`) | missing | `B-BIND`: null no-op; stage without bind | `S-BIND` | Replace main then inspect stale paths before explicit bind. |
| `setGlobalViewModelInstance` (`state_machine_instance.hpp:140-141`; `.cpp:2735-2774`) | missing | `B-BIND`: validation and named-slot replacement | `S-BIND`, `S-ORDER` | Null/file/name/non-global failures; put type B in slot A; preserve other slot order. |
| `bind` (`state_machine_instance.hpp:144`; `.cpp:2776-2790`) | scattered | `B-BIND`: complete → artboard → machine | `S-BIND` | No context no-op; shared context with missing defaults and sibling. |
| `globalViewModelInstance` (`state_machine_instance.hpp:147`; `.cpp:2844-2859`) | missing | `B-BIND`: pure slot read without global validation | `S-BIND`, `S-API` | Unknown/non-global name and unusual slot keys. |
| `bindDataContext` (`state_machine_instance.hpp:148`; `.cpp:2861-2868`) | divergent | `B-BIND`: clear/register/clear artboard/bind artboard/bind machine | `S-BIND` | Null must not become safe clear; incomplete context remains incomplete. |
| `inheritDataContext` (`state_machine_instance.hpp:149`; `.cpp:2870-2878`) | scattered | `B-BIND`: null no-op, no prior clear, machine only | `S-BIND` | Inherit A then B leaves stale registration on A. |
| `dataContext(setter)` (`state_machine_instance.hpp:150`; `.cpp:2880-2884`) | scattered | `B-BIND`: clear then internal apply, machine only | `S-BIND` | Null with/without VM listeners. |
| `dataContext(getter)` (`state_machine_instance.hpp:151`) | missing | `B-BIND`: retained optional getter | `S-BIND`, `S-API` | After clear. |
| `rebind` (`state_machine_instance.hpp:152`; `.cpp:2916-2921`) | scattered | `B-BIND`: artboard clear/apply then machine apply | `S-BIND` | Rebind after clear with VM listener. |
| `currentAnimationCount` (`state_machine_instance.hpp:154`; `.cpp:2985-2996`) | faithful-looking | `B-LAYER`: per-layer count | `S-LAYER`, `S-API` | Two layers same source animation count twice. |
| `currentAnimationByIndex` (`state_machine_instance.hpp:155`; `.cpp:2998-3014`) | faithful-looking | `B-LAYER`: compact authored order | `S-LAYER`, `S-API` | Interleaved non-animation layers. |
| `stateChangedCount` (`state_machine_instance.hpp:159`; `.cpp:2955-2966`) | faithful-looking result, divergent cache shape | `B-LAYER`: scan retained flags; derived cache allowed | `S-LAYER` | Several transitions in one layer count one. |
| `stateChangedByIndex` (`state_machine_instance.hpp:164`; `.cpp:2968-2983`) | missing | `B-LAYER`: compact authored changed-layer order | `S-LAYER`, `S-API` | Layers 0 and 2 changed; index 1 returns layer 2 current state; out of range null. |
| `advanceAndApply(seconds)` (`state_machine_instance.hpp:166`; `.cpp:2601-2604`) | scattered | `B-ADV`: exact delegate with VM=true | `S-ADV`, `S-OWNER` | Byte-equivalent to bool overload true. |
| `advanceAndApply(seconds,advanceViewModels)` (`state_machine_instance.hpp:171`; `.cpp:2606-2665`) | faithful-looking behavior, scattered owner | `B-ADV`: consolidated owner | `S-ADV`, `S-OWNER` | Idle `-0`; hidden focus; sixth dirt pass; VM=false; infinite nested ping-pong. |
| `advancedDataContext` (`state_machine_instance.hpp:172`; `.cpp:2587-2593`) | scattered | `B-ADV`: each settlement iteration if bound | `S-ADV` | Five dirty passes produce five advances. |
| `reset` (`state_machine_instance.hpp:173`; `.cpp:2595-2599`) | scattered | `B-ADV`: VM advanced before artboard reset | `S-ADV`, `S-ORDER` | Resettable observes post-consumed trigger. |
| `name` (`state_machine_instance.hpp:174`; `.cpp:2678`) | scattered | `B-API`: source machine name | `S-API` | Null source remains malformed, not empty string. |
| `pointerMove` (`state_machine_instance.hpp:175-177`; `.cpp:1568-1573`) | divergent | `B-HIT`: C++ path forwards coordinates/timestamp | `S-HIT`, `S-ADV` | Multiple IDs; negative/NaN timestamp and nonfinite coordinates. |
| `pointerDown` (`state_machine_instance.hpp:178`; `.cpp:1574-1577`) | divergent | `B-HIT`: timestamp zero through hit owners | `S-HIT` | Down outside after hover; duplicate click listeners. |
| `pointerUp` (`state_machine_instance.hpp:179`; `.cpp:1578-1581`) | divergent | `B-HIT`: click/up phase | `S-HIT` | Up without down; different pointer IDs; click plus up. |
| `pointerExit` (`state_machine_instance.hpp:180`; `.cpp:1582-1585`) | divergent | `B-HIT`: process then release pointer state | `S-HIT` | Exit during drag; repeated exit. |
| `dragStart` (`state_machine_instance.hpp:181-184`; `.cpp:1586-1597`) | divergent | `B-HIT`: optional disable before timestamp-zero event | `S-HIT` | External default versus internal disable=false; nested base no-op. |
| `dragEnd` (`state_machine_instance.hpp:185`; `.cpp:1598-1606`) | faithful-looking tail, surrounding divergence | `B-HIT`: enable → dragEnd(0) → move(timestamp) | `S-HIT`, `S-ORDER` | Different drag/move target; move opaque but return drag result. |
| `tryChangeState` (`state_machine_instance.hpp:187`; `.cpp:2306-2318`) | faithful-looking | `B-ADV`: bind update then every layer | `S-ADV`, `S-ORDER` | Two layers become eligible and both transition. |
| `hitTest` (`state_machine_instance.hpp:188`; `.cpp:1547-1566`) | divergent | `B-HIT`: first geometric hit in sorted hit list | `S-HIT` | Occluded geometric target still true; hidden raw-path edge; nested/list target; NaN forwarded. |
| `durationSeconds` (`state_machine_instance.hpp:190`) | missing | `B-SOURCE`: constant `-1` | `S-API` | Exact constant. |
| `loop` (`state_machine_instance.hpp:191`) | missing | `B-SOURCE`: `Loop::oneShot` | `S-API` | Exact enum. |
| `isTranslucent` (`state_machine_instance.hpp:192`) | missing | `B-SOURCE`: constant true | `S-API` | Exact constant. |
| `artboard` (`state_machine_instance.hpp:196`) | scattered | `B-API`: explicit-borrow adaptation | `S-OWNER`, `S-API` | Correct backing instance on every call. |
| `setParentStateMachineInstance` (`state_machine_instance.hpp:198-201`) | missing | `B-EVENT`: owner-safe parent identity | `S-FIELDS` | Set, replace, clear. |
| `parentStateMachineInstance` (`state_machine_instance.hpp:202-205`) | missing | `B-EVENT`: optional getter/adaptation | `S-API` | Null and nested parent. |
| `setParentNestedArtboard` (`state_machine_instance.hpp:207-210`) | scattered | `B-EVENT`: local-ID/handle adaptation | `S-FIELDS` | Set, replace, clear. |
| `parentNestedArtboard` (`state_machine_instance.hpp:211`) | missing | `B-EVENT`: optional getter/adaptation | `S-API` | Null and nested parent. |
| `notify` (`state_machine_instance.hpp:212-213`; `.cpp:3041-3046`) | scattered | `B-EVENT`: immediate nested dispatch then bind update | `S-EVENT`, `S-ORDER` | Nested action dirties bind; bubbling precedes final local bind update. |
| `notifyListenerViewModels` (`state_machine_instance.hpp:214-215`; `.cpp:3048-3060`) | faithful-looking | `B-EVENT`: snapshot FIFO/duplicates | `S-EVENT` | First reports second; null malformed pointer; terminal Rust error documented. |
| `reportEvent` (`state_machine_instance.hpp:219`; `.cpp:3016-3019`) | scattered | `B-EVENT`: exact FIFO report append | `S-EVENT` | Duplicate report; null malformed event; negative/NaN/infinite/`-0` delay. |
| `applyEvents` (`state_machine_instance.hpp:221`; `.cpp:2320-2344`) | faithful-looking | `B-EVENT` | `S-EVENT` | Event chains event+VM; exactly 100 finite batches; 101 chain; callback count query. |
| `reportListenerViewModel` (`state_machine_instance.hpp:223`; `.cpp:3021-3025`) | faithful-looking | `B-EVENT`: borrowed/indexed FIFO append | `S-EVENT` | Same listener twice; null malformed. |
| `reportedEventCount` (`state_machine_instance.hpp:226`; `.cpp:3027-3030`) | faithful-looking | `B-EVENT`: pending-only visibility | `S-EVENT`, `S-API` | Inside callback after chaining one event. |
| `reportedEventAt` (`state_machine_instance.hpp:229`; `.cpp:3032-3039`) | divergent API adaptation | `B-EVENT`: live projection and out-of-range adaptation | `S-EVENT`, `S-API` | Index==count; C++ null/+0 sentinel; Rust `None`; mutable payload refresh. |
| `playsAudio` (`state_machine_instance.hpp:230`) | missing | `B-SOURCE`: constant true | `S-EVENT`, `S-SEAM`, `S-API` | Audio play call stays recorded under `audio_event.cpp`. |
| `bindablePropertyInstance` (`state_machine_instance.hpp:231-232`; `.cpp:3189-3199`) | scattered | `B-KEY`: exact source identity → typed clone | `S-FIELDS` | Equivalent different address/global ID. |
| `bindableDataBindToSource` (`state_machine_instance.hpp:233-234`; `.cpp:3201-3210`) | scattered | `B-KEY`: last duplicate source bind | `S-FIELDS` | Two ToSource binds. |
| `bindableDataBindToTarget` (`state_machine_instance.hpp:235-236`; `.cpp:3212-3221`) | scattered | `B-KEY`: last target bind | `S-FIELDS` | ToTarget and TwoWay duplicates. |
| `findTransitionPropertyInstance` (`state_machine_instance.hpp:241-243`; `.cpp:3223-3237`) | scattered | `B-LAYER`: two-key occurrence lookup adaptation | `S-FIELDS` | Missing outer/inner key; duplicate duration bind. |
| file-local `keyFrameHolderPropertyKey` (`.cpp:3239-3256`) | scattered | `B-KEY`: number/color/bool/string only | `S-KEY` | ID/uint/custom returns zero/unbound. |
| file-local `makeKeyFrameValueHolder` (`.cpp:3258-3274`) | scattered | `B-KEY`: exact holder type | `S-KEY` | Four supported types; unsupported null. |
| `buildStateKeyFrameBinds` (`state_machine_instance.hpp:251`; `.cpp:3276-3374`) | scattered | `B-KEY` | `S-KEY`, `S-ORDER` | Duplicate source bind first wins; unsupported/null keyframe; bound context; build twice; observable initialize/converter order. |
| `removeStateKeyFrameBinds` (`state_machine_instance.hpp:255`; `.cpp:3376-3390`) | scattered | `B-KEY`: remove/delete in build order then erase | `S-KEY`, `S-LIFE` | Unknown state; removal during update callback; destructor with active binds. |
| `hasListeners` (`state_machine_instance.hpp:257`) | divergent | `B-HIT`: hit-owner nonempty meaning | `S-HIT`, `S-API` | Nested/component-list hit proxy with no authored pointer listener. |
| `hasFocusNodes` (`state_machine_instance.hpp:258`; `.cpp:3392-3397`) | faithful-looking | `B-FOCUS`: selected manager result | `S-API` | Manager exists but no nodes. |
| `focusNext` (`state_machine_instance.hpp:259`; `.cpp:3399-3404`) | faithful-looking | `B-FOCUS`: delegate and defer callbacks | `S-API`, `S-SEAM` | Hidden current target dropped first. |
| `focusPrevious` (`state_machine_instance.hpp:260`; `.cpp:3406-3411`) | faithful-looking | `B-FOCUS`: delegate and defer callbacks | `S-API`, `S-SEAM` | No primary focus with several roots. |
| `clearFocus` (`state_machine_instance.hpp:261`; `.cpp:3413-3418`) | faithful-looking | `B-FOCUS`: focus clears before callback | `S-API`, `S-SEAM` | Call twice; only first blur. |
| `clearDataContext` (`state_machine_instance.hpp:262`; `.cpp:2923-2934`) | scattered | `B-BIND`: unregister/null then clear listener cells only | `S-BIND`, `S-ORDER` | State-machine binds/artboard/scripts retain their pinned state. |
| `relinkDataContext` (`state_machine_instance.hpp:263`; `.cpp:2936-2939`) | scattered | `B-BIND`: artboard-only delegation | `S-BIND` | Nested VM reference used only by state-machine listener remains unaffected here. |
| `rebuildDataBind` (`state_machine_instance.hpp:264`; `.cpp:2941-2947`) | scattered | `B-BIND`: context-bind subtype only | `S-BIND` | Plain bind ignored; null malformed; cleared context forwarded. |
| `internalDataContext` (`state_machine_instance.hpp:265`; `.cpp:2901-2914`) | scattered | `B-BIND`: assign → binds → listener cells → script contexts → init/hydrate | `S-BIND`, `S-ORDER` | Null with VM listeners; script mutates context; multiple script visits. |
| `scriptedObject` (`state_machine_instance.hpp:266`; `.cpp:2130-2139`) | scattered | `B-CTOR`: exact source/global identity adaptation | `S-FIELDS`, `S-API` | Equivalent different source returns none. |
| `queueFocusEvent` (`state_machine_instance.hpp:269`; `.cpp:2409-2414`) | faithful-looking | `B-FOCUS`: FIFO append and mark | `S-EVENT` | Null malformed group; duplicates. |
| `queueSemanticEvent` (`state_machine_instance.hpp:272-273`; `.cpp:2475-2480`) | faithful-looking | `B-FOCUS`: FIFO append and mark | `S-EVENT` | Duplicate same action. |
| `fireSemanticAction` (`state_machine_instance.hpp:276-277`; `.cpp:2509-2544`) | missing dependency | `B-FOCUS`: dispatch orchestration to recorded lookup seam | `S-SEAM` | Missing manager/id/data; invalid enum; nested owner. |
| mutable `focusManager` (`state_machine_instance.hpp:281-285`) | scattered | `B-FOCUS`: external else internal selection | `S-SEAM` | Null external falls back. |
| const `focusManager` (`state_machine_instance.hpp:289-293`) | scattered | `B-FOCUS`: same selection | `S-SEAM` | Same as mutable. |
| `hasExternalFocusManager` (`state_machine_instance.hpp:296-299`) | missing | `B-FOCUS`: identity query/adaptation | `S-API`, `S-SEAM` | Install, replace, clear. |
| `internalFocusManager` (`state_machine_instance.hpp:304`) | missing | `B-FOCUS`: owned-manager access/adaptation | `S-API`, `S-SEAM` | Ignores selected external manager. |
| `submitGamepadsFromBuffer` (`state_machine_instance.hpp:310`; `gamepad_batch.cpp:165-296`) | missing, out of FL-C5 source definition | `B-SOURCE`: declaration/seam only | `S-SEAM` | Null/version/truncation/rollback/NaN cases remain owning row’s proof. |
| `broadcastGamepadToScriptedDrawables` (`state_machine_instance.hpp:317-319`; `gamepad_batch.cpp:298-362`) | faithful-looking, out-of-scope definition | `B-SOURCE`: declaration and caller boundary | `S-SEAM` | Nested before local; skip focused; direct script hit nonopaque. |
| `setExternalFocusManager` (`state_machine_instance.hpp:325`; `.cpp:2346-2368`) | divergent dependency shape | `B-FOCUS`: clean old → assign → rebuild | `S-SEAM`, `S-ORDER` | Focused switch queues blur; identical pointer no-op despite parent change. |
| `setFocus` (`state_machine_instance.hpp:328`; `.cpp:2416-2428`) | missing | `B-FOCUS`: node or clear | `S-API`, `S-SEAM` | FocusData with null node behaves as clear. |
| `focusState` (`state_machine_instance.hpp:343`; `.cpp:2430-2447`) | missing | `B-FOCUS`: `{hasFocus, expectsKeyboardInput}` | `S-API`, `S-SEAM` | Focused node without Focusable; accepting/nonaccepting Focusable. |
| `semanticManager` (`state_machine_instance.hpp:348-352`) | missing dependency | `B-FOCUS`: selected-manager boundary only | `S-SEAM` | External, internal, neither. |
| `enableSemantics` (`state_machine_instance.hpp:357`; `.cpp:2370-2381`) | missing dependency | `B-FOCUS`: idempotent orchestration to seam | `S-SEAM` | External already set; null artboard. |
| `setExternalSemanticManager` (`state_machine_instance.hpp:364-365`; `.cpp:2383-2407`) | missing dependency | `B-FOCUS`: clean/assign/rebuild orchestration to seam | `S-SEAM`, `S-ORDER` | Same manager/different parent no-op; external→null with/without internal. |
| testing `hitComponentsCount` (`state_machine_instance.hpp:368`) | missing | `B-HIT`: list length | `S-TOOLS`, `S-HIT` | Provider/nested/list-only hits count. |
| testing `hitComponent` (`state_machine_instance.hpp:369-376`) | missing | `B-HIT`: indexed optional projection | `S-TOOLS`, `S-HIT` | Index==count. |
| testing `layerState` (`state_machine_instance.hpp:377`; `.cpp:1609-1616`) | missing | `B-LAYER`: machine-count bound then current state | `S-TOOLS`, `S-LAYER` | Definition count disagrees with occurrence length; out of range. |
| `enablePointerEvents` (`state_machine_instance.hpp:379`; `.cpp:3173-3179`) | missing | `B-HIT`: current sorted hit walk | `S-HIT` | Negative pointer ID; duplicates. |
| `disablePointerEvents` (`state_machine_instance.hpp:380`; `.cpp:3181-3187`) | missing | `B-HIT`: current sorted hit walk | `S-HIT` | Disable twice then enable once. |
| `dispose` (`state_machine_instance.hpp:381`; `.cpp:2201-2206`) | missing | `B-LIFE`: explicit nested detach, repeatable | `S-LIFE` | Call twice then child emits. |
| `removeEventListeners` (`state_machine_instance.hpp:421`; `.cpp:2208-2243`) | scattered ownership adaptation | `B-LIFE`: current nested traversal and all-duplicate removal | `S-LIFE` | Child removed/replaced before disposal; null elements skipped. |
| `initScriptedObjects` (`state_machine_instance.hpp:422`; `.cpp:2886-2899`) | divergent, approved facade-timing adaptation | `B-CTOR`, `B-BIND`: initialization/hydration phase equivalence | `S-LIFE`, `S-BIND` | Two observable scripts; hydration failure does not abort later ordinary C++ work; terminal resource fence remains documented. |
| `processFocusEvents` (`state_machine_instance.hpp:452`; `.cpp:2449-2473`) | faithful-looking | `B-FOCUS`: moved one-batch FIFO | `S-EVENT` | Callback changes focus; chained event waits next frame. |
| `processSemanticEvents` (`state_machine_instance.hpp:463`; `.cpp:2482-2507`) | faithful-looking | `B-FOCUS`: moved one-batch FIFO with null skips | `S-EVENT` | Null/valid/null-listener/valid. |
| tools `onInputChanged` (`state_machine_instance.hpp:467-470`) | missing | `B-SOURCE`: replace nullable callback | `S-TOOLS` | Set, replace, clear. |
| tools `onDataBindChanged` (`state_machine_instance.hpp:471`; `.cpp:2245-2253`) | missing | `B-SOURCE`: current shadow-vector behavior or documented tools adaptation | `S-TOOLS` | Later keyframe bind does not inherit callback; null clears. |

## Twelve required adversarial publication rows

- [ ] **1. Definition import and collection ownership.** Prove all five
  definition collections; missing artboard/state-machine importers; null-object
  input holes; input → layer → listener dirty/clean ordering; first-error stop
  without rollback; exact counts and index/name lookup; duplicate names,
  duplicate pointers, case mismatch, `index == count`, and `SIZE_MAX`.
- [ ] **2. Occurrence construction order.** Prove inputs and tools indices
  precede layers; Any/Entry keyframe binds and Entry callbacks can run before
  ordinary machine binds/listeners; bindable reuse and duplicate transition
  property overwrite; event/VM exclusive listener paths; focus/keyboard/
  semantic/pointer/gamepad availability; provider groups, nested registrations,
  component lists, TextInput, scripted clones/facilities, hit sort, then focus
  tree. Include null machine/artboard/input/gamepad target and partial failure
  after a nested registration.
- [ ] **3. Ordered duplicates and nullable slots.** Prove inert malformed
  listeners retain indices; null/unsupported inputs retain slots; duplicate
  listener groups, actions, notifications, bind targets, scripted source
  pointers, provider targets, component-list indices, and nested notifier
  registrations remain observable in their pinned order. No `filter_map`,
  set, or map may replace an authored occurrence vector.
- [ ] **4. Transition search and state change.** Prove Any before current,
  first-match nonrandom selection, weighted authored order, wrapping weight
  sum, strict cumulative boundary, RNG 0/exact-boundary/1/NaN, waiting-for-exit,
  early interruption, spilled time, zero-duration callback pairing, 101-success
  guard, held animation/reset ordering, per-layer changed flags, compressed
  changed-state/current-animation access, and state reset during a transition.
- [ ] **5. Hit listener and focus ownership.** Prove shared-target dedup and
  duplicate listener append; reset → prepare → process; opacity propagation
  without skipping cleanup; Artboard-first/draw-chain sorting and counter
  re-sort; shape/layout/text geometry; nested authored routing; component-list
  reverse routing and opaque→exit cleanup; drag disable/enable; provider opacity
  upgrade/discard rules; frame-origin transforms; focused-manager switch order;
  and provider/nested/list-only `hasListeners`.
- [ ] **6. DataContext bind, rebind, and clear.** Prove all distinct null
  branches; staged main/global setters; completion order and cross-model global
  occupancy; complete → artboard → machine bind; bind-null’s limited unbind;
  `bindDataContext(nullptr)` failure; inherited A→B prior-registration hazard;
  setter/getter, artboard-only relink, subtype-only rebuild, listener-cell
  clear/relink, scripted context pass order, and destructor unbind order.
- [ ] **7. Event application and chained reports.** Prove both pending queues
  are snapshotted/cleared before callbacks; events precede VM reports; callback
  inspection sees only newly pending reports; exactly 100 batches run and the
  boundary warning semantics are retained; batch 101 remains pending; single
  listener breaks after its first match while multi-input listeners continue
  across events; local dispatch precedes bubbling and the recorded audio seam;
  host draining is isolated; trigger zero and signed zero are suppressed.
  Includes closing recorded gap `flc5-vm-listener-firing-boundary`: the live
  NESTED-relative claimed-path differential demonstrated Rust applies a
  ViewModel-listener change one advance earlier than pinned C++
  (queue-on-one-advance, apply-at-next-new-frame `applyEvents`); the three
  flat claimed-path probes already hold strict per-step equality. WP6 must
  restore the C++ boundary and flip the nested probe's explicit divergence
  pin to strict per-step equality.
- [ ] **8. Zero-second and floating-point edges.** On every
  C++-corresponding path forward `+0`, `-0`, NaN, positive/negative infinity,
  and negative ordinary values without Rust finite validation. Cover advance
  seconds, pointer positions/timestamps, event delays, transition mix/duration,
  animation duration, spilled time, frame origin, and singular transforms.
  Keep validation only on separately named Rust convenience entry points.
- [ ] **9. Advance return and pending work.** Prove raw new-frame order is
  draw-sort check → focus batch → semantic batch → apply events → clear latch →
  pre-layer binds → authored layers → converter advance → every input
  `advanced`; same-frame calls retain state-change flags; reports created after
  application wait for the next new frame; raw/facade return terms differ as
  pinned; both signed zeros force facade keep-going; no clean-zero fast path;
  and every one of five settlement passes probes transitions unconditionally.
- [ ] **10. Keyframe DataBind lifecycle.** Prove first source bind per keyframe
  target, supported number/color/bool/string holders, traversal order,
  holder-before-clone sequence, initialize before converter installation,
  enrollment and live resolution, already-bound-context immediate binding,
  converter advancement, duplicate build behavior, remove-in-build-order,
  removal during processing hazard, and destructor tracking cleanup.
- [ ] **11. Clone, remount, and teardown isolation.** Distinguish the approved
  Rust snapshot from a cold remount. Prove immutable definitions may share but
  mutable layers, random scratch, trigger IDs, hit/group pointer state, event
  and notification queues, registrations, script tables, bind occurrences,
  contexts, and callback sinks do not alias. Prove only snapshots retain
  pending owned values; cold remounts are empty; `dispose` detaches nested
  registrations; observable Drop order matches the C++ owner boundary.
- [ ] **12. Direct C++ file correspondence.** Require the two new focused owner
  files, keep `state_machine_layer_instance.rs` as the private-layer owner,
  reduce both old files to thin entry/re-export surfaces, keep `artboard.rs`
  wrappers delegating only, preserve every W4 §C public name, and reject any
  displaced implementation or a false fidelity claim across a recorded seam.

## Out-of-scope recorded seams

FL-C5 must carry these dependencies as `RECORDED`; it must not make their owner
rows faithful or implement their internals under a StateMachine filename.

| Deferred owner | FL-C5-visible seam | Owning row / required disposition |
| --- | --- | --- |
| `src/listener_group.cpp` | Hover/click phase, consumed/dragged, disabled state, and per-pointer group internals behind the ListenerGroup-shaped seam | FL-D `listener_group.cpp`; keep Rust pointer capture/history tables until that row lands. Delete only per-listener orchestration displaced by FL-C5 hit owners. |
| `src/animation/text_input_listener_group.cpp` | Text-input listener-group internals | FL-E row. FL-C5 only preserves construction/routing order to the seam. |
| `src/input/gamepad_batch.cpp` | `submitGamepadsFromBuffer` definition and byte parser; scripted broadcast definition | Its own pending manifest row. Header declarations remain mapped; no FL-C5 implementation claim. |
| `src/input/focus_manager.cpp` | Focus tree traversal, cleanup, and manager internals | Existing `focus.rs` row remains `DIVERGENT`. FL-C5 ports its own selection/queue/process/call ordering only. |
| `src/semantic/semantic_manager.cpp` | Manager/tree/node-ID lookup for `enableSemantics`, `setExternalSemanticManager`, `semanticManager`, and `fireSemanticAction` | Absent semantic-manager row. FL-C5 ports its own queue/process/dispatch orchestration and records the dependency. |
| `src/semantic/semantic_data.cpp` | Semantic node callback internals | Absent semantic-data row. Same recorded boundary. |
| `audio_event.cpp` | Actual audio playback at the tail of `notifyEventListeners` | Absent audio-event row. FL-C5 ports listener → bubble → audio-seam order and `playsAudio == true`, not playback. |
| State-machine/artboard importers | Import-stack mechanics and importer ownership | Existing importer rows. FL-C5 represents `state_machine.cpp` import/onAdded semantics through the accepted Rust import architecture as a documented adaptation. |

## Compensation KEEP / DELETE decisions

Every W4 §B mechanism is accounted for. `KEEP` means a documented Rust
adaptation, not permission to bypass a C++-corresponding primary path.

| Rust-only mechanism (W4 §B) | Binding verdict | Closure citation and required proof |
| --- | --- | --- |
| Retained definition arena + numeric `state_machine_index` | **KEEP** | `stateMachine`, `m_machine`, W4 public API list; `B-LIFE`, `B-API`, `S-FIELDS`. |
| Public snapshot `Clone` | **KEEP** | Deleted copy-constructor and lifecycle rows; `B-LIFE`, `S-LIFE`. |
| `listener_definitions: Arc<Vec<_>>` | **KEEP** | Listener slot/group rows; stable immutable identity, no compaction; `S-SLOTS`. |
| File/default/owned VM catalogs and selectors | **KEEP** | Bind-family and public typed-context rows; they delegate to the primary bind family. |
| `requires_post_update_state_probe` and `post_update_probe_pending` | **DELETE** | Advance rows; `B-ADV` proves unconditional probing each settlement pass; `S-ADV` rejects both flags/gates. |
| Cached `changed_state_count` as sole state | **DELETE** | Restore per-layer flags and `stateChangedByIndex`; an aggregate may remain only as a derived cache; `B-LAYER`, `S-LAYER`. |
| `has_advanced_once` + clean zero-delta fast path | **DELETE** | Raw/facade advance rows; `B-ADV` proves bind/layer/input bookkeeping still runs; `S-ADV`. |
| Public/core event dual cursors | **KEEP** | Event queue/report access rows; `B-EVENT`, `S-EVENT`. |
| Rust notification queue object | **KEEP** | Listener-VM fields/report rows; duplicate FIFO and weak sink adaptation. |
| `RuntimeDataBindContainerQueue`, occurrence vector/enum | **KEEP** | Definition/data-bind/keyframe rows; authored cross-family order remains one logical queue. |
| Per-animation keyframe graph cache | **KEEP** | Keyframe rows; prove equivalence to on-demand C++ clones and occurrence isolation. |
| `owned_view_model_rebind_sink` | **KEEP** | Bind/relink rows; pushed structural replacement adaptation. |
| Pointer capture/history tables | **KEEP until FL-D** | Hit rows and recorded `listener_group.cpp` seam; delete only displaced per-listener traversal. |
| Finite/nonnegative validation + `Result` host seams | **KEEP only on distinct Rust convenience APIs** | C++ pointer/advance rows must forward all FP values; `B-HIT`, `B-ADV`, `S-ADV`. |
| Script lifecycle maps/flags | **KEEP** | Constructor/bind/init/lifecycle rows; facade mount timing adaptation. |
| `scripted_input_group_generation` and synchronization API | **KEEP** | Public API list and constructor seam; late-mount adaptation. |
| Terminal retained `script_error` | **KEEP** | Listener/event/script lifecycle rows; ordinary protected-call failure is consumed, selected resource failure stays terminal. |
| `active_owned_view_model_advance_context` | **KEEP** | Advanced-context/reset and public context API rows. |
| `scripted_facade_root_view_model` identity cache | **KEEP** | Bind/lifecycle rows; repeated A→A versus A→B facade adaptation. |
| Per-layer monotonic trigger-layer ID | **KEEP** | Layer fields/Clone rows; regenerated on snapshot to prevent aliasing. |
| Per-layer evaluated-random-weight scratch | **KEEP** | Layer scratch row; equal output plus cross-instance isolation. |
| Typed bindable arrays and transition-duration occurrences | **KEEP** | Definition/property/keyframe rows; preserve duplicate occurrences and converter ownership. |
| Action owner arena/handles | **KEEP** | Definition/action ordering and public scripted APIs; stable-owner adaptation. |
| Definition-level `requires_post_update_state_probe` scan | **DELETE** | Same unconditional-probe proof as the instance flags; `S-ADV`. |
| Host report snapshot/refresh projection | **KEEP** | `reportedEventAt`, host drain, and public event-context rows; live payload refresh required. |
| Formula-random injection/count APIs | **KEEP** | W4 public API list; oracle/test seam. |
| Transition-duration and VM-trigger probes | **KEEP** | W4 public API list; differential introspection seam. |
| Directional focus convenience APIs | **KEEP** | W4 public API list; remain distinct delegating extensions. |
| Alternate boolean return shapes | **KEEP on distinct Rust APIs** | W4 public API list; C++-corresponding primary methods retain their own result/FP behavior. |

## Public API preservation list

Every row from W4 §C must remain reachable through the thin entry points.
`S-API` must check names and visibility; the cited downstream evidence must
compile after the split.

| Public Rust API to preserve | Current definition / evidence | Required adaptation |
| --- | --- | --- |
| `RuntimeStateMachine` public `global_id`, `name`, `inputs`, `layers` | `state_machine.rs:171-177`; `artboard.rs:3687-3693,4444-4480` | Re-export unchanged from the new definition owner. |
| `RuntimeStateMachine::scripted_listener_actions` | `state_machine.rs:210-214`; scripted lifecycle tests | Keep filtered Rust convenience view. |
| `StateMachineInstance: Clone` | `instance.rs:642-762`; `flow_session.rs:1250-1252` | Keep approved non-aliasing snapshot. |
| `state_machine_index` | `instance.rs:2927-2929`; artboard and `cpp_probe` consumers | Keep stable numeric handle. |
| `input_index_named` | `instance.rs:3045-3049`; flow/scene/C API/public tests | Re-export unchanged. |
| Indexed `set_bool`, `set_number`, `fire_trigger` | `instance.rs:3051-3081`; flow/scene/public tests | Keep mutating convenience APIs. |
| `focus_up`, `focus_down`, `focus_left`, `focus_right` | `instance.rs:3096-3110`; higher-level focus routing | Keep directional extensions. |
| `key_input`, `text_input`, `gamepad_dispatch` | `instance.rs:3171-3371`; facade/input tests | Keep typed host-input APIs; do not claim the gamepad buffer parser. |
| Pointer API families with owned/event context, timestamp, or script host | `instance.rs:3665-4467`; scene/fuzz/workspace consumers | Keep signatures; route C++-corresponding base paths through hit owners and keep validating convenience paths distinct. |
| `pointer_down_with_event_context`, `pointer_up_with_event_context` | `instance.rs:3665-3684,3968-3987`; pointer context tests | Keep rendered occurrence metadata. |
| `take_reported_events` | `instance.rs:8711-8724`; flow/scene | Keep host cursor isolation. |
| `reported_event_snapshot` | `instance.rs:8688-8695`; `cpp_probe` | Keep immutable projection. |
| `has_pending_listener_view_model_reports` | `instance.rs:8663-8669`; frame loop/tests | Keep private-queue visibility. |
| `script_error`, `retain_scripted_object_data_context_error` | `instance.rs:2881-2894`; facade/flow | Keep terminal error channel. |
| `scripted_objects`, instance `scripted_listener_actions` | `instance.rs:1597-1606`; facade/lifecycle tests | Keep occurrence exposure. |
| Script occurrence installation/hydration APIs (`set_script_instance_for_global`, `set_script_input_for_global`, `set_scripted_listener_action_instance`, `set_scripted_object_instance`, `hydrate_and_initialize_*`, `install_scripted_object_data_context`) | `instance.rs:1574-1589,2037-2099,2518-2784`; facade/golden runner | Keep late-mount bridge. |
| `synchronize_scripted_input_groups` | `instance.rs:1464-1480`; facade | Keep generation-based cache rebuild. |
| Scripted binding/query family (`scripted_listener_action_input_snapshots`, `bind_scripted_listener_action_sources`, `bind_scripted_listener_input_source`, `bind_scripted_listener_converter_own_sources`, `finalize_scripted_listener_input_sources`, converter occurrence/snapshot APIs) | `instance.rs:1608-1762,2109-2560`; facade/golden/tests | Keep graph/converter/VM bridge. |
| Facade context transactions (`begin_scripted_object_data_context_bind`, `begin_retained_scripted_object_data_context_rebind`, `finish_scripted_object_data_context_bind`) | `instance.rs:7617-7704`; facade/golden | Keep fallible phased wrapper around primary bind family. |
| Transaction transfer (`adopt_scripted_listener_action_state_from`, `rehome_owned_data_context_for_transaction`) | `instance.rs:2785-2880`; flow | Keep candidate/commit adaptation. |
| Context-binding family (`bind_empty_data_context`, `bind_default_view_model_context`, `bind_view_model_instance_context`, `bind_imported_view_model_context`, `bind_owned_view_model_context`, `bind_owned_view_model_handle`, `bind_owned_view_model_context_handle`, `bind_owned_view_model_context_mut`, `bind_owned_view_model_contexts`, `bind_script_artboard_data_context`) | `instance.rs:7440-7753,8039-8108`; facade/probes | Keep typed wrappers, delegating to C++-shaped primary operations. |
| `set_bindable_{number,boolean,integer,color,string,enum,asset,artboard,list,trigger,view_model}_for_data_bind` | `instance.rs:5409-5635`; artboard/probes | Keep direct typed mutation seams. |
| Default-VM source setter/query/handle families | `instance.rs:5687-6867`; `cpp_probe` | Keep typed path/source handle APIs. |
| Imported/owned VM source setter families | `instance.rs:6868-7429`; `cpp_probe` | Keep ownership-specific adaptations. |
| Converter binding APIs (`bind_state_machine_data_bind_source`, `bind_state_machine_data_converter_own_sources`, `finalize_state_machine_data_bind_source`, `rebind_state_machine_data_converter_final_input`) | `instance.rs:2173-2298`; workspace consumers | Keep graph build-phase bridge. |
| `update_data_binds_apply_target_to_source` | `instance.rs:8501-8643`; downstream runtime/facade | Keep explicit public container update. |
| Formula-random APIs | `instance.rs:7474-7484`; differential probes | Keep deterministic oracle seam. |
| Transition-duration probes | `instance.rs:7486-7502`; differential probes | Keep occurrence introspection. |
| VM-trigger probes | `instance.rs:8727-8764`; differential probes | Keep trigger-cell introspection. |
| `bindable_*_value_for_data_bind` and default-source query families | `instance.rs:5687-5928`; differential probes | Keep graph introspection. |
| `StateMachineEventContext`, `StateMachineReportedEvent` accessors | `state_machine/event_report.rs:45-67,175-210`; flow/scene | Re-export ownership-safe report/context projections unchanged. |

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

Before the immutable candidate is submitted:

- [ ] every member row and all twelve adversarial rows are checked;
- [ ] the two new owner modules exist and both legacy files are thin entry
  points/re-exports;
- [ ] every compensation `KEEP` is documented in the corresponding member
  proof and every `DELETE` has a C++ differential proving the behavior it
  masked;
- [ ] every recorded seam names its owning row and remains unpromoted;
- [ ] every structural rule has a passing injected negative control;
- [ ] focused Rust tests and pinned-C++ differentials are green;
- [ ] the complete non-performance correctness floor, public/downstream API
  floor, structural checker, format/lint, C API, Apple, browser, pixel, size,
  and provenance gates required by the family procedure are green;
- [ ] exact source citations, test names, checker counts, gate counts, trace
  receipt, and immutable candidate identity are recorded here and in the
  mechanical status layers; and
- [ ] no performance measurement was run or used to select work.

