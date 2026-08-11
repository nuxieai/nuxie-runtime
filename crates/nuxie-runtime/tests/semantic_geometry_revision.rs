use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, read_runtime_file};
use nuxie_graph::GraphFile;
use nuxie_render_api::NullFactory;
use nuxie_runtime::{ArtboardInstance, ComponentDirt, TransformProperty};
#[cfg(feature = "tools")]
use nuxie_runtime::{
    reset_runtime_shape_paint_command_report_count, runtime_shape_paint_command_report_count,
};
use std::{collections::BTreeMap, sync::Arc};

fn fixture_artboard() -> ArtboardInstance {
    fixture_artboard_from_bytes(include_bytes!(
        "../../../fixtures/univ-1275/transform_live_write.riv"
    ))
}

fn fixture_artboard_from_bytes(bytes: &[u8]) -> ArtboardInstance {
    let file = read_runtime_file(bytes).expect("semantic-geometry fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("semantic-geometry graph builds");
    ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
        .expect("semantic-geometry artboard instantiates")
}

fn fixture_property(type_name: &str, property_name: &str, value: FixtureValue) -> FixtureProperty {
    let definition = nuxie_schema::definition_by_name(type_name).expect("fixture type exists");
    let property = std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .expect("fixture property exists");
    FixtureProperty {
        key: property.key.int,
        value,
    }
}

fn fixture_record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
    FixtureRecord {
        type_key: nuxie_schema::definition_by_name(type_name)
            .expect("fixture type exists")
            .type_key
            .int,
        properties,
    }
}

