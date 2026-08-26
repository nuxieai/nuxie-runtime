//! Exact ports of pinned `layout_participant_test.cpp`.
//!
//! These tests deliberately keep each upstream fixture, action order, and
//! assertion at the public `ArtboardInstance` owner.  They do not substitute a
//! synthetic layout or assert against parsed source metadata.

use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_runtime::{ArtboardInstance, RuntimeLayoutBounds, RuntimeLayoutBoundsReport};

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
    file: RuntimeFile,
    graphs: GraphFile,
    artboard_index: usize,
    artboard: ArtboardInstance,
}

impl Fixture {
    fn graph(&self) -> &ArtboardGraph {
        &self.graphs.artboards[self.artboard_index]
    }

    fn report(&self) -> Vec<RuntimeLayoutBoundsReport> {
        self.artboard
            .debug_taffy_layout_bounds_report(&self.file, self.graph())
            .expect("fixture has a runtime layout solve")
    }

    fn named_local(&self, name: &str) -> usize {
        self.graph()
            .local_objects
            .iter()
            .find(|object| object.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("fixture has object named {name}"))
            .local_id
    }
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
        file,
        graphs,
        artboard_index,
        artboard,
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

fn shape_reports(fixture: &Fixture) -> Vec<RuntimeLayoutBoundsReport> {
    fixture
        .report()
        .into_iter()
        .filter(|report| report.type_name == "Shape")
        .collect()
}

fn only_shape_bounds(fixture: &Fixture) -> RuntimeLayoutBounds {
    let shapes = shape_reports(fixture);
    assert_eq!(shapes.len(), 1, "upstream requires exactly one Shape");
    let shape = &shapes[0];
    RuntimeLayoutBounds {
        x: shape.x,
        y: shape.y,
        width: shape.width,
        height: shape.height,
    }
}

#[test]
fn a_fill_participant_fills_a_stack_cell_from_a_riv_file() {
    let fixture = fixture("layout/stack_participant.riv", None, true);
    let bounds = only_shape_bounds(&fixture);
    assert_eq!(bounds.width, 200.0);
    assert_eq!(bounds.height, 200.0);
}

#[test]
fn a_solos_active_child_is_laid_out_through_it_from_a_riv_file() {
    let fixture = fixture("layout/solo_participant.riv", None, true);
    let solo = fixture
        .artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Solo")
        .expect("exactly one Solo");
    let all_shapes = fixture
        .artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "Shape")
        .collect::<Vec<_>>();
    assert_eq!(all_shapes.len(), 2);
    let shapes = shape_reports(&fixture);
    assert_eq!(
        shapes.len(),
        1,
        "only the active Solo child joins the solve"
    );
    let active = &shapes[0];
    assert_eq!(active.parent_local, Some(solo.local_id));
    assert_eq!(active.width, 200.0);
    assert_eq!(active.height, 200.0);
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
    let shapes = shape_reports(&fixture);
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes.iter().filter(|shape| shape.collapsed).count(), 1);
    let shown = shapes
        .iter()
        .find(|shape| !shape.collapsed)
        .expect("one shown Shape");
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

