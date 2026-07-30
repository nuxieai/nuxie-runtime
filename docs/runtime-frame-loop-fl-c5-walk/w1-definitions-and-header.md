# W1 — StateMachine definition and header-surface inventory

All citations are relative to the pinned C++ oracle at `/Users/levi/dev/oss/rive-runtime`.

## 1. `StateMachine`

### Class and fields

`StateMachine` derives from `StateMachineBase`; `StateMachineImporter` is its friend and is the only class able to call the four private owning add methods. `include/rive/animation/state_machine.hpp:15-29`

| Member | Declaration | Contract |
|---|---:|---|
| `m_Layers` | `state_machine.hpp:20` | Owns `StateMachineLayer` definitions through `unique_ptr`; default-empty, authored-order vector. Destroyed automatically with the machine. |
| `m_Inputs` | `state_machine.hpp:21` | Owns input definitions through `unique_ptr`; may contain a null entry inserted by `StateMachineImporter::readNullObject`. `state_machine_importer.cpp:35-39` |
| `m_Listeners` | `state_machine.hpp:22` | Owns listener definitions through `unique_ptr`; default-empty, authored-order vector. |
| `m_dataBinds` | `state_machine.hpp:23` | Owns state-machine-targeted `DataBind` definitions; default-empty, authored-order vector. |
| `m_scriptedObjects` | `state_machine.hpp:24` | Non-owning raw pointers to scripted transition conditions/actions; default-empty and returned by value from `scriptedObjects()`. |
| `addLayer` | `state_machine.hpp:26`; `state_machine.cpp:82-85` | Construction-time append; transfers ownership and retains duplicates/order. Called by `StateMachineImporter::addLayer`. Null is not rejected. |
| `addInput` | `state_machine.hpp:27`; `state_machine.cpp:87-90` | Construction-time append; transfers ownership and retains duplicates/order. Called by importer and by the null-object compatibility path. |
| `addListener` | `state_machine.hpp:28`; `state_machine.cpp:92-95` | Construction-time append; transfers ownership and retains duplicates/order. No null/type validation. |
| `addDataBind` | `state_machine.hpp:29`; `state_machine.cpp:97-100` | Construction-time append; transfers ownership and retains duplicates/order. No null validation. |

Adversarial tests for the fields/adders: duplicate names and duplicate pointers must remain in insertion order; a serialized unknown object consumed by `readNullObject()` must create an input hole, not be silently removed; inserting a null input must expose the C++ crash behavior during `onAddedDirty`, described below.

### Public member inventory

#### `StateMachine::StateMachine` (`state_machine.hpp:32`; `state_machine.cpp:12`)

- role: trivial construction; all vector members are default-empty.
- calls / called-by: no family calls.
- once vs per-frame: construction-time.
- ownership: does not preallocate or synthesize layers, inputs, listeners, binds, or scripts.
- validation: none.
- lifecycle: source definitions are later referenced by `StateMachineInstance`; the machine itself is not copied here.
- adversarial case: a zero-member machine remains valid at this level.

#### `StateMachine::~StateMachine` (`state_machine.hpp:33`; `state_machine.cpp:14`)

- role: empty explicit destructor.
- lifecycle: automatic vector destruction deletes layers, inputs, listeners, and data binds; `m_scriptedObjects` pointers are not deleted.
- adversarial case: the same object must not be independently owned by two owning collections.

#### `StateMachine::import` (`state_machine.hpp:35`; `state_machine.cpp:70-80`)

- role: attaches the machine to the current artboard importer, then delegates to `StateMachineBase::import`.
- calls / called-by: calls `ArtboardImporter::addStateMachine`, `Super::import`; called during sequential file import.
- once vs per-frame: import-time.
- ownership/nullability: if no current `ArtboardImporter` exists, returns `MissingObject`; otherwise the artboard receives the raw machine pointer. `state_machine.cpp:72-79`
- ordering: artboards append machines in parse order. `artboard_importer.cpp:24-27`
- validation: does not inspect member collections.
- adversarial case: a top-level state machine without an artboard importer must return `MissingObject` and not be attached.

#### Count accessors (`state_machine.hpp:37-40`)

- `layerCount`, `inputCount`, `listenerCount`, and `dataBindCount` are inline, on-demand vector sizes.
- Null entries still count. No overflow/out-of-range behavior is involved beyond `size_t`.
- Adversarial case: a single null compatibility input produces `inputCount() == 1`.

#### `StateMachine::addScriptedObject` (`state_machine.hpp:41`; `state_machine.cpp:162-165`)

- role: appends a borrowed scripted object pointer.
- calls / called-by: called through `StateMachineImporter::addScriptedObject`; importers for `ScriptedTransitionCondition` and `ScriptedListenerAction` are the concrete producers. `scripted_transition_condition.cpp:93-107`; `scripted_listener_action.cpp:125-140`
- once vs per-frame: import-time.
- ownership/nullability: does not own, clone, or reject the pointer.
- ordering/duplicates: parse order retained; duplicates retained.
- lifecycle: instances clone every entry later; the source object must remain alive with the source file. `state_machine_instance.cpp:2072-2081`
- adversarial case: two references to the same source pointer create two append entries but the instance’s pointer-keyed map ultimately holds one clone entry.

#### `StateMachine::scriptedObjects` (`state_machine.hpp:42-45`)

- role: returns a copy of the raw-pointer vector.
- timing: on-demand.
- ownership: callers receive no ownership; modifying the returned vector does not modify the machine.
- ordering/duplicates: preserves the source vector’s order and duplicates.
- adversarial case: clearing the returned vector must not clear `m_scriptedObjects`.

#### `StateMachine::input(std::string)` (`state_machine.hpp:47`; `state_machine.cpp:102-112`)

- role: exact-name lookup.
- timing: on-demand linear scan.
- nullability: returns the first matching input or `nullptr`; a null collection entry is dereferenced and crashes before later entries can match.
- ordering/duplicates: first authored duplicate wins.
- validation: case-sensitive `std::string` equality; no normalization.
- adversarial cases: duplicate names return index 0; absent name returns null; a leading null slot followed by a match reproduces the null dereference.

