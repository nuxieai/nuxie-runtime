use super::*;
use crate::RuntimeImageAssetOwners;
use nuxie_binary::read_runtime_file;
use nuxie_graph::{ArtboardGraph, GraphFile};
use std::{path::PathBuf, sync::Arc};

fn fixture(name: &str) -> (RuntimeFile, GraphFile) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let path = root.join("tests/unit_tests/assets").join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let file = read_runtime_file(&bytes)
        .unwrap_or_else(|error| panic!("import {}: {error:#}", path.display()));
    let graphs = GraphFile::from_runtime_file(&file).expect("build pinned graph");
    (file, graphs)
}

fn local_named(graph: &ArtboardGraph, kind: &str, name: &str) -> usize {
    graph
        .local_objects
        .iter()
        .find(|object| object.type_name == Some(kind) && object.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing {kind} named {name}"))
        .local_id
}

fn only_local(graph: &ArtboardGraph, kind: &str) -> usize {
    let locals = graph
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some(kind))
        .map(|object| object.local_id)
        .collect::<Vec<_>>();
    assert_eq!(locals.len(), 1, "exactly one {kind}");
    locals[0]
}

fn property_key(type_name: &str, name: &str) -> u16 {
    crate::properties::property_key_for_name(type_name, name)
        .unwrap_or_else(|| panic!("missing {type_name}.{name}"))
}

fn instantiate(file: &RuntimeFile, graphs: &GraphFile, index: usize) -> ArtboardInstance {
    ArtboardInstance::from_graph_with_artboards(file, &graphs.artboards[index], &graphs.artboards)
        .expect("instantiate pinned artboard")
}

#[test]
fn wave_c2_layout_006_intrinsic_text_size_from_retained_text_owner() {
    let (file, graphs) = fixture("layout/measure_tests.riv");
    let graph_index = graphs
        .artboards
        .iter()
        .position(|graph| graph.name.as_deref() == Some("hi"))
        .expect("hi artboard");
    let graph = &graphs.artboards[graph_index];
    let text_local = local_named(graph, "Text", "HiText");
    let mut instance = instantiate(&file, &graphs, graph_index);
    instance.advance(0.0).expect("Artboard::advance(0)");
    let bounds = instance
        .component(text_local)
        .and_then(|component| component.concrete.text.as_ref())
        .and_then(|text| text.bounds())
        .expect("live Text::localBounds owner after advance");
    assert_eq!(bounds, (0.0, 0.0, 62.48047, 72.62695));
}

#[test]
fn wave_c2_library_002_named_nested_owner_retains_exact_simple_animation() {
    let (file, graphs) = fixture("library_export_animation_test.riv");
    let host = &graphs.artboards[0];
    let nested_local = local_named(host, "NestedArtboard", "The nested artboard");
    let nested_animation_local = only_local(host, "NestedSimpleAnimation");
    let instance = instantiate(&file, &graphs, 0);

    let nested = instance
        .nested_artboards
        .get(&nested_local)
        .expect("named live NestedArtboard occurrence");
    assert_eq!(
        instance.string_property(nested_local, property_key("NestedArtboard", "name")),
        Some("The nested artboard".as_bytes())
    );
    assert_eq!(
        instance.double_property(nested_local, property_key("NestedArtboard", "x")),
        Some(1.0)
    );
    assert_eq!(
        instance.double_property(nested_local, property_key("NestedArtboard", "y")),
        Some(2.0)
    );
    assert_eq!(
        instance.uint_property(nested_local, property_key("NestedArtboard", "artboardId")),
        Some(1)
    );
    assert_eq!(nested.child.linear_animations().len(), 1);
    assert_eq!(file.file_assets().len(), 0);
    assert_eq!(nested.animations.len(), 1);
    let RuntimeNestedAnimationInstance::Simple {
        local_id,
        animation,
        ..
    } = &nested.animations[0]
    else {
        panic!("exact nested owner must retain a simple animation");
    };
    assert_eq!(*local_id, nested_animation_local);
    assert_eq!(
        instance.string_property(
            nested_animation_local,
            property_key("NestedSimpleAnimation", "name")
        ),
        Some("".as_bytes())
    );
    assert_eq!(animation.animation_index(), 0);
    assert_eq!(
        nested
            .child
            .linear_animation(0)
            .and_then(|value| value.name.as_deref()),
        Some("LA Rocket")
    );
}

