//! Exact ports of pinned `layout_participant_test.cpp`.
//!
//! These tests deliberately keep each upstream fixture, action order, and
//! assertion at the public `ArtboardInstance` owner.  They do not substitute a
//! synthetic layout or assert against parsed source metadata.

use std::path::PathBuf;

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::{ArtboardInstance, RuntimeLayoutBounds};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

struct Fixture {
    artboard: ArtboardInstance,
    graphs: GraphFile,
    artboard_index: usize,
}

#[derive(Debug, Clone, Copy)]
struct RetainedShapeBounds {
    local_id: usize,
    parent_local: Option<usize>,
    collapsed: bool,
    bounds: Option<RuntimeLayoutBounds>,
}

fn fixture(name: &str, artboard_name: Option<&str>, advance: bool) -> Fixture {
    let file = read_runtime_file(&pinned_fixture(name))
        .unwrap_or_else(|error| panic!("{name} imports: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("{name} graphs: {error:#}"));
    let artboard_index = artboard_name.map_or(0, |wanted| {
        graphs
            .artboards
            .iter()
            .position(|graph| graph.name.as_deref() == Some(wanted))
            .unwrap_or_else(|| panic!("{name} has artboard {wanted}"))
    });
    let graph = &graphs.artboards[artboard_index];
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .unwrap_or_else(|error| panic!("{name} instantiates: {error:#}"));
    if advance {
        artboard.advance(0.0).expect("initial Artboard::advance(0)");
    }
    Fixture {
        artboard,
        graphs,
        artboard_index,
    }
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("schema definition");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("ancestor definition")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("property {type_name}.{property_name}"))
        .key
        .int
}

fn retained_shape_bounds(fixture: &Fixture) -> Vec<RetainedShapeBounds> {
    let graph = &fixture.graphs.artboards[fixture.artboard_index];
    fixture
        .artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "Shape")
        .map(|component| RetainedShapeBounds {
            local_id: component.local_id,
            parent_local: graph
                .components
                .iter()
                .find(|node| node.local_id == component.local_id)
                .and_then(|node| node.parent_local),
            collapsed: component.is_collapsed(),
            bounds: fixture.artboard.layout_bounds(component.local_id),
        })
        .collect()
}

fn only_shape_bounds(fixture: &Fixture) -> RuntimeLayoutBounds {
    let shapes = retained_shape_bounds(fixture);
    assert_eq!(shapes.len(), 1, "upstream requires exactly one Shape");
    shapes[0]
        .bounds
        .expect("Shape retains its LayoutNodeProvider bounds after advance")
}

#[test]
fn a_fill_participant_fills_a_stack_cell_from_a_riv_file() {
    let fixture = fixture("layout/stack_participant.riv", None, true);
    let bounds = only_shape_bounds(&fixture);
    assert_eq!(bounds.width, 200.0);
    assert_eq!(bounds.height, 200.0);
}

#[test]
fn a_hug_participant_hugs_its_content_from_a_riv_file() {
    let fixture = fixture("layout/hug_participant.riv", None, true);
    let bounds = only_shape_bounds(&fixture);
    assert!((bounds.width - 10.0).abs() <= f32::EPSILON);
    assert!((bounds.height - 10.0).abs() <= f32::EPSILON);
}

#[test]
fn a_fixed_size_participant_keeps_its_size_from_a_riv_file() {
    let fixture = fixture("layout/fixed_participant.riv", None, true);
    let bounds = only_shape_bounds(&fixture);
    assert!((bounds.width - 60.0).abs() <= f32::EPSILON);
    assert!((bounds.height - 40.0).abs() <= f32::EPSILON);
}

#[test]
fn a_display_none_participant_collapses_and_leaves_the_flow_from_a_riv_file() {
    let fixture = fixture("layout/display_none_participant.riv", None, true);
    let shapes = retained_shape_bounds(&fixture);
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes.iter().filter(|shape| shape.collapsed).count(), 1);
    let shown = shapes
        .iter()
        .find(|shape| !shape.collapsed)
        .expect("one shown Shape");
    let shown = shown
        .bounds
        .expect("shown Shape retains its LayoutNodeProvider bounds");
    assert!((shown.width - 200.0).abs() <= f32::EPSILON);
    assert!((shown.height - 200.0).abs() <= f32::EPSILON);
}

#[test]
fn min_max_constraints_clamp_a_participant_slot_from_a_riv_file() {
    let fixture = fixture("layout/constrained_participant.riv", None, true);
    let bounds = only_shape_bounds(&fixture);
    assert!((bounds.width - 50.0).abs() <= f32::EPSILON);
    assert!((bounds.height - 30.0).abs() <= f32::EPSILON);
}

