# Reverse correspondence map

## Scope and method

Snapshot: `58cbe38b071982dfc932ee7ac3ceab8465a6125b`, taken `2026-08-02T00:02:43Z`. The concurrent audit had dirty edits in `artboard.rs`, `draw.rs`, and `tests/scene_authoring.rs`; those working-tree contents are included.

The manifest pins `rive-app/rive-runtime` at `d788e8ec6e8b598526607d6a1e8818e8b637b60c` and declares 448 C++ rows. It contains `upstream` and semicolon-delimited `rust_module` fields, but no separate `anchor` field. The inversion produced 522 Rust references, 338 unique attributed Rust paths, 425 attributed C++ rows, and 23 blank/unported rows. [Manifest schema and pin](/Users/levi/dev/worktrees/nuxie-fld1/file-correspondence-manifest.toml:7) [Representative row](/Users/levi/dev/worktrees/nuxie-fld1/file-correspondence-manifest.toml:27)

LOC means physical lines in every `*.rs` under the five crate trees, including `build.rs`, unit-test regions, and integration tests. No builds ran.

Class codes:

- `D`: DIRECT-OWNER, exactly one C++ entry in the inverted Rust→C++ index.
- `Cₙ`: COORDINATOR, `n` C++ entries.
- `M`: MIXED coordinator; mapped behavior plus substantial unattributed regions.
- `O-SA`: NUXIE-ORIGINAL, Scene/authoring API.
- `O-FA`: NUXIE-ORIGINAL, FlowSession/host ABI.
- `O-RR`: NUXIE-ORIGINAL, retained-render layer.
- `O-CS`: NUXIE-ORIGINAL, codegen/schema/module plumbing.
- `O-TI`: NUXIE-ORIGINAL, test infrastructure.
- `O-?`: UNKNOWN/uncharted; no manifest attribution and no clear product-layer justification.

`D` is a file-accounting result, not a claim that the implementation has zero behavioral adaptation. Likewise, mixed files remain in the coordinator bucket to avoid double-counting their embedded product/test regions as whole-file originals.

## Summary

| Whole-file bucket | Files | LOC | Total |
|---|---:|---:|---:|
| Direct owners | 310 | 85,083 | 19.20% |
| Coordinators excluding mixed | 23 | 35,221 | 7.95% |
| Mixed coordinators | 5 | 95,299 | 21.50% |
| **All coordinators** | **28** | **130,520** | **29.45%** |
| Original — Scene/authoring API | 2 | 39,418 | 8.89% |
| Original — FlowSession/ABI | 7 | 9,020 | 2.04% |
| Original — retained render | 3 | 5,476 | 1.24% |
| Original — codegen/schema | 18 | 6,914 | 1.56% |
| Original — test infrastructure | 41 | 156,729 | 35.36% |
| Original — UNKNOWN | 14 | 10,061 | 2.27% |
| **Total** | **423** | **443,221** | **100%** |

By crate:

| Crate | Files | LOC |
|---|---:|---:|
| `nuxie-runtime` | 312 | 291,291 |
| `nuxie` | 20 | 81,302 |
| `nuxie-scripting` | 40 | 20,916 |
| `nuxie-binary` | 48 | 45,212 |
| `nuxie-render-api` | 3 | 4,500 |

The headline delta is therefore 51.35% zero-attribution LOC. Most is deliberately non-port code: 35.36% test infrastructure and 8.89% Scene/authoring. The suspicious zero-attribution floor is 10,061 LOC, or 2.27%, before considering uncharted regions embedded in mixed coordinators.

## Five largest coordinators

Estimates use top-level impl/module boundaries and explicit test boundaries, not token-level provenance. Treat them as ±10 percentage points.

| File | LOC | C++ rows | Est. unattributed | Major unattributed subsystems |
|---|---:|---:|---:|---|
| [draw.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/draw.rs:1) | 33,351 | 18 | ~55%, ~18.3k LOC | Public geometry/hit-query API; Taffy layout adapter; backend resource/cache ownership; exact stroke/fill geometry; path measuring/effects; extensive inline tests. The explicit additions begin with geometry queries, continue through the Rust layout engine and renderer caches, and the main test module starts around line 25,064. [geometry API](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/draw.rs:662) [Taffy adapter](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/draw.rs:10682) [renderer caches](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/draw.rs:13159) [exact geometry](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/draw.rs:20276) [tests](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/draw.rs:25064) |
| [artboard.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/artboard.rs:1) | 21,444 | 13 | ~55%, ~11.8k LOC | Rust occurrence identity and cold-clone rules; external font/image ownership; scripting lifecycle; generated-property façade; retained data-bind/view-model bridge; geometry/debug APIs; ~10.1k lines of inline tests beginning at line 11,318. [occurrence/script state](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/artboard.rs:183) [external fonts](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/artboard.rs:944) [scripting lifecycle](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/artboard.rs:2644) [tests](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/artboard.rs:11318) |
| [state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:1) | 20,781 | 2 | ~45%, ~9.4k LOC | Probe chronology and host seams; scripted-input/data-converter hydration; Rust retained-cell integration; convenience setters and data-context projections; ~7.4k lines of tests. [probe seam](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:86) [scripted hydration](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:3618) [retained bindings](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:8182) [tests](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:13417) |
| [nuxie-binary/lib.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-binary/src/lib.rs:1) | 12,911 | 4 | ~85%, ~11.0k LOC | Centralized runtime model, authoring records, generated-schema accessors, data-bind/converter simulation, view-model queries, import validation and fallback decoding. Only four manifest rows attribute this entire surface. [runtime/authoring model](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-binary/src/lib.rs:43) [authoring API](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-binary/src/lib.rs:449) [converter surface](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-binary/src/lib.rs:1243) [generic decoder](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-binary/src/lib.rs:11453) |
| [nuxie/lib.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie/src/lib.rs:1) | 6,812 | 3 | ~60%, ~4.1k LOC | Script authorization/mount orchestration, bounded import, external assets, borrowed and owned public façades, geometry/view-model convenience APIs, and tests. These are overwhelmingly justified product/API deltas. [script authorization](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie/src/lib.rs:50) [public file façade](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie/src/lib.rs:3016) [import limits](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie/src/lib.rs:3133) [owned façade](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie/src/lib.rs:4473) |

