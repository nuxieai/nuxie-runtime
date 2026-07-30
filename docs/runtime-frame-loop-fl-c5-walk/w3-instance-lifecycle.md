# W3 — `state_machine_instance.cpp` lines 1707–3418 inventory

Scope result: **74 definitions** have bodies beginning at or after line 1707, including the two file-local keyframe helper functions not named in the seed list. Header context is `include/rive/animation/state_machine_instance.hpp:62-474`.

## Construction and destruction

### `StateMachineInstance::StateMachineInstance` (`src/animation/state_machine_instance.cpp:1707-2128`)

- role: constructs all per-instance inputs, layers, data binds, listener/hit structures, scripted objects, and the initial focus tree.
- calls / called-by: `buildStateKeyFrameBinds` (indirectly through layer initialization), `addToHitLookup`, `initScriptedObjects`, `sortHitComponents` / —
- once vs per-frame: construction-time.
- exact availability order:

  1. `Scene(instance)` and `m_machine` are initialized first. The constructor immediately dereferences `machine`; neither argument is validated here. (`src/animation/state_machine_instance.cpp:1707-1712`)
  2. The input vector is resized to authored input count, then typed instances are allocated in index order. Null or unsupported inputs leave null slots. Tools builds assign each successfully created instance its authored index. (`src/animation/state_machine_instance.cpp:1711-1745`)
  3. The layer array is allocated and initialized in authored layer order. Layer initialization creates the any-state instance, builds its keyframe binds, changes to the entry state, builds that state’s keyframe binds, and can fire entry-state start events/actions before ordinary state-machine data binds or listeners exist. (`src/animation/state_machine_instance.cpp:1747-1752`; `src/animation/state_machine_instance.cpp:150-175`; `src/animation/state_machine_instance.cpp:395-407`)
  4. Machine data binds are cloned in authored order. Binds with no target are skipped. Converter clones are installed, and bindable targets share one per-instance clone keyed by original pointer. (`src/animation/state_machine_instance.cpp:1754-1805`)
  5. Non-bindable transition targets receive new instance-local numeric properties. The clone is first enrolled with `addDataBind`, then retargeted; a repeated `(transition, propertyKey)` overwrites the lookup entry without deleting the prior property. New numeric properties default to `0.0f`. (`src/animation/state_machine_instance.cpp:1765-1772`; `src/animation/state_machine_instance.cpp:1806-1823`; `include/rive/generated/data_bind/bindable_property_number_base.hpp:34-44`)
  6. Authored listeners are visited in authored order. Event listeners short-circuit all other listener construction; ViewModel listeners similarly create a raw-owned `ListenerViewModel` and continue. Focus, keyboard/text, semantic, pointer, and gamepad groups otherwise may all be created for one listener. (`src/animation/state_machine_instance.cpp:1827-1967`)
  7. Focus/keyboard targets and semantic targets are validated as nodes, with only the first direct matching `FocusData`/`SemanticData` child used. Gamepad construction instead dereferences `target->as<Node>()` without a null/type guard. (`src/animation/state_machine_instance.cpp:1844-1923`; `src/animation/state_machine_instance.cpp:1945-1965`)
  8. Pointer listener groups are created even if their target is missing; hit components are deduplicated through the local component-pointer lookup. (`src/animation/state_machine_instance.cpp:1925-1944`)
  9. Component-provided groups are collected in artboard-object order, their targets are enrolled, group ownership is adopted, temporary target/group wrappers are deleted, and provider-specific hit components are appended. (`src/animation/state_machine_instance.cpp:1969-2014`)
  10. Nested-artboard hit components are appended and this instance is registered as a nested event listener on each existing nested state-machine/linear-animation notifier. Component-list hit components and optional TextInput hit targets follow. (`src/animation/state_machine_instance.cpp:2016-2070`)
  11. Scripted objects are cloned into an unordered map, each clone first receives the artboard’s current DataContext, then initialization/hydration runs in unordered-map traversal order. Script-provided keyboard/text and gamepad facilities are collected afterward. (`src/animation/state_machine_instance.cpp:2072-2120`)
  12. Hit components are sorted, then the focus tree is built with the currently selected manager and a null parent. (`src/animation/state_machine_instance.cpp:2121-2127`)

- ownership & nullability: input instances, layers, data binds, listener ViewModels, bindable-property clones, transition-property clones, and scripted clones are raw-owned; hit/listener/focus/keyboard/gamepad/semantic groups are `unique_ptr`-owned. Null inputs remain in `m_inputInstances`; later `advance` and named lookup dereference every slot. (`include/rive/animation/state_machine_instance.hpp:384-418`; `src/animation/state_machine_instance.cpp:1711-1737`; `src/animation/state_machine_instance.cpp:2576-2581`; `src/animation/state_machine_instance.cpp:2692-2697`)
- ordering & duplicates: bindable-property reuse is by original pointer; the source/target bind maps retain the last clone assigned for a property. Transition-property duplicate keys retain the last pointer and leak the overwritten property. Nested-listener registration has no deduplication. (`src/animation/state_machine_instance.cpp:1773-1804`; `src/animation/state_machine_instance.cpp:1817-1822`; `include/rive/nested_animation.hpp:28-31`)
- callback facilities: tools input indices are ready during input construction, while the callback itself defaults null and is installed externally; data-bind callbacks are not installed by construction and `onDataBindChanged` only visits binds present when later called. (`src/animation/state_machine_instance.cpp:1738-1744`; `include/rive/animation/state_machine_instance.hpp:465-473`; `src/animation/state_machine_instance.cpp:2246-2251`)
- lifecycle / partial failure: there is no rollback or exception guard. If construction throws, raw allocations already stored in vectors/maps—inputs, layer array, binds, listener ViewModels, bindable/transition properties, and scripted clones—are not reclaimed by the unentered destructor body. `unique_ptr` members do unwind. Nested notifier registrations and a partly built focus tree are external side effects with no constructor rollback. (`src/animation/state_machine_instance.cpp:1707-2128`; `src/animation/state_machine_instance.cpp:2141-2199`; `src/animation/state_machine_instance.cpp:2029-2043`)
- queues/dirt/timing: state-entry reports generated during layer initialization remain pending until a later `advance(newFrame=true)`. Shape enrollment can force path dirt immediately. (`src/animation/state_machine_instance.cpp:1747-1752`; `src/animation/state_machine_instance.cpp:400-407`; `src/animation/state_machine_instance.cpp:1657-1660`; `src/animation/state_machine_instance.cpp:2320-2335`)
- adversarial cases: null `machine`; null/unsupported authored input followed by named lookup or advance; null gamepad target; duplicate transition-property binds; constructor failure after nested-listener registration; entry listener action that expects ordinary binds/listeners already to exist.

### `StateMachineInstance::scriptedObject` (`src/animation/state_machine_instance.cpp:2130-2139`)

- role: returns the clone associated with an exact source `ScriptedObject*`.
- calls / called-by: — / external scripted consumers.
- once vs per-frame: on-demand.
- ownership & nullability: returned pointer is borrowed and owned by this instance; null or any unrecognized pointer returns null. (`src/animation/state_machine_instance.cpp:2133-2138`)
- ordering & duplicates: lookup is pointer identity, not ID/name/equality; constructor map assignment makes one visible clone per source key. (`src/animation/state_machine_instance.cpp:2072-2077`)
- lifecycle: the pointer becomes invalid when the instance deletes scripted clones. (`src/animation/state_machine_instance.cpp:2193-2198`)
- adversarial case: pass an equivalent scripted definition at a different address and require null.

### `StateMachineInstance::~StateMachineInstance` (`src/animation/state_machine_instance.cpp:2141-2199`)

- role: tears down owned runtime state, but does **not** detach this instance from nested event notifiers.
- calls / called-by: `unbind` / destruction.
- once vs per-frame: destruction-time.
- exact teardown order:

  1. Clean the focus tree only when using the internal manager; removing a focused node can synchronously queue a blur, but no later event-processing call runs. (`src/animation/state_machine_instance.cpp:2143-2149`; `src/artboard.cpp:2044-2071`; `src/input/focus_manager.cpp:226-239`; `src/animation/focus_listener_group.cpp:36-42`)
  2. Clean the semantic tree only when the semantic manager is internally owned, then clear embedder gamepads. (`src/animation/state_machine_instance.cpp:2151-2158`)
  3. `unbind`: remove this container from its current DataContext, clear listener property bindings, then unbind every state-machine data bind. (`src/animation/state_machine_instance.cpp:2160`; `src/animation/state_machine_instance.cpp:2923-2933`; `src/animation/state_machine_instance.cpp:2949-2953`; `src/data_bind/data_bind_container.cpp:16-23`)
  4. Delete inputs, reset pointer listener groups, and delete all data binds. DataBind destruction unbinds again. (`src/animation/state_machine_instance.cpp:2161-2169`; `src/data_bind/data_bind.cpp:242-249`)
  5. Clear stale keyframe-bind tracking before deleting layers and their animation instances/holders. (`src/animation/state_machine_instance.cpp:2170-2174`; `src/animation/linear_animation_instance.cpp:67-78`)
  6. Delete bindable-property clones, then transition-property clones, then listener ViewModels. (`src/animation/state_machine_instance.cpp:2175-2192`)
  7. Delete scripted clones in unordered-map order; their virtual destructor calls `scriptDispose`, which detaches inputs/tracked properties and Lua/VM registrations. (`src/animation/state_machine_instance.cpp:2193-2198`; `include/rive/scripted/scripted_object.hpp:58-60`; `src/scripted/scripted_object.cpp:484-503`)