#[test]
fn wave_c2_library_003_named_nested_owner_retains_exact_state_machine() {
    let (file, graphs) = fixture("library_export_state_machine_test.riv");
    let host = &graphs.artboards[0];
    let nested_local = local_named(host, "NestedArtboard", "The nested artboard");
    let nested_state_machine_local = only_local(host, "NestedStateMachine");
    let instance = instantiate(&file, &graphs, 0);

    let nested = instance
        .nested_artboards
        .get(&nested_local)
        .expect("named live NestedArtboard occurrence");
    assert_eq!(
        instance.string_property(nested_local, property_key("NestedArtboard", "name")),
        Some("The nested artboard".as_bytes())
    );
    assert_eq!(
        instance.double_property(nested_local, property_key("NestedArtboard", "x")),
        Some(1.0)
    );
    assert_eq!(
        instance.double_property(nested_local, property_key("NestedArtboard", "y")),
        Some(2.0)
    );
    assert_eq!(
        instance.uint_property(nested_local, property_key("NestedArtboard", "artboardId")),
        Some(1)
    );
    assert_eq!(nested.child.state_machines().len(), 1);
    assert_eq!(file.file_assets().len(), 0);
    assert_eq!(nested.animations.len(), 1);
    let RuntimeNestedAnimationInstance::StateMachine(state_machine) = &nested.animations[0] else {
        panic!("exact nested owner must retain a state machine");
    };
    assert_eq!(state_machine.local_id(), nested_state_machine_local);
    assert_eq!(
        instance.string_property(
            nested_state_machine_local,
            property_key("NestedStateMachine", "name")
        ),
        Some("".as_bytes())
    );
    assert_eq!(state_machine.animation_id(), 0);
    assert!(state_machine.has_state_machine());
    assert_eq!(
        nested
            .child
            .state_machine(0)
            .and_then(|value| value.name.as_deref()),
        Some("SM Rocket")
    );
}

fn assert_live_image_asset_owner(
    file: &RuntimeFile,
    graphs: &GraphFile,
    graph_index: usize,
    asset_name: &str,
    expected_image_asset_id: Option<u64>,
) {
    let graph = &graphs.artboards[graph_index];
    let image_local = only_local(graph, "Image");
    let mut instance = instantiate(file, graphs, graph_index);
    instance.attach_runtime_image_assets_tree(Arc::new(RuntimeImageAssetOwners::default()));
    if let Some(expected_image_asset_id) = expected_image_asset_id {
        assert_eq!(
            instance.uint_property(image_local, property_key("Image", "assetId")),
            Some(expected_image_asset_id)
        );
    }
    let asset_global = instance
        .runtime_images
        .asset_global_for_test(image_local)
        .expect("live Image::imageAsset owner");
    let asset = file
        .object(asset_global as usize)
        .expect("exact ImageAsset");
    assert_eq!(asset.type_name, "ImageAsset");
    assert_eq!(asset.string_property("name"), Some(asset_name));
}

#[test]
fn wave_c2_library_006_live_library_image_retains_exact_asset_owner() {
    let (file, graphs) = fixture("library_with_image.riv");
    assert_eq!(file.file_assets().len(), 1);
    let host = &graphs.artboards[0];
    let nested_local = local_named(host, "NestedArtboard", "The instance");
    let host_instance = instantiate(&file, &graphs, 0);
    let child_global = host_instance
        .nested_artboards
        .get(&nested_local)
        .expect("live nested source")
        .child
        .graph_global_id;
    let child_index = graphs
        .artboards
        .iter()
        .position(|graph| graph.global_id == child_global)
        .expect("nested source graph");
    assert_live_image_asset_owner(&file, &graphs, child_index, "MyImageAsset", Some(0));
}

#[test]
fn wave_c2_library_007_each_live_library_image_retains_its_exact_asset_owner() {
    let (file, graphs) = fixture("double_library_with_image.riv");
    assert_eq!(file.file_assets().len(), 2);
    let host = &graphs.artboards[0];
    let host_instance = instantiate(&file, &graphs, 0);
    for (nested_name, asset_name) in [
        ("The nested artboard", "MyFirstImageAsset"),
        ("Another nested artboard", "MyOtherImageAsset"),
    ] {
        let nested_local = local_named(host, "NestedArtboard", nested_name);
        let child_global = host_instance
            .nested_artboards
            .get(&nested_local)
            .expect("live nested source")
            .child
            .graph_global_id;
        let child_index = graphs
            .artboards
            .iter()
            .position(|graph| graph.global_id == child_global)
            .expect("nested source graph");
        assert_live_image_asset_owner(&file, &graphs, child_index, asset_name, None);
    }
}

#[test]
fn wave_c2_library_009_nested_owners_exist_before_root_advance() {
    let (file, graphs) = fixture("library_view_model_test.riv");
    let root_graph = &graphs.artboards[0];
    let root_nested_local = only_local(root_graph, "NestedArtboard");
    let mut root = instantiate(&file, &graphs, 0);
    let middle = root
        .nested_artboards
        .get(&root_nested_local)
        .expect("level-two live occurrence exists before advance");
    let middle_graph = graphs
        .artboards
        .iter()
        .find(|graph| graph.global_id == middle.child.graph_global_id)
        .expect("middle graph");
    assert_eq!(middle_graph.name.as_deref(), Some("2"));
    let middle_nested_local = only_local(middle_graph, "NestedArtboard");
    let leaf = middle
        .child
        .nested_artboards
        .get(&middle_nested_local)
        .expect("level-one live occurrence exists before advance");
    let leaf_graph = graphs
        .artboards
        .iter()
        .find(|graph| graph.global_id == leaf.child.graph_global_id)
        .expect("leaf graph");
    assert_eq!(leaf_graph.name.as_deref(), Some("1"));
    root.advance(0.0)
        .expect("advance after both exact owners are retained");
}

