# W2 inventory — `state_machine_instance.cpp` definitions beginning at or before line 1706

## Scope and completeness

This inventory covers every definition whose body begins at or before `src/animation/state_machine_instance.cpp:1706`. The `StateMachineInstance` constructor begins at line 1707 and is used only as caller/lifecycle context; its own inventory belongs to W3. The complete source and header were read. There are no statics initialized by lambdas in this range. The only lambda in scope is the recursive child callback inside `addToHitLookup`. (`src/animation/state_machine_instance.cpp:1693-1702`)

## Cross-cutting listener and hit-order behavior

- Authored listeners are visited in machine-authored order. Event and view-model listeners are diverted away from pointer hit construction; pointer groups are created only when a listener contains one of the nine types in `kPointerHitListenerTypes`. Even if its target is null or unresolved, such a listener group is still retained in `m_listenerGroups`. (`src/animation/state_machine_instance.cpp:1827-1944`)
- A single hit component is reused per component through `hitLookup`; every listener reference is appended, preserving discovery order and duplicates. Component-provided groups follow authored groups, preserving provider/object, group, target, and returned-hit-component iteration order. Nested artboards, component lists, and text inputs are appended afterward. (`src/animation/state_machine_instance.cpp:1830-1944`; `src/animation/state_machine_instance.cpp:1969-2070`)
- Final hit order is not construction order. Artboard-targeting entries are moved to the front, then components whose `component()` equals a drawable are selected in the exact linked-list traversal obtained by rewinding from `firstDrawable()` through `prev` and then walking `next`. Entries not matched to a drawable retain whatever residual positions the swaps leave. The same sort is repeated after a draw-order counter change. (`src/animation/state_machine_instance.cpp:2255-2304`; `src/animation/state_machine_instance.cpp:2546-2554`)
- Every event performs three passes: reset listener groups, prepare every hit component, then process every hit component. Once an opaque result is seen, later components are still called but receive `canHit=false`, allowing exit/unhover cleanup. The result is `hitOpaque` if any component returns it, otherwise `hit` if any returns a hit, otherwise `none`. (`src/animation/state_machine_instance.cpp:1494-1545`)
- Listener pointer hover, prior hover, click phase, and previous position are keyed by pointer ID. However, `ListenerGroup::m_isConsumed` and `m_hasDragged` are shared across all pointer IDs, while `HitDrawable::isHovered` is also a single transient field rather than per-pointer state. (`include/rive/listener_group.hpp:19-30`; `include/rive/listener_group.hpp:60-66`; `src/animation/state_machine_instance.cpp:732-739`)
- During the prepare pass, a group attached to multiple hit components is marked hovered if any of them hits. During processing, the first occurrence that performs changes consumes that group, so later occurrences in the same event are skipped. Duplicate group entries that do not perform changes can be processed repeatedly. (`src/animation/state_machine_instance.cpp:743-767`; `src/animation/state_machine_instance.cpp:794-811`; `src/listener_group.cpp:180-264`)
- `ListenerGroup::reset(pointerId)` creates per-pointer state if absent. Unless that pointer is disabled, it clears the group-wide consumed flag, transfers current hover to previous hover, and clears current hover. A `clicked` phase becomes `out` even when the disabled branch prevented the hover reset. (`src/listener_group.cpp:24-60`)
- Pointer exit releases and pools the addressed pointer’s state only after processing. The pooled object is reset to unhovered, phase `out`, and previous position `(0,0)`. (`src/animation/state_machine_instance.cpp:1534-1541`; `src/listener_group.cpp:62-77`)
- A shape hit uses AABB, ancestor hit checks, and raw non-collapsed paths with a radius of 2; it does not inspect fills or strokes. The shape override calls `Component::hitTestPoint`, not its own `Drawable::hitTestPoint`, so the shape’s own hidden drawable flag is not directly checked. Ancestor drawables/layouts can still reject the hit, and collapsed paths are skipped. (`src/shapes/shape.cpp:161-183`; `src/shapes/shape.cpp:241-259`; `src/component.cpp:97-106`)
- Text runs similarly use their contours and an ancestor hit check; the owning text drawable therefore participates in hidden-state rejection. Layout hits invert their transform, test local bounds, and then invoke `Drawable::hitTestPoint`, which rejects hidden or collapsed drawables. (`src/text/text_value_run.cpp:180-209`; `src/layout_component.cpp:49-79`; `src/drawable.cpp:62-78`)
- Provider `isOpaque` is propagated recursively but actually applied only to layout targets. Shape and text-run branches discard the argument. Independently, `Drawable::isTargetOpaque()` can make any processed hit opaque. (`src/animation/state_machine_instance.cpp:1619-1705`; `src/animation/state_machine_instance.cpp:812-817`)

---

## File-static definitions

### `getStateName` (`src/animation/state_machine_instance.cpp:95-120`)

- role: Microprofile-only formatter returning `"(null)"`, an animation name or `"Animation"`, `"Entry"`, `"Exit"`, `"Any"`, or the fallback `"Blend"`.
- calls / called-by: `StateInstance::state`, type checks, `AnimationState::animation` / `StateMachineLayerInstance::tryChangeState`.
- once vs per-frame: per recorded transition, and absent entirely unless `RIVE_MICROPROFILE` is enabled.
- ownership & nullability: borrows the state instance, state, and animation; only the state-instance and animation pointers are checked. A non-null instance with a null `state()` is dereferenced.
- ordering & duplicates: n/a.
- validation & malformed input: unknown state types deliberately collapse to `"Blend"`.
- adversarial cases: null instance; animation state with null animation; an unrecognized state subtype.

### `kPointerHitListenerTypes` (`src/animation/state_machine_instance.cpp:127-137`)

- role: Exact pointer-listener membership set: `enter`, `exit`, `down`, `up`, `move`, `click`, `dragStart`, `dragEnd`, and `drag`.
- calls / called-by: none / constructor listener classification.
- once vs per-frame: compile-time constant, queried during construction.
- ordering & duplicates: fixed order above, with no duplicates.
- validation & malformed input: `event`, `componentProvided`, `textInput`, `viewModel`, `focus`, `blur`, `keyboard`, `semanticAction`, and `gamepad` are not pointer-hit members.
- adversarial cases: a listener containing only `dragStart`; one containing only `componentProvided`; one combining a pointer type with focus or keyboard.

---

# `StateMachineLayerInstance`

## Class and fields (`src/animation/state_machine_instance.cpp:140-711`)

- role: Owns one runtime layer’s current/previous/any-state instances, transition mix state, transition reset, and one-shot held-animation application.
- ownership & nullability:
  - `maxIterations=100` bounds chained transitions. (`src/animation/state_machine_instance.cpp:686-687`)
  - `m_stateMachineInstance`, `m_layer`, and `m_artboardInstance` are borrowed and initialized by `init`; all begin null. (`src/animation/state_machine_instance.cpp:688-690`)
  - `m_anyStateInstance`, `m_currentState`, and `m_stateFrom` are raw owned state instances, initially null. (`src/animation/state_machine_instance.cpp:692-694`)
  - `m_transition` is a borrowed shared transition; `m_transitionDurationProperty` is a borrowed per-instance binding override. (`src/animation/state_machine_instance.cpp:696-697`)
  - `m_animationReset` uniquely owns a reset object; `m_transitionCompleted`, `m_holdAnimationFrom`, `m_stateMachineChangedOnAdvance`, and `m_waitingForExit` are state flags. (`src/animation/state_machine_instance.cpp:698-707`)
  - `m_mix` and `m_mixFrom` start at `1`; `m_holdAnimation` is borrowed and `m_holdTime` starts at zero. (`src/animation/state_machine_instance.cpp:703-710`)
