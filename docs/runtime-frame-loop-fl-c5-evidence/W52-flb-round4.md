Verdict: three blocking findings remain.

## Spec

1. **BLOCKING — Published structural/provenance checker fails.**

   On clean frozen `eaf8a6f6`, the trace records 7,290 files and fingerprint `f0b77c…` ([trace](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-trace.json:7790)). The current tree computes 7,285 files and `a45b4f…`. Running the live checker rejects both the candidate fingerprint and runner provenance through [check.py](/Users/levi/dev/worktrees/nuxie-e2-review/tools/runtime-frame-loop-port/check.py:993).

   This directly contradicts the green-checker claim in [W48](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-evidence/W48-round4-corrective-report.md:72). The E2 receipt relocations changed files included by [source_fingerprint.py](/Users/levi/dev/worktrees/nuxie-e2-review/tools/runtime-frame-loop-port/source_fingerprint.py:68) after the fingerprint was generated.

2. **BLOCKING — Retained-arena resolution is not implemented everywhere.**

   The direct animation facades are corrected: apply, advance, event advance, and keep-going use the instance-retained arena ([animation.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/animation.rs:2136), [animation.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/animation.rs:2260), [artboard.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/artboard.rs:4457)). The new wrong-artboard differential exercises those direct facades ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/tests/cpp_probe.rs:19788)).

   But `linear_animation_instance_definition` still resolves the numeric handle through the caller artboard ([artboard.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/artboard.rs:4504)). State-machine transition duration, exit-time, and refresh paths use that resolver ([state_machine_layer_instance.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:579), [state_machine_layer_instance.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:669), [state_machine_layer_instance.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:738), [state_machine_layer_instance.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:848)). Reset construction also resolves indices against the caller artboard ([animation_reset_factory.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/state_machine/animation_reset_factory.rs:101)).

   This is publicly reachable by creating a `StateMachineInstance` on A and advancing it through B ([artboard.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/artboard.rs:5394)). Pinned C++ instead reads retained `m_animation` during advance and apply ([linear_animation_instance.cpp](/Users/levi/dev/oss/rive-runtime/src/animation/linear_animation_instance.cpp:187), [linear_animation_instance.hpp](/Users/levi/dev/oss/rive-runtime/include/rive/animation/linear_animation_instance.hpp:78)) and for loop queries ([linear_animation_instance.cpp](/Users/levi/dev/oss/rive-runtime/src/animation/linear_animation_instance.cpp:408)).

B-a’s narrow implementation otherwise passes inspection: full existing `rust_ref`, publication-only ancestor validation, exact eight-key schema/manifest equality, and injected ref/hash negatives exist ([check.py](/Users/levi/dev/worktrees/nuxie-e2-review/tools/runtime-frame-loop-port/check.py:391), [test_check.py](/Users/levi/dev/worktrees/nuxie-e2-review/tools/runtime-frame-loop-port/test_check.py:522)). Direct isolated positive and mutation calls produced the expected accept/reject results.

## Standards

3. **BLOCKING — Receipt-stamp negative is relocation-vacuous.**

   The test claiming to inspect tracked receipts uses non-recursive `glob("floor*.log")` and hard-codes 13 root files ([test_stamp_floor_receipt.py](/Users/levi/dev/worktrees/nuxie-e2-review/tools/runtime-frame-loop-port/test_stamp_floor_receipt.py:17)). E2 moved five `floor2-*` logs under `superseded/`; the tree now contains 18 tracked floor logs, so those five are unguarded. All 18 currently have valid stamps, but the permanent negative no longer enforces the universal rule in [README.md](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-evidence/README.md:3).

4. **NON-BLOCKING — Conflicting checker count.**

   The final checklist still says 59 tests ([closure](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-closure.md:919)), while the final gate table and W48 say 66/66. This violates the exact-count publication requirement ([implementation spec](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-impl-spec.md:553)).

## Regression and scope checks

The seven round-one/two corrections remain structurally present: exact 45-file scope, signed loop semantics, invalid-interpolator erasure, importer-cursor survival, negative-speed remapping, NaN direct-blend handling, and empty-baseline reset. Their differentials remain at [cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/tests/cpp_probe.rs:19276) through the NaN/reset cases at [cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/tests/cpp_probe.rs:24013). No regression was found in those seven fixes.

The 66-test suite was attempted, but this read-only sandbox has no writable temporary directory: 62 tests failed during setup and four executed. That result is environmentally inconclusive, not counted as a candidate failure. The live checker itself required no temporary writes and produced the blocking failure above.

The ledger has exactly one 45-entry FL-B frozen scope, includes `keyed_property_importer.cpp`, excludes `scripted_listener_action.cpp`, and matches the checker’s 45-entry constant ([ownership ledger](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-ownership.toml:184), [check.py](/Users/levi/dev/worktrees/nuxie-e2-review/tools/runtime-frame-loop-port/check.py:55)). Exactly eight FL-B member rows remain present ([ownership ledger](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-ownership.toml:3289)).

Delta sweep: `edddf491..95333c41` contains 43 changed paths; the complete publication through `eaf8a6f6` contains 48. No unexpected FL-B scope addition was found, but the E2-only provenance and receipt-test regressions affect the frozen publication under review.

Axis summary: Spec has two blocking findings; Standards has one blocking and one non-blocking finding.

REJECT for FL-B reacceptance.