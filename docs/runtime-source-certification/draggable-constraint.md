# Draggable constraint source certification

## Scope

This is a fresh literal audit against pinned upstream commit
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The following files were read
completely rather than inheriting the stale B6-0127 verdict:

- `src/constraints/draggable_constraint.cpp` (85 lines)
- `include/rive/constraints/draggable_constraint.hpp` (106 lines)
- `crates/nuxie-runtime/src/constraints/draggable_constraint.rs`
- the mapped proxy lifecycle in `crates/nuxie-runtime/src/constraints.rs`
- the mapped listener-group state in `crates/nuxie-runtime/src/listener_group.rs`
- the component-provided event owner in
  `crates/nuxie-runtime/src/state_machine/state_machine_instance/state_machine_instance.rs`

The pinned pair contains two out-of-line definitions and 16 executable inline
definitions. Pure-virtual declarations (`draggables`, `startDrag`, `drag`, and
`endDrag`) and the out-of-line `processEvent` declaration are not counted as
inline definitions.

## Out-of-line definitions

| Pinned definition | Literal behavior and side effects | Rust owner |
| --- | --- | --- |
| `DraggableConstraint::listenerGroups` | Iterates concrete draggable proxies in returned order; allocates a `componentProvided` single listener and one draggable listener group per proxy; reads the proxy hittable and opacity; for a non-null Component hittable, creates one `HitTarget` and appends one `ListenerGroupWithTargets`. | `runtime_draggable_proxies` creates a fresh typed proxy set per `StateMachineInstance`; `initialize_component_provided_groups` creates a draggable `ListenerGroup`, attaches it to the exact hittable, carries opacity, and retains provider order before hit sorting. Invalid/unresolved typed targets fail closed instead of retaining raw pointers. |
| `DraggableConstraintListenerGroup::processEvent` | Captures the previous pointer phase, runs base `ListenerGroup::processEvent`, then: ends the proxy on Down-to-Clicked/Out; if it had scrolled, recursively calls `StateMachineInstance::dragEnd`, clears `m_hasScrolled`, and returns `scroll`; starts the proxy on the first transition into Down and clears scroll state; on Move while Down, drags the proxy and, on the first successful drag, recursively calls `dragStart(position,time,false,pointerId)` before setting `m_hasScrolled`; every successful drag returns `scroll`; all other paths return `none`. | `process_listener_group_event` preserves the same phase branches and proxy start/drag/end mutations. A successful first drag or scrolled completion interrupts the outer Rust hit traversal, restores the temporarily taken group/hit owners, recursively invokes the complete state-machine drag traversal, retakes the recursively mutated owners, and resumes immediately after the triggering group. Completion clears `has_scrolled` only after `dragEnd`; `dragEnd` retains its final `pointerMove`. |

## Sixteen executable inline definitions

| # | Pinned inline definition | Observable contract | Rust ownership |
| ---: | --- | --- | --- |
| 1 | `DraggableProxy::~DraggableProxy` | Virtual empty base destructor. | Value-owned `RuntimeDraggableProxy` drops without a base cleanup side effect. |
| 2 | `DraggableProxy::isOpaque` | Defaults to `false`. | `RuntimeDraggableProxy::new` receives the concrete proxy's opacity; viewport and track defaults are false. |
| 3 | `DraggableProxy::hittable` | Returns the retained `m_hittable`. | The exact `ComponentHandle` is retained in `RuntimeDraggableProxy::hittable`. |
| 4 | `DraggableConstraint::DraggableConstraint` | Empty construction. | Concrete imported constraint state needs no additional base initialization. |
| 5 | `DraggableConstraint::direction` | Casts `directionValue()` to the three-value direction enum. | Runtime constraint math reads the same generated `directionValue` and interprets `0/1/2` as horizontal/vertical/all. |
| 6 | `DraggableConstraint::constrainsHorizontal` | True for horizontal or all. | Direction checks use `matches!(direction, 0 | 2)`. |
| 7 | `DraggableConstraint::constrainsVertical` | True for vertical or all. | Direction checks use `matches!(direction, 1 | 2)`. |
| 8 | `DraggableConstraint::hitComponents` | Always returns an empty vector; listener-group targets own hit registration. | Component-provided groups are added through `add_to_hit_lookup`; no second provider hit-component list is synthesized. |
| 9 | `DraggableConstraintListenerGroup` constructor | Initializes the base listener and retains constraint/proxy pointers; `m_hasScrolled` remains false. | `ListenerGroup::draggable(proxy_index)` and a cold `RuntimeDraggableProxy` retain the typed occurrence; `has_scrolled` starts false. |
| 10 | `~DraggableConstraintListenerGroup` | Deletes the synthetic listener and concrete proxy. | Both are instance-owned Rust values and drop with the occurrence. |
| 11 | `enable` | Intentional no-op. | `ListenerGroup::enable` now returns immediately for `Draggable`. |
| 12 | `disable` | Intentional no-op. | `ListenerGroup::disable` now returns immediately for `Draggable`. |
| 13 | `constraint` | Returns the retained constraint pointer. | `RuntimeDraggableProxy::constraint` retains the exact component handle. |
| 14 | `canEarlyOut` | Always false. | Adding a component-provided group forces its hit owner `can_early_out` false. |
| 15 | `needsDownListener` | Always true. | The component-provided hit owner is evaluated for Down through its non-early-out registration. |
| 16 | `needsUpListener` | Always true. | The component-provided hit owner is evaluated for Up through its non-early-out registration. |

