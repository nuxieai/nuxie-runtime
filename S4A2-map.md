# S4A2 integration map

Date: 2026-08-02

## State

- Branch: `levi/s4-ports-s4a`
- S4-42 upstream: `36aabf60d771a91a6e32b453409add2b5831b3c5`
- Completed local commit: `358513e56c3eb2a83e1cd02ccea5b659080ea9c7`
- Original verified base: `a93bf88533090392fb205f969e825dae5f330d74`
- Current shared `origin/main`: `01e226009a264620abab99e6af72b668b27a184c`
- State at handoff: ahead 1, behind 8

The implementation was committed as the one requested upstream-change commit. All requested gates passed on the original verified base. `origin/main` advanced during the gate run, and `git rebase origin/main` was unable to create `.git/worktrees/nuxie-p1c-importers/rebase-merge` because the shared Git directory is outside the writable sandbox. No rebase started.

## Integration boundary

Port only S4-42. S4-45 upstream `3c77a64d` remains WATCH and is not present. In particular, preserve the S4-42 `ViewModelRuntimeDataType::AssetBlob` typed-runtime variant, but do not add S4-45's command-queue/server `ViewModelPropertyAssetBlob -> AssetBlob` metadata mapping.

The eight commits newly present on `origin/main` touch 89 files. Their intersection with the 29-path S4-42 commit is limited to:

- `.s4-deferred-corpus.toml`
- `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`
- `tools/fetch-test-assets.sh`

An integrator with normal Git-directory permissions should rebase/cherry-pick `358513e56c3eb2a83e1cd02ccea5b659080ea9c7` onto `01e226009a264620abab99e6af72b668b27a184c`, merge those three paths additively, retain the exact commit-message format, and rerun every gate listed in `S4A2-report.md`.

## Committed path map

Schema and binary import:

- `crates/nuxie-schema/src/generated/schema.rs`
- `crates/nuxie-schema/tests/generated_schema.rs`
- `crates/nuxie-binary/src/importers/mod.rs`
- `crates/nuxie-binary/src/importers/viewmodel_instance_importer.rs`
- `crates/nuxie-binary/src/lib.rs`

Runtime values, binding, and typed facade:

- `crates/nuxie-runtime/src/data_bind/context/context_value.rs`
- `crates/nuxie-runtime/src/lib.rs`
- `crates/nuxie-runtime/src/scripting.rs`
- `crates/nuxie-runtime/src/state_machine/bindables.rs`
- `crates/nuxie-runtime/src/state_machine/data_bind_template.rs`
- `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`
- `crates/nuxie-runtime/src/view_model.rs`
- `crates/nuxie-runtime/src/view_model_cell.rs`
- `crates/nuxie-runtime/src/viewmodel/runtime/viewmodel_instance_runtime.rs`
- `crates/nuxie-runtime/src/viewmodel/runtime/viewmodel_instance_value_runtime.rs`
- `crates/nuxie-runtime/src/viewmodel/runtime/viewmodel_runtime.rs`
- `crates/nuxie-runtime/src/viewmodel/viewmodel_instance.rs`
- `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_asset.rs`
- `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_value.rs`
- `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_viewmodel.rs`

Scripting:

- `crates/nuxie-scripting/src/vm/lua_blob.rs`
- `crates/nuxie-scripting/src/vm/view_model.rs`

Tests, fixtures, and corpus tooling:

- `crates/nuxie-runtime/tests/blob_view_model.rs`
- `fixtures/sync/data_bind_blob_test.riv`
- `fixtures/sync/data_bind_blob_test.sriv`
- `fixtures/sync/data_enum_roundtrip.rml`
- `.s4-deferred-corpus.toml`
- `.gitignore`
- `tools/fetch-test-assets.sh`

No `Cargo.lock`, pin, vendored Taffy, vendored Luau, or existing golden is part of the commit.
