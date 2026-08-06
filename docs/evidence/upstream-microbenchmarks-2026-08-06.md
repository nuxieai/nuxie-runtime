# Upstream C++/Rust microbenchmark evidence — 2026-08-06

This evidence measures exact committed Rust source
`2153266181a99734ff99ecfd2a4e2c180e5298bf` against pinned rive-runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

- Run ID: `20260806T173109Z-2153266181a9`
- Benchmark-content SHA-256:
  `69e5a950d1fd37a349490eb68cc9360adc90b4a72f72eecda9466fa521c6084d`
- Inventory SHA-256:
  `666f97fc3da7e2452954ca22e6a80acbc151d0daf4a471db39d7c2189388c8ac`
- Pinned committed C++ source archive SHA-256:
  `7e4706ec5e02fc9da5a16ad36dd93bdad52a96a3edb037c0fa8a3b2a800b2d5a`
- Sealed C++ build-input record SHA-256:
  `a930df9f3fb44b83c6d1d0aae0cbdd0e086910fbc5ee2c698a76285a00af44f5`
- Run-scoped C++ release benchmark binary SHA-256:
  `5908ee2cfd8359c5773349525113469b0042fea220bbb903ab55e41917726d92`
- C++ build log SHA-256:
  `ea7b43b62d232dfc72c9a80f50e2e3c729d787bf3f4ea44fa285cac827ae15ad`
- Raw C++ output SHA-256:
  `a3cf31790769e61825a0f93ed6cb57a3394494c8cecbfce11ae4f89a6c9eab8e`
- Toolchain: `rustc 1.97.1 (8bab26f4f 2026-07-14) (Homebrew)`,
  `cargo 1.97.1 (c980f4866 2026-06-30) (Homebrew)`, and Apple clang 21.0.0
- Host: macOS Darwin 25.5.0, arm64
- C++ duration: 5 seconds per case
- Criterion: 3-second warmup, 10-second requested measurement, 20 samples;
  Criterion extended heavy cases as required to collect all 20 samples
- Statistic: minimum individually timed invocation from each harness

The runner validated a clean checkout at the pinned C++ commit, archived only
its committed source into the run namespace, and built the release `bench`
target there. The C++ build starts from a fresh, constructed environment whose
only keys are `CC`, `CXX`, `GIT_CONFIG_GLOBAL`, `GIT_CONFIG_NOSYSTEM`,
`GIT_TERMINAL_PROMPT`, `HOME`, `LANG`, `LC_ALL`, `PATH`, `RIVE_BUILD_SYSTEM`,
`RIVE_CONFIG`, `RIVE_OUT`, `RIVE_PREMAKE_ARGS`, `RIVE_PREMAKE_TAG`, and
`TMPDIR`; ambient upstream-sensitive variables such as `RIVE_OS`, `RIVE_ARCH`,
`RIVE_VARIANT`, `DEPENDENCIES`, `PREMAKE_PATH`, `SDKROOT`, and compiler flags
are not inherited. The build-input record seals the exact command and
environment, eight resolved tool binaries, and deterministic identities for
`build/dependencies` (3,878 entries,
`e27c51f0a681080a8bfb0b902b1583d274e2fed502597d64995423ef59267748`)
and `tests/dependencies` (7,143 entries,
`60d75d0e95399ea47ae5f06cd3d92183b44b0c59c7cf31c3e2c08e5887d28ac9`).

The sealed manifest has the exact v5 schema and exact 26-artifact set:
inventory, committed C++ source archive, C++ build-input record, build log,
built binary, C++ output, and one raw Criterion `sample.json` per case.
Comparison validated each artifact's canonical path and hash, verified one
common Criterion run namespace, then parsed the retained bytes read during that
single validation pass rather than reopening `cpp.txt` or Criterion samples.
It also revalidated the effective tool and dependency inputs. The pinned
inventory hashes the upstream `RenderContextNULL` capability source, and the
gate verifies that it enables RasterOrdering. The report below was regenerated
from that manifest by `make microbench-compare`.

The comparison contract requires exactly ten named ratio cases (`BuildRawPath`,
`IntersectionBoardBench_marty`, `IntersectionBoardBench_paper`,
`IntersectionTileBench`, `IntersectionTileBenchWithOverlap`, `IterateRawPath`,
`MapPointsAffine`, `MapPointsScaleTrans`, `MeasurePath`, and `RawPathBounds`)
and exactly ten named directional cases (`DrawCustomFeathers`,
`DrawFeatheredPaths_paper`, `DrawOneChopStrokes`, `DrawOneCuspStrokes`,
`DrawRiveRenderPaths`, `DrawRiveRenderPathsAsRoundJoinStrokes`,
`DrawRiveRenderPathsAsStrokes`, `DrawTwoChopStrokes`, `DrawTwoCuspStrokes`, and
`DrawZeroChopStrokes`). The Draw rows remain directional because upstream
`RenderContextNULL` selects `RasterOrdering` while Rust exercises
`ClockwiseAtomic`; equal interlock mode is tracked by
[UNIV-1727](https://universe.basis.dev/issue/UNIV-1727).

## Equivalent boundaries (minimum sample versus minimum sample)

| Benchmark | C++ | Rust | Rust/C++ |
|---|---:|---:|---:|
| `BuildRawPath` | 2.200000 ms | 3553.022125 ms | 1615.010x |
| `IntersectionBoardBench_marty` | 0.718200 ms | 0.887584 ms | 1.236x |
| `IntersectionBoardBench_paper` | 0.289100 ms | 0.311833 ms | 1.079x |
| `IntersectionTileBench` | 3.122000 ms | 3.837792 ms | 1.229x |
| `IntersectionTileBenchWithOverlap` | 3.190000 ms | 201.593583 ms | 63.195x |
| `IterateRawPath` | 2.942000 ms | 3.026250 ms | 1.029x |
| `MapPointsAffine` | 1.997000 ms | 2.466792 ms | 1.235x |
| `MapPointsScaleTrans` | 1.960000 ms | 2.463458 ms | 1.257x |
| `MeasurePath` | 43.910000 ms | 216.460125 ms | 4.930x |
| `RawPathBounds` | 0.436000 ms | 0.299459 ms | 0.687x |

## Directional timings (not ratio-comparable)

| Benchmark | C++ workload | Rust primitive | Why no ratio |
|---|---:|---:|---|
| `DrawCustomFeathers` | 175.600000 ms | 746.537500 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawFeatheredPaths_paper` | 66.050000 ms | 6826.652084 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneChopStrokes` | 22.500000 ms | 83.122583 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneCuspStrokes` | 38.530000 ms | 135.426959 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPaths` | 7.432000 ms | 63.443750 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsRoundJoinStrokes` | 13.060000 ms | 54.777000 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsStrokes` | 13.000000 ms | 55.035250 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoChopStrokes` | 31.070000 ms | 118.240583 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoCuspStrokes` | 79.050000 ms | 174.478583 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawZeroChopStrokes` | 10.180000 ms | 45.652791 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
