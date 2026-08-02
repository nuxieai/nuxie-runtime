# MR-1 per-hotspot move plan

Phase 5 step 1 scout deliverable for branch `levi/mr1-move-plan`. This is a move plan only: no Rust source, manifest row, generated surface, or fixture is changed here.

## Scope and snapshot

- Parsed `file-correspondence-manifest.toml` at upstream ref `d788e8ec6e8b598526607d6a1e8818e8b637b60c`: 448 rows.
- Current manifest has 164 rows listing 2+ non-empty Rust files. The exhaustive union with rows touching a shared Rust file is 266 upstream rows.
- There are 48 real Rust files referenced by 2+ rows (396 hotspot-to-row references). Empty `rust_module` values are not files and are excluded.
- Classification: 191 actionable rows and 75 justified exceptions.
- The Phase 5 prose records an older baseline (57/42/41/28/22/19/19). The current manifest parses as 57/44/42/29/22/19/19 for the same seven roots; this plan uses the current manifest, so the added artboard/animation/draw rows are included.

## Decision rules

1. If one current Rust module has the same basename as the upstream C++ row, that module is the canonical target even when its Rust directory reflects an established semantic grouping (for example, animation listener code under `state_machine/`).
2. Otherwise mirror the path below upstream `src/` beneath the owning crate's `src/`. Preserve the established adaptations `src/lua/** -> crates/nuxie-scripting/src/vm/**` and `src/audio/** -> crates/nuxie-audio/src/**`.
3. `pure-move` means an existing basename-matching owner absorbs fragments from aggregate/glue modules without changing item bodies. `split-needed` means the aggregate must be partitioned into a new canonical file before the move can be mechanical. `exception` means the manifest legitimately remains multi-file; its one-line note is listed below.
4. Public re-exports and `mod` declarations remain in crate roots as Rust module plumbing; correspondence attaches implementation bodies to the canonical target, not to root wiring.
5. Every landing preserves `rust_ref`, ticket tag, `b6_row_id`, `audit_record`, and B6 verdict. Manifest `rust_module` updates land in the same pure-move commit.

## Collision check

- 191 actionable rows produce 191 unique target paths.
- 84 targets already exist and are already referenced by their owning row; 107 are new.
- Duplicate proposed targets: 0. Existing-but-not-owned target collisions: 0.
- Therefore every actionable target is collision-free against the current module tree. Re-run this exact check immediately before each landing because Phase 5 executes after a write freeze.

## Primary hotspot tables

Rows can appear in more than one table when the current manifest scatters one upstream row across multiple hotspots. The target and move-kind are row-global and therefore identical in every occurrence.

### `crates/nuxie-binary/src/lib.rs` (57 rows)

| Upstream row | Target module | Move-kind |
|---|---|---|
| `B6-0172` · `src/data_bind/converters/data_converter.cpp` | — (see E-B6-0172) | exception |
| `B6-0173` · `src/data_bind/converters/data_converter_boolean_negate.cpp` | — (see E-B6-0173) | exception |
| `B6-0174` · `src/data_bind/converters/data_converter_formula.cpp` | — (see E-B6-0174) | exception |
| `B6-0175` · `src/data_bind/converters/data_converter_group.cpp` | — (see E-B6-0175) | exception |
| `B6-0176` · `src/data_bind/converters/data_converter_group_item.cpp` | — (see E-B6-0176) | exception |
| `B6-0177` · `src/data_bind/converters/data_converter_interpolator.cpp` | — (see E-B6-0177) | exception |
| `B6-0178` · `src/data_bind/converters/data_converter_list_to_length.cpp` | — (see E-B6-0178) | exception |
| `B6-0179` · `src/data_bind/converters/data_converter_number_to_list.cpp` | — (see E-B6-0179) | exception |
| `B6-0180` · `src/data_bind/converters/data_converter_operation.cpp` | — (see E-B6-0180) | exception |
| `B6-0181` · `src/data_bind/converters/data_converter_operation_value.cpp` | — (see E-B6-0181) | exception |
| `B6-0182` · `src/data_bind/converters/data_converter_operation_viewmodel.cpp` | — (see E-B6-0182) | exception |
| `B6-0183` · `src/data_bind/converters/data_converter_range_mapper.cpp` | — (see E-B6-0183) | exception |
| `B6-0184` · `src/data_bind/converters/data_converter_rounder.cpp` | — (see E-B6-0184) | exception |
| `B6-0185` · `src/data_bind/converters/data_converter_string_pad.cpp` | — (see E-B6-0185) | exception |
| `B6-0186` · `src/data_bind/converters/data_converter_string_remove_zeros.cpp` | — (see E-B6-0186) | exception |
| `B6-0187` · `src/data_bind/converters/data_converter_string_trim.cpp` | — (see E-B6-0187) | exception |
| `B6-0188` · `src/data_bind/converters/data_converter_system_degs_to_rads.cpp` | — (see E-B6-0188) | exception |
| `B6-0189` · `src/data_bind/converters/data_converter_system_normalizer.cpp` | — (see E-B6-0189) | exception |
| `B6-0190` · `src/data_bind/converters/data_converter_to_number.cpp` | — (see E-B6-0190) | exception |
| `B6-0191` · `src/data_bind/converters/data_converter_to_string.cpp` | — (see E-B6-0191) | exception |
| `B6-0192` · `src/data_bind/converters/data_converter_trigger.cpp` | — (see E-B6-0192) | exception |
| `B6-0193` · `src/data_bind/converters/formula/formula_token.cpp` | — (see E-B6-0193) | exception |
| `B6-0194` · `src/data_bind/data_bind.cpp` | — (see E-B6-0194) | exception |
| `B6-0195` · `src/data_bind/data_bind_container.cpp` | — (see E-B6-0195) | exception |
| `B6-0196` · `src/data_bind/data_bind_context.cpp` | — (see E-B6-0196) | exception |
| `B6-0200` · `src/data_bind/data_context.cpp` | — (see E-B6-0200) | exception |
| `B6-0208` · `src/file.cpp` | — (see E-B6-0208) | exception |
| `B6-0213` · `src/importers/backboard_importer.cpp` | — (see E-B6-0213) | exception |
| `B6-0228` · `src/importers/state_machine_importer.cpp` | — (see E-B6-0228) | exception |
| `B6-0232` · `src/importers/state_transition_importer.cpp` | — (see E-B6-0232) | exception |
| `B6-0234` · `src/importers/transition_viewmodel_condition_importer.cpp` | — (see E-B6-0234) | exception |
| `B6-0235` · `src/importers/viewmodel_importer.cpp` | `crates/nuxie-binary/src/importers/viewmodel_importer.rs` | pure-move |
| `B6-0236` · `src/importers/viewmodel_instance_importer.cpp` | `crates/nuxie-binary/src/importers/viewmodel_instance_importer.rs` | pure-move |
| `B6-0237` · `src/importers/viewmodel_instance_list_importer.cpp` | `crates/nuxie-binary/src/importers/viewmodel_instance_list_importer.rs` | pure-move |
| `B6-0409` · `src/viewmodel/data_enum.cpp` | — (see E-B6-0409) | exception |
| `B6-0410` · `src/viewmodel/data_enum_value.cpp` | — (see E-B6-0410) | exception |
| `B6-0411` · `src/viewmodel/property_symbol_dependent.cpp` | — (see E-B6-0411) | exception |
| `B6-0426` · `src/viewmodel/viewmodel.cpp` | — (see E-B6-0426) | exception |
| `B6-0427` · `src/viewmodel/viewmodel_instance.cpp` | — (see E-B6-0427) | exception |
| `B6-0428` · `src/viewmodel/viewmodel_instance_artboard.cpp` | — (see E-B6-0428) | exception |
| `B6-0429` · `src/viewmodel/viewmodel_instance_asset.cpp` | — (see E-B6-0429) | exception |
| `B6-0430` · `src/viewmodel/viewmodel_instance_asset_font.cpp` | — (see E-B6-0430) | exception |
| `B6-0431` · `src/viewmodel/viewmodel_instance_asset_image.cpp` | — (see E-B6-0431) | exception |
| `B6-0432` · `src/viewmodel/viewmodel_instance_boolean.cpp` | — (see E-B6-0432) | exception |
| `B6-0433` · `src/viewmodel/viewmodel_instance_color.cpp` | — (see E-B6-0433) | exception |
| `B6-0434` · `src/viewmodel/viewmodel_instance_enum.cpp` | — (see E-B6-0434) | exception |
| `B6-0435` · `src/viewmodel/viewmodel_instance_list.cpp` | — (see E-B6-0435) | exception |
| `B6-0436` · `src/viewmodel/viewmodel_instance_list_item.cpp` | — (see E-B6-0436) | exception |
| `B6-0437` · `src/viewmodel/viewmodel_instance_number.cpp` | — (see E-B6-0437) | exception |
| `B6-0438` · `src/viewmodel/viewmodel_instance_string.cpp` | — (see E-B6-0438) | exception |
| `B6-0439` · `src/viewmodel/viewmodel_instance_symbol_list_index.cpp` | — (see E-B6-0439) | exception |
| `B6-0440` · `src/viewmodel/viewmodel_instance_trigger.cpp` | — (see E-B6-0440) | exception |
| `B6-0441` · `src/viewmodel/viewmodel_instance_value.cpp` | — (see E-B6-0441) | exception |
| `B6-0442` · `src/viewmodel/viewmodel_instance_viewmodel.cpp` | — (see E-B6-0442) | exception |
| `B6-0443` · `src/viewmodel/viewmodel_property.cpp` | — (see E-B6-0443) | exception |
| `B6-0444` · `src/viewmodel/viewmodel_property_enum.cpp` | — (see E-B6-0444) | exception |
| `B6-0445` · `src/viewmodel/viewmodel_property_enum_system.cpp` | — (see E-B6-0445) | exception |

### `crates/nuxie-runtime/src/artboard.rs` (44 rows)