- callback behavior: internal focus cleanup may enqueue blur but does not execute listener changes; external focus/semantic trees are left to their owner. Script deletion performs disposal bookkeeping, not state-machine event dispatch. (`src/animation/state_machine_instance.cpp:2143-2157`; `src/input/focus_manager.cpp:150-161`; `src/animation/state_machine_instance.cpp:2449-2471`)
- malformed/lifecycle hazards: because the destructor does not call `dispose`/`removeEventListeners`, higher-level instances must call `dispose` first or nested notifiers can retain a stale listener pointer. (`src/animation/state_machine_instance.cpp:2201-2206`; `include/rive/nested_animation.hpp:28-38`)
- adversarial cases: destroy without `dispose` and then report a nested event; destroy with internal focused content and verify blur actions are not executed; destroy while using external managers and verify their trees are not cleaned.

### `StateMachineInstance::dispose` (`src/animation/state_machine_instance.cpp:2201-2206`)

- role: explicitly detaches this instance from child nested animation event notifiers.
- calls / called-by: `removeEventListeners` / external higher-level lifecycle.
- once vs per-frame: disposal-time; repeatable.
- lifecycle: performs no ordinary object teardown and fires no state-machine callbacks. Repetition is safe because notifier removal erases all matching pointers. (`src/animation/state_machine_instance.cpp:2206`; `include/rive/nested_animation.hpp:32-38`)
- adversarial case: call twice, then emit a nested event and verify no notification reaches this instance.

### `StateMachineInstance::removeEventListeners` (`src/animation/state_machine_instance.cpp:2208-2243`)

- role: walks current nested artboards and removes this listener from nested state-machine and linear-animation notifiers.
- calls / called-by: — / `dispose`.
- once vs per-frame: disposal-time/on-demand.
- ownership & nullability: null artboard instance, nested artboard, animation, or notifier is skipped. (`src/animation/state_machine_instance.cpp:2210-2238`)
- ordering & duplicates: traversal follows current nested-artboard and nested-animation order; notifier removal erases all duplicate registrations. (`src/animation/state_machine_instance.cpp:2212-2240`; `include/rive/nested_animation.hpp:32-38`)
- lifecycle limitation: it can only see animations still mounted at disposal time; it does not clear this object’s own nested-listener vector. (`src/animation/state_machine_instance.cpp:2212-2241`)
- adversarial case: remove or replace a child animation before disposal and check whether its old notifier still retains the registration.

### `StateMachineInstance::onDataBindChanged` (`src/animation/state_machine_instance.cpp:2245-2253`, tools builds only)

- role: installs the same tools callback on every currently enrolled state-machine data bind.
- calls / called-by: — / external tools API.
- once vs per-frame: on-demand.
- ordering & duplicates: visits `m_dataBinds` in container order; duplicate binds each receive the callback. Binds added later, including later state keyframe binds, do not inherit it. (`src/animation/state_machine_instance.cpp:2248-2251`; `include/rive/data_bind/data_bind.hpp:136-140`)
- validation: null callback is forwarded and effectively clears each current bind’s callback; null bind pointers would be dereferenced. (`src/animation/state_machine_instance.cpp:2248-2251`)
- adversarial case: install the callback, transition into a state that creates keyframe binds afterward, and verify those new binds lack it.

## Hit ordering and state/event processing

### `StateMachineInstance::sortHitComponents` (`src/animation/state_machine_instance.cpp:2255-2304`)

- role: reorders hit components to match artboard/drawable hit-processing precedence.
- calls / called-by: — / constructor, `advance`.
- once vs per-frame: construction-time and whenever draw-order counter changes.
- ordering & duplicates: components targeting the Artboard are swapped to the front first. Remaining components are selected by walking from the drawable-chain end through `next`, with a scan-and-swap for matching component pointers. This is not a stable sort; unmatched components remain in their residual tail order. (`src/animation/state_machine_instance.cpp:2257-2303`)
- validation: null component pointers are skipped only by the Artboard-front pass and simply fail all drawable equality tests. (`src/animation/state_machine_instance.cpp:2262-2265`; `src/animation/state_machine_instance.cpp:2285-2297`)
- lifecycle/dirt: constructor sorts without synchronizing `m_drawOrderChangeCounter`; the first advance may sort again if the artboard counter is nonzero. (`src/animation/state_machine_instance.cpp:2121-2122`; `src/animation/state_machine_instance.cpp:2549-2554`)
- adversarial case: multiple Artboard targets, duplicate hit components for one drawable, and an unmatched custom component; assert exact resulting order rather than stable-sort semantics.

### `StateMachineInstance::tryChangeState` (`src/animation/state_machine_instance.cpp:2306-2318`)

- role: applies pending source-to-target bind updates, then asks every layer once whether it can change state.
- calls / called-by: `updateDataBinds` / `advanceAndApply`.
- once vs per-frame: up to five times per facade frame.
- ordering: data binds update first; layers are checked in authored order, without short-circuiting after one changes. (`src/animation/state_machine_instance.cpp:2308-2316`)
- queues/dirt: uses `updateDataBinds(false)`, so target-to-source propagation is disabled; returns only whether any layer changed. (`src/animation/state_machine_instance.cpp:2308-2317`; `src/data_bind/data_bind_container.cpp:134-152`)
- adversarial case: two layers become eligible from the same dirty binding and both must transition in one call.

### `StateMachineInstance::applyEvents` (`src/animation/state_machine_instance.cpp:2320-2344`)

- role: drains pending reported-event and ViewModel-listener batches, allowing callbacks to chain more batches.
- calls / called-by: `notifyEventListeners`, `notifyListenerViewModels` / `advance(newFrame=true)`.
- once vs per-frame: at the beginning of each new-frame advance.
- queue timing: each iteration first updates data binds, **copies** both pending queues into reporting vectors, clears both pending queues, dispatches event listeners, then dispatches the previously snapshotted ViewModel listeners. It does not swap the vectors. (`src/animation/state_machine_instance.cpp:2324-2335`)
- ordering and chaining: events always precede ViewModel notifications for a snapshot. Anything either callback reports goes to the now-empty pending vectors and is handled in the next loop iteration during the same `applyEvents` call. (`src/animation/state_machine_instance.cpp:2328-2335`; `src/animation/state_machine_instance.cpp:3016-3025`)
- cap behavior: at most 100 batches are dispatched. `currentIteration >= 100` logs even when exactly the 100th batch drained all queues; undrained pending queues survive for a later call. (`src/animation/state_machine_instance.cpp:2322-2343`)
- visibility: while a callback handles the reporting batch, `reportedEventCount/At` expose only newly chained pending events, not the current reporting batch. (`src/animation/state_machine_instance.cpp:2329-2334`; `src/animation/state_machine_instance.cpp:3027-3038`)
- adversarial cases: event callback reports another event and ViewModel change; exactly 100 finite batches; 101 chained batches; callback queries `reportedEventCount`.

## Focus and semantics

### `StateMachineInstance::setExternalFocusManager` (`src/animation/state_machine_instance.cpp:2346-2368`)

- role: switches between an external focus manager and the owned internal manager.
- calls / called-by: `focusManager` / external nesting API.
- once vs per-frame: on-demand.
- ordering/lifecycle: identical pointer is a no-op. Otherwise the old artboard focus tree is cleaned first, the external pointer is assigned, and a new root tree is built with null parent. (`src/animation/state_machine_instance.cpp:2348-2367`)
- callbacks/queues: cleaning a focused old tree can synchronously clear focus and queue blur events before the new manager is assigned. (`src/artboard.cpp:2051-2071`; `src/input/focus_manager.cpp:226-239`; `src/animation/focus_listener_group.cpp:36-42`)
- nullability: null means “return to internal manager,” not “disable focus.” (`include/rive/animation/state_machine_instance.hpp:281-293`)
- adversarial cases: switch managers while a node is focused; call again with the same manager but a desired different parent—no rebuild occurs.

### `StateMachineInstance::enableSemantics` (`src/animation/state_machine_instance.cpp:2370-2381`)

- role: lazily creates an internal semantic manager and builds the semantic tree.
- calls / called-by: `semanticManager` / external API.
- once vs per-frame: on-demand/idempotent.
- validation: if either an external or internal manager is already selected, it returns without rebuilding. (`src/animation/state_machine_instance.cpp:2372-2376`; `include/rive/animation/state_machine_instance.hpp:348-351`)
- lifecycle: with an artboard, the tree is built at manager root; null artboard merely leaves the new manager allocated. (`src/animation/state_machine_instance.cpp:2376-2380`)
- adversarial case: set an external manager first, call `enableSemantics`, and verify no internal manager/tree is created.

