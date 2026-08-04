# Tier-5 performance and static-library size evidence

Measured 2026-08-04 on source revision `e1f33a42` against pinned
`rive-runtime` revision `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
These results are evidence, not a tolerance or budget change. They do not close
V10: Rust is slower than C++ on all five selected fixtures.

## Advance + draw wall time

The fixture set is the five largest distinct files enrolled in `corpus.toml`,
ranked by bytes on disk. Both ordinary (scripting-disabled) golden runners were
built with their release configurations. Each fresh runner process loaded and
instantiated its fixture before the runner-internal clock began, then advanced
and drew 100 sequential frames at 60 Hz (`t = 0/60` through `99/60`). No input
script was applied. Each runtime ran the 100-frame session five times. The
reported value is the median of the five per-run `advance + draw` phase sums,
divided by 100. C++ ran first for each paired fixture measurement.

`perf-compare --runner-benchmark` now derives `advance_draw` inside each run
before aggregation. That makes this the median of five combined measurements,
not the sum of independently selected advance and draw medians. Loading,
parsing, instantiation, stream serialization, input dispatch, prepare, and
process startup are outside this metric.

| Rank | Fixture | File bytes | C++ ms/frame | Rust ms/frame | Rust/C++ |
|---:|---|---:|---:|---:|---:|
| 1 | `text_vertical_trim_test` | 10,467,863 | 0.004629 | 0.117580 | 25.402x |
| 2 | `jellyfish_test` | 3,057,595 | 0.000500 | 0.011172 | 22.341x |
| 3 | `echo_show_demo` | 3,012,517 | 0.050823 | 2.085713 | 41.039x |
| 4 | `car_widgets_v01` | 2,875,164 | 0.035157 | 76.895463 | 2,187.196x |
| 5 | `zombie_skins` | 1,952,536 | 0.037295 | 1.494437 | 40.071x |
| **500-frame weighted aggregate** | — | — | **0.025681** | **16.120873** | **627.741x** |

The aggregate sums the five fixture medians and divides each runtime by 500
frames. The especially large `car_widgets_v01` result is not rounded away: its
100-frame Rust `advance + draw` measurements ranged from 7,413.899 ms to
7,795.899 ms, with a 7,689.546 ms median; C++ ranged from 3.459 ms to 3.572 ms,
with a 3.516 ms median.

The machine was an 18-core Apple Silicon arm64 host running macOS 26.5.2
(25F84), `rustc 1.97.1` (LLVM 22.1.8), and Apple clang 21.0.0. Measurements
were not CPU-pinned and the interactive host retained normal desktop
background activity, so the absolute wall times are point-in-time evidence;
the five-run ranges in the raw reports expose the observed spread.

Raw `rive-perf-compare-json-v1` reports:

- [`text_vertical_trim_test.json`](evidence/tier5-2026-08-04/text_vertical_trim_test.json)
- [`jellyfish_test.json`](evidence/tier5-2026-08-04/jellyfish_test.json)
- [`echo_show_demo.json`](evidence/tier5-2026-08-04/echo_show_demo.json)
- [`car_widgets_v01.json`](evidence/tier5-2026-08-04/car_widgets_v01.json)
- [`zombie_skins.json`](evidence/tier5-2026-08-04/zombie_skins.json)

The measurement commands were equivalent to:

```sh
make CPP_CONFIG=release RUST_PROFILE=release golden-runner rust-golden-runner
cargo build --release -p perf-compare --bin perf-compare
target/release/perf-compare \
  --cpp-runner tools/golden-runner/build/macosx/bin/release/rive_golden_runner \
  --rust-runner target/release/rust-golden-runner \
  --file /Users/levi/dev/oss/rive-runtime/tests/unit_tests/assets/FIXTURE.riv \
  --samples FRAME_0_THROUGH_99_AT_60_HZ \
  --iterations 5 --warmups 0 --aggregate median \
  --runner-order cpp-first --runner-benchmark --json REPORT.json
