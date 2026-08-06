# Flow and CommandServer equivalence decision

UNIV-1631 asks whether the completed C++ `CommandQueue`/`CommandServer` port can replace product-owned `FlowSession` compensation machinery. The answer is **no** at the transaction boundary. A scalar write reaches the same runtime value, but the protocols have different ordering, rollback, scheduling, and failure contracts.

This decision removes no Flow machinery. The evidence does not prove any Flow-only clone, graph-copy, transaction-transfer, settlement, or host-effect-cycle hook redundant.

## Reproduce the evidence

The test uses one synthetic in-memory `.riv` file for both paths. It has a root view model and one boolean property, so the comparison does not depend on the upstream Rive checkout or ignored fixtures.

```sh
cargo test -p nuxie-product --test flow_command_equivalence
cargo run -p nuxie-product --example flow_command_equivalence -- 100
```

The test asserts the semantic decisions. The executable reports diagnostic latency and process-allocation counts; those numbers are deliberately not a pass/fail gate.

On an Apple Silicon development build on 2026-08-05, 100 complete set/read iterations measured:

| path | ns/iteration | allocations/iteration |
|---|---:|---:|
| Flow | 32,222 | 114.99 |
| CommandServer | 8,700 | 17.06 |

Each timed path creates its fixture once, then runs the requested number of complete set/read iterations. The per-iteration figures therefore include amortized setup. They describe this build and machine only; they do not establish substitutability.

## Responsibility decisions

| responsibility | decision | evidence and consequence |
|---|---|---|
| Scalar value mutation | Equivalent in isolation | Both paths set and read the same boolean value on the same synthetic file. This proves only the runtime primitive, not the surrounding protocol. |
| Exact output phases | Non-equivalent | Flow returns typed outputs synchronously, with `sequence`, `cycle`, and `FlowOutputPhase`. CommandServer listeners see nothing until both `process_commands()` and `process_messages()` have run. Removing Flow ordering would change observable product behavior. |
| Atomic rollback | Non-equivalent | A Flow batch containing one valid write and one missing-property write rejects the whole batch and retains the old value. CommandServer processes the valid command, reports an error for the invalid command, and retains the successful write. Flow graph cloning and transaction transfer stay. |
| Wake scheduling | Non-equivalent | Flow returns `wake_after_seconds` as part of the operation result. CommandQueue exposes command availability through a `Condvar` and emits settlement events; neither is an equivalent product follow-up deadline. Flow wake policy stays. |
| Terminal errors | Non-equivalent | `post_mutation_advance_failure_terminally_poisoned_session_rejects_every_later_operation` proves that Flow poisons a session after a post-mutation result failure. CommandServer's command tests and the mixed-validity harness prove that an error is reported while later commands continue. Flow's terminal-failure guard stays. |
| Wasm suitability | Non-equivalent host model | Flow is a caller-driven synchronous object. CommandQueue owns `Arc<Mutex<_>>`, `Condvar`, and a separately driven server loop. A local `wasm32-unknown-unknown` check could not run because the active Rust toolchain lacked that target's `core`/`std`, so target compilation remains unclaimed. Even if both compile, the blocking/server-loop contract is not a replacement for Flow scheduling. |
| Latency | Deferred as a design criterion | The executable measures both paths, but performance cannot override the semantic differences above. Re-measure in release mode only if a future adapter first proves equivalent behavior. |
| Allocations | Deferred as a design criterion | The executable counts allocator calls for both paths. The count is useful for later adapter work but platform/profile dependent. |
| Flow-only graph and host-cycle machinery | Non-equivalent | Clone/copy/transfer hooks stage the atomic candidate, and host-effect checkpoints make effects commit with the operation. CommandServer's per-command continuation has no equivalent transaction to inherit these responsibilities. No thinning is justified. |

## Boundary decision

`CommandQueue` and `CommandServer` remain the direct baseline port. Their command-owned artboard bounds, event properties, list and name-path operations, and imported-runtime primitives remain intact in `nuxie`.

`FlowSession` remains a product protocol in `nuxie-product`. It owns batching, synchronous output ordering, atomic rollback, wake policy, terminal failure, settlement aggregation, and host-effect commit cycles. Product hosts may build adapters on CommandServer later, but an adapter must first reproduce these contracts and extend the harness before removing any Flow machinery.
