# FL-C5 StateMachine / StateMachineInstance closure

Planning boundary only. The complete pinned-source walk has not begun, so this
file authorizes no semantic Rust edit and makes no fidelity claim. The next
session must read the complete headers and both complete sources before
expanding this into the binding member-by-member closure checklist:

- `include/rive/animation/state_machine.hpp`
- `include/rive/animation/state_machine_instance.hpp`
- `src/animation/state_machine.cpp`
- `src/animation/state_machine_instance.cpp`

The direct Rust destinations must be filename-corresponding focused modules;
the existing giant `state_machine.rs` and `state_machine/instance.rs` files
must become thinner entry points as each touched owner moves.

## Required adversarial rows

- [ ] Definition import and collection ownership: map every definition
  collection, importer handoff, index/name lookup, and validation outcome.
- [ ] Occurrence construction order: prove layer, listener, DataBind, focus,
  hit, and callback-facility availability in exact constructor order.
- [ ] Ordered duplicates and nullable slots: retain authored occurrence order,
  duplicates, supported nulls, and malformed-import behavior.
- [ ] Transition search and state change: prove retained search order,
  interruption, exit waiting, random selection, and state-change side effects.
- [ ] Hit listener and focus ownership: prove hit lookup construction,
  sorting, nested ownership, focus-node timing, and dispatch order.
- [ ] DataContext bind rebind and clear: prove inherited/local contexts,
  scripted-object visits, source replacement, and teardown.
- [ ] Event application and chained reports: prove queue swaps, event-before-
  ViewModel order, chained completion, and deferred batches.
- [ ] Zero-second and floating-point edges: cover zero duration, NaN,
  infinities, signed zero, and zero-second frame timing.
- [ ] Advance return and pending work: prove raw advance versus facade return,
  forced zero-second continuation, pending reports, and needs-advance state.
- [ ] Keyframe DataBind lifecycle: prove occurrence construction, enrollment,
  live resolution, advancement, removal, and ordering.
- [ ] Clone remount and teardown isolation: distinguish Rust snapshot
  adaptation from cold remount and prove registrations/queues do not alias.
- [ ] Direct C++ file correspondence: create/use
  `state_machine/state_machine.rs` and
  `state_machine/state_machine_instance.rs`, preserving APIs by re-export.
- [ ] Permanent structural ratchets: turn every confirmed omission or
  replacement shape into a checker rule or live differential.

No performance measurement belongs in this family.
