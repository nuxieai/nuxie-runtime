# Renderer ports preparation status

Date: 2026-08-22

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Result: **RED; translation is not admitted**

## Frozen denominators

- 200 unique pinned source candidates: Vulkan 40, WebGPU 32, WebGL2 41,
  shared shader/build authority 87.
- 200 exclusive source-to-target assignments across 135 complete ownership
  units. No source omission, duplicate owner, or target collision is allowed.
- Every WebGPU semantic owner is a new translation target. The legacy
  Rust-WGPU implementation is diagnostic evidence and a post-closeout deletion
  target, not the port target.
- 924 source dependency occurrences, including 227 owned-source edges, 440
  generated-from-owned-source edges, 107 pinned external source edges, 129
  SDK/system edges, 18 tool-module edges, and 3 generated ASTC header edges.
- A dependency-first ownership-unit order computed after collapsing include
  cycles. It includes all 135 ownership units, including units with no internal
  edge.
- Product intent: exact Vulkan, WebGPU, and WebGL2 ports; explicit WebGPU/WebGL2
  selection in the web editor; legacy Rust-WGPU deletion only after all three
  exact ports pass frozen closeout.

The source, ownership, dependency, and unit-order ledgers are generated from
the pinned upstream checkout. Their checks fail on source drift, stale output,
duplicate ownership, overlapping targets, unresolved quoted dependencies, or
an incomplete unit graph.

## Remaining preparation blockers

1. Freeze all generated commands, tool identities, artifacts, and output
   digests, including the ASTC generated header's separate authority.
2. Derive and review all state-bearing records and fields under every backend
   configuration; then freeze construction, borrow, callback, synchronization,
   thread, failure, and destruction rules.
3. Freeze every preprocessor/configuration branch, backend extension/feature
   query, and shader specialization combination.
4. Freeze exact primary-oracle builds, corpus membership and exclusions,
   adapter/device/browser identity, and the platform/hardware matrix.
5. Freeze rooted product artifacts, explicit editor selection, forbidden
   unselected routes, and the legacy Rust-WGPU deletion contract.
6. Add one fail-closed preparation check that joins every independent ledger
   and rejects a queue transition while any denominator is incomplete.

No compiler integration, fixture-driven implementation, or backend translation
is allowed while this report remains red.