### `StateMachineInstance::setExternalSemanticManager` (`src/animation/state_machine_instance.cpp:2383-2407`)

- role: switches semantic tree ownership and optionally attaches the rebuilt tree beneath a parent node.
- calls / called-by: `semanticManager` / external nesting API.
- once vs per-frame: on-demand.
- ordering/lifecycle: same manager pointer is a no-op even if `parentNode` differs. Otherwise any active tree is cleaned, the external pointer changes, and the tree is rebuilt using the newly selected external or retained internal manager. (`src/animation/state_machine_instance.cpp:2387-2406`)
- nullability: null falls back to the internal manager; if none exists, `buildSemanticTree(nullptr, parentNode)` is a no-op. (`include/rive/animation/state_machine_instance.hpp:348-351`; `src/artboard.cpp:2134-2140`)
- adversarial cases: reparent with the same manager; external→null with and without an already-created internal manager.

### `StateMachineInstance::queueFocusEvent` (`src/animation/state_machine_instance.cpp:2409-2414`)

- role: appends a deferred focus/blur event and sets `m_needsAdvance`.
- calls / called-by: — / `FocusListenerGroup`.
- once vs per-frame: per focus change.
- ownership & validation: stores a borrowed group pointer with no null/lifetime check. (`src/animation/state_machine_instance.cpp:2409-2413`)
- ordering/duplicates: FIFO append; duplicates are preserved. (`src/animation/state_machine_instance.cpp:2412-2413`)
- adversarial case: queue null and advance—the later processor dereferences it.

### `StateMachineInstance::setFocus` (`src/animation/state_machine_instance.cpp:2416-2428`)

- role: focuses the supplied `FocusData` node or clears focus for null.
- calls / called-by: `focusManager` / external API.
- once vs per-frame: on-demand.
- nullability: null `FocusData` clears focus; a non-null `FocusData` whose `focusNode()` is null passes null to `FocusManager::setFocus`, which also clears current focus. (`src/animation/state_machine_instance.cpp:2418-2427`; `src/input/focus_manager.cpp:131-148`)
- queues/timing: FocusManager notifications are synchronous, but listener groups defer state-machine changes through `queueFocusEvent`. (`src/input/focus_manager.cpp:145-148`; `src/animation/focus_listener_group.cpp:27-42`)
- adversarial case: pass a `FocusData` with no created node while another node is focused and verify it behaves as a clear.

### `StateMachineInstance::focusState` (`src/animation/state_machine_instance.cpp:2430-2447`)

- role: returns cheap host-pollable focus and keyboard-expectation flags.
- calls / called-by: — / external host polling.
- once vs per-frame: on-demand.
- nullability: no primary focus returns both false. A focused node sets `hasFocus`; `expectsKeyboardInput` changes only when it has a `Focusable` that accepts keyboard input. (`src/animation/state_machine_instance.cpp:2432-2446`)
- ownership: borrows the manager’s raw primary-focus pointer without a refcount increment. (`src/animation/state_machine_instance.cpp:2433-2436`)
- adversarial case: focused scope/node without `Focusable` must report `{true,false}`.

### `StateMachineInstance::processFocusEvents` (`src/animation/state_machine_instance.cpp:2449-2473`)

- role: executes one snapshot of deferred focus/blur events.
- calls / called-by: — / `advance(newFrame=true)`.
- once vs per-frame: once at new-frame start.
- queue timing: moves the whole queue to a local vector and clears the member before callbacks. Events queued by those callbacks remain for the next new-frame call, not this pass. (`src/animation/state_machine_instance.cpp:2451-2460`)
- ordering/duplicates: preserves snapshot order and duplicates; it performs changes only when the event direction matches a listener type. (`src/animation/state_machine_instance.cpp:2459-2471`)
- validation: group and listener pointers are dereferenced without checks. (`src/animation/state_machine_instance.cpp:2461-2470`)
- adversarial case: a focus callback changes focus again; verify the chained event is deferred.

### `StateMachineInstance::queueSemanticEvent` (`src/animation/state_machine_instance.cpp:2475-2480`)

- role: appends a deferred semantic action and sets `m_needsAdvance`.
- calls / called-by: — / `SemanticListenerGroup`.
- once vs per-frame: per semantic listener match.
- ownership/ordering: borrowed group pointer; FIFO append; duplicates and nulls are retained. (`src/animation/state_machine_instance.cpp:2475-2479`)
- adversarial case: queue the same action twice and require two later invocations.

### `StateMachineInstance::processSemanticEvents` (`src/animation/state_machine_instance.cpp:2482-2507`)

- role: executes one snapshot of deferred semantic listener actions.
- calls / called-by: — / `advance(newFrame=true)`.
- once vs per-frame: once at new-frame start.
- queue timing: moves and clears before callbacks, so chained semantic actions wait for another new frame. (`src/animation/state_machine_instance.cpp:2484-2492`)
- validation: null group or null listener is skipped, unlike focus processing. (`src/animation/state_machine_instance.cpp:2492-2502`)
- ordering/duplicates: valid snapshot events execute in FIFO order with no deduplication. (`src/animation/state_machine_instance.cpp:2492-2505`)
- adversarial case: include null, valid, null-listener, valid entries and preserve the valid entries’ order.

### `StateMachineInstance::fireSemanticAction` (`src/animation/state_machine_instance.cpp:2509-2544`)

- role: resolves a semantic node ID and asks its `SemanticData` to fire tap/increase/decrease.
- calls / called-by: `semanticManager` / external accessibility API.
- once vs per-frame: per host semantic action.
- validation: missing manager, unknown ID, or structural boundary node without `SemanticData` is a silent no-op. (`src/animation/state_machine_instance.cpp:2516-2531`)
- ordering/timing: `SemanticData` visits its registered listeners in stored order; matching groups enqueue actions on their owning state machines rather than immediately performing changes. (`src/semantic/semantic_data.cpp:550-571`; `src/animation/semantic_listener_group.cpp:31-53`)
- malformed input: the enum switch has no default; an out-of-range cast performs nothing. (`src/animation/state_machine_instance.cpp:2532-2543`)
- adversarial cases: boundary ID, unknown ID, and a nested node whose listener queues on a different owning state machine.

## Advancement

### `StateMachineInstance::advance(float, bool)` (`src/animation/state_machine_instance.cpp:2546-2585`)

- role: raw state-machine advancement; it does not run the artboard facade loop.
- calls / called-by: `sortHitComponents`, `processFocusEvents`, `processSemanticEvents`, `applyEvents` / both `advanceAndApply` paths and the facade’s zero-time follow-up.
- once vs per-frame: per raw advance; may run multiple times in one facade frame.
- exact ordering:

  1. Re-sort hit components if the artboard draw-order counter changed. (`src/animation/state_machine_instance.cpp:2549-2554`)
  2. When `newFrame`, process one focus snapshot, one semantic snapshot, then drain ordinary event/ViewModel batches; afterward unconditionally clear `m_needsAdvance`. (`src/animation/state_machine_instance.cpp:2555-2561`)
  3. Update state-machine data binds with target-to-source disabled. (`src/animation/state_machine_instance.cpp:2562`)
  4. Advance layers in authored order, setting `m_needsAdvance` if any layer wants continuation. (`src/animation/state_machine_instance.cpp:2563-2569`)
  5. Advance all converters/data binds, setting `m_needsAdvance` if any reports activity. (`src/animation/state_machine_instance.cpp:2571-2574`; `src/data_bind/data_bind_container.cpp:37-51`)
  6. Call `advanced()` on every input slot; triggers are cleared even for zero seconds and even when `newFrame=false`. (`src/animation/state_machine_instance.cpp:2576-2581`; `include/rive/animation/state_machine_input_instance.hpp:104-116`)

- raw return: returns `m_needsAdvance || pending reported events || pending ViewModel reports`. Focus/semantic queues themselves are absent from the expression. (`src/animation/state_machine_instance.cpp:2583-2584`)
- `newFrame` semantics: layer “changed on advance” flags reset only on `newFrame=true`; zero-time follow-ups preserve/extend the current frame’s state-change view. With `newFrame=false`, an already-true `m_needsAdvance` is never cleared. (`src/animation/state_machine_instance.cpp:225-230`; `src/animation/state_machine_instance.cpp:2555-2568`)
- lost continuation edge: focus/semantic/event callbacks can queue a later focus/semantic event and set `m_needsAdvance`, but the unconditional clear after all three processors erases that signal; unless later layers/binds/reports continue, raw return can be false while a deferred focus/semantic event remains. (`src/animation/state_machine_instance.cpp:2409-2414`; `src/animation/state_machine_instance.cpp:2475-2480`; `src/animation/state_machine_instance.cpp:2555-2561`; `src/animation/state_machine_instance.cpp:2583-2584`)
- report timing: reports created during layer/bind advancement occur after `applyEvents`, remain pending, make the return true, and become visible to listeners on the next `newFrame=true` call. (`src/animation/state_machine_instance.cpp:2555-2574`; `src/animation/state_machine_instance.cpp:2583-2584`)
- FP/zero edges: seconds are unvalidated and forwarded to layers/converters. `+0.0f` and `-0.0f` both reach layer/data-bind code and clear triggers. NaN and infinities are likewise forwarded. Downstream linear animation treats either signed zero as zero; NaN contaminates animation time because comparisons fail; positive infinity can yield NaN in looping `fmod`, and ping-pong infinity alternates between infinities in an unbounded loop. (`src/animation/state_machine_instance.cpp:2563-2581`; `src/animation/linear_animation_instance.cpp:187-209`; `src/animation/linear_animation_instance.cpp:260-279`; `src/animation/linear_animation_instance.cpp:309-347`)
- adversarial cases: `advance(-0.0f,true)` with a fired trigger; `advance(NaN,true)` on a one-shot; `advance(+∞,true)` on loop and ping-pong animations; `newFrame=false` after a prior true continuation; chained focus during focus processing.