fn nested_fixture_artboard() -> ArtboardInstance {
    let file = RuntimeFile::from_fixture_records(vec![
        fixture_record("Backboard", vec![]),
        fixture_record("Artboard", vec![]),
        fixture_record(
            "NestedArtboard",
            vec![
                fixture_property("NestedArtboard", "parentId", FixtureValue::Uint(0)),
                fixture_property("NestedArtboard", "artboardId", FixtureValue::Uint(1)),
            ],
        ),
        fixture_record("Artboard", vec![]),
        fixture_record(
            "Node",
            vec![fixture_property("Node", "parentId", FixtureValue::Uint(0))],
        ),
    ])
    .expect("nested fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("nested fixture graphs");
    ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
        .expect("nested fixture instantiates")
}

fn nested_layout_fixture_artboard() -> ArtboardInstance {
    nested_layout_fixture_artboard_with_parent(false)
}

fn nested_layout_fixture_artboard_with_parent(has_parent: bool) -> ArtboardInstance {
    let mut records = vec![
        fixture_record("Backboard", vec![]),
        fixture_record(
            "Artboard",
            vec![
                fixture_property("LayoutComponent", "width", FixtureValue::Double(100.0)),
                fixture_property("LayoutComponent", "height", FixtureValue::Double(100.0)),
            ],
        ),
    ];
    if has_parent {
        records.push(fixture_record(
            "Node",
            vec![fixture_property("Node", "parentId", FixtureValue::Uint(0))],
        ));
    }
    records.extend([
        fixture_record(
            "NestedArtboardLayout",
            vec![
                fixture_property(
                    "NestedArtboardLayout",
                    "parentId",
                    FixtureValue::Uint(u64::from(has_parent)),
                ),
                fixture_property("NestedArtboardLayout", "artboardId", FixtureValue::Uint(1)),
            ],
        ),
        fixture_record(
            "Artboard",
            vec![
                fixture_property("LayoutComponent", "width", FixtureValue::Double(20.0)),
                fixture_property("LayoutComponent", "height", FixtureValue::Double(20.0)),
            ],
        ),
        fixture_record(
            "Shape",
            vec![fixture_property("Shape", "parentId", FixtureValue::Uint(0))],
        ),
        fixture_record(
            "Fill",
            vec![fixture_property("Fill", "parentId", FixtureValue::Uint(1))],
        ),
        fixture_record(
            "SolidColor",
            vec![
                fixture_property("SolidColor", "parentId", FixtureValue::Uint(2)),
                fixture_property("SolidColor", "colorValue", FixtureValue::Color(0xff33_66aa)),
            ],
        ),
        fixture_record(
            "Rectangle",
            vec![
                fixture_property("Rectangle", "parentId", FixtureValue::Uint(1)),
                fixture_property("Rectangle", "width", FixtureValue::Double(10.0)),
                fixture_property("Rectangle", "height", FixtureValue::Double(10.0)),
            ],
        ),
    ]);
    let file = RuntimeFile::from_fixture_records(records).expect("nested-layout fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("nested-layout fixture graphs");
    ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
        .expect("nested-layout fixture instantiates")
}

fn image_fixture_artboard() -> ArtboardInstance {
    image_fixture().2
}

fn image_fixture() -> (RuntimeFile, GraphFile, ArtboardInstance) {
    let file = RuntimeFile::from_fixture_records(vec![
        fixture_record("Backboard", vec![]),
        fixture_record("ImageAsset", vec![]),
        fixture_record("Artboard", vec![]),
        fixture_record(
            "Image",
            vec![
                fixture_property("Image", "parentId", FixtureValue::Uint(0)),
                fixture_property("Image", "assetId", FixtureValue::Uint(0)),
            ],
        ),
    ])
    .expect("image fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("image fixture graph builds");
    let artboard =
        ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
            .expect("image fixture instantiates");
    (file, graphs, artboard)
}

fn png_bytes(width: u32, height: u32) -> Arc<[u8]> {
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("PNG header encodes");
        writer
            .write_image_data(&vec![0; (width * height * 4) as usize])
            .expect("PNG pixels encode");
    }
    encoded.into()
}

fn component_list_fixture_artboard() -> ArtboardInstance {
    let file = RuntimeFile::from_fixture_records(vec![
        fixture_record("Backboard", vec![]),
        fixture_record("Artboard", vec![]),
        fixture_record(
            "ArtboardComponentList",
            vec![fixture_property(
                "ArtboardComponentList",
                "parentId",
                FixtureValue::Uint(0),
            )],
        ),
    ])
    .expect("component-list fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("component-list fixture graphs");
    ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
        .expect("component-list fixture instantiates")
}

fn nested_component_list_fixture_artboard() -> ArtboardInstance {
    let file = RuntimeFile::from_fixture_records(vec![
        fixture_record("Backboard", vec![]),
        fixture_record("Artboard", vec![]),
        fixture_record(
            "NestedArtboard",
            vec![
                fixture_property("NestedArtboard", "parentId", FixtureValue::Uint(0)),
                fixture_property("NestedArtboard", "artboardId", FixtureValue::Uint(1)),
            ],
        ),
        fixture_record("Artboard", vec![]),
        fixture_record(
            "ArtboardComponentList",
            vec![fixture_property(
                "ArtboardComponentList",
                "parentId",
                FixtureValue::Uint(0),
            )],
        ),
    ])
    .expect("nested component-list fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("nested component-list fixture graphs");
    ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
        .expect("nested component-list fixture instantiates")
}

fn draw_order_fixture_artboard() -> ArtboardInstance {
    let file = RuntimeFile::from_fixture_records(vec![
        fixture_record("Backboard", vec![]),
        fixture_record("Artboard", vec![]),
        fixture_record(
            "Shape",
            vec![fixture_property("Shape", "parentId", FixtureValue::Uint(0))],
        ),
        fixture_record(
            "DrawTarget",
            vec![
                fixture_property("DrawTarget", "parentId", FixtureValue::Uint(0)),
                fixture_property("DrawTarget", "drawableId", FixtureValue::Uint(1)),
            ],
        ),
        fixture_record(
            "DrawRules",
            vec![
                fixture_property("DrawRules", "parentId", FixtureValue::Uint(0)),
                fixture_property("DrawRules", "drawTargetId", FixtureValue::Uint(2)),
            ],
        ),
    ])
    .expect("draw-order fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("draw-order fixture graphs");
    ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
        .expect("draw-order fixture instantiates")
}

fn path_fixture_artboard() -> ArtboardInstance {
    let file = RuntimeFile::from_fixture_records(vec![
        fixture_record("Backboard", vec![]),
        fixture_record("Artboard", vec![]),
        fixture_record(
            "Shape",
            vec![fixture_property("Shape", "parentId", FixtureValue::Uint(0))],
        ),
        fixture_record(
            "Rectangle",
            vec![
                fixture_property("Rectangle", "parentId", FixtureValue::Uint(1)),
                fixture_property("Rectangle", "width", FixtureValue::Double(10.0)),
                fixture_property("Rectangle", "height", FixtureValue::Double(10.0)),
            ],
        ),
    ])
    .expect("path fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("path fixture graphs");
    ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
        .expect("path fixture instantiates")
}

fn solid_color_fixture_artboard() -> ArtboardInstance {
    let file = RuntimeFile::from_fixture_records(vec![
        fixture_record("Backboard", vec![]),
        fixture_record("Artboard", vec![]),
        fixture_record(
            "Shape",
            vec![fixture_property("Shape", "parentId", FixtureValue::Uint(0))],
        ),
        fixture_record(
            "Fill",
            vec![fixture_property("Fill", "parentId", FixtureValue::Uint(1))],
        ),
        fixture_record(
            "SolidColor",
            vec![
                fixture_property("SolidColor", "parentId", FixtureValue::Uint(2)),
                fixture_property("SolidColor", "colorValue", FixtureValue::Color(0xff33_66aa)),
            ],
        ),
        fixture_record(
            "Rectangle",
            vec![
                fixture_property("Rectangle", "parentId", FixtureValue::Uint(1)),
                fixture_property("Rectangle", "width", FixtureValue::Double(10.0)),
                fixture_property("Rectangle", "height", FixtureValue::Double(10.0)),
            ],
        ),
    ])
    .expect("solid-color fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("solid-color fixture graph builds");
    ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
        .expect("solid-color fixture instantiates")
}

fn stroke_fixture_artboard() -> ArtboardInstance {
    let file = RuntimeFile::from_fixture_records(vec![
        fixture_record("Backboard", vec![]),
        fixture_record("Artboard", vec![]),
        fixture_record(
            "Shape",
            vec![fixture_property("Shape", "parentId", FixtureValue::Uint(0))],
        ),
        fixture_record(
            "Stroke",
            vec![
                fixture_property("Stroke", "parentId", FixtureValue::Uint(1)),
                fixture_property("Stroke", "thickness", FixtureValue::Double(2.0)),
            ],
        ),
        fixture_record(
            "SolidColor",
            vec![
                fixture_property("SolidColor", "parentId", FixtureValue::Uint(2)),
                fixture_property("SolidColor", "colorValue", FixtureValue::Color(0xff33_66aa)),
            ],
        ),
        fixture_record(
            "Rectangle",
            vec![
                fixture_property("Rectangle", "parentId", FixtureValue::Uint(1)),
                fixture_property("Rectangle", "width", FixtureValue::Double(10.0)),
                fixture_property("Rectangle", "height", FixtureValue::Double(10.0)),
            ],
        ),
    ])
    .expect("stroke fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("stroke fixture graph builds");
    ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
        .expect("stroke fixture instantiates")
}

#[derive(Clone, Copy, Debug)]
enum ShapeExclusionGate {
    DrawableHidden,
    Collapsed,
    ZeroRenderOpacity,
    NoVisiblePath,
}

#[derive(Clone, Copy, Debug)]
enum PaintMembershipMutation {
    FillVisibility,
    StrokeThickness,
}

fn assert_paint_membership_is_stable_under_shape_gate(
    gate: ShapeExclusionGate,
    mutation: PaintMembershipMutation,
) {
    let mut artboard = match mutation {
        PaintMembershipMutation::FillVisibility => solid_color_fixture_artboard(),
        PaintMembershipMutation::StrokeThickness => stroke_fixture_artboard(),
    };
    artboard.update_pass();
    let shape_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .expect("fixture has a Shape");
    match gate {
        ShapeExclusionGate::DrawableHidden => {
            let drawable_flags =
                fixture_property("Drawable", "drawableFlags", FixtureValue::Uint(0)).key;
            assert!(artboard.set_uint_property(shape_local, drawable_flags, 1));
        }
        ShapeExclusionGate::Collapsed => {
            assert!(artboard.collapse_component(shape_local, true));
        }
        ShapeExclusionGate::ZeroRenderOpacity => {
            assert!(artboard.set_transform_property(shape_local, TransformProperty::Opacity, 0.0,));
            artboard.update_pass();
        }
        ShapeExclusionGate::NoVisiblePath => {
            let rectangle_local = artboard
                .components()
                .iter()
                .find(|component| component.type_name == "Rectangle")
                .map(|component| component.local_id)
                .expect("fixture has a Rectangle");
            let path_flags = fixture_property("Path", "pathFlags", FixtureValue::Uint(0)).key;
            assert!(artboard.set_uint_property(rectangle_local, path_flags, 1));
        }
    }
    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "the {gate:?} Shape is already absent from the public catalogue",
    );
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    match mutation {
        PaintMembershipMutation::FillVisibility => {
            let fill_local = artboard
                .components()
                .iter()
                .find(|component| component.type_name == "Fill")
                .map(|component| component.local_id)
                .expect("fixture has a Fill");
            let is_visible =
                fixture_property("ShapePaint", "isVisible", FixtureValue::Bool(true)).key;
            assert!(artboard.set_bool_property(fill_local, is_visible, false));
        }
        PaintMembershipMutation::StrokeThickness => {
            let stroke_local = artboard
                .components()
                .iter()
                .find(|component| component.type_name == "Stroke")
                .map(|component| component.local_id)
                .expect("fixture has a Stroke");
            let thickness = fixture_property("Stroke", "thickness", FixtureValue::Double(0.0)).key;
            assert!(artboard.set_double_property(stroke_local, thickness, 0.0));
        }
    }

    assert!(artboard.visible_geometry_with_bounds().is_empty());
    assert_eq!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "{mutation:?} below a {gate:?} Shape cannot change visible catalogue membership",
    );
}

fn gradient_fixture_artboard() -> ArtboardInstance {
    gradient_fixture().2
}

fn gradient_fixture() -> (RuntimeFile, GraphFile, ArtboardInstance) {
    let file = RuntimeFile::from_fixture_records(vec![
        fixture_record("Backboard", vec![]),
        fixture_record("Artboard", vec![]),
        fixture_record(
            "Shape",
            vec![fixture_property("Shape", "parentId", FixtureValue::Uint(0))],
        ),
        fixture_record(
            "Fill",
            vec![fixture_property("Fill", "parentId", FixtureValue::Uint(1))],
        ),
        fixture_record(
            "LinearGradient",
            vec![
                fixture_property("LinearGradient", "parentId", FixtureValue::Uint(2)),
                fixture_property("LinearGradient", "endX", FixtureValue::Double(10.0)),
            ],
        ),
        fixture_record(
            "GradientStop",
            vec![
                fixture_property("GradientStop", "parentId", FixtureValue::Uint(3)),
                fixture_property(
                    "GradientStop",
                    "colorValue",
                    FixtureValue::Color(0xffff_0000),
                ),
                fixture_property("GradientStop", "position", FixtureValue::Double(0.0)),
            ],
        ),
        fixture_record(
            "GradientStop",
            vec![
                fixture_property("GradientStop", "parentId", FixtureValue::Uint(3)),
                fixture_property(
                    "GradientStop",
                    "colorValue",
                    FixtureValue::Color(0xff00_00ff),
                ),
                fixture_property("GradientStop", "position", FixtureValue::Double(1.0)),
            ],
        ),
        fixture_record(
            "Rectangle",
            vec![
                fixture_property("Rectangle", "parentId", FixtureValue::Uint(1)),
                fixture_property("Rectangle", "width", FixtureValue::Double(10.0)),
                fixture_property("Rectangle", "height", FixtureValue::Double(10.0)),
            ],
        ),
    ])
    .expect("gradient fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("gradient fixture graph builds");
    let artboard =
        ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
            .expect("gradient fixture instantiates");
    (file, graphs, artboard)
}

#[cfg(feature = "tools")]
#[test]
fn renderer_preparation_skips_shape_paint_command_compatibility_reports() {
    let (file, graphs, mut artboard) = gradient_fixture();
    artboard.update_pass();
    let graph = &graphs.artboards[0];

    reset_runtime_shape_paint_command_report_count();
    let commands = artboard.draw_commands(graph);
    assert!(
        commands
            .iter()
            .any(|command| !command.shape_paints.is_empty()),
        "the public compatibility report must retain ShapePaint commands",
    );
    assert!(
        runtime_shape_paint_command_report_count() > 0,
        "the diagnostic must observe public compatibility report construction",
    );

    reset_runtime_shape_paint_command_report_count();
    let mut factory = NullFactory::new();
    artboard
        .synchronize_artboard_renderer(
            &file,
            graph,
            &graphs.artboards,
            &BTreeMap::new(),
            &mut factory,
            None,
        )
        .expect("renderer preparation succeeds");
    assert_eq!(
        runtime_shape_paint_command_report_count(),
        0,
        "backend preparation reads retained ShapePaint owners directly and must not build the public compatibility report",
    );
}

fn variable_font_fixture_artboard() -> ArtboardInstance {
    fixture_artboard_from_bytes(include_bytes!(
        "../../../fixtures/fl-e8/text_variation_modifier.riv"
    ))
}

#[test]
fn semantic_geometry_revision_changes_for_nested_artboard_geometry() {
    let mut artboard = nested_fixture_artboard();
    artboard.update_pass();
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");
    let mut changed = false;

    artboard
        .try_visit_artboard_tree_instances_mut(&mut |_, _, child| {
            if changed {
                return Ok::<_, ()>(());
            }
            let Some((local_id, x)) = child.components().iter().find_map(|component| {
                component.capabilities.transform.then(|| {
                    child
                        .transform_property(component.local_id, TransformProperty::X)
                        .map(|x| (component.local_id, x))
                })?
            }) else {
                return Ok(());
            };
            changed = child.set_transform_property(local_id, TransformProperty::X, x + 1.0);
            child.update_pass();
            Ok(())
        })
        .expect("nested visitor succeeds");

    assert!(changed, "fixture has a mutable nested artboard occurrence");
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before
    );
}

#[test]
fn semantic_geometry_revision_changes_when_nested_host_opacity_hides_visible_geometry() {
    let mut artboard = nested_layout_fixture_artboard();
    artboard.update_pass();
    let host_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "NestedArtboardLayout")
        .map(|component| component.local_id)
        .expect("fixture has a nested host");
    assert_eq!(
        artboard.visible_geometry_with_bounds().len(),
        1,
        "the mounted nested child contributes visible geometry",
    );
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_transform_property(host_local, TransformProperty::Opacity, 0.0));
    artboard.update_components_with_hook(|_, _, _| {});
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "the nested-host write must publish before a later semantic read",
    );

    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "an effective-zero nested host removes the child occurrence",
    );
}

#[test]
fn semantic_geometry_revision_changes_for_generic_nested_host_opacity() {
    let mut artboard = nested_layout_fixture_artboard();
    artboard.update_pass();
    let host_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "NestedArtboardLayout")
        .map(|component| component.local_id)
        .expect("fixture has a nested host");
    assert_eq!(artboard.visible_geometry_with_bounds().len(), 1);
    let opacity = fixture_property("Node", "opacity", FixtureValue::Double(1.0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_double_property(host_local, opacity, 0.0));
    artboard.update_components_with_hook(|_, _, _| {});

    assert!(artboard.visible_geometry_with_bounds().is_empty());
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "generic nested-host opacity must publish from derived settlement",
    );
}

#[test]
fn semantic_geometry_revision_changes_for_inherited_nested_host_opacity() {
    let mut artboard = nested_layout_fixture_artboard_with_parent(true);
    artboard.update_pass();
    let parent_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Node")
        .map(|component| component.local_id)
        .expect("fixture has a host parent");
    assert_eq!(artboard.visible_geometry_with_bounds().len(), 1);
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_transform_property(parent_local, TransformProperty::Opacity, 0.0));
    artboard.update_components_with_hook(|_, _, _| {});

    assert!(artboard.visible_geometry_with_bounds().is_empty());
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "inherited nested-host opacity must publish from derived settlement",
    );
}

