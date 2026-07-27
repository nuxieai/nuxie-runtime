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

The public layer/state family remains the five files below. Its rejected
candidate also exposed that the previously accepted `src/math/random.cpp` row
had been mapped to an unrelated consolidated module and was not actually
reachable from Rust transitions. This replacement candidate therefore maps
the transition call site directly to
`crates/nuxie-runtime/src/math/random.rs`. The supporting row stays pending:
pinned `DataConverterFormula` also calls the global provider, while its later
FL-D Rust owner still uses a separate legacy stream. FL-C3 proves the complete
layer consumer and records that dependent gap instead of falsely promoting the
whole-program random owner.

| Pinned C++ owner | Complete semantics | Focused Rust owner | Required evidence |
| --- | --- | --- | --- |
| `src/animation/layer_state.cpp` | one insertion-ordered transition-definition owner; dirty and clean finalization visit every transition in order and return the first failure; import requires the current layer importer; destruction deletes transitions in order; the base `makeInstance` creates a system/no-op occurrence | `crates/nuxie-runtime/src/state_machine/layer_state.rs` | ordered import/finalization structural proof; nullable generic-state differential; definition/occurrence identity test |
| `src/animation/state_instance.cpp` | each occurrence retains exactly one immutable `LayerState` definition identity; the base owns no copied transition/animation payload; virtual destruction and default no-op lifecycle hooks remain explicit | `crates/nuxie-runtime/src/state_machine/state_instance.rs` | two occurrences share the definition but isolate mutable state; definition mutation is observed at the same C++ read sites; clone/remount proof |
| `src/animation/state_machine_layer.cpp` | one insertion-ordered state-definition owner; dirty finalization visits states in order, resolves the last authored Any/Entry/Exit occurrence, and rejects a layer missing any required system state; clean finalization preserves order; import transfers unique layer ownership to the current state-machine importer; destruction deletes states in order | `crates/nuxie-runtime/src/state_machine/state_machine_layer.rs` | live malformed-layer differentials for missing Any, Entry, and Exit; duplicate-system-state ordering differential; nullable-state index and transition-target differential |
| `src/animation/system_state_instance.cpp` | construction retains the exact definition; `advance` and `apply` are no-ops; `keepGoing` is always false | `crates/nuxie-runtime/src/state_machine/system_state_instance.rs` | live entry/exit/any/generic no-op differential; unchanged-frame keep-going proof |
| `src/animation/nested_state_machine.cpp` | owns one optional child `StateMachineInstance` plus an insertion-ordered, non-owning NestedInput list; initializes by `animationId`; shares the parent focus manager before synchronizing the nested focus tree; applies authored bool/number inputs in order but not triggers; forwards advance, hit/pointer/drag, context binding, clear, and `tryChangeState`; index lookup is bounds-checked and name lookup is first-match, with an empty name for a missing child input; dependency release clears the state machine before the child Artboard dies; generated object clone is cold and the surrounding Artboard clone later rebuilds the authored input list | `crates/nuxie-runtime/src/state_machine/nested_state_machine.rs` | live initialization/input-order differential; missing-input empty-name proof; pointer/hit forwarding differential; context attach/clear proof; cold-clone and teardown-order test |
| `src/math/random.cpp` plus `include/rive/math/random.hpp` (supporting row remains pending for FL-D consumer closure) | one process-global platform-C `rand` provider; each layer initialization reseeds it with `1` in deterministic mode and the target standard library's `high_resolution_clock` source otherwise; TESTING replaces draws with one counted FIFO and returns zero when exhausted; native Apple/Unix/Windows clock selection and the pinned browser build's Emscripten 3.1.61 musl algorithm/monotonic nanosecond seed are part of the platform boundary | `crates/nuxie-runtime/src/math/random.rs`, with native and Wasm target adapters in `math/random/native.rs` and `math/random/wasm.rs` | injected later-candidate differential; exact call count; `uint32_t` overflow selection edge; native deterministic reseed test and wall-clock ratchet; fixed pinned-Emscripten sequence; Apple plus `wasm32-unknown-unknown` runtime/browser-smoke builds; production provider/source audit; explicit pending DataConverterFormula consumer |

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
- [x] State-machine construction initializes one layer occurrence and runs
  that layer's entry callbacks before initializing the next authored layer.
  Pinned C++'s concrete Entry/Any `makeInstance` paths do not read
  state-machine inputs, so this ordering is locked by the literal
  `StateMachineLayerInstance::init` loop, a mutation-sensitive Rust
  construction observer, and a same-artifact C++/Rust entry-effect
  differential—not by inventing an input-consuming C++ constructor.
