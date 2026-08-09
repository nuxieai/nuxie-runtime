# Product-neutral player scheduling contract

`NuxPlayerSchedulingInfo` is runtime evidence owned by one successful
`NuxPlayerStepResult`. It gives a host enough information to decide whether to
submit the current occurrence without introducing sessions, experiences,
screens, frame clocks, or SDK lifecycle policy into Rust.

The facts have deliberately separate meanings:

- `dirty` is true exactly when that committed operation changed observable
  runtime state. It says nothing about whether a previous render is still
  awaiting presentation.
- `settled` is true when no selected root or nested runtime occurrence has
  pending continuation. It is not derived from `keep_going`.
- `render_required` is persistent occurrence state. A new occurrence starts
  with a nonzero unpresented `render_revision`. Every committed visual/runtime
  change and renderer-domain reset increments that revision. Timeout,
  occlusion, zero-size, device-loss, failed, and skipped submissions leave it
  pending.
- `has_wake_deadline` makes the deadline optional. When true,
  `wake_deadline_clock` names the platform clock whose epoch defines the
  absolute nanosecond value in `wake_deadline_monotonic_ns`; domains and process
  epochs are never interchangeable. The current runtime has no timed
  deferred-work source, so it reports the field absent, the clock as
  `Unspecified`, and the value as zero. Immediate continuation is represented
  by `settled == false`; the ABI never fabricates a deadline from a host clock.

`NuxPlayerStepInfo.keep_going` retains the exact pinned C++
`advanceAndApply()` result. In particular, zero-time and static operations can
return true while scheduling is settled. Changing its meaning would break the
runtime oracle and is not permitted.

## Presentation acknowledgement

The Apple renderer acknowledges the revision it actually drew only after
`AppleSurface::present` returns `Presented`. Every other disposition preserves
the pending revision. A host using another presentation mechanism calls
`nux_player_acknowledge_presented(player, render_revision)` after successful
presentation. The acknowledgement is occurrence-scoped and accepts only the
exact current nonzero revision; a stale acknowledgement returns
`NUX_STATUS_HANDLE_MISMATCH` and cannot clear newer work.

Multiple players over one retained artboard occurrence share the revision and
presentation state, just as they share the mutable artboard. A renderer-domain
reset invalidates the occurrence even if a render was already pending, so an
in-flight acknowledgement from the prior domain becomes stale.

## ABI evolution

`NuxPlayerSchedulingInfo` is a caller-sized C struct. Callers set
`struct_size`; the runtime writes only its known prefix, rejects anything
smaller than `NUX_PLAYER_SCHEDULING_INFO_V3_MIN_SIZE`, and leaves unknown
suffix bytes untouched. A failed step owns diagnostics but exposes no
scheduling snapshot.

Swift may use the facts as inputs to its own policy: submit while
`render_required`, continue runtime work while not `settled`, and arm a timer
only when a wake deadline is present. Actor choice, display-link cadence,
drawable acquisition, visibility, retry/backoff, and product lifecycle remain
Swift responsibilities.