fn animated_participant(name: &str) -> (Fixture, usize, usize) {
    let fixture = fixture(name, None, true);
    let shape_local = fixture
        .artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Shape")
        .expect("exactly one Shape")
        .local_id;
    let container_local = fixture
        .artboard
        .components()
        .iter()
        .find(|component| component.type_name == "LayoutComponent")
        .expect("non-artboard styled LayoutComponent")
        .local_id;
    (fixture, shape_local, container_local)
}

fn width(fixture: &Fixture, shape_local: usize) -> f32 {
    fixture
        .artboard
        .layout_bounds(shape_local)
        .expect("Shape retains its live LayoutNodeProvider bounds")
        .width
}

#[test]
fn disabling_a_layouts_interpolation_frees_participant_animation() {
    let (mut fixture, shape, container) = animated_participant("layout/animated_participant.riv");
    let style_local = fixture
        .artboard
        .components()
        .iter()
        .find(|component| component.type_name == "LayoutComponentStyle")
        .expect("container style")
        .local_id;
    assert!(fixture.artboard.set_double_property(
        style_local,
        property_key("LayoutComponentStyle", "interpolationTime"),
        0.0,
    ));
    fixture.artboard.advance(0.0).expect("style recascade");
    assert!(fixture.artboard.set_double_property(
        container,
        property_key("LayoutComponent", "width"),
        100.0,
    ));
    fixture.artboard.advance(0.016).expect("snap advance");
    assert!((width(&fixture, shape) - 100.0).abs() <= f32::EPSILON);
}

#[test]
fn participants_size_to_grid_cells_from_a_riv_file() {
    let fixture = fixture("layout/grid_participant.riv", None, true);
    let mut shapes = retained_shape_bounds(&fixture);
    assert_eq!(shapes.len(), 2);
    shapes.sort_by(|left, right| {
        left.bounds
            .expect("grid participant retains bounds")
            .x
            .total_cmp(&right.bounds.expect("grid participant retains bounds").x)
    });
    let first = shapes[0].bounds.expect("first grid participant bounds");
    let second = shapes[1].bounds.expect("second grid participant bounds");
    assert!((first.width - 100.0).abs() <= f32::EPSILON);
    assert!((first.height - 200.0).abs() <= f32::EPSILON);
    assert!((second.width - 100.0).abs() <= f32::EPSILON);
    assert!((second.height - 50.0).abs() <= f32::EPSILON);
}

#[test]
fn a_participant_retargets_a_cubic_animation_while_smoothing() {
    let (mut fixture, shape, container) =
        animated_participant("layout/animated_cubic_participant.riv");
    let width_key = property_key("LayoutComponent", "width");
    assert!(
        fixture
            .artboard
            .set_double_property(container, width_key, 100.0)
    );
    let mut current = 200.0;
    for _ in 0..8 {
        if current < 200.0 {
            break;
        }
        fixture.artboard.advance(0.1).expect("cubic flight");
        current = width(&fixture, shape);
    }
    assert!(current < 200.0);
    assert!(
        fixture
            .artboard
            .set_double_property(container, width_key, 80.0)
    );
    fixture.artboard.advance(0.1).expect("smoothing advance");
    assert!(
        fixture
            .artboard
            .set_double_property(container, width_key, 50.0)
    );
    for _ in 0..20 {
        fixture
            .artboard
            .advance(1.0)
            .expect("second retarget settle");
    }
    assert!((width(&fixture, shape) - 50.0).abs() <= f32::EPSILON);
}

#[test]
fn participants_nested_in_groups_and_in_a_grouped_solo_are_laid_out() {
    let fixture = fixture("layout/nested_group_participant.riv", None, true);
    let solo = fixture
        .artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Solo")
        .expect("one Solo");
    let all_shapes = fixture
        .artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "Shape")
        .collect::<Vec<_>>();
    assert_eq!(all_shapes.len(), 3);
    let shapes = retained_shape_bounds(&fixture)
        .into_iter()
        .filter(|shape| shape.bounds.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        shapes.len(),
        2,
        "inactive Solo sibling is excluded from the solve"
    );
    let active = shapes
        .iter()
        .find(|shape| shape.parent_local == Some(solo.local_id))
        .expect("active grouped Solo child");
    let active_bounds = active.bounds.expect("active Solo participant bounds");
    assert_eq!(active_bounds.width, 200.0);
    assert_eq!(active_bounds.height, 200.0);
    let inactive = all_shapes
        .iter()
        .find(|shape| shape.local_id != active.local_id && shape.is_collapsed())
        .expect("inactive grouped Solo child");
    assert!(fixture.artboard.layout_bounds(inactive.local_id).is_none());
    let deep = shapes
        .iter()
        .find(|shape| shape.parent_local != Some(solo.local_id))
        .expect("participant two groups deep");
    let deep_bounds = deep.bounds.expect("deep participant bounds");
    assert_eq!(deep_bounds.width, 200.0);
    assert_eq!(deep_bounds.height, 200.0);
}