### `StateMachineInstance::advancedDataContext` (`src/animation/state_machine_instance.cpp:2587-2593`)

- role: tells every ViewModel instance in the current DataContext that it has advanced.
- calls / called-by: — / `reset`.
- once vs per-frame: once per facade update-loop iteration when ViewModels are enabled.
- nullability/order: null context is a no-op; otherwise `DataContext::advanced` visits all stored instances in context order without null checks. (`src/animation/state_machine_instance.cpp:2589-2592`; `src/data_bind/data_context.cpp:255-260`)
- adversarial case: force five dirty facade iterations and verify each bound ViewModel receives five `advanced()` calls.

### `StateMachineInstance::reset` (`src/animation/state_machine_instance.cpp:2595-2599`)

- role: consumes/advances the DataContext, then resets artboard resettables.
- calls / called-by: `advancedDataContext` / `advanceAndApply(..., true)`.
- once vs per-frame: once per facade update-loop iteration, not necessarily once per public frame.
- ordering: ViewModels are advanced before artboard reset; artboard reset visits resettables in stored order. (`src/animation/state_machine_instance.cpp:2597-2598`; `src/artboard.cpp:1483-1493`)
- adversarial case: a resettable observes a ViewModel’s consumed trigger and must see post-`advanced()` state.

### `StateMachineInstance::advanceAndApply(float)` (`src/animation/state_machine_instance.cpp:2601-2604`)

- role: public facade overload enabling ViewModel advancement.
- calls / called-by: `advanceAndApply(float,bool)` / Scene API.
- once vs per-frame: per facade frame.
- behavior: exactly delegates with `advanceViewModels=true`; no separate validation or return transformation. (`src/animation/state_machine_instance.cpp:2601-2604`)
- FP/zero edges: inherits all behavior of the boolean overload.
- adversarial case: compare byte-for-byte observable behavior with `advanceAndApply(seconds,true)`.

### `StateMachineInstance::advanceAndApply(float, bool)` (`src/animation/state_machine_instance.cpp:2606-2665`)

- role: full facade advancement of state machine, focus, artboard, nested animations, component dirt, and optionally ViewModels.
- calls / called-by: `advance`, `tryChangeState`, `reset` / one-argument overload and scripted/nested callers.
- once vs per-frame: per facade frame.
- ordering and `newFrame`:

  1. Call raw `advance(seconds,true)`. Facade `keepGoing` is raw result OR exact `seconds == 0.0f`. (`src/animation/state_machine_instance.cpp:2610-2612`)
  2. Drop focus if its target became hidden. This can queue blur after the raw advance. (`src/animation/state_machine_instance.cpp:2613`; `src/input/focus_manager.cpp:53-63`)
  3. Advance artboard/nested animations by `seconds` with `NewFrame`. (`src/animation/state_machine_instance.cpp:2614-2620`)
  4. Up to five times: run an artboard update pass; update binds and try state changes; if changed, run `advance(0,false)` and force `keepGoing`; advance artboard/nested animations at zero without `NewFrame`; then reset artboard and optionally ViewModels. Stop when component dirt clears. (`src/animation/state_machine_instance.cpp:2622-2656`)
  5. If enabled, advance detached scripted ViewModels once after the loop. (`src/animation/state_machine_instance.cpp:2657-2662`)

- raw vs facade return: final return is accumulated `keepGoing` OR pending event/ViewModel reports. It does not directly test `m_needsAdvance`. A blur queued by `dropFocusIfFocusTargetHidden` can therefore leave `needsAdvance()==true` while this function returns false if no later operation sets `keepGoing` or reports. (`src/animation/state_machine_instance.cpp:2612-2619`; `src/animation/state_machine_instance.cpp:2663-2664`)
- zero/signed zero: exact equality forces both `+0.0f` and `-0.0f` to return true regardless of actual work. A transition discovered in the loop receives a zero-time `newFrame=false` advance and clears triggers again. (`src/animation/state_machine_instance.cpp:2610-2612`; `src/animation/state_machine_instance.cpp:2629-2634`; `src/animation/state_machine_instance.cpp:2576-2581`)
- NaN/infinity: neither is rejected; NaN does not satisfy the forced-zero clause. Both are forwarded to state-machine, artboard, nested animation, and converter code, inheriting the contamination/nontermination cases described for raw advance. (`src/animation/state_machine_instance.cpp:2612-2617`; `src/animation/state_machine_instance.cpp:2636-2639`)
- ViewModel switch: `false` skips both bound DataContext `advanced()` calls and detached scripted ViewModel advancement, but still performs artboard reset every iteration. (`src/animation/state_machine_instance.cpp:2643-2650`; `src/animation/state_machine_instance.cpp:2657-2662`)
- dirt cap: persistent component dirt is silently abandoned after five passes; no warning or special return bit is added solely because the cap was reached. (`src/animation/state_machine_instance.cpp:2622-2656`)
- adversarial cases: idle `-0.0f`; hidden focused node with otherwise idle scene; component dirt requiring six passes; `advanceViewModels=false` with bound and detached ViewModels; infinity on ping-pong nested animation.

### `StateMachineInstance::markNeedsAdvance` (`src/animation/state_machine_instance.cpp:2667`)

- role: sets the sticky continuation flag.
- calls / called-by: — / input changes and external dependents.
- once vs per-frame: on-demand.
- queues/dirt: only writes `true`; it never clears or schedules work itself. (`src/animation/state_machine_instance.cpp:2667`)
- adversarial case: call before `advance(...,false)` and verify the flag persists.

### `StateMachineInstance::needsAdvance` (`src/animation/state_machine_instance.cpp:2668`)

- role: exposes only the `m_needsAdvance` flag.
- calls / called-by: — / external scheduler.
- once vs per-frame: on-demand.
- queue semantics: does not include pending reported events, ViewModel reports, or focus/semantic queues, unlike raw `advance`’s return for reported queues. (`src/animation/state_machine_instance.cpp:2668`; `src/animation/state_machine_instance.cpp:2583-2584`)
- adversarial case: queue only an ordinary event and verify `needsAdvance()==false` while raw return can be true.

### `StateMachineInstance::resetState` (`src/animation/state_machine_instance.cpp:2670-2676`)

- role: delegates state reset to every layer in authored order.
- calls / called-by: — / external reset API.
- once vs per-frame: on-demand.
- lifecycle: each layer removes/deletes prior state instances and changes to entry, which can rebuild keyframe binds and fire entry events. It does not reset inputs, event queues, DataContext, or `m_needsAdvance`. (`src/animation/state_machine_instance.cpp:2672-2675`; `src/animation/state_machine_instance.cpp:177-192`; `src/animation/state_machine_instance.cpp:378-408`)
- malformed state: layer reset does not explicitly clear its transition/mix fields in the shown implementation. (`src/animation/state_machine_instance.cpp:177-192`; `src/animation/state_machine_instance.cpp:696-707`)
- adversarial case: reset during an active transition and verify all non-state layer fields reproduce the C++ behavior.

## Names and inputs

### `StateMachineInstance::name` (`src/animation/state_machine_instance.cpp:2678`)

- role: returns the source machine’s name by value.
- calls / called-by: — / external API and diagnostics.
- once vs per-frame: on-demand.
- nullability: unconditionally dereferences `m_machine`. (`src/animation/state_machine_instance.cpp:2678`)
- adversarial case: malformed null source must not be silently converted to an empty name.

### `StateMachineInstance::input` (`src/animation/state_machine_instance.cpp:2680-2687`)

- role: indexed input lookup.
- calls / called-by: — / external input API.
- once vs per-frame: on-demand.
- validation: out-of-range returns null; an in-range unsupported/null-authored slot also returns its stored null. (`src/animation/state_machine_instance.cpp:2682-2686`; `src/animation/state_machine_instance.cpp:1711-1737`)
- ownership: returned pointer is borrowed.
- adversarial case: distinguish out-of-range null from an in-range null slot only through the index/count relationship.

### `StateMachineInstance::getNamedInput<SMType,InstType>` (`src/animation/state_machine_instance.cpp:2689-2701`)