| Upstream row | Target module | Move-kind |
|---|---|---|
| `B6-0001` · `src/advancing_component.cpp` | `crates/nuxie-runtime/src/advancing_component.rs` | split-needed |
| `B6-0067` · `src/animation/property_recorder.cpp` | `crates/nuxie-runtime/src/animation/property_recorder.rs` | split-needed |
| `B6-0077` · `src/animation/state_machine_instance.cpp` | — (see E-B6-0077) | exception |
| `B6-0094` · `src/artboard.cpp` | — (see E-B6-0094) | exception |
| `B6-0095` · `src/artboard_component_list.cpp` | `crates/nuxie-runtime/src/artboard_component_list.rs` | pure-move |
| `B6-0113` · `src/audio_event.cpp` | — (see E-B6-0113) | exception |
| `B6-0117` · `src/bones/skin.cpp` | `crates/nuxie-runtime/src/bones/skin.rs` | split-needed |
| `B6-0120` · `src/bones/weight.cpp` | `crates/nuxie-runtime/src/bones/weight.rs` | split-needed |
| `B6-0123` · `src/component.cpp` | `crates/nuxie-runtime/src/component.rs` | split-needed |
| `B6-0125` · `src/constraints/constraint.cpp` | `crates/nuxie-runtime/src/constraints/constraint.rs` | split-needed |
| `B6-0129` · `src/constraints/ik_constraint.cpp` | `crates/nuxie-runtime/src/constraints/ik_constraint.rs` | split-needed |
| `B6-0131` · `src/constraints/list_follow_path_constraint.cpp` | `crates/nuxie-runtime/src/constraints/list_follow_path_constraint.rs` | split-needed |
| `B6-0142` · `src/constraints/targeted_constraint.cpp` | `crates/nuxie-runtime/src/constraints/targeted_constraint.rs` | split-needed |
| `B6-0194` · `src/data_bind/data_bind.cpp` | — (see E-B6-0194) | exception |
| `B6-0200` · `src/data_bind/data_context.cpp` | — (see E-B6-0200) | exception |
| `B6-0202` · `src/dependency_sorter.cpp` | `crates/nuxie-runtime/src/dependency_sorter.rs` | split-needed |
| `B6-0203` · `src/draw_rules.cpp` | `crates/nuxie-runtime/src/draw_rules.rs` | pure-move |
| `B6-0204` · `src/draw_target.cpp` | `crates/nuxie-runtime/src/draw_target.rs` | pure-move |
| `B6-0205` · `src/drawable.cpp` | `crates/nuxie-runtime/src/drawable.rs` | split-needed |
| `B6-0258` · `src/layout_component.cpp` | `crates/nuxie-runtime/src/layout_component.rs` | pure-move |
| `B6-0303` · `src/nested_artboard.cpp` | `crates/nuxie-runtime/src/nested_artboard.rs` | pure-move |
| `B6-0304` · `src/nested_artboard_layout.cpp` | `crates/nuxie-runtime/src/nested_artboard_layout.rs` | pure-move |
| `B6-0305` · `src/nested_artboard_leaf.cpp` | `crates/nuxie-runtime/src/nested_artboard_leaf.rs` | pure-move |
| `B6-0310` · `src/profiler/rive_profile.cpp` | `crates/nuxie-runtime/src/profiler/rive_profile.rs` | split-needed |
| `B6-0322` · `src/scripted/scripted_drawable.cpp` | — (see E-B6-0322) | exception |
| `B6-0324` · `src/scripted/scripted_layout.cpp` | — (see E-B6-0324) | exception |
| `B6-0325` · `src/scripted/scripted_object.cpp` | — (see E-B6-0325) | exception |
| `B6-0326` · `src/scripted/scripted_path_effect.cpp` | `crates/nuxie-runtime/src/scripted/scripted_path_effect.rs` | split-needed |
| `B6-0331` · `src/shapes/clipping_shape.cpp` | `crates/nuxie-runtime/src/shapes/clipping_shape.rs` | pure-move |
| `B6-0340` · `src/shapes/mesh.cpp` | `crates/nuxie-runtime/src/shapes/mesh.rs` | pure-move |
| `B6-0352` · `src/shapes/paint/shape_paint.cpp` | `crates/nuxie-runtime/src/shapes/paint/shape_paint.rs` | pure-move |
| `B6-0355` · `src/shapes/paint/solid_color.cpp` | `crates/nuxie-runtime/src/shapes/paint/solid_color.rs` | pure-move |
| `B6-0357` · `src/shapes/paint/stroke_effect.cpp` | `crates/nuxie-runtime/src/shapes/paint/stroke_effect.rs` | pure-move |
| `B6-0361` · `src/shapes/path.cpp` | `crates/nuxie-runtime/src/shapes/path.rs` | pure-move |
| `B6-0362` · `src/shapes/path_composer.cpp` | `crates/nuxie-runtime/src/shapes/path_composer.rs` | pure-move |
| `B6-0365` · `src/shapes/points_path.cpp` | `crates/nuxie-runtime/src/shapes/points_path.rs` | pure-move |
| `B6-0368` · `src/shapes/shape.cpp` | `crates/nuxie-runtime/src/shapes/shape.rs` | pure-move |
| `B6-0376` · `src/solo.cpp` | `crates/nuxie-runtime/src/solo.rs` | pure-move |
| `B6-0385` · `src/text/text.cpp` | `crates/nuxie-runtime/src/text.rs` | pure-move |
| `B6-0388` · `src/text/text_input.cpp` | `crates/nuxie-runtime/src/text_input.rs` | pure-move |
| `B6-0399` · `src/text/text_style.cpp` | `crates/nuxie-runtime/src/text/text_style.rs` | pure-move |
| `B6-0404` · `src/text/text_value_run.cpp` | `crates/nuxie-runtime/src/text/text_value_run.rs` | pure-move |
| `B6-0405` · `src/text/text_variation_helper.cpp` | `crates/nuxie-runtime/src/text/text_variation_helper.rs` | pure-move |
| `B6-0408` · `src/transform_component.cpp` | `crates/nuxie-runtime/src/transform_component.rs` | split-needed |

### `crates/nuxie-runtime/src/animation.rs` (42 rows)

| Upstream row | Target module | Move-kind |
|---|---|---|
| `B6-0004` · `src/animation/animation_state.cpp` | `crates/nuxie-runtime/src/animation/animation_state.rs` | split-needed |
| `B6-0005` · `src/animation/animation_state_instance.cpp` | `crates/nuxie-runtime/src/animation/animation_state_instance.rs` | split-needed |
| `B6-0006` · `src/animation/blend_animation.cpp` | `crates/nuxie-runtime/src/animation/blend_animation.rs` | split-needed |
| `B6-0007` · `src/animation/blend_animation_1d.cpp` | `crates/nuxie-runtime/src/animation/blend_animation_1d.rs` | split-needed |
| `B6-0008` · `src/animation/blend_animation_direct.cpp` | `crates/nuxie-runtime/src/animation/blend_animation_direct.rs` | split-needed |
| `B6-0009` · `src/animation/blend_state.cpp` | `crates/nuxie-runtime/src/animation/blend_state.rs` | split-needed |
| `B6-0010` · `src/animation/blend_state_1d.cpp` | `crates/nuxie-runtime/src/animation/blend_state_1d.rs` | split-needed |
| `B6-0011` · `src/animation/blend_state_1d_input.cpp` | `crates/nuxie-runtime/src/animation/blend_state_1d_input.rs` | split-needed |
| `B6-0012` · `src/animation/blend_state_1d_instance.cpp` | `crates/nuxie-runtime/src/animation/blend_state_1d_instance.rs` | split-needed |
| `B6-0013` · `src/animation/blend_state_1d_viewmodel.cpp` | `crates/nuxie-runtime/src/animation/blend_state_1d_viewmodel.rs` | split-needed |
| `B6-0014` · `src/animation/blend_state_direct.cpp` | `crates/nuxie-runtime/src/animation/blend_state_direct.rs` | split-needed |
| `B6-0016` · `src/animation/blend_state_transition.cpp` | `crates/nuxie-runtime/src/animation/blend_state_transition.rs` | split-needed |
| `B6-0017` · `src/animation/cubic_ease_interpolator.cpp` | `crates/nuxie-runtime/src/animation/cubic_ease_interpolator.rs` | split-needed |
| `B6-0018` · `src/animation/cubic_interpolator.cpp` | `crates/nuxie-runtime/src/animation/cubic_interpolator.rs` | split-needed |
| `B6-0019` · `src/animation/cubic_interpolator_component.cpp` | `crates/nuxie-runtime/src/animation/cubic_interpolator_component.rs` | split-needed |
| `B6-0020` · `src/animation/cubic_interpolator_solver.cpp` | `crates/nuxie-runtime/src/animation/cubic_interpolator_solver.rs` | split-needed |
| `B6-0021` · `src/animation/cubic_value_interpolator.cpp` | `crates/nuxie-runtime/src/animation/cubic_value_interpolator.rs` | split-needed |
| `B6-0022` · `src/animation/elastic_ease.cpp` | `crates/nuxie-runtime/src/animation/elastic_ease.rs` | split-needed |
| `B6-0023` · `src/animation/elastic_interpolator.cpp` | `crates/nuxie-runtime/src/animation/elastic_interpolator.rs` | split-needed |
| `B6-0029` · `src/animation/interpolating_keyframe.cpp` | `crates/nuxie-runtime/src/animation/interpolating_keyframe.rs` | split-needed |
| `B6-0031` · `src/animation/keyed_object.cpp` | `crates/nuxie-runtime/src/animation/keyed_object.rs` | split-needed |
| `B6-0032` · `src/animation/keyed_property.cpp` | `crates/nuxie-runtime/src/animation/keyed_property.rs` | split-needed |
| `B6-0033` · `src/animation/keyframe.cpp` | `crates/nuxie-runtime/src/animation/keyframe.rs` | split-needed |
| `B6-0034` · `src/animation/keyframe_bool.cpp` | `crates/nuxie-runtime/src/animation/keyframe_bool.rs` | split-needed |
| `B6-0035` · `src/animation/keyframe_callback.cpp` | `crates/nuxie-runtime/src/animation/keyframe_callback.rs` | split-needed |
| `B6-0036` · `src/animation/keyframe_color.cpp` | `crates/nuxie-runtime/src/animation/keyframe_color.rs` | split-needed |
| `B6-0037` · `src/animation/keyframe_double.cpp` | `crates/nuxie-runtime/src/animation/keyframe_double.rs` | split-needed |
| `B6-0038` · `src/animation/keyframe_id.cpp` | `crates/nuxie-runtime/src/animation/keyframe_id.rs` | split-needed |
| `B6-0039` · `src/animation/keyframe_interpolator.cpp` | `crates/nuxie-runtime/src/animation/keyframe_interpolator.rs` | split-needed |
| `B6-0040` · `src/animation/keyframe_string.cpp` | `crates/nuxie-runtime/src/animation/keyframe_string.rs` | split-needed |
| `B6-0041` · `src/animation/keyframe_uint.cpp` | `crates/nuxie-runtime/src/animation/keyframe_uint.rs` | split-needed |
| `B6-0043` · `src/animation/linear_animation.cpp` | `crates/nuxie-runtime/src/animation/linear_animation.rs` | split-needed |
| `B6-0044` · `src/animation/linear_animation_instance.cpp` | `crates/nuxie-runtime/src/animation/linear_animation_instance.rs` | split-needed |
| `B6-0059` · `src/animation/nested_animation.cpp` | `crates/nuxie-runtime/src/animation/nested_animation.rs` | split-needed |
| `B6-0060` · `src/animation/nested_bool.cpp` | `crates/nuxie-runtime/src/nested_bool.rs` | pure-move |
| `B6-0061` · `src/animation/nested_linear_animation.cpp` | `crates/nuxie-runtime/src/animation/nested_linear_animation.rs` | split-needed |
| `B6-0062` · `src/animation/nested_number.cpp` | `crates/nuxie-runtime/src/nested_number.rs` | pure-move |
| `B6-0063` · `src/animation/nested_remap_animation.cpp` | `crates/nuxie-runtime/src/animation/nested_remap_animation.rs` | split-needed |
| `B6-0064` · `src/animation/nested_simple_animation.cpp` | `crates/nuxie-runtime/src/animation/nested_simple_animation.rs` | split-needed |
| `B6-0066` · `src/animation/nested_trigger.cpp` | `crates/nuxie-runtime/src/nested_trigger.rs` | pure-move |
| `B6-0221` · `src/importers/keyed_property_importer.cpp` | `crates/nuxie-runtime/src/importers/keyed_property_importer.rs` | split-needed |
| `B6-0323` · `src/scripted/scripted_interpolator.cpp` | — (see E-B6-0323) | exception |

