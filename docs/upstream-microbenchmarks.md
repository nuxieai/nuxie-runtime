# Upstream microbenchmark mirror

This repository mirrors the 20 benchmarks registered by the pinned C++
runtime at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The authoritative inventory,
source hashes, comparison classification, fixture hashes, and pinned ref live in
`microbenchmarks.toml`.

The mirror is diagnostic evidence, not a merge ratchet. Ratios are emitted only
where both sides use the same input construction, operation boundary,
repetition count, and minimum-per-iteration statistic. Ten cases currently meet
that requirement; the ten `Draw*` cases are explicitly blocked.

## Workload correspondence

Ten cases have equivalent measured boundaries and receive direct ratios:

- `BuildRawPath`, `IterateRawPath`, `MeasurePath`, and `RawPathBounds`.
- `MapPointsScaleTrans` and `MapPointsAffine` use the production bulk
  `map_points`/`map_points_in_place` slice APIs for the same 4,096-point buffers
  and 4,096 passes as C++ `Mat2D::mapPoints`.
- The four `Intersection*` cases.

`MeasurePath` uses the opt-in runtime support seam to construct the production
measure directly from the transformed `RawPath`; no Rust-only command adapter is
inside the timed boundary.

The ten `Draw*` cases have no ratio. Upstream's null backend skips only final GPU
submission; production `RenderContext::flush()` still lays out logical flushes,
grows and maps shadow-buffer rings, writes every typed resource buffer, plans
draws, and tears down the frame. Rust's corresponding planning is inseparable
from concrete WebGPU devices, pipelines, attachments, and encoders. The
[dated blocker evidence](evidence/upstream-draw-microbenchmark-blocker-2026-08-06.md)
maps that dependency. Direct tessellation helpers are not accepted as a
substitute, and `microbench-run` refuses evidence while any case is blocked.

The path coordinates, matrix values, C `srand(0)`/`rand()` inputs, and ten-frame
draw loops follow the pinned sources. Random point normalization uses the host C
library's supported `RAND_MAX` contract: 32,767 on Windows and 2,147,483,647 on
the Apple/Linux targets used by the upstream suite. Forced paper feathering
uses Round joins and Round caps, matching upstream and production Rust rather
than the authored paint styles.

The two bbox arrays and `paper.riv` are deterministic byte conversions of the
upstream generated headers. `make microbench-gate` parses `REGISTER_BENCH`
directly from the pinned C++ sources, requires exactly the declared 20 cases,
checks every benchmark source hash, and checks fixture conversions and hashes.

## Reproducible run

The local upstream checkout must be at the manifest ref and its release
`tests/out/release/bench` must be built from that checkout. Evidence runs require
a clean committed Rust worktree.

```sh
make microbench-gate RIVE_RUNTIME_DIR=/path/to/rive-runtime
make microbench-build
make microbench-run \
  RIVE_RUNTIME_DIR=/path/to/rive-runtime \
  MICROBENCH_RUN_DIR=target/microbench/run-001 \
  MICROBENCH_CPP_DURATION=5 \
  MICROBENCH_WARM_UP=3 \
  MICROBENCH_MEASUREMENT=10 \
  MICROBENCH_SAMPLE_SIZE=20
make microbench-compare \
  MICROBENCH_RUN_MANIFEST=target/microbench/run-001/run.json
```

`microbench-run` creates a unique Criterion output namespace, records the Rust
and C++ revisions, benchmark-content identity, C++ binary hash, tool versions,
settings, inventory hash, and every raw output hash in `run.json`. An existing
non-empty run directory is rejected. If `CRITERION_HOME` is set, the unique run
namespace is created below it. If `CARGO_TARGET_DIR` is set, Cargo uses and
records it; otherwise the repository target directory is recorded. Comparison
accepts only `run.json`, verifies every artifact hash, the current inventory,
and the committed repository content identity. That identity hashes the full
Git tree except `docs/evidence/`, allowing the measured run's evidence-only
descendant commit to revalidate while rejecting any benchmark, tool, input,
production-source, manifest, or other content change. Uncommitted changes are
also rejected except beneath `docs/evidence/`.

Both harnesses report the minimum observed elapsed time per iteration. For
Criterion this is calculated from each raw `sample.json` pair of elapsed time
and iteration count, rather than its median estimate.

## Feature-gated support seam

The timed binaries call production-compiled code through doc-hidden
`upstream-microbenchmarks` modules in `nuxie-runtime` and `nuxie-renderer`.
These opt-in public symbols are not in default builds, but they are a permanent
API and maintenance tradeoff. The draw workload currently panics rather than
time a shallow tessellation substitute; it can become runnable only after the
backend-neutral production `LogicalFrame` described in the blocker evidence is
available. This is preferable to copying private source modules into the bench
target, which compiles `cfg(test)` counters and statistics into timed code and
can suppress real tests under `--all-features`.

Criterion remains a dev-only dependency. Renderer's optional `libc` dependency
is enabled only by the benchmark-support feature for the upstream C PRNG input
stream.
