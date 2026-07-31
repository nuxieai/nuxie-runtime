## Findings

- **BLOCKING — the frozen publication is not reproducible.** The new detector inherits workspace fields ([detector Cargo.toml](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/Cargo.toml:1)), but the root workspace does not include or exclude it ([Cargo.toml](/Users/levi/dev/worktrees/nuxie-e6-review/Cargo.toml:1)). Consequently, the checker’s mandatory `cargo build --locked` step ([check.py](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/check.py:398)) fails:

  ```text
  current package believes it's in a workspace when it's not
  ```

  `Cargo.lock` also omits the detector package. The recorded 67/67 checker receipt cannot be reproduced from commit `499d86b8`.

- **BLOCKING — E6 fingerprint and runner provenance are stale.** On clean detached `499d86b8`, the repository’s official fingerprint command returns:

  ```text
  file_count=7301
  sha256=ab4fcbca55b50e1345c7317ac3dcee9e4a1659fcfd8b2202b87d3093c3c7b8b1
  ```

  The trace records `26aeffef37fa…` in both candidate and runner provenance ([trace](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-trace.json:7850)); the checker explicitly rejects both mismatches ([check.py](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/check.py:1139)). The recorded hash reproduces only in the development worktree containing the uncommitted `Cargo.toml`/`Cargo.lock` membership changes. This contradicts the “fingerprint-last/full checker green” publication claims ([status](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-status.md:23), [closure](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-c5-closure.md:937)).

- **BLOCKING — the detector remains structurally evadable.** All prior W66/W68/W71/W72/W73 forms now register: function aliases, cross-file audio aliases, UFCS, spaced `::`, raw identifiers, type aliases, plain/glob variant imports, and the full-token `member!(…, notify_events)` form. Method-on-alias and ordinary trait indirection also register. Unparseable content containing `notify_events` trips lexically.

  At least five new valid forms returned no hit:

  1. `#[cfg(not(test))]` production code is incorrectly skipped because any nested `test` token is classified as test-only ([main.rs](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:131), [visitor](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:461)).
  2. Qualified and nested module re-exports, e.g. `bridge::Anim::StateMachine`, are not propagated across module scopes ([resolver](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:155)).
  3. Trait-associated type indirection—`type Anim = <Host as Carrier>::Anim`—escapes selection resolution ([alias collection](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:180)).
  4. Attribute-macro composition such as `#[delegate(StateMachineInstance, notify_events)]` escapes.
  5. Identifier-building composition such as `paste! { StateMachineInstance::[<notify_ events>](...) }` escapes because macro scanning requires an already-complete guarded identifier, while the lexical tripwire runs only after whole-file parse failure ([macro scanner](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:428), [parse fallback](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:598)).

  The permanent macro negative only supplies the complete `notify_events` token ([test_check.py](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/test_check.py:4080)), and no permanent module-re-export negative exists, contrary to the closure claim ([closure](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-c5-closure.md:910)).

- **BLOCKING — the allowlist does not require an explicit comment tag.** It performs raw same-line substring matching ([main.rs](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:344)). A string literal containing `flc5-owner-ratchet-allow: dispatch`, or a comment ending in `dispatching`, suppresses the hit. Previous-line, wrong-kind, and bare tags correctly fail, but the documented comment-only/exact-kind policy is not enforced.

- **NON-BLOCKING — FL-G03 retains an expired round-seven action.** It still says W71/W72/W73 must classify the 30-to-11 topology difference ([gaps ledger](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-gaps.toml:68)), although those reviews are complete and W71–W74 contain no disposition. Reassign it to round eight or record its classification.

## Verified

- `NEXT` is finally current: it names round-eight review of `99ef7700`, contains no publish-this instruction, and gives acceptance/finding branches ([status](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-status.md:837)).
- The coordinator floor policy is consistently recorded as a Levi directive dated 2026-07-30 ([evidence README](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-c5-evidence/README.md:23), [implementation spec](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-c5-impl-spec.md:587)).
- The requested delta contains `3bef19da`, `99ef7700`, and `499d86b8`; `git diff --check 171b5703..499d86b8` passes.
- The trace’s `rust_ref`, artifact hashes, and floor7 SHA stamps are otherwise consistent.

**REJECT**