#### `StateMachine::input(size_t)` (`state_machine.hpp:48`; `state_machine.cpp:114-121`)

- role: authored-index lookup.
- nullability: returns the stored pointer when `index < size`, including null; otherwise returns `nullptr`.
- adversarial cases: `index == size`, `SIZE_MAX`, and an in-range null slot all return null.

#### `StateMachine::layer(std::string)` (`state_machine.hpp:49`; `state_machine.cpp:123-133`)

- role: exact-name, first-match lookup in authored order.
- nullability: not found returns `nullptr`; a null layer entry would be dereferenced.
- adversarial cases: duplicate names select the first; case-only differences do not match.

#### `StateMachine::layer(size_t)` (`state_machine.hpp:50`; `state_machine.cpp:135-142`)

- role: authored-index lookup.
- nullability: out of range returns `nullptr`; an in-range null remains null.
- adversarial cases: zero-layer lookup and `SIZE_MAX` return null.

#### `StateMachine::dataBind(size_t)` (`state_machine.hpp:51`; `state_machine.cpp:153-160`)

- role: authored-index lookup.
- nullability: out of range returns `nullptr`; no name lookup exists.
- ordering/duplicates: retains import order and duplicates.
- adversarial case: `index == dataBindCount()` returns null.

#### `StateMachine::listener(size_t)` (`state_machine.hpp:52`; `state_machine.cpp:144-151`)

- role: authored-index lookup.
- nullability: out of range returns `nullptr`.
- ordering/duplicates: retains import order and duplicates.
- adversarial case: `SIZE_MAX` returns null.

#### `StateMachine::onAddedDirty` (`state_machine.hpp:54`; `state_machine.cpp:16-41`)

- role: performs first-phase child initialization in the fixed order inputs, layers, listeners.
- calls / called-by: calls each child’s `onAddedDirty`; called from source-artboard initialization. `artboard.cpp:290-312`
- once vs per-frame: source import/initialization only.
- ordering: fully processes each collection in authored order before moving to the next.
- validation/failure: returns immediately on the first non-`Ok` status; there is no rollback. Data binds and scripted objects are not visited.
- malformed input: every pointer is dereferenced without a null check. The null input deliberately inserted by `readNullObject()` therefore causes undefined behavior/crash here. `state_machine.cpp:19-35`; `state_machine_importer.cpp:35-39`
- adversarial cases: failure in input 2 prevents all layers/listeners; a null input crashes; malformed layer missing its `Any`, `Entry`, or `Exit` state returns `InvalidObject`. `state_machine_layer.cpp:19-47`

#### `StateMachine::onAddedClean` (`state_machine.hpp:55`; `state_machine.cpp:43-68`)

- role: second-phase initialization in the same inputs, layers, listeners order.
- validation/failure: immediate first-error return, no rollback; binds/scripts omitted.
- lifecycle: source artboard only, avoiding duplicate list population in artboard instances. `artboard.cpp:398-415`
- adversarial case: a layer clean-phase failure prevents later layers and all listeners.

FP/zero edges for every `StateMachine` member: n/a; no floating-point data flows through this class’s own members.

## 2. Definition collections and importer handoff

### Collection population

| Collection | Producer and handoff | Ordering and duplicate contract |
|---|---|---|
| Layers | `StateMachineLayer::import` requires the current `StateMachineImporter`, transfers `this` into a `unique_ptr`, and the importer forwards to `StateMachine::addLayer`. `state_machine_layer.cpp:69-79`; `state_machine_importer.cpp:14-17` | Sequential file parse plus `push_back` gives authored order. Duplicates are retained. |
| Inputs | `StateMachineInput::import` performs the same ownership transfer. `state_machine_input.cpp:18-28`; `state_machine_importer.cpp:19-22` | Authored order retained. Unsupported/null serialized objects may add a null input hole. |
| Listeners | `StateMachineListener::import` transfers ownership to the machine importer. `state_machine_listener.cpp:52-63`; `state_machine_importer.cpp:24-28` | Authored order retained and later controls event-listener execution order. `state_machine_instance.cpp:3068-3077` |
| Data binds | A bind whose target is one of the bindable-property, transition-comparator, or state-transition types is transferred to the current state-machine importer. `data_bind.cpp:104-127` | Parse order retained. Multiple binds are retained; instance-side lookup maps may overwrite earlier map entries while the cloned binds themselves remain. |
| Scripted objects | `ScriptedTransitionCondition` and `ScriptedListenerAction` register borrowed pointers through the importer. `scripted_transition_condition.cpp:93-107`; `scripted_listener_action.cpp:125-140` | Parse order and duplicates retained in the source vector; no ownership transfer. |
| No other definition collection | The five vectors in `state_machine.hpp:20-24` are the complete `StateMachine`-owned/recorded collection set. | — |

The file reader processes objects serially and invokes `object->import(importStack)` before advancing to the next runtime object, establishing the authored-order guarantee. `file.cpp:299-326`

### Import and malformed-input outcomes

- `StateMachineImporter::resolve()` always returns `Ok`; it performs no collection validation. `state_machine_importer.cpp:47`
- `StateMachineImporter::readNullObject()` assumes an unknown object represents a new input type, appends a null input, and returns `true`. Because null-object dispatch walks active importers newest-first, whichever importer first consumes the null determines its meaning. `state_machine_importer.cpp:35-40`; `import_stack.hpp:93-103`
- Missing `StateMachineImporter` causes layer/input/listener imports to return `MissingObject` without transferring ownership. `state_machine_layer.cpp:71-79`; `state_machine_input.cpp:20-28`; `state_machine_listener.cpp:54-63`
- Layer importer resolution validates transition destination indices and returns `InvalidObject` for an out-of-range state index; unknown state types are replaced with an inert generic `LayerState`. `state_machine_layer_importer.cpp:18-50,53-60`
- During `StateMachine::onAddedDirty`, each layer requires `Any`, `Entry`, and `Exit`; omission returns `InvalidObject`. `state_machine_layer.cpp:19-47`
- Artboard initialization only stops immediately for `InvalidObject`; other statuses such as `MissingObject` returned by an `onAdded*` callback are ignored and initialization continues. `artboard.cpp:204-208,274-312,325-415`
- Import-stack resolution, by contrast, treats any non-`Ok` importer result as failure. Final file import is `malformed` when the reader errored or stack resolution was non-`Ok`. `import_stack.hpp:71-90`; `file.cpp:664-670`
- There is no transactional rollback. Objects appended before a later failure remain appended until the containing file/artboard is destroyed.

