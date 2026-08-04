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
