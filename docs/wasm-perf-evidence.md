# Wasm advance+draw performance evidence

Status: **report-only**. This evidence establishes a baseline; it does not enforce a budget.

Git `4af16c1d8320dbe735b8ec96b9c7ff482fbd9c14`; rive-runtime `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; browser `chrome 150.0.7871.189`; 100 segments/run; 1 discarded browser warmup(s).

| Fixture | Workload identity | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `text_vertical_trim_test` | state machine 0; VM schema-default | 10,467,863 | 0.400 ms (19.9%) | 1.606 ms (17.6%) | 0.249x | 0.090x | 0.384x |
| `data_bind_test_cmdq` | state machine 0; VM schema-default | 1,690,399 | 0.800 ms (13.6%) | 3.474 ms (10.4%) | 0.230x | 0.090x | 0.649x |
| `background_measure` | state machine 0; VM none | 1,301,468 | 0.200 ms (63.9%) | 0.327 ms (2.6%) | 0.611x | 0.758x | 10.763x |
| `hit_test_test` | state machine 0; VM schema-default | 883,710 | 1.600 ms (7.3%) | 1.245 ms (11.9%) | 1.286x | 1.485x | 1.094x |
| `component_list_1` | state machine 0; VM schema-default | 806,848 | 0.100 ms (69.7%) | 0.142 ms (16.1%) | 0.704x | 0.000x | 3.773x |

Each side uses fresh total and phase-pass instances, primes retained topology before timing, and reports multiple independent runs. Browser timings use monotonic `performance.now()` around the same production advance and draw calls as the native facade. Scripted, input-driven, and image-decoding fixtures are unsupported and fail closed; very small phase values may show browser-clock quantization.

Provenance seal: source tree `3679e352be233c1b17e66257fc57223fcf40403c`; rive-runtime tree `a475a5651bd10789ae0333e1010a7a920768e48f`; native runner SHA-256 `26e35811e9ca86ac8d487ac151c865535356cbe85391e35b94704462784b3ac5`; Wasm SHA-256 `5bba56f92c042c67fdff677fb7bc762957a56303c3cde6c12dae2fe04da1cf74`; wasm-bindgen JavaScript SHA-256 `6256534cadc2fbd30adc6570da96d38aeecc442b13d02d8a2c8af25bf795a829`. The runner verified clean source checkouts before building, sealed these artifacts before browser measurement, and reverified the source and artifacts before and after native measurement.

Reproduce with `make wasm-perf` from clean source and rive-runtime checkouts. The command writes the complete `nuxie-wasm-perf-v1` machine report, including all per-run timings, identity, source trees, artifact hashes, accounted time, bookkeeping, and segment counts, to `target/wasm-perf.json` and the generated table to `target/wasm-perf.md`.