## 3. `StateMachineInstance` header-declared surface map

### Forward declarations and typedefs

| Declaration | Header line | Definition status |
|---|---:|---|
| `FocusData` | 28 | Other-family definition; no definition in `state_machine_instance.cpp`. |
| `FocusListenerGroup` | 29 | Other-family definition. |
| `StateMachine` | 30 | Defined in `state_machine.hpp:15`. |
| `LayerState` | 31 | Other-family definition. |
| `SMIInput` | 32 | FL-C1 definition. |
| `ArtboardInstance` | 33 | Defined in `artboard.hpp:696`. |
| `SMIBool`, `SMINumber`, `SMITrigger` | 34-36 | FL-C1 definitions. |
| `Shape` | 37 | Other-family definition. |
| `StateMachineLayerInstance` | 38 | Defined privately in `state_machine_instance.cpp:140`. |
| `HitComponent` | 39 | Defined inline in this header at line 476. |
| `HitShape` | 40 | No definition found; stale forward declaration. |
| `ListenerGroup` | 41 | Other-family definition. |
| `NestedArtboard` | 42 | Other-family definition. |
| `NestedEventListener`, `NestedEventNotifier` | 43-44 | Other-family definitions in `nested_animation.hpp:12-27`. |
| `Event` | 45 | Other-family definition. |
| `KeyedProperty` | 46 | FL-C3/animation definition. |
| `EventReport` | 47 | Other-family definition. |
| `DataBind`, `BindableProperty` | 48-49 | Data-bind-family definitions. |
| `StateInstance` | 50 | FL-C3 definition. |
| `HitDrawable` | 51 | Defined privately in `state_machine_instance.cpp:716`. |
| `ListenerViewModel` | 52 | Defined privately in `state_machine_instance.cpp:1321`. |
| `ScriptedListenerAction`, `ScriptedDrawable` | 53-54 | Other-family definitions. |
| `DataBindChanged` | 55 | Function-pointer typedef; defined inline in the header. |
| Tools-only `StateMachineInstance` forward declaration | 58 | Completed at line 62. |
| Tools-only `InputChanged` | 59 | Function-pointer typedef; defined inline in the header. |

### Class, inheritance, and friends

`StateMachineInstance` is defined at `state_machine_instance.hpp:62-474`; it derives from `Scene`, `NestedEventNotifier`, `NestedEventListener`, and `DataBindContainer`. Friends are `SMIInput`, `KeyedProperty`, `HitComponent`, and `StateMachineLayerInstance`; friend declarations have no runtime definition. `state_machine_instance.hpp:62-70`

### Private method map

| Member | Declaration | Definition |
|---|---:|---:|
| `updateListeners` | 75-78 | `state_machine_instance.cpp:1494-1545` |
| `getNamedInput<SMType, InstType>` | 80-81 | `state_machine_instance.cpp:2689-2701` |
| `notifyEventListeners` | 82-83 | `state_machine_instance.cpp:3062-3171` |
| `sortHitComponents` | 84 | `state_machine_instance.cpp:2255-2304` |
| `randomValue` | 85 | **No definition found.** A same-named method belongs to the private `StateMachineLayerInstance`, not this class. |
| `findRandomTransition` | 86-88 | **No definition found.** Transition selection moved to `StateMachineLayerInstance`. |
| `findAllowedTransition` | 89-91 | **No definition found.** Transition selection moved to `StateMachineLayerInstance`. |
| `completeViewModelInstances` | 97 | `state_machine_instance.cpp:2792-2829` |
| `addToHitLookup` | 98-102 | `state_machine_instance.cpp:1619-1705` |
| `unbind` | 420 | `state_machine_instance.cpp:2949-2953` |
| `removeEventListeners` | 421 | `state_machine_instance.cpp:2208-2243` |
| `initScriptedObjects` | 422 | `state_machine_instance.cpp:2886-2899` |
| `processFocusEvents` | 452 | `state_machine_instance.cpp:2449-2473` |
| `processSemanticEvents` | 463 | `state_machine_instance.cpp:2482-2507` |

The three undefined transition-selection declarations must not accidentally be linked to `StateMachineLayerInstance` in Rust. Either omit them from an internal Rust surface or implement explicit unreachable/stub behavior if ABI compatibility requires symbols.

### Public method map and concise contract inventory

#### Construction, advancing, and inspection