#[test]
fn wave_c2_library_010_nested_shape_retains_exact_first_fill_owner() {
    let (file, graphs) = fixture("library_vmtest_1_host.riv");
    let root_graph = &graphs.artboards[0];
    let nested_local = only_local(root_graph, "NestedArtboard");
    let mut root = instantiate(&file, &graphs, 0);
    let view_model_index = file
        .resolved_view_model_for_artboard(0)
        .expect("host ViewModel")
        .view_model_index;
    let instance_index = file
        .view_model_default_instance(view_model_index)
        .expect("host default ViewModel instance")
        .instance_index;
    let view_model = crate::RuntimeOwnedViewModelHandle::new(
        crate::RuntimeOwnedViewModelInstance::from_instance(
            &file,
            view_model_index,
            instance_index,
        )
        .expect("instantiate host default ViewModel"),
    );
    let mut bound = root.bind_default_view_model_artboard_list_context(&file);
    bound |= root.bind_owned_view_model_artboard_handle(&file, &view_model);
    assert!(bound, "bind exact default ViewModel instance");
    let nested = root
        .nested_artboards
        .get(&nested_local)
        .expect("lib2 live occurrence exists before advance");
    let child_graph = graphs
        .artboards
        .iter()
        .find(|graph| graph.global_id == nested.child.graph_global_id)
        .expect("lib2 source graph");
    assert_eq!(child_graph.name.as_deref(), Some("lib2artboard"));
    assert_eq!(child_graph.shape_paint_containers.len(), 1);
    let shape_local = child_graph.shape_paint_containers[0].local_id;
    assert_eq!(child_graph.shape_paint_containers[0].paints.len(), 1);
    let first_paint = &child_graph.shape_paint_containers[0].paints[0];
    let fill_local = first_paint.local_id;
    let solid_local = first_paint
        .mutator_local
        .expect("exact first Fill retains its SolidColor mutator");
    let live_shape = nested
        .child
        .runtime_shapes
        .get(shape_local)
        .expect("live Shape owner");
    assert_eq!(live_shape.paint_owners.len(), 1);
    assert_eq!(live_shape.paint_owners[0].paint_local, fill_local);
    assert_eq!(
        nested
            .child
            .component(fill_local)
            .map(|component| component.type_name),
        Some("Fill")
    );
    root.advance(0.0)
        .expect("advance after exact nested/paint owners are retained");
    let nested = root
        .nested_artboards
        .get(&nested_local)
        .expect("lib2 live occurrence remains after bind/advance");
    let live_shape = nested
        .child
        .runtime_shapes
        .get(shape_local)
        .expect("live Shape owner after bind/advance");
    assert_eq!(live_shape.paint_owners.len(), 1);
    assert_eq!(live_shape.paint_owners[0].paint_local, fill_local);
    assert_eq!(
        nested
            .child
            .component(solid_local)
            .map(|component| component.type_name),
        Some("SolidColor")
    );
    assert_eq!(
        nested
            .child
            .color_property(solid_local, property_key("SolidColor", "colorValue")),
        Some(0xff10_1566)
    );
}

#[test]
#[ignore = "expected-red: exact styled_flex runtime solve offsets the child by its margins in addition to the pinned padding-only expectation"]
fn wave_c2_layout_023_live_parent_selects_padding_child() {
    let (file, graphs) = fixture("layout/styled_flex.riv");
    let mut instance = instantiate(&file, &graphs, 0);
    instance.advance(0.0).expect("Artboard::advance(0)");
    let child_local = instance
        .components()
        .iter()
        .filter(|component| component.type_name == "LayoutComponent")
        .filter(|component| {
            instance
                .layout_component_style_local(component.local_id)
                .is_some()
        })
        .find_map(|component| {
            let parent_local = instance.component_parent_local(component.local_id)?;
            (parent_local != 0
                && instance
                    .component(parent_local)
                    .is_some_and(|parent| parent.type_name == "LayoutComponent"))
            .then_some(component.local_id)
        })
        .expect("live styled LayoutComponent whose live parent is a non-Artboard LayoutComponent");
    let parent_local = instance
        .component_parent_local(child_local)
        .expect("live parent relation");
    assert_eq!(
        instance
            .component(parent_local)
            .map(|parent| parent.type_name),
        Some("LayoutComponent")
    );
    assert_ne!(parent_local, 0);
    let child = instance
        .layout_bounds(child_local)
        .expect("live child LayoutComponent bounds");
    assert_eq!(
        (child.x, child.y, child.width, child.height),
        (10.0, 20.0, 160.0, 140.0)
    );
}
