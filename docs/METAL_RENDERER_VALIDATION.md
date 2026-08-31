# Native Metal validation

Validation follows integration under [PARITY_WORKFLOW.md](PARITY_WORKFLOW.md);
it does not select a new translation campaign. The authoritative comparison is
the pinned C++ native Metal implementation on the same adapter, inputs, mode,
dimensions, and clear color.

## Retained manual entry points

Run applicable targets from the repository root:

```sh
make renderer-native-metal-platform-matrix
make renderer-native-metal-v3
make renderer-metal-oracle-tracers
make renderer-metal-atomic-oracle-tracer
make renderer-metal-msaa-contract
```

- The platform matrix checks the configured Apple target triples. Required
  SDKs, target libraries, and nightly build-std support must be installed.
  Compile success is not evidence that unavailable hardware executed.
- The native V3 target enables Metal validation and requires live Metal tests.
  A test that skips because no device exists must not be reported as hardware
  coverage. Preserve nonzero test selection and report ignored/skipped tests.
- The oracle tracer targets compare the retained capability-driven and forced
  generic-atomic corpus inputs in `tools/renderer-tracers/` against pinned C++
  Metal. These are bounded lanes, not proof of all renderer behavior.
- The MSAA contract preserves the distinction between native Metal's execution
  modes and WebGPU-style MSAA. It must not relabel Dawn output as Metal.

Use the current Makefile for build options and the
[Apple release contract](nux-capi-apple-release.md) for rooted product and device
artifact checks.

## What to verify

Keep success/failure behavior, resource identities, release order, ring reuse,
abandonment, command/pass/draw ordering, capability branches, and shader
bindings aligned with the pinned source. Test both native API behavior and
rendered output. Exact occupancy and source-shaped work can expose defects
that pixel tolerance alone misses.

Use only predeclared source-oracle pixel budgets. Candidate output cannot
establish or widen its own tolerance. Legacy WGPU is neither an oracle nor a
fallback. Final Apple artifacts must preserve the no-WGPU/runtime-Naga contract.

Record commands, source and toolchain identities, adapter/OS/capabilities,
input and executable hashes, test outcomes, and failure logs. Retain reference,
candidate, and diff images for visual failures. Report device coverage that
did not run explicitly; do not expand a bounded fixture pass into a claim of
complete parity.

The old campaign's dashboards, census ledger, promotion stages, and WGPU
diagnostic lane are retired. Historical results remain available in Git
history and the [postmortem](METAL_RENDERER_PORT_POSTMORTEM.md).