| Member | Declaration → definition | Contract and adversarial checks |
|---|---|---|
| Constructor | 105-106 → `state_machine_instance.cpp:1707-2128` | Construction-time only. Borrows non-null machine/artboard; creates input instances by authored index, a heap layer array, cloned binds/properties, listener groups, nested event registrations, scripted clones, hit order, and focus tree. No argument validation. Null machine/artboard, null layers/listeners, or malformed gamepad target can crash. |
| Deleted copy constructor | 107 → defined deleted inline | Instances cannot be copied; Rust adaptation must also prevent shallow clone. |
| Destructor | 108 → `state_machine_instance.cpp:2141-2199` | Cleans owned focus/semantic trees, unbinds, deletes inputs, data binds, layers, cloned properties, listener VMs, and scripted clones. It does **not** call `dispose()`/`removeEventListeners()`. |
| `markNeedsAdvance` | 110 → `state_machine_instance.cpp:2667` | Sets the scheduling bit only. |
| `advance(seconds,newFrame)` | 113 → `state_machine_instance.cpp:2546-2585` | Per-advance. On a new frame: focus events, semantic events, then reported events are processed and the scheduling bit cleared. Binds/layers then advance, followed by bind advancement and `SMIInput::advanced`. Returns true for scheduling or pending report queues. A null input slot is dereferenced at lines 2576-2581. NaN/infinity/negative seconds are forwarded without validation. |
| Inline `advance(seconds)` | 115 → inline in hpp | Calls `advance(seconds,true)`. |
| `needsAdvance` | 118 → `state_machine_instance.cpp:2668` | Returns only `m_needsAdvance`; it does not independently inspect report queues. |
| `resetState` | 120 → `state_machine_instance.cpp:2670-2676` | Rebuilds each layer at its entry state; does not clear inputs, event queues, focus, data context, or scheduling state. |
| `stateMachine` | 123 → inline | Returns borrowed `m_machine`. |
| `currentAnimationCount` | 154 → `state_machine_instance.cpp:2985-2996` | Counts layers whose current state is an animation. |
| `currentAnimationByIndex` | 155 → `state_machine_instance.cpp:2998-3014` | Uses a compressed index over animation-bearing layers; out of range returns null. |
| `stateChangedCount` | 159 → `state_machine_instance.cpp:2955-2966` | Counts layer change flags from the previous/new-frame advance. |
| `stateChangedByIndex` | 164 → `state_machine_instance.cpp:2968-2983` | Compressed authored-layer order; returns `nullptr` out of range despite the header comment saying “empty string.” |
| `advanceAndApply(seconds)` | 166 → `state_machine_instance.cpp:2601-2604` | Calls the two-argument overload with view-model advancement enabled. |
| `advanceAndApply(seconds,advanceViewModels)` | 171 → `state_machine_instance.cpp:2606-2665` | Runs one new-frame state-machine advance, artboard advance, up to five dirt/update/state-change passes, and optional VM consumption. Exact `+0.0` and `-0.0` force `keepGoing=true`; NaN does not satisfy the zero special case. |
| `advancedDataContext` | 172 → `state_machine_instance.cpp:2587-2593` | Calls `DataContext::advanced()` only when bound. |
| `reset` | 173 → `state_machine_instance.cpp:2595-2599` | Consumes the data context then resets artboard components; it is not full state-machine reset. |
| `name` | 174 → `state_machine_instance.cpp:2678` | Returns `m_machine->name()`; null machine crashes. |
| `durationSeconds` | 190 → inline | Always `-1`. |
| `loop` | 191 → inline | Always `Loop::oneShot`. |
| `isTranslucent` | 192 → inline | Always true. |
| `artboard` | 196 → inline | Returns the borrowed backing `ArtboardInstance` as `Artboard*`. |
| Parent state-machine setter/getter | 198-205 → inline | Stores/returns a nullable, non-owning raw pointer. |
| Parent nested-artboard setter/getter | 207-211 → inline | Stores/returns a nullable, non-owning raw pointer. |
| `tryChangeState` | 187 → `state_machine_instance.cpp:2306-2318` | Updates binds, then asks every layer in authored order to update state; returns whether any changed. |
| `hitTest` | 188 → `state_machine_instance.cpp:1547-1566` | Applies frame-origin offset and returns on the first hit component. NaN coordinates are not rejected. |
| `dispose` | 381 → `state_machine_instance.cpp:2206` | Removes this instance from nested event notifiers only; does not destroy other state. |

Constructor ordering and duplicate details:

- `m_inputInstances.resize(count)` value-initializes pointer slots to null; known bool/number/trigger definitions replace their slot, while unknown types remain null. `state_machine_instance.cpp:1711-1745`
- Layers are constructed in authored order and immediately initialize entry/any state instances. `state_machine_instance.cpp:1747-1752`
- State-machine data binds are cloned in authored order. Targetless binds are skipped. Shared bindable targets reuse one clone, while to-source/to-target maps overwrite earlier entries with the same cloned property key. `state_machine_instance.cpp:1754-1825`
- Duplicate bindings for the same transition/property allocate a new holder and overwrite the map entry without deleting the previous holder, leaving earlier cloned binds targeting the older holder. `state_machine_instance.cpp:1810-1823`
- Listener initialization follows authored listener order. Event-only and VM listeners take exclusive early paths; other listener categories may create multiple groups. `state_machine_instance.cpp:1831-1967`
- The gamepad listener path dereferences `resolve(targetId())->as<Node>()` without null/type validation. `state_machine_instance.cpp:1945-1950`
- Scripted source objects are cloned into a pointer-keyed unordered map, so duplicate source pointers overwrite prior clones; iteration for context/init is unordered. `state_machine_instance.cpp:2072-2082`

#### Input surface

| Member | Declaration → definition | Contract |
|---|---|---|
| `inputCount` | 125 → inline | Returns slot count, including null/unsupported slots. |
| `input(index)` | 126 → `state_machine_instance.cpp:2680-2687` | In-range returns the stored pointer, possibly null; out of range returns null. |
| `getNamedInput` | 80-81 → `state_machine_instance.cpp:2689-2701` | Linear authored-slot scan, exact name/type, first match. It dereferences every slot, so a null hole crashes. |
| `getBool` | 127 → `state_machine_instance.cpp:2703-2706` | Typed first-name lookup via `getNamedInput`. |
| `getNumber` | 128 → `state_machine_instance.cpp:2707-2710` | Typed first-name lookup via `getNamedInput`. |
| `getTrigger` | 129 → `state_machine_instance.cpp:2711-2714` | Typed first-name lookup via `getNamedInput`. |

Rejecting tests: an unknown input followed by a valid named input must expose the null dereference; duplicate same-type names return the first; same name with differing types is filtered by type.

#### Data-context and view-model surface