#[test]
fn semantic_geometry_revision_changes_when_nested_host_drawable_flags_hide_visible_geometry() {
    let mut artboard = nested_layout_fixture_artboard();
    artboard.update_pass();
    let host_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "NestedArtboardLayout")
        .map(|component| component.local_id)
        .expect("fixture has a nested host");
    assert_eq!(
        artboard.visible_geometry_with_bounds().len(),
        1,
        "the visible nested host contributes its mounted child geometry",
    );
    let drawable_flags = fixture_property("Drawable", "drawableFlags", FixtureValue::Uint(0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_uint_property(host_local, drawable_flags, 1));
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "the nested-host hidden-bit write must publish before a later semantic read",
    );
    artboard.update_pass();

    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "the hidden nested host removes the mounted child occurrence",
    );
}

#[test]
fn semantic_geometry_revision_changes_for_nested_artboard_topology() {
    let mut artboard = nested_fixture_artboard();
    artboard.update_pass();
    let host_local_id = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "NestedArtboard")
        .map(|component| component.local_id)
        .expect("fixture has a nested host");
    let artboard_id = fixture_property("NestedArtboard", "artboardId", FixtureValue::Uint(0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_uint_property(host_local_id, artboard_id, u64::from(u32::MAX)));

    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before
    );
}

