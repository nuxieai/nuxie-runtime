# Parity evidence freshness

`make parity-evidence-freshness` validates captured runtime-parity proof inputs
and writes a deterministic changed-since-proof report under
`target/parity-evidence-freshness/`. Staleness is an investigation signal, not
a gate failure: malformed, missing, or historically false provenance fails the
gate, while legitimate source drift produces a successful report with explicit
reasons.

The checked-in `parity-evidence-proofs.json` binds each enrolled proof to the
pinned upstream revision and content-addressed inputs:

- current B6 structural rows bind their exact audit row plus reviewed C++ and
  Rust line windows;
- frame-loop behavioral contracts bind lifecycle C++/Rust windows, the trace
  implementation, their exact ownership-ledger member, trace artifact, input
  scripts, and all enrolled fixtures;
- a trace captured before Git history linearization may bind its durable
  main-history Rust commit by the identical Git tree object, so registry
  reproduction never depends on an ephemeral feature-branch ref;
- frozen B6 rows remain historically valid but explicitly
  `legacy-unbound`/stale until recaptured. They are not promoted by inference.

Run the public report seam directly when filtering or automating re-analysis:

```sh
python3 tools/parity-evidence-freshness/freshness.py report \
  --repo-root . \
  --rive-runtime-dir "$RIVE_RUNTIME_DIR" \
  --output target/parity-evidence-freshness/report.json \
  --markdown-output target/parity-evidence-freshness/report.md
```

The JSON groups stale proofs by subsystem, then ranks within each subsystem by
product reach, changed-source churn, and stable proof ID. It separately reports new,
removed, and changed upstream proof owners. An unrelated edit does not
invalidate a proof, and a bound item may move within its file without becoming
stale when its captured content remains intact and uniquely identifiable.

The bootstrap command is intentionally reproducibility-oriented. Normal source
changes do not rewrite the registry or require evidence-only commits:

```sh
make parity-evidence-registry-check
```

Only a reviewed recapture should regenerate the registry, using its recorded
capture commits and the pinned read-only upstream checkout.
