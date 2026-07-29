# FL-B Animation Owner-Family Translation Spec

This is the frozen pre-translation mini-map for FL-B. It binds the complete
KeyFrame-through-blend owner family before production code changes begin.

Pinned C++: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

Accepted dependency: FL-A promotion
`f86d5ba0146697abc996310c62fa45e1f053144b`.

Next-wave main boundary:
`e72323c808b91d706ba3b745396beaca7accd69a`.

FL-B boundary merge:
`b5d5bc8afeaa0369cbc248b85366111649cb9010`.

## Finite closure

The executable ledger contains 45 FL-B C++ file rows and these eight pending
member rows:

1. `keyframe.seconds`
2. `keyframe.values`
3. `keyed_property.targets`
4. `keyed_object.membership`
5. `linear_animation.definition`
6. `linear_animation.instance`
7. `animation.reset`
8. `blend_animation.owners`

The frozen 45-file membership includes
`src/importers/keyed_property_importer.cpp`. The earlier provisional use of
`src/animation/scripted_listener_action.cpp` as the 45th row is deliberately
superseded: that owner belongs to FL-C's listener/action family and remains
assigned there. Reacceptance must preserve this exact membership rather than
moving `scripted_listener_action.cpp` back into FL-B.

The production Rust closure is deliberately narrow:

- `crates/nuxie-runtime/src/animation.rs`
- `crates/nuxie-runtime/src/artboard.rs`
- `crates/nuxie-runtime/src/state_machine.rs`
- `crates/nuxie-runtime/src/state_machine/animation_reset_factory.rs`
- `crates/nuxie-runtime/src/state_machine/blend_state_direct_instance.rs`

The two filename-corresponding state-machine submodules are incremental
FLR-16 extractions of the touched reset-factory and direct-blend-instance
owners; their surrounding FL-B families remain coordinated by
`state_machine.rs`.

Tests, the frame-loop ledger/checker, and evidence documents may change with
the owner family. Renderer backend code and later FL-C/FL-D/FL-E production
owners do not.

## Binding safety adaptations

Pinned C++ declares `LinearAnimationInstance::m_didLoop` without a constructor
initializer and exposes `didLoop()` before the first `advance`
(`include/rive/animation/linear_animation_instance.hpp:97-100,150`;
`src/animation/linear_animation_instance.cpp:17-30,193-198,356`).
There is no defined C++ value to translate at that point.

The user decision on 2026-07-25 applies FLR-3 as follows:

- safe Rust initializes `did_loop` to `false`;
- the pre-first-advance value is an explicit binding adaptation, not a claim
  that pinned C++ defines `false`;
- every `advance` path writes the exact C++ result;
- Rust does not emulate indeterminate memory, add `Option<bool>`, or break the
  existing public boolean API.

Pinned `AnimationResetFactory` serializes signed color `int` values through
`float`, and `AnimationReset::apply` converts the decoded float back to `int`
(`src/animation/animation_reset_factory.cpp:126-168`;
`src/animation/animation_reset.cpp:30-35,54-67`). Rust retains and replays
that exact float representation for every defined result. For the narrow
positive range whose float representation is 2^31, C++ float-to-int conversion
is undefined; user-approved project decision D2 applies Rust's saturating
conversion. FL-G10 records this boundary so it is never mislabeled as C++
behavior.

## Atomic production lanes

### FL-B1 — KeyFrame and keyed definition ownership

C++ files:

- `src/animation/keyframe.cpp`
- every `src/animation/keyframe_*.cpp` value/callback/interpolator subclass
- `src/animation/keyed_property.cpp`
- `src/animation/keyed_object.cpp`
- `src/importers/keyed_property_importer.cpp`

Retention boundary:

- each concrete KeyFrame occurrence retains `m_seconds` at attachment;
- one KeyedProperty owns one insertion-ordered concrete-frame sequence;
- interpolator and callback identity stay on that occurrence;
- dirty and clean validation preserve C++ loop separation and erase order.

Displaced Rust removed in this lane:

- per-read `frame / fps` calculations and zero-fps results;
- six parallel type-specific keyframe vectors;
- repeated property-family redispatch and duplicate source snapshots.

Primary citations:

- `include/rive/animation/keyframe.hpp:9-16`
- `include/rive/animation/keyed_property.hpp:47-49`
- `src/importers/keyed_property_importer.cpp:13-17`
- `src/animation/keyed_property.cpp:13-177`
- `src/animation/keyed_object.cpp:20-91`

### FL-B2 — LinearAnimation definition and occurrence

C++ files:

- `src/animation/linear_animation.cpp`
- `src/animation/linear_animation_instance.cpp`
- nested/simple/remap/linear animation owners in the FL-B ledger

Retention boundary:

- the Artboard definition arena is the one logical LinearAnimation owner;
- each occurrence retains a typed definition handle rather than a cloned
  `RuntimeLinearAnimation`;
- ordered KeyedObjects and KeyedProperties remain definition-owned;
- construct, time, advance, apply, copy, reset, report, and teardown follow
  the pinned initializer lists and call order.

