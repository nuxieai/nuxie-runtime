# Wasm advance+draw performance evidence

Status: **report-only**. This evidence establishes a baseline; it does not enforce a budget.

Git `67368d0d16d833980e7956e923764678774fc22a`; rive-runtime `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; browser `chrome 150.0.7871.189`; 100 segments/run; 1 discarded browser warmup(s).

| Fixture | Workload identity | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `text_vertical_trim_test` | state machine 0; VM schema-default | 10,467,863 | 0.500 ms (20.3%) | 1.956 ms (18.8%) | 0.256x | 0.089x | 0.582x |
| `data_bind_test_cmdq` | state machine 0; VM schema-default | 1,690,399 | 0.800 ms (13.6%) | 3.729 ms (8.9%) | 0.215x | 0.045x | 0.634x |
| `background_measure` | state machine 0; VM none | 1,301,468 | 0.100 ms (70.7%) | 0.386 ms (30.3%) | 0.259x | 0.000x | 3.709x |
| `hit_test_test` | state machine 0; VM schema-default | 883,710 | 1.600 ms (7.7%) | 1.198 ms (3.4%) | 1.335x | 1.352x | 2.082x |
| `component_list_1` | state machine 0; VM schema-default | 806,848 | 0.200 ms (34.2%) | 0.157 ms (15.1%) | 1.277x | 1.582x | 0.000x |

Each side uses fresh total and phase-pass instances, primes retained topology before timing, and reports multiple independent runs. Browser timings use monotonic `performance.now()` around the same production advance and draw calls as the native facade. Scripted, input-driven, and image-decoding fixtures are unsupported and fail closed; very small phase values may show browser-clock quantization.

Provenance seal: source tree `ab4c4f7e1ab62d352e2fccc6d041eb65c6eaf666`; rive-runtime tree `a475a5651bd10789ae0333e1010a7a920768e48f`; native runner SHA-256 `ee0470c34f932f1efa4363cc3a92a2e743420bd92873453c5f53bec2042fc486`; Wasm SHA-256 `c2273b24d6e30c8c51ff5ada39e9dc93c25713514efdb335abf3241b53aed785`; wasm-bindgen JavaScript SHA-256 `6256534cadc2fbd30adc6570da96d38aeecc442b13d02d8a2c8af25bf795a829`. The runner verified clean source checkouts before building; derived retained source and artifact identity from sealed provenance; sealed the repeat, run, warmup, and fixture measurement contract; sealed the original and staged bytes of all five fixtures against their configured SHA-256 identities; verified the browser-loaded bytes, browser-reported measurement contract, and native runner's read bytes against the same seal; and reverified source, artifacts, and fixture copies before and after measurement.

Reproduce with `make wasm-perf` from clean source and rive-runtime checkouts. The command writes the complete `nuxie-wasm-perf-v1` machine report, including all per-run timings, identity, source trees, artifact and fixture hashes, accounted time, bookkeeping, and segment counts, to `target/wasm-perf.json` and the generated table to `target/wasm-perf.md`.
