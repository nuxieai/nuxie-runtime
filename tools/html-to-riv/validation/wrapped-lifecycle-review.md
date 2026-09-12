# Immutable native lifecycle cost of rounded wrapping

The current ordinary composition runs at 1, 3, 8 and 34 visible owners, but cost grows substantially. This is measured resource evidence, not public wrapping admission or a production performance guarantee. At 34 owners, the optimized native probe has median import, initial settle and clone times around 1.4 seconds each and median resize/update around 29 milliseconds on the recorded machine. Passing the private 100,000-record arithmetic budget therefore does not establish practical usability.

## Method and identity

`wrapped-lifecycle-probe.rs` imports an ordinary `.riv` without source maps, CSS policy or setters. Each trial creates a fresh RecordingFactory, imports once, settles, clones the original once, then runs 320×200 → 160×320 → 96×240 → 320×200 on the same original and clone. It times factory setup, import, initial settle, clone, each resize/update, each draw-recording and explicit release of clone/original/file/factory. It checks finite world transforms outside timing regions. File reads and JSON output are excluded from these stage timings.

Draw means CPU RecordingFactory command construction, including frame setup/finalization. It does **not** measure GPU rendering, submission, presentation or raster output. Release time records explicit object drops; it does not prove allocator return to the OS or absence of retained memory. Peak RSS is the entire process high-water mark over all trials, including recording buffers, measured with Darwin `/usr/bin/time -l`; it is not live-scene or leaked memory.

There are two warmups followed by nine measured fresh-import trials per scene in each profile. Stage medians and nearest-rank p95 values come from those nine trials; frame distributions pool 72 original/clone resize occurrences per scene. Those occurrences are correlated, and p95 for nine trials is the maximum. Raw timings remain available. Each profile ran sequentially after the coordinator confirmed builds, native captures and observer work had stopped. This controls competing task work, not all operating-system background activity.

Hardware: Apple M5 Max, 18 logical CPUs, 128 GiB RAM, arm64 macOS 26.6.2 (25G83). Both toolchains use unchanged baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3` and its own locked root Cargo dependency resolution/features. The development build disables debug symbols and incremental compilation; it is unoptimized. The second build uses the existing standard `--release` profile without profile overrides; the standalone observer links with `-O -C lto=fat -C codegen-units=1`, matching the baseline's release optimization/LTO policy. Probe source and scene bytes are identical across profiles. Build manifests bind commands, rustc version, feature graph, root manifests, source guard and native link libraries.

The first release observer link used only `-O`, allowing release LLVM 22 bitcode to reach Apple's LLVM 21 linker, which rejected it. Its failed command, source and logs remain in `wrapped-lifecycle-toolchain-release-r1`. The subsequent `release-r2` toolchain linked with rustc fat LTO successfully. No runtime code, dependency or release profile was changed to resolve the observer-link issue.

## Results

Optimized medians in milliseconds (GPU work excluded):

| Owners | Records | File bytes | Import | Settle | Clone | Resize/update | Draw recording | Release | Peak RSS MiB |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 2,359 | 38,849 | 1.507 | 1.528 | 1.162 | 0.374 | 0.015 | 0.410 | 18.78 |
| 3 | 7,239 | 118,956 | 7.809 | 8.472 | 7.119 | 1.143 | 0.067 | 1.280 | 34.75 |
| 8 | 19,999 | 330,171 | 54.855 | 58.016 | 53.417 | 4.164 | 0.431 | 3.690 | 81.78 |
| 34 | 99,247 | 1,658,530 | 1,385.640 | 1,396.172 | 1,370.399 | 29.234 | 7.842 | 23.722 | 405.22 |

The unoptimized import medians were 10.609, 45.169, 232.451 and 5,267.046 ms respectively; resize/update medians were 2.870, 9.005, 26.512 and 135.040 ms. Complete min/median/p95/max distributions and raw trials for both profiles are preserved in `wrapped-lifecycle-r1` and `wrapped-lifecycle-release-r1`. `wrapped-lifecycle-evidence.py` recomputes summaries and matches all input identities without running another benchmark.

The 34-owner fixture is the maximum accepted owner count for this exact one-paint recipe under the private 100,000-record budget: `16N² + 2376N − 33`. The separate construction evidence rejects 35 owners (102,727 records) before scene output. Native `objects()` omits the file Backboard, so its reported object count is one below the encoded record count.

## Remaining work

The current helper count is a material unresolved cost before public admission. The next compiler investigation is sharing graphs, folding proven constants and using source-bound coordinate lattices to avoid unnecessary rounding helpers, while preserving exact finite-domain evidence. Optimizing ordinary file compositions is permitted; modifying the runtime to improve these measurements is excluded. A practical public resource policy needs realistic design sizes, GPU measurements and lifecycle/memory testing beyond process completion and explicit drop timings. This campaign has four fixed-size-owner fixtures and cannot qualify arbitrary authoring semantics or hardware.
