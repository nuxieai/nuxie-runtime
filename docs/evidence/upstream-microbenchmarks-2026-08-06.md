# Upstream C++/Rust microbenchmark evidence — 2026-08-06

This evidence measures exact committed Rust source
`c13089a18a14218b0e25d954d9d6a79d3983a8fd` against pinned rive-runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

- Run ID: `20260806T152136Z-c13089a18a14`
- Benchmark-content SHA-256:
  `252a5169e434de80282da5a8a8fd2a844f7e5fc008b68b7b4aacd3bc96ce61f3`
- Inventory SHA-256:
  `b3847e3e9e732074287d94c58a9a8635a9f8de16bf9fe7d31fa439104dbdf599`
- Pinned committed C++ source archive SHA-256:
  `7e4706ec5e02fc9da5a16ad36dd93bdad52a96a3edb037c0fa8a3b2a800b2d5a`
- Run-scoped C++ release benchmark binary SHA-256:
  `5908ee2cfd8359c5773349525113469b0042fea220bbb903ab55e41917726d92`
- C++ build log SHA-256:
  `819bce6f25cc5d27481907020f2fa60eaf5432485dbb37a9e2d2a1932979af85`
- Raw C++ output SHA-256:
  `875690ff50ff28d08e05f7d049a68429e7cfb5c88b83cb86b07419476e2fc792`
- Toolchain: `rustc 1.97.1 (8bab26f4f 2026-07-14) (Homebrew)`,
  `cargo 1.97.1 (c980f4866 2026-06-30) (Homebrew)`, and Apple clang 21.0.0
- Host: macOS Darwin 25.5.0, arm64
- C++ duration: 5 seconds per case
- Criterion: 3-second warmup, 10-second requested measurement, 20 samples;
  Criterion extended heavy cases as required to collect all 20 samples
- Statistic: minimum individually timed invocation from each harness

The runner validated a clean checkout at the pinned C++ commit, archived only
its committed source into the run namespace, and built the release `bench`
target there. The sealed manifest hashes all 25 artifacts: inventory, committed
C++ source archive, build log, built binary, C++ output, and the raw Criterion
`sample.json` for every case. The report below was regenerated from that
manifest by `make microbench-compare`.

Ten semantically equivalent cases receive direct ratios. The ten `Draw*` rows
remain directional because upstream `RenderContextNULL` advertises
`rasterOrdering=false` while Rust exercises `ClockwiseAtomic`; equal capability
negotiation is tracked by
[UNIV-1727](https://universe.basis.dev/issue/UNIV-1727).

## Equivalent boundaries (minimum sample versus minimum sample)

| Benchmark | C++ | Rust | Rust/C++ |
|---|---:|---:|---:|
| `BuildRawPath` | 2.230000 ms | 3547.273334 ms | 1590.706x |
| `IntersectionBoardBench_marty` | 0.748900 ms | 0.936000 ms | 1.250x |
| `IntersectionBoardBench_paper` | 0.293000 ms | 0.315084 ms | 1.075x |
| `IntersectionTileBench` | 3.370000 ms | 4.057375 ms | 1.204x |
| `IntersectionTileBenchWithOverlap` | 3.341000 ms | 196.206875 ms | 58.727x |
| `IterateRawPath` | 3.079000 ms | 3.086208 ms | 1.002x |
| `MapPointsAffine` | 2.020000 ms | 2.495208 ms | 1.235x |
| `MapPointsScaleTrans` | 1.992000 ms | 2.511292 ms | 1.261x |
| `MeasurePath` | 47.190000 ms | 254.008166 ms | 5.383x |
| `RawPathBounds` | 0.442800 ms | 0.300583 ms | 0.679x |

## Directional timings (not ratio-comparable)

| Benchmark | C++ workload | Rust primitive | Why no ratio |
|---|---:|---:|---|
| `DrawCustomFeathers` | 204.200000 ms | 776.587792 ms | upstream RenderContextNULL advertises rasterOrdering=false while Rust exercises ClockwiseAtomic; track equivalent capability negotiation in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawFeatheredPaths_paper` | 106.500000 ms | 6872.931625 ms | upstream RenderContextNULL advertises rasterOrdering=false while Rust exercises ClockwiseAtomic; track equivalent capability negotiation in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneChopStrokes` | 28.330000 ms | 83.475167 ms | upstream RenderContextNULL advertises rasterOrdering=false while Rust exercises ClockwiseAtomic; track equivalent capability negotiation in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneCuspStrokes` | 48.710000 ms | 141.147500 ms | upstream RenderContextNULL advertises rasterOrdering=false while Rust exercises ClockwiseAtomic; track equivalent capability negotiation in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPaths` | 8.518000 ms | 63.836416 ms | upstream RenderContextNULL advertises rasterOrdering=false while Rust exercises ClockwiseAtomic; track equivalent capability negotiation in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsRoundJoinStrokes` | 17.580000 ms | 55.332208 ms | upstream RenderContextNULL advertises rasterOrdering=false while Rust exercises ClockwiseAtomic; track equivalent capability negotiation in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsStrokes` | 16.280000 ms | 56.087916 ms | upstream RenderContextNULL advertises rasterOrdering=false while Rust exercises ClockwiseAtomic; track equivalent capability negotiation in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoChopStrokes` | 37.850000 ms | 138.547333 ms | upstream RenderContextNULL advertises rasterOrdering=false while Rust exercises ClockwiseAtomic; track equivalent capability negotiation in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoCuspStrokes` | 82.370000 ms | 167.664583 ms | upstream RenderContextNULL advertises rasterOrdering=false while Rust exercises ClockwiseAtomic; track equivalent capability negotiation in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawZeroChopStrokes` | 10.230000 ms | 46.759416 ms | upstream RenderContextNULL advertises rasterOrdering=false while Rust exercises ClockwiseAtomic; track equivalent capability negotiation in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
