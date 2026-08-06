# Upstream microbenchmark mirror

This repository mirrors the 20 benchmarks registered by the pinned C++
runtime at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The authoritative inventory,
source correspondence, fixture hashes, and pinned ref live in
`microbenchmarks.toml`.

The mirror is diagnostic evidence, not a merge ratchet. A ratio can identify a
primitive worth investigating, but it must not block a change until repeated
measurements establish a stable threshold and the two implementations are
confirmed to measure equivalent work.

## Workload correspondence

| Rust target | Cases | Pinned C++ source |
|---|---|---|
| `nuxie-runtime` | `BuildRawPath`, `IterateRawPath`, `MeasurePath`, `RawPathBounds`, `MapPointsScaleTrans`, `MapPointsAffine` | `tests/bench/build_raw_path.cpp`, `iterate_raw_path.cpp`, `measure_path.cpp`, `raw_path_bounds.cpp`, `map_points.cpp` |
| `nuxie-renderer` | four intersection cases | `tests/bench/intersection_board_bench.cpp` |
| `nuxie-renderer` | ten `Draw*` cases | `tests/bench/draw_pls_path.cpp` |

The iteration counts, path coordinates, matrix values, C `srand(0)`/`rand()`
inputs, and ten-frame draw loops follow the pinned sources. The two bbox arrays
and `paper.riv` are deterministic byte conversions of the upstream generated
headers; `make microbench-gate` checks both the committed hashes and a fresh
conversion from the pinned checkout.

The Rust cases exercise the closest current production primitive. This makes
some ratios directional rather than instruction-for-instruction comparisons:

- The C++ draw cases use `RiveRenderer` with a null render context. Rust has no
  equivalent public null-renderer seam, so the mirror captures the same paths
  and paints and times the crate-private CPU fill, stroke, and feather
  preparation used by the renderer.
- C++ `mapPoints` uses its vectorized bulk implementation. Rust currently
  exposes scalar `Mat2D::map_point`, which the mirror applies to the identical
  4,096-point working set and iteration count.
- Rust `MeasurePath` must transform `RawPath`, convert it into runtime path
  commands, and then construct `RuntimePathMeasure`; that adapter work is part
  of the result because there is no direct raw-path measure entry point.
- `BuildRawPath` intentionally calls the normal `RawPath` mutators. Using the
  scoped `rebuild` builder would avoid current mutation/contour bookkeeping and
  conceal the production-path cost this benchmark is intended to reveal.

## Commands

The local upstream checkout must be at the ref declared in
`microbenchmarks.toml`, and its release `tests/out/release/bench` target must be
built from that checkout.

```sh
make microbench-gate RIVE_RUNTIME_DIR=/path/to/rive-runtime
make microbench-build
make microbench-rust
make microbench-cpp RIVE_RUNTIME_DIR=/path/to/rive-runtime
make microbench-compare
```

Criterion accepts filters and measurement arguments after the Cargo separator,
for example:

```sh
make microbench-rust \
  MICROBENCH_CRITERION_ARGS='--warm-up-time 1 --measurement-time 5 --sample-size 50'
```

The comparison tool reads C++ milliseconds and Criterion's median point
estimate, then emits a 20-row `Rust/C++` table in manifest order. The upstream
harness reports its best run over the selected duration, whereas Criterion
reports a sampled median. Always record those settings, the machine, both
source revisions, and ambient system load with a committed table.

## Private renderer visibility

The renderer benchmark uses `#[path]` to compile the existing private
`draw`, `gpu`, `gr_triangulator`, and `intersection_board` modules into the
bench executable. Cargo enables `cfg(test)` for bench targets, which would also
compile `gr_triangulator`'s GPU oracle tests; those tests depend on helpers that
exist only at the renderer library root. The opt-in
`upstream-microbenchmarks` feature suppresses that test module only while this
non-test harness is built. It adds no exported production symbol and is covered
by the portable feature-compile gate.

Criterion and its dependency closure are dev-only dependencies of the two
measured crates. Default features are disabled and only
`cargo_bench_support` is enabled, avoiding the unused Plotters and Rayon
closures. The remaining Criterion packages appear in `Cargo.lock` so benchmark
builds are reproducible, but are not linked into shipped library targets.
