# Renderer ports preparation status

Date: 2026-08-22

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Result: **GREEN; translation admission may begin after this barrier is checkpointed**

## Frozen denominators

- 200 unique pinned source candidates: Vulkan 40, WebGPU 32, WebGL2 41,
  shared shader/build authority 87.
- 200 exclusive source-to-target assignments across 135 complete ownership
  units. No source omission, duplicate owner, or target collision is allowed.
- The exact WebGL2 translation set follows the pinned Emscripten/WebGL2 build
  graph. Ten inventoried native-GL or Objective-C implementations excluded by
  that graph remain visible as `excluded-by-pinned-build`; they are not silently
  omitted or translated into the WebGL2 backend.
- Every WebGPU semantic owner is a new translation target. The legacy
  Rust-WGPU implementation is diagnostic evidence and a post-closeout deletion
  target, not the port target.
- 924 source dependency occurrences, including 227 owned-source edges, 440
  generated-from-owned-source edges, 110 pinned external source edges, 129
  SDK/system edges, and 18 tool-module edges.
- A dependency-first ownership-unit order computed after collapsing include
  cycles. It includes all 135 ownership units, including units with no internal
  edge.
- 520 Make-declared shader outputs regenerated with a captured local toolchain:
  514 retained artifacts have exact SHA-256 identities and six ephemeral WGSL
  intermediates are bound by retained final headers. All 197 directly included
  generated artifacts are covered.
- 5,409 derived preprocessor and configuration/capability rows across the
  complete pinned source set. Repeated symbol occurrences are grouped by
  source with exact line and enclosing-condition sets.
- 907 Clang-derived state fields across Vulkan, WebGPU Wagyu v1, WebGPU Dawn
  v2, and WebGL2/Emscripten configurations. Field order and declared types are
  frozen independently of any Rust target.
- 2,431 lifecycle evidence rows spanning construction/allocation,
  destruction/release, mapping, synchronization/submission, callbacks/async,
  threads/locks, and failure/loss.
- Eight reviewed owner-contract families cover all 135 ownership units exactly
  once and bind every configuration, field/ABI, ownership, lifetime,
  synchronization, failure, and destruction review row to its source owner.
- Product intent: exact Vulkan, WebGPU, and WebGL2 ports; explicit WebGPU/WebGL2
  selection in the web editor; legacy Rust-WGPU deletion only after all three
  exact ports pass frozen closeout.
- The oracle and product contract admits all 1,469 frozen corpus entries for
  every backend with no exclusions, names distinct backend identities, freezes
  local and cross-platform requirements, and forbids candidate-derived
  tolerances.
- A 4,407-row source-repeatability ledger reserves every corpus row for all
  three primary backends. WebGPU is captured and frozen at 1,469/1,469
  byte-identical source reruns with 0/0 per-row budgets on Apple M5 Max.
  Vulkan is captured and frozen at 1,469/1,469 repeatable source reruns on the
  same machine: 1,289 are byte-identical, and 180 freeze their observed
  one-channel-value variation with zero pixels beyond the corpus threshold.
  WebGL2 is captured and frozen at 1,469/1,469 byte-identical source reruns
  with 0/0 per-row budgets in Google Chrome 151.0.7922.173.
- The Vulkan source root is green through pinned Rive Vulkan plus the exact
  `rive-app/MoltenVK` commit
  `7de494443641fc4f81d8232fe379c336face30ab`. The upstream helper names a
  moving MoltenVK branch, so the resolved commit and every fetched transitive
  Git revision are frozen explicitly in the oracle contract.
- The WebGL2 source root is green through the pinned Rive WebGL2 archives,
  Emscripten 3.1.61, Rust 1.91.1, Playwright 1.55.0, and installed Chrome
  151.0.7922.173. Its HTML harness, browser coordinator, process-compatible
  replay client, JavaScript loader, Wasm module, C++ bridge, and ten upstream
  archives have frozen SHA-256 identities in
  `docs/backend-port-webgl2-source-root.toml`. The coordinator reloads the
  sealed Wasm module every 16 renders to prevent Emscripten process-main
  re-entry from accumulating lifetime state; every individual comparison still
  creates and destroys a fresh WebGL2 renderer context.

The source, ownership, dependency, and unit-order ledgers are generated from
the pinned upstream checkout. Their checks fail on source drift, stale output,
duplicate ownership, overlapping targets, unresolved quoted dependencies, or
an incomplete unit graph.

## Preparation closeout

The joined preparation gate reports all denominators clean and admits
translation. No candidate output was observed while any source repeatability
budget was derived. Translation must still begin only after this green state is
preserved as an immutable Git checkpoint, and it must follow the frozen
ownership-unit dependency order. Compiler integration, product cutover, and
legacy Rust-WGPU deletion remain later, separate queues.
