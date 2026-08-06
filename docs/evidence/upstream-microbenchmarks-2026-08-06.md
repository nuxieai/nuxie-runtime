# Upstream C++/Rust microbenchmark evidence — 2026-08-06

This evidence measures exact committed Rust source
`ee4f4b5744e517f9acafc57bb9665770161a9b3a` against pinned rive-runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

- Run ID: `20260806T181452Z-ee4f4b5744e5`
- Benchmark-content SHA-256:
  `5d575e311f22483b9331516447cf7bd7e93b03e7161c916a86e1c0c239f66158`
- Inventory SHA-256:
  `666f97fc3da7e2452954ca22e6a80acbc151d0daf4a471db39d7c2189388c8ac`
- Pinned committed C++ source archive SHA-256:
  `7e4706ec5e02fc9da5a16ad36dd93bdad52a96a3edb037c0fa8a3b2a800b2d5a`
- Sealed C++ build-input record SHA-256:
  `7f91c4e06020a25125cc3e74f7f5cfaf4ee3feed9cd167afad998091cb8fa4d4`
- Run-scoped C++ release benchmark binary SHA-256:
  `5908ee2cfd8359c5773349525113469b0042fea220bbb903ab55e41917726d92`
- C++ build log SHA-256:
  `d2e58fc1fe3c0e0ac4bd4d394a1d16c8372e1d9f939741c5c3fb159f141b6411`
- Raw C++ output SHA-256:
  `5eaf331bdd2c5aef6d11a8c741054168ddb3a12da581aa795bf659580050ed95`
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
`87fd70a643c8078b74dbc60a4894bb6941f390a0e17022cbdd58f2ca6fa582aa`)
and `tests/dependencies` (7,143 entries,
`ab9ae198b7147117cde7a9bfef16d4d7103f5b37899560bfb4b41e7523aa6e12`).

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
| `BuildRawPath` | 2.232000 ms | 3561.358250 ms | 1595.591x |
| `IntersectionBoardBench_marty` | 0.721500 ms | 0.926209 ms | 1.284x |
| `IntersectionBoardBench_paper` | 0.289200 ms | 0.313208 ms | 1.083x |
| `IntersectionTileBench` | 3.136000 ms | 4.151458 ms | 1.324x |
| `IntersectionTileBenchWithOverlap` | 3.420000 ms | 217.660750 ms | 63.643x |
| `IterateRawPath` | 3.006000 ms | 3.055667 ms | 1.017x |
| `MapPointsAffine` | 2.083000 ms | 2.466209 ms | 1.184x |
| `MapPointsScaleTrans` | 1.993000 ms | 2.465750 ms | 1.237x |
| `MeasurePath` | 54.840000 ms | 241.910291 ms | 4.411x |
| `RawPathBounds` | 0.443500 ms | 0.297666 ms | 0.671x |

## Directional timings (not ratio-comparable)

| Benchmark | C++ workload | Rust primitive | Why no ratio |
|---|---:|---:|---|
| `DrawCustomFeathers` | 187.000000 ms | 854.596125 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawFeatheredPaths_paper` | 69.770000 ms | 7128.833500 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneChopStrokes` | 25.080000 ms | 84.653250 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawOneCuspStrokes` | 58.670000 ms | 137.709458 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPaths` | 11.380000 ms | 70.165708 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsRoundJoinStrokes` | 22.830000 ms | 71.873625 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawRiveRenderPathsAsStrokes` | 16.260000 ms | 60.747833 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoChopStrokes` | 35.900000 ms | 124.498334 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawTwoCuspStrokes` | 91.250000 ms | 172.502125 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
| `DrawZeroChopStrokes` | 10.460000 ms | 47.428458 ms | upstream RenderContextNULL selects RasterOrdering while Rust exercises ClockwiseAtomic; track equal interlock mode in [UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) |