- lifecycle: `resetState` does not clear the transition pointer, duration binding, animation reset, held animation/time, hold flags, or mix fields; only the state instances are reset. (`src/animation/state_machine_instance.cpp:177-192`)
- adversarial cases: reset during a non-completed transition; destruction while two owned state fields alias; transition to null state.

### `StateMachineLayerInstance::~StateMachineLayerInstance` (`src/animation/state_machine_instance.cpp:143-148`)

- role: Deletes `m_anyStateInstance`, then `m_currentState`, then `m_stateFrom`.
- calls / called-by: none / deletion of the layer array.
- once vs per-frame: destruction-time.
- ownership & nullability: `delete nullptr` is harmless, but there is no alias guard; aliased non-null fields would be deleted more than once.
- lifecycle: does not call `removeStateKeyFrameBinds` or `clearAnimationReset`; outer destruction removes all data binds before deleting the layer array. (`src/animation/state_machine_instance.cpp:2169-2174`)
- adversarial cases: make `m_currentState == m_anyStateInstance`; verify exact deletion order.

### `StateMachineLayerInstance::init` (`src/animation/state_machine_instance.cpp:150-175`)

- role: Seeds the process-global C RNG, stores owner/artboard pointers, creates the any-state instance and its keyframe binds, stores the layer, and enters its entry state.
- calls / called-by: `buildStateKeyFrameBinds`, `changeState` / `StateMachineInstance` constructor.
- once vs per-frame: construction-time per layer.
- ownership & nullability: assumes all three arguments, `layer->anyState()`, and its instance are valid; only `m_layer == nullptr` is asserted.
- ordering & duplicates: RNG seeding happens once per layer, so constructing multiple layers repeatedly overwrites the same global seed. Deterministic mode always seeds `1`; otherwise nanoseconds are truncated to `unsigned int`.
- lifecycle: any-state keyframe binds are built before `m_layer` is assigned and before entering the entry state.
- adversarial cases: two-layer deterministic machine; null any state; an already-initialized layer in release mode where the assertion is absent.

### `StateMachineLayerInstance::resetState` (`src/animation/state_machine_instance.cpp:177-192`)

- role: Removes binds and deletes obsolete `stateFrom` and current state, nulls them, then recreates the layer entry state.
- calls / called-by: `removeStateKeyFrameBinds`, `changeState` / `StateMachineInstance::resetState`.
- once vs per-frame: on-demand reset.
- ownership & nullability: avoids deleting `stateFrom` when it equals any-state or current; avoids deleting current when it equals any-state. `removeStateKeyFrameBinds(nullptr)` is permitted by its implementation.
- ordering & duplicates: old `stateFrom` is handled before current; entry-state end/start behavior comes from `changeState`.
- lifecycle: stale transition/mix/hold fields survive this reset.
- adversarial cases: `stateFrom == current`; `current == any`; reset with a pending held animation.

### `StateMachineLayerInstance::updateMix` (`src/animation/state_machine_instance.cpp:194-223`)

- role: Advances and clamps transition mix, or forces it to `1`; when mix first reaches `1`, releases the animation reset, then fires transition end events followed by end listener actions.
- calls / called-by: `resolvedDuration`, `resolvedMixTime`, `clearAnimationReset`, `fireEvents`, `performListenerActions` / `advance`, `tryChangeState`.
- once vs per-frame: per layer advance and at zero time immediately after a transition.
- ownership & nullability: transition and `stateFrom` must both exist for timed mixing. A zero duration takes the force-to-one branch without end callbacks here because zero-duration callbacks are emitted in `tryChangeState`.
- ordering & duplicates: all transition events are fired before all matching listener actions.
- queues/dirt/timing: completion callbacks occur in the same call that produces `m_mix == 1`.
- FP/zero edges: exact zero `mixTime` forces `1`. Otherwise `m_mix + seconds/mixTime` is clamped to `[0,1]`; positive infinity becomes `1`, negative infinity becomes `0`, and with conventional `std::min/max` ordering a NaN candidate collapses to the lower bound. Signed zero time leaves the arithmetic value unchanged.
- adversarial cases: percentage duration over a zero-duration animation; negative seconds; NaN seconds; already-complete transition with `m_transitionCompleted=false`.

### `StateMachineLayerInstance::advance` (`src/animation/state_machine_instance.cpp:225-278`)

- role: Advances current state, updates the mix, conditionally advances the outgoing state, applies states, repeatedly takes allowed transitions and reapplies, clears spilled time, and reports whether work remains.
- calls / called-by: `updateMix`, `apply`, `updateState` / `StateMachineInstance::advance`.
- once vs per-frame: per layer per advance; `newFrame=false` is also used for same-frame zero-time convergence.
- ownership & nullability: dereferences `m_currentState` before any null check and again when clearing spilled time; a null current state crashes.
- ordering & duplicates: current advances first, then mix completion callbacks, then outgoing state, then apply. Each successful chained transition is followed immediately by another apply.
- validation & malformed input: the guard is tested after a successful transition. Successful iterations numbered `0` through `100` execute; reaching `i==100` prints an error and returns `false`.
- queues/dirt/timing: `m_stateMachineChangedOnAdvance` is cleared only on a new frame.
- FP/zero edges: seconds is forwarded without validation to both states and `updateMix`.
- adversarial cases: 101 immediately enabled transitions; null entry state; `newFrame=false` after a prior state change.

### `StateMachineLayerInstance::resolvedDuration` (`src/animation/state_machine_instance.cpp:283-291`)

- role: Returns the rounded bound duration override, clamped only against negative values, or the shared transition duration.
- calls / called-by: none / `updateMix`, `resolvedMixTime`, `isTransitioning`, `tryChangeState`.
- ownership & nullability: if no override exists, `m_transition` is dereferenced without a null check.
- FP/zero edges: negative values become `0`; `-0` rounds/converts to zero; non-negative fractions use `std::round` before conversion. NaN, infinity, or an out-of-range rounded value reaches a floating-to-`uint32_t` conversion outside the representable range, which is undefined behavior in C++.
- adversarial cases: `-0.1`, `0.5`, `1.5`, NaN, infinity, and a value above `UINT32_MAX`.

### `StateMachineLayerInstance::resolvedMixTime` (`src/animation/state_machine_instance.cpp:294-316`)

- role: Converts duration to seconds, interpreting percentage duration against the outgoing animation’s duration.
- calls / called-by: `resolvedDuration` / `updateMix`.
- ownership & nullability: percentage mode dereferences `m_stateFrom`; a non-animation or animation state with null animation yields animation duration zero.
- FP/zero edges: resolved duration zero returns exact zero. Percentage mode multiplies by the animation duration; NaN or infinity from that duration propagates. Millisecond mode divides the unsigned duration by `1000`.
- adversarial cases: percentage transition from a blend state; null animation; infinite animation duration.

### `StateMachineLayerInstance::isTransitioning` (`src/animation/state_machine_instance.cpp:318-322`)

