# MR-2b C03 runtime-constraints report

## Scope

C03 processed the 18 plan-classified `split-needed` constraint rows rooted in `crates/nuxie-runtime/src/constraints.rs`. All changes are behavior-neutral source moves or direct importer redirects; signatures and public API remain unchanged. The foreign-owned crate roots listed in the assignment were not modified.

## Moved rows

- B6-0124 → `crates/nuxie-runtime/src/constraints/constrainable_list.rs`
- B6-0125 → `crates/nuxie-runtime/src/constraints/constraint.rs`
- B6-0126 → `crates/nuxie-runtime/src/constraints/distance_constraint.rs`
- B6-0127 → `crates/nuxie-runtime/src/constraints/draggable_constraint.rs`
- B6-0128 → `crates/nuxie-runtime/src/constraints/follow_path_constraint.rs`
- B6-0129 → `crates/nuxie-runtime/src/constraints/ik_constraint.rs`
- B6-0130 → `crates/nuxie-runtime/src/constraints/list_constraint.rs`
- B6-0131 → `crates/nuxie-runtime/src/constraints/list_follow_path_constraint.rs`
- B6-0132 → `crates/nuxie-runtime/src/constraints/rotation_constraint.rs`
- B6-0133 → `crates/nuxie-runtime/src/constraints/scale_constraint.rs`
- B6-0138 → `crates/nuxie-runtime/src/constraints/scrolling/scroll_constraint.rs` (with shared metrics/rendezvous helpers retained in `constraints.rs`)
- B6-0139 → `crates/nuxie-runtime/src/constraints/scrolling/scroll_constraint_proxy.rs`
- B6-0140 → `crates/nuxie-runtime/src/constraints/scrolling/scroll_physics.rs`
- B6-0141 → `crates/nuxie-runtime/src/constraints/scrolling/scroll_virtualizer.rs`
- B6-0142 → `crates/nuxie-runtime/src/constraints/targeted_constraint.rs`
- B6-0143 → `crates/nuxie-runtime/src/constraints/transform_constraint.rs`
- B6-0144 → `crates/nuxie-runtime/src/constraints/translation_constraint.rs`

Each moved row updates the file-correspondence manifest, the frame-loop per-row ownership array, the `component-update-graph` source-set module list, and attribution-comment location. Importers were redirected instead of leaving one-line forwarding shims.

## Justified exceptions

- B6-0134: the clamped branches are embedded in the shared `RuntimeScrollPhysicsState` lifecycle with entangled siblings B6-0140 and B6-0135. That shared lifecycle moved to the smallest natural owner, `scrolling/scroll_physics.rs`; forcing a separate `clamped_scroll_physics.rs` would fragment individual match branches.

The corresponding manifest note names the entangled siblings and explains why no behavior-neutral complete split exists.

## Queued rows

- B6-0388 remains queued for the C02-owned `text_input.rs` landing, as required by the move plan. No partial C03 change was made for it.

## Verification

- Per-batch `cargo check --workspace --exclude nux-capi`: passed for the committed leaf batch and subsequent extraction batches, with existing warnings.
- Focused `cargo check -p nuxie-runtime`: passed after the constraint, virtualizer, and proxy splits.
- Final `cargo check --workspace --exclude nux-capi`: passed, with existing warnings.
- `make runtime-frame-loop-port-check`: passed (108 tests plus test-correspondence and ownership-ledger validation).
- `make rust-attribution-check`: passed (10 tests; every in-scope Rust source classified).
- Git commit `6caa235a` contains B6-0126, B6-0132, B6-0133, B6-0143, and B6-0144. Later commit attempts were blocked by shared-worktree Git metadata permissions (`index.lock: Operation not permitted`); the remaining verified changes are left in the worktree.
