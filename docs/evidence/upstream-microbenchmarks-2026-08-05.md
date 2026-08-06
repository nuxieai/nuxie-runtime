# Upstream microbenchmark evidence — 2026-08-05

First diagnostic table for the 20-case upstream mirror in
[UNIV-1688](https://universe.basis.dev/issue/UNIV-1688). This is evidence, not a
performance ratchet.

## Environment and protocol

- Machine: MacBook Pro `Mac17,6`, Apple M5 Max, 18 logical CPUs, 128 GiB RAM.
- OS: macOS 26.5.2 (25F84), arm64.
- Rust: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, Criterion 0.7.0, Cargo bench
  profile (optimized).
- Rust base revision: `784375ec421e9cedab526444cb2dbddf40d9ddd6` plus the
  UNIV-1688 worktree changes.
- C++ revision: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; release bench
  target verified current with `make -C tests/out/release bench -j8`.
- C++ sampling: `--duration 1`; the upstream harness reports the minimum.
- Rust sampling: `--warm-up-time 0.1 --measurement-time 0.2 --sample-size 10`;
  the table uses Criterion's median point estimate.
- Both suites ran on the same machine in the same session. The machine was not
  isolated: other build and UI processes were active, so this short first run
  is suitable for prioritization only. Re-run under controlled load before
  setting any threshold.

Commands:

```sh
cargo bench -p nuxie-runtime --bench upstream_microbenchmarks -- \
  --warm-up-time 0.1 --measurement-time 0.2 --sample-size 10
cargo bench -p nuxie-renderer --features upstream-microbenchmarks \
  --bench upstream_microbenchmarks -- \
  --warm-up-time 0.1 --measurement-time 0.2 --sample-size 10
make microbench-cpp RIVE_RUNTIME_DIR=/Users/levi/dev/oss/rive-runtime \
  MICROBENCH_CPP_DURATION=1 \
  MICROBENCH_CPP_OUTPUT=target/microbench/cpp-2026-08-05.txt
python3 tools/microbench/microbench.py --repo-root . compare \
  --cpp-output target/microbench/cpp-2026-08-05.txt
```

## Results

| Benchmark | C++ | Rust | Rust/C++ |
|---|---:|---:|---:|
| `BuildRawPath` | 2.258000 ms | 5420.163646 ms | 2400.427x |
| `DrawCustomFeathers` | 187.400000 ms | 562.698355 ms | 3.003x |
| `DrawFeatheredPaths_paper` | 80.330000 ms | 229.063917 ms | 2.852x |
| `DrawOneChopStrokes` | 24.200000 ms | 91.262375 ms | 3.771x |
| `DrawOneCuspStrokes` | 44.390000 ms | 140.916771 ms | 3.175x |
| `DrawRiveRenderPaths` | 7.861000 ms | 24.651208 ms | 3.136x |
| `DrawRiveRenderPathsAsRoundJoinStrokes` | 16.430000 ms | 56.150771 ms | 3.418x |
| `DrawRiveRenderPathsAsStrokes` | 15.060000 ms | 51.970562 ms | 3.451x |
| `DrawTwoChopStrokes` | 35.670000 ms | 128.016250 ms | 3.589x |
| `DrawTwoCuspStrokes` | 99.510000 ms | 175.626354 ms | 1.765x |
| `DrawZeroChopStrokes` | 10.960000 ms | 46.056646 ms | 4.202x |
| `IntersectionBoardBench_marty` | 0.987500 ms | 1.404764 ms | 1.423x |
| `IntersectionBoardBench_paper` | 0.375300 ms | 0.472792 ms | 1.260x |
| `IntersectionTileBench` | 4.135000 ms | 4.634858 ms | 1.121x |
| `IntersectionTileBenchWithOverlap` | 3.922000 ms | 221.250188 ms | 56.413x |
| `IterateRawPath` | 3.047000 ms | 3.324034 ms | 1.091x |
| `MapPointsAffine` | 2.064000 ms | 3.601213 ms | 1.745x |
| `MapPointsScaleTrans` | 1.966000 ms | 3.625250 ms | 1.844x |
| `MeasurePath` | 44.580000 ms | 480.527146 ms | 10.779x |
| `RawPathBounds` | 0.435800 ms | 0.509179 ms | 1.168x |

The largest investigation signals in this run are `BuildRawPath`, the
overlap-allowed intersection tile, and path measurement. The draw and map
ratios must be interpreted with the primitive-level differences documented in
`docs/upstream-microbenchmarks.md`; none of them compare a complete Rust frame
against a complete C++ frame.