- role: returns the first typed input whose source name exactly matches.
- calls / called-by: — / `getBool`, `getNumber`, `getTrigger`.
- once vs per-frame: on-demand linear scan.
- ordering/duplicates: authored input order; first same-type duplicate name wins. (`src/animation/state_machine_instance.cpp:2692-2699`)
- validation hazard: it dereferences every `inst` before checking anything; any null slot from constructor causes undefined behavior/crash before later entries can match. (`src/animation/state_machine_instance.cpp:2692-2695`; `src/animation/state_machine_instance.cpp:1715-1737`)
- adversarial case: unsupported input at index 0 and valid named input at index 1.

### `StateMachineInstance::getBool` (`src/animation/state_machine_instance.cpp:2703-2706`)

- role: typed name lookup for boolean inputs.
- calls / called-by: `getNamedInput` / external API.
- once vs per-frame: on-demand.
- validation/duplicates: exact name, first authored boolean; inherits null-slot dereference hazard. (`src/animation/state_machine_instance.cpp:2703-2705`; `src/animation/state_machine_instance.cpp:2692-2700`)
- adversarial case: same name on number then boolean—number is skipped and boolean returned.

### `StateMachineInstance::getNumber` (`src/animation/state_machine_instance.cpp:2707-2710`)

- role: typed name lookup for numeric inputs.
- calls / called-by: `getNamedInput` / external API.
- once vs per-frame: on-demand.
- validation/duplicates: exact name, first authored number; inherits null-slot dereference hazard. (`src/animation/state_machine_instance.cpp:2707-2709`; `src/animation/state_machine_instance.cpp:2692-2700`)
- adversarial case: duplicate numeric names return the first instance.

### `StateMachineInstance::getTrigger` (`src/animation/state_machine_instance.cpp:2711-2714`)

- role: typed name lookup for trigger inputs.
- calls / called-by: `getNamedInput` / external API.
- once vs per-frame: on-demand.
- validation/duplicates: exact name, first authored trigger; inherits null-slot dereference hazard. (`src/animation/state_machine_instance.cpp:2711-2713`; `src/animation/state_machine_instance.cpp:2692-2700`)
- adversarial case: unsupported/null input before a matching trigger.

## DataContext and ViewModel binding

### `StateMachineInstance::setViewModelInstance` (`src/animation/state_machine_instance.cpp:2716-2733`)

- role: stages/replaces the main ViewModel without binding paths.
- calls / called-by: — / `bindViewModelInstance`.
- once vs per-frame: on-demand.
- nullability: null is a no-op and cannot clear the current main. (`src/animation/state_machine_instance.cpp:2719-2722`)
- lifecycle: with no context, creates one and registers only this container. With an existing context, replacing the main detaches/re-attaches all containers already registered with that shared context. (`src/animation/state_machine_instance.cpp:2723-2732`; `src/data_bind/data_context.cpp:231-240`; `src/data_bind/data_context.cpp:35-57`)
- timing: does not call `bind`; path resolution remains stale until separately rebound. (`src/animation/state_machine_instance.cpp:2716-2733`; `src/animation/state_machine_instance.cpp:2776-2790`)
- adversarial case: replace the main and inspect old path bindings before and after explicit `bind()`.

### `StateMachineInstance::setGlobalViewModelInstance` (`src/animation/state_machine_instance.cpp:2735-2774`)

- role: stages/replaces one named global slot.
- calls / called-by: — / external binding API.
- once vs per-frame: on-demand.
- validation: rejects null instance, missing file, unknown/out-of-range name, null ViewModel definition, and non-global definition. (`src/animation/state_machine_instance.cpp:2739-2764`)
- replacement semantics: slot identity comes from the requested name, not the supplied instance’s own ViewModel type; cross-ViewModel overrides are accepted. Existing slot position is preserved; new globals are inserted by ascending slot key after any main. (`src/animation/state_machine_instance.cpp:2748-2752`; `src/animation/state_machine_instance.cpp:2770-2773`; `src/data_bind/data_context.cpp:166-202`)
- lifecycle: creates a null-main DataContext and registers this container if necessary, but does not bind. (`src/animation/state_machine_instance.cpp:2765-2769`)
- adversarial cases: put an instance of ViewModel B into global slot A; replace A and verify other slot order is unchanged.

### `StateMachineInstance::bind` (`src/animation/state_machine_instance.cpp:2776-2790`)

- role: completes missing ViewModels and applies the current context to artboard first, then state machine.
- calls / called-by: `completeViewModelInstances`, `internalDataContext` / `bindViewModelInstance`.
- once vs per-frame: on-demand.
- nullability: no context is a no-op. (`src/animation/state_machine_instance.cpp:2778-2781`)
- ordering: completion precedes binding; `m_artboardInstance->internalDataContext` runs before this instance’s `internalDataContext`. (`src/animation/state_machine_instance.cpp:2782-2789`)
- lifecycle: does not first clear either side and does not register the artboard as a dependent container itself. (`src/animation/state_machine_instance.cpp:2785-2789`; `src/artboard.cpp:2551-2574`)
- adversarial case: shared context with an already-bound sibling and missing defaults; verify completion mutations reach registered containers before path rebind.

### `StateMachineInstance::completeViewModelInstances` (`src/animation/state_machine_instance.cpp:2792-2829`)

- role: fills a missing main instance and every empty global slot with defaults.
- calls / called-by: — / `bind`.
- once vs per-frame: per explicit bind.
- validation: missing file is a no-op; assumes `m_DataContext` is non-null. Null default creation is simply skipped. (`src/animation/state_machine_instance.cpp:2794-2809`; `src/animation/state_machine_instance.cpp:2821-2827`)
- ordering/duplicates: missing main is inserted first. Globals are traversed in file global order, while DataContext canonicalizes new slot placement by numeric slot key. An existing cross-model override counts as occupied. (`src/animation/state_machine_instance.cpp:2799-2827`; `src/data_bind/data_context.cpp:187-202`)
- lifecycle: setters detach old and attach new instances to all currently registered dependent containers. (`src/data_bind/data_context.cpp:119-143`; `src/data_bind/data_context.cpp:166-184`; `src/data_bind/data_context.cpp:231-240`)
- adversarial case: global slot contains a different ViewModel type; completion must not replace it.

### `StateMachineInstance::bindViewModelInstance` (`src/animation/state_machine_instance.cpp:2831-2842`)

- role: convenience bind/clear operation for a main instance.
- calls / called-by: `clearDataContext`, `setViewModelInstance`, `bind` / external DataBindContainer API.
- once vs per-frame: on-demand.
- null branch: clears only this state machine’s DataContext/listener property bindings, then calls **artboard** `unbind`; it does not call this state machine’s `unbindDataBinds`. (`src/animation/state_machine_instance.cpp:2834-2838`; `src/animation/state_machine_instance.cpp:2923-2933`; `src/artboard.cpp:2604-2612`)
- non-null branch: stages the main and performs full `bind`. (`src/animation/state_machine_instance.cpp:2840-2841`)
- adversarial case: bind null and verify state-machine DataBind source references are not explicitly unbound by this path.

### `StateMachineInstance::globalViewModelInstance` (`src/animation/state_machine_instance.cpp:2844-2859`)

- role: reads the instance currently occupying a name-derived slot.
- calls / called-by: — / external API.
- once vs per-frame: on-demand.
- nullability: missing context or file returns null. (`src/animation/state_machine_instance.cpp:2849-2857`)
- validation: unlike the setter, it does not check that the name resolves in range or names a global; it directly passes `viewModelId(name)` to `instanceForSlot`. (`src/animation/state_machine_instance.cpp:2853-2858`)
- lifecycle: never creates or completes an instance. (`src/animation/state_machine_instance.cpp:2847-2858`)
- adversarial case: unknown/non-global name and a context containing unusual slot keys.

### `StateMachineInstance::bindDataContext` (`src/animation/state_machine_instance.cpp:2861-2868`)

- role: replaces local binding with the supplied context and applies it to artboard and state machine.
- calls / called-by: `clearDataContext`, `internalDataContext` / external API.
- once vs per-frame: on-demand.
- ordering: clear this state machine; register it on the new context; clear the artboard context; bind artboard; bind state machine. (`src/animation/state_machine_instance.cpp:2863-2867`)
- validation: no null guard—null is dereferenced at `addDependentContainer`. (`src/animation/state_machine_instance.cpp:2863-2865`)
- lifecycle: does not call `completeViewModelInstances`; supplied contents are used as-is. (`src/animation/state_machine_instance.cpp:2861-2868`)
- adversarial case: null context must fail rather than behaving like clear; incomplete context must remain incomplete.

### `StateMachineInstance::inheritDataContext` (`src/animation/state_machine_instance.cpp:2870-2878`)

- role: installs a shared/inherited context on the state machine only.
- calls / called-by: `internalDataContext` / external nesting API.
- once vs per-frame: on-demand.
- nullability: null is a no-op. (`src/animation/state_machine_instance.cpp:2872-2875`)
- lifecycle hazard: it does not clear/remove this container from a previous context before registering with the new one. Replacing inherited context can leave this container registered on both while `m_DataContext` points only to the latest. (`src/animation/state_machine_instance.cpp:2876-2877`; `src/animation/state_machine_instance.cpp:2923-2929`)
- scope: does not update the artboard DataContext. (`src/animation/state_machine_instance.cpp:2870-2878`)
- adversarial case: inherit A then inherit B; mutate A and verify the stale dependent registration is reproduced.

