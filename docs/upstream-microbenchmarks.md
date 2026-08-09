# Upstream microbenchmark mirror

This repository mirrors the 20 benchmarks registered by the pinned C++
runtime at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The authoritative inventory,
source hashes, comparison classification, fixture hashes, and pinned ref live in
`microbenchmarks.toml`.

The mirror is diagnostic evidence, not a merge ratchet. Ratios are emitted only
where both sides use the same input construction, operation boundary,
repetition count, execution capabilities, and minimum-individual-invocation
statistic. All 20 cases meet that requirement. The ten `Draw*` cases use the
production RasterOrdering logical-frame mode on both sides.

## Workload correspondence

All 20 cases have equivalent measured boundaries and receive direct ratios:

- `BuildRawPath`, `IterateRawPath`, `MeasurePath`, and `RawPathBounds`.
- `MapPointsScaleTrans` and `MapPointsAffine` use the production bulk
  `map_points`/`map_points_in_place` slice APIs for the same 4,096-point buffers
  and 4,096 passes as C++ `Mat2D::mapPoints`.
- The four `Intersection*` cases.

`MeasurePath` uses the opt-in runtime support seam to construct the production
measure directly from the transformed `RawPath`; no Rust-only command adapter is
inside the timed boundary.

- The ten `Draw*` cases use the production `LogicalFrame` through its retained
  `NullLogicalRenderer`. Both sides execute ten 1600x1600 begin/draw/flush
  frames, include logical planning and typed shadow-buffer writes, and omit
  final GPU submission. Pinned C++ `RenderContextNULL` and Rust's production
  Null logical renderer both select `RasterOrdering`. The
  [dated resolution record](evidence/upstream-draw-microbenchmark-blocker-2026-08-06.md)
  maps the shared production seam and its regression coverage.

The path coordinates, matrix values, C `srand(0)`/`rand()` inputs, and ten-frame
draw loops follow the pinned sources. Paper capture preserves authored color,
blend mode, and linear/radial gradients before applying upstream's forced
stroke or feather mutations. Random point normalization uses the host C
library's supported `RAND_MAX` contract: 32,767 on Windows and 2,147,483,647 on
the Apple/Linux targets used by the upstream suite. Forced paper feathering
preserves authored paint styles while setting feather to 100 and the path fill
rule to clockwise, matching upstream.

The two bbox arrays and `paper.riv` are deterministic byte conversions of the
upstream generated headers. `make microbench-gate` parses `REGISTER_BENCH`
directly from the pinned C++ sources, requires exactly the declared 20 cases,
checks every benchmark source hash, verifies the pinned `RenderContextNULL`
capability source still enables RasterOrdering, and checks fixture conversions
and hashes.

## Reproducible run

The local upstream checkout and the Rust worktree must both be clean and at
their declared commits. The evidence runner archives committed source from the
validated pinned checkout, expands it inside the unique run directory, and
builds the C++ `bench` target there; it never accepts a prebuilt external
benchmark binary or uncommitted upstream source.

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

`microbench-run` creates unique C++ build and Criterion output namespaces,
records the Rust and C++ revisions, benchmark-content identity, C++ source
archive hash, exact build command/cwd/output directory, build log and binary hashes, tool versions,
settings, inventory hash, and every raw output hash in `run.json`. The fresh C++
build receives an allowlisted environment rather than `os.environ`: release,
gmake2, the default text/layout/canvas Premake flags, the pinned Premake tag,
run-local HOME/TMPDIR/output paths, fixed system PATH, and resolved CC/CXX are
explicit; ambient `RIVE_CONFIG`, `RIVE_PREMAKE_ARGS`, `RIVE_OS`, `RIVE_ARCH`,
`RIVE_VARIANT`, `DEPENDENCIES`, `PREMAKE_PATH`, SDK/compiler flags, and similar
overrides cannot enter the build. A sealed `cpp-build-inputs.json` records that
exact environment, the path/size/hash of each required build tool, and content
identities for the fetched build/test dependency trees. Comparison validates
the document, current tool bytes, and retained dependency-tree bytes. An existing
non-empty run directory is rejected. If `CRITERION_HOME` is set, the unique run
namespace is created below it. If `CARGO_TARGET_DIR` is set, Cargo uses and
records it; otherwise the repository target directory is recorded. Comparison
accepts only `run.json`, requires the exact v6 run schema and exact artifact key
set (the six fixed artifacts plus one `criterion:<case>` entry per inventory
case), verifies every path and hash, and reads each C++ output and Criterion
sample once while hashing it. Comparison parses only those retained validated
bytes, so replacing a path after validation cannot change the table. The
informational `criterion_home` setting cannot
redirect comparison, and mixed-run sample namespaces are rejected. Comparison
also verifies the current inventory and committed repository content identity.
That identity hashes the full Git tree except `docs/evidence/`, allowing the
measured run's evidence-only descendant commit to revalidate while rejecting
any benchmark, tool, input, production-source, manifest, or other content
change. Uncommitted changes are also rejected except beneath `docs/evidence/`.

The classification contract names all 20 ratio cases explicitly and rejects
any directional label.

Both harnesses report the minimum observed individually timed invocation. The
Criterion benches use `iter_custom` to start and stop the clock around every
operation, then encode each sample's minimum as `elapsed / iterations` in raw
`sample.json`. Comparison selects the minimum of those per-sample minima rather
than Criterion's median estimate.

## Feature-gated support seam

The timed binaries call production-compiled code through doc-hidden
`upstream-microbenchmarks` modules in `nuxie-runtime` and `nuxie-renderer`.
These opt-in public symbols are not in default builds, but they are a permanent
API and maintenance tradeoff. Draw workloads cross the production
`LogicalFrame` seam through the Null adapter; they never call shallow direct
tessellation substitutes. This is preferable to copying private source modules
into the bench target, which compiles `cfg(test)` counters and statistics into
timed code and can suppress real tests under `--all-features`.

Criterion remains a dev-only dependency. Renderer's optional `libc` dependency
is enabled only by the benchmark-support feature for the upstream C PRNG input
stream.
