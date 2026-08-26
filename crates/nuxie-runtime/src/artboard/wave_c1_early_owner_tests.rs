//! Exact owner-level Wave C1 evidence for image, instancing, joystick, and grid cases.

use super::*;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use nuxie_binary::{read_runtime_file, RuntimeFile, RuntimeObject};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_render_api::RecordingFactory;

use crate::RuntimeFileAssetOwners;

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn load_graph(name: &str) -> (RuntimeFile, GraphFile) {
    let file = read_runtime_file(&pinned_fixture(name))
        .unwrap_or_else(|error| panic!("{name} imports: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("{name} graph builds: {error:#}"));
    (file, graphs)
}

fn first_graph(name: &str) -> (RuntimeFile, GraphFile, usize) {
    let (file, graphs) = load_graph(name);
    assert!(!graphs.artboards.is_empty(), "{name} has an artboard");
    (file, graphs, 0)
}

fn named_local(graph: &ArtboardGraph, name: &str, type_name: &str) -> usize {
    graph
        .local_objects
        .iter()
        .find(|object| object.name.as_deref() == Some(name) && object.type_name == Some(type_name))
        .unwrap_or_else(|| panic!("missing {type_name} named {name}"))
        .local_id
}

fn object_at_local<'a>(
    file: &'a RuntimeFile,
    graph: &ArtboardGraph,
    local_id: usize,
) -> &'a RuntimeObject {
    let global_id = graph
        .local_objects
        .get(local_id)
        .unwrap_or_else(|| panic!("missing local object {local_id}"))
        .global_id;
    file.object(global_id as usize)
        .unwrap_or_else(|| panic!("missing global object {global_id}"))
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    crate::properties::property_key_for_name(type_name, property_name)
        .unwrap_or_else(|| panic!("property {type_name}.{property_name}"))
}

fn source_occurrence(
    file: &RuntimeFile,
    graphs: &GraphFile,
    graph_index: usize,
) -> ArtboardInstance {
    ArtboardInstance::from_graph_with_artboards(
        file,
        &graphs.artboards[graph_index],
        &graphs.artboards,
    )
    .expect("artboard instantiates")
}

fn prepare_source_meshes(
    file: &RuntimeFile,
    graphs: &GraphFile,
    graph_index: usize,
    source: &ArtboardInstance,
) {
    let mut factory = RecordingFactory::new();
    let _resources = crate::draw::preallocate_render_paint_cache_for_artboard_tree(
        file,
        source,
        &graphs.artboards[graph_index],
        &graphs.artboards,
        &mut factory,
    );
}

#[test]
#[ignore = "expected-red: fallback ImageAsset::decodedByteSize is decoded RGBA length, not the pinned 308-byte source payload"]
fn wave_c1_in_band_asset_003_fallback_live_decoded_byte_size_owner() {
    let file = read_runtime_file(&pinned_fixture("in_band_asset.riv")).expect("fixture imports");
    let asset = file.file_assets()[0];
    assert_eq!(asset.type_name, "ImageAsset");
    let mut attempted_bytes = 0;
    let mut loader = |_asset: &crate::RuntimeFileAsset,
                      bytes: &[u8],
                      _factory: &mut dyn nuxie_render_api::Factory| {
        attempted_bytes = bytes.len();
        false
    };
    let mut factory = RecordingFactory::new();
    let owners = RuntimeFileAssetOwners::import_with_loader(&file, None, &mut factory, &mut loader);

    assert_eq!(attempted_bytes, 308);
    assert!(owners.image_assets().get(asset.id).is_some());
    assert_eq!(
        owners.image_assets().decoded_byte_length_for_test(asset.id),
        Some(308),
        "fallback decode retains the exact source byte count on the live owner",
    );
}

#[test]
#[ignore = "expected-red: the live ImageAsset owner does not retain pinned decoded source byte size"]
fn wave_c1_image_mesh_001_image_with_mesh_loads_correctly() {
    let (file, graphs, graph_index) = first_graph("tape.riv");
    let graph = &graphs.artboards[graph_index];
    let source = source_occurrence(&file, &graphs, graph_index);
    prepare_source_meshes(&file, &graphs, graph_index, &source);

    let image_local = named_local(graph, "Tape body.png", "Image");
    let image = object_at_local(&file, graph, image_local);
    let image_asset = file
        .resolved_file_asset_for_referencer(image)
        .expect("Tape body.png Image occurrence resolves its ImageAsset");
    let mesh = graph.meshes.first().expect("Tape body.png owns a Mesh");
    assert_eq!(
        source.runtime_images.asset_global_for_test(image_local),
        Some(image_asset.id),
        "the live Image occurrence retains its decoded ImageAsset owner",
    );
    assert_eq!(
        source.runtime_images.mesh(image_local),
        Some(crate::draw::image::RuntimeImageMeshOwner::Mesh(
            mesh.local_id
        )),
        "the live Image occurrence retains its exact Mesh owner",
    );
    assert_eq!(mesh.vertices.len(), 24);
    assert_eq!(
        source
            .runtime_meshes
            .mesh(mesh.local_id)
            .expect("live Mesh occurrence")
            .shared
            .borrow()
            .borrow()
            .index_count,
        31 * 3,
    );

    let assets = source
        .scripted_runtime_image_assets()
        .expect("file-owned ImageAsset arena is attached");
    assert_eq!(
        assets.decoded_byte_length_for_test(image_asset.id),
        Some(70_903),
        "pinned ImageAsset::decodedByteSize retains the decoded source byte count",
    );
}

#[test]
fn wave_c1_image_mesh_002_duplicating_a_mesh_shares_the_indices() {
    let (file, graphs, graph_index) = first_graph("tape.riv");
    let graph = &graphs.artboards[graph_index];
    let source = source_occurrence(&file, &graphs, graph_index);
    prepare_source_meshes(&file, &graphs, graph_index, &source);

    let image_local = named_local(graph, "Tape body.png", "Image");
    let mesh = graph.meshes.first().expect("Tape body.png owns a Mesh");
    let instances = [source.clone(), source.clone(), source.clone()];
    let asset_owners = instances.each_ref().map(|instance| {
        instance
            .scripted_runtime_image_assets()
            .expect("clone retains file-owned ImageAsset arena")
    });
    assert!(Arc::ptr_eq(&asset_owners[0], &asset_owners[1]));
    assert!(Arc::ptr_eq(&asset_owners[1], &asset_owners[2]));

    let shared_indices = instances.each_ref().map(|instance| {
        assert!(instance
            .runtime_images
            .asset_global_for_test(image_local)
            .is_some());
        assert_eq!(
            instance.runtime_images.mesh(image_local),
            Some(crate::draw::image::RuntimeImageMeshOwner::Mesh(
                mesh.local_id
            )),
        );
        let shared = Rc::clone(
            &instance
                .runtime_meshes
                .mesh(mesh.local_id)
                .expect("clone-owned Mesh occurrence")
                .shared
                .borrow(),
        );
        assert_eq!(shared.borrow().index_count, 31 * 3);
        shared
    });
    assert!(Rc::ptr_eq(&shared_indices[0], &shared_indices[1]));
    assert!(Rc::ptr_eq(&shared_indices[1], &shared_indices[2]));
}

#[test]
fn wave_c1_instancing_002_artboard_clone_preserves_clipping_properties() {
    let (file, graphs, graph_index) = first_graph("circle_clips.riv");
    let graph = &graphs.artboards[graph_index];
    let definition = source_occurrence(&file, &graphs, graph_index);
    let mut artboard = definition.clone();
    let node = named_local(graph, "TopEllipse", "Shape");

    let clipping_sources = artboard
        .component(node)
        .and_then(|component| component.concrete.drawable.as_ref())
        .expect("live TopEllipse Drawable occurrence")
        .clipping_shapes
        .iter()
        .map(|handle| {
            let clipping_local = artboard
                .component_local_id(*handle)
                .expect("clone-local ClippingShape handle");
            let source_local = graph
                .clipping_shapes
                .iter()
                .find(|clipping| clipping.local_id == clipping_local)
                .and_then(|clipping| clipping.source_local)
                .expect("ClippingShape source owner");
            graph.local_objects[source_local]
                .name
                .as_deref()
                .expect("clipping source name")
        })
        .collect::<Vec<_>>();
    assert_eq!(clipping_sources, ["ClipRect2", "BabyEllipse"]);

    artboard.update_pass();
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    artboard
        .draw_artboard(
            &file,
            graph,
            &graphs.artboards,
            &mut factory,
            &mut renderer,
            &BTreeMap::new(),
            None,
            true,
        )
        .expect("clipped clone draws");
}

#[test]
fn wave_c1_instancing_003_artboard_instances_share_animation_definitions() {
    let (file, graphs, graph_index) = first_graph("juice.riv");
    let definition = source_occurrence(&file, &graphs, graph_index);
    let animation_count = definition.linear_animations().len();
    assert!(animation_count > 0);
    let definition_arena = Arc::clone(&definition.linear_animations);
    let instance = definition.clone();

    assert_eq!(instance.linear_animations().len(), animation_count);
    assert!(Arc::ptr_eq(
        &definition.linear_animations,
        &instance.linear_animations,
    ));
    assert!(std::ptr::eq(
        definition
            .linear_animation(0)
            .expect("definition animation"),
        instance.linear_animation(0).expect("instance animation"),
    ));

    drop(instance);
    assert_eq!(definition.linear_animations().len(), animation_count);
    assert!(std::ptr::eq(
        definition
            .linear_animation(0)
            .expect("surviving definition"),
        &definition_arena[0],
    ));

    let weak_arena = Arc::downgrade(&definition_arena);
    drop(definition);
    assert!(weak_arena.upgrade().is_some());
    drop(definition_arena);
    assert!(weak_arena.upgrade().is_none());
}

fn apply_joystick(artboard: &mut ArtboardInstance, joystick_local: usize) {
    let joystick_index = artboard
        .joysticks
        .iter()
        .position(|joystick| {
            artboard.component_local_id(joystick.component()) == Some(joystick_local)
        })
        .expect("live Joystick occurrence");
    artboard.apply_runtime_joystick_at(joystick_index);
}

fn assert_flags(artboard: &ArtboardInstance, local_id: usize, expected: [bool; 3]) {
    const INVERT_X: u64 = 1 << 0;
    const INVERT_Y: u64 = 1 << 1;
    const WORLD_SPACE: u64 = 1 << 2;
    let flags = artboard
        .debug_joystick_flags(local_id)
        .expect("Joystick flags");
    assert_eq!(flags & INVERT_X != 0, expected[0]);
    assert_eq!(flags & INVERT_Y != 0, expected[1]);
    assert_eq!(flags & WORLD_SPACE != 0, expected[2]);
}

#[test]
fn wave_c1_joystick_flags_001_flags_and_inverted_axes_match() {
    let (file, graphs, graph_index) = first_graph("joystick_flag_test.riv");
    let graph = &graphs.artboards[graph_index];
    let mut artboard = source_occurrence(&file, &graphs, graph_index);
    let invert_x = named_local(graph, "Invert X Joystick", "Joystick");
    let invert_y = named_local(graph, "Invert Y Joystick", "Joystick");
    let world = named_local(graph, "World Joystick", "Joystick");
    let normal = named_local(graph, "Normal Joystick", "Joystick");

    assert_flags(&artboard, invert_x, [true, false, false]);
    for (value, expected_x) in [(0.0, 350.0), (1.0, 300.0), (-1.0, 400.0)] {
        artboard.set_double_property(invert_x, property_key("Joystick", "x"), value);
        apply_joystick(&mut artboard, invert_x);
        assert_eq!(
            artboard.double_property(
                named_local(graph, "invert_x_rect", "Shape"),
                property_key("Shape", "x"),
            ),
            Some(expected_x),
        );
    }

    assert_flags(&artboard, invert_y, [false, true, false]);
    for (value, expected_x) in [(0.0, 425.0), (1.0, 400.0), (-1.0, 450.0)] {
        artboard.set_double_property(invert_y, property_key("Joystick", "y"), value);
        apply_joystick(&mut artboard, invert_y);
        assert_eq!(
            artboard.double_property(
                named_local(graph, "invert_y_ellipse", "Shape"),
                property_key("Shape", "x"),
            ),
            Some(expected_x),
        );
    }

    assert_flags(&artboard, world, [false, false, true]);
    assert_flags(&artboard, normal, [false, false, false]);
}