### `crates/nuxie-runtime/src/draw.rs` (29 rows)

| Upstream row | Target module | Move-kind |
|---|---|---|
| `B6-0094` · `src/artboard.cpp` | — (see E-B6-0094) | exception |
| `B6-0095` · `src/artboard_component_list.cpp` | `crates/nuxie-runtime/src/artboard_component_list.rs` | pure-move |
| `B6-0203` · `src/draw_rules.cpp` | `crates/nuxie-runtime/src/draw_rules.rs` | pure-move |
| `B6-0204` · `src/draw_target.cpp` | `crates/nuxie-runtime/src/draw_target.rs` | pure-move |
| `B6-0205` · `src/drawable.cpp` | `crates/nuxie-runtime/src/drawable.rs` | split-needed |
| `B6-0210` · `src/foreground_layout_drawable.cpp` | `crates/nuxie-runtime/src/foreground_layout_drawable.rs` | pure-move |
| `B6-0248` · `src/layout/artboard_component_list_override.cpp` | `crates/nuxie-runtime/src/layout/artboard_component_list_override.rs` | pure-move |
| `B6-0258` · `src/layout_component.cpp` | `crates/nuxie-runtime/src/layout_component.rs` | pure-move |
| `B6-0282` · `src/lua/renderer/lua_image.cpp` | — (see E-B6-0282) | exception |
| `B6-0299` · `src/math/raw_path.cpp` | `crates/nuxie-runtime/src/math/raw_path.rs` | split-needed |
| `B6-0303` · `src/nested_artboard.cpp` | `crates/nuxie-runtime/src/nested_artboard.rs` | pure-move |
| `B6-0304` · `src/nested_artboard_layout.cpp` | `crates/nuxie-runtime/src/nested_artboard_layout.rs` | pure-move |
| `B6-0305` · `src/nested_artboard_leaf.cpp` | `crates/nuxie-runtime/src/nested_artboard_leaf.rs` | pure-move |
| `B6-0322` · `src/scripted/scripted_drawable.cpp` | — (see E-B6-0322) | exception |
| `B6-0324` · `src/scripted/scripted_layout.cpp` | — (see E-B6-0324) | exception |
| `B6-0331` · `src/shapes/clipping_shape.cpp` | `crates/nuxie-runtime/src/shapes/clipping_shape.rs` | pure-move |
| `B6-0352` · `src/shapes/paint/shape_paint.cpp` | `crates/nuxie-runtime/src/shapes/paint/shape_paint.rs` | pure-move |
| `B6-0354` · `src/shapes/paint/shape_paint_path.cpp` | `crates/nuxie-runtime/src/shapes/paint/shape_paint_path.rs` | pure-move |
| `B6-0356` · `src/shapes/paint/stroke.cpp` | `crates/nuxie-runtime/src/shapes/paint/stroke.rs` | pure-move |
| `B6-0357` · `src/shapes/paint/stroke_effect.cpp` | `crates/nuxie-runtime/src/shapes/paint/stroke_effect.rs` | pure-move |
| `B6-0359` · `src/shapes/paint/trim_path.cpp` | `crates/nuxie-runtime/src/shapes/paint/trim_path.rs` | pure-move |
| `B6-0361` · `src/shapes/path.cpp` | `crates/nuxie-runtime/src/shapes/path.rs` | pure-move |
| `B6-0362` · `src/shapes/path_composer.cpp` | `crates/nuxie-runtime/src/shapes/path_composer.rs` | pure-move |
| `B6-0368` · `src/shapes/shape.cpp` | `crates/nuxie-runtime/src/shapes/shape.rs` | pure-move |
| `B6-0369` · `src/shapes/shape_paint_container.cpp` | `crates/nuxie-runtime/src/shapes/shape_paint_container.rs` | pure-move |
| `B6-0383` · `src/text/raw_text.cpp` | — (see E-B6-0383) | exception |
| `B6-0385` · `src/text/text.cpp` | `crates/nuxie-runtime/src/text.rs` | pure-move |
| `B6-0390` · `src/text/text_input_drawable.cpp` | `crates/nuxie-runtime/src/text/text_input_drawable.rs` | pure-move |
| `B6-0402` · `src/text/text_style_paint.cpp` | `crates/nuxie-runtime/src/text/text_style_paint.rs` | pure-move |

### `crates/nuxie-binary/src/importers/mod.rs` (22 rows)

| Upstream row | Target module | Move-kind |
|---|---|---|
| `B6-0212` · `src/importers/artboard_importer.cpp` | `crates/nuxie-binary/src/importers/artboard_importer.rs` | pure-move |
| `B6-0213` · `src/importers/backboard_importer.cpp` | — (see E-B6-0213) | exception |
| `B6-0214` · `src/importers/bindable_property_importer.cpp` | — (see E-B6-0214) | exception |
| `B6-0215` · `src/importers/data_bind_path_importer.cpp` | `crates/nuxie-binary/src/importers/data_bind_path_importer.rs` | pure-move |
| `B6-0216` · `src/importers/data_converter_formula_importer.cpp` | `crates/nuxie-binary/src/importers/data_converter_formula_importer.rs` | pure-move |
| `B6-0217` · `src/importers/data_converter_group_importer.cpp` | `crates/nuxie-binary/src/importers/data_converter_group_importer.rs` | pure-move |
| `B6-0218` · `src/importers/enum_importer.cpp` | `crates/nuxie-binary/src/importers/enum_importer.rs` | pure-move |
| `B6-0219` · `src/importers/file_asset_importer.cpp` | `crates/nuxie-binary/src/importers/file_asset_importer.rs` | pure-move |
| `B6-0220` · `src/importers/keyed_object_importer.cpp` | `crates/nuxie-binary/src/importers/keyed_object_importer.rs` | pure-move |
| `B6-0222` · `src/importers/layer_state_importer.cpp` | `crates/nuxie-binary/src/importers/layer_state_importer.rs` | pure-move |
| `B6-0223` · `src/importers/linear_animation_importer.cpp` | `crates/nuxie-binary/src/importers/linear_animation_importer.rs` | pure-move |
| `B6-0224` · `src/importers/listener_input_type_gamepad_importer.cpp` | `crates/nuxie-binary/src/importers/listener_input_type_gamepad_importer.rs` | pure-move |
| `B6-0225` · `src/importers/listener_input_type_keyboard_importer.cpp` | `crates/nuxie-binary/src/importers/listener_input_type_keyboard_importer.rs` | pure-move |
| `B6-0226` · `src/importers/listener_input_type_semantic_importer.cpp` | `crates/nuxie-binary/src/importers/listener_input_type_semantic_importer.rs` | pure-move |
| `B6-0227` · `src/importers/scripted_object_importer.cpp` | `crates/nuxie-binary/src/importers/scripted_object_importer.rs` | pure-move |
| `B6-0229` · `src/importers/state_machine_layer_component_importer.cpp` | `crates/nuxie-binary/src/importers/state_machine_layer_component_importer.rs` | pure-move |
| `B6-0230` · `src/importers/state_machine_layer_importer.cpp` | `crates/nuxie-binary/src/importers/state_machine_layer_importer.rs` | pure-move |
| `B6-0231` · `src/importers/state_machine_listener_importer.cpp` | `crates/nuxie-binary/src/importers/state_machine_listener_importer.rs` | pure-move |
| `B6-0233` · `src/importers/text_asset_importer.cpp` | `crates/nuxie-binary/src/importers/text_asset_importer.rs` | pure-move |
| `B6-0235` · `src/importers/viewmodel_importer.cpp` | `crates/nuxie-binary/src/importers/viewmodel_importer.rs` | pure-move |
| `B6-0236` · `src/importers/viewmodel_instance_importer.cpp` | `crates/nuxie-binary/src/importers/viewmodel_instance_importer.rs` | pure-move |
| `B6-0237` · `src/importers/viewmodel_instance_list_importer.cpp` | `crates/nuxie-binary/src/importers/viewmodel_instance_list_importer.rs` | pure-move |

### `crates/nuxie-runtime/src/components.rs` (19 rows)

| Upstream row | Target module | Move-kind |
|---|---|---|
| `B6-0115` · `src/bones/bone.cpp` | `crates/nuxie-runtime/src/bones/bone.rs` | split-needed |
| `B6-0116` · `src/bones/root_bone.cpp` | `crates/nuxie-runtime/src/bones/root_bone.rs` | split-needed |
| `B6-0117` · `src/bones/skin.cpp` | `crates/nuxie-runtime/src/bones/skin.rs` | split-needed |
| `B6-0118` · `src/bones/skinnable.cpp` | `crates/nuxie-runtime/src/bones/skinnable.rs` | split-needed |
| `B6-0119` · `src/bones/tendon.cpp` | `crates/nuxie-runtime/src/bones/tendon.rs` | split-needed |
| `B6-0120` · `src/bones/weight.cpp` | `crates/nuxie-runtime/src/bones/weight.rs` | split-needed |
| `B6-0123` · `src/component.cpp` | `crates/nuxie-runtime/src/component.rs` | split-needed |
| `B6-0289` · `src/math/aabb.cpp` | `crates/nuxie-runtime/src/math/aabb.rs` | split-needed |
| `B6-0290` · `src/math/bezier_utils.cpp` | `crates/nuxie-runtime/src/math/bezier_utils.rs` | split-needed |
| `B6-0291` · `src/math/bit_field_loc.cpp` | `crates/nuxie-runtime/src/math/bit_field_loc.rs` | split-needed |
| `B6-0292` · `src/math/contour_measure.cpp` | `crates/nuxie-runtime/src/math/contour_measure.rs` | split-needed |
| `B6-0294` · `src/math/mat2d.cpp` | `crates/nuxie-runtime/src/math/mat2d.rs` | split-needed |
| `B6-0295` · `src/math/mat2d_find_max_scale.cpp` | `crates/nuxie-runtime/src/math/mat2d_find_max_scale.rs` | split-needed |
| `B6-0296` · `src/math/n_slicer_helpers.cpp` | `crates/nuxie-runtime/src/math/n_slicer_helpers.rs` | split-needed |
| `B6-0297` · `src/math/path_measure.cpp` | `crates/nuxie-runtime/src/math/path_measure.rs` | split-needed |
| `B6-0299` · `src/math/raw_path.cpp` | `crates/nuxie-runtime/src/math/raw_path.rs` | split-needed |
| `B6-0300` · `src/math/raw_path_utils.cpp` | `crates/nuxie-runtime/src/math/raw_path_utils.rs` | split-needed |
| `B6-0302` · `src/math/vec2d.cpp` | `crates/nuxie-runtime/src/math/vec2d.rs` | split-needed |
| `B6-0388` · `src/text/text_input.cpp` | `crates/nuxie-runtime/src/text_input.rs` | pure-move |

