# Parity scorecard

`make parity-scorecard` validates recorded output from the three parity-floor
ratchets and prints the five tiers from `docs/parity-closeout-map.md`. It also
writes `target/parity-scorecard/scorecard.json` using schema
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

`record` returns the gate's exit status. `check` fails if required evidence is
missing, stale, malformed, nonzero, or disagrees with the current corpus
manifests. A successful summary from a failed command is therefore still red.
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
