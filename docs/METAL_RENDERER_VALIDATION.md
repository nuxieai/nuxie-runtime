# Whole Metal Renderer Validation Contract

This document defines the exit gates for the file-first native Metal renderer
port. It is subordinate to `PARITY_WORKFLOW.md`, `METAL_PORTING.md`, and
`METAL_RENDERER_PORT_PLAN.md`.

Validation does not select implementation work. The pinned source determines
what is translated; these suites determine whether the complete translation
is correct.

## Oracle hierarchy

The renderer has two different parity questions, and they must not be merged:

1. **Did Rust mechanically preserve the native Metal implementation?**
   The primary oracle is pinned C++ Metal at
   `4ac7b32798da0482e441ef09304dc3b480ed3ee5`, run on the same adapter with
   pinned bridge inputs.
2. **Does the native Metal product preserve the behavior users currently see
   through Rust-WGPU?** The secondary oracle is the current Rust-WGPU backend
   built from the same Nuxie source revision and replayed from the same stream,
   frame, clear color, dimensions, and authored mode.

Rust-WGPU is not source authority for pipeline keys, pass topology, ownership,
barriers, allocation timing, or GPU work counts. Those are compared to pinned
C++ Metal. WGPU is a product-output differential: pixels, occupancy, success
versus failure, and stable public error category.

## Current corpus inventory

`corpus-r.toml` currently contains 1,469 unique entries:

- 736 `clockwise-atomic` rows, which form the final Metal-compatible visual
  differential;
- 733 `msaa` rows, which are excluded from native Metal execution because the
  pinned Metal backend does not implement the WebGPU-style MSAA mode.

The count is mechanically checked by the progress-page generator and the
Makefile guard. Commit `53b4c035a` added the 1,469th row; the stale 1,468-row
Makefile expectation was corrected when this contract was adopted.

The existing WGPU-secondary tracer manifest contains only four rows. It is an
early regression lane, not whole-renderer parity evidence.

## Required suites

### V0 — inventory and provenance

Run:

```text
make metal-port-check
make metal-port-progress-check
```

Acceptance:

- the pinned upstream revision and input digests match;
- corpus IDs are unique and the declared 1,469/736/733 counts match;
- the source map has continuous, non-overlapping coverage;
- the checked-in dashboard is current.

### V1 — source and ownership closure

Run the campaign checker after every preparation or translation checkpoint.
Whole-owner promotion requires zero `partial` and zero `missing` ranges in the
primary header and implementation, every state-bearing field accounted for,
and no required owner left `pending` or `in-progress`.

A passing image cannot override this gate.

### V2 — compile and configuration matrix

After the whole translation is wired, run warning-clean native-feature checks
for Apple builds and portable cfg checks for non-Apple builds. Compile every
pinned platform configuration that the repository can represent: macOS Apple
Silicon, macOS Intel, iOS device, iOS simulator, tvOS, and visionOS. A platform
that cannot be executed must have a checked compile gate and an explicit
configuration exclusion; it may not disappear silently.

### V3 — native API, structure, lifecycle, and failure

The whole native suite runs with Metal API and shader validation enabled:

```text
MTL_DEBUG_LAYER=1 MTL_SHADER_VALIDATION=1 \
  cargo test -p nuxie-renderer \
  --features native-metal-experimental \
  --test native_metal_tracer

cargo test -p nuxie-renderer \
  --features native-metal-experimental --lib
```

The final suite must cover the complete source-shaped flush path, exact tables
and bindings, command/pass/draw ordering, cache fallback, resource replacement,
abandonment, ring pressure, failed command buffers, context drop before GPU
completion, and destruction order. Assertions are compared with pinned Metal
behavior, not WGPU physical work.

### V4 — primary pinned C++ Metal parity

The candidate Rust Metal replay and pinned C++ Metal reference replay must run
on the same physical adapter. The reference must carry its pinned upstream and
input-manifest provenance.

Existing bounded lanes remain useful:

```text
make renderer-metal-oracle-tracers
make renderer-metal-atomic-oracle-tracer
```

They currently cover four capability-driven rows and eight forced generic
atomic rows. Before whole-owner promotion, this becomes a complete
Metal-compatible same-runner corpus rather than a hand-selected entry list.

Acceptance for every row:

- candidate success/failure matches the pinned backend for every reachable
  source path;
- pixels stay within the predeclared Metal tolerance;
- exact occupancy is preserved independently of pixel tolerance;
- structural inventory selects the native path—no silent raster route, WGPU,
  CPU, or unsupported fallback;
- a tolerance is never widened from candidate output.

### V5 — Rust Metal versus current Rust-WGPU product differential

The existing four-row lane in
`tools/metal-port/tracer-corpus-wgpu-secondary.toml` remains an early smoke
test. It is currently amber because 4 of 736 Metal-compatible rows are not a
whole-renderer claim.

The final behavior phase must add one reproducible target,
`renderer-metal-wgpu-parity`, with this contract:

1. derive a checked manifest containing every `clockwise-atomic` entry from
   `corpus-r.toml`—currently exactly 736 unique rows;
2. build the Rust Metal candidate and Rust-WGPU reference from the same Nuxie
   revision;
3. run candidate backend `rust-metal` and reference backend `rust-wgpu` with
   identical stream, frame, mode, dimensions, clear color, and replay inputs;
4. run serially on Metal unless the runner proves independent device/queue
   ownership;
5. compare pixels and exact differs-from-clear occupancy;
6. compare success/failure and stable public error category;
7. retain candidate, WGPU reference, and visual diff images plus a
   machine-readable summary for every failure.

Cross-backend rasterization budgets must be declared before the Rust Metal
candidate run. A nonzero budget requires independent provenance—normally the
pinned C++ Metal versus current Rust-WGPU difference—and review. Candidate
output may not establish or widen its own budget. Rows without an approved
cross-backend budget are byte-exact.

The final gate is green only at 736/736. No sample, percentage, or aggregate
score can substitute for all rows.

The following are deliberately not compared between Rust Metal and WGPU:

- pipeline keys and shader permutations;
- render-pass, barrier, upload, allocation, or draw counts;
- native resource ownership and destruction timing;
- backend performance counters.

Those architectures differ. Each backend's physical work is validated against
its own source authority; the cross-backend gate compares observable product
behavior.

### V6 — explicit MSAA exclusion

Run:

```text
make renderer-metal-msaa-contract
```

All 733 WebGPU `msaa` rows remain outside native Metal parity until the pinned
Metal source itself supports that mode. The gate must continue to reject a
harness that relabels Dawn/WGPU output as native Metal output.

### V7 — platform and hardware policy

Execute the complete corpus on Apple Silicon macOS and Intel/discrete macOS,
including raster-order, explicit memory-barrier, and render-pass-break policy
where hardware permits. Device/simulator Apple targets receive live or checked
compile evidence according to the campaign matrix.

### V8 — rooted product and no-fallback proof

Run the rooted native product path, then inspect the exact executable and
Cargo graph. The final artifact must contain the translated Metal path and no
WGPU, Naga, Dawn, hidden WGSL translation, or CPU-render fallback. Record the
Mach-O hash and size, linked libraries, compiled shader inventory, and one or
more deterministic rendered outputs.

### V9 — independent closeout

After V0–V8 are green, rerun independent source/spec and
ownership/standards reviews. Fix every accepted finding in its source owner and
rerun the affected suites. The header and implementation may become `ported`
only after all exit gates are green.

## Evidence retention

Every suite writes or links:

- exact command line and source revision;
- adapter, OS, and relevant Metal capability data;
- input and executable digests;
- machine-readable row results;
- complete failure logs;
- reference, candidate, and diff images for visual failures;
- structural/work inventories where the suite owns them.

The generated progress dashboard displays these states and images, but the raw
files remain the evidence of record.