### `crates/nuxie-runtime/src/constraints.rs` (19 rows)

| Upstream row | Target module | Move-kind |
|---|---|---|
| `B6-0124` · `src/constraints/constrainable_list.cpp` | `crates/nuxie-runtime/src/constraints/constrainable_list.rs` | split-needed |
| `B6-0125` · `src/constraints/constraint.cpp` | `crates/nuxie-runtime/src/constraints/constraint.rs` | split-needed |
| `B6-0126` · `src/constraints/distance_constraint.cpp` | `crates/nuxie-runtime/src/constraints/distance_constraint.rs` | split-needed |
| `B6-0127` · `src/constraints/draggable_constraint.cpp` | `crates/nuxie-runtime/src/constraints/draggable_constraint.rs` | split-needed |
| `B6-0128` · `src/constraints/follow_path_constraint.cpp` | `crates/nuxie-runtime/src/constraints/follow_path_constraint.rs` | split-needed |
| `B6-0129` · `src/constraints/ik_constraint.cpp` | `crates/nuxie-runtime/src/constraints/ik_constraint.rs` | split-needed |
| `B6-0130` · `src/constraints/list_constraint.cpp` | `crates/nuxie-runtime/src/constraints/list_constraint.rs` | split-needed |
| `B6-0131` · `src/constraints/list_follow_path_constraint.cpp` | `crates/nuxie-runtime/src/constraints/list_follow_path_constraint.rs` | split-needed |
| `B6-0132` · `src/constraints/rotation_constraint.cpp` | `crates/nuxie-runtime/src/constraints/rotation_constraint.rs` | split-needed |
| `B6-0133` · `src/constraints/scale_constraint.cpp` | `crates/nuxie-runtime/src/constraints/scale_constraint.rs` | split-needed |
| `B6-0134` · `src/constraints/scrolling/clamped_scroll_physics.cpp` | `crates/nuxie-runtime/src/constraints/scrolling/clamped_scroll_physics.rs` | split-needed |
| `B6-0138` · `src/constraints/scrolling/scroll_constraint.cpp` | `crates/nuxie-runtime/src/constraints/scrolling/scroll_constraint.rs` | split-needed |
| `B6-0139` · `src/constraints/scrolling/scroll_constraint_proxy.cpp` | `crates/nuxie-runtime/src/constraints/scrolling/scroll_constraint_proxy.rs` | split-needed |
| `B6-0140` · `src/constraints/scrolling/scroll_physics.cpp` | `crates/nuxie-runtime/src/constraints/scrolling/scroll_physics.rs` | split-needed |
| `B6-0141` · `src/constraints/scrolling/scroll_virtualizer.cpp` | `crates/nuxie-runtime/src/constraints/scrolling/scroll_virtualizer.rs` | split-needed |
| `B6-0142` · `src/constraints/targeted_constraint.cpp` | `crates/nuxie-runtime/src/constraints/targeted_constraint.rs` | split-needed |
| `B6-0143` · `src/constraints/transform_constraint.cpp` | `crates/nuxie-runtime/src/constraints/transform_constraint.rs` | split-needed |
| `B6-0144` · `src/constraints/translation_constraint.cpp` | `crates/nuxie-runtime/src/constraints/translation_constraint.rs` | split-needed |
| `B6-0388` · `src/text/text_input.cpp` | `crates/nuxie-runtime/src/text_input.rs` | pure-move |

## Secondary shared-file hotspots

This grouped table is the per-hotspot plan for every remaining Rust file referenced by 2+ upstream rows.

