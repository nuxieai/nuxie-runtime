# Wasm advance+draw performance evidence

Status: **report-only**. This evidence establishes a baseline; it does not enforce a budget.

Git `da6c0cd8f6e7cd4ba8bb9652df0554c96e1ca486`; rive-runtime `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; browser `chrome 150.0.7871.189`; 100 segments/run; 1 discarded browser warmup(s).

| Fixture | Workload identity | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `text_vertical_trim_test` | state machine 0; VM schema-default | 10,467,863 | 0.400 ms (22.0%) | 1.922 ms (16.5%) | 0.208x | 0.095x | 0.425x |
| `data_bind_test_cmdq` | state machine 0; VM schema-default | 1,690,399 | 0.800 ms (10.7%) | 3.401 ms (12.6%) | 0.235x | 0.188x | 0.370x |
| `background_measure` | state machine 0; VM none | 1,301,468 | 0.100 ms (37.3%) | 0.315 ms (4.9%) | 0.317x | 0.711x | 5.506x |
| `hit_test_test` | state machine 0; VM schema-default | 883,710 | 1.500 ms (3.0%) | 1.188 ms (3.2%) | 1.263x | 1.271x | 3.073x |
| `component_list_1` | state machine 0; VM schema-default | 806,848 | 0.200 ms (50.0%) | 0.128 ms (12.1%) | 1.563x | 1.912x | 0.000x |

Each side uses fresh total and phase-pass instances, primes retained topology before timing, and reports multiple independent runs. Browser timings use monotonic `performance.now()` around the same production advance and draw calls as the native facade. Scripted, input-driven, and image-decoding fixtures are unsupported and fail closed; very small phase values may show browser-clock quantization.

Reproduce with `make wasm-perf`. The command writes the complete `nuxie-wasm-perf-v1` machine report, including all per-run timings, identity, accounted time, bookkeeping, and segment counts, to `target/wasm-perf.json` and the generated table to `target/wasm-perf.md`.
