# Wasm advance+draw performance evidence

Status: **report-only**. This evidence establishes a baseline; it does not enforce a budget.

Git `26bbd16213081130ba614f9c5c303d645df9f493`; rive-runtime `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; browser `chrome 150.0.7871.189`; 100 segments/run; 1 discarded browser warmup(s).

| Fixture | Workload identity | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `text_vertical_trim_test` | state machine 0; VM schema-default | 10,467,863 | 0.400 ms (17.7%) | 2.383 ms (18.8%) | 0.168x | 0.063x | 0.319x |
| `data_bind_test_cmdq` | state machine 0; VM schema-default | 1,690,399 | 0.800 ms (12.4%) | 4.731 ms (4.4%) | 0.169x | 0.108x | 0.373x |
| `background_measure` | state machine 0; VM none | 1,301,468 | 0.100 ms (70.7%) | 0.342 ms (5.1%) | 0.292x | 0.000x | 0.000x |
| `hit_test_test` | state machine 0; VM schema-default | 883,710 | 1.600 ms (5.4%) | 1.520 ms (9.8%) | 1.053x | 1.149x | 1.542x |
| `component_list_1` | state machine 0; VM schema-default | 806,848 | 0.100 ms (37.3%) | 0.167 ms (10.2%) | 0.599x | 1.294x | 0.000x |

Each side uses fresh total and phase-pass instances, primes retained topology before timing, and reports multiple independent runs. Browser timings use monotonic `performance.now()` around the same production advance and draw calls as the native facade. Scripted, input-driven, and image-decoding fixtures are unsupported and fail closed; very small phase values may show browser-clock quantization.

Provenance seal: source tree `088a784011895dfaee07b5084d204d5a409f7061`; rive-runtime tree `a475a5651bd10789ae0333e1010a7a920768e48f`; independent run-seal SHA-256 `73fce109f99e7419fecda990bcb94b6c8cda0ef80f0d8e5e2a76d36fa22abfb9`; native runner SHA-256 `ee0470c34f932f1efa4363cc3a92a2e743420bd92873453c5f53bec2042fc486`; Wasm SHA-256 `c2273b24d6e30c8c51ff5ada39e9dc93c25713514efdb335abf3241b53aed785`; wasm-bindgen JavaScript SHA-256 `6256534cadc2fbd30adc6570da96d38aeecc442b13d02d8a2c8af25bf795a829`. Before measurement, the coordinator captured the canonical run-seal digest outside the mutable config and required it during browser execution and finalization. The runner verified clean source checkouts before building; derived retained source and artifact identity from anchored provenance; sealed the repeat, run, warmup, and fixture measurement contract; sealed the original and staged bytes of all five fixtures against their configured SHA-256 identities; verified the browser-loaded bytes, browser-reported measurement contract, and native runner's read bytes against the same seal; and reverified the seal, source, artifacts, and fixture copies after measurement.

Reproduce with `make wasm-perf` from clean source and rive-runtime checkouts. The command writes the complete `nuxie-wasm-perf-v1` machine report, including all per-run timings, identity, source trees, artifact and fixture hashes, accounted time, bookkeeping, and segment counts, to `target/wasm-perf.json` and the generated table to `target/wasm-perf.md`.