#[test]
fn semantic_geometry_revision_changes_when_image_dimensions_are_registered_late() {
    let mut artboard = image_fixture_artboard();
    artboard.update_pass();
    let image_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Image")
        .map(|component| component.local_id)
        .expect("fixture has an Image");
    assert!(
        artboard.object_world_bounds(image_local).is_none(),
        "an unresolved image has no authoritative world bounds",
    );
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    artboard
        .register_image_dimensions(1, 100, 50)
        .expect("late dimensions register");

    let bounds = artboard
        .object_world_bounds(image_local)
        .expect("registered dimensions publish image bounds");
    assert_eq!(bounds.max_x - bounds.min_x, 100.0);
    assert_eq!(bounds.max_y - bounds.min_y, 50.0);
    let after = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");
    assert_ne!(
        after, before,
        "late intrinsic dimensions must invalidate retained semantic geometry",
    );

    artboard
        .register_image_dimensions(1, 100, 50)
        .expect("identical dimensions remain valid");
    assert_eq!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        after,
        "re-registering identical dimensions must keep the authority stable",
    );
}

#[test]
fn semantic_geometry_revision_changes_immediately_for_image_origin_mutation() {
    for (property_name, expected_min_x, expected_min_y) in
        [("originX", -25.0, -25.0), ("originY", -50.0, -12.5)]
    {
        let mut artboard = image_fixture_artboard();
        artboard.update_pass();
        let image_local = artboard
            .components()
            .iter()
            .find(|component| component.type_name == "Image")
            .map(|component| component.local_id)
            .expect("fixture has an Image");
        artboard
            .register_image_dimensions(1, 100, 50)
            .expect("image dimensions register");
        let before_bounds = artboard
            .object_world_bounds(image_local)
            .expect("registered dimensions publish image bounds");
        assert_eq!((before_bounds.min_x, before_bounds.min_y), (-50.0, -25.0));
        let before = artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry");
        let origin = fixture_property("Image", property_name, FixtureValue::Double(0.5)).key;

        assert!(artboard.set_double_property(image_local, origin, 0.25));

        let after_bounds = artboard
            .object_world_bounds(image_local)
            .expect("mutated Image origin retains authoritative bounds");
        assert_eq!(
            (after_bounds.min_x, after_bounds.min_y),
            (expected_min_x, expected_min_y),
            "Image.{property_name} moves the public object bounds immediately",
        );
        assert_ne!(
            artboard
                .try_semantic_geometry_revision()
                .expect("fixture has covered semantic geometry"),
            before,
            "Image.{property_name} bounds changes must invalidate retained semantic geometry",
        );
    }
}

