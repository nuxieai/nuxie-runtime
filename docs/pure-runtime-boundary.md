# Pure-runtime dependency boundary

Status: ratified architecture contract. This is a forward-facing ownership
rule for runtime and product work, not a port-phase plan.

## Dependency direction

```text
editor authoring --------+
shared product host -----+---> parity baseline <--- portable C ABI
browser adapter ---------+            ^             replay/oracle tools
Apple adapter -----------+            |
                              general-purpose crates
```

Dependencies point toward the parity baseline. Baseline, portable-ABI, replay,
oracle, fuzz, golden, and performance packages must not depend on shared
product, authoring, browser, or Apple packages.

### Parity baseline

The baseline owns behavior specified by the pinned C++ runtime: `.riv` import,
object and graph construction, animation/state-machine execution, frame
advance/apply, scripting semantics, backend-neutral rendering, and faithful
embedder operations. Host-safety adaptations remain baseline only when they
are product-neutral and explicitly recorded.

The baseline must not know about Nuxie flows, authored transactions, ProjectDO
identities, Nux artifact manifests, product host commands, or platform
presentation lifecycles. A product requirement enters through a small
baseline-owned interface implemented by an adapter above the boundary.

Luau source compilation/evaluation, source-module registration, deterministic
critique sampling, raw-WGSL render plans, and direct pixel-readback workflows
are editor tooling. The baseline accepts compiled ScriptAsset bytecode,
including shared-global bytecode evaluation for editor callers, and executes
imported GPUCanvas userdata through `GpuCanvasPlan`; nuxie-dev owns the source
and snapshot layer above those contracts.

### Shared product host

The shared product layer owns FlowSession, player selection, transactional host
batches, product output/wake/error policy, the private Nuxie Luau module and
host effects, ProjectDO vocabulary/programs, and Nux artifact authentication.
It may consume baseline operations but may not replace pinned runtime
semantics.

### Editor authoring

The editor layer owns Scene transactions, stable authored identities, schema
and lowering policy, dynamic construction, authored observation, and
authoring-only binary builders. Runtime instances and import/advance/draw
semantics remain baseline-owned.

### Browser adapter

The browser adapter owns canvas attachment, browser resize/recovery policy,
and direct WebGPU presentation. Backend-neutral renderer factory, frame, and
render-target mechanics remain in the baseline.

### Apple adapter

The Apple adapter owns CAMetalLayer/drawable lifecycle, presentation
completion/disposition, trusted-image admission policy, and the Apple product
ABI. Backend-neutral Metal/WebGPU mechanics remain in the baseline.

### Portable ABI and oracle consumers

`nux-capi` adapts baseline operations into C calling conventions, handles,
errors, callbacks, and buffer negotiation. Product operations belong in a
separately named product ABI. Replay and oracle tools import the baseline
directly so parity evidence cannot depend on product glue.

## Current migration debt

The repository is not yet physically split at every ownership boundary. The
guard therefore ratchets, reports, and only permits these audited debt classes:

- ProjectDO implementation and re-exports in runtime/data-binding files;
- product host commands and host resource limits in scripting files;
- test-support authoring builders in binary/runtime fixture owners;
- the exact `nux-capi -> nuxie` mixed-facade edge needed by the current
  portable ABI.

Each exception names exact files or one exact manifest edge. It may shrink or
disappear; it may not spread. The UNIV-1621 child issues own the physical
extractions, so this document does not duplicate their sequencing.

## Executable ratchet

`tools/pure-runtime-boundary/check.py` derives protected packages from Cargo
workspace membership, including in-repository path dependencies Cargo treats
as implicit members. Product/platform consumers are exempt packages and also
forbidden upward dependency targets. New workspace packages are protected by
default.

The manifest check covers dependency, dev-dependency, build-dependency, target,
optional, default-disabled, measurement, and portable-ABI declarations. It
resolves package aliases and workspace-inherited specifications using Cargo's
path and default-feature behavior. The temporary mixed facade is constrained
to its exact dependency table, key, effective defaults, feature forwarding,
provider activation/dependency shape, and approved baseline symbols.
Both sides of the temporary edge are pinned to their audited in-workspace
providers; registry, git, alias, duplicate, and target-specific substitutions
are rejected.

The source check scans every Rust source in each package, including build
scripts and custom-target module trees outside conventional folders. It
rejects product paths and cross-package compiler source edges, including
conditional `path` attributes and `include!` forms it cannot prove local. The
runtime's exact generated-object include is the sole audited dynamic exception.
The check prevents every audited internal-debt family from appearing outside
its exact files. Comments and literals are stripped before matching. Cleared or
deleted exceptions fail as stale so debt cannot be silently reintroduced later.

Run the independent unit and live-workspace verdicts with:

```sh
make pure-runtime-boundary-gate
```