| Current hotspot | Upstream row | Target module | Move-kind |
|---|---|---|---|
| `crates/nuxie-audio/src/engine.rs` (2) | `B6-0109` · `src/audio/audio_engine.cpp` | `crates/nuxie-audio/src/audio_engine.rs` | split-needed |
| `crates/nuxie-audio/src/engine.rs` (2) | `B6-0111` · `src/audio/audio_sound.cpp` | `crates/nuxie-audio/src/audio_sound.rs` | split-needed |
| `crates/nuxie-audio/src/source.rs` (2) | `B6-0110` · `src/audio/audio_reader.cpp` | `crates/nuxie-audio/src/audio_reader.rs` | split-needed |
| `crates/nuxie-audio/src/source.rs` (2) | `B6-0112` · `src/audio/audio_source.cpp` | `crates/nuxie-audio/src/audio_source.rs` | split-needed |
| `crates/nuxie-binary/src/assets/file_asset.rs` (2) | `B6-0100` · `src/assets/file_asset.cpp` | `crates/nuxie-binary/src/assets/file_asset.rs` | pure-move |
| `crates/nuxie-binary/src/assets/file_asset.rs` (2) | `B6-0208` · `src/file.cpp` | — (see E-B6-0208) | exception |
| `crates/nuxie-binary/src/assets/file_asset_contents.rs` (3) | `B6-0099` · `src/assets/blob_asset.cpp` | `crates/nuxie-binary/src/assets/blob_asset.rs` | pure-move |
| `crates/nuxie-binary/src/assets/file_asset_contents.rs` (3) | `B6-0101` · `src/assets/file_asset_contents.cpp` | `crates/nuxie-binary/src/assets/file_asset_contents.rs` | pure-move |
| `crates/nuxie-binary/src/assets/file_asset_contents.rs` (3) | `B6-0208` · `src/file.cpp` | — (see E-B6-0208) | exception |
| `crates/nuxie-runtime/src/assets/file_asset_loader.rs` (2) | `B6-0098` · `src/assets/audio_asset.cpp` | `crates/nuxie-runtime/src/assets/audio_asset.rs` | pure-move |
| `crates/nuxie-runtime/src/assets/file_asset_loader.rs` (2) | `B6-0103` · `src/assets/font_asset.cpp` | `crates/nuxie-runtime/src/assets/font_asset.rs` | pure-move |
| `crates/nuxie-runtime/src/data_bind/data_bind_context.rs` (2) | `B6-0196` · `src/data_bind/data_bind_context.cpp` | — (see E-B6-0196) | exception |
| `crates/nuxie-runtime/src/data_bind/data_bind_context.rs` (2) | `B6-0282` · `src/lua/renderer/lua_image.cpp` | — (see E-B6-0282) | exception |
| `crates/nuxie-runtime/src/focus.rs` (3) | `B6-0238` · `src/input/focus_manager.cpp` | `crates/nuxie-runtime/src/input/focus_manager.rs` | split-needed |
| `crates/nuxie-runtime/src/focus.rs` (3) | `B6-0239` · `src/input/focus_node.cpp` | `crates/nuxie-runtime/src/input/focus_node.rs` | split-needed |
| `crates/nuxie-runtime/src/focus.rs` (3) | `B6-0240` · `src/input/focusable.cpp` | `crates/nuxie-runtime/src/input/focusable.rs` | split-needed |
| `crates/nuxie-runtime/src/layout_component.rs` (3) | `B6-0252` · `src/layout/layout_component_style.cpp` | `crates/nuxie-runtime/src/layout/layout_component_style.rs` | pure-move |
| `crates/nuxie-runtime/src/layout_component.rs` (3) | `B6-0253` · `src/layout/layout_node_provider.cpp` | `crates/nuxie-runtime/src/layout/layout_node_provider.rs` | pure-move |
| `crates/nuxie-runtime/src/layout_component.rs` (3) | `B6-0258` · `src/layout_component.cpp` | `crates/nuxie-runtime/src/layout_component.rs` | pure-move |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0001` · `src/advancing_component.cpp` | `crates/nuxie-runtime/src/advancing_component.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0145` · `src/container_component.cpp` | `crates/nuxie-runtime/src/container_component.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0146` · `src/core.cpp` | `crates/nuxie-runtime/src/core.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0202` · `src/dependency_sorter.cpp` | `crates/nuxie-runtime/src/dependency_sorter.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0207` · `src/factory.cpp` | `crates/nuxie-runtime/src/factory.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0208` · `src/file.cpp` | — (see E-B6-0208) | exception |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0209` · `src/focus_data.cpp` | `crates/nuxie-runtime/src/focus_data.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0307` · `src/node.cpp` | `crates/nuxie-runtime/src/node.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0311` · `src/renderer.cpp` | `crates/nuxie-runtime/src/renderer.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0312` · `src/resetting_component.cpp` | `crates/nuxie-runtime/src/resetting_component.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0375` · `src/simple_array.cpp` | `crates/nuxie-runtime/src/simple_array.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0408` · `src/transform_component.cpp` | `crates/nuxie-runtime/src/transform_component.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0446` · `src/virtualizing_component.cpp` | `crates/nuxie-runtime/src/virtualizing_component.rs` | split-needed |
| `crates/nuxie-runtime/src/lib.rs` (14) | `B6-0447` · `src/world_transform_component.cpp` | `crates/nuxie-runtime/src/world_transform_component.rs` | split-needed |
| `crates/nuxie-runtime/src/listener_group.rs` (2) | `B6-0083` · `src/animation/text_input_listener_group.cpp` | `crates/nuxie-runtime/src/state_machine/text_input_listener_group.rs` | pure-move |
| `crates/nuxie-runtime/src/listener_group.rs` (2) | `B6-0259` · `src/listener_group.cpp` | `crates/nuxie-runtime/src/listener_group.rs` | pure-move |
| `crates/nuxie-runtime/src/profiler.rs` (2) | `B6-0309` · `src/profiler/profiler.cpp` | `crates/nuxie-runtime/src/profiler.rs` | pure-move |
| `crates/nuxie-runtime/src/profiler.rs` (2) | `B6-0310` · `src/profiler/rive_profile.cpp` | `crates/nuxie-runtime/src/profiler/rive_profile.rs` | split-needed |
| `crates/nuxie-runtime/src/rectangles_to_contour.rs` (2) | `B6-0301` · `src/math/rectangles_to_contour.cpp` | `crates/nuxie-runtime/src/rectangles_to_contour.rs` | pure-move |
| `crates/nuxie-runtime/src/rectangles_to_contour.rs` (2) | `B6-0398` · `src/text/text_selection_path.cpp` | `crates/nuxie-runtime/src/text/text_selection_path.rs` | pure-move |
| `crates/nuxie-runtime/src/scripted_data_converter.rs` (2) | `B6-0106` · `src/assets/script_asset.cpp` | — (see E-B6-0106) | exception |
| `crates/nuxie-runtime/src/scripted_data_converter.rs` (2) | `B6-0321` · `src/scripted/scripted_data_converter.cpp` | — (see E-B6-0321) | exception |
| `crates/nuxie-runtime/src/scripting.rs` (6) | `B6-0282` · `src/lua/renderer/lua_image.cpp` | — (see E-B6-0282) | exception |
| `crates/nuxie-runtime/src/scripting.rs` (6) | `B6-0322` · `src/scripted/scripted_drawable.cpp` | — (see E-B6-0322) | exception |
| `crates/nuxie-runtime/src/scripting.rs` (6) | `B6-0323` · `src/scripted/scripted_interpolator.cpp` | — (see E-B6-0323) | exception |
| `crates/nuxie-runtime/src/scripting.rs` (6) | `B6-0324` · `src/scripted/scripted_layout.cpp` | — (see E-B6-0324) | exception |
| `crates/nuxie-runtime/src/scripting.rs` (6) | `B6-0325` · `src/scripted/scripted_object.cpp` | — (see E-B6-0325) | exception |
| `crates/nuxie-runtime/src/scripting.rs` (6) | `B6-0326` · `src/scripted/scripted_path_effect.cpp` | `crates/nuxie-runtime/src/scripted/scripted_path_effect.rs` | split-needed |
| `crates/nuxie-runtime/src/shapes/image.rs` (2) | `B6-0282` · `src/lua/renderer/lua_image.cpp` | — (see E-B6-0282) | exception |
| `crates/nuxie-runtime/src/shapes/image.rs` (2) | `B6-0338` · `src/shapes/image.cpp` | `crates/nuxie-runtime/src/shapes/image.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/animation_reset_factory.rs` (2) | `B6-0002` · `src/animation/animation_reset.cpp` | `crates/nuxie-runtime/src/animation/animation_reset.rs` | split-needed |
| `crates/nuxie-runtime/src/state_machine/animation_reset_factory.rs` (2) | `B6-0003` · `src/animation/animation_reset_factory.cpp` | `crates/nuxie-runtime/src/state_machine/animation_reset_factory.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/bindables.rs` (2) | `B6-0077` · `src/animation/state_machine_instance.cpp` | — (see E-B6-0077) | exception |
| `crates/nuxie-runtime/src/state_machine/bindables.rs` (2) | `B6-0214` · `src/importers/bindable_property_importer.cpp` | — (see E-B6-0214) | exception |
| `crates/nuxie-runtime/src/state_machine/data_bind_template.rs` (3) | `B6-0077` · `src/animation/state_machine_instance.cpp` | — (see E-B6-0077) | exception |
| `crates/nuxie-runtime/src/state_machine/data_bind_template.rs` (3) | `B6-0194` · `src/data_bind/data_bind.cpp` | — (see E-B6-0194) | exception |
| `crates/nuxie-runtime/src/state_machine/data_bind_template.rs` (3) | `B6-0213` · `src/importers/backboard_importer.cpp` | — (see E-B6-0213) | exception |
| `crates/nuxie-runtime/src/state_machine/data_converter_binding.rs` (3) | `B6-0077` · `src/animation/state_machine_instance.cpp` | — (see E-B6-0077) | exception |
| `crates/nuxie-runtime/src/state_machine/data_converter_binding.rs` (3) | `B6-0195` · `src/data_bind/data_bind_container.cpp` | — (see E-B6-0195) | exception |
| `crates/nuxie-runtime/src/state_machine/data_converter_binding.rs` (3) | `B6-0196` · `src/data_bind/data_bind_context.cpp` | — (see E-B6-0196) | exception |
| `crates/nuxie-runtime/src/state_machine/listener_action_owner.rs` (3) | `B6-0045` · `src/animation/listener_action.cpp` | `crates/nuxie-runtime/src/state_machine/listener_action.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/listener_action_owner.rs` (3) | `B6-0073` · `src/animation/state_machine_fire_action.cpp` | `crates/nuxie-runtime/src/state_machine/state_machine_fire_action.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/listener_action_owner.rs` (3) | `B6-0094` · `src/artboard.cpp` | — (see E-B6-0094) | exception |
| `crates/nuxie-runtime/src/state_machine/scripted_listener_action.rs` (2) | `B6-0068` · `src/animation/scripted_listener_action.cpp` | — (see E-B6-0068) | exception |
| `crates/nuxie-runtime/src/state_machine/scripted_listener_action.rs` (2) | `B6-0106` · `src/assets/script_asset.cpp` | — (see E-B6-0106) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_input.rs` (2) | `B6-0075` · `src/animation/state_machine_input.cpp` | `crates/nuxie-runtime/src/state_machine/state_machine_input.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/state_machine_input.rs` (2) | `B6-0228` · `src/importers/state_machine_importer.cpp` | — (see E-B6-0228) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_input_instance.rs` (2) | `B6-0076` · `src/animation/state_machine_input_instance.cpp` | `crates/nuxie-runtime/src/state_machine/state_machine_input_instance.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/state_machine_input_instance.rs` (2) | `B6-0228` · `src/importers/state_machine_importer.cpp` | — (see E-B6-0228) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0058` · `src/animation/listener_viewmodel_change.cpp` | `crates/nuxie-runtime/src/state_machine/listener_viewmodel_change.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0077` · `src/animation/state_machine_instance.cpp` | — (see E-B6-0077) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0083` · `src/animation/text_input_listener_group.cpp` | `crates/nuxie-runtime/src/state_machine/text_input_listener_group.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0113` · `src/audio_event.cpp` | — (see E-B6-0113) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0175` · `src/data_bind/converters/data_converter_group.cpp` | — (see E-B6-0175) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0194` · `src/data_bind/data_bind.cpp` | — (see E-B6-0194) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0195` · `src/data_bind/data_bind_container.cpp` | — (see E-B6-0195) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0196` · `src/data_bind/data_bind_context.cpp` | — (see E-B6-0196) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0200` · `src/data_bind/data_context.cpp` | — (see E-B6-0200) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0310` · `src/profiler/rive_profile.cpp` | `crates/nuxie-runtime/src/profiler/rive_profile.rs` | split-needed |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0321` · `src/scripted/scripted_data_converter.cpp` | — (see E-B6-0321) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (12) | `B6-0440` · `src/viewmodel/viewmodel_instance_trigger.cpp` | — (see E-B6-0440) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs` (2) | `B6-0077` · `src/animation/state_machine_instance.cpp` | — (see E-B6-0077) | exception |
| `crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs` (2) | `B6-0310` · `src/profiler/rive_profile.cpp` | `crates/nuxie-runtime/src/profiler/rive_profile.rs` | split-needed |
| `crates/nuxie-runtime/src/state_machine/state_transition.rs` (2) | `B6-0081` · `src/animation/state_transition.cpp` | `crates/nuxie-runtime/src/state_machine/state_transition.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/state_transition.rs` (2) | `B6-0232` · `src/importers/state_transition_importer.cpp` | — (see E-B6-0232) | exception |
| `crates/nuxie-runtime/src/state_machine/transition_condition_op.rs` (2) | `B6-0089` · `src/animation/transition_number_condition.cpp` | `crates/nuxie-runtime/src/state_machine/transition_number_condition.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/transition_condition_op.rs` (2) | `B6-0093` · `src/animation/transition_viewmodel_condition.cpp` | `crates/nuxie-runtime/src/state_machine/transition_viewmodel_condition.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/transition_duration_binding.rs` (3) | `B6-0077` · `src/animation/state_machine_instance.cpp` | — (see E-B6-0077) | exception |
| `crates/nuxie-runtime/src/state_machine/transition_duration_binding.rs` (3) | `B6-0081` · `src/animation/state_transition.cpp` | `crates/nuxie-runtime/src/state_machine/state_transition.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/transition_duration_binding.rs` (3) | `B6-0195` · `src/data_bind/data_bind_container.cpp` | — (see E-B6-0195) | exception |
| `crates/nuxie-runtime/src/state_machine/transition_viewmodel_condition.rs` (2) | `B6-0093` · `src/animation/transition_viewmodel_condition.cpp` | `crates/nuxie-runtime/src/state_machine/transition_viewmodel_condition.rs` | pure-move |
| `crates/nuxie-runtime/src/state_machine/transition_viewmodel_condition.rs` (2) | `B6-0234` · `src/importers/transition_viewmodel_condition_importer.cpp` | — (see E-B6-0234) | exception |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0383` · `src/text/raw_text.cpp` | — (see E-B6-0383) | exception |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0384` · `src/text/raw_text_input.cpp` | `crates/nuxie-runtime/src/text/raw_text_input.rs` | pure-move |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0385` · `src/text/text.cpp` | `crates/nuxie-runtime/src/text.rs` | pure-move |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0388` · `src/text/text_input.cpp` | `crates/nuxie-runtime/src/text_input.rs` | pure-move |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0389` · `src/text/text_input_cursor.cpp` | `crates/nuxie-runtime/src/text/text_input_cursor.rs` | pure-move |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0390` · `src/text/text_input_drawable.cpp` | `crates/nuxie-runtime/src/text/text_input_drawable.rs` | pure-move |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0391` · `src/text/text_input_selected_text.cpp` | `crates/nuxie-runtime/src/text/text_input_selected_text.rs` | pure-move |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0392` · `src/text/text_input_selection.cpp` | `crates/nuxie-runtime/src/text/text_input_selection.rs` | pure-move |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0393` · `src/text/text_input_text.cpp` | `crates/nuxie-runtime/src/text/text_input_text.rs` | pure-move |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0394` · `src/text/text_interface.cpp` | `crates/nuxie-runtime/src/text/text_interface.rs` | pure-move |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0398` · `src/text/text_selection_path.cpp` | `crates/nuxie-runtime/src/text/text_selection_path.rs` | pure-move |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0399` · `src/text/text_style.cpp` | `crates/nuxie-runtime/src/text/text_style.rs` | pure-move |
| `crates/nuxie-runtime/src/text.rs` (13) | `B6-0404` · `src/text/text_value_run.cpp` | `crates/nuxie-runtime/src/text/text_value_run.rs` | pure-move |
| `crates/nuxie-runtime/src/text/font_hb.rs` (2) | `B6-0379` · `src/text/font_hb.cpp` | `crates/nuxie-runtime/src/text/font_hb.rs` | pure-move |
| `crates/nuxie-runtime/src/text/font_hb.rs` (2) | `B6-0383` · `src/text/raw_text.cpp` | — (see E-B6-0383) | exception |
| `crates/nuxie-runtime/src/text/text_engine.rs` (2) | `B6-0383` · `src/text/raw_text.cpp` | — (see E-B6-0383) | exception |
| `crates/nuxie-runtime/src/text/text_engine.rs` (2) | `B6-0386` · `src/text/text_engine.cpp` | `crates/nuxie-runtime/src/text/text_engine.rs` | pure-move |
| `crates/nuxie-runtime/src/text/text_style.rs` (2) | `B6-0394` · `src/text/text_interface.cpp` | `crates/nuxie-runtime/src/text/text_interface.rs` | pure-move |
| `crates/nuxie-runtime/src/text/text_style.rs` (2) | `B6-0399` · `src/text/text_style.cpp` | `crates/nuxie-runtime/src/text/text_style.rs` | pure-move |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0200` · `src/data_bind/data_context.cpp` | — (see E-B6-0200) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0427` · `src/viewmodel/viewmodel_instance.cpp` | — (see E-B6-0427) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0428` · `src/viewmodel/viewmodel_instance_artboard.cpp` | — (see E-B6-0428) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0429` · `src/viewmodel/viewmodel_instance_asset.cpp` | — (see E-B6-0429) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0432` · `src/viewmodel/viewmodel_instance_boolean.cpp` | — (see E-B6-0432) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0433` · `src/viewmodel/viewmodel_instance_color.cpp` | — (see E-B6-0433) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0434` · `src/viewmodel/viewmodel_instance_enum.cpp` | — (see E-B6-0434) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0435` · `src/viewmodel/viewmodel_instance_list.cpp` | — (see E-B6-0435) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0437` · `src/viewmodel/viewmodel_instance_number.cpp` | — (see E-B6-0437) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0438` · `src/viewmodel/viewmodel_instance_string.cpp` | — (see E-B6-0438) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0439` · `src/viewmodel/viewmodel_instance_symbol_list_index.cpp` | — (see E-B6-0439) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0440` · `src/viewmodel/viewmodel_instance_trigger.cpp` | — (see E-B6-0440) | exception |
| `crates/nuxie-runtime/src/view_model.rs` (13) | `B6-0442` · `src/viewmodel/viewmodel_instance_viewmodel.cpp` | — (see E-B6-0442) | exception |
| `crates/nuxie-runtime/src/view_model_cell.rs` (7) | `B6-0077` · `src/animation/state_machine_instance.cpp` | — (see E-B6-0077) | exception |
| `crates/nuxie-runtime/src/view_model_cell.rs` (7) | `B6-0194` · `src/data_bind/data_bind.cpp` | — (see E-B6-0194) | exception |
| `crates/nuxie-runtime/src/view_model_cell.rs` (7) | `B6-0195` · `src/data_bind/data_bind_container.cpp` | — (see E-B6-0195) | exception |
| `crates/nuxie-runtime/src/view_model_cell.rs` (7) | `B6-0427` · `src/viewmodel/viewmodel_instance.cpp` | — (see E-B6-0427) | exception |
| `crates/nuxie-runtime/src/view_model_cell.rs` (7) | `B6-0430` · `src/viewmodel/viewmodel_instance_asset_font.cpp` | — (see E-B6-0430) | exception |
| `crates/nuxie-runtime/src/view_model_cell.rs` (7) | `B6-0440` · `src/viewmodel/viewmodel_instance_trigger.cpp` | — (see E-B6-0440) | exception |
| `crates/nuxie-runtime/src/view_model_cell.rs` (7) | `B6-0441` · `src/viewmodel/viewmodel_instance_value.cpp` | — (see E-B6-0441) | exception |
| `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_asset_image.rs` (2) | `B6-0282` · `src/lua/renderer/lua_image.cpp` | — (see E-B6-0282) | exception |
| `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_asset_image.rs` (2) | `B6-0431` · `src/viewmodel/viewmodel_instance_asset_image.cpp` | — (see E-B6-0431) | exception |
| `crates/nuxie-scripting/src/vm.rs` (7) | `B6-0106` · `src/assets/script_asset.cpp` | — (see E-B6-0106) | exception |
| `crates/nuxie-scripting/src/vm.rs` (7) | `B6-0265` · `src/lua/lua_data_value.cpp` | `crates/nuxie-scripting/src/vm/lua_data_value.rs` | split-needed |
| `crates/nuxie-scripting/src/vm.rs` (7) | `B6-0266` · `src/lua/lua_image_decode.cpp` | — (see E-B6-0266) | exception |
| `crates/nuxie-scripting/src/vm.rs` (7) | `B6-0268` · `src/lua/lua_promise.cpp` | `crates/nuxie-scripting/src/vm/lua_promise.rs` | split-needed |
| `crates/nuxie-scripting/src/vm.rs` (7) | `B6-0269` · `src/lua/lua_properties.cpp` | — (see E-B6-0269) | exception |
| `crates/nuxie-scripting/src/vm.rs` (7) | `B6-0288` · `src/lua/rive_lua_libs.cpp` | — (see E-B6-0288) | exception |
| `crates/nuxie-scripting/src/vm.rs` (7) | `B6-0323` · `src/scripted/scripted_interpolator.cpp` | — (see E-B6-0323) | exception |
| `crates/nuxie-scripting/src/vm/listener_invocation.rs` (2) | `B6-0267` · `src/lua/lua_listener_invocation.cpp` | `crates/nuxie-scripting/src/vm/lua_listener_invocation.rs` | split-needed |
| `crates/nuxie-scripting/src/vm/listener_invocation.rs` (2) | `B6-0274` · `src/lua/math/lua_input.cpp` | `crates/nuxie-scripting/src/vm/math/lua_input.rs` | split-needed |
| `crates/nuxie-scripting/src/vm/promise.rs` (2) | `B6-0266` · `src/lua/lua_image_decode.cpp` | — (see E-B6-0266) | exception |
| `crates/nuxie-scripting/src/vm/promise.rs` (2) | `B6-0268` · `src/lua/lua_promise.cpp` | `crates/nuxie-scripting/src/vm/lua_promise.rs` | split-needed |
| `crates/nuxie-scripting/src/vm/view_model.rs` (4) | `B6-0262` · `src/lua/lua_audio.cpp` | — (see E-B6-0262) | exception |
| `crates/nuxie-scripting/src/vm/view_model.rs` (4) | `B6-0264` · `src/lua/lua_data_context.cpp` | `crates/nuxie-scripting/src/vm/lua_data_context.rs` | split-needed |
| `crates/nuxie-scripting/src/vm/view_model.rs` (4) | `B6-0272` · `src/lua/lua_state.cpp` | `crates/nuxie-scripting/src/vm/lua_state.rs` | split-needed |
| `crates/nuxie-scripting/src/vm/view_model.rs` (4) | `B6-0282` · `src/lua/renderer/lua_image.cpp` | — (see E-B6-0282) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0068` · `src/animation/scripted_listener_action.cpp` | — (see E-B6-0068) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0077` · `src/animation/state_machine_instance.cpp` | — (see E-B6-0077) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0094` · `src/artboard.cpp` | — (see E-B6-0094) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0106` · `src/assets/script_asset.cpp` | — (see E-B6-0106) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0113` · `src/audio_event.cpp` | — (see E-B6-0113) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0208` · `src/file.cpp` | — (see E-B6-0208) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0213` · `src/importers/backboard_importer.cpp` | — (see E-B6-0213) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0260` · `src/lua/logging_scripting_context.cpp` | — (see E-B6-0260) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0261` · `src/lua/lua_artboards.cpp` | — (see E-B6-0261) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0262` · `src/lua/lua_audio.cpp` | — (see E-B6-0262) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0319` · `src/script_input_trigger.cpp` | — (see E-B6-0319) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0320` · `src/script_input_viewmodel_property.cpp` | — (see E-B6-0320) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0321` · `src/scripted/scripted_data_converter.cpp` | — (see E-B6-0321) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0322` · `src/scripted/scripted_drawable.cpp` | — (see E-B6-0322) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0323` · `src/scripted/scripted_interpolator.cpp` | — (see E-B6-0323) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0324` · `src/scripted/scripted_layout.cpp` | — (see E-B6-0324) | exception |
| `crates/nuxie/src/lib.rs` (17) | `B6-0325` · `src/scripted/scripted_object.cpp` | — (see E-B6-0325) | exception |

