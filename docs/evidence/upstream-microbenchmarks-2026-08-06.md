# Upstream C++/Rust microbenchmark evidence — 2026-08-06

This evidence measures the exact committed Rust source
`30efffee8e812bc9cf924937085b5945c811427b` against pinned rive-runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

- Run ID: `20260806T143649Z-30efffee8e81`
- Benchmark-content SHA-256:
  `027d290b299713c9a6db7bebc6ce443e24ca22f1a6a269ecc3b6b2f13dbcbf55`
- Inventory SHA-256:
  `8ee2dbe518bd488b39e883c61bdf6483ad67301eaee03de99102f813668fd8ba`
- Pinned C++ release benchmark binary SHA-256:
  `31b002e6620ac36ae9ad8834099411e3a778de7d206529f037055c4520d00c5d`
- Raw C++ output SHA-256:
  `f2c055c40a75e741b8a0b6ea827225f777842ba3d018a05f940e4c158887a203`
- Toolchain: `rustc 1.97.1 (8bab26f4f 2026-07-14) (Homebrew)` and
  `cargo 1.97.1 (c980f4866 2026-06-30) (Homebrew)`
- Host: macOS Darwin 25.5.0, arm64
- C++ duration: 5 seconds per case
- Criterion: 3-second warmup, 10-second requested measurement, 20 samples;
  Criterion extended heavy cases as required to collect all 20 samples
- Statistic: minimum elapsed nanoseconds per iteration from each harness

The sealed run manifest records and hashes all 23 artifacts: inventory, C++
binary/output, and the raw Criterion `sample.json` for every case. The report
below was generated from that manifest by `make microbench-compare`.

## Equivalent boundaries (minimum sample versus minimum sample)

| Benchmark | C++ | Rust | Rust/C++ |
|---|---:|---:|---:|
| `BuildRawPath` | 2.202000 ms | 3506.256208 ms | 1592.305x |
| `DrawCustomFeathers` | 177.500000 ms | 734.778875 ms | 4.140x |
| `DrawFeatheredPaths_paper` | 66.640000 ms | 6299.903875 ms | 94.536x |
| `DrawOneChopStrokes` | 21.960000 ms | 84.183299 ms | 3.833x |
| `DrawOneCuspStrokes` | 37.500000 ms | 133.147729 ms | 3.551x |
| `DrawRiveRenderPaths` | 6.758000 ms | 58.917354 ms | 8.718x |
| `DrawRiveRenderPathsAsRoundJoinStrokes` | 12.490000 ms | 53.633705 ms | 4.294x |
| `DrawRiveRenderPathsAsStrokes` | 12.230000 ms | 52.752336 ms | 4.313x |
| `DrawTwoChopStrokes` | 30.000000 ms | 117.163225 ms | 3.905x |
| `DrawTwoCuspStrokes` | 78.610000 ms | 169.115236 ms | 2.151x |
| `DrawZeroChopStrokes` | 9.299000 ms | 51.013458 ms | 5.486x |
| `IntersectionBoardBench_marty` | 0.689900 ms | 1.150246 ms | 1.667x |
| `IntersectionBoardBench_paper` | 0.290000 ms | 0.485165 ms | 1.673x |
| `IntersectionTileBench` | 3.116000 ms | 3.976431 ms | 1.276x |
| `IntersectionTileBenchWithOverlap` | 3.192000 ms | 192.848347 ms | 60.416x |
| `IterateRawPath` | 2.898000 ms | 3.285135 ms | 1.134x |
| `MapPointsAffine` | 1.997000 ms | 2.610408 ms | 1.307x |
| `MapPointsScaleTrans` | 1.975000 ms | 2.565857 ms | 1.299x |
| `MeasurePath` | 39.860000 ms | 237.142541 ms | 5.949x |
| `RawPathBounds` | 0.437500 ms | 0.336815 ms | 0.770x |