The coordinator mapping itself comes directly from the inverted manifest—for example the `state_machine_instance.cpp`, `artboard.cpp`, binary importer, and script-asset rows. [state-machine mapping](/Users/levi/dev/worktrees/nuxie-fld1/file-correspondence-manifest.toml:939) [artboard mapping](/Users/levi/dev/worktrees/nuxie-fld1/file-correspondence-manifest.toml:1143) [binary mapping](/Users/levi/dev/worktrees/nuxie-fld1/file-correspondence-manifest.toml:2763) [script mapping](/Users/levi/dev/worktrees/nuxie-fld1/file-correspondence-manifest.toml:1287)

## Uncharted list

Ranked by exact file LOC or estimated major-region size. These are not merely “unattributed”; they also lack an obvious Scene, host ABI, retained-render, schema, or testing justification.

| Rank | File/region | LOC basis | Why uncharted |
|---:|---|---:|---|
| 1 | `nuxie-binary/src/lib.rs` generic converter/view-model/import regions | ~8–10k region | Port-shaped C++ semantics far exceed the four manifest rows; the central owner needs additional C++ rows or a documented consolidation rule. |
| 2 | `nuxie-runtime/src/draw.rs` shape/path/effect and geometry-core regions | ~6–8k region | Numerous direct-port comments cite `shape.cpp`, `shape_paint.cpp`, clipping, stroke, trim and path-measure behavior absent from the file’s 18-row index. |
| 3 | `state_machine_instance.rs` retained view-model/data-bind/action regions | ~5–7k region | Much of the implementation is pinned-C++ machinery, not a product façade, but only two C++ rows attribute the file. |
| 4 | [view_model.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/view_model.rs:1) | 4,215 exact | Large retained/default view-model implementation with zero manifest attribution. |
| 5 | `artboard.rs` retained data-bind/view-model and callback regions | ~3–5k region | Port-shaped runtime behavior beyond its 13 mapped rows; product scripting/assets and tests were excluded from this estimate. |
| 6 | [state_machine/bindables.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/bindables.rs:1) | 2,216 exact | Implements C++ bindable families, including explicit `BindablePropertyAsset` semantics, but has no row. |
| 7 | [view_model_cell.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/view_model_cell.rs:1) | 2,115 exact | Self-identifies as a port of `ViewModelInstanceValue`, `DependencyHelper`, and `SuppressDelegation`; this is a clear manifest omission. |
| 8 | [listener_action_owner.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/listener_action_owner.rs:1) | 485 exact | Explicitly implements `ListenerAction`, `StateMachineFireAction`, and `Artboard::instance` ownership. |
| 9 | [data_bind_template.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/data_bind_template.rs:1) | 408 exact | Explicit port of backboard/data-bind clone and authored-order semantics. |
| 10 | [transition_duration_binding.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/transition_duration_binding.rs:1) | 163 exact | Runtime transition/data-bind behavior with no row or product-layer explanation. |
| 11 | [scripted_object_lifecycle.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/scripted_object_lifecycle.rs:1) | 127 exact | Explicit adaptation of `StateMachineInstance::internalDataContext`; should be attributed even if divergence is intentional. |
| 12 | [data_converter_binding.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/data_converter_binding.rs:1) | 94 exact | Explicit port of `DataBindContext::bindFromContext` call ordering. |
| 13 | [transition_condition_op.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/transition_condition_op.rs:1) | 57 exact | Runtime transition behavior without attribution. |
| 14 | [data_converter_trigger.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/data_converter_trigger.rs:1) | 56 exact | Explicitly claims pinned `DataConverterTrigger` semantics despite no manifest row. |
| 15 | [state_machine_fire_event.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/state_machine/state_machine_fire_event.rs:1) | 48 exact | Explicit pinned-C++ event occurrence with no row. |
| 16 | [nested_bool.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/nested_bool.rs:1) | 38 exact | Explicit port of `nested_bool.cpp`; the manifest currently attributes that C++ file only to `animation.rs`. |
| 17 | [nested_number.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/nested_number.rs:1) | 23 exact | Explicit port of `nested_number.cpp`; missing reverse attribution. |
| 18 | [nested_trigger.rs](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-runtime/src/nested_trigger.rs:1) | 16 exact | Explicit port of `nested_trigger.cpp`; missing reverse attribution. |

The 14 exact unknown files total 10,061 LOC. The four mixed-file regions show the larger structural risk: the ledger often points a C++ file at a thin direct module while substantial implementation lives in a coordinator without a corresponding row.

# Complete per-file ledger

The tables below are sorted by crate and then path. C++ cells are the literal inverted manifest values.

<details>
<summary><strong>crates/nuxie-runtime — 312 files, 291,291 LOC</strong></summary>