## Scattered rows with no shared-file hotspot

These rows meet the 2+ files rule even though none of their current files is shared by another upstream row.

| Upstream row | Target module | Move-kind |
|---|---|---|
| `B6-0107` · `src/assets/shader_asset.cpp` | — (see E-B6-0107) | exception |
| `B6-0206` · `src/event.cpp` | `crates/nuxie-runtime/src/event.rs` | pure-move |
| `B6-0298` · `src/math/random.cpp` | — (see E-B6-0298) | exception |

## Worker clusters and single-writer ownership

Workers may prepare branches in parallel, but landing is serialized through the manifest integrator. A root listed below has exactly one source-file writer for the entire regrouping. In particular, C07 is the only writer for `crates/nuxie-binary/src/lib.rs`, and C08 is the only writer for `crates/nuxie-scripting/src/vm.rs`.

| Cluster / worker | Owned roots | Size | Parallel-dispatch scope |
|---|---|---|---|
| C01 / runtime-animation | `animation.rs` | L | 42-row extraction mini-queue; can prepare beside C02/C03/C06. |
| C02 / runtime-components | `components.rs` | M | Bones/math rows; defer cross-root finalization to the ordered landing train. |
| C03 / runtime-constraints | `constraints.rs` | L | Constraint/scrolling rows; defer B6-0388 finalization until C02 is rebased. |
| C04 / runtime-artboard | `artboard.rs` | L | 44-row spine mini-queue; lands before C05. |
| C05 / runtime-draw | `draw.rs` | L | 29-row render/draw extraction; prepare in parallel, rebase after C04. |
| C06 / binary-importers | `importers/mod.rs` | M | Move importer-specific glue into the 22 dedicated importer files. |
| C07 / binary-lib-integrator | `nuxie-binary/src/lib.rs` only | L | Sole binary root writer; schema exceptions stay and actionable importer fragments leave. |
| C08 / scripting-vm-integrator | `vm.rs` plus shared `vm/**` leaves | L | Sole VM-root writer; serial sub-queue for Lua rows. |
| C09 / runtime-lib-integrator | `nuxie-runtime/src/lib.rs` | M | Root implementations leave; module declarations/re-exports stay. |
| C10 / nuxie-facade-integrator | `nuxie/src/lib.rs` | M | Public façade exceptions and any root-owned integration fragments. |
| C11 / runtime-state-machine | shared `state_machine/**` roots | M | Listener/input/transition leaf consolidations. |
| C12 / runtime-text | shared `text.rs` and `text/**` roots | M | Text engine/input/style leaf consolidations. |
| C13 / runtime-viewmodel | shared `view_model*.rs` and `viewmodel/**` roots | M | View-model leaf consolidations after schema exception notes are pinned. |
| C14 / binary-assets | shared `assets/**` roots | S | Blob/file contents ownership. |
| C15 / audio | shared audio roots | S | Engine/source split. |
| C16 / runtime-secondary | all other shared runtime roots | M | Focus/layout/profiler/scripting/miscellaneous two-to-seven-row roots. |
| C17 / manifest-integrator | `file-correspondence-manifest.toml` | M | Assembles root-owner patches and lands each cross-root row/batch atomically with its manifest update. |

