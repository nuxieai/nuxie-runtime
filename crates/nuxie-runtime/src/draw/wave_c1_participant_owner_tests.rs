//! Exact occurrence-owner corrections for Wave C1 layout-participant rows.

use super::*;
use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use std::path::PathBuf;

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn fixture(
    name: &str,
    artboard_name: Option<&str>,
) -> (RuntimeFile, GraphFile, usize, ArtboardInstance) {
    let runtime = read_runtime_file(&pinned_fixture(name))
        .unwrap_or_else(|error| panic!("{name} imports: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&runtime)
        .unwrap_or_else(|error| panic!("{name} graphs: {error:#}"));
    let artboard_index = artboard_name.map_or(0, |wanted| {
        graphs
            .artboards
            .iter()
            .position(|graph| graph.name.as_deref() == Some(wanted))
            .unwrap_or_else(|| panic!("{name} has artboard {wanted}"))
    });
    let artboard = ArtboardInstance::from_graph_with_artboards(
        &runtime,
        &graphs.artboards[artboard_index],
        &graphs.artboards,
    )
    .unwrap_or_else(|error| panic!("{name} instantiates: {error:#}"));
    (runtime, graphs, artboard_index, artboard)
}

fn active_solo_child_index(artboard: &ArtboardInstance, solo_local: usize) -> Option<usize> {
    let solo = artboard.component(solo_local)?.concrete.solo.as_ref()?;
    solo.cpp_local_ids.iter().position(|child_local| {
        artboard
            .component(*child_local)
            .is_some_and(|child| !child.is_collapsed())
    })
}

#[test]
fn wave_c1_participant_002_solo_and_active_child_keep_exact_provider_owners() {
    let (_runtime, _graphs, _index, mut artboard) = fixture("layout/solo_participant.riv", None);
    artboard.advance(0.0).expect("initial Artboard::advance(0)");
    let solo_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Solo")
        .expect("exactly one Solo")
        .local_id;
    assert!(
        artboard.layout_bounds(solo_local).is_none(),
        "the Solo itself is not a retained LayoutNodeProvider"
    );
    let active_index = active_solo_child_index(&artboard, solo_local).expect("active child index");
    let active_local = artboard
        .component(solo_local)
        .and_then(|component| component.concrete.solo.as_ref())
        .and_then(|solo| solo.cpp_local_ids.get(active_index))
        .copied()
        .expect("active Solo child owner");
    let bounds = artboard
        .layout_bounds(active_local)
        .expect("active child is a retained LayoutNodeProvider");
    assert_eq!((bounds.width, bounds.height), (200.0, 200.0));
}

#[test]
fn wave_c1_participant_007_solo_active_child_index_helpers_use_exact_owner() {
    let (_runtime, _graphs, _index, mut artboard) = fixture("layout/solo_participant.riv", None);
    artboard.advance(0.0).expect("initial Artboard::advance(0)");
    let solo_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Solo")
        .expect("exactly one Solo")
        .local_id;
    assert_eq!(active_solo_child_index(&artboard, solo_local), Some(0));
    assert!(artboard.set_solo_active_child_by_index(solo_local, 1.0));
    assert_eq!(active_solo_child_index(&artboard, solo_local), Some(1));
}

#[test]
fn wave_c1_participant_014_plain_group_has_no_provider_owner() {
    let (_runtime, _graphs, _index, mut artboard) = fixture("layout/group_participant.riv", None);
    artboard.advance(0.0).expect("initial Artboard::advance(0)");
    let shape = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Shape")
        .expect("exactly one Shape");
    let parent_local = artboard
        .component_parent_local(shape.local_id)
        .expect("Shape group parent");
    assert_eq!(artboard.component(parent_local).unwrap().type_name, "Node");
    assert!(
        artboard.layout_bounds(parent_local).is_none(),
        "the transparent plain Group is not a retained LayoutNodeProvider"
    );
    let bounds = artboard
        .layout_bounds(shape.local_id)
        .expect("Shape is the retained provider");
    assert_eq!((bounds.width, bounds.height), (200.0, 200.0));
}

fn component_list_artboard_is_leaf(name: &str) -> bool {
    let (_runtime, _graphs, _index, mut artboard) = fixture(name, None);
    artboard.advance(0.0).expect("initial Artboard::advance(0)");
    let lists = artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "ArtboardComponentList")
        .collect::<Vec<_>>();
    assert_eq!(lists.len(), 1);
    let parent_local = artboard
        .component_parent_local(lists[0].local_id)
        .expect("component-list group parent");
    assert_eq!(artboard.component(parent_local).unwrap().type_name, "Node");
    assert_eq!(artboard.component(0).unwrap().type_name, "Artboard");
    assert!(
        artboard.layout_bounds(0).is_some(),
        "the exact post-advance Artboard layout owner is retained"
    );
    TaffyRuntimeLayoutEngine
        .layout_provider_children(&artboard, 0)
        .expect("post-advance Artboard provider topology")
        .is_empty()
}

#[test]
fn wave_c1_participant_016_component_list_keeps_artboard_leaf_after_advance() {
    assert!(
        component_list_artboard_is_leaf("clipping_and_draw_order.riv"),
        "the post-advance Taffy Artboard owner remains a leaf"
    );
}

#[test]
fn wave_c1_participant_017_flagged_component_list_makes_artboard_non_leaf() {
    assert!(
        !component_list_artboard_is_leaf("layout/list_in_group_joins_layout.riv"),
        "the post-advance Taffy Artboard owner is no longer a leaf"
    );
}

fn shape_intrinsic_bounds(
    artboard: &ArtboardInstance,
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    shape_local: usize,
    paths: &mut RuntimeArtboardPathState,
) -> Option<RenderAabb> {
    let prepared = paths.live_traversal_frame(artboard, graph, Some(runtime));
    let layout_bounds = prepared.layout_bounds;
    let mut commands = artboard.runtime_shape_query_world_path_commands(
        shape_local,
        graph,
        layout_bounds.as_ref().as_ref(),
        paths,
    );
    let world = paths.component_world_transform_with_bounds(
        artboard,
        graph,
        shape_local,
        layout_bounds.as_ref().as_ref(),
    );
    transform_path_commands(&mut commands, world.invert_or_identity());
    runtime_exact_path_bounds(&commands)
}

fn grid_participant_fixture() -> (RuntimeFile, GraphFile, usize, ArtboardInstance) {
    fixture("layout_grid_stack.riv", Some("GridWithLayoutParticipants"))
}

#[test]
#[ignore = "expected-red: pre-advance noncollapsed PointsPath has no Shape intrinsic-bounds owner"]
fn wave_c1_participant_018_custom_paths_compute_intrinsic_bounds_before_advance() {
    let (_runtime, _graphs, _index, artboard) = grid_participant_fixture();
    let shapes = artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .collect::<Vec<_>>();
    assert!(!shapes.is_empty());
    let mut custom_path_shapes = 0;
    for shape_local in shapes {
        let has_live_points_path = artboard
            .runtime_shapes
            .get(shape_local)
            .into_iter()
            .flat_map(|shape| &shape.path_locals)
            .any(|path_local| {
                artboard
                    .component(*path_local)
                    .is_some_and(|path| path.type_name == "PointsPath" && !path.is_collapsed())
            });
        if has_live_points_path {
            custom_path_shapes += 1;
            let shape = artboard
                .runtime_shapes
                .get(shape_local)
                .expect("live Shape occurrence");
            let retained = shape.paint_paths
                [runtime_shape_paint_path_kind_slot(RuntimeShapePaintPathKind::World)]
            .retained
            .borrow();
            let commands = runtime_path_commands_from_raw_path(
                retained
                    .as_ref()
                    .expect("Shape::computeIntrinsicBounds must build the pre-advance PointsPath")
                    .raw_path
                    .as_ref(),
            );
            let bounds = runtime_exact_path_bounds(&commands)
                .expect("Shape::computeIntrinsicBounds before advance");
            assert!(bounds.width() > 0.0);
            assert!(bounds.height() > 0.0);
        }
    }
    assert!(custom_path_shapes > 0);
}

#[test]
fn wave_c1_participant_019_empty_path_intrinsic_bounds_keep_world_transform_sane() {
    let (runtime, graphs, index, mut artboard) = grid_participant_fixture();
    artboard.advance(0.0).expect("initial Artboard::advance(0)");
    let graph = &graphs.artboards[index];
    let shapes = artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .collect::<Vec<_>>();
    assert!(!shapes.is_empty());
    let mut paths = RuntimeArtboardPathState::default();
    for shape_local in shapes {
        let bounds = shape_intrinsic_bounds(&artboard, &runtime, graph, shape_local, &mut paths)
            .expect("Shape::computeIntrinsicBounds after advance");
        assert!(bounds.width() >= 0.0);
        assert!(bounds.height() >= 0.0);
        let world = artboard
            .object_world_transform(shape_local)
            .expect("Shape::worldTransform");
        assert!(world.0[4].abs() < 1.0e6);
        assert!(world.0[5].abs() < 1.0e6);
    }
}
