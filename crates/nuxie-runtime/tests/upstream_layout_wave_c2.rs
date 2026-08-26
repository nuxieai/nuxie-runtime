//! Exact owner-flow ports for Wave C2 layout stack and direct layout cases.

use std::path::PathBuf;

use nuxie_binary::{read_runtime_file, RuntimeFile};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_runtime::{ArtboardInstance, ComponentDirt, RuntimeLayoutBoundsReport};

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
    graph_index: usize,
    artboard: ArtboardInstance,
}

impl Fixture {
    fn graph(&self) -> &ArtboardGraph {
        &self.graphs.artboards[self.graph_index]
    }

    fn report(&self) -> Vec<RuntimeLayoutBoundsReport> {
        self.artboard
            .debug_taffy_layout_bounds_report(&self.file, self.graph())
            .expect("runtime layout report")
    }

    fn local(&self, name: &str) -> usize {
        self.graph()
            .local_objects
            .iter()
            .find(|object| object.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("missing object named {name}"))
            .local_id
    }

    fn style(&self, layout_local: usize) -> usize {
        let global_id = self.graph().local_objects[layout_local].global_id;
        usize::try_from(
            self.file
                .object(global_id as usize)
                .and_then(|object| object.uint_property("styleId"))
                .expect("LayoutComponent styleId"),
        )
        .expect("styleId fits usize")
    }

    fn bounds(&self, local_id: usize) -> RuntimeLayoutBoundsReport {
        self.report()
            .into_iter()
            .find(|entry| entry.local_id == local_id)
            .unwrap_or_else(|| panic!("missing layout report for local {local_id}"))
    }

    fn world_xy(&mut self, local_id: usize) -> (f32, f32) {
        let transform = self
            .artboard
            .component_world_transform_with_scroll(local_id)
            .unwrap_or_else(|| panic!("missing world transform for local {local_id}"));
        transform.translation()
    }

    fn advance(&mut self) {
        self.artboard.advance(0.0).expect("Artboard::advance(0)");
    }
}

