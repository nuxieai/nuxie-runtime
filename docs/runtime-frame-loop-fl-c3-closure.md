# Layer/state-occurrence owner-family closure

This is the publication checklist for the complete pinned-C++ layer and
state-occurrence family at
`d788e8ec6e8b598526607d6a1e8818e8b637b60c`. Production is not eligible for
review until every row below has a filename-corresponding Rust owner and either
a live C++ differential or an explicit source-cited structural proof.

The earlier FL-C mini-map described a stable collection of state occurrences.
The complete source walk corrected that wording: `StateMachineLayer` retains
one ordered collection of **state definitions**, while each
`StateMachineLayerInstance` dynamically owns only the `any`, `current`, and
transition-source (`stateFrom`) occurrences. C++ creates and destroys current
and source occurrences at state changes; Rust must preserve that triad and its
alias/deletion guards rather than inventing a prebuilt occurrence collection.

## Source-to-Rust closure

| Pinned C++ owner | Complete semantics | Focused Rust owner | Required evidence |
| --- | --- | --- | --- |
| `src/animation/layer_state.cpp` | one insertion-ordered transition-definition owner; dirty and clean finalization visit every transition in order and return the first failure; import requires the current layer importer; destruction deletes transitions in order; the base `makeInstance` creates a system/no-op occurrence | `crates/nuxie-runtime/src/state_machine/layer_state.rs` | ordered import/finalization structural proof; nullable generic-state differential; definition/occurrence identity test |
| `src/animation/state_instance.cpp` | each occurrence retains exactly one immutable `LayerState` definition identity; the base owns no copied transition/animation payload; virtual destruction and default no-op lifecycle hooks remain explicit | `crates/nuxie-runtime/src/state_machine/state_instance.rs` | two occurrences share the definition but isolate mutable state; definition mutation is observed at the same C++ read sites; clone/remount proof |
| `src/animation/state_machine_layer.cpp` | one insertion-ordered state-definition owner; dirty finalization visits states in order, resolves the last authored Any/Entry/Exit occurrence, and rejects a layer missing any required system state; clean finalization preserves order; import transfers unique layer ownership to the current state-machine importer; destruction deletes states in order | `crates/nuxie-runtime/src/state_machine/state_machine_layer.rs` | live malformed-layer differentials for missing Any, Entry, and Exit; duplicate-system-state ordering differential; nullable-state index and transition-target differential |
| `src/animation/system_state_instance.cpp` | construction retains the exact definition; `advance` and `apply` are no-ops; `keepGoing` is always false | `crates/nuxie-runtime/src/state_machine/system_state_instance.rs` | live entry/exit/any/generic no-op differential; unchanged-frame keep-going proof |
| `src/animation/nested_state_machine.cpp` | owns one optional child `StateMachineInstance` plus an insertion-ordered, non-owning NestedInput list; initializes by `animationId`; shares the parent focus manager before synchronizing the nested focus tree; applies authored bool/number inputs in order but not triggers; forwards advance, hit/pointer/drag, context binding, clear, and `tryChangeState`; index lookup is bounds-checked and name lookup is first-match; dependency release clears the state machine before the child Artboard dies; generated clone is cold and does not copy the live instance or input list | `crates/nuxie-runtime/src/state_machine/nested_state_machine.rs` | live initialization/input-order differential; pointer/hit forwarding differential; context attach/clear proof; cold-clone and teardown-order test |

Supporting oracle files are read as part of this family but are not promoted by
this lane:

- `src/importers/state_machine_layer_importer.cpp` supplies ordered state
  attachment, nullable generic states, animation/transition target validation,
  and out-of-range rejection.
- `src/importers/layer_state_importer.cpp` supplies ordered transition
  attachment and BlendState transition resolution.
- `src/animation/state_machine_instance.cpp:140-711` contains the private
  `StateMachineLayerInstance` occurrence owner consumed by these five public
  files.
- `src/nested_artboard.cpp:48-75,167-185,250-373,673-741,800-941,943-1015`
  supplies nested focus, context, advance, and teardown owner order.
- Generated `StateMachineLayerBase::clone` and
  `NestedStateMachineBase::clone` allocate an empty concrete owner and copy
  generated/base fields only; custom child collections and live occurrence
  state are not copied.

The private layer-occurrence class maps to
`crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs`.
This is an explicit filename adaptation because pinned C++ defines that class
inside `state_machine_instance.cpp`; it does not create another promoted C++
file row.

## Layer-occurrence lifecycle closure

Every item below is mandatory even though the private C++ class shares
`state_machine_instance.cpp` with the later complete instance lane:

- [x] Construction creates one `any` system occurrence, installs the retained
  layer definition, then changes to the entry state in that order.
- [x] The empty `AnyState` occurrence is retained and probed before current
  state on every eligible update; Rust must delete its current
  `!transitions.is_empty()` shortcut.
- [x] `current`, `stateFrom`, and `any` are typed owner-local occurrence
  identities, not a state index plus parallel animation/blend payloads.
- [x] State changes no-op only when the current occurrence references the
  exact target definition; end actions run before the new occurrence is made;
  keyframe binds attach before start actions.
- [x] A transition interruption deletes the previous `stateFrom` after
  removing its keyframe binds, retains the outgoing current occurrence as the
  new `stateFrom`, and installs the new current occurrence in pinned order.