| Rust file | LOC | Pinned C++ | Class |
|---|---:|---|---|
| `build.rs` | 689 | — | O-CS |
| `src/animation.rs` | 3,667 | 41 animation/importer rows | C₄₁ |
| `src/artboard.rs` | 21,444 | `property_recorder.cpp`; `artboard.cpp`; `artboard_component_list.cpp`; `draw_rules.cpp`; `draw_target.cpp`; `nested_artboard.cpp`; `nested_artboard_layout.cpp`; `nested_artboard_leaf.cpp`; `scripted_drawable.cpp`; `scripted_object.cpp`; `scripted_path_effect.cpp`; `solo.cpp`; `text_input.cpp` | M |
| `src/artboard_component_list.rs` | 136 | `src/artboard_component_list.cpp` | D |
| `src/artboard_component_list_order.rs` | 60 | `src/artboard_component_list.cpp` | D |
| `src/artboard_data_bind.rs` | 5 | `src/scripted/scripted_data_converter.cpp` | D |
| `src/artboard_list_map_rule.rs` | 10 | `src/artboard_list_map_rule.cpp` | D |
| `src/artboard_referencer.rs` | 32 | `src/artboard_referencer.cpp` | D |
| `src/assets/file_asset_loader.rs` | 171 | `src/assets/font_asset.cpp` | D |
| `src/assets/font_asset.rs` | 305 | `src/assets/font_asset.cpp` | D |
| `src/assets/image_asset.rs` | 481 | `src/assets/image_asset.cpp` | D |
| `src/bindable_artboard.rs` | 12 | `src/bindable_artboard.cpp` | D |
| `src/components.rs` | 2,609 | six `bones/*`; `component.cpp`; eleven `math/*`; `text_input.cpp` | C₁₉ |
| `src/constraints.rs` | 6,118 | eighteen `constraints/*`; `text_input.cpp` | C₁₉ |
| `src/custom_property_container.rs` | 251 | `src/custom_property_container.cpp` | D |
| `src/data_bind/context/context_target_value.rs` | 10 | `src/data_bind/context/context_target_value.cpp` | D |
| `src/data_bind/context/context_value.rs` | 13,309 | `src/data_bind/context/context_value.cpp` | D |
| `src/data_bind/context/context_value_any.rs` | 12 | `src/data_bind/context/context_value_any.cpp` | D |
| `src/data_bind/context/context_value_artboard.rs` | 11 | `src/data_bind/context/context_value_artboard.cpp` | D |
| `src/data_bind/context/context_value_asset_font.rs` | 7 | `src/data_bind/context/context_value_asset_font.cpp` | D |
| `src/data_bind/context/context_value_asset_image.rs` | 7 | `src/data_bind/context/context_value_asset_image.cpp` | D |
| `src/data_bind/context/context_value_boolean.rs` | 12 | `src/data_bind/context/context_value_boolean.cpp` | D |
| `src/data_bind/context/context_value_color.rs` | 10 | `src/data_bind/context/context_value_color.cpp` | D |
| `src/data_bind/context/context_value_enum.rs` | 19 | `src/data_bind/context/context_value_enum.cpp` | D |
| `src/data_bind/context/context_value_list.rs` | 23 | `src/data_bind/context/context_value_list.cpp` | D |
| `src/data_bind/context/context_value_number.rs` | 10 | `src/data_bind/context/context_value_number.cpp` | D |
| `src/data_bind/context/context_value_string.rs` | 12 | `src/data_bind/context/context_value_string.cpp` | D |
| `src/data_bind/context/context_value_symbol_list_index.rs` | 7 | `src/data_bind/context/context_value_symbol_list_index.cpp` | D |
| `src/data_bind/context/context_value_trigger.rs` | 7 | `src/data_bind/context/context_value_trigger.cpp` | D |
| `src/data_bind/context/context_value_viewmodel.rs` | 12 | `src/data_bind/context/context_value_viewmodel.cpp` | D |
| `src/data_bind/converters/data_converter.rs` | 3,290 | `src/data_bind/converters/data_converter.cpp` | D |
| `src/data_bind/converters/data_converter_boolean_negate.rs` | 5 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_formula.rs` | 12 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_group.rs` | 9 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_group_item.rs` | 22 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_interpolator.rs` | 334 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_list_to_length.rs` | 5 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_number_to_list.rs` | 9 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_operation.rs` | 59 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_operation_value.rs` | 9 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_operation_viewmodel.rs` | 9 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_range_mapper.rs` | 43 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_rounder.rs` | 6 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_string_pad.rs` | 5 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_string_remove_zeros.rs` | 5 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_string_trim.rs` | 5 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_system_degs_to_rads.rs` | 9 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_system_normalizer.rs` | 9 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_to_number.rs` | 5 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_to_string.rs` | 9 | corresponding `.cpp` | D |
| `src/data_bind/converters/data_converter_trigger.rs` | 11 | corresponding `.cpp` | D |
| `src/data_bind/converters/formula/formula_token.rs` | 14 | corresponding `.cpp` | D |
| `src/data_bind/data_bind.rs` | 1,301 | corresponding `.cpp` | D |
| `src/data_bind/data_bind_container.rs` | 389 | corresponding `.cpp` | D |
| `src/data_bind/data_bind_context.rs` | 11,755 | corresponding `.cpp` | D |
| `src/data_bind/data_bind_list_item_consumer.rs` | 46 | corresponding `.cpp` | D |
| `src/data_bind/data_bind_path.rs` | 139 | corresponding `.cpp` | D |
| `src/data_bind/data_bind_viewmodel_consumer.rs` | 7 | corresponding `.cpp` | D |
| `src/data_bind/data_context.rs` | 259 | corresponding `.cpp` | D |
| `src/data_bind_container.rs` | 3 | — | O-CS |
| `src/data_bind_graph.rs` | 5 | `src/scripted/scripted_data_converter.cpp` | D |
| `src/data_bind_path_referencer.rs` | 64 | `src/data_bind_path_referencer.cpp` | D |
| `src/data_converter.rs` | 5 | — | O-CS |
| `src/data_converter_trigger.rs` | 56 | — | O-? |
| `src/draw.rs` | 33,351 | `artboard.cpp`; `artboard_component_list.cpp`; `draw_rules.cpp`; `draw_target.cpp`; `drawable.cpp`; `foreground_layout_drawable.cpp`; `layout/artboard_component_list_override.cpp`; `layout_component.cpp`; three nested-artboard rows; two scripted rows; `shape_paint_container.cpp`; four text rows | M |
| `src/draw_rules.rs` | 43 | `src/draw_rules.cpp` | D |
| `src/draw_target.rs` | 42 | `src/draw_target.cpp` | D |
| `src/event.rs` | 38 | `src/event.cpp` | D |
| `src/focus.rs` | 2,288 | `focus_manager.cpp`; `focus_node.cpp`; `focusable.cpp` | C₃ |
| `src/foreground_layout_drawable.rs` | 16 | corresponding `.cpp` | D |
| `src/hittest_command_path.rs` | 82 | corresponding `.cpp` | D |
| `src/intrinsically_sizeable.rs` | 106 | corresponding `.cpp` | D |
| `src/joystick.rs` | 391 | corresponding `.cpp` | D |
| `src/layout/artboard_component_list_override.rs` | 39 | corresponding `.cpp` | D |
| `src/layout/axis.rs` | 18 | corresponding `.cpp` | D |
| `src/layout/axis_x.rs` | 17 | corresponding `.cpp` | D |
| `src/layout/axis_y.rs` | 17 | corresponding `.cpp` | D |
| `src/layout/layout_component_style.rs` | 252 | corresponding `.cpp` | D |
| `src/layout/layout_node_provider.rs` | 38 | corresponding `.cpp` | D |
| `src/layout/n_sliced_node.rs` | 489 | corresponding `.cpp` | D |
| `src/layout/n_slicer.rs` | 137 | corresponding `.cpp` | D |
| `src/layout/n_slicer_details.rs` | 71 | corresponding `.cpp` | D |
| `src/layout/n_slicer_tile_mode.rs` | 15 | corresponding `.cpp` | D |
| `src/layout.rs` | 55 | `src/layout.cpp` | D |
| `src/layout_component.rs` | 89 | `layout_component_style.cpp`; `layout_node_provider.cpp`; `layout_component.cpp` | C₃ |
| `src/lib.rs` | 647 | 14 miscellaneous core rows | C₁₄ |
| `src/listener_group.rs` | 395 | `text_input_listener_group.cpp`; `listener_group.cpp` | C₂ |
| `src/math/hit_test.rs` | 339 | corresponding `.cpp` | D |
| `src/math/mod.rs` | 2 | — | O-CS |
| `src/math/random/native.rs` | 121 | `src/math/random.cpp` | D |
| `src/math/random/wasm.rs` | 46 | `src/math/random.cpp` | D |
| `src/math/random.rs` | 308 | `src/math/random.cpp` | D |
| `src/nested_artboard.rs` | 257 | corresponding `.cpp` | D |
| `src/nested_artboard_layout.rs` | 24 | corresponding `.cpp` | D |
| `src/nested_artboard_leaf.rs` | 69 | corresponding `.cpp` | D |
| `src/nested_artboard_origin.rs` | 39 | corresponding `.cpp` | D |
| `src/nested_bool.rs` | 38 | — | O-? |
| `src/nested_number.rs` | 23 | — | O-? |
| `src/nested_trigger.rs` | 16 | — | O-? |
| `src/objects.rs` | 996 | — | O-CS |
| `src/parent_traversal.rs` | 110 | corresponding `.cpp` | D |
| `src/project_data_converter.rs` | 2,686 | — | O-SA |
| `src/properties.rs` | 310 | — | O-CS |
| `src/rectangles_to_contour.rs` | 436 | `rectangles_to_contour.cpp`; `text_selection_path.cpp` | C₂ |
| `src/retained_data_bind.rs` | 4 | — | O-CS |
| `src/scene.rs` | 156 | `src/scene.cpp` | D |
| `src/script_asset.rs` | 189 | `src/assets/script_asset.cpp` | D |
| `src/script_input_artboard.rs` | 150 | corresponding `.cpp` | D |
| `src/script_input_boolean.rs` | 17 | corresponding `.cpp` | D |
| `src/script_input_color.rs` | 17 | corresponding `.cpp` | D |
| `src/script_input_number.rs` | 17 | corresponding `.cpp` | D |
| `src/script_input_string.rs` | 22 | corresponding `.cpp` | D |
| `src/script_input_trigger.rs` | 17 | corresponding `.cpp` | D |
| `src/script_input_viewmodel_property.rs` | 202 | corresponding `.cpp` | D |
| `src/scripted_data_converter.rs` | 1,929 | `src/scripted/scripted_data_converter.cpp` | D |
| `src/scripted_object.rs` | 550 | corresponding `.cpp` | D |
| `src/scripting.rs` | 2,321 | five `src/scripted/*` rows | C₅ |
| `src/shapes/clipping_shape.rs` | 12 | corresponding `.cpp` | D |
| `src/shapes/cubic_asymmetric_vertex.rs` | 23 | corresponding `.cpp` | D |
| `src/shapes/cubic_detached_vertex.rs` | 23 | corresponding `.cpp` | D |
| `src/shapes/cubic_mirrored_vertex.rs` | 23 | corresponding `.cpp` | D |
| `src/shapes/cubic_vertex.rs` | 14 | corresponding `.cpp` | D |
| `src/shapes/deformer.rs` | 7 | corresponding `.cpp` | D |
| `src/shapes/ellipse.rs` | 11 | corresponding `.cpp` | D |
| `src/shapes/image.rs` | 874 | corresponding `.cpp` | D |
| `src/shapes/list_path.rs` | 672 | corresponding `.cpp` | D |
| `src/shapes/mesh.rs` | 541 | corresponding `.cpp` | D |
| `src/shapes/mesh_vertex.rs` | 24 | corresponding `.cpp` | D |
| `src/shapes/mod.rs` | 137 | — | O-CS |
| `src/shapes/paint/color.rs` | 105 | corresponding `.cpp` | D |
| `src/shapes/paint/dash.rs` | 62 | corresponding `.cpp` | D |
| `src/shapes/paint/dash_path.rs` | 27 | corresponding `.cpp` | D |
| `src/shapes/paint/effects_container.rs` | 30 | corresponding `.cpp` | D |
| `src/shapes/paint/feather.rs` | 42 | corresponding `.cpp` | D |
| `src/shapes/paint/fill.rs` | 21 | corresponding `.cpp` | D |
| `src/shapes/paint/gradient_stop.rs` | 23 | corresponding `.cpp` | D |
| `src/shapes/paint/group_effect.rs` | 19 | corresponding `.cpp` | D |
| `src/shapes/paint/linear_gradient.rs` | 18 | corresponding `.cpp` | D |
| `src/shapes/paint/mod.rs` | 18 | — | O-CS |
| `src/shapes/paint/radial_gradient.rs` | 8 | corresponding `.cpp` | D |
| `src/shapes/paint/shape_paint.rs` | 12 | corresponding `.cpp` | D |
| `src/shapes/paint/shape_paint_mutator.rs` | 7 | corresponding `.cpp` | D |
| `src/shapes/paint/shape_paint_path.rs` | 23 | corresponding `.cpp` | D |
| `src/shapes/paint/solid_color.rs` | 6 | corresponding `.cpp` | D |
| `src/shapes/paint/stroke.rs` | 26 | corresponding `.cpp` | D |
| `src/shapes/paint/stroke_effect.rs` | 12 | corresponding `.cpp` | D |
| `src/shapes/paint/target_effect.rs` | 10 | corresponding `.cpp` | D |
| `src/shapes/paint/trim_path.rs` | 30 | corresponding `.cpp` | D |
| `src/shapes/parametric_path.rs` | 42 | corresponding `.cpp` | D |
| `src/shapes/path.rs` | 18 | corresponding `.cpp` | D |
| `src/shapes/path_composer.rs` | 12 | corresponding `.cpp` | D |
| `src/shapes/path_vertex.rs` | 13 | corresponding `.cpp` | D |
| `src/shapes/points_common_path.rs` | 16 | corresponding `.cpp` | D |
| `src/shapes/points_path.rs` | 13 | corresponding `.cpp` | D |
| `src/shapes/polygon.rs` | 20 | corresponding `.cpp` | D |
| `src/shapes/rectangle.rs` | 25 | corresponding `.cpp` | D |
| `src/shapes/shape.rs` | 19 | corresponding `.cpp` | D |
| `src/shapes/shape_paint_container.rs` | 187 | corresponding `.cpp` | D |
| `src/shapes/slice_mesh.rs` | 466 | corresponding `.cpp` | D |
| `src/shapes/star.rs` | 16 | corresponding `.cpp` | D |
| `src/shapes/straight_vertex.rs` | 17 | corresponding `.cpp` | D |
| `src/shapes/triangle.rs` | 12 | corresponding `.cpp` | D |
| `src/shapes/vertex.rs` | 16 | corresponding `.cpp` | D |
| `src/solo.rs` | 31 | `src/solo.cpp` | D |
| `src/state_machine/animation_reset_factory.rs` | 257 | `animation_reset.cpp`; `animation_reset_factory.cpp` | C₂ |
| `src/state_machine/bindables.rs` | 2,216 | — | O-? |
| `src/state_machine/blend_state_direct_instance.rs` | 168 | corresponding animation `.cpp` | D |
| `src/state_machine/data_bind_template.rs` | 408 | — | O-? |
| `src/state_machine/data_converter_binding.rs` | 94 | — | O-? |
| `src/state_machine/event_report.rs` | 226 | `src/event.cpp` | D |
| `src/state_machine/focus_action_clear.rs` | 21 | corresponding animation `.cpp` | D |
| `src/state_machine/focus_action_target.rs` | 160 | corresponding animation `.cpp` | D |
| `src/state_machine/focus_action_traversal.rs` | 141 | corresponding animation `.cpp` | D |
| `src/state_machine/focus_listener_group.rs` | 53 | corresponding animation `.cpp` | D |
| `src/state_machine/focused_input_dispatch.rs` | 382 | `src/animation/state_machine_instance.cpp` | D |
| `src/state_machine/gamepad_listener_group.rs` | 182 | corresponding animation `.cpp` | D |
| `src/state_machine/instance.rs` | 3 | — | O-CS |
| `src/state_machine/keyboard_listener_group.rs` | 262 | corresponding animation `.cpp` | D |
| `src/state_machine/layer_state.rs` | 65 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_action.rs` | 1,140 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_action_owner.rs` | 485 | — | O-? |
| `src/state_machine/listener_align_target.rs` | 312 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_bool_change.rs` | 45 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_fire_event.rs` | 34 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_input_change.rs` | 222 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_invocation.rs` | 189 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_number_change.rs` | 48 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_trigger_change.rs` | 48 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_types/gamepad_input.rs` | 83 | `src/inputs/gamepad_input.cpp` | D |
| `src/state_machine/listener_types/keyboard_input.rs` | 45 | `src/inputs/keyboard_input.cpp` | D |
| `src/state_machine/listener_types/listener_input_type.rs` | 66 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_types/listener_input_type_gamepad.rs` | 167 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_types/listener_input_type_keyboard.rs` | 186 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_types/listener_input_type_semantic.rs` | 94 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_types/listener_input_type_viewmodel.rs` | 156 | corresponding animation `.cpp` | D |
| `src/state_machine/listener_types/mod.rs` | 17 | — | O-CS |
| `src/state_machine/listener_types/semantic_input.rs` | 17 | `src/inputs/semantic_input.cpp` | D |
| `src/state_machine/listener_viewmodel_change.rs` | 510 | corresponding animation `.cpp` | D |
| `src/state_machine/nested_state_machine.rs` | 463 | corresponding animation `.cpp` | D |
| `src/state_machine/scripted_listener_action.rs` | 4,278 | corresponding animation `.cpp` | D |
| `src/state_machine/scripted_object_lifecycle.rs` | 127 | — | O-? |
| `src/state_machine/scripted_transition_condition.rs` | 109 | corresponding animation `.cpp` | D |
| `src/state_machine/semantic_listener_group.rs` | 39 | corresponding animation `.cpp` | D |
| `src/state_machine/state_instance.rs` | 313 | corresponding animation `.cpp` | D |
| `src/state_machine/state_machine.rs` | 2,228 | `src/animation/state_machine.cpp` | D |
| `src/state_machine/state_machine_fire_action.rs` | 413 | corresponding animation `.cpp` | D |
| `src/state_machine/state_machine_fire_event.rs` | 48 | — | O-? |
| `src/state_machine/state_machine_fire_trigger.rs` | 75 | corresponding animation `.cpp` | D |
| `src/state_machine/state_machine_input.rs` | 95 | `state_machine_input.cpp`; `audio_asset.cpp`; `state_machine_importer.cpp` | C₃ |
| `src/state_machine/state_machine_input_instance.rs` | 199 | `state_machine_input_instance.cpp`; `audio_asset.cpp`; `state_machine_importer.cpp` | C₃ |
| `src/state_machine/state_machine_instance.rs` | 20,781 | `state_machine_instance.cpp`; `text_input_listener_group.cpp` | M |
| `src/state_machine/state_machine_layer.rs` | 132 | corresponding animation `.cpp` | D |
| `src/state_machine/state_machine_layer_instance.rs` | 1,101 | `src/animation/state_machine_instance.cpp` | D |
| `src/state_machine/state_machine_listener.rs` | 750 | corresponding animation `.cpp` | D |
| `src/state_machine/state_machine_listener_single.rs` | 38 | corresponding animation `.cpp` | D |
| `src/state_machine/state_transition.rs` | 252 | `state_transition.cpp`; `state_transition_importer.cpp` | C₂ |
| `src/state_machine/system_state_instance.rs` | 29 | corresponding animation `.cpp` | D |
| `src/state_machine/text_input_listener_group.rs` | 120 | corresponding animation `.cpp` | D |
| `src/state_machine/transition_bool_condition.rs` | 43 | corresponding animation `.cpp` | D |
| `src/state_machine/transition_comparator.rs` | 13 | corresponding animation `.cpp` | D |
| `src/state_machine/transition_condition.rs` | 151 | corresponding animation `.cpp` | D |
| `src/state_machine/transition_condition_op.rs` | 57 | — | O-? |
| `src/state_machine/transition_duration_binding.rs` | 163 | — | O-? |
| `src/state_machine/transition_focus_condition.rs` | 65 | corresponding animation `.cpp` | D |
| `src/state_machine/transition_input_condition.rs` | 39 | corresponding animation `.cpp` | D |
| `src/state_machine/transition_number_condition.rs` | 45 | corresponding animation `.cpp` | D |
| `src/state_machine/transition_property_comparator.rs` | 53 | corresponding animation `.cpp` | D |
| `src/state_machine/transition_property_viewmodel_comparator.rs` | 83 | corresponding animation `.cpp` | D |
| `src/state_machine/transition_trigger_condition.rs` | 37 | corresponding animation `.cpp` | D |
| `src/state_machine/transition_viewmodel_condition.rs` | 1,944 | `transition_viewmodel_condition.cpp`; corresponding importer | C₂ |
| `src/state_machine.rs` | 188 | — | O-CS |
| `src/static_scene.rs` | 67 | corresponding `.cpp` | D |
| `src/text/cursor.rs` | 99 | corresponding `.cpp` | D |
| `src/text/font_hb.rs` | 427 | `font_hb.cpp`; `raw_text.cpp` | C₂ |
| `src/text/fully_shaped_text.rs` | 563 | corresponding `.cpp` | D |
| `src/text/glyph_lookup.rs` | 19 | corresponding `.cpp` | D |
| `src/text/line_breaker.rs` | 107 | corresponding `.cpp` | D |
| `src/text/raw_text.rs` | 1,032 | corresponding `.cpp` | D |
| `src/text/raw_text_input.rs` | 757 | corresponding `.cpp` | D |
| `src/text/text.rs` | 120 | corresponding `.cpp` | D |
| `src/text/text_engine.rs` | 523 | `raw_text.cpp`; `text_engine.cpp` | C₂ |
| `src/text/text_follow_path_modifier.rs` | 246 | corresponding `.cpp` | D |
| `src/text/text_input_cursor.rs` | 27 | corresponding `.cpp` | D |
| `src/text/text_input_drawable.rs` | 30 | corresponding `.cpp` | D |
| `src/text/text_input_selected_text.rs` | 18 | corresponding `.cpp` | D |
| `src/text/text_input_selection.rs` | 10 | corresponding `.cpp` | D |
| `src/text/text_input_text.rs` | 3 | corresponding `.cpp` | D |
| `src/text/text_interface.rs` | 18 | corresponding `.cpp` | D |
| `src/text/text_modifier.rs` | 30 | corresponding `.cpp` | D |
| `src/text/text_modifier_group.rs` | 494 | corresponding `.cpp` | D |
| `src/text/text_modifier_range.rs` | 477 | corresponding `.cpp` | D |
| `src/text/text_selection_path.rs` | 196 | corresponding `.cpp` | D |
| `src/text/text_style.rs` | 40 | `text_interface.cpp`; `text_style.cpp` | C₂ |
| `src/text/text_style_axis.rs` | 39 | corresponding `.cpp` | D |
| `src/text/text_style_feature.rs` | 47 | corresponding `.cpp` | D |
| `src/text/text_style_paint.rs` | 15 | corresponding `.cpp` | D |
| `src/text/text_target_modifier.rs` | 36 | corresponding `.cpp` | D |
| `src/text/text_value_run.rs` | 60 | corresponding `.cpp` | D |
| `src/text/text_variation_helper.rs` | 84 | corresponding `.cpp` | D |
| `src/text/text_variation_modifier.rs` | 76 | corresponding `.cpp` | D |
| `src/text/utf.rs` | 13 | corresponding `.cpp` | D |
| `src/text.rs` | 6,045 | 13 `src/text/*` rows | C₁₃ |
| `src/text_input.rs` | 935 | `src/text/text_input.cpp` | D |
| `src/view_model.rs` | 4,215 | — | O-? |
| `src/view_model_cell.rs` | 2,115 | — | O-? |
| `src/viewmodel/data_enum.rs` | 67 | corresponding `.cpp` | D |
| `src/viewmodel/data_enum_value.rs` | 10 | corresponding `.cpp` | D |
| `src/viewmodel/mod.rs` | 33 | — | O-CS |
| `src/viewmodel/property_symbol_dependent.rs` | 80 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/mod.rs` | 23 | — | O-CS |
| `src/viewmodel/runtime/viewmodel_instance_artboard_runtime.rs` | 74 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_asset_font_runtime.rs` | 31 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_asset_image_runtime.rs` | 61 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_boolean_runtime.rs` | 40 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_color_runtime.rs` | 64 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_enum_runtime.rs` | 116 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_list_index_runtime.rs` | 157 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_list_runtime.rs` | 182 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_number_runtime.rs` | 40 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_runtime.rs` | 603 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_string_runtime.rs` | 44 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_trigger_runtime.rs` | 31 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_instance_value_runtime.rs` | 115 | corresponding `.cpp` | D |
| `src/viewmodel/runtime/viewmodel_runtime.rs` | 226 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel.rs` | 353 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance.rs` | 5,693 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_artboard.rs` | 144 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_asset.rs` | 10 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_asset_font.rs` | 117 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_asset_image.rs` | 140 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_boolean.rs` | 94 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_color.rs` | 93 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_enum.rs` | 43 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_list.rs` | 504 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_list_item.rs` | 65 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_number.rs` | 109 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_string.rs` | 114 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_symbol_list_index.rs` | 124 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_trigger.rs` | 98 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_value.rs` | 266 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_instance_viewmodel.rs` | 2,031 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_property.rs` | 718 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_property_enum.rs` | 50 | corresponding `.cpp` | D |
| `src/viewmodel/viewmodel_property_enum_system.rs` | 6 | corresponding `.cpp` | D |
| `src/work_pool.rs` | 432 | `src/async/work_pool.cpp` | D |
| `tests/cpp_probe.rs` | 91,234 | — | O-TI |
| `tests/f14_helpers.rs` | 96 | — | O-TI |
| `tests/project_data_converter.rs` | 1,537 | — | O-TI |
| `tests/public_api_fl_c5.rs` | 987 | — | O-TI |
| `tests/script_input_bindings.rs` | 508 | — | O-TI |
| `tests/work_pool.rs` | 475 | — | O-TI |
| `tests/work_task.rs` | 96 | — | O-TI |

“Corresponding `.cpp`” means the identical path below pinned `src/`, replacing `.rs` with `.cpp`; state-machine rows explicitly say “animation” where the C++ directory differs.

</details>

<details>
<summary><strong>crates/nuxie — 20 files, 81,302 LOC</strong></summary>

| Rust file | LOC | C++ | Class |
|---|---:|---|---|
| `build.rs` | 4,419 | — | O-CS |
| `src/flow_session.rs` | 7,207 | — | O-FA |
| `src/lib.rs` | 6,812 | `src/assets/script_asset.cpp`; `src/lua/logging_scripting_context.cpp`; `src/scripted/scripted_data_converter.cpp` | M |
| `src/raw_text.rs` | 158 | `src/text/raw_text.cpp` | D |
| `src/scene.rs` | 36,732 | — | O-SA |
| `src/script_import.rs` | 211 | — | O-FA |
| `src/scripted_listener_action_lifecycle_tests.rs` | 4,938 | — | O-TI |
| `tests/data_converter_authoring.rs` | 721 | — | O-TI |
| `tests/empty_text_binding.rs` | 320 | — | O-TI |
| `tests/flow_session_contract.rs` | 335 | — | O-TI |
| `tests/imported_gpu_canvas.rs` | 725 | — | O-TI |
| `tests/layout_observation.rs` | 229 | — | O-TI |
| `tests/machine_observation.rs` | 243 | — | O-TI |
| `tests/public_api.rs` | 875 | — | O-TI |
| `tests/raw_text.rs` | 160 | — | O-TI |
| `tests/raw_text_differential.rs` | 540 | — | O-TI |
| `tests/scene_authoring.rs` | 14,874 | — | O-TI |
| `tests/transform_differential.rs` | 568 | — | O-TI |
| `tests/vector_scripted_drawable.rs` | 1,134 | — | O-TI |
| `tests/vm_hot_path_allocations.rs` | 101 | — | O-TI |

</details>

<details>
<summary><strong>crates/nuxie-scripting — 40 files, 20,916 LOC</strong></summary>

| Rust file | LOC | C++ | Class |
|---|---:|---|---|
| `src/envelope.rs` | 123 | — | O-FA |
| `src/gpu_canvas.rs` | 1,331 | — | O-RR |
| `src/lib.rs` | 22 | — | O-FA |
| `src/shader_asset.rs` | 640 | `src/assets/shader_asset.cpp` | D |
| `src/vm/buffer_ext.rs` | 450 | `src/lua/lua_buffer_ext.cpp` | D |
| `src/vm/bytecode.rs` | 602 | — | O-FA |
| `src/vm/command_server.rs` | 74 | `src/command_server.cpp` | D |
| `src/vm/host_commands.rs` | 637 | — | O-FA |
| `src/vm/listener_invocation.rs` | 863 | `lua_listener_invocation.cpp`; `math/lua_input.cpp` | C₂ |
| `src/vm/logging_scripting_context.rs` | 80 | corresponding `.cpp` | D |
| `src/vm/lua_artboards.rs` | 360 | corresponding `.cpp` | D |
| `src/vm/lua_color.rs` | 264 | `src/lua/math/lua_color.cpp` | D |
| `src/vm/lua_mat2d.rs` | 179 | corresponding math `.cpp` | D |
| `src/vm/lua_mat4.rs` | 758 | corresponding math `.cpp` | D |
| `src/vm/lua_math.rs` | 20 | corresponding math `.cpp` | D |
| `src/vm/lua_paint.rs` | 375 | corresponding renderer `.cpp` | D |
| `src/vm/lua_path.rs` | 436 | corresponding renderer `.cpp` | D |
| `src/vm/lua_renderer.rs` | 170 | corresponding renderer `.cpp` | D |
| `src/vm/lua_renderer_library.rs` | 46 | corresponding renderer `.cpp` | D |
| `src/vm/lua_rive_base.rs` | 24 | corresponding `.cpp` | D |
| `src/vm/lua_vec2d.rs` | 264 | corresponding math `.cpp` | D |
| `src/vm/promise.rs` | 544 | `src/lua/lua_promise.cpp` | D |
| `src/vm/renderer.rs` | 75 | `src/lua/renderer/lua_gradient.cpp` | D |
| `src/vm/resource_limits.rs` | 218 | — | O-FA |
| `src/vm/view_model.rs` | 1,732 | `lua_data_context.cpp`; `lua_state.cpp` | C₂ |
| `src/vm.rs` | 3,571 | `script_asset.cpp`; `lua_data_value.cpp`; `lua_promise.cpp`; `lua_properties.cpp`; `rive_lua_libs.cpp` | C₅ |
| `tests/buffer_extensions.rs` | 665 | — | O-TI |
| `tests/corpus_scripts.rs` | 390 | — | O-TI |
| `tests/gpu_canvas_tools.rs` | 328 | — | O-TI |
| `tests/host_logging.rs` | 183 | — | O-TI |
| `tests/library_scope.rs` | 197 | — | O-TI |
| `tests/listener_invocations.rs` | 1,052 | — | O-TI |
| `tests/mat4_bindings.rs` | 473 | — | O-TI |
| `tests/nuxie_host_commands.rs` | 1,274 | — | O-TI |
| `tests/path_bindings.rs` | 112 | — | O-TI |
| `tests/path_render_path_lifetime.rs` | 50 | — | O-TI |
| `tests/promise_scenarios.rs` | 787 | — | O-TI |
| `tests/renderer_bindings.rs` | 132 | — | O-TI |
| `tests/shader_asset_resolution.rs` | 959 | — | O-TI |
| `tests/vm_boot.rs` | 456 | — | O-TI |

</details>

<details>
<summary><strong>crates/nuxie-binary — 48 files, 45,212 LOC</strong></summary>

| Rust file | LOC | C++ | Class |
|---|---:|---|---|
| `src/assets/blob_asset.rs` | 1 | `src/assets/blob_asset.cpp` | D |
| `src/assets/file_asset.rs` | 184 | corresponding `.cpp` | D |
| `src/assets/file_asset_contents.rs` | 196 | `blob_asset.cpp`; `file_asset_contents.cpp` | C₂ |
| `src/assets/file_asset_referencer.rs` | 50 | corresponding `.cpp` | D |
| `src/assets/manifest_asset.rs` | 282 | corresponding `.cpp` | D |
| `src/assets/mod.rs` | 15 | — | O-CS |
| `src/assets/shader_asset.rs` | 1 | corresponding `.cpp` | D |
| `src/bin/riv-inspect.rs` | 42 | — | O-TI |
| `src/binary_data_reader.rs` | 162 | `src/core/binary_data_reader.cpp` | D |
| `src/binary_writer.rs` | 115 | `src/core/binary_writer.cpp` | D |
| `src/core/binary_reader.rs` | 77 | corresponding `.cpp` | D |
| `src/core/field_types/core_bool_type.rs` | 6 | corresponding `.cpp` | D |
| `src/core/field_types/core_bytes_type.rs` | 7 | corresponding `.cpp` | D |
| `src/core/field_types/core_color_type.rs` | 6 | corresponding `.cpp` | D |
| `src/core/field_types/core_double_type.rs` | 6 | corresponding `.cpp` | D |
| `src/core/field_types/core_string_type.rs` | 6 | corresponding `.cpp` | D |
| `src/core/field_types/core_uint64_type.rs` | 6 | corresponding `.cpp` | D |
| `src/core/field_types/core_uint_type.rs` | 14 | corresponding `.cpp` | D |
| `src/core/field_types/mod.rs` | 50 | — | O-CS |
| `src/core/mod.rs` | 2 | — | O-CS |
| `src/importers/artboard_importer.rs` | 185 | corresponding `.cpp` | D |
| `src/importers/backboard_importer.rs` | 43 | corresponding `.cpp` | D |
| `src/importers/bindable_property_importer.rs` | 77 | corresponding `.cpp` | D |
| `src/importers/data_bind_path_importer.rs` | 65 | corresponding `.cpp` | D |
| `src/importers/data_converter_formula_importer.rs` | 186 | corresponding `.cpp` | D |
| `src/importers/data_converter_group_importer.rs` | 84 | corresponding `.cpp` | D |
| `src/importers/enum_importer.rs` | 55 | corresponding `.cpp` | D |
| `src/importers/file_asset_importer.rs` | 24 | corresponding `.cpp` | D |
| `src/importers/keyed_object_importer.rs` | 20 | corresponding `.cpp` | D |
| `src/importers/layer_state_importer.rs` | 23 | corresponding `.cpp` | D |
| `src/importers/linear_animation_importer.rs` | 125 | corresponding `.cpp` | D |
| `src/importers/listener_input_type_gamepad_importer.rs` | 18 | corresponding `.cpp` | D |
| `src/importers/listener_input_type_keyboard_importer.rs` | 18 | corresponding `.cpp` | D |
| `src/importers/listener_input_type_semantic_importer.rs` | 18 | corresponding `.cpp` | D |
| `src/importers/mod.rs` | 507 | 22 importer rows | C₂₂ |
| `src/importers/scripted_object_importer.rs` | 71 | corresponding `.cpp` | D |
| `src/importers/state_machine_layer_component_importer.rs` | 19 | corresponding `.cpp` | D |
| `src/importers/state_machine_layer_importer.rs` | 741 | corresponding `.cpp` | D |
| `src/importers/state_machine_listener_importer.rs` | 22 | corresponding `.cpp` | D |
| `src/importers/text_asset_importer.rs` | 9 | corresponding `.cpp` | D |
| `src/importers/viewmodel_importer.rs` | 136 | corresponding `.cpp` | D |
| `src/importers/viewmodel_instance_importer.rs` | 33 | corresponding `.cpp` | D |
| `src/importers/viewmodel_instance_list_importer.rs` | 16 | corresponding `.cpp` | D |
| `src/lib.rs` | 12,911 | `audio_asset.cpp`; `state_machine_importer.cpp`; `state_transition_importer.cpp`; `transition_viewmodel_condition_importer.cpp` | M |
| `tests/authoring_records.rs` | 532 | — | O-TI |
| `tests/cpp_import.rs` | 19,275 | — | O-TI |
| `tests/f14_helpers.rs` | 153 | — | O-TI |
| `tests/fixtures.rs` | 8,618 | — | O-TI |

</details>

<details>
<summary><strong>crates/nuxie-render-api — 3 files, 4,500 LOC</strong></summary>

| Rust file | LOC | C++ | Class |
|---|---:|---|---|
| `src/lib.rs` | 3,474 | — | O-RR |
| `src/serializing.rs` | 671 | — | O-RR |
| `tests/canonical_recording.rs` | 355 | — | O-TI |

The render API source itself documents translation from the upstream renderer/factory headers while remaining absent from this `.cpp`-only manifest, so its zero attribution is a manifest-scope consequence rather than unexplained runtime behavior. [render API provenance](/Users/levi/dev/worktrees/nuxie-fld1/crates/nuxie-render-api/src/lib.rs:1)

</details>