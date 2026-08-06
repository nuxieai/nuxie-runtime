# Wasm advance+draw performance evidence

Status: **report-only**. This evidence establishes a baseline; it does not enforce a budget.

Git `0fb494e9e931696e16763fb630dfa118183300a2`; rive-runtime `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; browser `chrome 150.0.7871.189`; 100 segments/run; 1 discarded browser warmup(s).

| Fixture | Workload identity | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `text_vertical_trim_test` | state machine 0; VM schema-default | 10,467,863 | 0.400 ms (25.9%) | 2.362 ms (23.9%) | 0.169x | 0.191x | 0.595x |
| `data_bind_test_cmdq` | state machine 0; VM schema-default | 1,690,399 | 0.800 ms (15.6%) | 3.834 ms (9.7%) | 0.209x | 0.083x | 0.599x |
| `background_measure` | state machine 0; VM none | 1,301,468 | 0.100 ms (39.1%) | 0.343 ms (39.0%) | 0.291x | 0.000x | 0.000x |
| `hit_test_test` | state machine 0; VM schema-default | 883,710 | 1.600 ms (8.8%) | 1.236 ms (15.1%) | 1.295x | 1.310x | 2.197x |
| `component_list_1` | state machine 0; VM schema-default | 806,848 | 0.200 ms (35.4%) | 0.140 ms (8.8%) | 1.425x | 1.589x | 3.174x |

Each side uses fresh total and phase-pass instances, primes retained topology before timing, and reports multiple independent runs. Browser timings use monotonic `performance.now()` around the same production advance and draw calls as the native facade. Scripted, input-driven, and image-decoding fixtures are unsupported and fail closed; very small phase values may show browser-clock quantization.

Provenance seal: source tree `f961baeabe4a616a490c2bd46fe2744c4b856a91`; rive-runtime tree `a475a5651bd10789ae0333e1010a7a920768e48f`; native runner SHA-256 `ee0470c34f932f1efa4363cc3a92a2e743420bd92873453c5f53bec2042fc486`; Wasm SHA-256 `c2273b24d6e30c8c51ff5ada39e9dc93c25713514efdb335abf3241b53aed785`; wasm-bindgen JavaScript SHA-256 `6256534cadc2fbd30adc6570da96d38aeecc442b13d02d8a2c8af25bf795a829`. The runner verified clean source checkouts before building; sealed the original and staged bytes of all five fixtures against their configured SHA-256 identities; verified the browser-loaded bytes and the native runner's read bytes against the same identities; and reverified source, artifacts, and fixture copies before and after measurement.

Reproduce with `make wasm-perf` from clean source and rive-runtime checkouts. The command writes the complete `nuxie-wasm-perf-v1` machine report, including all per-run timings, identity, source trees, artifact and fixture hashes, accounted time, bookkeeping, and segment counts, to `target/wasm-perf.json` and the generated table to `target/wasm-perf.md`.
