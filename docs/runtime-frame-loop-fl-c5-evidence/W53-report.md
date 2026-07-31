Implemented the joint round-5 corrective with the binding per-source chain-atomicity design.

### Findings resolved

- **A — Chain atomicity:** nested sources now complete local dispatch, ancestor propagation, and audio unwind before advancing the next source; earlier audio is flushed before later script errors. [policy](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12571), [production tests](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/artboard.rs:11624), [live differential](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/cpp_probe.rs:83958)
- **B — Retained arena:** transition duration, exit-time, refresh, and reset paths resolve retained animation definitions. [definition lookup](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/artboard.rs:4504), [layer consumers](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:615), [wrong-artboard differential](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/cpp_probe.rs:19914)
- **C — Semantic scanner:** resolver detection is repository-wide, followed by scanning every state-machine source; sibling-file negative added. [checker](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/check.py:1349), [negative](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/test_check.py:4331)
- **D — Stamp guard:** receipts are recursively enumerated through Git, including superseded receipts, and every receipt is validated. [guard](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/stamp_floor_receipt.py:19), [corruption coverage](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/test_stamp_floor_receipt.py:17)
- **E — Reproducibility/prose:** tools-feature commands, live checker count wording, and the P4→E3→review pointer are corrected. [implementation spec](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-fl-c5-impl-spec.md:580), [closure](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-fl-c5-closure.md:921), [status](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-status.md:832)

### Acceptance

- Runtime library: **718/718**
- Tools-enabled C++ differential: **818/818**
- `nuxie` library: **146/146**
- Checker tests: **67/67**
- Ordinary golden: **317/317 entries, 647/647 segments, zero divergence**
- Scripted golden: **317/317 entries, 647/647 segments, zero divergence**
- `cargo fmt --check` and working-tree `git diff --check`: green
- Spec and standards reviews: no unresolved findings
- Live port checker: only the two expected E3-owned failures remain:
  - stale candidate-source fingerprint
  - missing/stale Rust-runner provenance

No evidence files were touched, no tests were weakened, no rm-style commands were used, and no commit was created.

### Staging status

Staging was attempted, but the actual worktree index is outside the writable sandbox, so Git could not create `index.lock`. All 21 files therefore remain modified and unstaged.

- Production: 11 Rust source files under `crates/nuxie-runtime/src/`
- Tests/tools/ratchets: `cpp_probe.rs`, C++ probe, gaps ledger, checker/stamp tools and tests
- Spec prose: `runtime-frame-loop-fl-c5-closure.md`, `runtime-frame-loop-fl-c5-impl-spec.md`, `runtime-frame-loop-status.md`

Goal usage: **1,514,321 tokens over about 1h 34m**.