fn fixture(name: &str, artboard_name: Option<&str>) -> Fixture {
    let file = read_runtime_file(&pinned_fixture(name))
        .unwrap_or_else(|error| panic!("{name} imports: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("{name} graphs: {error:#}"));
    let graph_index = artboard_name.map_or(0, |wanted| {
        graphs
            .artboards
            .iter()
            .position(|graph| graph.name.as_deref() == Some(wanted))
            .unwrap_or_else(|| panic!("missing artboard {wanted}"))
    });
    let artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        &graphs.artboards[graph_index],
        &graphs.artboards,
    )
    .unwrap_or_else(|error| panic!("{name} instantiates: {error:#}"));
    Fixture {
        file,
        graphs,
        graph_index,
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

fn stack_locals(fixture: &Fixture) -> (usize, usize, usize) {
    let layout_type = property_key("LayoutComponentStyle", "layoutTypeValue");
    let width_scale = property_key("LayoutComponentStyle", "layoutWidthScaleType");
    let mut stack = None;
    let mut fill = None;
    let mut box_child = None;
    for object in &fixture.graph().local_objects {
        if object.type_name != Some("LayoutComponent") || object.local_id == 0 {
            continue;
        }
        let style = fixture.style(object.local_id);
        match (
            fixture.artboard.debug_uint_property(style, layout_type),
            fixture.artboard.debug_uint_property(style, width_scale),
        ) {
            (Some(2), _) => stack = Some(object.local_id),
            (_, Some(1)) => fill = Some(object.local_id),
            _ => box_child = Some(object.local_id),
        }
    }
    (
        stack.expect("stack LayoutComponent"),
        fill.expect("fill LayoutComponent"),
        box_child.expect("fixed LayoutComponent"),
    )
}

#[test]
fn wave_c2_layout_stack_001_overlaps_children_and_aligns_from_file() {
    let mut fixture = fixture("layout/stack.riv", None);
    fixture.advance();
    let (_, fill, box_child) = stack_locals(&fixture);
    let fill = fixture.bounds(fill);
    let box_child = fixture.bounds(box_child);
    assert_eq!(
        (fill.x, fill.y, fill.width, fill.height),
        (0.0, 0.0, 200.0, 200.0)
    );
    assert_eq!(
        (box_child.x, box_child.y, box_child.width, box_child.height),
        (160.0, 160.0, 40.0, 40.0)
    );
}

#[test]
fn wave_c2_layout_stack_002_alignment_positions_fixed_child() {
    let mut fixture = fixture("layout/stack.riv", None);
    fixture.advance();
    let (stack, _, box_child) = stack_locals(&fixture);
    let style = fixture.style(stack);
    let alignment = property_key("LayoutComponentStyle", "layoutAlignmentType");
    for (value, x, y) in [
        (0, 0.0, 0.0),
        (1, 80.0, 0.0),
        (2, 160.0, 0.0),
        (3, 0.0, 80.0),
        (4, 80.0, 80.0),
        (5, 160.0, 80.0),
        (6, 0.0, 160.0),
        (7, 80.0, 160.0),
        (8, 160.0, 160.0),
    ] {
        assert!(fixture.artboard.set_uint_property(style, alignment, value));
        fixture.advance();
        let bounds = fixture.bounds(box_child);
        assert_eq!((bounds.x, bounds.y), (x, y), "alignment {value}");
    }
}

#[test]
fn wave_c2_layout_stack_003_engine_display_folds_visibility_and_type() {
    let mut fixture = fixture("layout/stack.riv", None);
    fixture.advance();
    let (stack, _, _) = stack_locals(&fixture);
    let style = fixture.style(stack);
    let display = property_key("LayoutComponentStyle", "displayValue");
    let layout_type = property_key("LayoutComponentStyle", "layoutTypeValue");

    for (visible, kind, expected_layouts) in [
        (0, 0, 3usize),
        (0, 1, 3),
        (0, 2, 3),
        (1, 0, 0),
        (1, 1, 0),
        (1, 2, 0),
    ] {
        let _ = fixture.artboard.set_uint_property(style, display, visible);
        let _ = fixture.artboard.set_uint_property(style, layout_type, kind);
        fixture.advance();
        let descendants = fixture
            .report()
            .into_iter()
            .filter(|entry| entry.local_id != 0 && !entry.collapsed)
            .count();
        assert_eq!(
            descendants, expected_layouts,
            "display={visible} layoutType={kind}"
        );
    }
}

fn assert_named_world_positions(name: &str, expected: &[(&str, f32, f32)]) {
    let mut fixture = fixture(name, None);
    fixture.advance();
    for (object, x, y) in expected {
        let local = fixture.local(object);
        assert_eq!(fixture.world_xy(local), (*x, *y), "{object}");
    }
}

#[test]
fn wave_c2_layout_001_flex_direction_row() {
    let mut fixture = fixture("layout/layout_horizontal.riv", None);
    fixture.advance();
    let first = fixture.local("LayoutComponent1");
    let style = fixture.style(first);
    assert_eq!(
        fixture.artboard.debug_uint_property(
            style,
            property_key("LayoutComponentStyle", "flexDirectionValue")
        ),
        Some(2)
    );
    for (name, x) in [
        ("LayoutComponent1", 0.0),
        ("LayoutComponent2", 100.0),
        ("LayoutComponent3", 200.0),
    ] {
        let local = fixture.local(name);
        assert_eq!(fixture.world_xy(local), (x, 0.0));
    }
}

#[test]
fn wave_c2_layout_002_flex_direction_column() {
    assert_named_world_positions(
        "layout/layout_vertical.riv",
        &[
            ("LayoutComponent1", 0.0, 0.0),
            ("LayoutComponent2", 0.0, 100.0),
            ("LayoutComponent3", 0.0, 200.0),
        ],
    );
}

#[test]
fn wave_c2_layout_003_flex_direction_row_with_gap() {
    assert_named_world_positions(
        "layout/layout_horizontal_gaps.riv",
        &[
            ("LayoutComponent1", 0.0, 0.0),
            ("LayoutComponent2", 110.0, 0.0),
            ("LayoutComponent3", 220.0, 0.0),
        ],
    );
}

#[test]
fn wave_c2_layout_004_flex_direction_row_with_wrap() {
    assert_named_world_positions(
        "layout/layout_horizontal_wrap.riv",
        &[("LayoutComponent6", 0.0, 100.0)],
    );
}

#[test]
fn wave_c2_layout_005_center_alignment() {
    assert_named_world_positions(
        "layout/layout_center.riv",
        &[("LayoutComponent1", 200.0, 200.0)],
    );
}

#[test]
#[ignore = "expected-red: the exact HiText localBounds owner is not retained by the Rust Text occurrence"]
fn wave_c2_layout_006_intrinsic_text_size() {
    let mut fixture = fixture("layout/measure_tests.riv", Some("hi"));
    fixture.advance();
    let text = fixture.local("HiText");
    let bounds = fixture.bounds(text);
    assert_eq!(
        (bounds.x, bounds.y, bounds.width, bounds.height),
        (0.0, 0.0, 62.48047, 72.62695)
    );
}

fn assert_style_f32(fixture: &Fixture, style: usize, property: &str, expected: f32) {
    assert_eq!(
        fixture
            .artboard
            .double_property(style, property_key("LayoutComponentStyle", property)),
        Some(expected),
        "{property}"
    );
}

fn assert_style_u64(fixture: &Fixture, style: usize, property: &str, expected: u64) {
    assert_eq!(
        fixture
            .artboard
            .debug_uint_property(style, property_key("LayoutComponentStyle", property)),
        Some(expected),
        "{property}"
    );
}

#[test]
fn wave_c2_layout_007_padding_px() {
    let mut fixture = fixture("layout/layout_complex1.riv", None);
    fixture.advance();
    let parent = fixture.local("LayoutLeft");
    let style = fixture.style(parent);
    for property in ["paddingLeft", "paddingRight", "paddingTop", "paddingBottom"] {
        assert_style_f32(&fixture, style, property, 20.0);
    }
    for property in [
        "paddingLeftUnitsValue",
        "paddingRightUnitsValue",
        "paddingTopUnitsValue",
        "paddingBottomUnitsValue",
    ] {
        assert_style_u64(&fixture, style, property, 1);
    }
    for (name, x, y) in [
        ("LayoutLeft", 0.0, 0.0),
        ("LayoutLeftChild1", 20.0, 20.0),
        ("LayoutLeftChild2", 130.0, 20.0),
    ] {
        let local = fixture.local(name);
        assert_eq!(fixture.world_xy(local), (x, y));
    }
}

#[test]
fn wave_c2_layout_008_margin_px_and_percent() {
    let mut fixture = fixture("layout/layout_complex1.riv", None);
    fixture.advance();
    let child1 = fixture.local("LayoutRightChild1");
    let child2 = fixture.local("LayoutRightChild2");
    for (style, value, units, alignment, wrap) in [
        (fixture.style(child1), 10.0, 1, 4, 0),
        (fixture.style(child2), 5.0, 2, 0, 1),
    ] {
        for property in ["marginLeft", "marginRight", "marginTop", "marginBottom"] {
            assert_style_f32(&fixture, style, property, value);
        }
        for property in [
            "marginLeftUnitsValue",
            "marginRightUnitsValue",
            "marginTopUnitsValue",
            "marginBottomUnitsValue",
        ] {
            assert_style_u64(&fixture, style, property, units);
        }
        assert_style_u64(&fixture, style, "layoutAlignmentType", alignment);
        assert_style_u64(&fixture, style, "flexWrapValue", wrap);
    }
    for (name, x, y) in [
        ("LayoutRight", 250.0, 0.0),
        ("LayoutRightChild1", 285.0, 35.0),
        ("LayoutRightChild2", 285.0, 215.0),
    ] {
        let local = fixture.local(name);
        assert_eq!(fixture.world_xy(local), (x, y));
    }
}

#[test]
fn wave_c2_layout_009_corner_radius() {
    let mut fixture = fixture("layout/layout_complex1.riv", None);
    fixture.advance();
    let style = fixture.style(fixture.local("LayoutLeftChild1"));
    for property in [
        "cornerRadiusTL",
        "cornerRadiusTR",
        "cornerRadiusBL",
        "cornerRadiusBR",
    ] {
        assert_style_f32(&fixture, style, property, 15.0);
    }
}

#[test]
#[ignore = "expected-red: the live Text occurrence retains authored alignValue but exposes no actual-direction-derived align owner"]
fn wave_c2_layout_010_direction() {
    let mut fixture = fixture("layout/layout_direction.riv", None);
    fixture.advance();
    for (name, x) in [("Layout1", 200.0), ("Layout2", 100.0), ("Layout3", 0.0)] {
        let local = fixture.local(name);
        assert_eq!(fixture.world_xy(local).0, x);
    }
    let text = fixture.local("SampleText");
    assert_eq!(
        fixture
            .artboard
            .debug_uint_property(text, property_key("Text", "alignValue")),
        Some(2)
    );
}

#[test]
fn wave_c2_layout_011_forced_size_dirt() {
    let mut fixture = fixture("layout/layout_complex1.riv", None);
    let layout = fixture.local("LayoutLeftChild1");
    assert_eq!(
        fixture.artboard.debug_layout_forced_size(layout),
        Some((None, None))
    );
    assert!(fixture
        .artboard
        .debug_set_layout_forced_size(layout, 100.0, 150.0));
    assert_eq!(
        fixture.artboard.debug_layout_forced_size(layout),
        Some((Some(100.0), Some(150.0)))
    );
    assert!(fixture
        .artboard
        .debug_component_dirt(layout)
        .is_some_and(|dirt| dirt.contains(ComponentDirt::LAYOUT_STYLE)));
    fixture.advance();
    assert!(!fixture
        .artboard
        .debug_component_dirt(layout)
        .is_some_and(|dirt| dirt.contains(ComponentDirt::LAYOUT_STYLE)));
    assert!(!fixture
        .artboard
        .debug_set_layout_forced_size(layout, 100.0, 150.0));
    assert!(!fixture
        .artboard
        .debug_component_dirt(layout)
        .is_some_and(|dirt| dirt.contains(ComponentDirt::LAYOUT_STYLE)));
}

#[test]
fn wave_c2_layout_012_alignment_mutation() {
    let mut fixture = fixture("layout/layout_alignment.riv", None);
    let container = fixture.local("LayoutContainer");
    let style = fixture.style(container);
    let alignment = property_key("LayoutComponentStyle", "layoutAlignmentType");
    let flex_direction = property_key("LayoutComponentStyle", "flexDirectionValue");
    let cases = [
        (
            Some((alignment, 9)),
            [(0.0, 0.0), (200.0, 0.0), (400.0, 0.0)],
        ),
        (
            Some((alignment, 10)),
            [(0.0, 200.0), (200.0, 200.0), (400.0, 200.0)],
        ),
        (
            Some((flex_direction, 0)),
            [(200.0, 0.0), (200.0, 200.0), (200.0, 400.0)],
        ),
        (
            Some((alignment, 11)),
            [(400.0, 0.0), (400.0, 200.0), (400.0, 400.0)],
        ),
    ];
    for (write, expected) in cases {
        let (key, value) = write.expect("mutation");
        assert!(fixture.artboard.set_uint_property(style, key, value));
        fixture.advance();
        for (name, xy) in ["Layout1", "Layout2", "Layout3"].into_iter().zip(expected) {
            let local = fixture.local(name);
            assert_eq!(fixture.world_xy(local), xy);
        }
    }
}

#[test]
fn wave_c2_layout_013_prevent_percent_margin_on_artboard() {
    let mut fixture = fixture("layout/artboard_percent_margin.riv", None);
    fixture.advance();
    let root = fixture.bounds(0);
    assert_eq!((root.width, root.height), (501.0, 512.0));
}

#[test]
#[ignore = "expected-red: exact styled_flex runtime solve offsets the child by its margins in addition to the pinned padding-only expectation"]
fn wave_c2_layout_023_padding_insets_fill_child() {
    let mut fixture = fixture("layout/styled_flex.riv", None);
    fixture.advance();
    let report = fixture.report();
    let child = report
        .iter()
        .find(|entry| {
            entry.type_name == "LayoutComponent"
                && entry.parent_local.is_some_and(|parent| parent != 0)
        })
        .expect("fill child nested under container");
    assert_eq!(
        (child.x, child.y, child.width, child.height),
        (10.0, 20.0, 160.0, 140.0)
    );
}
