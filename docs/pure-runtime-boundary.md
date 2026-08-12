# Pure-runtime dependency boundary

Status: ratified architecture contract. This is a forward-facing ownership
rule for runtime and product work, not a port-phase plan.

## Dependency direction

```text
editor authoring --------+
Swift SDK ---------------+---> parity baseline <--- portable C ABI
browser adapter ---------+            ^             replay/oracle tools
Apple platform services -+            |
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

The baseline must not know about Nuxie experiences, flows, screens, SDK
sessions, authored transactions, ProjectDO identities, Nux artifact manifests,
package authentication, product host commands, or host/UI
presentation lifecycle policy. Product-neutral backend mechanics may validate
and present a caller-borrowed native target as described by the Apple platform
extension below; layer ownership, target acquisition, actor choice, and frame
scheduling remain above the boundary. A product requirement enters through a
small baseline-owned interface implemented by an adapter above the boundary.

`nuxie` is the protected, general-purpose facade over this baseline. Its own
manifest, sources, and first-party dependency closure follow the same downward
dependency rule as the lower-level runtime crates; the facade is not an
application-layer exemption.

Luau source compilation/evaluation, source-module registration, deterministic
critique sampling, raw-WGSL render plans, and direct pixel-readback workflows
are editor tooling. The baseline accepts compiled ScriptAsset bytecode,
including shared-global bytecode evaluation for editor callers, and executes
imported GPUCanvas userdata through `GpuCanvasPlan`; nuxie-dev owns the source
and snapshot layer above those contracts.

### Product layer

Experience lifecycle, Flow/session policy, `.nux` acquisition and
authentication, product-specific host commands, and Apple application
orchestration are no longer runtime crates. The Swift SDK owns those concepts
on Apple platforms and consumes the raw C ABI. `nuxie-project-data` remains an
authored-data conversion owner above the baseline. The distributed
`nux-apple-product-extension` may install that converter through one explicit,
product-named import entrypoint; `nux-capi` itself never depends on or installs
the converter.

### Editor authoring

The editor layer owns Scene transactions, stable authored identities, schema
and lowering policy, dynamic construction, authored observation, and
authoring-only binary builders. Runtime instances and import/advance/draw
semantics remain baseline-owned.

### Browser adapter

The browser adapter owns canvas attachment, browser resize/recovery policy,
and direct WebGPU presentation. It is physically owned by nuxie-dev and targets
the exact nuxie-runtime gitlink pinned by that repository. Backend-neutral
renderer factory, frame, presentation-target mechanics, and parity/oracle
tooling remain in the baseline.

### Apple platform extension

The protected `nuxie-renderer` package owns product-neutral Metal mechanics,
including validation and presentation of a caller-borrowed CAMetalDrawable and
device-health reporting. It never owns CAMetalLayer, acquires a drawable, or
imports UIKit/AppKit policy. The Apple-only `nux-capi/apple-metal` feature
exposes those mechanics through product-neutral C handles and fixed-width
outcomes. `nux-capi` also composes the generic scripting and image-decoding
seams required by the SDK. The caller owns layer configuration, drawable
acquisition, actor and frame scheduling, and every product concept.

The shipping Apple archive is rooted at `nux-apple-product-extension`. That
upper leaf combines `nux-capi` with `nuxie-project-data` while exporting only
`nux_product_file_import_configured` in addition to the product-neutral CAPI.
It does not own platform rendering or lifecycle policy, and it does not revive
the retired `NuxieRuntimeFFI` package.

### Portable ABI and oracle consumers

`nux-capi` adapts baseline operations into C calling conventions, handles,
errors, callbacks, and buffer negotiation. Its Apple feature is a
product-neutral platform extension. Product lifecycle operations remain
absent; the separately owned authored-data leaf adds only converter
installation plus configured import. Replay and oracle tools import the
baseline directly so parity evidence cannot depend on product glue.

The direct `nux-capi -> nuxie` dependency is a permanent, narrowly approved ABI
edge rather than migration debt. It reaches only the audited baseline facade:
the manifest form, local provider, feature behavior, and imported Rust symbols
are independently constrained. The whole `nuxie` provider graph remains
protected by the ordinary baseline rules.

The forwarded `scripting` feature and generic host-command values are runtime
mechanics for an exact-artifact caller. They do not select product commands or
authorize product policy: approved dependency shapes, facade symbols,
constructors, nested imports, and owning files are each enumerated, while
aliases, globs, unknown features and symbols, and product lifecycle vocabulary
remain rejected.

## Current compatibility debt

The repository is not yet physically split at every ownership boundary, but
protected baseline consumers have no grandfathered source-debt files.
Synthetic record construction is available only through `nuxie-binary`'s
non-default `test-support` feature, and shared binary assets live under the
neutral root fixture corpus. The default shipping API remains byte-import
only. One compatibility module in `nuxie-binary` retains the former
editor-facing names so already-pinned nuxie-dev builds do not break; the guard
confines those names to that zero-shipping seam and prevents `test-support`
from entering the default feature set.

ProjectDO evaluation is now physically owned by `nuxie-project-data` and enters
the baseline through the product-neutral external-data seam documented in
`docs/project-data-runtime-seam.md`; its former runtime debt class is empty.

Any future debt exception must name exact files and is an architecture-policy
change. The approved portable-ABI edge is enforced as permanent architecture
policy and is not included in debt reporting. The former Apple/product runtime
crates are deleted and retained as forbidden dependency vocabulary so they
cannot quietly return.

## Executable ratchet

`tools/pure-runtime-boundary/check.py` derives protected packages from Cargo
workspace membership, including in-repository path dependencies Cargo treats
as implicit members. Product/platform consumers are exempt packages and also
forbidden upward dependency targets. New workspace packages are protected by
default. The dependency-closure guarantee is first-party and in-repository:
local path providers reachable from a protected package must themselves be in
the scanned package set rather than escaping through a workspace exclusion.
An exact audited set of third-party vendored packages, plus registry and git
dependency internals, remains the responsibility of the repository's lockfile
and supply-chain controls; the `vendor/` prefix itself is not an exemption.

The manifest check covers dependency, dev-dependency, build-dependency, target,
optional, default-disabled, measurement, and portable-ABI declarations. It
resolves package aliases and workspace-inherited specifications using Cargo's
path and default-feature behavior. The approved portable-ABI facade edge is
constrained to its exact dependency table, key, effective defaults, feature
forwarding, audited in-workspace provider, and approved baseline symbols.
Apple vocabulary and generic presentation markers are likewise confined to an
exact audited set of extension manifests, headers, sources, tests, and renderer
implementation files; product vocabulary remains forbidden there, and a new
file fails closed. This is platform-backend policy, not a product-layer or
portable-build exemption.
Registry, git, alias, duplicate, and target-specific substitutions are
rejected. Root Cargo path patches are resolved too: non-excluded local providers
join the protected scan, while excluded providers must be in the exact audited
third-party set. Repository Cargo configuration may not override dependency
providers through `[patch]`, `paths`, or `[source]`; committed overrides belong
in the audited root manifest. Deprecated Cargo `[replace]` overrides are
rejected outright.
Dependencies below `nuxie` are checked by the normal protected-package rules
instead of by a facade-specific provider-shape exception.

The source check scans every Rust source in each package, including build
scripts and custom-target module trees outside conventional folders. It
rejects product paths and cross-package compiler source edges, including
conditional `path` attributes and `include!` forms it cannot prove local. The
runtime's exact generated-object include is the sole audited dynamic exception.
Literal `include_bytes!` and `include_str!` data paths may use neutral
repository fixtures but may not reach into another Cargo package's ownership.
`include_bytes!(concat!(env!("OUT_DIR"), "/literal"))` may consume a
non-traversing build output owned by the same protected package; other dynamic
data-include forms still fail closed.
The check prevents every audited internal-debt family from appearing outside
its exact files. Comments and literals are stripped before matching. Cleared or
deleted exceptions fail as stale so debt cannot be silently reintroduced later.

Run the independent unit and live-workspace verdicts with:

```sh
make pure-runtime-boundary-gate
```
