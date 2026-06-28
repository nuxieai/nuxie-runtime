# Binary Import Public Surface Audit

Date: 2026-06-28

This audit classifies the current public `rive-binary` surface against
[`binary-import-completion-contract.md`](binary-import-completion-contract.md).
It exists to keep the completion goal finite: `rive-binary` should finish binary
import parity, not quietly become the post-import runtime.

## Evidence

Current inventory:

- `RuntimeFile` public methods live in `crates/rive-binary/src/lib.rs`.
- The `RuntimeFile` implementation currently exposes 296 public methods.
- The widest families are view-model imported-data helpers, data-bind runtime
  behavior helpers, artboard import collections, converter helpers, and import-time
  resolution helpers.

This document classifies helpers by method family. `_for_object`, named lookup,
byte-slice lookup, index lookup, and object-id wrapper variants inherit the same
classification as their family unless called out separately.

## Classification Labels

- **Import-owned**: belongs in `rive-binary`; it describes bytes, decoded objects,
  import status, generated defaults, or immediate C++ import relationships.
- **Test-supporting**: acceptable while proving parity, but should not become a
  reason to keep expanding the public interface.
- **Move-later**: existing surface that models post-import runtime behavior. Keep
  frozen for now, but move to `rive-graph` or a future runtime crate when that
  crate owns the corresponding state.
- **Do-not-expand**: no new helpers should be added in this family unless the
  completion contract's admission rule proves the work changes immediate import.

## Import-Owned Surface

These families are within the `rive-binary` seam.

| Family | Representative helpers | Reason |
| --- | --- | --- |
| Core imported arena | `object_count`, `known_object_count`, `object`, `import_status`, `imported_object_count` | Direct representation of decoded object slots and C++ keep/drop/null status. |
| File header and top-level read functions | `read_runtime_file`, `read_runtime_file_with_error_kind`, `RuntimeHeader` data | Byte-level file loading and import-result classification. |
| Runtime object property views | `property`, `skipped_property`, `string_property`, `uint_property`, `bool_property`, `color_property`, `double_property`, `bytes_property` | Sparse serialized properties plus generated default lookup are core import output. |
| Encoded imported byte views | `id_list_property`, `data_bind_path_ids`, `mesh_triangle_indices`, `file_asset_cdn_uuid_string` | C++ decodes these embedded byte fields during or immediately after import. |
| File asset helpers | `file_asset_extension`, `file_asset_unique_name`, `file_asset_unique_filename`, `file_assets`, `file_asset`, `resolved_file_asset_for_referencer` | C++ file import exposes these through imported asset collections and helper methods. |
| Manifest resolver | `manifest`, `resolve_name`, `resolve_name_bytes`, `resolve_path` | Manifest contents are imported from file asset content and used by immediate path resolution. |
| Artboard collection lookup | `artboards`, `artboard`, `default_artboard`, `artboard_named`, `artboard_named_bytes` | Mirrors `File::artboard(...)` immediately after import. |
| Imported artboard-local collections | `artboard_animations`, `artboard_linear_animations`, `artboard_state_machines`, `artboard_state_machine_graphs`, `artboard_data_binds` | Finite imported ownership facts used by C++ corpus comparison. |
| Imported artboard-local relationship snapshots | `artboard_skins`, `artboard_meshes`, `artboard_paths`, `artboard_shapes`, `artboard_shape_paint_containers`, `artboard_n_slicer_details` | Immediate registration/validation facts from C++ import or artboard validation. |
| File-level imported collections | `view_models`, `data_enums`, `data_converters`, `data_converter_interpolators`, `scroll_physics` | Mirrors backboard/file collections populated while reading. |
| Import-time reference resolution | `resolved_view_model_for_artboard`, `resolved_artboard_for_referencer`, `resolved_scroll_physics_for_constraint`, `resolved_data_converter_for_data_bind`, `resolved_interpolator_for_data_converter`, `resolved_data_converter_for_group_item`, `resolved_view_model_for_number_to_list_converter` | C++ establishes or reads these relationships during file import or immediate validation. |
| Data-bind path buffers | `data_bind_path_for_referencer`, `resolved_data_bind_path_ids_for_referencer`, `listener_input_type_view_model_path_ids_buffer`, `data_bind_context_source_path_ids`, `data_bind_context_resolved_source_path_ids` | Imported encoded buffers and manifest-backed path expansion are static import facts. |
| Data enum lookup | `data_enum`, `data_enum_value_for_key`, `data_enum_value_for_index`, `data_enum_value_index_for_key`, `data_enum_value_index_for_index` | Mirrors imported enum collections and C++ lookup semantics over those collections. |
| View-model collections and ownership | `view_model`, `view_model_named`, `view_model_property_named`, `view_model_property_for_symbol`, `view_model_instance_named`, `view_model_default_instance` | C++ completes these ownership relationships during file import. |
| Imported view-model value snapshots | `view_model_instance_value_*`, `view_model_property_for_instance_value`, `referenced_view_model_instance_for_value`, `referenced_view_model_instance_for_list_item` | Imported data values and import-time view-model instance references. |
| View-model enum and asset references | `data_enum_for_view_model_property`, `view_model_property_enum_value_*`, `data_enum_for_view_model_instance_enum_value`, `view_model_instance_enum_*`, `view_model_instance_asset_file_assets`, `resolved_file_asset_for_view_model_instance_asset` | Static imported-data helper surfaces over imported enum and asset collections. |

These families may still need polish, but they are not scope violations.

## Test-Supporting Surface

These helpers are acceptable as parity probes, but they should be treated as
supporting evidence rather than the desired long-term interface.