### `StateMachineInstance::dataContext(rcp<DataContext>)` (`src/animation/state_machine_instance.cpp:2880-2884`)

- role: clears current state-machine registration and directly applies another context to this instance only.
- calls / called-by: `clearDataContext`, `internalDataContext` / external API.
- once vs per-frame: on-demand.
- lifecycle: does not register this container on the new context and does not touch the artboard. (`src/animation/state_machine_instance.cpp:2882-2883`)
- null hazard: forwards null into `internalDataContext`; a ViewModel listener’s `bindFromContext` then dereferences it. (`src/animation/state_machine_instance.cpp:2901-2908`; `src/animation/state_machine_instance.cpp:1331-1338`)
- adversarial case: `dataContext(nullptr)` on machines with and without ViewModel listeners.

### `StateMachineInstance::initScriptedObjects` (`src/animation/state_machine_instance.cpp:2886-2899`)

- role: initializes and hydrates every cloned scripted object with an asset.
- calls / called-by: — / constructor, `internalDataContext`.
- once vs per-frame: construction/rebind-time.
- ordering: unordered-map traversal; no authored-order guarantee. (`src/animation/state_machine_instance.cpp:2888-2898`)
- lifecycle: if Lua user init is not done, initializes first; `hydrateScriptInputs` runs afterward on every visit regardless of init-done state. Return/failure values are ignored. (`src/animation/state_machine_instance.cpp:2890-2897`)
- nullability: null map value would be dereferenced; null script asset skips all work.
- adversarial case: two scripts with observable initialization order; hydration failure must not abort the remaining visit.

### `StateMachineInstance::internalDataContext` (`src/animation/state_machine_instance.cpp:2901-2914`)

- role: stores a context and rebuilds state-machine binds, ViewModel listeners, and scripted-object context.
- calls / called-by: `initScriptedObjects` / `bind`, `bindDataContext`, `inheritDataContext`, `dataContext`, `rebind`.
- once vs per-frame: bind/rebind-time.
- exact ordering: assign `m_DataContext`; bind all state-machine DataBindContext binds; rebuild ListenerViewModel property bindings in listener-vector order; assign context to scripted clones in unordered-map order; initialize/hydrate scripts in another unordered-map pass. (`src/animation/state_machine_instance.cpp:2903-2913`)
- nullability: no guard. Listener binding dereferences a null context; scripts accept and retain an `rcp` context value. (`src/animation/state_machine_instance.cpp:2903-2912`; `src/animation/state_machine_instance.cpp:1331-1372`; `include/rive/scripted/scripted_object.hpp:80-81`)
- keyframe lifecycle: currently enrolled keyframe DataBinds participate because `bindDataBindsFromContext` visits the full container vector. (`src/data_bind/data_bind_container.cpp:25-35`)
- adversarial case: context mutation during a script’s hydration and multiple scripts whose map iteration order is observable.

### `StateMachineInstance::rebind` (`src/animation/state_machine_instance.cpp:2916-2921`)

- role: reapplies the current context to artboard, then state machine.
- calls / called-by: `internalDataContext` / DataBindContainer dependency callbacks.
- once vs per-frame: on-demand/replacement-time.
- ordering: clear artboard context first, reapply it, then reapply state-machine binds/listeners/scripts. (`src/animation/state_machine_instance.cpp:2918-2920`)
- lifecycle: does not complete missing ViewModels and does not remove/re-add this container’s dependent registration. Artboard clearing resets its scripted init flags. (`src/artboard.cpp:2614-2628`; `src/animation/state_machine_instance.cpp:2916-2921`)
- null hazard: current null context is still passed to both internal binding paths.
- adversarial case: rebind after `clearDataContext`, with ViewModel listeners present.

### `StateMachineInstance::clearDataContext` (`src/animation/state_machine_instance.cpp:2923-2934`)

- role: removes this container from its current context and drops listener property bindings.
- calls / called-by: — / `bindViewModelInstance`, `bindDataContext`, `dataContext`, `unbind`.
- once vs per-frame: unbind/replacement-time.
- ordering: unregister and null `m_DataContext` first; then clear every ListenerViewModel’s property-binding vector. (`src/animation/state_machine_instance.cpp:2925-2933`)
- lifecycle limitation: it does not unbind state-machine DataBinds, touch artboard context, or clear scripted-object contexts. `ListenerViewModel::clearDataContext` also leaves its stored `rcp<DataContext>` intact, retaining the old context. (`src/animation/state_machine_instance.cpp:2923-2934`; `src/animation/state_machine_instance.cpp:1330-1334`; `src/animation/state_machine_instance.cpp:1393-1398`)
- adversarial case: clear and drop all external references to the old context; listener/script clones may still retain it.

### `StateMachineInstance::relinkDataContext` (`src/animation/state_machine_instance.cpp:2936-2939`)

- role: delegates relinking exclusively to the artboard.
- calls / called-by: — / DataBindContainer API.
- once vs per-frame: on-demand.
- scope: does not relink this instance’s own DataBinds, ListenerViewModels, or scripts. Artboard relink itself updates artboard hosts only. (`src/animation/state_machine_instance.cpp:2936-2939`; `src/artboard.cpp:2578-2594`)
- adversarial case: replace a nested ViewModel reference used only by a state-machine listener and call this method.

### `StateMachineInstance::rebuildDataBind` (`src/animation/state_machine_instance.cpp:2941-2947`)

- role: rebinds one supplied `DataBindContext` against the current context.
- calls / called-by: — / DataBind dependency machinery.
- once vs per-frame: on-demand.
- validation: non-context binds are ignored; null `dataBind` is dereferenced. Current null context is forwarded. (`src/animation/state_machine_instance.cpp:2943-2946`)
- scope: does not rebuild other binds, artboard binds, listeners, or scripts.
- adversarial case: pass a plain DataBind, a null pointer, and a DataBindContext after context clear.

### `StateMachineInstance::unbind` (`src/animation/state_machine_instance.cpp:2949-2953`)

- role: unregisters the DataContext/listener bindings, then unbinds all state-machine DataBinds.
- calls / called-by: `clearDataContext` / destructor.
- once vs per-frame: destruction/unbind-time.
- ordering: context removal and listener-property destruction precede DataBind source/observer/converter unbinding. (`src/animation/state_machine_instance.cpp:2951-2952`; `src/data_bind/data_bind_container.cpp:16-23`; `src/data_bind/data_bind.cpp:354-370`)
- lifecycle limitation: does not unbind the artboard and does not clear stored contexts on listener/script objects. (`src/animation/state_machine_instance.cpp:2949-2953`; `src/animation/state_machine_instance.cpp:1330-1334`)
- adversarial case: explicitly exercise destructor ordering with a DataBind source also referenced by a ListenerViewModel binding.

## State and animation queries

### `StateMachineInstance::stateChangedCount` (`src/animation/state_machine_instance.cpp:2955-2966`)

- role: counts layers whose per-frame changed flag is set.
- calls / called-by: — / external query API.
- once vs per-frame: on-demand after advancement.
- ordering/duplicates: counts each authored layer independently. The flag spans the latest `newFrame=true` advance plus any later `newFrame=false` follow-ups. (`src/animation/state_machine_instance.cpp:2957-2965`; `src/animation/state_machine_instance.cpp:225-230`)
- adversarial case: one public facade call causes several zero-time transitions in one layer; count remains per layer, not per transition.

### `StateMachineInstance::stateChangedByIndex` (`src/animation/state_machine_instance.cpp:2968-2983`)

- role: returns the current state of the Nth changed layer.
- calls / called-by: — / external query API.
- once vs per-frame: on-demand.
- ordering: compacted authored-layer order, not raw layer index; out-of-range returns null. (`src/animation/state_machine_instance.cpp:2970-2982`)
- ownership: borrowed `LayerState*`.
- adversarial case: layers 0 and 2 changed; index 1 must return layer 2’s current state.

### `StateMachineInstance::currentAnimationCount` (`src/animation/state_machine_instance.cpp:2985-2996`)

- role: counts layers whose current state is an animation state.
- calls / called-by: — / external query API.
- once vs per-frame: on-demand.
- ordering/duplicates: one count per layer; identical animation instances/pointers across layers are not deduplicated. (`src/animation/state_machine_instance.cpp:2987-2995`; `src/animation/state_machine_instance.cpp:675-684`)
- adversarial case: two layers drive the same source animation and count must be two.

### `StateMachineInstance::currentAnimationByIndex` (`src/animation/state_machine_instance.cpp:2998-3014`)