| Member | Declaration → definition | Contract |
|---|---|---|
| `bindViewModelInstance` | 130-131 → `state_machine_instance.cpp:2831-2842` | Null clears this context and unbinds the artboard; non-null sets main instance and calls `bind`. |
| `setViewModelInstance` | 135 → `state_machine_instance.cpp:2716-2733` | Null is ignored. Creates a context if absent, otherwise replaces the shared main slot without immediately binding. |
| `setGlobalViewModelInstance` | 140-141 → `state_machine_instance.cpp:2735-2774` | Rejects null, absent file, unknown name, and non-global slot. Creates an empty-main context if needed and replaces only the named slot. |
| `bind` | 144 → `state_machine_instance.cpp:2776-2790` | No-op without a context; fills missing main/global defaults, then binds artboard and state-machine data binds in that order. |
| `globalViewModelInstance` | 147 → `state_machine_instance.cpp:2844-2859` | Pure read; null without context/file. It does not validate that the name is global before asking for the slot. |
| `bindDataContext` | 148 → `state_machine_instance.cpp:2861-2868` | Clears prior context, then dereferences the supplied context without a null check; binds both artboard and SMI. |
| `inheritDataContext` | 149 → `state_machine_instance.cpp:2870-2878` | Null no-op; adds this dependent and binds only this SMI, without first clearing a prior context. |
| `dataContext(setter)` | 150 → `state_machine_instance.cpp:2880-2884` | Clears previous dependency then applies the supplied context, including null. |
| `dataContext(getter)` | 151 → inline | Returns a retained `rcp` copy. |
| `rebind` | 152 → `state_machine_instance.cpp:2916-2921` | Clears/reapplies artboard context, then rebinds this SMI to the current context. |
| `completeViewModelInstances` | 97 → `state_machine_instance.cpp:2792-2829` | Creates a missing artboard-default main instance and one default instance for every unoccupied global slot; existing cross-VM slot overrides remain. |
| `internalDataContext` | 265 → `state_machine_instance.cpp:2901-2914` | Stores the context, binds data binds and VM listeners, propagates context to scripted clones, then initializes/hydrates scripts. |
| `clearDataContext` | 262 → `state_machine_instance.cpp:2923-2934` | Removes this dependent and clears listener VM property bindings; it does not itself call `unbindDataBinds`. |
| `relinkDataContext` | 263 → `state_machine_instance.cpp:2936-2939` | Delegates only to the artboard. |
| `rebuildDataBind` | 264 → `state_machine_instance.cpp:2941-2947` | Rebinds only `DataBindContext` instances against `m_DataContext`. |
| `unbind` | 420 → `state_machine_instance.cpp:2949-2953` | Clears context/listener dependencies, then unbinds inherited data binds. |
| `initScriptedObjects` | 422 → `state_machine_instance.cpp:2886-2899` | Unordered map walk; initializes script assets once and hydrates inputs each call. |

Rejecting tests: `bindDataContext(nullptr)` must not be “safely ignored” unless the Rust port deliberately introduces a documented divergence; `setViewModelInstance(nullptr)` and `bindViewModelInstance(nullptr)` have intentionally different behavior.

#### Pointer and hit surface

| Member | Declaration → definition | Contract |
|---|---|---|
| `updateListeners` | 75-78 → `state_machine_instance.cpp:1494-1545` | Resets listener groups, prepares every hit component, then processes hit components in sorted order. An opaque hit makes later components receive `canHit=false` but does not stop iteration. Exit releases events after processing. |
| `pointerMove` | 175-177 → `state_machine_instance.cpp:1568-1573` | Dispatches `move` with timestamp and pointer ID. |
| `pointerDown` | 178 → `state_machine_instance.cpp:1574-1577` | Dispatches `down` with timestamp 0. |
| `pointerUp` | 179 → `state_machine_instance.cpp:1578-1581` | Dispatches `up` with timestamp 0. |
| `pointerExit` | 180 → `state_machine_instance.cpp:1582-1585` | Dispatches `exit` and releases listener events. |
| `dragStart` | 181-184 → `state_machine_instance.cpp:1586-1597` | Optionally disables pointer events first. The supplied timestamp is deliberately **not** forwarded; dispatch uses 0. |
| `dragEnd` | 185 → `state_machine_instance.cpp:1598-1606` | Enables pointer events, dispatches `dragEnd` with timestamp 0, then dispatches a timestamped pointer move. |
| `addToHitLookup` | 98-102 → `state_machine_instance.cpp:1619-1705` | Deduplicates layout/shape/text hit wrappers by component pointer; recursively expands container children in child order. Unsupported target kinds add nothing. |
| `sortHitComponents` | 84 → `state_machine_instance.cpp:2255-2304` | Moves artboard targets first, then orders matched components by artboard drawable order; unmatched entries retain their remaining relative arrangement only as a consequence of swaps, not a stable-sort guarantee. |
| `hasListeners` | 257 → inline | True iff `m_hitComponents` is nonempty, including nested/component-list hit proxies with no authored pointer listener. |
| `enablePointerEvents` | 379 → `state_machine_instance.cpp:3173-3179` | Forwards to all hit components in current sorted order. |
| `disablePointerEvents` | 380 → `state_machine_instance.cpp:3181-3187` | Same, disabling the given pointer ID. |

FP edges: pointer coordinates, origin offsets, and timestamps are not checked for NaN/infinity. `dragStart`/`dragEnd` ignoring timestamps is a required compatibility detail.

#### Events and listener view models

| Member | Declaration → definition | Contract |
|---|---|---|
| `notify` | 212-213 → `state_machine_instance.cpp:3041-3046` | Immediately processes nested events, then updates data binds. |
| `notifyListenerViewModels` | 214-215 → `state_machine_instance.cpp:3048-3060` | Processes queued listener-VM reports in vector order; duplicates fire repeatedly. |
| `reportEvent` | 219 → `state_machine_instance.cpp:3016-3019` | Appends an `EventReport`; does not set `m_needsAdvance`. Null event is accepted here but later dereferenced during audio handling. Delay, including NaN/infinity/negative, is stored unchanged. |
| `applyEvents` | 221 → `state_machine_instance.cpp:2320-2344` | Drains reported event and VM queues in same-frame batches, clearing each source queue before callbacks so recursively reported events enter the next iteration. Stops at 100 iterations and prints a warning. |
| `reportListenerViewModel` | 223 → `state_machine_instance.cpp:3021-3025` | Appends borrowed pointer; no null check and no scheduling-bit update. |
| `reportedEventCount` | 226 → `state_machine_instance.cpp:3027-3030` | Size of the pending report queue only, not the currently reporting batch. |
| `reportedEventAt` | 229 → `state_machine_instance.cpp:3032-3039` | Returns by value; out of range returns `EventReport(nullptr, 0)`. |
| `playsAudio` | 230 → inline | Always true. |
| `notifyEventListeners` | 82-83 → `state_machine_instance.cpp:3062-3171` | Listener-major, then event-minor authored order. Matching single listeners break after the first matching event; multi-input listeners break only their input scan. Events then bubble immediately to nested listeners, followed by audio playback. |