- role: Reports a timed, incomplete transition only when transition and outgoing state exist, resolved duration is nonzero, and mix is below one.
- calls / called-by: `resolvedDuration` / `updateState`.
- once vs per-frame: per transition-selection attempt.
- FP/zero edges: NaN `m_mix` fails the `<1` comparison and therefore reports false.
- adversarial cases: nonzero duration with null `stateFrom`; `m_mix==1`; NaN mix.

### `StateMachineLayerInstance::updateState` (`src/animation/state_machine_instance.cpp:324-341`)

- role: Blocks transition selection during a non-early-exit mix, otherwise clears waiting state, tries any-state transitions first, and only then current-state transitions.
- calls / called-by: `isTransitioning`, `tryChangeState` / `advance`, `StateMachineInstance::tryChangeState`.
- ordering & duplicates: any-state has strict priority; a successful any transition prevents evaluation of current-state transitions.
- queues/dirt/timing: `m_waitingForExit` is reset before evaluating either source and can be set again by failed conditions.
- adversarial cases: both any and current have allowed transitions; early exit disabled; any-state condition returns waiting while current transition succeeds.

### `StateMachineLayerInstance::fireEvents` (`src/animation/state_machine_instance.cpp:343-353`)

- role: Performs every fire action whose occurrence equals the requested occurrence.
- calls / called-by: `StateMachineFireAction::perform` / `updateMix`, `changeState`, `tryChangeState`.
- once vs per-frame: on state/transition start or end.
- ordering & duplicates: preserves vector order and performs duplicate pointers repeatedly.
- ownership & nullability: borrowed action pointers are dereferenced without null checks.
- queues/dirt/timing: effects occur synchronously.
- adversarial cases: two identical actions; a null action; mixed occurrence values.

### `StateMachineLayerInstance::performListenerActions` (`src/animation/state_machine_instance.cpp:355-367`)

- role: Performs every matching scheduled listener action using `ListenerInvocation::none()`.
- calls / called-by: `ListenerAction::perform` / `updateMix`, `changeState`, `tryChangeState`.
- ordering & duplicates: preserves vector order; duplicates execute repeatedly.
- ownership & nullability: actions are owned by `unique_ptr` in the source vector and are assumed non-null.
- queues/dirt/timing: synchronous, after corresponding fire events at every call site.
- adversarial cases: duplicate matching actions; start and end actions interleaved in the vector.

### `StateMachineLayerInstance::canChangeState` (`src/animation/state_machine_instance.cpp:369-374`)

- role: Rejects a transition whose destination pointer equals the current state definition pointer.
- calls / called-by: none / `findRandomTransition`, `findAllowedTransition`.
- ownership & nullability: null destination is allowed unless current is also null.
- adversarial cases: self-transition authored as the same pointer; distinct equivalent state object; null current and null destination.

### `StateMachineLayerInstance::randomValue` (`src/animation/state_machine_instance.cpp:376`)

- role: Returns `RandomProvider::generateRandomFloat()`.
- calls / called-by: `RandomProvider::generateRandomFloat` / `findRandomTransition`.
- once vs per-frame: only when at least one random transition contributes positive total weight.
- FP/zero edges: production can return exactly `1` because it divides by `RAND_MAX`; tests can inject arbitrary float values. (`include/rive/math/random.hpp:30-47`)
- adversarial cases: injected negative, NaN, or `1.0` value.

### `StateMachineLayerInstance::changeState` (`src/animation/state_machine_instance.cpp:378-410`)

- role: If destination differs, fires outgoing-state end events/actions, replaces `m_currentState`, builds incoming keyframe binds, and fires incoming-state start events/actions.
- calls / called-by: `fireEvents`, `performListenerActions`, `buildStateKeyFrameBinds` / `init`, `resetState`, `tryChangeState`.
- ownership & nullability: creates and owns the new instance but does not delete the old current instance; callers must preserve/delete it. Null destination sets current to null.
- ordering & duplicates: outgoing events, outgoing actions, instance construction, incoming bind construction, incoming events, incoming actions.
- lifecycle: same-state pointer is a complete no-op.
- adversarial cases: direct repeated calls outside `tryChangeState`; null destination; `makeInstance` returning null.

### `StateMachineLayerInstance::findRandomTransition` (`src/animation/state_machine_instance.cpp:412-468`)

- role: Evaluates every authored transition, records eligible weights, totals them, draws one weighted transition, consumes its layer conditions, and returns it.
- calls / called-by: `canChangeState`, `randomValue` / `findAllowedTransition`.
- ordering & duplicates: first evaluation and weighted selection both use authored transition order. Duplicate transition entries contribute repeatedly.
- validation & malformed input: disallowed/self transitions receive evaluated weight zero. `waitingForExit` is remembered. `uint32_t totalWeight` can wrap on overflow.
- queues/dirt/timing: `evaluatedRandomWeight` is mutated on every visited transition; only the selected transition calls `useLayerInConditions`.
- FP/zero edges: zero total avoids RNG. Selection uses strict `currentWeight + weight > randomWeight`; exact cumulative boundaries select the following positive bucket, zero-weight entries never win, and random value exactly `1` normally returns null.
- adversarial cases: weights summing past `UINT32_MAX`; RNG `0`, exact boundary, `1`, NaN; all transitions waiting for exit.

### `StateMachineLayerInstance::findAllowedTransition` (`src/animation/state_machine_instance.cpp:470-509`)

- role: Delegates random states to weighted selection; otherwise returns the first authored transition to a different state whose conditions allow it.
- calls / called-by: `findRandomTransition`, `canChangeState` / `tryChangeState`.
- ordering & duplicates: non-random selection is first-match authored order and stops immediately.
- validation & malformed input: denied candidates get evaluated weight zero and can set waiting-for-exit. A same-state candidate is skipped without clearing any stale evaluated weight.
- queues/dirt/timing: chosen transition stores its random weight and consumes layer conditions before return.
- adversarial cases: stale weight on a skipped self-transition; first denied/second allowed; first waiting/second allowed.

### `StateMachineLayerInstance::buildAnimationResetForTransition` (`src/animation/state_machine_instance.cpp:511-517`)

- role: Replaces `m_animationReset` with a reset produced from outgoing/current states and the artboard.
- calls / called-by: `AnimationResetFactory::fromStates` / `tryChangeState`.
- ownership & nullability: assigns into a `unique_ptr`; source pointers are passed as-is.
- lifecycle: caller clears any prior pooled reset before invoking it.
- adversarial cases: null outgoing/current state; factory returning null.

### `StateMachineLayerInstance::clearAnimationReset` (`src/animation/state_machine_instance.cpp:519-526`)

- role: Returns a non-null reset to `AnimationResetFactory::release`, then explicitly nulls the pointer.
- calls / called-by: `AnimationResetFactory::release` / `updateMix`, `tryChangeState`.
- once vs per-frame: on timed transition completion or replacement.
- lifecycle: null is a no-op; release occurs before the explicit null assignment.
- adversarial cases: repeated clear; transition replaced before completion.

### `StateMachineLayerInstance::tryChangeState` (`src/animation/state_machine_instance.cpp:528-630`)

