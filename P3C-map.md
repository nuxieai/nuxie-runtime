# P3C2 scatter-fix map

Git metadata is read-only in this sandbox. The checked-out lane is still
`levi/mr2-c10` at merge commit `95b34b97`; creating
`levi/p3c-semantics` and committing both fail when Git tries to create a
lock under the shared `.git` directory. The behavior-neutral patch is left
in this worktree for the orchestrator.

## Consolidation

- `crates/nuxie-runtime/src/semantic_data.rs` now contains
  `RuntimeSemanticData` plus the types from the header-only upstream
  `semantic_node.hpp`. The standalone `semantic_node.rs` residue is
  deleted.
- `crates/nuxie-runtime/src/semantic_manager.rs` now contains
  `SemanticManager` plus the payload types from the header-only upstream
  `semantic_snapshot.hpp`. The standalone `semantic_snapshot.rs` residue
  is deleted.
- `crates/nuxie-runtime/src/lib.rs`, `semantic_provider.rs`, and
  `semantic_inference_registry.rs` point directly at those consolidated
  owners; the public crate-root API is unchanged.

## Four-place residue

- `file-correspondence-manifest.toml`: B6-0327 maps only to
  `semantic_data.rs`; B6-0329 maps only to `semantic_manager.rs`.
- `docs/runtime-frame-loop-ownership.toml`: this P3-c lane never landed
  semantic ownership rows or a semantic source set because the ledger remains
  the frozen `fl-e8-wave-candidate`; therefore there are no stale paths to
  move in either location.
- Attribution comments for `semantic_node.hpp` and
  `semantic_snapshot.hpp` moved with their definitions.
- The two standalone Rust files were deleted rather than retained as shims.
- `test-correspondence-manifest.toml` no longer cites the deleted node file.

No scatter exception or `[scatter_ratchet]` change is part of this patch.
See `P3C2-report.md` for gate evidence.