```

## Static-library bytes

The Rust runtime's shipped static archive is the `nux-capi` staticlib, a thin C
ABI over the runtime. It was built with
`cargo build --release -p nux-capi --no-default-features`. The comparator's
C++ `librive.a` was produced by the ordinary release golden-runner build.

| Release archive | Bytes | MiB | SHA-256 |
|---|---:|---:|---|
| Rust `target/release/libnux_capi.a` | 108,554,768 | 103.526 | `5992b7cc3307d9f3fcbbe35b5906d83571fde4186f0836f42d42d3950165fd5c` |
| C++ `target/golden-runner-librive/ordinary-release/librive.a` | 23,781,600 | 22.680 | `565439bd016b8280064d258ba7fa5f3d1598d19ac793e139659bd94c72105a68` |

The Rust archive is 84,773,168 bytes larger, or **4.565x** the raw C++ archive
size. Both values are unstripped on-disk `ar` archives. They are not final
application footprint: archive member layout differs between Rust fat-LTO and
C++, and a consuming linker dead-strips unreferenced members. The repository's
post-link SDK budget remains governed separately by `make size-report`; this
requested archive comparison does not replace or relax that gate.

## 2026-08-04 perf-parity fix-lane addendum

The measurements below use the same 100-frame, five-iteration median method
described above, with C++ first and `rive-runtime` pinned at `4ac7b327`.
Loading and process startup remain outside the reported `advance + draw`
metric. The lane baseline was remeasured at source revision `3f94fe1f` before
the first fix.

| Step | Fixture | C++ ms/frame | Rust ms/frame | Change from lane baseline |
|---|---|---:|---:|---:|
| Lane baseline | `car_widgets_v01` | 0.033261 | 69.710143 | — |
| Lane baseline | `zombie_skins` | 0.035936 | 1.361694 | — |
| Fix 1: retained opacity-owner index | `car_widgets_v01` | 0.033128 | 8.861663 | -87.29% |
| Fix 1: retained opacity-owner index | `zombie_skins` | 0.044959 | 0.912388 | -33.00% |
| Fix 2: structure-gated renderer tree initialization | `car_widgets_v01` | 0.033028 | 6.895138 | -90.11% |
| Fix 2: structure-gated renderer tree initialization | `zombie_skins` | 0.035613 | 0.590826 | -56.61% |
| Fix 3: clean prepare-to-draw occurrence boundary | `car_widgets_v01` | 0.033063 | 6.833802 | -90.20% |
| Fix 3: clean prepare-to-draw occurrence boundary | `zombie_skins` | 0.036598 | 0.582881 | -57.19% |

The branch's original comparator predated the `advance_draw` derived phase and
therefore printed total hot-loop time (including `prepare`). It was updated to
the method above before all three revisions were remeasured. Authoritative raw
reports are the `corrected-*` baseline/fix-1 files and the `fix2-*`/`fix3-*` files in
[`evidence/perffix-2026-08-04/`](evidence/perffix-2026-08-04/).

## 2026-08-04 V10 blocking-gate baseline

The V10 lane broadened the measurement to the 24 checked-in entries in
`perf-corpus.toml`. Nineteen are the largest practical exact, input-free files
from `corpus.toml`; five targeted rows add explicit scripted,
list/virtualization, nested-artboard, text, and layout coverage. The otherwise
size-eligible `data_viz_demo` is excluded because one current 100-frame Rust
session takes minutes, which would turn a landing ratchet into a soak test.

The gate uses scripting-enabled C++ and Rust release runners for the whole
corpus, and passes `--execute-scripts` to Rust. That keeps the runtime modes
paired and ensures the selected scripted rows exercise their script hot paths.
Each session measures 100 sequential frames at 60 Hz, C++ first, no input
script, and the median of five runner-internal `advance + draw` sums divided by
100. Four complete median-of-five sessions were captured to measure host
variance, with the fourth retained after it caught a boundary flake in the
initial ceiling. The displayed times are from session 4. The ratchet baseline is the
largest current ratio observed across the four sessions; the ceiling is
exactly `ceil(ratchet baseline * 1.15)`.

| Fixture | Session-4 C++ ms/frame | Session-4 Rust ms/frame | Ratchet baseline | Four-session range | Ceiling |
|---|---:|---:|---:|---:|---:|
| `text_vertical_trim_test` | 0.005206 | 0.099735 | 21.554x | 15.714–21.554x | 25x |
| `jellyfish_test` | 0.000554 | 0.005085 | 12.538x | 9.177–12.538x | 15x |
| `car_widgets_v01` | 0.023279 | 6.759487 | 290.367x | 230.441–290.367x | 334x |
| `zombie_skins` | 0.048665 | 0.994659 | 20.595x | 18.777–20.595x | 24x |
| `script_dependency_test_using_library_v2` | 0.001497 | 0.002252 | 1.504x | 1.029–1.504x | 2x |
| `script_dependency_test_using_library` | 0.001615 | 0.002923 | 1.810x | 1.210–1.810x | 3x |
| `data_bind_test_cmdq` | 0.007535 | 0.079167 | 12.560x | 10.507–12.560x | 15x |
| `viewmodel_based_condition` | 0.000901 | 0.003473 | 4.401x | 3.853–4.401x | 6x |
| `image_scripting_property_value` | 0.000467 | 0.008180 | 17.503x | 16.809–17.503x | 21x |
| `background_measure` | 0.001689 | 0.004175 | 3.139x | 2.471–3.139x | 4x |
| `audio_script` | 0.003238 | 0.010192 | 3.147x | 2.560–3.147x | 4x |
| `spotify_kids_demo` | 0.016930 | 0.214261 | 14.574x | 12.655–14.574x | 17x |
| `library_with_text_and_image` | 0.000241 | 0.000884 | 4.072x | 3.670–4.072x | 5x |
| `library` | 0.001357 | 0.003940 | 2.904x | 2.623–2.904x | 4x |
| `layout_grid_stack` | 0.001186 | 0.030558 | 25.770x | 24.016–25.770x | 30x |
| `gamepad_test` | 0.002286 | 0.390702 | 170.922x | 137.819–170.922x | 197x |
| `local_bounds` | 0.001964 | 0.007800 | 3.972x | 2.817–3.972x | 5x |
| `hit_test_test` | 0.003306 | 0.077831 | 27.593x | 22.983–27.593x | 32x |
| `multi_listeners` | 0.012141 | 0.041407 | 3.411x | 2.756–3.411x | 4x |
| `script_create_text_runs` | 0.005329 | 1.693305 | 327.917x | 268.874–327.917x | 378x |
| `virtualize_blendmode` | 0.006185 | 0.236123 | 38.179x | 32.178–38.179x | 44x |
| `component_list_1` | 0.005370 | 0.090869 | 16.920x | 13.588–16.920x | 20x |
| `collapsing_elements` | 0.005576 | 0.091679 | 18.878x | 15.115–18.878x | 22x |
| `clear_viewmodel_list` | 0.001284 | 0.026903 | 20.956x | 19.043–20.956x | 25x |

The machine was the same 18-core `Mac17,6` Apple Silicon host described above:
macOS 26.5.2 (25F84), `rustc 1.97.1`, and Homebrew clang 22.1.8. macOS has no
supported API for pinning a process to performance cores, so these sessions ran
under the default scheduler; `tools/perf-gate/run-pinned.sh` pins the comparator
and inherited runner processes to a highest-maximum-frequency CPU on Linux when
`taskset` and `lscpu` are available. The ratio range, rather than absolute time,
is the relevant stability evidence. The blocking ratchet uses the worst of the
four session medians as its current stable baseline, then applies the requested
15% margin and integer ceiling. This deliberately absorbs the observed
`car_widgets` variance; a repeatedly flaky timing row must be removed or the
gate disabled, not papered over with an unrecorded ceiling increase.

The four baseline reports and final validation report were produced under
`target/` (never `/tmp`) with SHA-256:

- `perf-gate-scripted-1.json`: `829b1a8a3196e0ed7d3ce003cb4bec1d3e43bb7f4e90245eaf928d6d369d7da6`
- `perf-gate-scripted-2.json`: `1c43bb7092db3f995341e26a8b06753720d06b7059a66da4b421d39b82ff350f`
- `perf-gate-scripted-3.json`: `39a402d83a8826f39fd119e58cb14374e48bdbc020d30c3fcac8ea4bd98a9ce9`
- first `perf-gate.json` out-of-sample validation (boundary failure): `c3348edc3a3a39996a36958cd98b90b304e7dd79d20f0ca98ff148cd54abb4ef`
- final green `perf-gate.json`: `63054da1decf28e9648793b2ccc5013bd75b7ef8fcecf71f8d40289af179ac3a`

The equivalent comparator arguments are:

```sh
--corpus corpus.toml --corpus-ids IDS_FROM_PERF_CORPUS \
--runner-benchmark --benchmark-frames 100 --benchmark-hz 60 \
--rust-execute-scripts \
--iterations 5 --warmups 0 --aggregate median --runner-order cpp-first
```

## 2026-08-04 retained text-topology addendum

This lane measured the three requested text-heavy fixtures before the first
change (`ed5b66e8`) and after the retained-topology implementation
(`7e993be7`). Both runs used the pinned C++ runtime at `4ac7b327`, release
scripted runners, 100 frames at 60 Hz, C++ first, script execution enabled,
and the median of five iterations with no warmups. The table reports the
runner's `advance + draw` phase divided by 100; loading, startup, and text
topology construction before the measured loop are excluded.

| Fixture | Baseline Rust ms/frame | Retained Rust ms/frame | Rust change | Retained Rust/C++ |
|---|---:|---:|---:|---:|
| `script_create_text_runs` | 1.527619 | 1.353785 | -11.38% | 319.635x |
| `text_vertical_trim_test` | 0.094717 | 0.101325 | +6.98% | 21.683x |
| `layout_text_match` | 0.171508 | 0.173455 | +1.14% | 11.843x |

`script_create_text_runs`, the fixture that repeatedly dirties text, removes
0.173834 ms/frame from the measured Rust hot loop. The two mostly static
fixtures moved by 0.006608 ms/frame and 0.001947 ms/frame respectively; those
small regressions are within the run-to-run host variance seen by the broader
performance gate and are recorded rather than normalized away. The
independent 24-row `make perf-gate` sample passed, including
`script_create_text_runs` at 1.406114 ms/frame (271.253x, ceiling 378x) and
`text_vertical_trim_test` at 0.081804 ms/frame (17.400x, ceiling 25x).

Raw reports and their SHA-256 digests are checked in under
[`evidence/texttop-2026-08-04/`](evidence/texttop-2026-08-04/):

- `baseline.json`: `dd9e2cc89e4504b9b82eef58179dbe26b27f6bc6258e890c646e891fb3b93445`
- `retained-topology.json`: `e1c05e420eceaadc0ab5a47e5fece50375f4ce44f5ec9781052ee384184c49a7`

The equivalent comparator arguments are:

```sh
--corpus corpus.toml \
--corpus-ids script_create_text_runs,text_vertical_trim_test,layout_text_match \
--runner-benchmark --benchmark-frames 100 --benchmark-hz 60 \
--rust-execute-scripts \
--iterations 5 --warmups 0 --aggregate median --runner-order cpp-first
```