#[test]
fn semantic_geometry_revision_is_stable_for_identical_image_origin_write() {
    let mut artboard = image_fixture_artboard();
    artboard.update_pass();
    let image_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Image")
        .map(|component| component.local_id)
        .expect("fixture has an Image");
    artboard
        .register_image_dimensions(1, 100, 50)
        .expect("image dimensions register");
    let before_bounds = artboard
        .object_world_bounds(image_local)
        .expect("registered dimensions publish image bounds");
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    for property_name in ["originX", "originY"] {
        let origin = fixture_property("Image", property_name, FixtureValue::Double(0.5)).key;
        assert!(
            !artboard.set_double_property(image_local, origin, 0.5),
            "an identical Image.{property_name} write is not a mutation",
        );
    }

    assert_eq!(
        artboard
            .object_world_bounds(image_local)
            .expect("identical writes retain authoritative bounds"),
        before_bounds,
    );
    assert_eq!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "identical Image origin writes must keep the authority stable",
    );
}

#[test]
fn semantic_geometry_revision_changes_when_owned_images_are_observed() {
    let (file, graphs, mut artboard) = image_fixture();
    artboard.update_pass();
    let image_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Image")
        .map(|component| component.local_id)
        .expect("fixture has an Image");
    assert!(
        artboard.object_world_bounds(image_local).is_none(),
        "an unobserved renderer-owned image has no authoritative world bounds",
    );
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");
    let mut factory = NullFactory::new();
    artboard
        .initialize_artboard_renderer(
            &file,
            &graphs.artboards[0],
            &graphs.artboards,
            &BTreeMap::from([(0, png_bytes(100, 50))]),
            &mut factory,
            None,
        )
        .expect("renderer-owned image initializes");

    artboard
        .observe_owned_images()
        .expect("renderer-owned dimensions are observed");

    let bounds = artboard
        .object_world_bounds(image_local)
        .expect("observed renderer dimensions publish image bounds");
    assert_eq!(bounds.max_x - bounds.min_x, 100.0);
    assert_eq!(bounds.max_y - bounds.min_y, 50.0);
    let after = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");
    assert_ne!(
        after, before,
        "observing renderer-owned dimensions must invalidate retained semantic geometry",
    );
    artboard
        .observe_owned_images()
        .expect("identical renderer-owned dimensions remain valid");
    assert_eq!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        after,
        "observing an unchanged presented-dimension map keeps the authority stable",
    );
}

#[test]
fn semantic_geometry_revision_changes_for_path_geometry_mutation() {
    let mut artboard = path_fixture_artboard();
    artboard.update_pass();
    let rectangle_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Rectangle")
        .map(|component| component.local_id)
        .expect("fixture has a rectangle");
    let width = fixture_property("Rectangle", "width", FixtureValue::Double(0.0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_double_property(rectangle_local, width, 20.0));
    artboard.update_pass();

    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before
    );
}

#[test]
fn semantic_geometry_revision_changes_for_text_font_metrics_mutation() {
    let mut artboard = variable_font_fixture_artboard();
    artboard.update_pass();
    let axis_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "TextStyleAxis")
        .map(|component| component.local_id)
        .expect("fixture has a variable-font axis");
    let axis_value = fixture_property("TextStyleAxis", "axisValue", FixtureValue::Double(0.0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_double_property(axis_local, axis_value, 900.0));

    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before
    );
}

#[test]
fn semantic_geometry_revision_changes_for_world_transform_mutation() {
    let mut artboard = fixture_artboard();
    artboard.update_pass();
    let local_id = artboard
        .components()
        .iter()
        .find(|component| component.capabilities.transform)
        .map(|component| component.local_id)
        .expect("fixture has a transform component");
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");
    let x = artboard
        .transform_property(local_id, TransformProperty::X)
        .expect("transform has x");

    assert!(artboard.set_transform_property(local_id, TransformProperty::X, x + 1.0));
    artboard.update_pass();

    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before
    );
}

#[test]
fn semantic_geometry_revision_is_independent_across_cold_clone_mutation() {
    let mut source = fixture_artboard();
    source.update_pass();
    let mut clone = source.clone();
    clone.update_pass();
    let source_before = source
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");
    assert_ne!(
        clone
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        source_before,
        "a public clone is a distinct artboard occurrence",
    );
    let local_id = clone
        .components()
        .iter()
        .find(|component| component.capabilities.transform)
        .map(|component| component.local_id)
        .expect("fixture has a transform component");
    let x = clone
        .transform_property(local_id, TransformProperty::X)
        .expect("transform has x");

    assert!(clone.set_transform_property(local_id, TransformProperty::X, x + 1.0));
    clone.update_pass();

    assert_eq!(
        source
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        source_before,
        "mutating a cold clone must not invalidate the source occurrence",
    );
}

#[test]
fn semantic_geometry_revision_is_stable_across_repeated_visible_geometry_reads() {
    let mut artboard = nested_layout_fixture_artboard();
    artboard.update_pass();
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    let first = artboard.visible_geometry_with_bounds();
    assert!(!first.is_empty(), "fixture exposes nested visible geometry");
    let after_first = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");
    let second = artboard.visible_geometry_with_bounds();

    assert_eq!(
        second, first,
        "repeated public reads return the same geometry"
    );
    assert_eq!(
        after_first, before,
        "a public visible-geometry read must not invalidate its source occurrence",
    );
    assert_eq!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "repeated public visible-geometry settlement must leave the source token stable",
    );
}

#[test]
fn semantic_geometry_revision_authority_is_available_for_a_covered_artboard() {
    let artboard = fixture_artboard();
    let revision = artboard
        .try_semantic_geometry_revision()
        .expect("covered artboard has an authority");
    assert_eq!(
        format!("{revision:?}"),
        "SemanticGeometryRevision(..)",
        "the compare-only authority must not expose its identity or generation",
    );
    assert!(
        artboard.clone().try_semantic_geometry_revision().is_some(),
        "a covered cold clone recomputes covered authority",
    );
}

#[test]
fn semantic_geometry_revision_authority_fails_closed_for_component_lists() {
    let artboard = component_list_fixture_artboard();
    assert!(artboard.try_semantic_geometry_revision().is_none(),);
    assert!(
        artboard.clone().try_semantic_geometry_revision().is_none(),
        "a component-list cold clone remains conservatively uncovered",
    );
}

#[test]
fn semantic_geometry_revision_authority_fails_closed_for_nested_component_lists() {
    let artboard = nested_component_list_fixture_artboard();
    assert!(artboard.try_semantic_geometry_revision().is_none(),);
    assert!(
        artboard.clone().try_semantic_geometry_revision().is_none(),
        "recursive cold-clone adoption propagates uncovered child ownership",
    );
}

