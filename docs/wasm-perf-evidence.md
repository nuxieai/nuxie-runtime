# Wasm advance+draw performance evidence

Status: **report-only**. This evidence establishes a baseline; it does not enforce a budget.

Git `9ea9e53941d2127e2a229faa91d4927aa45ae647`; rive-runtime `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; browser `chrome 150.0.7871.189`; 100 segments/run; 1 discarded browser warmup(s).

| Fixture | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `text_vertical_trim_test` | 10,467,863 | 0.400 ms (17.7%) | 1.629 ms (16.4%) | 0.246x | 0.096x | 0.644x |
| `data_bind_test_cmdq` | 1,690,399 | 4.800 ms (3.3%) | 3.719 ms (5.6%) | 1.291x | 1.274x | 1.672x |
| `background_measure` | 1,301,468 | 0.100 ms (70.7%) | 0.322 ms (13.8%) | 0.311x | 0.000x | 3.524x |
| `hit_test_test` | 883,710 | 6.600 ms (2.1%) | 1.161 ms (15.8%) | 5.685x | 1.681x | 36.042x |
| `component_list_1` | 806,848 | 2.000 ms (2.2%) | 0.142 ms (6.9%) | 14.085x | 7.869x | 62.846x |

Each side uses fresh total and phase-pass instances, primes retained topology before timing, and reports multiple independent runs. Browser timings use monotonic `performance.now()` around the same production advance and draw calls as the native facade. Scripted, input-driven, and image-decoding fixtures are unsupported and fail closed; very small phase values may show browser-clock quantization.

Reproduce with `make wasm-perf`. The command writes the complete `nuxie-wasm-perf-v1` machine report, including all per-run timings, identity, accounted time, bookkeeping, and segment counts, to `target/wasm-perf.json` and the generated table to `target/wasm-perf.md`.
