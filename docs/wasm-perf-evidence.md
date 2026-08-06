# Wasm advance+draw performance evidence

Status: **report-only**. This evidence establishes a baseline; it does not enforce a budget.

Git `994cc96e4374a38462a839ea244ff07e2b9ec8ea`; rive-runtime `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; browser `chrome 150.0.7871.189`; 100 segments/run; 1 discarded browser warmup(s).

| Fixture | Workload identity | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `text_vertical_trim_test` | state machine 0; VM schema-default | 10,467,863 | 0.500 ms (28.3%) | 2.669 ms (21.9%) | 0.187x | 0.067x | 0.501x |
| `data_bind_test_cmdq` | state machine 0; VM schema-default | 1,690,399 | 0.900 ms (15.8%) | 4.938 ms (14.0%) | 0.182x | 0.099x | 0.489x |
| `background_measure` | state machine 0; VM none | 1,301,468 | 0.100 ms (39.1%) | 0.375 ms (11.9%) | 0.266x | 0.416x | 3.448x |
| `hit_test_test` | state machine 0; VM schema-default | 883,710 | 1.900 ms (2.8%) | 1.535 ms (4.1%) | 1.237x | 1.349x | 1.385x |
| `component_list_1` | state machine 0; VM schema-default | 806,848 | 0.200 ms (46.5%) | 0.175 ms (3.3%) | 1.142x | 3.793x | 0.000x |

Each side uses fresh total and phase-pass instances, primes retained topology before timing, and reports multiple independent runs. Browser timings use monotonic `performance.now()` around the same production advance and draw calls as the native facade. Scripted, input-driven, and image-decoding fixtures are unsupported and fail closed; very small phase values may show browser-clock quantization.

Provenance seal: source tree `6858ab99a55bde6fe486c19011d98ee94d2bf629`; rive-runtime tree `a475a5651bd10789ae0333e1010a7a920768e48f`; independent run-seal SHA-256 `f898d512b2cd97f2cd242b7e8461816707b364f26939d31ae0f3d5ec4c732e3d`; native runner SHA-256 `ee0470c34f932f1efa4363cc3a92a2e743420bd92873453c5f53bec2042fc486`; Wasm SHA-256 `c2273b24d6e30c8c51ff5ada39e9dc93c25713514efdb335abf3241b53aed785`; wasm-bindgen JavaScript SHA-256 `6256534cadc2fbd30adc6570da96d38aeecc442b13d02d8a2c8af25bf795a829`. Before measurement, the coordinator captured the canonical run-seal digest outside the mutable config and required it during browser execution and finalization. The browser fetched each JavaScript and Wasm artifact once, rejected identities that differed from the seal, then imported and initialized those exact fetched buffers; its report carries the executed identities. Native measurement copied the verified runner bytes to a private content-addressed executable and invoked only that sealed copy. The runner also verified clean source checkouts, sealed the repeat, run, warmup, and fixture measurement contract, verified the browser-loaded fixture bytes and browser-reported contract, and reverified the independent seal, sources, artifacts, and fixture copies after measurement.

Reproduce with `make wasm-perf` from clean source and rive-runtime checkouts. The command writes the complete `nuxie-wasm-perf-v1` machine report, including all per-run timings, identity, source trees, artifact and fixture hashes, accounted time, bookkeeping, and segment counts, to `target/wasm-perf.json` and the generated table to `target/wasm-perf.md`.
