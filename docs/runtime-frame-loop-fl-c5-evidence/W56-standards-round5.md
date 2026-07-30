## BLOCKING

1. Displaced event/audio orchestration remains, while the new ratchets are shape-vacuous.

The corrective requires nested event collection, owner selection, dispatch, and audio unwinding to move into `state_machine_instance.rs`, leaving only borrow closures elsewhere ([implementation spec](/Users/levi/dev/worktrees/nuxie-e3-review/docs/runtime-frame-loop-fl-c5-impl-spec.md:47)).

That requirement is not met:

- `RuntimeNestedStateMachineInstance::advance` still advances the machine, iterates its report queue, and copies events into the caller’s collection ([nested_state_machine.rs](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/nested_state_machine.rs:326)). This is substantive event collection, not merely a borrow closure.
- `RuntimeNestedArtboardInstance::advance` still detects outer ownership, composes animation advancement and descendant dispatch, orders data-bind advancement, and conditionally initiates audio unwinding ([artboard.rs](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/artboard.rs:10169)).

The four new ratchets only reject narrow spellings: local `Vec` allocation or direct `notify_events`/`flush_deferred_owner_audio_events` calls ([gaps ledger](/Users/levi/dev/worktrees/nuxie-e3-review/docs/runtime-frame-loop-gaps.toml:1397)). Their negatives exercise the same narrow shapes ([test_check.py](/Users/levi/dev/worktrees/nuxie-e3-review/tools/runtime-frame-loop-port/test_check.py:3627)). Consequently, the live checker reports all four as zero while the displaced orchestration survives. Round-4 finding 1 remains open.

## NON-BLOCKING

1. The status pointer is one publication step stale.

The document says E3 is already published and the exact-tree checker is green ([status](/Users/levi/dev/worktrees/nuxie-e3-review/docs/runtime-frame-loop-status.md:7)), but “Next” still begins by instructing publication of the staged E3 packet ([status](/Users/levi/dev/worktrees/nuxie-e3-review/docs/runtime-frame-loop-status.md:831)). At publication commit `50bf85e8`, the next step should begin with the independent reviews.

## Independently verified

- Frozen `HEAD` is exactly `50bf85e8`; production candidate is `691c5262`.
- The live structural/provenance checker passed. Recomputed fingerprint exactly matched 7,294 files and SHA-256 `94a61dd834efc8e937e3cbe2763e01ae87b9984458b2f3d8b662eae7021afc87`; runner provenance also matched ([trace](/Users/levi/dev/worktrees/nuxie-e3-review/docs/runtime-frame-loop-trace.json:7810), [enforcement](/Users/levi/dev/worktrees/nuxie-e3-review/tools/runtime-frame-loop-port/check.py:997)).
- The enclosing 67-test Make target could not run in this read-only sandbox because Python had no writable temporary directory; this is environmental, not a candidate failure.
- Repository-wide semantic seam discovery and sibling-file rejection are now effective ([checker](/Users/levi/dev/worktrees/nuxie-e3-review/tools/runtime-frame-loop-port/check.py:1347), [negative](/Users/levi/dev/worktrees/nuxie-e3-review/tools/runtime-frame-loop-port/test_check.py:4316)).
- Receipt enumeration found 23 Git-tracked logs, including 10 under `superseded/`; all validate cleanly. Enumeration is recursive through Git, and the corruption test covers every enumerated fixture receipt ([guard](/Users/levi/dev/worktrees/nuxie-e3-review/tools/runtime-frame-loop-port/stamp_floor_receipt.py:19), [test](/Users/levi/dev/worktrees/nuxie-e3-review/tools/runtime-frame-loop-port/test_stamp_floor_receipt.py:27)).
- Checker count is corrected to 67/67, and all 37 `cpp_probe` acceptance commands include `--features tools`.
- `git diff --check 95333c41..50bf85e8` is clean; no additional delta-sweep violation found.

REJECT