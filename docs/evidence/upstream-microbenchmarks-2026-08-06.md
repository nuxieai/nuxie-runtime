# Upstream C++/Rust microbenchmark evidence — 2026-08-06

This evidence measures exact committed Rust source
`10c51b19d4587f6ec9b8ceb62cf8469d3234cfa2` against pinned rive-runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

- Run ID: `20260806T162549Z-10c51b19d458`
- Benchmark-content SHA-256:
  `78220e56000c67ec5fcdc53ec0ee16fbd116c796a2b9e02e08bc920dec4d8266`
- Inventory SHA-256:
  `666f97fc3da7e2452954ca22e6a80acbc151d0daf4a471db39d7c2189388c8ac`
- Pinned committed C++ source archive SHA-256:
  `7e4706ec5e02fc9da5a16ad36dd93bdad52a96a3edb037c0fa8a3b2a800b2d5a`
- Run-scoped C++ release benchmark binary SHA-256:
  `5908ee2cfd8359c5773349525113469b0042fea220bbb903ab55e41917726d92`
- C++ build log SHA-256:
  `72a5d37e996ba91d5ce881e5c2969afa608776ce806c0e0c53e0e5f6482cd496`
- Raw C++ output SHA-256:
  `f4e6dfdb73d89720719806f124fb28ee1ca46fb0bfbb677e1d8a09c085f0d3c1`
- Toolchain: `rustc 1.97.1 (8bab26f4f 2026-07-14) (Homebrew)`,
  `cargo 1.97.1 (c980f4866 2026-06-30) (Homebrew)`, and Apple clang 21.0.0
- Host: macOS Darwin 25.5.0, arm64
- C++ duration: 5 seconds per case
- Criterion: 3-second warmup, 10-second requested measurement, 20 samples;
  Criterion extended heavy cases as required to collect all 20 samples
- Statistic: minimum individually timed invocation from each harness

The runner validated a clean checkout at the pinned C++ commit, archived only
its committed source into the run namespace, and built the release `bench`
target there. The sealed manifest has the exact v4 schema and exact 25-artifact
set: inventory, committed C++ source archive, build log, built binary, C++
output, and one raw Criterion `sample.json` per case. Comparison validated each
artifact's canonical path and hash, verified one common Criterion run namespace,
and loaded timings only through the sealed artifact entries. The pinned
inventory also hashes the upstream `RenderContextNULL` capability source, and
the gate verifies that it enables RasterOrdering. The report below was
regenerated from that manifest by `make microbench-compare`.

Ten semantically equivalent cases receive direct ratios. The ten `Draw*` rows
remain directional because upstream `RenderContextNULL` selects
`RasterOrdering` while Rust exercises `ClockwiseAtomic`; equal interlock mode is
tracked by [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727).

## Equivalent boundaries (minimum sample versus minimum sample)

| Benchmark | C++ | Rust | Rust/C++ |
|---|---:|---:|---:|
| `BuildRawPath` | 2.201000 ms | 3611.670958 ms | 1640.923x |
| `IntersectionBoardBench_marty` | 0.722200 ms | 0.963333 ms | 1.334x |
| `IntersectionBoardBench_paper` | 0.288700 ms | 0.423250 ms | 1.466x |
| `IntersectionTileBench` | 3.119000 ms | 4.037333 ms | 1.294x |
| `IntersectionTileBenchWithOverlap` | 3.194000 ms | 241.393541 ms | 75.577x |
| `IterateRawPath` | 2.933000 ms | 3.049000 ms | 1.040x |
| `MapPointsAffine` | 1.997000 ms | 2.865542 ms | 1.435x |
| `MapPointsScaleTrans` | 1.974000 ms | 2.456667 ms | 1.245x |
| `MeasurePath` | 39.640000 ms | 228.409208 ms | 5.762x |
| `RawPathBounds` | 0.435700 ms | 0.298042 ms | 0.684x |

## Directional timings (not ratio-comparable)

| Benchmark | C++ workload | Rust primitive | Why no ratio |
|---|---:|---:|---|
| `DrawCustomFeathers` | 180.300000 ms | 708.688500 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawFeatheredPaths_paper` | 69.200000 ms | 6519.957167 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneChopStrokes` | 22.820000 ms | 79.980334 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneCuspStrokes` | 39.250000 ms | 129.733000 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPaths` | 7.457000 ms | 65.037250 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsRoundJoinStrokes` | 13.450000 ms | 55.934750 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsStrokes` | 13.210000 ms | 56.149667 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoChopStrokes` | 31.270000 ms | 112.277041 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoCuspStrokes` | 83.910000 ms | 165.516666 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawZeroChopStrokes` | 10.620000 ms | 43.778875 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
