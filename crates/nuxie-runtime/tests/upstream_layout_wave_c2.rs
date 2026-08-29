//! Exact owner-flow ports for Wave C2 layout stack and direct layout cases.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    component_dirt::ComponentDirt,
    generated::{core_registry::CoreRegistry, layout_component_base::LayoutComponentBase},
    layout::{layout_component_style::LayoutComponentStyle, layout_style_applier::YGDisplay},
    layout_component::{Layout, LayoutComponent},
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeFactoryHandle, RuntimeFileHandle,
};

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
    // The pinned layout tests use File::artboard(), not an ArtboardInstance.
    artboard: CoreHandle,
    _file: RuntimeFileHandle,
}

impl Fixture {
    fn objects(&self) -> Vec<Option<CoreHandle>> {
        self.artboard
            .with_downcast::<Artboard, _>(|artboard| artboard.objects().to_vec())
            .expect("native Artboard")
    }

    fn object(&self, local: usize) -> CoreHandle {
        self.objects()
            .get(local)
            .cloned()
            .flatten()
            .expect("native local object")
    }

    fn report(&self) -> Vec<(usize, bool)> {
        self.objects()
            .into_iter()
            .enumerate()
            .filter_map(|(index, owner)| {
                owner?.with(|owner| {
                    owner
                        .as_layout_component()
                        .map(|layout| (index, layout.is_collapsed()))
                })?
            })
            .collect()
    }

    fn local(&self, name: &str) -> usize {
        let handle = self
            .artboard
            .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<LayoutComponent>(name))
            .flatten()
            .unwrap_or_else(|| panic!("missing object named {name}"));
        self.objects()
            .iter()
            .position(|owner| owner.as_ref() == Some(&handle))
            .expect("authored local object")
    }

    fn style(&self, layout_local: usize) -> usize {
        let style = self
            .object(layout_local)
            .with(|owner| {
                owner
                    .as_layout_component()
                    .expect("LayoutComponent")
                    .style_handle()
            })
            .flatten()
            .expect("attached LayoutComponentStyle");
        self.objects()
            .iter()
            .position(|owner| owner.as_ref() == Some(&style))
            .expect("native style object")
    }

    fn bounds(&self, local_id: usize) -> Layout {
        self.object(local_id)
            .with(|owner| {
                owner
                    .as_layout_component()
                    .expect("LayoutComponent")
                    .layout()
            })
            .expect("live layout owner")
    }

    fn world_xy(&mut self, local_id: usize) -> (f32, f32) {
        self.object(local_id)
            .with(|owner| {
                let transform = owner
                    .as_transform_component()
                    .expect("layout transform")
                    .world_transform();
                (transform[4], transform[5])
            })
            .expect("live layout transform")
    }

    fn uint_property(&self, local: usize, key: u16) -> Option<u64> {
        CoreRegistry::get_uint_handle(&self.object(local), i32::from(key)).map(u64::from)
    }

    fn double_property(&self, local: usize, key: u16) -> Option<f32> {
        CoreRegistry::get_double_handle(&self.object(local), i32::from(key))
    }

    fn set_uint_property(&self, local: usize, key: u16, value: u32) -> bool {
        CoreRegistry::set_uint_handle(&self.object(local), i32::from(key), value)
    }

    fn advance(&mut self) {
        Artboard::advance_handle(
            &self.artboard,
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
    }
}

