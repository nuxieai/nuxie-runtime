# P3C uncommitted hunk map

The sandbox cannot create `/Users/levi/dev/worktrees/nuxie-mr-c14/.git/worktrees/nuxie-mr-c14/index.lock`, so it blocked both switching to `levi/p3c-semantics` and committing. The work remains as an uncommitted patch on the pre-existing worktree branch and is grouped below for the orchestrator.

## Coherent step 1 — shared semantic types

- `crates/nuxie-runtime/src/semantic_node.rs` — retained node identity, role/state/trait/dirt flags, bounds, structural manager-only mutation.
- `crates/nuxie-runtime/src/semantic_snapshot.rs` — complete incremental diff payload types and ordering contract.
- `crates/nuxie-runtime/src/lib.rs` — module registration and public re-exports for the phase-1 semantic API.

## Coherent step 2 — semantic manager

- `crates/nuxie-runtime/src/semantic_manager.rs` — manager-local ids/collisions, child mutation, lookup/focus resolution, label derivation and absorption, boundary flattening, stable visual ordering, full and incremental diff production, refresh-ordered dirty-boundary reconciliation, fail-fast rejection of unresolved boundary drains, and 23 focused tests.

## Coherent step 3 — data, provider, inference, and listeners

- `crates/nuxie-runtime/src/semantic_data.rs` — lazy node ownership, authored propagation, focus state, hidden/collapsed removal and re-add, inference/bounds refresh, dirt, and duplicate-preserving semantic listener callbacks.
- `crates/nuxie-runtime/src/semantic_provider.rs` — authored-data precedence and drawable/container/fallback bounds projection.
- `crates/nuxie-runtime/src/semantic_inference_registry.rs` — pinned Text rule and authored-order run concatenation.
- `crates/nuxie-runtime/src/state_machine/semantic_listener_group.rs` — retained SemanticData callback registration/unregistration, callback queueing, constraint-filtered invocation drain, attribution, and a focused lifecycle test. The StateMachineInstance splice that calls these hooks remains in the conflict queue.

## Coherent step 4 — correspondence and evidence

- `file-correspondence-manifest.toml` — B6-0327 through B6-0330 now name their direct Rust owners but remain pending under RF-33. Existing B6-0070 also needs to reopen when the orchestrator advances the frozen pending ratchet because its new callback owner is not yet called by production StateMachineInstance lifecycle code.
- `docs/runtime-frame-loop-ownership.toml` — no final hunk. A P3-c source set plus four pending rows was prepared and mechanically checked, but the current ledger is still the frozen `fl-e8-wave-candidate`; its validator requires `pending=0` and exact FL-E8 trace scope. The candidate hunk was removed and is queued for the orchestrator phase-advance conflict instead of leaving the shared ledger invalid.
- `test-correspondence-manifest.toml` — all seven semantic test files (94 cases) are accounted for as evidence-backed `partial` or explicitly `pending`, with exact #LT-1 needs.
- `docs/parity-gap-register.md` — F6 moves from absent to phase-1 partial.
- `P3C-report.md` — status, evidence, pending work, and conflict queue.

## Preserved unrelated workspace changes

These were present or changed concurrently and are not P3C work: `INT-report.md`, `crates/nuxie-runtime/src/constraints.rs`, `crates/nuxie-runtime/src/state_machine.rs`, and `crates/nuxie-runtime/src/text/raw_text_input.rs`.