| Family | Representative helpers | Notes |
| --- | --- | --- |
| Data-bind static flags | `data_bind_to_source`, `data_bind_to_target`, `data_bind_binds_once`, `data_bind_is_main_to_source`, `data_bind_source_to_target_runs_first`, `data_bind_is_name_based` | Derived from imported flags. Fine to keep, but future runtime code should probably own richer data-bind state. |
| Data-bind target/output facts | `data_bind_target_supports_push`, `data_bind_uses_persisting_list`, `data_bind_source_output_type`, `data_bind_output_type` | Static over imported target/converter relationships today; avoid expanding into live target mutation. |
| Data-converter static facts | `data_converter_output_type`, `data_converter_group_items`, `data_converter_formula_tokens` | Useful to compare imported converter graphs. Runtime converter execution should not grow here. |
| Data context imported lookup | `data_context_view_model_property`, `data_context_relative_view_model_property`, `data_context_view_model_instance`, `data_context_relative_view_model_instance` | Uses imported view-model chains and manifest maps. Keep as a parity helper; future live data contexts belong in runtime. |
| Source-data snapshots | `view_model_instance_source_data_value` | Useful bridge from imported values to converter tests. Further source synchronization belongs outside `rive-binary`. |

The rule for this group: maintenance is allowed, but expansion requires a fresh
contract admission check.

## Move-Later, Do-Not-Expand Surface

These families model post-import runtime behavior. They exist because earlier
parity exploration reached into data-binding and converter lifecycle semantics.
They should be frozen, not used as precedent for more `rive-binary` work.

| Family | Representative helpers | Why it should move |
| --- | --- | --- |
| Data-bind lifecycle effects | `data_bind_add_effect`, `data_bind_bind_effect`, `data_bind_unbind_effect`, `data_bind_initialize_effect`, `data_bind_relink_effect`, `data_bind_context_bind_effect`, `data_bind_update_effect`, `data_bind_remove_effect` | These describe live binding, source/target observation, context assignment, and update behavior after import. |
| Data-bind dirt/collapse helpers | `data_bind_can_skip`, `data_bind_collapse_effect`, `data_bind_add_dirt_effect` | They depend on mutable collapsed/dirt/container state that is not part of the imported file. |
| Data-bind container scheduling | `data_bind_container_bind_context_effect`, `data_bind_container_unbind_effect`, `data_bind_container_advance_effect`, `data_bind_container_add_dirty_effect`, `data_bind_container_update_effect`, `data_bind_update_queue`, `sorted_data_bind_ids` | Queue membership, processing state, and ordering belong with the runtime scheduler. |
| Data-bind converter execution | `data_bind_convert`, `data_bind_convert_with_context`, `data_bind_reverse_convert`, `data_bind_reverse_convert_with_context`, `data_bind_stateful_advance` | This is live data-flow execution, not file import. |
| Data-converter lifecycle effects | `data_converter_bind_context_effect`, `data_converter_unbind_effect`, `data_converter_update_effect`, `data_converter_mark_dirty_effect`, `data_converter_property_change_effect`, `data_converter_add_dirty_data_bind_effect`, `data_converter_formula_add_dirt_effect`, `data_converter_reset_effect` | These describe runtime binding, dirt propagation, and lifecycle forwarding. |
| Stateless converter execution | `data_converter_convert`, `data_converter_convert_with_context`, `data_converter_convert_with_formula_randoms`, reverse-convert variants | Converter execution is runtime behavior even when deterministic. |
| Stateful converter execution | `data_converter_stateful_convert`, `data_converter_stateful_reverse_convert`, `data_converter_stateful_advance`, `data_converter_interpolator_convert`, `data_converter_interpolator_reverse_convert`, `data_converter_interpolator_advance` | Requires mutable runtime converter/interpolator state and elapsed time. |
| Joystick post-import resolution | `resolved_handle_source_for_joystick`, `resolved_x_animation_for_joystick`, `resolved_y_animation_for_joystick` | These are currently useful as audited `onAddedDirty`/`onAddedClean` parity facts, but future joystick behavior belongs with graph/runtime lifecycle. |
| Artboard component list selection | `artboard_component_list_map_rules`, `resolved_artboard_for_artboard_component_list_item` | Map rules are imported facts, but list-item artboard selection is close to runtime data-driven behavior. Treat as frozen unless needed for corpus parity. |

Existing tests around these helpers may remain as regression locks while the port
is young. New work should move toward extraction, not deeper `rive-binary`
coverage.

## Current Completion Implications

This audit satisfies the contract requirement that the public `RuntimeFile`
surface be classified, with one caveat: classification is by helper family rather
than by duplicating all 296 public method names. That is intentional. The wrappers
and `_for_object` variants are mechanically equivalent for scope purposes.

The goal is not complete merely because this audit exists. The remaining closure
work is:

- Use this audit as the gate for new `rive-binary` work.
- Stop adding new data-bind or converter runtime helpers to `rive-binary`.
- Decide later whether the move-later families should be hidden, feature-gated, or
  physically moved into a future runtime crate.
- Build a final completion matrix for the whole contract: schema coverage, binary
  decode coverage, import hook/source-audit coverage, C++ fixture comparison, and
  final verification commands.

## Admission Decision Template

For each proposed new helper or probe field, record:

```text
Name:
Classification: Import-owned | Test-supporting | Move-later | Out-of-scope
Admission rule answers:
1. Affects byte decoding?
2. Affects C++ accept/reject/drop/keep during import?
3. Creates an immediate import-time relationship fact?
4. Needed for fixture corpus comparison?
Decision:
```

If the decision is not clearly import-owned or narrowly test-supporting, it should
not be added to `rive-binary`.