fn fixture(name: &str, artboard_name: Option<&str>) -> Fixture {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained RecordingFactory");
    let mut result = ImportResult::Malformed;
    let file = File::import(
        &pinned_fixture(name),
        factory,
        Some(&mut result),
        None,
        None,
    )
    .unwrap_or_else(|| panic!("{name} imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    let artboard = file
        .with_file(|file| match artboard_name {
            Some(name) => file.artboard_named_source(name),
            None => file.artboard(),
        })
        .expect("native source artboard");
    Fixture {
        artboard,
        _file: file,
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
    for (local, object) in fixture.objects().into_iter().enumerate() {
        let Some(object) = object else {
            continue;
        };
        if object.core_type() != Some(LayoutComponentBase::TYPE_KEY) || local == 0 {
            continue;
        }
        let style = fixture.style(local);
        match (
            fixture.uint_property(style, layout_type),
            fixture.uint_property(style, width_scale),
        ) {
            (Some(2), _) => stack = Some(local),
            (_, Some(1)) => fill = Some(local),
            _ => box_child = Some(local),
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
        (fill.left(), fill.top(), fill.width(), fill.height()),
        (0.0, 0.0, 200.0, 200.0)
    );
    assert_eq!(
        (
            box_child.left(),
            box_child.top(),
            box_child.width(),
            box_child.height()
        ),
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
        assert!(fixture.set_uint_property(style, alignment, value));
        fixture.advance();
        let bounds = fixture.bounds(box_child);
        assert_eq!((bounds.left(), bounds.top()), (x, y), "alignment {value}");
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
        let _ = fixture.set_uint_property(style, display, visible);
        let _ = fixture.set_uint_property(style, layout_type, kind);
        // Preserve the original pinned assertion, in addition to the retained
        // descendant visibility check from this Rust test.
        let expected_display = if visible == 1 {
            YGDisplay::None
        } else if kind == 0 {
            YGDisplay::Flex
        } else {
            YGDisplay::Grid
        };
        assert_eq!(
            fixture
                .object(style)
                .with_downcast::<LayoutComponentStyle, _>(LayoutComponentStyle::display),
            Some(expected_display)
        );
        fixture.advance();
        let descendants = fixture
            .report()
            .into_iter()
            .filter(|(local, collapsed)| *local != 0 && !collapsed)
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
        fixture.uint_property(
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

fn assert_style_f32(fixture: &Fixture, style: usize, property: &str, expected: f32) {
    assert_eq!(
        fixture.double_property(style, property_key("LayoutComponentStyle", property)),
        Some(expected),
        "{property}"
    );
}

fn assert_style_u64(fixture: &Fixture, style: usize, property: &str, expected: u64) {
    assert_eq!(
        fixture.uint_property(style, property_key("LayoutComponentStyle", property)),
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
fn wave_c2_layout_011_forced_size_dirt() {
    let mut fixture = fixture("layout/layout_complex1.riv", None);
    let layout = fixture.object(fixture.local("LayoutLeftChild1"));
    let forced_size = || {
        layout
            .with_downcast::<LayoutComponent, _>(|layout| {
                (layout.forced_width(), layout.forced_height())
            })
            .expect("native LayoutComponent")
    };
    let has_style_dirt = || {
        layout
            .with(|owner| {
                owner
                    .as_component()
                    .expect("layout Component")
                    .dirt()
                    .contains(ComponentDirt::LAYOUT_STYLE)
            })
            .expect("live layout Component")
    };
    let before = forced_size();
    assert!(before.0.is_nan());
    assert!(before.1.is_nan());
    layout
        .with_downcast_mut::<LayoutComponent, _>(|layout| {
            layout.set_forced_width(100.0);
            layout.set_forced_height(150.0);
        })
        .expect("native LayoutComponent");
    // Native setters return void. Check the same change/no-change contract
    // through their actual retained fields, not a legacy debug-setter result.
    let after = forced_size();
    assert_ne!(
        (before.0.to_bits(), before.1.to_bits()),
        (after.0.to_bits(), after.1.to_bits())
    );
    assert_eq!(after, (100.0, 150.0));
    assert!(has_style_dirt());
    fixture.advance();
    assert!(!has_style_dirt());
    let before = forced_size();
    layout
        .with_downcast_mut::<LayoutComponent, _>(|layout| {
            layout.set_forced_width(100.0);
            layout.set_forced_height(150.0);
        })
        .expect("native LayoutComponent");
    assert_eq!(forced_size(), before);
    assert!(!has_style_dirt());
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
        assert!(fixture.set_uint_property(style, key, value));
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
    assert_eq!((root.width(), root.height()), (501.0, 512.0));
}