- role: Selects a transition, changes state, configures duration and callbacks, retires old outgoing state, builds reset/hold state, advances the new state by spilled time, and starts its mix.
- calls / called-by: `findAllowedTransition`, `clearAnimationReset`, `changeState`, `fireEvents`, `performListenerActions`, `resolvedDuration`, `buildAnimationResetForTransition`, `updateMix` / `updateState`.
- ownership & nullability: null source returns false. The previous current state becomes `m_stateFrom`; an older `m_stateFrom` is unbound/deleted unless it is any-state. A true `applyExitCondition` is followed by an unchecked `static_cast<AnimationStateInstance*>`.
- ordering & duplicates: state end/start callbacks from `changeState` occur before transition start callbacks. Zero duration then fires transition end events/actions immediately. Old outgoing state is deleted only afterward.
- lifecycle: any previous reset is released before changing state. A non-completed transition builds a reset. A held animation is consumed later by `apply`.
- queues/dirt/timing: new state advances immediately by outgoing animation spilled time, then mix is set to zero and updated at zero time in the same call.
- FP/zero edges: exact zero duration completes synchronously. `m_mixFrom` preserves the prior mix. Hold time and spilled time are forwarded without validation.
- adversarial cases: early transition interrupted at partial mix; zero-duration transition; exit condition true for a non-animation state; old `m_stateFrom==m_anyStateInstance`; null destination.

### `StateMachineLayerInstance::apply` (`src/animation/state_machine_instance.cpp:632-663`)

- role: Applies reset, applies and clears a one-shot held animation, obtains the transition interpolator, then applies outgoing and current states with their mix values.
- calls / called-by: `AnimationReset::apply`, `LinearAnimation::apply`, interpolator transform, state `apply` / `advance`.
- ordering & duplicates: reset first, held animation second, outgoing state third, current state last.
- ownership & nullability: held animation is borrowed and cleared after one application; reset remains until transition completion/replacement.
- FP/zero edges: outgoing state is applied only when `m_mix<1`; NaN mix suppresses it. Interpolator output is not clamped or validated.
- adversarial cases: held animation plus reset; null interpolator; NaN mix; current state null.

### `StateMachineLayerInstance::stateChangedOnAdvance` (`src/animation/state_machine_instance.cpp:665-668`)

- role: Trivial accessor for the per-new-frame state-change flag.
- calls / called-by: none / state-change reporting methods.
- once vs per-frame: on-demand.
- lifecycle: flag persists across `newFrame=false` advances.
- adversarial cases: query after a same-frame zero-time convergence pass.

### `StateMachineLayerInstance::currentState` (`src/animation/state_machine_instance.cpp:670-673`)

- role: Returns the shared state definition of current state, or null.
- calls / called-by: none / testing and state-change reporting.
- ownership & nullability: borrowed return.
- adversarial cases: current instance null.

### `StateMachineLayerInstance::currentAnimation` (`src/animation/state_machine_instance.cpp:675-684`)

- role: Returns the current animation instance only when current state exists and is an `AnimationState`.
- calls / called-by: none / current-animation reporting methods.
- ownership & nullability: borrowed return; null for missing or non-animation state.
- adversarial cases: blend/current null; animation-state instance with null animation instance.

---

# Hit-component hierarchy

## Header-defined base context: `HitComponent` (`include/rive/animation/state_machine_instance.hpp:476-506`)

This base is outside the pinned `.cpp` body cutoff but is necessary context. It stores borrowed `Component*` and `StateMachineInstance*`, exposes the component, has a trivial virtual destructor, pure event/gamepad/prepare/hit methods, and default no-op pointer enable/disable methods. Neither pointer is validated. (`include/rive/animation/state_machine_instance.hpp:476-506`)

## `HitDrawable` fields (`src/animation/state_machine_instance.cpp:716-739`)

- `hitRadius=2` is present but unused by this class; concrete shape/text hit tests independently hard-code radius 2. (`src/animation/state_machine_instance.cpp:732`; `src/shapes/shape.cpp:257`; `src/text/text_value_run.cpp:207`)
- `isHovered` is one transient boolean for the hit component, not per pointer. `canEarlyOut` starts true; down/up requirement flags start false; explicit `isOpaque` starts false. (`src/animation/state_machine_instance.cpp:733-737`)
- `m_drawable` is borrowed and assumed non-null. `listeners` is an ordered non-owning vector; ownership remains in `StateMachineInstance::m_listenerGroups` or the provider. (`src/animation/state_machine_instance.cpp:738-739`; `include/rive/animation/state_machine_instance.hpp:391-392`)

### `HitDrawable::HitDrawable` (`src/animation/state_machine_instance.cpp:719-731`)

- role: Initializes the base, stores drawable and explicit opacity, and disables early-out permanently when the drawable is target-opaque at construction.
- calls / called-by: `HitComponent::HitComponent`, `Drawable::isTargetOpaque` / derived constructors.
- once vs per-frame: construction-time.
- ownership & nullability: both pointers are borrowed and dereferenced without checks.
- lifecycle: later opacity changes are queried by `processEvent`, but `canEarlyOut` is not recomputed.
- adversarial cases: drawable changes from opaque to non-opaque or vice versa after construction; null drawable.

### `HitDrawable::hitTest` (`src/animation/state_machine_instance.cpp:741`)

- role: Base concrete fallback always returning false.
- calls / called-by: none / `prepareEvent`, direct `StateMachineInstance::hitTest` if an unsubclassed object existed.
- adversarial cases: instantiate through a subclass that does not override hit testing.

### `HitDrawable::prepareEvent` (`src/animation/state_machine_instance.cpp:743-767`)

- role: Optionally skips hit testing, otherwise sets transient hover to false for exit or to the geometric hit result, then marks every attached group hovered for the pointer.
- calls / called-by: `hitTest`, `ListenerGroup::hover` / `StateMachineInstance::updateListeners`.
- once vs per-frame: once per hit component per pointer event.
- ordering & duplicates: listener vector order is preserved; duplicate group pointers receive duplicate idempotent `hover` calls.
- validation & malformed input: early-out occurs unless this is a required down/up event when `canEarlyOut` is true. It leaves the previous `HitDrawable::isHovered` value untouched.
- FP/zero edges: position is forwarded unvalidated.
- adversarial cases: an up-only target during move after a prior hit, proving stale `isHovered` is harmless only because `processEvent` also early-outs; exit never geometrically hit-tests.

### `HitDrawable::processGamepadInvocation` (`src/animation/state_machine_instance.cpp:769-774`)

- role: Ignores invocation and returns `HitResult::none`.
- calls / called-by: none.
- ownership & nullability: arguments are unused.
- adversarial cases: gamepad-aware scripted drawable represented only by this hit type.

### `HitDrawable::processEvent` (`src/animation/state_machine_instance.cpp:776-818`)

- role: Processes every unconsumed listener group and returns a geometric hit, upgraded to opaque by explicit opacity, dynamic target opacity, or a listener result of `scroll`.
- calls / called-by: `ListenerGroup::processEvent`, `Drawable::isTargetOpaque` / `StateMachineInstance::updateListeners`.
- once vs per-frame: once per hit component per pointer event unless early-out.
- ordering & duplicates: exact `listeners` order; consumed entries are skipped. It does not stop after a scroll/blocking result.
- validation & malformed input: same early-out predicate as `prepareEvent`. Listener processing occurs even when this component is not hovered so exits/click-phase cleanup can happen.
- ownership & nullability: assumes listener pointers, component, drawable, and state-machine pointer are valid.
- queues/dirt/timing: listener actions are synchronous and may mark the state machine for advance; the returned hit depends only on `isHovered && canHit`, not on whether an action matched.
- FP/zero edges: position/timestamp are forwarded unvalidated.
- adversarial cases: hovered target with no matching action still returns hit; occluded target gets `canHit=false` and returns none; a scrolling listener makes the hit opaque.