#[test]
fn semantic_geometry_revision_authority_fails_closed_for_draw_order_owners() {
    assert!(
        draw_order_fixture_artboard()
            .try_semantic_geometry_revision()
            .is_none(),
    );
}

#[test]
fn semantic_geometry_revision_is_stable_when_visible_shape_opacity_remains_nonzero() {
    let mut artboard = solid_color_fixture_artboard();
    artboard.update_pass();
    let shape_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .expect("fixture has a Shape");
    let before_catalogue = artboard.visible_geometry_with_bounds();
    assert_eq!(before_catalogue.len(), 1, "the painted Shape is visible");
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_transform_property(shape_local, TransformProperty::Opacity, 0.5,));
    artboard.update_pass();

    assert_eq!(
        artboard.visible_geometry_with_bounds(),
        before_catalogue,
        "a nonzero opacity change preserves visible catalogue membership and bounds",
    );
    assert_eq!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "a nonzero opacity change must not invalidate settled semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_changes_when_shape_opacity_quantizes_paint_alpha_to_zero() {
    let mut artboard = solid_color_fixture_artboard();
    artboard.update_pass();
    let shape_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .expect("fixture has a Shape");
    assert_eq!(
        artboard.visible_geometry_with_bounds().len(),
        1,
        "the painted Shape is visible",
    );
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_transform_property(shape_local, TransformProperty::Opacity, 0.001,));
    artboard.update_pass();

    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "a tiny positive opacity quantizes the retained SolidColor alpha to zero",
    );
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "quantized Shape catalogue membership changes must invalidate semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_changes_when_shape_render_opacity_hides_visible_geometry() {
    let mut artboard = solid_color_fixture_artboard();
    artboard.update_pass();
    let shape_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .expect("fixture has a Shape");
    assert_eq!(
        artboard.visible_geometry_with_bounds().len(),
        1,
        "the painted Shape is visible",
    );
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_transform_property(shape_local, TransformProperty::Opacity, 0.0,));
    artboard.update_pass();

    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "effective-zero Shape opacity removes it from the visible catalogue",
    );
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "a visible-catalogue membership transition must invalidate semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_is_stable_when_visible_text_opacity_remains_nonzero() {
    let mut artboard = variable_font_fixture_artboard();
    artboard.update_pass();
    let text_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Text")
        .map(|component| component.local_id)
        .expect("fixture has Text");
    let before_hits = artboard
        .visible_geometry_with_bounds()
        .into_iter()
        .filter(|hit| {
            hit.path
                .last()
                .is_some_and(|segment| segment.local_id == text_local)
        })
        .collect::<Vec<_>>();
    assert!(!before_hits.is_empty(), "fixture exposes a public Text hit");
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_transform_property(text_local, TransformProperty::Opacity, 0.5,));
    artboard.update_pass();

    let after_hits = artboard
        .visible_geometry_with_bounds()
        .into_iter()
        .filter(|hit| {
            hit.path
                .last()
                .is_some_and(|segment| segment.local_id == text_local)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        after_hits, before_hits,
        "a nonzero opacity change preserves Text catalogue membership and bounds",
    );
    assert_eq!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "a nonzero Text opacity change must not invalidate settled semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_changes_when_text_opacity_becomes_non_visible() {
    for hidden_opacity in [-1.0, f32::NAN] {
        let mut artboard = variable_font_fixture_artboard();
        artboard.update_pass();
        let text_local = artboard
            .components()
            .iter()
            .find(|component| component.type_name == "Text")
            .map(|component| component.local_id)
            .expect("fixture has Text");
        assert!(
            artboard.visible_geometry_with_bounds().iter().any(|hit| {
                hit.path
                    .last()
                    .is_some_and(|segment| segment.local_id == text_local)
            }),
            "fixture exposes a public Text hit",
        );
        let before = artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry");

        assert!(artboard.set_transform_property(
            text_local,
            TransformProperty::Opacity,
            hidden_opacity,
        ));
        artboard.update_pass();

        assert!(
            artboard.visible_geometry_with_bounds().iter().all(|hit| {
                hit.path
                    .last()
                    .is_none_or(|segment| segment.local_id != text_local)
            }),
            "negative and non-finite Text opacity remove the public occurrence",
        );
        assert_ne!(
            artboard
                .try_semantic_geometry_revision()
                .expect("fixture has covered semantic geometry"),
            before,
            "Text catalogue membership changes must invalidate semantic geometry",
        );
    }
}

#[test]
fn semantic_geometry_revision_changes_when_text_render_opacity_hides_visible_geometry() {
    let mut artboard = variable_font_fixture_artboard();
    artboard.update_pass();
    let text_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Text")
        .map(|component| component.local_id)
        .expect("fixture has Text");
    assert!(
        artboard.visible_geometry_with_bounds().iter().any(|hit| {
            hit.path
                .last()
                .is_some_and(|segment| segment.local_id == text_local)
        }),
        "fixture exposes a public Text hit",
    );
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_transform_property(text_local, TransformProperty::Opacity, 0.0,));
    artboard.update_pass();

    assert!(
        artboard.visible_geometry_with_bounds().iter().all(|hit| {
            hit.path
                .last()
                .is_none_or(|segment| segment.local_id != text_local)
        }),
        "effective-zero Text opacity removes its public catalogue occurrence",
    );
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "a Text catalogue membership transition must invalidate semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_changes_when_text_drawable_flags_hide_visible_geometry() {
    let mut artboard = variable_font_fixture_artboard();
    artboard.update_pass();
    let text_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Text")
        .map(|component| component.local_id)
        .expect("fixture has Text");
    assert!(
        artboard.visible_geometry_with_bounds().iter().any(|hit| {
            hit.path
                .last()
                .is_some_and(|segment| segment.local_id == text_local)
        }),
        "fixture exposes a public Text hit",
    );
    let drawable_flags = fixture_property("Drawable", "drawableFlags", FixtureValue::Uint(0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_uint_property(text_local, drawable_flags, 1));
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "the Text hidden-bit write must publish before a later semantic read",
    );
    artboard.update_pass();

    assert!(
        artboard.visible_geometry_with_bounds().iter().all(|hit| {
            hit.path
                .last()
                .is_none_or(|segment| segment.local_id != text_local)
        }),
        "the hidden Text removes its public catalogue occurrence",
    );
}

