//! Exact occurrence-owner corrections for Wave C1 layout-participant rows.

use super::*;
use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::GraphFile;
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

fn active_solo_child_owner(
    artboard: &ArtboardInstance,
    solo_local: usize,
) -> Option<(usize, usize)> {
    let solo = artboard.component(solo_local)?.concrete.solo.as_ref()?;
    let active_local =
        usize::try_from(artboard.uint_property(solo_local, solo.active_component_property_key?)?)
            .ok()?;
    let active_index = solo
        .cpp_local_ids
        .iter()
        .position(|candidate| *candidate == active_local)?;
    Some((active_index, active_local))
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
    let (active_index, active_local) =
        active_solo_child_owner(&artboard, solo_local).expect("active child owner and index");
    assert_eq!(active_index, 0);
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
    let (initial_index, initial_local) =
        active_solo_child_owner(&artboard, solo_local).expect("initial active child owner");
    assert_eq!(initial_index, 0);
    assert_eq!(
        artboard
            .component(solo_local)
            .unwrap()
            .concrete
            .solo
            .as_ref()
            .unwrap()
            .cpp_local_ids[0],
        initial_local,
    );
    assert!(artboard.set_solo_active_child_by_index(solo_local, 1.0));
    let (updated_index, updated_local) =
        active_solo_child_owner(&artboard, solo_local).expect("updated active child owner");
    assert_eq!(updated_index, 1);
    assert_eq!(
        artboard
            .component(solo_local)
            .unwrap()
            .concrete
            .solo
            .as_ref()
            .unwrap()
            .cpp_local_ids[1],
        updated_local,
    );
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
