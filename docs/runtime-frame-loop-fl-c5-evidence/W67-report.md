Implemented all six WOULD-REJECT corrections and the NIT.

Key outcomes:

- Nested reports now bubble during `applyEvents`, before authored layers/mixes, while the reporting source remains addressable.
- Post-notify settlement is limited to `updateDataBinds(false)`.
- Deep C++/Rust differential proves matching report-time pre-mix source values.
- Actual Rust notify entry records singleton batches; both runtimes assert `[1, 1]`.
- Ownership ratchets scan every non-owner Rust source and resist function-item, const/static, enum-alias, labeled-loop, and literal-free audio evasions.
- Status and trace correctly describe pre-E5 round-seven work with operative publication `76ab8d86`; W63 links use tracked verdicts.
- Independent Standards and Spec re-reviews found no remaining issues.

Verification:

- Runtime library: 723 passed
- C++ probe suite: 823 passed
- Nuxie library: 146 passed
- Checker suite: 67 passed
- Ordinary golden: 317/317 entries, 647 segments, zero divergence
- Scripted golden: 317/317 entries, 647 segments, zero divergence
- C++ probe build and tools check: passed
- Live checker: only the three intentional pre-E5 receipt failures—stale Rust ref, fingerprint, and runner provenance
- `cargo fmt` and `git diff --check`: clean

No commit was created; HEAD remains `f4f013dd364389aaed18f27c7a2eec7183050821`. Goal elapsed time: approximately 1h 21m.