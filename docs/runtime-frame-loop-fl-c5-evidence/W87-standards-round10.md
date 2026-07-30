REJECT — two structural bypasses remain.

## BLOCKING findings

- Exhaustive catch-all selection evades the guarded-variant detector. Valid safe Rust can isolate `StateMachine` without naming it:

  ```rust
  match animation {
      RuntimeNestedAnimationInstance::Simple { .. } => {}
      RuntimeNestedAnimationInstance::Remap { .. } => {}
      selected_state_machine => move_policy(selected_state_machine),
  }
  ```

  Since the enum has exactly those three variants, the catch-all selects only `StateMachine` ([artboard.rs:1138](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/artboard.rs:1138)). Direct replay produced no selection hit. The analyzer considers only the final path segment—`Simple` or `Remap`—and records selection only for `StateMachine`, or for other variants when the function already contains a recognized event mechanic ([main.rs:770](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:770), [main.rs:787](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:787), [main.rs:801](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:801)). This violates the binding requirement to detect nested-animation selection matches in non-owner files ([W63:63](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:63)) and contradicts the publication claim that exotic spellings containing a guarded type trip ([closure:935](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-c5-closure.md:935), [closure:945](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-c5-closure.md:945)).

- The registry still permits one-for-one substitution inside the same enclosing item. I replayed two sources whose guarded occurrence moved from the approved logic to forbidden logic inside `Approved::choose`. Detector records differed only in `site_offset`:

  - approved: `Approved::choose StateMachine`, site 87
  - substituted: `Approved::choose StateMachine`, site 146

  The checker reduces both to `(anchor, guarded_name)` and explicitly excludes `site_offset` from its registry key ([check.py:1672](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1672), [check.py:1680](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1680)). Consequently, the replacement consumes the same registry row and marks it matched ([check.py:1687](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1687)). Existing substitution negatives change the enclosing function or impl anchor, so they do not exercise this case ([test_check.py:4779](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/test_check.py:4779), [test_check.py:4828](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/test_check.py:4828)). The registry is exact to function/name, not to the guarded AST site claimed by round 10.

## NON-BLOCKING findings

- Top-level canonical prose is one publication step stale. At publication commit `4ecce48a`, it still instructs the coordinator to publish the E8 fingerprint and then begin review ([parity-closeout-status.md:1010](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/parity-closeout-status.md:1010)). The subplan correctly says round-ten reviews are now next ([runtime-frame-loop-status.md:843](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-status.md:843)).

- The closure heading and closeout text say twelve adversarial rows, but candidate mode declares and completes thirteen: “Permanent structural ratchets” is the thirteenth ([closure:480](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-c5-closure.md:480), [closure:669](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-c5-closure.md:669), [ownership:30](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-ownership.toml:30)). Enforcement itself is correct.

## Verified

- Clean detached publication `4ecce48a`; parent candidate exactly `e729dd74`; clean porcelain.
- Fingerprint reproduced exactly: `file_count=7310`, `sha256=51eedc8f4f8484825bde436a2578a7bd8d37f49265e9b86b0415621c68da7b91`, matching source and runner provenance ([trace:7850](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-trace.json:7850)).
- Live candidate-mode checker passed with all four owner-boundary ratchets at zero. Candidate mode is active and all thirteen declared rows are checked ([ownership:22](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-ownership.toml:22), [check.py:1110](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1110)).
- All ten round-nine syntactic evasions replayed and tripped. Fully resolved plain enum/dispatcher controls stayed clean. `self`, `super`, and `crate` qualification, parenthesized aliases, and macro/attribute fragment composition all tripped.
- Additional attacks:

  - Neutral trait-object call alone stayed clean; adding a non-owner implementation that directly calls `notify_events` tripped.
  - Function-pointer parameter invocation stayed clean as an allowed policy closure; acquiring the pointer via `StateMachineInstance::notify_events` tripped.
  - A callback field named `notify_events` stayed clean, but obtaining the raw guarded function locally still trips. These are legitimately safe unless combined with a separate laundering source.
- FL-G03 now includes the previously missing batching/bubbling citations ([gaps:68](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-gaps.toml:68)).
- `cargo fmt --check`, locked offline detector metadata/tree resolution, both `git diff --check` ranges, and clean porcelain passed.
- The 77-test Make target could not independently rerun because this sandbox provides no writable temporary directory; 73 tests failed before their bodies at `TemporaryDirectory()`. The non-mutating live checker and direct content-addressed detector replays succeeded.

**REJECT**