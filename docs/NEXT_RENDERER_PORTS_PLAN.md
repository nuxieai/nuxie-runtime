# Vulkan, WebGPU, and WebGL2 port campaign

This plan governs the next renderer backend campaigns. It incorporates the
process corrections from the native Metal port and overrides any generic
workflow that would use a feature, fixture, compiler diagnostic, or focused
test as the translation queue.

The campaigns are pinned to Rive
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

## Execution constraint

The `implement` and `tdd` skills are explicitly out of scope for this
campaign. Agents must actively ignore both skills even when their trigger
descriptions would otherwise appear applicable. The campaign is driven only
by the ordered source-owner queues below.

No campaign may skip forward because an existing backend compiles, a fixture
passes, or a related backend already has a plausible Rust abstraction.

## Product disposition

| Backend | Port disposition | Shipping disposition |
| --- | --- | --- |
| Vulkan | Complete concrete renderer-platform and ORE source port | Available as an exact backend after closeout; shipping selection remains a separate product decision |
| WebGPU | Complete new renderer-platform and ORE source port | Replaces the legacy Rust-WGPU implementation after all three exact ports pass closeout |
| WebGL2 | Complete concrete GL/WebGL2 renderer-platform and ORE source port | Added as an explicitly selectable backend in the web editor alongside WebGPU |

`webgpu-only-browser-cut.md` records the previous product state, not the target
state of this campaign. The campaign deliberately reverses that cut for the web
editor by adding an exact WebGL2 backend and explicit selection. It does not
silently introduce failure-triggered fallback; fallback policy requires its own
explicit product contract.

The current Rust-WGPU implementation is legacy evidence during translation. It
must remain intact until Vulkan, the new WebGPU port, and WebGL2 all pass frozen
closeout. The cutover queue then roots the new WebGPU and WebGL2 implementations,
removes the legacy Rust-WGPU implementation and its exclusive dependencies, and
proves that no legacy route remains.

## Authority order

1. This campaign plan.
2. `PARITY_WORKFLOW.md` only where it does not prescribe feature slices or a
   test-first implementation queue.
3. The backend-specific guide and ledgers created during preparation.
4. The pinned upstream source, build rules, generators, and generated outputs.

Pinned source wins over current Rust behavior unless an explicit divergence is
reviewed and evidenced. A different backend is always diagnostic evidence,
never the primary oracle.

## Global ordered queues

The queues are phase barriers. Work does not interleave across them merely
because later commands are available.

1. **Audit:** locate all current Rust, C++ oracle, product, test, build, and
   historical code for all three backends.
2. **Preparation:** freeze complete source, shared dependency, generated-input,
   generated-output, configuration, include/import, field, ownership,
   lifetime, synchronization, threading, failure, and destruction authority.
3. **Translation admission:** assign every pinned source to one complete
   ownership unit and one exclusive Rust target set. Reject comment-only
   bodies, placeholder owners, disconnected declaration/implementation types,
   compiler-inert targets, stubs, and success-returning no-ops.
4. **Vulkan translation:** translate complete Vulkan renderer and ORE owners in
   dependency order.
5. **WebGPU translation:** translate complete C++ WebGPU/Dawn owners into a new
   exact backend in dependency order. Use the legacy Rust-WGPU backend only as
   secondary diagnostic evidence.
6. **WebGL2 translation:** translate complete GL/WebGL2 owners into a concrete
   browser backend in dependency order.
7. **Source review:** independently compare every complete backend source and
   generated authority to its Rust correspondence.
8. **Ownership review:** independently attack lifetime, native handles,
   mapping, synchronization, callbacks, device/context loss, thread rules,
   unsafe boundaries, ABI, and teardown.
9. **Correction:** resolve every review finding by source owner and rerun both
   affected review contexts.
10. **Compiler queue:** integrate once, save diagnostics, and fix groups in
    dependency order without deleting translated behavior.
11. **Rooted execution:** prove each backend through its general public or
    reference seam, not a fixture-specific path.
12. **Behavior and platform queues:** run primary-oracle parity, lifecycle,
    hostile failure, generated artifact, physical-work, platform/browser, and
    forbidden-unselected-route gates.
13. **Frozen-byte closeout:** rerun independent source and ownership reviews on
    the exact bytes that passed parity.
14. **Product cutover:** after all three ports pass closeout, add explicit
    WebGPU/WebGL2 selection to the web editor, switch WebGPU to the new exact
    port, delete the legacy Rust-WGPU implementation, and prove its code and
    exclusive dependencies are absent from rooted products.
15. **Post-green work:** only after closeout and cutover, consider idiomatic
    cleanup, unsafe reduction, deduplication, or a shared backend interface.

## Preparation completion target

Preparation is green only when all three backends have:

- an exact source inventory with no unclassified files;
- a dependency-ordered ownership-unit graph;
- an exact target inventory with no overlapping file ownership;
- complete state, lifetime, synchronization, configuration, and generated
  artifact ledgers;
- a frozen primary oracle, secondary diagnostics, corpus, exclusions, and
  adapter/device identity contract;
- a target/platform/browser matrix, including honest unavailable-hardware
  dispositions;
- a rooted artifact and forbidden-route contract;
- machine checks that derive the denominators independently and fail on every
  omission, invention, duplicate, drifted pin, empty owner, or phase jump.

Until then, all translation states remain `pending` regardless of how much
existing Rust code appears related.

## Backend-specific review emphasis

### Vulkan

- instance/device extensions and feature-chain order;
- queue-family and surface ownership;
- command pools, command buffers, fences, semaphores, and timeline values;
- descriptor layouts, dynamic offsets, push constants, and pipeline caches;
- memory requirements, types, mapping, flush/invalidate, and alignment;
- image layouts, stage/access masks, barriers, and queue transfers;
- SPIR-V generation and exact embedded payloads;
- validation-layer cleanliness and multi-vendor behavior.

### WebGPU

- Dawn/WebGPU is the primary backend authority; the legacy Rust-WGPU backend,
  native Metal, and Vulkan are diagnostic only;
- adapter selection, Core/Compatibility admission, features, and limits;
- bind-group layouts, dynamic offsets, copy alignment, and implicit layouts;
- error scopes, uncaptured errors, device loss, mapping, and async pipeline
  ownership;
- exact SPIR-V-to-WGSL generation and immutable WGSL identities;
- native Dawn and real-browser execution;
- explicit backend identity proof for both WebGPU and WebGL2 selections;
- explicit proof that neither selection reaches the legacy Rust-WGPU route.

### WebGL2

- common GL state plus the exact WebGL PLS implementation;
- extension discovery and fail-closed capability selection;
- framebuffer, blend, stencil, scissor, texture-unit, VAO, buffer, and
  pixel-store state;
- GLSL version, precision, bindings, link errors, and generated shader bytes;
- context loss/restoration and stale resource identity;
- readback, premultiplied alpha, color space, orientation, MSAA, and resolve;
- Chromium, Firefox, and Safari reference execution where available;
- explicit web-editor selection, rooted artifact identity, and isolation from
  the legacy Rust-WGPU route.

## Checkpoint rule

Create immutable commits at these barriers:

1. audit complete;
2. preparation green;
3. each backend translation complete;
4. each global review correction complete;
5. compiler green;
6. rooted execution green;
7. parity/platform green;
8. frozen-byte closeout green.
9. legacy Rust-WGPU deletion and web-editor cutover green.

Receipts and dashboards complement these commits; they do not replace
reviewable history.