#[test]
fn a_solos_active_child_index_helpers_work_from_a_riv_file() {
    let mut fixture = fixture("layout/solo_participant.riv", None, true);
    let solo_local = fixture
        .artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Solo")
        .expect("exactly one Solo")
        .local_id;
    let active_key = property_key("Solo", "activeComponentId");
    let shapes = fixture
        .artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .collect::<Vec<_>>();
    assert_eq!(shapes.len(), 2);
    assert!(
        !fixture
            .artboard
            .component(shapes[0])
            .unwrap()
            .is_collapsed()
    );
    assert!(
        fixture
            .artboard
            .component(shapes[1])
            .unwrap()
            .is_collapsed()
    );

    // `Solo::updateByIndex(1)` writes the second option's object-table id to
    // generated `activeComponentId`; Rust exposes that exact generated setter.
    assert!(
        fixture
            .artboard
            .set_uint_property(solo_local, active_key, shapes[1] as u64)
    );
    assert!(
        fixture
            .artboard
            .component(shapes[0])
            .unwrap()
            .is_collapsed()
    );
    assert!(
        !fixture
            .artboard
            .component(shapes[1])
            .unwrap()
            .is_collapsed()
    );
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
        .unwrap_or_else(|| {
            let report = fixture
                .report()
                .into_iter()
                .find(|report| report.local_id == shape_local)
                .expect("Shape is a live layout provider");
            RuntimeLayoutBounds {
                x: report.x,
                y: report.y,
                width: report.width,
                height: report.height,
            }
        })
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
    let mut shapes = shape_reports(&fixture);
    assert_eq!(shapes.len(), 2);
    shapes.sort_by(|left, right| left.x.total_cmp(&right.x));
    assert!((shapes[0].width - 100.0).abs() <= f32::EPSILON);
    assert!((shapes[0].height - 200.0).abs() <= f32::EPSILON);
    assert!((shapes[1].width - 100.0).abs() <= f32::EPSILON);
    assert!((shapes[1].height - 50.0).abs() <= f32::EPSILON);
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
fn a_participant_inside_a_group_is_laid_out_through_it() {
    let fixture = fixture("layout/group_participant.riv", None, true);
    let shapes = shape_reports(&fixture);
    assert_eq!(shapes.len(), 1);
    let shape = &shapes[0];
    let parent = shape
        .parent_local
        .expect("Shape has its plain group parent");
    assert_eq!(
        fixture.artboard.component(parent).unwrap().type_name,
        "Node"
    );
    assert_eq!(shape.width, 200.0);
    assert_eq!(shape.height, 200.0);
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
    let shapes = shape_reports(&fixture);
    assert_eq!(
        shapes.len(),
        2,
        "inactive Solo sibling is excluded from the solve"
    );
    let active = shapes
        .iter()
        .find(|shape| shape.parent_local == Some(solo.local_id))
        .expect("active grouped Solo child");
    assert_eq!(active.width, 200.0);
    assert_eq!(active.height, 200.0);
    let inactive = all_shapes
        .iter()
        .find(|shape| shape.local_id != active.local_id && shape.is_collapsed())
        .expect("inactive grouped Solo child");
    assert!(fixture.artboard.layout_bounds(inactive.local_id).is_none());
    let deep = shapes
        .iter()
        .find(|shape| shape.parent_local != Some(solo.local_id))
        .expect("participant two groups deep");
    assert_eq!(deep.width, 200.0);
    assert_eq!(deep.height, 200.0);
}

#[test]
fn an_artboard_component_list_inside_a_group_stays_out_of_the_layout() {
    let fixture = fixture("clipping_and_draw_order.riv", None, true);
    let lists = fixture
        .artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "ArtboardComponentList")
        .collect::<Vec<_>>();
    assert_eq!(lists.len(), 1);
    let list_report = fixture
        .report()
        .into_iter()
        .find(|report| report.local_id == lists[0].local_id);
    let parent = fixture
        .graph()
        .components
        .iter()
        .find(|component| component.local_id == lists[0].local_id)
        .and_then(|component| component.parent_local)
        .expect("list parent");
    assert_eq!(
        fixture.artboard.component(parent).unwrap().type_name,
        "Node"
    );
    assert!(list_report.is_none(), "artboard remains a layout leaf");
}

#[test]
#[ignore = "expected-red: Shape::computeIntrinsicBounds has no public exact pre-advance owner"]
fn a_custom_path_participant_measures_before_its_paths_are_built() {
    let mut fixture = fixture(
        "layout_grid_stack.riv",
        Some("GridWithLayoutParticipants"),
        false,
    );
    let shapes = fixture
        .artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .collect::<Vec<_>>();
    assert!(!shapes.is_empty());
    let mut custom_path_shapes = 0;
    for shape in shapes {
        // This executes the live Shape geometry owner before any advance.  A
        // missing/inverted result is the exact computeIntrinsicBounds seam.
        let bounds = fixture
            .artboard
            .object_world_bounds(shape)
            .expect("pre-advance Shape::computeIntrinsicBounds result");
        assert!(bounds.max_x - bounds.min_x >= 0.0);
        assert!(bounds.max_y - bounds.min_y >= 0.0);
        let has_points_path = fixture.graph().components.iter().any(|component| {
            component.type_name == "PointsPath" && component.parent_local == Some(shape)
        });
        if has_points_path {
            custom_path_shapes += 1;
            assert!(bounds.max_x - bounds.min_x > 0.0);
            assert!(bounds.max_y - bounds.min_y > 0.0);
        }
    }
    assert!(custom_path_shapes > 0);
}

#[test]
fn a_participant_with_an_empty_path_keeps_a_sane_world_transform() {
    let mut fixture = fixture(
        "layout_grid_stack.riv",
        Some("GridWithLayoutParticipants"),
        true,
    );
    let shapes = fixture
        .artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .collect::<Vec<_>>();
    assert!(!shapes.is_empty());
    for shape in shapes {
        let bounds = fixture
            .artboard
            .object_world_bounds(shape)
            .expect("Shape::computeIntrinsicBounds result");
        assert!(bounds.max_x - bounds.min_x >= 0.0);
        assert!(bounds.max_y - bounds.min_y >= 0.0);
        let world = fixture
            .artboard
            .object_world_transform(shape)
            .expect("Shape::worldTransform");
        assert!(world.0[4].abs() < 1.0e6);
        assert!(world.0[5].abs() < 1.0e6);
    }
}