- [x] Weighted transition selection consumes one value from a real,
  occurrence-mediated `RandomProvider` seam, preserves authored candidate
  order, and accumulates both total and evaluated weights with exact C++
  `uint32_t` wrapping arithmetic. A waiting-for-exit result remains latched
  while the complete weighted candidate scan runs, but a later selected
  transition clears it after changing state and before returning, exactly as
  `StateMachineLayerInstance::tryChangeState` does.
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
- [x] Missing/out-of-range nested animationId: retain the authored
  NestedStateMachine and every ordered NestedInput with a null child
  occurrence; advance, pointer/hit, context, and state-change forwarding
  return the pinned empty results, and public clone reconstructs the same
  cold null occurrence.
- [x] Weighted random selection: injected nonzero draws can select later
  candidates, draw consumption is exact, and overflowing weights wrap as
  C++ `uint32_t` before selection. A wait-then-selected live differential and
  focused latch test prove the scan retains the wait while the successful
  state change clears it before returning
  (`state_machine_instance.cpp:412-468,528-627`).
- [x] Browser random boundary: the Rust browser target does not call
  unavailable libc symbols. Its target adapter reproduces Emscripten 3.1.61's
  exact 32-bit-unsigned seed subtraction before widening, wrapping LCG, shift,
  `RAND_MAX`, and monotonic-clock multiply/round/narrow evaluation order;
  fixed-sequence unit coverage including seed zero and an FP-reassociation
  discriminator plus the complete browser-smoke compile prevent a native-only
  proof from being published again.
- [x] Native clock boundary: ordinary layer construction samples the target
  standard library's actual high-resolution clock source
  (`CLOCK_MONOTONIC_RAW` for Apple libc++, `CLOCK_REALTIME` for pinned Rive's
  default Linux clang+libstdc++ build, `CLOCK_MONOTONIC` for Android/WASI and
  other Unix libc++ targets, and `QueryPerformanceCounter` on Windows); a
  structural mutation rejects Rust `SystemTime` or manual Unix-epoch seeding.
  An operating-system
  clock failure uses deterministic seed `1` rather than unwinding through an
  embedder/FFI boundary, and a focused test plus structural mutation lock that
  safety adaptation.
- [x] Multi-layer initialization: first-layer entry actions complete before
  second-layer initialization. A test-only Rust observer proves the rejected
  construct-all-first loop fails; the same artifact proves the C++/Rust entry
  effect and first-advance consumer result match. Source proof at
  `state_machine_instance.cpp:150-175,378-409,1747-1752`,
  `layer_state.cpp:62-66`, and `state_instance.cpp:4-8` establishes that the
  pinned Any/Entry constructors retain definitions only and do not themselves
  consume state-machine inputs.
- [x] Constructor facility order: entry callbacks run before C++ clones
  state-machine DataBinds and populates bindable lookup maps. Rust's
  constructor-phase executor therefore exposes inputs and reports but makes
  DataBind-backed ViewModel changes and triggers unavailable until layer
  initialization completes. The live differential proves an entry
  `ListenerViewModelChange` leaves the dormant source and bindable occurrence
  unchanged (`state_machine_instance.cpp:1747-1754,3189-3198`;
  `listener_viewmodel_change.cpp:42-80`).
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
9. a constant/random-transition fallback, saturating/widened weight
   accumulation, or candidate reordering in place of `RandomProvider` plus
   C++ `uint32_t` arithmetic;
10. filtering a NestedStateMachine because its child occurrence is null, or
    making the authored input list conditional on child creation;
11. constructing all layer occurrences before running any authored layer's
    entry callbacks.
12. storing `RandomProvider` on a state-machine occurrence instead of
    retaining C++'s process-global boundary.
13. seeding every layer from the clock and omitting C++ deterministic mode.
14. treating a missing child input name as absent instead of C++'s empty
    string during first-match nested-input lookup.
15. letting Rust's retained script-error surface abort construction of later
    authored layer occurrences.
16. clearing or overwriting the layer's waiting-for-exit latch inside the
    weighted transition scan.
