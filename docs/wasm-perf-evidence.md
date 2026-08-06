# Wasm advance+draw performance evidence

Status: **report-only**. This evidence establishes a baseline; it does not enforce a budget.

Git `26d0ce591ad847bebec07bd258b6c520abc779f9`; rive-runtime `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; browser `chrome 150.0.7871.189`; 100 segments/run; 1 discarded browser warmup(s).

| Fixture | Workload identity | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `text_vertical_trim_test` | state machine 0; VM schema-default | 10,467,863 | 0.400 ms (30.6%) | 2.048 ms (22.6%) | 0.195x | 0.153x | 0.456x |
| `data_bind_test_cmdq` | state machine 0; VM schema-default | 1,690,399 | 0.800 ms (17.6%) | 4.341 ms (16.2%) | 0.184x | 0.089x | 0.506x |
| `background_measure` | state machine 0; VM none | 1,301,468 | 0.200 ms (34.2%) | 0.363 ms (9.3%) | 0.552x | 0.000x | 4.696x |
| `hit_test_test` | state machine 0; VM schema-default | 883,710 | 1.500 ms (7.4%) | 1.463 ms (52.1%) | 1.025x | 1.500x | 1.096x |
| `component_list_1` | state machine 0; VM schema-default | 806,848 | 0.100 ms (37.3%) | 0.178 ms (28.7%) | 0.562x | 1.334x | 3.443x |

Each side uses fresh total and phase-pass instances, primes retained topology before timing, and reports multiple independent runs. Browser timings use monotonic `performance.now()` around the same production advance and draw calls as the native facade. Scripted, input-driven, and image-decoding fixtures are unsupported and fail closed; very small phase values may show browser-clock quantization.

Provenance seal: source tree `be12d53c5be29b689e8732c63fcc2f54588c096c`; rive-runtime tree `a475a5651bd10789ae0333e1010a7a920768e48f`; independent run-seal SHA-256 `0f1f5b76f0c4b21078e29edb97bcd2204b59a770f3ca6ca6e000e25688589356`; native runner SHA-256 `ee0470c34f932f1efa4363cc3a92a2e743420bd92873453c5f53bec2042fc486`; Wasm SHA-256 `eb2c477fe8df3c10e0a2aa3948c3ff3ad842f74289f47ac058c5711e910ebdc1`; wasm-bindgen JavaScript SHA-256 `6256534cadc2fbd30adc6570da96d38aeecc442b13d02d8a2c8af25bf795a829`; harness driver SHA-256 `2b2af90439e113e248a0c74a96d9fd6623581c27695ba9fe34faa8e2a5e136f4`; harness HTML SHA-256 `76c1816ad4873ce0c004b41fae9e644346e6faf5746dcae5b191cce24c31aa43`. Before measurement, the coordinator captured the canonical run-seal digest outside the mutable config and required it during browser execution and finalization. The browser read and verified the exact HTML and driver bytes from the seal, served and executed them only from SHA-addressed intercepted URLs, fetched the exact sealed wasm-bindgen JavaScript and Wasm buffers, and reported all four executed artifact identities for verification. Native measurement copied the verified runner bytes to a private content-addressed executable and invoked only that sealed copy. The Wasm runner enabled deterministic runtime mode before importing each fixture, matching the native runner's deterministic setup. The runner also verified clean source checkouts, sealed the repeat, run, warmup, and fixture measurement contract, verified the browser-loaded fixture bytes and browser-reported contract, and reverified the independent seal, sources, artifacts, and fixture copies after measurement.

Reproduce with `make wasm-perf` from clean source and rive-runtime checkouts. The command writes the complete `nuxie-wasm-perf-v1` machine report, including all per-run timings, identity, source trees, artifact and fixture hashes, accounted time, bookkeeping, and segment counts, to `target/wasm-perf.json` and the generated table to `target/wasm-perf.md`.
