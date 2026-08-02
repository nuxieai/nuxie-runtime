# MR-2 major-root report: C06 / binary-importers

- Branch: `levi/mr2-c06`
- Owned root: `crates/nuxie-binary/src/importers/mod.rs`
- Base: `origin/main`
- Scope source: `.parity-decomp/mr-move-plan.md`, C06 primary hotspot table

## Moved

The importer-specific accept/drop dispatch and context-update routing now defer to the existing dedicated importer implementations. Each completed row's manifest correspondence names only its planned target module. `audit_record` and `b6_verdict` are unchanged.

| Row | Planned target |
|---|---|
| `B6-0212` | `crates/nuxie-binary/src/importers/artboard_importer.rs` |
| `B6-0215` | `crates/nuxie-binary/src/importers/data_bind_path_importer.rs` |
| `B6-0216` | `crates/nuxie-binary/src/importers/data_converter_formula_importer.rs` |
| `B6-0217` | `crates/nuxie-binary/src/importers/data_converter_group_importer.rs` |
| `B6-0218` | `crates/nuxie-binary/src/importers/enum_importer.rs` |
| `B6-0219` | `crates/nuxie-binary/src/importers/file_asset_importer.rs` |
| `B6-0220` | `crates/nuxie-binary/src/importers/keyed_object_importer.rs` |
| `B6-0222` | `crates/nuxie-binary/src/importers/layer_state_importer.rs` |
| `B6-0223` | `crates/nuxie-binary/src/importers/linear_animation_importer.rs` |
| `B6-0224` | `crates/nuxie-binary/src/importers/listener_input_type_gamepad_importer.rs` |
| `B6-0225` | `crates/nuxie-binary/src/importers/listener_input_type_keyboard_importer.rs` |
| `B6-0226` | `crates/nuxie-binary/src/importers/listener_input_type_semantic_importer.rs` |
| `B6-0227` | `crates/nuxie-binary/src/importers/scripted_object_importer.rs` |
| `B6-0229` | `crates/nuxie-binary/src/importers/state_machine_layer_component_importer.rs` |
| `B6-0230` | `crates/nuxie-binary/src/importers/state_machine_layer_importer.rs` |
| `B6-0231` | `crates/nuxie-binary/src/importers/state_machine_listener_importer.rs` |
| `B6-0233` | `crates/nuxie-binary/src/importers/text_asset_importer.rs` |

## Skipped

| Row | Classification | Reason |
|---|---|---|
| `B6-0213` | exception (`E-B6-0213`) | Planned exception; shared binary/root/runtime/facade fragments remain untouched. |
| `B6-0214` | exception (`E-B6-0214`) | Planned crate-bound trait/adapter split; runtime-owned fragment remains untouched. |

## Split-needed

| Row | Reason | Queue |
|---|---|---|
| `B6-0235` | Also touches C07-owned `crates/nuxie-binary/src/lib.rs`; no partial row or manifest move was made. | C07 + C17 atomic assembly |
| `B6-0236` | Also touches C07-owned `crates/nuxie-binary/src/lib.rs`; no partial row or manifest move was made. | C07 + C17 atomic assembly |
| `B6-0237` | Also touches C07-owned `crates/nuxie-binary/src/lib.rs`; no partial row or manifest move was made. | C07 + C17 atomic assembly |

## Cross-root

- `B6-0235`, `B6-0236`, and `B6-0237` were queued without edits. Their C06 and C07 fragments and manifest rows must be landed together by C17 after C07 reconciliation.
- No C07 (`nuxie-binary/src/lib.rs`), C08 (`vm.rs`), C09 (`nuxie-runtime/src/lib.rs`), or C10 (`nuxie/src/lib.rs`) owned root was edited.

## Verification and commits

- Batch 1: `cargo check --workspace --exclude nux-capi` passed (warnings only).
- Batch 2: `cargo check --workspace --exclude nux-capi` passed (warnings only).
- `cargo fmt --all -- --check` and `git diff --check` passed for both batches.
- Batch commit: `[MR-2/C06] Move B6-0212,B6-0215-B6-0220,B6-0222-B6-0223 from importers/mod.rs`.
- Batch commit: `[MR-2/C06] Move B6-0224-B6-0227,B6-0229-B6-0231,B6-0233 from importers/mod.rs`.

`nux-capi` was excluded as required because its check needs network access.
