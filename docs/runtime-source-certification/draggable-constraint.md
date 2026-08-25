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

The corrected 7,818-unit denominator assigns 19 authority units to this pinned
pair: two out-of-line functions in the `.cpp`, 16 executable inline functions
in the handwritten header, and the header's include-guard macro. The guard
(`include/rive/constraints/draggable_constraint.hpp::c7a31d185f145f5e:1`) is
preprocessor/build structure with no runtime side effect to translate. The 18
function rows are adjudicated below. Pure-virtual declarations (`draggables`,
`startDrag`, `drag`, and `endDrag`) and the out-of-line `processEvent`
declaration are not executable denominator units.

## Out-of-line definitions

| Pinned definition | Literal behavior and side effects | Rust owner |
| --- | --- | --- |
| `DraggableConstraint::listenerGroups` | Iterates concrete draggable proxies in returned order; allocates a `componentProvided` single listener and one draggable listener group per proxy; reads the proxy hittable and opacity; for a non-null Component hittable, creates one `HitTarget` and appends one `ListenerGroupWithTargets`. | `runtime_draggable_proxies` creates a fresh typed proxy set per `StateMachineInstance` in artboard object/provider order and each provider's proxy order; `initialize_component_provided_groups` creates a draggable `ListenerGroup`, attaches it to the exact hittable, and carries opacity. Hit components are sorted only at the later state-machine hit-sort boundary. Invalid/unresolved typed targets fail closed instead of retaining raw pointers. |
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

### Decisions use previous/current pointer phase

The corrected branch now consumes `ListenerGroup::process`'s exact phase
transition. `startDrag` runs only for non-Down to Down, `endDrag` runs for
Down to Clicked/Out regardless of whether the event was Up or Down, and
`drag` runs only for Move while the resulting phase is Down. A repeated
hovered Down therefore does not restart the proxy, while a Down outside an
existing capture ends it.

### Pointer release does not clear group-global scroll state

Exit still releases the individual pointer record and Rust's auxiliary active
pointer bookkeeping, but it no longer clears the proxy/group's shared
`has_scrolled`. This matches `ListenerGroup::releaseEvent`, which does not
touch `DraggableConstraintListenerGroup::m_hasScrolled`, and preserves the
two-pointer behavior until a new start transition or scrolled completion
clears the group-global bit.

### Provider order and hit order remain separate

The proxy constructor no longer sorts by draw/hit order. It retains the
pinned artboard-object provider order and each concrete provider's
`draggables()` order. The existing state-machine hit-sort pass still orders
the separately registered hit components afterward. This claim is specifically
the relative order of draggable providers: the other pinned
`ListenerGroupProvider` implementation, `ScriptedDrawable`, returns no listener
groups and contributes only a hit component, so it cannot interleave a
listener group into this owner vector.

### Recursive script execution context is not preserved

The restored recursion still differs at the Rust execution boundary. The outer
`update_listeners` call receives the caller's `ScriptHost`, optional owned view
model context, and event context. On the first successful proxy drag it calls
`drag_start_with_pointer_disable`; on scrolled completion it calls `drag_end`.
Both helpers recursively call `update_listeners` with `NoopScriptHost` and
`None` contexts. `drag_end` then performs its required final pointer Move via
the public `pointer_move` path, which also uses the no-op host/context path.

That is observable, not just a Rust ownership detail. An authored DragStart,
DragEnd, or final-Move scripted listener reached by component-provided
recursion can call `ScriptHost::mark_script_update`; the mark is delivered to a
temporary no-op host instead of the host supplied to
`try_pointer_move_with_timestamp_and_script_host` or the corresponding Up
entry point. If the supplied host requires atomic callbacks, an ordinary inner
script error is also swallowed under the no-op host's non-atomic policy rather
than being returned by the outer fallible pointer call. Pinned C++ re-enters
the same `StateMachineInstance` synchronously and does not replace the active
script execution environment between the interrupted event and nested
DragStart/DragEnd traversal.

## Direct regression evidence

- `component_provided_scroll_recurses_drag_events_at_the_pinned_call_site`
  loads pinned `scroll_threshold.riv`, crosses the real viewport threshold,
  and observes `DragStart` before the interrupted outer Move. Its Up observes
  `DragEnd`, then the pinned final pointerMove, then the resumed outer Up; the
  completion returns `HitOpaque` (the Rust projection of `scroll`) and clears
  retained scroll state.
- `draggable_enable_and_disable_are_literal_no_ops` proves neither override
  creates pointer data, disables the pointer, nor consumes the group.
- `draggable_uses_phase_transitions_for_repeated_and_outside_down` proves a
  repeated Down does not restart a proxy and a captured Down-to-Out transition
  ends it even though the second event is not Up.
- `releasing_one_pointer_does_not_clear_group_global_scroll_state` proves Exit
  releases only that pointer, retains the shared scroll bit, and prevents the
  remaining Down pointer from emitting a second DragStart.
- `draggable_proxy_lifecycle_matches_cpp_owner_state` now asserts the concrete
  provider order `Viewport`, `Thumb`, `Track` before the independent hit-sort
  pass.
- `fl_c5_pointer_drag_discards_event_timestamps_then_follows_with_move`
  continues to prove ordinary `dragStart` uses the default
  `disablePointer=true`, while `dragEnd` enables before dispatch and performs
  its final timestamped Move.

## Certification boundary and verdict

**Independent adversarial re-review: rejected.** Commit `90cfcde3a` correctly
repairs all three counterexamples it set out to repair: decisions now use the
pinned previous/current phases, Exit preserves group-global scroll state, and
group construction retains draggable-provider order until hit sorting. The
draggable-only no-op enable/disable overrides, proxy start/drag/end order,
scroll blocking result, recursive traversal position, and final pointer Move
also match for the no-script-host path exercised by the focused tests.

Certification remains red because the recursive helpers replace the caller's
script host and supplied contexts, including on `drag_end`'s final Move. The
current end-to-end fixture installs `NoopScriptHost` and observes a synthetic
`RecordingHitComponent`, so it cannot detect the lost scripted side effects or
atomic error policy. The repair must thread the active host/context through the
nested DragStart/DragEnd calls and through the final Move, then prove the
production component-provided path with an authored scripted listener and a
non-no-op host. This is a state-machine invocation-owner correction, not a
local draggable-proxy workaround.

Concrete proxy algorithms owned by `ScrollConstraint` and
`ScrollBarConstraint` remain dependencies and are not silently certified by
this translation-unit audit. The formerly malformed constructor machine ID is
now the valid denominator row
`include/rive/constraints/draggable_constraint.hpp::35b4670e57c27261:1`.
