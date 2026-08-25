# `ListPath` paired audit

Upstream owner: `src/shapes/list_path.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

Rust owners:

- `crates/nuxie-runtime/src/shapes/list_path.rs` owns each dynamic row's
  listener, subscriptions, and synthetic detached-cubic vertex.
- `crates/nuxie-runtime/src/artboard.rs` owns occurrence construction, cold
  clone, reconciliation, dirty propagation, and projected vertices.
- `crates/nuxie-runtime/src/data_bind/data_bind_context.rs` delivers list
  structure changes and flushes live property notifications in mutation order.
- `crates/nuxie-runtime/src/draw.rs` consumes the projected vertices through
  the ordinary `ListPath` path-building pipeline.

Verdict: adapted and behaviorally equivalent under Rust ownership.

The paired audit verified the 12 pinned symbol mappings; degrees-to-radians
conversion; single, multi, and point listeners; initial writes; dependent
registration/removal; positional reuse and remap; null-instance row skipping;
duplicate source instances; tail removal; unconditional path dirt after valid
reconciliation; cold clone; and synthetic `CubicDetachedVertex` control-cache
lifecycle. Rust's invalid-input boundary clears stale rows safely instead of
following the C++ null precondition, without claiming a successful update.

The live `list_to_path.riv` pinned-C++ probe plus Rust production fixture covers
initial empty ownership, four-row XY geometry, reorder, duplicate rows,
same-count replacement, old-source unsubscription, live replacement mutation,
tail shrink, empty rendering, and clone isolation. This supersedes F10's stale
claim that dynamic subscriptions/remap and detached-cubic occurrences were
absent.
