Implemented `make parity-scorecard` and left all changes uncommitted.

Key outputs:

- Deterministic aggregator: [ledger_scorecard.py](/Users/levi/dev/worktrees/nuxie-p1c-importers/tools/parity-scorecard/ledger_scorecard.py)
- Generated snapshot: [docs/parity-scorecard.md](/Users/levi/dev/worktrees/nuxie-p1c-importers/docs/parity-scorecard.md)
- Implementation report: [SCORE-report.md](/Users/levi/dev/worktrees/nuxie-p1c-importers/SCORE-report.md)
- Makefile and CI wiring updated while preserving the legacy evidence checker.

Gates all pass:

- `make parity-scorecard` — 26 tests
- Python unittest suite — 26 tests
- `make runtime-frame-loop-port-check`
- `make rust-attribution-check`
- Deterministic snapshot SHA-256: `83d244491e3fe3059ed328d36eb74fd0848a9200e3b7507017908fa206fb9b1f`

Final standards and specification reviews found no remaining defects. No commit was created.