### `HitDrawable::addListener` (`src/animation/state_machine_instance.cpp:820-838`)

- role: Incorporates a listener group’s early-out/down/up requirements, then appends it.
- calls / called-by: `canEarlyOut`, `needsDownListener`, `needsUpListener` / `addToHitLookup`, text-input construction.
- once vs per-frame: construction-time.
- ordering & duplicates: always appends; no uniqueness check.
- validation & malformed input: null group crashes. Once `canEarlyOut` becomes false it never returns to true.
- adversarial cases: same group added twice; click-only group must set both down and up requirements; enter listener disables early-out.

### `HitDrawable::enablePointerEvents` (`src/animation/state_machine_instance.cpp:840-846`)

- role: Enables the addressed pointer on every attached group.
- calls / called-by: `ListenerGroup::enable` / `StateMachineInstance::enablePointerEvents`, `dragEnd`.
- ordering & duplicates: listener order and duplicates are preserved.
- lifecycle: enabling sets pointer phase to `out`; it does not itself clear the group-wide consumed flag. (`src/listener_group.cpp:80-84`)
- adversarial cases: duplicate group attachment; enabling one pointer while another pointer consumed the group.

### `HitDrawable::disablePointerEvents` (`src/animation/state_machine_instance.cpp:848-854`)

- role: Disables the addressed pointer on every attached group.
- calls / called-by: `ListenerGroup::disable` / `StateMachineInstance::disablePointerEvents`, `dragStart`.
- ordering & duplicates: listener order and duplicates are preserved.
- lifecycle: disabling sets that pointer phase to `disabled` and consumes the whole group. (`src/listener_group.cpp:86-91`)
- adversarial cases: disable pointer 1 while pointer 2 is active; same group attached to several hit components.

## `HitExpandable`

### `HitExpandable::HitExpandable` (`src/animation/state_machine_instance.cpp:861-866`)

- role: Pure forwarding constructor for a drawable/component hit target.
- calls / called-by: `HitDrawable::HitDrawable` / shape, text-run, and text-input construction.
- ownership & nullability: borrows all pointers; no validation.
- adversarial cases: drawable and component are different objects, as for a text run.

### `HitExpandable::hitTest` (`src/animation/state_machine_instance.cpp:868-871`)

- role: Calls component hit testing with `skipOnUnclipped=true`, `isPrimaryHit=true`.
- calls / called-by: `Component::hitTestPoint` / `HitDrawable::prepareEvent`, `StateMachineInstance::hitTest`.
- FP/zero edges: position is forwarded unvalidated; transform/AABB/path implementations decide the result.
- adversarial cases: shape with no fill or stroke but a valid path; hidden shape; clipped ancestor; singular ancestor transform.

## `HitTextRun`

### `HitTextRun::HitTextRun` (`src/animation/state_machine_instance.cpp:877-887`)

- role: Creates a hit-expandable whose drawable is the owning text component and whose component is the text run, then marks a non-null run as a hit target.
- calls / called-by: `HitExpandable::HitExpandable`, `TextValueRun::isHitTarget` / `addToHitLookup`.
- ownership & nullability: component is conditionally checked only after the base has already stored it; null survives construction but later hit testing dereferences it.
- lifecycle: setting `isHitTarget(true)` is not undone by destruction.
- adversarial cases: null run; multiple listeners reuse one run; remove all listeners and observe the flag remains true.

## `HitLayout`

### `HitLayout::HitLayout` (`src/animation/state_machine_instance.cpp:893-897`)

- role: Uses one drawable as both the hit component and drawable.
- calls / called-by: `HitDrawable::HitDrawable` / `addToHitLookup`.
- ownership & nullability: borrowed pointer, unchecked.
- adversarial cases: a drawable proxy; an actual layout component; null target.

### `HitLayout::hitTest` (`src/animation/state_machine_instance.cpp:899-902`)

- role: Calls component hit testing with `skipOnUnclipped=false`, forcing layout bounds to participate.
- calls / called-by: `Component::hitTestPoint` / hit preparation and direct hit testing.
- FP/zero edges: a singular world transform returns false in `LayoutComponent::hitTestPoint`. (`src/layout_component.cpp:49-79`)
- adversarial cases: unclipped layout outside bounds; hidden layout; singular transform; frame-origin artboard.

## `HitNestedArtboard`

### `HitNestedArtboard::HitNestedArtboard` (`src/animation/state_machine_instance.cpp:908-911`)

- role: Wraps a nested-artboard component.
- calls / called-by: `HitComponent::HitComponent` / outer constructor’s nested-artboard loop.
- ownership & nullability: borrowed component, unchecked.
- adversarial cases: non-`NestedArtboard` component passed by mistake.

### `HitNestedArtboard::~HitNestedArtboard` (`src/animation/state_machine_instance.cpp:912`)

- role: Empty derived destructor.
- calls / called-by: none / `unique_ptr<HitComponent>` destruction.
- lifecycle: base destructor follows; nested artboard is not owned.

### `HitNestedArtboard::hitTest` (`src/animation/state_machine_instance.cpp:914-941`)

- role: Rejects collapsed/paused or untransformable artboards, transforms the position, then returns true on the first nested state machine whose `hitTest` succeeds.
- calls / called-by: nested `hitTest` / parent direct hit testing.
- ordering & duplicates: nested-animation authored order; ignores non-state-machine animations and stops on first hit.
- validation & malformed input: checks collapse, pause, and `worldToLocal`, but assumes every `NestedStateMachine` has a usable wrapper.
- FP/zero edges: zero-scale/non-invertible transform returns false; transformed coordinates are otherwise unvalidated.
- adversarial cases: paused artboard; first nested SM misses and second hits; singular transform.

### `HitNestedArtboard::processGamepadInvocation` (`src/animation/state_machine_instance.cpp:942-960`)

- role: Broadcasts to every nested state machine and always returns `HitResult::none`.
- calls / called-by: nested `broadcastGamepadToScriptedDrawables`.
- ordering & duplicates: nested-animation order; no result aggregation and no early-out.
- ownership & nullability: dereferences each nested state-machine instance without checking it.
- adversarial cases: nested broadcaster returns opaque; null nested instance; multiple nested machines.

### `HitNestedArtboard::processEvent` (`src/animation/state_machine_instance.cpp:961-1067`)

- role: Transforms and forwards supported pointer events to every nested state machine; when occluded, converts down/up/move/exit to child `pointerExit`.
- calls / called-by: nested pointer/drag methods / parent listener update.
- ordering & duplicates: nested-animation order. For down/up/move/exit, each later child overwrites `hitResult`, so a later `none` can erase an earlier `hitOpaque`.
- validation & malformed input: collapse, pause, and transform failure return none. `enter`, `event`, `click`, `componentProvided`, `textInput`, `viewModel`, `drag`, focus/blur/keyboard/semantic/gamepad are ignored.
- queues/dirt/timing: drag start/end side effects are forwarded but their return values are discarded, so this method returns none for those types.
- FP/zero edges: move/drag timestamp is forwarded; down/up/exit do not use it.
- adversarial cases: two nested machines where the first hits and second misses; occluded move triggers exits; drag start returns none despite child action.