- role: returns the Nth non-null current animation instance.
- calls / called-by: — / external query API.
- once vs per-frame: on-demand.
- ordering: compacted authored-layer order; out-of-range returns null. It calls `currentAnimation()` twice for the selected layer. (`src/animation/state_machine_instance.cpp:3001-3013`)
- ownership: borrowed pointer owned by the layer’s current state.
- adversarial case: non-animation layers interleaved between animation layers.

## Event reporting and dispatch

### `StateMachineInstance::reportEvent` (`src/animation/state_machine_instance.cpp:3016-3019`)

- role: appends an event report to the pending queue.
- calls / called-by: — / state/transition/keyed callbacks.
- once vs per-frame: per report.
- ownership/nullability: stores the raw event pointer in `EventReport`; no null check or ownership transfer. (`src/animation/state_machine_instance.cpp:3016-3018`)
- ordering/duplicates: FIFO append, duplicates preserved.
- FP/zero edges: delay is stored exactly; negative, signed zero, NaN, and infinity are neither delayed nor validated here. (`src/animation/state_machine_instance.cpp:3016-3018`)
- adversarial case: duplicate event reports with NaN and `-0.0f` delays.

### `StateMachineInstance::reportListenerViewModel` (`src/animation/state_machine_instance.cpp:3021-3025`)

- role: appends a pending ViewModel-listener notification.
- calls / called-by: — / `ListenerViewModel` property bindings.
- once vs per-frame: per observed change.
- ownership/nullability: borrowed raw pointer, no null check; duplicates preserved. (`src/animation/state_machine_instance.cpp:3021-3024`)
- adversarial case: report the same listener twice before advance and require two invocations.

### `StateMachineInstance::reportedEventCount` (`src/animation/state_machine_instance.cpp:3027-3030`)

- role: returns pending-event count.
- calls / called-by: — / external polling API.
- once vs per-frame: on-demand.
- queue visibility: excludes the reporting snapshot currently being dispatched and all already-drained reports. (`src/animation/state_machine_instance.cpp:3029`; `src/animation/state_machine_instance.cpp:2329-2333`)
- adversarial case: query from inside an event callback that has just chained one new report.

### `StateMachineInstance::reportedEventAt` (`src/animation/state_machine_instance.cpp:3032-3039`)

- role: returns a pending report by value.
- calls / called-by: — / external polling API.
- once vs per-frame: on-demand.
- validation: out-of-range returns sentinel `EventReport(nullptr,0.0f)`. (`src/animation/state_machine_instance.cpp:3034-3038`)
- queue visibility: indexes only the pending queue, not `m_reportingEvents`.
- adversarial case: index equal to count and verify exact null/positive-zero sentinel.

### `StateMachineInstance::notify` (`src/animation/state_machine_instance.cpp:3041-3046`)

- role: synchronously handles a nested notifier’s event batch, then updates data binds.
- calls / called-by: `notifyEventListeners` / child `NestedEventNotifier`.
- once vs per-frame: per nested notification.
- ordering: all local listener dispatch, upward bubbling, and audio playback inside `notifyEventListeners` complete before `updateDataBinds(false)`. (`src/animation/state_machine_instance.cpp:3044-3045`; `src/animation/state_machine_instance.cpp:3155-3169`)
- queue timing: this path is immediate and does not enqueue into `m_reportedEvents`.
- adversarial case: nested event action dirties a bind needed by the parent’s parent; bubbling occurs before this parent’s final bind update.

### `StateMachineInstance::notifyListenerViewModels` (`src/animation/state_machine_instance.cpp:3048-3060`)

- role: performs changes for every ViewModel-listener pointer in the supplied batch.
- calls / called-by: — / `applyEvents`.
- once vs per-frame: per batch.
- ordering/duplicates: vector order, duplicates preserved. (`src/animation/state_machine_instance.cpp:3051-3058`)
- validation: empty is a no-op; null listener pointer is dereferenced. (`src/animation/state_machine_instance.cpp:3051-3057`)
- queues/timing: changes may report more events/listeners into pending vectors for the next `applyEvents` iteration.
- adversarial case: first ViewModel listener reports a second listener; the new one must not join the currently iterated snapshot.

### `StateMachineInstance::notifyEventListeners` (`src/animation/state_machine_instance.cpp:3062-3171`)

- role: matches an event batch against authored event listeners, bubbles it upward, then plays audio events.
- calls / called-by: — / `applyEvents`, `notify`.
- once vs per-frame: per event batch.
- exact ordering:

  1. Authored listeners outermost, reported events innermost. (`src/animation/state_machine_instance.cpp:3068-3077`)
  2. Listener target is resolved before the later `listener != nullptr` check, so a null listener pointer is already dereferenced. (`src/animation/state_machine_instance.cpp:3069-3075`)
  3. Local-event dispatch validates old-file target ambiguity before matching. (`src/animation/state_machine_instance.cpp:3079-3107`)
  4. A `StateMachineListenerSingle` handles at most the first matching event in the batch because a match breaks the event loop. (`src/animation/state_machine_instance.cpp:3109-3123`)
  5. A multi-input listener scans input types for each event; the first matching event-input entry wins for that event, but later events are still considered. (`src/animation/state_machine_instance.cpp:3124-3151`)
  6. After all local listeners, the entire batch bubbles immediately to nested listeners in their stored order. (`src/animation/state_machine_instance.cpp:3155-3160`; `include/rive/nested_animation.hpp:39-42`)
  7. Audio events play only after upward bubbling completes. (`src/animation/state_machine_instance.cpp:3162-3169`)

- source filtering: for nested notifications, an event listener is eligible only when `source == resolved target`; source artboard resolution then uses the nested artboard instance. (`src/animation/state_machine_instance.cpp:3072-3081`)
- duplicates: duplicate reports remain observable except for the single-listener early break. Duplicate event input definitions do not cause duplicate invocation for one report. (`src/animation/state_machine_instance.cpp:3077-3151`)
- ownership/nullability: event pointers are unconditionally dereferenced during the audio pass; null reports can crash. (`src/animation/state_machine_instance.cpp:3162-3167`)
- chained completion: actions can append pending reports; `applyEvents` catches them in a same-call later batch, whereas direct nested `notify` does not itself loop over newly reported queues. (`src/animation/state_machine_instance.cpp:2324-2335`; `src/animation/state_machine_instance.cpp:3041-3046`)
- propagation edge: every ancestor’s `notifyEventListeners` runs its own final audio pass, so a nested `AudioEvent` can be played once per propagation level. (`src/animation/state_machine_instance.cpp:3155-3169`)
- adversarial cases: batch `[A,A]` against single and multi-input listeners; null listener/event; nested target mismatch; nested AudioEvent with two ancestors; listener action that reports another event.

## Pointer enablement and bindable lookups

### `StateMachineInstance::enablePointerEvents` (`src/animation/state_machine_instance.cpp:3173-3179`)

- role: enables the given pointer ID on every hit component.
- calls / called-by: — / `dragEnd` and external API.
- once vs per-frame: on-demand.
- ordering/duplicates: current hit-component order; duplicates all receive the call. (`src/animation/state_machine_instance.cpp:3175-3178`)
- validation: pointer ID is unvalidated and forwarded unchanged.
- adversarial case: negative pointer ID with multiple groups sharing one hit target.

### `StateMachineInstance::disablePointerEvents` (`src/animation/state_machine_instance.cpp:3181-3187`)

- role: disables the given pointer ID on every hit component.
- calls / called-by: — / `dragStart` and external API.
- once vs per-frame: on-demand.
- ordering/duplicates: current hit-component order; duplicates all receive the call. (`src/animation/state_machine_instance.cpp:3183-3186`)
- validation: pointer ID is unvalidated and forwarded unchanged.
- adversarial case: disable twice, then enable once, preserving listener-group implementation semantics.

### `StateMachineInstance::bindablePropertyInstance` (`src/animation/state_machine_instance.cpp:3189-3199`)

- role: maps a source bindable property pointer to its per-instance clone.
- calls / called-by: — / external/listener action binding.
- once vs per-frame: on-demand.
- identity/nullability: exact pointer-key lookup; missing or ordinary null key returns null unless such a key was explicitly inserted. Returned pointer is borrowed. (`src/animation/state_machine_instance.cpp:3192-3198`)
- lifecycle: invalid after destruction deletes clone values. (`src/animation/state_machine_instance.cpp:2175-2179`)
- adversarial case: structurally identical property at a different address.

### `StateMachineInstance::bindableDataBindToSource` (`src/animation/state_machine_instance.cpp:3201-3210`)

- role: retrieves the registered target-to-source bind for a per-instance bindable property.
- calls / called-by: — / listener actions.
- once vs per-frame: on-demand.
- identity/duplicates: exact clone-pointer key; constructor assignment means the last duplicate bind wins. (`src/animation/state_machine_instance.cpp:3204-3209`; `src/animation/state_machine_instance.cpp:1791-1799`)
- ownership: borrowed DataBind owned by the container.
- adversarial case: two ToSource binds target the same property; lookup returns the later clone.

### `StateMachineInstance::bindableDataBindToTarget` (`src/animation/state_machine_instance.cpp:3212-3221`)

