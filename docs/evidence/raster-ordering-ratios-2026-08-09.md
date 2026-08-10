# RasterOrdering direct-ratio evidence — 2026-08-09

[UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) requires the ten
upstream `Draw*` workloads to become direct Rust/C++ ratios only after the
production RasterOrdering semantic oracle passes. The oracle is recorded in
`raster-ordering-logical-oracle-2026-08-09.md`; this document records the
subsequent sealed timing run.

## Sealed run identity

- Run ID: `20260809T234911Z-a6bd0a29456d`
- Rust source: `a6bd0a29456d8ad1e3c782ee07aa0ffb24b5d0d0`
- Pinned rive-runtime source:
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`
- Benchmark content:
  `0903b783cd18fba44613b72d576d4a4ac8a13057a1e219d47395a790e1ad56b2`
- Inventory:
  `866d0a31b0bc5a68082220e17c59f00637e81b8f0fea8283501907efd5f9171b`
- C++ source archive:
  `7e4706ec5e02fc9da5a16ad36dd93bdad52a96a3edb037c0fa8a3b2a800b2d5a`
- C++ binary:
  `5908ee2cfd8359c5773349525113469b0042fea220bbb903ab55e41917726d92`
- Complete run manifest:
  `97603024f0831371c23b3f0ae78646ddd9e0a064e61475b6ea154adbbee6bc1b`
- Generated comparison:
  `9962ddd0c535483754677ebde86795b4db8854fd9024b4c99aefe145c7cc7077`

The run used the manifest's v6 contract, a five-second C++ duration, three
seconds of Criterion warmup, ten seconds of requested Criterion measurement,
20 Criterion samples, and the minimum individually timed invocation statistic
on both sides. `make microbench-compare` revalidated every sealed artifact and
produced exactly 20 ratio rows with no directional or blocked rows.

## Direct ratios

| Benchmark | C++ minimum | Rust minimum | Rust/C++ |
|---|---:|---:|---:|
| `BuildRawPath` | 2.255000 ms | 4014.631333 ms | 1780.324x |
| `DrawCustomFeathers` | 188.400000 ms | 1076.987208 ms | 5.716x |
| `DrawFeatheredPaths_paper` | 76.140000 ms | 7896.755208 ms | 103.714x |
| `DrawOneChopStrokes` | 25.550000 ms | 114.450291 ms | 4.479x |
| `DrawOneCuspStrokes` | 42.890000 ms | 161.107166 ms | 3.756x |
| `DrawRiveRenderPaths` | 7.446000 ms | 81.229083 ms | 10.909x |
| `DrawRiveRenderPathsAsRoundJoinStrokes` | 15.820000 ms | 72.073083 ms | 4.556x |
| `DrawRiveRenderPathsAsStrokes` | 14.890000 ms | 67.888459 ms | 4.559x |
| `DrawTwoChopStrokes` | 36.910000 ms | 156.549792 ms | 4.241x |
| `DrawTwoCuspStrokes` | 85.020000 ms | 186.778625 ms | 2.197x |
| `DrawZeroChopStrokes` | 11.000000 ms | 61.613750 ms | 5.601x |
| `IntersectionBoardBench_marty` | 0.725900 ms | 0.990084 ms | 1.364x |
| `IntersectionBoardBench_paper` | 0.292800 ms | 0.329250 ms | 1.124x |
| `IntersectionTileBench` | 3.369000 ms | 4.052625 ms | 1.203x |
| `IntersectionTileBenchWithOverlap` | 3.301000 ms | 203.902375 ms | 61.770x |
| `IterateRawPath` | 3.097000 ms | 3.066083 ms | 0.990x |
| `MapPointsAffine` | 2.075000 ms | 2.639833 ms | 1.272x |
| `MapPointsScaleTrans` | 1.970000 ms | 2.820667 ms | 1.432x |
| `MeasurePath` | 51.390000 ms | 261.301167 ms | 5.085x |
| `RawPathBounds` | 0.435600 ms | 0.309458 ms | 0.710x |

All ten `Draw*` rows are direct ratios over the production RasterOrdering
logical-frame boundary. The large ratios are optimization evidence, not failed
UNIV-1727 gates: the issue defines semantic equivalence and direct-ratio
availability, and does not define a maximum-ratio performance budget.

## Variance and ambient-load caveat

The machine retained a persistent `searchconsole-mcp` process and ordinary OS
application daemons that predated the run. They were treated as ambient
baseline and were not stopped or reprioritized. Before the sealed command
started, a targeted scan confirmed that all finite repository, browser, build,
`wasm-opt`, Cargo, and rustc jobs had exited. The runner then measured each
candidate/reference pair sequentially under the same ambient load.

Across the 20 Rust sample sets, 18 had a coefficient of variation below 13%.
The two exceptions were `BuildRawPath` at 32.41% and `MeasurePath` at 27.98%.
The minimum statistic used by the pinned C++ harness and the Rust mirror is
preserved in the table. Representative low-variance RasterOrdering rows were
`DrawRiveRenderPaths` at 1.58%, `DrawTwoChopStrokes` at 2.21%, and
`IntersectionTileBenchWithOverlap` at 3.20%. The raw 20-sample inputs and their
hashes remain bound by the complete run manifest.

## Acceptance verdict

The sealed timing qualification passes: 20 of 20 registered workloads produce
direct ratios, including all ten `Draw*` workloads; zero workloads remain
directional, blocked, or missing. Together with the committed semantic oracle,
this satisfies the ratio-evidence acceptance criteria for
[UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) without expanding into
renderer optimization work tracked elsewhere.
