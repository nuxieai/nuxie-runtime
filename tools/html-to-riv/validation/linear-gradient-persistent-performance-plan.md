# P06 persistent rendering performance validation

Status: implementation seam assessed; timing deliberately deferred while the full native visual suite owns the GPU. Existing subprocess measurements remain the only measured performance evidence.

## Evidence from the current code

- `tools/renderer-replay/src/main.rs` parses one stream, creates one NativeMetalFactory, replays one selected frame and writes one PNG. The CLI has no repeat/warmup/timing option. More frames in an input stream do not make it a persistent benchmark: `--frame` selects only one.
- `crates/nuxie-render-stream/src/lib.rs::RenderStream::replay_frame` reconstructs LoadedResources on every call. A naive loop around this method would still measure shader/path resource preparation and cannot represent a retained compiled scene.
- `crates/nuxie-renderer/src/native_metal/mod.rs::NativeMetalFactory::begin_frame_for_benchmark` already accepts backend-work accounting. `NativeMetalFrame::finish_for_benchmark` returns pixels, backend_work and execution_inventory. Its canonical path submits, waits for completion and reads back pixels. It has no GPU timestamp field.
- The internal `finish_without_readback` is test-only and discards the completion handle; exposing it alone would measure unsynchronized submission and is not a valid completed-frame metric.
- Existing `linear-gradient-performance-initial/receipt.json` contains 80 measured subprocess samples and 16 warmups. The medium/256-stop CSS case was 1.44–1.45 times ordinary replay latency. This includes setup and PNG compression and does not isolate shader cost.

## Minimal next implementation

Add only `tools/renderer-replay/src/bin/gradient-benchmark.rs`, with a native-metal cfg guard. It can use the package's existing dependencies and Cargo's bin auto-discovery; no renderer production change or stream API extension is required for the first useful measurement.

Construct a NativeMetalFactory once per case. Create the gradient shaders, paints and paths once through Factory and retain them across warmup and measured frames. Loop over begin_frame_for_benchmark, draw_path using those retained resources, and finish_for_benchmark. Keep all PNG encoding, hashes, JSON output and analytic correctness checks outside measured intervals. Do not call RenderStream::replay_frame in the timed loop.

Record separately: factory setup; retained resource creation; each complete frame wall time; each finish/wait/readback interval; backend work counters and execution inventory. The meaningful primary metric is **persistent completed-frame latency including GPU synchronization and readback**, not GPU-only time or on-screen frame throughput. Emit the raw samples, not only aggregates. Write a PNG before and after the measured sequence and compare every pixel against the analytic reference outside timing to prevent a fast blank rendering from qualifying.

No production timing hook is needed to remove process startup, resource creation and PNG encoding from the current measurement. If synchronized readback dominates this retained benchmark, a later narrowly scoped change could expose an awaited no-readback completion path in `native_metal/mod.rs` that returns counters, while keeping actual Metal completion wait semantics. GPU-only timing would additionally require capturing command-buffer GPU timestamps in `native_metal/mechanical_render_context.rs` and aggregating all buffers for the frame; CPU submission timing must never be presented as GPU execution time. Propose that separately after observing retained-frame results.

## Representative cases

Run both rust-metal and rust-metal-atomic (ordinary `RenderMode::Atomics`, preserving path fill rules), at 240×160 and 1024×768. Reuse the existing twelve-tile opaque controls with 2 and 256 authored stops. Compare ordinary, untiled premultiplied, and tiled premultiplied gradients with identical paths/stops. Use both distinct-color gradients and same-color/different-tile gradients, which separates stop-atlas sharing from per-draw tile transforms.

Add a focused transparent three-stop case, rectangular repeats under translated/scaled host transforms, and a maximum-stop hard discontinuity case. Keep correctness and performance receipts separate: ordinary straight-alpha colors are not a visual oracle for premultiplied transparent colors.

Use 20 warmups and 100 recorded frames per case initially, retaining all raw values; those counts are a sampling protocol, not pass thresholds. Use deterministic case order shuffling in at least three independent rounds. Confirm the full visual suite is terminal before timing and serialize all benchmark GPU work. Record renderer/source hashes, OS/device/adapter, power source, dimensions, mode, gradient/stop/path counts, and whether resource creation or resizing is timed.

A separate retained-resize experiment should reuse resources and cycle 240→390→768→240 dimensions. Report resize allocation latency separately from steady-state frames; this measures renderer resizing only and does not prove responsive compiler layout. Compiler compile-once resize correctness remains covered by the lifecycle suite.

## Interpretation

Report median and observed range plus raw samples, counters and paired ratios. Do not impose a universal frame budget or relax correctness thresholds. A difference that repeats across rounds motivates profiling of the finish interval or GPU timestamps; it is not automatically a blocker or proof of acceptable product performance. This work cannot qualify mobile hardware, offscreen opacity groups, arbitrary composition, memory pressure, asynchronous presentation or sustained thermal behavior.

## Implemented harness (not yet timed)

`tools/renderer-replay/src/bin/gradient-benchmark.rs` now implements retained native factory, shader, paint and path ownership. `cargo check -p renderer-replay --bin gradient-benchmark --features native-metal` passed; log: `output/playwright/html-to-riv/gradient-benchmark-check.log`. It emits cold, warmup and measured frame samples, setup costs, backend inventory, first/last PNGs, analytic interior-color checks and byte equality across every completed frame. Work counters unavailable from the canonical backend are explicitly identified as unmeasured rather than evidence of zero work.

`benchmark-retained-gradients.py` orchestrates 48 configurations per round: two backends, ordinary/CSS/tiled paints, two dimensions, two stop counts, distinct/shared stop content. Default is three rounds, 20 warmups and 100 measured frames. Python syntax validation passed. Neither the benchmark nor a native smoke frame has run: the existing GPU regression owns timing conditions, and execution must be scheduled after it is terminal. Freeze the built executable and record its hash before invoking the harness.

Example build: `CARGO_INCREMENTAL=0 cargo build -p renderer-replay --bin gradient-benchmark --features native-metal`. After the GPU slot is free, first run one case with 2 warmups/3 samples to verify native execution, then use the orchestrator. A smoke case is not performance qualification.


## Backend semantics and tiny-tile boundary

The retained binary now maps `rust-metal-atomic` to ordinary `RenderMode::Atomics`. The optional `rust-metal-clockwise` selector retains explicit `ClockwiseAtomic` for performance controls only; its report sets `cssEquivalenceEligible=false`. The harness defaults to the two semantic-preserving backends. `--include-clockwise` adds forced-winding controls separately (72 configurations per round), which cannot establish CSS equivalence because forced winding can change even-odd paths. The current mode update passes `cargo check -p renderer-replay --bin gradient-benchmark --features native-metal`; source/log hashes are bound in `output/playwright/html-to-riv/gradient-benchmark-atomics-check-receipt.json`. No Cargo or GPU invocation occurred during the shared renderer build.

Tile constructor acceptance now additionally requires finite reciprocals and origin/dimension ratios in f32 (`css_gradient_tile_is_representable`). Finite positive width alone is insufficient: a width of 1e-40 overflows the reciprocal. This is a checked-input correctness boundary, not a useful steady-state throughput workload. Keep tiny/unrepresentable dimension and large-origin ratio rejection in constructor/transport tests, including positive representable controls, and use already-qualified finite tiles for timed cases. Extremely small accepted tiles can alias at pixel scale; without a separate analytic sampling/precision contract they should not be added to the timing matrix or used to claim visual fidelity.
