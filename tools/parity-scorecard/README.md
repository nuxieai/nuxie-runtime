# Parity scorecard

`make parity-scorecard` prints one deterministic view of the checked-in parity
ledgers and regenerates `docs/parity-scorecard.md`. The snapshot contains no
generation timestamp, so it changes only when a source ledger changes.

The snapshot aggregates these existing sources without adding verdicts:

- `file-correspondence-manifest.toml` and its Rust-module inversion
- `rust-additions.toml`
- `test-correspondence-manifest.toml`
- `silver-corpus.toml` and its minimum-exact ratchet
- `corpus.toml`
- `docs/runtime-frame-loop-ownership.toml`
- `docs/runtime-frame-loop-gaps.toml`
- the D section of `docs/parity-gap-register.md`

The older recorded-gate scorecard remains available explicitly through
`parity_scorecard.py check`. It validates recorded output from the parity-floor
ratchets and can write `target/parity-scorecard/scorecard.json` using schema
`nuxie-parity-scorecard-v1`.

The required evidence files are:

- `target/parity-scorecard/evidence/golden-compare.json`
- `target/parity-scorecard/evidence/scripted-golden-compare.json`
- `target/parity-scorecard/evidence/renderer-golden.json`
- `target/parity-scorecard/evidence/cargo-test-workspace.json`
- `target/parity-scorecard/evidence/capi-smoke.json`

Produce evidence while running a gate so the report contains both its output
and real exit status:

```sh
python3 tools/parity-scorecard/parity_scorecard.py record \
  --gate golden-compare \
  --output target/parity-scorecard/evidence/golden-compare.json \
  -- make golden-compare
```

Each evidence file records its command, and both `record` and `check` require
the gate's canonical command: `make golden-compare`, `make
scripted-golden-compare`, `make renderer-golden`, `cargo test --workspace`, or
`make capi-smoke`. The separately required CI check `make
renderer-golden-same-runner` is deliberately not scorecard evidence; the
renderer scorecard floor is the checked-in-reference `make renderer-golden`
gate.

`record` returns the gate's exit status. `check` fails if required evidence is
missing, stale, malformed, nonzero, bound to a different command, or disagrees
with the current corpus manifests. A successful summary from a failed or
mismatched command is therefore still red.
The manifest-derived totals must also remain at or above the committed
317-entry/647-segment runtime and 1,468-entry renderer floors in
`parity-scorecard.toml`, so deleting coverage cannot lower both sides of a
comparison and masquerade as green.

Known future gates are different from unavailable evidence: they are printed
as `not built` with their ticket and keep their tier non-green, but do not make
the scorecard plumbing fail. The optional six-file perf artifact is reported
only as non-blocking evidence until #OR-9 lands. `parity-scorecard.toml` holds
the current A-row and verified-adapter checklist facts; tickets update those
lists only when their mechanical exit gate closes.
