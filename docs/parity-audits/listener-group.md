# `ListenerGroup` paired audit

Upstream owner: `src/listener_group.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

Rust owner: `crates/nuxie-runtime/src/listener_group.rs`, with orchestration in
`state_machine/state_machine_instance/state_machine_instance.rs`.

Verdict: adapted and behaviorally equivalent under Rust ownership.

- Each group owns pointer records keyed by pointer id plus a reuse pool.
  Release clears hover, phase, position, and Rust-only capture state before
  pooling without changing group-global consumption or drag state.
- Reset, hover, enable, disable, click phases, entry-position reset, previous
  position, and group-global drag state follow the pinned ordering.
- Hover, click/direct, and drag action selection preserves C++ overwrite
  precedence. Performing an action marks the machine for advance and consumes
  the group.
- A drag-ending up does not click. Pinned `dragEnd` recursively dispatches
  `DragEnd` and `Move`, resetting Clicked to Out before the outer group resumes;
  Rust preserves that observable reset explicitly across participating groups.
- `canEarlyOut` and down/up requirements are accumulated from the same
  listener types. Scroll and scripted component providers are represented by
  their direct Rust hit-component/provider owners rather than C++ inheritance.

The Rust-only captured event context retains synchronous invocation context;
it is an ownership adaptation, not a polling, generation, or reconstruction
mechanism.