### `HitNestedArtboard::prepareEvent` (`src/animation/state_machine_instance.cpp:1068-1071`)

- role: Intentional no-op.
- calls / called-by: none / parent prepare pass.
- adversarial cases: verify child hover is not updated until `processEvent`.

## `HitComponentList`

### `HitComponentList::HitComponentList` (`src/animation/state_machine_instance.cpp:1077-1080`)

- role: Wraps an `ArtboardComponentList`.
- calls / called-by: base constructor / outer constructor’s component-list loop.
- ownership & nullability: borrowed component, unchecked.

### `HitComponentList::~HitComponentList` (`src/animation/state_machine_instance.cpp:1081`)

- role: Empty derived destructor.
- lifecycle: list and item state machines are not owned by this wrapper.

### `HitComponentList::hitTest` (`src/animation/state_machine_instance.cpp:1083-1107`)

- role: For a non-collapsed list, visits `orderedListIndices()` in reverse, transforms per item, and returns true on the first non-null state machine that hits.
- calls / called-by: item `hitTest` / parent direct hit testing.
- ordering & duplicates: exact reverse index order; duplicate indices are revisited.
- validation & malformed input: transform failure and null state machine are skipped; index validity is delegated to the list.
- FP/zero edges: singular per-item transform is skipped.
- adversarial cases: collapsed list; duplicate indices; top item null and lower item hits.

### `HitComponentList::processEvent` (`src/animation/state_machine_instance.cpp:1108-1226`)

- role: Visits items in reverse order, transforms the event, forwards supported events, aggregates the strongest result, and suppresses/cleans up later items after opacity.
- calls / called-by: item pointer/drag methods / parent update.
- ordering & duplicates: reverse ordered indices. Result changes `none→hit/hitOpaque` or `hit→hitOpaque`; it never downgrades.
- validation & malformed input: collapsed list returns none; transform failures and null state machines are skipped.
- opaque/occlusion semantics: after an item produces `hitOpaque`, `runningCanHit=false`. Subsequent down/up/move/exit calls become `pointerExit`; drag start/end do nothing for those items.
- queues/dirt/timing: forwarded drag start uses timestamp `0` and `disablePointer=true`; drag end uses timestamp `0`. Their results are discarded.
- FP/zero edges: original timestamp is forwarded only for move.
- adversarial cases: first item opaque, second previously hovered; drag across multiple items; duplicate item index; initial parent `canHit=false`.

### `HitComponentList::processGamepadInvocation` (`src/animation/state_machine_instance.cpp:1227-1269`)

- role: Reverse-visits non-null item state machines, broadcasts while allowed, and aggregates `none/hit/hitOpaque`.
- calls / called-by: item `broadcastGamepadToScriptedDrawables`.
- ordering & duplicates: reverse item order; duplicates rebroadcast.
- opaque/occlusion semantics: after opaque, later state machines are not invoked at all.
- validation & malformed input: collapsed list returns none; null item state machines are skipped.
- adversarial cases: opaque first item; duplicate index; collapsed list.

### `HitComponentList::prepareEvent` (`src/animation/state_machine_instance.cpp:1270-1273`)

- role: Intentional no-op.
- calls / called-by: none / parent prepare pass.

---

# Listener view-model binding family

## `ListenerViewModelPropertyBinding` fields (`src/animation/state_machine_instance.cpp:1280-1293`)

- `m_parent` is a borrowed `ListenerViewModel*`, initially null; `m_viewModelInstanceValue` is a reference-counted property pointer, initially null. (`src/animation/state_machine_instance.cpp:1289-1292`)
- lifecycle: the property holds this object as a raw dependent, so the binding must remove itself before releasing its strong property reference. (`src/animation/state_machine_instance.cpp:1401-1424`)

### `ListenerViewModelPropertyBinding::ListenerViewModelPropertyBinding` (`src/animation/state_machine_instance.cpp:1401-1407`)

- role: Stores parent, takes an `rcp` reference to the property, and registers itself as a dependent.
- calls / called-by: `ref_rcp`, `addDependent` / derived constructors.
- once vs per-frame: binding/rebinding construction-time.
- ownership & nullability: `vmProp` is unconditionally dereferenced; parent may technically be null but later reporting then no-ops.
- ordering & duplicates: registration follows strong-reference acquisition; duplicate bindings register independently.
- adversarial cases: null property; two bindings to the same property; null parent.

### `ListenerViewModelPropertyBinding::relinkDataBind` (`src/animation/state_machine_instance.cpp:1409`)

- role: Base implementation is a no-op.
- calls / called-by: none / dependency relinking through the virtual interface.
- lifecycle: retains the old property unchanged.
- adversarial cases: instantiate/use base implementation across a context replacement.

### `ListenerViewModelPropertyBinding::~ListenerViewModelPropertyBinding` (`src/animation/state_machine_instance.cpp:1411-1414`)

- role: Calls `clearDataContext`.
- calls / called-by: `clearDataContext` / binding destruction.
- lifecycle: derived members are destroyed first; then the dependent is removed before the property `rcp` field’s own destruction.
- adversarial cases: property already cleared; property currently notifying dependents.

### `ListenerViewModelPropertyBinding::clearDataContext` (`src/animation/state_machine_instance.cpp:1416-1424`)

- role: Removes this dependent from a non-null property, then releases the reference.
- calls / called-by: `removeDependent` / destructor and derived relink methods.
- ordering & duplicates: removal strictly precedes setting the `rcp` to null.
- lifecycle: idempotent when already null.
- adversarial cases: repeated clear; unresolved relink after clear.

### `ListenerViewModelPropertyBinding::addDirt` (`src/animation/state_machine_instance.cpp:1481-1488`)

- role: Reports the current property through the parent whenever both pointers are non-null.
- calls / called-by: `ListenerViewModel::reportToStateMachine` / property dependency notification.
- once vs per-frame: per dirt notification.
- validation & malformed input: ignores both `value` and `recurse`; no dirt filtering or deduplication.
- queues/dirt/timing: enqueues immediately, but listener actions are applied later by `applyEvents`. (`src/animation/state_machine_instance.cpp:2320-2335`; `src/animation/state_machine_instance.cpp:3021-3025`)
- adversarial cases: repeated dirt in one frame; dirt after property clear; null parent.

## `ListenerViewModelPropertyBindingListener`

### Field `m_listener` (`src/animation/state_machine_instance.cpp:1294-1305`)

- role: Borrowed `StateMachineListenerSingle*` supplying the relink path.
- ownership & nullability: not owned and not null-checked.

### `ListenerViewModelPropertyBindingListener::ListenerViewModelPropertyBindingListener` (`src/animation/state_machine_instance.cpp:1426-1432`)

- role: Constructs the base binding, then stores the single listener.
- calls / called-by: base constructor / `ListenerViewModel::bindFromContext`.
- ordering & duplicates: dependent registration occurs before `m_listener` initialization because the base constructor runs first.
- adversarial cases: null listener; property emits synchronously during registration.

### `ListenerViewModelPropertyBindingListener::relinkDataBind` (`src/animation/state_machine_instance.cpp:1434-1452`)

