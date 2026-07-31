# Silver corpus

`silver-corpus` is the read-only adoption floor for upstream's SRIV v1
serialized-render corpus. It never rewrites a baseline.

The checked-in `silver-corpus.toml` is generated from literal upstream
`silver.matches(...)`/`serializer()->matches(...)` calls. Six dynamically named
layout-scroll cases are explicit generator data. `interpolator.sriv` and
`multitouch_debug.sriv` remain explicit `provenance-unknown` entries.

The runner validates all 238 streams structurally. For the 195 runtime cases,
the generator translates the replayable portion of each pinned C++ test body
into a shared action stream. Validation imports the recorded `.riv`, selects
the requested artboard/state machine/animation, applies those actions to the
Rust runtime, serializes the Rust render operations as SRIV v1, and compares
them operation by operation with the upstream baseline.

Every runtime row must be classified. Replayable differences are retained as
`diverges` findings with their first divergent operation in the note. Bodies
that cannot yet be represented or require an unported subsystem are
`unsupported-feature` with the blocker named. The 41 scripted cases remain
separately `pending-scripted`, and two baselines remain
`provenance-unknown`. The exact-ID ledger and `min_cpp_rust_exact` ratchet
prevent exact cases from being silently downgraded.

Run:

```sh
make silver-corpus
cargo run -p silver-corpus -- compare expected.sriv actual.sriv
```

Use `--id <id>` to replay one manifest entry. An optional
`--rust-output-dir` also probes pre-generated `<id>.sriv` files operation by
operation. Classified divergences keep the suite green but are printed
prominently; an exact entry that changes fails.