Rejecting tests: recursively report 101 generations; duplicate `ListenerViewModel*` entries; nested and local events with identical numeric IDs; out-of-range `reportedEventAt`; null event reaching audio inspection.

#### Data-bind/property helpers

| Member | Declaration → definition | Contract |
|---|---|---|
| `bindablePropertyInstance` | 231-232 → `state_machine_instance.cpp:3189-3199` | Source-property pointer lookup; not found returns null. |
| `bindableDataBindToSource` | 233-234 → `state_machine_instance.cpp:3201-3210` | Clone-property pointer lookup; not found returns null; duplicate registration leaves last map entry. |
| `bindableDataBindToTarget` | 235-236 → `state_machine_instance.cpp:3212-3221` | Same for to-target binds. |
| `findTransitionPropertyInstance` | 241-243 → `state_machine_instance.cpp:3223-3237` | Two-level pointer/property-key lookup; absent returns null. |
| `buildStateKeyFrameBinds` | 251 → `state_machine_instance.cpp:3276-3374` | For each driven animation, clones the **first** source-artboard bind per supported keyframe target, creates a typed holder, initializes the clone, and adds it to the instance bind container. Null state/artboard/source is a no-op. |
| `removeStateKeyFrameBinds` | 255 → `state_machine_instance.cpp:3376-3390` | Removes and deletes all tracked clones for the state, then erases the map entry; absent state is a no-op. |

Keyframe duplicate ordering is explicit: `unordered_map::emplace` preserves the first source bind for each target. `state_machine_instance.cpp:3290-3306`

#### Focus, semantic, and gamepad surface

| Member | Declaration → definition | Contract |
|---|---|---|
| `hasFocusNodes` | 258 → `state_machine_instance.cpp:3392-3397` | Delegates to active manager’s `hasFocusableContent`; debug-asserts non-null. |
| `focusNext` | 259 → `state_machine_instance.cpp:3399-3404` | Delegates and returns result. |
| `focusPrevious` | 260 → `state_machine_instance.cpp:3406-3411` | Delegates and returns result. |
| `clearFocus` | 261 → `state_machine_instance.cpp:3413-3418` | Delegates to active manager. |
| `queueFocusEvent` | 269 → `state_machine_instance.cpp:2409-2414` | Appends borrowed group plus flag and marks advance needed; processed next `newFrame` advance. |
| `queueSemanticEvent` | 272-273 → `state_machine_instance.cpp:2475-2480` | Same for semantic action. |
| `fireSemanticAction` | 276-277 → `state_machine_instance.cpp:2509-2544` | No-op without manager/node/semantic data; dispatches tap/increase/decrease to `SemanticData`. |
| non-const `focusManager` | 281-285 → inline | External manager if non-null, otherwise owned internal manager. |
| const `focusManager` | 289-293 → inline | Same selection. |
| `hasExternalFocusManager` | 296-299 → inline | Tests external pointer. |
| `internalFocusManager` | 304 → inline | Always returns owned manager, ignoring external selection. |
| `submitGamepadsFromBuffer` | 310 → `src/input/gamepad_batch.cpp:165-296` | Parses versioned little-endian records and dispatches each complete record through focus then script broadcast. Returns false on malformed/truncated input but does not roll back already dispatched records. |
| `broadcastGamepadToScriptedDrawables` | 317-319 → `src/input/gamepad_batch.cpp:298-362` | First recurses through hit-component children, then broadcasts to interested scripted drawables except the already-focused recipient. Direct script hits are never opaque. |
| `setExternalFocusManager` | 325 → `state_machine_instance.cpp:2346-2368` | Identity no-op; otherwise cleans the old tree, changes pointer, and rebuilds with external or internal manager. |
| `setFocus` | 328 → `state_machine_instance.cpp:2416-2428` | Non-null focuses `focusData->focusNode`; null clears focus. |
| `FocusState` | 333-342 → inline struct | Value snapshot with both booleans initialized false. |
| `focusState` | 343 → `state_machine_instance.cpp:2430-2447` | Polls without refcount bump; focused target sets `hasFocus`, and `Focusable::acceptsKeyboardInput` determines the second flag. |
| `semanticManager` | 348-352 → inline | External manager first, otherwise owned internal manager; may return null. |
| `enableSemantics` | 357 → `state_machine_instance.cpp:2370-2381` | No-op if any semantic manager exists; otherwise creates internal manager and builds tree. |
| `setExternalSemanticManager` | 364-365 → `state_machine_instance.cpp:2383-2407` | Identity no-op; cleans old tree, stores possibly null external pointer, then rebuilds using the selected manager and optional parent node. |
| `processFocusEvents` | 452 → `state_machine_instance.cpp:2449-2473` | Moves the queue aside, then matches focus/blur listener type in FIFO order. Assumes group/listener are valid. Newly queued events wait for a later frame. |
| `processSemanticEvents` | 463 → `state_machine_instance.cpp:2482-2507` | FIFO moved batch; null group/listener is skipped. Newly queued events wait for a later frame. |

Gamepad adversarial/FP details:

- Null buffer, wrong version, truncated record, unknown record type, update-before-connect, and out-of-range button/axis return false. `gamepad_batch.cpp:168-191,213-292`
- Any change-kind byte other than zero is treated as an axis, not rejected as an unknown kind. `gamepad_batch.cpp:240-245`
- Connect with the same device ID replaces the snapshot; disconnect of an unknown ID still dispatches a disconnect event. `gamepad_batch.cpp:196-210,272-287`
- Update changes mutate the stored snapshot sequentially. If a later change is invalid, earlier mutations remain, although no per-change dispatch occurs until the apply loop completes. `gamepad_batch.cpp:247-269`
- Float payloads are not finite/range checked. NaN button values are stored and compare false against the `>= 0.5` pressed threshold; signed zero is stored unchanged. `gamepad_batch.cpp:14-23,74-88,91-119`