- [x] The active transition is a retained definition handle. Rust may not copy
  transition fire-action, listener-action, interpolator, or duration payloads
  into the layer occurrence.
- [x] `advance` preserves current advance → mix update → source advance →
  apply → bounded chained state changes/apply → spilled-time clear, including
  the exact 100-iteration boundary and keep-going terms.
- [x] `resetState` removes keyframe binds and drops `stateFrom` only when it is
  distinct from `any` and current, drops current only when distinct from
  `any`, then re-enters the retained entry definition.
- [x] Drop releases `any`, current, and source occurrences without double
  release under the same identity constraints.
- [x] Rust's public snapshot clone remains an explicit Rust API adaptation:
  immutable definitions stay shared, mutable occurrences are independently
  copied, owner-local ids are refreshed, and no C++ generated-clone claim is
  made for that snapshot behavior.

## Adversarial publication review

- [x] Missing required system state: Any, Entry, and Exit are each rejected at
  the C++ finalization boundary.
- [x] Duplicate system states: the last authored Any/Entry/Exit wins exactly
  like the ordered C++ switch.
- [x] Nullable/unknown state: one generic no-op state occurrence retains its
  slot and remains a valid transition target.
- [x] Bad transition target: negative/sentinel and out-of-range `stateToId`
  fail import without compacting state indices.
- [x] Empty AnyState: it is constructed and probed; no transition-count
  shortcut remains.
- [x] Same-state target: no occurrence replacement, actions, or keyframe-bind
  churn occurs.
- [x] Transition interruption: previous source, outgoing current, new current,
  mix-from, pause/hold, actions, and keyframe binds follow pinned order.
- [x] Reset aliases: `any == stateFrom`, `any == current`, and
  `stateFrom == current` cannot double-drop or remove binds twice.
- [x] Zero-time and same-frame chaining: system states and zero-duration
  transitions settle in the same bounded advance as C++.
- [x] Nested input ordering: duplicate names use first-match lookup; index
  lookup preserves authored slots; initialization applies bool/number and
  deliberately skips trigger.
- [x] Nested focus and pointer forwarding: shared focus manager is installed
  before tree sync; hit/pointer/drag calls return the child result or the
  pinned empty result.
- [x] Nested context lifecycle: attach, replacement, clear, dependency release,
  clone, and parent teardown preserve owner order.
- [x] Clone/remount isolation: definitions are shared where C++ retains them,
  live state occurrences are occurrence-local, and generated nested/layer
  clones remain cold.
- [x] Permanent structural ratchets: reject the empty-Any shortcut, parallel
  state-occurrence payloads, copied active-transition payloads, prebuilt
  state-occurrence collections, nested-state-machine owner scans, and missing
  reset/drop identity guards, plus live nested occurrences copied into a
  generated public clone.

## Structural enforcement required before publication

The candidate checker must have injected negative controls for:

1. filtering `AnyState` by whether it has transitions;
2. storing current/source state as indices beside parallel animation/blend
   occurrence fields;
3. cloning active transition actions/interpolator/duration into a layer
   occurrence instead of retaining one typed definition handle;
4. prebuilding one occurrence for every state definition;
5. scanning every nested artboard/animation to rediscover a
   `NestedStateMachine` owner for an authored NestedInput;
6. omitting the reset/drop alias guards or keyframe-bind teardown;
7. reordering the current/mix/source/apply/chained-transition advance sequence.
8. copying a live nested state-machine occurrence into a generated public
   Artboard clone instead of reconstructing it cold.

## Closed evidence

- Live pinned-C++ differentials cover independently missing Any/Entry/Exit,
  sentinel and out-of-range transition targets, last-authored Entry selection,
  a generic `LayerState` no-op occurrence, same-state no-op, early-exit
  interruption, direct/1D blend transitions, reset, pause/hold, percentage
  duration/exit time, zero-time chaining, and ordinary animation-state
  advance (`crates/nuxie-runtime/tests/cpp_probe.rs`).
- Focused Rust tests cover last-authored system-state resolution, the complete
  system-state no-op, authored nested input slots/first-name lookup/trigger
  omission, every empty-child pointer and drag return, data-context attach and
  clear, quantized nested probing, and shared nested focus with snapshot-clone
  isolation (`state_machine/state_machine_layer.rs`,
  `state_machine/system_state_instance.rs`, `artboard.rs`, and `focus.rs`).
- Rust's distinct owned `Option<RuntimeStateInstance>` values make the C++
  raw-pointer alias cases unrepresentable. Drop performs the corresponding
  unique teardown; the checker rejects replacing those values with shared
  `Rc`/`Arc` aliases. The public snapshot clone is separately documented as a
  Rust API adaptation and refreshes layer occurrence identities.
- Eight checker ratchets have injected negative controls for every forbidden
  shape listed above. File and member rows remain pending verification until
  the immutable whole-family candidate receives independent acceptance.

## Publication boundary

Use focused tests while editing. Run the expensive runtime/workspace-probe,
ordinary/scripted golden, renderer pixel, ABI, size, format/lint, structural,
and performance floors once after every checklist row is closed on one frozen
candidate. Publish one immutable SHA for one independent whole-family verdict;
do not submit a partial layer, nested, reset, or transition-interruption slice.
