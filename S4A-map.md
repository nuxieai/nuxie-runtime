# S4A reconstruction map

The sandbox blocked writes to the assigned worktree's shared Git metadata. Apply
the patches below in lexical order onto base
`89728ccc73683eda97ad186cf7b2f05dbf5dc176` with
`git am --keep-non-patch S4A-patches/*.patch`. The `--keep-non-patch` option preserves the required
leading `[sync]` in each commit subject. This application path was verified in
a fresh clone.

## Ordered patches

1. `S4A-patches/0001-sync-Port-rive-runtime-36aabf60-feat-add-blob-view-m.patch`
   - SHA-256: `3f2dcecb4e184fa3dad486e2708ec89fac38a6f7f98e2e62e1321ce44d92213a`
   - Reconstructs shadow commit `e37677ea63ecf6f32197d4b9a9f59d72487aa360`.
2. `S4A-patches/0002-sync-Port-rive-runtime-3c77a64d-feat-cmdq-unreal-ass.patch`
   - SHA-256: `84d7d7e843fc6537f85135351426a8f151c12589ebb14d6bb0ec0c2706bba61a`
   - Reconstructs shadow commit `50aab265daa47a9622bcd0b51f5af198c7d7348f`.

## Commit 1 file/hunk map — S4-42 / `36aabf60`

- Schema/codegen: `crates/nuxie-schema/src/generated/schema.rs`,
  `crates/nuxie-schema/tests/generated_schema.rs`, and
  `tools/nuxie-codegen/tests/generated_schema.rs` add and pin the two AssetBlob
  definitions and keys.
- Binary model/import: `crates/nuxie-binary/src/lib.rs`,
  `crates/nuxie-binary/src/importers/mod.rs`, and
  `crates/nuxie-binary/src/importers/viewmodel_instance_importer.rs` add the
  AssetBlob runtime type/value and instance importer dispatch.
- Runtime value/ownership: `crates/nuxie-runtime/src/view_model_cell.rs`,
  `view_model.rs`, `lib.rs`, and the files under
  `src/viewmodel/{runtime/,}` add blob cell identity, owned/imported/nested
  storage, cloning, lookup/apply paths, and typed runtime access. New C++ owner
  portions are regrouped into manifest-owned Rust modules rather than enrolling
  new direct files before the pin advance.
- Data binding/state machine: `src/data_bind/context/context_value.rs` and
  `src/state_machine/{bindables.rs,data_bind_template.rs,state_machine_instance.rs}`
  retain live blob asset handles, resolve source values, and apply blob
  bindings in default, imported, and owned contexts.
- Scripting/Luau: `crates/nuxie-runtime/src/scripting.rs` and
  `crates/nuxie-scripting/src/vm/{lua_blob.rs,view_model.rs}` add blob property
  reads/writes, retained asset identity/name, listener dispatch, and
  nil/string/buffer/blob coercion.
- Tests: `crates/nuxie-runtime/tests/blob_view_model.rs` plus focused unit tests
  in the touched runtime/scripting modules cover import, pointer-preserving
  no-op writes, replacement, live empty payloads, clearing, and bind retention.
- Fixtures/provenance: `.gitignore`, `.s4-deferred-corpus.toml`,
  `tools/fetch-test-assets.sh`, and the three new `fixtures/sync/` assets vendor
  and checksum the wave without enrolling the corpus or moving a golden.

## Commit 2 file/hunk map — S4-45 in-scope portion / `3c77a64d`

- `crates/nuxie-runtime/src/viewmodel/runtime/viewmodel_runtime.rs`: map
  `ViewModelPropertyAssetBlob` to `ViewModelRuntimeDataType::AssetBlob`.
- `crates/nuxie-runtime/tests/blob_view_model.rs`: assert the public runtime
  property metadata exposes the AssetBlob type.
- Intentionally absent: command-queue server/protocol and Unreal changes (F3
  WATCH).