#### Testing- and tools-only declarations

| Member | Declaration | Definition/contract |
|---|---:|---|
| `hitComponentsCount` | 368 | Inline under `TESTING`; vector size. |
| `hitComponent(index)` | 369-376 | Inline under `TESTING`; in-range pointer, otherwise null. |
| `layerState(index)` | 377 | `state_machine_instance.cpp:1609-1616`; machine-layer-bound check, then current state, else null. |
| `onInputChanged` | 467-470 | Inline under `WITH_RIVE_TOOLS`; replaces nullable callback. |
| `onDataBindChanged` | 471 | `state_machine_instance.cpp:2246-2252`; iterates the derived class’s `m_dataBinds`. |
| `m_inputChangedCallback` | 472 | Inline field initialized null. |

`onDataBindChanged` exposes a surprising shadowing bug: `StateMachineInstance::m_dataBinds` is never populated anywhere in the implementation, while normal `addDataBind` calls populate `DataBindContainer`’s separate private vector. Consequently this callback loop is normally empty. `state_machine_instance.hpp:395`; `state_machine_instance.cpp:2246-2252`; `data_bind_container.hpp:14,29-45`

## 4. Field map and Rust initialization rules

All data-member declarations are definitions in `state_machine_instance.hpp`; no out-of-class field definitions exist.

| Field | Line | Initial state / ownership |
|---|---:|---|
| `m_DataContext` | 93 | Explicit null retained pointer. |
| `m_reportedEvents` | 384 | Default-empty owned values. |
| `m_reportingEvents` | 385 | Default-empty current-batch values. |
| `m_machine` | 386 | **No header initializer**; borrowed pointer initialized from constructor argument at cpp line 1709. |
| `m_needsAdvance` | 387 | Explicit false. |
| `m_inputInstances` | 388 | Default-empty; manually owns each pointer. |
| `m_layerCount` | 389 | **No header initializer**; assigned from machine at cpp line 1747. |
| `m_layers` | 390 | **No header initializer**; assigned heap array at cpp line 1748 and deleted at line 2174. |
| `m_hitComponents` | 391 | Default-empty; owns entries. |
| `m_listenerGroups` | 392 | Default-empty; owns entries. |
| `m_parentStateMachineInstance` | 393 | Explicit null, borrowed. |
| `m_parentNestedArtboard` | 394 | Explicit null, borrowed. |
| `m_dataBinds` | 395 | Default-empty borrowed pointers; shadows the base container vector. |
| `m_listenerViewModels` | 396 | Default-empty; manually owns entries. |
| `m_reportedListenerViewModels` | 397 | Default-empty borrowed queue. |
| `m_reportingListenerViewModels` | 398 | Default-empty borrowed current batch. |
| `m_bindablePropertyInstances` | 399-400 | Default-empty; owns mapped clones, deleted in destructor. |
| `m_scriptedObjectsMap` | 401-402 | Default-empty; owns mapped clones, pointer-keyed source lookup. |
| `m_bindableDataBindsToTarget` | 403-404 | Default-empty borrowed map into bind-container-owned clones. |
| `m_bindableDataBindsToSource` | 405-406 | Same. |
| `m_transitionPropertyInstances` | 410-412 | Default-empty; owns nested mapped property holders. |
| `m_stateKeyFrameDataBinds` | 417-418 | Default-empty tracking pointers; binds are owned by the bind container until explicit removal. |
| `m_drawOrderChangeCounter` | 419 | Explicit zero; wraps according to artboard counter behavior. |
| `m_focusManager` | 425 | Default-constructed, owned. |
| `m_externalFocusManager` | 426 | Explicit null, borrowed. |
| `m_focusListenerGroups` | 427 | Default-empty, owning. |
| `m_keyboardListenerGroups` | 428-429 | Default-empty, owning. |
| `m_gamepadListenerGroups` | 430 | Default-empty, owning. |
| `m_gamepadScriptedDrawables` | 437 | Default-empty, non-owning artboard back-references. |
| `m_embedderGamepads` | 439 | Default-empty snapshots, cleared in destructor. |
| `m_semanticManager` | 442 | Default-constructed null `unique_ptr`, owned when enabled. |
| `m_externalSemanticManager` | 443 | Explicit null, borrowed. |
| `QueuedFocusEvent::group` | 448 | **No header initializer**; aggregate raw pointer set by `queueFocusEvent`. |
| `QueuedFocusEvent::isFocus` | 449 | **No header initializer**; aggregate bool set by `queueFocusEvent`. |
| `m_queuedFocusEvents` | 451 | Default-empty FIFO vector. |
| `m_semanticListenerGroups` | 455-456 | Default-empty, owning. |
| `QueuedSemanticEvent::group` | 459 | **No header initializer**; aggregate raw pointer set by `queueSemanticEvent`. |
| `QueuedSemanticEvent::actionType` | 460 | **No header initializer**; aggregate enum set by `queueSemanticEvent`. |
| `m_queuedSemanticEvents` | 462 | Default-empty FIFO vector. |
| tools-only `m_inputChangedCallback` | 472 | Explicit null. |

### Required Rust adaptation for fields without header initializers

- `m_machine`: require a constructor-supplied borrowed/non-owning handle; do not create an observable null/default state.
- `m_layerCount`: derive atomically from the supplied machine before layer access.
- `m_layers`: allocate from `layerCount`; represent it as an owned Rust collection rather than a transient uninitialized raw pointer.
- `QueuedFocusEvent` and `QueuedSemanticEvent`: require all fields in their Rust constructors/struct literals; do not implement `Default` unless it creates an explicitly invalid private sentinel that can never be queued.
- These C++ fields are initialized before their first normal use, but their header declarations alone provide no initial value. Rust must not silently choose semantically meaningful defaults.

Class-type fields such as vectors, maps, `unique_ptr`, and `FocusManager` are default-constructed and therefore are not C++-uninitialized even when they lack `= ...` syntax.

## 5. `HitComponent` helper inventory

`HitComponent` is defined inline at `state_machine_instance.hpp:476-506`.

