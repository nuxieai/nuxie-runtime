# Upstream microbenchmark baseline — 2026-08-06

This is the first reproducible 20-case baseline for
[UNIV-1688](https://universe.basis.dev/issue/UNIV-1688). Every row is a direct
comparison at an equivalent production-compiled boundary. The run manifest was
accepted by `make microbench-compare`, including its clean-revision, inventory,
binary, C++ output, and 20 Criterion sample-file hash checks.

## Provenance

- Run ID: `20260806T070538Z-ed9dd9279118`
- Nuxie revision: `ed9dd927911862039ff1ca9083ac4f9857f95306`
- Pinned upstream revision: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`
- Inventory SHA-256: `e01d90ea7eff82d9e4e28a72387dae9d049113fcbbf109c03d97e758e8d66a07`
- C++ benchmark binary SHA-256: `31b002e6620ac36ae9ad8834099411e3a778de7d206529f037055c4520d00c5d`
- C++ output SHA-256: `89b877d8a7297d0782e5e1fd8f4f8721d6d05a673f094855b95e733db93da074`
- Host: Apple arm64, Darwin 25.5.0
- Rust: `rustc 1.97.1 (8bab26f4f 2026-07-14)`
- Cargo: `cargo 1.97.1 (c980f4866 2026-06-30)`

The upstream harness ran each case for five seconds. Criterion used a three-second
warm-up, a requested ten-second measurement window, and 20 samples. Criterion
automatically lengthened a window when necessary to collect all 20 samples. Both
sides report the minimum elapsed nanoseconds per iteration, matching the upstream
harness statistic.

## Results

| Benchmark | C++ | Rust | Rust/C++ |
|---|---:|---:|---:|
| `BuildRawPath` | 2.215000 ms | 3488.486333 ms | 1574.937x |
| `DrawCustomFeathers` | 177.800000 ms | 446.656563 ms | 2.512x |
| `DrawFeatheredPaths_paper` | 66.340000 ms | 187.530333 ms | 2.827x |
| `DrawOneChopStrokes` | 22.030000 ms | 86.135333 ms | 3.910x |
| `DrawOneCuspStrokes` | 40.890000 ms | 107.154950 ms | 2.621x |
| `DrawRiveRenderPaths` | 7.012000 ms | 23.735899 ms | 3.385x |
| `DrawRiveRenderPathsAsRoundJoinStrokes` | 12.610000 ms | 44.839311 ms | 3.556x |
| `DrawRiveRenderPathsAsStrokes` | 12.460000 ms | 72.095396 ms | 5.786x |
| `DrawTwoChopStrokes` | 30.490000 ms | 92.884717 ms | 3.046x |
| `DrawTwoCuspStrokes` | 78.210000 ms | 128.527552 ms | 1.643x |
| `DrawZeroChopStrokes` | 9.253000 ms | 36.664042 ms | 3.962x |
| `IntersectionBoardBench_marty` | 0.702700 ms | 1.271657 ms | 1.810x |
| `IntersectionBoardBench_paper` | 0.293400 ms | 0.453507 ms | 1.546x |
| `IntersectionTileBench` | 3.151000 ms | 4.083265 ms | 1.296x |
| `IntersectionTileBenchWithOverlap` | 3.226000 ms | 209.822521 ms | 65.041x |
| `IterateRawPath` | 2.956000 ms | 3.376458 ms | 1.142x |
| `MapPointsAffine` | 1.996000 ms | 2.819847 ms | 1.413x |
| `MapPointsScaleTrans` | 1.993000 ms | 2.523757 ms | 1.266x |
| `MeasurePath` | 38.450000 ms | 234.278667 ms | 6.093x |
| `RawPathBounds` | 0.435400 ms | 0.342821 ms | 0.787x |

The ratios are diagnostic baselines, not performance gates. In particular, they
make path construction, overlapping tile intersection, path measurement, and
stroke preparation concrete follow-up optimization targets.
