# State Machine Bindable Blend Source Audit

Date: 2026-06-29

This document continues roadmap item `#11` after random transition selection
from supported blend states. It audits the two remaining authored blend sources
that currently look like "view-model/data-bind blend" work and identifies the
next runtime seam to build before admitting either source.

## C++ Surface

The pending blend sources are:

| C++ object | Source files | Runtime behavior |
| --- | --- | --- |
| `BlendState1DViewModel` | `include/rive/animation/blend_state_1d_viewmodel.hpp`, `src/animation/blend_state_1d_viewmodel.cpp`, `src/animation/blend_state_1d_instance.cpp` | Imports only when a `StateMachineImporter` and latest `BindablePropertyImporter` exist. At advance time, `BlendState1DInstance` asks `StateMachineInstance::bindablePropertyInstance(...)` for the per-instance property clone, requires `BindablePropertyNumber`, and uses `propertyValue()` as the 1D blend value. |
| `BlendAnimationDirect` with `blendSource == dataBindId` | `include/rive/animation/blend_animation_direct.hpp`, `src/animation/blend_animation_direct.cpp`, `src/animation/blend_state_direct_instance.cpp` | Imports only when the latest `BindablePropertyImporter` exists, stores the imported `BindableProperty*`, and at advance time asks `StateMachineInstance::bindablePropertyInstance(...)` for the per-instance clone. If that clone is a `BindablePropertyNumber`, the direct mix is `clamp(propertyValue / 100, 0, 1)`. |

Important generated-definition detail:

- `BlendState1DViewModel.bindablePropertyId`,
  `BlendAnimationDirect.bindablePropertyId`, and
  `BindableProperty.dataBindId` are marked `runtime: false` in `dev/defs`.
- The C++ generated base classes do not deserialize those fields into runtime
  members.
- Runtime import relies on the current import-stack context, especially the
  latest `BindablePropertyImporter`, rather than directly reading those ids.

## Owning Runtime Seam

C++ does not read these blend values from a view-model directly. It reads them
from a per-state-machine-instance `BindablePropertyNumber` clone:

- `StateMachineInstance` clones each state-machine-owned `DataBind`.
- When the data-bind target is a `BindableProperty`, it clones that bindable
  property and records `original BindableProperty* -> cloned BindableProperty*`
  in `m_bindablePropertyInstances`.
- `StateMachineInstance::bindablePropertyInstance(...)` looks up that clone.
- Data binding may later mutate the clone, but the blend-state source only
  needs the clone and its current `propertyValue()`.

This means the first Rust runtime seam should not start with full view-model
binding. It should start with per-instance bindable-property clones for
state-machine-owned data binds that target `BindablePropertyNumber`.

## Current Rust State

Already available:

- `rive-schema` exposes `BlendState1DViewModel`, `BlendAnimationDirect`,
  `BindablePropertyNumber`, `DataBind`, and related property metadata.
- `rive-binary` tracks C++ import-stack acceptance for
  `BlendState1DViewModel` and `BlendAnimationDirect.blendSource == 2`.
- `rive-binary` already projects state-machine-owned data binds and identifies
  bindable-property targets as state-machine-owned.
- `rive-runtime` supports `BlendState1DInput`.
- `rive-runtime` supports `BlendStateDirect` with direct input and authored
  `mixValue` sources.

Missing before runtime admission:

- Runtime storage for imported bindable-property facts needed by state-machine
  instances.
- Per-instance `BindablePropertyNumber` clones seeded from imported
  `propertyValue`.
- A lookup from authored bindable-property identity to the per-instance clone.
- `BlendState1DViewModel` runtime construction that records the authored
  bindable-property identity.
- `BlendAnimationDirect.blendSource == dataBindId` construction that records
  the authored bindable-property identity instead of dropping the blend
  animation.
- C++ probe coverage that proves both sources read the per-instance
  `BindablePropertyNumber.propertyValue`.

## Scope Lock For The Next Slice

The next implementation slice should support only static imported
`BindablePropertyNumber.propertyValue` reads for blend sources:

- `BlendState1DViewModel` should use the per-instance bindable number value as
  the 1D blend value.
- `BlendAnimationDirect.blendSource == dataBindId` should use the per-instance
  bindable number value divided by `100` and clamped to `[0, 1]`.
- The source bindable property must be reachable through the same import-stack
  shape C++ accepts.
- The per-instance clone must exist only when a state-machine-owned `DataBind`
  targets the authored bindable property, matching C++'s ownership path.

The next slice must not implement:

- Live `ViewModelInstance` APIs.
- Binding or rebinding an external data context.
- Data-bind update queues.
- Data converters.
- Source-to-target or target-to-source mutation.
- Listener-owned data binding.
- Bindable-property types other than `BindablePropertyNumber`.
- Nested artboard data-context propagation.
- `ListenerViewModelChange`, `StateMachineFireTrigger`, or view-model
  transition conditions.

## Suggested First Probe

Build a synthetic state machine with:

- A `BindablePropertyNumber` with an authored `propertyValue`.
- A state-machine-owned `DataBind` targeting that bindable property, so C++
  creates a per-instance bindable clone.
- One `BlendState1DViewModel` fixture that blends two animations from the
  authored number value.
- One `BlendStateDirect` fixture with a `BlendAnimationDirect` whose
  `blendSource` is `dataBindId`.

The C++ comparison should cover state-machine advance reports and final
component state, but should not bind an external view-model instance or run a
live data-bind update path.

## Admission Rule

Before adding behavior to this seam, answer:

1. Does it only read an imported `BindablePropertyNumber.propertyValue` from a
   per-state-machine-instance clone?
2. Does C++ reach that clone through a state-machine-owned `DataBind` targeting
   the authored bindable property?
3. Can the behavior be compared with the existing C++ runtime probe without
   live data contexts, converters, listeners, nested artboards, or rendering?

If the answer is no to all three, leave it for the later data-binding runtime
work.
