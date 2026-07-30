# Silver corpus

`silver-corpus` is the read-only adoption floor for upstream's SRIV v1
serialized-render corpus. It never rewrites a baseline.

The checked-in `silver-corpus.toml` is generated from literal upstream
`silver.matches(...)`/`serializer()->matches(...)` calls. Six dynamically named
layout-scroll cases are explicit generator data. `interpolator.sriv` and
`multitouch_debug.sriv` remain explicit `provenance-unknown` entries.

This adoption step validates all 238 streams structurally and catalogs the
runtime producers. Runtime cases remain `pending` until their C++ test bodies
are translated to the shared action DSL and replayed by Rust; the 41 scripted
cases are separately `pending-scripted`. Accordingly, `actions =
"cpp-test-body"` is a deliberate pending marker, not a claim that the C++ body
is already executable by the corpus runner. The exact-ID ledger and
`min_cpp_rust_exact` ratchet prevent future exact cases from being silently
downgraded.

Run:

```sh
make silver-corpus
cargo run -p silver-corpus -- compare expected.sriv actual.sriv
```

An optional `--rust-output-dir` probes `<id>.sriv` files operation by operation.
Pending differences are reported as findings; exact-entry differences fail.