- role: retrieves the registered source-to-target bind for a per-instance bindable property.
- calls / called-by: — / transition/listener binding consumers.
- once vs per-frame: on-demand.
- identity/duplicates: exact clone-pointer key; last duplicate non-ToSource bind wins. (`src/animation/state_machine_instance.cpp:3215-3220`; `src/animation/state_machine_instance.cpp:1800-1804`)
- ownership: borrowed DataBind owned by the container.
- adversarial case: two ToTarget/TwoWay candidates for one property and verify overwrite order.

### `StateMachineInstance::findTransitionPropertyInstance` (`src/animation/state_machine_instance.cpp:3223-3237`)

- role: resolves a per-instance numeric transition property by transition pointer and original property key.
- calls / called-by: — / layer transition setup.
- once vs per-frame: on transition selection.
- identity/nullability: two-level exact-key lookup; either miss returns null. Borrowed pointer. (`src/animation/state_machine_instance.cpp:3227-3236`)
- lifecycle: constructor duplicate keys overwrite the lookup pointer; destructor deletes only values still represented in the map. (`src/animation/state_machine_instance.cpp:1817-1822`; `src/animation/state_machine_instance.cpp:2180-2187`)
- adversarial case: duplicate binds for one transition duration and assert last lookup plus leaked/unreferenced earlier property behavior.

## Keyframe DataBind lifecycle

### file-local `keyFrameHolderPropertyKey` (`src/animation/state_machine_instance.cpp:3239-3256`)

- role: maps supported keyframe core types to the holder’s `propertyValue` key.
- calls / called-by: — / `buildStateKeyFrameBinds`.
- once vs per-frame: per examined keyframe during state-instance construction.
- validation: number, color, boolean, and string are supported; every other type returns zero and is treated as unbound. (`src/animation/state_machine_instance.cpp:3241-3255`)
- adversarial case: ID/uint/custom keyframe type must return zero without fallback conversion.

### file-local `makeKeyFrameValueHolder` (`src/animation/state_machine_instance.cpp:3258-3274`)

- role: allocates a type-matched bindable holder for a supported keyframe type.
- calls / called-by: — / `buildStateKeyFrameBinds`.
- once vs per-frame: per bound keyframe/state/animation instance.
- ownership/nullability: returns raw new allocation for number/color/bool/string; unsupported returns null. Ownership is transferred to the `LinearAnimationInstance`. (`src/animation/state_machine_instance.cpp:3259-3273`; `src/animation/linear_animation_instance.cpp:67-89`)
- adversarial case: every supported type maps to its exact holder class; unsupported maps null.

### `StateMachineInstance::buildStateKeyFrameBinds` (`src/animation/state_machine_instance.cpp:3276-3374`)

- role: builds live per-animation-instance holder bindings for source-artboard keyframes used by a state instance.
- calls / called-by: `keyFrameHolderPropertyKey`, `makeKeyFrameValueHolder` / layer initialization and state changes.
- once vs per-frame: state-instance creation-time.
- validation: null state/artboard/source artboard returns. Source binds with null or non-keyframe targets are skipped; if none remain, animation/keyframe traversal is skipped entirely. (`src/animation/state_machine_instance.cpp:3278-3311`)
- duplicate handling: source binds are indexed by target with `unordered_map::emplace`, so the first encountered bind per shared keyframe wins and later duplicates are ignored. (`src/animation/state_machine_instance.cpp:3290-3306`)
- traversal order: state-provided animation-instance order, animation keyed-object order, keyed-property order, then keyframe order. Null animation, keyed object, or keyed property is skipped. A null keyframe itself is not checked before `coreType()`. (`src/animation/state_machine_instance.cpp:3313-3341`)
- enrollment sequence for each match: determine supported holder property; allocate holder; install it on the animation instance; clone bind; copy file; retarget/property-key it; call `initialize`; only then clone/install converter; add to this container; append the raw pointer to the state tracking vector. (`src/animation/state_machine_instance.cpp:3338-3369`)
- live resolution: keyframes ask their current `LinearAnimationInstance` for the holder and use its property value instead of serialized value. Number/color/bool/string all fall back to serialized value when no holder exists. (`src/animation/keyframe_double.cpp:29-40`; `src/animation/keyframe_color.cpp:23-33`; `src/animation/keyframe_bool.cpp:8-18`; `src/animation/keyframe_string.cpp:8-19`)
- advancement: `addDataBind` enrolls the clone in the normal container. If a DataContext is already present, it binds and updates immediately; otherwise later `internalDataContext` binds it. Converter advancement happens with every raw state-machine advance, after layer application, and can request another frame. (`src/data_bind/data_bind_container.cpp:86-113`; `src/animation/state_machine_instance.cpp:2562-2574`)
- lifecycle/duplicates: no “already built for this state” guard exists. Calling twice can overwrite a holder entry in the animation instance, leak the old holder, and retain multiple DataBind clones in state tracking. (`src/animation/state_machine_instance.cpp:3351-3369`; `src/animation/linear_animation_instance.cpp:81-89`)
- malformed/reentrant edge: holder creation precedes clone/add completion, so a later allocation/clone failure leaves the animation instance holding a partially established holder. (`src/animation/state_machine_instance.cpp:3351-3368`)
- adversarial cases: duplicate source binds; unsupported keyframe; null keyframe; build after context binding; call build twice for one state; converter whose initialization order relative to `initialize()` is observable.

### `StateMachineInstance::removeStateKeyFrameBinds` (`src/animation/state_machine_instance.cpp:3376-3390`)

- role: removes and deletes all tracked keyframe binds for one state before its animation holders are destroyed.
- calls / called-by: — / layer state reset/change.
- once vs per-frame: state-instance teardown-time.
- ordering: tracking miss is a no-op. Otherwise each bind is removed from the container then immediately deleted, in build order; the state map entry is erased last. (`src/animation/state_machine_instance.cpp:3379-3389`)
- lifecycle: normal layer reset/change calls this before deleting the state/animation instance. Destructor instead bulk-deletes all binds, clears tracking, then deletes layers/holders. (`src/animation/state_machine_instance.cpp:177-191`; `src/animation/state_machine_instance.cpp:573-579`; `src/animation/state_machine_instance.cpp:2169-2174`)
- reentrancy hazard: `DataBindContainer::removeDataBind` defers removal while it is processing, but this method still deletes the bind immediately; a reentrant call during `updateDataBinds` leaves active/pending container pointers dangling. (`src/animation/state_machine_instance.cpp:3384-3388`; `src/data_bind/data_bind_container.cpp:54-63`)
- adversarial cases: remove unknown state; remove during data-bind callback; destructor with active keyframe binds; preserve removal order for multiple binds.

## Focus convenience methods

### `StateMachineInstance::hasFocusNodes` (`src/animation/state_machine_instance.cpp:3392-3397`)

- role: reports whether the selected focus manager contains focusable content.
- calls / called-by: `focusManager` / external API.
- once vs per-frame: on-demand.
- nullability: asserts non-null, but `focusManager()` already always returns either external or owned internal manager. (`include/rive/animation/state_machine_instance.hpp:281-293`; `src/animation/state_machine_instance.cpp:3394-3396`)
- adversarial case: no built nodes returns manager result, not manager existence.

### `StateMachineInstance::focusNext` (`src/animation/state_machine_instance.cpp:3399-3404`)

- role: delegates forward focus traversal.
- calls / called-by: `focusManager` / external keyboard/gamepad API.
- once vs per-frame: on-demand.
- timing: FocusManager first drops hidden focus, then searches/moves to the next eligible node; focus/blur changes queue deferred state-machine actions. (`src/animation/state_machine_instance.cpp:3401-3403`; `src/input/focus_manager.cpp:285-289`; `src/animation/focus_listener_group.cpp:27-42`)
- adversarial case: current focused target becomes hidden immediately before traversal.

### `StateMachineInstance::focusPrevious` (`src/animation/state_machine_instance.cpp:3406-3411`)

- role: delegates reverse focus traversal.
- calls / called-by: `focusManager` / external keyboard/gamepad API.
- once vs per-frame: on-demand.
- timing: FocusManager drops hidden focus first, then searches backward; callbacks are deferred through queued focus events. (`src/animation/state_machine_instance.cpp:3408-3410`; `src/input/focus_manager.cpp:291-295`; `src/animation/focus_listener_group.cpp:27-42`)
- adversarial case: traversal from no primary focus with several eligible roots.

### `StateMachineInstance::clearFocus` (`src/animation/state_machine_instance.cpp:3413-3418`)

- role: delegates clearing primary focus.
- calls / called-by: `focusManager` / external API.
- once vs per-frame: on-demand.
- ordering/timing: FocusManager nulls/moves out primary focus before synchronous notification, so `hasFocus` is already false inside blur callbacks; state-machine listener changes remain deferred. (`src/animation/state_machine_instance.cpp:3415-3417`; `src/input/focus_manager.cpp:150-161`; `src/animation/focus_listener_group.cpp:36-42`)
- nullability: assertion is redundant because the owned internal manager always exists. (`include/rive/animation/state_machine_instance.hpp:281-293`)
- adversarial case: call twice; only the first call can produce blur notification.