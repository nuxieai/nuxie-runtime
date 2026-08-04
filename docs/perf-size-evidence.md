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

The method remains the one above: ordinary C++ and Rust release runners,
100 sequential frames at 60 Hz, C++ first, no input script, and the median of
five runner-internal `advance + draw` sums divided by 100. Three complete
median-of-five sessions were captured to measure host variance. The table's
times and current ratio are the final session; the last column is the range of
the three independently aggregated ratios.

| Fixture | C++ ms/frame | Rust ms/frame | Current Rust/C++ | Three-session range |
|---|---:|---:|---:|---:|
| `text_vertical_trim_test` | 0.005308 | 0.107919 | 20.333x | 17.386–20.333x |
| `jellyfish_test` | 0.000508 | 0.005099 | 10.045x | 10.045–11.620x |
| `car_widgets_v01` | 0.040715 | 8.792481 | 215.954x | 201.884–234.455x |
| `zombie_skins` | 0.043068 | 0.949082 | 22.037x | 22.037–25.721x |
| `script_dependency_test_using_library_v2` | 0.000295 | 0.001440 | 4.882x | 4.882–6.023x |
| `script_dependency_test_using_library` | 0.000252 | 0.001360 | 5.403x | 4.987–6.024x |
| `data_bind_test_cmdq` | 0.004652 | 0.036308 | 7.805x | 6.673–8.017x |
| `viewmodel_based_condition` | 0.000686 | 0.003021 | 4.402x | 4.402–5.401x |
| `image_scripting_property_value` | 0.000220 | 0.001088 | 4.939x | 4.939–5.388x |
| `background_measure` | 0.001260 | 0.004231 | 3.359x | 2.788–3.359x |
| `audio_script` | 0.002383 | 0.012990 | 5.452x | 5.158–5.452x |
| `spotify_kids_demo` | 0.012361 | 0.194912 | 15.768x | 13.516–15.768x |
| `library_with_text_and_image` | 0.000242 | 0.000993 | 4.107x | 4.107–5.403x |
| `library` | 0.001362 | 0.003949 | 2.899x | 2.899–3.269x |
| `layout_grid_stack` | 0.001245 | 0.033588 | 26.970x | 25.415–28.455x |
| `gamepad_test` | 0.000541 | 0.002914 | 5.388x | 5.388–7.014x |
| `local_bounds` | 0.001918 | 0.005848 | 3.049x | 3.049–3.404x |
| `hit_test_test` | 0.000483 | 0.011596 | 24.033x | 24.033–25.501x |
| `multi_listeners` | 0.005059 | 0.021570 | 4.264x | 3.616–5.059x |
| `script_create_text_runs` | 0.002248 | 0.057086 | 25.399x | 25.399–31.212x |
| `virtualize_blendmode` | 0.000368 | 0.006390 | 17.387x | 16.789–18.659x |
| `component_list_1` | 0.000337 | 0.001321 | 3.924x | 3.924–4.164x |
| `collapsing_elements` | 0.004486 | 0.078826 | 17.572x | 17.572–18.727x |
| `clear_viewmodel_list` | 0.000272 | 0.000963 | 3.546x | 3.546–4.932x |

The machine was the same 18-core `Mac17,6` Apple Silicon host described above:
macOS 26.5.2 (25F84), `rustc 1.97.1`, and Homebrew clang 22.1.8. macOS has no
supported API for pinning a process to performance cores, so these sessions ran
under the default scheduler; `tools/perf-gate/run-pinned.sh` pins the comparator
and inherited runner processes to a highest-maximum-frequency CPU on Linux when
`taskset` and `lscpu` are available. The ratio range, rather than absolute time,
is the relevant stability evidence. The blocking ratchet uses the worst of the
three session medians as its current baseline, then applies the requested 15%
margin and integer ceiling. This deliberately absorbs the observed `car_widgets`
variance; a repeatedly flaky timing row must be removed or the gate disabled,
not papered over with an unrecorded ceiling increase.

The three raw reports were produced under `target/` (never `/tmp`) with SHA-256:

- `perf-gate-baseline.json`: `90f329925ccbbd18e84b3388667092dbd0b4b7f93f36f86454f5684680f7326c`
- `perf-gate-stability-2.json`: `fe4daf7c8ea49e51f17416193d27f87637a5cf2ccd5d5311f9ea224b303f0bf0`
- `perf-gate-stability-3.json`: `81d5fcb63260c17ceb0bdb43a582f88b5bd5ec02fa98d16918414923944d80b1`

The equivalent comparator arguments are:

```sh
--corpus corpus.toml --corpus-ids IDS_FROM_PERF_CORPUS \
--runner-benchmark --benchmark-frames 100 --benchmark-hz 60 \
--iterations 5 --warmups 0 --aggregate median --runner-order cpp-first
```
