# Wasm advance+draw performance evidence

Status: **report-only**. This evidence establishes a baseline; it does not enforce a budget.

Git `6aa51600705e2b766df4e2cad5e029a64eb5a1c7`; rive-runtime `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; browser `chrome 150.0.7871.189`; 100 segments/run; 1 discarded browser warmup(s).

| Fixture | Workload identity | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `text_vertical_trim_test` | state machine 0; VM schema-default | 10,467,863 | 0.400 ms (22.0%) | 2.019 ms (23.8%) | 0.198x | 0.063x | 0.259x |
| `data_bind_test_cmdq` | state machine 0; VM schema-default | 1,690,399 | 0.800 ms (13.4%) | 5.340 ms (10.1%) | 0.150x | 0.073x | 0.488x |
| `background_measure` | state machine 0; VM none | 1,301,468 | 0.100 ms (39.1%) | 0.328 ms (12.4%) | 0.305x | 0.584x | 0.000x |
| `hit_test_test` | state machine 0; VM schema-default | 883,710 | 1.500 ms (2.9%) | 1.336 ms (8.7%) | 1.123x | 1.226x | 0.811x |
| `component_list_1` | state machine 0; VM schema-default | 806,848 | 0.100 ms (39.1%) | 0.144 ms (26.0%) | 0.696x | 2.900x | 0.000x |

Each side uses fresh total and phase-pass instances, primes retained topology before timing, and reports multiple independent runs. Browser timings use monotonic `performance.now()` around the same production advance and draw calls as the native facade. Scripted, input-driven, and image-decoding fixtures are unsupported and fail closed; very small phase values may show browser-clock quantization.

Provenance seal: source tree `0aa41754d6736c556e20db89f1d92d6f7210e4c1`; rive-runtime tree `a475a5651bd10789ae0333e1010a7a920768e48f`; native runner SHA-256 `ee0470c34f932f1efa4363cc3a92a2e743420bd92873453c5f53bec2042fc486`; Wasm SHA-256 `c2273b24d6e30c8c51ff5ada39e9dc93c25713514efdb335abf3241b53aed785`; wasm-bindgen JavaScript SHA-256 `6256534cadc2fbd30adc6570da96d38aeecc442b13d02d8a2c8af25bf795a829`. The runner verified clean source checkouts before building; sealed the repeat, run, warmup, and fixture measurement contract; sealed the original and staged bytes of all five fixtures against their configured SHA-256 identities; verified the browser-loaded bytes, browser-reported measurement contract, and native runner's read bytes against the same seal; and reverified source, artifacts, and fixture copies before and after measurement.

Reproduce with `make wasm-perf` from clean source and rive-runtime checkouts. The command writes the complete `nuxie-wasm-perf-v1` machine report, including all per-run timings, identity, source trees, artifact and fixture hashes, accounted time, bookkeeping, and segment counts, to `target/wasm-perf.json` and the generated table to `target/wasm-perf.md`.