#[test]
fn semantic_geometry_revision_changes_when_solid_color_hides_visible_geometry() {
    let mut artboard = solid_color_fixture_artboard();
    artboard.update_pass();
    assert_eq!(
        artboard.visible_geometry_with_bounds().len(),
        1,
        "the opaque Shape is in the no-point visible catalogue",
    );
    let solid_color_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "SolidColor")
        .map(|component| component.local_id)
        .expect("fixture has a SolidColor");
    let color_value = fixture_property("SolidColor", "colorValue", FixtureValue::Color(0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_color_property(solid_color_local, color_value, 0x0033_66aa));
    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "a transparent SolidColor removes its Shape from the no-point visible catalogue",
    );

    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "visible-catalogue membership changes must invalidate retained semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_changes_when_shape_paint_hides_visible_geometry() {
    let mut artboard = solid_color_fixture_artboard();
    artboard.update_pass();
    assert_eq!(
        artboard.visible_geometry_with_bounds().len(),
        1,
        "the visible Fill includes its Shape in the no-point visible catalogue",
    );
    let fill_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Fill")
        .map(|component| component.local_id)
        .expect("fixture has a Fill");
    let is_visible = fixture_property("ShapePaint", "isVisible", FixtureValue::Bool(true)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_bool_property(fill_local, is_visible, false));
    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "an invisible Fill removes its Shape from the no-point visible catalogue",
    );
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "ShapePaint visibility changes must invalidate retained semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_is_stable_for_paint_membership_under_hidden_shape() {
    assert_paint_membership_is_stable_under_shape_gate(
        ShapeExclusionGate::DrawableHidden,
        PaintMembershipMutation::FillVisibility,
    );
}

#[test]
fn semantic_geometry_revision_is_stable_for_paint_membership_under_collapsed_shape() {
    assert_paint_membership_is_stable_under_shape_gate(
        ShapeExclusionGate::Collapsed,
        PaintMembershipMutation::FillVisibility,
    );
}

#[test]
fn semantic_geometry_revision_is_stable_for_paint_membership_under_zero_opacity_shape() {
    assert_paint_membership_is_stable_under_shape_gate(
        ShapeExclusionGate::ZeroRenderOpacity,
        PaintMembershipMutation::FillVisibility,
    );
}

#[test]
fn semantic_geometry_revision_is_stable_for_paint_membership_without_visible_shape_path() {
    assert_paint_membership_is_stable_under_shape_gate(
        ShapeExclusionGate::NoVisiblePath,
        PaintMembershipMutation::FillVisibility,
    );
}

#[test]
fn semantic_geometry_revision_changes_when_stroke_thickness_hides_visible_geometry() {
    let mut artboard = stroke_fixture_artboard();
    artboard.update_pass();
    assert_eq!(
        artboard.visible_geometry_with_bounds().len(),
        1,
        "a positive-thickness Stroke includes its Shape in the no-point visible catalogue",
    );
    let stroke_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Stroke")
        .map(|component| component.local_id)
        .expect("fixture has a Stroke");
    let thickness = fixture_property("Stroke", "thickness", FixtureValue::Double(0.0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_double_property(stroke_local, thickness, 0.0));
    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "a zero-thickness Stroke removes its Shape from the no-point visible catalogue",
    );
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "Stroke thickness visibility transitions must invalidate retained semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_is_stable_for_stroke_membership_under_hidden_shape() {
    assert_paint_membership_is_stable_under_shape_gate(
        ShapeExclusionGate::DrawableHidden,
        PaintMembershipMutation::StrokeThickness,
    );
}

#[test]
fn semantic_geometry_revision_is_stable_for_stroke_membership_under_collapsed_shape() {
    assert_paint_membership_is_stable_under_shape_gate(
        ShapeExclusionGate::Collapsed,
        PaintMembershipMutation::StrokeThickness,
    );
}

#[test]
fn semantic_geometry_revision_is_stable_for_stroke_membership_under_zero_opacity_shape() {
    assert_paint_membership_is_stable_under_shape_gate(
        ShapeExclusionGate::ZeroRenderOpacity,
        PaintMembershipMutation::StrokeThickness,
    );
}

#[test]
fn semantic_geometry_revision_is_stable_for_stroke_membership_without_visible_shape_path() {
    assert_paint_membership_is_stable_under_shape_gate(
        ShapeExclusionGate::NoVisiblePath,
        PaintMembershipMutation::StrokeThickness,
    );
}

#[test]
fn semantic_geometry_revision_changes_when_drawable_flags_hide_visible_geometry() {
    let mut artboard = solid_color_fixture_artboard();
    artboard.update_pass();
    assert_eq!(
        artboard.visible_geometry_with_bounds().len(),
        1,
        "the visible Drawable includes its Shape in the no-point visible catalogue",
    );
    let shape_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .expect("fixture has a Shape");
    let drawable_flags = fixture_property("Drawable", "drawableFlags", FixtureValue::Uint(0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_uint_property(shape_local, drawable_flags, 1));
    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "Drawable hidden bit removes its Shape from the no-point visible catalogue",
    );
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "Drawable hidden-bit transitions must invalidate retained semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_is_stable_when_hidden_drawable_does_not_change_membership() {
    let mut artboard = solid_color_fixture_artboard();
    artboard.update_pass();
    let shape_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .expect("fixture has a Shape");
    let solid_color_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "SolidColor")
        .map(|component| component.local_id)
        .expect("fixture has a SolidColor");
    let color_value = fixture_property("SolidColor", "colorValue", FixtureValue::Color(0)).key;
    let drawable_flags = fixture_property("Drawable", "drawableFlags", FixtureValue::Uint(0)).key;

    assert!(artboard.set_color_property(solid_color_local, color_value, 0x0033_66aa));
    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "the transparent paint already removes the Shape from the catalogue",
    );
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_uint_property(shape_local, drawable_flags, 1));

    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "hiding the already-absent Shape preserves catalogue membership",
    );
    assert_eq!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "Drawable hidden-bit changes must not invalidate an already-absent Shape",
    );
}

