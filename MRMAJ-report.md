# MR-2 major-root report: C01/runtime-animation

Owned root: `crates/nuxie-runtime/src/animation.rs`

## Moved

| Row | Target | Result |
|---|---|---|
| `B6-0060` | `crates/nuxie-runtime/src/nested_bool.rs` | Correspondence consolidated to the dedicated module. The complete row-specific implementation already resided in the target at the starting snapshot; no item body remained in `animation.rs` to relocate. |
| `B6-0062` | `crates/nuxie-runtime/src/nested_number.rs` | Correspondence consolidated to the dedicated module. The complete row-specific implementation already resided in the target at the starting snapshot; no item body remained in `animation.rs` to relocate. |
| `B6-0066` | `crates/nuxie-runtime/src/nested_trigger.rs` | Correspondence consolidated to the dedicated module. The complete row-specific implementation already resided in the target at the starting snapshot; no item body remained in `animation.rs` to relocate. |

The three manifest edits change only `rust_module`. Their `audit_record` values and B6 verdicts remain unchanged. Existing declarations in `crates/nuxie-runtime/src/lib.rs` already wire all three dedicated modules, so no C09-owned root edit was required.

## Split-needed

The plan classifies these 38 rows as non-mechanical partitions. They were not forced into pure-move changes:

| Rows | Planned targets |
|---|---|
| `B6-0004`, `B6-0005`, `B6-0006`, `B6-0007`, `B6-0008` | `animation_state.rs`, `animation_state_instance.rs`, `blend_animation.rs`, `blend_animation_1d.rs`, `blend_animation_direct.rs` |
| `B6-0009`, `B6-0010`, `B6-0011`, `B6-0012`, `B6-0013` | `blend_state.rs`, `blend_state_1d.rs`, `blend_state_1d_input.rs`, `blend_state_1d_instance.rs`, `blend_state_1d_viewmodel.rs` |
| `B6-0014`, `B6-0016`, `B6-0017`, `B6-0018`, `B6-0019` | `blend_state_direct.rs`, `blend_state_transition.rs`, `cubic_ease_interpolator.rs`, `cubic_interpolator.rs`, `cubic_interpolator_component.rs` |
| `B6-0020`, `B6-0021`, `B6-0022`, `B6-0023`, `B6-0029` | `cubic_interpolator_solver.rs`, `cubic_value_interpolator.rs`, `elastic_ease.rs`, `elastic_interpolator.rs`, `interpolating_keyframe.rs` |
| `B6-0031`, `B6-0032`, `B6-0033`, `B6-0034`, `B6-0035` | `keyed_object.rs`, `keyed_property.rs`, `keyframe.rs`, `keyframe_bool.rs`, `keyframe_callback.rs` |
| `B6-0036`, `B6-0037`, `B6-0038`, `B6-0039`, `B6-0040` | `keyframe_color.rs`, `keyframe_double.rs`, `keyframe_id.rs`, `keyframe_interpolator.rs`, `keyframe_string.rs` |
| `B6-0041`, `B6-0043`, `B6-0044`, `B6-0059`, `B6-0061` | `keyframe_uint.rs`, `linear_animation.rs`, `linear_animation_instance.rs`, `nested_animation.rs`, `nested_linear_animation.rs` |
| `B6-0063`, `B6-0064`, `B6-0221` | `nested_remap_animation.rs`, `nested_simple_animation.rs`, `importers/keyed_property_importer.rs` |

## Skipped

| Row | Reason |
|---|---|
| `B6-0323` | Planned exception `E-B6-0323` (`scripted_interpolator.cpp`); its crate/tool-bound implementation is intentionally scattered. No manifest or code change was made. |

## Cross-root queue

| Row | Foreign-owned roots/fragments left untouched |
|---|---|
| `B6-0323` | `crates/nuxie-runtime/src/lib.rs` (C09), `crates/nuxie-scripting/src/vm.rs` (C08), and `crates/nuxie/src/lib.rs` (C10). The additional `scripting.rs`, dedicated runtime leaf, and golden-runner fragments remain part of exception `E-B6-0323`. |

## Gate and commit

- Gate: passed `cargo check --workspace --exclude nux-capi` (45.94s; warnings only).
- Commit: succeeded with subject `[MR-2/C01] Move B6-0060 B6-0062 B6-0066 from animation.rs`.