- role: Resolves the listener path in the parent’s current context and, if the pointer changed, unregisters/releases the old property and registers the new one.
- calls / called-by: parent `dataContext`, `getViewModelProperty`, `clearDataContext`, `addDependent`.
- ownership & nullability: if parent context is null, it does nothing and retains the old property. If the context exists but resolution returns null, it clears the old property.
- ordering & duplicates: pointer equality produces a complete no-op; otherwise old removal occurs before new reference/registration.
- adversarial cases: context becomes null; same property reached through a new context; unresolved path; null listener.

## `ListenerViewModelPropertyBindingInput`

### Field `m_listenerInput` (`src/animation/state_machine_instance.cpp:1307-1318`)

- role: Borrowed `ListenerInputTypeViewModel*` supplying the relink path.
- ownership & nullability: not owned and not null-checked.

### `ListenerViewModelPropertyBindingInput::ListenerViewModelPropertyBindingInput` (`src/animation/state_machine_instance.cpp:1454-1460`)

- role: Constructs the base binding and stores the view-model listener input.
- calls / called-by: base constructor / `ListenerViewModel::bindFromContext`.
- ordering & duplicates: base dependent registration precedes input-field initialization.
- adversarial cases: null listener input; duplicate authored VM input paths.

### `ListenerViewModelPropertyBindingInput::relinkDataBind` (`src/animation/state_machine_instance.cpp:1462-1479`)

- role: Same lifecycle as the single-listener relink, but resolves `m_listenerInput->dataBindPath()`.
- calls / called-by: parent `dataContext`, property resolution, clear, registration.
- ownership & nullability: null context retains old binding; unresolved property clears it; same pointer is unchanged.
- adversarial cases: duplicate inputs relinking to the same property; null input; path becomes unresolved.

## `ListenerViewModel` fields (`src/animation/state_machine_instance.cpp:1321-1399`)

- `m_stateMachineInstance` and `m_listener` are borrowed raw pointers, both initialized null before construction assignment. (`src/animation/state_machine_instance.cpp:1393-1396`)
- `m_dataContext` strongly references the current context. `m_propertyBindings` uniquely owns bindings in authored-discovery order. (`src/animation/state_machine_instance.cpp:1396-1398`)
- lifecycle: destruction body clears bindings first; after the body, the already-empty binding vector and then data-context reference are destroyed in reverse member order.

### `ListenerViewModel::ListenerViewModel` (`src/animation/state_machine_instance.cpp:1325-1328`)

- role: Stores owning state-machine and authored-listener pointers.
- once vs per-frame: construction-time for each view-model listener.
- ownership & nullability: both pointers are borrowed and unvalidated.
- adversarial cases: null listener; listener destroyed before the state-machine instance.

### `ListenerViewModel::~ListenerViewModel` (`src/animation/state_machine_instance.cpp:1490`)

- role: Clears all property bindings.
- calls / called-by: `clearDataContext` / state-machine destruction.
- lifecycle: state-machine destruction already calls `unbind`/`clearDataContext` before deleting these objects, so this is normally an idempotent second clear. (`src/animation/state_machine_instance.cpp:2160-2191`)
- adversarial cases: direct destruction while still bound.

### `ListenerViewModel::clearDataContext` (`src/animation/state_machine_instance.cpp:1330`)

- role: Clears the binding vector, destroying every binding and unregistering it.
- lifecycle: deliberately retains `m_dataContext`; only property bindings are cleared.
- ordering & duplicates: every duplicate binding is independently destroyed.
- adversarial cases: call clear then `dataContext()`—it still returns the retained context.

### `ListenerViewModel::bindFromContext` (`src/animation/state_machine_instance.cpp:1331-1373`)

- role: Stores the context, clears old bindings, then binds either one single-listener path or every `ListenerInputTypeViewModel` path.
- calls / called-by: `clearDataContext`, `getViewModelProperty`, binding constructors / `StateMachineInstance::internalDataContext`.
- ownership & nullability: dereferences `dataContext` without a null check. Missing properties are silently skipped.
- ordering & duplicates: for multi-input listeners, inputs are visited by increasing authored index; every view-model input resolving non-null creates a binding, including duplicate paths/properties.
- lifecycle: context replacement occurs before old bindings are destroyed.
- adversarial cases: null context; unresolved single path; interleaved non-VM/VM inputs; duplicate VM inputs.

### `ListenerViewModel::reportToStateMachine` (`src/animation/state_machine_instance.cpp:1374-1381`)

- role: Enqueues this listener unless the changed property is a trigger whose value equals zero.
- calls / called-by: `reportListenerViewModel` / binding `addDirt`.
- ownership & nullability: value and state-machine pointers are dereferenced without checks.
- ordering & duplicates: no deduplication; each qualifying dirt report appends another entry. (`src/animation/state_machine_instance.cpp:3021-3025`)
- queues/dirt/timing: trigger activation/non-trigger changes are processed during `applyEvents`; trigger reset-to-zero is suppressed.
- FP/zero edges: trigger comparison is `!=0`; signed zero is suppressed, while NaN would report if the property representation permits it.
- adversarial cases: trigger `0→1→0`; repeated dirt at value `1`; two bindings to one property.

### `ListenerViewModel::listener` (`src/animation/state_machine_instance.cpp:1382`)

- role: Trivial borrowed listener accessor.
- calls / called-by: none / `notifyListenerViewModels`.
- ownership & nullability: may return null; caller later dereferences without checking. (`src/animation/state_machine_instance.cpp:3048-3058`)

### `ListenerViewModel::dataContext` (`src/animation/state_machine_instance.cpp:1383-1391`)

- role: Returns the retained raw data-context pointer or null.
- calls / called-by: none / both derived relink methods.
- ownership & nullability: return is borrowed; the member `rcp` retains ownership.
- adversarial cases: bindings cleared while context remains non-null.

---

# `StateMachineInstance` event/hit definitions

### `StateMachineInstance::updateListeners` (`src/animation/state_machine_instance.cpp:1494-1545`)

- role: Normalizes frame-origin coordinates, resets all groups, prepares all hit components, processes them in hit order with opacity propagation, releases pointer state on exit, and returns the aggregate result.
- calls / called-by: listener `reset/releaseEvent`, hit `prepareEvent/processEvent` / pointer and drag entry points.
- once vs per-frame: per pointer event, not automatically per animation frame.
- ownership & nullability: unconditionally dereferences `m_artboardInstance`; vectors own hit components/groups.
- ordering & duplicates: complete reset pass precedes complete prepare pass, which precedes processing. Group and component vector order and duplicates are preserved.
- opaque/occlusion semantics: first opaque hit makes `canHit=false` for every later component, but none are skipped. Any opaque dominates any number of ordinary hits.
- queues/dirt/timing: listener effects happen synchronously during processing; resulting state-machine animation work is deferred through `markNeedsAdvance`.
- FP/zero edges: frame-origin subtraction uses unvalidated position, origin, layout width, and height. Timestamp is forwarded unchanged.
- adversarial cases: NaN layout size; one group targeting front and back shapes; opaque front target causing a back-target exit; exit of an unknown pointer ID creates then immediately pools pointer state.

### `StateMachineInstance::hitTest` (`src/animation/state_machine_instance.cpp:1547-1566`)