#[test]
fn semantic_geometry_revision_changes_when_path_flags_hide_visible_geometry() {
    let mut artboard = solid_color_fixture_artboard();
    artboard.update_pass();
    assert_eq!(
        artboard.visible_geometry_with_bounds().len(),
        1,
        "the visible Rectangle contributes geometry to the no-point catalogue",
    );
    let rectangle_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Rectangle")
        .map(|component| component.local_id)
        .expect("fixture has a Rectangle");
    let path_flags = fixture_property("Path", "pathFlags", FixtureValue::Uint(0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_uint_property(rectangle_local, path_flags, 1));
    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "Path hidden bit removes the Shape's only geometry from the no-point catalogue",
    );
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "Path hidden-bit transitions must invalidate retained semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_is_stable_when_non_hidden_path_flags_change() {
    let mut artboard = solid_color_fixture_artboard();
    artboard.update_pass();
    let rectangle_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Rectangle")
        .map(|component| component.local_id)
        .expect("fixture has a Rectangle");
    let path_flags = fixture_property("Path", "pathFlags", FixtureValue::Uint(0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_uint_property(rectangle_local, path_flags, 1 << 2));
    assert!(
        !artboard
            .components()
            .iter()
            .find(|component| component.local_id == rectangle_local)
            .expect("Rectangle remains mounted")
            .dirt
            .contains(ComponentDirt::PATH),
        "non-hidden pathFlags must not schedule a path rebuild",
    );
    assert_eq!(artboard.visible_geometry_with_bounds().len(), 1);
    assert_eq!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "non-hidden pathFlags must not invalidate retained semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_changes_when_gradient_stops_hide_visible_geometry() {
    let mut artboard = gradient_fixture_artboard();
    artboard.update_pass();
    assert_eq!(
        artboard.visible_geometry_with_bounds().len(),
        1,
        "the opaque gradient Shape is in the no-point visible catalogue",
    );
    let gradient_stops = artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "GradientStop")
        .map(|component| component.local_id)
        .collect::<Vec<_>>();
    assert_eq!(gradient_stops.len(), 2, "fixture has two GradientStops");
    let color_value = fixture_property("GradientStop", "colorValue", FixtureValue::Color(0)).key;

    let before_first = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");
    assert!(artboard.set_color_property(gradient_stops[0], color_value, 0x00ff_0000));
    assert_eq!(artboard.visible_geometry_with_bounds().len(), 1);
    assert_eq!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before_first,
        "one remaining opaque stop keeps the whole Shape effectively visible",
    );

    let before_last = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");
    assert!(artboard.set_color_property(gradient_stops[1], color_value, 0x0000_00ff));
    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "an all-transparent gradient removes its Shape from the no-point visible catalogue",
    );
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before_last,
        "effective-visible catalogue membership changes must invalidate retained semantic geometry",
    );
}

#[test]
fn semantic_geometry_revision_changes_for_coalesced_shape_and_gradient_visibility() {
    let mut artboard = gradient_fixture_artboard();
    artboard.update_pass();
    assert_eq!(artboard.visible_geometry_with_bounds().len(), 1);
    let shape_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .expect("fixture has a Shape");
    let gradient_stops = artboard
        .components()
        .iter()
        .filter(|component| component.type_name == "GradientStop")
        .map(|component| component.local_id)
        .collect::<Vec<_>>();
    assert_eq!(gradient_stops.len(), 2);
    let color_value = fixture_property("GradientStop", "colorValue", FixtureValue::Color(0)).key;
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.set_transform_property(shape_local, TransformProperty::Opacity, 0.5));
    assert!(artboard.set_color_property(gradient_stops[0], color_value, 0x00ff_0000));
    assert!(artboard.set_color_property(gradient_stops[1], color_value, 0x0000_00ff));
    artboard.update_pass();

    assert!(
        artboard.visible_geometry_with_bounds().is_empty(),
        "the coalesced all-transparent gradient removes its Shape",
    );
    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "the retained gradient transition must publish even when the opacity comparison did not",
    );
}

#[test]
fn semantic_geometry_revision_changes_for_collapsed_visibility() {
    let mut artboard = fixture_artboard();
    artboard.update_pass();
    let local_id = artboard
        .components()
        .iter()
        .find(|component| component.local_id != 0)
        .map(|component| component.local_id)
        .expect("fixture has a collapsible child");
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.collapse_component(local_id, true));

    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before
    );
}

#[test]
fn semantic_geometry_revision_changes_for_layout_mutation() {
    let mut artboard = fixture_artboard();
    artboard.update_pass();
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");
    let (width, height) = artboard.artboard_dimensions();

    assert!(artboard.set_artboard_dimensions(width + 1.0, height));
    artboard.update_pass();

    assert_ne!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before
    );
}

#[test]
fn semantic_geometry_revision_is_stable_across_non_geometry_settlement() {
    let mut artboard = fixture_artboard();
    artboard.update_pass();
    let before = artboard
        .try_semantic_geometry_revision()
        .expect("fixture has covered semantic geometry");

    assert!(artboard.add_dirt(0, ComponentDirt::PAINT, false));
    assert!(artboard.update_pass());

    assert_eq!(
        artboard
            .try_semantic_geometry_revision()
            .expect("fixture has covered semantic geometry"),
        before,
        "a frame may report applied paint work without invalidating settled semantic geometry"
    );
}

#[cfg(feature = "tools")]
#[test]
fn path_composer_reuses_retained_path_storage_across_rebuild() {
    let mut artboard = solid_color_fixture_artboard();
    artboard.update_pass();
    let shape_local = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Shape")
        .map(|component| component.local_id)
        .expect("fixture has a Shape");
    let before = artboard
        .debug_runtime_shape_path_identities(shape_local)
        .expect("fixture retains Shape path owners");
    assert!(before.iter().all(Option::is_some));

    assert!(artboard.set_transform_property(shape_local, TransformProperty::X, 5.0));
    artboard.update_pass();

    assert_eq!(
        artboard
            .debug_runtime_shape_path_identities(shape_local)
            .expect("Shape path owners survive their rebuild"),
        before,
        "pinned C++ PathComposer rewinds and reuses its three ShapePaintPaths"
    );
}
