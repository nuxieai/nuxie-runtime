# Upstream C++/Rust microbenchmark evidence — 2026-08-06

> Historical evidence: this run predates the RasterOrdering logical mode added
> by [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727). Its ten Draw rows
> remain intentionally directional; later evidence records the 20-ratio
> contract.

This evidence measures exact committed Rust source
`eb972bfdd7a93d56c64b444b6699aed51f5434f8` against pinned rive-runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

- Run ID: `20260806T184136Z-eb972bfdd7a9`
- Benchmark-content SHA-256:
  `d84602a58d4ec781497fbedca7430eea0658f8e7110a976f380ca36cf8a03f4a`
- Inventory SHA-256:
  `666f97fc3da7e2452954ca22e6a80acbc151d0daf4a471db39d7c2189388c8ac`
- Pinned committed C++ source archive SHA-256:
  `7e4706ec5e02fc9da5a16ad36dd93bdad52a96a3edb037c0fa8a3b2a800b2d5a`
- Sealed C++ build-input record SHA-256:
  `35169c63ab66d688bc1d0493e8661094c5031951ba2e0d3e8e33204967699b17`
- Run-scoped C++ release benchmark binary SHA-256:
  `5908ee2cfd8359c5773349525113469b0042fea220bbb903ab55e41917726d92`
- C++ build log SHA-256:
  `e3b5217616aed5f4f286401a1d58b6ace3076eb26c99e871882c8334ad1a59ef`
- Raw C++ output SHA-256:
  `ef8d819d022282a9433141359c65e1eb21a7e5e859f93beda90b8f99074e363a`
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
only keys are `AR`, `CC`, `CXX`, `DEVELOPER_DIR`, `GIT_CONFIG_GLOBAL`,
`GIT_CONFIG_NOSYSTEM`, `GIT_TERMINAL_PROMPT`, `HOME`, `LANG`, `LC_ALL`,
`PATH`, `RIVE_BUILD_SYSTEM`, `RIVE_CONFIG`, `RIVE_OUT`, `RIVE_PREMAKE_ARGS`,
`RIVE_PREMAKE_TAG`, `SDKROOT`, and `TMPDIR`; ambient upstream-sensitive
variables such as `RIVE_OS`, `RIVE_ARCH`, `RIVE_VARIANT`, `DEPENDENCIES`,
`PREMAKE_PATH`, `DEVELOPER_DIR`, `SDKROOT`, and compiler flags are not
inherited. Instead, the runner resolves the active developer directory with
`/usr/bin/xcode-select`, pins it in `DEVELOPER_DIR`, selects the macOS SDK with
that directory, and invokes the exact `xcrun`-selected `clang`, `clang++`,
`ar`, and `make` through the constructed environment and `PATH`. The
build-input record seals the Xcode `version.plist`, macOS `SDKSettings.json`,
the exact command and environment, 18 resolved tool invocation paths and
binaries (including the effective Xcode compiler, linker, archive, make,
Metal, and metallib tools rather than only `/usr/bin` dispatcher shims), and
deterministic identities for
`build/dependencies` (3,878 entries,
`5cf6eaa27df9eb036cc4d50dce0f2d5df0f425149e3a8133c60f44c3907c6275`)
and `tests/dependencies` (7,143 entries,
`7b34a3fc872cef03df2eb6ff53a210047afa473d869724bc293caacbed6fa56c`).

The sealed manifest has the exact v6 schema and exact 26-artifact set:
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
| `BuildRawPath` | 2.257000 ms | 3780.039334 ms | 1674.807x |
| `IntersectionBoardBench_marty` | 0.721000 ms | 0.919041 ms | 1.275x |
| `IntersectionBoardBench_paper` | 0.291900 ms | 0.315875 ms | 1.082x |
| `IntersectionTileBench` | 3.280000 ms | 4.049084 ms | 1.234x |
| `IntersectionTileBenchWithOverlap` | 3.352000 ms | 212.076042 ms | 63.269x |
| `IterateRawPath` | 3.151000 ms | 3.082167 ms | 0.978x |
| `MapPointsAffine` | 2.038000 ms | 2.476542 ms | 1.215x |
| `MapPointsScaleTrans` | 1.971000 ms | 2.466167 ms | 1.251x |
| `MeasurePath` | 46.410000 ms | 231.822917 ms | 4.995x |
| `RawPathBounds` | 0.435700 ms | 0.297000 ms | 0.682x |

## Directional timings (not ratio-comparable)

| Benchmark | C++ workload | Rust primitive | Why no ratio |
|---|---:|---:|---|
| `DrawCustomFeathers` | 191.300000 ms | 773.629083 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawFeatheredPaths_paper` | 80.730000 ms | 7277.044833 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneChopStrokes` | 26.070000 ms | 82.819500 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneCuspStrokes` | 44.330000 ms | 139.387750 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPaths` | 7.649000 ms | 67.240458 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsRoundJoinStrokes` | 14.520000 ms | 60.319625 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsStrokes` | 14.420000 ms | 63.837291 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoChopStrokes` | 34.260000 ms | 118.748708 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoCuspStrokes` | 92.570000 ms | 170.321792 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawZeroChopStrokes` | 11.550000 ms | 46.391000 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