## Demonstrated mismatches and corrections

### Missing global drag recursion

Rust previously performed only the local proxy mutation. On the first
successful component drag it did not call the containing state machine's
`dragStart(..., false, ...)`; on scrolled completion it did not call
`dragEnd` and returned a non-scroll result. That skipped nested and authored
DragStart/DragEnd listeners, pointer enable/disable traversal, and the final
pointerMove owned by `StateMachineInstance::dragEnd`.

The correction is the pinned recursion, not a synthesized local notification.
Because `update_listeners` temporarily takes `listener_groups` and
`hit_components` to satisfy Rust aliasing, the interrupted traversal records
its triggering group, restores both owner vectors, performs the recursive
state-machine call, retakes the mutated vectors, and resumes the same hit
target after that group. This preserves the C++ interleaving rather than
deferring the recursion until the outer event has finished.

### Draggable enable/disable must not mutate phase

The generic Rust listener-group methods previously changed draggable pointer
phase during drag recursion. The pinned subclass overrides both methods with
empty bodies. Rust now makes those methods literal no-ops only for
`ListenerGroupKind::Draggable`; authored and TextInput group behavior is
unchanged.

## Direct regression evidence

- `component_provided_scroll_recurses_drag_events_at_the_pinned_call_site`
  loads pinned `scroll_threshold.riv`, crosses the real viewport threshold,
  and observes `DragStart` before the interrupted outer Move. Its Up observes
  `DragEnd`, then the pinned final pointerMove, then the resumed outer Up; the
  completion returns `HitOpaque` (the Rust projection of `scroll`) and clears
  retained scroll state.
- `draggable_enable_and_disable_are_literal_no_ops` proves neither override
  creates pointer data, disables the pointer, nor consumes the group.
- `fl_c5_pointer_drag_discards_event_timestamps_then_follows_with_move`
  continues to prove ordinary `dragStart` uses the default
  `disablePointer=true`, while `dragEnd` enables before dispatch and performs
  its final timestamped Move.

## Certification boundary and verdict

**Independent adversarial review: rejected.** The first-drag recursion and
scrolled-completion recursion follow the pinned reentrant call site, including
restoring the temporarily taken owner vectors before recursion, resuming after
the triggering group, preserving `canHit` for the rest of that target, mapping
`scroll` to opaque blocking, and retaining `m_hasScrolled` until the recursive
`dragEnd` (and its final `pointerMove`) returns. The draggable-only no-op
`enable`/`disable` specialization also leaves authored-group behavior intact.
The following counterexamples prevent source certification:

1. The Rust event branches do not use the pinned previous/current phase
   predicates. Pinned C++ calls `startDrag` only for a transition from a
   non-Down phase into Down. Rust calls `runtime_draggable_proxy_start` for
   every hovered Down, even when that pointer was already Down. A repeated
   hovered Down therefore restarts the concrete proxy and clears
   `has_scrolled`; pinned C++ does neither. Conversely, pinned C++ calls
   `endDrag` for any Down-to-Clicked/Out transition. A Down outside the target
   after an earlier captured Down transitions the C++ pointer to Out and ends
   the proxy immediately. Rust's Down branch does nothing when not hovered,
   retains the pointer in `active_pointers`, and can continue dragging it on a
   later Move.
2. Rust clears proxy-global `has_scrolled` on Exit both in the draggable event
   branch and again in `release_draggable_pointer`. Pinned Exit does not change
   the phase inside `processEvent`; `releaseEvent` removes only that pointer's
   record and does not clear group-global `m_hasScrolled`. This is observable
   with two active pointers: after pointer A scrolls and exits while pointer B
   remains Down, B's next successful Move must not emit another `dragStart` in
   C++, but Rust emits one because A's Exit cleared the shared flag.
3. The `listenerGroups` owner row says Rust retains provider order before hit
   sorting, but `runtime_draggable_proxies` sorts proxies by hittable draw order
   before it constructs listener groups. Pinned C++ constructs groups in
   artboard provider order and each provider's `draggables()` order, then sorts
   hit components later. No governing adaptation or behavioral proof currently
   establishes that changing the owner-vector order is inert.

The existing direct regression uses one ordinary Down/Move/Up pointer. It
therefore proves the repaired happy-path recursion but cannot observe repeated
Down, Down-to-Out cancellation, or the two-pointer Exit state leak above.
Concrete proxy algorithms owned by `ScrollConstraint` and
`ScrollBarConstraint` remain dependencies and are not silently certified by
this translation-unit audit.

There is also a denominator tooling defect independent of the runtime verdict:
the generated symbol row for the inline
`DraggableConstraintListenerGroup` constructor is currently named
`DraggableConstraintListenerGroup::m_draggable`, having selected the final
initializer-list member instead of the constructor declarator. The receipt's
human count of 16 executable inline definitions is correct, but that malformed
machine ID cannot serve as constructor disposition evidence until the generic
initializer-list parser is corrected and the snapshot regenerated.