Displaced Rust removed in this lane:

- per-occurrence cloned animation descriptors;
- `Option<u64>` loop normalization instead of the C++ raw `int` sentinel;
- FL-B-local fps/range guards absent from the pinned source;
- copy/drop behavior that retains state omitted by the C++ copy constructor.

Primary citations:

- `include/rive/animation/linear_animation.hpp:14-18`
- `include/rive/animation/linear_animation_instance.hpp:38-39,140-182`
- `src/animation/linear_animation.cpp:27-150`
- `src/animation/linear_animation_instance.cpp:17-79,193-356`

### FL-B3 — AnimationReset factory lifecycle

C++ files:

- `src/animation/animation_reset.cpp`
- `src/animation/animation_reset_factory.cpp`
- `src/animation/property_recorder.cpp`

Retention boundary:

- first-seen object/property order is retained;
- only double/color values enter the reset;
- optional first-keyframe baseline uses the exact retained occurrence;
- the global factory owns, clears, and reuses released reset instances.

Displaced Rust removed in this lane:

- lifecycle-inaccurate reset clones;
- linear membership scans used as an ownership substitute;
- drop-only reset disposal where C++ returns a cleared owner to the pool.

Primary citations:

- `src/animation/animation_reset.cpp:8-70`
- `src/animation/animation_reset_factory.cpp:10-235`

### FL-B4 — Blend definition and occurrence ownership

C++ files:

- BlendAnimation, BlendState, 1D/direct definition and instance rows
- AnimationState and AnimationStateInstance rows
- associated blend inputs, ViewModel sources, and transitions

Retention boundary:

- BlendState uniquely owns ordered BlendAnimation definitions;
- each BlendAnimation retains one LinearAnimation definition pointer or the
  shared empty definition;
- each state occurrence owns one ordered vector of embedded animation
  instances;
- 1D `from`/`to` identities point into that stable occurrence vector;
- reset construction, application, and release follow the owning transition.

Displaced Rust removed in this lane:

- cloned definition payloads beside occurrence state;
- rediscovered from/to occurrences;
- reset ownership or update ordering not present in pinned C++.

Primary citations:

- `include/rive/animation/blend_state.hpp:15-21`
- `include/rive/animation/blend_state_instance.hpp:18-128`
- `src/animation/blend_state_1d_instance.cpp:9-150`
- `src/animation/blend_state_direct_instance.cpp:11-62`

## Focused correspondence matrix

The focused tests are lifecycle evidence, not replacements for the full
floors.

- KeyFrame seconds are fixed at attachment even if definition fps later
  changes.
- Null KeyFrame import adds no slot.
- Dirty validation erases unsupported properties/objects in C++ order and
  preserves the first effective status.
- One hostile mixed-concrete-type sequence proves there is one ordered frame
  owner, not parallel vectors.
- Binary-search cases cover duplicate seconds, exact offsets, both
  directions, start, and post-end.
- Full double/color mix performs no current-value read; partial mix performs
  exactly one.
- Two animation instances sharing one definition retain isolated keyframe
  holders and shared definition identity.
- Standalone, nested, AnimationState, and blend occurrences all retain the
  exact Artboard definition handle across clone/remount.
- Time, raw loop override, one-shot, loop, reverse-loop, ping-pong,
  multi-boundary callback ranges, spilled time, zero delta, reset, and copy
  match the pinned probe.
- Pre-first-advance `did_loop` is the approved `false` adaptation; every
  post-advance value is C++-exact.
- Reset acquisition/release proves stable order, no stale entries, and pool
  reuse.
- Blend tests cover invalid definition fallback, zero-mix advance, effective
  zero-mix apply skip, 1D duplicate/bounds search, direct source validation,
  clamping, and reset lifetime.

## Structural deletion ratchets

The FL-B checker must reject:

- KeyFrame `seconds(..., fps)` read-time APIs;
- parallel `key_frames`, `color_key_frames`, `bool_key_frames`,
  `uint_key_frames`, `string_key_frames`, and `callback_key_frames` storage;
- per-occurrence `RuntimeLinearAnimation` descriptor ownership;
- `LinearAnimationInstance::loop_value: Option<u64>`;
- FL-B-local zero-fps/range guards absent from pinned C++;
- reset/blend definition copies that coexist with their C++-corresponding
  owner.

Each negative control injects one forbidden form and proves the checker fails.

## Landing and acceptance

Every lane runs focused runtime tests plus the probe-armed workspace, ordinary
and scripted 317/317 + 647/647 zero-failure floors, and the structural checker.
The complete wave additionally runs the 1,468-row pixel referee, C API, Apple
product/release checks, lint/format/diff, and committed-tree 9 MiB size gate.
Rows remain pending until the entire 45-file/eight-member family is green and
the orchestrator independently verifies the candidate.

One canonical whole-corpus `perf-hot-loop` checkpoint is recorded after FL-B.
Its result is acceptance evidence only; it does not reorder the remaining
owner waves.