### Shared-root ownership ledger

| Shared root | Rows | Sole writer |
|---|---:|---|
| `crates/nuxie-binary/src/lib.rs` | 57 | C07 / binary-lib-integrator |
| `crates/nuxie-runtime/src/artboard.rs` | 44 | C04 / runtime-artboard |
| `crates/nuxie-runtime/src/animation.rs` | 42 | C01 / runtime-animation |
| `crates/nuxie-runtime/src/draw.rs` | 29 | C05 / runtime-draw |
| `crates/nuxie-binary/src/importers/mod.rs` | 22 | C06 / binary-importers |
| `crates/nuxie-runtime/src/components.rs` | 19 | C02 / runtime-components |
| `crates/nuxie-runtime/src/constraints.rs` | 19 | C03 / runtime-constraints |
| `crates/nuxie/src/lib.rs` | 17 | C10 / nuxie-facade-integrator |
| `crates/nuxie-runtime/src/lib.rs` | 14 | C09 / runtime-lib-integrator |
| `crates/nuxie-runtime/src/text.rs` | 13 | C12 / runtime-text |
| `crates/nuxie-runtime/src/view_model.rs` | 13 | C13 / runtime-viewmodel |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 12 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/view_model_cell.rs` | 7 | C13 / runtime-viewmodel |
| `crates/nuxie-scripting/src/vm.rs` | 7 | C08 / scripting-vm-integrator |
| `crates/nuxie-runtime/src/scripting.rs` | 6 | C16 / runtime-secondary |
| `crates/nuxie-scripting/src/vm/view_model.rs` | 4 | C08 / scripting-vm-integrator |
| `crates/nuxie-binary/src/assets/file_asset_contents.rs` | 3 | C14 / binary-assets |
| `crates/nuxie-runtime/src/focus.rs` | 3 | C16 / runtime-secondary |
| `crates/nuxie-runtime/src/layout_component.rs` | 3 | C16 / runtime-secondary |
| `crates/nuxie-runtime/src/state_machine/data_bind_template.rs` | 3 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/state_machine/data_converter_binding.rs` | 3 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/state_machine/listener_action_owner.rs` | 3 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/state_machine/transition_duration_binding.rs` | 3 | C11 / runtime-state-machine |
| `crates/nuxie-audio/src/engine.rs` | 2 | C15 / audio |
| `crates/nuxie-audio/src/source.rs` | 2 | C15 / audio |
| `crates/nuxie-binary/src/assets/file_asset.rs` | 2 | C14 / binary-assets |
| `crates/nuxie-runtime/src/assets/file_asset_loader.rs` | 2 | C16 / runtime-secondary |
| `crates/nuxie-runtime/src/data_bind/data_bind_context.rs` | 2 | C16 / runtime-secondary |
| `crates/nuxie-runtime/src/listener_group.rs` | 2 | C16 / runtime-secondary |
| `crates/nuxie-runtime/src/profiler.rs` | 2 | C16 / runtime-secondary |
| `crates/nuxie-runtime/src/rectangles_to_contour.rs` | 2 | C16 / runtime-secondary |
| `crates/nuxie-runtime/src/scripted_data_converter.rs` | 2 | C16 / runtime-secondary |
| `crates/nuxie-runtime/src/shapes/image.rs` | 2 | C16 / runtime-secondary |
| `crates/nuxie-runtime/src/state_machine/animation_reset_factory.rs` | 2 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/state_machine/bindables.rs` | 2 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/state_machine/scripted_listener_action.rs` | 2 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/state_machine/state_machine_input.rs` | 2 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/state_machine/state_machine_input_instance.rs` | 2 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs` | 2 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/state_machine/state_transition.rs` | 2 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/state_machine/transition_condition_op.rs` | 2 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/state_machine/transition_viewmodel_condition.rs` | 2 | C11 / runtime-state-machine |
| `crates/nuxie-runtime/src/text/font_hb.rs` | 2 | C12 / runtime-text |
| `crates/nuxie-runtime/src/text/text_engine.rs` | 2 | C12 / runtime-text |
| `crates/nuxie-runtime/src/text/text_style.rs` | 2 | C12 / runtime-text |
| `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_asset_image.rs` | 2 | C13 / runtime-viewmodel |
| `crates/nuxie-scripting/src/vm/listener_invocation.rs` | 2 | C08 / scripting-vm-integrator |
| `crates/nuxie-scripting/src/vm/promise.rs` | 2 | C08 / scripting-vm-integrator |

## Dependency-ordered landing sequence

0. Freeze and refresh. Confirm the Phase 5 green-scorecard/write-freeze gate; reparse the manifest; re-run collision detection; pin C07, C08, and C17 integrator identities.
1. Exception-note seed. C17 lands the 75 note-field justifications without changing `rust_module`. This makes intentional scatter explicit before the ratchet work starts.
2. Leaf batch A (parallel preparation). C14, C15, C11, C12, C13, and C16 prepare collision-free leaf consolidations. For any row touching multiple owned roots, the owners hand C17 one coordinated patch bundle; C17 lands the complete code move and manifest update in one atomic commit. Run the full gate battery and byte-stable goldens after every landing.
3. Major independent roots (parallel preparation). C01, C02, C03, and C06 prepare. Land C02 before C03 for their independent rows; assemble B6-0388 from both owners as one atomic code-plus-manifest commit. C06 prepares here but does not land until step 6. No worker other than its assigned owner authors changes to a listed root.
4. Runtime spine train. C04 and C05 prepare in parallel, then C17 lands artboard-only rows before draw-only rows. Rows present in both roots land as atomic bundles containing both owners' patches, the canonical target, and the manifest update; no intermediate commit may move only one fragment.
5. Crate-root integrators. C09 and C10 land root/facade cleanup. C07 alone performs every `nuxie-binary/src/lib.rs` edit. C08 alone performs every `nuxie-scripting/src/vm.rs` edit. C07 and C08 may prepare concurrently but land serially through C17.
6. Importer/root reconciliation. Rebase C06's still-unlanded preparation after C07's independent root rows. For rows touching both `lib.rs` and `importers/mod.rs`, C17 assembles C06 and C07 owner patches with the manifest update into one atomic commit; C06 remains the sole `importers/mod.rs` author and C07 remains the sole binary `lib.rs` author.
7. Final ratchet handoff. Reparse the manifest, prove only listed exceptions remain scattered, run the full gate/golden battery, and hand the resulting counts to MR-3. Do not invalidate `audit_record` or B6 verdicts for pure moves.

Cross-root rows are coordinated atomic bundles, never serial partial moves and never concurrent shared-root edits. Root owners author their portions; C17 assembles without rewriting them and lands the code plus manifest update as one commit. A worker that discovers behavior changes, signature changes, non-byte-stable goldens, or an unlisted collision stops and requeues the row as `split-needed`; it does not broaden a pure-move commit.

## Justified exceptions

Each note below is deliberately one line and is suitable for appending to that manifest row's `note` field. `Post-move retained modules` is computed per ownership boundary: basename-matching dedicated modules absorb same-crate aggregate fragments, while generated/root façade surfaces remain where their boundary requires them. The row is still an exception because the final mapping legitimately contains more than one file.

