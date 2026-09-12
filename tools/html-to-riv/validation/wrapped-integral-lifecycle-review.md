# Integral paint lifecycle comparison

The source-certified integral paint path substantially reduces the cost of these four fixed-size-owner fixtures. The improvement comes from smaller ordinary scene graphs on the **same unchanged optimized native runtime/probe**, not a runtime modification. The 34-owner case still takes roughly 71 ms to import, 75 ms to settle and 70 ms to clone; this is not a general public cost qualification.

## Matched measurement scope

Compared `output/wrapped-integral-lifecycle-release-r1` against preserved `output/wrapped-lifecycle-release-r1`. Probe SHA-256 `d7b5ab700580ede370992258374b2e037083adfdef375665b20cbfcb53af3f64`, toolchain manifest, probe source, benchmark runner and hardware description match exactly. The frozen runtime uses baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3`, its own locked Cargo graph and standard release profile, with matching optimized fat-LTO probe linkage. Both runs used Apple M5 Max, 18 logical CPUs, 128 GiB RAM and arm64 macOS 26.6.2. Root manifests, frozen build inputs and native link-library identities are bound separately from the source guard.

The numeric recipes match after dropping added source-provenance strings and equivalent absent/null optional fields. The scene bytes intentionally differ. Each scene was imported freshly for two warmups and nine measured trials. Each trial settled, cloned once, then resized/updated/drew the same original and clone through 320×200 → 160×320 → 96×240 → 320×200. Stage timings exclude file reads and JSON serialization. Both campaigns ran after the coordinator declared other builds, captures and observer work terminal; operating-system background work is not controlled.

The offline `wrapped-integral-lifecycle-evidence.py` verifies original/clone schedules, recomputes all recorded min/median/p95/max distributions directly from raw trials, checks scene byte lengths and native object counts against constructor proof costs, and checks process peak RSS against `/usr/bin/time -l` logs. It does not rerun a benchmark. Medians for import/settle/clone/release use nine trials; resize/draw distributions pool 72 correlated occurrences per scene. Nearest-rank p95 of nine trials is the maximum, not a robust population estimate.

## New measurements

Integral-path medians in milliseconds:

| Owners | Records | File bytes | Import | Settle | Clone | Resize/update | CPU draw recording | Release | Peak RSS MiB |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 44 | 597 | 0.023708 | 0.028750 | 0.015250 | 0.007417 | 0.006500 | 0.010875 | 10.438 |
| 3 | 279 | 3,714 | 0.134750 | 0.157334 | 0.100416 | 0.040105 | 0.027459 | 0.059875 | 11.922 |
| 8 | 1,339 | 17,666 | 0.814125 | 0.937584 | 0.675833 | 0.188250 | 0.171187 | 0.334334 | 18.438 |
| 34 | 17,732 | 229,490 | 71.163333 | 74.682291 | 69.571834 | 4.506375 | 3.500792 | 6.108042 | 163.969 |

Rounded-path median divided by integral-path median (larger means faster in this experiment):

| Owners | Import ratio | Settle ratio | Clone ratio | Resize/update ratio | CPU draw-recording ratio | Release ratio |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 63.56× | 53.14× | 76.21× | 50.36× | 2.35× | 37.71× |
| 3 | 57.95× | 53.84× | 70.90× | 28.49× | 2.43× | 21.37× |
| 8 | 67.38× | 61.88× | 79.04× | 22.12× | 2.52× | 11.04× |
| 34 | 19.47× | 18.69× | 19.70× | 6.49× | 2.24× | 3.88× |

The earlier rounded scenes contained 2,359 / 7,239 / 19,999 / 99,247 records and 38,849 / 118,956 / 330,171 / 1,658,530 bytes. The new 34-owner count is **17,732**, confirmed by the emitted-construction cost and 17,731 runtime objects (Backboard excluded). The earlier planning table's 17,731 encoded-record estimate was an arithmetic error; its formula `(27N² + 127N − 66)/2` correctly yields 17,732.

## Limits and next implications

These are four closed all-integral resource recipes, not arbitrary HTML/CSS designs or a complete resource-budget qualification. Mixed and rounded fallback scenes retain additional graph costs. The remaining quadratic replica/line-gate graph is visible in the 34-owner measurements. Public resource policy, broader realistic designs and further compiler graph optimization remain open; immutable runtime changes are excluded.

Draw timing measures CPU RecordingFactory command construction, including frame bookkeeping. It is **not GPU rendering, submission or presentation latency**. Release timing measures explicit object drops. Peak RSS is a whole-process high-water mark across trials, including allocator/recording buffers; it does not measure retained scene memory or prove absence of leaks. The matched probe's finite-world checks and successful process completion are useful lifecycle evidence, but GPU behavior and retained-memory qualification require separate work. No tolerance, runtime, dependency or viewport-domain change explains these results.
