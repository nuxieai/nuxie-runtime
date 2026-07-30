## Spec / Oracle findings

- **BLOCKING — same-item registry relocation remains fungible.** W89 requires any relocation of a blessed site to change `site_hash` and fail both registry directions ([W89:63](/Users/levi/dev/worktrees/nuxie-e9-verify/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W89-round11-spec.md:63), [W89:80](/Users/levi/dev/worktrees/nuxie-e9-verify/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W89-round11-spec.md:80)). I moved this unchanged statement between the permitted and forbidden branches of the same `Approved::choose` function:

  ```rust
  if let FutureAnim::StateMachine(owner) = animation {
      approved_policy(owner);
  }
  ```

  Candidate-detector output before and after:

  ```text
  match selection 33 107 Approved::choose StateMachine c82c3c83e48dc016...
  match selection 33 135 Approved::choose StateMachine c82c3c83e48dc016...
  ```

  Only `site_offset` changes. The detector hashes the innermost statement ([main.rs:1345](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:1345)) and records that hash separately from the offset ([main.rs:770](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:770)). The checker’s registry key and suppression match contain only `(anchor, guarded_name, site_hash)`, excluding `site_offset` ([check.py:1681](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/check.py:1681)). Thus the relocated occurrence consumes the original row and passes.

  The permanent test misses this exact attack: its “relocated” source changes both the scrutinee and policy body, guaranteeing a different statement hash ([test_check.py:5090](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/test_check.py:5090), [test_check.py:5109](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/test_check.py:5109)).

No non-blocking findings.

## Required confirmations

- **Scope: PASS.** Every `e729dd74..14b18765` path is under `docs/` or `tools/runtime-frame-loop-port/`, as required by [W89](/Users/levi/dev/worktrees/nuxie-e9-verify/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W89-round11-spec.md:11). The `crates` tree object is identical at `e729dd74`, `14b18765`, and publication `a4522812`:

  ```text
  473036c1d4eb1491bbc5b10f3b5ca2241dcc5d2c
  ```

  Publication `a4522812` changes docs only and preserves the candidate detector tree. The checkout is clean.

- **Reverse macro replay: CLOSED.** Exact `reverse_join!(events, notify_)` now emits:

  ```text
  match dispatch ... evades notify_events a796d622...
  ```

  The order-insensitive multiset search is present at [main.rs:1014](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:1014).

- **Owner-origin `Chosen`: CLOSED.** The owner source emits `export selection Chosen`; the non-owner consumer emits `match selection ... outside Chosen ...`. The permanent replay is present at [test_check.py:4751](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/test_check.py:4751).

- **38-row registry audit: PASS.** Regenerating all non-owner production matches with the content-addressed candidate detector and propagated owner exports produced:

  ```text
  registry rows:       38 unique
  detector matches:    38 unique
  missing rows:         0
  unregistered hits:    0
  duplicate keys:       0
  ```

  Breakdown: `artboard.rs` has 24 selection, 4 dispatch, and 2 audio records; `focused_input_dispatch.rs` has 8 selection records. The latter are four real paths, each producing separate enum-name and variant-name matches, as visible in production at [focused_input_dispatch.rs:55](/Users/levi/dev/worktrees/nuxie-e9-verify/crates/nuxie-runtime/src/state_machine/focused_input_dispatch.rs:55). The registry begins at [runtime-frame-loop-gaps.toml:1427](/Users/levi/dev/worktrees/nuxie-e9-verify/docs/runtime-frame-loop-gaps.toml:1427) and ends at [runtime-frame-loop-gaps.toml:1686](/Users/levi/dev/worktrees/nuxie-e9-verify/docs/runtime-frame-loop-gaps.toml:1686). No registry row is an unexplained loosening.

- **Held behavior: PASS by source identity.** `nuxie-runtime` remains `84af0b36…`, `nuxie` remains `35c5a2e4…`, and the runtime-tests subtree remains `91cd2cec…` across base and candidate. Consequently the complete round-10 held-behavior source set is byte-identical; no behavioral finding is introduced.

## Standards

No standards findings. Scope discipline and `git diff --check` pass.

The registry population is legitimate and two detector evasions are closed, but the third required W86 closure still accepts a literal same-item relocation.

**REJECT**