| Exception | Upstream row | Exception class | Manifest note text |
|---|---|---|---|
| E-B6-0068 | `src/animation/scripted_listener_action.cpp` | crate-bound trait/adapter split | Public façade/integration API and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0077 | `src/animation/state_machine_instance.cpp` | crate-bound trait/adapter split | Public façade/integration API and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0094 | `src/artboard.cpp` | crate-bound trait/adapter split | Public façade/integration API and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0106 | `src/assets/script_asset.cpp` | crate-bound trait/adapter split | Public host API, runtime trait implementation, and Luau VM adapter intentionally remain split across crate boundaries. |
| E-B6-0107 | `src/assets/shader_asset.cpp` | crate-bound trait/adapter split | Serialized asset representation and Luau VM host resource intentionally remain split across crate boundaries. |
| E-B6-0113 | `src/audio_event.cpp` | crate-bound trait/adapter split | Public façade/integration API and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0172 | `src/data_bind/converters/data_converter.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0173 | `src/data_bind/converters/data_converter_boolean_negate.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0174 | `src/data_bind/converters/data_converter_formula.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0175 | `src/data_bind/converters/data_converter_group.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0176 | `src/data_bind/converters/data_converter_group_item.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0177 | `src/data_bind/converters/data_converter_interpolator.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0178 | `src/data_bind/converters/data_converter_list_to_length.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0179 | `src/data_bind/converters/data_converter_number_to_list.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0180 | `src/data_bind/converters/data_converter_operation.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0181 | `src/data_bind/converters/data_converter_operation_value.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0182 | `src/data_bind/converters/data_converter_operation_viewmodel.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0183 | `src/data_bind/converters/data_converter_range_mapper.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0184 | `src/data_bind/converters/data_converter_rounder.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0185 | `src/data_bind/converters/data_converter_string_pad.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0186 | `src/data_bind/converters/data_converter_string_remove_zeros.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0187 | `src/data_bind/converters/data_converter_string_trim.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0188 | `src/data_bind/converters/data_converter_system_degs_to_rads.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0189 | `src/data_bind/converters/data_converter_system_normalizer.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0190 | `src/data_bind/converters/data_converter_to_number.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0191 | `src/data_bind/converters/data_converter_to_string.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0192 | `src/data_bind/converters/data_converter_trigger.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0193 | `src/data_bind/converters/formula/formula_token.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0194 | `src/data_bind/data_bind.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0195 | `src/data_bind/data_bind_container.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0196 | `src/data_bind/data_bind_context.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0200 | `src/data_bind/data_context.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0208 | `src/file.cpp` | schema/crate boundary | Public façade, binary loader surface, and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0213 | `src/importers/backboard_importer.cpp` | schema/crate boundary | Public façade, binary loader surface, and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0214 | `src/importers/bindable_property_importer.cpp` | crate-bound trait/adapter split | Binary importer ownership and runtime trait/state implementation intentionally remain split across crate boundaries. |
| E-B6-0228 | `src/importers/state_machine_importer.cpp` | schema/crate boundary | Binary importer ownership and runtime trait/state implementation intentionally remain split across crate boundaries. |
| E-B6-0232 | `src/importers/state_transition_importer.cpp` | schema/crate boundary | Binary importer ownership and runtime trait/state implementation intentionally remain split across crate boundaries. |
| E-B6-0234 | `src/importers/transition_viewmodel_condition_importer.cpp` | schema/crate boundary | Binary importer ownership and runtime trait/state implementation intentionally remain split across crate boundaries. |
| E-B6-0260 | `src/lua/logging_scripting_context.cpp` | crate-bound trait/adapter split | Public host integration and Luau VM binding intentionally remain split across crate boundaries. |
| E-B6-0261 | `src/lua/lua_artboards.cpp` | crate-bound trait/adapter split | Public host integration and Luau VM binding intentionally remain split across crate boundaries. |
| E-B6-0262 | `src/lua/lua_audio.cpp` | crate-bound trait/adapter split | Public host integration and Luau VM binding intentionally remain split across crate boundaries. |
| E-B6-0266 | `src/lua/lua_image_decode.cpp` | crate-bound trait/adapter split | Image codec backend and Luau VM binding intentionally remain split across crate boundaries. |
| E-B6-0269 | `src/lua/lua_properties.cpp` | shared VM lifecycle seam | ScriptViewModel property wrappers and parent-edge lifecycle share one retained state seam with E-B6-0288 across the VM root and view-model module; a per-row extraction would misattribute that shared state. |
| E-B6-0282 | `src/lua/renderer/lua_image.cpp` | crate-bound trait/adapter split | Runtime renderer state and Luau VM binding intentionally remain split across crate boundaries. |
| E-B6-0288 | `src/lua/rive_lua_libs.cpp` | shared VM lifecycle seam | Umbrella registration and retained view-model lifecycle share one state seam with E-B6-0269 across the VM root and view-model module; no behavior-neutral per-row owner can be isolated. |
| E-B6-0298 | `src/math/random.cpp` | target-specific impl split | Portable random API and cfg-specific native/Wasm implementations intentionally remain split by target. |
| E-B6-0319 | `src/script_input_trigger.cpp` | crate-bound trait/adapter split | Public façade/integration API and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0320 | `src/script_input_viewmodel_property.cpp` | crate-bound trait/adapter split | Public façade/integration API and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0321 | `src/scripted/scripted_data_converter.cpp` | crate-bound trait/adapter split | Public façade/integration API and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0322 | `src/scripted/scripted_drawable.cpp` | crate-bound trait/adapter split | Public façade/integration API and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0323 | `src/scripted/scripted_interpolator.cpp` | crate-bound trait/adapter split | Runtime, VM adapter, public façade, and golden-runner shim intentionally remain split across crate/tool boundaries. |
| E-B6-0324 | `src/scripted/scripted_layout.cpp` | crate-bound trait/adapter split | Public façade/integration API and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0325 | `src/scripted/scripted_object.cpp` | crate-bound trait/adapter split | Public façade/integration API and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0383 | `src/text/raw_text.cpp` | crate-bound trait/adapter split | Public façade/integration API and runtime implementation intentionally remain split across crate boundaries. |
| E-B6-0409 | `src/viewmodel/data_enum.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0410 | `src/viewmodel/data_enum_value.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0411 | `src/viewmodel/property_symbol_dependent.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0426 | `src/viewmodel/viewmodel.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0427 | `src/viewmodel/viewmodel_instance.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0428 | `src/viewmodel/viewmodel_instance_artboard.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0429 | `src/viewmodel/viewmodel_instance_asset.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0430 | `src/viewmodel/viewmodel_instance_asset_font.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0431 | `src/viewmodel/viewmodel_instance_asset_image.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0432 | `src/viewmodel/viewmodel_instance_boolean.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0433 | `src/viewmodel/viewmodel_instance_color.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0434 | `src/viewmodel/viewmodel_instance_enum.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0435 | `src/viewmodel/viewmodel_instance_list.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0436 | `src/viewmodel/viewmodel_instance_list_item.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0437 | `src/viewmodel/viewmodel_instance_number.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0438 | `src/viewmodel/viewmodel_instance_string.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0439 | `src/viewmodel/viewmodel_instance_symbol_list_index.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0440 | `src/viewmodel/viewmodel_instance_trigger.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0441 | `src/viewmodel/viewmodel_instance_value.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0442 | `src/viewmodel/viewmodel_instance_viewmodel.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0443 | `src/viewmodel/viewmodel_property.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0444 | `src/viewmodel/viewmodel_property_enum.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |
| E-B6-0445 | `src/viewmodel/viewmodel_property_enum_system.cpp` | schema/crate boundary | Schema-derived binary object surface and runtime behavior intentionally remain split across crate boundaries. |

### Post-move retained-module sets for exceptions

These sets are the expected final `rust_module` values after same-boundary aggregate fragments have been regrouped. They make the exception scope finite rather than grandfathering every current path.

| Exception | Post-move retained modules |
|---|---|
| E-B6-0068 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/state_machine/scripted_listener_action.rs` |
| E-B6-0077 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` |
| E-B6-0094 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/artboard.rs` |
| E-B6-0106 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/script_asset.rs`; `crates/nuxie-scripting/src/vm.rs` |
| E-B6-0107 | `crates/nuxie-binary/src/assets/shader_asset.rs`; `crates/nuxie-scripting/src/shader_asset.rs` |
| E-B6-0113 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/audio_event.rs` |
| E-B6-0172 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter.rs` |
| E-B6-0173 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_boolean_negate.rs` |
| E-B6-0174 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_formula.rs` |
| E-B6-0175 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_group.rs` |
| E-B6-0176 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_group_item.rs` |
| E-B6-0177 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_interpolator.rs` |
| E-B6-0178 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_list_to_length.rs` |
| E-B6-0179 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_number_to_list.rs` |
| E-B6-0180 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_operation.rs` |
| E-B6-0181 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_operation_value.rs` |
| E-B6-0182 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_operation_viewmodel.rs` |
| E-B6-0183 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_range_mapper.rs` |
| E-B6-0184 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_rounder.rs` |
| E-B6-0185 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_string_pad.rs` |
| E-B6-0186 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_string_remove_zeros.rs` |
| E-B6-0187 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_string_trim.rs` |
| E-B6-0188 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_system_degs_to_rads.rs` |
| E-B6-0189 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_system_normalizer.rs` |
| E-B6-0190 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_to_number.rs` |
| E-B6-0191 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/data_converter_to_string.rs` |
| E-B6-0192 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_converter_trigger.rs` |
| E-B6-0193 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/converters/formula/formula_token.rs` |
| E-B6-0194 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/data_bind.rs` |
| E-B6-0195 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/data_bind_container.rs` |
| E-B6-0196 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/data_bind_context.rs` |
| E-B6-0200 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/data_bind/data_context.rs` |
| E-B6-0208 | `crates/nuxie/src/lib.rs`; `crates/nuxie-binary/src/file.rs`; `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/lib.rs` |
| E-B6-0213 | `crates/nuxie/src/lib.rs`; `crates/nuxie-binary/src/importers/backboard_importer.rs`; `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/state_machine/data_bind_template.rs` |
| E-B6-0214 | `crates/nuxie-binary/src/importers/bindable_property_importer.rs`; `crates/nuxie-runtime/src/state_machine/bindables.rs` |
| E-B6-0228 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/importers/state_machine_importer.rs` |
| E-B6-0232 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/state_machine/state_transition.rs` |
| E-B6-0234 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/state_machine/transition_viewmodel_condition.rs` |
| E-B6-0260 | `crates/nuxie/src/lib.rs`; `crates/nuxie-scripting/src/vm/logging_scripting_context.rs` |
| E-B6-0261 | `crates/nuxie/src/lib.rs`; `crates/nuxie-scripting/src/vm/lua_artboards.rs` |
| E-B6-0262 | `crates/nuxie/src/lib.rs`; `crates/nuxie-scripting/src/vm/lua_audio.rs` |
| E-B6-0266 | `crates/nuxie-image-codec/src/lib.rs`; `crates/nuxie-scripting/src/vm/lua_image_decode.rs` |
| E-B6-0269 | `crates/nuxie-scripting/src/vm.rs`; `crates/nuxie-scripting/src/vm/view_model.rs` |
| E-B6-0282 | `crates/nuxie-runtime/src/lua/renderer/lua_image.rs`; `crates/nuxie-scripting/src/vm/lua_image.rs` |
| E-B6-0288 | `crates/nuxie-scripting/src/vm.rs`; `crates/nuxie-scripting/src/vm/view_model.rs` |
| E-B6-0298 | `crates/nuxie-runtime/src/math/random.rs`; `crates/nuxie-runtime/src/math/random/native.rs`; `crates/nuxie-runtime/src/math/random/wasm.rs` |
| E-B6-0319 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/script_input_trigger.rs` |
| E-B6-0320 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/script_input_viewmodel_property.rs` |
| E-B6-0321 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/scripted_data_converter.rs` |
| E-B6-0322 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/scripted/scripted_drawable.rs` |
| E-B6-0323 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/scripted_interpolator.rs`; `crates/nuxie-scripting/src/vm.rs`; `tools/rust-golden-runner/src/main.rs` |
| E-B6-0324 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/scripted_layout.rs` |
| E-B6-0325 | `crates/nuxie/src/lib.rs`; `crates/nuxie-runtime/src/scripted_object.rs` |
| E-B6-0383 | `crates/nuxie/src/raw_text.rs`; `crates/nuxie-runtime/src/text/raw_text.rs` |
| E-B6-0409 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/data_enum.rs` |
| E-B6-0410 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/data_enum_value.rs` |
| E-B6-0411 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/property_symbol_dependent.rs` |
| E-B6-0426 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel.rs` |
| E-B6-0427 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance.rs` |
| E-B6-0428 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_artboard.rs` |
| E-B6-0429 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_asset.rs` |
| E-B6-0430 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_asset_font.rs` |
| E-B6-0431 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_asset_image.rs` |
| E-B6-0432 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_boolean.rs` |
| E-B6-0433 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_color.rs` |
| E-B6-0434 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_enum.rs` |
| E-B6-0435 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_list.rs` |
| E-B6-0436 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_list_item.rs` |
| E-B6-0437 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_number.rs` |
| E-B6-0438 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_string.rs` |
| E-B6-0439 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_symbol_list_index.rs` |
| E-B6-0440 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_trigger.rs` |
| E-B6-0441 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_value.rs` |
| E-B6-0442 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_instance_viewmodel.rs` |
| E-B6-0443 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_property.rs` |
| E-B6-0444 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_property_enum.rs` |
| E-B6-0445 | `crates/nuxie-binary/src/lib.rs`; `crates/nuxie-runtime/src/viewmodel/viewmodel_property_enum_system.rs` |

No current covered row maps to a C ABI crate, so there is no C ABI-shim exception in this snapshot. If a later reparse introduces one, it must name the ABI boundary and remain under a single `nux-capi` integrator.

## MR-2 completion checks

- Every table row is reconciled against the manifest after rebase; duplicate table appearances do not create duplicate work.
- All 191 actionable targets still pass the no-duplicate/no-unowned-existing collision check.
- Each shared root is edited only by the worker in the ownership ledger; C07 and C08 are hard single-writer lanes.
- Every atomic landing commit updates only its affected manifest rows in lockstep with the complete code move, preserves ticket/audit attribution, runs the full gate battery, and proves goldens byte-stable.
- Final scatter equals the exception list (or a smaller reviewed subset); any new exception requires an explicit one-line manifest note and MR-1 plan amendment.