17. calling libc `rand`/`srand` from the shared or Wasm provider instead of
    isolating those symbols in the native target adapter.
18. seeding the native provider from Rust `SystemTime` or manual Unix-epoch
    arithmetic instead of the target C++ standard library's actual
    `high_resolution_clock` source, including substitutions among Apple's
    `CLOCK_MONOTONIC_RAW`, pinned Linux's `CLOCK_REALTIME`, the other
    Unix/WASI `CLOCK_MONOTONIC` source, or Windows' performance-counter API.
19. introducing `panic!`, `unwrap`, or `expect` into the native random/clock
    adapter instead of using the non-unwinding deterministic fallback.
20. returning from a successful weighted transition without clearing a wait
    latched by an earlier candidate.
21. exposing DataBind-backed ViewModel facilities to entry callbacks before
    the constructor reaches C++'s DataBind-cloning phase.

The checker evaluates every ratchet against the complete source file, not one
line at a time. A dedicated multi-line mutation test prevents formatted Rust
from evading rules whose source/consumer spans several lines.

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
- Checker ratchets have injected negative controls for every forbidden shape
  listed above. File and member rows remain pending verification until
  the immutable whole-family candidate receives independent acceptance.

## Publication boundary

Use focused tests while editing. Run the expensive runtime/workspace-probe,
ordinary/scripted golden, renderer pixel, ABI, size, format/lint, structural,
and packaging floors once after every checklist row is closed on one frozen
candidate. Performance is deliberately deferred until every mapped
FL-A-through-FL-E code row is ported and the complete correctness/structure
floor is green. Publish one immutable SHA for one independent whole-family
verdict; do not submit a partial layer, nested, reset, or
transition-interruption slice.

The first semantic translation was
`93a902558ad9860e1ecaeeef8e710223841e2dca`; immutable candidate
`dbc57130dc23e93690dd7a9a9f500c8be699728c` was independently rejected
despite its green floor because it used a constant transition draw, dropped
null-child NestedStateMachine owners, and constructed every layer before
running entry callbacks. Corrected semantic commit
`57d08cfcdb8e870602544e30af9c53d6f7ac34b7` closes all three findings plus
the next review's two lifecycle findings: a successful random selection now
clears a previously retained wait latch after changing state, and initial
entry callbacks cannot observe DataBind facilities before C++ constructs
them. Live pinned-C++ differentials cover both lifecycle boundaries. The
self-excluding `docs/runtime-frame-loop-trace.json` records the exact
candidate-source fingerprint and runner provenance. Fingerprinted closure
prose deliberately does not duplicate those self-referential values. The
replacement candidate has the following gate receipt:

- runtime 521 / 521;
- probe-armed workspace and pinned-C++ probes 746 / 746;
- ordinary and scripted golden each 317 / 317 entries and 647 / 647 segments,
  zero divergences;
- same-runner pixel corpus 1,468 / 1,468, 1,370 byte-exact, zero divergences;
- C API, native Apple, browser build, lint, format, and diff checks;
- committed-tree size 8,034,536 bytes without scripting and 8,935,640 bytes
  with scripting, both below the 9 MiB limit;
- Apple XCFramework build/package/ABI/header/C/Swift checks, checksum
  `22a0309091624bd584566f529a5d52bcc19aa9f7e3d2e7c475f8c8e7f7b361cd`;
- structural checker 37 / 37 and all 21 injected negative controls.

Ordinary and scripted golden were rerun serially after a discarded concurrent
harness build replaced their shared executable. Native Apple was rerun with a
fully isolated stable toolchain after a discarded mixed-cache run. The
XCFramework's first invocation correctly refused a checker-created untracked
Python bytecode cache; the cache was removed and the clean-tree package run
above passed. A later invocation exhausted the generated Cargo target cache;
`cargo clean` reclaimed only generated artifacts and unchanged source then
passed the complete package verification. These are discarded
environment/harness attempts, not product failures. No performance
measurement was run.

The review packet is this checklist plus its direct C++ citations, focused
adversarial tests, structural ratchets, exact pushed candidate SHA, and the
gate results above. All five layer/state family rows and
`state_machine.layer` remain pending-verification until the independent
verdict. The supporting `src/math/random.cpp` row remains pending after that
verdict until its FL-D formula consumer is routed through the same provider.