- role: Applies the same frame-origin coordinate adjustment and returns true on the first hit component whose raw hit test succeeds.
- calls / called-by: hit-component `hitTest` / nested artboards and component-list parents.
- ordering & duplicates: current sorted hit-component order; stops at first true.
- opaque/occlusion semantics: ignores opacity and listener consumption entirely.
- validation & malformed input: artboard is unconditionally dereferenced.
- FP/zero edges: no coordinate validation.
- adversarial cases: a geometrically hit but fully occluded target still makes this true; hidden shape with raw path; nested/list target.

### `StateMachineInstance::pointerMove` (`src/animation/state_machine_instance.cpp:1568-1573`)

- role: Dispatches `ListenerType::move` with caller timestamp and pointer ID.
- calls / called-by: `updateListeners` / public callers, `dragEnd`, nested/list forwarding.
- once vs per-frame: per move event.
- FP/zero edges: timestamp and position forwarded unchanged.
- adversarial cases: multiple pointer IDs; negative/NaN timestamp.

### `StateMachineInstance::pointerDown` (`src/animation/state_machine_instance.cpp:1574-1577`)

- role: Dispatches `ListenerType::down` with timestamp defaulting to zero.
- calls / called-by: `updateListeners` / public and nested/list forwarding.
- queues/dirt/timing: establishes click phase per listener group/pointer when hovered. (`src/listener_group.cpp:149-164`)
- adversarial cases: down outside after prior hover; duplicate authored click listeners.

### `StateMachineInstance::pointerUp` (`src/animation/state_machine_instance.cpp:1578-1581`)

- role: Dispatches `ListenerType::up` with timestamp zero.
- calls / called-by: `updateListeners` / public and nested/list forwarding.
- queues/dirt/timing: a hovered pointer whose phase is `down` becomes `clicked`; click can override the matched up listener type. (`src/listener_group.cpp:159-215`)
- adversarial cases: up without down; down on one pointer/up on another; both click and up authored.

### `StateMachineInstance::pointerExit` (`src/animation/state_machine_instance.cpp:1582-1585`)

- role: Dispatches `ListenerType::exit` with timestamp zero, then `updateListeners` releases that pointer’s group state.
- calls / called-by: `updateListeners` / public, occlusion cleanup, nested/list forwarding.
- opaque/occlusion semantics: ordinary `HitDrawable` prepares as unhovered and therefore returns none even when it fires an exit action; nested/list wrappers can forward child results.
- lifecycle: destroys active per-pointer gesture history by pooling it.
- adversarial cases: exit during down/drag; repeated exit for same pointer.

### `StateMachineInstance::dragStart` (`src/animation/state_machine_instance.cpp:1586-1597`)

- role: Optionally disables pointer events, dispatches `dragStart`, and returns that hit result.
- calls / called-by: `disablePointerEvents`, `updateListeners` / public, listener drag recognition, nested/list forwarding.
- ordering & duplicates: disabling occurs before group reset/prepare/process.
- queues/dirt/timing: the supplied timestamp is not passed to `updateListeners`; standard groups disabled by this call remain consumed through reset and are skipped. Internal drag recognition passes `disablePointer=false`, allowing drag-start listeners to run. (`src/listener_group.cpp:220-233`)
- FP/zero edges: timestamp is ignored.
- adversarial cases: default external drag start versus internal `disablePointer=false`; nested hit components whose base disable method is a no-op.

### `StateMachineInstance::dragEnd` (`src/animation/state_machine_instance.cpp:1598-1606`)

- role: Enables pointer events, dispatches `dragEnd`, then performs a move at the same position and returns only the drag-end result.
- calls / called-by: `enablePointerEvents`, `updateListeners`, `pointerMove` / public, listener drag completion, nested/list forwarding.
- ordering & duplicates: enable → drag-end event → move event.
- queues/dirt/timing: drag-end dispatch receives timestamp zero; only the follow-up move receives the supplied timestamp. Move side effects do not affect the returned result.
- adversarial cases: drag-end target differs from final move target; move returns opaque while drag-end returns none.

### `StateMachineInstance::layerState` (`src/animation/state_machine_instance.cpp:1609-1616`)

- role: Testing-only accessor returning a layer’s current shared state or null when index is out of machine layer range.
- calls / called-by: layer `currentState` / tests.
- once vs per-frame: on-demand and absent unless `TESTING`.
- validation & malformed input: bounds against `m_machine->layerCount()`, not the stored `m_layerCount`; machine and layer array are assumed valid.
- adversarial cases: machine layer count changes or disagrees with allocated array.

### `StateMachineInstance::addToHitLookup` (`src/animation/state_machine_instance.cpp:1619-1705`)

- role: Adds/reuses layout, shape, and text-run hit targets, or recursively expands a container’s children.
- calls / called-by: hit constructors, `HitDrawable::addListener`, itself / constructor’s authored and component-provided listener setup.
- once vs per-frame: construction-time only.
- ownership & nullability: target and listener group are borrowed and unconditionally dereferenced. New hit components are uniquely owned by `m_hitComponents`; `hitLookup` holds non-owning aliases.
- ordering & duplicates:
  - Layout: lookup/reuse by exact target pointer, append listener every time.
  - Shape: same, after setting `PathFlags::neverDeferUpdate` and recursively dirtying `ComponentDirt::Path` on first insertion.
  - Text run: same, after recursively dirtying the owning text component’s path and setting the run as hit target on first insertion.
  - Container: `forEachChild` order is used, the callback always returns `false`, and recursion continues through every child.
  (`src/animation/state_machine_instance.cpp:1626-1703`)
- opaque/occlusion semantics: a new layout receives `isOpaque`; a reused layout is permanently upgraded to opaque when any occurrence is opaque. Shape and text-run branches ignore `isOpaque`, including when reached through an opaque container. (`src/animation/state_machine_instance.cpp:1627-1690`)
- validation & malformed input: unsupported non-container component types are silently ignored. `isLayoutComponent=true` blindly casts target to `Drawable`. A null text run’s `textComponent()` would crash.
- lifecycle: no removal/rebuild occurs if listener targets change; draw-order sorting only reorders existing entries.
- queues/dirt/timing: first shape/text insertion dirties paths immediately during construction.
- adversarial cases:
  - unresolved authored target: no hit component, but its listener group remains owned/reset on every event. (`src/animation/state_machine_instance.cpp:1925-1944`)
  - same layout first non-opaque then opaque: reused target upgrades.
  - same shape marked opaque by provider: remains non-explicitly-opaque.
  - container containing layouts, shapes, text runs, unsupported children, and nested containers: exact child traversal must be retained.
  - same target repeated in provider targets: duplicate listener pointer is appended.
  - hidden/no-paint shape with valid raw paths: hit behavior follows raw geometry, not paint availability.

#### Container traversal lambda (`src/animation/state_machine_instance.cpp:1695-1702`)

- role: Recursively calls `addToHitLookup` for each child, deriving `isLayoutComponent` from `child->is<LayoutComponent>()`, preserving the current lookup/group/opacity, and returning `false`.
- calls / called-by: `addToHitLookup` / `ContainerComponent::forEachChild`.
- ordering & duplicates: exact `forEachChild` order; no duplicate suppression beyond the shared component lookup.
- ownership & nullability: captures the enclosing function state by reference; assumes every supplied child is non-null.
- lifecycle: lambda does not escape the call.
- adversarial cases: duplicate child pointer; layout drawable proxy child; deeply nested container; null child.