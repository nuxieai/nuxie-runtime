Verdict: publication commit `eaf8a6f6` is not structurally acceptable.

## BLOCKING

1. Substantive state-machine orchestration was reintroduced into unauthorized owners.

The spec permits `artboard.rs` changes only as thin delegation into the instance owner ([impl-spec](</Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-impl-spec.md:37>)). Commit `2e2d3c6d` instead added event collection, nested animation selection, dispatch, and audio unwinding directly in `artboard.rs` ([artboard.rs](</Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/artboard.rs:10155>)), plus notify/advance/report orchestration in the unauthorized `nested_state_machine.rs` owner ([nested_state_machine.rs](</Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/state_machine/nested_state_machine.rs:346>)).

The intended forbidden-shape ratchet ([gaps.toml](</Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-gaps.toml:1377>)) misses the new naming/indirection: direct evaluation produced zero matches while all four orchestration markers were present. This reopens W40’s first blocker ([W40](</Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-evidence/W40-standards-rereview.md:7>)). Therefore not every previously verified item held.

2. The semantic forbidden-projection ratchet remains extraction-vacuous.

The ratchet advertises recursive `state_machine/**/*.rs` coverage ([gaps.toml](</Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-gaps.toml:1567>)), but its semantic scanner immediately returns unless `SemanticNodeResolver` is declared in the same file ([check.py](</Users/levi/dev/worktrees/nuxie-e2-review/tools/runtime-frame-loop-port/check.py:304>)). The checker invokes that scanner separately for each file ([check.py](</Users/levi/dev/worktrees/nuxie-e2-review/tools/runtime-frame-loop-port/check.py:1380>)).

The negative puts both the trait and renamed fallback in one synthetic file ([test_check.py](</Users/levi/dev/worktrees/nuxie-e2-review/tools/runtime-frame-loop-port/test_check.py:4218>)). An adversarial read-only probe returned a hit for that same-file form but `[]` for the identical renamed ordinal helper in a sibling file. A fallback extracted to `semantic_listener_group.rs` therefore escapes while the repository-wide resolver seam still exists.

3. The live structural/provenance checker is red in the frozen publication checkout.

W48 claims it is green ([W48](</Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-evidence/W48-round4-corrective-report.md:72>)). Running the checker command wired by the Makefile ([Makefile](</Users/levi/dev/worktrees/nuxie-e2-review/Makefile:157>)) against clean `eaf8a6f6` exited 1:

- recorded fingerprint: 7,290 files, `f0b77c24…`
- actual frozen checkout: 7,285 files, `a45b4f98…`
- runner provenance: stale

The recorded values are in the trace ([trace.json](</Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-trace.json:7790>)), and equality is explicitly enforced ([check.py](</Users/levi/dev/worktrees/nuxie-e2-review/tools/runtime-frame-loop-port/check.py:993>)). Thus the committed packet cannot reproduce its claimed green provenance state.

4. The “13 tracked receipts” assertion checks the wrong set.

The test uses a non-recursive filesystem `glob("floor*.log")` and merely asserts that 13 root files exist ([test_stamp_floor_receipt.py](</Users/levi/dev/worktrees/nuxie-e2-review/tools/runtime-frame-loop-port/test_stamp_floor_receipt.py:17>)). It does not query Git and excludes the five receipts moved under `superseded/`, despite README making those retained historical evidence ([README](</Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-evidence/README.md:18>)). There are 18 tracked floor logs total. Corrupting or removing the successful Apple attempt-2 receipt would not fail this assertion.

## NON-BLOCKING

None.

## Verified closures

- The four field assertions move directly from an owned `RuntimeStateMachine` into exact return types, so Deref-wrapper substitution cannot coerce ([public_api_fl_c5.rs](</Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/tests/public_api_fl_c5.rs:37>)).
- All five hydration calls consume a borrowed, non-Clone, non-Send/non-Sync token, proving `FnOnce` and rejecting `Fn`/`FnMut` narrowing ([public_api_fl_c5.rs](</Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/tests/public_api_fl_c5.rs:875>)). The proof is inside the pinned digest ([gaps.toml](</Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-gaps.toml:958>)).
- The nested-remap type, query, re-export, and `cpp_probe` target are tools-gated ([Cargo.toml](</Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/Cargo.toml:8>), [lib.rs](</Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/lib.rs:48>)); the Makefile enables `--features tools` ([Makefile](</Users/levi/dev/worktrees/nuxie-e2-review/Makefile:431>)).
- All five operative floor3 receipts currently stamp full `95333c41`; all 18 tracked logs manually inspected have resolvable SHA stamps. Apple failure/rerun disclosure and W41 verdict prose are corrected, and all five independent verdict files are tracked.
- Full `rust_ref` and exact eight-artifact-hash validation exists, but the end-to-end checker failure above prevents acceptance.
- `git diff --check 05670a39..eaf8a6f6` is clean.

REJECT