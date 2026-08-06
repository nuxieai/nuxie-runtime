# Upstream C++/Rust microbenchmark evidence — 2026-08-06

This evidence measures exact committed Rust source
`d7b0bea057154657f92e1e221b046198a98091da` against pinned rive-runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

- Run ID: `20260806T155659Z-d7b0bea05715`
- Benchmark-content SHA-256:
  `0838fcb90b195ec24768c27c524f004c978833edf3fbdcdafc6ce97ee7927a17`
- Inventory SHA-256:
  `666f97fc3da7e2452954ca22e6a80acbc151d0daf4a471db39d7c2189388c8ac`
- Pinned committed C++ source archive SHA-256:
  `7e4706ec5e02fc9da5a16ad36dd93bdad52a96a3edb037c0fa8a3b2a800b2d5a`
- Run-scoped C++ release benchmark binary SHA-256:
  `5908ee2cfd8359c5773349525113469b0042fea220bbb903ab55e41917726d92`
- C++ build log SHA-256:
  `d97226b9371d96153a325a55eda111b0f90b7a041e27645a09169f01ff665b30`
- Raw C++ output SHA-256:
  `470849a027181827bd52435ca40fe50488ad72942ca5274b7b6930bc00b9cfc0`
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
`sample.json` for every case. The pinned inventory also hashes the upstream
`RenderContextNULL` capability source, and the gate verifies that it enables
RasterOrdering. The report below was regenerated from that manifest by
`make microbench-compare`.

Ten semantically equivalent cases receive direct ratios. The ten `Draw*` rows
remain directional because upstream `RenderContextNULL` selects
`RasterOrdering` while Rust exercises `ClockwiseAtomic`; equal interlock mode is
tracked by [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727).

## Equivalent boundaries (minimum sample versus minimum sample)

| Benchmark | C++ | Rust | Rust/C++ |
|---|---:|---:|---:|
| `BuildRawPath` | 2.231000 ms | 3608.114833 ms | 1617.263x |
| `IntersectionBoardBench_marty` | 0.752400 ms | 1.011916 ms | 1.345x |
| `IntersectionBoardBench_paper` | 0.296100 ms | 0.317083 ms | 1.071x |
| `IntersectionTileBench` | 3.527000 ms | 3.929500 ms | 1.114x |
| `IntersectionTileBenchWithOverlap` | 3.366000 ms | 200.098792 ms | 59.447x |
| `IterateRawPath` | 3.066000 ms | 3.072958 ms | 1.002x |
| `MapPointsAffine` | 2.120000 ms | 2.490542 ms | 1.175x |
| `MapPointsScaleTrans` | 1.992000 ms | 2.579416 ms | 1.295x |
| `MeasurePath` | 56.180000 ms | 226.533459 ms | 4.032x |
| `RawPathBounds` | 0.442800 ms | 0.335500 ms | 0.758x |

## Directional timings (not ratio-comparable)

| Benchmark | C++ workload | Rust primitive | Why no ratio |
|---|---:|---:|---|
| `DrawCustomFeathers` | 175.700000 ms | 746.859542 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawFeatheredPaths_paper` | 68.960000 ms | 7027.223541 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneChopStrokes` | 22.800000 ms | 95.072583 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneCuspStrokes` | 39.700000 ms | 142.600708 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPaths` | 8.039000 ms | 65.899417 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsRoundJoinStrokes` | 13.990000 ms | 57.512541 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsStrokes` | 17.300000 ms | 52.628916 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoChopStrokes` | 37.810000 ms | 128.456084 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoCuspStrokes` | 108.400000 ms | 185.345834 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawZeroChopStrokes` | 11.920000 ms | 52.473916 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