| Member | Declaration/definition | Contract |
|---|---:|---|
| `component` | 479 inline | Returns borrowed `m_component`, possibly null. |
| Constructor | 480-483 inline | Stores component and SMI raw pointers without validation. |
| Destructor | 484 inline | Virtual, empty. |
| `processEvent` | 485-489 | Pure virtual; called by `updateListeners`. |
| `processGamepadInvocation` | 490-492 | Pure virtual; called by gamepad broadcast. |
| `prepareEvent` | 493-495 | Pure virtual; called before pointer event processing. |
| `hitTest` | 496 | Pure virtual; called by SMI hit testing. |
| `enablePointerEvents` | 497 inline | Virtual default no-op. |
| `disablePointerEvents` | 498 inline | Virtual default no-op. |
| testing `earlyOutCount` | 500 | Explicit zero. |
| `m_component` | 504 | **No header initializer**; borrowed pointer initialized by constructor. |
| `m_stateMachineInstance` | 505 | **No header initializer**; borrowed pointer initialized by constructor. |

Rust adaptation: require both constructor fields explicitly, preserve nullable component if subclasses rely on it, and use trait methods for the four pure-virtual operations. The base enable/disable implementations must remain no-ops.

## 6. Cross-family boundary list

| Owner/family | Referenced names and locations | Contract FL-C5 relies on |
|---|---|---|
| FL-C1 inputs | `StateMachineInput`, `SMIInput`, `SMIBool`, `SMINumber`, `SMITrigger`; `state_machine.hpp:10,21,47-48`; `state_machine_instance.hpp:32,34-36,125-129` | Input definitions transfer ownership during import, retain authored indices, expose type/name, create per-instance wrappers, and support `advanced()` after every SMI advance. Null/unknown slots are not tolerated by named lookup or advancement. `state_machine_instance.cpp:1711-1745,2576-2581,2689-2714` |
| FL-C3 layers/states | `StateMachineLayer`, `LayerState`, `StateInstance`, `StateMachineLayerInstance`; `state_machine.hpp:9,20,49-50`; `state_machine_instance.hpp:31,38,50,154-164` | Layers retain authored order, contain required any/entry/exit states, construct state instances, advance/apply them, expose current state/animation, and own transition-selection timing. `state_machine_layer.cpp:19-47`; `state_machine_instance.cpp:140-714,1747-1752` |
| FL-C2 transitions/conditions | `StateTransition`, transition duration binding, `allowed`, `useLayerInConditions`; `state_machine_instance.hpp:86-91,241-243` | Transition order determines first allowed transition unless random mode is set; conditions may return yes/no/waiting-for-exit; duration can be overridden per SMI without mutating the shared transition. `state_machine_instance.cpp:366-450,473-573,1810-1823,3223-3237` |
| FL-C4 listeners/events | `StateMachineListener`, `ListenerGroup`, focus/keyboard/gamepad/semantic listener groups, `Event`, `EventReport`, nested notifier/listener; `state_machine.hpp:11,22,39,52`; `state_machine_instance.hpp:28-29,41,43-47,212-229` | Listener authored order is observable; target IDs resolve through the artboard; action order belongs to each listener; events can recursively report and immediately bubble; specialized groups remain alive while queued raw pointers exist. `state_machine_listener.cpp:85-93`; `state_machine_instance.cpp:1831-1967,2320-2344,3041-3171` |
| DataBind family | `DataBind`, `DataBindContainer`, `DataContext`, `BindableProperty`, `BindablePropertyNumber`; `state_machine.hpp:14,23,40,51`; `state_machine_instance.hpp:19-20,48-49,65,93,231-255` | Definitions are owned by the source state machine; instances clone them, own clones through `DataBindContainer`, update target/source in container-defined order, bind them to contexts, and create per-instance transition/keyframe targets. `state_machine_instance.cpp:1754-1825,2306-2309,2562-2574,3189-3390` |
| Artboard/Scene | `ArtboardImporter`, `Artboard`, `ArtboardInstance`, inherited `Scene::m_artboardInstance`; `state_machine.cpp:2-3,70-80`; `state_machine_instance.hpp:18,33,62` | Artboard owns source state machines, supplies object resolution/draw order/nested artboards/data binds, drives dirty/update/reset passes, and must outlive the SMI. It also owns focus/semantic tree attachment points. `artboard_importer.cpp:24-40`; `state_machine_instance.cpp:1499-1504,1747-1752,1827-2128,2606-2662` |
| Scripting | `ScriptedObject`, `ScriptedDrawable`, `ScriptedListenerAction`; `state_machine.hpp:13,24,41-45`; `state_machine_instance.hpp:53-54,266,317-319` | Source scripted objects outlive the machine; each SMI clones state-machine-owned script objects, propagates data context, hydrates inputs, and borrows artboard scripted drawables for gamepad broadcast. `state_machine_instance.cpp:2072-2120,2130-2139,2886-2914`; `gamepad_batch.cpp:298-362` |
| Focus/semantics | `FocusManager`, `FocusData`, `Focusable`, `SemanticManager`, `SemanticNode`, `SemanticData`; `state_machine_instance.hpp:21-24,28,258-365` | Managers own tree state and dispatch; an external manager may replace the internal one; artboard tree cleanup/rebuild must occur around manager changes/destruction; focused node and semantic node IDs remain valid through deferred dispatch. `state_machine_instance.cpp:2123-2128,2141-2157,2346-2544` |
| Nested animation boundary | `NestedArtboard`, `NestedStateMachine`, `NestedLinearAnimation`, `NestedEventNotifier/Listener`; `state_machine_instance.hpp:17,42-44,207-215` | Nested SMIs receive transformed pointer events, recursively receive gamepad broadcasts, and bubble event reports. Top-level embedders must call `dispose()` before retaining the artboard beyond SMI lifetime. `state_machine_instance.cpp:904-1071,2016-2046,2201-2243,3155-3160` |

This boundary deliberately leaves implementation of inputs, layer/state behavior, transition conditions, listener actions, data-binding internals, artboard dirt/update mechanics, and focus/semantic managers to their owning families. FL-C5 depends only on the contracts enumerated above.