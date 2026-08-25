use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("silver-corpus workspace root")
        .to_path_buf()
}

fn runtime_root(test: &str) -> Option<PathBuf> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").map(PathBuf::from);
    if root.is_none() {
        eprintln!(
            "skipping {test}; RIVE_RUNTIME_DIR is unset; point it at the pinned rive-runtime checkout"
        );
    }
    root
}

fn compare_case(id: &str) -> anyhow::Result<()> {
    let Some(runtime) = runtime_root(id) else {
        return Ok(());
    };
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))?;
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .ok_or_else(|| anyhow::anyhow!("missing silver case {id}"))?;
    let actual = parse_sriv(Execution::run(case, &runtime)?.bytes())?;
    let expected = parse_sriv(&std::fs::read(resolve_expected(&runtime, case))?)?;
    compare_sriv(&expected, &actual).map_err(|difference| anyhow::anyhow!("{id}: {difference}"))
}

#[test]
fn clip_apply_outside_hierarchy() {
    compare_case("clipping_and_draw_order").unwrap();
}

#[test]
fn clip_animated_nodes() {
    compare_case("animated_clipping-nodes").unwrap();
}

#[test]
fn clip_animated_layouts() {
    compare_case("animated_clipping-layout").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for component_list_virtualized_scroll_manual"]
fn component_list_virtualized_scroll_manual() {
    compare_case("component_list_virtualized_scroll_manual").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for artboard_list_overrides_horizontal"]
fn component_list_override_horizontal() {
    compare_case("artboard_list_overrides_horizontal").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for artboard_list_overrides_vertical"]
fn component_list_override_vertical() {
    compare_case("artboard_list_overrides_vertical").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for reset_phase_multi_main"]
fn component_list_reset_triggers() {
    compare_case("reset_phase_multi_main").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for component_list_grouped"]
fn component_list_non_layout_position() {
    compare_case("component_list_grouped").unwrap();
}

#[test]
fn component_list_follow_path() {
    compare_case("component_list_follow_path").unwrap();
}

#[test]
fn component_list_follow_path_distance() {
    compare_case("component_list_follow_path_distance").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for component_list_hit_order"]
fn component_list_hit_order() {
    compare_case("component_list_hit_order").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for virtualized_artboard_databound_children"]
fn component_list_virtualized_nested_data_binding() {
    compare_case("virtualized_artboard_databound_children").unwrap();
}

#[test]
fn component_list_map_rules() {
    compare_case("artboard_list_map_rules").unwrap();
}

#[test]
fn component_list_stateful_component() {
    compare_case("component_stateful").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for component_list_child_origin"]
fn component_list_child_origin() {
    compare_case("component_list_child_origin").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for draw_index_list"]
fn component_list_draw_index_order() {
    compare_case("draw_index_list").unwrap();
}

#[test]
fn component_origin_animated_clicks() {
    compare_case("nested_artboard_origin_override_test").unwrap();
}

#[test]
fn component_stateful_view_model() {
    compare_case("component_stateful_vm_instance").unwrap();
}

#[test]
fn component_stateful_view_model_multi() {
    compare_case("component_stateful_vm_instance_2").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for stateful_multi_property"]
fn component_stateful_multi_property() {
    compare_case("stateful_multi_property").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for stateful_nested"]
fn component_stateful_nested() {
    compare_case("stateful_nested").unwrap();
}

#[test]
#[ignore = "expected-red: silver replay lacks stateful_list_props_lifecycle required runtime feature"]
fn component_stateful_list_cleanup() {
    compare_case("stateful_list_props_lifecycle").unwrap();
}

#[test]
fn component_stateful_keyed_triggers() {
    compare_case("stateful_keyed_trigger").unwrap();
}

#[test]
fn data_bind_lists_reset_triggers() {
    compare_case("viewmodel_list_trigger").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for number_to_list_nested_children"]
fn data_bind_lists_number_to_list_children() {
    compare_case("number_to_list_nested_children").unwrap();
}

#[test]
fn data_bind_lists_add_remove_item() {
    compare_case("list_items").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for clear_viewmodel_list"]
fn data_bind_lists_clear() {
    compare_case("clear_viewmodel_list").unwrap();
}

#[test]
#[ignore = "expected-red: silver replay lacks data_bind_blob_test required runtime feature"]
fn data_binding_blobs_internal_external() {
    compare_case("data_bind_blob_test").unwrap();
}

#[test]
#[ignore = "expected-red: pinned SRIV stream diverges for computed_values_test"]
fn data_binding_computed_root_values() {
    compare_case("computed_values_test").unwrap();
}

#[test]
#[ignore = "expected-red: silver replay lacks image_computed_transform_bind required runtime feature"]
fn data_binding_computed_image_resize() {
    compare_case("image_computed_transform_bind").unwrap();
}
