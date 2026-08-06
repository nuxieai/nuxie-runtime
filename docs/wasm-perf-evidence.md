# Wasm advance+draw performance evidence

Status: **report-only**. This evidence establishes a baseline; it does not enforce a budget.

Git `486f184de7a2f3f1db0a2d668d3c774726239bfa`; rive-runtime `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; browser `chrome 150.0.7871.189`; 100 segments/run.

| Fixture | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `text_vertical_trim_test` | 10,467,863 | 0.400 ms (124.7%) | 2.261 ms (16.4%) | 0.177x | 0.167x | 0.745x |
| `data_bind_test_cmdq` | 1,690,399 | 6.200 ms (12.3%) | 5.844 ms (15.7%) | 1.061x | 1.114x | 1.411x |
| `background_measure` | 1,301,468 | 0.200 ms (63.9%) | 0.449 ms (96.2%) | 0.446x | 0.509x | 0.000x |
| `hit_test_test` | 883,710 | 7.800 ms (5.8%) | 1.419 ms (9.8%) | 5.498x | 1.655x | 39.408x |
| `component_list_1` | 806,848 | 2.300 ms (3.8%) | 0.156 ms (13.9%) | 14.775x | 7.557x | 68.861x |

Each side uses fresh total and phase-pass instances, primes retained topology before timing, and reports multiple independent runs. Browser timings use monotonic `performance.now()` around the same production advance and draw calls as the native facade. Scripted, input-driven, and image-decoding fixtures are unsupported and fail closed; very small phase values may show browser-clock quantization.

Reproduce with `make wasm-perf`. The command writes the complete `nuxie-wasm-perf-v1` machine report, including all per-run timings, identity, accounted time, bookkeeping, and segment counts, to `target/wasm-perf.json` and the generated table to `target/wasm-perf.md`.
